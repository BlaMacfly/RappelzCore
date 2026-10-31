# Player death and automatic revival: two native clients

Verified **2026-10-31**, Europe/Paris, against the existing local auth/world services. Two native clients observe an actual lethal monster retaliation and the existing server's immediate automatic return to the configured starting point. Both final processes exit **0**, and an independent comparison passes **26/26 conditions**.

## Preserved data and test scope

Before adding fixtures, a consistent logical backup of all three databases was taken: **18,895,122 bytes**, SHA-256 `deba5d2bd023e25526877c48b10269100b24ea854897029aa8b2b29e5a59e3a8`. Existing services remain loopback-only (MariaDB 3310, auth 4615, world 4514). No schema/content import, existing-character reset or world-data change was performed.

Two new GM-0 accounts/characters were created through normal account/character flows:

| Role | Character | GUID | Offline fixture |
|---|---|---:|---|
| Victim | LuaDeadF624 | 8 | Level 1, XP/JP 0, HP/MP 1, position 8450/7227. |
| Witness | LuaWitF624 | 9 | Level 1, XP/JP 0, HP 100, MP 50, position 8390/7200. |

Preparation/reset was restricted to these new GUIDs, their accounts, exact names and offline status. Each process uses separate addon/WTF directories and a junction to read installed MPQs. The previous combat/respawn fixtures were retained unchanged. Restarting the world service between attempts reset transient monster HP; it was shut down gracefully with zero players online. Original binaries and installed addon files were preserved before replacement.

The final preservation report passes **8/8 checks**: all pre-cycle account/access/realm-character records through ID 7, character/settings/inventory records through GUID 7, item records through GUID 4, and all existing world INSERTs compare unchanged. Installed WTF file hashes are also unchanged. Operational realm/uptime metadata is outside the equality claim. The new records, backups and local evidence are retained.

## Defects reproduced and corrected

1. **A revived observer entity disappeared after the corpse timeout.** HPMP set `dead_at` on zero HP but never cleared it on positive HP. The baseline receives real HP 0 then 100, yet the witness's render predicate becomes false after two seconds. Its native report fails with `revived player disappeared from observer rendering`. Positive HPMP now clears the corpse marker; a test covers death, revival after the timeout, persistent visibility and a later second death without a new ENTER or graphics context.
2. **The observer retained the pre-death position.** With only the render correction, a second native negative control sees a visible player at 8450/7227 while its own client has returned to 8330/7167. WARP addresses its owner; viewers that remain nearby received no position refresh. Teleport now immediately recalculates visibility: former viewers get LEAVE, new viewers get ENTER, and continuing viewers get a current ENTER/wear refresh without losing their selected target. Revival also clears remaining monster targets/chases for that player. Server tests cover continuing/former/new viewers and combat cleanup.

The two failing native attempts (`baseline/`, `after-render/`) and both failing pre-correction unit-test logs remain local. The final witness waits **at least 3.5 seconds after revival before sending any QUERY**. This prevents a query's fresh ENTER from masking either defect. It verifies the actual render-loop predicate, position and Lua target state first, then performs a separate fresh-query check. Both final screenshots were visually inspected.

## Actual session result

The victim starts with deliberately low saved health. Ordinary server regeneration raises it to **6 HP / 3 MP** before attacking; this is recorded, not bypassed. It sends one real attack for **14 damage**, leaving the existing monster alive at **91/105 HP**. A real **18-damage** retaliation reduces the victim to **0 HP**. The runner cancels its auto-attack after the first outgoing hit; it sends no GM kill, heal, teleport or resurrection command.

The existing death handler restores **HP 100/100 and MP 50/50** and returns the victim to **8330/7167**. Both clients receive identical attack and damage-HPMP records. The victim receives one WARP; the witness automatically receives the same handle's current ENTER, before its later manual query. More than 3.5 seconds later the player remains present at the correct location, and the witness's Lua label shows `Target HP: 100/100 MP: 50/50`.

Level remains 1; XP, JP and money remain zero for both characters. No extra automatic attack occurs after revival. The witness clears its target on peer logout, and both clients observe PLAYER_LOGOUT. The victim's saved row is offline at 8330/7167 with HP 100, MP 50, XP/JP 0; the witness is offline with unchanged fixture stats/position.

## Lua death/revival events

`PLAYER_DEAD` fires for a known positive→nonpositive health transition of the same player; `PLAYER_ALIVE` fires for the reverse. Both have no arguments and expose the new state through the normal unit APIs. Initial discovery, unknown health, target changes, another player identity and UI reload do not fabricate these events. They describe the implemented automatic revival flow, not full WoW ghost/spirit semantics.

RappelzDebug registers these events and records target vitals alongside existing player snapshots. During the final victim process it observes exactly one PLAYER_DEAD at HP 0 and one PLAYER_ALIVE at HP 100, in order, even though death/revival can arrive in one rendered frame. The witness receives neither event for its own player; its UNIT_HEALTH target messages report 0 then 100. Saved event counters remain cumulative within each dedicated test identity.

## Reproduction and verification

Use `-validate-revival <plan.json>` on each client as an alternative to `-validate-combat` / `-validate-addons`. The existing host/trace are reused. Plans contain `role` (`victim` or `witness`), the Lua* peer name and a shared fresh report directory. Loopback and dedicated LUA*/Lua* accounts remain required. Use the fixture above with the configured start point and separate client data roots; preparation must stay restricted to newly created offline records.

```powershell
python tools/testing/compare_revival_clients.py <attempt-directory>
python -m unittest discover -s tools/testing -p test_*.py -v
cargo test --workspace --all-targets --locked --offline -- --test-threads=4
```

The read-only comparator checks real lethal damage including overkill, death→full-vitals ordering, private XP isolation, position and continued visibility, unsolicited ENTER before QUERY, actual Lua health/event output, target retention/clearing and logout. Tests include an independent passing sample and reject missing death, false rewards, hidden/old-position entities, duplicate events and a query used to mask a missing automatic refresh.

The combined workspace passes **91 Rust tests** (one intentionally ignored model scan), including **15 real Lua integration tests**, and **10 Python tests**. Client/server/extractor optimized builds pass. Clippy completes with the same existing style/large-enum warnings and none in the changed code. All six local release executables remain AMD64.

Separate publication workspaces also pass full tests, optimized builds and all 10 Python tests: `RappelzCore` **39 Rust passed**, `RappelzRust` **79 passed / 1 ignored**. Shared tests run in both branches, so counts are not additive. Asset-dependent tests return early in clean publication worktrees; the combined workspace and native sessions used the supplied local assets.

Evidence remains under ignored `client-data/validation/2026-10-31-revival-f624b911/`: final reports/screenshots/process exit codes, negative controls, preservation report, backups and build/test logs. Credentials, original assets, binaries and SQL dumps are excluded from publication.

This validates the existing provisional immediate revival behavior. Manual resurrection choices, delayed respawn, death penalties, ghost states, repeated deaths under multiple simultaneous aggressors and retail formula fidelity remain separate work.
