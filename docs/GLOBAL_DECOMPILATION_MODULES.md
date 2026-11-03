# Global decompilation: verified module checkpoint

Frozen at **2026-11-03T04:30:23.578626+00:00 UTC**. The continuous goal remains active. This checkpoint separates **35 automatic original-native censuses**, **two special original-native censuses** and **one derived nonoriginal PE census**. Three managed assemblies and the extracted launcher payload keep their independent scopes. Running or prepared future analyses are excluded. No reliable overall completion percentage is available.

The supplied NoLimits package label is **1.3.60**. SFrame declares file/product version **9.1.0.0**, contains build lineage **Source9.5.2** and protocol literal **201507080**. The exact official Epic/patch remains unconfirmed; these labels have separate meanings. See [versions](VERSIONS.md).

## Native pseudocode availability

The table preserves the closed bulk-census partition: SFrame's 594 retained dossiers and 55,557 later bulk texts are disjoint availability groups at that freeze. New detailed dossiers revisit entries already represented by bulk text and are counted only in the continuation section below.

| Module | Internal entries | External entries | New successful texts | Retained at bulk-census freeze | Errors |
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
| libpng14-14 | 435 | 49 | 435 | 0 | 0 |
| zlib1 | 196 | 37 | 196 | 0 | 0 |
| intl | 256 | 85 | 256 | 0 | 0 |
| libexpat-1 | 200 | 14 | 200 | 0 | 0 |
| RappelzConfig | 3,572 | 291 | 3,572 | 0 | 0 |
| SplashLoading | 272 | 97 | 272 | 0 | 0 |
| cario | 4,800 | 239 | 4,798 | 0 | 2 |
| d3dx9_31 | 5,754 | 176 | 5,754 | 0 | 0 |
| d3dx9_40 | 7,819 | 206 | 7,818 | 0 | 1 |
| ILUT | 85 | 83 | 85 | 0 | 0 |
| libexpat | 321 | 48 | 321 | 0 | 0 |
| libgio-2.0-0 | 1,017 | 281 | 1,017 | 0 | 0 |
| libpng12-0 | 445 | 42 | 445 | 0 | 0 |
| lua5.1 | 1,182 | 93 | 1,181 | 0 | 1 |
| Win98sUpdateUtil | 1,976 | 329 | 1,976 | 0 | 0 |
| xdelta3 | 836 | 87 | 836 | 0 | 0 |
| unins000 | 2,935 | 425 | 2,935 | 0 | 0 |
| NoLimits_Launcher_3 | 813 | 142 | 813 | 0 | 0 |

The **37 original-artifact censuses** contain **107,320 internal entries**, **5,568 external entries**, **106,720 new successful texts** and **594 retained SFrame dossiers**, with **6 recorded failures**. Their **107,314 entries with text** measure availability within these provider censuses, not complete bodies, original source or verified behavior. External entries are excluded from internal attempts. Unins is inherited from special01 once; it is not counted again by special02.

The derived PE is counted separately:

| Derived artifact | Internal entries | External entries | Successful texts | Errors |
|---|---:|---:|---:|---:|
| uninstall UPX PE13 derivative | 20,510 | 786 | 20,510 | 0 |

Across the **38 distinct analyzed target artifacts**, availability totals **127,830 internal entries / 127,824 with text / 6 errors**. This mixed original/derived total does not mean 38 original native files are complete. The packed original `uninstall.exe` still lacks its own stub census; a nonexact derivative cannot silently satisfy that original scope.

The historical retained SFrame **index of 594 dossiers** is distributed between the **737-record principal archive (563 indexed Evidence IDs present)** and **31 separately pinned historical sources**. All 594 have nonempty pseudocode text checked against exact IDs, target, addresses, profile and source hashes. Its historical verified body-range union is **324,379 bytes**, separately bounded against **9,589,018 initialized executable bytes**. Bulk text does not enlarge that detailed-body metric.

Text availability does not establish complete recovery: **6 retained pseudocodes report an unrecovered jumptable**, and **24 retained Native value-flow graphs are flagged truncated**. These quality limits are distinct from nonempty text and the six producer failures.

