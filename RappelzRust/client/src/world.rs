//! Monde en jeu : terrain Rappelz (relief, textures de sol, décors), créatures 3D animées,
//! HUD et fenêtres d'origine (`.nui`), inventaire, dialogues PNJ, boutique, combat et chat.

use std::collections::HashMap;
use std::time::Instant;

use macroquad::prelude::*;
use rz_proto::msg::{self, chat_type, EnterKind, MovePoint};
use rz_proto::msg_game::{self as game, id as gid};
use rz_proto::{Message, RawPacket};
use rz_ui::UiState;

use crate::assets::Assets;
use crate::models::{self, ModelCache, Rig, RigKey};
use crate::net::{NetCmd, NetEvent};
use crate::rzui::RzUi;
use crate::terrain::Terrain;

const MAP_TILE: f32 = 16128.0;
const MINIMAP_PIECES: i32 = 16;
const PIECE: f32 = MAP_TILE / MINIMAP_PIECES as f32;
const MAX_CHAT_LINES: usize = 7;
/// Créatures assemblées au plus par image (le chargement des modèles est synchrone).
const RIGS_PER_FRAME: usize = 2;

#[cfg(test)]
mod addon_vitals_tests {
    use super::*;

    #[test]
    fn known_entity_vitals_reach_addon_target_and_preserve_zero_mana() {
        let mut entity = Entity::new("Peer".into(), Kind::Player, 0.0, 0.0, 2, None);
        assert_eq!((entity.unit(7).health, entity.unit(7).mana), (None, None));
        entity.set_vitals(104, 120, 60, 60);
        let unit = entity.unit(7);
        assert_eq!((unit.health, unit.max_health, unit.mana, unit.max_mana), (Some(104), Some(120), Some(60), Some(60)));
        entity.set_vitals(100, 120, 0, 0);
        assert_eq!((entity.unit(7).mana, entity.unit(7).max_mana), (Some(0), Some(0)));
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Player,
    Npc,
    Monster,
    Other,
}

#[derive(Clone)]
struct Motion {
    x: f32,
    y: f32,
    start: f64,
    points: Vec<MovePoint>,
    speed: f32,
}

impl Motion {
    fn at(x: f32, y: f32) -> Self {
        Self { x, y, start: 0.0, points: Vec::new(), speed: 0.0 }
    }

    /// Même interpolation que le worldserver (vitesse en unités par seconde).
    fn position(&self, now: f64) -> (f32, f32) {
        if self.points.is_empty() {
            return (self.x, self.y);
        }
        let mut budget = self.speed * (now - self.start).max(0.0) as f32;
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

    fn is_moving(&self, now: f64) -> bool {
        self.points.last().is_some_and(|last| {
            let (x, y) = self.position(now);
            (last.x - x).abs() > 0.5 || (last.y - y).abs() > 0.5
        })
    }

    /// Direction du déplacement en cours (pour l'orientation du modèle).
    fn heading(&self, now: f64) -> Option<f32> {
        let (x, y) = self.position(now);
        let target = self.points.iter().find(|p| (p.x - x).abs() > 0.5 || (p.y - y).abs() > 0.5)?;
        Some((target.x - x).atan2(-(target.y - y)))
    }
}

struct Entity {
    name: String,
    kind: Kind,
    motion: Motion,
    level: i32,
    hp: i32,
    max_hp: i32,
    health_known: bool,
    mana: Option<(i32, i32)>,
    say: Option<(String, f64)>,
    rig_key: Option<RigKey>,
    wear: [i32; game::WEAR_SLOTS],
    /// `None` = pas encore chargé ; `Some(None)` = pas de modèle (forme de secours).
    rig: Option<Option<Rig>>,
    facing: f32,
    dead_at: Option<f64>,
}

impl Entity {
    fn set_vitals(&mut self, hp: i32, max_hp: i32, mp: i32, max_mp: i32) {
        self.hp = hp;
        self.max_hp = max_hp.max(1);
        self.health_known = true;
        self.mana = Some((mp, max_mp));
    }

    fn unit(&self, handle: u32) -> rz_addon::UnitState {
        rz_addon::UnitState {
            handle, name: self.name.clone(), level: Some(self.level),
            health: self.health_known.then_some(self.hp), max_health: self.health_known.then_some(self.max_hp),
            mana: self.mana.map(|(current, _)| current), max_mana: self.mana.map(|(_, maximum)| maximum),
        }
    }

    fn new(name: String, kind: Kind, x: f32, y: f32, level: i32, rig_key: Option<RigKey>) -> Self {
        Self {
            name,
            kind,
            motion: Motion::at(x, y),
            level,
            hp: 1,
            max_hp: 1,
            health_known: false,
            mana: None,
            say: None,
            rig_key,
            wear: [0; game::WEAR_SLOTS],
            rig: None,
            facing: 0.0,
            dead_at: None,
        }
    }

    fn head_height(&self) -> f32 {
        match &self.rig {
            Some(Some(r)) => r.height + 4.0,
            _ => 24.0,
        }
    }
}

struct FloatText {
    pos: Vec3,
    text: String,
    color: Color,
    start: f64,
}

pub struct WorldState {
    clock: Instant,
    handle: u32,
    name: String,
    me: Entity,
    entities: HashMap<u32, Entity>,
    speed: f32,
    cam_yaw: f32,
    cam_pitch: f32,
    cam_dist: f32,
    last_mouse: Vec2,
    press_pos: Option<Vec2>,
    last_move_sent: f64,
    last_dir: Vec2,
    chat_log: Vec<(Color, String)>,
    chat_input: String,
    chat_open: bool,
    props: HashMap<String, i64>,
    terrain: Option<Terrain>,
    models: ModelCache,
    hp: i32,
    max_hp: i32,
    mp: i32,
    max_mp: i32,
    exp: i64,
    jp: i64,
    gold: i64,
    inventory: Vec<game::ItemInfo>,
    show_inventory: bool,
    dialog: Option<game::ScDialog>,
    market: Option<game::ScMarket>,
    target: Option<u32>,
    floats: Vec<FloatText>,
    offline: bool,
    addon_level: Option<i32>,
    addon_money: Option<i64>,
    bag_revision: u64,
    reload_ui: bool,
}

impl WorldState {
    pub fn new(lr: &msg::ScLoginResult) -> Self {
        let chat_log = vec![(YELLOW, "Bienvenue dans RappelzRust. Entrée : chat, .help : commandes, I : inventaire.".to_string())];
        let race = lr.race;
        let sex = lr.sex;
        let mut me = Entity::new(lr.name.clone(), Kind::Player, lr.x, lr.y, 1, Some(RigKey::Player { race, sex }));
        me.hp = lr.hp;
        me.max_hp = lr.max_hp.max(1);
        Self {
            clock: Instant::now(),
            handle: lr.handle,
            name: lr.name.clone(),
            me,
            entities: HashMap::new(),
            speed: 120.0,
            cam_yaw: std::f32::consts::FRAC_PI_2 + 0.4,
            cam_pitch: 0.45,
            cam_dist: 140.0,
            last_mouse: mouse_position().into(),
            press_pos: None,
            last_move_sent: 0.0,
            last_dir: Vec2::ZERO,
            chat_log,
            chat_input: String::new(),
            chat_open: false,
            props: HashMap::new(),
            terrain: None,
            models: ModelCache::default(),
            hp: lr.hp,
            max_hp: lr.max_hp.max(1),
            mp: lr.mp,
            max_mp: lr.max_mp.max(1),
            exp: 0,
            jp: 0,
            gold: 0,
            inventory: Vec::new(),
            show_inventory: false,
            dialog: None,
            market: None,
            target: None,
            floats: Vec::new(),
            offline: false,
            addon_level: None,
            addon_money: None,
            bag_revision: 0,
            reload_ui: false,
        }
    }

