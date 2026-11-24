//! Invocations (docs/re/summon_protocol.md) : une carte de créature (objet du groupe 13) porte
//! une créature persistée dans `character_pet`. Le client invoque et renvoie par TS_CS_SKILL
//! 4001 / 4002, la cible étant le handle de la **carte**.
//!
//! Socle actuel : fiches (301), formation (303/324), invoquer, renvoyer, suivi du maître,
//! visibilité. Pas encore : combat, EXP, évolution, monture, équipement, incantation.

use anyhow::Result;
use rz_database::pets::{self, Pet, NOT_IN_FORMATION};
use rz_proto::msg::{self, CreatureInfo, EnterKind, Message, MovePoint, ScEnter};
use rz_proto::msg_game::{self as game, id as gid, summon_skill, FORMATION_SLOTS};
use rz_proto::RawPacket;

use crate::gameplay::{max_hp_for, max_mp_for, ITEM_HANDLE_BASE};
use crate::session::Session;
use crate::world::{Motion, World};

/// Plage de handles des créatures (monstres 0x2000…, PNJ 0x4000…, sol 0x5000…, sac 0x6000…).
pub const SUMMON_HANDLE_BASE: u32 = 0x3000_0000;
/// Créatures invoquées simultanément par joueur [D] (le client a deux emplacements).
pub const MAX_SUMMONED: usize = 1;
/// Distance au-delà de laquelle la créature rejoint son maître, et distance d'arrêt [D].
const FOLLOW_START: f32 = 40.0;
const FOLLOW_STOP: f32 = 20.0;
/// Au-delà (téléportation), la créature réapparaît près du maître [D].
const FOLLOW_SNAP: f32 = 300.0;

pub fn summon_handle(card: u32) -> u32 {
    SUMMON_HANDLE_BASE + card
}

/// Coût en PM : 4001 = 60 + 20/niveau, 4002 = 5 (db_skill, mesuré) [O].
pub fn mana_cost(skill: u16, level: u8) -> i32 {
    if skill == summon_skill::SUMMON { 60 + 20 * i32::from(level) } else { 5 }
}

/// Niveau utilisable par les formules provisoires (u8) de `gameplay`.
fn capped(level: u16) -> u8 {
    level.min(u16::from(u8::MAX)) as u8
}

#[derive(Clone, Debug)]
pub struct SummonEntity {
    pub handle: u32,
    pub master: u32,
    pub code: u32,
    pub name: String,
    pub level: u16,
    pub hp: i32,
    pub max_hp: i32,
    pub mp: i32,
    pub max_mp: i32,
    pub layer: u8,
    pub motion: Motion,
    pub run_speed: f32,
    next_follow: u32,
}

impl World {
    pub(crate) fn summon_enter(&self, handle: u32, now: u32) -> Option<Vec<RawPacket>> {
        let s = self.summons.get(&handle)?;
        let (x, y) = s.motion.position(now);
        let creature = CreatureInfo {
            hp: s.hp,
            max_hp: s.max_hp,
            mp: s.mp,
            max_mp: s.max_mp,
            level: i32::from(s.level),
            ..Default::default()
        };
        let kind = EnterKind::Summon { creature, master: s.master, code: s.code, name: s.name.clone(), flag: 0 };
        let mut out =
            vec![ScEnter { enter_type: msg::enter_type::CREATURE, handle, x, y, z: 0.0, layer: s.layer, kind }.to_packet()];
        if let Some(movement) = Self::move_packet(handle, s.layer, &s.motion, now) {
            out.push(movement);
        }
        Some(out)
    }

    pub fn summoned_by(&self, master: u32) -> Vec<u32> {
        self.summons.values().filter(|s| s.master == master).map(|s| s.handle).collect()
    }

