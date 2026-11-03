# Terrain texture pilot and shared GPU tiles

Validated **2026-11-02** using two concurrent native Windows x64 RappelzRust clients in explicitly offline demo sessions. This is an asset/rendering comparison, not a network, original SFrame, or gameplay validation. The existing database, authentication/world services, installed executables, archives and WTF settings are preserved.

## Asset and renderer changes

The selected original diffuse texture is `Textures\zone005_grass@normal002.dds`: **512×512 DXT1**, SHA-256 `f52deb3cb1eef4ef9f49e07d53f277a59589f3574e15f037efb492e9d44bff9a`. Texture ID 0 has DETAIL 2 and repeats every **84 world units**. The prototype retains the original olive grass direction and density, with a slightly darker/greener appearance; it does not introduce a different biome.

The built-in image tool produced two local variants. The selected second image is **1254×1254**, not the requested 2048; no 2K claim is made. Its SHA-256 is `c76b50678fec8f10610cc47536e38d8226d80fe73b81c6ef1a7cb5ae0e1459d1`. The first, more yellow/wiry variant is retained locally but not selected. Prompts, input provenance, originals and comparison captures remain local and are not published as game assets.

`rz-extractor terrain-pack` prepares an opaque square PNG as a power-of-two **RGBA8 DDS with a full mip chain**, plus an independent `patch.MPQ` containing only the selected texture and a manifest. Resizing and file mip generation use periodic Lanczos3 filtering in linear RGB. A fresh output directory is mandatory, including rejection of existing directory/symlink targets; the command does not edit the original archives. The pilot converts 1254 to **1024×1024**, producing **5,592,532 DDS bytes / 11 levels**. This is uncompressed RGBA8, so it uses more disk space than the original DXT1 image.

The first native comparison showed why replacing the image alone was insufficient: the terrain baker still resampled every 1008-world-unit chunk into 1024² pixels. The replacement was resolved correctly, but much of its fine detail was lost.

The client offers experimental `-terrain-direct-tiles` to sample a **shared repeating source texture directly** for strictly uniform chunks. It keeps the same map-local UV phase, DETAIL scale, geometry and vertex lighting. Opaque power-of-two sources up to 1024 per side qualify; unresolved sources, active multiple IDs, transparency, unsupported dimensions and other backends retain the existing baker. The ordinary renderer remains baked unless this flag is supplied. Windows OpenGL uses managed texture ownership, repeat wrapping and generated trilinear GPU mips. The file's prepared lower DDS levels are not consumed by the current decoder: it reads the top level, and this path regenerates GPU levels from it.

Each shared texture's complete mip payload must fit the total 1024² baked payload of the currently requested chunks it replaces. A single 1024 tile therefore falls back to baking; two or more can share it. Route changes discard old mesh references before the shared cache reference, and obsolete textures are evicted. At most four cached/direct chunk builds, including a completed worker upload, are admitted per frame; cold mixed blending retains its existing worker. Texture payload accounting includes unique shared mip chains, rather than counting a copy for each mesh. Existing guarded bake/cancellation scenarios explicitly retain their historical baked route so those comparisons remain meaningful.

## Native evidence

Local evidence: `client-data/validation/2026-11-02-terrain-hd-61af2e09`. Original-only positive control uses two separate scratch roots; the edited pair uses original on the left and the separate patch on the right. Archives are opened for reading and shared by hard link; each client has independent scratch settings and no copied production addons. No account, SQL modification or server command is needed. Preview input and audio are disabled; captures are taken after terrain readiness and a settling interval. Four fixed camera views include opposing close views, a grazing distant view and an overhead view at **8440/7227**.

Both clients exit successfully in each comparison. A final two-client run without the experimental flag produces eight ground captures byte-identical to the earlier baked baseline and reports direct rendering disabled. The shared-tile cycle passes **278 comparisons**: both original clients produce byte-identical ground PNGs in all four views; the original is identical across runs; the edited image changes all four views; dimensions, visible chunk identities, source sizes, GPU accounting, memory limits and current-process samples agree. The earlier baked-only cycle passed 194 comparisons and is retained separately.

