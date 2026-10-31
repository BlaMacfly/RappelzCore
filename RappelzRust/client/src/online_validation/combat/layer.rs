//! Authorized live layer changes, ordinary-player denials, hidden interactions and relogin.
use super::{game, id, mark, read, unit, PacketTrace};
use crate::{App, NetCmd};
use anyhow::{ensure, Context, Result};
use rz_proto::Message;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq)]
enum Stage {
    Join,
    Permission,
    Dialog,
    Invalid,
    Away,
    Probe,
    FirstLogout,
    Reenter,
    Restore,
    Confirm,
    Hold,
    Leave,
    Logout,
    Done,
}

pub struct LayerValidation {
    directory: PathBuf,
    role: String,
    peer: String,
    character: String,
    stage: Stage,
    since: Instant,
    own: u32,
    old_operator: u32,
    operator: u32,
    peer_handle: u32,
    npc: u32,
    monster: u32,
    probe_enters: usize,
    probe_targets: usize,
    dialog_count: usize,
    query_enters: usize,
    probed: bool,
    restore_sent: bool,
    events: BTreeMap<String, usize>,
    messages: Vec<String>,
    steps: Vec<Value>,
    failure: Option<String>,
}

impl LayerValidation {
    pub fn new(plan: PathBuf, app: &App) -> Result<Self> {
        super::super::guard(app)?;
        let plan: Value = serde_json::from_slice(&std::fs::read(plan)?)?;
        let role = plan["role"].as_str().context("layer role")?.to_string();
        ensure!(
            role == "operator" || role == "witness",
            "role must be operator or witness"
        );
        let peer = plan["peer"].as_str().context("layer peer")?.to_string();
        ensure!(peer.starts_with("Lua"), "dedicated Lua peer required");
        let directory = PathBuf::from(plan["directory"].as_str().context("report directory")?);
        std::fs::create_dir_all(&directory)?;
        ensure!(
            !directory.join(format!("{role}.json")).exists(),
            "fresh attempt directory required"
        );
        Ok(Self {
            directory,
            role,
            peer,
            character: app.auto_enter.clone().context("character required")?,
            stage: Stage::Join,
            since: Instant::now(),
            own: 0,
            old_operator: 0,
            operator: 0,
            peer_handle: 0,
            npc: 0,
            monster: 0,
            probe_enters: 0,
            probe_targets: 0,
            dialog_count: 0,
            query_enters: 0,
            probed: false,
            restore_sent: false,
            events: BTreeMap::new(),
            messages: Vec::new(),
            steps: Vec::new(),
            failure: None,
        })
    }
    fn advance(&mut self, stage: Stage) {
        self.stage = stage;
        self.since = Instant::now();
    }
    fn chat(app: &App, text: &str) {
        app.send(NetCmd::Chat {
            chat_type: 0,
            target: String::new(),
            message: text.into(),
        });
    }
    fn trace<'a>(&self, app: &'a App) -> Result<&'a PacketTrace> {
        app.combat_trace.as_ref().context("trace missing")
    }
    pub fn message(&mut self, message: &str) {
        if let Some(s) = message.strip_prefix("[RappelzDebug] EVENT ") {
            if let Some(event) = s.split_whitespace().next() {
                *self.events.entry(event.into()).or_default() += 1;
            }
        }
        if self.messages.len() < 512 {
            self.messages.push(message.into());
        }
    }
    fn display_peer(&self) -> String {
        if self.role == "witness" {
            format!("<GM>{}", self.peer)
        } else {
            self.peer.clone()
        }
    }
    fn record(&mut self, app: &App, stage: &str) {
        let world = app.world.as_ref();
        let state = world
            .map(crate::world::WorldState::addon_state)
            .unwrap_or_default();
        self.steps.push(json!({"stage":stage,"player":state.player.as_ref().map(unit),"target":state.target.as_ref().map(unit),
            "layer":world.map(|w|w.layer()),"entities":world.map(|w|w.entity_count()),"dialog_open":world.map(|w|w.dialog_open()),
            "position":world.and_then(|w|w.unit_position(self.own)),"xp_jp":world.map(|w|w.experience()),"money":state.money,
            "bag_revision":state.bag_revision,"events":self.events,"display":super::super::AddonValidation::display(app)}));
    }
    fn capture(&self) -> Result<()> {
        self.capture_named(&self.role)
    }

    fn capture_named(&self, name: &str) -> Result<()> {
        let p = self.directory.join(format!("{name}.png"));
        macroquad::prelude::get_screen_data().export_png(p.to_str().context("image path")?);
        Ok(())
    }
    fn start_probe(&mut self, app: &App) {
        self.probe_enters = app.combat_trace.as_ref().unwrap().enters.len();
        self.probe_targets = app.combat_trace.as_ref().unwrap().targets.len();
        app.send(NetCmd::Raw(
            game::Handle::<{ id::TS_CS_QUERY }>(self.peer_handle).to_packet(),
        ));
        app.send(NetCmd::Raw(
            game::Handle::<{ id::TS_CS_TARGETING }>(self.peer_handle).to_packet(),
        ));
        if self.role == "operator" {
            app.send(NetCmd::Raw(
                game::CsAttackRequest {
                    handle: self.own,
                    target_handle: self.monster,
                }
                .to_packet(),
            ));
            app.send(NetCmd::Raw(
                game::Handle::<{ id::TS_CS_CONTACT }>(self.npc).to_packet(),
            ));
            app.send(NetCmd::Raw(
                game::CsDialog {
                    trigger: "market".into(),
                }
                .to_packet(),
            ));
        }
    }
    pub fn tick(&mut self, app: &mut App) -> Result<bool> {
        ensure!(app.error.is_none(), "client error: {:?}", app.error);
        ensure!(
            app.addons.error.is_none() && app.addons.runtime.errors().is_empty(),
            "addon error"
        );
        ensure!(
            self.trace(app)?.errors.is_empty(),
            "trace errors: {:?}",
            self.trace(app)?.errors
        );
        let other = if self.role == "operator" {
            "witness"
        } else {
            "operator"
        };
        if let Some(r) = read(&self.directory.join(format!("{other}.json"))) {
            ensure!(r["success"] != false, "peer failed: {}", r["failure"]);
        }
        if self.since.elapsed() > Duration::from_secs(45) {
            self.record(app, "Timeout");
            anyhow::bail!("layer timeout at {:?}", self.stage);
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
                let Some(peer) = world.visible_player(&self.display_peer()) else {
                    return Ok(false);
                };
                let Some(npc) = world.visible_npc() else {
                    return Ok(false);
                };
                let Some(monster) = world.living_monster() else {
                    return Ok(false);
                };
                ensure!(
                    world.layer() == 0
                        && player.layer == Some(0)
                        && player.level == Some(1)
                        && player.health == Some(100)
                        && player.mana == Some(50)
                        && world.experience() == (0, 0),
                    "layer-0 fixture changed"
                );
                self.own = player.handle;
                self.peer_handle = peer.handle;
                self.npc = npc.handle;
                self.monster = monster.handle;
                self.operator = if self.role == "operator" {
                    self.own
                } else {
                    peer.handle
                };
                self.old_operator = self.operator;
                app.auto_enter = None;
                let target = if self.role == "operator" {
                    self.npc
                } else {
                    self.operator
                };
                let command = app
                    .world
                    .as_mut()
                    .unwrap()
                    .select_target(target)
                    .context("initial target")?;
                app.send(command);
                app.sync_addons();
                self.record(app, "Join");
                if self.role == "operator" {
                    app.send(NetCmd::Raw(
                        game::Handle::<{ id::TS_CS_CONTACT }>(self.npc).to_packet(),
                    ));
                    self.advance(Stage::Dialog);
                } else {
                    Self::chat(app, ".layer 1");
                    self.advance(Stage::Permission);
                }
            }
            Stage::Permission => {
                if self.messages.iter().any(|m| m.contains("niveau requis")) {
                    ensure!(
                        app.world.as_ref().unwrap().layer() == 0
                            && self.trace(app)?.warps.is_empty(),
                        "ordinary player changed layer"
                    );
                    self.record(app, "PermissionDenied");
                    mark(&self.directory, "witness.ready", json!({"ready":true}))?;
                    self.advance(Stage::Away);
                }
            }
            Stage::Dialog => {
                if self.trace(app)?.dialogs.len() == 1
                    && app.world.as_ref().unwrap().dialog_open()
                    && read(&self.directory.join("witness.ready")).is_some()
                {
                    self.dialog_count = 1;
                    self.record(app, "InitialDialog");
                    Self::chat(app, ".layer 256");
                    self.advance(Stage::Invalid);
                }
            }
            Stage::Invalid => {
                if self.messages.iter().any(|m| m.contains("Usage : .layer")) {
                    ensure!(
                        self.trace(app)?.warps.is_empty()
                            && app.world.as_ref().unwrap().layer() == 0,
                        "invalid layer changed state"
                    );
                    self.record(app, "InvalidLayerDenied");
                    Self::chat(app, ".layer 1");
                    self.advance(Stage::Away);
                }
            }
            Stage::Away => {
                let world = app.world.as_ref().context("world missing at separation")?;
                if self.role == "operator" {
                    if world.layer() != 1 {
                        return Ok(false);
                    }
                    ensure!(
                        world.entity_count() == 0
                            && !world.dialog_open()
                            && world.addon_state().target.is_none(),
                        "old-layer content retained"
                    );
                    self.record(app, "Away");
                    self.start_probe(app);
                    mark(&self.directory, "operator.away", json!({"ready":true}))?;
                    self.advance(Stage::Probe);
                } else if self.trace(app)?.leaves.contains(&self.old_operator)
                    && read(&self.directory.join("operator.away")).is_some()
                {
                    ensure!(
                        world.layer() == 0
                            && world.unit(self.old_operator).is_none()
                            && world.addon_state().target.is_none(),
                        "operator retained across layers"
                    );
                    self.record(app, "Away");
                    self.start_probe(app);
                    self.advance(Stage::Probe);
                }
            }
            Stage::Probe => {
                if !self.probed && self.since.elapsed() >= Duration::from_secs(1) {
                    self.record(app, "Probe");
                    let trace = self.trace(app)?;
                    if self.role == "operator" {
                        ensure!(
                            trace.dialogs.len() == self.dialog_count
                                && !app.world.as_ref().unwrap().dialog_open(),
                            "hidden NPC dialogue or market accepted across layers"
                        );
                        ensure!(
                            trace
                                .denied_attacks
                                .iter()
                                .any(|d| d["attacker"] == self.own && d["target"] == self.monster),
                            "hidden monster attack not denied"
                        );
                    }
                    ensure!(
                        trace.enters.len() == self.probe_enters,
                        "hidden peer QUERY disclosed content"
                    );
                    ensure!(
                        trace.targets.len() > self.probe_targets
                            && trace.targets.last() == Some(&0),
                        "hidden target was not refused"
                    );
                    ensure!(trace.hits.is_empty(), "cross-layer attack occurred");
                    self.record(app, "Isolated");
                    if self.role == "operator" {
                        self.capture_named("operator-layer1")?;
                    }
                    mark(
                        &self.directory,
                        &format!("{}.probed", self.role),
                        json!({"ready":true}),
                    )?;
                    self.probed = true;
                    if self.role == "witness" {
                        self.advance(Stage::Restore);
                    }
                }
                if self.role == "operator"
                    && self.probed
                    && read(&self.directory.join("witness.probed")).is_some()
                {
                    app.send(NetCmd::ReturnLobby);
                    self.advance(Stage::FirstLogout);
                }
            }
            Stage::FirstLogout => {
                if app.world.is_none() && self.events.get("PLAYER_LOGOUT") == Some(&1) {
                    self.record(app, "Disconnected");
                    app.send(NetCmd::EnterWorld(self.character.clone()));
                    self.advance(Stage::Reenter);
                }
            }
            Stage::Reenter => {
                if app.loading.is_some() {
                    return Ok(false);
                }
                let Some(world) = app.world.as_ref() else {
                    return Ok(false);
                };
                let own = world
                    .addon_state()
                    .player
                    .context("player missing after relogin")?
                    .handle;
                self.own = own;
                self.operator = own;
                self.record(app, "Reentered");
                ensure!(
                    own != self.old_operator,
                    "relogin did not create a new session handle"
                );
                ensure!(
                    app.world.as_ref().unwrap().layer() == 1,
                    "saved layer was lost on relogin"
                );
                ensure!(
                    app.world.as_ref().unwrap().entity_count() == 0,
                    "layer-0 objects leaked after relogin"
                );
                Self::chat(app, ".layer 0");
                self.advance(Stage::Confirm);
            }
            Stage::Restore => {
                let Some(peer) = app
                    .world
                    .as_ref()
                    .context("witness world missing")?
                    .visible_player(&self.display_peer())
                else {
                    return Ok(false);
                };
                ensure!(
                    peer.handle != self.old_operator && peer.layer == Some(0),
                    "restored session identity/layer incorrect"
                );
                self.operator = peer.handle;
                let select = app
                    .world
                    .as_mut()
                    .unwrap()
                    .select_target(self.operator)
                    .context("restored target")?;
                app.send(select);
                app.sync_addons();
                self.record(app, "Restored");
                self.query_enters = self
                    .trace(app)?
                    .enters
                    .iter()
                    .filter(|e| e["handle"] == self.operator)
                    .count();
                app.send(NetCmd::Raw(
                    game::Handle::<{ id::TS_CS_QUERY }>(self.operator).to_packet(),
                ));
                self.advance(Stage::Confirm);
            }
            Stage::Confirm => {
                let world = app
                    .world
                    .as_ref()
                    .context("world missing after restoration")?;
                if self.role == "operator" {
                    if world.layer() != 0 || world.visible_npc().is_none() {
                        return Ok(false);
                    }
                    if !self.restore_sent {
                        self.query_enters = self
                            .trace(app)?
                            .enters
                            .iter()
                            .filter(|e| e["handle"] == self.peer_handle)
                            .count();
                        let select = app
                            .world
                            .as_mut()
                            .unwrap()
                            .select_target(self.peer_handle)
                            .context("restored witness")?;
                        app.send(select);
                        app.sync_addons();
                        app.send(NetCmd::Raw(
                            game::Handle::<{ id::TS_CS_QUERY }>(self.peer_handle).to_packet(),
                        ));
                        app.send(NetCmd::Raw(
                            game::Handle::<{ id::TS_CS_CONTACT }>(self.npc).to_packet(),
                        ));
                        self.restore_sent = true;
                    }
                    if self.trace(app)?.dialogs.len() != 2
                        || !app.world.as_ref().unwrap().dialog_open()
                    {
                        return Ok(false);
                    }
                    if self
                        .trace(app)?
                        .enters
                        .iter()
                        .filter(|e| e["handle"] == self.peer_handle)
                        .count()
                        <= self.query_enters
                    {
                        return Ok(false);
                    }
                    ensure!(
                        self.events.get("RAPPELZ_LAYER_CHANGED") == Some(&2),
                        "layer events missing/duplicated"
                    );
                    self.record(app, "Restored");
                    self.capture()?;
                    self.advance(Stage::Hold);
                } else {
                    let enters: Vec<_> = self
                        .trace(app)?
                        .enters
                        .iter()
                        .filter(|e| e["handle"] == self.operator)
                        .collect();
                    if enters.len() <= self.query_enters {
                        return Ok(false);
                    }
                    let last = enters.last().unwrap();
                    ensure!(
                        last["layer"] == 0 && last["hp"] == 100 && last["mp"] == 50,
                        "restored query incorrect"
                    );
                    self.record(app, "Queried");
                    self.capture()?;
                    mark(&self.directory, "witness.restored", json!({"ready":true}))?;
                    self.advance(Stage::Leave);
                }
            }
            Stage::Hold => {
                if read(&self.directory.join("witness.restored")).is_some() {
                    app.send(NetCmd::ReturnLobby);
                    self.advance(Stage::Logout);
                }
            }
            Stage::Leave => {
                if self.trace(app)?.leaves.contains(&self.operator) {
                    ensure!(
                        app.world.as_ref().unwrap().addon_state().target.is_none(),
                        "target retained after operator logout"
                    );
                    self.record(app, "PeerLogout");
                    app.send(NetCmd::ReturnLobby);
                    self.advance(Stage::Logout);
                }
            }
            Stage::Logout => {
                let expected = if self.role == "operator" { 2 } else { 1 };
                if app.world.is_none() && self.events.get("PLAYER_LOGOUT") == Some(&expected) {
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
            json!({"scenario":"layer","success":success,"failure":self.failure,
            "role":self.role,"account":app.account,"own":self.own,"old_operator":self.old_operator,"operator":self.operator,
            "npc":self.npc,"monster":self.monster,"events":self.events,"messages":self.messages,"steps":self.steps,"trace":self.trace(app)?.json()}),
        )?;
        Ok(success)
    }
}
