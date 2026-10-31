//! Real encrypted TCP stalls/resets through an external opaque loopback relay.
use crate::{terrain::AuditHandle, App, NetCmd, Screen};
use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Debug, PartialEq)]
enum Stage {
    Join,
    Settle,
    Start,
    StallAck,
    Hold,
    Resume,
    FindReset,
    Reset,
    Reconnect,
    Observe,
    Logout,
    Done,
}
pub struct TerrainNetworkValidation {
    directory: PathBuf,
    role: String,
    peer: String,
    character: String,
    password: String,
    stage: Stage,
    started: Instant,
    since: Instant,
    cycle: usize,
    view: usize,
    own: u32,
    peer_handle: u32,
    old_own: u32,
    job: u64,
    closing_instance: u64,
    announced: bool,
    audits: Vec<AuditHandle>,
    rotations: BTreeSet<u64>,
    steps: Vec<Value>,
    samples: Vec<Value>,
    events: BTreeMap<String, usize>,
    failure: Option<String>,
}
impl TerrainNetworkValidation {
    pub fn new(plan: PathBuf, app: &App) -> Result<Self> {
        super::guard(app)?;
        ensure!(
            crate::arg_value("-validate-world-proxy").is_some(),
            "explicit loopback relay required"
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
            cycle: 0,
            view: 0,
            own: 0,
            peer_handle: 0,
            old_own: 0,
            job: 0,
            closing_instance: 0,
            announced: false,
            audits: Vec::new(),
            rotations: BTreeSet::new(),
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
    fn actor(&self) -> &str {
        if self.cycle == 0 {
            "owner"
        } else {
            "witness"
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
    fn write(&self, name: &str, value: Value) -> Result<()> {
        std::fs::write(
            self.directory.join(name),
            serde_json::to_vec_pretty(&value)?,
        )?;
        Ok(())
    }
    fn mark(&self, name: &str) -> Result<()> {
        self.write(name, json!({"ready":true}))
    }
    fn read(&self, name: &str) -> Option<Value> {
        serde_json::from_slice(&std::fs::read(self.directory.join(name)).ok()?).ok()
    }
    fn ack(&self, action: &str) -> bool {
        self.ready(&format!("{}-{action}-{}.ack", self.actor(), self.cycle))
    }
    fn command(&self, action: &str) -> Result<()> {
        self.mark(&format!("{}-{action}-{}.request", self.role, self.cycle))
    }
    fn advance(&mut self, stage: Stage) {
        self.stage = stage;
        self.since = Instant::now();
        self.announced = false;
    }
    fn trace(&self, app: &App) -> Value {
        app.combat_trace.as_ref().unwrap().json()
    }
    fn probe(&self, kind: &str) -> String {
        format!("TerrainNet_{}_{kind}", self.cycle)
    }
    fn received(&self, app: &App, message: &str) -> usize {
        self.trace(app)["chats"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["message"] == message && c["sender"] == self.peer)
            .count()
    }
    fn whisper(&self, app: &App, kind: &str) {
        app.send(NetCmd::Chat {
            chat_type: rz_proto::msg::chat_type::WHISPER,
            target: self.peer.clone(),
            message: self.probe(kind),
        });
    }
    fn set_view(&mut self, app: &mut App, view: usize) {
        self.view = view;
        if let Some(w) = &mut app.world {
            w.terrain_validation_view(false, view);
        }
    }
    fn record(&mut self, app: &App, label: &str) {
        let w = app.world.as_ref();
        let state = w.map(|w| w.addon_state()).unwrap_or_default();
        self.steps.push(json!({"stage":label,"cycle":self.cycle,"elapsed_ms":self.started.elapsed().as_millis(),"stage_ms":self.since.elapsed().as_millis(),
            "own":self.own,"old_own":self.old_own,"peer_handle":self.peer_handle,"job":self.job,"closing_instance":self.closing_instance,
            "player":state.player.as_ref().map(|p|json!({"handle":p.handle,"name":p.name,"level":p.level,"hp":p.health,"mp":p.mana})),
            "position":w.and_then(|w|w.unit_position(self.own)),"peer":w.and_then(|w|w.visible_player(&self.peer)).map(|p|p.handle),
            "xp_jp":w.map(|w|w.experience()),"money":state.money,"layer":w.map(|w|w.layer()),
            "cache":w.map(|w|w.terrain_cache_snapshot()),"events":self.events,"chats":self.trace(app)["chats"],
            "error":app.error,"world_frame":app.world_frame,"loading":app.loading.is_some(),
            "audit_instance":self.audits.last().map(|a|a.instance()),
            "audit_cursor":self.audits.last().map(|a|a.snapshot()["events"].as_array().map_or(0,|e|e.len()))}));
    }
    fn partial_rotations(&self) -> BTreeSet<u64> {
        let Some(a) = self.audits.last() else {
            return BTreeSet::new();
        };
        let audit = a.snapshot();
        let e = audit["events"].as_array().unwrap();
        e.iter()
            .filter(|v| {
                v["event"] == "cancel_view"
                    && v["running"] == true
                    && v["rows"].as_u64().is_some_and(|n| n > 0 && n < 1024)
            })
            .filter_map(|c| {
                let id = c["id"].as_u64()?;
                (e.iter().any(|f| {
                    f["id"] == id
                        && f["event"] == "finished"
                        && f["cancelled"] == true
                        && f["produced"] == false
                        && f["rows"].as_u64().is_some_and(|n| n > 0 && n < 1024)
                }) && e
                    .iter()
                    .any(|f| f["id"] == id && f["event"] == "polled_cancelled"))
                .then_some(id)
            })
            .collect()
    }
    fn shutdown_proof(&self) -> Result<()> {
        let a = self
            .audits
            .iter()
            .find(|a| a.instance() == self.closing_instance)
            .context("closing audit")?
            .snapshot();
        let e = a["events"].as_array().unwrap();
        ensure!(
            e.iter().any(|v| v["id"] == self.job
                && v["event"] == "shutdown_begin"
                && v["running"] == true
                && v["rows"].as_u64().is_some_and(|n| n > 0 && n < 1024)),
            "remote reset missed running blend"
        );
        ensure!(
            e.iter()
                .any(|v| v["id"] == self.job && v["event"] == "cancel_shutdown")
                && e.iter().any(|v| v["id"] == self.job
                    && v["event"] == "finished"
                    && v["cancelled"] == true
                    && v["produced"] == false
                    && v["rows"].as_u64().is_some_and(|n| n > 0 && n < 1024)),
            "reset did not interrupt blending"
        );
        ensure!(
            e.last().is_some_and(|v| v["event"] == "joined"),
            "worker not joined"
        );
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
    pub fn message(&mut self, message: &str) {
        if let Some(e) = message
            .strip_prefix("[RappelzDebug] EVENT ")
            .and_then(|m| m.split_whitespace().next())
        {
            *self.events.entry(e.to_owned()).or_default() += 1;
        }
    }
    pub fn tick(&mut self, app: &mut App) -> Result<bool> {
        ensure!(
            self.started.elapsed() < Duration::from_secs(220),
            "network terrain timeout at {:?}, cycle {}",
            self.stage,
            self.cycle
        );
        ensure!(
            app.addons.error.is_none() && app.addons.runtime.errors().is_empty(),
            "addon error"
        );
        if self.stage != Stage::Reset {
            ensure!(
                app.error.is_none(),
                "unexpected client error: {:?}",
                app.error
            );
        }
        ensure!(
            self.trace(app)["errors"].as_array().unwrap().is_empty(),
            "packet decoder error"
        );
        if let Some(w) = &app.world {
            if let Some(a) = w.terrain_validation_audit() {
                if !self.audits.iter().any(|v| v.instance() == a.instance()) {
                    ensure!(self.audits.len() < 2, "extra world");
                    self.audits.push(a);
                }
            }
            let cache = w.terrain_cache_snapshot();
            ensure!(cache["worker_error"].is_null(), "worker error");
            if !cache.is_null() {
                self.samples.push(cache);
                ensure!(self.samples.len() < 16000, "bounded samples");
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
                    "fresh fixture data"
                );
                self.own = p.handle;
                self.peer_handle = peer.handle;
                app.auto_enter = None;
                self.record(app, "Joined");
                self.advance(Stage::Settle);
            }
            Stage::Settle => {
                if app.loading.is_some()
                    || !app
                        .world
                        .as_ref()
                        .is_some_and(|w| w.terrain_cache_snapshot()["ready"] == true)
                    || self.since.elapsed() < Duration::from_millis(300)
                {
                    return Ok(false);
                }
                let label = if self.cycle == 0 {
                    "initial"
                } else {
                    "recovered"
                };
                if !self.announced && self.capture(app, label)? {
                    self.record(
                        app,
                        if self.cycle == 0 {
                            "InitialSettled"
                        } else {
                            "RecoveredSettled"
                        },
                    );
                    self.mark(&format!("{}.{label}", self.role))?;
                    self.announced = true;
                }
                if self.announced
                    && self.ready(&format!("{}.{label}", self.other()))
                    && self.ready(&format!("release-{label}"))
                {
                    if self.cycle == 2 {
                        app.send(NetCmd::ReturnLobby);
                        self.advance(Stage::Logout);
                    } else {
                        self.advance(Stage::Start);
                    }
                }
            }
            Stage::Start => {
                if self.cycle == 1 && !self.ready(&format!("{}.cycle-done-0", self.other())) {
                    return Ok(false);
                }
                if self.role != self.actor() {
                    if self.ack("pause") {
                        self.whisper(app, "in");
                        self.record(app, "StallBegin");
                        self.mark(&format!("{}.probed-{}", self.role, self.cycle))?;
                        self.advance(Stage::Hold);
                    }
                } else {
                    let c = app
                        .world
                        .as_ref()
                        .context("world")?
                        .terrain_cache_snapshot();
                    let a = &c["active_job"];
                    if a["running"] == true
                        && a["cancelled"] == false
                        && a["rows"].as_u64().is_some_and(|n| n > 0 && n < 256)
                    {
                        self.job = a["id"].as_u64().context("job")?;
                        self.record(app, "PauseRequested");
                        self.command("pause")?;
                        self.advance(Stage::StallAck);
                    } else if c["ready"] == true {
                        self.set_view(app, (self.view + 1) % 4);
                    }
                }
            }
            Stage::StallAck => {
                if self.ack("pause") {
                    self.whisper(app, "out");
                    self.record(app, "StallBegin");
                    self.mark(&format!("{}.probed-{}", self.role, self.cycle))?;
                    self.advance(Stage::Hold);
                }
            }
            Stage::Hold => {
                if self.ack("resume") {
                    self.advance(Stage::Resume);
                    return Ok(false);
                }
                let expected = self.probe(if self.role == self.actor() {
                    "in"
                } else {
                    "out"
                });
                ensure!(
                    self.received(app, &expected) == 0,
                    "probe crossed paused transport"
                );
                ensure!(
                    app.world.as_ref().is_some_and(|w| w
                        .visible_player(&self.peer)
                        .is_some_and(|p| p.handle == self.peer_handle)),
                    "peer disappeared during stall"
                );
                if self.role == self.actor() {
                    self.rotations = self.partial_rotations();
                    if self.rotations.len() < 3 {
                        let c = app.world.as_ref().unwrap().terrain_cache_snapshot();
                        let a = &c["active_job"];
                        if a["running"] == true
                            && a["cancelled"] == false
                            && a["rows"].as_u64().is_some_and(|n| n > 0 && n < 384)
                        {
                            self.set_view(app, 1 - self.view.min(1));
                        } else if c["ready"] == true {
                            self.set_view(app, 1 - self.view.min(1));
                        }
                    }
                }
                if self.since.elapsed() >= Duration::from_secs(8)
                    && self.ready(&format!("{}.probed-{}", self.other(), self.cycle))
                    && (self.role != self.actor() || self.rotations.len() >= 3)
                {
                    if !self.announced {
                        self.record(app, "StallHeld");
                        self.mark(&format!("{}.held-{}", self.role, self.cycle))?;
                        self.announced = true;
                    }
                    if self.role == self.actor()
                        && self.ready(&format!("{}.held-{}", self.other(), self.cycle))
                    {
                        self.command("resume")?;
                        self.advance(Stage::Resume);
                    }
                }
            }
            Stage::Resume => {
                let expected = self.probe(if self.role == self.actor() {
                    "in"
                } else {
                    "out"
                });
                if self.ack("resume") && self.received(app, &expected) == 1 {
                    if !self.announced {
                        self.record(app, "Resumed");
                        self.mark(&format!("{}.resumed-{}", self.role, self.cycle))?;
                        self.announced = true;
                    }
                    if self.ready(&format!("{}.resumed-{}", self.other(), self.cycle)) {
                        self.advance(if self.role == self.actor() {
                            Stage::FindReset
                        } else {
                            Stage::Observe
                        });
                    }
                }
            }
            Stage::FindReset => {
                let c = app
                    .world
                    .as_ref()
                    .context("world")?
                    .terrain_cache_snapshot();
                let a = &c["active_job"];
                if a["running"] == true
                    && a["cancelled"] == false
                    && a["rows"].as_u64().is_some_and(|n| n > 0 && n < 256)
                {
                    self.old_own = self.own;
                    self.job = a["id"].as_u64().context("job")?;
                    self.closing_instance = self.audits.last().context("audit")?.instance();
                    self.record(app, "ResetRequested");
                    self.command("reset")?;
                    self.advance(Stage::Reset);
                } else if c["ready"] == true {
                    self.set_view(app, (self.view + 1) % 4);
                }
            }
            Stage::Reset => {
                if app.world.is_none() {
                    ensure!(
                        self.ack("reset") && matches!(app.screen, Screen::Login),
                        "unexpected world loss"
                    );
                    self.shutdown_proof()?;
                    self.record(app, "WorkerJoined");
                    app.error = None;
                    self.advance(Stage::Reconnect);
                }
            }
            Stage::Reconnect => {
                if !self.announced
                    && self.ready(&format!("{}.left-{}", self.other(), self.cycle))
                    && self.since.elapsed() > Duration::from_millis(500)
                {
                    let (host, port) =
                        crate::config::realmlist(&app.root, &app.config.get("locale", "frFR"));
                    app.auto_enter = Some(self.character.clone());
                    app.send(NetCmd::Login {
                        host,
                        port,
                        account: app.account.clone(),
                        password: self.password.clone(),
                    });
                    self.announced = true;
                }
                if self.announced && app.loading.is_none() {
                    if let Some(w) = &app.world {
                        if let Some(p) = w.addon_state().player {
                            if p.handle != self.old_own {
                                self.own = p.handle;
                                app.auto_enter = None;
                                self.view = 0;
                                self.record(app, "Reentered");
                                self.write(
                                    &format!("reentered-{}", self.cycle),
                                    json!({"old":self.old_own,"new":self.own}),
                                )?;
                                self.advance(Stage::Observe);
                            }
                        }
                    }
                }
            }
            Stage::Observe => {
                if self.role == self.actor() {
                    if self.ready(&format!("{}.seen-{}", self.other(), self.cycle)) {
                        self.finish_cycle(app)?;
                    }
                } else {
                    if !self.announced
                        && app
                            .world
                            .as_ref()
                            .is_some_and(|w| w.unit(self.peer_handle).is_none())
                        && self.trace(app)["leaves"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|h| *h == self.peer_handle)
                    {
                        self.record(app, "ObservedLeave");
                        self.mark(&format!("{}.left-{}", self.role, self.cycle))?;
                        self.announced = true;
                    }
                    if let Some(r) = self.read(&format!("reentered-{}", self.cycle)) {
                        let new = r["new"].as_u64().context("new handle")? as u32;
                        if self.announced
                            && app.world.as_ref().is_some_and(|w| {
                                w.unit(self.peer_handle).is_none()
                                    && w.visible_player(&self.peer)
                                        .is_some_and(|p| p.handle == new)
                            })
                        {
                            self.peer_handle = new;
                            self.record(app, "ObservedReentry");
                            self.mark(&format!("{}.seen-{}", self.role, self.cycle))?;
                            self.finish_cycle(app)?;
                        }
                    }
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
                                .is_some_and(|v| v["event"] == "joined")),
                        "unjoined worlds"
                    );
                    self.advance(Stage::Done);
                    return Ok(true);
                }
            }
            Stage::Done => return Ok(true),
        }
        Ok(false)
    }
    fn finish_cycle(&mut self, app: &mut App) -> Result<()> {
        self.mark(&format!("{}.cycle-done-{}", self.role, self.cycle))?;
        self.cycle += 1;
        self.set_view(app, 0);
        self.advance(if self.cycle == 2 {
            Stage::Settle
        } else {
            Stage::Start
        });
        Ok(())
    }
    pub fn fail(&mut self, e: anyhow::Error) {
        self.failure = Some(format!("{e:#}"));
    }
    pub fn finish(&self, app: &App) -> Result<bool> {
        let success = self.stage == Stage::Done && self.failure.is_none();
        self.write(&format!("{}.json",self.role),json!({"scenario":"terrain-network","success":success,"failure":self.failure,"role":self.role,"character":self.character,
            "steps":self.steps,"samples":self.samples,"rotations":self.rotations,"events":self.events,
            "audits":self.audits.iter().map(|a|a.snapshot()).collect::<Vec<_>>(),"trace":self.trace(app)}))?;
        Ok(success)
    }
}
