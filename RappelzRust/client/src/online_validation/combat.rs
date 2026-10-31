//! Two real native clients coordinated by test-only local barrier files.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{ensure, Context, Result};
use rz_addon::UnitState;
use rz_proto::msg::{EnterKind, Message};
use rz_proto::msg_game::{self as game, id};
use serde_json::{json, Value};

use crate::{App, NetCmd, NetEvent};

pub(super) mod revival;

#[derive(Default)]
pub struct PacketTrace {
    hits: Vec<Value>,
    hpmp: Vec<Value>,
    levels: Vec<Value>,
    experience: Vec<Value>,
    enters: Vec<Value>,
    leaves: Vec<u32>,
    lifecycle: Vec<Value>,
    warps: Vec<Value>,
    errors: Vec<String>,
}

impl PacketTrace {
    pub fn observe(&mut self, event: &NetEvent) {
        if self.hits.len()
            + self.hpmp.len()
            + self.enters.len()
            + self.leaves.len()
            + self.levels.len()
            + self.experience.len()
            + self.warps.len()
            + self.errors.len()
            >= 2048
        {
            if self.errors.is_empty() {
                self.errors.push("combat trace capacity exceeded".into());
            }
            return;
        }
        let result: Result<()> = (|| {
            match event {
                NetEvent::Enter(e) => {
                    let (name, creature, kind) = match &e.kind {
                        EnterKind::Player(p) => (p.name.as_str(), &p.creature, "player"),
                        EnterKind::Monster { creature, .. } => ("", creature, "monster"),
                        _ => return Ok(()),
                    };
                    self.enters.push(json!({"handle":e.handle,"name":name,"kind":kind,"level":creature.level,
                        "hp":creature.hp,"max_hp":creature.max_hp,"mp":creature.mp,"max_mp":creature.max_mp,
                        "x":e.x,"y":e.y,"z":e.z,"layer":e.layer}));
                    self.lifecycle.push(json!({"event":"enter","handle":e.handle,"hp":creature.hp,"mp":creature.mp}));
                }
                NetEvent::Leave(handle) => {
                    self.leaves.push(*handle);
                    self.lifecycle
                        .push(json!({"event":"leave","handle":handle}));
                }
                NetEvent::Warp { x, y, z } => self.warps.push(json!({"x":x,"y":y,"z":z})),
                NetEvent::Packet(p) => match p.id {
                    id::TS_SC_ATTACK_EVENT => {
                        let packet = game::ScAttackEvent::from_packet(p)?;
                        for hit in &packet.attacks {
                            self.hits.push(json!({"attacker":packet.attacker,"target":packet.target,"damage":hit.damage,
                                "target_hp":hit.target_hp,"target_mp":hit.target_mp,"attacker_hp":hit.attacker_hp,"attacker_mp":hit.attacker_mp}));
                        }
                    }
                    id::TS_SC_HPMP => {
                        let h = game::ScHpMp::from_packet(p)?;
                        self.hpmp.push(json!({"handle":h.handle,"hp":h.hp,"max_hp":h.max_hp,"mp":h.mp,"max_mp":h.max_mp,"add_hp":h.add_hp}));
                    }
                    id::TS_SC_LEVEL_UPDATE => {
                        let l = game::ScLevelUpdate::from_packet(p)?;
                        self.levels.push(
                            json!({"handle":l.handle,"level":l.level,"job_level":l.job_level}),
                        );
                    }
                    id::TS_SC_EXP_UPDATE => {
                        let e = game::ScExpUpdate::from_packet(p)?;
                        self.experience
                            .push(json!({"handle":e.handle,"xp":e.exp,"jp":e.jp}));
                    }
                    _ => {}
                },
                _ => {}
            }
            Ok(())
        })();
        if let Err(error) = result {
            self.errors.push(format!("{error:#}"));
        }
    }

