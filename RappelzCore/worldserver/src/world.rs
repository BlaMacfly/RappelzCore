//! État du monde partagé : joueurs connectés, PNJ, monstres, déplacements, visibilité et
//! combat. Équivalent simplifié du `World` + `Map` d'AzerothCore (une seule carte, visibilité
//! par distance recalculée à chaque tick).

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use rz_proto::msg::{self, CreatureInfo, EnterKind, MovePoint, PlayerEnterInfo, ScEnter};
use rz_proto::msg_game::{self as game, AttackInfo};
use rz_proto::{Message, RawPacket};
use tokio::sync::mpsc::UnboundedSender;

use crate::data::{GameData, MonsterInfo};

/// Plages de handles distinctes par type d'objet.
const NPC_HANDLE_BASE: u32 = 0x4000_0000;
const MONSTER_HANDLE_BASE: u32 = 0x2000_0000;

/// Durées en centièmes de seconde (horloge TS_TIMESYNC).
const PLAYER_ATTACK_DELAY: u32 = 150;
const MONSTER_ATTACK_DELAY: u32 = 200;
const MONSTER_CORPSE_TIME: u32 = 300;
const REGEN_INTERVAL: u32 = 300;

/// Trajet en cours : départ (position, horloge) puis points à suivre à vitesse constante.
#[derive(Clone, Debug)]
pub struct Motion {
    pub x: f32,
    pub y: f32,
    pub start_time: u32,
    pub points: Vec<MovePoint>,
    pub speed: f32,
}

impl Motion {
    pub fn at(x: f32, y: f32) -> Self {
        Self { x, y, start_time: 0, points: Vec::new(), speed: 0.0 }
    }

    /// Position à l'instant `now` (centièmes de seconde) ; `speed` en unités par seconde.
    pub fn position(&self, now: u32) -> (f32, f32) {
        if self.points.is_empty() || self.speed <= 0.0 {
            return (self.x, self.y);
        }
        let mut budget = self.speed * now.saturating_sub(self.start_time) as f32 / 100.0;
        let (mut x, mut y) = (self.x, self.y);
        for p in &self.points {
            let d = ((p.x - x).powi(2) + (p.y - y).powi(2)).sqrt();
            if d >= budget {
                let t = if d > 0.0 { budget / d } else { 0.0 };
                return (x + (p.x - x) * t, y + (p.y - y) * t);
            }
            budget -= d;
            x = p.x;
            y = p.y;
        }
        (x, y)
    }

    /// Fige la position courante comme nouveau départ.
    pub fn settle(&mut self, now: u32) {
        let (x, y) = self.position(now);
        *self = Self::at(x, y);
    }
}

/// Événements du monde destinés à la session du joueur (qui possède le personnage en base).
#[derive(Debug)]
pub enum WorldEvent {
    /// Monstre tué : récompenses.
    Reward { exp: i64, jp: i64, gold: i64, monster: String },
    /// Le joueur est mort.
    Died,
}

pub struct PlayerEntity {
    pub handle: u32,
    pub guid: u32,
    pub account_id: u32,
    pub name: String,
    pub gm_level: u8,
    pub z: f32,
    pub layer: u8,
    pub motion: Motion,
    pub enter: PlayerEnterInfo,
    pub wear: [i32; game::WEAR_SLOTS],
    pub tx: UnboundedSender<RawPacket>,
    pub events: UnboundedSender<WorldEvent>,
    pub visible: HashSet<u32>,
    pub hp: i32,
    pub max_hp: i32,
    pub mp: i32,
    pub max_mp: i32,
    pub attack: i32,
    pub defence: i32,
    pub attack_target: Option<u32>,
    pub next_attack: u32,
    pub last_combat: u32,
}

pub struct NpcEntity {
    pub handle: u32,
    pub npc_id: u32,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub creature: CreatureInfo,
}

pub struct MonsterEntity {
    pub handle: u32,
    pub info: MonsterInfo,
    pub home: (f32, f32),
    pub z: f32,
    pub motion: Motion,
    pub hp: i32,
    pub respawn_secs: u32,
    /// Horloge de la mort (cadavre puis réapparition).
    pub dead_since: Option<u32>,
    pub target: Option<u32>,
    pub next_attack: u32,
    pub next_chase: u32,
}

