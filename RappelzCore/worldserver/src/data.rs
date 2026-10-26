//! Données statiques chargées au démarrage, comme le `ObjectMgr` + `DBCStores` d'AzerothCore :
//! DBC du `DataDir` (Job, NpcResource, Monster…) et tables d'`acore_world`
//! (`creature` + `creature_template` pour les PNJ placés, `game_tele`).

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use rz_database::sqlx::{self, MySqlPool, Row};
use rz_rappelz::dbcstore::{NamedDbc, StringTable};

/// Décalage des entries de PNJ dans `creature_template` (voir rz-dbimport).
pub const NPC_ENTRY_OFFSET: u32 = 1_000_000;

#[derive(Clone, Debug)]
pub struct JobInfo {
    pub id: i32,
    pub name: String,
    pub job_class: i32,
    pub job_depth: i32,
}

#[derive(Clone, Debug)]
pub struct NpcSpawn {
    pub guid: u32,
    pub npc_id: u32,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub orientation: f32,
    pub level: i32,
    pub race: u8,
    pub hp: i32,
    pub mp: i32,
}

#[derive(Clone, Debug)]
pub struct Teleport {
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub struct GameData {
    pub jobs: HashMap<i32, JobInfo>,
    pub npcs: Vec<NpcSpawn>,
    pub teleports: Vec<Teleport>,
    pub strings: StringTable,
    pub monsters: HashMap<u32, MonsterInfo>,
    pub items: HashMap<u32, ItemInfo>,
    /// Articles vendus par entry de PNJ (`npc_vendor`).
    pub vendors: std::sync::RwLock<HashMap<u32, Vec<u32>>>,
    /// Monstres placés (`creature`, entry < NPC_ENTRY_OFFSET) : (guid, id, x, y, z, respawn).
    pub monster_spawns: Vec<(u32, u32, f32, f32, f32, u32)>,
}

/// Statistiques d'un monstre (Monster.dbc).
#[derive(Clone, Debug)]
pub struct MonsterInfo {
    pub id: u32,
    pub name: String,
    pub level: i32,
    pub hp: i32,
    pub mp: i32,
    pub attack: i32,
    pub defence: i32,
    pub exp: i64,
    pub jp: i64,
    pub gold_min: i32,
    pub gold_max: i32,
    pub run_speed: f32,
    pub attack_range: f32,
    pub race: u8,
}

/// Objet (Item.dbc + prix de item_template).
#[derive(Clone, Debug)]
pub struct ItemInfo {
    pub id: u32,
    pub name: String,
    pub wear_type: i32,
    pub buy_price: i64,
    pub sell_price: i64,
    pub endurance: u32,
}

impl GameData {
    pub async fn load(data_dir: &Path, world: &MySqlPool) -> Result<Self> {
        let dbc_dir = data_dir.join("dbc");
        let strings = match StringTable::open(&dbc_dir) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("{e} — lancez « rz-extractor dbc » (textes indisponibles)");
                StringTable::empty()
            }
        };

        let mut jobs = HashMap::new();
        match NamedDbc::open(&dbc_dir, "job") {
            Ok(dbc) => {
                for r in dbc.rows() {
                    jobs.insert(
                        r.id(),
                        JobInfo {
                            id: r.id(),
                            name: strings.get(r.int("text_id")).to_string(),
                            job_class: r.int("job_class"),
                            job_depth: r.int("job_depth"),
                        },
                    );
                }
            }
            Err(e) => tracing::warn!("Job.dbc : {e}"),
        }
        tracing::info!(">> {} métiers chargés (Job.dbc)", jobs.len());

