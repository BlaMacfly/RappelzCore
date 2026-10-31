# Lua API compatibility matrix

Verified through 2026-10-31. `## Interface: 30300` preserves a familiar manifest convention; it does not promise automatic WoW addon compatibility. Lua 5.1 executes real scripts. Separate addon VMs deliberately do not share globals.

Core lifecycle/state events were observed in the actual native/local-server session; see [LIVE_SERVER_VALIDATION.md](LIVE_SERVER_VALIDATION.md). Death/alive transitions and target HP 0→100 were subsequently verified with two native clients in [REVIVAL_VALIDATION.md](REVIVAL_VALIDATION.md). Unknown `(nil, nil)` target vitals do not generate fabricated health/mana changes. Missing level-change HP/MP server broadcasts were corrected for both GM and XP callers.

## Game APIs

| API | Status / data source |
|---|---|
| `UnitName("player"/"target"/"party1"–"party7")` | Current WorldState name or accepted party roster excluding self; nil for absent/unsupported unit. Party order follows server join order. |
| `UnitHealth`, `UnitHealthMax` | Player login/HPMP state, selected entity's known HP or accepted party roster values. Nil if not known; known zero remains zero. |
| `UnitMana`, `UnitManaMax` | Player login/HPMP state, known target ENTER/HPMP values or accepted party roster. Unknown values remain nil; known zero is preserved. Two-client combat/groups verified. |
| `UnitLevel` | Player level/property packets, target entity level or party roster; nil until known. |
| `UnitLayer("player"/"target"/"party1"–"party7")` | Read-only Rappelz extension: actual accepted login/WARP, target ENTER or party roster layer. Nil for unknown/absent units/layers; known zero is preserved. Two-client separation/re-entry/groups verified. |
| `GetMoney()` | Currency/property updates; nil until a value is known. |
| `GetNumPartyMembers()` | Accepted number of other group members: nil until the server confirms state, zero when solo, 1–7 when grouped. Real two-client group validation passes. |
| `GetPartyInviteName()` | Read-only Rappelz extension; actual pending inviter name, otherwise nil. No invitation/group action APIs are exposed. |
| `GetTime()` | Monotonic time since this addon host started; survives UI reload. |
| `print(...)` | Bounded client log and in-world chat output. |
| Other WoW/game action APIs | Absent; calling one raises the normal attributed Lua nil-function error. No combat/inventory command API is exposed. |

## Events

| Event | Trigger / arguments |
|---|---|
| `ADDON_LOADED` | Successful addon startup; addon name. Already loaded addons also receive later addons' notifications. |
| `PLAYER_LOGIN` | Transition into a player state; replayed after UI reload with an active player. |
| `PLAYER_ENTERING_WORLD` | Follows login/active-world UI reload. |
| `PLAYER_LOGOUT` | Player state removed or graceful client shutdown, with the last player state still readable. |
| `PLAYER_DEAD` | Same player's known positive HP becomes nonpositive; no arguments; new state readable. Initial/unknown health and target changes do not trigger it. |
| `PLAYER_ALIVE` | Same player's known nonpositive HP becomes positive; no arguments. Describes implemented automatic revival, not full WoW ghost semantics. Login/UI reload do not fabricate it. |
| `UNIT_HEALTH` | Known health/max-health tuple changes; `"player"` or `"target"`. |
| `UNIT_MANA` | Mana/max-mana tuple changes; unit token. |
| `PLAYER_LEVEL_UP` | Known player level increases; new level. Initial level discovery is not a fabricated level-up. |
| `RAPPELZ_LAYER_CHANGED` | Rappelz extension: same player's known layer changes; numeric new layer. Initial discovery, another handle and UI reload do not fabricate it. No layer-changing API is exposed. |
| `BAG_UPDATE` | Accepted inventory contents change; `0` for the current single logical bag. |
| `PLAYER_TARGET_CHANGED` | Selected target handle changes, including removal. |
| `CHAT_MSG_SYSTEM` | Server NOTICE chat received; actual message. Local `print` does not synthesize this event. |
| `CHAT_MSG_PARTY` | Actual group chat received; message, sender. Delivered to accepted group members only. |
| `PARTY_INVITE_REQUEST` | Accepted pending invitation appears/changes; inviter name. Identical snapshots do not repeat it. |
| `PARTY_MEMBERS_CHANGED` | Accepted roster identity/order or leader changes; no arguments. Initial confirmed empty roster, vitals-only changes and UI reload do not fabricate a membership change. |
| `RAPPELZ_PARTY_STATE_READY` | Rappelz extension: first accepted party state of the current player session; no arguments. Distinguishes known solo from unknown. |
| `RAPPELZ_PARTY_INVITE_CLOSED` | Rappelz extension: actual pending invitation removed by acceptance/refusal/expiry/revocation; no arguments. |
| `PLAYER_MONEY` | Currency snapshot changes, including first discovery. |
| Rappelz pet/summon/skill/combat-specific events | Not connected; implement from real future gameplay state transitions. |

