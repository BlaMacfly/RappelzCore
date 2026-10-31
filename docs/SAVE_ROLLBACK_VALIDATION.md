# Interrupted character-save transaction with two native clients

Verified **2026-10-31**, extending [movement/combat restart coverage](GAMEPLAY_RESTART_VALIDATION.md). Two real Windows x64 clients complete a cooperative fight, then the isolated worldserver is killed **after both character UPDATEs and before either character_settings REPLACE can finish**. Both uncommitted saves roll back completely. A fresh fight after restart is saved normally and read back on a third entry.

Subsequent implementation: [durable reward notifications](REWARD_DURABILITY_VALIDATION.md) now commits rewards before sending EXP/JP. This page records the earlier server behavior; its interrupted-save-after-announcement expectations require that retained previous server binary.

## What the test proves

The existing `rz_database::characters::save` already performs the character UPDATE and Rappelz settings REPLACE in one InnoDB transaction. XP/position/health are in `characters`; JP and other extra fields are in `character_settings`. This cycle changes no backend save/reward code, schema, gameplay formula or asset. It adds a guarded observation strategy to the existing restart controller, a bounded SQL probe, an optional mode of the existing native driver and independent acceptance checks.

The reward notification reaches the owner before the interrupted save: live XP/JP is **5/3**. After process death, both return to the last committed **0/0** checkpoint. This deliberately verifies atomic rollback of an uncommitted save. It does **not** promise durable rewards at notification time, an exactly-once reward journal, or recovery after MariaDB/whole-host failure. Those remain separate work.

## Controlled real save window

Only newly created clone-only ordinary characters **34/35** are involved. The original services at 3310/4615/4514 remain running. Separate persistent MariaDB/auth/world endpoints are **3331/24615/24514**, all loopback. The controller validates copied executable/config paths, SHA, database prefixes, endpoint isolation and strategy before starting; it kills only the process it owns through Popen.

1. Both clients move from their seed positions to **8452/7235** and **8452/7255**, using real movement requests and server acknowledgements. A first console `saveall` is independently confirmed in SQL: level 1, HP 100, MP 50, XP/JP/money 0, online 1.
2. Capture settled ground and Windows process/GPU counters. A competing same-port server start must fail without changing live flags/population.
3. Both attack the unchanged HP-105 local monster. The witness moves to **8468/7255**, assists for exactly three hits, then sends the production cancel-action request. The owner kills the monster and receives one private **5 EXP/3 JP** notification. Both clients confirm the completed fight and settled positions.
4. The probe locks only the two new fixture `character_settings` rows in its own transaction with `SELECT ... FOR UPDATE`. It checks fixture GUID/account bounds, names, current database identity, distinctness and InnoDB tables first. No trigger, migration, artificial production delay or changed combat statistic is used.
5. The controller sends a second real `saveall`. `INNODB_LOCK_WAITS`/`INNODB_TRX` must show **two different server transactions**, each with a modified row, waiting at `REPLACE INTO character_settings` on the probe's lock holder. A normal committed read must still equal the checkpoint. A separate `READ UNCOMMITTED` connection must see changed character rows with settings JP still 0. These observations establish the internal half-written window without exposing it to normal readers.
6. While the locks remain held, the controller force-kills its owned worldserver. The observation has a bounded 1.5-second lease; a stale/wrong-process/unheld window is rejected. The probe rolls back its own lock transaction, closes its CLI, requires both server transaction IDs to disappear, then confirms that both complete committed profiles still equal the checkpoint. Cleanup also releases its locks on failure.
7. Both real client worlds drop and join. Keep the world down at least five seconds, restart against the same persistent clone, and independently verify scoped online recovery before permitting reauthentication. Other-realm online state is preserved.
8. Second login restores the checkpoint, a new HP-105 monster and no selected target. Observe at least **3.2 seconds** without player routes or attacks being replayed. Recovered ground must match the checkpoint capture exactly.
9. Repeat the real cooperative fight and witness movement. The new owner reward is private and singular. Return both to the lobby normally, read saved XP/JP/positions/health in SQL, enter a third time and match raw login vitals and live progress. Exit normally again; all six world workers must join.

The test hook is `-validate-gameplay-restart <plan.json>` with `interrupt_save: true`; the earlier movement-active mode remains the default. Controller strategy is explicitly `save-rollback`. Production wire messages and ordinary client behavior are unchanged.

## Observed results

The final attempt passes **48/48 independent native checks**; both clients exit 0. World PID **27656** exits **1** after the actual kill; replacement PID **29808** later shuts down normally. Restart readiness is **5.168 seconds** after kill. The kill is **11 ms** after verified lock-window observation.

