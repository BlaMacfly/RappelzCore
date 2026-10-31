//! Two native clients: enter visibility mid-route, arrive, leave range and query the hidden handle.
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{ensure, Context, Result};
use rz_proto::{msg::MovePoint, Message};
use serde_json::{json, Value};

use super::{game, id, mark, read, unit, PacketTrace};
use crate::{App, NetCmd};

#[derive(Debug, PartialEq)]
enum Stage {
    Join,
    Forward,
    Waiting,
    Return,
    Hidden,
    Home,
    Logout,
    Done,
}

pub struct VisibilityValidation {
    directory: PathBuf,
    role: String,
    peer: String,
    stage: Stage,
    since: Instant,
    own: u32,
    mover: u32,
    selected: bool,
    arrived_wait: Option<Instant>,
    query_enters: usize,
    home_query: bool,
    events: BTreeMap<String, usize>,
    steps: Vec<Value>,
    failure: Option<String>,
}

fn near(position: Option<(f32, f32)>, x: f32) -> bool {
    position.is_some_and(|(px, py)| (px - x).abs() < 0.5 && (py - 7167.0).abs() < 0.5)
}

impl VisibilityValidation {
    pub fn new(plan: PathBuf, app: &App) -> Result<Self> {
        super::super::guard(app)?;
        let plan: Value = serde_json::from_slice(&std::fs::read(plan)?)?;
        let role = plan["role"]
            .as_str()
            .context("visibility role")?
            .to_string();
        ensure!(
            role == "mover" || role == "witness",
            "role must be mover or witness"
        );
        let peer = plan["peer"]
            .as_str()
            .context("visibility peer")?
            .to_string();
        ensure!(peer.starts_with("Lua"), "dedicated Lua peer required");
        let directory = PathBuf::from(plan["directory"].as_str().context("report directory")?);
        std::fs::create_dir_all(&directory)?;
        ensure!(
            !directory.join(format!("{role}.json")).exists(),
            "use a fresh attempt directory"
        );
        Ok(Self {
            directory,
            role,
            peer,
            stage: Stage::Join,
            since: Instant::now(),
            own: 0,
            mover: 0,
            selected: false,
            arrived_wait: None,
            query_enters: 0,
            home_query: false,
            events: BTreeMap::new(),
            steps: Vec::new(),
            failure: None,
        })
    }

