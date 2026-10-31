# Authoritative character combat stats and private character sheet

Verified **2026-10-31**, with **35/35 independent native checks**, two Windows x64 clients, an old-server negative control and direct SQL checkpoints. The C character sheet now shows the player's current physical attack, defence and configured movement speed, alongside vitals, exact JP, money and separately labeled raw job-base attributes. Lua and RappelzDebug read the same accepted state.

## Defect and supported behavior

Login previously advertised attack `10 + 2 × level` and defence `5 + level`, although melee already used `12 + 4 × level` and `4 + 2 × level`. A level-2 player therefore received **14/7** while combat used **20/8**. Level-up did not refresh that packet. One `World::publish_stats` path now reads the actual player entity at login, player-level change and an authorized own QUERY. Configured speed is retained independently of motion; an idle player still reports **120** in this scenario. Existing opaque fields, including the legacy second attack slot, stay unchanged.

This corrects agreement with the **current provisional combat implementation**. It does not establish original-game attack, defence, equipment, passive or JLV-bonus formulas. The C panel separates **Base du métier** from the three supported effective fields; raw base attributes are not advertised as final modified attributes.

The existing full `SC_STAT_INFO` frame is **110 bytes**: a seven-byte header, handle, 98 stats bytes and a trailing type byte. The shared decoder exposes only right-hand physical attack at packet **+0x1f**, defence **+0x27**, and move speed **+0x43**. It requires a nonzero owner, full type 0, nonnegative attack/defence, speed 0..255 and exact payload length. Foreign handles, delta packets and malformed snapshots cannot overwrite accepted own state. Unknown state is `None`/Lua `nil`, rather than guessed zero.

The sheet's **Actualiser** button and opening path use the existing own QUERY. The server returns a stat snapshot only for self; ordinary visible-peer queries retain their public ENTER/WEAR behavior. QUERY also checks exact session GUID/account ownership, preventing a replaced session from publishing private stats or mutating the replacement. No new opcode, SQL field or migration is introduced.

`GetPlayerCombatStat("attack"|"defence"|"move_speed")` is a read-only Rappelz Lua extension. Unsupported names and unknown state return nil. `RAPPELZ_COMBAT_STATS_CHANGED` has no arguments: the complete new tuple is readable during the callback. Initial discovery, changed values and fresh world reentry emit it; identical refreshes do not. UI reload follows the existing new-host discovery behavior. Logout uses the existing last-state-readable lifecycle.

## Real two-client run

An isolated MariaDB **3331**, auth **24615** and copied world **24514** use cloned schemas. Original database/auth/world **3310/4615/4514** stay active. New GM-0 cloned characters **34/35** start at:

| Field | Owner | Witness |
|---|---:|---:|
| Player level / EXP / JP | 2 / 799 / 7 | 1 / 0 / 0 |
| HP / MP | 120 / 60 | 100 / 50 |
| Money | 1000 | 2000 |
| Job / JLV | 100 / 1 | 200 / 1 |
| Position | 9440 / 7227 | 9440 / 7247 |
| Attack / defence / speed | 20 / 8 / 120 | 16 / 6 / 120 |

1. Both clients open the rendered sheet, issue two production own refreshes and two visible-peer probes. Each receives exactly **three own snapshots** and one Lua stat callback. Neither receives the other's private packet. Direct SQL confirms complete seeds before combat is released.
2. Owner attacks unchanged monster 651 at one additive clone-only spawn **1499**. Both clients receive the same **12 damage events**, including retaliation and one kill. The normal committed reward yields **level 3, EXP 4, JP 10, money 1012 (+12), HP/MP 140/70**. Owner receives stats **24/10/120**, after the public level update. Witness sees peer level 3/maxima 140/70 while keeping **16/6/120**, level 1, EXP/JP 0/0 and money 2000.
3. Both save normally to the lobby. Independent SQL verifies complete profiles and every unrelated settings key before permitting real reentry. Handles change owner **2→4**, witness **1→3**. Raw login vitals/positions and private rewards match the saved data; old targets/actions/level notifications are not replayed. Each issues one own refresh, then exits normally through a second lobby save.
4. Owner receives **six** private snapshots and **three** stat callbacks; witness **five** snapshots and **two** callbacks. Their Lua login/entry/logout counts are all **two**. Both clients and copied world PID **18164** exit **0**. A final independent SQL read matches the saved profiles. The owner sheet and RappelzDebug were visually inspected with attack 24, defence 10, speed 120 and money 1012.

The same new client against the retained previous server **7afad041…** rejects its raw **14/7/120** and **12/6/120** packets before sending gameplay requests; SQL seeds remain exact. Both negative-control clients exit 1 and the copied server shuts down normally. A first negative-control harness assertion expected both clients to report the direct mismatch; one instead stopped on its peer's failure. That failed harness attempt is retained. The corrected control explicitly checks both raw wrong tuples and permits the peer-failure propagation; it passes without a production-code change.

