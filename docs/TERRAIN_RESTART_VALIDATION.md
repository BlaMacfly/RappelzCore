# Terrain recovery after forced worldserver termination and restart

Historical terrain-only restart cycle. The later [movement/combat restart validation](GAMEPLAY_RESTART_VALIDATION.md) covers live moving combatants, verified durable checkpoints, no stale action replay and new rewards retained through normal save/re-entry. Counts, hashes and untested limits below belong to this earlier terrain cycle.

Verified **2026-10-31** with two real Windows x64 clients and an actual RappelzCore worldserver process kill/restart. The test uses cloned data, a separate MariaDB instance at **127.0.0.1:3331**, isolated authentication at **24615** and a worldserver at **24514**. The existing database/auth/world processes at 3310/4615/4514 remain running and unchanged. This follows [stalled/reset TCP validation](TERRAIN_NETWORK_VALIDATION.md); no relay is used here.

## Server defects corrected

The old startup reset character online flags but left `account.online` referencing a dead realm after a crash. New `auth::reset_realm_online` clears only accounts whose online value equals the positive current realm ID. Accounts belonging to another realm are preserved.

Startup previously also reset characters and published realm availability before binding its listener. A second process failing to bind could therefore clear a live world's flags and overwrite its population. The worldserver now binds first, then clears its character database's online flags and current realm account flags, then publishes availability. The native integration launches a competing instance against the occupied test port: it fails, while both live character/account flags and a population marker remain unchanged. This is protection for a failed same-endpoint start, not a distributed realm ownership lock.

The wire protocol is unchanged. Player handles start from 1 in each worldserver lifetime and may be reused after reboot. The diagnostic trace now retains login order; the driver/checker distinguish replacement worlds and authoritative ENTERs by session order/worker identity rather than incorrectly requiring different numeric handles. One Rust regression covers repeated handles with ordered logins.

## Isolation and preservation

A consistent **18,917,362-byte** SQL backup was made before testing, SHA-256 `6dbbe350da16f1e519ee1478b956843ea44690258f165c76ff710cfb2ebd2669`. All three source schemas were copied into `rz_crash_da8dddb0_*` on a separate local MariaDB process. Database content and resources match the existing environment; only isolated endpoints/realm metadata and new test fixtures differ.

New ordinary characters **34/35**, their accounts and account-only sentinels **36/37** exist **only in the cloned databases**. The main databases retain their original high-water marks of 33 accounts/characters and 4 item instances. Both players remain GM-0, level 1, HP 100/MP 50, zero XP/JP/money, layer 0 and positions 8440/7227 and 8440/7247. Sentinels exercise current-realm versus other-realm cleanup; their temporary flags are restored afterward.

The clients use independent roots/WTF and a separate realmlist. Read-only asset files are hard-linked into a separate asset view; the original realmlist and assets are not edited. The controller accepts only copied executable/config paths under a dedicated restart workspace, isolated loopback ports and a matching clone database prefix on a database service other than the original 3310 endpoint. It removes AC_* child environment overrides so they cannot redirect a validated config, and enables deterministic readiness logging. Only its own Popen process handle is killed; it never terminates a PID supplied by an input file or process search. Readiness failures also clean up the owned child.

## Native scenario

1. Start the isolated worldserver; both clients authenticate directly, enter the same world and see each other. Capture initial loaded ground/full scenes and Windows counters.
2. While both clients wait at the initial hold, attempt a second server start on the occupied port. Confirm nonzero exit and unchanged live database flags/population.
3. Release the clients to seek natural cold terrain blends. Each publishes a short-lived snapshot of its actual active job, row count and worker identity. The controller accepts only two contemporaneous positive partial jobs, then force-kills its worldserver process with **no saveall, console shutdown or signal handler**.
4. The process exits **1**. Its database retains both players' account/character online flags, confirming that graceful disconnect/save cleanup did not run. Both clients receive transport failure, drop their worlds, cancel/join their workers and emit Lua PLAYER_LOGOUT.
5. Keep the worldserver down for at least five seconds. Both clients remain at Login with no world/peer. Authentication and the separate database stay running.
6. Start a new process from the same tested executable and config. Before releasing client logins, read the databases: current player accounts/characters are offline, the current-realm sentinel is reset, and the other-realm sentinel remains marked 2.
7. Both clients reauthenticate, create new worlds/workers, receive authoritative peer ENTERs and restore the same ground view. Capture recovered images/counters; perform normal ReturnLobby. Stop the replacement worldserver gracefully after both clients exit 0.

No WARP, gameplay state or network event is injected in the client. The driver adds no automatic reconnect policy to ordinary gameplay. Actual row progression drives the kill selection, with no worker sleeps, held results or rendering-thread blend substitution.

## Verified results

The independent comparer passes **37/37 checks**. The killed server PID **2772** exits 1; replacement PID **25396** binds the same isolated port and eventually exits 0. Ready-after-restart is **5.166 seconds** after the recorded kill time. The primary service PIDs remain unchanged.