| View | Resident chunks | Earlier baked RGBA payload | Shared original, full mips | Shared HD, full mips |
|---|---:|---:|---:|---:|
| Close forward | 15 | 60 MiB | 1,398,100 bytes | 5,592,404 bytes |
| Close reverse | 18 | 72 MiB | 1,398,100 bytes | 5,592,404 bytes |
| Grazing distant | 21 | 84 MiB | 1,398,100 bytes | 5,592,404 bytes |
| Overhead | 6 | 24 MiB | 1,398,100 bytes | 5,592,404 bytes |

All pilot chunks use the selected grass. The decoded CPU source grows by exactly **3 MiB** per view (512² to 1024² RGBA), separately from the bounded **128 MiB** baked pixel cache. The GPU terrain texture payload rises from about 1.33 to 5.33 MiB with HD, while remaining below the earlier baked payload in every view. This does **not** mean total process GPU usage is 5.33 MiB.

Three driver/process memory samples per view and process are retained (24 per pair). In the edited pair, the peak of view medians for total GPU committed memory is **5,894,438,912 / 5,895,225,344 bytes** (original/HD); working sets are **379,465,728 / 380,203,008 bytes**. Driver pools, frame buffers, models and other allocations dominate these totals; only the logical terrain texture payload has the stated rendering bound. These are observations, not a whole-process memory guarantee or a steady-state FPS benchmark.

Visual inspection confirms substantially clearer near-ground detail from direct sampling, including with the original image. The HD variant changes blade definition and color more modestly. Overhead repetition and distant loss of detail remain visible. The generated opposite edges are not pixel-identical; no mathematically seamless result or improvement for every terrain/lighting condition is claimed.

## Reproduce without modifying the installation

In either branch, prepare a new optional pack from a locally available edited PNG:

```powershell
cargo run -p rz-extractor --release -- terrain-pack --image grass-edited.png --texture 'Textures/zone005_grass@normal002.dds' --out new-terrain-pack --size 1024
```

In the client branch, create separate scratch data roots containing the original archives and private WTF directories. Place the generated `patch.MPQ` only in the edited root's Data directory. An existing patch must be retained and its precedence handled deliberately; this command is not a general patch merger.

```powershell
RappelzRust.exe -data scratch-original -demo -terrain-direct-tiles -pos 8440,7227 -validate-terrain-assets output-left
RappelzRust.exe -data scratch-edited -demo -terrain-direct-tiles -pos 8440,7227 -validate-terrain-assets output-right
```

Each output directory must be new. The preview emits `phase-N.ready`; an external observer takes its memory samples and creates `release-N` for each phase 0..3. It writes a report and returns nonzero on failure/timeout. Login, entry, other screenshot drivers and other validation scenarios cannot be combined with this mode. The public Python comparator checks a completed original-only pair against a completed edited pair; its regression tests cover missing asset use, nondeterminism, network mode, stale PIDs, malformed PNGs and incorrect shared-mip costs.

Two additional mixed-area pairs used the original assets on both sides, comparing the baked binary with the same direct route before its opt-in guard was added. At **34776/50904**, 53 structural comparisons pass, including byte-identical full-bake fallback on the overhead view; the source lighting makes the captured terrain black, so this is not useful visual evidence. At illuminated **8568/89208**, 51 structural comparisons pass, with direct/baked counts **15/2, 7/10, 17/4, 3/4**. Payload is **9,786,708 / 43,341,140 / 18,175,316 / 18,175,316 bytes**, all below baked equivalents. Visual inspection exposes a visible detail/filtering boundary between directly sampled and baked portions. The loaded sources and geometry remain intact, but these structural checks do not establish acceptable transition quality.

**For that reason direct sampling is opt-in, not the new default.** The existing renderer is retained for ordinary play. Correct mixed/direct transition filtering and validate connected movement before promoting this route. The optional flag is not saved into installed WTF settings.

