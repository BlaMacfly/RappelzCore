# Ground texture detail: before and after

Verified **2026-10-31** following the report that the floor looked enlarged/pixelated. Ground source textures were being blended into a **128×128** image stretched over a **1008×1008 world-unit chunk**: only about 21 output texels for a typical 168-unit repetition. Most source detail was discarded before GPU filtering could display it.

The client now blends at **1024×1024**, eight times the linear resolution, with wrapped bilinear source sampling. Existing texture names, DETAIL repeat distances, layer weights, vertex lighting and terrain geometry are retained. Blended textures still use linear GPU filtering. Chunk construction is limited to **one per frame**, rather than two, to spread the higher construction cost. The existing distance-based chunk eviction remains in use.

Two native offline sessions used the same independent preview root, original MPQs, 1280×720 resolution, position **8440/7227**, camera and **120 frames**. The previous installed executable produced `terrain-before.png`; the validated new executable produced `terrain-after.png`. Both exit **0**, both images were visually inspected, and ground detail is substantially clearer while walls/map/player placement agree. Preview preferences are isolated from the installed WTF directory. No original texture or MPQ was modified.

Measured end-to-end time was **7,127 ms before / 9,604 ms after**, including asset loading, terrain construction, 120 frames and shutdown. This is **not a steady-state FPS measurement**. The higher density increases GPU storage from 64 KiB to 4 MiB per chunk; the 7×7 visible cache can hold about **196 MiB of baked RGBA textures per client**, plus existing resources. Broader hardware/distance testing and mip/quality options remain possible follow-ups; the original renderer is not claimed pixel-identical.

The final two-client corpse scenario also passes **48/48 checks** with this renderer. Full Rust/Python results and data preservation are recorded in [SKILL_CORPSE_VALIDATION.md](SKILL_CORPSE_VALIDATION.md). The installed/validated client SHA-256 is `0F6928115EFC771EAEC620EB85EBEB342E51C6D17DCA2748DA4DFDB7E562CEB0`.

Images, logs and timing JSON remain local and ignored under `client-data/validation/2026-10-31-corpse-skill-050a8eb7/`. These screenshots include supplied game assets and are not publication inputs. Reproduce using an independent client root:

```powershell
RappelzRust.exe -data <preview-root> -demo -pos 8440,7227 -screenshot <output.png> -frames 120
```
