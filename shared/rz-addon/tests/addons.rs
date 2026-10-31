use std::fs;
use std::path::Path;

use rz_addon::{AddonRuntime, Event, GameState, UnitState};

fn addon(root: &Path, name: &str, metadata: &str, code: &str) {
    let dir = root.join("Interface/AddOns").join(name);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(format!("{name}.toc")),
        format!("## Interface: 30300\n{metadata}\n{name}.lua\n"),
    )
    .unwrap();
    fs::write(dir.join(format!("{name}.lua")), code).unwrap();
}

fn debug_addon(root: &Path) {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../RappelzRust/Interface/AddOns/RappelzDebug");
    let dest = root.join("Interface/AddOns/RappelzDebug");
    fs::create_dir_all(&dest).unwrap();
    for file in ["RappelzDebug.toc", "RappelzDebug.lua", "RappelzDebug.xml"] {
        fs::copy(source.join(file), dest.join(file)).unwrap();
    }
}

fn state() -> GameState {
    GameState {
        player: Some(UnitState {
            handle: 7,
            name: "Gaia".into(),
            level: Some(42),
            layer: Some(0),
            health: Some(765),
            max_health: Some(1000),
            mana: Some(50),
            max_mana: Some(90),
        }),
        money: Some(12345),
        ..Default::default()
    }
}

