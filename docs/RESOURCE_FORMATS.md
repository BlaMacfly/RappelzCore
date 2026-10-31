# Resource formats and packaging

Audit date: 2026-10-30. Existing detailed format evidence remains authoritative in [re/data_archive.md](re/data_archive.md), [re/rdb_format.md](re/rdb_format.md), [re/terrain_format.md](re/terrain_format.md), [re/model_format.md](re/model_format.md) and [re/ui_format.md](re/ui_format.md). This file connects that evidence to the Rust implementation and distinguishes packaging from format conversion.

## Original client resources

`shared/rz-rappelz/src/archive.rs` reads the original `data.000` index and `data.001`–`data.008` payload archives. The index uses a rolling XOR key and encoded flat filenames; payload archive selection is derived from the encoded name. Content decryption depends on file type. Original archives remain available as read-only development/conversion input.

The real-client test asserts 125,282 index entries and verifies JPEG/DDS signatures after reading example entries. That count concerns the original index. The converter filters database resources from the common archive pass, whose historical log reports 125,212 entries; these numbers describe different populations.

The real-resource test exercises loose RDBs and the item reference table when supplied. Most RDBs have a 128-byte header followed by a four-byte count and fixed-size records. Strings and item references have dedicated parsers; monster IDs have a bit permutation. Keep observed field sizes separate from inferred names/semantics. Tests that return early when original assets are absent cannot establish real-file coverage in another checkout.

## Conversion and installation

`rz-extractor` provides `client`, `verify`, `dbc` and `addons` operations:

| Operation | Actual role |
|---|---|
| `client` | Read original archives/loose resources, arrange assets into MPQ paths, convert supported RDBs into WDBC tables, emit locale and client configuration. |
| `verify` | Compare original non-database archive entries byte-for-byte against their mapped MPQ resources. It does not validate every decoded model/terrain behavior or every generated DBC. |
| `dbc` | Extract converted `DBFilesClient/*.dbc` from installed MPQs into the server's `DataDir/dbc`. |
| `addons` | Install missing RappelzDebug TOC/Lua/XML files into an existing installation, preserving existing addon edits; also included by `client`. |

The generated layout currently uses `Data/common.MPQ`, numbered `common-2.MPQ` etc., `Data/<locale>/locale-<locale>.MPQ`, locale `realmlist.wtf`, and `WTF/Config.wtf`. The original asset name is retained inside its new path. Loose `Resource/music` and `Resource/jpg` files are also packaged. Loose RDBs override archive copies when building locale data; unsupported table conversions are retained under a raw fallback path rather than fabricated as valid DBCs.

The local converted installation contains `common.MPQ`, `common-2.MPQ` and `frFR/locale-frFR.MPQ`. Historical `client-data/extract.log` records 25 converted DBC tables and completion in 38 seconds on 2026-10-25. This is retained evidence, not a new conversion or a current performance guarantee.

Wrapping an NX3, NAF, NFM or NUI file in MPQ does not convert its internal format to WoW. RappelzRust uses its own `rz-model`, `rz-terrain`, `rz-ui` and `rz-texture` readers. DBC uses the WDBC container but carries Rappelz table definitions and is not a set of stock WoW gameplay tables.

## Archive lookup and patching

`shared/rz-mpq` supplies an MPQ v1 reader/writer and `MpqChain`. Each newly pushed archive has higher priority; lookup searches from last to first. The existing automatic mount groups, from low to high priority, are:

1. `Data/common.MPQ` and numbered common archives.
2. `Data/expansion.MPQ` and numbered expansion archives.
3. `Data/<locale>/locale-<locale>.MPQ` and numbered locale archives.
4. `Data/patch.MPQ` and numbered patch archives.
5. `Data/<locale>/patch-<locale>.MPQ` and numbered locale patches.

Within each group the unnumbered archive has rank 1; numbered suffixes of 2 and above sort numerically, so `patch-10.MPQ` overrides `patch-2.MPQ`. Discovery accepts gaps and works even when the unnumbered archive is absent. Prefix/extension matching is case-insensitive. These rules were added and tested in this development cycle. Patch behavior means whole-file override by path; no binary-delta patch format or deletion-tombstone support has been established.

