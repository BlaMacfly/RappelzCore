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
    /// Avec --talk : ouvre cette quête chez le PNJ, l'accepte ou la rend (comme la fenêtre de récompense).
    #[arg(long)]
    quest: Option<i32>,
    /// Avec --talk : ouvre l'entrepôt du PNJ, y dépose le premier objet du sac et 100 rupees.
    #[arg(long)]
    storage: bool,
    /// Propose un échange au joueur nommé (premier objet du sac + 50 rupees), jusqu'à la validation.
    #[arg(long)]
    trade_with: Option<String>,
    /// Lance cette compétence (niveau 1) sur le premier monstre visible, jusqu'à sa mort.
    #[arg(long)]
    cast: Option<u16>,
    /// Avec --cast : vise son propre personnage (bonus, soins).
    #[arg(long)]
    cast_self: bool,
    /// Utilise l'objet du sac portant ce code (une fois, ou --again secondes plus tard une seconde fois).
    #[arg(long)]
    use_item: Option<i32>,
    /// Avec --summon : la créature attaque le premier monstre visible (handle de la créature).
    #[arg(long)]
    summon_attack: bool,
    /// Avec --cast : redemande la compétence une fois, ce nombre de secondes après son tir (bascules).
    #[arg(long)]
    again: Option<u64>,
    /// Avec --summon ou --cast : bouge pendant l'incantation (elle doit être annulée).
    #[arg(long)]
    interrupt: bool,
    /// Coupe brutalement la connexion juste après avoir validé l'échange (test de robustesse).
    #[arg(long)]
    vanish: bool,
    /// Accepte la première demande d'échange reçue et valide.
    #[arg(long)]
    trade_accept: bool,
    /// Ramasse les objets au sol visibles (TS_CS_TAKE_ITEM).
    #[arg(long)]
    loot: bool,
    /// Invoque la créature de la première carte reçue (TS_CS_SKILL 4001, formation place 1).
    #[arg(long)]
    summon: bool,
    /// Renvoie la créature dès qu'elle apparaît (TS_CS_SKILL 4002).
    #[arg(long)]
    unsummon: bool,
    /// Jette au sol le premier objet du sac (TS_CS_DROP_ITEM).
    #[arg(long)]
    drop: bool,
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
    // Une incantation exige d'être immobile.
    if cli.cast.is_none() && !cli.summon {
        let target = MovePoint { x: lr.x + 40.0, y: lr.y + 30.0 };
        let points = vec![target];
        w.send(&msg::CsMoveRequest { handle: lr.handle, x: lr.x, y: lr.y, cur_time: 0, speed_sync: false, points }).await?;
    }
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
    // Avec --quest : PNJ à portée encore à essayer.
    let mut npc_queue: std::collections::VecDeque<u32> = Default::default();
    let mut cast_target = None::<u32>;
    let mut again = cli.again;
    let mut first_monster = None::<u32>;
    let mut used = false;
    // Invocations : handle de créature → handle de carte.
    let mut card_of = std::collections::HashMap::<u32, u32>::new();
    let skill = |skill_id: u16, caster: u32, target: u32| game::CsSkill { skill_id, caster, target, x: 0.0, y: 0.0, z: 0.0, layer: 0, skill_level: 1 };
    // Échange : partenaire, verrous et confirmations vus.
    let (mut trade_partner, mut requested) = (None::<u32>, false);
    let (mut frozen, mut confirmed, mut processed) = (std::collections::HashSet::<u32>::new(), std::collections::HashSet::<u32>::new(), false);
    if let (Some(id), true) = (cli.cast, cli.cast_self) {
        cast_target = Some(lr.handle);
        w.send(&skill(id, lr.handle, lr.handle)).await?;
    }
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
                if cli.talk && cli.quest.is_some() && talked && matches!(e.kind, EnterKind::Npc { .. }) && d < 200.0 {
                    npc_queue.push_back(e.handle);
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
                (e.handle, e.kind)
            } {
                (_, EnterKind::Npc { .. }) => npcs += 1,
                (h, EnterKind::Monster { monster_id, creature, .. }) => {
                    tracing::info!("[monde] monstre visible (id brut {monster_id}, {} PV)", creature.hp);
                    first_monster.get_or_insert(h);
                    if let (Some(id), None) = (cli.cast, cast_target) {
                        cast_target = Some(h);
                        w.send(&skill(id, lr.handle, h)).await?;
                    }
                }
                (enter_handle, EnterKind::Player(pl)) => {
                    if !requested && cli.trade_with.as_deref() == Some(pl.name.as_str()) {
                        requested = true;
                        tracing::info!("[échange] demande à {}", pl.name);
                        w.send(&game::Trade { handle: enter_handle, mode: game::trade_mode::REQUEST, ..Default::default() }).await?;
                    }
                    players.push(pl.name)
                }
                (h, EnterKind::Summon { creature, master, code, name, .. }) => {
                    tracing::info!("[invocation] {name} (créature {code}, niveau {}, handle {h}) de {master} apparaît", creature.level);
                    if let (true, Some(m)) = (cli.summon_attack && master == lr.handle, first_monster) {
                        tracing::info!("[invocation] la créature attaque {m}");
                        w.send(&game::CsAttackRequest { handle: h, target_handle: m }).await?;
                    }
                    if let (true, Some(&card)) = (cli.unsummon && master == lr.handle, card_of.get(&h)) {
                        w.send(&skill(game::summon_skill::UNSUMMON, lr.handle, card)).await?;
                    }
                }
                (h, EnterKind::Item { code, count, owners, .. }) => {
                    tracing::info!("[sol] objet {h} : code {code} ×{count} (ayant droit {})", owners[0]);
                    if cli.loot {
                        w.send(&game::CsTakeItem { taker_handle: lr.handle, item_handle: h }).await?;
                    }
                }
                (_, _) => {}
            },
            gid::TS_SC_ADD_SUMMON_INFO => {
                let a = game::ScAddSummonInfo::from_packet(&p)?;
                tracing::info!("[invocation] fiche : carte {} → créature {} « {} » niveau {}", a.card_handle, a.code, a.name, a.level);
                card_of.insert(a.summon_handle, a.card_handle);
                if cli.summon && card_of.len() == 1 {
                    let mut cards = [0; game::FORMATION_SLOTS];
                    cards[0] = a.card_handle;
                    w.send(&game::EquipSummon { open_dialog: 0, cards }).await?;
                    w.send(&skill(game::summon_skill::SUMMON, lr.handle, a.card_handle)).await?;
                }
            }
            gid::TS_SC_REMOVE_SUMMON_INFO => tracing::info!("[invocation] fiche retirée : carte {}", game::Handle::<{ gid::TS_SC_REMOVE_SUMMON_INFO }>::from_packet(&p)?.0),
            gid::TS_EQUIP_SUMMON => tracing::info!("[invocation] formation {:?}", game::EquipSummon::from_packet(&p)?.cards),
            gid::TS_SC_UNSUMMON => tracing::info!("[invocation] renvoi de {}", game::Handle::<{ gid::TS_SC_UNSUMMON }>::from_packet(&p)?.0),
            gid::TS_SC_SKILL => {
                let s = game::ScSkill::from_packet(&p)?;
                let phase = ["FIRE", "CASTING", "UPDATE", "CANCEL", "REGION_FIRE", "COMPLETE"].get(s.skill_type as usize).copied().unwrap_or("?");
                let int = |o: usize| s.payload.get(o..o + 4).map_or(0, |b| i32::from_le_bytes(b.try_into().unwrap()));
                // Impact : PV finaux en +5, montant en +9 (soin) ou +0xa (dégâts).
                let hit = s.payload.get(9).copied();
                match (phase, hit) {
                    ("CASTING", _) => tracing::info!("[compétence] {} CASTING {} cs", s.skill_id, int(0)),
                    ("FIRE", Some(game::skill_hit::RESULT)) => tracing::info!("[compétence] {} FIRE état posé : {}, PM {}", s.skill_id, s.payload.get(14) == Some(&1), s.caster_mp),
                    ("FIRE", Some(20)) => tracing::info!("[compétence] {} FIRE soin {} (PV {}), PM {}", s.skill_id, int(18), int(14), s.caster_mp),
                    ("FIRE", Some(_)) => {
                        // Un impact de 32 octets par coup et par cible.
                        let n = u16::from_le_bytes([s.payload[7], s.payload[8]]);
                        tracing::info!("[compétence] {} FIRE {n} impacts, {} cibles × {} coups, PM {}", s.skill_id, s.payload[5], s.payload[6], s.caster_mp);
                        for k in 0..usize::from(n) {
                            let o = 9 + 32 * k;
                            tracing::info!("[compétence]   → {} : {} dégâts (PV {})", int(o + 1), int(o + 10), int(o + 5));
                        }
                    }
                    _ => tracing::info!("[compétence] {} {phase}, PM {}", s.skill_id, s.caster_mp),
                }
                if cli.interrupt && phase == "CASTING" {
                    w.send(&msg::CsMoveRequest { handle: lr.handle, x: lr.x, y: lr.y, cur_time: 0, speed_sync: false, points: vec![MovePoint { x: lr.x + 5.0, y: lr.y }] }).await?;
                }
                if let (Some(id), Some(target), "FIRE", Some(secs)) = (cli.cast, cast_target, phase, again) {
                    again = None;
                    tokio::time::sleep(Duration::from_secs(secs)).await;
                    w.send(&skill(id, lr.handle, target)).await?;
                }
                if let (Some(id), Some(target)) = (cli.cast, cast_target) {
                    if phase == "FIRE" && hit != Some(game::skill_hit::RESULT) && int(14) > 0 {
                        tokio::time::sleep(Duration::from_millis(1100)).await;
                        w.send(&skill(id, lr.handle, target)).await?;
                    }
                }
            }
            gid::TS_SC_STATE => {
                let s = game::ScState::from_packet(&p)?;
                tracing::info!("[état] porteur {} : code {} niveau {} (instance {}, {} cs)", s.handle, s.state_code, s.state_level, s.state_handle, s.remain_time - s.start_time as i32);
            }
            gid::TS_SC_ITEM_COOL_TIME => {
                let c = game::ScItemCoolTime::from_packet(&p)?;
                let running: Vec<(usize, i32)> = c.cool_time.iter().enumerate().filter(|(_, v)| **v > 0).map(|(i, v)| (i + 1, *v)).collect();
                tracing::info!("[objet] recharges (groupe, cs) : {running:?}");
            }
            gid::TS_SC_UPDATE_ITEM_COUNT => {
                let u = game::ScUpdateItemCount::from_packet(&p)?;
                tracing::info!("[objet] {} : reste {}", u.item_handle, u.count);
            }
            gid::TS_SC_AURA => {
                let a = game::ScAura::from_packet(&p)?;
                tracing::info!("[bascule] compétence {} active : {}", a.skill_id, a.active);
            }
            gid::TS_SC_HPMP => {
                let v = game::ScHpMp::from_packet(&p)?;
                if v.handle == lr.handle {
                    tracing::info!("[vitaux] PV {}/{} PM {}/{}", v.hp, v.max_hp, v.mp, v.max_mp);
                }
            }
            gid::TS_SC_STATE_DAMAGE => {
                let d = game::ScStateDamage::from_packet(&p)?;
                tracing::info!("[état] tic {} sur {} : type {} valeur {} → {} (total {}, fin {})", d.state_code, d.target, d.kind, d.value, d.result, d.total, d.last);
            }
            id::TS_SC_STAT_INFO => {
                let s = msg::ScStatInfo::from_packet(&p)?;
                if let Some(c) = s.combat_stats() {
                    let base: Vec<i16> = (1..8).map(|n| i16::from_le_bytes([s.stats[2 * n], s.stats[2 * n + 1]])).collect();
                    let weight = i32::from_le_bytes(s.stats[0x3e..0x42].try_into().unwrap());
                    tracing::info!("[stats] attaque {} défense {} vitesse {} FOR/VIT/DEX/AGI/INT/SAG/CHA {base:?} poids {weight}", c.attack, c.defence, c.move_speed);
                }
            }
            gid::TS_SC_TAKE_ITEM_RESULT => {
                let r = game::ScTakeItemResult::from_packet(&p)?;
                tracing::info!("[sol] ramassage : objet {} par {}", r.0, r.1);
            }
            gid::TS_SC_ITEM_DROP_INFO => {
                let r = game::ScItemDropInfo::from_packet(&p)?;
                tracing::info!("[sol] butin : monstre {} → objet {}", r.0, r.1);
            }
            gid::TS_SC_DROP_RESULT => {
                let r = game::ScDropResult::from_packet(&p)?;
                tracing::info!("[sol] dépôt de {} : {}", r.item_handle, r.is_success);
            }
            id::TS_SC_RESULT => {
                let r = msg::ScResult::from_packet(&p)?;
                tracing::info!("[résultat] requête {} → {} (valeur {})", r.request, r.result, r.value);
            }
            id::TS_SC_LEAVE => tracing::info!("[monde] départ de {}", msg::ScLeave::from_packet(&p)?.handle),
            gid::TS_TRADE => {
                use game::trade_mode as tm;
                let t = game::Trade::from_packet(&p)?;
                tracing::info!("[échange] reçu mode {} de {} (objet {} ×{})", t.mode, t.handle, t.item.code, t.item.count);
                match t.mode {
                    tm::REQUEST if cli.trade_accept && trade_partner.is_none() => {
                        trade_partner = Some(t.handle);
                        w.send(&game::Trade { handle: t.handle, mode: tm::ACCEPT, ..Default::default() }).await?;
                    }
                    tm::BEGIN => {
                        trade_partner = Some(t.handle);
                        if cli.trade_with.is_some() {
                            if let Some(&h) = bag.first() {
                                let item = game::ItemInfo { handle: h, count: 1, ..Default::default() };
                                w.send(&game::Trade { handle: t.handle, mode: tm::ADD_ITEM, item }).await?;
                            }
                            let item = game::ItemInfo { count: 50, ..Default::default() };
                            w.send(&game::Trade { handle: t.handle, mode: tm::ADD_GOLD, item }).await?;
                        }
                        w.send(&game::Trade { handle: t.handle, mode: tm::FREEZE, ..Default::default() }).await?;
                    }
                    tm::FREEZE => {
                        frozen.insert(t.handle);
                        if frozen.len() == 2 {
                            w.send(&game::Trade { handle: trade_partner.unwrap_or(0), mode: tm::CONFIRM, ..Default::default() }).await?;
                        }
                    }
                    tm::CONFIRM => {
                        confirmed.insert(t.handle);
                        if confirmed.len() == 2 && !processed {
                            processed = true;
                            w.send(&game::Trade { handle: trade_partner.unwrap_or(0), mode: tm::PROCESS, ..Default::default() }).await?;
                            if cli.vanish {
                                tracing::info!("[échange] connexion coupée après validation");
                                std::process::exit(0);
                            }
                        }
                    }
                    tm::PROCESS => tracing::info!("[échange] terminé"),
                    _ => {}
                }
            }
            gid::TS_SC_OPEN_STORAGE => {
                tracing::info!("[entrepôt] ouvert ({})", game::ScOpenStorage::from_packet(&p)?.0);
                if let Some(&h) = bag.first() {
                    w.send(&game::CsStorage { item_handle: h, mode: game::storage_mode::DEPOSIT_ITEM, count: 0 }).await?;
                }
                w.send(&game::CsStorage { item_handle: 0, mode: game::storage_mode::DEPOSIT_GOLD, count: 100 }).await?;
                w.send(&game::CsStorage { item_handle: 0, mode: game::storage_mode::CLOSE, count: 0 }).await?;
            }
            id::TS_SC_PROPERTY => {
                let pr = msg::ScProperty::from_packet(&p)?;
                if pr.name == "storage_gold" {
                    tracing::info!("[entrepôt] rupees stockées : {}", pr.value);
                }
            }
            id::TS_SC_MOVE => moves += 1,
            id::TS_SC_CHAT_LOCAL => chats.push(msg::ScChatLocal::from_packet(&p)?.message),
            id::TS_SC_CHAT => {
                let c = msg::ScChat::from_packet(&p)?;
                chats.push(format!("{} : {}", c.sender, c.message));
            }
            id::TS_TIMESYNC => {}
            gid::TS_SC_INVENTORY => {
                for it in game::ScInventory::from_packet(&p)?.items {
                    tracing::info!(
                        "[inventaire] objet {} (code {}, ×{}, position {})",
                        it.handle,
                        it.code,
                        it.count,
                        it.wear_position
                    );
                    if it.wear_position == game::WEAR_NONE {
                        bag.push(it.handle);
                    }
                    if cli.use_item.is_some_and(|c| c as i64 == it.code as i64) && !used {
                        used = true;
                        tracing::info!("[objet] utilisation de {} (code {})", it.handle, it.code);
                        w.send(&game::CsUseItem { item_handle: it.handle, target_handle: 0, parameter: String::new() }).await?;
                        if let Some(secs) = cli.again {
                            tokio::time::sleep(Duration::from_secs(secs)).await;
                            tracing::info!("[objet] seconde utilisation");
                            w.send(&game::CsUseItem { item_handle: it.handle, target_handle: 0, parameter: String::new() }).await?;
                        }
                    }
                }
                if let (true, Some(&h)) = (cli.drop, bag.first()) {
                    w.send(&game::CsDropItem { item_handle: h, count: 1 }).await?;
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
                let quest_step = d.menu.iter().find_map(|(label, t)| {
                    if cli.quest.is_some_and(|c| *t == format!("quest_show|{c}") || *t == game::QuestTrigger::start(c)) {
                        Some(t.clone())
                    } else if label == "REWARD" && cli.quest.is_some_and(|c| *t == c.to_string()) {
                        // La fenêtre de récompense du client formate elle-même ce trigger.
                        Some(format!("end_quest( {t}, -1 )"))
                    } else {
                        None
                    }
                });
                if cli.storage && d.menu.iter().any(|(_, t)| t.starts_with("open_storage()")) {
                    tracing::info!("[entrepôt] choix « open_storage() »");
                    w.send(&game::CsDialog { trigger: "open_storage()".into() }).await?;
                } else if let Some(trigger) = quest_step {
                    tracing::info!("[quête] choix « {trigger} »");
                    w.send(&game::CsDialog { trigger }).await?;
                } else if let (Some(_), Some(next)) = (cli.quest, npc_queue.pop_front()) {
                    w.send(&game::Handle::<{ gid::TS_CS_CONTACT }>(next)).await?;
                } else if d.menu.iter().any(|(_, t)| t == "market") {
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
            gid::TS_SC_QUEST_LIST => {
                let l = game::ScQuestList::from_packet(&p)?;
                for q in &l.active {
                    tracing::info!("[quête] liste : {} statut {:?} progression {}", q.code, q.status, q.progress);
                }
                if l.active.is_empty() {
                    tracing::info!("[quête] liste vide");
                }
            }
            gid::TS_SC_QUEST_STATUS => {
                let s = game::ScQuestStatus::from_packet(&p)?;
                tracing::info!("[quête] mise à jour : {} statut {:?} progression {}", s.code, s.status, s.progress);
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
