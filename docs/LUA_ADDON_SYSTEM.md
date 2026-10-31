# Lua addon system

Verified development cycle: 2026-10-30. Implementation: `shared/rz-addon`; native adapter: `RappelzRust/client/src/addons.rs`. This is a working, deliberately limited addon foundation. It does not implement the complete WoW 3.3.5 UI/API.

Follow-up: [LIVE_SERVER_VALIDATION.md](LIVE_SERVER_VALIDATION.md) records a passing native online session, 13 Lua integration tests, actual event traces and preserved original data. RappelzDebug now displays MP/currency and keeps bounded event counters plus the last 32 observed state snapshots. Historical offline-only notes below are retained as the first cycle's verification record.

## Run and install

```powershell
cargo build --workspace --release
target/release/rz-extractor.exe addons --out client-data/RappelzRust
target/release/RappelzRust.exe -data client-data/RappelzRust -demo
```

The `addons` installer writes missing RappelzDebug files and preserves existing files. New `rz-extractor client` installations include it automatically. To develop an already installed copy, edit its files under `<installation>/Interface/AddOns/RappelzDebug/`; installing again does not replace local edits.

The native client discovers `<installation>/Interface/AddOns/<Name>/<Name>.toc`. The source example is `RappelzRust/Interface/AddOns/RappelzDebug/`. Use the **AddOns** button or F9 for the manager. It shows installed metadata, load status, dependency/compatibility errors and enable controls. Enable changes apply when **Recharger l'interface** is pressed. In-world chat intercepts `/reload` locally before network chat parsing. Reload saves settings, replaces Lua/UI states and emits addon/login lifecycle events while keeping the world and network connection.

## Runtime and dependencies