        let npc_res = NamedDbc::open(&dbc_dir, "npcresource").ok();
        let rows = sqlx::query(
            "SELECT c.guid, c.id1, c.position_x, c.position_y, c.position_z, c.orientation, t.name, t.minlevel \
             FROM creature c JOIN creature_template t ON t.entry = c.id1 WHERE c.id1 >= ? AND c.map = 0",
        )
        .bind(NPC_ENTRY_OFFSET)
        .fetch_all(world)
        .await
        .context("lecture de acore_world.creature")?;
        let npcs: Vec<NpcSpawn> = rows
            .into_iter()
            .map(|r| {
                let entry: u32 = r.get("id1");
                let npc_id = entry - NPC_ENTRY_OFFSET;
                let res = npc_res.as_ref().and_then(|d| d.by_id(npc_id as i32));
                NpcSpawn {
                    guid: r.get("guid"),
                    npc_id,
                    name: r.get("name"),
                    x: r.get("position_x"),
                    y: r.get("position_y"),
                    z: r.get("position_z"),
                    orientation: r.get("orientation"),
                    level: r.get::<u8, _>("minlevel") as i32,
                    race: res.map(|r| r.int("race_id") as u8).unwrap_or(0),
                    hp: res.map(|r| r.int("hp")).unwrap_or(100).max(1),
                    mp: res.map(|r| r.int("mp")).unwrap_or(0),
                }
            })
            .collect();
        tracing::info!(">> {} PNJ placés (acore_world.creature)", npcs.len());

        let teleports = sqlx::query("SELECT name, position_x, position_y, position_z FROM game_tele WHERE map = 0")
            .fetch_all(world)
            .await?
            .into_iter()
            .map(|r| Teleport { name: r.get("name"), x: r.get("position_x"), y: r.get("position_y"), z: r.get("position_z") })
            .collect::<Vec<_>>();
        tracing::info!(">> {} destinations de téléportation (game_tele)", teleports.len());

        let mut monsters = HashMap::new();
        match NamedDbc::open(&dbc_dir, "monster") {
            Ok(dbc) => {
                for r in dbc.rows() {
                    monsters.insert(
                        r.id() as u32,
                        MonsterInfo {
                            id: r.id() as u32,
                            name: strings.get(r.int("name_id")).to_string(),
                            level: r.int("level").max(1),
                            hp: r.int("hp").max(1),
                            mp: r.int("mp").max(0),
                            attack: r.int("attack_point").max(1),
                            defence: r.int("defence").max(0),
                            exp: r.int("exp").max(0) as i64,
                            jp: r.int("jp").max(0) as i64,
                            gold_min: r.int("gold_min").max(0),
                            gold_max: r.int("gold_max").max(0),
                            run_speed: (r.int("run_speed").max(1) as f32).min(250.0),
                            attack_range: (r.float("attack_range") * 12.0).clamp(20.0, 80.0),
                            race: r.int("race").clamp(0, 255) as u8,
                        },
                    );
                }
            }
            Err(e) => tracing::warn!("Monster.dbc : {e}"),
        }
        tracing::info!(">> {} monstres (Monster.dbc)", monsters.len());

        let prices: HashMap<u32, (i64, u32)> =
            rz_database::world::item_prices(world).await?.into_iter().map(|(e, b, s)| (e, (b, s))).collect();
        let mut items = HashMap::new();
        if let Ok(dbc) = NamedDbc::open(&dbc_dir, "item") {
            for r in dbc.rows() {
                let id = r.id() as u32;
                let (buy, sell) = prices.get(&id).copied().unwrap_or((r.int("price").max(0) as i64, 0));
                items.insert(
                    id,
                    ItemInfo {
                        id,
                        name: strings.get(r.int("name_id")).to_string(),
                        wear_type: r.int("wear_type"),
                        buy_price: buy,
                        sell_price: sell as i64,
                        endurance: r.int("endurance").max(0) as u32,
                    },
                );
            }
        }
        tracing::info!(">> {} objets (Item.dbc + item_template)", items.len());

        let mut vendors: HashMap<u32, Vec<u32>> = HashMap::new();
        for (entry, item) in rz_database::world::vendors(world).await? {
            vendors.entry(entry).or_default().push(item);
        }
        tracing::info!(">> {} PNJ marchands (npc_vendor)", vendors.len());
        let monster_spawns = rz_database::world::monster_spawns(world, NPC_ENTRY_OFFSET).await?;
        tracing::info!(">> {} monstres placés (acore_world.creature)", monster_spawns.len());

        Ok(Self { jobs, npcs, teleports, strings, monsters, items, vendors: std::sync::RwLock::new(vendors), monster_spawns })
    }

    pub fn find_teleport(&self, name: &str) -> Option<&Teleport> {
        let lower = name.to_lowercase();
        self.teleports
            .iter()
            .find(|t| t.name.to_lowercase() == lower)
            .or_else(|| self.teleports.iter().find(|t| t.name.to_lowercase().starts_with(&lower)))
    }
}
