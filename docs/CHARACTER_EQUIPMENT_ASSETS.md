# Character, armor, weapon and sky asset pass

Prepared **November 3, 2026**, using the supplied local resources and the built-in image tool. Original archives, models, animations, installed settings and database contents are not rewritten. The optional images and packs stay local.

## Character and armor candidates

| Material | Original and reference | Prepared candidate |
|---|---|---|
| Asura male face | `Character\asm_face001_000.dds`, 512×256; referenced by the default `asm_newface_common001.nx3` | Generated RGB atlas **1774×887**, packaged as **1024×512** RGBA8 DDS |
| Warrior armor | `Character\asm_body004a.dds`, 256×512; referenced by `asm_body_warrior001.nx3`, including item **231201**, “Cotte de Mailles de Guerre”, through ItemResource **715** | Generated atlas **887×1774**, packaged as **512×1024** RGBA8 DDS |

Both keep their source aspect ratio and material names. Prompts require the existing facial landmarks, armor panels, belts and buckles to remain in place. Image generation cannot establish exact UV-island fidelity by itself; the original geometry and UV data are preserved and the edited textures are reviewed on those meshes.

The face's alpha is **153 for 130,072 pixels and 255 for 1,000 pixels**. It is not a removable background. Image generation uses an opaque RGB reference, while packaging restores the original alpha independently at the corresponding normalized UV coordinates. At 1024×512, the actual prepared face contains **520,288 alpha-153 pixels and 4,000 alpha-255 pixels**, with **zero mismatches** against the source mask sampled at `(floor(x/2), floor(y/2))`. Armor alpha remains 255 everywhere. No claim about the original material-specific meaning of this mask is added.

Each prepared atlas has **11 file mip levels**, **2,796,332 DDS bytes** and **2,097,152 decoded top-level RGBA bytes**. This is four times the original top-level decoded payload for that material; it is not a claim of reduced total GPU usage. The model texture loader currently decodes the top level. Prepared lower file mips do not establish runtime mip consumption.

## Matched model review

A local offline comparison harness includes the current client `assets.rs` and `models.rs` directly, unchanged. Initial static-part controls exposed an unsuitable head rest orientation. The retained follow-up uses the actual client Asura male rig, the original warrior armor and a fixed idle pose at **1.0 second**, with matched original/edited cameras. It does not log in or run gameplay.

Separate original/original controls precede original/face and original/armor comparisons. The selected source model's geometry, indices, UVs and vertex colors agree, and all its textures resolve. The prepared patch is read through a scratch MPQ chain; source DDS and NX3 bytes remain intact. Three views per comparison support visual review of face features and armor placement. Small pixel differences also occur between identical-source control viewports, so no byte-identical rendering claim is made.

All four follow-up comparison processes exit 0. Across the six edited views, the original left-panel pixel regions remain identical to their controls while the candidate regions change. Post-run checks retain all nine protected-file hashes, the installed WTF tree and twelve source archive links. Visual review also finds an existing head/hair artifact in the rear view, present with both original and edited sources. It is not corrected by changing the source alpha mask; material/channel interpretation and complete head rendering need separate investigation.

These captures establish a fixed-pose model/material comparison, not full in-game equipment, animation-transition, lighting, distance-filtering or multiplayer acceptance. The packs remain optional and uninstalled.

## Weapon and sky limits

The actual sword material `Textures\1handswd_master.dds` is an opaque **256×128** atlas referenced by `World\1handswd_master.nx3` (**222 vertices / 284 triangles / 3 bones**). Its blade, grip and ornaments occupy separate islands; a whole-sword illustration would be incompatible. The built-in image tool rejected this edit. The original, model-reference evidence and attempted prompt are retained; **no remastered weapon image or weapon replacement pack was produced**.

Two sky-gradient candidates are prepared from the original **256²** `Textures\sky_zone001_sky01.dds`, with actual native outputs **1254²**. V2 reduces the first candidate's color drift and horizontal variation, but remains bluer than the reference and is not a color-fidelity acceptance. A separate **1024²** V2 pack is a proposal only.

The client currently clears to a solid sky color rather than loading this gradient. The three inventoried sky NX3 models reference a different **4×4** material on a dome, not six cubemap faces; cloud textures have separate alpha masks. This pass does not activate a skybox or establish which original runtime path consumes the 256² gradient. Sky rendering and cloud integration remain required work.

This describes the initial asset-preparation checkpoint. A subsequent [offline renderer experiment](EXPERIMENTAL_SKY_AND_CHARACTER_RENDERING.md) can display the gradient and mitigate the player head artifact, with both modes explicitly disabled by default. Its strict sky boundary check remains unsuccessful; no original-shader compatibility or production acceptance is claimed.

## Reusable `asset-pack` command

`rz-extractor asset-pack` now prepares rectangular diffuse atlases with an authoritative original alpha reference:

```powershell
cargo run -p rz-extractor --release --locked --offline -- asset-pack `
  --image face-edited.png --original face-original.png `
  --entry 'Character/asm_face001_000.dds' --out new-face-pack `
  --width 1024 --height 512
```

Inputs are 8-bit PNGs up to 4096 pixels per side, with bounded file/decode reads. The generated image must be opaque. Edited, original and output aspect ratios must agree within 1%; target dimensions are powers of two from 32 to 2048. The command permits one canonical supported MPQ folder and DDS basename, rejects path aliases and existing output targets, and creates a separate mono-entry `patch.MPQ` plus a manifest. It does not install or merge patches.

RGB resizing uses Lanczos3 in linear light with clamped edges. Top-level alpha takes the exact original byte at the corresponding floor-sampled UV. Lower mips use a linear-light premultiplied-alpha box filter, stored as straight RGBA8. The existing terrain packer remains unchanged.

**12 extractor tests pass**: seven new atlas tests and five existing terrain tests. They cover mask preservation, rectangular DDS headers/full mip payload, MPQ readback and unrelated entries, source preservation, linear-light/premultiplied filtering, bounded inputs, rejected paths, aspect ratios and existing outputs. Release packaging uses an isolated target directory; installed executables are preserved.

Local sources, prompts, comparisons, preservation checks and private helpers are under `client-data/art/character-equipment-2026-11-03`; packer build/test records are under `client-data/validation/2026-11-03-equipment-tools-6ab3d4e1`. Only authored source and documentation are published. No SQL access, service change or Native decompilation is part of this pass. Software and original-game identifiers remain in [VERSIONS.md](VERSIONS.md).
