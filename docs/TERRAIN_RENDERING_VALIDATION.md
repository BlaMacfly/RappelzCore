# Ground detail and two-client GPU memory validation

Verified **2026-10-31** with two ordinary native x64 clients connected concurrently to the existing local authentication/world services. The server and original assets were retained. The optimized renderer preserves the detailed ground image while allocating fewer full-resolution textures.

## Earlier detail correction

The earlier pixelation report came from blending ground into a **128×128** image stretched over a **1008×1008 world-unit chunk**, leaving about 21 texels across a typical 168-unit texture repetition. The first correction raised blending to **1024×1024** and used wrapped bilinear source sampling. Texture names, DETAIL repeats, layer weights, lighting, geometry and linear GPU filtering stayed unchanged. Matched offline previews at 8440/7227 demonstrated the improvement; their 7,127/9,604 ms startup-through-120-frame times were not steady-state FPS measurements.

## Memory correction without lowering detail

The old renderer prepared every chunk in the surrounding 7×7 square, including terrain outside the camera. Its eviction threshold could retain chunks after leaving that square. Each baked RGBA texture occupies **4 MiB** of texture payload.

The current renderer computes a conservative bounding box from every vertex of each candidate chunk. It rejects a chunk only if all eight box corners lie beyond one camera clip plane. This retains large boxes crossing the view, slopes above their base, near-plane crossings and viewport-edge contacts. Non-finite clip values fail open. It does not depend on whether a corner happens to project onto the screen.

Invisible textures are released **before** building missing chunks, every frame. Macroquad's managed texture lifecycle performs GPU deletion. Height bounds remain small CPU metadata. Normal rendering always enables culling; the older policy is accessible only through the guarded local validation plan. Blending resolution, source sampling and displayed textures are unchanged. Construction remains limited to **one chunk per frame**.

## Native comparison

Two new GM-0 characters, **22/23**, used separate client roots and the same asset junction. Both were seeded offline at level 1, HP 100/MP 50, XP/JP/money zero, layer 0, positions **8440/7227** and **8440/7247**. Existing characters were not reused or reset. Normal login, movement, authoritative server MOVE packets and logout were used.

The reference and optimized pairs ran the same six views at **1280×720**:

1. Default camera at home: yaw approximately 1.971, pitch 0.45, distance 140.
2. Camera turned 180 degrees.
3. Shallow view: pitch 0.08, distance 900.
4. Overhead view: pitch 1.35, distance 900.
5. Both players move east to X **10540**, crossing two chunk boundaries while retaining their respective Y coordinates; default camera.
6. Both return to X **8440**; default camera.

Each capture waits until every requested visible chunk is resident. The renderer captures ground/background before props, players and UI. **All 12 reference/optimized PNG pairs have identical bytes and SHA-256 hashes**, proving exact equality for these views. Full scene screenshots also show both players and live RappelzDebug state. Default and shallow-view captures were visually inspected. Both native processes exited **0** in each run.

| View | Old resident chunks per client | Optimized chunks per client | Optimized ground texture payload |
|---|---:|---:|---:|
| Home | 49 | 15 | 60 MiB |
| Turned | 49 | 18 | 72 MiB |
| Shallow | 49 | 21 | 84 MiB |
| Overhead | 49 | 6 | 24 MiB |
| East | 63 | 14 | 56 MiB |
| Returned home | 63 | 15 | 60 MiB |

The observed texture-payload peak falls from **252 to 84 MiB per client**, **66.7% lower**; for the pair, **504 to 168 MiB**. This is resource accounting for baked terrain textures, excluding geometry, models, UI, audio and driver overhead. **84 MiB is an observed peak, not a universal limit**: another view can retain up to all 49 chunks, or 196 MiB with the current radius.

The runner also sampled Windows **GPU Process Memory** counters three times per held view and per PID: **72 samples** across both runs. It aggregates dedicated/shared usage and Total Committed across matching adapter instances. The peak across per-view sums of per-client median **Total Committed** decreases from **8930.69 to 8627.43 MiB** for the pair, about **303.26 MiB / 3.40%**. This is whole-process Windows driver accounting on this machine, distinct from terrain payload and physical dedicated VRAM residency. Remaining client memory use is outside this terrain change.

The independent comparer passes **39/39 checks**: both clients complete all views, receive both ordinary movement routes, retain peer positions, preserve vitals/XP/JP/money/layer, expose no protocol errors/combat/warp, complete actual Lua login/logout, keep full-resolution textures, evict invisible chunks, reduce payload/driver memory and match every ground image. Unit tests cover enclosing/edge/high/near/behind-camera boxes and rotated views; comparer regressions reject blur, changed pixels, stale textures, missing routes, desynchronization, changed rewards and missing/increased GPU measurements.

## Preservation and reproducibility

A **18,907,725-byte** SQL backup precedes the run; SHA-256 `36c15f06980f2f1b79cb18cce1a9b7b247be7225b7c9afdf28cb7023ae7a5451`. Selected account/character records through **21**, items through **4**, all world INSERTs and installed WTF files compare unchanged: **9/9 preservation checks**. Both new fixtures return home with original stats/settings and save offline. No schema, world content, original texture, MPQ or normal/companion DBC is replaced. Database/auth/world services remain running.

Current verification: **142 Rust tests passed, 0 failed, 1 ignored**, including 22 Lua integration tests; **38 Python tests passed**. Both separate branch workspaces are tested and built independently. The native-tested installed client SHA-256 is `c9313be7994f5e4c68dc2f820de3a1f815e1426e3f99553aaf6fd5b7201bf78a`.

Run each pair with fresh report directories, dedicated LUA*/Lua* test accounts, independent roots and a loopback server:

```powershell
RappelzRust.exe -data <independent-root> -login <test-account:password> -enter <test-character> -validate-terrain <plan.json>
python tools/testing/compare_terrain_clients.py <attempt-root>
python -m unittest discover -s tools/testing -p 'test_*.py' -v
```

Plans specify `role` (`owner`/`witness`), `peer`, `renderer` (`reference`/`optimized`) and an absolute shared `directory`. The external runner waits for both `<role>-<phase>.ready` markers, samples GPU counters, then creates `release-<phase>`. Input is locked only for this automated scenario. The driver has a 180-second limit and bounded trace/frame samples; normal play controls are unchanged.

Images, plans, credentials, SQL backups, counter samples, traces and executable backups remain local and ignored under `client-data/validation/2026-10-31-terrain-9d61f0a2/`. Images contain supplied game assets and are excluded from publication. The previous skill-3212 validation remains recorded in [SKILL_CORPSE_VALIDATION.md](SKILL_CORPSE_VALIDATION.md).

The comparison covers settled views on this local map and GPU. Newly visible chunks still warm up one per frame during camera turns/movement; image equality is asserted after warmup. It is not an FPS benchmark, an all-map/hardware certification or a pixel-identity claim about original SFrame. Broader scene/hardware and frame-time work remain open.
