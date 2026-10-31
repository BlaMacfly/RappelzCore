//! RappelzCore worldserver.
//!
//! Démarrage calqué sur AzerothCore : lecture de `worldserver.conf`, connexion aux bases
//! Login/World/Character, chargement des DBC (`DataDir/dbc`) et des tables `acore_world`,
//! puis écoute des joueurs sur `WorldServerPort`, console `worldserver>` sur l'entrée standard.

mod commands;
mod data;
mod gameplay;
mod party;
mod session;
mod skills;
mod casting;
mod world;

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use rz_common::Config;
use rz_database::sqlx::MySqlPool;
use rz_database::{auth, characters};
use tokio::io::AsyncBufReadExt;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, watch};

#[derive(Parser)]
#[command(name = "worldserver", version, about = "RappelzCore worldserver")]
struct Cli {
    #[arg(short = 'c', long, default_value = "worldserver.conf")]
    config: PathBuf,
}

pub struct Settings {
    pub realm_id: u32,
    pub network_key: Vec<u8>,
    pub characters_per_realm: usize,
    pub start_level: u8,
    pub start_money: u32,
    start_positions: [(f32, f32); 3],
    pub region_size: i32,
    pub cell_size: i32,
    pub move_speed: u8,
    pub speed_scale: f32,
    pub move_tolerance: f32,
    pub save_interval: Duration,
    pub player_limit: usize,
    pub motd: String,
}

impl Settings {
    /// Position de départ par race Rappelz (3 Gaia, 4 Deva, 5 Asura).
    pub fn start_position(&self, race: i32) -> (f32, f32) {
        self.start_positions[(race - 3).clamp(0, 2) as usize]
    }
}

pub struct Ctx {
    pub login: MySqlPool,
    pub chars: MySqlPool,
    pub world_db: MySqlPool,
    pub data: data::GameData,
    pub world: Mutex<world::World>,
    pub settings: Settings,
    pub save_all: broadcast::Sender<()>,
    pub shutdown: watch::Sender<bool>,
    character_guid: AtomicU32,
    item_guid: AtomicU32,
    creature_guid: AtomicU32,
}

