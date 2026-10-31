# Development status

Audit/development date: **2026-10-30**. This records one verified continuation of the existing workspace, not completion of an MMORPG ecosystem.

**Latest follow-up (2026-10-31):** [Player death/revival validation](REVIVAL_VALIDATION.md) passes with two native clients: real lethal retaliation, automatic revival at the start point, correct observer position/rendering beyond the corpse timeout, Lua death/alive events and logout. Two native negative controls reproduce the observer defects before the fixes; the final comparison passes 26 conditions. **91 Rust tests + 10 Python tests pass**, one Rust test ignored. All records that predate this cycle, including the earlier fixtures, world content and installed WTF files compare unchanged.

The previous [corpse/respawn cycle](RESPAWN_VALIDATION.md) verified two kills, same-handle respawn and 22 identical attacks, fixing hidden QUERY disclosure and cross-layer combat. Its 85 Rust/6 Python counts remain historical.

The earlier [two-client combat validation](COMBAT_VALIDATION.md) fixed discarded target mana and stale fresh-ENTER vitals, and established actual XP-driven level 2→3, JP rewards and logout. Its 81 Rust/5 Python counts remain historical. The [single-client session](LIVE_SERVER_VALIDATION.md) is documented separately.

## Workspace and recovered progress

Initial recursive inventory found 16 Cargo members, 74 Rust source files, original `Rappelz NoLimits` data, converted client data, SQL files, reverse-engineering notes and ignored historical runtime/decompiler outputs. The added `rz-addon` makes 17 workspace members. Rust 1.99.0/Cargo 1.99.0 target `x86_64-pc-windows-msvc`.

The initial development folder has an **unborn `master`**, with sources untracked. Existing server/client histories were recovered. On the user's explicit publication request, separate `codex/validate-lua-core` and `codex/validate-lua-client` worktrees were created, preserving each branch's server/client split and existing license/logo/history. The first Lua/combat cycle was published as `841ef3c` / `4674ce9`, then the respawn cycle as `60f93c3` / `19a0ce5` on `RappelzCore` / `RappelzRust`; this follow-up continues those worktrees. The combined local folder remains intact; its Git state is not evidence of missing source progress. Retained artifacts corroborate earlier development but do not identify its authoring tool independently.

Original assets, databases and converted files remain under ignored `Rappelz NoLimits/`, `client-data/`, `target/`. No original game code/assets or public emulator code was copied into the implementation.

## Previous claims checked

| Claim | Current evidence / limitation |
|---|---|
| SFrame x86 analyzed | Fresh PE parsing and REA PE-resource inspection confirm x86; existing docs/decompiler text recovered. Fresh Ghidra overview exceeded the 300s MCP deadline; no new native decompilation obtained. |
| 125,282 archive entries | Real-client Rust test passes; independent Node archive statistics confirm all eight shards with zero gaps/overlaps. |
| Rust reader / JPG / DDS | Existing reader preserved; real-client signature/extraction test passes with supplied assets present. |
| MPQ crypto | Known-table-key and crypto roundtrip tests pass. No new external StormLib execution performed. |
| DBC implementation | Existing WDBC reader/writer roundtrip passes; RDB/DBC readers and server importer exist. |
| AzerothCore schemas imported | 439 base SQL tables verified (22 auth, 108 characters, 309 world); representative upstream source matches, historical import evidence and fresh local SQL/dump checks. No schema reimport was run. |
| SRP6 started | Existing verifier calculation/check/account storage implemented and 3 database-crate tests pass. Wire login uses Rappelz RSA/AES password exchange, not an SRP proof handshake. |
| Windows x64 client compiled | Full optimized workspace build succeeds. Generated client PE machine verified separately before delivery. Native demo renders supplied converted assets. |
| Prior live multiplayer/gameplay | Fresh native login, inventory/target validation, two-client real combat/XP and repeated kill/respawn pass. Original SFrame login remains untested. |

## Changes in this cycle

