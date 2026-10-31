//! Réseau du client, dans un thread tokio séparé du rendu.
//! Même protocole que SFrame.exe : RappelzRust peut se connecter à RappelzCore comme
//! à un serveur Rappelz compatible NoLimits.

use std::sync::mpsc::Sender;
use std::time::Instant;

use anyhow::{bail, Context, Result};
use rz_proto::conn::{self, PacketReader, PacketWriter};
use rz_proto::msg::{self, id, Message};
use rz_proto::secure::ClientKeyPair;
use tokio::net::TcpStream;
use tokio::sync::mpsc;

pub enum NetCmd {
    Login { host: String, port: u16, account: String, password: String },
    SelectServer(u16),
    CreateCharacter(msg::LobbyCharacterInfo),
    DeleteCharacter(String),
    EnterWorld(String),
    Move { x: f32, y: f32, points: Vec<msg::MovePoint> },
    Chat { chat_type: u8, target: String, message: String },
    ReturnLobby,
    /// Close the current transport; the existing outer loop then accepts a fresh Login.
    Disconnect,
    /// Message de gameplay déjà construit (rz_proto::msg_game).
    Raw(rz_proto::RawPacket),
}

pub enum NetEvent {
    RealmSelected(u16),
    Status(String),
    Error(String),
    ServerList(Vec<msg::ServerInfo>),
    CharacterList(Vec<msg::LobbyCharacterInfo>),
    Result { request: u16, result: u16 },
    LoginResult(msg::ScLoginResult),
    Enter(msg::ScEnter),
    Leave(u32),
    Move(msg::ScMove),
    Warp { x: f32, y: f32, z: f32, layer: u8 },
    Chat { sender: String, chat_type: u8, message: String },
    ChatLocal { handle: u32, message: String },
    Property { handle: u32, name: String, value: i64 },
    TimeSync,
    Disconnected(String),
    /// Message de gameplay (inventaire, dialogue, combat…) décodé par le monde.
    Packet(rz_proto::RawPacket),
}

/// Démarre le thread réseau ; renvoie l'émetteur de commandes.
pub fn spawn(events: Sender<NetEvent>) -> mpsc::UnboundedSender<NetCmd> {
    let (tx, rx) = mpsc::unbounded_channel();
    std::thread::Builder::new()
        .name("reseau".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("runtime tokio");
            rt.block_on(run(rx, events));
        })
        .expect("thread réseau");
    tx
}

async fn run(mut cmds: mpsc::UnboundedReceiver<NetCmd>, events: Sender<NetEvent>) {
    while let Some(cmd) = cmds.recv().await {
        if let NetCmd::Login { host, port, account, password } = cmd {
            let ev = events.clone();
            match session(&host, port, &account, &password, &mut cmds, &ev).await {
                Ok(()) => {
                    let _ = ev.send(NetEvent::Disconnected("Déconnecté.".into()));
                }
                Err(e) => {
                    let _ = ev.send(NetEvent::Error(format!("{e:#}")));
                }
            }
        }
    }
}

async fn expect(reader: &mut PacketReader, wanted: u16) -> Result<rz_proto::RawPacket> {
    loop {
        let p = reader.recv().await?.context("connexion fermée par le serveur")?;
        if p.id == wanted {
            return Ok(p);
        }
    }
}

