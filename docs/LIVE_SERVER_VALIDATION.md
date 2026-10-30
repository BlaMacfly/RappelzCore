# Local server / native addon validation

Date: **2026-10-30**, Europe/Paris. This follow-up validates the graphical Rust client against the existing MariaDB, authserver and worldserver, using actual packets and the production world-to-addon adapter. It supersedes the previous cycle's offline-only validation boundary.

## Preservation and service startup

Before any service started, the stopped database directory was copied to `client-data/validation/2026-10-30-db5edac4/mariadb-before`. All **1,105 files / 275,016,573 bytes** were verified by SHA-256; the manifest is `mariadb-before.sha256.json` in that run directory. The original client WTF directory and installed RappelzDebug files were also copied there. The backup captures the original stopped state, including recovery logs; it is not a live file-copy backup.

The existing MariaDB 13.0.2 binaries and `client-data/mariadb/my.ini` were reused. The server completed its normal InnoDB/table crash recovery and reported ready. A **17,218,730-byte logical dump** of all three databases was then saved as `logical-before.sql` before creating the validation account. No bootstrap SQL, DROP/CREATE script, content reimport, password reset or schema migration was run.

Services use loopback only; distributed/local configuration files were preserved:

| Service | Endpoint | Configuration |
|---|---|---|
| MariaDB | `127.0.0.1:3310` | Existing my.ini; bind address overridden on launch. |
| authserver | `127.0.0.1:4615` | Existing authserver.conf; AC_BIND_IP override. |
| worldserver | `127.0.0.1:4514` | Existing worldserver.conf, existing server-data/dbc; AC_BIND_IP override. |

The world server loaded 42 jobs, 1,497 NPCs, 407 teleport locations, 7,940 monster definitions, 34,682 items, one vendor and one existing monster placement. Services were left running locally after verification. Process/listener details are saved in the ignored run directory's `services.json`.

## Real session and negative control

A dedicated new account `LUADB5EDAC4` (id 3) and character `LuaProbe` (guid 3) isolate the test from TEST/Aria and JOUEUR2/Brann. The account was created via the existing world console and given GM level 3 for the controlled commands. No existing credentials were changed. `rz-bot` first verified the full auth → realm → character creation → world → logout path (`bootstrap-bot.log`).

The native client now supports an explicit regression runner:

```powershell
target/release/RappelzRust.exe -data client-data/RappelzRust `
  -login <LUA-test-account>:<password> -enter <Lua-test-character> `
  -validate-addons <absolute-report.json>
```

The runner refuses non-loopback auth endpoints and requires LUA*/Lua* account/character prefixes. It waits for actual world/loading completion, then uses the existing NetCmd path for `.level`, `.money 37`, `.additem 244201 1`, target selection and return-to-lobby. It uses the renderer's actual addon rectangles for a click and the native host's normal UI reload. It never fabricates packets, game-state snapshots or addon events. It modifies the selected test character; this is not a read-only command. Level-stat expectations intentionally reflect the existing provisional server formulas. Item 244201 must exist and a target must be nearby.

The runner records event lines printed by RappelzDebug itself, reads the actual UI labels and fails on host/addon/network errors, wrong values, missing events or timeouts. JSON records each completed stage, event counts and failure details. A PNG is captured after reload, while the character is still in the world. Successful runs return to the lobby before exiting. Nonzero exit status indicates failure.

The first run was an intentional baseline against the old server: **failed** in Level (`baseline.json`, `client-baseline.log`). The client correctly received level 2, but HP/MP remained 100/100 and 50/50, with no corresponding change events. This demonstrated the defect independently of the fix.

## Fixes