impl MonsterEntity {
    fn alive(&self) -> bool {
        self.dead_since.is_none()
    }

    fn creature_info(&self) -> CreatureInfo {
        CreatureInfo {
            hp: self.hp,
            max_hp: self.info.hp,
            mp: self.info.mp,
            max_mp: self.info.mp,
            level: self.info.level,
            race: self.info.race,
            ..Default::default()
        }
    }
}

pub struct World {
    started: Instant,
    pub players: HashMap<u32, PlayerEntity>,
    pub npcs: HashMap<u32, NpcEntity>,
    pub monsters: HashMap<u32, MonsterEntity>,
    next_handle: u32,
    next_monster_handle: u32,
    pub visibility_distance: f32,
    pub max_online: usize,
    last_regen: u32,
}

fn dist((ax, ay): (f32, f32), (bx, by): (f32, f32)) -> f32 {
    ((ax - bx).powi(2) + (ay - by).powi(2)).sqrt()
}

/// Dégâts d'un coup : attaque − défense/2, ±10 %, au moins 1.
fn roll_damage(attack: i32, defence: i32) -> i32 {
    let base = (attack - defence / 2).max(1) as f32;
    (base * (0.9 + rand::random::<f32>() * 0.2)).round().max(1.0) as i32
}

impl World {
    pub fn new(data: &GameData, visibility_distance: f32) -> Self {
        let npcs = data
            .npcs
            .iter()
            .enumerate()
            .map(|(i, n)| {
                let handle = NPC_HANDLE_BASE + i as u32;
                (
                    handle,
                    NpcEntity {
                        handle,
                        npc_id: n.npc_id,
                        name: n.name.clone(),
                        x: n.x,
                        y: n.y,
                        z: n.z,
                        creature: CreatureInfo {
                            face_direction: n.orientation,
                            hp: n.hp,
                            max_hp: n.hp,
                            mp: n.mp,
                            max_mp: n.mp,
                            level: n.level,
                            race: n.race,
                            ..Default::default()
                        },
                    },
                )
            })
            .collect();
        let mut world = Self {
            started: Instant::now(),
            players: HashMap::new(),
            npcs,
            monsters: HashMap::new(),
            next_handle: 1,
            next_monster_handle: MONSTER_HANDLE_BASE,
            visibility_distance,
            max_online: 0,
            last_regen: 0,
        };
        for &(_, id, x, y, z, respawn) in &data.monster_spawns {
            if let Some(info) = data.monsters.get(&id) {
                world.spawn_monster(info.clone(), x, y, z, respawn);
            }
        }
        world
    }

    /// Horloge de jeu en centièmes de seconde (TS_TIMESYNC).
    pub fn now(&self) -> u32 {
        (self.started.elapsed().as_millis() / 10) as u32
    }

    pub fn uptime_secs(&self) -> u64 {
        self.started.elapsed().as_secs()
    }

    pub fn allocate_handle(&mut self) -> u32 {
        let h = self.next_handle;
        self.next_handle += 1;
        h
    }

    pub fn spawn_monster(&mut self, info: MonsterInfo, x: f32, y: f32, z: f32, respawn_secs: u32) -> u32 {
        let handle = self.next_monster_handle;
        self.next_monster_handle += 1;
        let hp = info.hp;
        self.monsters.insert(
            handle,
            MonsterEntity {
                handle,
                info,
                home: (x, y),
                z,
                motion: Motion::at(x, y),
                hp,
                respawn_secs: respawn_secs.max(5),
                dead_since: None,
                target: None,
                next_attack: 0,
                next_chase: 0,
            },
        );
        handle
    }

    pub fn add_player(&mut self, player: PlayerEntity) {
        self.players.insert(player.handle, player);
        self.max_online = self.max_online.max(self.players.len());
    }

    /// Retire un joueur et prévient ceux qui le voyaient.
    pub fn remove_player(&mut self, handle: u32) -> Option<PlayerEntity> {
        let p = self.players.remove(&handle)?;
        let leave = msg::ScLeave { handle }.to_packet();
        for other in self.players.values_mut() {
            if other.visible.remove(&handle) {
                let _ = other.tx.send(leave.clone());
            }
        }
        for m in self.monsters.values_mut() {
            if m.target == Some(handle) {
                m.target = None;
            }
        }
        Some(p)
    }

