//! Accès aux trois bases d'AzerothCore : `acore_auth` (LoginDatabase),
//! `acore_characters` (CharacterDatabase) et `acore_world` (WorldDatabase).
//!
//! Les requêtes n'utilisent que des tables et colonnes du schéma AzerothCore
//! (voir `sql/base/`) ; les données propres à Rappelz sans équivalent WoW vont dans
//! les tables génériques prévues par AzerothCore (`character_settings`, etc.).

pub mod auth;
pub mod characters;
pub mod srp6;

use anyhow::{Context, Result};
use rz_common::config::DatabaseInfo;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};

pub use sqlx;

pub async fn connect(info: &str, name: &str, max_connections: u32) -> Result<MySqlPool> {
    let info = DatabaseInfo::parse(info).with_context(|| format!("{name}DatabaseInfo"))?;
    let pool = MySqlPoolOptions::new()
        .max_connections(max_connections.max(1))
        .connect(&info.url())
        .await
        .with_context(|| {
            format!(
                "connexion à la base {name} ({}@{}:{}/{})",
                info.user, info.host, info.port, info.database
            )
        })?;
    tracing::info!("Base {name} connectée : {}:{}/{}", info.host, info.port, info.database);
    Ok(pool)
}

pub mod world {
    //! Requêtes WorldDatabase (`acore_world`) utilisées en jeu.
    use anyhow::Result;
    use sqlx::{MySqlPool, Row};

    /// Articles vendus par PNJ (`npc_vendor`, comme AzerothCore) : (entry, item).
    pub async fn vendors(db: &MySqlPool) -> Result<Vec<(u32, u32)>> {
        let rows = sqlx::query("SELECT entry, item FROM npc_vendor ORDER BY entry, slot").fetch_all(db).await?;
        Ok(rows.into_iter().map(|r| (r.get::<u32, _>("entry"), r.get::<i32, _>("item") as u32)).collect())
    }

    /// Prix d'achat / de vente (`item_template.BuyPrice` / `SellPrice`).
    pub async fn item_prices(db: &MySqlPool) -> Result<Vec<(u32, i64, u32)>> {
        let rows = sqlx::query("SELECT entry, BuyPrice, SellPrice FROM item_template").fetch_all(db).await?;
        Ok(rows.into_iter().map(|r| (r.get("entry"), r.get("BuyPrice"), r.get("SellPrice"))).collect())
    }

    /// Ajoute un article à un PNJ (commande GM `.vendor add`).
    pub async fn add_vendor_item(db: &MySqlPool, entry: u32, item: u32) -> Result<()> {
        let slot: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM npc_vendor WHERE entry = ?").bind(entry).fetch_one(db).await?;
        sqlx::query("INSERT IGNORE INTO npc_vendor (entry, slot, item, maxcount, incrtime, ExtendedCost, VerifiedBuild) VALUES (?, ?, ?, 0, 0, 0, 0)")
            .bind(entry)
            .bind(slot as u16)
            .bind(item)
            .execute(db)
            .await?;
        Ok(())
    }

    /// Point d'apparition de monstre (`creature`, comme `.npc add` d'AzerothCore).
    pub async fn add_creature_spawn(db: &MySqlPool, guid: u32, entry: u32, x: f32, y: f32, z: f32, respawn: u32) -> Result<()> {
        sqlx::query(
            "INSERT INTO creature (guid, id1, map, position_x, position_y, position_z, orientation, spawntimesecs, \
             wander_distance, MovementType, VerifiedBuild, Comment) VALUES (?, ?, 0, ?, ?, ?, 0, ?, 0, 0, 0, 'RappelzCore .spawn')",
        )
        .bind(guid)
        .bind(entry)
        .bind(x)
        .bind(y)
        .bind(z)
        .bind(respawn)
        .execute(db)
        .await?;
        Ok(())
    }

    /// Monstres placés : (guid, entry, x, y, z, spawntimesecs).
    pub async fn monster_spawns(db: &MySqlPool, npc_offset: u32) -> Result<Vec<(u32, u32, f32, f32, f32, u32)>> {
        let rows = sqlx::query(
            "SELECT guid, id1, position_x, position_y, position_z, spawntimesecs FROM creature WHERE id1 < ? AND map = 0",
        )
        .bind(npc_offset)
        .fetch_all(db)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| (r.get("guid"), r.get("id1"), r.get("position_x"), r.get("position_y"), r.get("position_z"), r.get("spawntimesecs")))
            .collect())
    }

    pub async fn max_creature_guid(db: &MySqlPool) -> Result<u32> {
        let g: Option<u32> = sqlx::query_scalar("SELECT MAX(guid) FROM creature").fetch_one(db).await?;
        Ok(g.unwrap_or(0))
    }
}
