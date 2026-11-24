//! États en cours (docs/re/skill_protocol.md §4) : pose, mise à jour, expiration, effets
//! périodiques et modificateurs de statistiques.
//!
//! Côté client rien n'expire : chaque retrait est un TS_SC_STATE de niveau 0 avec la même clé
//! (porteur, state_handle, code). Les états ne sont pas sauvegardés : ils tombent à la mort, à
//! la déconnexion et à l'arrêt du serveur.

use rz_proto::msg::Message;
use rz_proto::msg_game as game;
use rz_proto::RawPacket;
use rz_rappelz::spells::StateQuote;
use rz_rappelz::states::{Mods, Pulse, Stat, StateDef};

use crate::gameplay::magic_for;
use crate::world::World;

/// Vitesse d'attaque de référence (TS_SC_STAT_INFO).
pub const BASE_ATTACK_SPEED: i32 = 100;
/// États simultanés par porteur, pour borner la mémoire et les paquets [D].
const MAX_STATES: usize = 40;

#[derive(Clone, Debug)]
pub(crate) struct ActiveState {
    /// Identifie l'instance auprès du client, avec le porteur et le code.
    state_handle: u16,
    level: u16,
    start: u32,
    end: u32,
    caster: u32,
    /// Attaque du lanceur à la pose (effets périodiques).
    power: i32,
    next_pulse: u32,
    total: i32,
    def: StateDef,
}

impl ActiveState {
    fn packet(&self, bearer: u32, level: u16) -> RawPacket {
        game::ScState {
            handle: bearer,
            state_handle: self.state_handle,
            state_level: level,
            state_code: self.def.id,
            remain_time: self.end as i32,
            start_time: self.start,
            state_value: 0,
        }
        .to_packet()
    }
}

#[derive(Default)]
pub(crate) struct States {
    bearers: std::collections::HashMap<u32, Vec<ActiveState>>,
    next_handle: u16,
}

impl World {
    /// Cumul des modificateurs du porteur (joueur ou monstre).
    pub(crate) fn mods(&self, bearer: u32) -> Mods {
        let mut m = Mods::default();
        for s in self.states.bearers.get(&bearer).into_iter().flatten() {
            m.add(&s.def.mods(s.level));
        }
        m
    }

    pub(crate) fn stat(&self, bearer: u32, stat: Stat, base: i32) -> i32 {
        self.mods(bearer).apply(stat, base)
    }

    /// Attaque (physique ou magique) effective d'un joueur.
    pub(crate) fn power_of(&self, player: u32, magical: bool) -> i32 {
        let Some(p) = self.players.get(&player) else { return 0 };
        if magical {
            self.stat(player, Stat::MagicAttack, magic_for(p.enter.creature.level.clamp(0, 255) as u8))
        } else {
            self.stat(player, Stat::Attack, p.attack)
        }
    }

    /// Délai d'attaque effectif (centièmes) : inversement proportionnel à la vitesse d'attaque.
    pub(crate) fn attack_delay(&self, bearer: u32, base: u32) -> u32 {
        let speed = self.stat(bearer, Stat::AttackSpeed, BASE_ATTACK_SPEED).clamp(20, 500) as u32;
        base * BASE_ATTACK_SPEED as u32 / speed
    }

    /// États d'un porteur, à envoyer après son ENTER.
    pub(crate) fn state_packets(&self, bearer: u32) -> Vec<RawPacket> {
        self.states.bearers.get(&bearer).into_iter().flatten().map(|s| s.packet(bearer, s.level)).collect()
    }

    /// Pose ou renouvelle un état ; faux si un état plus fort du même code est déjà présent.
    pub(crate) fn apply_state(&mut self, bearer: u32, caster: u32, q: &StateQuote, def: StateDef, power: i32) -> bool {
        let now = self.now();
        let end = now.saturating_add(q.duration);
        let next_pulse = now.saturating_add(def.periodic.map_or(0, |p| p.interval));
        let list = self.states.bearers.entry(bearer).or_default();
        if let Some(s) = list.iter_mut().find(|s| s.def.id == def.id) {
            if s.level > q.level {
                return false;
            }
            (s.level, s.start, s.end, s.caster, s.power, s.next_pulse, s.total) = (q.level, now, end, caster, power, next_pulse, 0);
            let packet = s.packet(bearer, s.level);
            self.broadcast_around(bearer, &packet, true);
            self.refresh_bearer(bearer);
            return true;
        }
        // Groupe d'exclusion : l'ancien état cède la place.
        let replaced: Vec<u16> = list.iter().filter(|s| s.def.excludes(&def)).map(|s| s.state_handle).collect();
        self.remove_states(bearer, |s| replaced.contains(&s.state_handle));
        if self.states.bearers.get(&bearer).is_some_and(|l| l.len() >= MAX_STATES) {
            return false;
        }
        self.states.next_handle = self.states.next_handle.wrapping_add(1).max(1);
        let state = ActiveState {
            state_handle: self.states.next_handle,
            level: q.level,
            start: now,
            end,
            caster,
            power,
            next_pulse,
            total: 0,
            def,
        };
        let packet = state.packet(bearer, state.level);
        self.states.bearers.entry(bearer).or_default().push(state);
        self.broadcast_around(bearer, &packet, true);
        self.refresh_bearer(bearer);
        true
    }

