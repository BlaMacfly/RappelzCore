//! RappelzCore authserver.
//!
//! Rôle identique à l'authserver d'AzerothCore : vérifier le compte (`acore_auth.account`,
//! vérificateur SRP6), appliquer les bannissements, publier la liste des royaumes
//! (`realmlist`) et remettre au client une clé de session que le worldserver vérifie
//! (`account.session_key`). Seul le protocole change : celui du client Rappelz.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use rand::RngCore;
use rz_common::Config;
use rz_database::{auth, sqlx::MySqlPool};
use rz_proto::conn::{self, PacketReader, PacketWriter};
use rz_proto::msg::{self, id, result, Message};
use rz_proto::packet::decode_str;
use rz_proto::secure::AesSession;
use tokio::net::TcpListener;

#[derive(Parser)]
#[command(name = "authserver", version, about = "RappelzCore authserver")]
struct Cli {
    /// Fichier de configuration (comme `authserver -c` d'AzerothCore).
    #[arg(short = 'c', long, default_value = "authserver.conf")]
    config: PathBuf,
}

struct Settings {
    key: Vec<u8>,
    wrong_pass_max: u32,
    wrong_pass_ban_time: u32,
    wrong_pass_ban_type: u8,
    auto_create: bool,
    screenshot_url: String,
}

struct Server {
    db: MySqlPool,
    settings: Settings,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let conf = Config::load(&cli.config)?;
    let level = match conf.int("LogLevel", 2u8) {
        0 => "error",
        1 => "warn",
        2 => "info",
        _ => "debug",
    };
    rz_common::init_logging(level);
    rz_common::print_banner("authserver");
    rz_proto::packet::set_codepage(&conf.string("Rappelz.Codepage", "windows-1252"));

    let db = rz_database::connect(
        &conf.string("LoginDatabaseInfo", "127.0.0.1;3306;acore;acore;acore_auth"),
        "Login",
        conf.int("LoginDatabase.WorkerThreads", 4u32),
    )
    .await?;
    let server = Arc::new(Server {
        db,
        settings: Settings {
            key: conf.string("Rappelz.NetworkKey", "}h79q~B%al;k'y $E").into_bytes(),
            wrong_pass_max: conf.int("WrongPass.MaxCount", 0u32),
            wrong_pass_ban_time: conf.int("WrongPass.BanTime", 600u32),
            wrong_pass_ban_type: conf.int("WrongPass.BanType", 0u8),
            auto_create: conf.bool("Rappelz.AutoCreateAccounts", false),
            screenshot_url: conf.string("Rappelz.ServerScreenshotUrl", ""),
        },
    });
    for realm in auth::realms(&server.db).await? {
        tracing::info!("Royaume {} « {} » → {}:{}", realm.id, realm.name, realm.address, realm.port);
    }

    let bind = format!("{}:{}", conf.string("BindIP", "0.0.0.0"), conf.int("RealmServerPort", 4615u16));
    let listener = TcpListener::bind(&bind).await.with_context(|| format!("écoute sur {bind}"))?;
    tracing::info!("Authserver prêt sur {bind}");

    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let (stream, addr) = accepted?;
                let server = server.clone();
                tokio::spawn(async move {
                    let (reader, writer) = conn::split(stream, &server.settings.key);
                    let mut session = AuthSession { server, addr, writer, aes: None, account: None };
                    if let Err(e) = session.run(reader).await {
                        tracing::debug!("{addr} : {e:#}");
                    }
                });
            }
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("Arrêt de l'authserver");
                return Ok(());
            }
        }
    }
}

struct AuthSession {
    server: Arc<Server>,
    addr: SocketAddr,
    writer: PacketWriter,
    aes: Option<AesSession>,
    account: Option<auth::Account>,
}

impl AuthSession {
    fn ip(&self) -> String {
        self.addr.ip().to_string()
    }

    async fn run(&mut self, mut reader: PacketReader) -> Result<()> {
        tracing::debug!("Connexion de {}", self.addr);
        while let Some(packet) = reader.recv().await? {
            match packet.id {
                id::TS_CA_VERSION => {
                    let v = msg::CaVersion::from_packet(&packet)?;
                    tracing::debug!("{} : client version {}", self.addr, v.0.version);
                }
                id::TS_CA_RSA_PUBLIC_KEY => {
                    let key = msg::CaRsaPublicKey::from_packet(&packet)?;
                    let session = AesSession::random();
                    let sealed = session.seal_for(&key.pem)?;
                    self.aes = Some(session);
                    self.writer.send(&msg::AcAesKeyIv { data: sealed }).await?;
                }
                id::TS_CA_ACCOUNT => self.handle_account(msg::CaAccount::from_packet(&packet)?).await?,
                id::TS_CA_SERVER_LIST => self.handle_server_list().await?,
                id::TS_CA_SELECT_SERVER => {
                    let sel = msg::CaSelectServer::from_packet(&packet)?;
                    self.handle_select_server(sel.server_idx).await?;
                }
                other => tracing::debug!("{} : paquet auth {other} ignoré", self.addr),
            }
        }
        Ok(())
    }

    async fn reply(&mut self, code: u16, login_flag: i32) -> Result<()> {
        self.writer
            .send(&msg::AcResult { request: id::TS_CA_ACCOUNT, result: code, login_flag })
            .await?;
        Ok(())
    }