    fn json(&self) -> Value {
        json!({"hits":self.hits,"hpmp":self.hpmp,"levels":self.levels,"experience":self.experience,
            "enters":self.enters,"leaves":self.leaves,"lifecycle":self.lifecycle,"warps":self.warps,"errors":self.errors})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_decodes_wire_values_and_records_corrupt_packets_as_errors() {
        let mut trace = PacketTrace::default();
        let hpmp = game::ScHpMp {
            handle: 7,
            hp: 104,
            max_hp: 120,
            mp: 60,
            max_mp: 60,
            add_hp: -16,
            ..Default::default()
        };
        trace.observe(&NetEvent::Packet(hpmp.to_packet()));
        assert_eq!(trace.hpmp[0]["hp"], 104);
        assert_eq!(trace.hpmp[0]["mp"], 60);
        let mut bad = hpmp.to_packet();
        bad.body.clear();
        trace.observe(&NetEvent::Packet(bad));
        assert_eq!(trace.errors.len(), 1);
        assert_eq!(trace.hpmp.len(), 1);
    }

    #[test]
    fn lifecycle_order_and_leave_only_capacity_are_preserved() {
        let mut trace = PacketTrace::default();
        trace.observe(&NetEvent::Enter(rz_proto::msg::ScEnter {
            enter_type: 0,
            handle: 8,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            layer: 0,
            kind: EnterKind::Monster {
                creature: rz_proto::msg::CreatureInfo {
                    hp: 105,
                    mp: 53,
                    ..Default::default()
                },
                is_tamed: false,
                monster_id: 711,
            },
        }));
        trace.observe(&NetEvent::Leave(8));
        assert_eq!(trace.lifecycle[0]["event"], "enter");
        assert_eq!(trace.lifecycle[1]["event"], "leave");
        for _ in 0..3000 {
            trace.observe(&NetEvent::Leave(8));
        }
        assert_eq!(trace.errors, ["combat trace capacity exceeded"]);
        assert_eq!(trace.enters.len() + trace.leaves.len(), 2048);
    }
}

#[derive(Debug, PartialEq)]
enum Stage {
    Join,
    Fight,
    DamageQuery,
    Query,
    Hold,
    Corpse,
    HiddenQuery,
    Respawn,
    RepeatFight,
    RepeatQuery,
    RepeatHold,
    Logout,
    Leave,
    Done,
}

pub struct CombatValidation {
    directory: PathBuf,
    role: String,
    peer: String,
    stage: Stage,
    since: Instant,
    fighter: u32,
    monster: u32,
    own: u32,
    query_enters: usize,
    events: BTreeMap<String, usize>,
    messages: Vec<String>,
    steps: Vec<Value>,
    failure: Option<String>,
    damage_query_done: bool,
    kills: u8,
    monster_enters: usize,
}

fn unit(unit: &UnitState) -> Value {
    json!({"handle":unit.handle,"name":unit.name,"level":unit.level,"hp":unit.health,"max_hp":unit.max_health,"mp":unit.mana,"max_mp":unit.max_mana})
}

fn read(path: &Path) -> Option<Value> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
}

fn mark(directory: &Path, name: &str, data: Value) -> Result<()> {
    let path = directory.join(name);
    let temporary = path.with_extension("tmp");
    std::fs::write(&temporary, serde_json::to_vec_pretty(&data)?)?;
    std::fs::rename(temporary, path)?;
    Ok(())
}

