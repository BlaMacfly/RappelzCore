# Warm terrain streaming: two-client motion validation

Verified **2026-10-31** after the [camera culling/memory cycle](TERRAIN_RENDERING_VALIDATION.md). That cycle reduced unnecessary GPU textures while retaining 1024² detail. It still rebuilt a full image after a camera turn brought an evicted chunk back into view. This follow-up reuses exact CPU pixels for revisited views; it does not reduce resolution or change original assets.

## Change and bounds

A per-world least-recently-used CPU cache retains at most **128 MiB of RGBA pixels**, or **32 full-size chunks per client**. The 31 chunks in the two opposing test views fit this budget. Replacements and evictions account for actual image bytes; oversized entries bypass storage. Cached images are not attached to evicted GPU meshes. Logout/world replacement releases the cache. A newly blended image can temporarily add one 4 MiB buffer before insertion evicts an older image; driver allocations and other resources are separate.

The renderer still discards off-camera GPU textures before allocation. It can restore at most **four** cached textures per frame (**16 MiB uploaded**) and never performs a cold blend in a frame that restored textures. If no restoration is needed, the existing limit of **one cold blend per frame** remains. Source sampling, repeat distances, lighting, geometry, 1024² RGBA values and GPU filtering are unchanged. No lossy encoding, quality setting or new resource file is introduced.

## Native comparison

New ordinary GM-0 fixtures **24/25** use separate roots, fresh preferences and the existing loopback services. They start at 8440/7227 and 8440/7247, level 1, HP 100/MP 50, XP/JP/money zero and layer 0. Existing characters are neither reused nor reset.

Both pairs use the same executable and normal camera-frustum policy. The guarded test plan disables only the CPU pixel cache for the control. Each pair performs six phases:

1. Settle and capture the default home view.
2. Settle and capture the opposing view, warming both sets of pixels in the cached run.
3. Run **96 rendered frames**, alternating the two views every **8 frames** (12 views); settle at the default view and capture.
4. Settle and capture the overhead view.
5. Move both players 2100 units east to X 10540, with the same 96-frame rotation sequence during movement; settle and capture.
6. Return home using ordinary movement packets; settle and capture, then log out normally.

The test records every rotation frame and motion/warmup frames, including unready chunks. This avoids the previous settled-only sampling blind spot. Preparation timing covers visibility/cache decisions, terrain blending, texture upload and mesh assembly before mesh submission and screenshot readback. It is not whole-client rendering time or FPS.

| 96-frame home rotation sequence, per client | Control owner / witness | Cached owner / witness |
|---|---:|---:|
| Cold texture blends | 96 / 96 | 0 / 0 |
| Exact pixel restorations | 0 / 0 | 174 / 174 |
| Sum of requested chunks not ready across frames | 960 / 960 | 234 / 234 |
| Terrain preparation total | 7016.27 / 7085.44 ms | 130.74 / 136.46 ms |
| Terrain preparation p95 | 74.40 / 76.34 ms | 3.56 / 3.89 ms |
| Terrain preparation maximum | 77.73 / 79.02 ms | 3.70 / 4.77 ms |

Requested-but-unready chunk counts decrease **75.6%** during the identical 96-frame sequence. This measures delayed requested geometry, not a pixel-area measurement of visible holes: conservative bounds can include occluded terrain. The cache avoids all cold reblends in the warmed rotation sequence. Newly visited areas still require blending.

**All 12 settled ground PNG pairs are byte-identical** between control and cache. Ten matching views also equal the previous installed client's ground captures, covering default, opposing, overhead, east and returned-home views for both players. Full-scene captures retain both players; reports/logs retain actual RappelzDebug login/vitals/JP/logout events. Both native clients exit **0** in both runs.

## Memory tradeoff

The CPU pixel cache reaches its **128 MiB payload cap per client**, **256 MiB for the pair**. This deliberately trades bounded host memory for less repeated blending; it is not a further 128 MiB of GPU texture storage. The independent comparer checks byte/upload/blend limits on every recorded frame.

