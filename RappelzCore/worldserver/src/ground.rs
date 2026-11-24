//! Objets au sol (docs/re/ground_item_protocol.md) : butin des monstres (`loot.rs`), objets
//! jetés depuis le sac, ramassage (TS_CS_TAKE_ITEM) et disparition.
//!
//! Le client ne retire jamais un objet de lui-même : le serveur envoie TS_SC_TAKE_ITEM_RESULT
//! (animation), puis LEAVE à tous ceux qui le voyaient, puis met à jour le sac du preneur et
//! confirme par TS_SC_RESULT(204, 0, objet).

use std::collections::HashMap;

use anyhow::Result;
use rz_proto::msg::{self, EnterKind, Message, ScEnter};
use rz_proto::msg_game::{self as game, id as gid};
use rz_proto::RawPacket;

use crate::session::Session;
use crate::world::World;

/// Plage de handles des objets au sol (monstres 0x2000…, PNJ 0x4000…, sac 0x6000…).
pub const GROUND_HANDLE_BASE: u32 = 0x5000_0000;
/// Exclusivité du tueur sur son butin, en centièmes de seconde [D] (le client libère
/// l'objet pour le familier après 30 s, docs §ENTER).
pub const OWNER_EXCLUSIVITY: u32 = 30 * 100;
/// Durée de vie d'un objet au sol [D].
pub const LIFETIME: u32 = 180 * 100;
/// Distance de ramassage : le client s'approche à 20 ; marge pour la latence [D].
pub const TAKE_RANGE: f32 = 60.0;
/// Code TS_SC_RESULT d'un ramassage refusé (affiché par le client, chaîne 0xdf) [O code, D sens].
pub const TAKE_REFUSED: u16 = 6;

#[derive(Clone, Debug, PartialEq)]
pub struct GroundItem {
    pub handle: u32,
    /// Code d'objet (Item.dbc) ; 0 = rupees.
    pub code: u32,
    pub count: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub layer: u8,
    /// Joueur ayant droit pendant [`OWNER_EXCLUSIVITY`] (0 = libre).
    pub owner: u32,
    pub dropped_at: u32,
}

impl GroundItem {
    pub fn can_take(&self, taker: u32, now: u32) -> bool {
        self.owner == 0 || self.owner == taker || now.saturating_sub(self.dropped_at) >= OWNER_EXCLUSIVITY
    }
}

#[derive(Default)]
pub struct Ground {
    pub items: HashMap<u32, GroundItem>,
    next: u32,
}

impl Ground {
    fn next_handle(&mut self) -> u32 {
        self.next = self.next.wrapping_add(1) % 0x0fff_ffff;
        GROUND_HANDLE_BASE + self.next.max(1)
    }
}

impl World {
    /// Pose un objet au sol et renvoie son handle ; il apparaît via `update_visibility`.
    pub fn drop_on_ground(&mut self, code: u32, count: u32, (x, y): (f32, f32), layer: u8, owner: u32) -> u32 {
        let now = self.now();
        let handle = self.ground.next_handle();
        self.ground.items.insert(handle, GroundItem { handle, code, count, x, y, z: 0.0, layer, owner, dropped_at: now });
        handle
    }

    pub(crate) fn ground_enter(&self, handle: u32) -> Option<RawPacket> {
        let g = self.ground.items.get(&handle)?;
        let kind = EnterKind::Item {
            code: g.code,
            count: g.count as i64,
            drop_time: g.dropped_at,
            owners: [g.owner, 0, 0],
            parties: [0; 3],
        };
        Some(ScEnter { enter_type: msg::enter_type::STATIC, handle, x: g.x, y: g.y, z: g.z, layer: g.layer, kind }.to_packet())
    }

