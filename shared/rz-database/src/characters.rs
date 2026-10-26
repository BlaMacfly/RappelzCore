//! Requêtes CharacterDatabase (`acore_characters`).
//!
//! Correspondance Rappelz → colonnes AzerothCore de `characters` :
//! race → `race`, sexe → `gender`, classe de base → `class`, niveau → `level`,
//! expérience → `xp`, rupees → `money`, position → `position_x/y/z` + `map` + `orientation`,
//! modèle (peau, visage, cheveux) → `skin`, `face`, `hairStyle`, `hairColor`, `facialStyle`,
//! PV/PM → `health`/`power1`.
//! Ce qui n'a pas d'équivalent WoW (métier exact, niveau de métier, JP, couche, parties
//! du modèle…) est stocké dans `character_settings` avec `source = 'rappelz'`, au format
//! `clé=valeur;clé=valeur`. Le schéma reste ainsi strictement celui d'AzerothCore.

use std::collections::BTreeMap;

use anyhow::Result;
use sqlx::{MySqlPool, Row};

pub const SETTINGS_SOURCE: &str = "rappelz";
/// `characters.name` est un varchar(12) dans AzerothCore.
pub const MAX_NAME_LEN: usize = 12;

#[derive(Clone, Debug, Default)]
pub struct Character {
    pub guid: u32,
    pub account: u32,
    pub name: String,
    pub race: u8,
    pub class: u8,
    pub gender: u8,
    pub level: u8,
    pub xp: u32,
    pub money: u32,
    pub skin: u8,
    pub face: u8,
    pub hair_style: u8,
    pub hair_color: u8,
    pub facial_style: u8,
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
    pub map: u16,
    pub orientation: f32,
    pub zone: u16,
    pub health: u32,
    pub power1: u32,
    pub online: bool,
    pub totaltime: u32,
    pub logout_time: u32,
    /// Données Rappelz sans colonne AzerothCore (stockées dans `character_settings`).
    pub extra: BTreeMap<String, String>,
}

impl Character {
    pub fn extra_i64(&self, key: &str, default: i64) -> i64 {
        self.extra.get(key).and_then(|v| v.parse().ok()).unwrap_or(default)
    }

    pub fn set_extra(&mut self, key: &str, value: impl ToString) {
        self.extra.insert(key.to_string(), value.to_string());
    }
}

fn encode_extra(extra: &BTreeMap<String, String>) -> String {
    extra
        .iter()
        .map(|(k, v)| format!("{k}={}", v.replace(';', ",")))
        .collect::<Vec<_>>()
        .join(";")
}

