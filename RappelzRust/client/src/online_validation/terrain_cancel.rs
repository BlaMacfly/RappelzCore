//! Real worker cancellation, world/lobby teardown and TCP reconnect on a guarded loopback session.
use crate::{terrain::AuditHandle, App, NetCmd, Screen};
use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq)]
enum Stage {
    Join,
    Rotate,
    Initial,
    FindJob,
    Leaving,
    Reenter,
    Observe,
    Recovered,
    Logout,
    Done,
}

pub struct TerrainCancellationValidation {
    directory: PathBuf,
    role: String,
    peer: String,
    character: String,
    password: String,
    stage: Stage,
    started: Instant,
    since: Instant,
    view: usize,
    cycle: usize,
    own: u32,
    old_own: u32,
    closing_instance: u64,
    closing_job: u64,
    last_flip: u64,
    audits: Vec<AuditHandle>,
    steps: Vec<Value>,
    events: BTreeMap<String, usize>,
    failure: Option<String>,
    announced: bool,
    settled: Option<Instant>,
    rotations: BTreeSet<u64>,
    samples: Vec<Value>,
}

impl TerrainCancellationValidation {
    pub fn new(plan: PathBuf, app: &App) -> Result<Self> {
        super::guard(app)?;
        let p: Value = serde_json::from_slice(&std::fs::read(plan)?)?;
        let role = p["role"]
            .as_str()
            .context("terrain cancellation role")?
            .to_owned();
        ensure!(
            role == "owner" || role == "witness",
            "owner/witness role required"
        );
        let peer = p["peer"].as_str().context("terrain peer")?.to_owned();
        ensure!(peer.starts_with("Lua"), "dedicated peer required");
        let directory = PathBuf::from(p["directory"].as_str().context("report directory")?);
        std::fs::create_dir_all(&directory)?;
        ensure!(
            !directory.join(format!("{role}.json")).exists(),
            "fresh report directory required"
        );
        let login = crate::arg_value("-login")
            .context("explicit test login required for real reconnection")?;
        let (account, password) = login.split_once(':').context("test login format")?;
        ensure!(account == app.account, "test login mismatch");
        Ok(Self {
            directory,
            role,
            peer,
            character: app.auto_enter.clone().context("test character")?,
            password: password.to_owned(),
            stage: Stage::Join,
            started: Instant::now(),
            since: Instant::now(),
            view: 0,
            cycle: 0,
            own: 0,
            old_own: 0,
            closing_instance: 0,
            closing_job: 0,
            last_flip: 0,
            audits: Vec::new(),
            steps: Vec::new(),
            events: BTreeMap::new(),
            failure: None,
            announced: false,
            settled: None,
            rotations: BTreeSet::new(),
            samples: Vec::new(),
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
    fn actor(&self) -> &str {
        if self.cycle < 2 {
            "owner"
        } else {
            "witness"
        }
    }
    fn tcp(&self) -> bool {
        self.cycle % 2 == 1
    }
    fn ready(&self, name: &str) -> bool {
        self.directory.join(name).exists()
    }
    fn write(&self, name: &str, value: Value) -> Result<()> {
        std::fs::write(
            self.directory.join(name),
            serde_json::to_vec_pretty(&value)?,
        )?;
        Ok(())
    }
    fn read(&self, name: &str) -> Option<Value> {
        serde_json::from_slice(&std::fs::read(self.directory.join(name)).ok()?).ok()
    }
    fn advance(&mut self, stage: Stage) {
        self.stage = stage;
        self.since = Instant::now();
        self.announced = false;
        self.settled = None;
    }
    fn record(&mut self, app: &App, name: &str) {
        let w = app.world.as_ref();
        let state = w.map(|w| w.addon_state()).unwrap_or_default();
        self.steps.push(json!({"stage":name,"cycle":self.cycle,"elapsed_ms":self.started.elapsed().as_millis(),
            "own":self.own,"view":self.view,"closing_instance":self.closing_instance,"closing_job":self.closing_job,
            "player":state.player.as_ref().map(|p|json!({"name":p.name,"handle":p.handle,"level":p.level,"hp":p.health,"mp":p.mana})),
            "position":w.and_then(|w|w.unit_position(self.own)),"peer":w.and_then(|w|w.visible_player(&self.peer)).map(|p|p.handle),
            "xp_jp":w.map(|w|w.experience()),"money":state.money,"layer":w.map(|w|w.layer()),
            "cache":w.map(|w|w.terrain_cache_snapshot()),"events":self.events}));
    }
    fn set_view(&mut self, app: &mut App, view: usize) {
        self.view = view;
        if let Some(w) = &mut app.world {
            w.terrain_validation_view(false, view);
        }
    }
    fn audit(&self, instance: u64) -> Result<Value> {
        Ok(self
            .audits
            .iter()
            .find(|a| a.instance() == instance)
            .context("world audit")?
            .snapshot())
    }
    fn rotation_ids(&self) -> BTreeSet<u64> {
        let Some(audit) = self.audits.first() else {
            return BTreeSet::new();
        };
        let snapshot = audit.snapshot();
        let Some(events) = snapshot["events"].as_array() else {
            return BTreeSet::new();
        };
        events
            .iter()
            .filter(|c| {
                c["event"] == "cancel_view"
                    && c["running"] == true
                    && c["rows"].as_u64().is_some_and(|n| n > 0 && n < 1024)
            })
            .filter_map(|c| {
                let id = c["id"].as_u64()?;
                (events.iter().any(|f| {
                    f["id"] == id
                        && f["event"] == "finished"
                        && f["cancelled"] == true
                        && f["produced"] == false
                        && f["rows"].as_u64().is_some_and(|n| n > 0 && n < 1024)
                }) && events
                    .iter()
                    .any(|f| f["id"] == id && f["event"] == "polled_cancelled"))
                .then_some(id)
            })
            .collect()
    }
    pub fn message(&mut self, message: &str) {
        if let Some(event) = message
            .strip_prefix("[RappelzDebug] EVENT ")
            .and_then(|m| m.split_whitespace().next())
        {
            *self.events.entry(event.to_owned()).or_default() += 1;
        }
    }
    fn capture(&self, app: &mut App, label: &str) -> Result<bool> {
        let path = self
            .directory
            .join(format!("{}-{label}.ground.png", self.role));
        if !path.exists() {
            app.world
                .as_mut()
                .context("world for capture")?
                .terrain_validation_capture(path);
            return Ok(false);
        }
        macroquad::prelude::get_screen_data().export_png(
            self.directory
                .join(format!("{}-{label}.png", self.role))
                .to_str()
                .context("capture path")?,
        );
        Ok(true)
    }
    fn shutdown_proof(&self) -> Result<()> {
        let snapshot = self.audit(self.closing_instance)?;
        let events = snapshot["events"].as_array().context("audit events")?;
        let matching = |e: &&Value| e["id"] == self.closing_job;
        ensure!(
            events
                .iter()
                .filter(matching)
                .any(|e| e["event"] == "shutdown_begin"
                    && e["running"] == true
                    && e["rows"].as_u64().is_some_and(|n| n > 0 && n < 1024)),
            "world teardown missed the running blend"
        );
        ensure!(
            events
                .iter()
                .filter(matching)
                .any(|e| e["event"] == "cancel_shutdown"),
            "shutdown did not cancel its job"
        );
        ensure!(
            events
                .iter()
                .filter(matching)
                .any(|e| e["event"] == "finished"
                    && e["cancelled"] == true
                    && e["produced"] == false
                    && e["rows"].as_u64().is_some_and(|n| n > 0 && n < 1024)),
            "shutdown did not interrupt actual pixel blending"
        );
        ensure!(
            events.last().is_some_and(|e| e["event"] == "joined"),
            "worker not joined"
        );
        Ok(())
    }
    pub fn tick(&mut self, app: &mut App) -> Result<bool> {
        ensure!(
            self.started.elapsed() < Duration::from_secs(180),
            "terrain cancellation timed out at {:?}",
            self.stage
        );
        ensure!(app.error.is_none(), "client error: {:?}", app.error);
        ensure!(
            app.addons.error.is_none() && app.addons.runtime.errors().is_empty(),
            "addon error"
        );
        ensure!(
            app.combat_trace.as_ref().context("trace")?.json()["errors"]
                .as_array()
                .is_some_and(|e| e.is_empty()),
            "packet trace error"
        );
        if let Some(done) = self.read(&format!("{}.json", self.other())) {
            ensure!(done["success"] == true, "peer failed: {}", done["failure"]);
        }
        if let Some(w) = &app.world {
            if let Some(a) = w.terrain_validation_audit() {
                if !self.audits.iter().any(|old| old.instance() == a.instance()) {
                    ensure!(self.audits.len() < 3, "unexpected extra world");
                    self.audits.push(a);
                }
            }
            let cache = w.terrain_cache_snapshot();
            ensure!(cache["worker_error"].is_null(), "worker failure");
            if !cache.is_null() {
                self.samples.push(cache);
                ensure!(self.samples.len() < 12000, "bounded frame samples");
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
                let s = w.addon_state();
                let Some(me) = s.player else {
                    return Ok(false);
                };
                if w.visible_player(&self.peer).is_none() {
                    return Ok(false);
                }
                ensure!(
                    me.level == Some(1)
                        && me.health == Some(100)
                        && me.mana == Some(50)
                        && w.experience() == (0, 0)
                        && s.money == Some(0)
                        && w.layer() == 0,
                    "fresh ordinary fixture required"
                );
                self.own = me.handle;
                app.auto_enter = None;
                self.record(app, "InitialJoin");
                self.write(&format!("{}.joined", self.role), json!({"handle":self.own}))?;
                self.advance(Stage::Rotate);
            }
            Stage::Rotate => {
                if !self.ready(&format!("{}.joined", self.other())) {
                    return Ok(false);
                }
                let ids = self.rotation_ids();
                if ids.len() > self.rotations.len() {
                    self.rotations = ids;
                    self.record(app, "RotationCancelled");
                }
                if self.rotations.len() >= 6 {
                    if !self.announced {
                        self.write(
                            &format!("{}.rotated", self.role),
                            json!({"ids":self.rotations}),
                        )?;
                        self.announced = true;
                    }
                    if self.ready(&format!("{}.rotated", self.other())) {
                        self.set_view(app, 0);
                        self.advance(Stage::Initial);
                    }
                } else if let Some(w) = &app.world {
                    let a = w.terrain_cache_snapshot()["active_job"].clone();
                    if a["running"] == true
                        && a["cancelled"] == false
                        && a["rows"].as_u64().is_some_and(|n| n > 0 && n < 384)
                        && a["id"].as_u64().is_some_and(|id| id != self.last_flip)
                    {
                        self.last_flip = a["id"].as_u64().unwrap();
                        self.record(app, "RotateRequested");
                        self.set_view(app, 1 - self.view);
                    } else if w.terrain_cache_snapshot()["ready"] == true {
                        self.set_view(app, 1 - self.view);
                    }
                }
            }
            Stage::Initial | Stage::Recovered => {
                let Some(w) = &app.world else {
                    return Ok(false);
                };
                if app.loading.is_some() || w.terrain_cache_snapshot()["ready"] != true {
                    return Ok(false);
                }
                let since = *self.settled.get_or_insert_with(Instant::now);
                if since.elapsed() < Duration::from_millis(300) {
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
                if self.role != self.actor() {
                    if let Some(request) = self.read(&format!("request-{}", self.cycle)) {
                        let handle = request["handle"].as_u64().context("leaving handle")? as u32;
                        if app.world.as_ref().is_some_and(|w| w.unit(handle).is_none())
                            && app.combat_trace.as_ref().unwrap().json()["leaves"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .any(|h| *h == handle)
                        {
                            self.record(app, "ObservedLeave");
                            self.write(
                                &format!("{}.left-{}", self.role, self.cycle),
                                json!({"handle":handle}),
                            )?;
                            self.advance(Stage::Observe);
                        }
                    }
                } else {
                    let Some(w) = &app.world else {
                        return Ok(false);
                    };
                    if app.loading.is_some() {
                        return Ok(false);
                    }
                    let cache = w.terrain_cache_snapshot();
                    let a = &cache["active_job"];
                    if a["running"] == true
                        && a["cancelled"] == false
                        && a["rows"].as_u64().is_some_and(|n| n > 0 && n < 256)
                    {
                        self.old_own = self.own;
                        self.closing_instance =
                            self.audits.last().context("active audit")?.instance();
                        self.closing_job = a["id"].as_u64().context("closing job")?;
                        self.record(
                            app,
                            if self.tcp() {
                                "TcpRequested"
                            } else {
                                "LobbyRequested"
                            },
                        );
                        self.write(
                            &format!("request-{}", self.cycle),
                            json!({"handle":self.own,"instance":self.closing_instance,
                            "job":self.closing_job,"kind":if self.tcp(){"tcp"}else{"lobby"}}),
                        )?;
                        app.send(if self.tcp() {
                            NetCmd::Disconnect
                        } else {
                            NetCmd::ReturnLobby
                        });
                        self.advance(Stage::Leaving);
                    } else if cache["ready"] == true {
                        self.set_view(app, (self.view + 1) % 4);
                    }
                }
            }
            Stage::Leaving => {
                if app.world.is_none() {
                    self.shutdown_proof()?;
                    self.record(app, "WorkerJoined");
                    self.advance(Stage::Reenter);
                }
            }
            Stage::Reenter => {
                if !self.announced
                    && self.ready(&format!("{}.left-{}", self.other(), self.cycle))
                    && self.since.elapsed() > Duration::from_millis(500)
                {
                    if self.tcp() {
                        ensure!(
                            matches!(app.screen, Screen::Login),
                            "TCP transport did not close"
                        );
                        let (host, port) =
                            crate::config::realmlist(&app.root, &app.config.get("locale", "frFR"));
                        app.auto_enter = Some(self.character.clone());
                        app.send(NetCmd::Login {
                            host,
                            port,
                            account: app.account.clone(),
                            password: self.password.clone(),
                        });
                    } else {
                        app.send(NetCmd::EnterWorld(self.character.clone()));
                    }
                    self.announced = true;
                }
                if self.announced && app.loading.is_none() {
                    if let Some(w) = &app.world {
                        let Some(me) = w.addon_state().player else {
                            return Ok(false);
                        };
                        if me.handle == self.old_own {
                            return Ok(false);
                        }
                        ensure!(
                            self.audits
                                .last()
                                .is_some_and(|a| a.instance() != self.closing_instance),
                            "old worker reused by new world"
                        );
                        self.own = me.handle;
                        self.view = 0;
                        app.auto_enter = None;
                        self.record(app, "Reentered");
                        self.write(
                            &format!("reentered-{}", self.cycle),
                            json!({"old":self.old_own,"new":self.own}),
                        )?;
                        self.advance(Stage::Observe);
                    }
                }
            }
            Stage::Observe => {
                if self.role == self.actor() {
                    if self.ready(&format!("{}.seen-{}", self.other(), self.cycle)) {
                        self.cycle += 1;
                        if self.cycle == 4 {
                            self.set_view(app, 0);
                            self.advance(Stage::Recovered);
                        } else {
                            self.advance(Stage::FindJob);
                        }
                    }
                } else if let Some(reentered) = self.read(&format!("reentered-{}", self.cycle)) {
                    let old = reentered["old"].as_u64().context("old handle")? as u32;
                    let new = reentered["new"].as_u64().context("new handle")? as u32;
                    if app.world.as_ref().is_some_and(|w| {
                        w.unit(old).is_none()
                            && w.visible_player(&self.peer)
                                .is_some_and(|p| p.handle == new)
                    }) {
                        self.record(app, "ObservedReentry");
                        self.write(
                            &format!("{}.seen-{}", self.role, self.cycle),
                            json!({"old":old,"new":new}),
                        )?;
                        self.cycle += 1;
                        if self.cycle == 4 {
                            self.set_view(app, 0);
                            self.advance(Stage::Recovered);
                        } else {
                            self.advance(Stage::FindJob);
                        }
                    }
                }
            }
            Stage::Logout => {
                if app.world.is_none() && self.events.get("PLAYER_LOGOUT") == Some(&3) {
                    ensure!(
                        self.audits.len() == 3
                            && self.audits.iter().all(|a| a.snapshot()["events"]
                                .as_array()
                                .is_some_and(|e| e.last().is_some_and(|v| v["event"] == "joined"))),
                        "world workers still alive"
                    );
                    self.record(app, "FinalLogout");
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
        self.write(&format!("{}.json",self.role),json!({"scenario":"terrain-cancellation","success":success,"failure":self.failure,
            "role":self.role,"character":self.character,"events":self.events,"steps":self.steps,"rotations":self.rotations,
            "samples":self.samples,"audits":self.audits.iter().map(|a|a.snapshot()).collect::<Vec<_>>(),
            "trace":app.combat_trace.as_ref().context("trace")?.json()}))?;
        Ok(success)
    }
}
