//! Compétences actives (docs/re/skill_protocol.md) : dégâts (cible unique, zone, coups multiples),
//! soins, états, invocation.
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
use rz_rappelz::spells::{Area, SpellKind, SpellQuote};
use rz_rappelz::spells::StateQuote;
use rz_rappelz::states::{Stat, StateDef};

use crate::session::Session;
use crate::summons::summon_handle;
use crate::world::World;

/// Marge de portée (rayons du lanceur et de la cible, latence), comme le combat de base [D].
const RANGE_SLACK: f32 = 45.0;
/// Portée d'attaque quand `cast_range` = −1 [D].
const MELEE_RANGE: f32 = 24.0;
/// db_creature `attack_range` : centièmes d'unité de portée × 12, comme STAT_INFO +0x47 [D].
pub(crate) const RANGE_UNIT_PER_POINT: f32 = 12.0 / 100.0;

#[derive(Clone, Debug)]
pub(crate) enum Action {
    Effect,
    State(StateDef),
    Summon { pet: Pet, run_speed: f32, attack_range: f32 },
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

/// Queue FIRE : par vagues (premier coup sur chaque cible, deuxième coup…), ordre attendu par les
/// coups d'arme continus ; le dernier impact de chaque cible porte ses PV finaux.
fn fire_hits(hits: &[Vec<game::SkillDamageHit>], q: &SpellQuote) -> Vec<u8> {
    let waves = (0..usize::from(q.hits)).flat_map(|k| hits.iter().filter_map(move |h| h.get(k).cloned()));
    let (target_count, fire_count) = q.fire_counts(hits.len());
    game::ScSkill::fire_payload(false, 0.0, target_count, fire_count, &waves.collect::<Vec<_>>())
}

/// Impact RESULT (10 octets) : succès, genre 10 = état ajouté [O].
fn state_payload(target: u32, success: bool) -> Vec<u8> {
    result_payload(target, success, 10)
}

fn result_payload(target: u32, success: bool, kind: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.bool(false).f32(0.0).u8(1).u8(1).u16(1).u8(skill_hit::RESULT).u32(target).bool(success).i32(kind);
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
            SpellKind::Heal | SpellKind::Bless | SpellKind::Toggle => match self.players.get(&target).filter(|t| t.hp > 0 && t.layer == p.layer) {
                Some(t) => t.motion.position(now),
                None => return false,
            },
            SpellKind::Physical | SpellKind::Magical | SpellKind::Curse => match self.monsters.get(&target).filter(|m| m.alive()) {
                Some(m) => m.motion.position(now),
                None => return false,
            },
        };
        if q.self_only {
            return target == caster;
        }
        let (x, y) = p.motion.position(now);
        let range = q.range.unwrap_or(MELEE_RANGE) + RANGE_SLACK;
        self.can_see(caster, target, now) && ((x - position.0).powi(2) + (y - position.1).powi(2)).sqrt() <= range
    }

    /// Monstres visibles dans la zone, la cible principale d'abord puis du plus proche au plus
    /// lointain.
    fn area_targets(&self, caster: u32, target: u32, area: Area) -> Vec<u32> {
        let now = self.now();
        let Some((x, y)) = self.monsters.get(&target).map(|m| m.motion.position(now)) else { return Vec::new() };
        let mut found: Vec<(f32, u32)> = self
            .monsters
            .iter()
            .filter(|(h, m)| m.alive() && self.can_see(caster, **h, now))
            .map(|(h, m)| {
                let p = m.motion.position(now);
                (if *h == target { -1.0 } else { ((p.0 - x).powi(2) + (p.1 - y).powi(2)).sqrt() }, *h)
            })
            .filter(|(d, _)| *d <= area.radius)
            .collect();
        found.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        found.into_iter().take(usize::from(area.max_targets)).map(|(_, h)| h).collect()
    }

    pub(crate) fn begin_spell(
        &mut self,
        handle: u32,
        guid: u32,
        target: u32,
        mut q: SpellQuote,
        action: Action,
        known: Vec<Known>,
    ) -> Result<(), &'static str> {
        let now = self.cast_clock();
        // Éteindre une bascule : immédiat et gratuit (la recharge locale part à chaque FIRE).
        let off = q.kind == SpellKind::Toggle && self.toggle_active(handle, q.id);
        if off {
            (q.mp, q.cast) = (0, 0);
        }
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
        if (!off && self.cast_cooldowns.contains_key(&(guid, q.group))) || self.cast_cooldowns.len() >= 8192 {
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
            Action::Effect | Action::State(_) => true,
            Action::Summon { pet, run_speed, attack_range } => self.summon(handle, pet, *run_speed, *attack_range),
            Action::Unsummon { card } => self.unsummon(summon_handle(*card)),
        };
        if !done {
            if let Some(packet) = self.spell_event(handle, &c, skill_type::CANCEL, 0, vec![0; 9]) {
                self.broadcast_around(handle, &packet, true);
            }
            self.notify(handle, "Invocation impossible.");
            return;
        }
        {
            let p = self.players.get_mut(&handle).unwrap();
            p.mp = (p.mp - q.mp).max(0);
            p.enter.creature.mp = p.mp;
            p.last_combat = now as u32;
        }
        let power = self.power_of(handle, q.kind != SpellKind::Physical);
        let level_now = self.now();
        let mut killed = Vec::new();
        // États à poser, après FIRE (le client attend 505 ensuite).
        let mut posed = Vec::new();
        // Vitaux finaux des cibles, pour ceux qui les voient sans voir le lanceur.
        let mut vitals = Vec::new();
        let state = match (&c.action, q.state) {
            (Action::State(def), Some(s)) => Some((s, def.clone())),
            _ => None,
        };
        let lands = |s: &Option<(StateQuote, StateDef)>| s.as_ref().is_some_and(|(s, _)| rand::random::<f32>() * 100.0 < s.chance);
        let payload = match q.kind {
            SpellKind::Physical | SpellKind::Magical => {
                let physical = q.kind == SpellKind::Physical;
                let nominal = q.nominal(power, 0);
                let targets = match q.area {
                    // Les compteurs de FIRE tiennent sur un octet.
                    Some(area) => {
                        let most = area.max_targets.min(255 / q.hits.max(1));
                        self.area_targets(handle, c.target, Area { max_targets: most, ..area })
                    }
                    None => vec![c.target],
                };
                let mut hits = vec![Vec::new(); targets.len()];
                for (n, &target) in targets.iter().enumerate() {
                    let mods = self.mods(target);
                    let m = self.monsters.get_mut(&target).unwrap();
                    let defence = if physical {
                        mods.apply(Stat::Defence, m.info.defence)
                    } else {
                        mods.apply(Stat::MagicDefence, m.info.magic_defence)
                    };
                    let mut total = 0;
                    // Chaque coup porte : les PV annoncés décroissent d'un impact à l'autre.
                    for _ in 0..q.hits {
                        let damage = crate::world::roll_damage(nominal, defence);
                        m.hp = (m.hp - damage).max(0);
                        total += damage;
                        hits[n].push(game::SkillDamageHit {
                            hit_type: if physical { skill_hit::DAMAGE } else { skill_hit::MAGIC_DAMAGE },
                            target,
                            target_hp: m.hp,
                            damage,
                            ..Default::default()
                        });
                    }
                    m.target.get_or_insert(handle);
                    vitals.push((target, World::hpmp_packet(target, m.hp, m.info.hp, m.info.mp, m.info.mp, -total)));
                    if m.hp == 0 {
                        killed.push(target);
                    } else if lands(&state) {
                        posed.push(target);
                    }
                }
                fire_hits(&hits, &q)
            }
            SpellKind::Heal => {
                let t = self.players.get_mut(&c.target).unwrap();
                let before = t.hp;
                t.hp = t.hp.saturating_add(q.nominal(power, t.max_hp)).min(t.max_hp);
                t.enter.creature.hp = t.hp;
                let amount = t.hp - before;
                vitals.push((c.target, World::hpmp_packet(c.target, t.hp, t.max_hp, t.mp, t.max_mp, amount)));
                let payload = heal_payload(c.target, t.hp, amount);
                if lands(&state) {
                    posed.push(c.target);
                }
                payload
            }
            SpellKind::Curse | SpellKind::Bless => {
                if lands(&state) {
                    posed.push(c.target);
                }
                if let Some(m) = self.monsters.get_mut(&c.target) {
                    m.target.get_or_insert(handle);
                }
                state_payload(c.target, !posed.is_empty())
            }
            // Genre 0 : sans message, 505 nomme déjà l'état ajouté ou retiré.
            SpellKind::Toggle => result_payload(c.target, true, 0),
            SpellKind::Summon | SpellKind::Unsummon => game::ScSkill::fire_payload(false, 0.0, 0, 1, &[]),
        };
        if let Some(packet) = self.spell_event(handle, &c, skill_type::FIRE, q.mp, payload) {
            self.broadcast_around(handle, &packet, true);
        }
        if matches!(q.kind, SpellKind::Physical | SpellKind::Magical) {
            for (target, _) in &vitals {
                self.wake(*target);
            }
        }
        if let (Some((s, def)), Some(toggle)) = (&state, q.toggle) {
            self.switch_toggle(handle, q.id, s, toggle, def.clone());
        } else if let Some((s, def)) = state {
            let physical = def.periodic.is_some_and(|p| p.pulse == rz_rappelz::states::Pulse::PhysicalDamage);
            let power = self.power_of(handle, !physical);
            for target in posed {
                self.apply_state(target, handle, &s, def.clone(), power);
            }
        }
        for (target, packet) in vitals {
            for viewer in self.players.keys().copied().collect::<Vec<_>>() {
                if viewer != handle && !self.can_see(viewer, handle, level_now) && self.can_see(viewer, target, level_now) {
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
        for m in killed {
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
        if !matches!(q.kind, SpellKind::Summon | SpellKind::Unsummon) {
            let held = crate::stats::profile(&self.ctx.data, &g.character, &g.items).held;
            if !self.ctx.data.spells.weapon_fits(q.id, held) {
                return Err("Arme requise non équipée.");
            }
        }
        let action = match q.kind {
            SpellKind::Summon | SpellKind::Unsummon => {
                let pet = self.pet_of(req.target).ok_or("Aucune carte de créature sélectionnée.")?;
                if !g.items.iter().any(|i| i.guid == pet.card && i.in_bag()) {
                    return Err("La carte doit être dans le sac.");
                }
                if q.kind == SpellKind::Summon {
                    let creature = self.ctx.data.creatures.get(pet.entry);
                    let run_speed = creature.map_or(80.0, |c| c.run_speed.clamp(1, 250) as f32);
                    let attack_range = creature.map_or(24.0, |c| (c.attack_range as f32 * crate::spells::RANGE_UNIT_PER_POINT).clamp(20.0, 80.0));
                    Action::Summon { pet: pet.clone(), run_speed, attack_range }
                } else {
                    Action::Unsummon { card: pet.card }
                }
            }
            _ => match q.state {
                Some(s) => Action::State(self.ctx.data.states.get(s.id).ok_or("État non pris en charge.")?.clone()),
                None => Action::Effect,
            },
        };
        let known = self.known_spells();
        self.ctx.world.lock().unwrap().begin_spell(g.handle, g.character.guid, req.target, q, action, known)
    }
}
