//! Échange entre deux personnages (TS_TRADE, docs/re/storage_trade_protocol.md), exécuté en une
//! seule transaction sur les tables AzerothCore `item_instance`, `character_inventory` et
//! `characters`. Toute incohérence (objet déplacé, quantité insuffisante, sac plein) annule tout.

use std::ops::Range;

use anyhow::Result;
use sqlx::{MySql, MySqlPool, Row, Transaction};

/// Objet proposé : `count` exemplaires de l'objet `guid` (pile entière ou partielle).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Offer {
    pub guid: u32,
    pub count: u32,
}

/// Objet reçu par un personnage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Received {
    pub guid: u32,
    pub entry: u32,
    pub count: u32,
    pub slot: u8,
}

/// Effets de l'échange pour un personnage.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Side {
    /// (guid, exemplaires restants ; 0 = l'objet a quitté l'inventaire).
    pub lost: Vec<(u32, u32)>,
    pub gained: Vec<Received>,
}

/// Une partie de l'échange.
pub struct Party<'a> {
    pub guid: u32,
    pub offer: &'a [Offer],
    pub gold: i64,
}

struct Held {
    entry: u32,
    count: u32,
}

async fn held(tx: &mut Transaction<'_, MySql>, owner: u32, offer: &Offer, bag: &Range<u8>) -> Result<Option<Held>> {
    let row = sqlx::query(
        "SELECT ii.itemEntry, ii.count, ci.slot FROM item_instance ii JOIN character_inventory ci ON ci.item = ii.guid \
         WHERE ii.guid = ? AND ci.guid = ? AND ii.owner_guid = ? AND ci.bag = 0 FOR UPDATE",
    )
    .bind(offer.guid)
    .bind(owner)
    .bind(owner)
    .fetch_optional(&mut **tx)
    .await?;
    Ok(row.and_then(|r| {
        let slot: u8 = r.get("slot");
        let count: u32 = r.get("count");
        (bag.contains(&slot) && offer.count > 0 && count >= offer.count).then(|| Held { entry: r.get("itemEntry"), count })
    }))
}

async fn free_slots(tx: &mut Transaction<'_, MySql>, owner: u32, leaving: &[u32], bag: &Range<u8>) -> Result<Vec<u8>> {
    let rows = sqlx::query("SELECT slot, item FROM character_inventory WHERE guid = ? AND bag = 0 FOR UPDATE")
        .bind(owner)
        .fetch_all(&mut **tx)
        .await?;
    let used: Vec<u8> = rows
        .iter()
        .filter(|r| !leaving.contains(&r.get::<u32, _>("item")))
        .map(|r| r.get("slot"))
        .collect();
    Ok(bag.clone().filter(|s| !used.contains(s)).collect())
}

/// Déplace les objets de `from` vers `to` ; `slots` = emplacements libres du destinataire.
async fn transfer(
    tx: &mut Transaction<'_, MySql>,
    from: &Party<'_>,
    to: u32,
    stacks: &[Held],
    slots: &mut std::vec::IntoIter<u8>,
    next_guid: &mut (dyn FnMut() -> u32 + Send),
    out_from: &mut Side,
    out_to: &mut Side,
) -> Result<bool> {
    for (offer, h) in from.offer.iter().zip(stacks) {
        let Some(slot) = slots.next() else { return Ok(false) };
        if offer.count == h.count {
            sqlx::query("UPDATE item_instance SET owner_guid = ? WHERE guid = ?").bind(to).bind(offer.guid).execute(&mut **tx).await?;
            sqlx::query("UPDATE character_inventory SET guid = ?, slot = ? WHERE item = ?")
                .bind(to)
                .bind(slot)
                .bind(offer.guid)
                .execute(&mut **tx)
                .await?;
            out_from.lost.push((offer.guid, 0));
            out_to.gained.push(Received { guid: offer.guid, entry: h.entry, count: h.count, slot });
        } else {
            let left = h.count - offer.count;
            sqlx::query("UPDATE item_instance SET count = ? WHERE guid = ?").bind(left).bind(offer.guid).execute(&mut **tx).await?;
            let guid = next_guid();
            sqlx::query(
                "INSERT INTO item_instance (guid, itemEntry, owner_guid, creatorGuid, giftCreatorGuid, count, duration, \
                 charges, flags, enchantments, randomPropertyId, durability, playedTime, text) \
                 VALUES (?, ?, ?, 0, 0, ?, 0, '', 0, '', 0, 0, 0, '')",
            )
            .bind(guid)
            .bind(h.entry)
            .bind(to)
            .bind(offer.count)
            .execute(&mut **tx)
            .await?;
            sqlx::query("INSERT INTO character_inventory (guid, bag, slot, item) VALUES (?, 0, ?, ?)")
                .bind(to)
                .bind(slot)
                .bind(guid)
                .execute(&mut **tx)
                .await?;
            out_from.lost.push((offer.guid, left));
            out_to.gained.push(Received { guid, entry: h.entry, count: offer.count, slot });
        }
    }
    Ok(true)
}