[`mlua` 0.11.6](https://docs.rs/crate/mlua/0.11.6) supplies vendored Lua 5.1; `lua51`, `vendored`, and `error-send` are enabled. The last feature allows Lua errors to flow through `anyhow`; it does not make the Lua states thread-safe. Lua remains on the render thread. The workspace builds on `x86_64-pc-windows-msvc`, Rust 1.99.0. `mlua` declares Rust 1.85 minimum and an MIT license; [upstream](https://github.com/mlua-rs/mlua) documents its supported runtimes. Bundled Lua has its own [MIT license](https://www.lua.org/license.html). The crate's Cargo.lock records exact dependency versions. These dependencies do not import third-party emulator code.

Each loaded addon has a separate Lua VM, globals and UI namespace. Dependencies establish load order and availability, but do **not** share Lua globals between addons. There is no `require`, cross-addon exported API or Lua-side `LoadAddOn` yet. The host manager can explicitly request a load-on-demand addon; this currently reloads the entire addon UI. A required load-on-demand dependency loads before its dependent. A broken or incompatible dependency blocks its dependents; unrelated addons continue to load.

Host callbacks use protected Lua calls. A script failure is attributed to its addon and retained with the Lua error/trace. That addon stops receiving callbacks and its widgets disappear until reload; other addons continue. Discovery/manifest errors appear in the manager error area even when no valid manifest could be listed. `print` goes to the client log and in-world system chat, with bounded output.

## Limits and restricted environment

- 64 discovered manifests, 32 loaded addons, 128 sources per manifest; 1 MiB per source and 8 MiB startup source total per addon.
- 8 MiB Lua allocation limit per addon. Widget/event/text data is separately bounded on the Rust side, outside that Lua allocation count.
- 200,000 Lua instructions shared across an addon's startup files. The budget resets for each dispatched event and pointer callback. Event callbacks for one addon share the event budget. A hook checks every 1,000 instructions.
- 256 widgets including UIParent, 128 event registrations per widget, 4,096 bytes per widget text, 32 levels of XML/persisted table nesting.
- No `io`, `os`, `package`, `debug`, coroutine library, native loader, file/process/network APIs, dynamic code loaders, environment mutation or bytecode loading. Startup chunks are text only. `string.dump` is removed.
- Lua-side `pcall`/`xpcall` are intentionally absent: in Lua 5.1 a loop can catch hook quota errors and continue indefinitely. Protection is supplied by the Rust host. Addons depending on those Lua functions need a port.
- Instruction hooks do not preempt native C library calls or provide an OS-process security boundary. This is a restricted in-process scripting runtime, not a complete hostile-code containment system. A dedicated worker/process and wall-clock supervision remain future work.

Paths must be relative normal components, without drive prefixes, alternate-stream separators or traversal. Canonical containment rejects source files that escape the addon directory, including through symlinks/junctions. XML DTD/entities are rejected. Addons receive no filesystem API.

## TOC subset

UTF-8 with an optional BOM, CRLF/LF, `#` comments, `## key: value` metadata and ordered source paths are accepted. Metadata keys are case-insensitive and retained. Required dependencies accept `Dependencies` or `RequiredDeps`; `OptionalDeps` is supported. Lists accept commas/whitespace. Names and SavedVariables identifiers use ASCII Lua identifiers. `Interface` must equal **30300**; incompatible addons stay blocked. This interface tag identifies the supported addon convention, not the Rappelz executable's version.

`LoadOnDemand: 1`, `SavedVariables` and `SavedVariablesPerCharacter` work. Source extensions are `.lua` and `.xml`; unsupported extensions fail clearly. There is no XML Include/Script-file loader: enumerate Lua and XML sources in their TOC order. Unknown metadata is retained without inventing behavior for it. Dependencies and file lookup are exact names on case-sensitive filesystems.

## State and event connection

`WorldState::addon_state` supplies the current player, selected target, known currency and inventory revision. The app updates Lua-visible state after each accepted network event and after world input. HP/MP packets, level/property updates, changed inventory packets and target selection/removal produce deltas. Repeated identical snapshots do not re-emit change events. A NOTICE received through the server chat packet generates `CHAT_MSG_SYSTEM` with its actual message. Login, entering-world and logout reflect real client lifecycle transitions, including the explicit offline demo lifecycle.

APIs return `nil` for unavailable units/fields. The client waits for a level/money update rather than presenting startup defaults as server data. Party membership now comes from the opt-in authoritative roster: nil until confirmation, zero when solo, and actual other-member count/`party1`–`party7` units when grouped. See [PARTY_VALIDATION.md](PARTY_VALIDATION.md) for real damage, hidden-layer party vitals, invitations, chat and disconnect cleanup. Target mana/max mana come from known ENTER/HPMP values, including genuine zeros; see [COMBAT_VALIDATION.md](COMBAT_VALIDATION.md). Offline demo values remain actual offline WorldState values, separate from online evidence.

Implemented event names/arguments are in [LUA_API_COMPATIBILITY.md](LUA_API_COMPATIBILITY.md). Pet, summon, other casting effects and other gameplay events must be connected when their state models exist; the runtime does not generate them artificially.

[SKILL_LEARNING_VALIDATION.md](SKILL_LEARNING_VALIDATION.md) adds actual private learned lists and own EXP_UPDATE JP to the production adapter: GetNumKnownSkills, GetSkillLevel, GetJobPoints, SKILL_LINES_CHANGED and RAPPELZ_JP_CHANGED. RappelzDebug displays/logs those values. Forty native comparison checks cover learning/refusal, privacy and saved character re-entry. The native K panel uses the same learning helper as the driver; physical K/button activation remains a manual check.

[SKILL_CASTING_VALIDATION.md](SKILL_CASTING_VALIDATION.md) adds RAPPELZ_SKILL_EVENT(id, phase, caster, nominal_amount) from accepted same-layer CASTING/CANCEL/FIRE packets. Confirmed FIRE vitals update before callbacks; duplicate HPMP/snapshots/reload do not replay changes. RappelzDebug logs phase/caster/amount, and the 34 native checks cover the original old-MP callback negative control. The current effect is a local self-MP profile; original 3212 corpse semantics remain unresolved.

## UI/XML subset

Widgets are Rust-backed records rendered by macroquad. `CreateFrame`, frames, buttons, font strings, solid-color textures, basic status bars and tooltip-shaped panels work. Child font strings and textures can be created from Lua. Anchors support nine standard points, a single active point, relative widget/table/global-name anchors, WoW-style upward-positive Y offsets and inherited scale. Relative widgets must already exist; this prevents layout cycles. Forward anchor references, templates, multiple simultaneous anchors and automatic size solving are unsupported.

`SetScript` accepts `OnEvent`, `OnClick`, `OnLoad`, `OnEnter` and `OnLeave`. Register events through `RegisterEvent`. XML `<OnLoad>` runs after the frame's children have been constructed. `OnLoad` set later via Lua is stored, but is not automatically invoked by `SetScript`. Buttons are tested against the same resolved rectangles used by the renderer. Hidden ancestors suppress child drawing/hit testing. Pointer input captured by an addon blocks world camera/selection actions; the manager is modal for world/HUD input.

The strict XML subset accepts `<Ui>` and these widgets, `<Size x="..." y="..."/>`, `<Anchors><Anchor point="..." relativeTo="..." relativePoint="..." x="..." y="..."/></Anchors>`, `<Frames>` children, and `<Scripts>` with inline bodies or `function="GlobalName"` references. Widget attributes: `name`, `parent`, `hidden`, `text`. Unsupported tags/attributes, templates and source extensions report errors. WoW's full schema (`Layers`, `AbsDimension`, `Offset`, inheritance, virtual templates etc.) is not implemented.

## SavedVariables

Only declared variables persist. Storage is non-executable tagged JSON representing Lua nil/booleans/finite numbers/UTF-8 strings/tables, including numeric and boolean table keys. Functions/userdata/threads, cyclic tables and invalid values are rejected. Shared acyclic table references are serialized as independent copies. Lua 5.1 numbers have double precision; large integer identifiers/currency/JP may lose precision beyond 2^53.

Account settings: `WTF/Account/<hex-account>/SavedVariables/<Addon>.json`.
Character settings: `WTF/Account/<hex-account>/SavedVariables/Characters/<hex-realm>/<hex-character>/<Addon>.json`.
Enable preferences: the account directory's `AddOns.json`.

Hex encodes complete identity bytes so account/realm/character strings cannot create path components. Online account names are normalized to uppercase; the realm scope includes authentication endpoint and selected realm ID. Offline mode uses a separate `offline` identity. Settings restore before startup scripts and `ADDON_LOADED`; account/character changes save the old scope first. Saves occur on reload, logout and graceful close. Temporary files are written/synced in the destination directory and atomically replaced using `tempfile::NamedTempFile::persist`. Each file is limited to 1 MiB and 20,000 serialized nodes. Multi-file saves are not a transaction. Abrupt process termination cannot run logout handlers.

## Verification

`cargo test -p rz-addon` executes **22 integration tests** against the vendored Lua 5.1 runtime, including the actual TOC/Lua/XML example. Coverage includes actual cast-phase arguments without snapshot/reload duplication, private learned skill/JP state, initial empty/zero discovery and deduplication across reload, party roster/invitation/vitals and initial known-zero readiness, known death/revival transitions even within one rendered frame, known-zero versus unknown layers, layer-event deduplication without discovery/new-identity/reload fabrication, suppression on unknown health/new identity/target/reload, lifecycle/state arguments, change deduplication, unknown-vitals suppression, click hit testing, reload, account/character/realm separation, numeric keys, enable persistence, dependencies/cycles/load-on-demand, version rejection, traversal, unsupported XML, isolated startup/event errors, memory/instruction/widget limits and preservation of previously saved data after a cyclic-table save failure.

[REVIVAL_VALIDATION.md](REVIVAL_VALIDATION.md) verifies PLAYER_DEAD and PLAYER_ALIVE against actual lethal damage/automatic revival with two native clients. RappelzDebug also records actual target HP/MP in its bounded diagnostics. These events follow known health transitions of the same player and are not synthesized for initial discovery or UI reload.

[LAYER_VALIDATION.md](LAYER_VALIDATION.md) verifies read-only UnitLayer and RAPPELZ_LAYER_CHANGED through real two-client 0→1→0 transitions and saved layer-1 character re-entry. RappelzDebug displays player/target layers; the native adapter retains actual login/WARP/ENTER layer data and clears previous-layer entities/target/NPC UI. A separate client test exercises this production adapter without graphics. The addon cannot request a layer change.

The first native Windows demo was visually inspected: a real mouse click changed `Clicks: 0` to `Clicks: 1`, reload retained `1`, and graceful close saved it. The subsequent real local-server session also passes all event/value/reload/logout assertions, with a JSON report and native screenshot; see [LIVE_SERVER_VALIDATION.md](LIVE_SERVER_VALIDATION.md). Automated keyboard injection did not establish keyboard shortcut/chat-command behavior; those paths still need a physical-keyboard check.
