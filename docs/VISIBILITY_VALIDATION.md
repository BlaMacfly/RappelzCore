# Movement and visibility: two native clients

Verified **2026-10-31**, Europe/Paris, on the existing local auth/world services. Two native processes complete an actual multi-waypoint walk into visibility and back out of range. Both final exit codes are **0**, and an independent comparison passes **25/25 checks**.

## Defect reproduced

A newly visible player received ENTER and wear data but no ongoing MOVE. The observing client created a stationary entity at the supplied current position. In the native negative control, the mover reaches approximately **9050/7167**, while the witness remains frozen at **8706.8/7167**. No witness QUERY is sent before arrival; a refresh cannot mask the defect. Both baseline processes fail and their reports are retained.

Monster snapshots had a related problem: ENTER supplied the interpolated current position, but the accompanying MOVE replayed the entire original route, including waypoints already reached. Unit tests reproduce both problems before the correction.

The server now sends **ENTER → wear → remaining MOVE** for a moving player, and **ENTER → remaining MOVE** for a moving monster. A shared route projection omits reached segments, including zero-length prefix points, uses the current packet clock and emits no MOVE for a finished/stationary route. Live monster chase/return broadcasts use the same projection. Tests cover player QUERY and new visibility, exact waypoint boundaries, zero-length prefix points, finished routes and zero speed. Native monster catch-up is not claimed exercised by this player scenario.

## Preserved data

A consistent logical backup of all three databases was taken before creating fixtures: **18,896,862 bytes**, SHA-256 `c6a672d8cb54ec5e5b8a397fad5ed090c22d61b396d20c81b225cff6b7d328ca`. Services remain loopback-only (3310/4615/4514). There was no schema/content import or existing-character reset.

Two new GM-0 accounts/characters were created through the normal flows:

| Role | Character | GUID | Offline fixture |
|---|---|---:|---|
| Mover | LuaMov6732 | 10 | Level 1, XP/JP/money 0, HP 100, MP 50; position 8330/7167. |
| Witness | LuaWit6732 | 11 | Same stats; position 9230/7167. |

The fixture's 900-unit separation exceeds the existing **540-unit visibility range**. Preparation/reset is restricted by these new GUIDs/accounts, exact names and offline status. Each native process has separate addon/WTF directories and reads the installed MPQs through a junction. Only these new records were reset between baseline and final attempts.

The final preservation comparison passes **8/8 checks**: all pre-cycle account/access/realm-character records through ID 9, character/settings/inventory through GUID 9, items through GUID 4, and all existing world INSERTs match the backup. Installed WTF hashes also match. Operational realm/uptime metadata is outside this comparison. All nine earlier characters, including prior combat/respawn/revival fixtures, remain unchanged. New records and backups are retained.

## Native result

- Both clients start with the peer absent. The mover follows **8330→8500→9050**, at y=7167, using the same local prediction/command path as normal keyboard/mouse movement. No GM teleport or synthetic received event is used.
- The witness receives the mover's actual ENTER at **8696/7167**, after the first waypoint is already behind it. The following MOVE contains only **9050/7167**, at speed 120/layer 0 with the current clock. Its packet order follows ENTER.
- Without a QUERY, the witness's production WorldState reaches approximately **9049.84/7167**, selects the correct mover and displays real target HP 100/100 and MP 50/50 through Lua. The mover also sees the stationary witness.
- The return route contains **8330/7167**. Both clients receive the same authoritative return broadcast, with matching handle/time/speed/layer/points. Each receives range LEAVE for the other.
- The witness clears the entity and selected target. It then explicitly queries the hidden mover and observes a one-second rejection window: no ENTER and no recreated target. Lua PLAYER_TARGET_CHANGED occurs twice: selection, then range removal.
- After the predicted return, the mover separately queries its own handle and checks the server's actual ENTER at **8330/7167** before logging out. This prevents local prediction alone from being mistaken for authoritative arrival.
- Both characters remain level 1, HP 100, MP 50, XP/JP/money 0. No combat, level change, death or WARP occurs. Both logout events occur, and database rows finish offline at **8330/7167** and **9230/7167** respectively.

Native HUD/target screenshots were inspected; packet reports and recorded world positions establish the movement assertions. The witness camera is not a complete on-screen framing of the entire 540-unit interest range.

## Reproduction and checks

Use the existing online-validation host with **`-validate-visibility <plan.json>`**, mutually exclusive with the other validation flags. Each plan contains role `mover` or `witness`, the Lua* peer name and a shared fresh attempt directory. Loopback and dedicated LUA*/Lua* identities remain required. Use the documented offline fixtures and separate data roots. Marker files coordinate readiness only; movement/visibility assertions use real packets, client state and Lua output.

```powershell
python tools/testing/compare_visibility_clients.py <attempt-directory>
python -m unittest discover -s tools/testing -p test_*.py -v
cargo test --workspace --all-targets --locked --offline -- --test-threads=4
```

The read-only comparator rejects stationary late observers, replayed completed waypoints, MOVE before ENTER, retained out-of-range targets, wrong rewards and a merely predicted return without server confirmation. It has an independent passing sample and mutation tests. The shared bounded trace now includes MOVE routes/clock/order and a movement-only capacity regression.

The combined workspace passes **94 Rust tests** (one intentionally ignored model scan) and **14 Python tests**. Optimized client/server builds pass; Clippy completes with the same existing style/large-enum warnings and none in the changed code. All six local release executables remain AMD64. Previous client/server executables are preserved before replacement.

The separate publication workspaces pass full tests, optimized builds and all 14 Python tests: `RappelzCore` **41 Rust passed**, `RappelzRust` **80 passed / 1 ignored**. Shared test counts are not additive. Asset-dependent tests return early in clean publication worktrees; the combined workspace and native sessions used the supplied assets.

Evidence remains local under ignored `client-data/validation/2026-10-31-movement-67322d42/`: failing baseline, successful pair/process codes/comparison, unit/build logs, preservation report, logical dump and backups. Credentials, binaries, original assets and runtime evidence are excluded from publication.

Coverage is for the current default speed **120**, scale **1.0**, layer **0** and two clients. Native layer changes, nondefault speed scaling, latency compensation, invalid route inputs, additional observers and native moving-monster snapshots remain separate work.