    fn advance(&mut self, stage: Stage) {
        self.stage = stage;
        self.since = Instant::now();
    }
    pub fn message(&mut self, message: &str) {
        if let Some(rest) = message.strip_prefix("[RappelzDebug] EVENT ") {
            if let Some(event) = rest.split_whitespace().next() {
                *self.events.entry(event.into()).or_default() += 1;
            }
        }
    }
    fn trace<'a>(&self, app: &'a App) -> Result<&'a PacketTrace> {
        app.combat_trace.as_ref().context("trace missing")
    }
    fn record(&mut self, app: &App, stage: &str) {
        let world = app.world.as_ref();
        let state = world
            .map(crate::world::WorldState::addon_state)
            .unwrap_or_default();
        self.steps.push(json!({"stage":stage,"player":state.player.as_ref().map(unit),"target":state.target.as_ref().map(unit),
            "own_position":world.and_then(|w|w.unit_position(self.own)),"mover_position":world.and_then(|w|w.unit_position(self.mover)),
            "peer_visible":world.is_some_and(|w|w.visible_player(&self.peer).is_some()),
            "xp_jp":world.map(|w|w.experience()),"money":state.money,"events":self.events,
            "display":super::super::AddonValidation::display(app)}));
    }
    fn capture(&self) -> Result<()> {
        let path = self.directory.join(format!("{}.png", self.role));
        macroquad::prelude::get_screen_data().export_png(path.to_str().context("screenshot path")?);
        Ok(())
    }

    pub fn tick(&mut self, app: &mut App) -> Result<bool> {
        ensure!(app.error.is_none(), "client error: {:?}", app.error);
        ensure!(
            app.addons.error.is_none() && app.addons.runtime.errors().is_empty(),
            "addon failure"
        );
        ensure!(
            self.trace(app)?.errors.is_empty(),
            "trace errors: {:?}",
            self.trace(app)?.errors
        );
        let other = if self.role == "mover" {
            "witness"
        } else {
            "mover"
        };
        if let Some(report) = read(&self.directory.join(format!("{other}.json"))) {
            ensure!(
                report["success"] != false,
                "peer failed: {}",
                report["failure"]
            );
        }
        if self.since.elapsed() > Duration::from_secs(45) {
            self.record(app, "Timeout");
            anyhow::bail!("visibility timeout at {:?}", self.stage);
        }
        match self.stage {
            Stage::Join => {
                if app.loading.is_some() {
                    return Ok(false);
                }
                let Some(world) = app.world.as_ref() else {
                    return Ok(false);
                };
                let Some(player) = world.addon_state().player else {
                    return Ok(false);
                };
                self.own = player.handle;
                if self.role == "mover" {
                    self.mover = self.own;
                }
                ensure!(
                    player.level == Some(1)
                        && player.health == Some(100)
                        && player.mana == Some(50)
                        && world.experience() == (0, 0),
                    "fresh level-1 full-vitals fixture required"
                );
                ensure!(
                    world.visible_player(&self.peer).is_none(),
                    "fixture peer already visible"
                );
                ensure!(
                    near(
                        world.unit_position(self.own),
                        if self.role == "mover" { 8330.0 } else { 9230.0 }
                    ),
                    "fixture position changed"
                );
                app.auto_enter = None;
                if self.role == "mover" {
                    if read(&self.directory.join("witness.ready")).is_none() {
                        return Ok(false);
                    }
                    self.record(app, "Join");
                    let command = app
                        .world
                        .as_mut()
                        .unwrap()
                        .move_path(vec![
                            MovePoint {
                                x: 8500.0,
                                y: 7167.0,
                            },
                            MovePoint {
                                x: 9050.0,
                                y: 7167.0,
                            },
                        ])
                        .context("forward route")?;
                    app.send(command);
                } else {
                    self.record(app, "Join");
                    mark(&self.directory, "witness.ready", json!({"ready":true}))?;
                }
                self.advance(Stage::Forward);
            }
            Stage::Forward => {
                let world = app.world.as_ref().context("world missing during route")?;
                if self.role == "mover" {
                    if near(world.unit_position(self.own), 9050.0) {
                        ensure!(
                            world.visible_player(&self.peer).is_some(),
                            "mover never saw observer"
                        );
                        self.record(app, "Arrived");
                        self.capture()?;
                        mark(&self.directory, "mover.arrived", json!({"ready":true}))?;
                        self.advance(Stage::Waiting);
                    }
                } else {
                    if !self.selected {
                        if let Some(peer) = world.visible_player(&self.peer) {
                            self.mover = peer.handle;
                            let select = app
                                .world
                                .as_mut()
                                .unwrap()
                                .select_target(self.mover)
                                .context("mover selection")?;
                            app.send(select);
                            app.sync_addons();
                            self.selected = true;
                            self.record(app, "Entry");
                        }
                    }
                    if read(&self.directory.join("mover.arrived")).is_some()
                        && self.arrived_wait.is_none()
                    {
                        self.arrived_wait = Some(Instant::now());
                    }
                    let arrived = near(
                        app.world.as_ref().unwrap().unit_position(self.mover),
                        9050.0,
                    );
                    if self
                        .arrived_wait
                        .is_some_and(|t| t.elapsed() > Duration::from_secs(2))
                        && !arrived
                    {
                        self.record(app, "Stalled");
                        anyhow::bail!(
                            "newly visible player stopped before destination: {:?}",
                            app.world.as_ref().unwrap().unit_position(self.mover)
                        );
                    }
                    if self.selected && arrived {
                        self.record(app, "Arrived");
                        self.capture()?;
                        mark(&self.directory, "witness.arrived", json!({"ready":true}))?;
                        self.advance(Stage::Return);
                    }
                }
            }
            Stage::Waiting => {
                if read(&self.directory.join("witness.arrived")).is_some() {
                    self.record(app, "ReturnStart");
                    let command = app
                        .world
                        .as_mut()
                        .context("world missing")?
                        .move_path(vec![MovePoint {
                            x: 8330.0,
                            y: 7167.0,
                        }])
                        .context("return route")?;
                    app.send(command);
                    self.advance(Stage::Return);
                }
            }
            Stage::Return => {
                let world = app.world.as_ref().context("world missing on return")?;
                if self.role == "mover" {
                    if near(world.unit_position(self.own), 8330.0) {
                        ensure!(
                            world.visible_player(&self.peer).is_none(),
                            "out-of-range observer retained"
                        );
                        self.record(app, "HomePredicted");
                        self.advance(Stage::Home);
                    }
                } else if self.trace(app)?.leaves.contains(&self.mover) {
                    ensure!(
                        world.unit(self.mover).is_none() && world.addon_state().target.is_none(),
                        "range LEAVE did not clear entity/target"
                    );
                    self.query_enters = self
                        .trace(app)?
                        .enters
                        .iter()
                        .filter(|e| e["handle"] == self.mover)
                        .count();
                    self.record(app, "RangeLeave");
                    app.send(NetCmd::Raw(
                        game::Handle::<{ id::TS_CS_QUERY }>(self.mover).to_packet(),
                    ));
                    self.advance(Stage::Hidden);
                }
            }
            Stage::Hidden => {
                ensure!(
                    app.world
                        .as_ref()
                        .context("world missing during hidden query")?
                        .unit(self.mover)
                        .is_none(),
                    "query reintroduced out-of-range player"
                );
                ensure!(
                    self.trace(app)?
                        .enters
                        .iter()
                        .filter(|e| e["handle"] == self.mover)
                        .count()
                        == self.query_enters,
                    "hidden query received ENTER"
                );
                if self.since.elapsed() >= Duration::from_secs(1) {
                    self.record(app, "HiddenQuery");
                    mark(&self.directory, "witness.hidden", json!({"ready":true}))?;
                    self.advance(Stage::Home);
                }
            }
            Stage::Home => {
                if self.role == "mover" && !self.directory.join("mover.home").exists() {
                    if !self.home_query && self.since.elapsed() >= Duration::from_millis(500) {
                        self.query_enters = self
                            .trace(app)?
                            .enters
                            .iter()
                            .filter(|e| e["handle"] == self.mover)
                            .count();
                        app.send(NetCmd::Raw(
                            game::Handle::<{ id::TS_CS_QUERY }>(self.mover).to_packet(),
                        ));
                        self.home_query = true;
                    }
                    let enters: Vec<_> = self
                        .trace(app)?
                        .enters
                        .iter()
                        .filter(|e| e["handle"] == self.mover)
                        .collect();
                    if self.home_query && enters.len() > self.query_enters {
                        let last = enters.last().unwrap();
                        ensure!(
                            last["x"] == 8330.0 && last["y"] == 7167.0,
                            "server has not reached home: {last}"
                        );
                        self.record(app, "Home");
                        mark(&self.directory, "mover.home", json!({"ready":true}))?;
                    }
                }
                if read(&self.directory.join("mover.home")).is_some()
                    && read(&self.directory.join("witness.hidden")).is_some()
                {
                    app.send(NetCmd::ReturnLobby);
                    self.advance(Stage::Logout);
                }
            }
            Stage::Logout => {
                if app.world.is_none() && self.events.get("PLAYER_LOGOUT") == Some(&1) {
                    self.record(app, "Logout");
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
        let _ = self.capture();
    }
    pub fn finish(&self, app: &App) -> Result<bool> {
        let success = self.stage == Stage::Done && self.failure.is_none();
        mark(
            &self.directory,
            &format!("{}.json", self.role),
            json!({"scenario":"visibility","success":success,"failure":self.failure,
            "role":self.role,"account":app.account,"own":self.own,"mover":self.mover,"events":self.events,"steps":self.steps,"trace":self.trace(app)?.json()}),
        )?;
        Ok(success)
    }
}
