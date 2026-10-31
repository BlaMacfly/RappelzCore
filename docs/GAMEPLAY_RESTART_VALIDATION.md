# Movement and combat during worldserver process failure

Verified **2026-10-31** with two real Windows x64 clients, a forced worldserver process kill and a restart against the same isolated persistent databases. This extends [terrain-only restart validation](TERRAIN_RESTART_VALIDATION.md). The original database/auth/world services and records remain unchanged; all gameplay below uses clone-only characters **34/35**.

## Behavior covered and movement fix

The driver deliberately distinguishes a durable checkpoint from unsaved actions. Both players first move to new coordinates and wait for the existing server console `saveall` path to write them. An independent database reader confirms the changed positions, healthy vitals and zero XP/JP before combat is released. Movement and damage after that checkpoint are intentionally not saved before the process kill. Recovery must restore the checkpoint and discard the old route/target/combat context; this is not a promise that unsaved changes survive a crash.

A production client defect was found while exercising looped paths: `Motion::is_moving` compared the current position with the final point. A route returning to its origin could therefore appear stopped at the start or when crossing that coordinate before consuming all waypoints. Movement animation/state now uses total route distance and elapsed travel budget, preserving the existing endpoint tolerance and treating zero speed as stationary. A regression covers repeated loops, intermediate origin crossings and completion. No position interpolation, server movement rule, combat formula, database schema or original resource is replaced.

PacketTrace keeps parallel packet-order metadata for hits, vitals and private XP updates, plus raw login vitals. Existing hit/vital records retain their shapes so earlier two-client comparisons remain valid. Order metadata separates the three login epochs even when the restarted server reuses handles. A Rust regression verifies metadata alignment; histories retain their existing bounded packet budget.

## Real scenario

The isolated environment reuses the earlier guarded controller, with a new explicit `gameplay` strategy. It runs MariaDB at **3331**, auth at **24615** and world at **24514**, all loopback. Only a copied process owned by its Popen handle can be killed. Existing config/path/database-prefix/environment guards remain; unknown strategies fail closed.

| Phase | Owner | Witness |
|---|---|---|
| Initial seed | 8440, 7227; HP 100/MP 50; XP/JP 0 | 8440, 7247; same stats |
| Durable movement checkpoint | 8452, 7235 | 8452, 7255 |
| Before crash | Moving a 16-waypoint circuit and attacking | Moving its own circuit and attacking |
| Restored checkpoint | 8452, 7235; HP 100; XP/JP 0 | 8452, 7255; same stats |
| New combat/movement | Complete the new fight and receive 5 EXP/3 JP | Move to 8468, 7255, assist for three hits, then cancel attack |
| Durable re-entry | Position retained; raw login HP 34, XP 5/JP 3 | Position retained; HP 100, XP/JP 0 |

Both remain GM-0, level 1, MP 50, money 0 and layer 0. The named-DBC reader and existing creature placement identify the unchanged local monster as **711**, placed at **8470/7227** (creature GUID 1498), HP **105**, EXP **5**, JP **3**, gold **0–0**. The raw probe is local evidence; no monster stats are edited to make the test pass.

1. Move both characters using the production `move_path` path and observe authoritative SC_MOVE acknowledgements. Save with the real console command and verify the resulting database rows.
2. Capture the settled checkpoint ground/full scene and Windows counters. The existing competing-start check also confirms that a failed same-port start preserves live online flags/population.
3. Release both real circuits and normal targeting/attack requests. The controller waits for contemporary snapshots proving both players are moving, displaced from the checkpoint, acknowledged by the server, each has dealt damage, retaliation has occurred and the monster is still alive. Then kill the actual worldserver process without shutdown/save handling.
4. Both clients drop and join their worlds. Confirm that persisted character profiles still equal the checkpoint despite live movement/damage, while online flags remain stale as expected after an unclean exit. Keep the world down at least five seconds, restart it and verify the earlier scoped online recovery before permitting login.
5. Both clients reauthenticate and restore the checkpoint. Observe at least **3.2 seconds** without any player movement or attack replay, with no selected target and a fresh HP-105 monster. Settled restored ground is captured before new actions are released.
6. Start a fresh cooperative fight and new witness movement. The witness stops after exactly three hits while the monster is alive; the owner lands the killing blow. Both see identical damage sequences; only the owner receives the private reward.
7. Return both normally to the lobby, read the actual saved positions/XP/JP/vitals, then enter again through the existing character-entry request. Raw login packets must match saved health/MP and positions; live XP/JP must match once. No old attacks, routes or targets reappear. Finish with another normal lobby exit.

All actions use existing production network commands. There are no synthetic gameplay events, forced worker delays or privilege changes. The existing server save/reward/relogin paths pass this cycle without a new backend gameplay change. Reconnection is orchestrated only by the guarded test driver.

## Final evidence

The independent comparison passes **45/45 checks**. Both native clients exit 0. World PID **13328** is terminated with exit **1**, and replacement PID **31276** later exits normally. Ready-after-restart is **5.183 seconds** after kill. Original service PIDs remain unchanged.

At kill selection, both players have delivered one hit and observed one retaliation; the monster has **78/105 HP**. Owner/witness positions are approximately **8472.708/7235** and **8472.887/7255**, more than **20 units** from their saved coordinates and still moving. Snapshot ages at the kill call are **11/12 ms**. Owner HP is 100 and witness HP **82**; after reboot both load the saved **100 HP**, confirming checkpoint rollback rather than accidental preservation of unsaved damage. The persisted profiles read while down exactly match their pre-action checkpoint profiles.