    /// Monde hors ligne : le joueur en (x, y) et les PNJ de NpcResource.dbc à proximité.
    pub fn demo(assets: &mut Assets, x: f32, y: f32) -> Self {
        let lr = msg::ScLoginResult { handle: 1, x, y, name: "Démo".into(), race: 4, sex: 1, hp: 100, max_hp: 100, mp: 50, max_mp: 50, ..Default::default() };
        let mut w = Self::new(&lr);
        w.offline = true;
        w.addon_level = Some(w.me.level);
        w.addon_money = Some(w.gold);
        if let Some(npcs) = &assets.npcs {
            for (i, r) in npcs.rows().enumerate() {
                let (nx, ny) = (r.int("x") as f32, r.int("y") as f32);
                if (nx - x).abs() < 1500.0 && (ny - y).abs() < 1500.0 {
                    let name = assets.strings.get(r.int("name_text_id")).to_string();
                    let key = rz_model::entity_cob_name(r.str("model_file")).map(RigKey::Entity);
                    let mut e = Entity::new(name, Kind::Npc, nx, ny, r.int("level"), key);
                    e.facing = (r.int("face") as f32).to_radians();
                    w.entities.insert(1000 + i as u32, e);
                }
            }
        }
        w.log(YELLOW, format!("Mode démo hors ligne : {} PNJ chargés depuis NpcResource.dbc.", w.entities.len()));
        w
    }

    fn now(&self) -> f64 {
        self.clock.elapsed().as_secs_f64()
    }

    pub fn addon_state(&self) -> rz_addon::GameState {
        rz_addon::GameState {
            player: Some(rz_addon::UnitState {
                handle: self.handle, name: self.name.clone(), level: self.addon_level,
                health: Some(self.hp), max_health: Some(self.max_hp), mana: Some(self.mp), max_mana: Some(self.max_mp),
            }),
            target: self.target.and_then(|h| self.unit(h)),
            money: self.addon_money, party_members: None, bag_revision: self.bag_revision,
        }
    }

    pub fn take_reload_ui(&mut self) -> bool { std::mem::take(&mut self.reload_ui) }

    pub fn addon_message(&mut self, line: String) { self.log(YELLOW, line); }

    pub fn unit(&self, handle: u32) -> Option<rz_addon::UnitState> {
        self.entities.get(&handle).map(|entity| entity.unit(handle))
    }

    pub fn visible_player(&self, name: &str) -> Option<rz_addon::UnitState> {
        self.entities.iter().find(|(_, entity)| entity.kind == Kind::Player && entity.name == name)
            .and_then(|(handle, _)| self.unit(*handle))
    }

    pub fn living_monster(&self) -> Option<rz_addon::UnitState> {
        self.entities.iter().filter(|(_, entity)| entity.kind == Kind::Monster && entity.hp > 0 && entity.dead_at.is_none())
            .min_by_key(|(handle, _)| **handle).and_then(|(handle, _)| self.unit(*handle))
    }

    pub fn experience(&self) -> (i64, i64) { (self.exp, self.jp) }

    pub fn attack_target(&self, handle: u32) -> Option<NetCmd> {
        self.entities.get(&handle).filter(|entity| entity.kind == Kind::Monster)?;
        Some(NetCmd::Raw(game::CsAttackRequest { handle: self.handle, target_handle: handle }.to_packet()))
    }

    /// Use the same selection path for native mouse input and the online regression runner.
    pub fn select_target(&mut self, handle: u32) -> Option<NetCmd> {
        self.entities.get(&handle)?;
        self.target = Some(handle);
        Some(NetCmd::Raw(game::Handle::<{ gid::TS_CS_TARGETING }>(handle).to_packet()))
    }

    pub fn nearby_target(&self) -> Option<u32> {
        self.entities.iter().filter(|(_, e)| matches!(e.kind, Kind::Npc | Kind::Monster))
            .min_by_key(|(handle, e)| (e.kind != Kind::Monster, **handle)).map(|(handle, _)| *handle)
    }

    fn log(&mut self, color: Color, line: String) {
        self.chat_log.push((color, line));
        if self.chat_log.len() > 200 {
            self.chat_log.remove(0);
        }
    }

    fn name_of(&self, handle: u32) -> String {
        if handle == self.handle {
            return self.name.clone();
        }
        self.entities.get(&handle).map(|e| e.name.clone()).unwrap_or_else(|| "?".into())
    }