fn text(runtime: &AddonRuntime) -> String {
    runtime
        .draw_widgets(1280.0, 720.0)
        .into_iter()
        .map(|w| w.widget.text)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn real_debug_addon_lifecycle_live_state_click_reload_and_scoped_persistence() {
    let dir = tempfile::tempdir().unwrap();
    debug_addon(dir.path());
    let mut rt = AddonRuntime::new(dir.path());
    rt.set_identity("account", "realm", Some("Gaia")).unwrap();
    assert_eq!(rt.addons()[0].status, "Loaded");
    assert!(rt
        .drain_messages()
        .iter()
        .any(|s| s.contains("ADDON_LOADED; Lua Lua 5.1")));
    assert!(text(&rt).contains("(not in world)"));
    rt.update_state(state());
    let display = text(&rt);
    for expected in ["Player: Gaia", "Level: 42", "HP: 765 / 1000", "Login: true"] {
        assert!(display.contains(expected), "{display}");
    }
    assert!(rt
        .drain_messages()
        .iter()
        .any(|s| s.contains("PLAYER_LOGIN: Gaia")));
    let button = rt
        .draw_widgets(1280.0, 720.0)
        .into_iter()
        .find(|w| w.widget.name.as_deref() == Some("RappelzDebugButton"))
        .unwrap();
    assert!(rt.pointer(
        1280.0,
        720.0,
        button.rect.x + 10.0,
        button.rect.y + 10.0,
        true
    ));
    assert!(text(&rt).contains("Clicks: 1"));
    rt.reload().unwrap();
    assert!(text(&rt).contains("Clicks: 1"));
    let mut next = state();
    next.player.as_mut().unwrap().health = Some(123);
    next.target = Some(UnitState {
        handle: 9,
        name: "Target".into(),
        ..Default::default()
    });
    rt.update_state(next);
    assert!(text(&rt).contains("HP: 123 / 1000"));
    assert!(text(&rt).contains("Target: Target"));
    rt.shutdown().unwrap();
    assert!(rt.errors().is_empty(), "{:?}", rt.errors());
    let mut other = AddonRuntime::new(dir.path());
    other
        .set_identity("another-account", "realm", Some("Gaia"))
        .unwrap();
    assert!(text(&other).contains("Clicks: 0"));
}

#[test]
fn game_state_deltas_dispatch_once_with_arguments_and_no_invented_party_data() {
    let dir = tempfile::tempdir().unwrap();
    addon(
        dir.path(),
        "Events",
        "",
        r#"
        assert(GetNumPartyMembers() == nil and GetMoney() == nil and UnitHealth("player") == nil)
        local f = CreateFrame("Frame", "EventsFrame", UIParent)
        for _, e in ipairs({"PLAYER_LOGIN", "UNIT_HEALTH", "UNIT_MANA", "PLAYER_LEVEL_UP", "BAG_UPDATE", "PLAYER_TARGET_CHANGED", "CHAT_MSG_SYSTEM", "PLAYER_LOGOUT"}) do f:RegisterEvent(e) end
        f:SetScript("OnEvent", function(self, event, arg) print(event, arg or "none", UnitName("player")) end)
    "#,
    );
    let mut rt = AddonRuntime::new(dir.path());
    rt.load().unwrap();
    rt.update_state(state());
    rt.drain_messages();
    rt.update_state(state());
    assert!(rt.drain_messages().is_empty());
    let mut s = state();
    s.player.as_mut().unwrap().health = Some(1);
    s.player.as_mut().unwrap().mana = Some(2);
    s.player.as_mut().unwrap().level = Some(43);
    s.bag_revision = 1;
    s.target = Some(UnitState {
        handle: 88,
        name: "Pet".into(),
        ..Default::default()
    });
    rt.update_state(s);
    let events = rt.drain_messages();
    for expected in [
        "UNIT_HEALTH player",
        "UNIT_MANA player",
        "PLAYER_LEVEL_UP 43",
        "BAG_UPDATE 0",
        "PLAYER_TARGET_CHANGED none",
    ] {
        assert_eq!(
            events.iter().filter(|s| s.contains(expected)).count(),
            1,
            "{events:?}"
        );
    }
    rt.dispatch(Event::new("CHAT_MSG_SYSTEM", vec!["Hello".into()]));
    assert!(rt.drain_messages()[0].contains("CHAT_MSG_SYSTEM Hello"));
    rt.update_state(GameState::default());
    assert!(rt.drain_messages()[0].contains("PLAYER_LOGOUT none Gaia"));
}

#[test]
fn unknown_target_vitals_do_not_emit_fabricated_health_or_mana_changes() {
    let dir = tempfile::tempdir().unwrap();
    addon(
        dir.path(),
        "UnknownVitals",
        "",
        r#"
        local f = CreateFrame("Frame")
        f:RegisterEvent("UNIT_HEALTH")
        f:RegisterEvent("UNIT_MANA")
        f:SetScript("OnEvent", function(self, event, unit) print(event, unit) end)
    "#,
    );
    let mut rt = AddonRuntime::new(dir.path());
    rt.load().unwrap();
    rt.update_state(state());
    rt.drain_messages();
    let mut next = state();
    next.target = Some(UnitState {
        handle: 55,
        name: "Unknown".into(),
        ..Default::default()
    });
    rt.update_state(next.clone());
    assert!(rt.drain_messages().is_empty());
    next.target.as_mut().unwrap().mana = Some(0);
    next.target.as_mut().unwrap().max_mana = Some(100);
    rt.update_state(next.clone());
    let messages = rt.drain_messages();
    assert_eq!(messages.len(), 1);
    assert!(messages[0].contains("UNIT_MANA target"));
    rt.update_state(next);
    assert!(rt.drain_messages().is_empty());
}

#[test]
fn debug_addon_observes_death_and_revival_once_even_in_one_frame() {
    let dir = tempfile::tempdir().unwrap();
    debug_addon(dir.path());
    let mut rt = AddonRuntime::new(dir.path());
    rt.load().unwrap();
    let mut s = state();
    s.player.as_mut().unwrap().health = Some(1);
    rt.update_state(s.clone());
    rt.drain_messages();
    s.player.as_mut().unwrap().health = Some(0);
    rt.update_state(s.clone());
    rt.update_state(s.clone());
    s.player.as_mut().unwrap().health = Some(1000);
    rt.update_state(s.clone());
    rt.update_state(s);
    let messages = rt.drain_messages();
    for (event, hp) in [("PLAYER_DEAD", 0), ("PLAYER_ALIVE", 1000)] {
        let matching: Vec<_> = messages
            .iter()
            .filter(|m| m.contains(&format!("EVENT {event} ")))
            .collect();
        assert_eq!(matching.len(), 1, "{messages:?}");
        assert!(
            matching[0].contains(&format!("HP={hp}/1000")),
            "{matching:?}"
        );
    }
    let dead = messages
        .iter()
        .position(|m| m.contains("EVENT PLAYER_DEAD "))
        .unwrap();
    let alive = messages
        .iter()
        .position(|m| m.contains("EVENT PLAYER_ALIVE "))
        .unwrap();
    assert!(dead < alive);
    assert!(rt.errors().is_empty());
}

#[test]
fn death_events_ignore_unknown_health_new_players_targets_and_reload() {
    let dir = tempfile::tempdir().unwrap();
    addon(
        dir.path(),
        "Life",
        "",
        r#"
        local f = CreateFrame("Frame")
        f:RegisterEvent("PLAYER_DEAD")
        f:RegisterEvent("PLAYER_ALIVE")
        f:SetScript("OnEvent", function(self, event) print(event) end)
    "#,
    );
    let mut rt = AddonRuntime::new(dir.path());
    rt.load().unwrap();
    let mut s = state();
    s.player.as_mut().unwrap().health = None;
    rt.update_state(s.clone());
    s.player.as_mut().unwrap().health = Some(0);
    rt.update_state(s.clone());
    s.player.as_mut().unwrap().handle = 8;
    s.player.as_mut().unwrap().health = Some(100);
    rt.update_state(s.clone());
    s.target = Some(UnitState {
        handle: 9,
        health: Some(0),
        ..Default::default()
    });
    rt.update_state(s.clone());
    assert!(rt.drain_messages().is_empty());
    s.player.as_mut().unwrap().health = Some(0);
    rt.update_state(s);
    assert_eq!(
        rt.drain_messages()
            .iter()
            .filter(|m| m.contains("PLAYER_DEAD"))
            .count(),
        1
    );
    rt.reload().unwrap();
    rt.update_state(GameState::default());
    rt.update_state(state());
    assert!(rt.drain_messages().is_empty());
}

#[test]
fn debug_addon_observes_known_layer_changes_including_zero_without_fake_reload_events() {
    let dir = tempfile::tempdir().unwrap();
    debug_addon(dir.path());
    let mut rt = AddonRuntime::new(dir.path());
    rt.load().unwrap();
    let mut s = state();
    rt.update_state(s.clone());
    rt.drain_messages();
    assert!(text(&rt).contains("Layer: 0"));
    s.player.as_mut().unwrap().layer = Some(1);
    rt.update_state(s.clone());
    rt.update_state(s.clone());
    let messages = rt.drain_messages();
    assert_eq!(
        messages
            .iter()
            .filter(|m| m.contains("EVENT RAPPELZ_LAYER_CHANGED 1 "))
            .count(),
        1
    );
    assert!(text(&rt).contains("Layer: 1"));
    s.player.as_mut().unwrap().layer = Some(0);
    rt.update_state(s);
    assert_eq!(
        rt.drain_messages()
            .iter()
            .filter(|m| m.contains("EVENT RAPPELZ_LAYER_CHANGED 0 "))
            .count(),
        1
    );
    rt.reload().unwrap();
    assert!(!rt
        .drain_messages()
        .iter()
        .any(|m| m.contains("EVENT RAPPELZ_LAYER_CHANGED ")));
}

#[test]
fn unit_layer_keeps_unknown_nil_known_zero_and_suppresses_identity_discovery_events() {
    let dir = tempfile::tempdir().unwrap();
    addon(
        dir.path(),
        "Layer",
        "",
        r#"
        local f=CreateFrame("Frame")
        f:RegisterEvent("CHECK");f:RegisterEvent("RAPPELZ_LAYER_CHANGED")
        f:SetScript("OnEvent",function(self,event,n) print(event,UnitLayer("player"),UnitLayer("target")) end)
    "#,
    );
    let mut rt = AddonRuntime::new(dir.path());
    rt.load().unwrap();
    let mut s = state();
    s.player.as_mut().unwrap().layer = None;
    rt.update_state(s.clone());
    rt.dispatch(Event::new("CHECK", vec![]));
    assert!(rt.drain_messages()[0].contains("CHECK nil nil"));
    s.player.as_mut().unwrap().layer = Some(0);
    rt.update_state(s.clone());
    assert!(rt.drain_messages().is_empty());
    s.target = Some(UnitState {
        handle: 9,
        layer: Some(0),
        ..Default::default()
    });
    rt.update_state(s.clone());
    rt.dispatch(Event::new("CHECK", vec![]));
    assert!(rt.drain_messages()[0].contains("CHECK 0 0"));
    s.player.as_mut().unwrap().handle = 8;
    s.player.as_mut().unwrap().layer = Some(1);
    rt.update_state(s);
    assert!(rt.drain_messages().is_empty());
    assert!(rt.errors().is_empty());
}

#[test]
fn party_roster_invitation_and_vitals_follow_actual_state_and_do_not_repeat_on_reload() {
    let dir = tempfile::tempdir().unwrap();
    debug_addon(dir.path());
    let mut rt = AddonRuntime::new(dir.path());
    rt.load().unwrap();
    let mut s = state();
    rt.update_state(s.clone());
    rt.drain_messages();
    assert!(text(&rt).contains("Party: ?"));
    s.party = Some(vec![]);
    s.party_members = Some(0);
    rt.update_state(s.clone());
    assert!(!rt
        .drain_messages()
        .iter()
        .any(|m| m.contains("PARTY_MEMBERS_CHANGED")));
    assert!(text(&rt).contains("Party: 0  Member: (none)"));
    s.party_invite = Some("Inviter".into());
    rt.update_state(s.clone());
    rt.update_state(s.clone());
    assert_eq!(
        rt.drain_messages()
            .iter()
            .filter(|m| m.contains("EVENT PARTY_INVITE_REQUEST Inviter"))
            .count(),
        1
    );
    s.party_invite = None;
    s.party_leader = Some(7);
    s.party_members = Some(1);
    s.party = Some(vec![UnitState {
        handle: 9,
        name: "Companion".into(),
        level: Some(1),
        layer: Some(0),
        health: Some(100),
        max_health: Some(100),
        mana: Some(50),
        max_mana: Some(50),
    }]);
    rt.update_state(s.clone());
    let messages = rt.drain_messages();
    assert!(messages
        .iter()
        .any(|m| m.contains("EVENT PARTY_MEMBERS_CHANGED ")));
    assert!(messages
        .iter()
        .any(|m| m.contains("EVENT RAPPELZ_PARTY_INVITE_CLOSED ")));
    assert!(text(&rt).contains("Party: 1  Member: Companion"));
    s.party.as_mut().unwrap()[0].health = Some(0);
    s.party.as_mut().unwrap()[0].mana = Some(0);
    rt.update_state(s.clone());
    rt.update_state(s.clone());
    let messages = rt.drain_messages();
    assert_eq!(
        messages
            .iter()
            .filter(|m| m.contains("EVENT UNIT_HEALTH party1 "))
            .count(),
        1
    );
    assert_eq!(
        messages
            .iter()
            .filter(|m| m.contains("EVENT UNIT_MANA party1 "))
            .count(),
        1
    );
    assert!(!messages
        .iter()
        .any(|m| m.contains("PARTY_MEMBERS_CHANGED") || m.contains("PLAYER_DEAD")));
    rt.reload().unwrap();
    assert!(!rt
        .drain_messages()
        .iter()
        .any(|m| m.contains("EVENT PARTY_MEMBERS_CHANGED ")
            || m.contains("EVENT PARTY_INVITE_REQUEST ")));
    s.party_leader = Some(9);
    rt.update_state(s.clone());
    assert_eq!(
        rt.drain_messages()
            .iter()
            .filter(|m| m.contains("EVENT PARTY_MEMBERS_CHANGED "))
            .count(),
        1
    );
    s.party = Some(vec![]);
    s.party_members = Some(0);
    s.party_leader = None;
    rt.update_state(s);
    assert!(text(&rt).contains("Party: 0  Member: (none)"));
    assert!(rt.errors().is_empty());
}

#[test]
fn party_unit_tokens_preserve_unknown_and_zero_and_reject_unsupported_indices() {
    let dir = tempfile::tempdir().unwrap();
    addon(
        dir.path(),
        "Party",
        "",
        r#"
        local f=CreateFrame("Frame");f:RegisterEvent("CHECK")
        f:SetScript("OnEvent",function()
            print(GetNumPartyMembers(),UnitName("party1"),UnitHealth("party1"),UnitMana("party1"),UnitLayer("party1"),UnitName("party0"),UnitName("party8"),GetPartyInviteName())
        end)
    "#,
    );
    let mut rt = AddonRuntime::new(dir.path());
    rt.load().unwrap();
    rt.dispatch(Event::new("CHECK", vec![]));
    assert!(rt.drain_messages()[0].contains("nil nil nil nil nil nil nil nil"));
    let mut s = state();
    s.party_members = Some(1);
    s.party = Some(vec![UnitState {
        handle: 9,
        name: "Peer".into(),
        health: Some(0),
        mana: Some(0),
        layer: Some(0),
        ..Default::default()
    }]);
    rt.update_state(s);
    rt.dispatch(Event::new("CHECK", vec![]));
    assert!(rt
        .drain_messages()
        .iter()
        .any(|m| m.contains("1 Peer 0 0 0 nil nil nil")));
    assert!(rt.errors().is_empty());
}

#[test]
fn learned_skill_and_jp_events_follow_confirmed_state_once_and_survive_reload() {
    let dir = tempfile::tempdir().unwrap();
    debug_addon(dir.path());
    let mut rt = AddonRuntime::new(dir.path());
    rt.load().unwrap();
    let mut s = state();
    rt.update_state(s.clone());
    rt.drain_messages();
    assert!(text(&rt).contains("Skills: ?  JP: ?"));
    s.learned_skills = Some(Default::default());
    s.job_points = Some(18);
    rt.update_state(s.clone());
    rt.update_state(s.clone());
    let m = rt.drain_messages();
    assert_eq!(
        m.iter()
            .filter(|s| s.contains("EVENT SKILL_LINES_CHANGED "))
            .count(),
        1
    );
    assert_eq!(
        m.iter()
            .filter(|s| s.contains("EVENT RAPPELZ_JP_CHANGED 18 "))
            .count(),
        1
    );
    assert!(text(&rt).contains("Skills: 0  JP: 18"));
    s.learned_skills.as_mut().unwrap().insert(42, 1);
    s.job_points = Some(14);
    rt.update_state(s.clone());
    assert!(text(&rt).contains("Skills: 1  JP: 14"));
    rt.drain_messages();
    s.learned_skills.as_mut().unwrap().insert(42, 2);
    s.job_points = Some(0);
    rt.update_state(s.clone());
    rt.update_state(s);
    let m = rt.drain_messages();
    assert_eq!(
        m.iter()
            .filter(|s| s.contains("EVENT SKILL_LINES_CHANGED "))
            .count(),
        1
    );
    assert_eq!(
        m.iter()
            .filter(|s| s.contains("EVENT RAPPELZ_JP_CHANGED 0 "))
            .count(),
        1
    );
    rt.reload().unwrap();
    assert!(!rt.drain_messages().iter().any(
        |s| s.contains("EVENT SKILL_LINES_CHANGED ") || s.contains("EVENT RAPPELZ_JP_CHANGED ")
    ));
    assert!(text(&rt).contains("Skills: 1  JP: 0"));
    assert!(rt.errors().is_empty());
}

#[test]
fn skill_apis_distinguish_unknown_from_unlearned_and_known_zero_jp() {
    let dir = tempfile::tempdir().unwrap();
    addon(
        dir.path(),
        "Skills",
        "",
        r#"
        local f=CreateFrame("Frame");f:RegisterEvent("CHECK")
        f:SetScript("OnEvent",function() print(GetNumKnownSkills(),GetSkillLevel(42),GetSkillLevel(99),GetJobPoints()) end)
    "#,
    );
    let mut rt = AddonRuntime::new(dir.path());
    rt.load().unwrap();
    rt.dispatch(Event::new("CHECK", vec![]));
    assert!(rt.drain_messages()[0].contains("nil nil nil nil"));
    let mut s = state();
    s.learned_skills = Some(Default::default());
    s.job_points = Some(0);
    rt.update_state(s.clone());
    rt.dispatch(Event::new("CHECK", vec![]));
    assert!(rt.drain_messages().iter().any(|m| m.contains("0 0 0 0")));
    s.learned_skills.as_mut().unwrap().insert(42, 2);
    rt.update_state(s);
    rt.dispatch(Event::new("CHECK", vec![]));
    assert!(rt.drain_messages().iter().any(|m| m.contains("1 2 0 0")));
    assert!(rt.errors().is_empty());
}

#[test]
fn failures_are_attributed_and_isolated_and_instruction_limit_cannot_be_swallowed() {
    let dir = tempfile::tempdir().unwrap();
    addon(
        dir.path(),
        "Loop",
        "",
        "assert(pcall == nil and xpcall == nil); while true do end",
    );
    addon(
        dir.path(),
        "Good",
        "",
        "local f = CreateFrame('Frame', 'GoodFrame'); f:SetText('still running')",
    );
    addon(
        dir.path(),
        "Broken",
        "",
        "error('intentional test failure')",
    );
    let mut rt = AddonRuntime::new(dir.path());
    rt.load().unwrap();
    assert!(text(&rt).contains("still running"));
    assert!(rt
        .errors()
        .iter()
        .any(|s| s.contains("Loop") && s.contains("instruction budget")));
    assert!(rt
        .errors()
        .iter()
        .any(|s| s.contains("Broken") && s.contains("intentional test failure")));
}

#[test]
fn restricted_libraries_and_no_native_loader() {
    let dir = tempfile::tempdir().unwrap();
    addon(
        dir.path(),
        "Restricted",
        "",
        r#"
        for _, name in ipairs({"io", "os", "debug", "package", "require", "dofile", "loadfile", "loadstring", "getfenv", "setfenv", "coroutine", "newproxy"}) do assert(_G[name] == nil, name) end
        assert(string.dump == nil)
        assert(_VERSION == "Lua 5.1")
        print("restricted")
    "#,
    );
    let mut rt = AddonRuntime::new(dir.path());
    rt.load().unwrap();
    assert!(rt.errors().is_empty(), "{:?}", rt.errors());
    assert!(rt.drain_messages()[0].contains("restricted"));
}

#[test]
fn dependency_order_cycles_missing_disabled_optional_and_load_on_demand() {
    let dir = tempfile::tempdir().unwrap();
    addon(
        dir.path(),
        "A",
        "## Dependencies: Z\n## OptionalDeps: Absent",
        "print('A')",
    );
    addon(dir.path(), "Z", "## LoadOnDemand: 1", "print('Z')");
    addon(dir.path(), "Cycle1", "## Dependencies: Cycle2", "");
    addon(dir.path(), "Cycle2", "## Dependencies: Cycle1", "");
    addon(dir.path(), "Missing", "## Dependencies: Absent", "");
    addon(
        dir.path(),
        "Demand",
        "## LoadOnDemand: 1",
        "print('Demand')",
    );
    let mut rt = AddonRuntime::new(dir.path());
    rt.load().unwrap();
    let messages = rt.drain_messages();
    assert!(messages[0].contains("Z"));
    assert!(messages[1].contains("A"));
    assert!(rt
        .addons()
        .iter()
        .find(|a| a.manifest.name == "Cycle1")
        .unwrap()
        .error
        .is_some());
    assert!(rt
        .addons()
        .iter()
        .find(|a| a.manifest.name == "Missing")
        .unwrap()
        .error
        .is_some());
    assert_eq!(
        rt.addons()
            .iter()
            .find(|a| a.manifest.name == "Demand")
            .unwrap()
            .status,
        "On demand"
    );
    rt.request_load("Demand").unwrap();
    assert!(rt.drain_messages().iter().any(|s| s.contains("Demand")));
    rt.set_enabled("Z", false).unwrap();
    rt.reload().unwrap();
    assert_eq!(
        rt.addons()
            .iter()
            .find(|a| a.manifest.name == "A")
            .unwrap()
            .status,
        "Blocked"
    );
}

#[test]
fn manifest_traversal_and_unsupported_xml_are_clear_errors() {
    let dir = tempfile::tempdir().unwrap();
    addon(dir.path(), "Traversal", "", "");
    fs::write(
        dir.path().join("Interface/AddOns/Traversal/Traversal.toc"),
        "## Interface: 30300\n../outside.lua",
    )
    .unwrap();
    addon(dir.path(), "Xml", "", "");
    let xml = dir.path().join("Interface/AddOns/Xml");
    fs::write(xml.join("Xml.toc"), "## Interface: 30300\nXml.xml").unwrap();
    fs::write(
        xml.join("Xml.xml"),
        "<Ui><Frame name='X' inherits='NotImplemented'/></Ui>",
    )
    .unwrap();
    let mut rt = AddonRuntime::new(dir.path());
    rt.load().unwrap();
    assert!(rt.errors().iter().any(|s| s.contains("traversal")));
    assert!(rt.errors().iter().any(|s| s.contains("inherits")));
}

#[test]
fn cyclic_saved_data_fails_safely_and_preserves_previous_atomic_file() {
    let dir = tempfile::tempdir().unwrap();
    addon(
        dir.path(),
        "Saved",
        "## SavedVariables: Settings",
        "Settings = Settings or { value = 1 }",
    );
    let mut rt = AddonRuntime::new(dir.path());
    rt.load().unwrap();
    rt.save().unwrap();
    let script = dir.path().join("Interface/AddOns/Saved/Saved.lua");
    fs::write(
        &script,
        "Settings = Settings or {}; Settings.self = Settings",
    )
    .unwrap();
    rt.reload().unwrap();
    assert!(rt
        .save()
        .unwrap_err()
        .to_string()
        .contains("account settings"));
    fs::write(
        &script,
        "assert(Settings.value == 1 and Settings.self == nil)",
    )
    .unwrap();
    let mut fresh = AddonRuntime::new(dir.path());
    fresh.load().unwrap();
    assert!(fresh.errors().is_empty(), "{:?}", fresh.errors());
}

#[test]
fn memory_and_widget_limits_stop_addon_and_keep_host_alive() {
    let dir = tempfile::tempdir().unwrap();
    addon(
        dir.path(),
        "Memory",
        "",
        "local x = string.rep('a', 32 * 1024 * 1024)",
    );
    addon(
        dir.path(),
        "Widgets",
        "",
        "for i=1,300 do CreateFrame('Frame') end",
    );
    let mut rt = AddonRuntime::new(dir.path());
    rt.load().unwrap();
    assert_eq!(rt.errors().len(), 2, "{:?}", rt.errors());
    assert!(rt.errors().iter().any(|s| s.contains("widget limit")));
}

#[test]
fn saved_variables_keep_account_character_and_realm_scopes_and_numeric_keys() {
    let dir = tempfile::tempdir().unwrap();
    addon(
        dir.path(),
        "Scopes",
        "## SavedVariables: AccountDB\n## SavedVariablesPerCharacter: CharacterDB",
        r#"
        AccountDB = AccountDB or { count = 0, [2] = "two", [true] = false }
        CharacterDB = CharacterDB or { count = 0 }
        assert(AccountDB[2] == "two" and AccountDB[true] == false)
        local f = CreateFrame("Button", "Counter", UIParent)
        f:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)
        f:SetScript("OnClick", function()
            AccountDB.count = AccountDB.count + 1
            CharacterDB.count = CharacterDB.count + 1
            print(AccountDB.count, CharacterDB.count)
        end)
    "#,
    );
    let mut rt = AddonRuntime::new(dir.path());
    rt.set_identity("account", "realm1", Some("one")).unwrap();
    rt.pointer(640.0, 480.0, 10.0, 10.0, true);
    assert!(rt.drain_messages()[0].ends_with("1 1"));
    rt.set_identity("account", "realm1", Some("two")).unwrap();
    rt.pointer(640.0, 480.0, 10.0, 10.0, true);
    assert!(rt.drain_messages()[0].ends_with("2 1"));
    rt.set_identity("account", "realm1", Some("one")).unwrap();
    rt.pointer(640.0, 480.0, 10.0, 10.0, true);
    assert!(rt.drain_messages()[0].ends_with("3 2"));
    rt.set_identity("account", "realm2", Some("one")).unwrap();
    rt.pointer(640.0, 480.0, 10.0, 10.0, true);
    assert!(rt.drain_messages()[0].ends_with("4 1"));
    rt.save().unwrap();
    assert!(rt.errors().is_empty(), "{:?}", rt.errors());
}

