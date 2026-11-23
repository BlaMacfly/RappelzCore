//! Entrepôt (docs/re/storage_trade_protocol.md) : ouverture par le trigger `open_storage()` chez un
//! PNJ `NPC_FLAG_BANKER`, contenu en TS_SC_INVENTORY avec `wear_position` -2, TS_CS_STORAGE pour
//! déposer / retirer objets et rupees. Objets : `character_inventory.slot` 150..249 ; rupees :
//! extra `storage_gold` du personnage (propriété client `storage_gold`).

use anyhow::Result;
use rz_database::characters;
use rz_proto::msg;
use rz_proto::msg_game::{self as game, storage_mode};

use crate::gameplay::{InvItem, BAG_CAPACITY, STORAGE_CAPACITY, STORAGE_SLOT_BASE};
use crate::session::Session;
use rz_database::characters::BAG_SLOT_BASE;

/// Trigger de menu PNJ reconnu par le client (préfixe) [O].
pub const OPEN_TRIGGER: &str = "open_storage()";

fn free_slot(items: &[InvItem], base: u8, capacity: u8) -> Option<u8> {
    (base..base.saturating_add(capacity)).find(|s| !items.iter().any(|i| i.slot == *s))
}

/// Quantité effectivement déplacée : 0 ou négatif = toute la pile, plafonnée à la pile.
fn moved_count(requested: i64, stack: u32) -> u32 {
    if requested <= 0 { stack } else { requested.min(stack as i64) as u32 }
}

impl Session {
    fn storage_gold(&self) -> i64 {
        self.in_game.as_ref().map(|g| g.character.extra_i64("storage_gold", 0)).unwrap_or(0)
    }

    fn send_storage_gold(&self) {
        let Some(g) = &self.in_game else { return };
        self.send(&msg::ScProperty { handle: g.handle, name: "storage_gold".into(), value: self.storage_gold(), text: None });
    }

    /// `open_storage()` choisi dans le menu d'un PNJ : TS_SC_OPEN_STORAGE puis contenu et rupees.
    pub fn open_storage(&mut self, npc_handle: u32) {
        let Some((npc_id, _)) = self.npc_at(npc_handle) else { return };
        if !self.ctx.data.is_banker(npc_id) {
            return;
        }
        self.storage_npc = Some(npc_handle);
        self.dialog_npc = None;
        self.send(&game::ScOpenStorage(npc_handle));
        let Some(g) = &self.in_game else { return };
        let stored: Vec<game::ItemInfo> = g.items.iter().filter(|i| i.stored()).map(|i| self.item_info(i)).collect();
        for chunk in stored.chunks(150) {
            self.send(&game::ScInventory { items: chunk.to_vec() });
        }
        self.send_storage_gold();
    }

    /// L'entrepôt est-il ouvert, et son PNJ toujours à portée ?
    fn storage_usable(&mut self) -> bool {
        match self.storage_npc {
            Some(npc) if self.npc_at(npc).is_some() => true,
            _ => {
                self.storage_npc = None;
                false
            }
        }
    }

    pub async fn handle_storage(&mut self, req: game::CsStorage) -> Result<()> {
        if req.mode == storage_mode::CLOSE {
            self.storage_npc = None;
            return Ok(());
        }
        if !self.storage_usable() {
            return Ok(());
        }
        match req.mode {
            storage_mode::DEPOSIT_ITEM => self.storage_move(req.item_handle, req.count, true).await?,
            storage_mode::WITHDRAW_ITEM => self.storage_move(req.item_handle, req.count, false).await?,
            storage_mode::DEPOSIT_GOLD | storage_mode::WITHDRAW_GOLD => {
                self.storage_gold_move(req.count, req.mode == storage_mode::DEPOSIT_GOLD)
            }
            _ => {}
        }
        Ok(())
    }