Audiere's `0x100519b2` and MSS32's `0x21138c8d` still fail with `Overlapping input varnodes`. Static bytes and assembly support related x87 `pow`-family math cores. Audiere's checked body is 467 bytes; MSS32's initially heuristic window is now corroborated as the provider's 453-byte body. Nearby nine-byte fragments remain outside their respective provider censuses. These diagnostics add no successful pseudocode and establish neither exact original symbols nor complete floating-point equivalence.

The other three failures have different causes. `cario.dll` retains two dead-decompiler-process errors at `0x100e9780` and `0x101300f0`; it is a separate target from `libcairo-2.dll`. `d3dx9_40.dll` retains one response-buffer-limit error at `0x1010ad70`. They remain failed items in their original batches, with no text success inferred from the rest of each module. The new `lua5.1.dll` census retains one `Overlapping input varnodes` failure at `0x1001a78d`; its failed item was not resubmitted. The six errors are **Audiere 1 / MSS32 1 / cario 2 / d3dx9_40 1 / Lua 1**.

The **38 final export bundles** total **234,476,401 bytes**. These are evidence-storage sizes, not executable-body coverage. The three special additions have independently checked subject/profile bindings, canonical selectors, nonempty successes, explicit errors and immutable operation markers. Existing official `parseEvidenceBundle` receipts were reused only after their exact export hashes and counters reconciled; no parser or native producer was replayed by this checkpoint.

The original inventory remains **38 native PE files plus three managed assemblies**. Thirty-seven original native targets now have closed provider-census attempt sets. The additional UPX PE13 target is explicitly **derived**, while its packed original remains open. The 68 embedded PE objects keep a separate uncensused scope. Closing an initial provider census does not finish global code or semantic reconstruction.

## Recovery after the host crash

The host restarted at **22:06:14.500000 UTC**. The earlier `xdelta3.exe` attempt had reached its inventory request but had produced no inventory response, bundle, queue database or completed batch. Its files were retained unchanged; it was never marked successful. A root-reviewed fresh audit after confirming the old processes absent recovered **836 internal entries / 836 texts / zero errors**. Only that completed audit contributes to this cohort; no old partial result was counted twice.

Post-restart hashes confirm all **41 original PE files**, **11 protected files**, the installed settings tree and the logical SQL backup unchanged. The SQL/game services were not listening after the reboot and were not started for this static audit. Live SQL contents were not rechecked; the preserved backup and absence of analysis writes do not replace a live database validation.

A fresh checkpoint08 root preservation pass again confirmed all41 original PEs,11 protected files, the entire installed WTF tree and the logical backup unchanged. It did not start services or verify live SQL contents. The module-progress input was copied into an immutable snapshot; its35 original rows remain unchanged, with CRT and Native supplements separate.

The root-selected checkpoint09 preservation receipt again confirms41 original PEs,11 protected files, the entire installed WTF tree and the logical backup unchanged. It neither starts services nor verifies live SQL.

## Bounded configuration study

[The RappelzConfig study](RAPPELZ_CONFIG_RECONSTRUCTION.md) links **77 retained-text binding candidates**, ten authenticated projections and five original-byte windows. It corroborates independent signed width/height fallbacks to 1024/768, the integer-read default of -1 and the profile-write API call. Full object ABI, save/reset transactions, renderer consumption and runtime compatibility remain unresolved. The key spelling difference is not an established user-visible defect. This later authored summary adds no Native dossier or complete function and does not change the frozen 35-module selection.

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

The obligation ledger retains **14 definitions with parent-supplied defaults**, **19 objects needing closure context** and **42 objects with exception regions**. Those pass-01/pass-02 receipts remain unchanged. The root module and formerly timed-out news routine remain partial.

[The separate generator pilot](LAUNCHER_PYTHON_GENERATORS.md) subsequently retried only selectors **3, 25, 28, 2, 27, 29 and 30** under a changed, reviewed normal-path model. It produced **seven AST-valid partial results**, with zero child failures or timeouts, **14 authored-fixture checks** and **42 passed repetitions of six guard tests**. The other **83 results are inherited from pass 02**, not new operations. The reference overlay contains **22 syntax-only labels / 68 partial results / 89 AST-valid fragments**, with one inherited syntax-invalid fragment. It adds no semantically complete function.

