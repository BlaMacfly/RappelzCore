//! Opt-in native-client regression run against a local server and a dedicated Lua test character.
//! This uses real network commands and the production world/addon adapter, never synthetic events.
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{bail, ensure, Context, Result};
use rz_addon::GameState;
use serde_json::{json, Value};

use crate::{App, NetCmd};

mod combat;
pub use combat::PacketTrace;

pub enum OnlineValidation {
    Addons(AddonValidation),
    Combat(combat::CombatValidation),
}

impl OnlineValidation {
    pub fn new(report: PathBuf, app: &App) -> Result<Self> {
        Ok(Self::Addons(AddonValidation::new(report, app)?))
    }
    pub fn combat(plan: PathBuf, app: &App) -> Result<Self> {
        Ok(Self::Combat(combat::CombatValidation::new(plan, app)?))
    }
    pub fn message(&mut self, message: &str) {
        match self {
            Self::Addons(v) => v.message(message),
            Self::Combat(v) => v.message(message),
        }
    }
    pub fn tick(&mut self, app: &mut App) -> Result<bool> {
        match self {
            Self::Addons(v) => v.tick(app),
            Self::Combat(v) => v.tick(app),
        }
    }
    pub fn fail(&mut self, error: anyhow::Error) {
        match self {
            Self::Addons(v) => v.fail(error),
            Self::Combat(v) => v.fail(error),
        }
    }
    pub fn finish(&self, app: &App) -> Result<bool> {
        match self {
            Self::Addons(v) => v.finish(app),
            Self::Combat(v) => v.finish(app),
        }
    }
}

fn guard(app: &App) -> Result<()> {
    let (host, _) = crate::config::realmlist(&app.root, &app.config.get("locale", "frFR"));
    ensure!(
        host == "127.0.0.1" || host == "localhost",
        "online validation requires a loopback server"
    );
    ensure!(
        app.account.to_ascii_uppercase().starts_with("LUA")
            && app
                .auto_enter
                .as_ref()
                .is_some_and(|c| c.starts_with("Lua")),
        "online validation requires a dedicated LUA* account and Lua* character via -login/-enter"
    );
    Ok(())
}

#[derive(Debug, PartialEq)]
enum Step {
    Login,
    Level,
    Money,
    Bag,
    Target,
    Reload,
    Logout,
    Done,
}

pub struct AddonValidation {
    report: PathBuf,
    step: Step,
    since: Instant,
    events: BTreeMap<String, usize>,
    steps: Vec<Value>,
    expected_level: i32,
    expected_hp: i32,
    expected_mp: i32,
    expected_money: i64,
    bag_revision: u64,
    clicks: u32,
    logins: usize,
    failure: Option<String>,
}

impl AddonValidation {
    pub fn new(report: PathBuf, app: &App) -> Result<Self> {
        guard(app)?;
        Ok(Self {
            report,
            step: Step::Login,
            since: Instant::now(),
            events: BTreeMap::new(),
            steps: Vec::new(),
            expected_level: 0,
            expected_hp: 0,
            expected_mp: 0,
            expected_money: 0,
            bag_revision: 0,
            clicks: 0,
            logins: 0,
            failure: None,
        })
    }

    pub fn message(&mut self, message: &str) {
        if let Some(rest) = message.strip_prefix("[RappelzDebug] EVENT ") {
            if let Some(event) = rest.split_whitespace().next() {
                *self.events.entry(event.into()).or_default() += 1;
            }
        }
    }