/// Flux complet : authserver puis worldserver, jusqu'à la déconnexion.
async fn session(
    host: &str,
    port: u16,
    account: &str,
    password: &str,
    cmds: &mut mpsc::UnboundedReceiver<NetCmd>,
    events: &Sender<NetEvent>,
) -> Result<()> {
    let status = |s: &str| {
        let _ = events.send(NetEvent::Status(s.to_string()));
    };
    status("Connexion au serveur d'authentification…");
    let stream = TcpStream::connect((host, port)).await.with_context(|| format!("connexion à {host}:{port}"))?;
    let (mut r, mut w) = conn::split(stream, rz_proto::cipher::DEFAULT_KEY);
    w.send(&msg::CaVersion(msg::Version { version: msg::CLIENT_VERSION.into() })).await?;
    let keys = tokio::task::spawn_blocking(ClientKeyPair::generate).await??;
    w.send(&msg::CaRsaPublicKey { pem: keys.public_pem() }).await?;
    let sealed = msg::AcAesKeyIv::from_packet(&expect(&mut r, id::TS_AC_AES_KEY_IV).await?)?;
    let aes = keys.open(&sealed.data)?;

    status("Vérification du compte…");
    let mut pwd = rz_proto::packet::encode_str(password);
    pwd.push(0);
    w.send(&msg::CaAccount { account: account.to_string(), password: aes.encrypt(&pwd) }).await?;
    let res = msg::AcResult::from_packet(&expect(&mut r, id::TS_AC_RESULT).await?)?;
    match res.result {
        msg::result::SUCCESS => {}
        msg::result::NOT_EXIST => bail!("Compte inconnu."),
        msg::result::INVALID_PASSWORD => bail!("Mot de passe incorrect."),
        msg::result::BLOCKED => bail!("Compte ou adresse bloqué."),
        other => bail!("Connexion refusée (code {other})."),
    }
    w.send(&msg::CaServerList).await?;
    let list = msg::AcServerList::from_packet(&expect(&mut r, id::TS_AC_SERVER_LIST).await?)?;
    let servers = list.servers.clone();
    let _ = events.send(NetEvent::ServerList(list.servers));

    // Attente du choix du royaume.
    let idx = loop {
        match cmds.recv().await {
            Some(NetCmd::SelectServer(i)) => break i,
            Some(_) => continue,
            None => return Ok(()),
        }
    };
    w.send(&msg::CaSelectServer { server_idx: idx }).await?;
    let _ = events.send(NetEvent::RealmSelected(idx));
    let sel = msg::AcSelectServer::from_packet(&expect(&mut r, id::TS_AC_SELECT_SERVER).await?)?;
    if sel.result != 0 {
        bail!("Royaume indisponible (code {}).", sel.result);
    }
    let key_bytes = aes.decrypt(&sel.encrypted_key)?;
    anyhow::ensure!(key_bytes.len() >= 8, "clé de session invalide");
    let one_time_key = u64::from_le_bytes(key_bytes[..8].try_into().unwrap());
    let _ = w.shutdown().await;
    drop(r);

    let server = servers.iter().find(|s| s.index == idx).context("royaume introuvable")?;
    status(&format!("Connexion à {}…", server.name));
    let stream = TcpStream::connect((server.ip.as_str(), server.port as u16))
        .await
        .with_context(|| format!("connexion à {}:{}", server.ip, server.port))?;
    let (mut r, mut w) = conn::split(stream, rz_proto::cipher::DEFAULT_KEY);
    w.send(&msg::CsVersion(msg::Version { version: msg::CLIENT_VERSION.into() })).await?;
    w.send(&msg::CsAccountWithAuth { account: account.to_string(), one_time_key }).await?;
    game_loop(account, &mut r, &mut w, cmds, events).await
}