    pub fn on_event(&mut self, ev: NetEvent, assets: &mut Assets) {
        let now = self.now();
        match ev {
            NetEvent::Enter(e) => {
                if e.handle == self.handle {
                    return;
                }
                let entity = match &e.kind {
                    EnterKind::Player(p) => {
                        let mut ent = Entity::new(p.name.clone(), Kind::Player, e.x, e.y, p.creature.level, Some(RigKey::Player { race: p.creature.race as i32, sex: p.sex as i32 }));
                        ent.set_vitals(p.creature.hp, p.creature.max_hp, p.creature.mp, p.creature.max_mp);
                        ent.facing = p.creature.face_direction;
                        ent
                    }
                    EnterKind::Npc { npc_id, creature } => {
                        let key = assets.npc_model(*npc_id).and_then(|m| rz_model::entity_cob_name(&m)).map(RigKey::Entity);
                        let mut ent = Entity::new(assets.npc_name(*npc_id), Kind::Npc, e.x, e.y, creature.level, key);
                        ent.set_vitals(creature.hp, creature.max_hp, creature.mp, creature.max_mp);
                        ent.facing = creature.face_direction;
                        ent
                    }
                    EnterKind::Monster { monster_id, creature, .. } => {
                        let (name, model, level) = assets.monster_by_raw(*monster_id).unwrap_or_else(|| (format!("Monstre {monster_id}"), String::new(), creature.level));
                        let key = rz_model::entity_cob_name(&model).map(RigKey::Entity);
                        let mut ent = Entity::new(name, Kind::Monster, e.x, e.y, level, key);
                        ent.set_vitals(creature.hp, creature.max_hp, creature.mp, creature.max_mp);
                        ent
                    }
                    EnterKind::Other { obj_type, .. } => Entity::new(format!("Objet {obj_type}"), Kind::Other, e.x, e.y, 0, None),
                };
                self.entities.insert(e.handle, entity);
            }
            NetEvent::Leave(h) => {
                self.entities.remove(&h);
                if self.target == Some(h) {
                    self.target = None;
                }
            }
            NetEvent::Move(m) => {
                let speed = m.speed as f32;
                if m.handle == self.handle {
                    self.speed = speed.max(1.0);
                    return;
                }
                if let Some(e) = self.entities.get_mut(&m.handle) {
                    let (x, y) = e.motion.position(now);
                    e.motion = Motion { x, y, start: now, points: m.points, speed };
                }
            }
            NetEvent::Warp { x, y, z } => {
                let _ = z; // la hauteur est recalculée depuis le terrain
                self.me.motion = Motion::at(x, y);
                self.log(YELLOW, format!("Téléporté en {x:.0}, {y:.0}."));
            }
            NetEvent::Chat { sender, chat_type: t, message } => {
                let (color, line) = match t {
                    chat_type::WHISPER => (Color::from_rgba(255, 128, 255, 255), format!("[{sender}] chuchote : {message}")),
                    chat_type::NOTICE => (YELLOW, message),
                    _ => (Color::from_rgba(255, 170, 90, 255), format!("[{sender}] : {message}")),
                };
                self.log(color, line);
            }
            NetEvent::ChatLocal { handle, message } => {
                if handle == self.handle {
                    self.me.say = Some((message.clone(), now));
                } else if let Some(e) = self.entities.get_mut(&handle) {
                    e.say = Some((message.clone(), now));
                }
                let name = self.name_of(handle);
                self.log(WHITE, format!("{name} : {message}"));
            }
            NetEvent::Property { handle, name, value } => {
                if handle == self.handle {
                    if name == "lv" {
                        self.me.level = value as i32;
                        self.addon_level = Some(value as i32);
                    }
                    if name == "gold" {
                        self.gold = value;
                        self.addon_money = Some(value);
                    }
                    self.props.insert(name, value);
                }
            }
            NetEvent::Packet(p) => self.on_packet(p, now),
            _ => {}
        }
    }

    fn on_packet(&mut self, p: RawPacket, now: f64) {
        let previous_inventory = matches!(p.id, gid::TS_SC_INVENTORY | gid::TS_SC_DESTROY_ITEM | gid::TS_SC_ITEM_WEAR_INFO)
            .then(|| self.inventory.clone());
        match p.id {
            gid::TS_SC_INVENTORY => {
                if let Ok(inv) = game::ScInventory::from_packet(&p) {
                    for it in inv.items {
                        match self.inventory.iter_mut().find(|i| i.handle == it.handle) {
                            Some(existing) => *existing = it,
                            None => self.inventory.push(it),
                        }
                    }
                }
            }
            gid::TS_SC_DESTROY_ITEM => {
                if let Ok(h) = game::Handle::<{ gid::TS_SC_DESTROY_ITEM }>::from_packet(&p) {
                    self.inventory.retain(|i| i.handle != h.0);
                }
            }
            gid::TS_SC_ITEM_WEAR_INFO => {
                if let Ok(w) = game::ScItemWearInfo::from_packet(&p) {
                    if let Some(it) = self.inventory.iter_mut().find(|i| i.handle == w.item_handle) {
                        it.wear_position = w.wear_position;
                    }
                }
            }
            gid::TS_SC_WEAR_INFO => {
                if let Ok(w) = game::ScWearInfo::from_packet(&p) {
                    let ent = if w.handle == self.handle { Some(&mut self.me) } else { self.entities.get_mut(&w.handle) };
                    if let Some(e) = ent {
                        if e.wear != w.wear_info {
                            e.wear = w.wear_info;
                            e.rig = None; // réassemblage avec l'équipement
                        }
                    }
                }
            }
            gid::TS_SC_GOLD_UPDATE => {
                if let Ok(g) = game::ScGoldUpdate::from_packet(&p) {
                    self.gold = g.gold;
                    self.addon_money = Some(g.gold);
                }
            }
            gid::TS_SC_EXP_UPDATE => {
                if let Ok(e) = game::ScExpUpdate::from_packet(&p) {
                    if e.handle == self.handle {
                        self.exp = e.exp;
                        self.jp = e.jp;
                    }
                }
            }
            gid::TS_SC_LEVEL_UPDATE => {
                if let Ok(l) = game::ScLevelUpdate::from_packet(&p) {
                    if l.handle == self.handle {
                        self.me.level = l.level;
                        self.addon_level = Some(l.level);
                    } else if let Some(e) = self.entities.get_mut(&l.handle) {
                        e.level = l.level;
                    }
                }
            }
            gid::TS_SC_HPMP => {
                if let Ok(h) = game::ScHpMp::from_packet(&p) {
                    if h.handle == self.handle {
                        (self.hp, self.max_hp, self.mp, self.max_mp) = (h.hp, h.max_hp.max(1), h.mp, h.max_mp.max(1));
                    } else if let Some(e) = self.entities.get_mut(&h.handle) {
                        e.set_vitals(h.hp, h.max_hp, h.mp, h.max_mp);
                        if h.hp <= 0 && e.dead_at.is_none() {
                            e.dead_at = Some(now);
                        }
                    }
                }
            }
            gid::TS_SC_ATTACK_EVENT => {
                if let Ok(a) = game::ScAttackEvent::from_packet(&p) {
                    let target_pos = if a.target == self.handle {
                        let (x, y) = self.me.motion.position(now);
                        Some((x, y, true))
                    } else {
                        self.entities.get(&a.target).map(|e| {
                            let (x, y) = e.motion.position(now);
                            (x, y, false)
                        })
                    };
                    // L'attaquant se tourne vers sa cible.
                    if let Some((tx, ty, _)) = target_pos {
                        let attacker = if a.attacker == self.handle { Some(&mut self.me) } else { self.entities.get_mut(&a.attacker) };
                        if let Some(e) = attacker {
                            let (ax, ay) = e.motion.position(now);
                            e.facing = (tx - ax).atan2(-(ty - ay));
                        }
                    }
                    if let (Some((x, y, me)), Some(hit)) = (target_pos, a.attacks.first()) {
                        let color = if me { Color::from_rgba(255, 80, 80, 255) } else { Color::from_rgba(255, 230, 120, 255) };
                        self.floats.push(FloatText { pos: vec3(x, 0.0, y), text: hit.damage.to_string(), color, start: now });
                    }
                }
            }
            gid::TS_SC_DIALOG => {
                if let Ok(d) = game::ScDialog::from_packet(&p) {
                    self.market = None;
                    self.dialog = Some(d);
                }
            }
            gid::TS_SC_MARKET => {
                if let Ok(m) = game::ScMarket::from_packet(&p) {
                    self.dialog = None;
                    self.market = Some(m);
                }
            }
            _ => {}
        }
        if previous_inventory.is_some_and(|before| before != self.inventory) {
            self.bag_revision = self.bag_revision.wrapping_add(1);
        }
    }

