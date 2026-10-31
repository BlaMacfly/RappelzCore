//! Real lethal retaliation and automatic revival, including the untouched observer view.
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{ensure, Context, Result};
use rz_proto::Message;
use serde_json::{json, Value};

use super::{game, id, mark, read, unit, PacketTrace};
use crate::{App, NetCmd};

#[derive(Debug, PartialEq)]
enum Stage {
    Join,
    Fatal,
    Observe,
    Query,
    Hold,
    Leave,
    Logout,
    Done,
}

pub struct RevivalValidation {
    directory: PathBuf,
    role: String,
    peer: String,
    stage: Stage,
    since: Instant,
    own: u32,
    victim: u32,
    monster: u32,
    query_enters: usize,
    cancelled: bool,
    events: BTreeMap<String, usize>,
    messages: Vec<String>,
    steps: Vec<Value>,
    failure: Option<String>,
}

fn revived(trace: &PacketTrace, handle: u32) -> bool {
    let mut died = false;
    for h in trace.hpmp.iter().filter(|h| h["handle"] == handle) {
        if h["hp"] == 0 {
            died = true;
        } else if died && h["hp"] == 100 && h["max_hp"] == 100 && h["mp"] == 50 && h["max_mp"] == 50
        {
            return true;
        }
    }
    false
}

fn at_spawn(position: Option<(f32, f32)>) -> bool {
    position.is_some_and(|(x, y)| (x - 8330.0).abs() < 0.1 && (y - 7167.0).abs() < 0.1)
}