Each corresponding settled view has **identical GPU terrain texture allocation** with and without the cache. The observed peak over all recorded motion frames is **72 MiB control / 76 MiB cached per client**: the faster renderer can fill one additional requested chunk during movement. Both retain the same visible-only GPU policy and radius bound; the earlier shallow-view 84 MiB peak remains a different scene measurement, not a universal limit.

Windows CPU/GPU process counters are sampled three times per held phase and PID, **72 samples** across both runs. The table reports the peak across per-phase sums of per-client medians:

| Whole-process accounting for the pair | Control | Cached |
|---|---:|---:|
| Working Set | 838.40 MiB | 981.68 MiB |
| Private Bytes | 9266.30 MiB | 9275.68 MiB |
| GPU Process Memory Total Committed | 8443.67 MiB | 8394.17 MiB |

These counters include the entire application and graphics driver. Working Set is resident process memory; Private Bytes is committed private memory, not a physical-RAM reading. GPU Total Committed is driver accounting, not terrain payload or dedicated VRAM residency. Allocator/driver behavior and paging prevent equating counter deltas directly to cached pixel bytes. These are observations on this machine, not universal memory/FPS promises.

## Verification and preservation

The independent motion comparer passes **48/48 checks**, including reciprocal identities, both authoritative movement routes, six completed phases, 96-frame camera patterns, moving samples, unchanged gameplay, bounded host/GPU resources and uploads, no hot reblends, lower preparation cost and unchanged ground images. Pixel-cache unit tests cover exact byte recovery, LRU recency, repeated replacement/eviction, disabled/oversized storage and byte accounting. Python regressions reject blur, changed GPU/CPU budgets, excess uploads, missing/incorrect motion, identity errors, reblending and missing/non-improving measurements.

Combined workspace: **144 Rust tests passed, 0 failed, 1 ignored**, including **22 Lua integration tests**; **42 Python tests passed**. Both separate branch workspaces are tested/built independently. The native-tested client SHA-256 is `13494c23a3dc9885c9aa9b782d376919b6aa97b5d0e8bcea406de6292f7a650a`.

A **18,909,331-byte** pre-run SQL backup has SHA-256 `8d5446a4c91440f050bb9f079dcbf9d4db877fb78bb2b8dbcb60a1d76c95cf95`. Selected existing account/character records through **23**, items through **4**, all world INSERTs and installed WTF compare unchanged: **9/9 preservation checks**. New fixtures 24/25 return home with unchanged vitals, XP/JP/money, settings and GM-0 status, saved offline. Services, server binary, schemas, MPQs, DBCs and original assets are retained.

## Reproduce

Use the existing `-validate-terrain <plan.json>` driver, with `renderer: "optimized"`, `motion: true`, `pixel_cache: "disabled"` for the control and `"enabled"` for the cached pair. Each plan also supplies `role` (`owner`/`witness`), `peer` and an absolute shared `directory`. The driver requires loopback and dedicated LUA*/Lua* accounts, locks only scenario input and enforces its 180-second/trace/sample bounds. The runner waits for both role-ready markers per phase, samples memory, then creates `release-<phase>`.

```powershell
python tools/testing/compare_terrain_motion_clients.py <attempt-root>
python -m unittest discover -s tools/testing -p 'test_*.py' -v
```

Runtime evidence, original-asset screenshots, credentials, backups and executables remain local/ignored under `client-data/validation/2026-10-31-terrain-motion-7c42b8e1/`. Publication includes only source, tests and documentation.

Cold first visits still blend synchronously one chunk per frame. Four restorations can leave requested chunks warming for several frames after a turn. The change reduces these delays rather than eliminating all loading gaps. Wider maps/heights, cache-budget tuning, cold-blend scheduling and hardware/frame-time validation remain open. Original SFrame behavior or all-game completeness is not claimed.
