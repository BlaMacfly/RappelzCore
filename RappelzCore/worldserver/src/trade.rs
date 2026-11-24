//! Échange entre joueurs (TS_TRADE 280, docs/re/storage_trade_protocol.md).
//!
//! La négociation (demande, objets, rupees, verrou, confirmation) vit dans l'état du monde ;
//! l'exécution est une transaction SQL unique (`rz_database::trade::exchange`) qui revérifie
//! chaque objet, puis chaque session applique sa part via `WorldEvent::TradeApplied`.
//! Pendant un échange, les actions qui modifient le sac ou les rupees sont refusées.

use std::collections::HashMap;

use anyhow::Result;
use rz_database::characters::BAG_SLOT_BASE;
use rz_database::trade::{self as db_trade, Offer, Party, Side};
use rz_proto::msg::Message;
use rz_proto::msg_game::{self as game, trade_mode as mode};

use crate::gameplay::{InvItem, BAG_CAPACITY};
use crate::session::Session;
use crate::world::{World, WorldEvent};

/// Objets proposables par joueur (fenêtre d'échange du client) [O].
pub const MAX_ITEMS: usize = 10;

#[derive(Clone, Debug)]
pub struct Offered {
    pub guid: u32,
    pub count: u32,
    pub info: game::ItemInfo,
}

#[derive(Clone, Debug)]
pub struct TradeSide {
    pub partner: u32,
    /// Guid du personnage (pour la transaction SQL).
    pub character: u32,
    pub items: Vec<Offered>,
    pub gold: i64,
    pub frozen: bool,
    pub confirmed: bool,
    pub processing: bool,
}

impl TradeSide {
    fn new(partner: u32, character: u32) -> Self {
        Self { partner, character, items: Vec::new(), gold: 0, frozen: false, confirmed: false, processing: false }
    }
}

#[derive(Default)]
pub struct Trades {
    /// Demande en attente : cible → demandeur.
    pub requests: HashMap<u32, u32>,
    pub open: HashMap<u32, TradeSide>,
}

impl Trades {
    pub fn busy(&self, handle: u32) -> bool {
        self.open.contains_key(&handle)
    }
}

fn packet(handle: u32, m: u8, item: game::ItemInfo) -> rz_proto::RawPacket {
    game::Trade { handle, mode: m, item }.to_packet()
}

impl World {
    /// Envoie la même action aux deux participants (`handle` = auteur).
    fn trade_broadcast(&self, author: u32, partner: u32, m: u8, item: game::ItemInfo) {
        let p = packet(author, m, item);
        self.send_to(author, p.clone());
        self.send_to(partner, p);
    }

    /// Annule l'échange (ou la demande) de `handle` et prévient les deux joueurs.
    pub fn cancel_trade(&mut self, handle: u32) {
        self.trades.requests.retain(|target, requester| *target != handle && *requester != handle);
        if let Some(side) = self.trades.open.remove(&handle) {
            self.trades.open.remove(&side.partner);
            self.send_to(handle, packet(side.partner, mode::CANCEL, Default::default()));
            self.send_to(side.partner, packet(handle, mode::CANCEL, Default::default()));
        }
    }

    pub fn trade_busy(&self, handle: u32) -> bool {
        self.trades.busy(handle)
    }
}

/// Après l'échange : objets perdus/gagnés et variation de rupees d'un joueur.
fn applied(side: Side, gold_delta: i64) -> WorldEvent {
    WorldEvent::TradeApplied { lost: side.lost, gained: side.gained, gold_delta }
}