- Added `shared/rz-addon`, using a real vendored Lua 5.1 VM per addon. Implemented TOC discovery, metadata/interface checks, dependency order/cycles, enable preferences, host load-on-demand, restricted execution, error attribution and limits.
- Connected game-state APIs and lifecycle/health/mana/level/inventory/target/currency/NOTICE events to the existing native client. Unavailable data returns nil; no invented party or game action behavior.
- Implemented renderer-backed frames/buttons/font strings/solid textures/basic status bars, strict XML subset, pointer handlers, anchoring/visibility/scale, native manager and UI reload. Added scoped atomic SavedVariables with safe non-executable serialization.
- Added real RappelzDebug TOC/Lua/XML example and a preserving `rz-extractor addons --out ...` installer. New client conversions include the addon automatically.
- Extended MPQ mounting to expansion archives and corrected sparse numeric patch ordering. Existing resources and schema files were preserved.
- Added static SFrame inspection script and the documentation index below. README version wording is corrected to keep launcher and binary identities distinct.
- Follow-up: corrected server level-change HP/MP synchronization and cached ENTER stats through a shared GM/XP helper; suppressed unknown-target vitals events; added an opt-in native online runner and Lua-side diagnostic traces. See LIVE_SERVER_VALIDATION for the negative control and passing final run.
- Latest follow-up: gate QUERY and player attacks on current announced visibility; stop cross-layer monster retaliation. Extend the existing native runner with an optional second kill after corpse LEAVE/respawn ENTER, and include every packet category in trace capacity accounting. See RESPAWN_VALIDATION for the failing native baseline and corrected pair.
- Death/revival follow-up: clear revived client corpse markers, refresh continuing teleport observers and recalculate old/new visibility immediately; clear remaining aggressor targets on revival. Add a guarded native scenario reusing the existing host/trace, real PLAYER_DEAD/PLAYER_ALIVE transitions and RappelzDebug target-health diagnostics. See REVIVAL_VALIDATION for two native negative controls and the corrected pair.

## Verification

| Check | Result |
|---|---|
| Baseline `cargo test --workspace --all-targets` | 63 passed, 0 failed, 1 ignored; supplied original assets present. |
| Final combined workspace tests | **91 passed, 0 failed, 1 ignored**, including 15 Lua integration tests plus client vitals/trace/revival, server level/visibility/lifecycle/teleport and MPQ tests. The comparison tools add 10 passing Python tests. |
| `cargo build --workspace --release` | All six executables build for native Windows x64. |
| Clippy on addon/client/extractor/MPQ all targets | Completed; no rz-addon warnings. Existing style/large-enum warnings remain in older crates/client code. |
| `rz-extractor verify` | **125,212 files identical, 0 errors**. Excludes original DB resources converted separately; this is not complete generated-DBC semantic validation. |
| Lua runtime/example | All 15 tests pass on actual Lua 5.1, including known death/revival transitions within one frame, no fabricated lifecycle events, state deltas, unknown-vitals handling, error isolation, UI click/layout, dependencies, restrictions/limits, reload and scoped persistence. |
| Native Windows UI | Demo frame/data visibly correct. Real mouse click increments 0 to 1. Manager opens, lists Loaded metadata, reloads without restarting, retains clicks and re-emits lifecycle. Graceful close writes clicks=1 JSON. |
| Keyboard automation | Injected key events did not establish the shortcut/chat paths; physical-keyboard check remains. Shortcut switched from F10 to F9 to avoid Windows menu-key handling. |
| Live DB/online addon session | Passed on existing local services with a dedicated test account/character; all 11 implemented event names observed, real values asserted, UI reload/persistence/logout checked. Original records and world content compare unchanged against backup. |
| Two native clients / XP combat | Passed: 12 identical attack records, damage HPMP equality, kill-driven level 2→3/XP 4/JP 3, live target HP/MP, fresh queries and leave/logout. Witness XP/JP remains zero. |
| Two native clients / respawn and second kill | Passed: 22 identical attack records, hidden-corpse QUERY rejected, same-handle respawn, second reward once, XP 9/JP 6 and final target HP 80/140, MP 70/70. All pre-cycle records including earlier test characters remain unchanged. |
| Two native clients / player death and revival | Passed: real lethal retaliation, HP 0→100/MP 50, automatic return to 8330/7167, continued observer visibility after 3.5 seconds before any query, real Lua death/alive/target-health events, fresh ENTER, logout and saved offline state. No XP/JP reward or level change. |

