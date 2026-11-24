//! Compétences actives à cible unique (docs/re/skill_protocol.md) : dégâts, soins, invocation.
//!
//! Déroulement : contrôles → CASTING si la compétence a une incantation → FIRE, qui applique le
//! coût et l'effet et démarre la recharge. Un mouvement, une annulation ou une condition perdue
//! donne CANCEL, sans coût ni recharge. Les restaurations de PM restent dans `casting.rs`.
//!
//! Valeur avant défense = formule d'info-bulle du client, avec l'attaque envoyée dans
//! TS_SC_STAT_INFO. Défense et ±10 % : règle provisoire du combat de base (absente du client).

use rz_database::pets::Pet;
use rz_proto::msg::{self, Message};
use rz_proto::msg_game::{self as game, id as gid, skill_hit, skill_type};
use rz_proto::{RawPacket, Writer};
use rz_rappelz::spells::{SpellKind, SpellQuote};

use crate::session::Session;
use crate::summons::summon_handle;
use crate::world::World;

/// Marge de portée (rayons du lanceur et de la cible, latence), comme le combat de base [D].
const RANGE_SLACK: f32 = 45.0;
/// Portée d'attaque quand `cast_range` = −1 [D].
const MELEE_RANGE: f32 = 24.0;

#[derive(Clone, Debug)]
pub(crate) enum Action {
    Effect,
    Summon { pet: Pet, run_speed: f32 },
    Unsummon { card: u32 },
}

/// Compétence apprise : (id, niveau, recharge totale, clé de recharge).
pub(crate) type Known = (u32, u8, u32, u32);

#[derive(Clone, Debug)]
pub(crate) struct PendingSpell {
    guid: u32,
    target: u32,
    position: (f32, f32, f32, u8),
    end: u64,
    quote: SpellQuote,
    action: Action,
    known: Vec<Known>,
}

fn casting_payload(duration: u32) -> Vec<u8> {
    let mut w = Writer::new();
    w.u32(duration).u16(0).zeros(3);
    w.finish(0).body
}

fn heal_payload(target: u32, hp: i32, amount: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.bool(false).f32(0.0).u8(1).u8(1).u16(1).u8(skill_hit::ADD_HP).u32(target).i32(hp).i32(amount);
    w.finish(0).body
}

impl World {
    fn spell_event(&self, caster: u32, c: &PendingSpell, skill_type: u8, mp_cost: i32, payload: Vec<u8>) -> Option<RawPacket> {
        let p = self.players.get(&caster)?;
        let (x, y, z, layer) = c.position;
        Some(
            game::ScSkill {
                skill_id: c.quote.id as u16,
                skill_level: c.quote.level,
                caster,
                target: c.target,
                x,
                y,
                z,
                layer,
                skill_type,
                hp_cost: 0,
                mp_cost,
                caster_hp: p.hp.max(0),
                caster_mp: p.mp.max(0),
                payload,
            }
            .to_packet(),
        )
    }

    /// Cible encore valable : vivante, visible, sur la même couche et à portée.
    fn spell_target_ok(&self, caster: u32, target: u32, q: &SpellQuote) -> bool {
        let Some(p) = self.players.get(&caster) else { return false };
        let now = self.now();
        let position = match q.kind {
            SpellKind::Summon | SpellKind::Unsummon => return true,
            SpellKind::Heal => match self.players.get(&target).filter(|t| t.hp > 0 && t.layer == p.layer) {
                Some(t) => t.motion.position(now),
                None => return false,
            },
            SpellKind::Physical | SpellKind::Magical => match self.monsters.get(&target).filter(|m| m.alive()) {
                Some(m) => m.motion.position(now),
                None => return false,
            },
        };
        let (x, y) = p.motion.position(now);
        let range = q.range.unwrap_or(MELEE_RANGE) + RANGE_SLACK;
        self.can_see(caster, target, now) && ((x - position.0).powi(2) + (y - position.1).powi(2)).sqrt() <= range
    }

