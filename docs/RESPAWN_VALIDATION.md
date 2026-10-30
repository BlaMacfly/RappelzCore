# Corpse removal, respawn and repeated combat

Verified **2026-10-31**, Europe/Paris, on two native RappelzRust processes connected to the existing local auth/world services. This extends [COMBAT_VALIDATION.md](COMBAT_VALIDATION.md); its default one-kill scenario still works.

## Defect reproduced and corrected

After a monster corpse disappeared through LEAVE, either client could send QUERY with its old handle. The server unconditionally sent ENTER, reintroducing the hidden corpse into the client's world. Both native baseline reports fail with `query reintroduced hidden corpse`. These retained failures establish the negative control; they are not successful runs.

QUERY now requires an already announced object that is still visible at the time of the request. It checks live distance, layer and corpse lifetime rather than relying solely on the previous tick's visibility set. Queries can still refresh a currently visible recent corpse or the viewer's own character.

The same visibility check gates melee initiation and every pending player attack. Dead players cannot arm an attack, and monsters stop pursuing/retaliating against a player on another layer. A regression test reproduces the stale range/layer query before the correction; additional tests cover expired/unannounced corpses, layer changes during combat, the exact respawn boundary and one reward per kill. Live range/layer transitions are not claimed tested by the native pair below.

## Preserved data and fixtures

A consistent logical backup of all three databases was taken before adding this cycle's fixtures: **18,893,457 bytes**, SHA-256 `a7d57b0e63774bd6aad62c5ff0e50518a1a460462e800bf8460e08d5fb84c40b`. The services remain bound to loopback (3310/4615/4514); no schema import or world-content change was made.

Two new GM-0 accounts/characters were created through normal account/character flows:

| Role | Character | GUID | Offline preparation |
|---|---|---:|---|
| Fighter | LuaCyc1253 | 6 | Level 2, XP 799, JP 0, HP 120, MP 60; position 8450/7227. |
| Witness | LuaObs1253 | 7 | Level 1, XP/JP 0, HP 100, MP 50; position 8420/7240. |

Preparation/reset was restricted by new GUID, account, exact name and offline status. Only these new records were reset between the failing baseline and corrected run. Each native process has its own addon/WTF root and a directory junction to read the installed MPQs. Existing test characters from the earlier cycle were not reused or reset.

The final preservation comparison passes **8/8 checks**: all original account/access/realm-character records through ID 5, characters/settings/inventory through GUID 5, items through GUID 4, and all existing world INSERTs match the new baseline. Aria, Brann, LuaProbe, LuaHitBFBC and LuaSeeBFBC remain unchanged. Operational realm/uptime metadata is outside this comparison. The new records and all backups are retained.

## Native two-client result

Both corrected reports succeed. The independent comparison passes **26/26 conditions**:

- The first real kill crosses level 2→3: XP 799→4, JP 0→3, HP 140/140 and MP 70/70.
- The corpse leaves both clients after its display interval. The fighter's target clears. Both clients explicitly query the hidden handle; neither receives ENTER or recreates the corpse during the one-second observation window.
- The same monster handle re-enters at the existing spawn's 15-second respawn, HP 105/105 and MP 53/53. Both clients select it, and their Lua labels report these actual values before the next attack.
- A second real kill awards another 5 EXP/3 JP, without another level-up. The fighter finishes **level 3, XP 9, JP 6, HP 80/140, MP 70/70**. Both clients observe identical damage and HPMP records: **22 attacks**, comprising 13 fighter hits and 9 retaliations across both fights.
- The witness refreshes the damaged fighter with QUERY and displays level 3, HP 80/140 and MP 70/70. Its own level 1 / XP 0 / JP 0 / HP 100 / MP 50 stays unchanged; private fighter XP is not broadcast to it.
- Both Lua logout events occur, the witness clears its target on peer LEAVE, and both database character rows finish offline. The fighter's saved XP/JP and vitals match the completed session.

Screenshots were visually inspected for both roles. All report JSON, screenshots, logs, logical dumps, prior executables and credentials remain under ignored `client-data/validation/2026-10-31-respawn-12536e5f/`. None are publication inputs.

## Reproduction and checks

Use the same guarded `-validate-combat <plan.json>` runner as the one-kill test, adding `"kills": 2` to **both** plans. Only 1 or 2 is supported; omission preserves the original one-kill behavior. Use a fresh shared attempt directory and separate client data roots. The runner requires dedicated LUA*/Lua* identities, loopback and the documented offline fixture; it never sends GM level/XP commands. Barrier files coordinate readiness only; assertions use received packets, production client state and Lua output.

```powershell
python tools/testing/compare_combat_clients.py <attempt-directory>
python -m unittest discover -s tools/testing -p test_*.py -v
cargo test --workspace --all-targets --locked --offline -- --test-threads=4
```

The comparator splits the ordered attack trace at each killing blow, verifies both HP sequences and exact XP/JP updates, and checks ordered ENTER→LEAVE→ENTER for the same monster. Local mutation checks reject a duplicated JP reward, missing lifecycle LEAVE and incorrect respawn target mana. Packet-trace capacity now includes LEAVE/level/XP records, with a regression for a LEAVE-only flood.

The combined workspace passes **85 Rust tests**, with one intentionally ignored model scan, and **6 Python tests** pass. Clippy completes with the same existing style/large-enum warnings and none in the changed combat/server code. Native optimized client and server builds pass; the server was restarted gracefully while no players were online. The previous client/server executables were preserved before replacement.

The separate publication workspaces also pass their full tests and optimized builds: `RappelzCore` **37 passed**, `RappelzRust` **75 passed / 1 ignored**. Shared tests run in both branches, so the counts are not additive. Clean publication worktrees do not contain original game assets; conditional asset tests there return early, while the combined workspace/native sessions use the supplied local assets.

This verifies the existing provisional combat formulas and this spawn's lifecycle. Player death/revival, additional observers, native movement across visibility/layer boundaries, groups, skills, pets and retail formula fidelity remain separate work.