    fn camera(&self, focus: Vec3) -> Camera3D {
        let offset = vec3(
            self.cam_dist * self.cam_pitch.cos() * self.cam_yaw.cos(),
            self.cam_dist * self.cam_pitch.sin(),
            self.cam_dist * self.cam_pitch.cos() * self.cam_yaw.sin(),
        );
        Camera3D { position: focus + offset, target: focus + vec3(0.0, 12.0, 0.0), up: Vec3::Y, fovy: 0.9, z_near: 2.0, z_far: 9000.0, ..Default::default() }
    }

    fn project(cam: &Camera3D, p: Vec3) -> Option<Vec2> {
        let clip = cam.matrix() * p.extend(1.0);
        if clip.w <= 0.0 {
            return None;
        }
        let ndc = clip.truncate() / clip.w;
        Some(vec2((ndc.x + 1.0) * 0.5 * screen_width(), (1.0 - ndc.y) * 0.5 * screen_height()))
    }

    /// Point du sol sous la souris (intersection avec le plan horizontal à la hauteur `h`).
    fn ground_under_mouse(cam: &Camera3D, h: f32) -> Option<Vec2> {
        let (mx, my) = mouse_position();
        let ndc = vec2(mx / screen_width() * 2.0 - 1.0, 1.0 - my / screen_height() * 2.0);
        let inv = cam.matrix().inverse();
        let near = inv * vec4(ndc.x, ndc.y, -1.0, 1.0);
        let far = inv * vec4(ndc.x, ndc.y, 1.0, 1.0);
        let (near, far) = (near.truncate() / near.w, far.truncate() / far.w);
        let dir = far - near;
        if dir.y.abs() < 1e-6 {
            return None;
        }
        let t = (h - near.y) / dir.y;
        (t > 0.0).then(|| {
            let p = near + dir * t;
            vec2(p.x, p.z)
        })
    }

    fn ensure_rig(&mut self, assets: &mut Assets, handle: Option<u32>, budget: &mut usize) {
        let e = match handle {
            None => &mut self.me,
            Some(h) => match self.entities.get_mut(&h) {
                Some(e) => e,
                None => return,
            },
        };
        if e.rig.is_some() || *budget == 0 {
            return;
        }
        *budget -= 1;
        let Some(key) = e.rig_key.clone() else {
            e.rig = Some(None);
            return;
        };
        let mut extra = Vec::new();
        if let RigKey::Player { race, sex } = key {
            for code in e.wear.iter().filter(|c| **c > 0) {
                if let Some(f) = rz_model::item_model_file(&assets.item_models(*code as u32), race, sex, 0) {
                    if f.starts_with(rz_model::race_prefix(race, sex).unwrap_or("?")) {
                        extra.push(f);
                    }
                }
            }
        }
        e.rig = Some(self.models.rig(assets, &key, &extra));
    }

    fn draw_creature(e: &mut Entity, pos: Vec3, color: Color, t: f32, moving: bool) {
        match &mut e.rig {
            Some(Some(rig)) => {
                let m = models::placement([pos.x, pos.z, pos.y], [0.0, 0.0, e.facing], [1.0, 1.0, 1.0]);
                rig.draw(m, t, moving);
            }
            _ => {
                draw_cylinder(pos, 4.0, 3.5, 18.0, None, color);
                draw_sphere(pos + vec3(0.0, 21.0, 0.0), 3.5, None, Color::from_rgba(235, 205, 170, 255));
            }
        }
    }