    /// Butin d'un monstre tué : objets tirés dans `creature_loot_template`, autour du corps.
    pub(crate) fn drop_monster_loot(&mut self, monster: u32, entry: u32, (x, y): (f32, f32), killer: u32) {
        let drops = self.loot.roll(entry, &mut crate::loot::ThreadDice);
        if drops.is_empty() {
            return;
        }
        let mut handles = Vec::new();
        for (i, (code, count)) in drops.into_iter().enumerate() {
            // Petit éventail autour du corps pour que les objets ne se superposent pas.
            let angle = i as f32 * 2.399;
            let pos = (x + 12.0 * angle.cos(), y + 12.0 * angle.sin());
            handles.push(self.drop_on_ground(code, count, pos, 0, killer));
        }
        // Les ENTER partent d'abord (visibilité), puis TS_SC_ITEM_DROP_INFO (monstre, objet) [D].
        self.update_visibility();
        for h in handles {
            self.broadcast_around(h, &game::Pair::<{ gid::TS_SC_ITEM_DROP_INFO }>(monster, h).to_packet(), false);
        }
    }

    /// Retire un objet ramassé : animation (210) et LEAVE pour tous ceux qui le voyaient.
    fn remove_ground(&mut self, handle: u32, taker: u32) -> Option<GroundItem> {
        let g = self.ground.items.remove(&handle)?;
        let anim = game::Pair::<{ gid::TS_SC_TAKE_ITEM_RESULT }>(handle, taker).to_packet();
        let leave = msg::ScLeave { handle }.to_packet();
        for p in self.players.values_mut() {
            if p.visible.remove(&handle) {
                let _ = p.tx.send(anim.clone());
                let _ = p.tx.send(leave.clone());
            }
        }
        Some(g)
    }

    /// Disparition des objets trop anciens.
    pub(crate) fn tick_ground(&mut self, now: u32) {
        let expired: Vec<u32> =
            self.ground.items.values().filter(|g| now.saturating_sub(g.dropped_at) >= LIFETIME).map(|g| g.handle).collect();
        let leave = |h: u32| msg::ScLeave { handle: h }.to_packet();
        for h in expired {
            self.ground.items.remove(&h);
            for p in self.players.values_mut() {
                if p.visible.remove(&h) {
                    let _ = p.tx.send(leave(h));
                }
            }
        }
    }

    /// Vérifie et retire un objet pour `taker` (distance, couche, droits).
    pub fn take_ground(&mut self, taker: u32, handle: u32) -> Option<GroundItem> {
        let now = self.now();
        let p = self.players.get(&taker)?;
        let pos = p.motion.position(now);
        let g = self.ground.items.get(&handle)?;
        let near = ((g.x - pos.0).powi(2) + (g.y - pos.1).powi(2)).sqrt() <= TAKE_RANGE;
        if !near || g.layer != p.layer || !g.can_take(taker, now) {
            return None;
        }
        self.remove_ground(handle, taker)
    }

    pub fn player_spot(&self, handle: u32) -> Option<((f32, f32), u8)> {
        let p = self.players.get(&handle)?;
        Some((p.motion.position(self.now()), p.layer))
    }
}

impl Session {
    /// TS_CS_TAKE_ITEM : ramassage par le joueur (le familier n'est pas encore géré).
    pub async fn take_item(&mut self, req: game::CsTakeItem) -> Result<()> {
        let Some(g) = &self.in_game else { return Ok(()) };
        let me = g.handle;
        if req.taker_handle != me {
            return Ok(());
        }
        let bag_full = g.items.iter().filter(|i| i.in_bag()).count() >= crate::gameplay::BAG_CAPACITY as usize;
        let refuse = |s: &Session| s.send(&msg::ScResult { request: gid::TS_CS_TAKE_ITEM, result: TAKE_REFUSED, value: req.item_handle as i32 });
        let taken = {
            let mut w = self.ctx.world.lock().unwrap();
            let is_gold = w.ground.items.get(&req.item_handle).is_some_and(|i| i.code == 0);
            if bag_full && !is_gold {
                None
            } else {
                w.take_ground(me, req.item_handle)
            }
        };
        let Some(item) = taken else {
            refuse(self);
            return Ok(());
        };
        if item.code == 0 {
            let money = self.in_game.as_ref().map(|g| g.character.money as i64).unwrap_or(0);
            self.set_money(money.saturating_add(item.count as i64));
        } else {
            let given = self.give_item(item.code, item.count).await;
            if !matches!(given, Ok(true)) {
                // Sac plein ou échec SQL : l'objet retourne au sol plutôt que d'être perdu.
                {
                    let mut w = self.ctx.world.lock().unwrap();
                    w.drop_on_ground(item.code, item.count, (item.x, item.y), item.layer, me);
                    w.update_visibility();
                }
                refuse(self);
                given?;
                return Ok(());
            }
        }
        self.send(&msg::ScResult { request: gid::TS_CS_TAKE_ITEM, result: 0, value: req.item_handle as i32 });
        Ok(())
    }