    /// Fait apparaître la créature de `master` ; faux si elle y est déjà ou si la limite est atteinte.
    pub fn summon(&mut self, master: u32, pet: &Pet, run_speed: f32) -> bool {
        let handle = summon_handle(pet.card);
        let now = self.now();
        if self.summons.contains_key(&handle) || self.summoned_by(master).len() >= MAX_SUMMONED {
            return false;
        }
        let Some(p) = self.players.get(&master).filter(|p| p.hp > 0) else { return false };
        let (x, y) = p.motion.position(now);
        let level = capped(pet.level);
        self.summons.insert(
            handle,
            SummonEntity {
                handle,
                master,
                code: pet.entry,
                name: pet.name.clone(),
                level: pet.level,
                hp: max_hp_for(level),
                max_hp: max_hp_for(level),
                mp: max_mp_for(level),
                max_mp: max_mp_for(level),
                layer: p.layer,
                motion: Motion::at(x + FOLLOW_STOP, y),
                run_speed,
                next_follow: 0,
            },
        );
        self.update_visibility();
        true
    }

    /// Fait disparaître une créature : 305 puis LEAVE pour tous ceux qui la voyaient.
    pub fn unsummon(&mut self, handle: u32) -> bool {
        if self.summons.remove(&handle).is_none() {
            return false;
        }
        let effect = game::Handle::<{ gid::TS_SC_UNSUMMON }>(handle).to_packet();
        self.hide(handle, Some(effect));
        true
    }

    fn hide(&mut self, handle: u32, before: Option<RawPacket>) {
        let leave = msg::ScLeave { handle }.to_packet();
        for p in self.players.values_mut() {
            if p.visible.remove(&handle) {
                if let Some(b) = &before {
                    let _ = p.tx.send(b.clone());
                }
                let _ = p.tx.send(leave.clone());
            }
        }
    }

    /// Les créatures suivent leur maître (et le rejoignent après une téléportation).
    pub(crate) fn tick_summons(&mut self, now: u32) {
        let handles: Vec<u32> = self.summons.keys().copied().collect();
        for h in handles {
            let s = &self.summons[&h];
            let Some(p) = self.players.get(&s.master) else {
                self.unsummon(h);
                continue;
            };
            let (mx, my) = p.motion.position(now);
            let (sx, sy) = s.motion.position(now);
            let d = ((mx - sx).powi(2) + (my - sy).powi(2)).sqrt();
            let (layer, master_speed) = (p.layer, f32::from(p.move_speed));
            if s.layer != layer || d > FOLLOW_SNAP {
                // Disparaît à l'ancienne position ; `update_visibility` la montre à la nouvelle.
                self.hide(h, None);
                let s = self.summons.get_mut(&h).unwrap();
                s.layer = layer;
                s.motion = Motion::at(mx + FOLLOW_STOP, my);
                continue;
            }
            if d > FOLLOW_START && now >= s.next_follow {
                let t = (d - FOLLOW_STOP) / d;
                let dest = MovePoint { x: sx + (mx - sx) * t, y: sy + (my - sy) * t };
                let s = self.summons.get_mut(&h).unwrap();
                s.next_follow = now + 50;
                s.motion = Motion { x: sx, y: sy, start_time: now, points: vec![dest], speed: s.run_speed.max(master_speed) };
                if let Some(packet) = Self::move_packet(h, layer, &self.summons[&h].motion, now) {
                    self.broadcast_around(h, &packet, false);
                }
            }
        }
    }
}

impl Session {
    /// Créature liée à un objet carte du personnage.
    pub fn pet_of(&self, item_handle: u32) -> Option<&Pet> {
        self.in_game.as_ref()?.pets.iter().find(|p| ITEM_HANDLE_BASE + p.card == item_handle)
    }

    /// La créature de cette carte est-elle actuellement invoquée ?
    pub fn is_summoned(&self, item_handle: u32) -> bool {
        self.pet_of(item_handle).is_some_and(|p| self.ctx.world.lock().unwrap().summons.contains_key(&summon_handle(p.card)))
    }