    /// Retire les états choisis (TS_SC_STATE niveau 0).
    fn remove_states(&mut self, bearer: u32, pick: impl Fn(&ActiveState) -> bool) {
        let Some(list) = self.states.bearers.get_mut(&bearer) else { return };
        let (gone, kept): (Vec<_>, Vec<_>) = std::mem::take(list).into_iter().partition(|s| pick(s));
        if kept.is_empty() {
            self.states.bearers.remove(&bearer);
        } else {
            *list = kept;
        }
        if gone.is_empty() {
            return;
        }
        for s in &gone {
            let packet = s.packet(bearer, 0);
            self.broadcast_around(bearer, &packet, true);
        }
        self.refresh_bearer(bearer);
    }

    pub(crate) fn clear_states(&mut self, bearer: u32) {
        self.remove_states(bearer, |_| true);
    }

    /// Statistiques envoyées au porteur (joueur) après un changement.
    fn refresh_bearer(&mut self, bearer: u32) {
        if self.players.contains_key(&bearer) {
            self.publish_stats(bearer);
        }
    }

    pub(crate) fn tick_states(&mut self, now: u32) {
        let bearers: Vec<u32> = self.states.bearers.keys().copied().collect();
        for b in bearers {
            let alive = self.players.get(&b).is_some_and(|p| p.hp > 0) || self.monsters.get(&b).is_some_and(|m| m.alive());
            if !alive {
                self.clear_states(b);
                continue;
            }
            let due: Vec<u16> = self.states.bearers[&b]
                .iter()
                .filter(|s| s.def.periodic.is_some() && now >= s.next_pulse && s.next_pulse <= s.end)
                .map(|s| s.state_handle)
                .collect();
            for h in due {
                self.pulse(b, h);
            }
            self.remove_states(b, |s| now >= s.end);
        }
    }

    /// Un tic de dégâts ou de soin périodique (TM_STATE_DAMAGE, suffisant seul côté client).
    fn pulse(&mut self, bearer: u32, state_handle: u16) {
        let Some(s) = self.states.bearers.get_mut(&bearer).and_then(|l| l.iter_mut().find(|s| s.state_handle == state_handle)) else {
            return; // mort ou retiré par un tic précédent
        };
        let Some(p) = s.def.periodic else { return };
        s.next_pulse = s.next_pulse.saturating_add(p.interval);
        let last = s.next_pulse > s.end;
        let (amount, caster, code, level) = (p.amount(s.level, s.power), s.caster, s.def.id, s.level);
        let (kind, value, result) = match p.pulse {
            Pulse::PhysicalDamage | Pulse::MagicDamage => {
                // Les malus ne sont posés que sur des monstres.
                let Some(m) = self.monsters.get_mut(&bearer) else { return };
                m.hp = (m.hp - amount).max(0);
                if m.target.is_none() {
                    m.target = Some(caster);
                }
                (game::state_damage::DAMAGE, amount, m.hp)
            }
            Pulse::HealHp | Pulse::HealMp => {
                let Some(t) = self.players.get_mut(&bearer) else { return };
                let (current, max) = if p.pulse == Pulse::HealHp { (&mut t.hp, t.max_hp) } else { (&mut t.mp, t.max_mp) };
                let before = *current;
                *current = current.saturating_add(amount).min(max);
                let (gain, now_at) = (*current - before, *current);
                t.enter.creature.hp = t.hp;
                t.enter.creature.mp = t.mp;
                (if p.pulse == Pulse::HealHp { game::state_damage::HEAL_HP } else { game::state_damage::HEAL_MP }, gain, now_at)
            }
        };
        let total = {
            let s = self.states.bearers.get_mut(&bearer).and_then(|l| l.iter_mut().find(|s| s.state_handle == state_handle)).unwrap();
            s.total = s.total.saturating_add(value);
            s.total
        };
        let dead = kind == game::state_damage::DAMAGE && result == 0;
        let packet = game::ScStateDamage { caster, target: bearer, state_code: code, state_level: level, kind, value, result, last: last || dead, total };
        self.broadcast_around(bearer, &packet.to_packet(), true);
        if dead {
            let now = self.now();
            self.kill_monster(bearer, caster, now);
        }
    }
}