The pack remains **optional and uninstalled** in the normal client. Source code/tools/docs are published; original/edited game images, MPQs, native binaries and raw captures stay local. Next fix mixed/direct transitions and expand to roads and soil, then validate the new direct path during connected movement, teleports and disconnects. The earlier network validations exercise the baker, not this new route.

## Source checks

The final locked/offline workspace runs pass **123 tests in the Core branch** and **160 tests in the client branch** (one ignored). These branch totals include shared-crate overlap and are not added as unique tests. Five Python comparer regression tests pass. New Rust coverage checks linear-light periodic resizing, DDS masks/mips, single-entry patch precedence and source preservation, rejected inputs, explicit offline preview guards, shared full-mip budgets, map/chunk UV continuity and conservative uniform-source eligibility. Release builds use a separate target directory; installed/baseline executables are not replaced.

## Versions and preservation

RappelzCore/RappelzRust **0.1.0 development**; NoLimits launcher **1.3.60**; original SFrame file/product **9.1.0.0**; embedded **Source9.5.2**, network **201507080**. The exact official Epic/patch is not established; see [versions](VERSIONS.md).

Nine established SQL/WTF preservation checks and eleven executable/original-file baseline hashes are verified before publication, along with the installed WTF tree and existing service ownership. No original asset replacement, database reset or production configuration change is part of this pilot.

## November 3 refinement: short-bladed grass

The broader [asset modernization pass](ASSET_MODERNIZATION.md) also prepares an inventory icon and a ship-material texture; their verification is separate from the terrain checks below.

The next local asset pass uses the built-in image tool on the original grass tile rather than enlarging the previous generated image. **V3** restores shorter, broader painted blades instead of the previous wiry detail. A second edit, **V4**, softens the appearance and attempts to improve wrapping; it remains an unselected visual alternative and was not tested in the client. Both native outputs are **1254×1254**, not 2048. Original images, previous variants and both prompts are retained locally under `client-data/art/terrain-hd-2026-11-03`.

V3 is selected for its blade definition and closer average source color. Mean sRGB is approximately **70.42/87.60/32.64**, against original **68.94/85.36/38.01** and V2 **63.01/83.41/31.01**. These descriptive averages do not establish perceptual equivalence. Opposite-edge pixel differences remain nonzero, and visual inspection of repeated tiles and the overhead view still shows repetition. Neither variant is claimed to be mathematically seamless.

The existing packer prepares V3 as **1024×1024 RGBA8 DDS, 11 mip levels, 5,592,532 bytes**, in a new mono-texture pack. The separate patch is **4,102,704 bytes**, SHA-256 `7e5e54bc65eca1bb0d41c83435962302658d1a9814a397309470805f9bd82741`. It is copied only into a new scratch client's Data directory. Reading the normal installed MPQ still returns the original 174,904-byte DDS; reading the edited scratch root returns the prepared DDS exactly.

Fresh native evidence is in `client-data/validation/2026-11-03-terrain-assets-4d71a8c2`. Two concurrent original clients form the control, followed by an original/V3 pair, using the existing opt-in preview executable and four fixed views at **8440/7227**. All four processes exit **0**. The existing comparator passes **278 checks**: four identical control views, unchanged original captures across runs, four changed edited views, matching chunk identities/routes and 24 current-process memory samples per pair. This is an **offline asset/rendering validation**, with no network login or original SFrame execution.

The shared terrain texture payload is **1,398,100 bytes original / 5,592,404 bytes V3** in each view, below the corresponding **24–84 MiB** baked payload. Decoded CPU source cost grows by exactly **3 MiB**. The new image does not lower memory relative to the previous 1024 HD variant; the established sharing bound remains unchanged. These payload values are not total process GPU memory or an FPS benchmark.

Post-run checks preserve 14 source/installed-file hashes, the installed WTF tree and all 24 scratch source links/copies. No SQL access or service start is performed. The installed client and renderer default remain unchanged. The private local comparison page presents original/V2 against V3/V4, repeat previews and all four native original/V3 views. Only this authored documentation is published; images, packs, raw captures and the separate Claude decompilation handoff remain local. Mixed/direct transitions and connected movement validation remain open before promoting the renderer.