impl Session {
    pub async fn handle_trade(&mut self, req: game::Trade) -> Result<()> {
        let Some(g) = &self.in_game else { return Ok(()) };
        let me = g.handle;
        let character = g.character.guid;
        let money = g.character.money as i64;
        // Objet désigné par le client (modes 5, 10, 11) : il doit être dans le sac.
        // … et une créature invoquée retient sa carte.
        let mine =
            g.items.iter().find(|i| i.handle() == req.item.handle && i.in_bag()).cloned().filter(|i| !self.is_summoned(i.handle()));
        let info = mine.as_ref().map(|i| self.item_info(i));
        let ctx = self.ctx.clone();
        let mut exec = None;
        {
        let mut w = ctx.world.lock().unwrap();
        let now = w.now();
        match req.mode {
            mode::REQUEST => {
                let target = req.handle;
                let ok = target != me
                    && w.players.contains_key(&target)
                    && w.can_see(me, target, now)
                    && !w.trades.busy(me)
                    && !w.trades.busy(target);
                if ok {
                    w.trades.requests.insert(target, me);
                    w.send_to(target, packet(me, mode::REQUEST, Default::default()));
                } else {
                    w.notify(me, "Échange impossible avec ce joueur.");
                }
            }
            mode::ACCEPT => {
                let requester = req.handle;
                if w.trades.requests.get(&me) != Some(&requester) {
                    return Ok(());
                }
                w.trades.requests.remove(&me);
                let Some(other) = w.players.get(&requester).map(|p| p.guid) else { return Ok(()) };
                if w.trades.busy(me) || w.trades.busy(requester) {
                    return Ok(());
                }
                w.trades.open.insert(me, TradeSide::new(requester, character));
                w.trades.open.insert(requester, TradeSide::new(me, other));
                w.send_to(me, packet(requester, mode::BEGIN, Default::default()));
                w.send_to(requester, packet(me, mode::BEGIN, Default::default()));
            }
            mode::REJECT => {
                let requester = req.handle;
                if w.trades.requests.get(&me) == Some(&requester) {
                    w.trades.requests.remove(&me);
                    w.send_to(requester, packet(me, mode::REJECT, Default::default()));
                }
            }
            // Un 3 tardif (échange déjà fermé) est ignoré.
            mode::CANCEL => w.cancel_trade(me),
            mode::ADD_ITEM | mode::REMOVE_ITEM | mode::ITEM_COUNT | mode::ADD_GOLD => {
                let Some(side) = w.trades.open.get_mut(&me) else { return Ok(()) };
                if side.frozen {
                    return Ok(());
                }
                let partner = side.partner;
                let mut item = game::ItemInfo::default();
                match req.mode {
                    mode::ADD_GOLD => {
                        side.gold = req.item.count.clamp(0, money);
                        item.count = side.gold;
                    }
                    mode::REMOVE_ITEM => {
                        let Some(pos) = side.items.iter().position(|o| o.info.handle == req.item.handle) else { return Ok(()) };
                        item = side.items.remove(pos).info;
                    }
                    _ => {
                        let (Some(it), Some(mut i)) = (mine, info) else { return Ok(()) };
                        let count = (req.item.count.max(1) as u64).min(it.count as u64) as u32;
                        i.count = count as i64;
                        let full = side.items.len() >= MAX_ITEMS;
                        match side.items.iter().position(|o| o.guid == it.guid) {
                            Some(pos) if req.mode == mode::ITEM_COUNT => {
                                side.items[pos].count = count;
                                side.items[pos].info = i.clone();
                            }
                            None if req.mode == mode::ADD_ITEM && !full => {
                                side.items.push(Offered { guid: it.guid, count, info: i.clone() });
                            }
                            _ => return Ok(()),
                        }
                        item = i;
                    }
                }
                // Toute modification annule les confirmations en cours.
                for h in [me, partner] {
                    if let Some(s) = w.trades.open.get_mut(&h) {
                        s.confirmed = false;
                    }
                }
                w.trade_broadcast(me, partner, req.mode, item);
            }
            mode::FREEZE => {
                let Some(side) = w.trades.open.get_mut(&me) else { return Ok(()) };
                side.frozen = true;
                let partner = side.partner;
                w.trade_broadcast(me, partner, mode::FREEZE, Default::default());
            }
            mode::CONFIRM => {
                let Some(partner) = w.trades.open.get(&me).map(|s| s.partner) else { return Ok(()) };
                let both_frozen = [me, partner].iter().all(|h| w.trades.open.get(h).is_some_and(|s| s.frozen));
                if !both_frozen {
                    return Ok(());
                }
                w.trades.open.get_mut(&me).unwrap().confirmed = true;
                w.trade_broadcast(me, partner, mode::CONFIRM, Default::default());
            }
            mode::PROCESS => {
                let Some(partner) = w.trades.open.get(&me).map(|s| s.partner) else { return Ok(()) };
                let both_confirmed = [me, partner].iter().all(|h| w.trades.open.get(h).is_some_and(|s| s.confirmed));
                if !both_confirmed {
                    return Ok(());
                }
                w.trades.open.get_mut(&me).unwrap().processing = true;
                if !w.trades.open.get(&partner).is_some_and(|s| s.processing) {
                    return Ok(());
                }
                // Les deux ont validé : on sort l'échange de l'état partagé et on l'exécute.
                let a = w.trades.open.remove(&me).unwrap();
                let b = w.trades.open.remove(&partner).unwrap();
                exec = Some((a, partner, b));
            }
            _ => {}
        }
        }
        if let Some((a, partner, b)) = exec {
            self.execute_trade(me, a, partner, b).await?;
        }
        Ok(())
    }

