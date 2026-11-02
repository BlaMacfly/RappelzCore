//! Explicit offline asset comparison. No accounts, network commands or production installation.
use crate::App;
use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub fn parse_args(args: &[String]) -> Result<Option<PathBuf>> {
    const FLAG: &str = "-validate-terrain-assets";
    let Some(i) = args.iter().position(|s| s == FLAG) else {
        return Ok(None);
    };
    ensure!(
        args.iter().filter(|s| s.as_str() == FLAG).count() == 1,
        "choose one terrain asset validation directory"
    );
    let path = args
        .get(i + 1)
        .filter(|s| !s.starts_with('-'))
        .context("terrain asset output directory required")?;
    ensure!(
        args.iter().any(|s| s == "-demo"),
        "terrain asset validation requires -demo"
    );
    ensure!(
        !args
            .iter()
            .any(|s| matches!(s.as_str(), "-login" | "-enter" | "-screenshot")),
        "asset preview cannot log in or use another screenshot driver"
    );
    ensure!(
        !args
            .iter()
            .any(|s| s.starts_with("-validate-") && s != FLAG),
        "choose one validation scenario"
    );
    Ok(Some(PathBuf::from(path)))
}

pub struct TerrainPreview {
    out: PathBuf,
    started: Instant,
    phase: usize,
    configured: bool,
    settled: Option<Instant>,
    requested: bool,
    captured: bool,
    steps: Vec<Value>,
    failure: Option<String>,
}
impl TerrainPreview {
    pub fn new(out: PathBuf) -> Result<Self> {
        std::fs::create_dir(&out).context("asset preview requires a new output directory")?;
        Ok(Self {
            out,
            started: Instant::now(),
            phase: 0,
            configured: false,
            settled: None,
            requested: false,
            captured: false,
            steps: Vec::new(),
            failure: None,
        })
    }
    pub fn before_draw(&mut self, app: &mut App) {
        if !self.configured {
            if let Some(world) = &mut app.world {
                world.terrain_validation_view(false, self.phase);
                self.configured = true;
            }
        }
    }
    pub fn tick(&mut self, app: &mut App) -> Result<bool> {
        ensure!(
            self.started.elapsed() < Duration::from_secs(180),
            "terrain asset preview timeout"
        );
        ensure!(app.error.is_none(), "client error: {:?}", app.error);
        ensure!(
            app.addons.error.is_none() && app.addons.runtime.errors().is_empty(),
            "asset preview addon error"
        );
        let world = app.world.as_mut().context("offline world required")?;
        let cache = world.terrain_cache_snapshot();
        ensure!(cache["worker_error"].is_null(), "terrain worker error");
        if cache["ready"] != true {
            self.settled = None;
            return Ok(false);
        }
        ensure!(
            cache["resident"] == cache["wanted"],
            "invisible terrain retained"
        );
        ensure!(
            cache["bake_px"] == 1024,
            "normal terrain resolution required"
        );
        let since = *self.settled.get_or_insert_with(Instant::now);
        if since.elapsed() < Duration::from_millis(700) {
            return Ok(false);
        }
        let ground = self.out.join(format!("view-{}-ground.png", self.phase));
        if !self.requested {
            world.terrain_validation_capture(ground);
            self.requested = true;
            return Ok(false);
        }
        if !self.captured {
            ensure!(ground.is_file(), "ground capture missing");
            macroquad::prelude::get_screen_data().export_png(
                self.out
                    .join(format!("view-{}-scene.png", self.phase))
                    .to_str()
                    .context("UTF8 output path")?,
            );
            self.steps.push(json!({"phase":self.phase,"cache":cache,"width":macroquad::prelude::screen_width(),"height":macroquad::prelude::screen_height(),
                "frame_ms":macroquad::prelude::get_frame_time()*1000.,"elapsed_ms":self.started.elapsed().as_millis()}));
            std::fs::write(
                self.out.join(format!("phase-{}.ready", self.phase)),
                b"settled and captured",
            )?;
            self.captured = true;
            tracing::info!("Terrain asset view {} ready", self.phase);
        }
        if self.out.join(format!("release-{}", self.phase)).is_file() {
            self.phase += 1;
            if self.phase == 4 {
                return Ok(true);
            }
            self.configured = false;
            self.settled = None;
            self.requested = false;
            self.captured = false;
        }
        Ok(false)
    }
    pub fn fail(&mut self, error: anyhow::Error) {
        self.failure = Some(format!("{error:#}"));
    }
    pub fn finish(self) -> Result<()> {
        let success = self.failure.is_none() && self.phase == 4 && self.steps.len() == 4;
        let report = json!({"success":success,"offline":true,"network_login":false,"views":self.steps,"failure":self.failure,
            "experimental_direct_tiles":std::env::args().any(|arg|arg=="-terrain-direct-tiles"),
            "input_locked":true,"normal_renderer":true,"audio_disabled":true,"elapsed_ms":self.started.elapsed().as_millis()});
        std::fs::write(
            self.out.join("report.json"),
            serde_json::to_vec_pretty(&report)?,
        )?;
        ensure!(success, "terrain asset preview did not complete");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(s: &[&str]) -> Result<Option<PathBuf>> {
        parse_args(&s.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }
    #[test]
    fn preview_is_explicit_offline_and_cannot_share_network_or_capture_drivers() {
        assert_eq!(
            parse(&["client", "-demo", "-validate-terrain-assets", "output"]).unwrap(),
            Some("output".into())
        );
        assert!(parse(&["client", "-login", "example"]).unwrap().is_none());
        for extra in ["-login", "-enter", "-screenshot", "-validate-terrain"] {
            assert!(parse(&[
                "client",
                "-demo",
                "-validate-terrain-assets",
                "out",
                extra,
                "x"
            ])
            .is_err());
        }
        assert!(parse(&["client", "-validate-terrain-assets", "out"]).is_err());
        assert!(parse(&["client", "-demo", "-validate-terrain-assets"]).is_err());
        assert!(parse(&["client", "-demo", "-validate-terrain-assets", "-demo"]).is_err());
        assert!(parse(&[
            "client",
            "-demo",
            "-validate-terrain-assets",
            "out",
            "-validate-terrain-assets",
            "out2"
        ])
        .is_err());
    }
}
