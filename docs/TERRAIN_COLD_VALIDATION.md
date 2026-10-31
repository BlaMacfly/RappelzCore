# Background cold terrain blending: two-client validation

This records the earlier steady cold-blending cycle. The later [native cancellation/reconnection validation](TERRAIN_CANCELLATION_VALIDATION.md) exercises partial view/lobby/TCP cancellations, and [WARP/map validation](TERRAIN_WARP_VALIDATION.md) covers immediate teleport invalidation; the counts, hashes and untested-cancellation limits below belong to the earlier cycle.

Verified **2026-10-31**, following [warm pixel reuse](TERRAIN_STREAMING_VALIDATION.md). Cached camera returns were already inexpensive, but a first visit still blended a 1024² texture in the rendering thread. This change moves that CPU blend to one background worker per active world while preserving exact pixels and existing cache/GPU limits.

## Implementation and lifetime

The rendering thread resolves the texture layers referenced by covered NFM cells, then submits immutable `Arc` references to the map/images, repeat distances and chunk coordinates. The worker runs the same wrapped bilinear blend, layer order, weights, 1024² sampling and byte conversion. It never accesses Assets, macroquad, GPU objects, the network or the database. GPU texture creation, mesh assembly and upload remain in the rendering thread.

A bounded request/result channel and active-job identity allow **one outstanding job**. A completed result must be polled before another job can be submitted. Its image is at most **4 MiB**. This is separate from the existing **128 MiB CPU pixel cache**; source maps/images remain shared references. A cached image or a completed image counts toward the same maximum of **four GPU uploads/frame**. No full CPU blend runs during rendering in the normal background mode. If the OS cannot start a worker, the existing synchronous path remains available.

Jobs leaving the requested view receive a monotonic cancellation token, checked once per row and again before publication. Rendering rechecks current visibility/job identity before accepting results. World teardown cancels the job, closes both channel ends and joins the thread. A worker panic is reported, stops further cold submissions and is visible in validation diagnostics; accepted resident terrain can continue rendering. Existing server/gameplay data is unaffected.

## Controlled native comparison

Two new GM-0 fixtures **26/27** use separate roots and the existing loopback server. They start at X 8440, respective Y 7227/7247, level 1, HP 100/MP 50, XP/JP/money zero and layer 0. Both modes keep the **same 128 MiB CPU cache, visible-only GPU policy and source assets**. The same executable runs both pairs; only background blending is disabled for the control.

The test policy is applied **before the first world draw**, including terrain creation. This corrects an initial test-driver limitation that applied the policy only after both players became visible. The first mixed-startup pair is retained locally but is not the published measurement. In the final control, submitted/completed/cancelled background job counters remain **zero** throughout the world session.

The six phases are default view, opposing view, returned default view, overhead view, 2100-unit eastward movement to X 10540, then return home. Every phase captures after all requested chunks are ready. Sampling includes cold work and moving entry rather than only settled frames. Both clients receive both authoritative MOVE routes and retain peer positions. Both native processes exit **0** in both modes, including normal world teardown.

The timing below covers terrain preparation before mesh submission/readback: visibility/cache decisions, input preparation, any synchronous blend, texture upload and mesh assembly. It excludes whole-client rendering, initial map discovery/decoding before `Terrain::draw`, general model/UI work and screenshot capture. Samples are cold-related frames: synchronous blends in the control; submission, active worker or completion/upload in background mode. The counts differ because the rendering loop can continue while CPU blending runs.

| Recorded cold-related frames, owner / witness | Synchronous | Background |
|---|---:|---:|
| Sample count | 29 / 31 | 177 / 177 |
| Main-thread full blends | 29 / 31 | **0 / 0** |
| Preparation p95 | 78.174 / 81.140 ms | **0.731 / 0.663 ms** |
| Preparation maximum | 79.350 / 83.264 ms | 1.065 / 1.043 ms |
| Preparation total over these samples | 2177.542 / 2376.161 ms | 24.858 / 23.254 ms |
| Background jobs submitted/completed over session | 0 / 0 | 41/41 owner, 42/42 witness |

The expensive calculation is moved, not removed: worker elapsed totals are **3.504 / 3.521 seconds** over the background jobs, including scheduling. Some initialization happens before sampled peers/configured phases; control blend sample counts are not a count of every initial-world blend. No claim is made that total loading time or whole-client FPS improves by the preparation ratio.