    /// TS_CS_DROP_ITEM : pose au sol tout ou partie d'une pile du sac.
    pub async fn drop_item(&mut self, req: game::CsDropItem) -> Result<()> {
        let ctx = self.ctx.clone();
        if self.pet_of(req.item_handle).is_some() {
            // L'objet au sol change de guid au ramassage : la créature de la carte serait perdue.
            self.send(&game::ScDropResult { item_handle: req.item_handle, is_success: false });
            return Ok(());
        }
        let Some(g) = &mut self.in_game else { return Ok(()) };
        let me = g.handle;
        let owner = g.character.guid;
        let Some(idx) = g.items.iter().position(|i| i.handle() == req.item_handle && i.in_bag()) else {
            self.send(&game::ScDropResult { item_handle: req.item_handle, is_success: false });
            return Ok(());
        };
        let stack = g.items[idx].count;
        let count = if req.count == 0 { stack } else { req.count.min(stack) };
        let (entry, guid) = (g.items[idx].entry, g.items[idx].guid);
        // La base d'abord : en cas d'échec SQL, rien ne quitte le sac ni n'apparaît au sol.
        if count == stack {
            rz_database::characters::delete_item(&ctx.chars, owner, guid).await?;
            self.in_game.as_mut().unwrap().items.remove(idx);
            self.send(&game::Handle::<{ gid::TS_SC_DESTROY_ITEM }>(req.item_handle));
        } else {
            let left = stack - count;
            rz_database::characters::set_item_count(&ctx.chars, guid, left).await?;
            self.in_game.as_mut().unwrap().items[idx].count = left;
            self.send(&game::ScUpdateItemCount { item_handle: req.item_handle, count: left as i64 });
        }
        self.send(&game::ScDropResult { item_handle: req.item_handle, is_success: true });
        {
            let mut w = ctx.world.lock().unwrap();
            if let Some((pos, layer)) = w.player_spot(me) {
                w.drop_on_ground(entry, count, pos, layer, 0);
                w.update_visibility();
            }
        }
        self.refresh_quests().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(owner: u32, dropped_at: u32) -> GroundItem {
        GroundItem { handle: GROUND_HANDLE_BASE + 1, code: 1, count: 1, x: 0.0, y: 0.0, z: 0.0, layer: 0, owner, dropped_at }
    }

    #[test]
    fn ownership_expires_after_exclusivity() {
        assert!(item(0, 0).can_take(9, 0), "libre");
        assert!(item(7, 0).can_take(7, 10), "propriétaire");
        assert!(!item(7, 0).can_take(9, OWNER_EXCLUSIVITY - 1), "exclusif");
        assert!(item(7, 0).can_take(9, OWNER_EXCLUSIVITY), "libéré");
    }

    #[test]
    fn handles_stay_in_their_own_range() {
        let mut g = Ground::default();
        let h = g.next_handle();
        assert!(h > GROUND_HANDLE_BASE && h < 0x6000_0000);
        g.next = 0x0fff_fffe;
        let h = g.next_handle();
        assert!(h > GROUND_HANDLE_BASE && h < 0x6000_0000);
    }
}