    /// Met à jour la simulation, dessine la scène et l'interface, renvoie les commandes réseau.
    pub fn update_and_draw(&mut self, assets: &mut Assets, ui: &mut RzUi, addon_mouse: bool, modal: bool) -> Vec<NetCmd> {
        let mut out = Vec::new();
        let now = self.now();
        if self.terrain.is_none() {
            self.terrain = Some(Terrain::new(assets));
        }
        let (mx, my) = self.me.motion.position(now);
        if let Some(h) = self.me.motion.heading(now) {
            self.me.facing = h;
        }
        let mut terrain = self.terrain.take().unwrap();
        let ground = |terrain: &mut Terrain, assets: &mut Assets, x: f32, y: f32| terrain.height_at(assets, x, y);
        let my_h = ground(&mut terrain, assets, mx, my);

        // Caméra : glisser avec le bouton gauche, molette pour le zoom.
        let mouse: Vec2 = mouse_position().into();
        if is_mouse_button_pressed(MouseButton::Left) && !addon_mouse {
            self.press_pos = Some(mouse);
        }
        let dragging = self.press_pos.is_some_and(|p| (p - mouse).length() > 4.0);
        if is_mouse_button_down(MouseButton::Left) && dragging && !self.chat_open && !addon_mouse {
            let d = mouse - self.last_mouse;
            self.cam_yaw += d.x * 0.008;
            self.cam_pitch = (self.cam_pitch + d.y * 0.006).clamp(0.05, 1.45);
        }
        self.last_mouse = mouse;
        let (_, wheel) = mouse_wheel();
        if wheel != 0.0 && !addon_mouse {
            self.cam_dist = (self.cam_dist * if wheel > 0.0 { 0.9 } else { 1.1 }).clamp(30.0, 900.0);
        }

        let focus = vec3(mx, my_h, my);
        let cam = self.camera(focus);
        clear_background(Color::from_rgba(150, 185, 220, 255));
        set_camera(&cam);
        terrain.draw(assets, mx, my);
        let mut prop_budget = 3;
        terrain.draw_props(assets, &mut self.models, mx, my, &mut prop_budget);

        // Créatures (les plus proches sont assemblées en premier).
        let mut rig_budget = RIGS_PER_FRAME;
        self.ensure_rig(assets, None, &mut rig_budget);
        let mut handles: Vec<(u32, f32)> = self
            .entities
            .iter()
            .map(|(&h, e)| {
                let (x, y) = e.motion.position(now);
                (h, (x - mx).powi(2) + (y - my).powi(2))
            })
            .collect();
        handles.sort_by(|a, b| a.1.total_cmp(&b.1));
        for &(h, _) in &handles {
            self.ensure_rig(assets, Some(h), &mut rig_budget);
        }
        let t = now as f32;
        let mut screen_labels: Vec<(u32, Vec3)> = Vec::new();
        for &(h, _) in &handles {
            let e = self.entities.get_mut(&h).unwrap();
            if e.dead_at.is_some_and(|d| now - d > 2.0) {
                continue;
            }
            let (x, y) = e.motion.position(now);
            if let Some(hd) = e.motion.heading(now) {
                e.facing = hd;
            }
            let z = terrain.height_at(assets, x, y);
            let pos = vec3(x, z, y);
            let color = match e.kind {
                Kind::Player => Color::from_rgba(90, 140, 230, 255),
                Kind::Npc => Color::from_rgba(230, 200, 80, 255),
                Kind::Monster => Color::from_rgba(210, 70, 60, 255),
                Kind::Other => GRAY,
            };
            let moving = e.motion.is_moving(now);
            Self::draw_creature(e, pos, color, t, moving);
            screen_labels.push((h, pos + vec3(0.0, e.head_height(), 0.0)));
        }
        let me_moving = self.me.motion.is_moving(now);
        Self::draw_creature(&mut self.me, focus, Color::from_rgba(80, 200, 120, 255), t, me_moving);
        if let Some(target) = self.target.and_then(|h| self.entities.get(&h)) {
            let (x, y) = target.motion.position(now);
            let z = terrain.height_at(assets, x, y);
            draw_cylinder_wires(vec3(x, z + 0.5, y), 9.0, 9.0, 0.5, None, RED);
        }
        for f in &mut self.floats {
            f.pos.y = terrain.height_at(assets, f.pos.x, f.pos.z) + 26.0;
        }
        self.terrain = Some(terrain);

        // Interface : la souris est-elle sur une fenêtre ?
        set_default_camera();
        let over_ui = if modal { true } else { self.draw_hud(assets, ui, &mut out, now) || addon_mouse };

        // Étiquettes (noms, barres de vie, bulles) et nombres de dégâts.
        for (h, head) in &screen_labels {
            let e = &self.entities[h];
            if let Some(s) = Self::project(&cam, *head) {
                let color = match e.kind {
                    Kind::Npc => Color::from_rgba(255, 220, 90, 255),
                    Kind::Monster => Color::from_rgba(255, 110, 100, 255),
                    _ => WHITE,
                };
                let label = if e.kind == Kind::Monster { format!("{} (niv. {})", e.name, e.level) } else { e.name.clone() };
                ui.label_in(&label, Rect::new(s.x - 100.0, s.y - 16.0, 200.0, 14.0), color);
                if e.kind == Kind::Monster || self.target == Some(*h) {
                    let frac = (e.hp as f32 / e.max_hp as f32).clamp(0.0, 1.0);
                    draw_rectangle(s.x - 25.0, s.y, 50.0, 4.0, Color::from_rgba(0, 0, 0, 180));
                    draw_rectangle(s.x - 25.0, s.y, 50.0 * frac, 4.0, Color::from_rgba(220, 40, 40, 255));
                }
                if let Some((say, st)) = &e.say {
                    if now - st < 6.0 {
                        ui.label_in(say, Rect::new(s.x - 150.0, s.y - 34.0, 300.0, 14.0), WHITE);
                    }
                }
            }
        }
        if let Some(s) = Self::project(&cam, focus + vec3(0.0, self.me.head_height(), 0.0)) {
            ui.label_in(&self.name, Rect::new(s.x - 100.0, s.y - 16.0, 200.0, 14.0), Color::from_rgba(120, 255, 150, 255));
            if let Some((say, st)) = &self.me.say {
                if now - st < 6.0 {
                    ui.label_in(say, Rect::new(s.x - 150.0, s.y - 34.0, 300.0, 14.0), WHITE);
                }
            }
        }
        self.floats.retain(|f| now - f.start < 1.2);
        for f in &self.floats {
            let k = (now - f.start) as f32;
            if let Some(s) = Self::project(&cam, f.pos + vec3(0.0, k * 18.0, 0.0)) {
                ui.label_in(&f.text, Rect::new(s.x - 40.0, s.y, 80.0, 14.0), f.color);
            }
        }

        // Déplacement et interaction.
        if !self.chat_open && !modal {
            self.handle_movement(&cam, mx, my, my_h, now, over_ui, &screen_labels, &mut out);
        }
        if is_mouse_button_released(MouseButton::Left) {
            self.press_pos = None;
        }
        out
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_movement(&mut self, cam: &Camera3D, mx: f32, my: f32, my_h: f32, now: f64, over_ui: bool, labels: &[(u32, Vec3)], out: &mut Vec<NetCmd>) {
        let forward = vec2(-self.cam_yaw.cos(), -self.cam_yaw.sin());
        let right = vec2(-forward.y, forward.x);
        let mut dir = Vec2::ZERO;
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Z) || is_key_down(KeyCode::Up) {
            dir += forward;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            dir -= forward;
        }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Q) || is_key_down(KeyCode::Left) {
            dir -= right;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            dir += right;
        }
        let dir = dir.normalize_or_zero();
        let changed = (dir - self.last_dir).length() > 0.01;
        if dir != Vec2::ZERO && (changed || now - self.last_move_sent > 0.8) {
            let target = vec2(mx, my) + dir * self.speed * 1.5;
            out.push(self.start_move(mx, my, target, now));
        } else if dir == Vec2::ZERO && changed && self.me.motion.is_moving(now) {
            out.push(self.start_move(mx, my, vec2(mx, my), now));
        }
        self.last_dir = dir;
        if over_ui {
            return;
        }
        // Clic gauche (sans glisser) : sélection d'une créature ; clic droit : interaction/déplacement.
        let clicked_left = is_mouse_button_released(MouseButton::Left) && self.press_pos.is_some_and(|p| (p - Vec2::from(mouse_position())).length() <= 4.0);
        let clicked_right = is_mouse_button_pressed(MouseButton::Right);
        if !clicked_left && !clicked_right {
            return;
        }
        let mouse: Vec2 = mouse_position().into();
        let picked = labels
            .iter()
            .filter_map(|(h, head)| {
                let s = Self::project(cam, *head - vec3(0.0, 10.0, 0.0))?;
                let d = (s - mouse).length();
                (d < 40.0).then_some((*h, d))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(h, _)| h);
        match picked {
            Some(h) => {
                if let Some(command) = self.select_target(h) { out.push(command); }
                if clicked_right {
                    let e = &self.entities[&h];
                    let (ex, ey) = e.motion.position(now);
                    let d = ((ex - mx).powi(2) + (ey - my).powi(2)).sqrt();
                    let stop = if e.kind == Kind::Npc { 35.0 } else { 25.0 };
                    if d > stop {
                        let k = (d - stop) / d;
                        out.push(self.start_move(mx, my, vec2(mx + (ex - mx) * k, my + (ey - my) * k), now));
                    }
                    match self.entities[&h].kind {
                        Kind::Monster => if let Some(command) = self.attack_target(h) { out.push(command); },
                        Kind::Npc => out.push(NetCmd::Raw(game::Handle::<{ gid::TS_CS_CONTACT }>(h).to_packet())),
                        _ => {}
                    }
                }
            }
            None if clicked_right => {
                if let Some(target) = Self::ground_under_mouse(cam, my_h) {
                    out.push(self.start_move(mx, my, target, now));
                    out.push(NetCmd::Raw(game::Handle::<{ gid::TS_CS_CANCEL_ACTION }>(self.handle).to_packet()));
                }
            }
            None => self.target = None,
        }
    }