impl Ctx {
    pub fn next_character_guid(&self) -> u32 {
        self.character_guid.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn next_item_guid(&self) -> u32 {
        self.item_guid.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn next_creature_guid(&self) -> u32 {
        self.creature_guid.fetch_add(1, Ordering::SeqCst) + 1
    }
}

fn parse_pos(s: &str) -> (f32, f32) {
    let v: Vec<f32> = s.split_whitespace().filter_map(|x| x.parse().ok()).collect();
    (v.first().copied().unwrap_or(0.0), v.get(1).copied().unwrap_or(0.0))
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let conf = Config::load(&cli.config)?;
    rz_common::init_logging(match conf.int("LogLevel", 2u8) {
        0 => "error",
        1 => "warn",
        2 => "info",
        _ => "debug",
    });
    rz_common::print_banner("worldserver");
    rz_proto::packet::set_codepage(&conf.string("Rappelz.Codepage", "windows-1252"));

    let workers = conf.int("WorldDatabase.WorkerThreads", 4u32);
    let login = rz_database::connect(&conf.string("LoginDatabaseInfo", "127.0.0.1;3306;acore;acore;acore_auth"), "Login", workers).await?;
    let world_db = rz_database::connect(&conf.string("WorldDatabaseInfo", "127.0.0.1;3306;acore;acore;acore_world"), "World", workers).await?;
    let chars = rz_database::connect(
        &conf.string("CharacterDatabaseInfo", "127.0.0.1;3306;acore;acore;acore_characters"),
        "Character",
        workers,
    )
    .await?;

    let data_dir = PathBuf::from(conf.string("DataDir", "."));
    tracing::info!("Chargement des données depuis {}", data_dir.display());
    let data = data::GameData::load(&data_dir, &world_db).await?;
    let max_guid = characters::max_guid(&chars).await?;
    let max_item_guid = characters::max_item_guid(&chars).await?;
    let max_creature_guid = rz_database::world::max_creature_guid(&world_db).await?;

    let settings = Settings {
        realm_id: conf.int("RealmID", 1u32),
        network_key: conf.string("Rappelz.NetworkKey", "}h79q~B%al;k'y $E").into_bytes(),
        characters_per_realm: conf.int("CharactersPerRealm", 5usize),
        start_level: conf.int("StartPlayerLevel", 1u8),
        start_money: conf.int("StartPlayerMoney", 0u32),
        start_positions: [
            parse_pos(&conf.string("Rappelz.StartPosition.Gaia", "8330 7167")),
            parse_pos(&conf.string("Rappelz.StartPosition.Deva", "8330 7167")),
            parse_pos(&conf.string("Rappelz.StartPosition.Asura", "8330 7167")),
        ],
        region_size: conf.int("Rappelz.RegionSize", 180i32),
        cell_size: conf.int("Rappelz.CellSize", 6i32),
        move_speed: conf.int("Rappelz.PlayerMoveSpeed", 120u8),
        speed_scale: conf.float("Rappelz.MoveSpeedScale", 1.0),
        move_tolerance: conf.float("Rappelz.MoveTolerance", 60.0),
        save_interval: Duration::from_millis(conf.int("PlayerSaveInterval", 900_000u64).max(10_000)),
        player_limit: conf.int("PlayerLimit", 1000usize),
        motd: conf.string("Motd", "Bienvenue sur RappelzCore."),
    };
    let world = world::World::new(&data, conf.float("Visibility.Distance", 540.0));
    let (save_all, _) = broadcast::channel(4);
    let (shutdown, _) = watch::channel(false);
    let ctx = Arc::new(Ctx {
        login,
        chars,
        world_db,
        data,
        world: Mutex::new(world),
        settings,
        save_all,
        shutdown,
        character_guid: AtomicU32::new(max_guid),
        item_guid: AtomicU32::new(max_item_guid),
        creature_guid: AtomicU32::new(max_creature_guid),
    });

    let realm_id = ctx.settings.realm_id;
    let start_time = chrono::Utc::now().timestamp();
    anyhow::ensure!(realm_id != 0, "RealmID must be positive");

    let bind = format!("{}:{}", conf.string("BindIP", "0.0.0.0"), conf.int("WorldServerPort", 4514u16));
    let listener = TcpListener::bind(&bind).await.with_context(|| format!("écoute sur {bind}"))?;
    // A competing start must fail its bind before it can clear a live world's flags.
    characters::reset_online(&ctx.chars).await?;
    auth::reset_realm_online(&ctx.login, realm_id).await?;
    auth::update_realm_status(&ctx.login, realm_id, true, 0.0).await?;
    tracing::info!("Worldserver prêt sur {bind} (royaume {realm_id}). Tapez « help » pour la console.");

    // Visibilité (ENTER/LEAVE) recalculée en continu.
    let tick_ctx = ctx.clone();
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_millis(200));
        loop {
            tick.tick().await;
            tick_ctx.world.lock().unwrap().tick();
        }
    });

    // Population du royaume et table uptime, comme AzerothCore.
    let status_ctx = ctx.clone();
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(60));
        loop {
            tick.tick().await;
            let (online, max, up) = {
                let w = status_ctx.world.lock().unwrap();
                (w.players.len(), w.max_online, w.uptime_secs())
            };
            let population = online as f32 / status_ctx.settings.player_limit.max(1) as f32;
            let _ = auth::update_realm_status(&status_ctx.login, realm_id, true, population).await;
            let _ = auth::record_uptime(&status_ctx.login, realm_id, start_time, up as u32, max as u16).await;
        }
    });

    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    let mut console_open = true;
    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let (stream, addr) = accepted?;
                let online = ctx.world.lock().unwrap().players.len();
                if online >= ctx.settings.player_limit {
                    tracing::warn!("{addr} refusé : PlayerLimit atteint");
                    continue;
                }
                let (reader, writer) = rz_proto::conn::split(stream, &ctx.settings.network_key);
                tokio::spawn(session::run(ctx.clone(), addr, reader, writer));
            }
            line = stdin.next_line(), if console_open => match line {
                Ok(Some(line)) => match commands::console(ctx.clone(), line.trim()).await {
                    Ok(true) => break,
                    Ok(false) => {}
                    Err(e) => println!("Erreur : {e:#}"),
                },
                _ => console_open = false,
            },
            _ = tokio::signal::ctrl_c() => break,
        }
    }

    tracing::info!("Arrêt : sauvegarde des joueurs…");
    let _ = ctx.shutdown.send(true);
    tokio::time::sleep(Duration::from_secs(2)).await;
    auth::update_realm_status(&ctx.login, realm_id, false, 0.0).await?;
    tracing::info!("Worldserver arrêté.");
    Ok(())
}