    async fn handle_account(&mut self, req: msg::CaAccount) -> Result<()> {
        let Some(aes) = &self.aes else {
            return self.reply(result::ACCESS_DENIED, 0).await;
        };
        let password = match aes.decrypt(&req.password) {
            Ok(p) => decode_str(&p),
            Err(_) => return self.reply(result::INVALID_PASSWORD, 0).await,
        };
        let username = req.account.trim().to_string();
        let ip = self.ip();
        let db = self.server.db.clone();

        let mut account = auth::find_account(&db, &username).await?;
        if account.is_none() && self.server.settings.auto_create && !username.is_empty() {
            let id = auth::create_account(&db, &username, &password, "").await?;
            tracing::info!("Compte {username} créé automatiquement (id {id})");
            account = auth::find_account(&db, &username).await?;
        }
        let Some(account) = account else {
            tracing::info!("{ip} : compte inconnu « {username} »");
            return self.reply(result::NOT_EXIST, 0).await;
        };
        if account.locked && account.last_ip != ip {
            return self.reply(result::BLOCKED, 0).await;
        }
        if auth::is_banned(&db, account.id, &ip).await? {
            tracing::info!("{ip} : compte {} banni", account.username);
            return self.reply(result::BLOCKED, 0).await;
        }
        if !rz_database::srp6::check_password(&username, &password, &account.salt, &account.verifier) {
            tracing::info!("{ip} : mauvais mot de passe pour {}", account.username);
            auth::record_failed_login(&db, &username, &ip).await?;
            self.apply_wrong_pass_policy(&account, &ip).await?;
            return self.reply(result::INVALID_PASSWORD, 0).await;
        }
        tracing::info!("{ip} : connexion du compte {} (id {})", account.username, account.id);
        self.account = Some(account);
        self.reply(result::SUCCESS, 0).await
    }

    /// Équivalent de WrongPass.MaxCount / BanTime / BanType d'AzerothCore.
    async fn apply_wrong_pass_policy(&self, account: &auth::Account, ip: &str) -> Result<()> {
        let s = &self.server.settings;
        if s.wrong_pass_max == 0 {
            return Ok(());
        }
        let failed: u32 = rz_database::sqlx::query_scalar("SELECT failed_logins FROM account WHERE id = ?")
            .bind(account.id)
            .fetch_one(&self.server.db)
            .await?;
        if failed < s.wrong_pass_max {
            return Ok(());
        }
        if s.wrong_pass_ban_type == 1 {
            rz_database::sqlx::query(
                "INSERT INTO account_banned (id, bandate, unbandate, bannedby, banreason, active) \
                 VALUES (?, UNIX_TIMESTAMP(), UNIX_TIMESTAMP() + ?, 'Trinity Auth', 'Failed login autoban', 1)",
            )
            .bind(account.id)
            .bind(s.wrong_pass_ban_time)
            .execute(&self.server.db)
            .await?;
        } else {
            rz_database::sqlx::query(
                "INSERT INTO ip_banned (ip, bandate, unbandate, bannedby, banreason) \
                 VALUES (?, UNIX_TIMESTAMP(), UNIX_TIMESTAMP() + ?, 'Trinity Auth', 'Failed login autoban')",
            )
            .bind(ip)
            .bind(s.wrong_pass_ban_time)
            .execute(&self.server.db)
            .await?;
        }
        tracing::warn!("{ip} : bannissement automatique après {failed} échecs");
        Ok(())
    }

    async fn visible_realms(&self) -> Result<Vec<(auth::Realm, msg::ServerInfo)>> {
        let Some(account) = &self.account else { return Ok(Vec::new()) };
        let local = self.addr.ip().is_loopback();
        let mut out = Vec::new();
        for realm in auth::realms(&self.server.db).await? {
            let gm = auth::gm_level(&self.server.db, account.id, realm.id as i32).await?;
            if gm < realm.allowed_security_level {
                continue;
            }
            let ip = if local { realm.local_address.clone() } else { realm.address.clone() };
            let info = msg::ServerInfo {
                index: realm.id as u16,
                name: realm.name.clone(),
                is_adult: false,
                screenshot_url: self.server.settings.screenshot_url.clone(),
                ip,
                port: realm.port as i32,
                user_ratio: (realm.population * 100.0).clamp(0.0, 100.0) as u16,
            };
            out.push((realm, info));
        }
        Ok(out)
    }

    async fn handle_server_list(&mut self) -> Result<()> {
        let servers = self.visible_realms().await?.into_iter().map(|(_, s)| s).collect();
        self.writer.send(&msg::AcServerList { last_login_server_idx: 1, servers }).await?;
        Ok(())
    }

    async fn handle_select_server(&mut self, idx: u16) -> Result<()> {
        let (Some(account), Some(aes)) = (self.account.clone(), self.aes.clone()) else {
            return Ok(());
        };
        let realms = self.visible_realms().await?;
        if !realms.iter().any(|(r, _)| r.id == idx as u32) {
            self.writer
                .send(&msg::AcSelectServer { result: result::NOT_EXIST, encrypted_key: Vec::new(), pending_time: 0 })
                .await?;
            return Ok(());
        }
        let one_time_key = rand::thread_rng().next_u64() | 1;
        // `session_key` est un binary(40) dans AzerothCore : la clé u64 occupe les 8 premiers octets.
        let mut session_key = [0u8; 40];
        session_key[..8].copy_from_slice(&one_time_key.to_le_bytes());
        auth::record_login(&self.server.db, account.id, &self.ip(), &session_key).await?;
        let encrypted_key = aes.encrypt(&one_time_key.to_le_bytes());
        tracing::info!("{} : {} rejoint le royaume {idx}", self.ip(), account.username);
        self.writer.send(&msg::AcSelectServer { result: result::SUCCESS, encrypted_key, pending_time: 0 }).await?;
        Ok(())
    }
}