Its **eight context occurrences represent six generator identities**, not eight additional recoveries; objects 25 and 28 remain distinct despite equal bytecode bytes. Only discarded resume values in the normal `YIELD_VALUE → RESUME 1 → POP_TOP` pattern are modeled. PEP479, complete exceptional behavior, closure bindings, defaults and parent iterator timing remain unresolved. No target/reconstructed source was compiled or executed; fixture compilation created test bytecode only. AST validity establishes neither original-source recovery nor semantic equivalence.

The **68 extracted PE objects are x64**, have no CLI header and remain outside the native table above. Their independent function censuses and native decompilation are future work. Static import names yield 211 candidate links, without proving runtime loading. Bundled support libraries are not automatically game logic; publisher/license provenance remains unverified.

## Supplemental SFrame CRT recovery

[The CRT initializer report](SFRAME_CRT_INITIALIZERS.md) now accounts for the finite missing startup frontier: **581 distinct local identities / 581 nonempty texts / 14,608 disjoint original body bytes / 3,769 instructions**. It combines the earlier C10 and C++15 with the completed remaining555 plus separate rank-six case. These custom profiles add **zero REA Native dossiers** or formal Native body bytes and leave the bulk table unchanged.

The new 556 texts retain **30 embedded warnings** (29 globals-overlap, one alloca-probe injection). All 25 indirect-call callbacks were included: 28 calls through six declared slots, corroborated by read-only import references; runtime implementation addresses and successful API effects remain unknown. The closed cyclic body does not prove termination. Rank six is one function with two disjoint ranges (10 + 47 bytes / 12 instructions), after actual exact/containing probes found no owners. Destination 0xbca73a is not an exact initial-census entry; no neighboring owner, split or merge was forced.

The completed explicit unsigned32/ANALYSIS return model aligned HighFunction to uint4/EAX4 and changed rendered text for **both known returns (2/2)**. Nonreturn ABI/callee metadata stayed unchanged. It adds no identity and does not recover original signedness or a debug prototype. Signature priority also locks an existing empty input list; that analytical consequence is documented.

All 556 primary results were forced to fresh per-entry files and reconciled with final reports. The verifier passed 35 fixtures and checked source/table/body/import/closure bindings. Immediate restoration remained false; discarded read-only projects are separate observations. The run's earlier Java PID was not captured: closure relies on owned wrapper/batch/completion receipts plus a historical final zero-Java snapshot. Raw pseudocode remains local; no automatic replay of crash partials is authorized.

## Detailed Native continuation

Checkpoint **13**. Bulk censuses 38, available texts 127824 and original failures 6 are copied unchanged from the frozen prior. No global semantic completion percentage is inferred.

| Closed metric | Value |
| --- | ---: |
| qualified dossiers | 2084 |
| initialized body bytes | 693392 |
| acquired case views | 214 |
| new Native producer results | 213 |
| local source operations | 851 |
| retained source record references | 852 |
| notation existing IDs | 63 |
| pending source cases | 1 |
| warning occurrences | 11 |
| truncated valueflow graphs | 3 |
| finished warning occurrences | 10 |
| finished truncated valueflow graphs | 2 |
| pending warning occurrences | 1 |
| pending truncated valueflow graphs | 1 |

Case views and new producers are separate. Inherited source completion may qualify true or false. The final public export contains local operations; portable proofs may retain inherited identities. Source counts come from exact records and never a four-per-case formula. The reported body ranges are inclusive; CFG ends remain exclusive and are not used as body endpoints.

Notation qualifies 63 existing IDs and creates zero producers, functions or Evidence IDs. The 63 original finished false projections remain unchanged. All 63 cases enter the qualified corpus through the separate notation model; one acquisition remains source-deferred in this closed scope. Warning/truncation totals cover all acquired views; the separate finished and pending rows retain their own units. Provider limitations remain private source evidence. Bodies are deduplicated within 9589018 initialized executable bytes; they do not prove exhaustive original functions or game semantics.