    fn start_move(&mut self, x: f32, y: f32, target: Vec2, now: f64) -> NetCmd {
        let points = vec![MovePoint { x: target.x, y: target.y }];
        self.me.motion = Motion { x, y, start: now, points: points.clone(), speed: self.speed };
        self.last_move_sent = now;
        NetCmd::Move { x, y, points }
    }

    /// HUD et fenêtres d'origine ; renvoie vrai si la souris est sur l'interface.
    fn draw_hud(&mut self, assets: &mut Assets, ui: &mut RzUi, out: &mut Vec<NetCmd>, now: f64) -> bool {
        let (sw, sh) = (screen_width(), screen_height());
        let mut over = false;

        // Fenêtre du personnage (PV, PM, EXP).
        let mut st = UiState::default();
        let job = self.props.get("job").copied().unwrap_or(0) as i32;
        st.set_caption("static_name", &self.name)
            .set_caption("mainframe_level", &self.me.level.to_string())
            .set_caption("player_text_job_01", &assets.job_name(job))
            .set_caption("static_hp_value", &format!("{}/{}", self.hp, self.max_hp))
            .set_caption("static_mp_value", &format!("{}/{}", self.mp, self.max_mp))
            .set_gauge("mainframe_hpbar", self.hp as f32 / self.max_hp as f32)
            .set_gauge("mainframe_mpbar", self.mp as f32 / self.max_mp as f32)
            .set_gauge("mainframe_staminabar", 1.0);
        let lvl = self.me.level.max(1) as i64;
        let need = 100 * lvl * lvl + 200 * lvl;
        let exp_frac = (self.exp as f32 / need as f32).clamp(0.0, 1.0);
        st.set_gauge("mainframe_expbar", exp_frac).set_caption("static_exp_value", &format!("{:.2}%", exp_frac * 100.0));
        over |= ui.draw(assets, "window_main_player.nui", vec2(4.0, 4.0), &st).contains_mouse();

        // Barre des tâches, raccourcis, menu.
        let task = ui.size(assets, "window_main_taskbar.nui");
        let mut tst = UiState::default();
        tst.set_caption("static_money", &format!("{} R", self.gold));
        over |= ui.draw(assets, "window_main_taskbar.nui", vec2(0.0, sh - task.y), &tst).contains_mouse();
        // Barre de raccourcis : seule la rangée du bas (y ≥ 127) est affichée, comme par défaut.
        let mut qst = UiState::default();
        ui.hide_where(assets, "window_main_quickslot.nui", &mut qst, |r| r.y1 < 127);
        let qs_full = ui.size(assets, "window_main_quickslot.nui");
        let qs = vec2(qs_full.x, (qs_full.y - 127.0).max(40.0));
        over |= ui.draw(assets, "window_main_quickslot.nui", vec2((sw - qs.x) / 2.0, sh - task.y - qs.y - 127.0), &qst).contains_mouse();
        let menu = ui.size(assets, "window_main_menu.nui");
        let menu_frame = ui.draw(assets, "window_main_menu.nui", vec2(sw - menu.x, sh - task.y - menu.y), &UiState::default());
        over |= menu_frame.contains_mouse();
        ui.label(&format!("{} rupees", self.gold), 12.0, sh - task.y - 6.0, Color::from_rgba(255, 220, 120, 255));

        // Chat.
        let chat = ui.size(assets, "window_main_chat.nui");
        let chat_origin = vec2(4.0, sh - task.y - qs.y - chat.y - 4.0);
        let chat_frame = ui.draw(assets, "window_main_chat.nui", chat_origin, &UiState::default());
        over |= chat_frame.contains_mouse();
        let area = chat_frame.rect("background").unwrap_or(Rect::new(chat_origin.x, chat_origin.y, chat.x, chat.y - 30.0));
        let lines: Vec<_> = self.chat_log.iter().rev().take(MAX_CHAT_LINES).collect();
        for (i, (color, line)) in lines.iter().enumerate() {
            ui.label(line, area.x + 16.0, area.y + area.h - 6.0 - i as f32 * 13.5, *color);
        }
        let edit = chat_frame.rect("chat_edit").unwrap_or(Rect::new(chat_origin.x + 26.0, chat_origin.y + chat.y - 22.0, 360.0, 18.0));
        self.update_chat(ui, edit, out, now);

        // Minicarte.
        self.draw_minimap(assets, now);

        // Fenêtres ouvertes.
        if is_key_pressed(KeyCode::I) && !self.chat_open {
            self.show_inventory = !self.show_inventory;
        }
        if self.show_inventory {
            over |= self.draw_inventory(assets, ui, out);
        }
        if self.dialog.is_some() {
            over |= self.draw_dialog(assets, ui, out);
        }
        if self.market.is_some() {
            over |= self.draw_market(assets, ui, out);
        }
        over
    }

    fn update_chat(&mut self, ui: &RzUi, edit: Rect, out: &mut Vec<NetCmd>, now: f64) {
        if self.chat_open {
            while let Some(c) = get_char_pressed() {
                if !c.is_control() && self.chat_input.chars().count() < 160 {
                    self.chat_input.push(c);
                }
            }
            if is_key_pressed(KeyCode::Backspace) {
                self.chat_input.pop();
            }
            let caret = if (now * 2.0) as i64 % 2 == 0 { "|" } else { "" };
            ui.label(&format!("{}{caret}", self.chat_input), edit.x + 4.0, edit.y + edit.h - 5.0, WHITE);
            if is_key_pressed(KeyCode::Escape) {
                self.chat_open = false;
                self.chat_input.clear();
            }
            if is_key_pressed(KeyCode::Enter) {
                let text = std::mem::take(&mut self.chat_input);
                self.chat_open = false;
                if text.trim().eq_ignore_ascii_case("/reload") {
                    self.reload_ui = true;
                } else if let Some(cmd) = Self::parse_chat(&text) {
                    out.push(cmd);
                }
            }
        } else if is_key_pressed(KeyCode::Enter) {
            self.chat_open = true;
            while get_char_pressed().is_some() {}
        } else if is_key_pressed(KeyCode::Escape) {
            if self.dialog.is_some() || self.market.is_some() || self.show_inventory {
                self.dialog = None;
                self.market = None;
                self.show_inventory = false;
            } else if !self.offline {
                out.push(NetCmd::ReturnLobby);
            }
        }
    }