Both observers record the same **three pre-crash hit records**, then the same **13 records** from the new fight, followed by **zero hits** in the third login epoch. Damage chains and the single owner killing blow agree. The owner receives one 5-EXP/3-JP update in the second world, and loads those same totals in the third; the witness remains at zero throughout. The recorded post-combat SQL checkpoint has owner HP **34**, and the third login explicitly returns **34**, rather than a stale/default value. Normal later regeneration raises the final saved owner HP to **39**; that increase is allowed and distinguished from damage/duplication. Final witness HP remains 100. All six Lua login/entry/logout sequences are accounted for: **three of each event per client**.

All **six client world-worker lifetimes join**. Terrain blending stays asynchronous and resource accounting passes. This active-gameplay kill does **not** claim a partial terrain blend was interrupted at process death: both first-world workers were idle then. A separate partial cancellation occurs during the final witness lobby teardown; the previous terrain-only restart cycle remains the evidence for simultaneous partial terrain interruption at server death.

Both recovered ground PNGs are **byte-identical** to their saved-checkpoint captures at the same coordinates/camera. The recovered full scene was visually inspected. The checkpoint differs from the older home scene, so older home screenshots are not used as coordinate-mismatched image references.

Earlier diagnostic attempts are retained separately: the action window was lengthened to keep routes active during combat, the post-restart level-1 fight was coordinated instead of relying on an unsuccessful solo attempt, and the SQL checker was corrected to permit real witness retaliation damage. Those attempts are not final successful measurements. Only new clone-only fixtures were reset between attempts; original records/resources were untouched.

## Memory and preservation

The CPU pixel cache stays capped at **128 MiB/client**, one outstanding blend and at most one **4 MiB** working/result image. GPU uploads stay at most four/frame with visible-only residency. Observed terrain CPU/GPU payload peaks are **60 MiB owner / 64 MiB witness** in this route. Whole-process/driver allocations remain separate.

Twelve Windows process samples were retained; sums of per-role phase medians, in MiB:

| Phase | Working Set | Private Bytes | GPU Total Committed |
|---|---:|---:|---:|
| initial | 857.06 | 9113.58 | 8335.16 |
| recovered | 855.19 | 9378.98 | 8595.92 |

These values are observations, not physical VRAM totals, leak thresholds or frame-time benchmarks.

A fresh **18,917,362-byte** pre-run SQL backup has SHA-256 `bdaa1b08ee0f54865a5e9c34a9a415f6da5a97e166779038ebe1a4910676ee21`. Original selected account/character records through **33**, items through **4**, all world INSERTs and installed WTF pass **9/9 preservation checks**. The copied pre-existing records/world pass **8/8 checks**. Only clone-only characters 34/35 gain the expected new positions, owner XP/JP and combat/regeneration vitals; unrelated settings remain identical. Account-only sentinel flags 36/37 are restored. Original schema high-water marks and service PIDs are unchanged; extra test services are stopped after verification and all clone/backup files are retained.

Combined verification: **161 Rust passed, 0 failed, 1 ignored**, including **22 Lua tests**, and **77 Python tests passed**. Publication branches are tested separately: Core **73 Rust/77 Python**, client **129 Rust/77 Python** (one Rust ignored); six optimized executables are AMD64. The controller adds three tests for live-gameplay eligibility, invalid/stale candidates and unknown strategy refusal; four comparison tests cover successful persistence and failure mutations.

Installed native-tested client SHA-256: `506c80d29c12e40be148fa2b25a8f575068949a67a0863e2258508f4c3acf045`. Isolated tested server SHA-256: `66d8c94c283bb698bbe9165080bf5595e73a09b6aeda83eade03a61fbac680c7`. Backend source is unchanged from the earlier startup-recovery fix; build paths can change executable bytes. The primary running server/binary remain preserved, SHA-256 `3ecfcb979e6dbffa4709945ca6403a931b2ef5e4cad743a79dfc064ce125a37c`.

## Reproduce and remaining scope

```powershell
python tools/testing/terrain_server_restart.py <isolated-server-plan-with-scenario-gameplay.json>
RappelzRust.exe -data <isolated-root> -login <dedicated-account:password> -enter <dedicated-character> -validate-gameplay-restart <plan.json>
python tools/testing/compare_gameplay_restart_clients.py <attempt-directory>
python -m unittest discover -s tools/testing -p 'test_*.py' -v
```

The controller server plan adds `scenario: "gameplay"` to the existing scoped restart plan. Native role/peer/directory plans require dedicated loopback identities, separate auth and no proxy. The runner verifies a console-requested durable movement checkpoint, initial/recovered capture holds, down/restarted database state, and normal-lobby saves before third entry. Game drivers/controllers, packet histories, samples and audits remain bounded. Credentials, databases, original-asset captures, probe output, executables and runtime reports remain local/ignored under `client-data/validation/2026-10-31-gameplay-terrain-restart-7860cab6/verified/` and its parent; publication contains source/tests/docs only.

This validates two moving combatants during a worldserver process failure, deterministic checkpoint recovery and new post-restart progress persisted through normal saves. It does not add crash-time saving, a write-ahead gameplay journal, database/whole-host crash guarantees, exactly-once reward transactions across an interrupted save, automatic reconnect, accurate final combat formulas or broader latency/hardware coverage.