    pub fn find_by_name(&self, name: &str) -> Option<&PlayerEntity> {
        self.players.values().find(|p| p.name.eq_ignore_ascii_case(name))
    }

    pub fn player_position(&self, handle: u32) -> Option<(f32, f32)> {
        let now = self.now();
        self.players.get(&handle).map(|p| p.motion.position(now))
    }

    fn player_enter(&self, p: &PlayerEntity, now: u32) -> RawPacket {
        let (x, y) = p.motion.position(now);
        ScEnter { enter_type: 0, handle: p.handle, x, y, z: p.z, layer: p.layer, kind: EnterKind::Player(p.enter.clone()) }
            .to_packet()
    }

    fn wear_packet(p: &PlayerEntity) -> RawPacket {
        game::ScWearInfo { handle: p.handle, wear_info: p.wear, ..Default::default() }.to_packet()
    }

    /// Paquets d'apparition d'un objet du monde (ENTER + données associées).
    fn enter_packets(&self, handle: u32, now: u32) -> Vec<RawPacket> {
        if let Some(p) = self.players.get(&handle) {
            return vec![self.player_enter(p, now), Self::wear_packet(p)];
        }
        if let Some(n) = self.npcs.get(&handle) {
            let kind = EnterKind::Npc { creature: n.creature.clone(), npc_id: n.npc_id };
            return vec![ScEnter { enter_type: 0, handle, x: n.x, y: n.y, z: n.z, layer: 0, kind }.to_packet()];
        }
        if let Some(m) = self.monsters.get(&handle) {
            let (x, y) = m.motion.position(now);
            // Le client compare l'id à la valeur brute (bit-permutée) de db_monster.rdb.
            let monster_id = rz_rappelz::rdb::monster_id_encode(m.info.id);
            let kind = EnterKind::Monster { creature: m.creature_info(), is_tamed: false, monster_id };
            let mut out = vec![ScEnter { enter_type: 0, handle, x, y, z: m.z, layer: 0, kind }.to_packet()];
            if !m.motion.points.is_empty() {
                out.push(self.monster_move_packet(m));
            }
            return out;
        }
        Vec::new()
    }

    /// Réponse à TS_CS_QUERY : renvoie l'ENTER d'un objet visible.
    pub fn query(&self, viewer: u32, handle: u32) {
        let now = self.now();
        for p in self.enter_packets(handle, now) {
            self.send_to(viewer, p);
        }
    }

    fn monster_move_packet(&self, m: &MonsterEntity) -> RawPacket {
        msg::ScMove {
            start_time: m.motion.start_time,
            handle: m.handle,
            tlayer: 0,
            speed: m.motion.speed.clamp(1.0, 255.0) as u8,
            points: m.motion.points.clone(),
        }
        .to_packet()
    }

    /// Recalcule ce que voit chaque joueur et envoie les TS_SC_ENTER / TS_SC_LEAVE.
    pub fn update_visibility(&mut self) {
        let now = self.now();
        let range2 = self.visibility_distance * self.visibility_distance;
        let positions: Vec<(u32, f32, f32, u8)> = self
            .players
            .values()
            .map(|p| {
                let (x, y) = p.motion.position(now);
                (p.handle, x, y, p.layer)
            })
            .collect();

        let mut outgoing: Vec<(u32, RawPacket)> = Vec::new();
        for &(handle, x, y, layer) in &positions {
            let mut now_visible = HashSet::new();
            for &(other, ox, oy, olayer) in &positions {
                if other != handle && olayer == layer && (ox - x).powi(2) + (oy - y).powi(2) <= range2 {
                    now_visible.insert(other);
                }
            }
            if layer == 0 {
                for n in self.npcs.values() {
                    if (n.x - x).powi(2) + (n.y - y).powi(2) <= range2 {
                        now_visible.insert(n.handle);
                    }
                }
                for m in self.monsters.values() {
                    let (mx, my) = m.motion.position(now);
                    let corpse = m.dead_since.is_some_and(|t| now.saturating_sub(t) < MONSTER_CORPSE_TIME);
                    if (m.alive() || corpse) && (mx - x).powi(2) + (my - y).powi(2) <= range2 {
                        now_visible.insert(m.handle);
                    }
                }
            }
            let player = &self.players[&handle];
            for &h in now_visible.difference(&player.visible) {
                for p in self.enter_packets(h, now) {
                    outgoing.push((handle, p));
                }
            }
            for &h in player.visible.difference(&now_visible) {
                outgoing.push((handle, msg::ScLeave { handle: h }.to_packet()));
            }
            self.players.get_mut(&handle).unwrap().visible = now_visible;
        }
        for (handle, packet) in outgoing {
            let _ = self.players[&handle].tx.send(packet);
        }
    }