#[test]
fn disabled_preferences_survive_process_restart_and_interface_is_checked() {
    let dir = tempfile::tempdir().unwrap();
    addon(
        dir.path(),
        "Disabled",
        "",
        "print('must not run after disable')",
    );
    addon(dir.path(), "Version", "", "");
    fs::write(
        dir.path().join("Interface/AddOns/Version/Version.toc"),
        "## Interface: 99999\nVersion.lua",
    )
    .unwrap();
    let mut rt = AddonRuntime::new(dir.path());
    rt.load().unwrap();
    assert!(rt
        .addons()
        .iter()
        .find(|a| a.manifest.name == "Version")
        .unwrap()
        .error
        .as_ref()
        .unwrap()
        .contains("99999"));
    rt.set_enabled("Disabled", false).unwrap();
    let mut fresh = AddonRuntime::new(dir.path());
    fresh.load().unwrap();
    assert_eq!(
        fresh
            .addons()
            .iter()
            .find(|a| a.manifest.name == "Disabled")
            .unwrap()
            .status,
        "Disabled"
    );
    assert!(fresh.drain_messages().is_empty());
}

#[test]
fn event_failure_is_reported_once_and_cannot_stop_other_addons() {
    let dir = tempfile::tempdir().unwrap();
    addon(
        dir.path(),
        "BadEvent",
        "",
        r#"
        local f = CreateFrame("Frame")
        f:RegisterEvent("CHAT_MSG_SYSTEM")
        f:SetScript("OnEvent", function() error("event handler failure") end)
    "#,
    );
    addon(
        dir.path(),
        "GoodEvent",
        "",
        r#"
        local f = CreateFrame("Frame")
        f:RegisterEvent("CHAT_MSG_SYSTEM")
        f:SetScript("OnEvent", function(self, event, text) print(text) end)
    "#,
    );
    let mut rt = AddonRuntime::new(dir.path());
    rt.load().unwrap();
    for _ in 0..2 {
        rt.dispatch(Event::new("CHAT_MSG_SYSTEM", vec!["delivered".into()]));
    }
    assert_eq!(rt.errors().len(), 1);
    assert!(
        rt.errors()[0].contains("BadEvent") && rt.errors()[0].contains("event handler failure")
    );
    assert_eq!(rt.drain_messages().len(), 2);
}

