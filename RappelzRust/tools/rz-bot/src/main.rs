//! rz-bot : client Rappelz sans interface, pour tester RappelzCore de bout en bout.
//!
//! `rz-bot --account test --password test --character Bot1 [--say "bonjour"] [--stay 10]`
//! Parcours : TS_CA_VERSION → RSA/AES → TS_CA_ACCOUNT → liste et choix du royaume →
//! worldserver → liste/création du personnage → TS_CS_LOGIN → déplacement → chat → sortie.

use std::time::Duration;

use anyhow::{bail, Context, Result};
use clap::Parser;
use rz_proto::conn::{self, PacketReader};
use rz_proto::msg::{self, id, EnterKind, Message, MovePoint};
use rz_proto::secure::ClientKeyPair;
use tokio::net::TcpStream;

#[derive(Parser)]
struct Cli {
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    #[arg(long, default_value_t = 4615)]
    port: u16,
    #[arg(long)]
    account: String,
    #[arg(long)]
    password: String,
    #[arg(long)]
    character: String,
    /// Race Rappelz : 3 Gaia, 4 Deva, 5 Asura.
    #[arg(long, default_value_t = 4)]
    race: i32,
    #[arg(long)]
    say: Option<String>,
    /// Secondes passées dans le monde (réception des ENTER/MOVE/CHAT des autres).
    #[arg(long, default_value_t = 3)]
    stay: u64,
    /// Commandes de chat envoyées après l'entrée en jeu (ex. « .additem 244201 »).
    #[arg(long = "cmd")]
    cmds: Vec<String>,
    /// Attaque le premier monstre visible.
    #[arg(long)]
    attack: bool,
    /// Équipe le premier objet du sac à l'emplacement donné.
    #[arg(long)]
    equip: Option<u8>,
    /// Parle au PNJ le plus proche et ouvre sa boutique.
    #[arg(long)]
    talk: bool,
}

async fn expect(r: &mut PacketReader, wanted: u16) -> Result<rz_proto::RawPacket> {
    let fut = async {
        loop {
            let p = r.recv().await?.context("connexion fermée")?;
            if p.id == wanted {
                return Ok::<_, anyhow::Error>(p);
            }
            tracing::debug!("paquet {} ignoré en attendant {wanted}", p.id);
        }
    };
    tokio::time::timeout(Duration::from_secs(10), fut)
        .await
        .with_context(|| format!("délai dépassé en attendant {wanted}"))?
}