    /// Accorde les fiches de créature avec les cartes possédées : nouvelles cartes (créature créée
    /// au niveau 1 ou reprise avec sa carte après un échange) et cartes disparues (302).
    pub async fn sync_pets(&mut self) -> Result<()> {
        let ctx = self.ctx.clone();
        let Some(g) = &self.in_game else { return Ok(()) };
        let owner = g.character.guid;
        let cards: Vec<(u32, u32)> = g
            .items
            .iter()
            .filter_map(|i| ctx.data.items.get(&i.entry).map(|d| (i.guid, d.card_creature)).filter(|(_, c)| *c != 0))
            .collect();
        let gone: Vec<u32> = g.pets.iter().map(|p| p.card).filter(|c| !cards.iter().any(|(guid, _)| guid == c)).collect();
        let missing: Vec<(u32, u32)> = cards.into_iter().filter(|(c, _)| !g.pets.iter().any(|p| p.card == *c)).collect();

        for card in gone {
            self.ctx.world.lock().unwrap().unsummon(summon_handle(card));
            self.in_game.as_mut().unwrap().pets.retain(|p| p.card != card);
            self.send(&game::Handle::<{ gid::TS_SC_REMOVE_SUMMON_INFO }>(ITEM_HANDLE_BASE + card));
        }
        for (card, entry) in missing {
            let pet = match pets::by_card(&ctx.chars, card).await? {
                Some((row_owner, mut pet)) => {
                    if row_owner != owner {
                        pets::transfer(&ctx.chars, card, owner).await?;
                        pet.slot = NOT_IN_FORMATION;
                    }
                    pet
                }
                None => {
                    let name = ctx.data.creatures.get(entry).map(|c| ctx.data.strings.get(c.name_id)).unwrap_or_default();
                    // Nom du paquet : 18 caractères utiles (char[19]).
                    let pet = Pet { card, entry, level: 1, exp: 0, name: name.chars().take(18).collect(), slot: NOT_IN_FORMATION };
                    pets::insert(&ctx.chars, owner, &pet).await?;
                    pet
                }
            };
            self.send_pet(&pet);
            self.in_game.as_mut().unwrap().pets.push(pet);
        }
        Ok(())
    }

    /// Fiche d'une créature : 301 puis niveau, EXP et PV/PM sur son handle.
    fn send_pet(&self, pet: &Pet) {
        let handle = summon_handle(pet.card);
        let level = capped(pet.level);
        self.send(&game::ScAddSummonInfo {
            card_handle: ITEM_HANDLE_BASE + pet.card,
            summon_handle: handle,
            name: pet.name.clone(),
            code: pet.entry as i32,
            level: i32::from(pet.level),
            sp: 0,
        });
        self.send(&game::ScLevelUpdate { handle, level: i32::from(pet.level), job_level: 0 });
        self.send(&game::ScExpUpdate { handle, exp: i64::from(pet.exp), jp: 0 });
        let (hp, mp) = (max_hp_for(level), max_mp_for(level));
        self.send(&game::ScHpMp { handle, hp, max_hp: hp, mp, max_mp: mp, ..Default::default() });
    }

    /// TM_EQUIP_SUMMON : formation actuelle.
    pub fn send_formation(&self) {
        let Some(g) = &self.in_game else { return };
        let mut cards = [0; FORMATION_SLOTS];
        for p in g.pets.iter().filter(|p| usize::from(p.slot) < FORMATION_SLOTS) {
            cards[usize::from(p.slot)] = ITEM_HANDLE_BASE + p.card;
        }
        self.send(&game::EquipSummon { open_dialog: 0, cards });
    }