#[test]
fn cast_events_keep_real_arguments_and_do_not_repeat_on_snapshots_or_reload() {
    let dir=tempfile::tempdir().unwrap();
    addon(dir.path(),"Casts","",r#"
        local f=CreateFrame("Frame")
        f:RegisterEvent("RAPPELZ_SKILL_EVENT")
        f:SetScript("OnEvent",function(self,event,id,phase,caster,amount)
            print(id .. ":" .. phase .. ":" .. caster .. ":" .. amount)
        end)
    "#);
    let mut rt=AddonRuntime::new(dir.path());rt.load().unwrap();
    let mut s=state();rt.update_state(s.clone());assert!(rt.drain_messages().is_empty());
    for (sequence,phase,amount) in [(1,"CASTING",0),(2,"CANCEL",0),(3,"CASTING",0),(4,"FIRE",3500)] {
        s.skill_event=Some(rz_addon::SkillEventState{sequence,skill:3212,phase:phase.into(),caster:7,amount});
        rt.update_state(s.clone());rt.update_state(s.clone());
        let lines=rt.drain_messages();assert_eq!(lines.len(),1);assert!(lines[0].contains(&format!("3212:{phase}:7:{amount}")));
    }
    rt.reload().unwrap();rt.update_state(s);assert!(rt.drain_messages().is_empty());assert!(rt.errors().is_empty());
}

#[test]
fn xml_inline_handlers_visibility_scale_and_anchors_use_same_hit_test_as_renderer() {
    let dir = tempfile::tempdir().unwrap();
    addon(dir.path(), "Layout", "", "");
    let path = dir.path().join("Interface/AddOns/Layout");
    fs::write(path.join("Layout.toc"), "## Interface: 30300\nLayout.xml").unwrap();
    fs::write(
        path.join("Layout.xml"),
        r#"
        <Ui><Frame name="Panel"><Size x="100" y="50"/>
          <Anchors><Anchor point="CENTER" relativeTo="UIParent" relativePoint="CENTER"/></Anchors>
          <Frames><Button name="Child"><Size x="20" y="10"/>
            <Anchors><Anchor point="TOPLEFT" x="5" y="-5"/></Anchors>
            <Scripts><OnClick>print("hit"); Panel:Hide()</OnClick></Scripts>
          </Button></Frames>
          <Scripts><OnLoad>self:SetScale(2)</OnLoad></Scripts>
        </Frame></Ui>
    "#,
    )
    .unwrap();
    let mut rt = AddonRuntime::new(dir.path());
    rt.load().unwrap();
    assert!(rt.errors().is_empty(), "{:?}", rt.errors());
    let child = rt
        .draw_widgets(640.0, 480.0)
        .into_iter()
        .find(|w| w.widget.name.as_deref() == Some("Child"))
        .unwrap();
    assert_eq!(
        (
            child.rect.x,
            child.rect.y,
            child.rect.width,
            child.rect.height
        ),
        (230.0, 200.0, 40.0, 20.0)
    );
    assert!(rt.pointer(640.0, 480.0, 235.0, 205.0, true));
    assert!(rt.drain_messages()[0].contains("hit"));
    assert!(rt.draw_widgets(640.0, 480.0).is_empty());
}