    /// Inventaire (`window_main_inventory_bag.nui`) : icônes d'objets ; clic droit = équiper.
    fn draw_inventory(&mut self, assets: &mut Assets, ui: &mut RzUi, out: &mut Vec<NetCmd>) -> bool {
        let (sw, sh) = (screen_width(), screen_height());
        let size = ui.size(assets, "window_main_inventory_bag.nui");
        let origin = vec2(sw - size.x.min(450.0) - 10.0, (sh - size.y) / 2.0 - 40.0);
        let mut st = UiState::default();
        st.set_caption("text_01", "Inventaire");
        let frame = ui.draw(assets, "window_main_inventory_bag.nui", origin, &st);
        if frame.clicked("button_close") {
            self.show_inventory = false;
        }
        let list = frame.rect("listcontrol01").unwrap_or(Rect::new(origin.x + 4.0, origin.y + 45.0, 423.0, 366.0));
        let cell = 34.0;
        let cols = ((list.w - 6.0) / cell).floor().max(1.0) as usize;
        let mouse: Vec2 = mouse_position().into();
        let mut tooltip = None;
        let mut items: Vec<game::ItemInfo> = self.inventory.clone();
        items.sort_by_key(|i| (i.wear_position < 0, i.index, i.handle));
        for (n, it) in items.iter().enumerate() {
            let (cx, cy) = (list.x + 3.0 + (n % cols) as f32 * cell, list.y + 3.0 + (n / cols) as f32 * cell);
            if cy + cell > list.y + list.h {
                break;
            }
            let r = Rect::new(cx, cy, cell - 2.0, cell - 2.0);
            draw_rectangle(r.x, r.y, r.w, r.h, Color::from_rgba(0, 0, 0, 120));
            if let Some(icon) = assets.item_icon(it.code) {
                draw_texture_ex(&icon, r.x, r.y, WHITE, DrawTextureParams { dest_size: Some(vec2(r.w, r.h)), ..Default::default() });
            }
            if it.wear_position >= 0 {
                draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, Color::from_rgba(255, 210, 90, 255));
                ui.label("E", r.x + 2.0, r.y + 11.0, Color::from_rgba(255, 210, 90, 255));
            }
            if it.count > 1 {
                ui.label(&it.count.to_string(), r.x + r.w - 14.0, r.y + r.h - 2.0, WHITE);
            }
            if r.contains(mouse) {
                tooltip = Some((it.code, mouse));
                if is_mouse_button_pressed(MouseButton::Right) {
                    if it.wear_position >= 0 {
                        out.push(NetCmd::Raw(game::CsPutOffItem { position: it.wear_position as u8, target_handle: self.handle }.to_packet()));
                    } else if let Some(wear) = assets.items.as_ref().and_then(|d| d.by_id(it.code as i32)).map(|r| r.int("wear_type")).filter(|w| (0..game::WEAR_SLOTS as i32).contains(w)) {
                        out.push(NetCmd::Raw(game::CsPutOnItem { position: wear as u8, item_handle: it.handle, target_handle: self.handle }.to_packet()));
                    }
                }
                if self.market.is_some() && is_mouse_button_pressed(MouseButton::Left) && is_key_down(KeyCode::LeftShift) {
                    out.push(NetCmd::Raw(game::CsSellItem { item_handle: it.handle, count: it.count }.to_packet()));
                }
            }
        }
        ui.label(&format!("{} rupees — clic droit : équiper / retirer", self.gold), list.x + 4.0, list.y + list.h + 20.0, Color::from_rgba(255, 220, 120, 255));
        if let Some((code, m)) = tooltip {
            let name = assets.item_name(code);
            let w = ui.measure(&name, 13) + 12.0;
            draw_rectangle(m.x + 14.0, m.y + 4.0, w, 20.0, Color::from_rgba(0, 0, 0, 220));
            ui.label(&name, m.x + 20.0, m.y + 18.0, Color::from_rgba(255, 230, 160, 255));
        }
        frame.contains_mouse()
    }

    /// Dialogue PNJ (`window_npc_talk.nui`) : nom, texte et choix.
    fn draw_dialog(&mut self, assets: &mut Assets, ui: &mut RzUi, out: &mut Vec<NetCmd>) -> bool {
        let Some(d) = self.dialog.clone() else { return false };
        let (sw, sh) = (screen_width(), screen_height());
        let origin = vec2((sw - 822.0).max(0.0) / 2.0, sh - 330.0);
        let mut st = UiState::default();
        st.set_caption("npc_name", &d.title).set_caption("npc_story", &d.text).set_caption("npc_job_name", "");
        st.hide("button_prev").hide("npc_select");
        let frame = ui.draw(assets, "window_npc_talk.nui", origin, &st);
        if frame.bounds.w <= 0.0 {
            draw_rectangle(origin.x, origin.y, 822.0, 220.0, Color::from_rgba(10, 10, 20, 220));
            ui.label(&d.title, origin.x + 20.0, origin.y + 30.0, Color::from_rgba(255, 220, 90, 255));
            ui.label(&d.text, origin.x + 20.0, origin.y + 60.0, WHITE);
        }
        let choice = frame.rect("npc_story").unwrap_or(Rect::new(origin.x + 245.0, origin.y + 62.0, 490.0, 120.0));
        let mouse: Vec2 = mouse_position().into();
        for (i, (label, trigger)) in d.menu.iter().enumerate() {
            let r = Rect::new(choice.x, choice.y + choice.h - 18.0 * (d.menu.len() - i) as f32, choice.w, 17.0);
            let hover = r.contains(mouse);
            ui.label(&format!("› {label}"), r.x + 6.0, r.y + 13.0, if hover { Color::from_rgba(255, 220, 90, 255) } else { Color::from_rgba(180, 220, 255, 255) });
            if hover && is_mouse_button_pressed(MouseButton::Left) {
                out.push(NetCmd::Raw(game::CsDialog { trigger: trigger.clone() }.to_packet()));
                if trigger != "market" {
                    self.dialog = None;
                }
            }
        }
        if frame.clicked("npc_button_next") || frame.clicked("button_hide") {
            self.dialog = None;
        }
        frame.contains_mouse() || frame.bounds.w <= 0.0
    }