    fn count(&self, event: &str) -> usize {
        self.events.get(event).copied().unwrap_or(0)
    }
    fn advance(&mut self, step: Step) {
        self.step = step;
        self.since = Instant::now();
    }
    fn chat(app: &App, message: String) {
        app.send(NetCmd::Chat {
            chat_type: 0,
            target: String::new(),
            message,
        });
    }
    fn display(app: &App) -> String {
        app.addons
            .runtime
            .draw_widgets(1280.0, 720.0)
            .into_iter()
            .find(|w| w.widget.name.as_deref() == Some("RappelzDebugInfo"))
            .map(|w| w.widget.text)
            .unwrap_or_default()
    }
    fn clicks(app: &App) -> Result<u32> {
        Self::display(app)
            .lines()
            .find_map(|l| l.strip_prefix("Clicks: "))
            .and_then(|l| l.split_whitespace().next())
            .context("RappelzDebug click label missing")?
            .parse()
            .context("click count")
    }
    fn snapshot(&mut self, app: &App, state: &GameState) {
        self.steps.push(json!({"step":format!("{:?}",self.step), "player":state.player.as_ref().map(|p|
            json!({"name":p.name,"level":p.level,"hp":p.health,"max_hp":p.max_health,"mp":p.mana,"max_mp":p.max_mana})),
            "money":state.money,"bag_revision":state.bag_revision,"target":state.target.as_ref().map(|t| &t.name),
            "display":Self::display(app),"events":self.events}));
    }