fn decode_extra(data: &str) -> BTreeMap<String, String> {
    data.split(';')
        .filter_map(|kv| kv.split_once('='))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

const SELECT: &str = "SELECT c.guid, c.account, c.name, c.race, c.class, c.gender, c.level, c.xp, c.money, \
     c.skin, c.face, c.hairStyle, c.hairColor, c.facialStyle, c.position_x, c.position_y, c.position_z, \
     c.map, c.orientation, c.zone, c.health, c.power1, c.online, c.totaltime, c.logout_time, s.data AS extra \
     FROM characters c LEFT JOIN character_settings s ON s.guid = c.guid AND s.source = 'rappelz'";

/// Colonne texte, y compris en collation binaire (`characters.name` est en utf8mb4_bin,
/// que MariaDB annonce comme VARBINARY).
fn text(r: &sqlx::mysql::MySqlRow, col: &str) -> String {
    r.try_get::<String, _>(col)
        .or_else(|_| r.try_get::<Vec<u8>, _>(col).map(|b| String::from_utf8_lossy(&b).into_owned()))
        .unwrap_or_default()
}

fn from_row(r: sqlx::mysql::MySqlRow) -> Character {
    Character {
        guid: r.get("guid"),
        account: r.get("account"),
        name: text(&r, "name"),
        race: r.get("race"),
        class: r.get("class"),
        gender: r.get("gender"),
        level: r.get("level"),
        xp: r.get("xp"),
        money: r.get("money"),
        skin: r.get("skin"),
        face: r.get("face"),
        hair_style: r.get("hairStyle"),
        hair_color: r.get("hairColor"),
        facial_style: r.get("facialStyle"),
        position_x: r.get("position_x"),
        position_y: r.get("position_y"),
        position_z: r.get("position_z"),
        map: r.get("map"),
        orientation: r.get("orientation"),
        zone: r.get("zone"),
        health: r.get("health"),
        power1: r.get("power1"),
        online: r.get::<u8, _>("online") != 0,
        totaltime: r.get("totaltime"),
        logout_time: r.get("logout_time"),
        extra: decode_extra(&text(&r, "extra")),
    }
}

pub async fn list_for_account(db: &MySqlPool, account: u32) -> Result<Vec<Character>> {
    let rows = sqlx::query(&format!("{SELECT} WHERE c.account = ? AND c.deleteDate IS NULL ORDER BY c.guid"))
        .bind(account)
        .fetch_all(db)
        .await?;
    Ok(rows.into_iter().map(from_row).collect())
}

pub async fn by_name(db: &MySqlPool, name: &str) -> Result<Option<Character>> {
    let row = sqlx::query(&format!("{SELECT} WHERE c.name = ?"))
        .bind(name)
        .fetch_optional(db)
        .await?;
    Ok(row.map(from_row))
}

pub async fn by_guid(db: &MySqlPool, guid: u32) -> Result<Option<Character>> {
    let row = sqlx::query(&format!("{SELECT} WHERE c.guid = ?"))
        .bind(guid)
        .fetch_optional(db)
        .await?;
    Ok(row.map(from_row))
}

pub async fn name_exists(db: &MySqlPool, name: &str) -> Result<bool> {
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM characters WHERE name = ?")
        .bind(name)
        .fetch_one(db)
        .await?;
    Ok(n > 0)
}

/// Comme AzerothCore, les guid sont alloués par le serveur à partir de MAX(guid).
pub async fn max_guid(db: &MySqlPool) -> Result<u32> {
    let g: Option<u32> = sqlx::query_scalar("SELECT MAX(guid) FROM characters")
        .fetch_one(db)
        .await?;
    Ok(g.unwrap_or(0))
}

pub async fn max_item_guid(db: &MySqlPool) -> Result<u32> {
    let g: Option<u32> = sqlx::query_scalar("SELECT MAX(guid) FROM item_instance")
        .fetch_one(db)
        .await?;
    Ok(g.unwrap_or(0))
}

pub async fn insert(db: &MySqlPool, c: &Character) -> Result<()> {
    let mut tx = db.begin().await?;
    sqlx::query(
        "INSERT INTO characters (guid, account, name, race, class, gender, level, xp, money, skin, face, \
         hairStyle, hairColor, facialStyle, bankSlots, restState, playerFlags, position_x, position_y, position_z, \
         map, orientation, taximask, online, zone, health, power1, exploredZones, equipmentCache, knownTitles, \
         innTriggerId, creation_date) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 0, 0, ?, ?, ?, ?, ?, '', 0, ?, ?, ?, '', '', '', 0, NOW())",
    )
    .bind(c.guid)
    .bind(c.account)
    .bind(&c.name)
    .bind(c.race)
    .bind(c.class)
    .bind(c.gender)
    .bind(c.level)
    .bind(c.xp)
    .bind(c.money)
    .bind(c.skin)
    .bind(c.face)
    .bind(c.hair_style)
    .bind(c.hair_color)
    .bind(c.facial_style)
    .bind(c.position_x)
    .bind(c.position_y)
    .bind(c.position_z)
    .bind(c.map)
    .bind(c.orientation)
    .bind(c.zone)
    .bind(c.health)
    .bind(c.power1)
    .execute(&mut *tx)
    .await?;
    sqlx::query("REPLACE INTO character_settings (guid, source, data) VALUES (?, ?, ?)")
        .bind(c.guid)
        .bind(SETTINGS_SOURCE)
        .bind(encode_extra(&c.extra))
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

/// Sauvegarde périodique / à la déconnexion (équivalent de `Player::SaveToDB`).
pub async fn save(db: &MySqlPool, c: &Character) -> Result<()> {
    let mut tx = db.begin().await?;
    sqlx::query(
        "UPDATE characters SET level = ?, xp = ?, money = ?, position_x = ?, position_y = ?, position_z = ?, \
         map = ?, orientation = ?, zone = ?, health = ?, power1 = ?, online = ?, totaltime = ?, \
         logout_time = UNIX_TIMESTAMP(), class = ? WHERE guid = ?",
    )
    .bind(c.level)
    .bind(c.xp)
    .bind(c.money)
    .bind(c.position_x)
    .bind(c.position_y)
    .bind(c.position_z)
    .bind(c.map)
    .bind(c.orientation)
    .bind(c.zone)
    .bind(c.health)
    .bind(c.power1)
    .bind(c.online as u8)
    .bind(c.totaltime)
    .bind(c.class)
    .bind(c.guid)
    .execute(&mut *tx)
    .await?;
    sqlx::query("REPLACE INTO character_settings (guid, source, data) VALUES (?, ?, ?)")
        .bind(c.guid)
        .bind(SETTINGS_SOURCE)
        .bind(encode_extra(&c.extra))
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn set_online(db: &MySqlPool, guid: u32, online: bool) -> Result<()> {
    sqlx::query("UPDATE characters SET online = ? WHERE guid = ?")
        .bind(online as u8)
        .bind(guid)
        .execute(db)
        .await?;
    Ok(())
}

/// Au démarrage, AzerothCore remet tous les personnages hors ligne.
pub async fn reset_online(db: &MySqlPool) -> Result<()> {
    sqlx::query("UPDATE characters SET online = 0 WHERE online <> 0").execute(db).await?;
    Ok(())
}

/// Suppression « douce » comme AzerothCore (CharDeleteMethod = 1) : le perso est détaché
/// du compte et conservé avec `deleteInfos_*`.
pub async fn soft_delete(db: &MySqlPool, guid: u32, account: u32) -> Result<bool> {
    let res = sqlx::query(
        "UPDATE characters SET deleteInfos_Name = name, deleteInfos_Account = account, \
         deleteDate = UNIX_TIMESTAMP(), name = '', account = 0 WHERE guid = ? AND account = ?",
    )
    .bind(guid)
    .bind(account)
    .execute(db)
    .await?;
    Ok(res.rows_affected() > 0)
}

#[derive(Clone, Debug)]
pub struct InventoryItem {
    pub item_guid: u32,
    pub entry: u32,
    pub count: u32,
    pub bag: u32,
    pub slot: u8,
    pub durability: u16,
    pub enchantments: String,
}

pub async fn inventory(db: &MySqlPool, guid: u32) -> Result<Vec<InventoryItem>> {
    let rows = sqlx::query(
        "SELECT ci.item, ci.bag, ci.slot, ii.itemEntry, ii.count, ii.durability, ii.enchantments \
         FROM character_inventory ci JOIN item_instance ii ON ii.guid = ci.item WHERE ci.guid = ? ORDER BY ci.bag, ci.slot",
    )
    .bind(guid)
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| InventoryItem {
            item_guid: r.get("item"),
            entry: r.get("itemEntry"),
            count: r.get("count"),
            bag: r.get("bag"),
            slot: r.get("slot"),
            durability: r.get("durability"),
            enchantments: r.get("enchantments"),
        })
        .collect())
}

pub async fn add_item(db: &MySqlPool, owner: u32, item_guid: u32, entry: u32, count: u32, bag: u32, slot: u8) -> Result<()> {
    let mut tx = db.begin().await?;
    sqlx::query(
        "INSERT INTO item_instance (guid, itemEntry, owner_guid, creatorGuid, giftCreatorGuid, count, duration, \
         charges, flags, enchantments, randomPropertyId, durability, playedTime, text) \
         VALUES (?, ?, ?, 0, 0, ?, 0, '', 0, '', 0, 0, 0, '')",
    )
    .bind(item_guid)
    .bind(entry)
    .bind(owner)
    .bind(count)
    .execute(&mut *tx)
    .await?;
    sqlx::query("INSERT INTO character_inventory (guid, bag, slot, item) VALUES (?, ?, ?, ?)")
        .bind(owner)
        .bind(bag)
        .bind(slot)
        .bind(item_guid)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extra_roundtrip() {
        let mut m = BTreeMap::new();
        m.insert("job".to_string(), "101".to_string());
        m.insert("jp".to_string(), "42".to_string());
        assert_eq!(decode_extra(&encode_extra(&m)), m);
    }
}

// ── Inventaire (AzerothCore : character_inventory + item_instance) ──
//
// Convention RappelzCore pour `character_inventory.slot` (bag = 0) : 0..27 = emplacement
// d'équipement Rappelz (wear_position), 28 et plus = sac (index = slot − 28).

pub const BAG_SLOT_BASE: u8 = 28;

pub async fn move_item(db: &MySqlPool, owner: u32, item_guid: u32, slot: u8) -> Result<()> {
    sqlx::query("UPDATE character_inventory SET slot = ? WHERE guid = ? AND item = ?")
        .bind(slot)
        .bind(owner)
        .bind(item_guid)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn set_item_count(db: &MySqlPool, item_guid: u32, count: u32) -> Result<()> {
    sqlx::query("UPDATE item_instance SET count = ? WHERE guid = ?").bind(count).bind(item_guid).execute(db).await?;
    Ok(())
}

pub async fn delete_item(db: &MySqlPool, owner: u32, item_guid: u32) -> Result<()> {
    let mut tx = db.begin().await?;
    sqlx::query("DELETE FROM character_inventory WHERE guid = ? AND item = ?")
        .bind(owner)
        .bind(item_guid)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM item_instance WHERE guid = ?").bind(item_guid).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

/// Objets équipés (wear_position → code d'objet) pour l'écran de sélection.
pub async fn equipped_codes(db: &MySqlPool, guid: u32) -> Result<Vec<(u8, u32)>> {
    let rows = sqlx::query(
        "SELECT ci.slot, ii.itemEntry FROM character_inventory ci JOIN item_instance ii ON ii.guid = ci.item \
         WHERE ci.guid = ? AND ci.bag = 0 AND ci.slot < ?",
    )
    .bind(guid)
    .bind(BAG_SLOT_BASE)
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().map(|r| (r.get("slot"), r.get("itemEntry"))).collect())
}
