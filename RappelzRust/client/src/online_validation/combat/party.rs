//! Opt-in native group lifecycle using real gameplay, roster packets and Lua diagnostics.
use super::{game, id, mark, party_state, read, unit};
use crate::{world::WorldState, App, NetCmd};
use anyhow::{ensure, Context, Result};
use rz_proto::Message;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Debug, PartialEq)]
enum Stage {
    Join,
    Decline,
    InviteAgain,
    Accept,
    Joined,
    Damage,
    Isolate,
    Restore,
    Chat,
    Transfer,
    Kick,
    Rejoin,
    Leave,
    LastJoin,
    Disconnect,
    Logout,
    Done,
}
pub struct PartyValidation {
    directory: PathBuf,
    role: String,
    peer: String,
    stage: Stage,
    since: Instant,
    own: u32,
    other: u32,
    monster: u32,
    cycle: u32,
    first_id: u32,
    sent: bool,
    events: BTreeMap<String, usize>,
    messages: Vec<String>,
    steps: Vec<Value>,
    failure: Option<String>,
}
impl PartyValidation {
    pub fn new(plan: PathBuf, app: &App) -> Result<Self> {
        super::super::guard(app)?;
        let p: Value = serde_json::from_slice(&std::fs::read(plan)?)?;
        let role = p["role"].as_str().context("party role")?.to_string();
        ensure!(role == "leader" || role == "member", "party role");
        let peer = p["peer"].as_str().context("party peer")?.to_string();
        ensure!(peer.starts_with("Lua"), "dedicated peer");
        let directory = PathBuf::from(p["directory"].as_str().context("party directory")?);
        std::fs::create_dir_all(&directory)?;
        ensure!(
            !directory.join(format!("{role}.json")).exists(),
            "fresh directory required"
        );
        Ok(Self {
            directory,
            role,
            peer,
            stage: Stage::Join,
            since: Instant::now(),
            own: 0,
            other: 0,
            monster: 0,
            cycle: 0,
            first_id: 0,
            sent: false,
            events: BTreeMap::new(),
            messages: Vec::new(),
            steps: Vec::new(),
            failure: None,
        })
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
    fn advance(&mut self, stage: Stage) {
        self.stage = stage;
        self.since = Instant::now();
        self.sent = false;
    }
    fn ready(&self, key: &str) -> bool {
        read(&self.directory.join(key)).is_some()
    }
    fn mark(&self, key: &str) -> Result<()> {
        mark(&self.directory, key, json!({"ready":true}))
    }
    fn action(app: &App, action: &str) {
        app.send(WorldState::party_action(action));
    }
    fn chat(app: &App, message: &str) {
        app.send(NetCmd::Chat {
            chat_type: rz_proto::msg::chat_type::PARTY,
            target: String::new(),
            message: message.into(),
        });
    }
    fn command(app: &App, message: &str) {
        app.send(NetCmd::Chat {
            chat_type: 0,
            target: String::new(),
            message: message.into(),
        });
    }
    fn record(&mut self, app: &App, stage: &str) {
        let w = app.world.as_ref();
        let state = w.map(WorldState::addon_state).unwrap_or_default();
        self.steps.push(json!({"stage":stage,"party":w.and_then(WorldState::party_state).map(party_state),"player":state.player.as_ref().map(unit),
            "party_count":state.party_members,"party_units":state.party.as_ref().map(|p|p.iter().map(unit).collect::<Vec<_>>()),
            "peer_visible":w.is_some_and(|w|w.unit(self.other).is_some()),"events":self.events,
            "display":super::super::AddonValidation::display(app)}));
    }
    fn capture(&self, name: &str) -> Result<()> {
        let path = self.directory.join(format!("{name}.png"));
        macroquad::prelude::get_screen_data().export_png(path.to_str().context("capture path")?);
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
            app.combat_trace
                .as_ref()
                .context("trace")?
                .errors
                .is_empty(),
            "trace decoding error"
        );
        let other = if self.role == "leader" {
            "member"
        } else {
            "leader"
        };
        if let Some(report) = read(&self.directory.join(format!("{other}.json"))) {
            ensure!(
                report["success"] != false,
                "peer failure: {}",
                report["failure"]
            );
        }
        ensure!(
            self.since.elapsed() < Duration::from_secs(45),
            "party timeout at {:?}",
            self.stage
        );
        if self.stage == Stage::Logout {
            if app.world.is_none() && self.events.get("PLAYER_LOGOUT") == Some(&1) {
                self.record(app, "Logout");
                self.advance(Stage::Done);
                return Ok(true);
            }
            return Ok(false);
        }
        let Some(world) = app.world.as_ref() else {
            return Ok(false);
        };
        let Some(party) = world.party_state().cloned() else {
            return Ok(false);
        };
        let player = world.addon_state().player.context("player")?;
        let grouped = party.members.len() == 2;
        let leader = if self.role == "leader" {
            self.own
        } else {
            self.other
        };
        let member = if self.role == "member" {
            self.own
        } else {
            self.other
        };
        match self.stage {
            Stage::Join => {
                if app.loading.is_some() {
                    return Ok(false);
                }
                let display = if self.role == "member" {
                    format!("<GM>{}", self.peer)
                } else {
                    self.peer.clone()
                };
                let Some(peer) = world.visible_player(&display) else {
                    return Ok(false);
                };
                ensure!(
                    party.members.is_empty()
                        && party.invite.is_none()
                        && player.health == Some(100)
                        && player.mana == Some(50),
                    "dedicated solo fixture changed"
                );
                self.own = player.handle;
                self.other = peer.handle;
                self.monster = world.living_monster().context("monster")?.handle;
                app.auto_enter = None;
                self.record(app, "Solo");
                self.mark(&format!("{}.ready", self.role))?;
                if !self.ready(&format!("{other}.ready")) {
                    return Ok(false);
                }
                if self.role == "leader" {
                    Self::action(app, &format!("invite {}", self.peer));
                }
                self.advance(Stage::Decline);
            }
            Stage::Decline => {
                if self.role == "member" {
                    if party.invite.as_ref().is_some_and(|i| i.inviter == leader) {
                        ensure!(!grouped, "invitation joined without consent");
                        self.record(app, "Invited");
                        Self::action(app, "decline");
                        self.advance(Stage::InviteAgain);
                    }
                } else if self
                    .messages
                    .iter()
                    .any(|m| m.contains("Invitation de groupe refusée"))
                    && self.ready("member.declined")
                {
                    self.record(app, "Declined");
                    Self::action(app, &format!("invite {}", self.peer));
                    self.advance(Stage::Accept);
                }
            }
            Stage::InviteAgain => {
                if party.invite.is_none() {
                    self.record(app, "Declined");
                    self.mark("member.declined")?;
                    self.advance(Stage::Accept);
                }
            }
            Stage::Accept => {
                if self.role == "member"
                    && party.invite.as_ref().is_some_and(|i| i.inviter == leader)
                {
                    Self::action(app, "accept");
                    self.advance(Stage::Joined);
                } else if self.role == "leader" && grouped {
                    self.advance(Stage::Joined);
                }
            }
            Stage::Joined => {
                if !grouped {
                    return Ok(false);
                }
                ensure!(
                    party.leader == leader && party.members.iter().any(|m| m.handle == member),
                    "wrong initial group"
                );
                self.first_id = party.party_id;
                self.record(app, "Joined");
                self.capture(&format!("{}-joined", self.role))?;
                if self.role == "leader" {
                    app.send(NetCmd::Raw(
                        game::CsAttackRequest {
                            handle: self.own,
                            target_handle: self.monster,
                        }
                        .to_packet(),
                    ));
                }
                self.advance(Stage::Damage);
            }
            Stage::Damage => {
                if self.role == "leader" && player.health.is_some_and(|hp| hp > 0 && hp < 100) {
                    app.send(NetCmd::Raw(
                        game::Handle::<{ id::TS_CS_CANCEL_ACTION }>(self.own).to_packet(),
                    ));
                    Self::command(app, ".layer 1");
                    self.advance(Stage::Isolate);
                } else if self.role == "member"
                    && party
                        .members
                        .iter()
                        .any(|m| m.handle == leader && m.hp > 0 && m.hp < 100 && m.layer == 1)
                    && world.unit(leader).is_none()
                {
                    self.record(app, "DamagedIsolated");
                    self.capture("member-isolated")?;
                    self.mark("member.isolated")?;
                    self.advance(Stage::Restore);
                }
            }
            Stage::Isolate => {
                if world.layer() == 1 && self.ready("member.isolated") {
                    self.record(app, "DamagedIsolated");
                    Self::command(app, ".layer 0");
                    Self::command(app, ".level 1");
                    self.advance(Stage::Restore);
                }
            }
            Stage::Restore => {
                let healed = party
                    .members
                    .iter()
                    .find(|m| m.handle == leader)
                    .is_some_and(|m| m.layer == 0 && m.hp == 100 && m.mp == 50);
                if !healed || world.unit(self.other).is_none() {
                    return Ok(false);
                }
                self.record(app, "Restored");
                if self.role == "member" {
                    Self::action(app, "disband");
                }
                self.advance(Stage::Chat);
            }
            Stage::Chat => {
                let chats = &app.combat_trace.as_ref().unwrap().party_chats;
                if self.role == "member"
                    && !self.sent
                    && self.messages.iter().any(|m| m.contains("Seul le chef"))
                {
                    self.record(app, "PermissionDenied");
                    Self::chat(app, "LuaPartyMember");
                    self.mark("member.denied")?;
                    self.sent = true;
                }
                if self.role == "leader" && !self.sent && self.ready("member.denied") {
                    Self::chat(app, "LuaPartyLeader");
                    self.sent = true;
                }
                if chats.len() == 2 {
                    self.record(app, "Chat");
                    self.advance(Stage::Transfer);
                    if self.role == "leader" {
                        Self::action(app, &format!("leader {}", self.peer));
                    }
                }
            }
            Stage::Transfer => {
                if grouped && party.leader == member {
                    self.record(app, "Transferred");
                    self.mark(&format!("{}.transferred", self.role))?;
                    self.advance(Stage::Kick);
                }
            }
            Stage::Kick => {
                if self.role == "member" && !self.sent && self.ready("leader.transferred") {
                    Self::action(app, &format!("kick {}", self.peer));
                    self.sent = true;
                }
                if party.members.is_empty() {
                    self.record(app, "Kicked");
                    self.mark(&format!("{}.kicked", self.role))?;
                    self.cycle = 1;
                    if self.role == "leader" {
                        Self::chat(app, "LuaPartyOutside");
                    }
                    self.advance(Stage::Rejoin);
                }
            }
            Stage::Rejoin | Stage::LastJoin => {
                if self.role == "leader" && !self.sent {
                    if self.cycle == 1 && !self.ready("member.kicked") {
                        return Ok(false);
                    }
                    if self.cycle == 1
                        && !self
                            .messages
                            .iter()
                            .any(|m| m.contains("Vous n'êtes pas dans un groupe"))
                    {
                        return Ok(false);
                    }
                    Self::action(app, &format!("invite {}", self.peer));
                    self.sent = true;
                }
                if self.role == "member" && party.invite.is_some() && !self.sent {
                    Self::action(app, "accept");
                    self.sent = true;
                }
                if grouped {
                    ensure!(
                        party.party_id != self.first_id && party.leader == leader,
                        "group id reused"
                    );
                    if self.cycle == 1 {
                        self.record(app, "Rejoined");
                        self.mark(&format!("{}.rejoined", self.role))?;
                        self.advance(Stage::Leave);
                    } else {
                        self.record(app, "FinalGroup");
                        self.capture(&format!("{}-final", self.role))?;
                        self.mark(&format!("{}.final", self.role))?;
                        self.advance(Stage::Disconnect);
                    }
                }
            }
            Stage::Leave => {
                if self.role == "member" && !self.sent && self.ready("leader.rejoined") {
                    Self::action(app, "leave");
                    self.sent = true;
                }
                if party.members.is_empty() {
                    self.record(app, "Left");
                    self.cycle = 2;
                    self.advance(Stage::LastJoin);
                }
            }
            Stage::Disconnect => {
                if self.role == "leader" && self.ready("member.final") {
                    app.send(NetCmd::ReturnLobby);
                    self.advance(Stage::Logout);
                } else if self.role == "member"
                    && party.members.is_empty()
                    && world.unit(self.other).is_none()
                {
                    self.record(app, "PeerLogout");
                    app.send(NetCmd::ReturnLobby);
                    self.advance(Stage::Logout);
                }
            }
            Stage::Logout | Stage::Done => {}
        }
        Ok(false)
    }
    pub fn fail(&mut self, error: anyhow::Error) {
        self.failure = Some(format!("{error:#}"));
        let _ = self.capture(&format!("{}-failure", self.role));
    }
    pub fn finish(&self, app: &App) -> Result<bool> {
        let success = self.stage == Stage::Done && self.failure.is_none();
        mark(
            &self.directory,
            &format!("{}.json", self.role),
            json!({"scenario":"party","role":self.role,"success":success,"failure":self.failure,
            "own":self.own,"other":self.other,"monster":self.monster,"events":self.events,"messages":self.messages,"steps":self.steps,"trace":app.combat_trace.as_ref().context("trace")?.json()}),
        )?;
        Ok(success)
    }
}