    pub(crate) fn begin_spell(
        &mut self,
        handle: u32,
        guid: u32,
        target: u32,
        q: SpellQuote,
        action: Action,
        known: Vec<Known>,
    ) -> Result<(), &'static str> {
        let now = self.cast_clock();
        self.cast_cooldowns.retain(|_, end| *end > now);
        let p = self.players.get(&handle).filter(|p| p.guid == guid).ok_or("Hors du monde.")?;
        if p.hp <= 0 {
            return Err("Vous êtes mort.");
        }
        if p.mp < q.mp {
            return Err("PM insuffisants.");
        }
        if self.casts.contains_key(&handle) || self.spells.contains_key(&handle) {
            return Err("Une compétence est déjà en cours.");
        }
        if self.cast_cooldowns.contains_key(&(guid, q.group)) || self.cast_cooldowns.len() >= 8192 {
            return Err("Compétence en recharge.");
        }
        if q.cast > 0 && !p.motion.remaining_points(self.now()).is_empty() {
            return Err("Impossible en se déplaçant.");
        }
        if !self.spell_target_ok(handle, target, &q) {
            return Err("Cible invalide ou hors de portée.");
        }
        let (x, y) = p.motion.position(self.now());
        let position = (x, y, p.z, p.layer);
        let pending = PendingSpell { guid, target, position, end: now + u64::from(q.cast), quote: q, action, known };
        if q.cast == 0 {
            self.fire_spell(handle, pending);
            return Ok(());
        }
        self.players.get_mut(&handle).unwrap().attack_target = None;
        if let Some(packet) = self.spell_event(handle, &pending, skill_type::CASTING, 0, casting_payload(q.cast)) {
            self.broadcast_around(handle, &packet, true);
        }
        self.spells.insert(handle, pending);
        Ok(())
    }

    pub(crate) fn cancel_spell(&mut self, handle: u32) {
        let Some(c) = self.spells.remove(&handle) else { return };
        if let Some(packet) = self.spell_event(handle, &c, skill_type::CANCEL, 0, vec![0; 9]) {
            self.broadcast_around(handle, &packet, true);
        }
    }

    /// Joueur en train d'invoquer la créature de cette carte.
    pub(crate) fn summoning(&self, card: u32) -> Option<u32> {
        self.spells.iter().find(|(_, c)| matches!(&c.action, Action::Summon { pet, .. } if pet.card == card)).map(|(h, _)| *h)
    }

    pub(crate) fn tick_spells(&mut self) {
        let now = self.cast_clock();
        let mut handles: Vec<u32> = self.spells.keys().copied().collect();
        handles.sort_by_key(|h| (self.spells[h].end, *h));
        for h in handles {
            let Some(c) = self.spells.get(&h) else { continue };
            let valid = self.players.get(&h).is_some_and(|p| {
                let (x, y) = p.motion.position(self.now());
                p.guid == c.guid
                    && p.hp > 0
                    && p.layer == c.position.3
                    && p.mp >= c.quote.mp
                    && p.attack_target.is_none()
                    && (x - c.position.0).abs() < 0.01
                    && (y - c.position.1).abs() < 0.01
            }) && self.spell_target_ok(h, c.target, &c.quote);
            if !valid {
                self.cancel_spell(h);
            } else if now >= c.end {
                let c = self.spells.remove(&h).unwrap();
                self.fire_spell(h, c);
            }
        }
    }

    fn fire_spell(&mut self, handle: u32, c: PendingSpell) {
        let now = self.cast_clock();
        let q = c.quote;
        // L'invocation peut échouer (déjà présente, limite) : ni coût ni recharge dans ce cas.
        let done = match &c.action {
            Action::Effect => true,
            Action::Summon { pet, run_speed } => self.summon(handle, pet, *run_speed),
            Action::Unsummon { card } => self.unsummon(summon_handle(*card)),
        };
        if !done {
            if let Some(packet) = self.spell_event(handle, &c, skill_type::CANCEL, 0, vec![0; 9]) {
                self.broadcast_around(handle, &packet, true);
            }
            self.notify(handle, "Invocation impossible.");
            return;
        }
        let (power, level_now) = {
            let p = self.players.get_mut(&handle).unwrap();
            p.mp = (p.mp - q.mp).max(0);
            p.enter.creature.mp = p.mp;
            p.last_combat = now as u32;
            let magic = crate::gameplay::magic_for(p.enter.creature.level.clamp(0, 255) as u8);
            (if q.kind == SpellKind::Physical { p.attack } else { magic }, self.now())
        };
        let mut killed = None;
        // Vitaux finaux de la cible, pour ceux qui la voient sans voir le lanceur.
        let mut vitals = None;
        let payload = match q.kind {
            SpellKind::Physical | SpellKind::Magical => {
                let m = self.monsters.get_mut(&c.target).unwrap();
                let physical = q.kind == SpellKind::Physical;
                let nominal = q.nominal(power, 0);
                let damage = crate::world::roll_damage(nominal, if physical { m.info.defence } else { m.info.magic_defence });
                m.hp = (m.hp - damage).max(0);
                if m.target.is_none() {
                    m.target = Some(handle);
                }
                if m.hp == 0 {
                    killed = Some(c.target);
                }
                vitals = Some(World::hpmp_packet(c.target, m.hp, m.info.hp, m.info.mp, m.info.mp, -damage));
                let hit = game::SkillDamageHit {
                    hit_type: if physical { skill_hit::DAMAGE } else { skill_hit::MAGIC_DAMAGE },
                    target: c.target,
                    target_hp: m.hp,
                    damage,
                    ..Default::default()
                };
                game::ScSkill::fire_payload(false, 0.0, 1, 1, &[hit])
            }
            SpellKind::Heal => {
                let t = self.players.get_mut(&c.target).unwrap();
                let before = t.hp;
                t.hp = t.hp.saturating_add(q.nominal(power, t.max_hp)).min(t.max_hp);
                t.enter.creature.hp = t.hp;
                let amount = t.hp - before;
                vitals = Some(World::hpmp_packet(c.target, t.hp, t.max_hp, t.mp, t.max_mp, amount));
                heal_payload(c.target, t.hp, amount)
            }
            SpellKind::Summon | SpellKind::Unsummon => game::ScSkill::fire_payload(false, 0.0, 0, 1, &[]),
        };
        if let Some(packet) = self.spell_event(handle, &c, skill_type::FIRE, q.mp, payload) {
            self.broadcast_around(handle, &packet, true);
        }
        if let Some(packet) = vitals {
            for viewer in self.players.keys().copied().collect::<Vec<_>>() {
                if viewer != handle && !self.can_see(viewer, handle, level_now) && self.can_see(viewer, c.target, level_now) {
                    self.send_to(viewer, packet.clone());
                }
            }
        }
        if q.cooldown > 0 {
            self.cast_cooldowns.insert((c.guid, q.group), now + u64::from(q.cooldown));
        }
        // Recharge commune : totaux et restes des compétences apprises du même groupe.
        let skills: Vec<game::SkillInfo> = c
            .known
            .iter()
            .filter(|k| k.3 == q.group)
            .map(|(id, level, total, group)| game::SkillInfo {
                skill_id: *id as i32,
                base_skill_level: *level,
                current_skill_level: *level,
                total_cool_time: *total,
                remain_cool_time: self.cooldown_remaining(c.guid, *group),
            })
            .collect();
        if !skills.is_empty() {
            self.send_to(handle, game::ScSkillList { target: handle, modification_type: 0, skills }.to_packet());
        }
        if let Some(m) = killed {
            self.kill_monster(m, handle, level_now);
        }
    }
}

