//! Direct sockets to an isolated worldserver, externally killed and restarted.
use crate::{terrain::AuditHandle, App, NetCmd, Screen};
use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Debug, PartialEq)]
enum Stage {
    Join,
    Initial,
    FindJob,
    Dropped,
    Reconnect,
    Rejoin,
    Recovered,
    Logout,
    Done,
}
pub struct TerrainRestartValidation {
    directory: PathBuf,
    role: String,
    peer: String,
    character: String,
    password: String,
    stage: Stage,
    started: Instant,
    since: Instant,
    view: usize,
    own: u32,
    old_own: u32,
    peer_handle: u32,
    job: u64,
    instance: u64,
    announced: bool,
    audits: Vec<AuditHandle>,
    steps: Vec<Value>,
    samples: Vec<Value>,
    events: BTreeMap<String, usize>,
    failure: Option<String>,
}
impl TerrainRestartValidation {
    pub fn new(plan: PathBuf, app: &App) -> Result<Self> {
        super::guard(app)?;
        let (_, port) = crate::config::realmlist(&app.root, &app.config.get("locale", "frFR"));
        ensure!(
            port != 4615 && crate::arg_value("-validate-world-proxy").is_none(),
            "separate auth and direct world sockets required"
        );
        let p: Value = serde_json::from_slice(&std::fs::read(plan)?)?;
        let role = p["role"].as_str().context("role")?.to_owned();
        ensure!(role == "owner" || role == "witness", "role");
        let peer = p["peer"].as_str().context("peer")?.to_owned();
        ensure!(peer.starts_with("Lua"), "dedicated peer");
        let directory = PathBuf::from(p["directory"].as_str().context("directory")?);
        ensure!(
            directory.is_dir() && !directory.join(format!("{role}.json")).exists(),
            "fresh report required"
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
            stage: Stage::Join,
            started: Instant::now(),
            since: Instant::now(),
            view: 0,
            own: 0,
            old_own: 0,
            peer_handle: 0,
            job: 0,
            instance: 0,
            announced: false,
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
    fn advance(&mut self, stage: Stage) {
        self.stage = stage;
        self.since = Instant::now();
        self.announced = false;
    }
    fn trace(&self, app: &App) -> Value {
        app.combat_trace.as_ref().unwrap().json()
    }
    fn record(&mut self, app: &App, label: &str) {
        let w = app.world.as_ref();
        let state = w.map(|w| w.addon_state()).unwrap_or_default();
        self.steps.push(json!({"stage":label,"elapsed_ms":self.started.elapsed().as_millis(),"stage_ms":self.since.elapsed().as_millis(),
            "own":self.own,"old_own":self.old_own,"peer_handle":self.peer_handle,"job":self.job,"closing_instance":self.instance,
            "player":state.player.as_ref().map(|p|json!({"name":p.name,"handle":p.handle,"level":p.level,"hp":p.health,"mp":p.mana})),
            "position":w.and_then(|w|w.unit_position(self.own)),"peer":w.and_then(|w|w.visible_player(&self.peer)).map(|p|p.handle),
            "xp_jp":w.map(|w|w.experience()),"money":state.money,"layer":w.map(|w|w.layer()),"cache":w.map(|w|w.terrain_cache_snapshot()),
            "error":app.error,"events":self.events,"audit_instance":self.audits.last().map(|a|a.instance()),"world_frame":app.world_frame}));
    }
    fn set_view(&mut self, app: &mut App, view: usize) {
        self.view = view;
        if let Some(w) = &mut app.world {
            w.terrain_validation_view(false, view);
        }
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
    fn shutdown_proof(&self) -> Result<()> {
        let a = self.audits.first().context("old audit")?.snapshot();
        ensure!(a["instance"] == self.instance, "wrong old worker");
        let e = a["events"].as_array().unwrap();
        ensure!(
            e.iter().any(|v| v["id"] == self.job
                && v["event"] == "shutdown_begin"
                && v["running"] == true
                && v["rows"].as_u64().is_some_and(|n| n > 0 && n < 1024)),
            "server death missed running blend"
        );
        ensure!(
            e.iter()
                .any(|v| v["id"] == self.job && v["event"] == "cancel_shutdown")
                && e.iter().any(|v| v["id"] == self.job
                    && v["event"] == "finished"
                    && v["cancelled"] == true
                    && v["produced"] == false
                    && v["rows"].as_u64().is_some_and(|n| n > 0 && n < 1024)),
            "server death did not interrupt actual pixel work"
        );
        ensure!(
            e.last().is_some_and(|v| v["event"] == "joined"),
            "old worker alive"
        );
        Ok(())
    }
    pub fn message(&mut self, message: &str) {
        if let Some(e) = message
            .strip_prefix("[RappelzDebug] EVENT ")
            .and_then(|m| m.split_whitespace().next())
        {
            *self.events.entry(e.to_owned()).or_default() += 1;
        }
    }
    pub fn tick(&mut self, app: &mut App) -> Result<bool> {
        if let Some(controller)=self.read("server-process.json") {
            ensure!(controller["success"]==true,"isolated server controller failed");
        }
        ensure!(
            self.started.elapsed() < Duration::from_secs(180),
            "restart validation timeout at {:?}",
            self.stage
        );
        ensure!(
            app.addons.error.is_none() && app.addons.runtime.errors().is_empty(),
            "addon error"
        );
        if !matches!(self.stage, Stage::FindJob | Stage::Dropped) {
            ensure!(app.error.is_none(), "unexpected error: {:?}", app.error);
        }
        ensure!(
            self.trace(app)["errors"].as_array().unwrap().is_empty(),
            "decoder errors"
        );
        if let Some(w) = &app.world {
            if let Some(a) = w.terrain_validation_audit() {
                if !self.audits.iter().any(|v| v.instance() == a.instance()) {
                    ensure!(self.audits.len() < 2, "extra world");
                    self.audits.push(a);
                }
            }
            let c = w.terrain_cache_snapshot();
            ensure!(c["worker_error"].is_null(), "worker error");
            if !c.is_null() {
                self.samples.push(c);
                ensure!(self.samples.len() < 12000, "sample bound");
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
                    p.level == Some(1)
                        && p.health == Some(100)
                        && p.mana == Some(50)
                        && w.experience() == (0, 0)
                        && w.layer() == 0,
                    "fixture data"
                );
                self.own = p.handle;
                self.peer_handle = peer.handle;
                app.auto_enter = None;
                self.record(app, "Joined");
                self.advance(Stage::Initial);
            }
            Stage::Initial | Stage::Recovered => {
                if app.loading.is_some()
                    || !app
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
                            "InitialSettled"
                        } else {
                            "RecoveredSettled"
                        },
                    );
                    self.write(&format!("{}.{label}", self.role), json!({"ready":true}))?;
                    self.announced = true;
                }
                if self.announced
                    && self.ready(&format!("{}.{label}", self.other()))
                    && self.ready(&format!("release-{label}"))
                {
                    if self.stage == Stage::Initial {
                        self.advance(Stage::FindJob);
                    } else {
                        app.send(NetCmd::ReturnLobby);
                        self.advance(Stage::Logout);
                    }
                }
            }
            Stage::FindJob => {
                if let Some(kill) = self.read("server-killed.json") {
                    let chosen = &kill["jobs"][&self.role];
                    self.job = chosen["id"].as_u64().context("selected job")?;
                    self.instance = chosen["instance"].as_u64().context("selected worker")?;
                    self.old_own = self.own;
                    self.record(app, "ServerKilled");
                    self.advance(Stage::Dropped);
                } else if let Some(w) = &app.world {
                    let c = w.terrain_cache_snapshot();
                    let a = &c["active_job"];
                    let mut v = a.clone();
                    if let Some(v) = v.as_object_mut() {
                        v.insert("role".into(), json!(self.role));
                        v.insert(
                            "instance".into(),
                            json!(self.audits.first().context("audit")?.instance()),
                        );
                        v.insert(
                            "unix_ms".into(),
                            json!(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis()),
                        );
                    }
                    self.write(&format!("{}.candidate.json", self.role), v)?;
                    if c["ready"] == true {
                        self.set_view(app, (self.view + 1) % 4);
                    }
                }
            }
            Stage::Dropped => {
                if app.world.is_none() {
                    ensure!(
                        matches!(app.screen, Screen::Login)
                            && self.events.get("PLAYER_LOGOUT") == Some(&1),
                        "missing production logout"
                    );
                    let t = self.trace(app);
                    ensure!(
                        t["network_errors"].as_array().unwrap().len()
                            + t["disconnects"].as_array().unwrap().len()
                            == 1,
                        "one terminal transport event required"
                    );
                    self.shutdown_proof()?;
                    self.record(app, "WorkerJoined");
                    app.error = None;
                    self.write(
                        &format!("{}.dropped", self.role),
                        json!({"joined":true,"instance":self.instance,"job":self.job}),
                    )?;
                    self.advance(Stage::Reconnect);
                }
            }
            Stage::Reconnect => {
                ensure!(app.world.is_none(), "world revived during downtime");
                if self.ready("clients-reconnect") && self.ready("server-up-1.json") {
                    self.record(app, "DowntimeHeld");
                    let (host, port) =
                        crate::config::realmlist(&app.root, &app.config.get("locale", "frFR"));
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
                let Some(me) = w.addon_state().player else {
                    return Ok(false);
                };
                let Some(peer) = w.visible_player(&self.peer) else {
                    return Ok(false);
                };
                ensure!(
                    self.audits.len() == 2
                        && self.audits[0].instance() != self.audits[1].instance(),
                    "old world reused after reboot"
                );
                self.own = me.handle;
                self.peer_handle = peer.handle;
                app.auto_enter = None;
                if !self.announced {
                    self.record(app, "Reentered");
                    self.write(
                        &format!("{}.reentered", self.role),
                        json!({"handle":self.own,"instance":self.audits[1].instance()}),
                    )?;
                    self.announced = true;
                }
                if let Some(other) = self.read(&format!("{}.reentered", self.other())) {
                    ensure!(
                        other["handle"] == self.peer_handle,
                        "peer reboot identity mismatch"
                    );
                    self.record(app, "PairReunited");
                    self.set_view(app, 0);
                    self.advance(Stage::Recovered);
                }
            }
            Stage::Logout => {
                if app.world.is_none() && self.events.get("PLAYER_LOGOUT") == Some(&2) {
                    ensure!(
                        self.audits.len() == 2
                            && self.audits.iter().all(|a| a.snapshot()["events"]
                                .as_array()
                                .unwrap()
                                .last()
                                .is_some_and(|e| e["event"] == "joined")),
                        "worker alive after final logout"
                    );
                    self.advance(Stage::Done);
                    return Ok(true);
                }
            }
            Stage::Done => return Ok(true),
        }
        Ok(false)
    }
    pub fn fail(&mut self, error: anyhow::Error) {
        self.failure = Some(format!("{error:#}"));
    }
    pub fn finish(&self, app: &App) -> Result<bool> {
        let success = self.stage == Stage::Done && self.failure.is_none();
        self.write(&format!("{}.json",self.role),json!({"scenario":"terrain-restart","success":success,"failure":self.failure,"role":self.role,"character":self.character,"steps":self.steps,"samples":self.samples,"events":self.events,"audits":self.audits.iter().map(|a|a.snapshot()).collect::<Vec<_>>(),"trace":self.trace(app)}))?;
        Ok(success)
    }
}