    /// Envoie un paquet au joueur et à tous ceux qui le voient.
    pub fn broadcast_around(&self, handle: u32, packet: &RawPacket, include_self: bool) {
        for p in self.players.values() {
            if (include_self && p.handle == handle) || p.visible.contains(&handle) {
                let _ = p.tx.send(packet.clone());
            }
        }
    }

    pub fn broadcast_all(&self, packet: &RawPacket) {
        for p in self.players.values() {
            let _ = p.tx.send(packet.clone());
        }
    }

    pub fn send_to(&self, handle: u32, packet: RawPacket) {
        if let Some(p) = self.players.get(&handle) {
            let _ = p.tx.send(packet);
        }
    }

    /// Message système (canal NOTICE) à un joueur.
    pub fn notify(&self, handle: u32, text: &str) {
        self.send_to(
            handle,
            msg::ScChat { sender: "@NOTICE".into(), chat_type: msg::chat_type::NOTICE, message: text.to_string() }.to_packet(),
        );
    }

    /// Téléporte un joueur (TS_SC_WARP) ; la visibilité suit au prochain tick.
    pub fn teleport(&mut self, handle: u32, x: f32, y: f32, z: f32) {
        if let Some(p) = self.players.get_mut(&handle) {
            p.motion = Motion::at(x, y);
            p.z = z;
            let _ = p.tx.send(msg::ScWarp { x, y, z, layer: p.layer }.to_packet());
        }
    }

    /// Met à jour l'apparence équipée d'un joueur et la diffuse (TS_SC_WEAR_INFO).
    pub fn set_wear(&mut self, handle: u32, wear: [i32; game::WEAR_SLOTS]) {
        if let Some(p) = self.players.get_mut(&handle) {
            p.wear = wear;
            let packet = Self::wear_packet(p);
            self.broadcast_around(handle, &packet, true);
        }
    }

    fn hpmp_packet(handle: u32, hp: i32, max_hp: i32, mp: i32, max_mp: i32, add_hp: i32) -> RawPacket {
        game::ScHpMp { handle, add_hp, hp, max_hp, add_mp: 0, mp, max_mp, need_to_display: add_hp != 0 }.to_packet()
    }

    // ───────────────────────── Combat ─────────────────────────

    pub fn start_attack(&mut self, attacker: u32, target: u32) -> bool {
        let alive = self.monsters.get(&target).is_some_and(|m| m.alive());
        if let Some(p) = self.players.get_mut(&attacker) {
            p.attack_target = alive.then_some(target);
        }
        if !alive {
            self.send_to(attacker, game::ScCantAttack { attacker, target, reason: 1 }.to_packet());
        }
        alive
    }

    pub fn stop_attack(&mut self, attacker: u32) {
        if let Some(p) = self.players.get_mut(&attacker) {
            p.attack_target = None;
        }
    }

    /// Tick de jeu : combat, poursuite, mort et réapparition des monstres, régénération.
    pub fn tick(&mut self) {
        let now = self.now();
        self.tick_player_attacks(now);
        self.tick_monsters(now);
        if now.saturating_sub(self.last_regen) >= REGEN_INTERVAL {
            self.last_regen = now;
            self.tick_regen(now);
        }
        self.update_visibility();
    }