Screenshot: local ignored `client-data/shot_lua_addon_final.png` (native offline scene, updated installed executable, persisted click count still 1 after process restart). The demo player values are actual offline state, not evidence of an online server response. Automated tests supply explicit state fixtures; the native production adapter reads existing accepted packets.

The latest optimized client is installed into `client-data/RappelzRust/RappelzRust.exe`, SHA-256 `343BFEFE7937433F75DDA3B27C7C2EF3ABC71951BBB74C7D17C58476A3C4D430`. Its preceding client/server executables and installed addon files are preserved under `client-data/validation/2026-10-31-revival-f624b911/`; older backups also remain. All six release executable PE machine fields are `0x8664` (AMD64). The latest two-client screenshots/reports are under that run's `final/`; earlier combat/respawn and single-client evidence remains available separately.

## Existing systems and gaps

Server code already implements authentication/account handling, realm/character flow, world sessions, visibility/movement/chat, console/GM commands, persistence, basic inventory/equipment/NPC shop and provisional melee/rewards. It is not a complete server: skills, quests, parties/guilds, pet/summon gameplay, instances, loot/economy and accurate combat formulas need work. Database mappings should be reviewed before broader content imports.

Client already has macroquad rendering, MPQ/DBC resources, original UI support, terrain, models/animation, network/lobby/world, audio, basic inventory/NPC interactions. Water, effects, grass, sky/fog, accurate attachments/lighting and full gameplay interfaces remain. The new addon framework has an explicit subset: no full WoW API/XML compatibility, shared addon globals, templates, secure actions or image-backed addon textures. In-process Lua instruction limits cannot preempt all C library work.

The original executable's exact official Epic/patch remains **unresolved**: Windows VERSIONINFO 9.1.0.0, launcher 1.3.60, source9.5.2 paths and wire literal 201507080 are distinct observations. No exact matching public server is proven. Do not use the README's former v1.3.60 shorthand as an executable/Epic identity.

## Documentation / restart index

- [CLIENT_REVERSE_ENGINEERING.md](CLIENT_REVERSE_ENGINEERING.md): hash-bound identity, metadata conflicts, REA limits and retained artifacts.
- [SERVER_REFERENCES.md](SERVER_REFERENCES.md): primary public sources, pinned revisions, licenses, protocol discrepancies.
- [NETWORK_PROTOCOL.md](NETWORK_PROTOCOL.md): implemented transport and evidence boundaries; links original detailed notes.
- [DATABASE_ARCHITECTURE.md](DATABASE_ARCHITECTURE.md): logical databases, actual mappings and migration/import limitations.
- [RESOURCE_FORMATS.md](RESOURCE_FORMATS.md): archive/format/conversion behavior and integrity limits.
- [LUA_ADDON_SYSTEM.md](LUA_ADDON_SYSTEM.md): installation, execution model, XML, persistence and tests.
- [LUA_API_COMPATIBILITY.md](LUA_API_COMPATIBILITY.md): actual APIs/events and intentional differences.
- [NEXT_TASKS.md](NEXT_TASKS.md): prioritized concrete restart point.
- [RESPAWN_VALIDATION.md](RESPAWN_VALIDATION.md): repeated combat, hidden queries, lifecycle synchronization and preserved data.
- [REVIVAL_VALIDATION.md](REVIVAL_VALIDATION.md): player death, automatic revival, observer rendering/position, Lua transitions and preserved data.

Keep `docs/re/` as the detailed existing evidence record. Newly retained REA files under `client-data/proto-re/` are ignored and local, with their incomplete Ghidra coverage explicitly recorded.
