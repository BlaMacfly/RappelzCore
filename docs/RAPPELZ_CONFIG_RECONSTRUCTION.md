# RappelzConfig: bounded static configuration contract

Checked **2026-11-02** against `Rappelz NoLimits/RappelzConfig.exe`, SHA-256 `065d4a6196b63aca3a4252621b67e1e21c92791c9c5b1b0cc67d7fd17e3d1b13`. This report describes retained decompiler text and selected original instruction windows. It does not establish complete source recovery, a running configuration session or compatibility of the reconstructed client. Game/package version labels remain separate in [VERSIONS.md](VERSIONS.md).

## File access and resolution fallback

The text at `0x40c480` constructs a current-directory path ending in `\rappelz_v1.opt` and probes a binary read open. Original instructions corroborate the call to `GetCurrentDirectoryA`, path-literal use and the AL-valued result paths. The internal open/close routine names come from the analysis database; their full CRT implementations were not independently reconstructed here. Long-path and directory-query failure behavior remain unqualified.

The integer-reader window contains an IAT-bound call to `GetPrivateProfileIntA` at `0x40b460`, with a pushed default of **−1**. The resolution-loading instructions then store the two results and compare them as signed 32-bit values:

| Stored field | Condition | Replacement | Original store |
|---|---|---:|---|
| Width, object `+0x144` | Value is nonpositive | 1,024 | `0x40b5c6` |
| Height, object `+0x148` | Value is nonpositive | 768 | `0x40b5d4` |

This is a fallback in this configuration program. It does not prove which display modes the original renderer accepts, nor impose a maximum on positive values.

The write-helper window separately contains an IAT-bound call to `WritePrivateProfileStringA` at `0x40c648`. Extra stack arguments, value formatting and section selection remain incompletely typed. This call alone does not prove a complete save/reset transaction.

## Setting bindings remain candidates

Retained text at `0x40a3d0` contains **77 distinct straight key/offset pairs** passed through the same helper sequence. Examples include:

| Textual key | Candidate object offset |
|---|---|
| `opt.ev.GRAPHIC_RESOLUTION_WIDTH` | `+0x144` |
| `opt.ev.GRAPHIC_RESOLUTION_HEIGHT` | `+0x148` |
| `opt.ev.GRAPHIC_MIPBIAS` | `+0x17c` |
| `opt.ev.GRAPHIC_RESOURCEQUAL` | `+0x180` |

The first local extraction found 75 pairs because its pattern accepted hexadecimal offsets only. Independent review also captures decimal `400` / `GRAPHIC_LOWSHADOW` and `600` / `PLAY_DAMAGE`, giving 77. The earlier local report is preserved with this explicit correction.

The binding text spells one name `GRAPHIC_BACKGRONDDIS`, whereas the profile-reader text uses `GRAPHIC_BACKGROUNDDIS`. These are different namespaces and routes. Their relationship and any visible defect remain unresolved; no spelling or setting was changed. The registry's scalar types, complete object construction, event delivery and renderer consumption still need evidence.

## Evidence and remaining work

Independent review authenticates ten selected text projections against two retained REA Evidence records, then checks five bounded instruction windows and every retained instruction byte against the original PE. Import names are bound through the original IAT. Windows are **not** complete-function body claims; this adds no detailed Native dossier or semantically complete function.

Private proofs are retained under `client-data/audit/2026-11-02-module-rappelzconfig-9fb0a464/`:

| Proof | SHA-256 |
|---|---|
| `settings-independent-review-03/report.local.json` | `2b0c5e0e0a2167d52b0ef7356f690c6cd6e7932bbc8d02698a7c626ee681b670` |
| `settings-independent-review-03/verification.local.json` | `0459a2c61cb8c4c2476b5b9533f6bdc40160496b0c07fe4c5d12228ec4087a17` |
| `settings-independent-review-03/evidence-manifest.local.json` | `7293018780ed5828f99ef5a43c69aa61157386d8cf1b60e44deb3cf8d0833b75` |

Full ABI and exceptional paths, inherited no-return annotations, original UI save/reset behavior and consumption by the renderer remain open. No original executable or reconstructed source was executed or compiled. Installed game data and settings were unchanged. See the [global module checkpoint](GLOBAL_DECOMPILATION_MODULES.md) for the separate census and text-availability metrics.
