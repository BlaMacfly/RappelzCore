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

    /// A snapshot already supplies the current position; do not replay reached waypoints.
    pub(crate) fn remaining_points(&self, now: u32) -> Vec<MovePoint> {
        if self.speed <= 0.0 { return Vec::new(); }
        let mut budget = self.speed * now.saturating_sub(self.start_time) as f32 / 100.0;
        let (mut x, mut y) = (self.x, self.y);
        for (index, point) in self.points.iter().enumerate() {
            let distance = dist((x, y), (point.x, point.y));
            if distance > budget { return self.points[index..].to_vec(); }
            budget -= distance;
            (x, y) = (point.x, point.y);
        }
        Vec::new()
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
    pub(crate) corpse_consumed:bool,
    pub(crate) corpse_generation:u64,
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
    pub(crate) casts:HashMap<u32,crate::casting::PendingCast>,
    pub(crate) cast_cooldowns:HashMap<(u32,u32),u64>,
    pub(crate) parties: crate::party::Parties,
    pub(crate) party_clients: HashSet<u32>,
    pub(crate) party_sent: HashMap<u32,rz_proto::party::ScPartyState>,
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
            parties:Default::default(),party_clients:HashSet::new(),party_sent:HashMap::new(),
            started: Instant::now(),
            players: HashMap::new(),
            npcs,
            monsters: HashMap::new(),
            next_handle: 1,
            next_monster_handle: MONSTER_HANDLE_BASE,
            visibility_distance,
            max_online: 0,
            last_regen: 0, casts:HashMap::new(),cast_cooldowns:HashMap::new(),
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
    pub(crate) fn cast_clock(&self)->u64 {(self.started.elapsed().as_millis()/10) as u64}

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
                corpse_consumed:false,corpse_generation:0,
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
        self.cancel_restoration(handle);
        let p = self.players.remove(&handle)?;
        self.parties.disconnected(handle);
        self.party_clients.remove(&handle);self.party_sent.remove(&handle);
        self.publish_parties();
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
        let mut enter = p.enter.clone();
        // Combat/regen update the live entity; the appearance cache is not a vitals store.
        enter.creature.hp = p.hp;
        enter.creature.max_hp = p.max_hp;
        enter.creature.mp = p.mp;
        enter.creature.max_mp = p.max_mp;
        ScEnter { enter_type: 0, handle: p.handle, x, y, z: p.z, layer: p.layer, kind: EnterKind::Player(enter) }
            .to_packet()
    }

    fn wear_packet(p: &PlayerEntity) -> RawPacket {
        game::ScWearInfo { handle: p.handle, wear_info: p.wear, ..Default::default() }.to_packet()
    }

    /// Paquets d'apparition d'un objet du monde (ENTER + données associées).
    fn enter_packets(&self, handle: u32, now: u32) -> Vec<RawPacket> {
        if let Some(p) = self.players.get(&handle) {
            let mut packets = vec![self.player_enter(p, now), Self::wear_packet(p)];
            if let Some(movement) = Self::move_packet(handle, p.layer, &p.motion, now) { packets.push(movement); }
            return packets;
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
            if let Some(movement) = Self::move_packet(handle, 0, &m.motion, now) {
                out.push(movement);
            }
            return out;
        }
        Vec::new()
    }

    /// Réponse à TS_CS_QUERY : renvoie l'ENTER d'un objet visible.
    pub fn query(&self, viewer: u32, handle: u32) {
        let now = self.now();
        if !self.can_see(viewer, handle, now) {
            return;
        }
        for p in self.enter_packets(handle, now) {
            self.send_to(viewer, p);
        }
    }

    /// Require an announced object and current visibility: the cached set can lag a tick.
    pub fn can_see(&self, viewer: u32, handle: u32, now: u32) -> bool {
        let Some(player) = self.players.get(&viewer) else { return false };
        if viewer == handle { return true; }
        if !player.visible.contains(&handle) { return false; }
        let position = if let Some(other) = self.players.get(&handle) {
            if other.layer != player.layer { return false; }
            other.motion.position(now)
        } else if player.layer != 0 {
            return false;
        } else if let Some(npc) = self.npcs.get(&handle) {
            (npc.x, npc.y)
        } else if let Some(monster) = self.monsters.get(&handle) {
            if monster.corpse_consumed {return false;}
            if !monster.alive() && !monster.dead_since.is_some_and(|t| now.saturating_sub(t) < MONSTER_CORPSE_TIME) {
                return false;
            }
            monster.motion.position(now)
        } else {
            return false;
        };
        dist(player.motion.position(now), position) <= self.visibility_distance
    }

    pub fn visible_npc(&self, viewer: u32, handle: u32) -> Option<(u32,String)> {
        if !self.can_see(viewer,handle,self.now()) { return None; }
        self.npcs.get(&handle).map(|npc|(npc.npc_id,npc.name.clone()))
    }

    fn move_packet(handle: u32, layer: u8, motion: &Motion, now: u32) -> Option<RawPacket> {
        let points = motion.remaining_points(now);
        if points.is_empty() { return None; }
        Some(msg::ScMove { start_time: now, handle, tlayer: layer,
            speed: motion.speed.clamp(1.0, 255.0) as u8, points }.to_packet())
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
                    let corpse = !m.corpse_consumed && m.dead_since.is_some_and(|t| now.saturating_sub(t) < MONSTER_CORPSE_TIME);
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

    /// Téléporte un joueur (TS_SC_WARP) et synchronise immédiatement ses observateurs.
    pub fn teleport(&mut self, handle: u32, x: f32, y: f32, z: f32) {
        self.cancel_restoration(handle);
        if !self.players.contains_key(&handle) { return; }
        let previous_viewers: Vec<u32> = self.players.values()
            .filter(|p| p.visible.contains(&handle)).map(|p| p.handle).collect();
        let p = self.players.get_mut(&handle).unwrap();
        p.motion = Motion::at(x, y);
        p.z = z;
        let _ = p.tx.send(msg::ScWarp { x, y, z, layer: p.layer }.to_packet());
        // Send LEAVE/ENTER immediately to viewers losing/gaining this player.
        self.update_visibility();
        // Viewers remaining nearby need a fresh position too; WARP addresses only its owner.
        let packets = self.enter_packets(handle, self.now());
        for viewer in previous_viewers {
            if self.players.get(&viewer).is_some_and(|p| p.visible.contains(&handle)) {
                for packet in &packets { self.send_to(viewer, packet.clone()); }
            }
        }
    }

    /// Server-authoritative phase transition; ordinary gameplay cannot select arbitrary layers.
    pub fn change_layer(&mut self, handle: u32, layer: u8) -> bool {
        self.cancel_restoration(handle);
        let now = self.now();
        let Some(player) = self.players.get_mut(&handle) else { return false };
        if player.layer == layer { return true; }
        let (x, y) = player.motion.position(now);
        let z = player.z;
        player.layer = layer;
        player.attack_target = None;
        for monster in self.monsters.values_mut() {
            if monster.target == Some(handle) {
                monster.target = None;
                monster.motion.settle(now);
            }
        }
        self.teleport(handle, x, y, z);
        true
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

    /// Apply level-derived stats and publish the same authoritative state to all viewers.
    pub fn set_player_level(&mut self, handle: u32, level: u8, job_level: i32) -> bool {
        let Some(player) = self.players.get_mut(&handle) else { return false };
        player.max_hp = crate::gameplay::max_hp_for(level);
        player.max_mp = crate::gameplay::max_mp_for(level);
        player.hp = player.max_hp;
        player.mp = player.max_mp;
        player.attack = crate::gameplay::attack_for(level);
        player.defence = crate::gameplay::defence_for(level);
        let creature = &mut player.enter.creature;
        creature.level = level as i32;
        creature.hp = player.hp;
        creature.max_hp = player.max_hp;
        creature.mp = player.mp;
        creature.max_mp = player.max_mp;
        let hpmp = Self::hpmp_packet(handle, player.hp, player.max_hp, player.mp, player.max_mp, 0);
        self.broadcast_around(handle, &game::ScLevelUpdate { handle, level: level as i32, job_level }.to_packet(), true);
        self.broadcast_around(handle, &hpmp, true);
        self.send_to(handle, msg::ScProperty { handle, name: "lv".into(), value: level as i64, text: None }.to_packet());
        true
    }

    // ───────────────────────── Combat ─────────────────────────

    pub fn start_attack(&mut self, attacker: u32, target: u32) -> bool {
        let alive = self.monsters.get(&target).is_some_and(|m| m.alive())
            && self.players.get(&attacker).is_some_and(|p| p.hp > 0)
            && self.can_see(attacker, target, self.now());
        if alive {self.cancel_restoration(attacker);}
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
        self.tick_restorations(self.cast_clock());
        self.tick_player_attacks(now);
        self.tick_monsters(now);
        if now.saturating_sub(self.last_regen) >= REGEN_INTERVAL {
            self.last_regen = now;
            self.tick_regen(now);
        }
        self.update_visibility();
        self.publish_parties();
    }

    fn tick_player_attacks(&mut self, now: u32) {
        let attackers: Vec<(u32, u32)> = self
            .players
            .values()
            .filter(|p| p.attack_target.is_some() && now >= p.next_attack && p.hp > 0)
            .map(|p| (p.handle, p.attack_target.unwrap()))
            .collect();
        for (ph, mh) in attackers {
            if !self.can_see(ph, mh, now) {
                self.players.get_mut(&ph).unwrap().attack_target = None;
                continue;
            }
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
        m.corpse_consumed=false;m.corpse_generation=m.corpse_generation.saturating_add(1);
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
                    m.corpse_consumed=false;
                    m.hp = m.info.hp;
                    m.motion = Motion::at(m.home.0, m.home.1);
                }
                continue;
            }
            let Some(target) = m.target else { continue };
            let Some(p) = self.players.get(&target).filter(|p| p.hp > 0 && p.layer == 0) else {
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
                if let Some(packet) = Self::move_packet(mh, 0, &self.monsters[&mh].motion, now) {
                    self.broadcast_around(mh, &packet, false);
                }
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
                    if let Some(packet) = Self::move_packet(mh, 0, &self.monsters[&mh].motion, now) {
                        self.broadcast_around(mh, &packet, false);
                    }
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
        if !self.players.contains_key(&handle) { return; }
        if let Some(p) = self.players.get_mut(&handle) {
            p.hp = p.max_hp;
            p.mp = p.max_mp;
            p.attack_target = None;
            let packet = Self::hpmp_packet(handle, p.hp, p.max_hp, p.mp, p.max_mp, 0);
            self.broadcast_around(handle, &packet, true);
        }
        let now = self.now();
        for monster in self.monsters.values_mut() {
            if monster.target == Some(handle) {
                monster.target = None;
                monster.motion.settle(now);
            }
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

    fn test_world() -> (World, Vec<tokio::sync::mpsc::UnboundedReceiver<RawPacket>>) {
        let mut world = World {
            parties:Default::default(),party_clients:HashSet::new(),party_sent:HashMap::new(),
            started: Instant::now(), players: HashMap::new(), npcs: HashMap::new(), monsters: HashMap::new(),
            next_handle: 3, next_monster_handle: MONSTER_HANDLE_BASE, visibility_distance: 540.0,
            max_online: 0, last_regen: 0, casts:HashMap::new(),cast_cooldowns:HashMap::new(),
        };
        let mut receivers = Vec::new();
        for handle in [1, 2] {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            receivers.push(rx);
            world.players.insert(handle, PlayerEntity {
                handle, guid: handle, account_id: handle, name: format!("Player{handle}"), gm_level: 0,
                z: 0.0, layer: 0, motion: Motion::at(0.0, 0.0), enter: PlayerEnterInfo::default(),
                wear: [0; game::WEAR_SLOTS], tx, events: tokio::sync::mpsc::unbounded_channel().0,
                visible: if handle == 2 { HashSet::from([1]) } else { HashSet::new() },
                hp: 50, max_hp: 100, mp: 20, max_mp: 50, attack: 16, defence: 6,
                attack_target: None, next_attack: 0, last_combat: 0,
            });
        }
        (world, receivers)
    }

    #[test]
    fn level_change_synchronizes_owner_observer_and_future_enter_packets() {
        let (mut world, mut receivers) = test_world();
        assert!(world.set_player_level(1, 2, 7));
        for rx in &mut receivers {
            let level = game::ScLevelUpdate::from_packet(&rx.try_recv().unwrap()).unwrap();
            assert_eq!((level.handle, level.level, level.job_level), (1, 2, 7));
            let hpmp = game::ScHpMp::from_packet(&rx.try_recv().unwrap()).unwrap();
            assert_eq!((hpmp.handle, hpmp.hp, hpmp.max_hp, hpmp.mp, hpmp.max_mp), (1, 120, 120, 60, 60));
        }
        let property = msg::ScProperty::from_packet(&receivers[0].try_recv().unwrap()).unwrap();
        assert_eq!((property.name.as_str(), property.value), ("lv", 2));
        assert!(receivers[1].try_recv().is_err());
        world.query(2, 1);
        let enter = msg::ScEnter::from_packet(&receivers[1].try_recv().unwrap()).unwrap();
        let EnterKind::Player(player) = enter.kind else { panic!("expected player ENTER") };
        assert_eq!((player.creature.level, player.creature.hp, player.creature.max_hp, player.creature.mp, player.creature.max_mp),
            (2, 120, 120, 60, 60));
        assert!(!world.set_player_level(99, 2, 1));
    }

    #[test]
    fn party_state_requires_opt_in_consent_and_stays_private_across_layers_and_logout() {
        use rz_proto::party::{ScPartyState,SC_PARTY_STATE,VERSION};
        let (mut world,mut receivers)=test_world();
        let (mut other,_)=test_world();let mut outsider=other.players.remove(&1).unwrap();
        let (tx,mut outsider_rx)=tokio::sync::mpsc::unbounded_channel();
        outsider.handle=3;outsider.name="Outsider".into();outsider.tx=tx;world.add_player(outsider);
        for p in world.players.values_mut() {p.enter.creature.level=1;}
        world.update_visibility();
        for rx in &mut receivers {while rx.try_recv().is_ok(){}}while outsider_rx.try_recv().is_ok(){}
        world.publish_parties();assert!(receivers.iter_mut().all(|rx|rx.try_recv().is_err()));
        world.enable_party(1,VERSION+1);assert!(receivers[0].try_recv().is_err());
        world.enable_party(1,VERSION);world.enable_party(2,VERSION);
        for rx in &mut receivers {assert_eq!(ScPartyState::from_packet(&rx.try_recv().unwrap()).unwrap(),ScPartyState::default());}
        assert!(outsider_rx.try_recv().is_err());
        world.party_command(1,&["invite","Player2"]);
        assert!(world.parties.group(1).is_none());
        let pending:Vec<_>=std::iter::from_fn(||receivers[1].try_recv().ok()).collect();
        let state=ScPartyState::from_packet(pending.iter().find(|p|p.id==SC_PARTY_STATE).unwrap()).unwrap();
        assert_eq!(state.invite.unwrap().inviter,1);assert!(state.members.is_empty());
        world.party_command(2,&["accept"]);
        for rx in &mut receivers {
            let packets:Vec<_>=std::iter::from_fn(||rx.try_recv().ok()).collect();
            let state=ScPartyState::from_packet(packets.iter().rev().find(|p|p.id==SC_PARTY_STATE).unwrap()).unwrap();
            assert_eq!(state.members.len(),2);assert_eq!(state.leader,1);assert!(state.invite.is_none());
        }
        world.party_command(2,&["disband"]);assert!(world.parties.group(1).is_some());
        world.players.get_mut(&2).unwrap().hp=25;world.players.get_mut(&2).unwrap().mp=4;
        world.change_layer(2,1);world.publish_parties();
        let packets:Vec<_>=std::iter::from_fn(||receivers[0].try_recv().ok()).collect();
        let state=ScPartyState::from_packet(packets.iter().rev().find(|p|p.id==SC_PARTY_STATE).unwrap()).unwrap();
        assert_eq!((state.members[1].hp,state.members[1].mp,state.members[1].layer),(25,4,1));
        assert!(!world.can_see(1,2,world.now()));
        for rx in &mut receivers {while rx.try_recv().is_ok(){}}while outsider_rx.try_recv().is_ok(){}
        world.party_chat(1,"private group message");
        for rx in &mut receivers {assert_eq!(msg::ScChat::from_packet(&rx.try_recv().unwrap()).unwrap().message,"private group message");}
        assert!(outsider_rx.try_recv().is_err());
        world.party_chat(3,"injected outsider message");assert!(receivers.iter_mut().all(|rx|rx.try_recv().is_err()));
        world.remove_player(1);
        let packets:Vec<_>=std::iter::from_fn(||receivers[1].try_recv().ok()).collect();
        assert_eq!(ScPartyState::from_packet(packets.iter().find(|p|p.id==SC_PARTY_STATE).unwrap()).unwrap(),ScPartyState::default());
        assert!(!world.party_clients.contains(&1));assert!(!world.party_sent.contains_key(&1));
    }

    fn isolated_session(skills:Option<rz_rappelz::skills::SkillBook>)->(crate::session::Session,tokio::sync::mpsc::UnboundedReceiver<RawPacket>) {
        use crate::{Ctx,Settings,session::{Session,InGame},gameplay::InvItem};
        use std::{sync::{Arc,Mutex,RwLock,atomic::AtomicU32},time::Duration};
        let (world,mut receivers)=test_world();let tx=world.players[&1].tx.clone();
        // No real database is reachable by this fixture; any SQL path is a test failure.
        let pool=rz_database::sqlx::mysql::MySqlPoolOptions::new().acquire_timeout(Duration::from_millis(20))
            .connect_lazy("mysql://unused:unused@127.0.0.1:1/unused").unwrap();
        let data=GameData {restoration:None,skills,jobs:HashMap::new(),npcs:vec![],teleports:vec![],strings:rz_rappelz::dbcstore::StringTable::empty(),
            monsters:HashMap::new(),items:HashMap::new(),vendors:RwLock::new(HashMap::new()),monster_spawns:vec![]};
        let settings=Settings {realm_id:1,network_key:vec![],characters_per_realm:1,start_level:1,start_money:0,start_positions:[(0.0,0.0);3],
            region_size:180,cell_size:6,move_speed:120,speed_scale:1.0,move_tolerance:60.0,save_interval:Duration::from_secs(60),player_limit:3,motd:String::new()};
        let ctx=Arc::new(Ctx {login:pool.clone(),chars:pool.clone(),world_db:pool,data,world:Mutex::new(world),settings,
            save_all:tokio::sync::broadcast::channel(4).0,shutdown:tokio::sync::watch::channel(false).0,
            character_guid:AtomicU32::new(1),item_guid:AtomicU32::new(99),creature_guid:AtomicU32::new(0)});
        let item=InvItem {guid:99,entry:500,count:7,slot:28};
        let character=rz_database::characters::Character {guid:1,account:1,level:1,health:100,power1:50,..Default::default()};
        let session=Session {ctx,addr:"127.0.0.1:1".parse().unwrap(),tx,account:None,gm_level:0,events_rx:None,dialog_npc:None,skill_catalog:None,
            in_game:Some(InGame {handle:1,character,items:vec![item]})};
        (session,receivers.remove(0))
    }

    #[tokio::test]
    async fn unsupported_ground_drop_keeps_inventory_and_never_attempts_a_database_mutation() {
        let (mut session,mut rx)=isolated_session(None);
        let item_handle=session.in_game.as_ref().unwrap().items[0].handle();
        for (handle,count) in [(item_handle,1),(item_handle,0),(item_handle,u32::MAX),(123,1)] {
            session.handle_gameplay(game::CsDropItem {item_handle:handle,count}.to_packet()).await.unwrap();
            let items=&session.in_game.as_ref().unwrap().items;
            assert_eq!(items.len(),1);assert_eq!((items[0].guid,items[0].entry,items[0].count,items[0].slot),(99,500,7,28));
            let notice=msg::ScChat::from_packet(&rx.try_recv().unwrap()).unwrap();
            assert!(notice.message.contains("L'objet reste dans le sac"));assert!(rx.try_recv().is_err());
        }
    }

    #[tokio::test]
    async fn failed_learning_commit_keeps_jp_skills_and_session_snapshot_unchanged() {
        use rz_rappelz::skills::{SkillBook,SkillDefinition,LearningRule};
        let book=SkillBook::from_parts(vec![SkillDefinition {id:42,valid:true,passive:false,learning_jp:[3;50]}],
            vec![LearningRule {tree:10,skill:42,group:0,min:1,max:3,player_level:1,job_level:1,ratio:1.0,prerequisites:[(0,0);3],enhance_min:0,enhance_max:5}],vec![(100,10)]);
        let (mut session,mut rx)=isolated_session(Some(book));
        let c=&mut session.in_game.as_mut().unwrap().character;c.set_extra("job",100);c.set_extra("jlv",1);c.set_extra("jp",10);
        let before=c.clone();
        session.learn_skill(game::CsLearnSkill {target:1,skill_id:42,base_random_skill_id:0,skill_level:1}).await.unwrap();
        let after=&session.in_game.as_ref().unwrap().character;
        assert_eq!(after.extra,before.extra);assert_eq!(after.health,before.health);assert_eq!(after.power1,before.power1);
        let packets:Vec<_>=std::iter::from_fn(||rx.try_recv().ok()).collect();
        assert_eq!(packets.len(),2);assert!(packets.iter().all(|p|p.id!=game::ScSkillList::ID && p.id!=game::ScExpUpdate::ID));
        let result=msg::ScResult::from_packet(&packets[0]).unwrap();assert_eq!(result.request,game::id::TS_CS_LEARN_SKILL);assert_ne!(result.result,0);
        assert!(msg::ScChat::from_packet(&packets[1]).unwrap().message.contains("aucun JP débité"));
    }

    #[tokio::test]
    async fn reward_keeps_wide_job_points_nonnegative_without_integer_wrap() {
        let (mut session,mut rx)=isolated_session(None);
        session.in_game.as_mut().unwrap().character.set_extra("jp",i64::MAX-1);
        session.handle_world_event(WorldEvent::Reward {exp:0,jp:3,gold:0,monster:"Fixture".into()}).await.unwrap();
        assert_eq!(session.in_game.as_ref().unwrap().character.extra_i64("jp",0),i64::MAX);
        let packets:Vec<_>=std::iter::from_fn(||rx.try_recv().ok()).collect();
        let exp=game::ScExpUpdate::from_packet(packets.iter().find(|p|p.id==game::ScExpUpdate::ID).unwrap()).unwrap();
        assert_eq!(exp.jp,i64::MAX);
    }

    #[test]
    fn query_and_new_visibility_read_current_vitals_after_combat_or_regeneration() {
        let (mut world, mut receivers) = test_world();
        let player = world.players.get_mut(&1).unwrap();
        player.enter.creature.hp = 100;
        player.enter.creature.mp = 50;
        player.hp = 43;
        player.mp = 12;
        world.query(2, 1);
        let enter = msg::ScEnter::from_packet(&receivers[1].try_recv().unwrap()).unwrap();
        let EnterKind::Player(player) = enter.kind else { panic!("expected player") };
        assert_eq!((player.creature.hp, player.creature.max_hp, player.creature.mp, player.creature.max_mp), (43, 100, 12, 50));
        while receivers[1].try_recv().is_ok() {}
        let player = world.players.get_mut(&1).unwrap();
        player.hp = 48;
        player.mp = 15;
        world.players.get_mut(&2).unwrap().visible.clear();
        world.update_visibility();
        let enter = msg::ScEnter::from_packet(&receivers[1].try_recv().unwrap()).unwrap();
        let EnterKind::Player(player) = enter.kind else { panic!("expected player") };
        assert_eq!((player.creature.hp, player.creature.mp), (48, 15));
    }

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

    fn test_monster(world: &mut World) -> u32 {
        world.spawn_monster(MonsterInfo {
            id: 711, name: "Fixture".into(), level: 1, hp: 10, mp: 5,
            attack: 20, defence: 6, exp: 5, jp: 3, gold_min: 0, gold_max: 0,
            run_speed: 100.0, attack_range: 20.0, race: 0,
        }, 10.0, 0.0, 0.0, 15)
    }

    #[test]
    fn queries_reject_stale_range_layer_and_expired_corpse_visibility() {
        let (mut world, mut receivers) = test_world();
        let monster = test_monster(&mut world);
        world.update_visibility();
        for rx in &mut receivers { while rx.try_recv().is_ok() {} }
        // Cached visibility has not yet caught up with these live changes.
        world.players.get_mut(&1).unwrap().motion = Motion::at(1000.0, 0.0);
        world.query(2, 1);
        assert!(receivers[1].try_recv().is_err(), "out-of-range player disclosed");
        world.players.get_mut(&1).unwrap().motion = Motion::at(0.0, 0.0);
        world.players.get_mut(&1).unwrap().layer = 1;
        world.query(2, 1);
        assert!(receivers[1].try_recv().is_err(), "other-layer player disclosed");
        world.started = Instant::now() - std::time::Duration::from_secs(4);
        world.monsters.get_mut(&monster).unwrap().dead_since = Some(0);
        world.monsters.get_mut(&monster).unwrap().hp = 0;
        world.query(2, monster);
        assert!(receivers[1].try_recv().is_err(), "expired corpse reintroduced");
        world.monsters.get_mut(&monster).unwrap().dead_since = Some(world.now());
        world.query(2, monster);
        assert!(receivers[1].try_recv().is_ok(), "recent visible corpse should be queryable");
        while receivers[1].try_recv().is_ok() {}
        world.players.get_mut(&2).unwrap().visible.remove(&monster);
        world.query(2, monster);
        assert!(receivers[1].try_recv().is_err(), "unannounced object disclosed");
    }

    #[test]
    fn combat_rechecks_live_visibility_and_stops_across_layers() {
        let (mut world, _) = test_world();
        let monster = test_monster(&mut world);
        world.update_visibility();
        assert!(world.start_attack(1, monster));
        world.players.get_mut(&1).unwrap().layer = 1;
        world.tick_player_attacks(1);
        assert_eq!(world.monsters[&monster].hp, 10, "attack crossed layers");
        assert_eq!(world.players[&1].attack_target, None);
        assert!(!world.start_attack(1, monster));
        world.monsters.get_mut(&monster).unwrap().target = Some(1);
        world.tick_monsters(1);
        assert_eq!(world.players[&1].hp, 50, "retaliation crossed layers");
        assert_eq!(world.monsters[&monster].target, None);
        world.players.get_mut(&1).unwrap().layer = 0;
        world.players.get_mut(&1).unwrap().hp = 0;
        assert!(!world.start_attack(1, monster), "dead player armed an attack");
        world.players.get_mut(&1).unwrap().hp = 50;
        world.players.get_mut(&1).unwrap().motion = Motion::at(1000.0, 0.0);
        assert!(!world.start_attack(1, monster), "hidden monster targeted");
    }

    #[test]
    fn corpse_leave_respawn_and_second_kill_award_once_each() {
        let (mut world, mut receivers) = test_world();
        let monster = test_monster(&mut world);
        let (events, mut rewards) = tokio::sync::mpsc::unbounded_channel();
        world.players.get_mut(&1).unwrap().events = events;
        world.update_visibility();
        for rx in &mut receivers { while rx.try_recv().is_ok() {} }
        assert!(world.start_attack(1, monster));
        world.tick_player_attacks(0);
        assert_eq!(world.monsters[&monster].hp, 0);
        assert!(matches!(rewards.try_recv().unwrap(), WorldEvent::Reward { exp: 5, jp: 3, .. }));
        world.tick_player_attacks(150);
        assert!(rewards.try_recv().is_err());
        world.started = Instant::now() - std::time::Duration::from_secs(4);
        world.update_visibility();
        for rx in &mut receivers {
            let packets: Vec<_> = std::iter::from_fn(|| rx.try_recv().ok()).collect();
            assert!(packets.iter().any(|p| p.id == msg::ScLeave::ID
                && msg::ScLeave::from_packet(p).unwrap().handle == monster));
        }
        world.tick_monsters(1499);
        assert!(!world.monsters[&monster].alive());
        world.tick_monsters(1500);
        world.started = Instant::now() - std::time::Duration::from_secs(15);
        world.update_visibility();
        for rx in &mut receivers {
            let enter = msg::ScEnter::from_packet(&rx.try_recv().unwrap()).unwrap();
            assert_eq!(enter.handle, monster);
            let EnterKind::Monster { creature, .. } = enter.kind else { panic!("expected monster") };
            assert_eq!((creature.hp, creature.mp), (10, 5));
        }
        assert!(world.start_attack(1, monster));
        world.tick_player_attacks(1500);
        assert!(matches!(rewards.try_recv().unwrap(), WorldEvent::Reward { exp: 5, jp: 3, .. }));
        assert!(rewards.try_recv().is_err());
    }

    #[test]
    fn revival_refreshes_nearby_observers_and_clears_combat() {
        let (mut world, mut receivers) = test_world();
        let monster = test_monster(&mut world);
        world.monsters.get_mut(&monster).unwrap().target = Some(1);
        world.update_visibility();
        for rx in &mut receivers { while rx.try_recv().is_ok() {} }
        let player = world.players.get_mut(&1).unwrap();
        player.hp = 0;
        player.mp = 1;
        player.attack_target = Some(monster);
        world.revive(1, 120.0, 30.0);
        assert_eq!(world.players[&1].attack_target, None);
        let own: Vec<_> = std::iter::from_fn(|| receivers[0].try_recv().ok()).collect();
        assert_eq!(own[0].id, game::ScHpMp::ID);
        let warp = msg::ScWarp::from_packet(&own[1]).unwrap();
        assert_eq!((warp.x, warp.y), (120.0, 30.0));
        let packets: Vec<_> = std::iter::from_fn(|| receivers[1].try_recv().ok()).collect();
        let enter = packets.iter().find(|p| p.id == msg::ScEnter::ID).expect("observer did not receive the revived position");
        let enter = msg::ScEnter::from_packet(enter).unwrap();
        assert_eq!((enter.handle, enter.x, enter.y), (1, 120.0, 30.0));
        let EnterKind::Player(player) = enter.kind else { panic!("expected player") };
        assert_eq!((player.creature.hp, player.creature.mp), (100, 50));
        assert!(!packets.iter().any(|p| p.id == msg::ScLeave::ID), "nearby target should remain selected");
        assert_eq!(world.monsters[&monster].target, None, "revived player retained an aggressor");
    }

    #[test]
    fn teleport_sends_leave_and_enter_to_the_correct_observers_immediately() {
        let (mut world, mut receivers) = test_world();
        let (mut other_world, mut other_receivers) = test_world();
        let mut newcomer = other_world.players.remove(&2).unwrap();
        newcomer.handle = 3;
        newcomer.motion = Motion::at(1000.0, 0.0);
        newcomer.visible.clear();
        world.add_player(newcomer);
        world.update_visibility();
        for rx in receivers.iter_mut().chain(other_receivers.iter_mut()) { while rx.try_recv().is_ok() {} }
        world.teleport(1, 1000.0, 0.0, 0.0);
        let former: Vec<_> = std::iter::from_fn(|| receivers[1].try_recv().ok()).collect();
        assert!(former.iter().any(|p| p.id == msg::ScLeave::ID
            && msg::ScLeave::from_packet(p).unwrap().handle == 1), "old observer retained teleported player");
        assert!(!former.iter().any(|p| p.id == msg::ScEnter::ID));
        let entered: Vec<_> = std::iter::from_fn(|| other_receivers[1].try_recv().ok()).collect();
        assert_eq!(entered.iter().filter(|p| p.id == msg::ScEnter::ID).count(), 1);
        let enter = msg::ScEnter::from_packet(entered.iter().find(|p| p.id == msg::ScEnter::ID).unwrap()).unwrap();
        assert_eq!((enter.handle, enter.x), (1, 1000.0));
    }

    #[test]
    fn moving_player_query_and_new_visibility_resume_only_the_unfinished_route() {
        let (mut world, mut receivers) = test_world();
        world.players.get_mut(&1).unwrap().motion = Motion {
            x: 0.0, y: 0.0, start_time: 0, speed: 100.0,
            points: vec![MovePoint { x: 100.0, y: 0.0 }, MovePoint { x: 300.0, y: 0.0 }],
        };
        world.started = Instant::now() - std::time::Duration::from_millis(1500);
        world.query(2, 1);
        let packets: Vec<_> = std::iter::from_fn(|| receivers[1].try_recv().ok()).collect();
        let entered = msg::ScEnter::from_packet(&packets[0]).unwrap();
        assert!(entered.x >= 100.0 && entered.x < 300.0);
        let movement = packets.iter().find(|p| p.id == msg::ScMove::ID).expect("moving player ENTER omitted remaining motion");
        let movement = msg::ScMove::from_packet(movement).unwrap();
        assert_eq!(movement.points, [MovePoint { x: 300.0, y: 0.0 }]);
        assert_eq!(movement.speed, 100);
        assert!(movement.start_time >= 150);
        world.players.get_mut(&2).unwrap().visible.clear();
        world.update_visibility();
        let packets: Vec<_> = std::iter::from_fn(|| receivers[1].try_recv().ok()).collect();
        assert!(packets.iter().any(|p| p.id == msg::ScMove::ID), "late observer did not resume movement");
        world.started = Instant::now() - std::time::Duration::from_secs(4);
        world.query(2, 1);
        let packets: Vec<_> = std::iter::from_fn(|| receivers[1].try_recv().ok()).collect();
        assert!(!packets.iter().any(|p| p.id == msg::ScMove::ID), "completed route was replayed");
    }

    #[test]
    fn moving_monster_snapshot_does_not_replay_reached_or_zero_length_points() {
        let (mut world, _) = test_world();
        let handle = test_monster(&mut world);
        world.monsters.get_mut(&handle).unwrap().motion = Motion {
            x: 0.0, y: 0.0, start_time: 100, speed: 100.0,
            points: vec![MovePoint { x: 0.0, y: 0.0 }, MovePoint { x: 100.0, y: 0.0 }, MovePoint { x: 300.0, y: 0.0 }],
        };
        let packets = world.enter_packets(handle, 200);
        let movement = msg::ScMove::from_packet(packets.iter().find(|p| p.id == msg::ScMove::ID).unwrap()).unwrap();
        assert_eq!(movement.points, [MovePoint { x: 300.0, y: 0.0 }]);
        assert_eq!(movement.start_time, 200);
        assert_eq!(world.enter_packets(handle, 500).len(), 1, "finished monster motion replayed");
        world.monsters.get_mut(&handle).unwrap().motion.speed = 0.0;
        assert_eq!(world.enter_packets(handle, 100).len(), 1, "stationary monster emitted motion");
    }

    #[test]
    fn layer_change_publishes_warp_visibility_and_clears_combat_before_return() {
        let (mut world,mut receivers)=test_world();let monster=test_monster(&mut world);
        world.update_visibility();
        for rx in &mut receivers {while rx.try_recv().is_ok(){}}
        world.players.get_mut(&1).unwrap().attack_target=Some(monster);
        world.monsters.get_mut(&monster).unwrap().target=Some(1);
        assert!(world.change_layer(1,1));
        assert_eq!(world.players[&1].layer,1);
        assert!(world.players[&1].visible.is_empty());
        assert_eq!(world.players[&1].attack_target,None);
        assert_eq!(world.monsters[&monster].target,None);
        let own:Vec<_>=std::iter::from_fn(||receivers[0].try_recv().ok()).collect();
        let warp=msg::ScWarp::from_packet(&own[0]).unwrap();assert_eq!(warp.layer,1);
        let packets:Vec<_>=std::iter::from_fn(||receivers[1].try_recv().ok()).collect();
        assert!(packets.iter().any(|p|p.id==msg::ScLeave::ID && msg::ScLeave::from_packet(p).unwrap().handle==1));
        assert!(!world.can_see(2,1,world.now()));assert!(!world.can_see(1,2,world.now()));
        assert!(!world.start_attack(1,monster));
        while receivers[0].try_recv().is_ok(){}
        assert!(world.change_layer(1,0));
        let packets:Vec<_>=std::iter::from_fn(||receivers[1].try_recv().ok()).collect();
        let enter=msg::ScEnter::from_packet(packets.iter().find(|p|p.id==msg::ScEnter::ID).unwrap()).unwrap();
        assert_eq!((enter.handle,enter.layer),(1,0));
        while receivers[0].try_recv().is_ok(){}
        assert!(world.change_layer(1,0));assert!(receivers[0].try_recv().is_err());
        assert!(!world.change_layer(999,1));
    }

    #[test]
    fn npc_access_rechecks_layer_even_with_stale_announced_handle() {
        let (mut world,_)=test_world();let npc=NPC_HANDLE_BASE;
        world.npcs.insert(npc,NpcEntity {handle:npc,npc_id:1001,name:"Fixture".into(),x:0.0,y:0.0,z:0.0,creature:CreatureInfo::default()});
        world.update_visibility();assert_eq!(world.visible_npc(1,npc).unwrap().0,1001);
        world.change_layer(1,1);world.players.get_mut(&1).unwrap().visible.insert(npc);
        assert!(world.visible_npc(1,npc).is_none());
        world.change_layer(1,0);assert!(world.visible_npc(1,npc).is_some());
        world.players.get_mut(&1).unwrap().motion=Motion::at(1000.0,0.0);
        assert!(world.visible_npc(1,npc).is_none());
    }

    #[test]
    fn damage_is_positive() {
        for _ in 0..100 {
            assert!(roll_damage(1, 1000) >= 1);
            let d = roll_damage(100, 20);
            assert!((80..=100).contains(&d));
        }
    }

    fn restoration_fixture()->(rz_rappelz::restoration::Restoration,game::CsSkill) {
        (rz_rappelz::restoration::Restoration{id:3212,mp_base:0,mp_per_level:5,cast_base:150,cast_per_level:-5,
            cooldown:2500,group:103,minimum_level:0,minimum_hp:0,minimum_mp:0,amount_base:2000.,amount_per_level:1500.,..Default::default()},
        game::CsSkill{skill_id:3212,caster:1,target:1,skill_level:1,..Default::default()})
    }
    fn corpse_fixture(w:&mut World)->(u32,rz_rappelz::restoration::Quote){
        let h=test_monster(w);w.monsters.get_mut(&h).unwrap().hp=0;w.kill_monster(h,1,w.now());w.update_visibility();
        let (mut d,_)=restoration_fixture();d.corpse=true;d.range=192;d.cooldown=6000;d.cooldown_per_level=-200;d.amount_base=200.;d.amount_per_level=120.;
        (h,d.quote(1).unwrap())
    }
    #[test]
    fn corpse_requires_current_dead_visible_in_range_monster_and_rechecks_before_fire(){
        let (mut w,_)=test_world();let (h,q)=corpse_fixture(&mut w);let r=game::CsSkill{skill_id:3212,skill_level:1,caster:1,target:h,..Default::default()};
        for target in [1,2,h+1] {assert!(!w.begin_restoration(1,&game::CsSkill{target,..r.clone()},q.clone(),vec![],0));}
        w.players.get_mut(&1).unwrap().layer=1;assert!(!w.begin_restoration(1,&r,q.clone(),vec![],0));w.players.get_mut(&1).unwrap().layer=0;
        w.monsters.get_mut(&h).unwrap().hp=1;assert!(!w.begin_restoration(1,&r,q.clone(),vec![],0));w.monsters.get_mut(&h).unwrap().hp=0;
        assert!(w.begin_restoration(1,&r,q,vec![],0));w.players.get_mut(&1).unwrap().motion=Motion::at(250.,0.);w.tick_restorations(145);
        assert_eq!(w.players[&1].mp,20);assert!(!w.monsters[&h].corpse_consumed);assert!(w.cast_cooldowns.is_empty());
    }
    #[test]
    fn competing_casts_consume_one_body_and_restore_only_first_caster_without_duplicate_rewards(){
        let (mut w,mut rx)=test_world();let (h,q)=corpse_fixture(&mut w);
        for r in &mut rx {while r.try_recv().is_ok(){}}
        for caster in [1,2] {assert!(w.begin_restoration(caster,&game::CsSkill{skill_id:3212,skill_level:1,caster,target:h,..Default::default()},q.clone(),vec![],0));}
        w.tick_restorations(145);assert_eq!((w.players[&1].mp,w.players[&2].mp),(50,20));assert!(w.monsters[&h].corpse_consumed);assert!(w.casts.is_empty());
        assert!(w.cast_cooldowns.contains_key(&(1,103)));assert!(!w.cast_cooldowns.contains_key(&(2,103)));
        for r in &mut rx {let packets:Vec<_>=std::iter::from_fn(||r.try_recv().ok()).collect();
            let phases:Vec<_>=packets.iter().filter(|p|p.id==game::id::TS_SC_SKILL).map(|p|rz_proto::restoration::decode(&game::ScSkill::from_packet(p).unwrap()).unwrap()).collect();
            assert_eq!(phases,vec![rz_proto::restoration::Phase::Casting(145),rz_proto::restoration::Phase::Casting(145),rz_proto::restoration::Phase::CorpseFire{mana:50,amount:320},rz_proto::restoration::Phase::Cancel]);
            assert_eq!(packets.iter().filter(|p|p.id==msg::id::TS_SC_LEAVE && msg::ScLeave::from_packet(p).unwrap().handle==h).count(),1);
        }
        w.query(2,h);assert!(!w.can_see(2,h,w.now()));assert!(rx[1].try_recv().is_err());
    }
    #[test]
    fn corpse_expiry_or_new_death_generation_cancels_pending_cast_without_charge(){
        for expire in [false,true] {
            let (mut w,_)=test_world();let(h,q)=corpse_fixture(&mut w);let r=game::CsSkill{skill_id:3212,skill_level:1,caster:1,target:h,..Default::default()};
            assert!(w.begin_restoration(1,&r,q,vec![],0));
            if expire {w.started-=std::time::Duration::from_secs(4);}else{w.kill_monster(h,1,w.now());}
            w.tick_restorations(145);assert_eq!(w.players[&1].mp,20);assert!(!w.monsters[&h].corpse_consumed);assert!(w.cast_cooldowns.is_empty());
        }
    }
    #[test]
    fn consumed_body_keeps_original_respawn_timer_and_returns_alive_on_same_handle(){
        let(mut w,_)=test_world();let(h,q)=corpse_fixture(&mut w);let stamp=w.monsters[&h].dead_since.unwrap();let generation=w.monsters[&h].corpse_generation;
        assert!(w.begin_restoration(1,&game::CsSkill{skill_id:3212,skill_level:1,caster:1,target:h,..Default::default()},q,vec![],0));w.tick_restorations(145);
        assert_eq!(w.monsters[&h].dead_since,Some(stamp));w.tick_monsters(stamp+w.monsters[&h].respawn_secs*100);w.update_visibility();
        assert!(!w.monsters[&h].corpse_consumed);assert_eq!(w.monsters[&h].dead_since,None);assert_eq!(w.monsters[&h].hp,w.monsters[&h].info.hp);assert_eq!(w.monsters[&h].corpse_generation,generation);assert!(w.can_see(1,h,w.now()));
    }
    #[test]
    fn restoration_charges_once_at_fire_caps_mana_and_keeps_cooldown_across_handles() {
        use rz_proto::restoration::{decode,Phase};
        let (mut w,mut rx)=test_world();let (d,r)=restoration_fixture();let q=d.quote(1).unwrap();
        let known=vec![(3212,1,Some(q.clone()))];
        assert!(w.begin_restoration(1,&r,q.clone(),known.clone(),0));
        assert!(!w.begin_restoration(1,&r,q.clone(),known.clone(),0));
        assert_eq!(w.players[&1].mp,20);
        for r in &mut rx {assert_eq!(decode(&game::ScSkill::from_packet(&r.try_recv().unwrap()).unwrap()).unwrap(),Phase::Casting(145));}
        w.tick_restorations(144);assert_eq!(w.players[&1].mp,20);assert!(rx.iter_mut().all(|r|r.try_recv().is_err()));
        w.tick_restorations(145);assert_eq!(w.players[&1].mp,50);
        for r in &mut rx {assert_eq!(decode(&game::ScSkill::from_packet(&r.try_recv().unwrap()).unwrap()).unwrap(),Phase::Fire{mana:50,amount:3500});let h=game::ScHpMp::from_packet(&r.try_recv().unwrap()).unwrap();assert_eq!((h.mp,h.add_mp),(50,30));}
        let list=game::ScSkillList::from_packet(&rx[0].try_recv().unwrap()).unwrap();assert_eq!((list.skills[0].total_cool_time,list.skills[0].remain_cool_time),(2500,2500));assert!(rx[1].try_recv().is_err());
        w.tick_restorations(146);assert!(rx.iter_mut().all(|r|r.try_recv().is_err()));
        let mut p=w.remove_player(1).unwrap();p.handle=3;w.players.insert(3,p);
        let r=game::CsSkill{caster:3,target:3,..r};
        assert!(!w.begin_restoration(3,&r,q.clone(),known.clone(),200));
        assert!(w.begin_restoration(3,&r,q,known,2645));
    }
    #[test]
    fn restoration_refuses_forgery_and_rechecks_cost_and_state_before_fire() {
        let (mut w,_rx)=test_world();let (d,r)=restoration_fixture();let q=d.quote(1).unwrap();
        for invalid in [game::CsSkill{caster:2,..r.clone()},game::CsSkill{target:2,..r.clone()},game::CsSkill{layer:1,..r.clone()},game::CsSkill{x:f32::NAN,..r.clone()}] {assert!(!w.begin_restoration(1,&invalid,q.clone(),vec![],0));}
        w.players.get_mut(&1).unwrap().mp=4;assert!(!w.begin_restoration(1,&r,q.clone(),vec![],0));
        w.players.get_mut(&1).unwrap().mp=20;assert!(w.begin_restoration(1,&r,q,vec![],0));
        w.players.get_mut(&1).unwrap().mp=3;w.tick_restorations(145);
        assert_eq!(w.players[&1].mp,3);assert!(w.casts.is_empty());assert!(w.cast_cooldowns.is_empty());
    }
    #[test]
    fn canceled_hidden_layer_and_disconnected_casts_cannot_restore_or_charge() {
        let (mut w,mut rx)=test_world();let (d,r)=restoration_fixture();let q=d.quote(1).unwrap();
        assert!(w.begin_restoration(1,&r,q.clone(),vec![],0));w.cancel_restoration(1);w.tick_restorations(145);assert_eq!(w.players[&1].mp,20);assert!(w.cast_cooldowns.is_empty());
        assert!(w.begin_restoration(1,&r,q.clone(),vec![],200));w.change_layer(1,1);w.tick_restorations(345);assert_eq!(w.players[&1].mp,20);
        for r in &mut rx {while r.try_recv().is_ok(){}}
        let r=game::CsSkill{layer:1,..r};assert!(w.begin_restoration(1,&r,q.clone(),vec![],400));
        assert!(rx[1].try_recv().is_err());w.remove_player(1);w.tick_restorations(545);assert!(w.casts.is_empty());assert!(w.cast_cooldowns.is_empty());
    }
    #[tokio::test]
    async fn cast_handler_requires_learned_level_without_touching_unreachable_sql() {
        let (mut session,mut rx)=isolated_session(None);let (d,r)=restoration_fixture();
        std::sync::Arc::get_mut(&mut session.ctx).unwrap().data.restoration=Some(rz_rappelz::restoration::RestorationBook::from_definition(d));
        session.cast_skill(r.clone());assert_ne!(msg::ScResult::from_packet(&rx.try_recv().unwrap()).unwrap().result,0);
        while rx.try_recv().is_ok(){}
        session.in_game.as_mut().unwrap().character.set_extra("learned_skill_3212",1);
        session.cast_skill(game::CsSkill{skill_level:2,..r.clone()});assert_ne!(msg::ScResult::from_packet(&rx.try_recv().unwrap()).unwrap().result,0);while rx.try_recv().is_ok(){}
        session.cast_skill(r);assert_eq!(rx.try_recv().unwrap().id,game::id::TS_SC_SKILL);assert_eq!(msg::ScResult::from_packet(&rx.try_recv().unwrap()).unwrap().result,0);
        assert_eq!(session.ctx.world.lock().unwrap().players[&1].mp,20);
    }
}