    fn tick_player_attacks(&mut self, now: u32) {
        let attackers: Vec<(u32, u32)> = self
            .players
            .values()
            .filter(|p| p.attack_target.is_some() && now >= p.next_attack && p.hp > 0)
            .map(|p| (p.handle, p.attack_target.unwrap()))
            .collect();
        for (ph, mh) in attackers {
            let ppos = self.players[&ph].motion.position(now);
            let Some(m) = self.monsters.get(&mh).filter(|m| m.alive()) else {
                self.players.get_mut(&ph).unwrap().attack_target = None;
                continue;
            };
            if dist(ppos, m.motion.position(now)) > m.info.attack_range + 45.0 {
                continue; // le client rapproche le joueur de sa cible
            }
            let (attack, p_hp, p_mp) = {
                let p = self.players.get_mut(&ph).unwrap();
                p.next_attack = now + PLAYER_ATTACK_DELAY;
                p.last_combat = now;
                (p.attack, p.hp, p.mp)
            };
            let m = self.monsters.get_mut(&mh).unwrap();
            let damage = roll_damage(attack, m.info.defence);
            m.hp = (m.hp - damage).max(0);
            if m.target.is_none() {
                m.target = Some(ph);
            }
            let (m_hp, m_max, m_mp) = (m.hp, m.info.hp, m.info.mp);
            let event = game::ScAttackEvent {
                attacker: ph,
                target: mh,
                attack_speed: 100,
                attack_delay: PLAYER_ATTACK_DELAY as u16 * 10,
                attack_action: game::attack_action::ATTACK,
                attack_flag: 0,
                attacks: vec![AttackInfo {
                    damage,
                    target_hp: m_hp as i64,
                    target_mp: m_mp,
                    attacker_hp: p_hp as i64,
                    attacker_mp: p_mp,
                    ..Default::default()
                }],
            }
            .to_packet();
            self.broadcast_around(ph, &event, true);
            self.broadcast_around(mh, &Self::hpmp_packet(mh, m_hp, m_max, m_mp, m_mp, -damage), false);
            if m_hp == 0 {
                self.kill_monster(mh, ph, now);
            }
        }
    }

    fn kill_monster(&mut self, mh: u32, killer: u32, now: u32) {
        let Some(m) = self.monsters.get_mut(&mh) else { return };
        m.dead_since = Some(now);
        m.target = None;
        m.motion.settle(now);
        let info = m.info.clone();
        for p in self.players.values_mut() {
            if p.attack_target == Some(mh) {
                p.attack_target = None;
            }
        }
        let gold = if info.gold_max > 0 {
            info.gold_min as i64 + (rand::random::<u32>() % (info.gold_max - info.gold_min + 1).max(1) as u32) as i64
        } else {
            0
        };
        if let Some(p) = self.players.get(&killer) {
            let _ = p.events.send(WorldEvent::Reward { exp: info.exp.max(1), jp: info.jp, gold, monster: info.name.clone() });
        }
    }

