//! Groupes dans les tables AzerothCore inchangées `groups` et `group_member`.
//!
//! Correspondance : `groups.guid` = identifiant du groupe envoyé au client, `leaderGuid` =
//! chef, `lootMethod` = mode de partage (`/pshare`), `groupType` = 0 (groupe normal). Les
//! autres colonnes WoW restent à 0. Le nom du groupe n'a pas de colonne : il n'est gardé
//! qu'en mémoire et redevient le nom du chef après un redémarrage.

use anyhow::Result;
use sqlx::{MySqlPool, Row};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StoredMember {
    pub guid: u32,
    pub name: String,
    pub level: u8,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StoredGroup {
    pub id: u32,
    pub leader: u32,
    pub share: u8,
    pub members: Vec<StoredMember>,
}

/// Groupes enregistrés avec leurs membres encore existants (personnages supprimés exclus).
pub async fn load_all(db: &MySqlPool) -> Result<Vec<StoredGroup>> {
    let mut groups: Vec<StoredGroup> = sqlx::query("SELECT guid, leaderGuid, lootMethod FROM `groups` ORDER BY guid")
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|r| StoredGroup { id: r.get("guid"), leader: r.get("leaderGuid"), share: r.get("lootMethod"), members: Vec::new() })
        .collect();
    let rows = sqlx::query(
        "SELECT gm.guid, gm.memberGuid, c.name, c.level FROM group_member gm \
         JOIN characters c ON c.guid = gm.memberGuid WHERE c.account <> 0 ORDER BY gm.guid, gm.memberGuid",
    )
    .fetch_all(db)
    .await?;
    for r in rows {
        let id: u32 = r.get("guid");
        if let Some(g) = groups.iter_mut().find(|g| g.id == id) {
            g.members.push(StoredMember { guid: r.get("memberGuid"), name: crate::characters::text(&r, "name"), level: r.get("level") });
        }
    }
    Ok(groups)
}

/// Plus grand identifiant de groupe utilisé.
pub async fn max_id(db: &MySqlPool) -> Result<u32> {
    let g: Option<u32> = sqlx::query_scalar("SELECT MAX(guid) FROM `groups`").fetch_one(db).await?;
    Ok(g.unwrap_or(0))
}

/// Nouveau groupe avec son chef pour seul membre.
pub async fn create(db: &MySqlPool, id: u32, leader: u32, share: u8) -> Result<()> {
    let mut tx = db.begin().await?;
    sqlx::query("DELETE FROM group_member WHERE memberGuid = ?").bind(leader).execute(&mut *tx).await?;
    sqlx::query(
        "INSERT INTO `groups` (guid, leaderGuid, lootMethod, looterGuid, lootThreshold, icon1, icon2, icon3, icon4, \
         icon5, icon6, icon7, icon8, groupType, difficulty, raidDifficulty, masterLooterGuid) \
         VALUES (?, ?, ?, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)",
    )
    .bind(id)
    .bind(leader)
    .bind(share)
    .execute(&mut *tx)
    .await?;
    sqlx::query("INSERT INTO group_member (guid, memberGuid, memberFlags, subgroup, roles) VALUES (?, ?, 0, 0, 0)")
        .bind(id)
        .bind(leader)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn add_member(db: &MySqlPool, id: u32, member: u32) -> Result<()> {
    sqlx::query(
        "INSERT INTO group_member (guid, memberGuid, memberFlags, subgroup, roles) VALUES (?, ?, 0, 0, 0) \
         ON DUPLICATE KEY UPDATE guid = VALUES(guid)",
    )
    .bind(id)
    .bind(member)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn remove_member(db: &MySqlPool, member: u32) -> Result<()> {
    sqlx::query("DELETE FROM group_member WHERE memberGuid = ?").bind(member).execute(db).await?;
    Ok(())
}

pub async fn set_leader(db: &MySqlPool, id: u32, leader: u32) -> Result<()> {
    sqlx::query("UPDATE `groups` SET leaderGuid = ? WHERE guid = ?").bind(leader).bind(id).execute(db).await?;
    Ok(())
}

pub async fn set_share(db: &MySqlPool, id: u32, share: u8) -> Result<()> {
    sqlx::query("UPDATE `groups` SET lootMethod = ? WHERE guid = ?").bind(share).bind(id).execute(db).await?;
    Ok(())
}

/// Dissolution : le groupe et tous ses membres.
pub async fn delete(db: &MySqlPool, id: u32) -> Result<()> {
    let mut tx = db.begin().await?;
    sqlx::query("DELETE FROM group_member WHERE guid = ?").bind(id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM `groups` WHERE guid = ?").bind(id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}
