//! Guarded two-client casting, interruption, private cooldown and lobby re-entry scenario.
use super::{game, id, mark, read, unit};
use crate::{App, NetCmd};
use anyhow::{ensure, Context, Result};
use rz_proto::{msg::MovePoint, Message};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    time::{Duration, Instant},
};

pub struct CastingValidation {
    directory: PathBuf,
    role: String,
    peer: String,
    character: String,
    stage: u8,
    started: Instant,
    own: u32,
    initial: u32,
    other: u32,
    initial_other: u32,
    ready: bool,
    requests: Vec<Value>,
    steps: Vec<Value>,
    events: BTreeMap<String, usize>,
    messages: Vec<String>,
    failure: Option<String>,
}
impl CastingValidation {
    pub fn new(plan: PathBuf, app: &App) -> Result<Self> {
        super::super::guard(app)?;
        let p: Value = serde_json::from_slice(&std::fs::read(plan)?)?;
        let role = p["role"].as_str().context("cast role")?.to_string();
        ensure!(role == "owner" || role == "witness", "cast role");
        let peer = p["peer"].as_str().context("peer")?.to_string();
        ensure!(peer.starts_with("Lua"), "dedicated peer required");
        let directory = PathBuf::from(p["directory"].as_str().context("directory")?);
        std::fs::create_dir_all(&directory)?;
        ensure!(
            !directory.join(format!("{role}.json")).exists(),
            "fresh attempt required"
        );
        Ok(Self {
            directory,
            role,
            peer,
            character: app.auto_enter.clone().context("character")?,
            stage: 0,
            started: Instant::now(),
            own: 0,
            initial: 0,
            other: 0,
            initial_other: 0,
            ready: false,
            requests: vec![],
            steps: vec![],
            events: BTreeMap::new(),
            messages: vec![],
            failure: None,
        })
    }
    pub fn message(&mut self, line: &str) {
        if let Some(event) = line
            .strip_prefix("[RappelzDebug] EVENT ")
            .and_then(|s| s.split_whitespace().next())
        {
            *self.events.entry(event.into()).or_default() += 1;
        }
        if self.messages.len() < 512 {
            self.messages.push(line.into());
        }
    }
    fn ready(&self, key: &str) -> bool {
        read(&self.directory.join(key)).is_some()
    }
    fn mark(&self, key: &str) -> Result<()> {
        mark(&self.directory, key, json!({"ready":true}))
    }
    fn record(&mut self, app: &App, name: &str) {
        let w = app.world.as_ref();
        let s = w.map(|w| w.addon_state()).unwrap_or_default();
        self.steps.push(json!({"stage":name,"player":s.player.as_ref().map(unit),"target":s.target.as_ref().map(unit),
            "learned":s.learned_skills,"jp":s.job_points,"money":s.money,"bag_revision":s.bag_revision,
            "remaining":w.map(|w|w.skill_remaining(3212)),"casting":w.map(|w|w.is_casting()),
            "position":w.and_then(|w|w.unit_position(self.own)),"display":super::super::AddonValidation::display(app),"events":self.events}));
    }
    fn capture(&self, name: &str) -> Result<()> {
        let p = self.directory.join(format!("{name}.png"));
        macroquad::prelude::get_screen_data().export_png(p.to_str().context("capture path")?);
        Ok(())
    }
    fn request(&mut self, app: &App, r: game::CsSkill) {
        self.requests.push(json!({"id":r.skill_id,"level":r.skill_level,"caster":r.caster,"target":r.target,"layer":r.layer}));
        app.send(NetCmd::Raw(r.to_packet()));
    }
    fn raw(&mut self, app: &App) {
        self.request(
            app,
            game::CsSkill {
                skill_id: 3212,
                caster: self.own,
                target: self.own,
                skill_level: 1,
                layer: 0,
                ..Default::default()
            },
        );
    }
    fn cast(&mut self, app: &App) -> Result<()> {
        let q = app
            .assets
            .as_ref()
            .context("assets")?
            .restoration_quote(3212, 1)
            .context("verified restoration data")?;
        let NetCmd::Raw(p) = app
            .world
            .as_ref()
            .context("world")?
            .cast_skill(&q)
            .context("production cast helper refused ready cast")?
        else {
            anyhow::bail!("cast command");
        };
        self.request(app, game::CsSkill::from_packet(&p)?);
        Ok(())
    }
    pub fn tick(&mut self, app: &mut App) -> Result<bool> {
        ensure!(
            app.error.is_none()
                && app.addons.error.is_none()
                && app.addons.runtime.errors().is_empty(),
            "client/addon error"
        );
        ensure!(
            self.started.elapsed() < Duration::from_secs(100),
            "casting stage {} timed out",
            self.stage
        );
        let trace = app.combat_trace.as_ref().context("trace")?;
        ensure!(trace.errors.is_empty(), "trace error");
        let other_role = if self.role == "owner" {
            "witness"
        } else {
            "owner"
        };
        if let Some(done) = read(&self.directory.join(format!("{other_role}.json"))) {
            ensure!(done["success"] == true, "peer failed");
        }
        if self.stage == 14 {
            let count = if self.role == "owner" { 2 } else { 1 };
            if app.world.is_none() && self.events.get("PLAYER_LOGOUT") == Some(&count) {
                self.record(app, "Logout");
                self.stage = 15;
                return Ok(true);
            }
            return Ok(false);
        }
        if self.stage == 7 {
            if app.world.is_none() && self.events.get("PLAYER_LOGOUT") == Some(&1) {
                self.record(app, "Lobby");
                app.send(NetCmd::EnterWorld(self.character.clone()));
                self.stage = 8;
            }
            return Ok(false);
        }
        let Some(w) = app.world.as_ref() else {
            return Ok(false);
        };
        if app.loading.is_some() {
            return Ok(false);
        }
        let Some(known) = w.learned_skills() else {
            return Ok(false);
        };
        let Some(catalog) = w.skill_catalog() else {
            return Ok(false);
        };
        let phases = |phase: &str| {
            trace
                .cast_events
                .iter()
                .filter(|e| e["phase"] == phase)
                .count()
        };
        if self.stage == 0 {
            let Some(peer) = w.visible_player(&self.peer) else {
                return Ok(false);
            };
            if !self.ready {
                self.own = w.addon_state().player.context("player")?.handle;
                self.initial = self.own;
                self.other = peer.handle;
                self.initial_other = peer.handle;
                ensure!(
                    catalog.jp == if self.role == "owner" { 180 } else { 0 },
                    "fixture JP"
                );
                ensure!(
                    known.len() == if self.role == "owner" { 1 } else { 0 },
                    "fixture learned list"
                );
                app.auto_enter = None;
                app.world.as_mut().unwrap().focus_skill(3212);
                if self.role == "witness" {
                    if let Some(cmd) = app.world.as_mut().unwrap().select_target(self.other) {
                        app.send(cmd);
                    }
                }
                self.record(app, "Initial");
                self.mark(&format!("{}.ready", self.role))?;
                self.ready = true;
            }
            if !self.ready(&format!("{other_role}.ready")) {
                return Ok(false);
            }
            if self.role == "owner" {
                let base = game::CsSkill {
                    skill_id: 3212,
                    caster: self.own,
                    target: self.own,
                    skill_level: 1,
                    ..Default::default()
                };
                for r in [
                    game::CsSkill {
                        caster: self.other,
                        ..base.clone()
                    },
                    game::CsSkill {
                        target: self.other,
                        ..base.clone()
                    },
                    game::CsSkill {
                        skill_id: 65535,
                        ..base.clone()
                    },
                    base.clone(),
                    game::CsSkill {
                        skill_id: 63203,
                        ..base.clone()
                    },
                    game::CsSkill {
                        skill_level: 0,
                        ..base
                    },
                ] {
                    self.request(app, r);
                }
            } else {
                self.raw(app);
                self.request(
                    app,
                    game::CsSkill {
                        skill_id: 3212,
                        caster: self.other,
                        target: self.other,
                        skill_level: 1,
                        ..Default::default()
                    },
                );
            }
            self.stage = 1;
            return Ok(false);
        }
        if self.role == "witness" {
            match self.stage {
                1 => {
                    if trace.cast_results.len() == 2 && self.ready("owner.reentered") {
                        let Some(peer) = w
                            .visible_player(&self.peer)
                            .filter(|p| p.handle != self.initial_other)
                        else {
                            return Ok(false);
                        };
                        self.other = peer.handle;
                        self.record(app, "ReenteredPeer");
                        app.send(NetCmd::Raw(
                            game::Handle::<{ id::TS_CS_QUERY }>(self.other).to_packet(),
                        ));
                        self.stage = 2;
                    }
                }
                2 => {
                    if trace
                        .enters
                        .iter()
                        .filter(|e| e["handle"] == self.other)
                        .count()
                        >= 2
                    {
                        ensure!(
                            known.is_empty() && catalog.jp == 0,
                            "foreign private data leaked"
                        );
                        if let Some(cmd) = app.world.as_mut().unwrap().select_target(self.other) {
                            app.send(cmd);
                        }
                        self.record(app, "FreshQuery");
                        self.mark("witness.queried")?;
                        self.stage = 3;
                    }
                }
                3 => {
                    if self.ready("owner.done") && phases("FIRE") == 2 && phases("CANCEL") == 2 {
                        self.record(app, "Observed");
                        self.capture("witness")?;
                        self.mark("witness.done")?;
                        self.stage = 4;
                    }
                }
                4 => {
                    if trace.leaves.contains(&self.other) && w.unit(self.other).is_none() {
                        self.record(app, "PeerLogout");
                        app.send(NetCmd::ReturnLobby);
                        self.stage = 14;
                    }
                }
                _ => {}
            }
            return Ok(false);
        }
        match self.stage {
            1 => {
                if trace.cast_results.len() == 6 {
                    ensure!(
                        trace.cast_results.iter().all(|r| *r != 0),
                        "invalid cast accepted"
                    );
                    self.record(app, "Refused");
                    let command = w.learn_skill(3212, 1).context("learning offer")?;
                    app.send(command);
                    self.stage = 2;
                }
            }
            2 => {
                if known.get(&3212) == Some(&1) && catalog.jp == 0 {
                    ensure!(
                        !catalog
                            .entries
                            .iter()
                            .find(|e| e.id == 3212)
                            .context("skill offer")?
                            .passive,
                        "active flag inverted"
                    );
                    app.world.as_mut().unwrap().focus_skill(3212);
                    self.record(app, "Learned");
                    self.cast(app)?;
                    self.stage = 3;
                }
            }
            3 => {
                if phases("CASTING") == 1 && w.is_casting() {
                    self.record(app, "CastingCancel");
                    app.send(w.cancel_skill());
                    self.stage = 4;
                }
            }
            4 => {
                if phases("CANCEL") == 1 && !w.is_casting() {
                    self.record(app, "Canceled");
                    self.cast(app)?;
                    self.stage = 5;
                }
            }
            5 => {
                if phases("FIRE") == 1
                    && w.addon_state()
                        .player
                        .as_ref()
                        .is_some_and(|p| p.mana == Some(50))
                    && w.skill_remaining(3212) > 0.
                {
                    self.record(app, "FirstFire");
                    self.capture("owner-restored")?;
                    self.raw(app);
                    self.raw(app);
                    self.stage = 6;
                }
            }
            6 => {
                if trace.cast_results.len() == 10 {
                    app.send(NetCmd::ReturnLobby);
                    self.stage = 7;
                }
            }
            8 => {
                if known.get(&3212) == Some(&1) && catalog.jp == 0 {
                    self.own = w.addon_state().player.context("reentered player")?.handle;
                    ensure!(self.own != self.initial, "fresh handle required");
                    ensure!(w.skill_remaining(3212) > 0., "re-entry lost cooldown");
                    app.world.as_mut().unwrap().focus_skill(3212);
                    self.record(app, "Reentered");
                    self.raw(app);
                    self.mark("owner.reentered")?;
                    self.stage = 9;
                }
            }
            9 => {
                if trace.cast_results.len() == 11
                    && self.ready("witness.queried")
                    && !self.directory.join("owner-reentered.png").exists()
                {
                    self.capture("owner-reentered")?;
                }
                if trace.cast_results.len() == 11
                    && self.ready("witness.queried")
                    && w.skill_remaining(3212) <= 0.
                {
                    self.record(app, "CooldownExpired");
                    self.cast(app)?;
                    self.stage = 10;
                }
            }
            10 => {
                if phases("CASTING") == 3 && w.is_casting() {
                    let (x, y) = w.unit_position(self.own).context("position")?;
                    let command = app
                        .world
                        .as_mut()
                        .unwrap()
                        .move_path(vec![MovePoint { x, y }])
                        .context("normal move")?;
                    self.record(app, "CastingMove");
                    app.send(command);
                    self.stage = 11;
                }
            }
            11 => {
                if phases("CANCEL") == 2 && !w.is_casting() && !trace.moves.is_empty() {
                    self.record(app, "MoveCanceled");
                    self.cast(app)?;
                    self.stage = 12;
                }
            }
            12 => {
                if phases("FIRE") == 2
                    && w.addon_state()
                        .player
                        .as_ref()
                        .is_some_and(|p| p.mana == Some(50))
                {
                    self.record(app, "SecondFire");
                    self.mark("owner.done")?;
                    self.stage = 13;
                }
            }
            13 => {
                if self.ready("witness.done") {
                    app.send(NetCmd::ReturnLobby);
                    self.stage = 14;
                }
            }
            _ => {}
        }
        Ok(false)
    }
    pub fn fail(&mut self, error: anyhow::Error) {
        self.failure = Some(format!("{error:#}"));
        let _ = self.capture(&format!("{}-failure", self.role));
    }
    pub fn finish(&self, app: &App) -> Result<bool> {
        let success = self.stage == 15 && self.failure.is_none();
        mark(
            &self.directory,
            &format!("{}.json", self.role),
            json!({"scenario":"casting","role":self.role,"success":success,"failure":self.failure,"own":self.own,"initial":self.initial,"other":self.other,"initial_other":self.initial_other,"requests":self.requests,"steps":self.steps,"events":self.events,"messages":self.messages,"trace":app.combat_trace.as_ref().context("trace")?.json()}),
        )?;
        Ok(success)
    }
}