impl Session {
    /// Compétence prise en charge par ce module : niveau appris, recharge et clé.
    pub(crate) fn known_spells(&self) -> Vec<Known> {
        let Some(levels) = self.in_game.as_ref().and_then(|g| crate::skills::learned(&g.character).ok()) else {
            return Vec::new();
        };
        levels
            .into_iter()
            .filter_map(|(id, level)| self.ctx.data.spells.quote(id, level).map(|q| (id, level, q.cooldown, q.group)))
            .collect()
    }

    /// TS_CS_SKILL d'une compétence de ce module. Répond par TS_SC_RESULT, plus un message si refusé.
    pub fn cast_spell(&mut self, req: &game::CsSkill, q: SpellQuote) {
        let outcome = self.try_cast_spell(req, q);
        let result = if outcome.is_ok() { msg::result::SUCCESS } else { msg::result::ACCESS_DENIED };
        self.send(&msg::ScResult { request: gid::TS_CS_SKILL, result, value: 0 });
        if let (Err(why), Some(g)) = (outcome, &self.in_game) {
            self.ctx.world.lock().unwrap().notify(g.handle, why);
        }
    }

    fn try_cast_spell(&self, req: &game::CsSkill, q: SpellQuote) -> Result<(), &'static str> {
        let g = self.in_game.as_ref().ok_or("Hors du monde.")?;
        let known = crate::skills::learned(&g.character).ok().and_then(|s| s.get(&q.id).copied()).unwrap_or(0);
        if req.skill_level == 0 || req.skill_level > known {
            return Err("Compétence non apprise.");
        }
        if req.caster != g.handle || ![req.x, req.y, req.z].iter().all(|v| v.is_finite()) {
            return Err("Demande invalide.");
        }
        let action = match q.kind {
            SpellKind::Summon | SpellKind::Unsummon => {
                let pet = self.pet_of(req.target).ok_or("Aucune carte de créature sélectionnée.")?;
                if !g.items.iter().any(|i| i.guid == pet.card && i.in_bag()) {
                    return Err("La carte doit être dans le sac.");
                }
                if q.kind == SpellKind::Summon {
                    let run_speed = self.ctx.data.creatures.get(pet.entry).map_or(80.0, |c| c.run_speed.clamp(1, 250) as f32);
                    Action::Summon { pet: pet.clone(), run_speed }
                } else {
                    Action::Unsummon { card: pet.card }
                }
            }
            _ => Action::Effect,
        };
        let known = self.known_spells();
        self.ctx.world.lock().unwrap().begin_spell(g.handle, g.character.guid, req.target, q, action, known)
    }
}
