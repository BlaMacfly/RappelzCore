# Native terrain cancellation on teleport and map changes

Verified **2026-10-31** against the existing loopback MariaDB/auth/world services with two real Windows x64 clients. This follows [rapid-turn/lobby/TCP cancellation](TERRAIN_CANCELLATION_VALIDATION.md). The server binary, wire format, schemas and original game resources remain unchanged.

## Defect and fix

Previously a WARP moved the player immediately, but pending terrain cancellation waited for the next draw's wanted-chunk filtering. A short jump could keep the pending chunk visible and accept its pre-WARP pixels; a map jump also allowed old work to continue during destination discovery.

`WorldState::apply_warp` now calls terrain invalidation **before** player/layer changes, map discovery and the next draw. Every pending job receives its existing monotonic cancellation token, even if its key is still wanted. The worker checks it once per image row and suppresses finished-but-unpolled results as well. Returning to an earlier map cannot revive an invalidated job. Completed valid CPU/GPU cache entries still use global chunk keys and remain reusable. No blend resolution, sampling or source texture changes are made.

The audit adds `warp` (destination global chunk) and `cancel_warp` (old job identity/key and actual progress). Snapshots expose wanted keys, focus map, WARP and map-switch counts. Two new Rust tests cover completed-but-unpolled overlapping jumps and rapid away/return invalidation. Diagnostic cancellation disabling is used only by the guarded loopback test driver; normal play defaults to immediate cancellation.

## Real two-client route

New fixtures **30/31** start at **8440/7227** and **8440/7247**, level 1, HP 100/MP 50, XP/JP/money zero, layer 0. Only these new accounts temporarily receive GM-1 for the existing `.go` chat command. No local WARP injection is used. Each plan binds the other dedicated character, locks automated input and requires loopback LUA*/Lua* identities.

| Leg | Owner destination (witness Y +20) | Main NFM map | Purpose |
|---|---|---|---|
| Initial | 8440, 7227 | m000_000 | Home baseline |
| 0 | 8500, 7227 | m000_000 | Short jump, pending key still wanted |
| 1 | 12000, 7227 | m000_000 | Distant same-map jump |
| 2 | 72952, 23355 | m004_001 | Different textured map, relief |
| 3 | 89080, 23355 | m005_001 | Second textured map, steeper relief |
| 4 | 8440, 7227 | m000_000 | Return home and verify exact pixels |

For each leg, the owner requests the jump while a natural 1024-row blend is running, with positive early progress. The witness first verifies a near ENTER refresh or distant LEAVE, then follows during its own running blend. Both verify their unchanged peer handle at the authoritative destination and a new server ENTER snapshot. Each settled default view is captured before props/UI; a full-scene image and three Windows CPU/GPU samples per process are also retained. The worker and player handle persist across WARP, with one final normal logout/join per client.

The MPQ probe found 136 NFM files and read/decoded the final route resources without modifying them:

| NFM | Bytes | SHA-256 |
|---|---:|---|
| `m000_000.nfm` | 3,526,057 | `b9c93e6f0a24e6ac5c2d7798e0e55f7d0368d68a2e9ea0637be1163ef888d48c` |
| `m004_001.nfm` | 4,634,775 | `4811f43bb7cd516975a567fc03203e0a5cd78dae876501ec62486f8a44ff583b` |
| `m005_001.nfm` | 4,641,273 | `e80b961ce32ef2b422f4e8497e3cb6d0b2cb00fa40f61d305a6983792605821e` |

The two destination files contain 14/22 texture-layer IDs, 0/64 placed props and white sampled vertex colors; sampled local heights are approximately 17.57/1.05. Adjacent NFM files are also discovered for props, so this is not a cold-file-I/O benchmark or a claim that only three files are loaded. Earlier retained attempts used m001_000/m000_001: both original decoded maps have all 147,456 vertex colors black and flat sampled height. Their black ground is asset-authored; no lighting/asset override was introduced. The final route adds meaningful textured/relief inspection.

## Evidence and negative control

The final independent comparison passes **47/47 checks** (35 causal/gameplay/resource checks plus 12 cross-mode ground-image checks). Both native processes exit 0. Five actual WARP packets per actor produce **ten partial WARP interruptions**, with cancellation progress **133–223 rows** and cancelled finishes at **134–224 rows**. Every targeted result stays out of CPU cache acceptance and GPU upload. Destination uploads use the correct global map namespace. Each client makes three main-map switches and retains one world worker/player handle.