async fn move_gold(tx: &mut Transaction<'_, MySql>, from: u32, to: u32, gold: i64) -> Result<()> {
    if gold <= 0 {
        return Ok(());
    }
    // L'or vivant est celui des sessions (sauvegardé en absolu) ; on garde la base cohérente
    // en cas d'arrêt avant la prochaine sauvegarde.
    sqlx::query("UPDATE characters SET money = IF(money >= ?, money - ?, 0) WHERE guid = ?")
        .bind(gold)
        .bind(gold)
        .bind(from)
        .execute(&mut **tx)
        .await?;
    sqlx::query("UPDATE characters SET money = LEAST(money + ?, 4294967295) WHERE guid = ?")
        .bind(gold)
        .bind(to)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

/// Exécute l'échange ; `None` (rien n'est modifié) si un objet n'est plus détenu dans le sac
/// en quantité suffisante, ou si un sac n'a pas assez de place.
pub async fn exchange(
    db: &MySqlPool,
    a: Party<'_>,
    b: Party<'_>,
    bag: Range<u8>,
    next_guid: &mut (dyn FnMut() -> u32 + Send),
) -> Result<Option<(Side, Side)>> {
    let mut tx = db.begin().await?;
    let mut stacks = (Vec::new(), Vec::new());
    for (party, out) in [(&a, &mut stacks.0), (&b, &mut stacks.1)] {
        for offer in party.offer {
            match held(&mut tx, party.guid, offer, &bag).await? {
                Some(h) => out.push(h),
                None => return Ok(None),
            }
        }
    }
    // Les piles cédées entièrement libèrent leur emplacement chez le donneur. A → B est
    // transféré d'abord : les piles de B sont encore en place (unicité guid/bag/slot), seules
    // celles de A, déjà parties, peuvent accueillir les objets de B.
    let leaving_a: Vec<u32> =
        a.offer.iter().zip(&stacks.0).filter(|(o, h)| o.count == h.count).map(|(o, _)| o.guid).collect();
    let free_a = free_slots(&mut tx, a.guid, &leaving_a, &bag).await?;
    let free_b = free_slots(&mut tx, b.guid, &[], &bag).await?;
    if free_a.len() < b.offer.len() || free_b.len() < a.offer.len() {
        return Ok(None);
    }
    let (mut side_a, mut side_b) = (Side::default(), Side::default());
    let mut slots_b = free_b.into_iter();
    let mut slots_a = free_a.into_iter();
    if !transfer(&mut tx, &a, b.guid, &stacks.0, &mut slots_b, next_guid, &mut side_a, &mut side_b).await?
        || !transfer(&mut tx, &b, a.guid, &stacks.1, &mut slots_a, next_guid, &mut side_b, &mut side_a).await?
    {
        return Ok(None);
    }
    move_gold(&mut tx, a.guid, b.guid, a.gold).await?;
    move_gold(&mut tx, b.guid, a.guid, b.gold).await?;
    tx.commit().await?;
    Ok(Some((side_a, side_b)))
}