    pub fn tick(&mut self, app: &mut App) -> Result<bool> {
        ensure!(
            app.addons.runtime.errors().is_empty(),
            "addon errors: {:?}",
            app.addons.runtime.errors()
        );
        ensure!(
            app.addons.error.is_none(),
            "addon host error: {:?}",
            app.addons.error
        );
        ensure!(app.error.is_none(), "client/network error: {:?}", app.error);
        let state = app
            .world
            .as_ref()
            .map(crate::world::WorldState::addon_state)
            .unwrap_or_default();
        if self.since.elapsed() > Duration::from_secs(if self.step == Step::Login { 25 } else { 8 })
        {
            self.snapshot(app, &state);
            bail!("timeout in {:?}: {:?}", self.step, state);
        }
        match self.step {
            Step::Login => {
                if app.loading.is_some() {
                    return Ok(false);
                }
                if let Some(player) = &state.player {
                    if let (Some(level), Some(hp), Some(mp), Some(money)) = (
                        player.level,
                        player.max_health,
                        player.max_mana,
                        state.money,
                    ) {
                        if self.count("PLAYER_ENTERING_WORLD") == 0
                            || self.count("CHAT_MSG_SYSTEM") == 0
                        {
                            return Ok(false);
                        }
                        ensure!(
                            level < 254 && money < u32::MAX as i64 - 37,
                            "test character level/currency too high"
                        );
                        ensure!(
                            Self::display(app).contains(&format!("Level: {level}")),
                            "initial level not displayed"
                        );
                        self.snapshot(app, &state);
                        self.expected_level = level + 1;
                        self.expected_hp = hp + 20;
                        self.expected_mp = mp + 10;
                        self.expected_money = money + 37;
                        app.auto_enter = None;
                        Self::chat(app, format!(".level {}", self.expected_level));
                        self.advance(Step::Level);
                    }
                }
            }
            Step::Level => {
                if state.player.as_ref().is_some_and(|p| {
                    p.level == Some(self.expected_level)
                        && p.health == Some(self.expected_hp)
                        && p.max_health == Some(self.expected_hp)
                        && p.mana == Some(self.expected_mp)
                        && p.max_mana == Some(self.expected_mp)
                }) {
                    ensure!(self.count("PLAYER_LEVEL_UP") > 0, "missing PLAYER_LEVEL_UP");
                    ensure!(
                        Self::display(app)
                            .contains(&format!("HP: {} / {}", self.expected_hp, self.expected_hp)),
                        "HP UI did not refresh"
                    );
                    self.snapshot(app, &state);
                    Self::chat(app, ".money 37".into());
                    self.advance(Step::Money);
                }
            }
            Step::Money => {
                if state.money == Some(self.expected_money) {
                    ensure!(
                        Self::display(app).contains(&format!("Money: {}", self.expected_money)),
                        "currency UI did not refresh"
                    );
                    self.snapshot(app, &state);
                    self.bag_revision = state.bag_revision;
                    Self::chat(app, ".additem 244201 1".into());
                    self.advance(Step::Bag);
                }
            }
            Step::Bag => {
                if state.bag_revision > self.bag_revision {
                    ensure!(self.count("BAG_UPDATE") > 0, "missing BAG_UPDATE");
                    self.snapshot(app, &state);
                    let world = app.world.as_mut().context("world missing")?;
                    let handle = world
                        .nearby_target()
                        .context("no server-provided target nearby")?;
                    let command = world.select_target(handle).context("target disappeared")?;
                    app.send(command);
                    app.sync_addons();
                    self.advance(Step::Target);
                }
            }
            Step::Target => {
                if let Some(target) = &state.target {
                    ensure!(
                        Self::display(app).contains(&target.name),
                        "selected target UI did not refresh"
                    );
                    self.snapshot(app, &state);
                    self.clicks = Self::clicks(app)?;
                    let button = app
                        .addons
                        .runtime
                        .draw_widgets(1280.0, 720.0)
                        .into_iter()
                        .find(|w| w.widget.name.as_deref() == Some("RappelzDebugButton"))
                        .context("debug button missing")?;
                    app.addons.runtime.pointer(
                        1280.0,
                        720.0,
                        button.rect.x + 5.0,
                        button.rect.y + 5.0,
                        true,
                    );
                    ensure!(
                        Self::clicks(app)? == self.clicks + 1,
                        "button did not increment"
                    );
                    self.logins = self.count("PLAYER_LOGIN");
                    app.addons.reload();
                    self.advance(Step::Reload);
                }
            }
            Step::Reload => {
                if self.count("PLAYER_LOGIN") > self.logins {
                    ensure!(
                        Self::clicks(app)? == self.clicks + 1,
                        "saved click count lost on reload"
                    );
                    ensure!(
                        state
                            .player
                            .as_ref()
                            .is_some_and(|p| p.level == Some(self.expected_level)),
                        "world changed on reload"
                    );
                    self.snapshot(app, &state);
                    let image = self.report.with_extension("png");
                    macroquad::prelude::get_screen_data()
                        .export_png(image.to_str().context("screenshot path must be UTF-8")?);
                    app.send(NetCmd::ReturnLobby);
                    self.advance(Step::Logout);
                }
            }
            Step::Logout => {
                if state.player.is_none() && self.count("PLAYER_LOGOUT") > 0 {
                    for event in [
                        "ADDON_LOADED",
                        "PLAYER_LOGIN",
                        "PLAYER_ENTERING_WORLD",
                        "PLAYER_LOGOUT",
                        "UNIT_HEALTH",
                        "UNIT_MANA",
                        "PLAYER_LEVEL_UP",
                        "PLAYER_MONEY",
                        "BAG_UPDATE",
                        "PLAYER_TARGET_CHANGED",
                        "CHAT_MSG_SYSTEM",
                    ] {
                        ensure!(
                            self.count(event) > 0,
                            "event never observed in RappelzDebug: {event}"
                        );
                    }
                    self.snapshot(app, &state);
                    self.advance(Step::Done);
                    return Ok(true);
                }
            }
            Step::Done => return Ok(true),
        }
        Ok(false)
    }

    pub fn fail(&mut self, error: anyhow::Error) {
        self.failure = Some(format!("{error:#}"));
    }
    pub fn finish(&self, app: &App) -> Result<bool> {
        let success = self.step == Step::Done && self.failure.is_none();
        if let Some(parent) = self.report.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(
            &self.report,
            serde_json::to_vec_pretty(&json!({"success":success,"failure":self.failure,
            "account":app.account,"events":self.events,"steps":self.steps,"addon_errors":app.addons.runtime.errors()}))?,
        )?;
        tracing::info!(
            "Online addon validation: success={success}, report={}",
            self.report.display()
        );
        Ok(success)
    }
}