impl RevivalValidation {
    pub fn new(plan: PathBuf, app: &App) -> Result<Self> {
        super::super::guard(app)?;
        let plan: Value = serde_json::from_slice(&std::fs::read(plan)?)?;
        let role = plan["role"].as_str().context("revival role")?.to_string();
        ensure!(
            role == "victim" || role == "witness",
            "revival role must be victim or witness"
        );
        let peer = plan["peer"].as_str().context("revival peer")?.to_string();
        ensure!(peer.starts_with("Lua"), "dedicated Lua peer required");
        let directory = PathBuf::from(plan["directory"].as_str().context("report directory")?);
        std::fs::create_dir_all(&directory)?;
        ensure!(
            !directory.join(format!("{role}.json")).exists(),
            "use a fresh run directory"
        );
        Ok(Self {
            directory,
            role,
            peer,
            stage: Stage::Join,
            since: Instant::now(),
            own: 0,
            victim: 0,
            monster: 0,
            query_enters: 0,
            cancelled: false,
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

    pub fn message(&mut self, message: &str) {
        if let Some(rest) = message.strip_prefix("[RappelzDebug] EVENT ") {
            if let Some(event) = rest.split_whitespace().next() {
                *self.events.entry(event.into()).or_default() += 1;
            }
        }
        if self.messages.len() < 512 {
            self.messages.push(message.into());
        }
    }

    fn trace<'a>(&self, app: &'a App) -> Result<&'a PacketTrace> {
        app.combat_trace.as_ref().context("revival trace missing")
    }

    fn record(&mut self, app: &App) {
        let world = app.world.as_ref();
        let state = world
            .map(crate::world::WorldState::addon_state)
            .unwrap_or_default();
        self.steps.push(json!({"stage":format!("{:?}",self.stage),
            "player":state.player.as_ref().map(unit),"target":state.target.as_ref().map(unit),
            "victim_position":world.and_then(|w|w.unit_position(self.victim)),
            "victim_renderable":world.and_then(|w|w.unit_renderable(self.victim)),
            "victim_enter_count":app.combat_trace.as_ref().map(|t|t.enters.iter().filter(|e|e["handle"]==self.victim).count()),
            "observation_ms":if self.stage==Stage::Observe { self.since.elapsed().as_millis() } else {0},
            "xp_jp":world.map(|w|w.experience()),"money":state.money,
            "display":super::super::AddonValidation::display(app),"events":self.events}));
    }

    fn capture(&self) -> Result<()> {
        let path = self.directory.join(format!("{}.png", self.role));
        macroquad::prelude::get_screen_data().export_png(path.to_str().context("screenshot path")?);
        Ok(())
    }

    pub fn tick(&mut self, app: &mut App) -> Result<bool> {
        ensure!(app.error.is_none(), "client/network error: {:?}", app.error);
        ensure!(
            app.addons.error.is_none() && app.addons.runtime.errors().is_empty(),
            "addon error: {:?} {:?}",
            app.addons.error,
            app.addons.runtime.errors()
        );
        ensure!(
            self.trace(app)?.errors.is_empty(),
            "trace error: {:?}",
            self.trace(app)?.errors
        );
        let other = if self.role == "victim" {
            "witness"
        } else {
            "victim"
        };
        if let Some(report) = read(&self.directory.join(format!("{other}.json"))) {
            ensure!(
                report["success"] != false,
                "peer failed: {}",
                report["failure"]
            );
        }
        if self.since.elapsed() > Duration::from_secs(45) {
            self.record(app);
            anyhow::bail!("revival timeout at {:?} ({})", self.stage, self.role);
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
                let Some(peer) = world.visible_player(&self.peer) else {
                    return Ok(false);
                };
                let Some(monster) = world.living_monster() else {
                    return Ok(false);
                };
                self.own = player.handle;
                self.victim = if self.role == "victim" {
                    player.handle
                } else {
                    peer.handle
                };
                self.monster = monster.handle;
                ensure!(
                    player.level == Some(1) && world.experience() == (0, 0),
                    "level-1/no-XP fixture required"
                );
                ensure!(
                    monster.health == Some(105) && monster.mana == Some(53),
                    "monster fixture changed"
                );
                app.auto_enter = None;
                if self.role == "victim" {
                    ensure!(
                        player.health.is_some_and(|hp| hp > 0 && hp <= 40),
                        "dedicated low-HP fixture required"
                    );
                    if read(&self.directory.join("witness.ready")).is_none() {
                        return Ok(false);
                    }
                    let world = app.world.as_mut().unwrap();
                    let select = world
                        .select_target(self.monster)
                        .context("monster selection")?;
                    let attack = world
                        .attack_target(self.monster)
                        .context("monster attack")?;
                    app.send(select);
                    app.sync_addons();
                    self.record(app);
                    app.send(attack);
                } else {
                    ensure!(
                        player.health == Some(100) && player.mana == Some(50),
                        "witness fixture changed"
                    );
                    let select = app
                        .world
                        .as_mut()
                        .unwrap()
                        .select_target(self.victim)
                        .context("victim selection")?;
                    app.send(select);
                    app.sync_addons();
                    self.record(app);
                    mark(&self.directory, "witness.ready", json!({"handle":self.own}))?;
                }
                self.advance(Stage::Fatal);
            }
            Stage::Fatal => {
                let trace = self.trace(app)?;
                if self.role == "victim"
                    && !self.cancelled
                    && trace.hits.iter().any(|h| h["attacker"] == self.victim)
                {
                    // Stop our auto-attack; the living monster continues its real retaliation.
                    app.send(NetCmd::Raw(
                        game::Handle::<{ id::TS_CS_CANCEL_ACTION }>(self.victim).to_packet(),
                    ));
                    self.cancelled = true;
                }
                if !revived(self.trace(app)?, self.victim) {
                    return Ok(false);
                }
                ensure!(
                    self.trace(app)?
                        .hits
                        .iter()
                        .any(|h| h["attacker"] == self.monster
                            && h["target"] == self.victim
                            && h["target_hp"] == 0),
                    "no real lethal retaliation"
                );
                ensure!(self.trace(app)?.levels.is_empty(), "death changed level");
                ensure!(
                    app.world.as_ref().context("world missing")?.experience() == (0, 0),
                    "death awarded XP/JP"
                );
                self.record(app);
                self.advance(Stage::Observe);
            }
            Stage::Observe => {
                // Never QUERY before this check: a fresh ENTER would mask observer defects.
                if self.since.elapsed() < Duration::from_millis(3500) {
                    return Ok(false);
                }
                self.record(app);
                let world = app.world.as_ref().context("world missing after revival")?;
                ensure!(
                    world.unit_renderable(self.victim) == Some(true),
                    "revived player disappeared from observer rendering"
                );
                ensure!(
                    at_spawn(world.unit_position(self.victim)),
                    "revived position not synchronized: {:?}",
                    world.unit_position(self.victim)
                );
                let state = world.addon_state();
                let revived = if self.role == "victim" {
                    state.player.as_ref()
                } else {
                    state.target.as_ref()
                }
                .context("revived unit missing")?;
                ensure!(
                    revived.health == Some(100)
                        && revived.max_health == Some(100)
                        && revived.mana == Some(50)
                        && revived.max_mana == Some(50)
                        && revived.level == Some(1),
                    "revival stats: {revived:?}"
                );
                let count = if self.role == "victim" { 1 } else { 0 };
                for event in ["PLAYER_DEAD", "PLAYER_ALIVE"] {
                    ensure!(
                        self.events.get(event).copied().unwrap_or(0) == count,
                        "incorrect {event} event count"
                    );
                }
                self.capture()?;
                if self.role == "victim" {
                    ensure!(
                        self.trace(app)?
                            .warps
                            .iter()
                            .any(|w| w["x"] == 8330.0 && w["y"] == 7167.0),
                        "revival WARP missing"
                    );
                    self.advance(Stage::Hold);
                } else {
                    self.query_enters = self
                        .trace(app)?
                        .enters
                        .iter()
                        .filter(|e| e["handle"] == self.victim)
                        .count();
                    app.send(NetCmd::Raw(
                        game::Handle::<{ id::TS_CS_QUERY }>(self.victim).to_packet(),
                    ));
                    self.advance(Stage::Query);
                }
            }
            Stage::Query => {
                let trace = self.trace(app)?;
                let enters: Vec<_> = trace
                    .enters
                    .iter()
                    .filter(|e| e["handle"] == self.victim)
                    .collect();
                if enters.len() <= self.query_enters {
                    return Ok(false);
                }
                let last = enters.last().unwrap();
                ensure!(
                    last["x"] == 8330.0
                        && last["y"] == 7167.0
                        && last["hp"] == 100
                        && last["mp"] == 50,
                    "fresh revival ENTER disagrees: {last}"
                );
                self.record(app);
                mark(
                    &self.directory,
                    "witness.verified",
                    json!({"handle":self.victim}),
                )?;
                self.advance(Stage::Leave);
            }
            Stage::Hold => {
                if read(&self.directory.join("witness.verified")).is_some() {
                    app.send(NetCmd::ReturnLobby);
                    self.advance(Stage::Logout);
                }
            }
            Stage::Leave => {
                if self.trace(app)?.leaves.contains(&self.victim) {
                    ensure!(
                        app.world
                            .as_ref()
                            .context("witness world missing")?
                            .addon_state()
                            .target
                            .is_none(),
                        "target retained after victim logout"
                    );
                    self.record(app);
                    app.send(NetCmd::ReturnLobby);
                    self.advance(Stage::Logout);
                }
            }
            Stage::Logout => {
                if app.world.is_none() && self.events.get("PLAYER_LOGOUT") == Some(&1) {
                    self.record(app);
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
            json!({"scenario":"revival","success":success,
            "failure":self.failure,"role":self.role,"account":app.account,"own":self.own,"victim":self.victim,
            "monster":self.monster,"events":self.events,"messages":self.messages,"steps":self.steps,"trace":self.trace(app)?.json()}),
        )?;
        Ok(success)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revival_requires_zero_then_full_vitals_for_the_same_handle() {
        let mut trace = PacketTrace {
            hpmp: vec![json!({"handle":1,"hp":100,"max_hp":100,"mp":50,"max_mp":50})],
            ..Default::default()
        };
        assert!(!revived(&trace, 1));
        trace.hpmp.insert(0, json!({"handle":2,"hp":0}));
        assert!(!revived(&trace, 1));
        trace.hpmp.insert(0, json!({"handle":1,"hp":0}));
        assert!(revived(&trace, 1));
        trace.hpmp[2]["mp"] = json!(1);
        assert!(!revived(&trace, 1));
    }
}
