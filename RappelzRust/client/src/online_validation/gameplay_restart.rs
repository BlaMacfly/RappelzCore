//! Real movement/combat, durable checkpoints, owned server death and recovery.
use crate::{terrain::AuditHandle, App, NetCmd, Screen};
use anyhow::{ensure, Context, Result};
use rz_proto::{msg::MovePoint,Message};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Debug, PartialEq)]
enum Stage {
    Join,
    CheckpointMove,
    CheckpointWait,
    Initial,
    Active,
    BeforeSave,
    PeerReturn,
    PeerLobby,
    PendingSave,
    Down,
    Reconnect,
    Rejoin,
    Quiet,
    Recovered,
    PostFight,
    ResumeSave,
    SaveLobby,
    Lobby,
    Durable,
    Logout,
    Done,
}
pub struct GameplayRestartValidation {
    directory: PathBuf,
    role: String,
    peer: String,
    character: String,
    password: String,
    interrupt_save: bool,
    durable_rewards: bool,
    money_level: bool,
    database_restart: bool,
    ambiguous_commit: bool,
    second_fight: bool,
    fight_boundary: u64,
    stage: Stage,
    started: Instant,
    since: Instant,
    own: u32,
    peer_handle: u32,
    monster: u32,
    monster_hp: i32,
    announced: bool,
    boundary: u64,
    audits: Vec<AuditHandle>,
    steps: Vec<Value>,
    samples: Vec<Value>,
    events: BTreeMap<String, usize>,
    failure: Option<String>,
}
impl GameplayRestartValidation {
    pub fn new(plan: PathBuf, app: &App) -> Result<Self> {
        super::guard(app)?;
        let (_, port) = crate::config::realmlist(&app.root, &app.config.get("locale", "frFR"));
        ensure!(
            port != 4615 && crate::arg_value("-validate-world-proxy").is_none(),
            "separate auth and direct world sockets required"
        );
        let p: Value = serde_json::from_slice(&std::fs::read(plan)?)?;
        ensure!(!(p["interrupt_save"]==true && p["durable_rewards"]==true), "choose one save strategy");
        ensure!(p["money_level"]!=true || p["durable_rewards"]==true,"money/level requires durable reward mode");
        ensure!(p["database_restart"]!=true || p["money_level"]==true,"database restart requires currency profile");
        ensure!(p["ambiguous_commit"]!=true || (p["money_level"]==true && p["database_restart"]!=true),"choose isolated COMMIT or database failure");
        let role = p["role"].as_str().context("role")?.to_owned();
        ensure!(role == "owner" || role == "witness", "role");
        let peer = p["peer"].as_str().context("peer")?.to_owned();
        ensure!(peer.starts_with("Lua"), "dedicated peer");
        let directory = PathBuf::from(p["directory"].as_str().context("directory")?);
        ensure!(
            directory.is_dir() && !directory.join(format!("{role}.json")).exists(),
            "fresh report"
        );
        let login = crate::arg_value("-login").context("login")?;
        let (account, password) = login.split_once(':').context("login format")?;
        ensure!(account == app.account, "login mismatch");
        Ok(Self {
            directory,
            role,
            peer,
            character: app.auto_enter.clone().context("character")?,
            password: password.to_owned(),
            interrupt_save: p["interrupt_save"].as_bool().unwrap_or(false),
            durable_rewards: p["durable_rewards"].as_bool().unwrap_or(false),
            money_level: p["money_level"].as_bool().unwrap_or(false),
            database_restart: p["database_restart"].as_bool().unwrap_or(false),
            ambiguous_commit:p["ambiguous_commit"].as_bool().unwrap_or(false),
            second_fight:false,
            fight_boundary:0,
            stage: Stage::Join,
            started: Instant::now(),
            since: Instant::now(),
            own: 0,
            peer_handle: 0,
            monster: 0,
            monster_hp: 0,
            announced: false,
            boundary: 0,
            audits: Vec::new(),
            steps: Vec::new(),
            samples: Vec::new(),
            events: BTreeMap::new(),
            failure: None,
        })
    }
    pub(super) fn before_draw(&self, app: &mut App) {
        if let Some(w) = &mut app.world {
            w.terrain_validation_policy(false, true, true);
        }
    }
    fn other(&self) -> &str {
        if self.role == "owner" {
            "witness"
        } else {
            "owner"
        }
    }
    fn checkpoint(&self, role: &str) -> [f32; 2] {
        [if self.money_level {9452.} else {8452.}, if role == "owner" { 7235. } else { 7255. }]
    }
    fn final_position(&self, role: &str) -> [f32; 2] {
        let mut p = self.checkpoint(role);
        if role == "witness" {
            p[0] += 16.;
        }
        p
    }
    fn baseline_xp(&self)->(i64,i64) {if self.money_level && self.role=="owner" {(799,7)} else {(0,0)}}
    fn awarded_xp(&self)->(i64,i64) {if self.role=="owner" {if self.money_level {(4,10)} else {(5,3)}} else {(0,0)}}
    fn award_ready(&self,app:&App)->bool {
        let Some(w)=&app.world else {return false};
        if w.experience()!=self.fight_expected(){return false;}
        if self.money_level {
            let s=w.addon_state();let Some(p)=s.player else {return false};
            p.level==Some(if self.role=="owner" {3} else {1}) && s.money.is_some_and(|g|if self.role=="owner" {(1008..=1014).contains(&g)} else {g==2000})
        } else {true}
    }
    fn fight_expected(&self)->(i64,i64) {if self.second_fight {(9,13)} else {self.awarded_xp()}}
    fn fight_hits(&self,app:&App)->Vec<Value> {
        if !self.second_fight {return self.current_hits(app);}
        let t=self.trace(app);
        t["hits"].as_array().unwrap().iter().zip(t["hit_orders"].as_array().unwrap()).filter(|(_,o)|o.as_u64().unwrap_or(0)>self.fight_boundary).map(|(h,_)|h.clone()).collect()
    }
    fn ready(&self, name: &str) -> bool {
        self.directory.join(name).exists()
    }
    fn read(&self, name: &str) -> Option<Value> {
        serde_json::from_slice(&std::fs::read(self.directory.join(name)).ok()?).ok()
    }
    fn write(&self, name: &str, value: Value) -> Result<()> {
        std::fs::write(
            self.directory.join(name),
            serde_json::to_vec_pretty(&value)?,
        )?;
        Ok(())
    }
    fn trace(&self, app: &App) -> Value {
        app.combat_trace.as_ref().unwrap().json()
    }
    fn advance(&mut self, stage: Stage) {
        self.stage = stage;
        self.since = Instant::now();
        self.announced = false;
    }
    fn current_hits(&self, app: &App) -> Vec<Value> {
        let t = self.trace(app);
        let first = t["logins"]
            .as_array()
            .unwrap()
            .last()
            .and_then(|v| v["order"].as_u64())
            .unwrap_or(0);
        t["hits"]
            .as_array()
            .unwrap()
            .iter()
            .zip(t["hit_orders"].as_array().unwrap())
            .filter(|(_, o)| o.as_u64().unwrap_or(0) > first)
            .map(|(h, _)| h.clone())
            .collect()
    }
    fn current_moves(&self, app: &App) -> Vec<Value> {
        let t = self.trace(app);
        let first = t["logins"]
            .as_array()
            .unwrap()
            .last()
            .and_then(|v| v["order"].as_u64())
            .unwrap_or(0);
        t["moves"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| {
                v["order"].as_u64().unwrap_or(0) > first
                    && (v["handle"] == self.own || v["handle"] == self.peer_handle)
            })
            .cloned()
            .collect()
    }
    fn record(&mut self, app: &App, label: &str) {
        let w = app.world.as_ref();
        let s = w.map(|w| w.addon_state()).unwrap_or_default();
        self.steps.push(json!({"stage":label,"elapsed_ms":self.started.elapsed().as_millis(),"stage_ms":self.since.elapsed().as_millis(),"own":self.own,"peer_handle":self.peer_handle,"monster":self.monster,"boundary":self.boundary,"fight_boundary":self.fight_boundary,
            "player":s.player.as_ref().map(|p|json!({"handle":p.handle,"name":p.name,"level":p.level,"hp":p.health,"mp":p.mana,"max_hp":p.max_health,"max_mp":p.max_mana})),"position":w.and_then(|w|w.unit_position(self.own)),
            "motion":w.and_then(|w|w.validation_motion(self.own)),"peer_motion":w.and_then(|w|w.validation_motion(self.peer_handle)),"peer_unit":w.and_then(|w|w.unit(self.peer_handle)).map(|p|json!({"level":p.level,"hp":p.health,"mp":p.mana,"max_hp":p.max_health,"max_mp":p.max_mana})),"target":s.target.map(|p|json!({"handle":p.handle,"hp":p.health})),
            "monster_hp":w.and_then(|w|w.unit(self.monster)).and_then(|p|p.health),"xp_jp":w.map(|w|w.experience()),"money":s.money,"layer":w.map(|w|w.layer()),
            "hits":self.current_hits(app),"moves":self.current_moves(app),"order":self.trace(app)["order"],"events":self.events,"audit_instance":self.audits.last().map(|a|a.instance()),"error":app.error}));
    }
    fn move_to(&self, app: &mut App, position: [f32; 2]) -> Result<()> {
        let cmd = app
            .world
            .as_mut()
            .context("world")?
            .move_path(vec![MovePoint {
                x: position[0],
                y: position[1],
            }])
            .context("move")?;
        app.send(cmd);
        Ok(())
    }
    fn at(&self, app: &App, role: &str, pos: [f32; 2]) -> bool {
        let h = if role == self.role {
            self.own
        } else {
            self.peer_handle
        };
        app.world
            .as_ref()
            .and_then(|w| w.unit_position(h))
            .is_some_and(|(x, y)| (x - pos[0]).abs() < 0.05 && (y - pos[1]).abs() < 0.05)
    }
    fn begin_attack(&mut self, app: &mut App) -> Result<()> {
        let w = app.world.as_mut().context("world")?;
        let m = w.living_monster().context("living monster")?;
        self.monster = m.handle;
        self.monster_hp = m.max_health.context("monster max HP")?;
        ensure!(self.monster_hp == 105, "verified local monster required");
        let select = w.select_target(self.monster).context("select")?;
        let attack = w.attack_target(self.monster).context("attack")?;
        app.send(select);
        app.send(attack);
        Ok(())
    }
    fn capture(&self, app: &mut App, label: &str) -> Result<bool> {
        let path = self
            .directory
            .join(format!("{}-{label}.ground.png", self.role));
        if !path.exists() {
            app.world
                .as_mut()
                .context("world")?
                .terrain_validation_capture(path);
            return Ok(false);
        }
        macroquad::prelude::get_screen_data().export_png(
            self.directory
                .join(format!("{}-{label}.png", self.role))
                .to_str()
                .context("path")?,
        );
        Ok(true)
    }
    pub fn message(&mut self, m: &str) {
        if let Some(e) = m
            .strip_prefix("[RappelzDebug] EVENT ")
            .and_then(|m| m.split_whitespace().next())
        {
            *self.events.entry(e.to_owned()).or_default() += 1;
        }
    }
    pub fn tick(&mut self, app: &mut App) -> Result<bool> {
        ensure!(
            self.started.elapsed() < Duration::from_secs(220),
            "gameplay restart timeout at {:?}",
            self.stage
        );
        if let Some(c) = self.read("server-process.json") {
            ensure!(c["success"] == true, "server controller failed");
        }
        ensure!(
            app.addons.error.is_none() && app.addons.runtime.errors().is_empty(),
            "addon errors"
        );
        if !matches!(self.stage, Stage::Active | Stage::PendingSave | Stage::Down) && !(self.database_restart && self.second_fight) && !(self.ambiguous_commit && self.role=="owner" && self.stage==Stage::BeforeSave) {
            ensure!(app.error.is_none(), "unexpected error {:?}", app.error);
        }
        ensure!(
            self.trace(app)["errors"].as_array().unwrap().is_empty(),
            "trace decoding error"
        );
        if let Some(w) = &app.world {
            if let Some(a) = w.terrain_validation_audit() {
                if !self.audits.iter().any(|v| v.instance() == a.instance()) {
                    ensure!(self.audits.len() < 3, "extra world");
                    self.audits.push(a);
                }
            }
            let c = w.terrain_cache_snapshot();
            ensure!(c["worker_error"].is_null(), "worker error");
            if !c.is_null() {
                self.samples.push(c);
                ensure!(self.samples.len() < 16000, "sample bound");
            }
        }
        match self.stage {
            Stage::Join => {
                if app.loading.is_some() {
                    return Ok(false);
                }
                let Some(w) = &app.world else {
                    return Ok(false);
                };
                let Some(p) = w.addon_state().player else {
                    return Ok(false);
                };
                let Some(peer) = w.visible_player(&self.peer) else {
                    return Ok(false);
                };
                ensure!(
                    p.level == Some(if self.money_level && self.role=="owner" {2} else {1})
                        && p.health == Some(if self.money_level && self.role=="owner" {120} else {100})
                        && p.mana == Some(if self.money_level && self.role=="owner" {60} else {50})
                        && w.experience() == self.baseline_xp(),
                    "fresh level-one fixtures required"
                );
                self.own = p.handle;
                self.peer_handle = peer.handle;
                app.auto_enter = None;
                self.record(app, "Joined");
                self.move_to(app, self.checkpoint(&self.role))?;
                self.advance(Stage::CheckpointMove);
            }
            Stage::CheckpointMove => {
                if self.at(app, &self.role, self.checkpoint(&self.role))
                    && self.at(app, &self.peer_role(), self.checkpoint(self.other()))
                {
                    self.record(app, "CheckpointArrived");
                    self.write(
                        &format!("{}.checkpoint-ready", self.role),
                        json!({"position":self.checkpoint(&self.role),"handle":self.own}),
                    )?;
                    self.advance(Stage::CheckpointWait);
                }
            }
            Stage::CheckpointWait => {
                if self.ready("checkpoint-verified") {
                    self.advance(Stage::Initial);
                }
            }
            Stage::Initial | Stage::Recovered => {
                if !app
                    .world
                    .as_ref()
                    .is_some_and(|w| w.terrain_cache_snapshot()["ready"] == true)
                    || self.since.elapsed() < Duration::from_millis(300)
                {
                    return Ok(false);
                }
                let label = if self.stage == Stage::Initial {
                    "initial"
                } else {
                    "recovered"
                };
                if !self.announced && self.capture(app, label)? {
                    self.record(
                        app,
                        if label == "initial" {
                            "CheckpointSaved"
                        } else {
                            "RecoveredQuiet"
                        },
                    );
                    self.write(&format!("{}.{label}", self.role), json!({"ready":true}))?;
                    self.announced = true;
                }
                if self.announced
                    && self.ready(&format!("{}.{label}", self.other()))
                    && self.ready(&format!("release-{label}"))
                {
                    if self.stage == Stage::Initial && self.ready("actions-release") {
                        if self.interrupt_save || self.durable_rewards {
                            self.record(app, "BeforeSaveActionsStarted");
                            if self.role == "witness" {
                                self.move_to(app, self.final_position(&self.role))?;
                            }
                            self.begin_attack(app)?;
                            self.advance(Stage::BeforeSave);
                            return Ok(false);
                        }
                        let [x, y] = self.checkpoint(&self.role);
                        let circuit = vec![
                            MovePoint { x: x + 24., y },
                            MovePoint {
                                x: x + 24.,
                                y: y + 24.,
                            },
                            MovePoint { x, y: y + 24. },
                            MovePoint { x, y },
                        ];
                        let route=circuit.iter().cloned().cycle().take(16).collect();
                        let cmd = app
                            .world
                            .as_mut()
                            .unwrap()
                            .move_path(route)
                            .context("active route")?;
                        app.send(cmd);
                        self.begin_attack(app)?;
                        self.record(app, "ActionsStarted");
                        self.advance(Stage::Active);
                    } else if self.stage == Stage::Recovered {
                        if self.durable_rewards {
                            if !self.ready(&format!("{}.post-ready",self.role)) {
                                self.boundary=self.trace(app)["order"].as_u64().unwrap();
                                self.record(app,"PostActionsStarted");
                                self.write(&format!("{}.post-ready",self.role),json!({"ready":true}))?;
                                return Ok(false);
                            }
                            if !self.ready(&format!("{}.post-ready",self.other())) {return Ok(false);}
                            if self.role=="witness" {self.move_to(app,self.final_position(&self.role))?;}
                            self.advance(Stage::ResumeSave);
                            return Ok(false);
                        }
                        self.boundary = self.trace(app)["order"].as_u64().unwrap();
                        self.record(app, "PostActionsStarted");
                        if self.role == "owner" {
                            self.begin_attack(app)?;
                        } else {
                            self.move_to(app, self.final_position(&self.role))?;
                            self.begin_attack(app)?;
                        }
                        self.advance(Stage::PostFight);
                    }
                }
            }
            Stage::Active => {
                if self.ready("server-killed.json") {
                    self.record(app, "ServerKilled");
                    self.advance(Stage::Down);
                } else if let Some(w) = &app.world {
                    let hits = self.current_hits(app);
                    let own_hits = hits
                        .iter()
                        .filter(|h| h["attacker"] == self.own && h["target"] == self.monster)
                        .count();
                    let peer_hits = hits
                        .iter()
                        .filter(|h| {
                            h["attacker"] == self.peer_handle && h["target"] == self.monster
                        })
                        .count();
                    let retaliations = hits
                        .iter()
                        .filter(|h| {
                            h["attacker"] == self.monster
                                && (h["target"] == self.own || h["target"] == self.peer_handle)
                        })
                        .count();
                    let pos = w.unit_position(self.own).context("position")?;
                    let cp = self.checkpoint(&self.role);
                    let state = w.addon_state();
                    let monster = w.unit(self.monster).context("monster")?;
                    let moving = w.validation_motion(self.own).unwrap()["moving"].clone();
                    let peer_moving =
                        w.validation_motion(self.peer_handle).unwrap()["moving"].clone();
                    self.write(&format!("{}.activity.json",self.role),json!({"role":self.role,"unix_ms":SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis(),"instance":self.audits[0].instance(),"own":self.own,"peer":self.peer_handle,"monster":self.monster,
                        "moving":moving,"peer_moving":peer_moving,"position":pos,"checkpoint":cp,"distance_from_checkpoint":((pos.0-cp[0]).powi(2)+(pos.1-cp[1]).powi(2)).sqrt(),
                        "own_hits":own_hits,"peer_hits":peer_hits,"retaliations":retaliations,"route_acks":self.current_moves(app).iter().filter(|m|m["handle"]==self.own).count(),"own_hp":state.player.unwrap().health,"monster_hp":monster.health,"monster_max_hp":monster.max_health}))?;
                }
            }
            Stage::Down => {
                if app.world.is_none() {
                    ensure!(
                        matches!(app.screen, Screen::Login)
                            && self.events.get("PLAYER_LOGOUT") == Some(&1),
                        "missing crash logout"
                    );
                    ensure!(
                        self.audits[0].snapshot()["events"]
                            .as_array()
                            .unwrap()
                            .last()
                            .is_some_and(|e| e["event"] == "joined"),
                        "old worker alive"
                    );
                    self.record(app, "WorldDropped");
                    app.error = None;
                    self.write(&format!("{}.dropped", self.role), json!({"joined":true,"unix_ms":SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis()}))?;
                    self.advance(Stage::Reconnect);
                }
            }
            Stage::Reconnect => {
                ensure!(app.world.is_none(), "phantom world");
                if self.ready("clients-reconnect") && self.ready(if self.ambiguous_commit {"commit-response-reset.json"} else if self.database_restart {"database-up-1.json"} else {"server-up-1.json"}) {
                    self.record(app, "DowntimeHeld");
                    let (host, port) =
                        crate::config::realmlist(&app.root, &app.config.get("locale", "frFR"));
                    if self.ambiguous_commit && self.role=="witness" {
                        app.send(NetCmd::EnterWorld(self.character.clone()));self.advance(Stage::Rejoin);return Ok(false);
                    }
                    app.auto_enter = Some(self.character.clone());
                    app.send(NetCmd::Login {
                        host,
                        port,
                        account: app.account.clone(),
                        password: self.password.clone(),
                    });
                    self.advance(Stage::Rejoin);
                }
            }
            Stage::Rejoin => {
                if app.loading.is_some() {
                    return Ok(false);
                }
                let Some(w) = &app.world else {
                    return Ok(false);
                };
                let Some(p) = w.addon_state().player else {
                    return Ok(false);
                };
                let Some(peer) = w.visible_player(&self.peer) else {
                    return Ok(false);
                };
                self.own = p.handle;
                self.peer_handle = peer.handle;
                self.monster = w.living_monster().context("fresh monster")?.handle;
                app.auto_enter = None;
                ensure!(
                    self.audits.len() == 2
                        && self.audits[0].instance() != self.audits[1].instance(),
                    "stale world"
                );
                let saved=self.read("reward-committed.json");
                let expected=if self.durable_rewards {self.awarded_xp()} else {(0,0)};
                let raw=self.trace(app)["logins"].as_array().unwrap().last().unwrap().clone();
                let vitals_ok=if self.durable_rewards {
                    let profile=&saved.as_ref().context("reward checkpoint")?[&self.role];
                    raw["hp"]==profile["health"] && raw["mp"]==profile["mp"]
                } else {p.health==Some(100)};
                ensure!(
                    self.at(app, &self.role, self.checkpoint(&self.role))
                        && self.at(app, self.other(), self.checkpoint(self.other()))
                        && vitals_ok
                        && w.experience() == expected,
                    "last durable checkpoint not restored"
                );
                self.record(app, "CheckpointRestored");
                self.write(
                    &format!("{}.reentered", self.role),
                    json!({"handle":self.own}),
                )?;
                self.advance(Stage::Quiet);
            }
            Stage::Quiet => {
                let w = app.world.as_ref().context("world")?;
                ensure!(
                    self.current_hits(app).is_empty() && self.current_moves(app).is_empty(),
                    "old movement or attack replayed after reboot"
                );
                ensure!(
                    w.validation_motion(self.own).unwrap()["moving"] == false
                        && w.addon_state().target.is_none()
                        && w.unit(self.monster).and_then(|m| m.health) == Some(105),
                    "stale transient state"
                );
                if self.since.elapsed() >= Duration::from_millis(3200)
                    && self.ready(&format!("{}.reentered", self.other()))
                {
                    self.record(app, "NoReplay");
                    self.advance(Stage::Recovered);
                }
            }
            Stage::BeforeSave | Stage::PostFight => {
                if self.ambiguous_commit {
                    if self.ready("commit-response-reset.json") {
                        self.record(app,"CommitResponseLost");
                        if self.role=="owner" {self.advance(Stage::Down);} else {self.move_to(app,self.checkpoint(&self.role))?;self.advance(Stage::PeerReturn);}
                        return Ok(false);
                    }
                    if self.role=="owner" && app.world.is_none() {return Ok(false);}
                }
                if self.database_restart && self.second_fight {
                    if self.ready("database-killed.json") {
                        self.record(app,"DatabaseKilled");self.second_fight=false;self.advance(Stage::Down);return Ok(false);
                    }
                    if app.world.is_none() {return Ok(false);}
                }
                let before = self.stage == Stage::BeforeSave;
                if self.role=="witness" && !self.announced && self.fight_hits(app).iter().filter(|h|h["attacker"]==self.own && h["target"]==self.monster).count()>=(if self.second_fight {2} else {3}) {
                    app.send(NetCmd::Raw(rz_proto::msg_game::CsCancelAction{0:self.own}.to_packet()));
                    self.record(app,if self.second_fight {"SecondAssistStopped"} else if before {"BeforeSaveAssistStopped"} else {"AssistStopped"});self.announced=true;
                }
                let w = app.world.as_ref().context("world")?;
                let hits = self.fight_hits(app);
                let owner = if self.role == "owner" {
                    self.own
                } else {
                    self.peer_handle
                };
                if !hits.iter().any(|h| {
                    h["attacker"] == owner && h["target"] == self.monster && h["target_hp"] == 0
                }) {
                    return Ok(false);
                }
                if self.database_restart && self.second_fight && self.role=="witness" {
                    self.record(app,"SecondKillObserved");self.write("witness.second-killed",json!({"ready":true}))?;
                    self.advance(Stage::PendingSave);return Ok(false);
                }
                if self.role == "owner" {
                    if !self.award_ready(app) {
                        if before && self.durable_rewards && w.experience()==(if self.second_fight {self.awarded_xp()} else {self.baseline_xp()}) {
                            if !self.announced {self.record(app,if self.second_fight {"SecondRewardWithheld"} else {"RewardWithheld"});self.announced=true;}
                            self.write("owner.pending-reward.json",json!({"unix_ms":SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis(),"xp_jp":app.world.as_ref().unwrap().experience(),"level":app.world.as_ref().unwrap().addon_state().player.unwrap().level,"money":app.world.as_ref().unwrap().addon_state().money,"order":self.trace(app)["order"],"monster":self.monster}))?;
                        }
                        return Ok(false);
                    }
                    ensure!(!self.second_fight && !self.ambiguous_commit,"uncertain reward was unexpectedly announced");
                    self.write(if before {"owner.before-save-rewarded"} else {"owner.rewarded"}, json!({"xp_jp":self.awarded_xp()}))?;
                } else {
                    ensure!(w.experience() == (0, 0), "private reward leaked");
                }
                if !self.at(app, &self.role, self.final_position(&self.role))
                    || !self.at(app, self.other(), self.final_position(self.other()))
                    || !self.ready(if before {"owner.before-save-rewarded"} else {"owner.rewarded"})
                {
                    return Ok(false);
                }
                if before {
                    let xp_jp = w.experience();
                    self.record(app, "BeforeInterruptedSave");
                    self.write(&format!("{}.save-ready",self.role),json!({"role":self.role,"own":self.own,"peer":self.peer_handle,"monster":self.monster,"xp_jp":xp_jp,"position":self.final_position(&self.role)}))?;
                    self.advance(Stage::PendingSave);
                    return Ok(false);
                }
                self.record(app, "CombatRecovered");
                self.write(
                    &format!("{}.finish-ready", self.role),
                    json!({"ready":true}),
                )?;
                self.advance(Stage::SaveLobby);
            }
            Stage::PeerReturn => {
                if self.at(app,&self.role,self.checkpoint(&self.role)) {
                    self.record(app,"PeerReturnedToCheckpoint");app.send(NetCmd::ReturnLobby);self.advance(Stage::PeerLobby);
                }
            }
            Stage::PeerLobby => {
                if app.world.is_none() && self.events.get("PLAYER_LOGOUT")==Some(&1) {
                    ensure!(matches!(app.screen,Screen::Characters),"peer transport should stay connected");
                    ensure!(self.audits[0].snapshot()["events"].as_array().unwrap().last().is_some_and(|e|e["event"]=="joined"),"peer worker alive");
                    self.record(app,"PeerLobbySaved");
                    self.write("witness.dropped",json!({"joined":true,"normal_lobby":true,"unix_ms":SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis()}))?;
                    self.advance(Stage::Reconnect);
                }
            }
            Stage::ResumeSave => {
                ensure!(self.current_hits(app).is_empty(),"reward replay triggered combat");
                if self.at(app,&self.role,self.final_position(&self.role)) && self.at(app,self.other(),self.final_position(self.other())) {
                    self.record(app,"RewardRetained");
                    self.write(&format!("{}.finish-ready",self.role),json!({"ready":true}))?;
                    self.advance(Stage::SaveLobby);
                }
            }
            Stage::PendingSave => {
                if self.database_restart {
                    if self.ready("database-killed.json") {
                        self.record(app,"DatabaseKilled");
                        if self.role=="witness" {
                            self.record(app,"SaveDuringDatabaseOutage");app.send(NetCmd::ReturnLobby);
                        }
                        self.second_fight=false;self.advance(Stage::Down);return Ok(false);
                    }
                    if !self.second_fight && self.ready("second-actions-release") && app.world.as_ref().is_some_and(|w|w.living_monster().is_some()) {
                        self.second_fight=true;self.fight_boundary=self.trace(app)["order"].as_u64().unwrap();
                        self.record(app,"SecondActionsStarted");self.begin_attack(app)?;self.advance(Stage::BeforeSave);
                    }
                    return Ok(false);
                }
                if self.ready("server-killed.json") {
                    self.record(app,"ServerKilled");
                    self.advance(Stage::Down);
                }
            }
            Stage::SaveLobby => {
                if self.ready(&format!("{}.finish-ready", self.other())) {
                    self.record(app, "BeforeDurableSave");
                    app.send(NetCmd::ReturnLobby);
                    self.advance(Stage::Lobby);
                }
            }
            Stage::Lobby => {
                if app.world.is_none() && self.events.get("PLAYER_LOGOUT") == Some(&2) {
                    if !self.announced {
                        self.record(app, "SavedLobby");
                        self.write(&format!("{}.saved-lobby", self.role), json!({"saved":true}))?;
                        self.announced = true;
                    }
                    if self.ready("saved-lobby-verified") {
                        app.send(NetCmd::EnterWorld(self.character.clone()));
                        self.advance(Stage::Durable);
                    }
                }
            }
            Stage::Durable => {
                if app.loading.is_some() {
                    return Ok(false);
                }
                let Some(w) = &app.world else {
                    return Ok(false);
                };
                let Some(p) = w.addon_state().player else {
                    return Ok(false);
                };
                let Some(peer) = w.visible_player(&self.peer) else {
                    return Ok(false);
                };
                self.own = p.handle;
                self.peer_handle = peer.handle;
                let expected = self.awarded_xp();
                if w.experience() != expected || !self.award_ready(app) {
                    return Ok(false);
                }
                let saved = self.read("saved-lobby-verified").context("saved profile")?;
                let t = self.trace(app);
                let login = t["logins"].as_array().unwrap().last().unwrap();
                ensure!(
                    login["hp"] == saved[&self.role]["health"]
                        && login["mp"] == saved[&self.role]["mp"],
                    "saved vitals not loaded by actual login"
                );
                ensure!(
                    self.at(app, &self.role, self.final_position(&self.role))
                        && self.at(app, self.other(), self.final_position(self.other()))
                        && self.current_hits(app).is_empty()
                        && self.current_moves(app).is_empty()
                        && w.addon_state().target.is_none(),
                    "stale state after durable re-entry"
                );
                if !self.announced {
                    self.record(app, "DurableReentry");
                    self.write(&format!("{}.durable", self.role), json!({"verified":true}))?;
                    self.announced = true;
                }
                if self.ready(&format!("{}.durable", self.other())) {
                    app.send(NetCmd::ReturnLobby);
                    self.advance(Stage::Logout);
                }
            }
            Stage::Logout => {
                if app.world.is_none() && self.events.get("PLAYER_LOGOUT") == Some(&3) {
                    ensure!(
                        self.audits.len() == 3
                            && self.audits.iter().all(|a| a.snapshot()["events"]
                                .as_array()
                                .unwrap()
                                .last()
                                .is_some_and(|e| e["event"] == "joined")),
                        "worker not joined"
                    );
                    self.advance(Stage::Done);
                    return Ok(true);
                }
            }
            Stage::Done => return Ok(true),
        }
        Ok(false)
    }
    fn peer_role(&self) -> String {
        self.other().to_owned()
    }
    pub fn fail(&mut self, e: anyhow::Error) {
        self.failure = Some(format!("{e:#}"));
    }
    pub fn finish(&self, app: &App) -> Result<bool> {
        let success = self.stage == Stage::Done && self.failure.is_none();
        self.write(&format!("{}.json",self.role),json!({"scenario":if self.durable_rewards {"reward-durable"} else if self.interrupt_save {"save-rollback"} else {"gameplay-restart"},"money_level":self.money_level,"database_restart":self.database_restart,"ambiguous_commit":self.ambiguous_commit,"success":success,"failure":self.failure,"role":self.role,"character":self.character,"steps":self.steps,"samples":self.samples,"events":self.events,"audits":self.audits.iter().map(|a|a.snapshot()).collect::<Vec<_>>(),"trace":self.trace(app)}))?;
        Ok(success)
    }
}
