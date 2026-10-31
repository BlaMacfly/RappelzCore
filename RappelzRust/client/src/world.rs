//! Monde en jeu : terrain Rappelz (relief, textures de sol, décors), créatures 3D animées,
//! HUD et fenêtres d'origine (`.nui`), inventaire, dialogues PNJ, boutique, combat et chat.

use std::collections::{HashMap,BTreeMap};
use std::time::Instant;

use macroquad::prelude::*;
use macroquad::camera::Camera;
use rz_proto::msg::{self, chat_type, EnterKind, MovePoint};
use rz_proto::msg_game::{self as game, id as gid};
use rz_proto::{Message, RawPacket};
use rz_ui::UiState;

use crate::assets::Assets;
use crate::models::{self, ModelCache, Rig, RigKey};
use crate::net::{NetCmd, NetEvent};
use crate::rzui::RzUi;
use crate::terrain::Terrain;

mod party_ui;
mod skills_ui;
mod casting;

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

    #[test]
    fn revived_entity_remains_renderable_after_the_corpse_timeout() {
        let mut entity = Entity::new("Peer".into(), Kind::Player, 0.0, 0.0, 1, None);
        entity.apply_hpmp(0, 100, 1, 50, 1.0);
        assert!(entity.renderable(2.0));
        assert!(!entity.renderable(3.1));
        entity.apply_hpmp(100, 100, 50, 50, 4.0);
        assert!(entity.renderable(10.0), "revived player disappeared after corpse timeout");
        assert_eq!(entity.dead_at, None);
        entity.apply_hpmp(0, 100, 0, 50, 12.0);
        assert_eq!(entity.dead_at, Some(12.0));
    }

    #[test]
    fn layer_warp_updates_real_adapter_and_clears_previous_layer_ui_without_graphics() {
        let mut world=WorldState::from_login(&msg::ScLoginResult {
            handle:7,name:"LuaLayer".into(),layer:0,hp:100,max_hp:100,mp:50,max_mp:50,..Default::default()
        },Vec2::ZERO);
        let mut peer=Entity::new("Peer".into(),Kind::Player,10.0,0.0,1,None);
        peer.layer=Some(0);world.entities.insert(8,peer);world.target=Some(8);
        world.dialog=Some(game::ScDialog { dialog_type:0,npc_handle:8,title:String::new(),text:String::new(),menu:Vec::new() });
        world.market=Some(game::ScMarket { npc_handle:8,items:Vec::new() });
        world.apply_warp(1.0,2.0,0.0,0);
        assert_eq!(world.addon_state().target.unwrap().layer,Some(0));
        world.apply_warp(5.0,6.0,0.0,1);
        assert_eq!(world.addon_state().player.unwrap().layer,Some(1));
        assert!(world.addon_state().target.is_none());
        assert_eq!(world.entity_count(),0);assert!(!world.dialog_open());
        assert_eq!(world.unit_position(7),Some((5.0,6.0)));
        world.apply_warp(5.0,6.0,0.0,0);
        assert_eq!(world.addon_state().player.unwrap().layer,Some(0));
        assert!(world.entities.is_empty(),"return must wait for fresh ENTER, not revive cached objects");
        world.apply_server_target(8);assert!(world.target.is_none());
        world.entities.insert(8,Entity::new("Peer".into(),Kind::Player,0.0,0.0,1,None));
        world.apply_server_target(8);assert_eq!(world.target,Some(8));
        world.apply_server_target(0);assert!(world.target.is_none());
    }

    #[test]
    fn party_adapter_uses_authoritative_roster_without_disclosing_hidden_world_entities() {
        use rz_proto::party::{PartyMember,ScPartyState};
        let mut world=WorldState::from_login(&msg::ScLoginResult {handle:7,hp:100,max_hp:100,mp:50,max_mp:50,..Default::default()},Vec2::ZERO);
        assert_eq!(world.addon_state().party_members,None);
        world.on_packet(ScPartyState::default().to_packet(),0.0);
        assert_eq!(world.addon_state().party_members,Some(0));
        let member=|handle|PartyMember {handle,name:format!("Player{handle}"),level:1,hp:100,max_hp:100,mp:50,max_mp:50,..Default::default()};
        let mut state=ScPartyState {party_id:1,leader:7,members:vec![member(7),member(8)],invite:None};
        state.members[1].layer=1;state.members[1].hp=25;state.members[1].mp=4;
        world.on_packet(state.to_packet(),0.0);
        let addon=world.addon_state();assert_eq!(addon.party_members,Some(1));
        let peer=&addon.party.unwrap()[0];assert_eq!((peer.handle,peer.health,peer.mana,peer.layer),(8,Some(25),Some(4),Some(1)));
        assert!(world.entities.is_empty());assert!(world.select_target(8).is_none());
        world.apply_warp(10.0,20.0,0.0,1);assert_eq!(world.party_state(),Some(&state));
        let mut wrong=state.clone();wrong.members[0].handle=9;wrong.leader=9;
        world.on_packet(wrong.to_packet(),0.0);assert_eq!(world.party_state(),Some(&state));
        let mut truncated=state.to_packet();truncated.body.pop();world.on_packet(truncated,0.0);
        assert_eq!(world.party_state(),Some(&state));
        world.on_packet(ScPartyState::default().to_packet(),0.0);assert_eq!(world.addon_state().party_members,Some(0));
        assert!(matches!(WorldState::parse_chat("/p hello"),Some(NetCmd::Chat {chat_type:chat_type::PARTY,message,..}) if message=="hello"));
        assert!(WorldState::parse_chat("/party ").is_none());
    }

    #[test]
    fn skill_adapter_and_learning_command_wait_for_private_server_acknowledgements(){
        use rz_proto::skills::{ScSkillCatalog,SkillOffer,catalog,reason};
        let mut w=WorldState::from_login(&msg::ScLoginResult {handle:7,..Default::default()},Vec2::ZERO);
        assert!(w.addon_state().learned_skills.is_none());assert!(w.addon_state().job_points.is_none());
        w.on_packet(game::ScSkillList {target:7,modification_type:0,skills:vec![]}.to_packet(),0.0);
        w.on_packet(game::ScExpUpdate {handle:7,exp:0,jp:12}.to_packet(),0.0);
        let catalog=ScSkillCatalog {target:7,job:100,job_level:1,player_level:1,jp:12,status:catalog::READY,entries:vec![SkillOffer {id:42,learned:0,next:1,maximum:3,cost:Some(4),player_level:1,job_level:1,prerequisites:[(0,0);3],passive:false,reason:reason::READY}]};
        w.on_packet(catalog.to_packet(),0.0);
        let NetCmd::Raw(packet)=w.learn_skill(42,1).unwrap() else {panic!("expected learning request")};
        assert_eq!(game::CsLearnSkill::from_packet(&packet).unwrap(),game::CsLearnSkill {target:7,skill_id:42,base_random_skill_id:0,skill_level:1});
        assert!(w.learn_skill(42,2).is_none());assert_eq!(w.addon_state().job_points,Some(12));assert!(w.learned_skills().unwrap().is_empty());
        let skill=game::SkillInfo {skill_id:42,base_skill_level:1,current_skill_level:1,..Default::default()};
        w.on_packet(game::ScExpUpdate {handle:7,exp:0,jp:8}.to_packet(),0.0);
        w.on_packet(game::ScSkillList {target:7,modification_type:0,skills:vec![skill.clone()]}.to_packet(),0.0);
        assert_eq!(w.addon_state().learned_skills,Some(BTreeMap::from([(42,1)])));
        w.on_packet(game::ScSkillList {target:8,modification_type:0,skills:vec![]}.to_packet(),0.0);
        w.on_packet(game::ScSkillList {target:7,modification_type:0,skills:vec![skill.clone(),skill]}.to_packet(),0.0);
        assert_eq!(w.learned_skills().unwrap(),&BTreeMap::from([(42,1)]));
        let mut other=catalog.clone();other.target=8;w.on_packet(other.to_packet(),0.0);assert_eq!(w.skill_catalog(),Some(&catalog));
        w.apply_warp(0.0,0.0,0.0,1);assert_eq!(w.addon_state().job_points,Some(8));assert_eq!(w.learned_skills().unwrap().len(),1);
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
    layer: Option<u8>,
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
    fn renderable(&self, now: f64) -> bool {
        // A monster corpse remains selectable until the server removes/consumes it.
        if self.kind==Kind::Monster {return true;}
        !self.dead_at.is_some_and(|d| now - d > 2.0)
    }

    fn apply_hpmp(&mut self, hp: i32, max_hp: i32, mp: i32, max_mp: i32, now: f64) {
        self.set_vitals(hp, max_hp, mp, max_mp);
        if hp > 0 { self.dead_at = None; }
        else if self.dead_at.is_none() { self.dead_at = Some(now); }
    }
    fn set_vitals(&mut self, hp: i32, max_hp: i32, mp: i32, max_mp: i32) {
        self.hp = hp;
        self.max_hp = max_hp.max(1);
        self.health_known = true;
        self.mana = Some((mp, max_mp));
    }

    fn unit(&self, handle: u32) -> rz_addon::UnitState {
        rz_addon::UnitState {
            handle, name: self.name.clone(), level: Some(self.level),
            layer: self.layer,
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
            layer: None,
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
    casting:HashMap<u32,(u16,u8,f64,u32)>,
    skill_cooldowns:BTreeMap<u32,(f64,u32)>,
    skill_event:Option<rz_addon::SkillEventState>,
    learned_skills:Option<BTreeMap<u32,u8>>,
    skill_catalog:Option<rz_proto::skills::ScSkillCatalog>,
    addon_jp:Option<i64>,
    show_skills:bool,
    skill_page:usize,
    party: Option<rz_proto::party::ScPartyState>,
    show_party: bool,
    clock: Instant,
    handle: u32,
    layer: u8,
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
    pub(crate) fn terrain_validation_view(&mut self, reference: bool, view: usize) {
        let yaw = std::f32::consts::FRAC_PI_2 + 0.4;
        (self.cam_yaw, self.cam_pitch, self.cam_dist) = match view {
            1 => (yaw + std::f32::consts::PI, 0.45, 140.0),
            2 => (yaw, 0.08, 900.0),
            3 => (yaw, 1.35, 900.0),
            _ => (yaw, 0.45, 140.0),
        };
        if let Some(terrain) = &mut self.terrain { terrain.validation_reference(reference); }
    }

    pub(crate) fn terrain_cache_snapshot(&self) -> serde_json::Value {
        self.terrain.as_ref().map(|t| t.cache_snapshot()).unwrap_or(serde_json::Value::Null)
    }

    pub(crate) fn terrain_validation_pixel_cache(&mut self, enabled: bool) {
        if let Some(terrain) = &mut self.terrain { terrain.validation_pixel_cache(enabled); }
    }

    pub(crate) fn terrain_validation_capture(&mut self, path: std::path::PathBuf) {
        if let Some(terrain) = &mut self.terrain { terrain.validation_capture(path); }
    }

    pub fn new(lr: &msg::ScLoginResult) -> Self {
        Self::from_login(lr, mouse_position().into())
    }

    fn from_login(lr: &msg::ScLoginResult, last_mouse: Vec2) -> Self {
        let chat_log = vec![(YELLOW, "Bienvenue dans RappelzRust. Entrée : chat, .help : commandes, I : inventaire.".to_string())];
        let race = lr.race;
        let sex = lr.sex;
        let mut me = Entity::new(lr.name.clone(), Kind::Player, lr.x, lr.y, 1, Some(RigKey::Player { race, sex }));
        me.hp = lr.hp;
        me.max_hp = lr.max_hp.max(1);
        Self {
            clock: Instant::now(),
            casting:HashMap::new(),skill_cooldowns:BTreeMap::new(),skill_event:None,
            handle: lr.handle,
            learned_skills:None,skill_catalog:None,addon_jp:None,show_skills:false,skill_page:0,
            party:None,show_party:true,
            layer: lr.layer,
            name: lr.name.clone(),
            me,
            entities: HashMap::new(),
            speed: 120.0,
            cam_yaw: std::f32::consts::FRAC_PI_2 + 0.4,
            cam_pitch: 0.45,
            cam_dist: 140.0,
            last_mouse,
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
                layer: Some(self.layer),
                health: Some(self.hp), max_health: Some(self.max_hp), mana: Some(self.mp), max_mana: Some(self.max_mp),
            }),
            target: self.target.and_then(|h| self.unit(h)),
            money: self.addon_money,
            party_members: self.party.as_ref().map(|p|p.members.iter().filter(|m|m.handle!=self.handle).count() as u32),
            party: self.party.as_ref().map(|p|p.members.iter().filter(|m|m.handle!=self.handle).map(|m|rz_addon::UnitState {
                handle:m.handle,name:m.name.clone(),level:Some(m.level),layer:Some(m.layer),
                health:Some(m.hp),max_health:Some(m.max_hp),mana:Some(m.mp),max_mana:Some(m.max_mp),
            }).collect()),
            party_leader:self.party.as_ref().and_then(|p|(p.leader!=0).then_some(p.leader)),
            party_invite:self.party.as_ref().and_then(|p|p.invite.as_ref().map(|i|i.name.clone())),
            bag_revision: self.bag_revision,
            learned_skills:self.learned_skills.clone(),job_points:self.addon_jp,
            skill_event:self.skill_event.clone(),
        }
    }

    pub fn take_reload_ui(&mut self) -> bool { std::mem::take(&mut self.reload_ui) }

    pub fn addon_message(&mut self, line: String) { self.log(YELLOW, line); }

    pub fn unit(&self, handle: u32) -> Option<rz_addon::UnitState> {
        self.entities.get(&handle).map(|entity| entity.unit(handle))
    }

    pub fn unit_position(&self, handle: u32) -> Option<(f32, f32)> {
        let entity = if handle == self.handle { Some(&self.me) } else { self.entities.get(&handle) }?;
        Some(entity.motion.position(self.now()))
    }

    /// Same corpse-visibility predicate used by the actual creature render loop.
    pub fn unit_renderable(&self, handle: u32) -> Option<bool> {
        if handle == self.handle { return Some(true); }
        self.entities.get(&handle).map(|entity| entity.renderable(self.now()))
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
    pub fn learned_skills(&self)->Option<&BTreeMap<u32,u8>> {self.learned_skills.as_ref()}
    pub fn skill_catalog(&self)->Option<&rz_proto::skills::ScSkillCatalog>{self.skill_catalog.as_ref()}
    pub fn show_skills(&mut self,show:bool){self.show_skills=show;}
    pub fn learn_skill(&self,id:u32,next:u8)->Option<NetCmd> {
        let catalog=self.skill_catalog.as_ref()?;
        if catalog.status!=rz_proto::skills::catalog::READY || !catalog.entries.iter().any(|e|e.id==id && e.id<=i32::MAX as u32 && e.next==next && e.reason==rz_proto::skills::reason::READY){return None;}
        Some(NetCmd::Raw(game::CsLearnSkill {target:self.handle,skill_id:id as i32,base_random_skill_id:0,skill_level:i16::from(next)}.to_packet()))
    }
    pub fn party_state(&self) -> Option<&rz_proto::party::ScPartyState> {self.party.as_ref()}
    pub fn party_action(action:&str) -> NetCmd {
        NetCmd::Chat {chat_type:chat_type::NORMAL,target:String::new(),message:format!(".party {action}")}
    }
    pub fn layer(&self) -> u8 { self.layer }
    pub fn entity_count(&self) -> usize { self.entities.len() }
    pub fn dialog_open(&self) -> bool { self.dialog.is_some() || self.market.is_some() }
    pub fn visible_npc(&self) -> Option<rz_addon::UnitState> {
        self.entities.iter().filter(|(_,e)|e.kind==Kind::Npc).min_by_key(|(h,_)|**h).map(|(h,e)|e.unit(*h))
    }

    /// Programmatic route uses the same local prediction as keyboard/mouse movement.
    pub fn move_path(&mut self, points: Vec<MovePoint>) -> Option<NetCmd> {
        if points.is_empty() || points.len() > 64 || points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) { return None; }
        let now = self.now();
        let (x, y) = self.me.motion.position(now);
        Some(self.start_path(x, y, points, now))
    }

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
                let mut entity = match &e.kind {
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
                entity.layer = Some(e.layer);
                self.entities.insert(e.handle, entity);
            }
            NetEvent::Leave(h) => {
                self.casting.remove(&h);
                self.entities.remove(&h);
                if self.target == Some(h) {
                    self.target = None;
                }
                if self.dialog.as_ref().is_some_and(|d|d.npc_handle==h) { self.dialog=None; }
                if self.market.as_ref().is_some_and(|m|m.npc_handle==h) { self.market=None; }
            }
            NetEvent::Move(m) => {
                let speed = m.speed as f32;
                if m.handle == self.handle {
                    self.speed = speed.max(1.0);
                    self.layer = m.tlayer;
                    return;
                }
                if let Some(e) = self.entities.get_mut(&m.handle) {
                    let (x, y) = e.motion.position(now);
                    e.motion = Motion { x, y, start: now, points: m.points, speed };
                }
            }
            NetEvent::Warp { x, y, z, layer } => self.apply_warp(x, y, z, layer),
            NetEvent::Chat { sender, chat_type: t, message } => {
                let (color, line) = match t {
                    chat_type::WHISPER => (Color::from_rgba(255, 128, 255, 255), format!("[{sender}] chuchote : {message}")),
                    chat_type::NOTICE => (YELLOW, message),
                    chat_type::PARTY => (Color::from_rgba(120,190,255,255),format!("[Groupe] {sender} : {message}")),
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
            gid::TS_SC_SKILL=>{
                if let Ok(event)=game::ScSkill::from_packet(&p){self.apply_restoration(event,now);}
            }
            rz_proto::skills::SC_SKILL_CATALOG=>{
                if let Ok(state)=rz_proto::skills::ScSkillCatalog::from_packet(&p){if state.target==self.handle {self.skill_catalog=Some(state);}}
            }
            gid::TS_SC_SKILL_LIST=>{
                if let Ok(list)=game::ScSkillList::from_packet(&p){self.apply_skill_list(list,now);}
            }
            rz_proto::party::SC_PARTY_STATE => {
                if let Ok(state)=rz_proto::party::ScPartyState::from_packet(&p) {
                    if state.members.is_empty() || state.members.iter().any(|m|m.handle==self.handle) {self.party=Some(state);}
                }
            }
            gid::TS_SC_TARGET => {
                if let Ok(target)=game::Handle::<{gid::TS_SC_TARGET}>::from_packet(&p) {
                    self.apply_server_target(target.0);
                }
            }
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
                        self.addon_jp=Some(e.jp);
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
                        e.apply_hpmp(h.hp, h.max_hp, h.mp, h.max_mp, now);
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

    fn apply_server_target(&mut self, handle:u32) {
        self.target=self.entities.contains_key(&handle).then_some(handle);
    }

    fn apply_skill_list(&mut self,list:game::ScSkillList,now:f64) {
        if list.target!=self.handle || list.modification_type!=0 || list.skills.len()>rz_proto::skills::MAX_ENTRIES {return;}
        let mut accepted=BTreeMap::new();
        let mut cooldowns=BTreeMap::new();
        for skill in list.skills {
            if skill.skill_id<=0 || skill.base_skill_level==0 || skill.base_skill_level>50 || skill.total_cool_time>360_000 || skill.remain_cool_time>skill.total_cool_time || accepted.insert(skill.skill_id as u32,skill.base_skill_level).is_some(){return;}
            cooldowns.insert(skill.skill_id as u32,(now+f64::from(skill.remain_cool_time)/100.,skill.total_cool_time));
        }
        self.learned_skills=Some(accepted);
        self.skill_cooldowns=cooldowns;
    }

    fn apply_warp(&mut self, x: f32, y: f32, _z: f32, layer: u8) {
        self.casting.clear();
        // Terrain supplies rendered height. Previous-layer entities/UI cannot survive the transition.
        if self.layer != layer {
            self.entities.clear(); self.target=None; self.dialog=None; self.market=None;
        }
        self.layer=layer;
        self.me.motion=Motion::at(x,y);
        self.log(YELLOW,format!("Téléporté en {x:.0}, {y:.0}."));
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
    pub fn update_and_draw(&mut self, assets: &mut Assets, ui: &mut RzUi, addon_mouse: bool, modal: bool,input_locked:bool) -> Vec<NetCmd> {
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
        if is_mouse_button_pressed(MouseButton::Left) && !addon_mouse && !input_locked {
            self.press_pos = Some(mouse);
        }
        let dragging = self.press_pos.is_some_and(|p| (p - mouse).length() > 4.0);
        if is_mouse_button_down(MouseButton::Left) && dragging && !self.chat_open && !addon_mouse && !input_locked {
            let d = mouse - self.last_mouse;
            self.cam_yaw += d.x * 0.008;
            self.cam_pitch = (self.cam_pitch + d.y * 0.006).clamp(0.05, 1.45);
        }
        self.last_mouse = mouse;
        let (_, wheel) = mouse_wheel();
        if wheel != 0.0 && !addon_mouse && !input_locked {
            self.cam_dist = (self.cam_dist * if wheel > 0.0 { 0.9 } else { 1.1 }).clamp(30.0, 900.0);
        }

        let focus = vec3(mx, my_h, my);
        let cam = self.camera(focus);
        clear_background(Color::from_rgba(150, 185, 220, 255));
        set_camera(&cam);
        terrain.draw(assets, mx, my, cam.matrix());
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
            if !e.renderable(now) {
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
        if !self.chat_open && !modal && !input_locked {
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
        self.start_path(x, y, points, now)
    }

    fn start_path(&mut self, x: f32, y: f32, points: Vec<MovePoint>, now: f64) -> NetCmd {
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
        if is_key_pressed(KeyCode::K) && !self.chat_open {self.show_skills = !self.show_skills;}
        if self.show_skills {over |= self.draw_skills(assets,ui,out);}
        if is_key_pressed(KeyCode::P) && !self.chat_open {self.show_party = !self.show_party;}
        if self.party.as_ref().is_some_and(|p|p.invite.is_some()) {self.show_party=true;}
        if self.show_party && !self.show_skills {over |= self.draw_party(ui,out);}
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
        if let Some(m)=text.strip_prefix("/p ").or_else(||text.strip_prefix("/party ")) {
            return (!m.trim().is_empty()).then(||chat(chat_type::PARTY,"",m.trim()));
        }
        if text=="/p" || text=="/party" {return None;}
        Some(chat(chat_type::NORMAL, "", text))
    }
}