    /// TM_EQUIP_SUMMON reçu : nouvelle formation (cartes du sac uniquement, sans doublon).
    pub async fn set_formation(&mut self, req: game::EquipSummon) -> Result<()> {
        let ctx = self.ctx.clone();
        let Some(g) = &self.in_game else { return Ok(()) };
        let owner = g.character.guid;
        let in_bag = |card: u32| g.items.iter().any(|i| i.guid == card && i.in_bag());
        let wanted: Vec<(u32, u8)> = req
            .cards
            .iter()
            .enumerate()
            .filter(|(_, h)| **h != 0)
            .map(|(slot, h)| (h.wrapping_sub(ITEM_HANDLE_BASE), slot as u8))
            .collect();
        let valid = wanted.iter().all(|(card, _)| g.pets.iter().any(|p| p.card == *card) && in_bag(*card))
            && wanted.iter().all(|(card, _)| wanted.iter().filter(|(c, _)| c == card).count() == 1);
        if valid {
            let slot_of = |card: u32| wanted.iter().find(|(c, _)| *c == card).map_or(NOT_IN_FORMATION, |(_, s)| *s);
            let changes: Vec<(u32, u8)> =
                g.pets.iter().map(|p| (p.card, slot_of(p.card))).filter(|(c, s)| g.pets.iter().any(|p| p.card == *c && p.slot != *s)).collect();
            pets::save_formation(&ctx.chars, owner, &changes).await?;
            for (card, slot) in changes {
                if let Some(p) = self.in_game.as_mut().unwrap().pets.iter_mut().find(|p| p.card == card) {
                    p.slot = slot;
                }
            }
        }
        // Confirmation (ou rappel de la formation en vigueur si la demande est invalide).
        self.send_formation();
        Ok(())
    }

    /// TS_CS_SKILL 4001 / 4002. Immédiat : ni incantation ni TS_SC_SKILL pour l'instant.
    pub fn summon_skill(&mut self, req: &game::CsSkill) {
        let ok = self.try_summon_skill(req);
        let result = if ok.is_ok() { msg::result::SUCCESS } else { msg::result::ACCESS_DENIED };
        self.send(&msg::ScResult { request: gid::TS_CS_SKILL, result, value: 0 });
        if let (Err(why), Some(g)) = (ok, &self.in_game) {
            self.ctx.world.lock().unwrap().notify(g.handle, why);
        }
    }

    fn try_summon_skill(&self, req: &game::CsSkill) -> std::result::Result<(), &'static str> {
        let g = self.in_game.as_ref().ok_or("Hors du monde.")?;
        let known = crate::skills::learned(&g.character).ok().and_then(|s| s.get(&u32::from(req.skill_id)).copied()).unwrap_or(0);
        if req.skill_level == 0 || req.skill_level > known {
            return Err("Compétence non apprise.");
        }
        let pet = self.pet_of(req.target).ok_or("Aucune carte de créature sélectionnée.")?;
        if !g.items.iter().any(|i| i.guid == pet.card && i.in_bag()) {
            return Err("La carte doit être dans le sac.");
        }
        let run_speed = self.ctx.data.creatures.get(pet.entry).map_or(80.0, |c| c.run_speed.clamp(1, 250) as f32);
        let cost = mana_cost(req.skill_id, req.skill_level);
        let mut w = self.ctx.world.lock().unwrap();
        let owned = w.players.get(&g.handle).is_some_and(|p| p.guid == g.character.guid && p.hp > 0 && p.mp >= cost);
        if !owned {
            return Err("PM insuffisants ou état invalide.");
        }
        let done = if req.skill_id == summon_skill::SUMMON {
            w.summon(g.handle, pet, run_speed)
        } else {
            w.unsummon(summon_handle(pet.card))
        };
        if !done {
            return Err(if req.skill_id == summon_skill::SUMMON {
                "Une créature est déjà invoquée."
            } else {
                "Cette créature n'est pas invoquée."
            });
        }
        w.spend_mana(g.handle, cost);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mana_costs_and_handle_ranges() {
        assert_eq!(mana_cost(summon_skill::SUMMON, 1), 80);
        assert_eq!(mana_cost(summon_skill::SUMMON, 5), 160);
        assert_eq!(mana_cost(summon_skill::UNSUMMON, 5), 5);
        assert!(summon_handle(1) > SUMMON_HANDLE_BASE && summon_handle(0x0fff_ffff) < 0x4000_0000);
        assert_eq!(capped(300), 255);
    }
}
