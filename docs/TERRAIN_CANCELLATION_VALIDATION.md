# Native terrain cancellation: rapid turns, lobby and TCP reconnect

Verified **2026-10-31** with two real Windows x64 clients and the existing loopback database/auth/world services. The previous [cold-terrain cycle](TERRAIN_COLD_VALIDATION.md) validated background blending but did not trigger native cancellation. This cycle exercises cancellation while actual image rows are being calculated and after a completed result is waiting for rendering.

## Implementation and observability

The existing worker already cancels off-view jobs, suppresses cancelled results during polling and cancels/joins on world teardown. It now exposes bounded lifecycle diagnostics: submission, start, row progress, cancellation request, finish, polling, cache acceptance, GPU upload and shutdown/join. A per-worker identity separates successive worlds. The audit retains **1024 metadata events at most**, plus an overflow count, and holds no pixels, maps or GPU objects. Each immutable blend input shares one atomic completed-row counter; the blend checks cancellation once per row as before. Pixel calculations and sampling are unchanged.

Join failures, previously ignored, now produce an explicit `join_failed` diagnostic rather than a successful `joined` event. Tests/checkers reject worker errors, overflow, invalid job ordering, reused identities and cancelled cache/GPU acceptance. Additional unit tests cover cancellation after completion but before polling, running shutdown with retained audit metadata, audit capacity/identities and bounded transport-disconnect tracing.

An internal `NetCmd::Disconnect` closes the real game transport through the existing network loop. The server uses its existing EOF save/visibility cleanup; the client receives the normal disconnected event and can authenticate a new session. No new wire packet, server command, schema or Lua action API is added. The guarded native scenario uses ordinary `ReturnLobby`, TCP close, reauthentication and world-entry paths. Normal gameplay controls are unchanged.

## Native scenario

New ordinary GM-0 fixtures **28/29** start at **8440/7227** and **8440/7247**, level 1, HP 100/MP 50, XP/JP/money zero and layer 0. Existing characters are not reused or reset. The clients use independent roots/preferences and the same original asset junction. Input is locked only for the automated loopback LUA*/Lua* scenario.

1. Both clients join and see each other. Each targets **six confirmed partial blends** during rapid opposing camera turns. Decisions use real completed-row counts, without artificial worker sleeps or held results.
2. Both return to the default camera and capture the fully loaded initial ground image. The runner samples memory.
3. The owner returns to the lobby while a blend is running, then re-enters. The witness observes the old handle leave and the new handle enter.
4. The owner closes the TCP transport during another running blend, then authenticates and re-enters. The witness verifies departure/reappearance again.
5. The witness performs the same lobby and TCP cycles while the owner observes.
6. Both recover the default view, capture ground/full-scene images, then perform final normal logout.

Every targeted teardown checks the worker's actual shutdown progress, partial finish and joined event. Each client records **three real world logins/entries/logouts**, **one transport disconnect** and **three distinct worker lifetimes**. Re-entry receives a new player handle. Both processes exit **0** and all six workers end with a successful join.

## Final results

The independent comparer passes **39/39 checks**. Across the final pair:

- **13 partial view cancellations** are recorded: six owner and seven witness, including return to the default view.
- **Four partial teardown cancellations** are confirmed: lobby and TCP closure for each client.
- **One already-completed result** is cancelled before polling: witness world 1, job 10. Its finish records 1024 rows and a produced image, followed by view cancellation and cancelled polling.
- **18 cancelled job identities in total** have **zero cache acceptances and zero GPU uploads**. Partial cancellation requires a started job, positive progress below 1024 rows and a cancelled finish with no produced image.
- All **six worker lifetimes join**; measured shutdown-begin-to-join time is **0.195–0.857 ms** on this machine. This is worker teardown time, not total logout/reconnect latency.
- Both recovered ground captures are **byte-identical** to their initial captures. All **four captures** also match the previous installed renderer's corresponding default views. A recovered full scene was visually inspected.

Audit IDs are scoped by client/worker, so a new world can start again at job 1. The comparer verifies causal submission/start/finish/poll/cache/upload ordering, distinct worker identities, termination, actual transport/Lua events, peer ENTER/LEAVE packets, unchanged character data and resource bounds. Python regressions reject stale acceptance, worker/identity errors, completed/unstarted work misreported as partial, missing TCP closure or peer cleanup, changed images and budget/accounting errors.

## Memory and data preservation

The existing limits remain **128 MiB of cached CPU pixels per client**, one **4 MiB working/result image**, one outstanding background job and at most four GPU uploads/frame. Observed cached pixels and GPU terrain peak are **60 MiB per client** on this stationary route; this is a scene measurement, not a universal cap. Audits retain only bounded metadata across teardown in the diagnostic driver.

The runner samples Windows process counters three times per PID at initial/recovered holds (**12 samples**). Per-phase sums of per-client medians are:

| Whole-process accounting, both clients | Initial | Recovered |
|---|---:|---:|
| Working Set | 831.11 MiB | 819.82 MiB |
| Private Bytes | 9276.86 MiB | 9267.84 MiB |
| GPU Process Memory Total Committed | 8520.42 MiB | 8539.42 MiB |

These include the entire application/driver and are observations rather than leak thresholds. Private Bytes measures committed private memory, Working Set resident process memory, and GPU Total Committed driver accounting. Worker ownership/join traces provide the lifetime proof; counter deltas alone do not.

A **18,912,543-byte** pre-run SQL backup has SHA-256 `8250fdb359e3d5fc1e439a6601f43f2665a443da04c834d6231a246c0e0f509a`. Selected existing account/character records through **27**, items through **4**, all world INSERTs and installed WTF compare unchanged: **9/9 preservation checks**. New fixtures 28/29 save offline with their original stats, positions, settings and GM-0 status. Existing services/server binary, schemas, original assets, MPQs and normal/companion DBCs are retained.

Combined verification: **152 Rust tests passed, 0 failed, 1 ignored**, including **22 Lua integration tests**; **50 Python tests passed**. Both Git branch workspaces are tested and built independently. The native-tested installed client SHA-256 is `b96fc487896a737a38d29442b11080175262c505b4a246ada652999eebaba4b8`.

## Reproduce and limits

```powershell
RappelzRust.exe -data <independent-root> -login <dedicated-account:password> -enter <dedicated-character> -validate-terrain-cancel <plan.json>
python tools/testing/compare_terrain_cancellation_clients.py <attempt-directory>
python -m unittest discover -s tools/testing -p 'test_*.py' -v
```

Each plan supplies role (`owner`/`witness`), peer and an absolute shared directory. The runner waits for both `initial`/`recovered` markers, samples memory and creates `release-initial`/`release-recovered`. The driver has a **180-second** bound, bounded packet/frame/audit traces and the existing loopback/dedicated-account guards. Passwords stay in process memory/local ignored inputs and are excluded from reports/publication.

Final evidence lives under ignored `client-data/validation/2026-10-31-terrain-cancel-e619f40c/verified/`. The initial successful pair is also retained. Original-asset screenshots, SQL/executable backups, credentials and runtime reports are not publication inputs.

This validates local camera turns, graceful world exit and client-initiated TCP close/reconnect. Server crash, process kill, warp/map-change cancellation, long network stalls and other maps/hardware remain separate cases. First visits remain progressive; this is not a whole-frame FPS benchmark or a complete-game/original-SFrame compatibility claim.