async fn game_loop(
    account: &str,
    r: &mut PacketReader,
    w: &mut PacketWriter,
    cmds: &mut mpsc::UnboundedReceiver<NetCmd>,
    events: &Sender<NetEvent>,
) -> Result<()> {
    let started = Instant::now();
    let mut server_offset: i64 = 0;
    let server_now = |offset: i64| (started.elapsed().as_millis() as i64 / 10 + offset).max(0) as u32;
    let mut handle = 0u32;
    loop {
        tokio::select! {
            packet = r.recv() => {
                let Some(p) = packet? else { return Ok(()) };
                let ev = match p.id {
                    id::TS_SC_RESULT => {
                        let res = msg::ScResult::from_packet(&p)?;
                        if res.request == id::TS_CS_ACCOUNT_WITH_AUTH {
                            if res.result != 0 {
                                bail!("Le serveur de jeu a refusé la session (code {}).", res.result);
                            }
                            w.send(&msg::CsCharacterList { account: account.to_string() }).await?;
                            continue;
                        }
                        if matches!(res.request, id::TS_CS_CREATE_CHARACTER | id::TS_CS_DELETE_CHARACTER | id::TS_CS_RETURN_LOBBY)
                            && res.result == 0
                        {
                            w.send(&msg::CsCharacterList { account: account.to_string() }).await?;
                        }
                        NetEvent::Result { request: res.request, result: res.result }
                    }
                    id::TS_SC_CHARACTER_LIST => NetEvent::CharacterList(msg::ScCharacterList::from_packet(&p)?.characters),
                    id::TS_SC_LOGIN_RESULT => {
                        let lr = msg::ScLoginResult::from_packet(&p)?;
                        handle = lr.handle;
                        w.send(&rz_proto::party::CsPartyHello(rz_proto::party::VERSION)).await?;
                        w.send(&rz_proto::skills::CsSkillsHello(rz_proto::skills::VERSION)).await?;
                        NetEvent::LoginResult(lr)
                    }
                    id::TS_SC_ENTER => NetEvent::Enter(msg::ScEnter::from_packet(&p)?),
                    id::TS_SC_LEAVE => NetEvent::Leave(msg::ScLeave::from_packet(&p)?.handle),
                    id::TS_SC_MOVE => NetEvent::Move(msg::ScMove::from_packet(&p)?),
                    id::TS_SC_WARP => {
                        let wp = msg::ScWarp::from_packet(&p)?;
                        NetEvent::Warp { x: wp.x, y: wp.y, z: wp.z, layer: wp.layer }
                    }
                    id::TS_SC_CHAT => {
                        let c = msg::ScChat::from_packet(&p)?;
                        NetEvent::Chat { sender: c.sender, chat_type: c.chat_type, message: c.message }
                    }
                    id::TS_SC_CHAT_LOCAL => {
                        let c = msg::ScChatLocal::from_packet(&p)?;
                        NetEvent::ChatLocal { handle: c.handle, message: c.message }
                    }
                    id::TS_SC_PROPERTY => {
                        let pr = msg::ScProperty::from_packet(&p)?;
                        NetEvent::Property { handle: pr.handle, name: pr.name, value: pr.value }
                    }
                    id::TS_TIMESYNC => {
                        let t = msg::TimeSync::from_packet(&p)?;
                        server_offset = t.time as i64 - started.elapsed().as_millis() as i64 / 10;
                        w.send(&t).await?;
                        NetEvent::TimeSync
                    }
                    id::TS_SC_DISCONNECT_DESC => {
                        let d = msg::ScDisconnectDesc::from_packet(&p)?;
                        return Err(anyhow::anyhow!("Déconnecté par le serveur (code {}).", d.code));
                    }
                    _ => NetEvent::Packet(p),
                };
                if events.send(ev).is_err() {
                    return Ok(());
                }
            }
            cmd = cmds.recv() => {
                let Some(cmd) = cmd else { return Ok(()) };
                match cmd {
                    NetCmd::CreateCharacter(info) => w.send(&msg::CsCreateCharacter { info }).await?,
                    NetCmd::DeleteCharacter(name) => w.send(&msg::CsDeleteCharacter { name, security_no: String::new() }).await?,
                    NetCmd::EnterWorld(name) => w.send(&msg::CsLogin { name }).await?,
                    NetCmd::Move { x, y, points } => {
                        w.send(&msg::CsMoveRequest { handle, x, y, cur_time: server_now(server_offset), speed_sync: false, points }).await?
                    }
                    NetCmd::Chat { chat_type, target, message } => {
                        w.send(&msg::CsChatRequest { target, request_id: 0, chat_type, message }).await?
                    }
                    NetCmd::ReturnLobby => w.send(&msg::Empty::<{ id::TS_CS_RETURN_LOBBY }>).await?,
                    NetCmd::Disconnect => return Ok(()),
                    NetCmd::Raw(p) => w.send_raw(&p).await?,
                    NetCmd::Login { .. } | NetCmd::SelectServer(_) => {}
                }
            }
        }
    }
}