All **12 settled ground PNG pairs are byte-identical** between the final modes. All 12 also equal the previous installed version's corresponding views, validating the extracted/prepared-input blend against the earlier implementation. A full-scene eastward capture was visually inspected; it retains both players and live RappelzDebug state. Reports retain actual Lua login, world-entry, vitals/JP and logout events, with no combat, rewards, warp or protocol errors.

## Memory and verification

Both modes reach the same **128 MiB cached pixel cap per client** and **72 MiB observed GPU terrain peak per client** on this route. Corresponding settled GPU allocations are identical. A working/completed background image can add up to **4 MiB** outside the cache, plus thread/descriptor overhead; total application memory is not limited to these payloads. Existing Assets/map/model caches and driver resources are separate.

Windows process counters are sampled three times per held phase/PID, **72 samples** over the final runs. Peaks across per-phase sums of per-client medians are:

| Whole-process accounting, both clients | Synchronous | Background |
|---|---:|---:|
| Working Set | 975.67 MiB | 951.17 MiB |
| Private Bytes | 9332.09 MiB | 9247.72 MiB |
| GPU Process Memory Total Committed | 8446.67 MiB | 8391.67 MiB |

These are observations on this machine, not a memory-saving promise. Private Bytes is committed private memory; Working Set is resident process memory. GPU Total Committed is driver accounting rather than terrain payload or physical dedicated VRAM. Allocator, paging and driver variation prevent interpreting these deltas as the worker's exact footprint.

The independent comparer passes **50/50 checks**: mode purity, one-job accounting, progress in the opposing/new-area views, no main-thread blends in background mode, upload/cache bounds, lower cold-related preparation p95, exact images, reciprocal identities, both routes, unchanged gameplay and memory samples. Four worker unit tests cover single-job/result identity, stale cancellation, teardown without polling and isolated panic/recovery. Native cancellation was **not triggered** in the final steady route; that property has unit coverage, while broader rapid-turn/warp cancellation remains a future native check.

Combined workspace: **148 Rust tests passed, 0 failed, 1 ignored**, including **22 Lua integration tests**; **46 Python tests passed**. Both branch workspaces are tested and built separately. The native-tested installed client SHA-256 is `788620310d924601c14b1d22bce309c3df30b773e8689e3e268e1d435b0ada5d`.

A **18,910,937-byte** pre-run SQL backup has SHA-256 `8acd0a4f4f785ede379d571f0af4462ee20b8b4d0ab37652702f16a9d09e0218`. Selected account/character records through **25**, items through **4**, all world INSERTs and installed WTF compare unchanged: **9/9 preservation checks**. New fixtures 26/27 return home and save offline with unchanged stats/settings/GM-0 status. No schema, server binary, original asset, MPQ or normal/companion DBC is replaced. Existing services remain running.

## Reproduce and limits

Use `-validate-terrain <plan.json>` with `renderer: "optimized"`, `motion: true`, `cold: true`, `pixel_cache: "enabled"` and `async_bake: "disabled"` for the control or `"enabled"` for the background pair. Supply dedicated LUA*/Lua* identities, role, peer and an absolute shared directory. The existing guarded loopback driver locks scenario input, samples all phases, holds ready markers for external CPU/GPU sampling and enforces 180-second/trace/sample bounds. Older motion plans default to synchronous blending to preserve their original warm-cache comparison.

```powershell
python tools/testing/compare_terrain_cold_clients.py <attempt-root>
python -m unittest discover -s tools/testing -p 'test_*.py' -v
```

Final evidence is under ignored `client-data/validation/2026-10-31-terrain-async-d30a9b76/verified/`, referring to the retained `reference-final/` and `optimized-final/` sessions. Earlier attempts, original-asset screenshots, credentials, SQL/executable backups and measurements remain local; publication contains only source/tests/docs.

First visits still load progressively while the worker runs. Map discovery, texture decoding, GPU upload and other game rendering can still pause a frame. Broader scenes/hardware, cold-load latency, native cancellation under rapid turns/warp/logout and more complete whole-frame profiling remain open. The work does not claim original-SFrame pixel identity or a complete game implementation.
