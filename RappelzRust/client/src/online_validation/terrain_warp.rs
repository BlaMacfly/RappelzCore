//! Guarded native two-client WARP validation. All movement uses the server's GM chat command.
use crate::{terrain::AuditHandle, App, NetCmd};
use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Debug, PartialEq)]
enum Stage {
    Join,
    Settle,
    Ready,
    FindJob,
    Warping,
    Follow,
    Logout,
    Done,
}

pub struct TerrainWarpValidation {
    directory: PathBuf,
    role: String,
    peer: String,
    character: String,
    cancel: bool,
    stage: Stage,
    started: Instant,
    since: Instant,
    leg: usize,
    own: u32,
    peer_handle: u32,
    view: usize,
    job: u64,
    enters: usize,
    leaves: usize,
    warps: usize,
    announced: bool,
    audit: Option<AuditHandle>,
    steps: Vec<Value>,
    samples: Vec<Value>,
    events: BTreeMap<String, usize>,
    failure: Option<String>,
}
impl TerrainWarpValidation {
    pub fn new(plan: PathBuf, app: &App) -> Result<Self> {
        super::guard(app)?;
        let p: Value = serde_json::from_slice(&std::fs::read(plan)?)?;
        let role = p["role"].as_str().context("role")?.to_owned();
        ensure!(
            role == "owner" || role == "witness",
            "owner/witness required"
        );
        let peer = p["peer"].as_str().context("peer")?.to_owned();
        ensure!(peer.starts_with("Lua"), "dedicated peer required");
        let directory = PathBuf::from(p["directory"].as_str().context("directory")?);
        std::fs::create_dir_all(&directory)?;
        ensure!(
            !directory.join(format!("{role}.json")).exists(),
            "fresh report required"
        );
        Ok(Self {
            directory,
            role,
            peer,
            character: app.auto_enter.clone().context("character")?,
            cancel: p["cancel_on_warp"].as_bool().unwrap_or(true),
            stage: Stage::Join,
            started: Instant::now(),
            since: Instant::now(),
            leg: 0,
            own: 0,
            peer_handle: 0,
            view: 0,
            job: 0,
            enters: 0,
            leaves: 0,
            warps: 0,
            announced: false,
            audit: None,
            steps: Vec::new(),
            samples: Vec::new(),
            events: BTreeMap::new(),
            failure: None,
        })
    }
    pub(super) fn before_draw(&self, app: &mut App) {
        if let Some(w) = &mut app.world {
            w.terrain_validation_policy(false, true, true);
            w.terrain_validation_warp_cancel(self.cancel);
        }
    }
    fn other(&self) -> &str {
        if self.role == "owner" {
            "witness"
        } else {
            "owner"
        }
    }
    fn target(&self, role: &str) -> [f32; 2] {
        let [x, y] = [
            [8500., 7227.],
            [12000., 7227.],
            [72952., 23355.],
            [89080., 23355.],
            [8440., 7227.],
        ][self.leg.min(4)];
        [x, y + if role == "witness" { 20. } else { 0. }]
    }
    fn display_peer(&self) -> String {
        format!("<GM>{}", self.peer)
    }
    fn ready(&self, name: &str) -> bool {
        self.directory.join(name).exists()
    }
    fn mark(&self, name: &str) -> Result<()> {
        std::fs::write(self.directory.join(name), b"ready")?;
        Ok(())
    }
    fn advance(&mut self, s: Stage) {
        self.stage = s;
        self.since = Instant::now();
        self.announced = false;
    }
    fn trace(&self, app: &App) -> Value {
        app.combat_trace.as_ref().unwrap().json()
    }
    fn peer_counts(&self, app: &App) -> (usize, usize) {
        let t = self.trace(app);
        (
            t["enters"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["handle"] == self.peer_handle)
                .count(),
            t["leaves"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| **e == self.peer_handle)
                .count(),
        )
    }
    fn record(&mut self, app: &App, label: &str) {
        let w = app.world.as_ref();
        let state = w.map(|w| w.addon_state()).unwrap_or_default();
        self.steps.push(json!({"stage":label,"leg":self.leg,"elapsed_ms":self.started.elapsed().as_millis(),
            "own":self.own,"peer_handle":self.peer_handle,"job":self.job,"view":self.view,"destination":self.target(&self.role),
            "player":state.player.as_ref().map(|p|json!({"name":p.name,"handle":p.handle,"level":p.level,"hp":p.health,"mp":p.mana})),
            "position":w.and_then(|w|w.unit_position(self.own)),"peer_position":w.and_then(|w|w.unit_position(self.peer_handle)),
            "xp_jp":w.map(|w|w.experience()),"money":state.money,"layer":w.map(|w|w.layer()),
            "cache":w.map(|w|w.terrain_cache_snapshot()),"peer_counts":self.peer_counts(app),"warps":self.trace(app)["warps"]}));
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
    pub fn message(&mut self, message: &str) {
        if let Some(e) = message
            .strip_prefix("[RappelzDebug] EVENT ")
            .and_then(|s| s.split_whitespace().next())
        {
            *self.events.entry(e.to_owned()).or_default() += 1;
        }
    }
    pub fn tick(&mut self, app: &mut App) -> Result<bool> {
        ensure!(
            self.started.elapsed() < Duration::from_secs(240),
            "warp timeout at {:?}, leg {}",
            self.stage,
            self.leg
        );
        ensure!(
            app.error.is_none()
                && app.addons.error.is_none()
                && app.addons.runtime.errors().is_empty(),
            "client/addon error"
        );
        ensure!(
            self.trace(app)["errors"]
                .as_array()
                .is_some_and(|v| v.is_empty()),
            "trace errors"
        );
        if let Some(w) = &app.world {
            if let Some(a) = w.terrain_validation_audit() {
                if let Some(old) = &self.audit {
                    ensure!(old.instance() == a.instance(), "world replaced during warp");
                } else {
                    self.audit = Some(a);
                }
            }
            let cache = w.terrain_cache_snapshot();
            ensure!(cache["worker_error"].is_null(), "worker error");
            if !cache.is_null() {
                self.samples.push(cache);
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
                let Some(peer) = w.visible_player(&self.display_peer()) else {
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
                self.advance(Stage::Settle);
            }
            Stage::Settle => {
                let Some(w) = &app.world else {
                    return Ok(false);
                };
                if w.terrain_cache_snapshot()["ready"] != true
                    || self.since.elapsed() < Duration::from_millis(300)
                {
                    return Ok(false);
                }
                let label = if self.leg == 0 && self.warps == 0 {
                    "initial".into()
                } else {
                    format!("leg-{}", self.leg)
                };
                if !self.announced && self.capture(app, &label)? {
                    self.record(app, "Settled");
                    self.mark(&format!("{}.{label}", self.role))?;
                    self.announced = true;
                }
                if self.announced
                    && self.ready(&format!("{}.{label}", self.other()))
                    && self.ready(&format!("release-{label}"))
                {
                    if self.warps > 0 {
                        self.leg += 1;
                    }
                    if self.leg == 5 {
                        app.send(NetCmd::ReturnLobby);
                        self.advance(Stage::Logout);
                    } else {
                        (self.enters, self.leaves) = self.peer_counts(app);
                        self.mark(&format!("{}.ready-{}", self.role, self.leg))?;
                        self.advance(Stage::Ready);
                    }
                }
            }
            Stage::Ready => {
                if !self.ready(&format!("{}.ready-{}", self.other(), self.leg)) {
                    return Ok(false);
                }
                if self.role == "owner" {
                    self.advance(Stage::FindJob);
                } else {
                    let (enters, leaves) = self.peer_counts(app);
                    let w = app.world.as_ref().context("world")?;
                    let observed = if self.leg == 0 {
                        enters > self.enters
                            && w.unit_position(self.peer_handle)
                                == Some(self.target("owner").into())
                    } else {
                        leaves > self.leaves && w.unit(self.peer_handle).is_none()
                    };
                    if observed {
                        self.record(app, "ObservedOwner");
                        self.mark(&format!("witness.saw-owner-{}", self.leg))?;
                        self.advance(Stage::FindJob);
                    }
                }
            }
            Stage::FindJob => {
                let w = app.world.as_ref().context("world")?;
                let c = w.terrain_cache_snapshot();
                let a = &c["active_job"];
                if a["running"] == true
                    && a["cancelled"] == false
                    && a["rows"].as_u64().is_some_and(|n| n > 0 && n < 192)
                {
                    self.job = a["id"].as_u64().context("job")?;
                    self.record(app, "WarpRequested");
                    let [x, y] = self.target(&self.role);
                    app.send(NetCmd::Chat {
                        chat_type: 0,
                        target: String::new(),
                        message: format!(".go {x} {y} 0"),
                    });
                    self.advance(Stage::Warping);
                } else if c["ready"] == true {
                    self.set_view(app, (self.view + 1) % 4);
                }
            }
            Stage::Warping => {
                let t = self.trace(app);
                let w = app.world.as_ref().context("world")?;
                if t["warps"].as_array().unwrap().len() != self.warps + 1
                    || w.unit_position(self.own) != Some(self.target(&self.role).into())
                {
                    return Ok(false);
                }
                let a = self.audit.as_ref().context("audit")?.snapshot();
                if !a["events"].as_array().unwrap().iter().any(|e| {
                    e["id"] == self.job
                        && (e["event"] == "polled_cancelled" || e["event"] == "polled_value")
                }) {
                    return Ok(false);
                }
                self.warps += 1;
                self.record(app, "WarpApplied");
                self.mark(&format!("{}.warped-{}", self.role, self.leg))?;
                self.advance(Stage::Follow);
            }
            Stage::Follow => {
                let w = app.world.as_ref().context("world")?;
                let (enters, _) = self.peer_counts(app);
                if self.ready(&format!("{}.warped-{}", self.other(), self.leg))
                    && enters > self.enters
                    && w.unit_position(self.peer_handle) == Some(self.target(self.other()).into())
                {
                    self.record(app, "PairReunited");
                    self.set_view(app, 0);
                    self.advance(Stage::Settle);
                }
            }
            Stage::Logout => {
                if app.world.is_none() && self.events.get("PLAYER_LOGOUT") == Some(&1) {
                    ensure!(
                        self.audit.as_ref().context("audit")?.snapshot()["events"]
                            .as_array()
                            .unwrap()
                            .last()
                            .is_some_and(|e| e["event"] == "joined"),
                        "worker still alive"
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
        std::fs::write(
            self.directory.join(format!("{}.json", self.role)),
            serde_json::to_vec_pretty(&json!({
            "scenario":"terrain-warp","success":success,"failure":self.failure,"role":self.role,"character":self.character,
            "cancel_on_warp":self.cancel,"events":self.events,"steps":self.steps,"samples":self.samples,
            "audits":self.audit.iter().map(|a|a.snapshot()).collect::<Vec<_>>(),"trace":self.trace(app)}))?,
        )?;
        Ok(success)
    }
}