    async fn execute_trade(&mut self, me: u32, a: TradeSide, partner: u32, b: TradeSide) -> Result<()> {
        let ctx = self.ctx.clone();
        let offer = |s: &TradeSide| s.items.iter().map(|o| Offer { guid: o.guid, count: o.count }).collect::<Vec<_>>();
        let (offer_a, offer_b) = (offer(&a), offer(&b));
        let mut next_guid = || ctx.next_item_guid();
        let result = db_trade::exchange(
            &ctx.chars,
            Party { guid: a.character, offer: &offer_a, gold: a.gold },
            Party { guid: b.character, offer: &offer_b, gold: b.gold },
            BAG_SLOT_BASE..BAG_SLOT_BASE + BAG_CAPACITY,
            &mut next_guid,
        )
        .await;
        let w = ctx.world.lock().unwrap();
        let outcome = match result {
            Ok(Some(sides)) => sides,
            other => {
                if let Err(error) = other {
                    tracing::error!("échange {me} ↔ {partner} annulé : {error:#}");
                }
                for (h, o) in [(me, partner), (partner, me)] {
                    w.send_to(h, packet(o, mode::CANCEL, Default::default()));
                    w.notify(h, "Échange annulé : objets modifiés ou sac plein.");
                }
                return Ok(());
            }
        };
        let (side_a, side_b) = outcome;
        for (h, side, delta) in [(me, side_a, b.gold - a.gold), (partner, side_b, a.gold - b.gold)] {
            if let Some(p) = w.players.get(&h) {
                let _ = p.events.send(applied(side, delta));
            }
        }
        w.send_to(me, packet(partner, mode::PROCESS, Default::default()));
        w.send_to(partner, packet(me, mode::PROCESS, Default::default()));
        tracing::info!("échange {me} ↔ {partner} effectué ({} / {} objets)", offer_a.len(), offer_b.len());
        Ok(())
    }

    /// Part de l'échange revenant à ce joueur (événement du monde).
    pub async fn apply_trade(&mut self, lost: Vec<(u32, u32)>, gained: Vec<db_trade::Received>, gold_delta: i64) -> Result<()> {
        let Some(g) = &mut self.in_game else { return Ok(()) };
        let mut destroyed = Vec::new();
        let mut updated = Vec::new();
        for (guid, left) in lost {
            if let Some(pos) = g.items.iter().position(|i| i.guid == guid) {
                if left == 0 {
                    destroyed.push(g.items.remove(pos).handle());
                } else {
                    g.items[pos].count = left;
                    updated.push((g.items[pos].handle(), left));
                }
            }
        }
        let new_items: Vec<InvItem> =
            gained.into_iter().map(|r| InvItem { guid: r.guid, entry: r.entry, count: r.count, slot: r.slot }).collect();
        g.items.extend(new_items.iter().cloned());
        let money = g.character.money as i64 + gold_delta;
        for h in destroyed {
            self.send(&game::Handle::<{ game::id::TS_SC_DESTROY_ITEM }>(h));
        }
        for (h, c) in updated {
            self.send(&game::ScUpdateItemCount { item_handle: h, count: c as i64 });
        }
        if !new_items.is_empty() {
            let infos = new_items.iter().map(|i| self.item_info(i)).collect();
            self.send(&game::ScInventory { items: infos });
        }
        if gold_delta != 0 {
            self.set_money(money);
        }
        self.refresh_quests().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn busy_and_cancel_bookkeeping() {
        let mut t = Trades::default();
        t.requests.insert(2, 1);
        t.open.insert(1, TradeSide::new(2, 10));
        t.open.insert(2, TradeSide::new(1, 20));
        assert!(t.busy(1) && t.busy(2) && !t.busy(3));
        t.requests.retain(|target, requester| *target != 1 && *requester != 1);
        assert!(t.requests.is_empty());
    }
}
