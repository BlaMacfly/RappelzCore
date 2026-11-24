//! Créatures d'invocation dans la table AzerothCore inchangée `character_pet`.
//!
//! Correspondance : `id` = guid de l'objet carte (`item_instance.guid`), `entry` = id de
//! db_creature, `owner` = personnage, `slot` = place dans la formation (0..5) ou
//! [`NOT_IN_FORMATION`] (100, valeur `PET_SAVE_NOT_IN_SLOT` d'AzerothCore).
//!
//! La créature suit sa carte : après un échange, le nouveau propriétaire de l'objet retrouve
//! la ligne par son guid. `characters::delete_item` supprime la ligne avec l'objet.

use anyhow::Result;
use sqlx::{MySqlPool, Row};

pub const NOT_IN_FORMATION: u8 = 100;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pet {
    /// Guid de l'objet carte.
    pub card: u32,
    pub entry: u32,
    pub level: u16,
    pub exp: u32,
    pub name: String,
    pub slot: u8,
}

/// Créature liée à une carte, avec son propriétaire enregistré.
pub async fn by_card(db: &MySqlPool, card: u32) -> Result<Option<(u32, Pet)>> {
    let row = sqlx::query("SELECT owner, entry, level, exp, name, slot FROM character_pet WHERE id = ?")
        .bind(card)
        .fetch_optional(db)
        .await?;
    Ok(row.map(|r| {
        let pet = Pet {
            card,
            entry: r.get("entry"),
            level: r.get("level"),
            exp: r.get("exp"),
            name: r.get("name"),
            slot: r.get("slot"),
        };
        (r.get("owner"), pet)
    }))
}

pub async fn insert(db: &MySqlPool, owner: u32, pet: &Pet) -> Result<()> {
    sqlx::query(
        "INSERT INTO character_pet (id, entry, owner, level, exp, name, slot, curhealth, curmana, savetime, abdata) \
         VALUES (?, ?, ?, ?, ?, ?, ?, 1, 0, UNIX_TIMESTAMP(), '')",
    )
    .bind(pet.card)
    .bind(pet.entry)
    .bind(owner)
    .bind(pet.level)
    .bind(pet.exp)
    .bind(&pet.name)
    .bind(pet.slot)
    .execute(db)
    .await?;
    Ok(())
}

/// La carte a changé de main : nouveau propriétaire, hors formation.
pub async fn transfer(db: &MySqlPool, card: u32, owner: u32) -> Result<()> {
    sqlx::query("UPDATE character_pet SET owner = ?, slot = ? WHERE id = ?")
        .bind(owner)
        .bind(NOT_IN_FORMATION)
        .bind(card)
        .execute(db)
        .await?;
    Ok(())
}

/// Enregistre la formation complète (carte, emplacement) en une transaction.
pub async fn save_formation(db: &MySqlPool, owner: u32, slots: &[(u32, u8)]) -> Result<()> {
    let mut tx = db.begin().await?;
    for (card, slot) in slots {
        sqlx::query("UPDATE character_pet SET slot = ? WHERE id = ? AND owner = ?")
            .bind(slot)
            .bind(card)
            .bind(owner)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}
