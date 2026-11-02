# Global decompilation: verified module checkpoint

Frozen on **2026-11-02**, from the progress snapshot read at **20:49:52 UTC**. The continuous goal remains active. This checkpoint includes **19 native modules** with verified exports, three managed assemblies and the extracted launcher payload. Later arrivals and running batches are excluded. No overall completion percentage is reported: other modules, unidentified code and semantic reconstruction still lack complete denominators.

The supplied NoLimits package label is **1.3.60**. SFrame declares file/product version **9.1.0.0**, contains build lineage **Source9.5.2** and protocol literal **201507080**. The exact official Epic/patch remains unconfirmed; these labels have separate meanings. See [versions](VERSIONS.md).

## Native pseudocode availability

| Module | Internal entries | External entries | New successful texts | Retained detailed dossiers | Errors |
|---|---:|---:|---:|---:|---:|
| SFrame | 56,151 | 582 | 55,557 | 594 | 0 |
| RappelzCmdLauncher | 146 | 61 | 146 | 0 | 0 |
| audiere | 1,576 | 93 | 1,575 | 0 | 1 |
| dbghelp | 4,141 | 198 | 4,141 | 0 | 0 |
| DevIL | 1,923 | 69 | 1,923 | 0 | 0 |
| freetype6 | 994 | 51 | 994 | 0 | 0 |
| ILU | 134 | 43 | 134 | 0 | 0 |
| libcairo-2 | 2,480 | 229 | 2,480 | 0 | 0 |
| libgobject-2.0-0 | 769 | 125 | 769 | 0 | 0 |
| libiconv2 | 79 | 20 | 79 | 0 | 0 |
| libpango-1.0-0 | 795 | 195 | 795 | 0 | 0 |
| libpangocairo-1.0-0 | 311 | 195 | 311 | 0 | 0 |
| mss32 | 887 | 136 | 886 | 0 | 1 |
| libfontconfig-1 | 602 | 108 | 602 | 0 | 0 |
| libglib-2.0-0 | 2,195 | 195 | 2,195 | 0 | 0 |
| libgmodule-2.0-0 | 90 | 58 | 90 | 0 | 0 |
| libgthread-2.0-0 | 103 | 59 | 103 | 0 | 0 |
| libpangoft2-1.0-0 | 751 | 244 | 751 | 0 | 0 |
| libpangowin32-1.0-0 | 279 | 183 | 279 | 0 | 0 |

These censuses contain **74,406 internal entries**, **2,844 external entries**, **73,810 new successful pseudocodes** and **594 retained SFrame dossiers**, with **two recorded failures**. The resulting **74,404 entries with text** measure availability within these provider censuses. Complete bodies, detailed analysis and verified behavior remain separate questions. External entries are excluded from internal attempts. Every included module has a frozen publication, verified official export and operation receipt; SFrame and Audiere retain their historically named verification files.

The retained SFrame **index of 594 dossiers** is distributed between the **737-record principal archive (563 indexed Evidence IDs present)** and **31 separately pinned historical sources**. All 594 have nonempty pseudocode text checked against exact IDs, target, addresses, profile and source hashes. Its verified body-range union remains **324,379 bytes**, separately bounded against **9,589,018 initialized executable bytes**. Bulk text does not enlarge that detailed-body metric.

Text availability does not establish complete recovery: **6 retained pseudocodes report an unrecovered jumptable**, and **24 retained Native value-flow graphs are flagged truncated**. These quality limits are distinct from nonempty text and the two producer failures.

Audiere's `0x100519b2` and MSS32's `0x21138c8d` still fail with `Overlapping input varnodes`. Static bytes and assembly support related x87 `pow`-family math cores. Audiere's checked body is 467 bytes; MSS32's initially heuristic window is now corroborated as the provider's 453-byte body. Nearby nine-byte fragments remain outside their respective provider censuses. These diagnostics add no successful pseudocode and establish neither exact original symbols nor complete floating-point equivalence.

## Supplemental native launcher reconstruction

[The native launcher report](LAUNCHER_NATIVE_RECONSTRUCTION.md) separates the original 146-entry REA census from **four supplemental entry recoveries / 90 bytes / 34 instructions** in a different local analysis profile. The four are counted once across V1–V4 and do not add REA Native dossiers or rows to the table.

The V4 model uses custom **ECX:4 cookie input and EAX:4 preserved-result input/output**. Original bytes, metadata and printed text jointly establish three normal results when the cookie matches: `0xffffffff` for insufficient arguments, zero after successful process creation, and the immediate `GetLastError` result after failed creation. Zero establishes creation of the child process, not its completion. Ordinary two-argument `__fastcall` C does not encode this custom ECX/EAX arrangement.

The invalid-cookie path remains unqualified, including the strength of its inherited `noreturn` annotation. A requested rollback did **not** restore metadata immediately in the observed probe. After analysis-process exit under `-readOnly`, the isolated project files were absent and original/copy bytes unchanged: that observed discard is separate from immediate metadata restoration. No launcher, client or reconstructed implementation was executed or compiled.

## Managed reconstruction