    fn tick_monsters(&mut self, now: u32) {
        let handles: Vec<u32> = self.monsters.keys().copied().collect();
        for mh in handles {
            let m = &self.monsters[&mh];
            // Réapparition.
            if let Some(t) = m.dead_since {
                if now.saturating_sub(t) >= m.respawn_secs * 100 {
                    let m = self.monsters.get_mut(&mh).unwrap();
                    m.dead_since = None;
                    m.hp = m.info.hp;
                    m.motion = Motion::at(m.home.0, m.home.1);
                }
                continue;
            }
            let Some(target) = m.target else { continue };
            let Some(p) = self.players.get(&target).filter(|p| p.hp > 0) else {
                self.monsters.get_mut(&mh).unwrap().target = None;
                continue;
            };
            let ppos = p.motion.position(now);
            let mpos = m.motion.position(now);
            let d = dist(ppos, mpos);
            if d > self.visibility_distance * 1.5 {
                // Trop loin : le monstre abandonne et rentre.
                let m = self.monsters.get_mut(&mh).unwrap();
                m.target = None;
                m.hp = m.info.hp;
                let home = m.home;
                m.motion = Motion { x: mpos.0, y: mpos.1, start_time: now, points: vec![MovePoint { x: home.0, y: home.1 }], speed: m.info.run_speed };
                let packet = self.monster_move_packet(&self.monsters[&mh]);
                self.broadcast_around(mh, &packet, false);
                continue;
            }
            if d > m.info.attack_range {
                if now >= m.next_chase {
                    let m = self.monsters.get_mut(&mh).unwrap();
                    m.next_chase = now + 100;
                    // S'arrête à portée d'attaque du joueur.
                    let t = ((d - m.info.attack_range * 0.8) / d).clamp(0.0, 1.0);
                    let dest = MovePoint { x: mpos.0 + (ppos.0 - mpos.0) * t, y: mpos.1 + (ppos.1 - mpos.1) * t };
                    m.motion = Motion { x: mpos.0, y: mpos.1, start_time: now, points: vec![dest], speed: m.info.run_speed };
                    let packet = self.monster_move_packet(&self.monsters[&mh]);
                    self.broadcast_around(mh, &packet, false);
                }
                continue;
            }
            if now < m.next_attack {
                continue;
            }
            let (attack, m_hp, m_mp) = (m.info.attack, m.hp, m.info.mp);
            self.monsters.get_mut(&mh).unwrap().next_attack = now + MONSTER_ATTACK_DELAY;
            let p = self.players.get_mut(&target).unwrap();
            let damage = roll_damage(attack, p.defence);
            p.hp = (p.hp - damage).max(0);
            p.last_combat = now;
            let (p_hp, p_max, p_mp, p_mmax) = (p.hp, p.max_hp, p.mp, p.max_mp);
            let event = game::ScAttackEvent {
                attacker: mh,
                target,
                attack_speed: 100,
                attack_delay: MONSTER_ATTACK_DELAY as u16 * 10,
                attack_action: game::attack_action::ATTACK,
                attack_flag: 0,
                attacks: vec![AttackInfo {
                    damage,
                    target_hp: p_hp as i64,
                    target_mp: p_mp,
                    attacker_hp: m_hp as i64,
                    attacker_mp: m_mp,
                    ..Default::default()
                }],
            }
            .to_packet();
            self.broadcast_around(target, &event, true);
            self.broadcast_around(target, &Self::hpmp_packet(target, p_hp, p_max, p_mp, p_mmax, -damage), true);
            if p_hp == 0 {
                let p = &self.players[&target];
                let _ = p.events.send(WorldEvent::Died);
                self.monsters.get_mut(&mh).unwrap().target = None;
            }
        }
    }

    fn tick_regen(&mut self, now: u32) {
        let mut packets = Vec::new();
        for p in self.players.values_mut() {
            if p.hp <= 0 || now.saturating_sub(p.last_combat) < 500 || (p.hp >= p.max_hp && p.mp >= p.max_mp) {
                continue;
            }
            let add = (p.max_hp / 20).max(1);
            p.hp = (p.hp + add).min(p.max_hp);
            p.mp = (p.mp + (p.max_mp / 20).max(1)).min(p.max_mp);
            packets.push((p.handle, Self::hpmp_packet(p.handle, p.hp, p.max_hp, p.mp, p.max_mp, 0)));
        }
        for (h, packet) in packets {
            self.broadcast_around(h, &packet, true);
        }
    }

    /// Résurrection après la mort : PV/PM au maximum, renvoi au point indiqué.
    pub fn revive(&mut self, handle: u32, x: f32, y: f32) {
        if let Some(p) = self.players.get_mut(&handle) {
            p.hp = p.max_hp;
            p.mp = p.max_mp;
            p.attack_target = None;
            let packet = Self::hpmp_packet(handle, p.hp, p.max_hp, p.mp, p.max_mp, 0);
            self.broadcast_around(handle, &packet, true);
        }
        self.teleport(handle, x, y, 0.0);
    }

    pub fn monster_count(&self) -> usize {
        self.monsters.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn motion_interpolation() {
        let m = Motion {
            x: 0.0,
            y: 0.0,
            start_time: 100,
            points: vec![MovePoint { x: 100.0, y: 0.0 }, MovePoint { x: 100.0, y: 100.0 }],
            speed: 100.0,
        };
        assert_eq!(m.position(100), (0.0, 0.0));
        assert_eq!(m.position(150), (50.0, 0.0));
        assert_eq!(m.position(250), (100.0, 50.0));
        assert_eq!(m.position(10_000), (100.0, 100.0));
    }

    #[test]
    fn damage_is_positive() {
        for _ in 0..100 {
            assert!(roll_damage(1, 1000) >= 1);
            let d = roll_damage(100, 20);
            assert!((80..=100).contains(&d));
        }
    }
}
