//! Quêtes du personnage, dans les tables AzerothCore inchangées :
//! `character_queststatus` (quêtes en cours) et `character_queststatus_rewarded` (quêtes rendues).
//!
//! Correspondance Rappelz → AzerothCore (docs/re/quest_protocol.md) :
//! - `status` : état AzerothCore (`QUEST_STATUS_INCOMPLETE` 3, `QUEST_STATUS_COMPLETE` 1) ;
//! - `timer` : échéance absolue (secondes Unix) des quêtes à durée limitée, 0 sinon ;
//! - `mobcount1..4`, `itemcount1..2` : les six compteurs de progression Rappelz (`status[6]`) ;
//! - `itemcount3..6`, `playercount` : inutilisés (0).

use anyhow::Result;
use sqlx::{MySqlPool, Row};

/// `QUEST_STATUS_COMPLETE` d'AzerothCore : objectifs remplis, quête à rendre.
pub const STATUS_COMPLETE: u8 = 1;
/// `QUEST_STATUS_INCOMPLETE` d'AzerothCore : quête en cours.
pub const STATUS_INCOMPLETE: u8 = 3;

/// Nombre de compteurs de progression d'une quête Rappelz.
pub const PROGRESS_SLOTS: usize = 6;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct QuestStatus {
    pub quest: u32,
    pub status: u8,
    /// Échéance (secondes Unix) ou 0.
    pub timer: u32,
    pub counts: [u16; PROGRESS_SLOTS],
}

/// Quêtes en cours du personnage.
pub async fn active(db: &MySqlPool, guid: u32) -> Result<Vec<QuestStatus>> {
    let rows = sqlx::query(
        "SELECT quest, status, timer, mobcount1, mobcount2, mobcount3, mobcount4, itemcount1, itemcount2 \
         FROM character_queststatus WHERE guid = ? ORDER BY quest",
    )
    .bind(guid)
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| QuestStatus {
            quest: r.get("quest"),
            status: r.get("status"),
            timer: r.get("timer"),
            counts: [
                r.get("mobcount1"),
                r.get("mobcount2"),
                r.get("mobcount3"),
                r.get("mobcount4"),
                r.get("itemcount1"),
                r.get("itemcount2"),
            ],
        })
        .collect())
}

/// Quêtes déjà rendues (non répétables : on ne les propose plus).
pub async fn rewarded(db: &MySqlPool, guid: u32) -> Result<Vec<u32>> {
    let rows = sqlx::query("SELECT quest FROM character_queststatus_rewarded WHERE guid = ? AND active = 1")
        .bind(guid)
        .fetch_all(db)
        .await?;
    Ok(rows.into_iter().map(|r| r.get("quest")).collect())
}

/// Crée ou met à jour une quête en cours.
pub async fn upsert(db: &MySqlPool, guid: u32, q: &QuestStatus) -> Result<()> {
    sqlx::query(
        "INSERT INTO character_queststatus (guid, quest, status, explored, timer, mobcount1, mobcount2, mobcount3, \
         mobcount4, itemcount1, itemcount2, itemcount3, itemcount4, itemcount5, itemcount6, playercount) \
         VALUES (?, ?, ?, 0, ?, ?, ?, ?, ?, ?, ?, 0, 0, 0, 0, 0) \
         ON DUPLICATE KEY UPDATE status = VALUES(status), timer = VALUES(timer), mobcount1 = VALUES(mobcount1), \
         mobcount2 = VALUES(mobcount2), mobcount3 = VALUES(mobcount3), mobcount4 = VALUES(mobcount4), \
         itemcount1 = VALUES(itemcount1), itemcount2 = VALUES(itemcount2)",
    )
    .bind(guid)
    .bind(q.quest)
    .bind(q.status)
    .bind(q.timer)
    .bind(q.counts[0])
    .bind(q.counts[1])
    .bind(q.counts[2])
    .bind(q.counts[3])
    .bind(q.counts[4])
    .bind(q.counts[5])
    .execute(db)
    .await?;
    Ok(())
}

/// Abandon : la quête en cours disparaît.
pub async fn remove(db: &MySqlPool, guid: u32, quest: u32) -> Result<()> {
    sqlx::query("DELETE FROM character_queststatus WHERE guid = ? AND quest = ?")
        .bind(guid)
        .bind(quest)
        .execute(db)
        .await?;
    Ok(())
}

/// Quête rendue : retirée des quêtes en cours, ajoutée aux quêtes rendues et personnage récompensé
/// (expérience, JP, rupees) sauvegardé, le tout dans une seule transaction.
pub async fn complete(db: &MySqlPool, character: &crate::characters::Character, quest: u32) -> Result<()> {
    let guid = character.guid;
    let mut tx = db.begin().await?;
    crate::characters::save_in(&mut tx, character).await?;
    sqlx::query("DELETE FROM character_queststatus WHERE guid = ? AND quest = ?")
        .bind(guid)
        .bind(quest)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO character_queststatus_rewarded (guid, quest, active) VALUES (?, ?, 1) \
         ON DUPLICATE KEY UPDATE active = 1",
    )
    .bind(guid)
    .bind(quest)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}