| Supplied assembly | Assembly version | MethodDef rows | Nonzero RVA | Raw IL bodies hash-verified | Reconstructed C# files |
|---|---|---:|---:|---:|---:|
| Ionic.Zip.dll | 1.7.2.26 | 620 | 612 | 612 | 54 |
| Ionic.Zip_2.dll | 1.8.4.31 | 861 | 847 | 847 | 75 |
| mscorlib.dll | 1.0.3300.0 | 14,497 | 11,969 | Not measured by this IL audit | 1,279 |

The **1,459 Ionic bodies / 166,536 raw IL bytes** were checked against the original bytes. Mscorlib's census agrees with ILDasm; its 11,969 nonzero RVAs must not be added to that verified-IL count. REA's earlier mscorlib metadata view remains limited by its 64 MiB representation budget. Unsupported Ionic signature forms remain explicit.

ILSpy **9.1.0.7988** produced **1,408 C# files** across the three targets. A separate isolated pass with matching .NET **2.0.0.0** references reduced Ionic's 108 unknown-type and two unknown-stack comments to zero in the same literal diagnostic scan. This replaces neither the original binaries nor the first frozen pass, and does not add target coverage. Generated files are reconstructed C#, not original source; no project was built, restored or executed.

The auxiliary `Microsoft.NETFramework.ReferenceAssemblies.net20` **1.0.3** pack is a resolution aid, not another game target or the original game runtime. Catalog/transport hashes and cryptographic PKCS7 checks passed. Author/repository certificates report `NotTimeValid` at the audit date; full NuGet ZIP signature, timestamp and revocation validation was unavailable. Exact metadata identity does not establish full package trust or semantic equivalence.

## Packaged launcher and Python reconstruction

Launcher3 yielded **1,012 CArchive entries**, **219 PYZ entries** and **155 nested ZIP/PYC entries**, with source bounds, lengths, compression, hashes and ZIP CRCs checked. Logical paths were never used as output destinations. The raw CArchive and TOC remain preserved locally.

Matching Python **3.12.10** statically decoded **383 code payloads**, **11,102 code objects** and **1,115,106 wordcode instructions**. The application script contains **90 code objects / 20,397 instructions**. Code objects include module/class bodies and generated bodies; they are not a function-only or gameplay denominator. Six presentation summaries have bounded literal/argument previews; raw payloads and instruction offsets/opcodes remain complete.

[The Python reconstruction report](LAUNCHER_PYTHON_RECONSTRUCTION.md) records two attempts over the same 90 application objects:

| Result | Pass 01 | Pass 02 |
|---|---:|---:|
| Syntax-only success | 42 | 22 |
| Partial | 41 | 61 |
| Failed | 6 | 7 |
| Wall timeout | 1 | 0 |
| AST-valid fragments, overlapping the rows above | 82 | 82 |

Each pass emitted 83 fragments. Pass 02's stricter qualification preserves definition/default, closure and exception obligations; the label changes do not measure a functional regression. Its **90 guard receipts contain 540 passed checks of six distinct protections**, with no unexpected guard event. **12 authored-fixture checks** passed. The four priority receipts were reused, followed by 86 other attempts. Pass 01 had only **89 final receipts** because of its timeout; available receipts passed their guards, while its aggregate guard flag remains false.

The obligation ledger retains **14 definitions with parent-supplied defaults**, **19 objects needing closure context** and **42 objects with exception regions**. Seven objects still fail on unsupported Python 3.12 generator handling. The root module and the formerly timed-out news routine remain partial; a bounded diagnostic is not successful reconstruction. A generator suspension/resumption model, complete exceptional control flow and behavioral validation remain future work. No target/reconstructed source was compiled or executed; fixture compilation created test bytecode only. No original-source or semantic-equivalence claim follows from AST validity.

The **68 extracted PE objects are x64**, have no CLI header and remain outside the native table above. Their independent function censuses and native decompilation are future work. Static import names yield 211 candidate links, without proving runtime loading. Bundled support libraries are not automatically game logic; publisher/license provenance remains unverified.

## Frozen publication pins

The private `coverage-checkpoint-04.local.json` supersedes checkpoint 03, preserves the previous public report byte-identically and pins every included artifact, official export receipt, frozen proof and the two launcher publication manifests. The launcher documents were linked without modification.

| Linked report | Document SHA-256 | Local publication manifest SHA-256 |
|---|---|---|
| [Native launcher](LAUNCHER_NATIVE_RECONSTRUCTION.md) | `80d5249db16883d7828e3d49dffcf2336641f13891e04a7f79d9aa4befddcb58` | `a0c55c707c1d513f02fb03444ac74a4a5a004d364bf4d0da9667dc780ad03c0d` |
| [Python launcher](LAUNCHER_PYTHON_RECONSTRUCTION.md) | `969adb1f736cc0220ac6c4384616e2e85a37801504d51855a740433e3dc391a8` | `8384828a251d2ac87a8502a789b44c414e42c8a9169d83afc5da88290ce9ae31` |

The manifests are `launcher-native-reconstruction-publication.local.json` under the CmdLauncher audit and `launcher-python-reconstruction-document.local.json` under the global audit. Their source lists and the pass-02 evidence manifest were freshly hash-checked. Earlier bulk-source pins are inherited explicitly from the immutable checkpoint 03; the large retained archive was not reread.

Raw binaries, reconstructed code and detailed evidence remain local. This documentation update changes no original artifact, game data, settings or running service. Functional reconstruction and runtime validation remain ongoing.