The final pair additionally records one partial camera cancellation and one already-completed view result suppressed before polling: **12 invalidated jobs total**, zero CPU/GPU acceptance. Both workers join after final logout (worker join intervals 0.315/0.381 ms, not end-to-end transition latency). RappelzDebug records exactly one PLAYER_LOGIN, PLAYER_ENTERING_WORLD and PLAYER_LOGOUT per actor, unchanged vitals/XP/JP/money and no decoder/combat/transport errors. The peer sees every destination and the witness confirms four distant departures.

The **same executable and assets** are also run with immediate WARP cancellation disabled to reproduce the prior behavior. Both legacy-mode clients complete the route and exit 0, but the strict comparer rejects **six checks**: lack of partial WARP cancellation/ordering and pre-WARP pending-result acceptance for both actors. The short jump keeps the old key wanted, exposing the defect. This is an expected negative-control failure, retained separately. The corrected run passes all causal requirements.

All **12 settled ground PNG pairs** are byte-identical between modes, including both new maps. Initial and return-home captures also match in each client, and all **four home views** match the previously installed renderer. Full-scene destination captures were visually inspected. Python regressions reject absent/late/unstarted cancellation, stale acceptance, wrong coordinates/maps/uploads, missing overlap/peer state, changed pixels and resource/join/overflow failures.

## Resources and preservation

The CPU pixel cache stays capped at **128 MiB/client**, with one outstanding job and at most one **4 MiB** working/result image. GPU uploads remain at most four per frame, with visible-only terrain residency. Final observed terrain GPU payload peaks are **60/60 MiB** (owner/witness), not a universal total-application cap. Map/image/model caches and driver allocations remain separate.

The runner retained **36 process samples** across six holds. Sums of each role's per-phase medians, in MiB:

| Phase | Working Set | Private Bytes | GPU Total Committed |
|---|---:|---:|---:|
| initial | 838.87 | 9284.68 | 8527.42 |
| leg-0 | 849.58 | 9290.82 | 8530.17 |
| leg-1 | 948.13 | 9393.86 | 8534.42 |
| leg-2 | 1021.81 | 9483.80 | 8534.42 |
| leg-3 | 1049.71 | 9418.88 | 8457.92 |
| leg-4 | 1056.17 | 9502.42 | 8535.42 |

These whole-process/driver observations are not leak thresholds, physical VRAM totals or an FPS/loading-time claim. Worker lifetime and stale-result traces provide the ownership proof.

The pre-run SQL backup contains **18,914,150 bytes**, SHA-256 `c3c8d2ba296aa9daa7ead702112dc1ae611cc7b221f62474339d91bbbbf971ec`. Selected original account/character records through **29**, items through **4**, all world INSERTs and installed WTF compare unchanged: **9/9 preservation checks**. Fixtures 30/31 are retained, home and offline with unchanged stats/settings; temporary permissions are restored to **GM-0**. All characters are offline after validation. Existing services, server executable, original assets/MPQs, normal and companion DBCs are preserved.

Combined verification: **154 Rust tests passed, 0 failed, 1 ignored**, including **22 Lua integration tests**, and **54 Python tests passed**. Independently tested publication branches: Core **73 Rust/54 Python**, client **122 Rust/54 Python** (one ignored Rust test). All six optimized executables are AMD64. Core shares docs/checkers and reuses the verified server paths; client carries the immediate invalidation and native driver. Installed native-tested client SHA-256: `2f665f47468a720b667f073220c48347ac7988ccb7c153b2ede58bf010b6418a`.

## Reproduce and remaining scope

```powershell
RappelzRust.exe -data <independent-root> -login <dedicated-account:password> -enter <dedicated-character> -validate-terrain-warp <plan.json>
python tools/testing/compare_terrain_warp_clients.py <corrected-directory> <legacy-mode-directory>
python -m unittest discover -s tools/testing -p 'test_*.py' -v
```

Plan fields: `role` owner/witness, plain `peer` character name, absolute shared `directory`, optional `cancel_on_warp` (defaults true; false reproduces legacy cancellation only). Both test accounts need temporary GM-1; restore it after normal logout. The runner waits for `owner/witness.initial` and `.leg-0` through `.leg-4`, samples counters, then creates matching `release-<phase>` files. The driver has a 240-second bound, bounded packet/frame/audit traces and no artificial worker delays.

Evidence is local and ignored under `client-data/validation/2026-10-31-terrain-warp-2a83c97e/verified-textured/` and `legacy-textured/`; earlier route attempts, SQL/executable backups, credentials, extracted original resources and screenshots are retained locally. Publication includes source/tests/docs only.

This validates local coordinated same-map jumps, overlapping visibility, two new maps with relief and return home. It does not establish arbitrary malformed WARP behavior, server crash/process kill, long network stalls, whole-frame latency, all maps/heights or other hardware. Map/texture discovery still runs on rendering and first visits remain progressive. Original-client fidelity, water/effects/sky and broader game systems remain separate work.