The native driver exercises production helpers, networking, Lua and rendering. Physical C-key and mouse-click injection are not claimed as acceptance evidence. No new crash test, terrain-quality comparison or original SFrame interoperability test is claimed in this cycle; earlier reports retain those separate scopes.

## Additional resource investigation

Read-only investigation covers **125,282 archive-index entries**, all **69 named RDB resources**, and **89 candidate text files / 400,576 bytes** without skipped candidates. No named JobLevelBonus dataset or server promotion-script body was identified within that boundary. This is not proof that equivalent opaque data/constants are absent elsewhere.

Archived `db_job.rdb` has **45 × 292-byte rows**, SHA-256 `3d17273218396345ca42f4f11fd98f9ac586ddefc44042461684373c7173d7ad`, in `data.006` at **744707388**. All **42 common loose/archive rows are byte-identical**. Extra jobs **105, 115, 125** have no matching BasicStat row; they are not activated using invented fallback attributes. The validated job layout supplies static stat IDs, depth, transition prerequisites and successor IDs, not a per-JLV stat-growth vector.

The pinned primary [JobLevelBonus loader](https://github.com/NGemity/RZEmulator/blob/38ceb2c6065fabf6ff4ba71d52f955f362c6c839/Chihiro/src/Globals/ObjectMgr.cpp#L955) and [bonus calculation](https://github.com/NGemity/RZEmulator/blob/38ceb2c6065fabf6ff4ba71d52f955f362c6c839/Chihiro/src/Globals/ObjectMgr.cpp#L1314) use a separate server SQL dataset whose coefficients are not supplied here. Original NPC rows identify `NPC_JobChange_contact()` for IDs 1004/2004/4004/6004/7004/7032, and `NPC_Tutorial_Job_Change_contact()` for 3019. They identify entry points, not their missing server behavior. The reference [independent job/JLV/JP/history setters](https://github.com/NGemity/RZEmulator/blob/38ceb2c6065fabf6ff4ba71d52f955f362c6c839/Chihiro/src/Scripting/XLua.cpp#L385) do not establish promotion resets, JP policy or history backfill. No effective JLV bonus, promotion/reset or inheritance rule is guessed. These public references do not prove version equivalence with the supplied SFrame. This cycle adds resource/reference inspection, not a new native SFrame decompilation.

## Checks, installation and preservation

- Combined workspace: **196 Rust pass**, one exhaustive model scan ignored, **24 Lua tests included**; **123 Python pass**.
- Publication branches: Core **101 Rust**, client **147 Rust / one ignored**, **123 Python each**, optimized AMD64 builds.
- Added regressions: two protocol, three server, two client adapter, one actual Lua test and five independent comparer/mutation tests. They reject stale/foreign ownership, opaque-field changes, wrong wire offsets/lengths, invalid/delta/private packets, duplicate/partial callbacks, old values, early/replayed level updates, private gains, mixed combat traces, partial SQL, unrelated-key loss and forged negative-control success.
- Original data/WTF pass **9 preservation checks**, copied existing data plus exact additive spawn pass **9**. Original high-water marks stay account/character **33**, items **4**, world creature **1498**. Final new fixtures are offline GM-0. No original row, datadir, schema or progression resource is replaced.
- Logical backup: **18,917,362 bytes**, SHA-256 `ccbee7579339b78ff8ba3cabe6af1cbb97d5b1f74ba37e6b3b124bbfcaa7b932`.
- Exact native-tested installed client: `344b07289ace67471a63fdd71914cefe856c588e0e8a61d202a37173c73d326a`; copied tested world: `30afb4b7d8974f630620df9b953a2d88684fc37ae84f8a0ceaa3d2ce505ebcb9`. The primary running world remains `3ecfcb979e6dbffa4709945ca6403a931b2ef5e4cad743a79dfc064ce125a37c`. Installed extractor embedding the updated addon: `33256c1a962a5a8531ff44e59e28e18821b716df610a2ac77eec8f0bc9192962`; fresh export matches all three source files and a second export preserves an existing user-file sentinel.

The predecessor client and installed addon code are backed up. WTF remains unchanged. Original assets, profiles, executable images, cloned datadir, SQL backups, credentials, screenshots and raw analysis remain ignored. Native evidence is retained in `client-data/validation/2026-10-31-character-stats-4642bec8/`, with `verified/` and `negative/` the accepted sessions. Resource ledgers are retained in `client-data/audit/2026-11-01-job-growth-3280926f/`; its directory date is a label, while this investigation ran on 2026-10-31. Scoped test services are stopped after publication checks; files are retained. Both GitHub branches receive only source/tests/docs.

Next: obtain verified job bonus coefficients/promotion server behavior, define explicit cumulative/wide EXP persistence and migration, then extend effects, equipment and summons with their own preserved-data tests.
