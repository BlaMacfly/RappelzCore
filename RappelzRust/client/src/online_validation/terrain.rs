//! Loopback-only terrain cache comparison, using ordinary movement and independent clients.
use crate::{App, NetCmd};
use anyhow::{ensure, Context, Result};
use rz_proto::msg::MovePoint;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub struct TerrainValidation {
    directory: PathBuf,
    role: String,
    peer: String,
    reference: bool,
    phase: usize,
    configured: bool,
    started: Instant,
    settled: Option<Instant>,
    captured: bool,
    recorded: bool,
    logout: bool,
    done: bool,
    events: BTreeMap<String, usize>,
    steps: Vec<Value>,
    frames: Vec<f32>,
    failure: Option<String>,
    motion: bool,
    pixel_cache: bool,
    motion_frame: usize,
    motion_view: usize,
    motion_frames: Vec<Value>,
    cold: bool,
    async_bake: bool,
}

impl TerrainValidation {
    pub(super) fn before_draw(&self, app: &mut App) {
        if let Some(world) = &mut app.world {
            world.terrain_validation_policy(self.reference, self.pixel_cache, self.async_bake);
        }
    }
    pub fn new(plan: PathBuf, app: &App) -> Result<Self> {
        super::guard(app)?;
        let p: Value = serde_json::from_slice(&std::fs::read(plan)?)?;
        let role = p["role"].as_str().context("terrain role")?.to_owned();
        ensure!(
            role == "owner" || role == "witness",
            "terrain role must be owner or witness"
        );
        let peer = p["peer"].as_str().context("terrain peer")?.to_owned();
        ensure!(peer.starts_with("Lua"), "dedicated terrain peer required");
        let reference = match p["renderer"].as_str() {
            Some("reference") => true,
            Some("optimized") => false,
            _ => anyhow::bail!("renderer must be reference or optimized"),
        };
        let directory = PathBuf::from(p["directory"].as_str().context("terrain directory")?);
        let motion = p["motion"].as_bool().unwrap_or(false);
        let cold = p["cold"].as_bool().unwrap_or(false);
        ensure!(!cold || motion, "cold validation requires motion sampling");
        let async_bake = match p["async_bake"].as_str() {
            None => !motion || cold,
            Some("enabled") => true,
            Some("disabled") => false,
            _ => anyhow::bail!("async_bake must be enabled or disabled"),
        };
        let pixel_cache = match p["pixel_cache"].as_str() {
            None | Some("enabled") => true,
            Some("disabled") if motion => false,
            _ => anyhow::bail!("pixel_cache must be enabled or disabled for the motion comparison"),
        };
        std::fs::create_dir_all(&directory)?;
        ensure!(
            !directory.join(format!("{role}.json")).exists(),
            "fresh terrain attempt required"
        );
        Ok(Self {
            directory,
            role,
            peer,
            reference,
            phase: 0,
            configured: false,
            started: Instant::now(),
            settled: None,
            captured: false,
            recorded: false,
            logout: false,
            done: false,
            events: BTreeMap::new(),
            steps: Vec::new(),
            frames: Vec::new(),
            failure: None,
            motion,
            pixel_cache,
            motion_frame: 0,
            motion_view: 0,
            motion_frames: Vec::new(),
            cold,
            async_bake,
        })
    }

    fn write(&self, name: &str, value: Value) -> Result<()> {
        std::fs::write(
            self.directory.join(name),
            serde_json::to_vec_pretty(&value)?,
        )?;
        Ok(())
    }
    fn marker(&self, role: &str, phase: usize) -> PathBuf {
        self.directory.join(format!("{role}-{phase}.ready"))
    }
    fn ground(&self) -> PathBuf {
        self.directory
            .join(format!("{}-{}.ground.png", self.role, self.phase))
    }
    pub fn message(&mut self, message: &str) {
        if let Some(event) = message
            .strip_prefix("[RappelzDebug] EVENT ")
            .and_then(|m| m.split_whitespace().next())
        {
            *self.events.entry(event.to_owned()).or_default() += 1;
        }
    }