At selection, owner/witness job **69** is running with **209/183 rows** calculated. The selected snapshots are **10/11 ms** old at the recorded kill call (the selection lease is checked at 8 ms before file-read/call overhead). Production teardown cancels at **512/485 rows** and finishes at **513/486**, with no produced image. Both targeted results have **zero CPU cache acceptance and zero GPU uploads**. All **four worker lifetimes join**; worker shutdown-begin-to-join intervals are **0.283–1.114 ms**, not total disconnect/reconnect latency.

Each client records **two actual logins**, two PLAYER_LOGIN/PLAYER_ENTERING_WORLD/PLAYER_LOGOUT events, and one terminal transport error. The owner uses handle **2** before and after reboot, and the witness uses **1** both times. New worker identities and ordered login/peer ENTER events prove a replacement world despite reused numbers. Both peers and unchanged character data are correct after reconnect; no combat, warp or decoder errors occur.

Recovered ground images are **byte-identical** to initial images. All **four initial/recovered captures** also match the previous installed client's corresponding home view. A recovered full scene was visually inspected with both players and live RappelzDebug state. The comparer rejects graceful exits misreported as kills, wrong PIDs, stale/finished candidate jobs, missing interruption/join, uncleared or over-broad account flags, duplicate-start mutations, old-session ENTERs, phantom worlds, incorrect Lua counts, changed pixels and resource/counter failures.

Four controller tests cover endpoint/path/database/hash isolation, idle/null/malformed/stale/future/nonpartial candidate rejection, child environment isolation and owned-child cleanup on readiness failure. Four comparison tests cover the successful reused-handle case and failure mutations. Earlier retained attempts exposed inherited warn-only readiness logging and idle null candidate parsing in the harness; these were corrected and regression-tested. They are not final successful runtime measurements.

## Resources and verification

The CPU pixel-cache limit stays **128 MiB/client**, with one outstanding job, at most one **4 MiB** working/result image, visible-only GPU terrain and at most four uploads/frame. Actual observed cached pixels reach **128 MiB/client**, and terrain GPU payload peaks at **84 MiB/client** while seeking simultaneous partial work. Map/model/driver and database-process allocations are separate.

Twelve Windows process samples were retained. Sums of per-role phase medians, in MiB:

| Phase | Working Set | Private Bytes | GPU Total Committed |
|---|---:|---:|---:|
| initial | 841.30 | 9088.80 | 8323.67 |
| recovered | 847.99 | 9120.50 | 8369.67 |

These are whole-client/driver observations, not leak thresholds, physical VRAM totals or a frame-time benchmark.

Preservation passes **9/9 checks on the original environment**, including selected accounts/characters through 33, items through 4, all world INSERTs and installed WTF. The same pre-existing selected records/world content inside the clone pass **8/8 checks**. The two new clone-only characters save offline at home with unchanged stats/settings; sentinel flags are restored. Isolated service data, backups and evidence are retained; the extra test services are stopped after verification. No original/earlier character is reset.

Combined tests: **159 Rust passed, 0 failed, 1 ignored**, including **22 Lua tests**; **70 Python tests passed**. Independently tested branches: Core **73 Rust/70 Python**, client **127 Rust/70 Python** (one ignored Rust test), and six optimized AMD64 executables. Core carries startup recovery and shared diagnostics/docs; client carries the guarded native scenario and ordered trace.

Native-tested client SHA-256: `141e6d7c9d6ee25ae6f43b2889d5c4f56057c0b3e0234b21ef2dbe9eb62d7992`. Tested worldserver SHA-256: `f51b9becaa2fe00e61273cae5a1ce06db41122f38898b3ce5bad7093276d3fc5`. The client is installed locally; the tested server copy remains under this cycle's ignored workspace. The existing running primary server and its binary are deliberately preserved, SHA-256 `3ecfcb979e6dbffa4709945ca6403a931b2ef5e4cad743a79dfc064ce125a37c`.

## Reproduce and remaining scope

```powershell
python tools/testing/terrain_server_restart.py <isolated-server-plan.json>
RappelzRust.exe -data <isolated-root> -login <dedicated-account:password> -enter <dedicated-character> -validate-terrain-restart <plan.json>
python tools/testing/compare_terrain_restart_clients.py <attempt-directory>
python -m unittest discover -s tools/testing -p 'test_*.py' -v
```

The server plan supplies a dedicated workspace/directory, copied executable and SHA-256, and clone-only configuration. Native plans supply role, peer and shared directory, require dedicated loopback identities, a nondefault isolated authentication port and no proxy. The runner samples initial/recovered barriers and captures duplicate/down/restarted database checkpoints before permitting reboot or client login. Driver/controller waits and frame/packet/audit histories are bounded. SQL passwords, cloned data, original-asset captures, executable backups and runtime evidence remain local and ignored under `client-data/validation/2026-10-31-terrain-restart-da8dddb0/verified/` and its parent.

This validates forced **worldserver process termination**, followed by a restart against the same cloned persistent databases. It does not validate a database crash, power loss/whole-host restart, durability of unsaved gameplay, distributed realm ownership, active combat/movement during failure, permanent silent black holes or other hardware. Automatic reconnect, water/effects and broader gameplay work remain separate.