#[tokio::main]
async fn main() -> Result<()> {
    rz_common::init_logging("info");
    let cli = Cli::parse();
    let key = rz_proto::cipher::DEFAULT_KEY;

    // ── Authserver
    let (mut r, mut w) = conn::split(
        TcpStream::connect((cli.host.as_str(), cli.port)).await?,
        key,
    );
    w.send(&msg::CaVersion(msg::Version {
        version: msg::CLIENT_VERSION.into(),
    }))
    .await?;
    let keys = ClientKeyPair::generate()?;
    w.send(&msg::CaRsaPublicKey {
        pem: keys.public_pem(),
    })
    .await?;
    let aes = keys
        .open(&msg::AcAesKeyIv::from_packet(&expect(&mut r, id::TS_AC_AES_KEY_IV).await?)?.data)?;
    let mut pwd = cli.password.clone().into_bytes();
    pwd.push(0);
    w.send(&msg::CaAccount {
        account: cli.account.clone(),
        password: aes.encrypt(&pwd),
    })
    .await?;
    let res = msg::AcResult::from_packet(&expect(&mut r, id::TS_AC_RESULT).await?)?;
    if res.result != 0 {
        bail!("authentification refusée (code {})", res.result);
    }
    tracing::info!("[auth] compte {} accepté", cli.account);
    w.send(&msg::CaServerList).await?;
    let list = msg::AcServerList::from_packet(&expect(&mut r, id::TS_AC_SERVER_LIST).await?)?;
    let server = list.servers.first().context("aucun royaume")?.clone();
    tracing::info!(
        "[auth] {} royaume(s) ; choix de « {} » ({}:{})",
        list.servers.len(),
        server.name,
        server.ip,
        server.port
    );
    w.send(&msg::CaSelectServer {
        server_idx: server.index,
    })
    .await?;
    let sel = msg::AcSelectServer::from_packet(&expect(&mut r, id::TS_AC_SELECT_SERVER).await?)?;
    let key_bytes = aes.decrypt(&sel.encrypted_key)?;
    let otk = u64::from_le_bytes(key_bytes[..8].try_into()?);
    tracing::info!("[auth] clé one-time reçue");
    drop((r, w));

    // ── Worldserver : lobby
    let (mut r, mut w) = conn::split(
        TcpStream::connect((server.ip.as_str(), server.port as u16)).await?,
        key,
    );
    w.send(&msg::CsVersion(msg::Version {
        version: msg::CLIENT_VERSION.into(),
    }))
    .await?;
    w.send(&msg::CsAccountWithAuth {
        account: cli.account.clone(),
        one_time_key: otk,
    })
    .await?;
    let res = msg::ScResult::from_packet(&expect(&mut r, id::TS_SC_RESULT).await?)?;
    if res.result != 0 {
        bail!("worldserver : session refusée (code {})", res.result);
    }
    w.send(&msg::CsCharacterList {
        account: cli.account.clone(),
    })
    .await?;
    let mut chars =
        msg::ScCharacterList::from_packet(&expect(&mut r, id::TS_SC_CHARACTER_LIST).await?)?
            .characters;
    tracing::info!(
        "[lobby] {} personnage(s) : {:?}",
        chars.len(),
        chars.iter().map(|c| &c.name).collect::<Vec<_>>()
    );
    if !chars.iter().any(|c| c.name == cli.character) {
        w.send(&msg::CsCheckCharacterName {
            name: cli.character.clone(),
        })
        .await?;
        let chk = msg::ScResult::from_packet(&expect(&mut r, id::TS_SC_RESULT).await?)?;
        tracing::info!("[lobby] vérification du nom : code {}", chk.result);
        let info = msg::LobbyCharacterInfo {
            sex: 2,
            race: cli.race,
            level: 1,
            job: 0,
            name: cli.character.clone(),
            ..Default::default()
        };
        w.send(&msg::CsCreateCharacter { info }).await?;
        let created = msg::ScResult::from_packet(&expect(&mut r, id::TS_SC_RESULT).await?)?;
        if created.result != 0 {
            bail!("création refusée (code {})", created.result);
        }
        w.send(&msg::CsCharacterList {
            account: cli.account.clone(),
        })
        .await?;
        chars =
            msg::ScCharacterList::from_packet(&expect(&mut r, id::TS_SC_CHARACTER_LIST).await?)?
                .characters;
        tracing::info!(
            "[lobby] personnage {} créé ; liste : {:?}",
            cli.character,
            chars.iter().map(|c| &c.name).collect::<Vec<_>>()
        );
    }

    // ── Monde
    w.send(&msg::CsLogin {
        name: cli.character.clone(),
    })
    .await?;
    let lr = msg::ScLoginResult::from_packet(&expect(&mut r, id::TS_SC_LOGIN_RESULT).await?)?;
    if lr.result != 0 {
        bail!("entrée en jeu refusée (code {})", lr.result);
    }
    tracing::info!(
        "[monde] {} en jeu : handle {}, position ({:.0}, {:.0}), race {}",
        lr.name,
        lr.handle,
        lr.x,
        lr.y,
        lr.race
    );
    let target = MovePoint {
        x: lr.x + 40.0,
        y: lr.y + 30.0,
    };
    w.send(&msg::CsMoveRequest {
        handle: lr.handle,
        x: lr.x,
        y: lr.y,
        cur_time: 0,
        speed_sync: false,
        points: vec![target],
    })
    .await?;
    for c in &cli.cmds {
        w.send(&msg::CsChatRequest {
            target: String::new(),
            request_id: 0,
            chat_type: 0,
            message: c.clone(),
        })
        .await?;
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    if let Some(say) = &cli.say {
        w.send(&msg::CsChatRequest {
            target: String::new(),
            request_id: 0,
            chat_type: msg::chat_type::NORMAL,
            message: say.clone(),
        })
        .await?;
    }
    w.send(&msg::CsChatRequest {
        target: String::new(),
        request_id: 0,
        chat_type: 0,
        message: ".gps".into(),
    })
    .await?;

    use rz_proto::msg_game::{self as game, id as gid};
    let (mut npcs, mut players, mut moves, mut chats) = (0, Vec::new(), 0, Vec::new());
    let (mut bag, mut attacked, mut hits, mut talked) = (Vec::<u32>::new(), false, 0, false);
    let mut nearest_npc: Option<(u32, f32)> = None;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(cli.stay);
    while let Ok(Ok(Some(p))) = tokio::time::timeout_at(deadline, r.recv()).await {
        match p.id {
            id::TS_SC_ENTER => match {
                let e = msg::ScEnter::from_packet(&p)?;
                let d = ((e.x - lr.x).powi(2) + (e.y - lr.y).powi(2)).sqrt();
                if matches!(e.kind, EnterKind::Npc { .. })
                    && nearest_npc.is_none_or(|(_, nd)| d < nd)
                {
                    nearest_npc = Some((e.handle, d));
                }
                if cli.talk && !talked && matches!(e.kind, EnterKind::Npc { .. }) && d < 200.0 {
                    talked = true;
                    w.send(&game::Handle::<{ gid::TS_CS_CONTACT }>(e.handle))
                        .await?;
                }
                if cli.attack && !attacked && matches!(e.kind, EnterKind::Monster { .. }) {
                    attacked = true;
                    tracing::info!("[combat] attaque du monstre {} à {:.0} unités", e.handle, d);
                    // Comme le client : s'approcher de la cible avant de frapper.
                    let t = ((d - 30.0) / d).max(0.0);
                    let dest = MovePoint { x: lr.x + (e.x - lr.x) * t, y: lr.y + (e.y - lr.y) * t };
                    w.send(&msg::CsMoveRequest {
                        handle: lr.handle,
                        x: lr.x,
                        y: lr.y,
                        cur_time: 0,
                        speed_sync: false,
                        points: vec![dest],
                    })
                    .await?;
                    w.send(&game::CsAttackRequest {
                        handle: lr.handle,
                        target_handle: e.handle,
                    })
                    .await?;
                }
                e.kind
            } {
                EnterKind::Npc { .. } => npcs += 1,
                EnterKind::Monster { monster_id, .. } => {
                    tracing::info!("[monde] monstre visible (id brut {monster_id})")
                }
                EnterKind::Player(pl) => players.push(pl.name),
                _ => {}
            },
            id::TS_SC_MOVE => moves += 1,
            id::TS_SC_CHAT_LOCAL => chats.push(msg::ScChatLocal::from_packet(&p)?.message),
            id::TS_SC_CHAT => {
                let c = msg::ScChat::from_packet(&p)?;
                chats.push(format!("{} : {}", c.sender, c.message));
            }
            id::TS_SC_STAT_INFO | id::TS_SC_PROPERTY | id::TS_TIMESYNC => {}
            gid::TS_SC_INVENTORY => {
                for it in game::ScInventory::from_packet(&p)?.items {
                    tracing::info!(
                        "[inventaire] objet {} (code {}, ×{}, position {})",
                        it.handle,
                        it.code,
                        it.count,
                        it.wear_position
                    );
                    if it.wear_position < 0 {
                        bag.push(it.handle);
                    }
                }
                if let (Some(pos), Some(&h)) = (cli.equip, bag.first()) {
                    w.send(&game::CsPutOnItem {
                        position: pos,
                        item_handle: h,
                        target_handle: lr.handle,
                    })
                    .await?;
                }
            }
            gid::TS_SC_ITEM_WEAR_INFO => {
                let iw = game::ScItemWearInfo::from_packet(&p)?;
                tracing::info!(
                    "[équipement] objet {} → position {}",
                    iw.item_handle,
                    iw.wear_position
                );
            }
            gid::TS_SC_WEAR_INFO => {
                let wi = game::ScWearInfo::from_packet(&p)?;
                let worn: Vec<_> = wi
                    .wear_info
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| **c != 0)
                    .collect();
                tracing::info!("[équipement] apparence de {} : {worn:?}", wi.handle);
            }
            gid::TS_SC_GOLD_UPDATE => {
                tracing::info!("[or] {} rupees", game::ScGoldUpdate::from_packet(&p)?.gold)
            }
            gid::TS_SC_EXP_UPDATE => {
                let e = game::ScExpUpdate::from_packet(&p)?;
                tracing::info!("[exp] {} EXP, {} JP", e.exp, e.jp);
            }
            gid::TS_SC_LEVEL_UPDATE => {
                tracing::info!("[niveau] {}", game::ScLevelUpdate::from_packet(&p)?.level)
            }
            gid::TS_SC_ATTACK_EVENT => {
                let a = game::ScAttackEvent::from_packet(&p)?;
                hits += 1;
                if let Some(i) = a.attacks.first() {
                    tracing::info!(
                        "[combat] {} frappe {} : {} dégâts (PV cible {})",
                        a.attacker,
                        a.target,
                        i.damage,
                        i.target_hp
                    );
                }
            }
            gid::TS_SC_DIALOG => {
                let d = game::ScDialog::from_packet(&p)?;
                tracing::info!("[pnj] {} : « {} » menu {:?}", d.title, d.text, d.menu);
                if d.menu.iter().any(|(_, t)| t == "market") {
                    w.send(&game::CsDialog {
                        trigger: "market".into(),
                    })
                    .await?;
                }
            }
            gid::TS_SC_MARKET => {
                let m = game::ScMarket::from_packet(&p)?;
                tracing::info!(
                    "[boutique] {} article(s) : {:?}",
                    m.items.len(),
                    m.items
                        .iter()
                        .map(|i| (i.code, i.price))
                        .collect::<Vec<_>>()
                );
                if let Some(first) = m.items.first() {
                    w.send(&game::CsBuyItem {
                        item_code: first.code,
                        count: 1,
                    })
                    .await?;
                }
            }
            other => tracing::debug!("[monde] paquet {other}"),
        }
    }
    tracing::info!("[combat] {hits} TS_SC_ATTACK_EVENT reçus");
    tracing::info!(
        "[monde] PNJ visibles : {npcs}, joueurs visibles : {players:?}, TS_SC_MOVE reçus : {moves}"
    );
    for c in &chats {
        tracing::info!("[chat] {c}");
    }
    w.send(&msg::Empty::<{ id::TS_CS_LOGOUT }>).await?;
    let _ = expect(&mut r, id::TS_SC_RESULT).await;
    tracing::info!("[monde] déconnexion propre");
    Ok(())
}