    pub fn tick(&mut self, app: &mut App) -> Result<bool> {
        ensure!(
            self.started.elapsed() < Duration::from_secs(180),
            "terrain scenario exceeded 180 seconds"
        );
        ensure!(app.error.is_none(), "terrain client error: {:?}", app.error);
        ensure!(
            app.addons.error.is_none() && app.addons.runtime.errors().is_empty(),
            "terrain addon error"
        );
        let other = if self.role == "owner" {
            "witness"
        } else {
            "owner"
        };
        let peer_report = self.directory.join(format!("{other}.json"));
        if let Ok(bytes) = std::fs::read(peer_report) {
            let p: Value = serde_json::from_slice(&bytes)?;
            ensure!(
                p["success"] != false,
                "terrain peer failed: {}",
                p["failure"]
            );
        }
        if self.logout {
            if app.world.is_none() && self.events.get("PLAYER_LOGOUT") == Some(&1) {
                self.done = true;
                return Ok(true);
            }
            return Ok(false);
        }
        if app.loading.is_some() {
            return Ok(false);
        }
        let Some(world) = app.world.as_ref() else {
            return Ok(false);
        };
        let Some(me) = world.addon_state().player else {
            return Ok(false);
        };
        let Some(peer) = world.visible_player(&self.peer) else {
            return Ok(false);
        };
        let y = if self.role == "owner" { 7227. } else { 7247. };
        let peer_y = if self.role == "owner" { 7247. } else { 7227. };
        if !self.configured {
            ensure!(
                me.level == Some(1) && me.health == Some(100) && me.mana == Some(50),
                "full-vitals level-1 fixture required"
            );
            ensure!(
                world.experience() == (0, 0) && world.layer() == 0,
                "fresh terrain fixture required"
            );
            app.auto_enter = None;
            let world = app.world.as_mut().unwrap();
            world.terrain_validation_view(
                self.reference,
                if self.motion && self.phase == 2 {
                    0
                } else {
                    self.phase
                },
            );
            world.terrain_validation_pixel_cache(self.pixel_cache);
            world.terrain_validation_async(self.async_bake);
            self.motion_frame = 0;
            self.motion_view = 0;
            if self.phase == 4 || self.phase == 5 {
                let x = if self.phase == 4 { 10540. } else { 8440. };
                let command = world
                    .move_path(vec![MovePoint { x, y }])
                    .context("terrain route")?;
                app.send(command);
            }
            self.configured = true;
            return Ok(false);
        }
        if self.motion && !self.recorded && (self.cold || matches!(self.phase, 2 | 4 | 5)) {
            let world = app.world.as_ref().unwrap();
            let sample = world.terrain_cache_snapshot();
            let position = world.unit_position(me.handle);
            if self.cold || self.phase != 2 || self.motion_frame < 96 {
                self.motion_frames
                    .push(json!({"phase":self.phase,"frame":self.motion_frame,
                    "view":self.motion_view,"cache":sample,"position":position,
                    "frame_ms":macroquad::prelude::get_frame_time()*1000.}));
                ensure!(
                    self.motion_frames.len() < 12000,
                    "bounded terrain motion samples"
                );
            }
            if !self.cold && matches!(self.phase, 2 | 4) && self.motion_frame < 96 {
                self.motion_frame += 1;
                let view = if self.motion_frame < 96 {
                    (self.motion_frame / 8) % 2
                } else {
                    0
                };
                app.world
                    .as_mut()
                    .unwrap()
                    .terrain_validation_view(false, view);
                self.motion_view = view;
                if self.phase == 2 {
                    self.settled = None;
                    return Ok(false);
                }
            } else if self.cold || self.phase != 2 {
                self.motion_frame += 1;
            }
        }
        let world = app.world.as_ref().unwrap();
        let x = if self.phase == 4 { 10540. } else { 8440. };
        let close = |p: Option<(f32, f32)>, ty: f32| {
            p.is_some_and(|(px, py)| (px - x).abs() < 0.05 && (py - ty).abs() < 0.05)
        };
        let cache = world.terrain_cache_snapshot();
        if !close(world.unit_position(me.handle), y)
            || !close(world.unit_position(peer.handle), peer_y)
            || cache["ready"] != true
            || cache["reference"] != self.reference
        {
            self.settled = None;
            return Ok(false);
        }
        if !self.reference {
            ensure!(
                cache["resident"] == cache["wanted"],
                "terrain retained invisible textures"
            );
        }
        self.frames
            .push(macroquad::prelude::get_frame_time() * 1000.);
        ensure!(self.frames.len() < 20000, "bounded terrain frame samples");
        let settled = *self.settled.get_or_insert_with(Instant::now);
        if settled.elapsed() < Duration::from_millis(1200) {
            return Ok(false);
        }
        if !self.captured {
            let path = self.ground();
            app.world.as_mut().unwrap().terrain_validation_capture(path);
            self.captured = true;
            return Ok(false);
        }
        if !self.ground().exists() {
            return Ok(false);
        }
        if !self.recorded {
            let world = app.world.as_ref().unwrap();
            self.steps.push(json!({"phase":self.phase,"cache":cache,"own":me.handle,"peer":peer.handle,
                "position":world.unit_position(me.handle),"peer_position":world.unit_position(peer.handle),
                "level":me.level,"hp":me.health,"mp":me.mana,"xp_jp":world.experience(),
                "layer":world.layer(),"money":world.addon_state().money,"events":self.events}));
            macroquad::prelude::get_screen_data().export_png(
                self.directory
                    .join(format!("{}-{}.png", self.role, self.phase))
                    .to_str()
                    .context("terrain capture path")?,
            );
            self.write(
                &format!("{}-{}.ready", self.role, self.phase),
                json!({"phase":self.phase,"cache":cache,"position":world.unit_position(me.handle)}),
            )?;
            self.recorded = true;
        }
        // The external runner samples Windows' GPU process counters while both clients hold.
        if self.marker(other, self.phase).exists()
            && self
                .directory
                .join(format!("release-{}", self.phase))
                .exists()
        {
            if self.phase == 5 {
                app.send(NetCmd::ReturnLobby);
                self.logout = true;
            } else {
                self.phase += 1;
                self.configured = false;
                self.settled = None;
                self.captured = false;
                self.recorded = false;
            }
        }
        Ok(false)
    }

    pub fn fail(&mut self, error: anyhow::Error) {
        self.failure = Some(format!("{error:#}"));
    }
    pub fn finish(&self, app: &App) -> Result<bool> {
        let success = self.done && self.failure.is_none() && self.steps.len() == 6;
        self.write(&format!("{}.json",self.role), json!({"scenario":if self.cold {"terrain-cold"}else if self.motion {"terrain-motion"}else{"terrain"},"role":self.role,
            "renderer":if self.reference {"reference"} else {"optimized"},"success":success,"failure":self.failure,
            "steps":self.steps,"events":self.events,"frame_ms":self.frames,
            "motion":self.motion,"pixel_cache_enabled":self.pixel_cache,"motion_frames":self.motion_frames,
            "cold":self.cold,"async_bake":self.async_bake,
            "trace":app.combat_trace.as_ref().context("terrain trace")?.json()}))?;
        Ok(success)
    }
}