    /// Boutique : liste des articles du PNJ (`TS_SC_MARKET`) ; clic = acheter 1.
    fn draw_market(&mut self, assets: &mut Assets, ui: &mut RzUi, out: &mut Vec<NetCmd>) -> bool {
        let Some(m) = self.market.clone() else { return false };
        let r = Rect::new(40.0, 120.0, 380.0, 40.0 + 38.0 * m.items.len().min(10) as f32);
        draw_rectangle(r.x, r.y, r.w, r.h, Color::from_rgba(12, 10, 18, 230));
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, Color::from_rgba(150, 120, 60, 255));
        ui.label("Boutique — clic : acheter, Maj+clic dans le sac : vendre", r.x + 10.0, r.y + 18.0, Color::from_rgba(255, 220, 90, 255));
        let mouse: Vec2 = mouse_position().into();
        for (i, it) in m.items.iter().take(10).enumerate() {
            let row = Rect::new(r.x + 8.0, r.y + 30.0 + i as f32 * 38.0, r.w - 16.0, 36.0);
            let hover = row.contains(mouse);
            if hover {
                draw_rectangle(row.x, row.y, row.w, row.h, Color::from_rgba(120, 90, 30, 90));
            }
            if let Some(icon) = assets.item_icon(it.code as u32) {
                draw_texture_ex(&icon, row.x, row.y + 1.0, WHITE, DrawTextureParams { dest_size: Some(vec2(34.0, 34.0)), ..Default::default() });
            }
            ui.label(&assets.item_name(it.code as u32), row.x + 42.0, row.y + 15.0, WHITE);
            ui.label(&format!("{} rupees", it.price), row.x + 42.0, row.y + 31.0, Color::from_rgba(255, 220, 120, 255));
            if hover && is_mouse_button_pressed(MouseButton::Left) {
                out.push(NetCmd::Raw(game::CsBuyItem { item_code: it.code, count: 1 }.to_packet()));
            }
        }
        self.show_inventory = true;
        r.contains(mouse)
    }

    /// Minicarte (coin supérieur droit) : morceaux `mXXX_YYY_L_C.jpg` autour du joueur.
    fn draw_minimap(&self, assets: &mut Assets, now: f64) {
        const SIZE: f32 = 180.0;
        const RANGE: f32 = 1400.0;
        let (mx, my) = self.me.motion.position(now);
        let x0 = screen_width() - SIZE - 10.0;
        let y0 = 10.0;
        let scale = SIZE / RANGE;
        draw_rectangle(x0 - 3.0, y0 - 3.0, SIZE + 6.0, SIZE + 6.0, Color::from_rgba(110, 85, 40, 255));
        draw_rectangle(x0, y0, SIZE, SIZE, Color::from_rgba(20, 30, 25, 255));
        let (left, top) = (mx - RANGE / 2.0, my - RANGE / 2.0);
        let (gx0, gy0) = ((left / PIECE).floor() as i32, (top / PIECE).floor() as i32);
        let (gx1, gy1) = (((left + RANGE) / PIECE).floor() as i32, ((top + RANGE) / PIECE).floor() as i32);
        for gy in gy0.max(0)..=gy1 {
            for gx in gx0.max(0)..=gx1 {
                let (tx, ty, px, py) = (gx / MINIMAP_PIECES, gy / MINIMAP_PIECES, gx % MINIMAP_PIECES, gy % MINIMAP_PIECES);
                let Some(tex) = assets.texture(&format!("m{tx:03}_{ty:03}_{py}_{px}.jpg")) else { continue };
                let (wx0, wy0) = (gx as f32 * PIECE, gy as f32 * PIECE);
                let (cx0, cy0) = (wx0.max(left), wy0.max(top));
                let (cx1, cy1) = ((wx0 + PIECE).min(left + RANGE), (wy0 + PIECE).min(top + RANGE));
                if cx1 <= cx0 || cy1 <= cy0 {
                    continue;
                }
                let (tw, th) = (tex.width(), tex.height());
                let src = Rect::new((cx0 - wx0) / PIECE * tw, (cy0 - wy0) / PIECE * th, (cx1 - cx0) / PIECE * tw, (cy1 - cy0) / PIECE * th);
                draw_texture_ex(&tex, x0 + (cx0 - left) * scale, y0 + (cy0 - top) * scale, WHITE, DrawTextureParams { dest_size: Some(vec2((cx1 - cx0) * scale, (cy1 - cy0) * scale)), source: Some(src), ..Default::default() });
            }
        }
        let npc_icon = assets.texture("common_mark_minimap_npc.jpg");
        for e in self.entities.values() {
            let (ex, ey) = e.motion.position(now);
            let (sx, sy) = (x0 + (ex - left) * scale, y0 + (ey - top) * scale);
            if sx < x0 || sy < y0 || sx > x0 + SIZE || sy > y0 + SIZE {
                continue;
            }
            match (e.kind, &npc_icon) {
                (Kind::Npc, Some(icon)) => draw_texture_ex(icon, sx - 5.0, sy - 5.0, WHITE, DrawTextureParams { dest_size: Some(vec2(10.0, 10.0)), ..Default::default() }),
                (Kind::Player, _) => draw_circle(sx, sy, 3.0, Color::from_rgba(90, 160, 255, 255)),
                (Kind::Monster, _) => draw_circle(sx, sy, 3.0, RED),
                _ => draw_circle(sx, sy, 3.0, YELLOW),
            }
        }
        let (cx, cy) = (x0 + SIZE / 2.0, y0 + SIZE / 2.0);
        let dir = vec2(-self.cam_yaw.cos(), -self.cam_yaw.sin());
        let side = vec2(-dir.y, dir.x);
        draw_triangle(vec2(cx, cy) + dir * 8.0, vec2(cx, cy) - dir * 5.0 + side * 5.0, vec2(cx, cy) - dir * 5.0 - side * 5.0, Color::from_rgba(120, 255, 150, 255));
    }

    /// `/w nom message` chuchoter, `/y message` crier, `.commande` commande serveur.
    fn parse_chat(text: &str) -> Option<NetCmd> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }
        let chat = |t, target: &str, m: &str| NetCmd::Chat { chat_type: t, target: target.to_string(), message: m.to_string() };
        if let Some(rest) = text.strip_prefix("/w ").or_else(|| text.strip_prefix("/whisper ")) {
            let (target, m) = rest.split_once(' ')?;
            return Some(chat(chat_type::WHISPER, target, m));
        }
        if let Some(m) = text.strip_prefix("/y ") {
            return Some(chat(chat_type::YELL, "", m));
        }
        Some(chat(chat_type::NORMAL, "", text))
    }
}
