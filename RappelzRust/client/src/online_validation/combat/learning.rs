//! Two native clients, real learning requests, durable costs and private re-entry state.
use super::{game, id, mark, read, skill_catalog, unit};
use crate::{world::WorldState, App, NetCmd};
use anyhow::{ensure, Context, Result};
use rz_proto::Message;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    time::{Duration, Instant},
};

#[path="job_progression.rs"] mod job_progression;

#[derive(Debug, PartialEq)]
enum Stage {
    Join,
    Probes,
    First,
    Duplicates,
    Second,
    Upgrade,
    Exhausted,
    Inspect,
    WaitWitness,
    FirstLogout,
    Reenter,
    WaitQuery,
    ObserveReentry,
    FreshQuery,
    FinalLeave,
    Logout,
    Done,
}
pub struct LearningValidation {
    job:Option<job_progression::JobValidation>,
    directory: PathBuf,
    role: String,
    peer: String,
    character: String,
    stage: Stage,
    since: Instant,
    own: u32,
    initial: u32,
    other: u32,
    initial_other: u32,
    sent: bool,
    query_enters: usize,
    requests: Vec<Value>,
    events: BTreeMap<String, usize>,
    messages: Vec<String>,
    steps: Vec<Value>,
    failure: Option<String>,
}
impl LearningValidation {
    pub fn new(plan: PathBuf, app: &App) -> Result<Self> {
        super::super::guard(app)?;
        let p: Value = serde_json::from_slice(&std::fs::read(plan)?)?;
        let role = p["role"].as_str().context("skills role")?.to_string();
        ensure!(role == "owner" || role == "witness", "skills role");
        let peer = p["peer"].as_str().context("skills peer")?.to_string();
        ensure!(peer.starts_with("Lua"), "dedicated peer required");
        let directory = PathBuf::from(p["directory"].as_str().context("report directory")?);
        std::fs::create_dir_all(&directory)?;
        ensure!(
            !directory.join(format!("{role}.json")).exists(),
            "fresh attempt required"
        );
        let job=if p["job_progression"]==true{Some(job_progression::JobValidation::new(&p,app)?)}else{None};
        Ok(Self {
            job,
            directory,
            role,
            peer,
            character: app.auto_enter.clone().context("character")?,
            stage: Stage::Join,
            since: Instant::now(),
            own: 0,
            initial: 0,
            other: 0,
            initial_other: 0,
            sent: false,
            query_enters: 0,
            requests: vec![],
            events: BTreeMap::new(),
            messages: vec![],
            steps: vec![],
            failure: None,
        })
    }
    pub fn message(&mut self, message: &str) {
        if let Some(job)=self.job.as_mut(){job.message(message);return;}
        if let Some(s) = message.strip_prefix("[RappelzDebug] EVENT ") {
            if let Some(e) = s.split_whitespace().next() {
                *self.events.entry(e.into()).or_default() += 1;
            }
        }
        if self.messages.len() < 512 {
            self.messages.push(message.into());
        }
    }
    fn advance(&mut self, stage: Stage) {
        self.stage = stage;
        self.since = Instant::now();
        self.sent = false;
    }
    fn ready(&self, key: &str) -> bool {
        read(&self.directory.join(key)).is_some()
    }
    fn mark(&self, key: &str) -> Result<()> {
        mark(&self.directory, key, json!({"ready":true}))
    }
    fn request(&mut self, app: &App, target: u32, id: i32, random: u32, level: i16) {
        let r = game::CsLearnSkill {
            target,
            skill_id: id,
            base_random_skill_id: random,
            skill_level: level,
        };
        self.requests
            .push(json!({"target":target,"id":id,"random":random,"level":level}));
        app.send(NetCmd::Raw(r.to_packet()));
    }
    fn learn(&mut self, app: &App, id: u32, level: u8) -> Result<()> {
        let command = app
            .world
            .as_ref()
            .context("world")?
            .learn_skill(id, level)
            .context("production UI refused an expected ready offer")?;
        let NetCmd::Raw(packet) = command else {
            anyhow::bail!("learning command must be raw");
        };
        let r = game::CsLearnSkill::from_packet(&packet)?;
        self.request(
            app,
            r.target,
            r.skill_id,
            r.base_random_skill_id,
            r.skill_level,
        );
        Ok(())
    }
    fn record(&mut self, app: &App, stage: &str) {
        let w = app.world.as_ref();
        let s = w.map(WorldState::addon_state).unwrap_or_default();
        self.steps.push(json!({"stage":stage,"player":s.player.as_ref().map(unit),"jp":s.job_points,
            "learned":w.and_then(WorldState::learned_skills).map(|m|m.iter().map(|(id,lv)|json!([id,lv])).collect::<Vec<_>>()),
            "catalog":w.and_then(WorldState::skill_catalog).map(skill_catalog),"xp_jp":w.map(WorldState::experience),
            "position":w.and_then(|w|w.unit_position(self.own)),"money":s.money,"bag_revision":s.bag_revision,
            "display":super::super::AddonValidation::display(app),"events":self.events}));
    }
    fn capture(&self, name: &str) -> Result<()> {
        let path = self.directory.join(format!("{name}.png"));
        macroquad::prelude::get_screen_data().export_png(path.to_str().context("capture path")?);
        Ok(())
    }
    fn learned(world: &WorldState, expected: &[(u32, u8)], jp: i64) -> bool {
        world
            .learned_skills()
            .is_some_and(|m| m.iter().map(|(id, lv)| (*id, *lv)).collect::<Vec<_>>() == expected)
            && world.addon_state().job_points == Some(jp)
            && world.skill_catalog().is_some_and(|c| c.jp == jp)
    }
    fn query(&mut self, app: &App) {
        self.query_enters = app
            .combat_trace
            .as_ref()
            .unwrap()
            .enters
            .iter()
            .filter(|e| e["handle"] == self.other)
            .count();
        app.send(NetCmd::Raw(
            game::Handle::<{ id::TS_CS_QUERY }>(self.other).to_packet(),
        ));
    }
    pub fn tick(&mut self, app: &mut App) -> Result<bool> {
        if let Some(job)=self.job.as_mut(){return job.tick(app);}
        ensure!(
            app.error.is_none()
                && app.addons.error.is_none()
                && app.addons.runtime.errors().is_empty(),
            "client/addon error"
        );
        let trace = app.combat_trace.as_ref().context("trace")?;
        ensure!(trace.errors.is_empty(), "trace error: {:?}", trace.errors);
        let other = if self.role == "owner" {
            "witness"
        } else {
            "owner"
        };
        if let Some(r) = read(&self.directory.join(format!("{other}.json"))) {
            ensure!(r["success"] != false, "peer failure: {}", r["failure"]);
        }
        ensure!(
            self.since.elapsed() < Duration::from_secs(45),
            "skills timeout at {:?}",
            self.stage
        );
        if self.stage == Stage::Logout {
            let count = if self.role == "owner" { 2 } else { 1 };
            if app.world.is_none() && self.events.get("PLAYER_LOGOUT") == Some(&count) {
                self.record(app, "Logout");
                self.advance(Stage::Done);
                return Ok(true);
            }
            return Ok(false);
        }
        if self.stage == Stage::FirstLogout {
            if app.world.is_none() && self.events.get("PLAYER_LOGOUT") == Some(&1) {
                self.record(app, "Lobby");
                app.send(NetCmd::EnterWorld(self.character.clone()));
                self.advance(Stage::Reenter);
            }
            return Ok(false);
        }
        let Some(world) = app.world.as_ref() else {
            return Ok(false);
        };
        let Some(catalog) = world.skill_catalog() else {
            return Ok(false);
        };
        ensure!(
            catalog.status == rz_proto::skills::catalog::READY,
            "learning catalog unavailable: {}",
            catalog.status
        );
        let replies = trace.learn_results.len();
        match self.stage {
            Stage::Join => {
                if app.loading.is_some() {
                    return Ok(false);
                }
                let Some(peer) = world.visible_player(&self.peer) else {
                    return Ok(false);
                };
                let own = world.addon_state().player.context("player")?.handle;
                let jp = if self.role == "owner" { 18 } else { 0 };
                ensure!(
                    Self::learned(world, &[], jp),
                    "fixture is not a fresh learning character"
                );
                if !self.sent {
                    self.own = own;
                    self.initial = own;
                    self.other = peer.handle;
                    self.initial_other = peer.handle;
                    app.auto_enter = None;
                    app.world.as_mut().unwrap().show_skills(true);
                    self.record(app, "Initial");
                    self.mark(&format!("{}.ready", self.role))?;
                    self.sent = true;
                }
                if !self.ready(&format!("{other}.ready")) {
                    return Ok(false);
                }
                if self.role == "owner" {
                    for (target, id, random, level) in [
                        (self.other, 1004, 0, 1),
                        (self.own, 1004, 1, 1),
                        (self.own, 1004, 0, -1),
                        (self.own, 1004, 0, 2),
                        (self.own, 2602, 0, 1),
                        (self.own, 1801, 0, 1),
                        (self.own, 1003, 0, 1),
                        (self.own, i32::MAX, 0, 1),
                    ] {
                        self.request(app, target, id, random, level);
                    }
                } else {
                    self.request(app, self.own, 1004, 0, 1);
                }
                self.advance(Stage::Probes);
            }
            Stage::Probes => {
                if replies < self.requests.len() {
                    return Ok(false);
                }
                ensure!(
                    trace.learn_results.iter().all(|r| *r != 0),
                    "invalid learning request succeeded"
                );
                ensure!(
                    Self::learned(world, &[], if self.role == "owner" { 18 } else { 0 }),
                    "denied requests changed progression"
                );
                if !self.sent {
                    self.record(app, "Denied");
                    self.mark(&format!("{}.denied", self.role))?;
                    self.sent = true;
                }
                if !self.ready(&format!("{other}.denied")) {
                    return Ok(false);
                }
                if self.role == "owner" {
                    self.learn(app, 1004, 1)?;
                    self.advance(Stage::First);
                } else {
                    self.advance(Stage::Inspect);
                }
            }
            Stage::First => {
                if replies == 9 && Self::learned(world, &[(1004, 1)], 14) {
                    self.record(app, "FirstLearned");
                    self.request(app, self.own, 1004, 0, 1);
                    self.request(app, self.own, 1004, 0, 1);
                    self.advance(Stage::Duplicates);
                }
            }
            Stage::Duplicates => {
                if replies == 11 {
                    ensure!(
                        trace.learn_results[9..].iter().all(|r| *r != 0)
                            && Self::learned(world, &[(1004, 1)], 14),
                        "duplicate request debited JP"
                    );
                    self.record(app, "DuplicatesDenied");
                    self.learn(app, 1003, 1)?;
                    self.advance(Stage::Second);
                }
            }
            Stage::Second => {
                if replies == 12 && Self::learned(world, &[(1003, 1), (1004, 1)], 10) {
                    self.record(app, "SecondLearned");
                    self.learn(app, 1003, 2)?;
                    self.advance(Stage::Upgrade);
                }
            }
            Stage::Upgrade => {
                if replies == 13 && Self::learned(world, &[(1003, 2), (1004, 1)], 0) {
                    self.record(app, "Upgraded");
                    self.capture("owner-upgraded")?;
                    self.request(app, self.own, 1003, 0, 3);
                    self.request(app, self.own, 1004, 0, 2);
                    self.advance(Stage::Exhausted);
                }
            }
            Stage::Exhausted => {
                if replies == 15 {
                    ensure!(
                        trace.learn_results[13..].iter().all(|r| *r != 0)
                            && Self::learned(world, &[(1003, 2), (1004, 1)], 0),
                        "exhausted or capped learning succeeded"
                    );
                    self.record(app, "Exhausted");
                    self.mark("owner.learned")?;
                    self.advance(Stage::WaitWitness);
                }
            }
            Stage::Inspect => {
                if self.ready("owner.learned") && !self.sent {
                    self.query(app);
                    self.request(app, self.other, 1004, 0, 1);
                    self.sent = true;
                }
                if self.sent
                    && replies == 2
                    && trace
                        .enters
                        .iter()
                        .filter(|e| e["handle"] == self.other)
                        .count()
                        > self.query_enters
                {
                    ensure!(
                        trace.learn_results.iter().all(|r| *r != 0) && Self::learned(world, &[], 0),
                        "witness learned or received foreign progression"
                    );
                    self.record(app, "Inspected");
                    self.capture("witness")?;
                    self.mark("witness.inspected")?;
                    self.advance(Stage::ObserveReentry);
                }
            }
            Stage::WaitWitness => {
                if self.ready("witness.inspected") {
                    app.send(NetCmd::ReturnLobby);
                    self.advance(Stage::FirstLogout);
                }
            }
            Stage::Reenter => {
                if app.loading.is_some() {
                    return Ok(false);
                }
                let own = world
                    .addon_state()
                    .player
                    .context("re-entered player")?
                    .handle;
                if !Self::learned(world, &[(1003, 2), (1004, 1)], 0) {
                    return Ok(false);
                }
                ensure!(
                    own != self.initial,
                    "re-entry must load a fresh character session"
                );
                self.own = own;
                app.world.as_mut().unwrap().show_skills(true);
                self.record(app, "Reentered");
                self.mark("owner.reentered")?;
                self.advance(Stage::WaitQuery);
            }
            Stage::ObserveReentry => {
                let Some(peer) = world
                    .visible_player(&self.peer)
                    .filter(|p| p.handle != self.initial_other)
                else {
                    return Ok(false);
                };
                if self.ready("owner.reentered") {
                    self.other = peer.handle;
                    self.query(app);
                    self.advance(Stage::FreshQuery);
                }
            }
            Stage::FreshQuery => {
                if trace
                    .enters
                    .iter()
                    .filter(|e| e["handle"] == self.other)
                    .count()
                    > self.query_enters
                {
                    ensure!(
                        Self::learned(world, &[], 0),
                        "fresh query exposed foreign skills"
                    );
                    self.record(app, "FreshQuery");
                    self.mark("witness.queried")?;
                    self.advance(Stage::FinalLeave);
                }
            }
            Stage::WaitQuery => {
                if self.ready("witness.queried") {
                    self.capture("owner-reentered")?;
                    app.send(NetCmd::ReturnLobby);
                    self.advance(Stage::Logout);
                }
            }
            Stage::FinalLeave => {
                if trace.leaves.contains(&self.other) && world.unit(self.other).is_none() {
                    self.record(app, "PeerLogout");
                    app.send(NetCmd::ReturnLobby);
                    self.advance(Stage::Logout);
                }
            }
            Stage::FirstLogout | Stage::Logout | Stage::Done => {}
        }
        Ok(false)
    }
    pub fn fail(&mut self, error: anyhow::Error) {
        if let Some(job)=self.job.as_mut(){job.fail(error);return;}
        self.failure = Some(format!("{error:#}"));
        let _ = self.capture(&format!("{}-failure", self.role));
    }
    pub fn finish(&self, app: &App) -> Result<bool> {
        if let Some(job)=self.job.as_ref(){return job.finish(app);}
        let success = self.stage == Stage::Done && self.failure.is_none();
        mark(
            &self.directory,
            &format!("{}.json", self.role),
            json!({"scenario":"skills","role":self.role,"success":success,"failure":self.failure,
            "own":self.own,"initial":self.initial,"other":self.other,"initial_other":self.initial_other,"requests":self.requests,"events":self.events,"messages":self.messages,"steps":self.steps,"trace":app.combat_trace.as_ref().context("trace")?.json()}),
        )?;
        Ok(success)
    }
}
