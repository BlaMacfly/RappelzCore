# Client asset modernization

## Scope and current results — November 3, 2026

Modernization covers the client's art beyond terrain: inventory/skill icons, interface artwork, scenery, equipment, characters, creatures and effects. Work starts from the locally supplied original resources and preserves the painted fantasy style. Generated assets are candidates for review, not a claim that every category has been converted.

| Asset | Prepared result | Verification and remaining work |
|---|---|---|
| Olive grass | Shorter painted blades in selected V3; 1254² native output, separate 1024² DDS pack | Fresh two-client offline comparison passes 278 checks in four views. Repetition, mixed/direct transitions and connected validation remain open. See [terrain validation](TERRAIN_HD_ASSET_VALIDATION.md). |
| Blue potion inventory icon | Faithful repaint of `Interface\Icons\icon_item_potion_bluepotion_lv1.jpg`; original 34², native edit 1254², prepared opaque 256² JPEG at quality 95 | Item **602201**, “Potion Bleue Niv.1”, references this image through ItemResource index **3118** and normal extension fallback. Independent scratch-MPQ readback resolves the replacement exactly and preserves an unrelated icon. Native inventory display quality remains unvalidated. |
| Wrecked ship wood | Repaint of `Textures\rondoh_renka_lake_wreckedship01_mainwood.dds`; original 512², native edit 1254², separate 1024² RGBA8 DDS pack with 11 file mips | Existing NX3 parser establishes the texture reference in the ship model. Independent scratch-MPQ readback preserves original texture/model and resolves the replacement exactly. Native model rendering remains to be validated. |

The blue vial keeps its diagonal silhouette, blue liquid, brass stopper, dark background and thin grey frame. Its prepared decoded image uses 262,144 RGBA bytes, versus 4,624 at the original 34² size; this is an image payload comparison, not total client memory. It is a standalone image rather than an interface atlas.

The table stores the extensionless name `icon_item_potion_bluepotion_Lv1`. The current resolver tries the bare name, then TGA, PNG, JPG, DDS and BMP. The first three candidates are absent in the original three-archive chain, so the existing JPG is selected by basename. This establishes the resource reference, not a native inventory screenshot or a change to UI scale.

The wood keeps dark brown/olive weathering and horizontal boards. `World\rondoh_renka_lake_wreckedship01.nx3` references it on a mesh with **597 vertices / 494 triangles**. Its UVs range approximately **−8.13…12.09 / −3.16…2.35**, making repeat continuity especially important. Geometry and UVs are unchanged. Image generation does not prove exact seam-coordinate or edge continuity preservation: inspect the actual textured model before accepting it. Terrain-specific texture-sharing/mip budgets do not apply automatically to the model renderer.

The four images in this initial grass/icon/scenery pass use the **built-in image tool**, with originals inspected first. Both grass variants, the icon and the wood are actually **1254×1254**; requested larger dimensions are not claimed. Original sources, prompts, rejected/alternative variants, exact readbacks and local comparison pages remain under `client-data/art/terrain-hd-2026-11-03` and `client-data/art/asset-expansion-2026-11-03`. The latter page includes original/remastered icon previews at 34/68/136/256 pixels and repeated wood previews. No generated image, original game texture or MPQ is published.

The subsequent [character/equipment pass](CHARACTER_EQUIPMENT_ASSETS.md) prepares a default Asura face and warrior armor, preserves the original face alpha mask, and reviews them with the client rig renderer in a fixed offline pose. The new `asset-pack` command supports rectangular atlases and original alpha references. Sky gradients remain proposals because the current client uses a solid background; the attempted sword edit was rejected by the image tool, so no remastered weapon is claimed.

A follow-up adds [explicit offline rendering experiments](EXPERIMENTAL_SKY_AND_CHARACTER_RENDERING.md) for a world-oriented sky gradient and player alpha cutout. Ordinary rendering remains unchanged. The player experiment removes the sampled rear head artifact without rewriting textures. Sky sources draw in six completed clients, but the strict horizon-pixel containment check fails; acceptance remains open. Work is stopped at the user's request, with all evidence preserved.

## Integration rules and next work

- Keep each original resource name and material reference. Preserve silhouettes, palette, normalized texture layout, UVs and channel meanings; review transparent masks and atlas regions separately.
- Use new optional output packs and scratch clients. These packs are currently separate mono-entry `patch.MPQ` files, not a composed or installed pack. Do not overwrite one pack with another or replace an existing installation patch.
- Check actual resource-table references and resolution precedence before claiming an icon is active. A file's presence in an archive is insufficient.
- Validate the new icon inside the real inventory at native/scaled UI sizes, then validate the ship with fixed original/edited model views, repetition, lighting and memory observations.
- Expand to a coherent icon family, interface panels/buttons, then scenery and equipment/character textures. Atlas and skin work requires stronger layout/alpha checks; models, animation and effects are separate asset tasks and are not completed by repainting diffuse images.

No server or SQL change is needed for this preparation. Installed assets/settings remain untouched. The latest terrain checks are offline and do not establish original SFrame or multiplayer interoperability. Core/client version identifiers remain documented in [VERSIONS.md](VERSIONS.md). Decompilation is handed to Claude through a separate local `memo.md`; this asset pass launches no Native analysis engine.
