RappelzDebugDB = RappelzDebugDB or { clicks = 0, loads = 0 }
RappelzDebugDB.events = RappelzDebugDB.events or {}
RappelzDebugDB.recent = RappelzDebugDB.recent or {}
RappelzDebugCharacterDB = RappelzDebugCharacterDB or { clicks = 0 }
local loggedIn = false

local function known(value)
    if value == nil then return "?" end
    return tostring(value)
end

function RappelzDebugRefresh()
    if RappelzDebugInfo == nil then return end
    RappelzDebugInfo:SetText(
        "Player: " .. (UnitName("player") or "(not in world)") ..
        "\nLevel: " .. known(UnitLevel("player")) ..
        "\nLayer: " .. known(UnitLayer("player")) ..
        "\nHP: " .. known(UnitHealth("player")) .. " / " .. known(UnitHealthMax("player")) ..
        "\nMP: " .. known(UnitMana("player")) .. " / " .. known(UnitManaMax("player")) ..
        "\nMoney: " .. known(GetMoney()) ..
        "\nParty: " .. known(GetNumPartyMembers()) .. "  Member: " .. (UnitName("party1") or "(none)") ..
        "\nSkills: " .. known(GetNumKnownSkills()) .. "  JP: " .. known(GetJobPoints()) ..
        "\nTarget: " .. (UnitName("target") or "(none)") ..
        "\nTarget level: " .. known(UnitLevel("target")) ..
        "\nTarget layer: " .. known(UnitLayer("target")) ..
        "\nTarget HP: " .. known(UnitHealth("target")) .. "/" .. known(UnitHealthMax("target")) ..
        " MP: " .. known(UnitMana("target")) .. "/" .. known(UnitManaMax("target")) ..
        "\nClicks: " .. known(RappelzDebugDB.clicks) ..
        "  Login: " .. tostring(loggedIn))
end

function RappelzDebugOnLoad(self)
    self:RegisterEvent("ADDON_LOADED")
    self:RegisterEvent("PLAYER_LOGIN")
    self:RegisterEvent("PLAYER_ENTERING_WORLD")
    self:RegisterEvent("PLAYER_LOGOUT")
    self:RegisterEvent("PLAYER_DEAD")
    self:RegisterEvent("PLAYER_ALIVE")
    self:RegisterEvent("RAPPELZ_LAYER_CHANGED")
    self:RegisterEvent("PARTY_MEMBERS_CHANGED")
    self:RegisterEvent("RAPPELZ_PARTY_STATE_READY")
    self:RegisterEvent("SKILL_LINES_CHANGED")
    self:RegisterEvent("RAPPELZ_JP_CHANGED")
    self:RegisterEvent("RAPPELZ_SKILL_EVENT")
    self:RegisterEvent("PARTY_INVITE_REQUEST")
    self:RegisterEvent("RAPPELZ_PARTY_INVITE_CLOSED")
    self:RegisterEvent("CHAT_MSG_PARTY")
    self:RegisterEvent("UNIT_HEALTH")
    self:RegisterEvent("UNIT_MANA")
    self:RegisterEvent("PLAYER_LEVEL_UP")
    self:RegisterEvent("PLAYER_MONEY")
    self:RegisterEvent("BAG_UPDATE")
    self:RegisterEvent("PLAYER_TARGET_CHANGED")
    self:RegisterEvent("CHAT_MSG_SYSTEM")
    RappelzDebugRefresh()
end

function RappelzDebugOnEvent(self, event, ...)
    local arg = select(1, ...)
    RappelzDebugDB.events[event] = (RappelzDebugDB.events[event] or 0) + 1
    local snapshot = {
        event = event, arg = arg, time = GetTime(),
        skillPhase = event == "RAPPELZ_SKILL_EVENT" and select(2, ...) or nil,
        skillCaster = event == "RAPPELZ_SKILL_EVENT" and select(3, ...) or nil,
        skillAmount = event == "RAPPELZ_SKILL_EVENT" and select(4, ...) or nil,
        name = UnitName("player"), level = UnitLevel("player"), layer = UnitLayer("player"),
        hp = UnitHealth("player"), maxHP = UnitHealthMax("player"),
        mp = UnitMana("player"), maxMP = UnitManaMax("player"),
        money = GetMoney(), target = UnitName("target"),
        party = GetNumPartyMembers(), party1 = UnitName("party1"), partyHP = UnitHealth("party1"),
        partyInvite = GetPartyInviteName(),
        skills = GetNumKnownSkills(), jp = GetJobPoints(),
        targetHP = UnitHealth("target"), targetMaxHP = UnitHealthMax("target"),
        targetMP = UnitMana("target"), targetMaxMP = UnitManaMax("target"), targetLayer = UnitLayer("target")
    }
    table.insert(RappelzDebugDB.recent, snapshot)
    if #RappelzDebugDB.recent > 32 then table.remove(RappelzDebugDB.recent, 1) end
    RappelzDebugDB.lastPlayer = snapshot
    if event == "RAPPELZ_SKILL_EVENT" then
        print("SKILL " .. known(arg) .. " " .. known(snapshot.skillPhase) .. " caster=" .. known(snapshot.skillCaster) .. " amount=" .. known(snapshot.skillAmount))
    end
    print("EVENT " .. event .. " " .. known(arg) ..
        " HP=" .. known(snapshot.hp) .. "/" .. known(snapshot.maxHP) ..
        " MP=" .. known(snapshot.mp) .. "/" .. known(snapshot.maxMP) ..
        " money=" .. known(snapshot.money) .. " targetHP=" .. known(snapshot.targetHP) .. " layer=" .. known(snapshot.layer) ..
        " party=" .. known(snapshot.party) .. " partyHP=" .. known(snapshot.partyHP) .. " skills=" .. known(snapshot.skills) .. " jp=" .. known(snapshot.jp))
    if event == "ADDON_LOADED" and select(1, ...) == "RappelzDebug" then
        RappelzDebugDB.loads = (RappelzDebugDB.loads or 0) + 1
        print("ADDON_LOADED; Lua " .. _VERSION)
    elseif event == "PLAYER_LOGIN" then
        loggedIn = true
        print("PLAYER_LOGIN: " .. (UnitName("player") or "?"))
    elseif event == "PLAYER_LOGOUT" then
        loggedIn = false
    end
    RappelzDebugRefresh()
end

function RappelzDebugOnClick(self, button)
    RappelzDebugDB.clicks = (RappelzDebugDB.clicks or 0) + 1
    RappelzDebugCharacterDB.clicks = (RappelzDebugCharacterDB.clicks or 0) + 1
    RappelzDebugRefresh()
    print("Click " .. tostring(RappelzDebugDB.clicks) .. " (" .. button .. ")")
end