Handler convention: `function(self, event, ...)`. Legacy `this`, `event`, `arg1` globals are not supplied. Hidden frames can still receive registered events, as registration is independent of visibility.

## UI bindings

| Feature/API | Supported subset |
|---|---|
| `CreateFrame(type, name, parent)` | Frame, Button, FontString, Texture, StatusBar, GameTooltip. Default parent UIParent. Names local to each addon VM. Fourth/template argument rejected. |
| `CreateFontString(name, layer)`, `CreateTexture(name, layer)` | Child widgets; layer argument accepted but draw order is creation order; templates rejected. |
| `SetSize`, `SetWidth`, `SetHeight`, `GetWidth`, `GetHeight` | Explicit dimensions with finite/clamped sizes; no automatic layout sizing. |
| `SetScale` | Inherited scale; bounded output scale. |
| `SetPoint(point, relativeWidget, relativePoint, x, y)` | Nine anchors, one active point; relative object must have been created earlier. Full five-argument form; abbreviated numeric WoW overloads unsupported. |
| `Show`, `Hide`, `IsShown` | Local visibility; hidden parents suppress child rendering/hit testing. |
| `GetName`, `SetText`, `GetText`, `SetTextColor` | Bounded UTF-8 labels; multiline drawing. Font/template selection and advanced wrapping unsupported. |
| `SetColorTexture` | Solid color only; no image-backed `SetTexture` yet. |
| `SetMinMaxValues`, `SetValue` | Basic status-bar fill; no full WoW status-bar styling. |
| `EnableMouse` | Pointer capture/hit testing. |
| `RegisterEvent`, `UnregisterEvent`, `UnregisterAllEvents` | Per-widget bounded event subscriptions. |
| `SetScript` | OnEvent, OnClick, OnLoad, OnEnter, OnLeave; nil removes handler. Other script types error. XML invokes OnLoad after children. |
| Buttons | LeftButton OnClick, real native mouse path and renderer-shared hit test verified. |
| Tooltip behavior | GameTooltip panel type only; no SetOwner/AddLine API. |
| XML | Ui/widget/Size/Anchors/Frames/Scripts subset documented in LUA_ADDON_SYSTEM. Unknown elements/attributes fail. |
| Templates, secure/protected actions, animation groups, OnUpdate, texture files, drag/resize, full XML inheritance | Not implemented. |

## Manifest, runtime and persistence differences

| Feature | Status |
|---|---|
| TOC metadata, ordered files, BOM/comments/CRLF | Implemented. |
| Required/optional dependencies and cycle checks | Implemented; dependency globals are isolated. |
| Interface compatibility | Exactly 30300; manager reports incompatible addons. |
| Enable/disable | Account preferences; apply on reload. |
| LoadOnDemand | Host manager request, implemented via complete UI reload. No Lua loader API. |
| SavedVariables / SavedVariablesPerCharacter | Declared, scoped, safe tagged JSON; restore before startup, atomic per-file saves. |
| Filesystem/native network/OS access | Not exposed. |
| Lua pcall/xpcall, coroutine, dynamic load, bytecode | Deliberately absent; see execution-limit rationale. |
| In-game manager and reload | Native overlay; world/network retained. `/reload` interception compiled; native reload button verified. |

See [LUA_ADDON_SYSTEM.md](LUA_ADDON_SYSTEM.md) for limits, storage formats and the tested demonstration.