Only the explicitly selected closed proofs enter this freeze. Future and mutable results are excluded. Previous checkpoint/source pins, both authored reports, version labels and frozen CRT/UPX leaves are preserved. Live SQL contents were not verified. No provider, engine, official parser, target program or Git operation is run by this workflow.

## Special original artifacts and the UPX derivative

Original `unins000.exe` now has **2,935 internal entries /2,935 texts /zero errors**. Its qualified10,586-byte Inno-message tail and207 strings remain data-format evidence, not extra functions or game-version labels. Original `NoLimits_Launcher_3.exe` now has **813 internal entries /813 texts /zero errors** in its native bootloader census. Its Python payloads and68 embedded PEs are not covered by those813 native entries.

The separate derived uninstall PE retains SHA-256 `5ef772bfd3c163011967a6c5f8eacb53df2f653337594b0529e548f2296e76d1`, size3,805,696bytes, and the original packed-source → exact UPX5.3.0 tool → derived-output lineage. Its completed census appears only in the derived table. The original packed `uninstall.exe` is not counted as censused by the derivative. Legacy PE13 reconstruction remains nonexact: neither byte identity with a historical unpacked original nor semantic equivalence is established. The uncompressed-stream Adler32 was not independently established. No original, packed copy or derived executable was launched.


[The original packed-stub report](UPX_PACKED_STUB.md) now qualifies **184 static instruction decodings /432 physical bytes**, with three overlapping decode pairs sharing four bytes. It identifies declared import calls and a conditional normal handoff matching the derivative entrypoint. This bounded original-byte audit adds no original census or Native dossier; compressed bytes were not treated as code, and runtime behavior remains unobserved.

## Windows controller recovery

A263-character temporary output path stopped the Unins controller after its first batch had already been acquired. The Windows adapter now uses a shorter sibling temporary name for atomic publication. **39 checks pass:30 existing controller-integrity cases plus nine new temporary-path cases.** The same warm workerPID7144 and its original first-batch response/marker were retained; only the controller was replaced. That first batch was not produced again.

The three final special censuses are counted from closed exports and verifications. Warm128-batch phases and mutable progress journals are not promoted to final success. The isolated Windows adapter and its tests are authored tools; raw targets, pseudocode and private evidence remain local.

## Frozen publication pins

Root-selected checkpoint **13** preserves the prior checkpoint and both reports byte-identically. This freeze lists 24532 exact source pins, admits only the selected closed gate/notations, and reuses existing official parser proofs without replay. Future results, live SQL verification and global semantic completion are not claimed.

| Frozen launcher report | Document SHA-256 | Local publication manifest SHA-256 |
|---|---|---|
| [Native launcher](LAUNCHER_NATIVE_RECONSTRUCTION.md) | `80d5249db16883d7828e3d49dffcf2336641f13891e04a7f79d9aa4befddcb58` | `a0c55c707c1d513f02fb03444ac74a4a5a004d364bf4d0da9667dc780ad03c0d` |
| [Python passes 01/02](LAUNCHER_PYTHON_RECONSTRUCTION.md) | `969adb1f736cc0220ac6c4384616e2e85a37801504d51855a740433e3dc391a8` | `8384828a251d2ac87a8502a789b44c414e42c8a9169d83afc5da88290ce9ae31` |
| [Python generator pilot 03](LAUNCHER_PYTHON_GENERATORS.md) | `f3b8324feab914698fd3060bc0e6b8333e9831703a8f9720aa55625a20b509fe` | `0be694dc28387ad770429598e761d536baefb64ca7adcd0b4e12f4c23c9bd48c` |

All three launcher reports remain unchanged by this checkpoint. The separately linked configuration summary is 4,257 bytes, SHA-256 `0b63d55cef0b2e07ff61eb237991ede0b12adb995951d106e70876c91bc75234`, with local document manifest `e86eb1c66bf857b9cd44ca39b46f4124076e5fca9b79f4dbce6246e1aea1b588`. It is an authored summary admitted after the census freeze, not a later native result. Raw binaries, reconstructed code and detailed evidence remain local. This static update changes no original artifact, game data or settings and starts no game/SQL service. The continuous goal remains active; functional reconstruction, missing censuses and runtime validation are unfinished.