Path hashing normalizes case and separators. Client `Assets` first tries the full MPQ path and can fall back through a basename index built from `(listfile)` records. Use full paths when basename ambiguity matters. Locale choice selects the locale directory; there is no demonstrated automatic fallback chain across multiple locales.

## Formats, caching and integrity

| Component | Implemented scope / practical limits |
|---|---|
| MPQ | v1 tables, path hashes, encrypted table/file reading, zlib compression, single-unit/sector reads, writer/listfile support. Unsupported compression reports an error. Files/archives remain subject to v1 32-bit format limits. |
| WDBC | Read/write 20-byte WDBC headers, fixed-size records, string block offsets and per-table format checking. |
| Textures | DDS DXT1/3/5 and supported uncompressed formats, plus TGA/JPEG/PNG/BMP decoding to RGBA8; only the first mip level is decoded. |
| Models/terrain/UI | Dedicated Rappelz parsers and client rendering code; consult detailed RE notes for supported versions and unknown fields. |
| Caching | Client texture cache includes failed lookups; model caches and an audio-effect cache are separate. There is no demonstrated global byte-cache budget/eviction policy or automatic patch hot reload. |

MPQ parsing checks selected lengths, sector offsets and decompression results. The sector-CRC flag affects table layout, but per-sector CRC validation and a cryptographic asset manifest are not implemented by the inspected reader. Byte comparison through `rz-extractor verify` is the strongest existing conversion integrity check for its covered entries; it is not a signed update mechanism.

All four MPQ tests passed in this development cycle. They cover known crypto keys, crypto round trips, write/read/listfile behavior, and the new `expansion_locale_and_sparse_numbered_patches_override_in_numeric_order` regression. That test mounts all five groups, confirms an expansion-only file remains accessible, gives locale patches priority, and verifies numeric patch ordering when the base patch and intermediate numbers are missing. DBC tests exercise read/write round trips, and original-asset tests cover archive/RDB data when present. The earlier README mentions StormLib validation; the current `rz-mpq` test tree contains no independent StormLib test fixture, so that claim is historical rather than newly reproduced here. See [DEVELOPMENT_STATUS.md](DEVELOPMENT_STATUS.md) for the complete test/extraction results from this session.

Next work should establish bounded cache policies, extend format support from verified evidence, and add a reproducible integrity manifest. Preserve existing converted assets and do not rerun conversion casually: the current converter removes matching common archives in its chosen output directory before rebuilding.


## Isolated archived corpse profile and ground detail, 2026-10-31

The normal RDB collector deliberately lets Resource loose files replace archive entries. For3212 this produces a valid conversion of a different self-recovery rule:15fields differ from the archived corpse511 profile, while SkillStage/tooltip retain corpse targeting. `rz-extractor corpse-skill --client <original-root> --out <data-root>` reads archive entries directly and creates only CorpseSkill/Skill.dbc and SkillStage.dbc. Existing normal DBCs/MPQs and original resources are preserved; conflicting companion files are refused. Full client conversion and server DBC extraction also carry this isolated profile when available. See [SKILL_CORPSE_VALIDATION.md](SKILL_CORPSE_VALIDATION.md) for hashes, flags, formulas and native evidence.

The floor-quality correction changes runtime blending resolution/filtering, not packaged textures or DETAIL repeat distances. Matched native captures demonstrate128²→1024² ground detail and record the additional construction/storage cost in [TERRAIN_RENDERING_VALIDATION.md](TERRAIN_RENDERING_VALIDATION.md). No full archive repack/reimport was needed.

The two-client terrain follow-up keeps the 1024² blend and original source data unchanged. Conservative camera bounds and per-frame texture eviction reduce the observed baked-texture peak from 252 to 84 MiB per client. All 12 ground captures match exactly; see [TERRAIN_RENDERING_VALIDATION.md](TERRAIN_RENDERING_VALIDATION.md) for separate Windows GPU accounting and warmup/hardware limits.