1. **Missing level-change HP/MP synchronization.** Both the GM command and XP level-up path changed server stats but omitted TS_SC_HPMP. `World::set_player_level` now applies the existing formulas, updates the cached ENTER creature stats, and broadcasts level and HP/MP to the owner and observing players. The owner also receives the level property. Both callers share this function. A regression test decodes emitted owner/observer packets and a future ENTER to assert the same values. The GM path was exercised online; XP awards use the tested common helper but a live monster kill was not part of this run.
2. **Fabricated unknown-target mana event.** The real trace showed UNIT_MANA for a selected target whose mana remained unknown. Runtime deltas now compare absent units with the equivalent unknown `(nil, nil)` tuple. Selecting an unknown-vitals target produces no health/mana event until data actually changes. A test covers unknown → known mana, including a valid value of zero and repeated-snapshot deduplication.
3. **Insufficient debug visibility.** RappelzDebug now subscribes to all implemented lifecycle/state/system events, displays mana and money, and retains bounded event counts/recent state snapshots in SavedVariables. This supplies direct Lua-side evidence rather than inferring dispatch from host state alone. The installed copy was backed up before updating it.

## Final result

`final.json`: **success=true**, no addon errors, client exit code 0. Accepted live state changed from level 3 / HP 140 / MP 70 / money 37 to **level 4 / HP 160/160 / MP 80/80 / money 74**. The selected server-provided target was **Tortue de Ullr**. The click counter changed from 1 to 2 and stayed 2 after UI reload, while world stats remained unchanged. A real return-to-lobby emitted PLAYER_LOGOUT and the database shows the test character offline with saved values.

| Lua event | Observed final-run count |
|---|---:|
| ADDON_LOADED | 3 (initial unscoped load, scoped load, reload) |
| PLAYER_LOGIN / PLAYER_ENTERING_WORLD | 2 each (entry and UI reload) |
| PLAYER_LOGOUT | 1 |
| UNIT_HEALTH | 3 (player initial, level update, known target) |
| UNIT_MANA | 2 (player initial and level update; no unknown-target event) |
| PLAYER_LEVEL_UP | 1 |
| PLAYER_MONEY | 2 |
| BAG_UPDATE | 2 (existing test inventory received, then added item) |
| PLAYER_TARGET_CHANGED | 1 |
| CHAT_MSG_SYSTEM | 4 |

Evidence lives under the ignored `client-data/validation/2026-10-30-db5edac4/`: `baseline.json`, `after-level-fix.json`, `final.json`, `final.png`, corresponding client logs, bootstrap log, backup manifests, SQL snapshots and `preservation.json`. Dumps/backup data include private local account material and should remain excluded from Git/distribution.

## Data preservation result

`preservation.json`: **success=true**. Data INSERT statements for the original accounts/access/realm-character counts and original characters/settings/inventory/items match the pre-test logical dump. All existing world data INSERTs (seven populated tables) also match. This checks whole selected records, not only the displayed health/currency columns. Realm status, uptime and other operational metadata are expected to change while services run and are outside that equality claim.

The read-only comparison tool is `tools/testing/compare_preserved_data.py`. It handles MariaDB 13 multiline INSERTs and refuses empty/unrecognized baseline account, character or world comparisons. Three Python tests cover multiline parsing, empty-baseline rejection and detection of an altered record. No preservation conclusion relies on empty hashes.

Aria remains level 15 / money 1500 / HP 380 / MP 190; Brann remains level 1 / money 0 / HP 100 / MP 100. Both remain offline. Only the dedicated test account/character and its two new item rows were added; the test rows were deliberately retained, not deleted.

## Build and limits

- Workspace Rust tests: **78 passed, 0 failed, 1 intentionally ignored**.
- Snapshot-comparison Python tests: **3 passed**.
- Optimized workspace build succeeds; addon Clippy passes with warnings denied. Broader Clippy retains pre-existing style/large-enum warnings.
- Updated client installed at `client-data/RappelzRust/RappelzRust.exe`; SHA-256 `067D0F312D769667E3A776FF86D58ECD03C6723FDD45848ED3329C9232C4D10C`. Previous executable retained as `client-data/backups/RappelzRust-3CFBE003C98DD339.exe`.
- This validates the Rust client and local Rust server together. It does not prove original SFrame interoperability, retail Epic identification, full combat balance or unfinished pet/quest/skill systems.
- The scripted runner exercises the production selection/reload paths, not physical keyboard input. F9 and typed `/reload` remain a separate physical-keyboard validation item.