The two waiting transactions each report one modified row and the actual settings REPLACE. Dirty-read owner state is HP **33**, XP **5**, JP **0**, at **8452/7235**; dirty-read witness position is **8468/7255**. Normal readers still see HP **100**, XP/JP **0/0** and the old witness checkpoint **8452/7255**. After kill and release, both transactions disappear and those complete old profiles remain unchanged.

Both observers record identical **13-hit sequences in each server lifetime**, then zero third-entry hits. Before the crash the owner advertised 5/3, but second entry correctly loads 0/0. The new post-restart fight produces a single private 5/3 reward. Normal saves and third raw login preserve owner HP **35**, MP **50**, position **8452/7235**, XP **5**, JP **3**; witness HP is **100**, position **8468/7255**, XP/JP **0/0**. Final new fixture rows remain offline, GM-0, with unchanged unrelated settings.

Each client produces three actual Lua `PLAYER_LOGIN`, `PLAYER_ENTERING_WORLD` and `PLAYER_LOGOUT` events. No addon or packet-decoding failure is accepted. All **six world workers join**; two partial final-lobby jobs are cancelled without cache/GPU acceptance. Workers were idle at process death; this cycle does not claim simultaneous partial terrain interruption there. Both recovered ground PNGs are byte-identical to their checkpoint captures; the recovered full scene was visually inspected. Terrain texture payload peaks at **60 MiB per client**, with the existing 128 MiB CPU cap and four-upload/frame budget. Twelve Windows counter samples cover the two settled holds; pair GPU Total Committed peaks within those holds are **8278.66/8295.16 MiB**, not a claimed reduction or overall frame-rate result.

The first attempt is retained as failed: batch-mode MariaDB interpreted a CLI `quit` token as SQL after lock release. The probe now sends ROLLBACK and closes stdin to end the batch cleanly. Only the two new clone-only fixtures were reset for the successful fresh attempt; no original records were reset.

## Builds and preservation

- Combined workspace: **161 Rust tests pass**, one exhaustive model test remains ignored; the count includes **22 Lua tests**.
- Publication branches: Core **73 Rust**, client **129 Rust** with one ignored; **82 Python tests pass on each**. Optimized builds produce all six expected AMD64 tools/servers/client.
- Five new Python regressions cover complete rollback acceptance and rejection of partial committed state, missing waits, settings already written, stale lock leases, missing rollback, missing advertised rewards, old/duplicate fixture identities and unsafe kill-window selection. Previous 45-check movement-active behavior remains covered.
- Original selected accounts/characters through **33**, item instances through **4**, world data and installed WTF pass **9** checks. Copied pre-existing clone content passes **8**. Only new clone-only characters 34/35 and sentinel accounts 36/37 are used; sentinel flags are restored. Original high-water marks remain unchanged.
- Logical backup: **18,917,362 bytes**, SHA-256 `b7dc380e7b4c35189435be6c6b57f1a65305e4378fe10ba1fdbc97d8b3bcdcaf`.
- Native-tested installed client SHA-256: `a3ac05a4367fa6c814053795a5c1b87c5638f7c6dd78e211b0ecc5bd2d0b8502`.
- Isolated tested server SHA-256: `66d8c94c283bb698bbe9165080bf5595e73a09b6aeda83eade03a61fbac680c7`. Original running server remains `3ecfcb979e6dbffa4709945ca6403a931b2ef5e4cad743a79dfc064ce125a37c` with the original service PIDs.

Runtime reports, backups, clone files, identities and credentials stay ignored under `client-data/validation/2026-10-31-save-terrain-restart-fd052fdf/`; `verified/` is the successful attempt. Only source, public test tools and documentation are published to the separate Core/client branches. Additional test services are stopped at finalization; persistent clone files and failed evidence are retained.

## Repeating safely

Prepare a fresh dedicated cloned environment using the established restart workflow. Do not reimport or bootstrap an existing installation. Supply the controller plan with `scenario: "save-rollback"`, copied executable/config/SHA and a fresh attempt directory. Both native plans use that directory, their dedicated role/peer and `interrupt_save: true`. Supply `character_save_probe.py` with the server plan and a separate probe JSON containing `mysql_executable`, the isolated `fixtures` file path and independently measured `existing_character_max` / `existing_account_max` bounds. The fixtures must be newly created beyond those bounds. Credentials are read from the isolated config and passed to MariaDB through MYSQL_PWD, never published or printed.

Run the owned server controller, probe and two clients together. Preserve the existing SQL/capture gates: checkpoint verification, initial/recovered counter holds, rollback-complete before down-profile read/restart permission, online recovery before re-login and saved-profile verification before third entry. Run `python tools/testing/compare_gameplay_restart_clients.py <attempt-directory>` after clean client/controller/probe exits, then the original/clone preservation checks, branch builds and tests. Do not treat a failed/partial attempt as successful or delete its evidence.
