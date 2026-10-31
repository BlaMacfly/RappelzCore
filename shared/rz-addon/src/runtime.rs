use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Instant;

use anyhow::{ensure, Context, Result};
use mlua::{
    ChunkMode, Function, HookTriggers, Lua, LuaOptions, MultiValue, StdLib, Table, Value, VmState,
};

use crate::manifest::{self, Manifest};
use crate::ui::{self, DrawWidget, Widgets};
use crate::{saved, xml};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UnitState {
    pub handle: u32,
    pub name: String,
    pub level: Option<i32>,
    pub layer: Option<u8>,
    pub health: Option<i32>,
    pub max_health: Option<i32>,
    pub mana: Option<i32>,
    pub max_mana: Option<i32>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SkillEventState {
    pub sequence:u64,pub skill:u16,pub phase:String,pub caster:u32,pub amount:i32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GameState {
    pub player: Option<UnitState>,
    pub target: Option<UnitState>,
    pub money: Option<i64>,
    pub party_members: Option<u32>,
    /// Accepted roster excluding the player, in stable server order. None means unknown.
    pub party: Option<Vec<UnitState>>,
    pub party_leader: Option<u32>,
    pub party_invite: Option<String>,
    pub learned_skills: Option<BTreeMap<u32, u8>>,
    pub job_points: Option<i64>,
    pub skill_event:Option<SkillEventState>,
    /// Incremented by the client only after a changed inventory is accepted.
    pub bag_revision: u64,
}

#[derive(Clone, Debug)]
pub enum EventArg {
    String(String),
    Number(f64),
}
impl From<&str> for EventArg {
    fn from(s: &str) -> Self {
        Self::String(s.into())
    }
}
impl From<String> for EventArg {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

#[derive(Clone, Debug)]
pub struct Event {
    pub name: String,
    pub args: Vec<EventArg>,
}
impl Event {
    pub fn new(name: &str, args: Vec<EventArg>) -> Self {
        Self {
            name: name.into(),
            args,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AddonInfo {
    pub manifest: Manifest,
    pub enabled: bool,
    pub status: String,
    pub error: Option<String>,
}

struct Instance {
    lua: Lua,
    widgets: Widgets,
    budget: Rc<Cell<u32>>,
    info_index: usize,
    failed: bool,
}

impl Instance {
    fn reset_budget(&self) {
        self.budget.set(200_000);
    }

    fn call(&self, id: usize, script: &str, event: Option<&Event>) -> Result<()> {
        let frames: Table = self.lua.named_registry_value(ui::FRAMES)?;
        let frame: Table = frames.raw_get(id + 1)?;
        let scripts: Table = frame.raw_get("__scripts")?;
        if let Some(callback) = scripts.raw_get::<Option<Function>>(script)? {
            let mut args = MultiValue::new();
            args.push_back(Value::Table(frame));
            if let Some(event) = event {
                args.push_back(Value::String(self.lua.create_string(&event.name)?));
                for arg in &event.args {
                    args.push_back(match arg {
                        EventArg::String(s) => Value::String(self.lua.create_string(s)?),
                        EventArg::Number(n) => Value::Number(*n),
                    });
                }
            } else if script == "OnClick" {
                args.push_back(Value::String(self.lua.create_string("LeftButton")?));
            }
            callback.call::<()>(args)?;
        }
        Ok(())
    }
}

pub struct AddonRuntime {
    root: PathBuf,
    account: String,
    realm: String,
    character: Option<String>,
    state: Rc<RefCell<GameState>>,
    start: Instant,
    infos: Vec<AddonInfo>,
    instances: Vec<Instance>,
    enabled: BTreeMap<String, bool>,
    requested: BTreeSet<String>,
    errors: Vec<String>,
    messages: Rc<RefCell<Vec<String>>>,
    hovered: Option<(usize, usize)>,
}

impl AddonRuntime {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().into(),
            account: "offline".into(),
            realm: "offline".into(),
            character: None,
            state: Default::default(),
            start: Instant::now(),
            infos: Vec::new(),
            instances: Vec::new(),
            enabled: BTreeMap::new(),
            requested: BTreeSet::new(),
            errors: Vec::new(),
            messages: Default::default(),
            hovered: None,
        }
    }

    pub fn addons(&self) -> &[AddonInfo] {
        &self.infos
    }
    pub fn errors(&self) -> &[String] {
        &self.errors
    }
    pub fn drain_messages(&mut self) -> Vec<String> {
        std::mem::take(&mut *self.messages.borrow_mut())
    }

    fn report(&mut self, message: String) {
        if self.errors.len() == 128 {
            self.errors.remove(0);
        }
        self.errors.push(message.chars().take(8192).collect());
    }

    /// Changing identity saves the old scope before any new character data is restored.
    pub fn set_identity(
        &mut self,
        account: &str,
        realm: &str,
        character: Option<&str>,
    ) -> Result<()> {
        ensure!(
            account.len() <= 128 && realm.len() <= 128 && character.is_none_or(|s| s.len() <= 128),
            "addon identity too long"
        );
        if self.account == account && self.realm == realm && self.character.as_deref() == character
        {
            return Ok(());
        }
        self.save()?;
        self.instances.clear();
        self.hovered = None;
        self.account = account.into();
        self.realm = realm.into();
        self.character = character.map(str::to_owned);
        let path = self.account_path().join("AddOns.json");
        self.enabled = if path.exists() {
            saved::read_json(&path)?
        } else {
            BTreeMap::new()
        };
        self.load()
    }

    fn account_path(&self) -> PathBuf {
        saved::account_dir(&self.root, &self.account)
    }
    fn variable_paths(&self, name: &str) -> (PathBuf, Option<PathBuf>) {
        (
            self.account_path().join(format!("{name}.json")),
            self.character.as_ref().map(|c| {
                saved::character_dir(&self.root, &self.account, &self.realm, c)
                    .join(format!("{name}.json"))
            }),
        )
    }

    pub fn load(&mut self) -> Result<()> {
        ensure!(
            self.instances.is_empty(),
            "addons already loaded; use reload()"
        );
        let preferences = self.account_path().join("AddOns.json");
        if preferences.exists() {
            self.enabled = saved::read_json(&preferences)?;
        }
        self.infos.clear();
        let root = self.root.join("Interface").join("AddOns");
        if !root.exists() {
            return Ok(());
        }
        let root = root.canonicalize()?;
        let mut dirs = std::fs::read_dir(&root)?.collect::<std::io::Result<Vec<_>>>()?;
        dirs.sort_by_key(|d| d.file_name());
        let mut manifests = Vec::new();
        for entry in dirs {
            if !entry.file_type()?.is_dir() {
                continue;
            }
            if manifests.len() >= 64 {
                self.report("only the first 64 installed addons are discovered".into());
                break;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = entry.path();
            let result = (|| {
                ensure!(
                    path.canonicalize()?.starts_with(&root),
                    "addon directory escapes AddOns"
                );
                let source = manifest::read_source(&path, &format!("{name}.toc"))?;
                Manifest::parse(&name, path, &source)
            })();
            match result {
                Ok(m) => manifests.push(m),
                Err(e) => self.report(format!("{name}: {e:#}")),
            }
        }
        for m in &manifests {
            self.infos.push(AddonInfo {
                enabled: self.enabled.get(&m.name).copied().unwrap_or(true),
                status: "Waiting".into(),
                error: None,
                manifest: m.clone(),
            });
        }
        // Resolve each root separately so a broken addon cannot prevent unrelated addons loading.
        let mut order = Vec::new();
        for (i, m) in manifests.iter().enumerate() {
            if !self.infos[i].enabled {
                self.infos[i].status = "Disabled".into();
                continue;
            }
            if m.load_on_demand && !self.requested.contains(&m.name) {
                self.infos[i].status = "On demand".into();
                continue;
            }
            let mut selection: BTreeMap<String, bool> =
                manifests.iter().map(|m| (m.name.clone(), false)).collect();
            fn select(
                name: &str,
                ms: &[Manifest],
                enabled: &BTreeMap<String, bool>,
                selected: &mut BTreeMap<String, bool>,
            ) {
                if selected.get(name) == Some(&true) || enabled.get(name) == Some(&false) {
                    return;
                }
                selected.insert(name.into(), true);
                if let Some(m) = ms.iter().find(|m| m.name == name) {
                    for dep in m.dependencies.iter().chain(&m.optional_dependencies) {
                        select(dep, ms, enabled, selected);
                    }
                }
            }
            select(&m.name, &manifests, &self.enabled, &mut selection);
            let requested: BTreeSet<String> = selection
                .iter()
                .filter(|(_, on)| **on)
                .map(|(n, _)| n.clone())
                .collect();
            match manifest::load_order(&manifests, &selection, &requested) {
                Ok(resolved) => {
                    for j in resolved {
                        if !order.contains(&j) {
                            order.push(j);
                        }
                    }
                }
                Err(e) => {
                    let message = format!("{}: {e:#}", m.name);
                    self.infos[i].error = Some(message.clone());
                    self.infos[i].status = "Blocked".into();
                    self.report(message);
                }
            }
        }
        for i in order {
            if self.instances.len() >= 32 {
                self.report("loaded addon limit (32) reached".into());
                break;
            }
            if let Some(dep) = manifests[i].dependencies.iter().find(|dep| {
                !self
                    .instances
                    .iter()
                    .any(|a| !a.failed && self.infos[a.info_index].manifest.name == **dep)
            }) {
                self.infos[i].status = "Blocked".into();
                self.infos[i].error = Some(format!("dependency {dep} failed to load"));
                continue;
            }
            match self.create_instance(i) {
                Ok(a) => {
                    self.instances.push(a);
                    self.infos[i].status = "Loaded".into();
                    self.dispatch(Event::new(
                        "ADDON_LOADED",
                        vec![manifests[i].name.clone().into()],
                    ));
                }
                Err(e) => {
                    let message = format!("{}: {e:#}", manifests[i].name);
                    self.infos[i].status = "Failed".into();
                    self.infos[i].error = Some(message.clone());
                    self.report(message);
                }
            }
        }
        if self.state.borrow().player.is_some() {
            self.dispatch(Event::new("PLAYER_LOGIN", vec![]));
            self.dispatch(Event::new("PLAYER_ENTERING_WORLD", vec![]));
        }
        Ok(())
    }

    fn create_instance(&self, index: usize) -> Result<Instance> {
        let m = &self.infos[index].manifest;
        let lua = Lua::new_with(
            StdLib::TABLE | StdLib::STRING | StdLib::MATH,
            LuaOptions::default(),
        )?;
        lua.set_memory_limit(8 * 1024 * 1024)?;
        let globals = lua.globals();
        // pcall/xpcall can swallow a hook quota error indefinitely in Lua 5.1.
        for name in [
            "io",
            "os",
            "package",
            "debug",
            "coroutine",
            "require",
            "dofile",
            "loadfile",
            "load",
            "loadstring",
            "getfenv",
            "setfenv",
            "newproxy",
            "collectgarbage",
            "pcall",
            "xpcall",
            "module",
        ] {
            globals.raw_set(name, Value::Nil)?;
        }
        globals
            .raw_get::<Table>("string")?
            .raw_set("dump", Value::Nil)?;
        let budget = Rc::new(Cell::new(200_000u32));
        let hook_budget = budget.clone();
        lua.set_hook(
            HookTriggers::new().every_nth_instruction(1000),
            move |_, _| {
                let left = hook_budget.get();
                if left < 1000 {
                    return Err(mlua::Error::RuntimeError(
                        "addon instruction budget exceeded".into(),
                    ));
                }
                hook_budget.set(left - 1000);
                Ok(VmState::Continue)
            },
        )?;
        for name in [
            "UnitName",
            "UnitHealth",
            "UnitHealthMax",
            "UnitMana",
            "UnitManaMax",
            "UnitLevel",
            "UnitLayer",
        ] {
            let state = self.state.clone();
            globals.raw_set(
                name,
                lua.create_function(move |lua, token: String| {
                    let state = state.borrow();
                    let unit = match token.as_str() {
                        "player" => state.player.as_ref(),
                        "target" => state.target.as_ref(),
                        _ => token
                            .strip_prefix("party")
                            .and_then(|n| n.parse::<usize>().ok())
                            .filter(|n| (1..=7).contains(n))
                            .and_then(|n| state.party.as_ref()?.get(n - 1)),
                    };
                    Ok(match name {
                        "UnitName" => match unit {
                            Some(u) => Value::String(lua.create_string(&u.name)?),
                            None => Value::Nil,
                        },
                        _ => unit
                            .and_then(|u| match name {
                                "UnitHealth" => u.health,
                                "UnitHealthMax" => u.max_health,
                                "UnitMana" => u.mana,
                                "UnitManaMax" => u.max_mana,
                                "UnitLayer" => u.layer.map(i32::from),
                                _ => u.level,
                            })
                            .map_or(Value::Nil, |n| Value::Number(n as f64)),
                    })
                })?,
            )?;
        }
        let state = self.state.clone();
        globals.raw_set(
            "GetMoney",
            lua.create_function(move |_, ()| Ok(state.borrow().money))?,
        )?;
        let state = self.state.clone();
        globals.raw_set(
            "GetNumPartyMembers",
            lua.create_function(move |_, ()| Ok(state.borrow().party_members))?,
        )?;
        let state = self.state.clone();
        globals.raw_set(
            "GetPartyInviteName",
            lua.create_function(move |_, ()| Ok(state.borrow().party_invite.clone()))?,
        )?;
        let state = self.state.clone();
        globals.raw_set(
            "GetNumKnownSkills",
            lua.create_function(move |_, ()| {
                Ok(state
                    .borrow()
                    .learned_skills
                    .as_ref()
                    .map(|s| s.len() as u32))
            })?,
        )?;
        let state = self.state.clone();
        globals.raw_set(
            "GetSkillLevel",
            lua.create_function(move |_, id: u32| {
                Ok(if id == 0 {
                    None
                } else {
                    state
                        .borrow()
                        .learned_skills
                        .as_ref()
                        .map(|s| s.get(&id).copied().unwrap_or(0))
                })
            })?,
        )?;
        let state = self.state.clone();
        globals.raw_set(
            "GetJobPoints",
            lua.create_function(move |_, ()| Ok(state.borrow().job_points))?,
        )?;
        let start = self.start;
        globals.raw_set(
            "GetTime",
            lua.create_function(move |_, ()| Ok(start.elapsed().as_secs_f64()))?,
        )?;
        let output = self.messages.clone();
        let addon = m.name.clone();
        globals.raw_set(
            "print",
            lua.create_function(move |_, args: MultiValue| {
                let mut parts = Vec::new();
                for v in args.iter().take(16) {
                    parts.push(match v {
                        Value::String(s) => s.to_string_lossy(),
                        Value::Nil => "nil".into(),
                        Value::Boolean(v) => v.to_string(),
                        Value::Integer(v) => v.to_string(),
                        Value::Number(v) => v.to_string(),
                        _ => "<value>".into(),
                    });
                }
                let mut output = output.borrow_mut();
                if output.len() < 128 {
                    output.push(format!(
                        "[{addon}] {}",
                        parts.join(" ").chars().take(1024).collect::<String>()
                    ));
                }
                Ok(())
            })?,
        )?;
        let widgets: Widgets = Default::default();
        lua.set_named_registry_value(ui::FRAMES, lua.create_table()?)?;
        ui::create(
            &lua,
            widgets.clone(),
            "Frame",
            Some("UIParent".into()),
            None,
        )?;
        let state = widgets.clone();
        globals.raw_set(
            "CreateFrame",
            lua.create_function(
                move |lua,
                      (kind, name, parent, template): (
                    String,
                    Option<String>,
                    Option<Table>,
                    Option<String>,
                )| {
                    if template.is_some() {
                        return Err(mlua::Error::RuntimeError(
                            "UI templates are not supported".into(),
                        ));
                    }
                    let parent = parent.or_else(|| lua.globals().raw_get("UIParent").ok());
                    ui::create(lua, state.clone(), &kind, name, parent)
                },
            )?,
        )?;
        globals.raw_set("_ADDON_NAME", m.name.clone())?;
        let (account, character) = self.variable_paths(&m.name);
        saved::restore(&lua, &account, &m.saved_variables).context("account SavedVariables")?;
        if let Some(path) = &character {
            saved::restore(&lua, path, &m.character_variables)
                .context("character SavedVariables")?;
        }
        // One budget for all startup files and XML handlers prevents per-file quota multiplication.
        let instance = Instance {
            lua,
            widgets,
            budget,
            info_index: index,
            failed: false,
        };
        let mut source_bytes = 0usize;
        for file in &m.files {
            let source = manifest::read_source(&m.directory, file)?;
            source_bytes += source.len();
            ensure!(
                source_bytes <= 8 * 1024 * 1024,
                "addon startup sources exceed 8 MiB"
            );
            if file.to_ascii_lowercase().ends_with(".lua") {
                instance
                    .lua
                    .load(&source)
                    .set_name(format!("@{}/{file}", m.name))
                    .set_mode(ChunkMode::Text)
                    .exec()?;
            } else if file.to_ascii_lowercase().ends_with(".xml") {
                xml::load(
                    &instance.lua,
                    instance.widgets.clone(),
                    &source,
                    &format!("{}/{file}", m.name),
                )?;
            } else {
                anyhow::bail!("unsupported addon source extension: {file}");
            }
        }
        Ok(instance)
    }

    fn fail(&mut self, index: usize, error: anyhow::Error) {
        self.instances[index].failed = true;
        let info = &mut self.infos[self.instances[index].info_index];
        let message = format!("{}: {error:#}", info.manifest.name);
        info.error = Some(message.clone());
        info.status = "Failed".into();
        self.report(message);
    }

    pub fn dispatch(&mut self, event: Event) {
        for i in 0..self.instances.len() {
            if self.instances[i].failed {
                continue;
            }
            self.instances[i].reset_budget();
            let ids: Vec<_> = self.instances[i]
                .widgets
                .borrow()
                .iter()
                .filter(|w| w.events.contains(&event.name))
                .map(|w| w.id)
                .collect();
            for id in ids {
                if let Err(e) = self.instances[i].call(id, "OnEvent", Some(&event)) {
                    self.fail(i, e);
                    break;
                }
            }
        }
    }

    pub fn update_state(&mut self, state: GameState) {
        let old = self.state.replace(state.clone());
        if old.player.is_some() && state.player.is_none() {
            self.state.replace(old.clone());
            self.dispatch(Event::new("PLAYER_LOGOUT", vec![]));
            if let Err(e) = self.save() {
                self.report(format!("save on logout: {e:#}"));
            }
            self.state.replace(state);
            return;
        }
        if old.player.is_none() && state.player.is_some() {
            self.dispatch(Event::new("PLAYER_LOGIN", vec![]));
            self.dispatch(Event::new("PLAYER_ENTERING_WORLD", vec![]));
        }
        for (token, before, after) in [
            ("player", old.player.as_ref(), state.player.as_ref()),
            ("target", old.target.as_ref(), state.target.as_ref()),
        ] {
            if let Some(after) = after {
                if before.map(|u| (u.health, u.max_health)).unwrap_or_default()
                    != (after.health, after.max_health)
                {
                    self.dispatch(Event::new("UNIT_HEALTH", vec![token.into()]));
                }
                if before.map(|u| (u.mana, u.max_mana)).unwrap_or_default()
                    != (after.mana, after.max_mana)
                {
                    self.dispatch(Event::new("UNIT_MANA", vec![token.into()]));
                }
            }
        }
        if let (Some(before), Some(after)) = (&old.player, &state.player) {
            // Only a known transition of the same player is a death/revival.
            if before.handle == after.handle {
                if let (Some(a), Some(b)) = (before.layer, after.layer) {
                    if a != b {
                        self.dispatch(Event::new(
                            "RAPPELZ_LAYER_CHANGED",
                            vec![EventArg::Number(b as f64)],
                        ));
                    }
                }
                match (before.health, after.health) {
                    (Some(a), Some(b)) if a > 0 && b <= 0 => {
                        self.dispatch(Event::new("PLAYER_DEAD", vec![]))
                    }
                    (Some(a), Some(b)) if a <= 0 && b > 0 => {
                        self.dispatch(Event::new("PLAYER_ALIVE", vec![]))
                    }
                    _ => {}
                }
            }
            if let (Some(a), Some(b)) = (before.level, after.level) {
                if b > a {
                    self.dispatch(Event::new(
                        "PLAYER_LEVEL_UP",
                        vec![EventArg::Number(b as f64)],
                    ));
                }
            }
        }
        if old.target.as_ref().map(|u| u.handle) != state.target.as_ref().map(|u| u.handle) {
            self.dispatch(Event::new("PLAYER_TARGET_CHANGED", vec![]));
        }
        let roster = |s: &GameState| {
            s.party
                .as_ref()
                .map(|members| members.iter().map(|u| u.handle).collect::<Vec<_>>())
                .unwrap_or_default()
        };
        if old.party.is_none() && state.party.is_some() {
            self.dispatch(Event::new("RAPPELZ_PARTY_STATE_READY", vec![]));
        }
        if state.party.is_some()
            && (roster(&old) != roster(&state) || old.party_leader != state.party_leader)
        {
            self.dispatch(Event::new("PARTY_MEMBERS_CHANGED", vec![]));
        }
        if old.party_invite != state.party_invite {
            if let Some(name) = &state.party_invite {
                self.dispatch(Event::new(
                    "PARTY_INVITE_REQUEST",
                    vec![name.clone().into()],
                ));
            } else if old.party_invite.is_some() {
                self.dispatch(Event::new("RAPPELZ_PARTY_INVITE_CLOSED", vec![]));
            }
        }
        for (index, after) in state.party.iter().flatten().enumerate() {
            let before = old
                .party
                .as_ref()
                .and_then(|v| v.get(index))
                .filter(|u| u.handle == after.handle);
            let token = format!("party{}", index + 1);
            if before.map(|u| (u.health, u.max_health)).unwrap_or_default()
                != (after.health, after.max_health)
            {
                self.dispatch(Event::new("UNIT_HEALTH", vec![token.clone().into()]));
            }
            if before.map(|u| (u.mana, u.max_mana)).unwrap_or_default()
                != (after.mana, after.max_mana)
            {
                self.dispatch(Event::new("UNIT_MANA", vec![token.into()]));
            }
        }
        if old.bag_revision != state.bag_revision {
            self.dispatch(Event::new("BAG_UPDATE", vec![EventArg::Number(0.0)]));
        }
        if old.money != state.money {
            self.dispatch(Event::new("PLAYER_MONEY", vec![]));
        }
        if state.learned_skills.is_some() && old.learned_skills != state.learned_skills {
            self.dispatch(Event::new("SKILL_LINES_CHANGED", vec![]));
        }
        if old.job_points != state.job_points {
            if let Some(jp) = state.job_points {
                self.dispatch(Event::new(
                    "RAPPELZ_JP_CHANGED",
                    vec![EventArg::Number(jp as f64)],
                ));
            }
        }
        if old.skill_event!=state.skill_event {
            if let Some(e)=&state.skill_event {
                self.dispatch(Event::new("RAPPELZ_SKILL_EVENT",vec![EventArg::Number(f64::from(e.skill)),e.phase.clone().into(),EventArg::Number(f64::from(e.caster)),EventArg::Number(f64::from(e.amount))]));
            }
        }
    }

    pub fn draw_widgets(&self, width: f32, height: f32) -> Vec<DrawWidget> {
        let mut out = Vec::new();
        for a in self.instances.iter().filter(|a| !a.failed) {
            if let Some(root) = a.widgets.borrow_mut().first_mut() {
                root.width = width;
                root.height = height;
            }
            out.extend(ui::layout(
                &self.infos[a.info_index].manifest.name,
                &a.widgets.borrow(),
                width,
                height,
            ));
        }
        out
    }

    /// Same layout/hit-testing path used by both native rendering and headless integration tests.
    pub fn pointer(&mut self, width: f32, height: f32, x: f32, y: f32, click: bool) -> bool {
        let hit = self
            .draw_widgets(width, height)
            .into_iter()
            .rev()
            .find(|w| w.widget.mouse_enabled && w.rect.contains(x, y));
        let hovered = hit.as_ref().and_then(|w| {
            self.instances
                .iter()
                .position(|a| self.infos[a.info_index].manifest.name == w.addon)
                .map(|i| (i, w.widget.id))
        });
        for (target, script) in [
            (self.hovered.filter(|h| Some(*h) != hovered), "OnLeave"),
            (hovered.filter(|h| Some(*h) != self.hovered), "OnEnter"),
            (hovered.filter(|_| click), "OnClick"),
        ] {
            if let Some((i, id)) = target {
                if !self.instances[i].failed {
                    self.instances[i].reset_budget();
                    if let Err(e) = self.instances[i].call(id, script, None) {
                        self.fail(i, e);
                    }
                }
            }
        }
        self.hovered = hovered;
        hit.is_some()
    }

    pub fn save(&self) -> Result<()> {
        for a in &self.instances {
            let m = &self.infos[a.info_index].manifest;
            let (account, character) = self.variable_paths(&m.name);
            saved::save(&a.lua, &account, &m.saved_variables)
                .with_context(|| format!("{} account settings", m.name))?;
            if let Some(path) = character {
                saved::save(&a.lua, &path, &m.character_variables)
                    .with_context(|| format!("{} character settings", m.name))?;
            }
        }
        Ok(())
    }

    pub fn reload(&mut self) -> Result<()> {
        // Keep the live runtime intact if settings cannot be saved.
        self.save()?;
        self.instances.clear();
        self.hovered = None;
        self.load()
    }

    pub fn set_enabled(&mut self, name: &str, enabled: bool) -> Result<()> {
        ensure!(
            self.infos.iter().any(|i| i.manifest.name == name),
            "unknown addon: {name}"
        );
        let mut next = self.enabled.clone();
        next.insert(name.into(), enabled);
        saved::atomic_json(&self.account_path().join("AddOns.json"), &next)?;
        self.enabled = next;
        for info in &mut self.infos {
            if info.manifest.name == name {
                info.enabled = enabled;
            }
        }
        Ok(())
    }

    /// Host-controlled load-on-demand request; no addon receives filesystem or loader access.
    pub fn request_load(&mut self, name: &str) -> Result<()> {
        ensure!(
            self.infos.iter().any(|i| i.manifest.name == name),
            "unknown addon: {name}"
        );
        self.requested.insert(name.into());
        self.reload()
    }

    pub fn shutdown(&mut self) -> Result<()> {
        if self.state.borrow().player.is_some() {
            self.dispatch(Event::new("PLAYER_LOGOUT", vec![]));
        }
        self.save()
    }
}