    /// Dépôt (`deposit`) ou retrait d'une pile, entière ou partielle.
    async fn storage_move(&mut self, handle: u32, count: i64, deposit: bool) -> Result<()> {
        let ctx = self.ctx.clone();
        let Some(g) = &mut self.in_game else { return Ok(()) };
        let owner = g.character.guid;
        let player = g.handle;
        let Some(idx) = g.items.iter().position(|i| i.handle() == handle && if deposit { i.in_bag() } else { i.stored() }) else {
            return Ok(());
        };
        let (base, capacity) = if deposit { (STORAGE_SLOT_BASE, STORAGE_CAPACITY) } else { (BAG_SLOT_BASE, BAG_CAPACITY) };
        let Some(slot) = free_slot(&g.items, base, capacity) else {
            let text = if deposit { "L'entrepôt est plein." } else { "Le sac est plein." };
            ctx.world.lock().unwrap().notify(player, text);
            return Ok(());
        };
        let moved = moved_count(count, g.items[idx].count);
        if moved == g.items[idx].count {
            // Pile entière : même objet, nouvel emplacement. Le client retire l'ancien
            // exemplaire avant de recevoir le nouveau (handle conservé).
            g.items[idx].slot = slot;
            let it = g.items[idx].clone();
            characters::move_item(&ctx.chars, owner, it.guid, slot).await?;
            self.send(&game::Handle::<{ game::id::TS_SC_DESTROY_ITEM }>(handle));
            let info = self.item_info(&it);
            self.send(&game::ScInventory { items: vec![info] });
        } else {
            let left = g.items[idx].count - moved;
            g.items[idx].count = left;
            let entry = g.items[idx].entry;
            characters::set_item_count(&ctx.chars, g.items[idx].guid, left).await?;
            let guid = ctx.next_item_guid();
            characters::add_item(&ctx.chars, owner, guid, entry, moved, 0, slot).await?;
            let it = InvItem { guid, entry, count: moved, slot };
            g.items.push(it.clone());
            self.send(&game::ScUpdateItemCount { item_handle: handle, count: left as i64 });
            let info = self.item_info(&it);
            self.send(&game::ScInventory { items: vec![info] });
        }
        self.refresh_quests().await
    }

    fn storage_gold_move(&mut self, amount: i64, deposit: bool) {
        let Some(g) = &mut self.in_game else { return };
        let money = g.character.money as i64;
        let stored = g.character.extra_i64("storage_gold", 0);
        let amount = if deposit {
            amount.min(money).min(u32::MAX as i64 - stored)
        } else {
            amount.min(stored).min(u32::MAX as i64 - money)
        };
        if amount <= 0 {
            return;
        }
        let (money, stored) = if deposit { (money - amount, stored + amount) } else { (money + amount, stored - amount) };
        g.character.set_extra("storage_gold", stored);
        // Sauvegardé avec le personnage (périodique / déconnexion), comme les achats.
        self.set_money(money);
        self.send_storage_gold();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_and_slots() {
        assert_eq!(moved_count(0, 7), 7);
        assert_eq!(moved_count(3, 7), 3);
        assert_eq!(moved_count(99, 7), 7);
        let items = vec![InvItem { guid: 1, entry: 1, count: 1, slot: STORAGE_SLOT_BASE }];
        assert_eq!(free_slot(&items, STORAGE_SLOT_BASE, STORAGE_CAPACITY), Some(STORAGE_SLOT_BASE + 1));
        let stored = InvItem { guid: 2, entry: 1, count: 1, slot: STORAGE_SLOT_BASE + 5 };
        assert!(stored.stored() && !stored.in_bag() && !stored.worn());
        let bag = InvItem { guid: 3, entry: 1, count: 1, slot: BAG_SLOT_BASE };
        assert!(bag.in_bag());
        assert!(STORAGE_SLOT_BASE as u16 + STORAGE_CAPACITY as u16 <= 256);
        assert!(BAG_SLOT_BASE as u16 + BAG_CAPACITY as u16 <= STORAGE_SLOT_BASE as u16);
    }
}