impl CombatValidation {
    pub fn new(plan: PathBuf, app: &App) -> Result<Self> {
        super::guard(app)?;
        let plan: Value = serde_json::from_slice(&std::fs::read(plan)?)?;
        let kills = plan
            .get("kills")
            .map(|v| v.as_u64().context("kills must be 1 or 2"))
            .transpose()?
            .unwrap_or(1);
        ensure!((1..=2).contains(&kills), "kills must be 1 or 2");
        let role = plan["role"].as_str().context("combat role")?.to_string();
        ensure!(
            role == "fighter" || role == "witness",
            "combat role must be fighter or witness"
        );
        let peer = plan["peer"].as_str().context("peer name")?.to_string();
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
            fighter: 0,
            monster: 0,
            own: 0,
            query_enters: 0,
            events: BTreeMap::new(),
            messages: Vec::new(),
            steps: Vec::new(),
            failure: None,
            damage_query_done: false,
            kills: kills as u8,
            monster_enters: 0,
        })
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
    fn mark(&self, name: &str, data: Value) -> Result<()> {
        mark(&self.directory, name, data)
    }
    fn advance(&mut self, stage: Stage) {
        self.stage = stage;
        self.since = Instant::now();
    }
    fn record(&mut self, app: &App) {
        let state = app
            .world
            .as_ref()
            .map(crate::world::WorldState::addon_state)
            .unwrap_or_default();
        self.steps.push(json!({"stage":format!("{:?}",self.stage),"player":state.player.as_ref().map(unit),
            "target":state.target.as_ref().map(unit),"xp_jp":app.world.as_ref().map(|w|w.experience()),
            "display":super::AddonValidation::display(app),"events":self.events}));
    }
    fn capture(&self) -> Result<()> {
        let path = self.directory.join(format!("{}.png", self.role));
        macroquad::prelude::get_screen_data().export_png(path.to_str().context("screenshot path")?);
        Ok(())
    }
    fn trace<'a>(&self, app: &'a App) -> Result<&'a PacketTrace> {
        app.combat_trace.as_ref().context("trace missing")
    }

    pub fn tick(&mut self, app: &mut App) -> Result<bool> {
        ensure!(app.error.is_none(), "client/network error: {:?}", app.error);
        ensure!(
            app.addons.error.is_none() && app.addons.runtime.errors().is_empty(),
            "addon failure: {:?} {:?}",
            app.addons.error,
            app.addons.runtime.errors()
        );
        ensure!(
            self.trace(app)?.errors.is_empty(),
            "trace error: {:?}",
            self.trace(app)?.errors
        );
        let other = if self.role == "fighter" {
            "witness"
        } else {
            "fighter"
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
            anyhow::bail!("timeout at {:?} ({})", self.stage, self.role);
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
                self.monster = monster.handle;
                self.fighter = if self.role == "fighter" {
                    player.handle
                } else {
                    peer.handle
                };
                app.auto_enter = None;
                if self.role == "fighter" {
                    ensure!(
                        player.level == Some(2) && world.experience() == (799, 0),
                        "fighter fixture must be level 2, XP 799, JP 0"
                    );
                    if read(&self.directory.join("witness.ready")).is_none() {
                        return Ok(false);
                    }
                    let world = app.world.as_mut().unwrap();
                    let select = world
                        .select_target(self.monster)
                        .context("monster disappeared")?;
                    let attack = world.attack_target(self.monster).context("attack target")?;
                    app.send(select);
                    app.sync_addons();
                    self.record(app);
                    app.send(attack);
                } else {
                    ensure!(
                        player.level == Some(1) && world.experience() == (0, 0),
                        "witness fixture must be level 1 without XP/JP"
                    );
                    let select = app
                        .world
                        .as_mut()
                        .unwrap()
                        .select_target(self.fighter)
                        .context("fighter disappeared")?;
                    app.send(select);
                    app.sync_addons();
                    self.record(app);
                    self.mark(
                        "witness.ready",
                        json!({"handle":self.own,"fighter":self.fighter}),
                    )?;
                }
                self.advance(Stage::Fight);
            }
            Stage::Fight => {
                let world = app.world.as_ref().context("world missing during fight")?;
                let state = world.addon_state();
                if self.role == "fighter" {
                    let Some(player) = state.player.as_ref() else {
                        return Ok(false);
                    };
                    if player.level != Some(3)
                        || player.health != Some(140)
                        || player.max_health != Some(140)
                        || player.mana != Some(70)
                        || player.max_mana != Some(70)
                    {
                        return Ok(false);
                    }
                    ensure!(
                        world.experience() == (4, 3),
                        "kill did not award expected XP remainder/JP: {:?}",
                        world.experience()
                    );
                    ensure!(
                        player.health == Some(140)
                            && player.max_health == Some(140)
                            && player.mana == Some(70)
                            && player.max_mana == Some(70),
                        "fighter vitals after XP level: {player:?}"
                    );
                    let trace = self.trace(app)?;
                    ensure!(
                        trace
                            .hits
                            .iter()
                            .any(|h| h["attacker"] == self.monster && h["target"] == self.fighter),
                        "no monster retaliation"
                    );
                    ensure!(
                        trace.hits.iter().any(|h| h["attacker"] == self.fighter
                            && h["target"] == self.monster
                            && h["target_hp"] == 0),
                        "monster death not received"
                    );
                    ensure!(
                        trace.hpmp.iter().any(|h| h["handle"] == self.fighter
                            && h["hp"].as_i64().is_some_and(|hp| hp < 120)),
                        "fighter never received damage"
                    );
                    self.record(app);
                    self.capture()?;
                    self.mark(
                        "fighter.reward",
                        json!({"fighter":self.fighter,"monster":self.monster,"xp":4,"jp":3}),
                    )?;
                    self.advance(Stage::Hold);
                } else {
                    let Some(target) = state.target.as_ref() else {
                        return Ok(false);
                    };
                    if !self.damage_query_done
                        && target.health.is_some_and(|hp| hp < 120)
                        && target.level == Some(2)
                    {
                        self.query_enters = self
                            .trace(app)?
                            .enters
                            .iter()
                            .filter(|e| e["handle"] == self.fighter)
                            .count();
                        self.record(app);
                        app.send(NetCmd::Raw(
                            game::Handle::<{ id::TS_CS_QUERY }>(self.fighter).to_packet(),
                        ));
                        self.advance(Stage::DamageQuery);
                        return Ok(false);
                    }
                    if target.level != Some(3) {
                        return Ok(false);
                    }
                    if target.health != Some(140)
                        || target.max_health != Some(140)
                        || target.mana != Some(70)
                        || target.max_mana != Some(70)
                    {
                        return Ok(false);
                    }
                    let trace = self.trace(app)?;
                    ensure!(
                        trace
                            .hits
                            .iter()
                            .any(|h| h["attacker"] == self.monster && h["target"] == self.fighter),
                        "witness missed retaliation"
                    );
                    ensure!(
                        trace.hits.iter().any(|h| h["attacker"] == self.fighter
                            && h["target"] == self.monster
                            && h["target_hp"] == 0),
                        "witness missed kill"
                    );
                    ensure!(
                        world.experience() == (0, 0),
                        "witness gained killer-only XP or JP"
                    );
                    ensure!(
                        trace.experience.iter().all(|e| e["handle"] != self.fighter),
                        "private fighter XP leaked to witness"
                    );
                    ensure!(
                        state.player.as_ref().is_some_and(|p| p.level == Some(1)
                            && p.health == Some(100)
                            && p.mana == Some(50)),
                        "witness own state changed"
                    );
                    self.query_enters = trace
                        .enters
                        .iter()
                        .filter(|e| e["handle"] == self.fighter)
                        .count();
                    self.record(app);
                    app.send(NetCmd::Raw(
                        game::Handle::<{ id::TS_CS_QUERY }>(self.fighter).to_packet(),
                    ));
                    self.advance(Stage::Query);
                }
            }
            Stage::DamageQuery => {
                let trace = self.trace(app)?;
                let enters: Vec<_> = trace
                    .enters
                    .iter()
                    .filter(|e| e["handle"] == self.fighter)
                    .collect();
                if enters.len() <= self.query_enters {
                    return Ok(false);
                }
                let last = enters.last().unwrap();
                let hpmp = trace
                    .hpmp
                    .iter()
                    .rev()
                    .find(|e| e["handle"] == self.fighter)
                    .context("fighter damage HPMP missing")?;
                ensure!(
                    last["hp"] == hpmp["hp"]
                        && last["max_hp"] == hpmp["max_hp"]
                        && last["mp"] == hpmp["mp"]
                        && last["max_mp"] == hpmp["max_mp"],
                    "fresh ENTER disagrees with combat HPMP: ENTER={last}, HPMP={hpmp}"
                );
                self.damage_query_done = true;
                self.record(app);
                self.advance(Stage::Fight);
            }
            Stage::Query => {
                let trace = self.trace(app)?;
                let enters: Vec<_> = trace
                    .enters
                    .iter()
                    .filter(|e| e["handle"] == self.fighter)
                    .collect();
                if enters.len() <= self.query_enters {
                    return Ok(false);
                }
                let last = enters.last().unwrap();
                ensure!(
                    last["level"] == 3
                        && last["hp"] == 140
                        && last["max_hp"] == 140
                        && last["mp"] == 70
                        && last["max_mp"] == 70,
                    "fresh ENTER has stale stats: {last}"
                );
                self.record(app);
                self.capture()?;
                self.mark(
                    "witness.synced",
                    json!({"fighter":self.fighter,"fresh_enter":last}),
                )?;
                self.advance(if self.kills == 2 {
                    Stage::Corpse
                } else {
                    Stage::Leave
                });
            }
            Stage::Hold => {
                if read(&self.directory.join("witness.synced")).is_some() {
                    self.record(app);
                    if self.kills == 2 {
                        self.advance(Stage::Corpse);
                    } else {
                        app.send(NetCmd::ReturnLobby);
                        self.advance(Stage::Logout);
                    }
                }
            }
            Stage::Corpse => {
                let world = app
                    .world
                    .as_ref()
                    .context("world missing at corpse removal")?;
                if !self.trace(app)?.leaves.contains(&self.monster)
                    || world.unit(self.monster).is_some()
                {
                    return Ok(false);
                }
                if self.role == "fighter" {
                    ensure!(
                        world.addon_state().target.is_none(),
                        "corpse target was not cleared"
                    );
                }
                self.monster_enters = self
                    .trace(app)?
                    .enters
                    .iter()
                    .filter(|e| e["handle"] == self.monster)
                    .count();
                self.record(app);
                // A stale handle must not bring a hidden corpse back into the world.
                app.send(NetCmd::Raw(
                    game::Handle::<{ id::TS_CS_QUERY }>(self.monster).to_packet(),
                ));
                self.advance(Stage::HiddenQuery);
            }
            Stage::HiddenQuery => {
                ensure!(
                    app.world
                        .as_ref()
                        .context("world missing during hidden query")?
                        .unit(self.monster)
                        .is_none(),
                    "query reintroduced hidden corpse"
                );
                ensure!(
                    self.trace(app)?
                        .enters
                        .iter()
                        .filter(|e| e["handle"] == self.monster)
                        .count()
                        == self.monster_enters,
                    "hidden corpse query received ENTER"
                );
                if self.since.elapsed() >= Duration::from_secs(1) {
                    self.record(app);
                    self.advance(Stage::Respawn);
                }
            }
            Stage::Respawn => {
                let world = app.world.as_ref().context("world missing at respawn")?;
                let Some(monster) = world.unit(self.monster) else {
                    return Ok(false);
                };
                ensure!(
                    monster.health == Some(105)
                        && monster.max_health == Some(105)
                        && monster.mana == Some(53)
                        && monster.max_mana == Some(53),
                    "respawn vitals: {monster:?}"
                );
                ensure!(
                    self.trace(app)?
                        .enters
                        .iter()
                        .filter(|e| e["handle"] == self.monster)
                        .count()
                        > self.monster_enters,
                    "respawn did not arrive via ENTER"
                );
                if !self
                    .directory
                    .join(format!("{}.respawn", self.role))
                    .exists()
                {
                    let select = app
                        .world
                        .as_mut()
                        .unwrap()
                        .select_target(self.monster)
                        .context("respawn selection")?;
                    app.send(select);
                    app.sync_addons();
                    self.record(app);
                    self.mark(
                        &format!("{}.respawn", self.role),
                        json!({"handle":self.monster}),
                    )?;
                }
                if self.role == "fighter" {
                    if read(&self.directory.join("witness.respawn")).is_none() {
                        return Ok(false);
                    }
                    let attack = app
                        .world
                        .as_ref()
                        .unwrap()
                        .attack_target(self.monster)
                        .context("respawn attack")?;
                    app.send(attack);
                }
                self.advance(Stage::RepeatFight);
            }
            Stage::RepeatFight => {
                let trace = self.trace(app)?;
                let deaths = trace
                    .hits
                    .iter()
                    .filter(|h| {
                        h["attacker"] == self.fighter
                            && h["target"] == self.monster
                            && h["target_hp"] == 0
                    })
                    .count();
                if deaths < 2 {
                    return Ok(false);
                }
                ensure!(deaths == 2, "unexpected extra kill");
                let world = app.world.as_ref().context("world missing at second kill")?;
                if self.role == "fighter" {
                    if world.experience() != (9, 6) {
                        return Ok(false);
                    }
                    let player = world.addon_state().player.context("fighter missing")?;
                    ensure!(
                        player.level == Some(3)
                            && player.health.is_some_and(|hp| hp > 0 && hp < 140)
                            && player.max_health == Some(140)
                            && player.mana == Some(70),
                        "second kill state: {player:?}"
                    );
                    self.record(app);
                    self.capture()?;
                    self.mark("fighter.repeat.reward", json!({"fighter":self.fighter}))?;
                    self.advance(Stage::RepeatHold);
                } else {
                    ensure!(world.experience() == (0, 0), "witness gained repeat reward");
                    self.query_enters = trace
                        .enters
                        .iter()
                        .filter(|e| e["handle"] == self.fighter)
                        .count();
                    let select = app
                        .world
                        .as_mut()
                        .unwrap()
                        .select_target(self.fighter)
                        .context("fighter selection after repeat")?;
                    app.send(select);
                    app.sync_addons();
                    self.record(app);
                    app.send(NetCmd::Raw(
                        game::Handle::<{ id::TS_CS_QUERY }>(self.fighter).to_packet(),
                    ));
                    self.advance(Stage::RepeatQuery);
                }
            }
            Stage::RepeatQuery => {
                let trace = self.trace(app)?;
                let enters: Vec<_> = trace
                    .enters
                    .iter()
                    .filter(|e| e["handle"] == self.fighter)
                    .collect();
                if enters.len() <= self.query_enters {
                    return Ok(false);
                }
                let last = enters.last().unwrap();
                let hpmp = trace
                    .hpmp
                    .iter()
                    .rev()
                    .find(|e| e["handle"] == self.fighter)
                    .context("repeat HPMP missing")?;
                ensure!(
                    last["level"] == 3
                        && ["hp", "max_hp", "mp", "max_mp"]
                            .iter()
                            .all(|k| last[k] == hpmp[k]),
                    "repeat query disagrees with live HPMP: {last} versus {hpmp}"
                );
                self.record(app);
                self.capture()?;
                self.mark("witness.repeat.synced", json!({"fighter":self.fighter}))?;
                self.advance(Stage::Leave);
            }
            Stage::RepeatHold => {
                if read(&self.directory.join("witness.repeat.synced")).is_some() {
                    self.record(app);
                    app.send(NetCmd::ReturnLobby);
                    self.advance(Stage::Logout);
                }
            }
            Stage::Leave => {
                if self.trace(app)?.leaves.contains(&self.fighter) {
                    ensure!(
                        app.world
                            .as_ref()
                            .is_some_and(|world| world.addon_state().target.is_none()),
                        "target not cleared after peer logout"
                    );
                    self.record(app);
                    app.send(NetCmd::ReturnLobby);
                    self.advance(Stage::Logout);
                }
            }
            Stage::Logout => {
                if app.world.is_none() && self.events.get("PLAYER_LOGOUT").copied().unwrap_or(0) > 0
                {
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
        self.mark(&format!("{}.json",self.role),json!({"success":success,"failure":self.failure,"role":self.role,
            "account":app.account,"fighter":self.fighter,"monster":self.monster,"own":self.own,"kills":self.kills,"events":self.events,
            "messages":self.messages,"steps":self.steps,"trace":self.trace(app)?.json()}))?;
        Ok(success)
    }
}
