# Experimental offline sky and player materials

Implemented **November 3, 2026**, following the [character/equipment asset review](CHARACTER_EQUIPMENT_ASSETS.md). Both features require explicit demo flags and remain disabled during ordinary play. They do not replace original images, change UVs or write graphics flags to WTF settings.

## Player head and hair

The rear-view eye/face artifact was already present with original textures. Source inspection establishes that the existing 3D pipeline already uses **LessOrEqual depth testing and depth writes**. It draws both triangle sides and blends texture alpha for every model. The supplied face is mostly alpha **153**, and the original hair includes both alpha-zero gaps and alpha-153 regions. Treating these uniformly as blended opacity exposes overlapping head surfaces. The original client's material-specific alpha interpretation remains unconfirmed.

Controlled original-resource probes compare the default material, back-face culling, discard alone, opaque output without discard, and an explicit cutout pass. Culling changes two-sided hair; discard alone retains the translucent face; opaque output without discard fills hair cards. The selected experiment discards samples below **0.5** and writes opaque RGB for retained fragments, with depth testing/writing preserved and culling disabled.

`-character-cutout` enables that material **only for typed player rigs**. NPCs and monsters retain their existing material path, including potentially intentional transparency. One material is owned by the model cache and shared by subsequently assembled player rigs. It is not recompiled per frame; disabling the cache mode does not rewrite materials on an already assembled rig. The normal material is restored after drawing. Unsupported backends or material-loading errors retain the previous renderer.

```powershell
RappelzRust.exe -data separate-demo-root -demo -character-cutout
```

Five real comparison runs cover original-source idle, edited-face idle, edited-armor idle, one original-source running pose and the unchanged historical control, with three camera views each. Original left-panel controls remain exact; the legacy renderer retains its previous captures. A separate GPU fixture preserves the source alpha texels and verifies that rejected samples do not occlude a background drawn afterwards, while retained alpha-153/255 samples draw opaque. Original DDS, models and UVs remain intact.

These are fixed-pose samples, not validation of every animation, race, hairstyle, effect or lighting condition. Alpha-test edges are harder than blended edges. The new mode is an explicit rendering experiment, not a claim that the original shader has been reconstructed.

## World-oriented sky gradient

`-sky-gradient` adds an optional gradient backdrop before terrain. Camera rays sample the center column of `Textures\sky_zone001_sky01.dds`; the source row is determined by the world ray's vertical direction. Translation and yaw do not shift this vertical atmosphere, while camera pitch moves the horizon to the corresponding screen position. Downward rays retain the horizon color until geometry occludes them.

```powershell
RappelzRust.exe -data separate-original-root -demo -sky-gradient
```

The source is selected through ordinary MPQ precedence. A separate edited root may contain the previously prepared V2 patch; the original root remains unpatched. The V2 artwork still has color drift and remains a proposal. This renderer does not load a six-face cubemap, reproduce the original sky shader, bind map/fog settings, or add cloud layers. The original dome's separate 4×4 material remains unchanged.

The backdrop writes no depth, uses no blending and restores the default material before terrain and UI. A missing, invalid or non-opaque DDS, unsupported backend, material error or invalid camera falls back to the existing solid **150/185/220** background. The accepted DDS dimensions are at most 2048 per side. Each world owns one top-level GPU texture: **262,144 bytes** for the supplied 256² source or **4,194,304 bytes** for the 1024² V2 source. Lower file mip levels are not uploaded as a runtime mip chain. Resources follow the world lifetime and are not retained in the general asset texture cache.

The 24 MiB encoded-size check happens after the current archive reader has returned its buffer. Decoder/GPU checks do not establish a hard bound on the initial MPQ allocation or on total process/driver memory.

## Guards and verification

Both boolean flags require `-demo`; duplicate flags, explicit values, login/entry arguments and online validation scenarios are rejected. Terrain asset preview remains available for fixed-camera comparisons. The world also refuses to enable these modes on an online state, independently of CLI parsing. Neither feature is activated by installing an edited image alone.

**57 client tests pass**, including seven sky tests, player-material guards/defaults and world activation/lazy-resource tests. Sky rays are checked against inverse projection, translation/yaw invariance, camera pitch and orthographic projection. DDS validation rejects incompatible alpha and excessive dimensions. Eight independent Python comparator tests cover image decoding, process lifetimes, false success reports, renderer modes, memory accounting and containment of sky changes within the former background.

Native sky validation uses separate original/default, original-gradient and edited-gradient pairs, the ordinary terrain baker and the existing fixed-camera preview. Its report checks process exits, the actual loaded/drawn sky state, memory observations and pixels outside the former flat-background mask. Complete source and runtime verification records remain local under `client-data/validation/2026-11-03-sky-renderer-8bc4f2a7`; head comparisons remain under `client-data/art/renderer-followup-2026-11-03`.

**Stopped at the user's request. Strict sky acceptance is not obtained.** All six native clients exit 0, with four views and 24 memory observations per pair, and both sky sources load/draw without fallback. The comparator nevertheless fails its containment check: views 0/1/2 have **633 / 659 / 645** changed pixels outside the exact old background mask, all immediately adjacent to that mask; the overhead view has zero changes. This is consistent with a coverage-edge issue but does not establish its cause. The comparator is retained unchanged rather than treating these differences as a pass. No further rendering trial was launched after the stop request.

Post-run preservation retains **17 protected hashes**, the installed WTF tree, 24 source links/copies and existing service lifetimes. All comparison processes are closed. Further investigation of horizon coverage and its verification is required before accepting or promoting the sky mode. Both experiments remain default-off, demo-only; the running-pose samples do not establish full animation acceptance.

Only authored source/tests/docs are published. Original assets, edited images, MPQ packs, executables, captures, private proofs and SQL backups remain local. No database access, service restart or Native decompilation is required by this work. See [VERSIONS.md](VERSIONS.md) for project and original-client identifiers.
