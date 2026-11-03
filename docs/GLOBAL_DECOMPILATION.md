# Global decompilation: scope, checkpoint and continuation

The continuous decompilation goal remains active. The instruction is to work through the available client functions without requiring repeated requests to continue. This document freezes a verified publication point from **2026-11-03**; a later live phase has already continued beyond it. Completing this inventory will still require analysis, reconstruction and gameplay validation before it establishes functional compatibility.

## Cross-module follow-up

[Checkpoint 15](GLOBAL_DECOMPILATION_MODULES.md) and the [Native report](SFRAME_NATIVE_DOSSIERS.md) qualify **2,688 distinct SFrame dossiers covering 900,691 deduplicated original body bytes**, within 9,589,018 initialized executable bytes. The normally closed session has 233 case views: 232 new Native acquisitions and one inherited source-only completion. The raw legacy comparison qualifies 178 views: 177 new bodies and the inherited 4,774-byte body at 0x47a1f5 under its unchanged Native identity. A separately checked notation model qualifies 54 already acquired IDs; their original false projections remain unchanged. Eight portable checkpoints authenticate 929 retained source references; the sealed 91,775,824-byte final session export contains 928 local records and excludes the inherited primary. All 233 views retain two warning occurrences and four truncated value-flow graphs, all within the 232 finished views; the deferred 0x492fee view has neither. Bulk availability stays 127,824 texts with six original failures. Original files, protected settings and the logical SQL backup match their hashes; live SQL contents were not checked. The next slot is excluded. MAIN+0x458, faithful game reconstruction and uncensused scopes remain unfinished. Distinct [software and original-game version labels](VERSIONS.md) remain preserved. The local immutable checkpoint SHA-256 is `25d93a98628dd470559f16811375bdc14d83f3f230387a106ec5b5493396955e`; root freshly rehashed 30,277 pinned files.

## Target and versions

The current bulk queue covers the supplied `Rappelz NoLimits/SFrame.exe`, PE32 x86, **12,481,536 bytes**, SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`. It uses the current provider's procedure census, which may omit unidentified code or imperfect function boundaries. It does not yet measure every module in the package.

| Component or observed label | Version / scope |
|---|---|
| RappelzCore and RappelzRust | **0.1.0**, development |
| NoLimits launcher/package | **1.3.60** |
| Original SFrame file/product declarations | **9.1.0.0 / 9.1.0.0** |
| Embedded build lineage | **Source9.5.2** |
| Original protocol literal / implemented profile | **201507080** |
| Official game Epic and exact patch | **Unconfirmed** |

These labels have different origins and must remain separate; see [versions and original game target](VERSIONS.md).

## Frozen checkpoint 2022

The publication sources are local files in the ignored directory `client-data/audit/2026-11-02-global-decompilation-ec98a7d4/`: `publication-checkpoint-2022.local.json`, `checkpoint-2022-verification.local.json`, `coverage-baseline.local.json`, `verification-2022.local.json`, `artifact-inventory.local.json`, `tool-tests-54.local.json` and `resume-verification.local.json`. Mutable live summaries must not replace the following frozen counts.

| Measurement | At checkpoint 2022 |
|---|---:|
| Provider-defined procedures | 56,733 |
| Internal functions | 56,151 |
| External procedures, excluded from internal attempts | 582 |
| Internal thunks, retained as eligible | 169 |
| Existing detailed Native dossiers, all matched to inventory | 594 |
| Additional functions with successful pseudocode | 55,557 |
| Combined functions with pseudocode text | **56,151 / 56,151 = 100%** |
| Remaining internal functions awaiting an attempt | 0 |
| Completed bulk batches | 2,022 |
| Decompiler errors / batch item errors / empty text / unresolved identity | 0 / 0 / 0 / 0 |

The **100%** measures text availability within this SFrame census. It does not measure reviewed behavior, gameplay implementation, semantic correctness, original source recovery, total executable bytes, all package modules or remaining effort. Decompiled text contains generated names and inferred types; it is not the original source.

At the frozen checkpoint2022, detailed Native coverage was **594 functions**, covering **324,379 /9,589,018 initialized executable bytes =3.3828%**. The separate checkpoint 15 continuation below qualifies 2,688 dossiers covering a 900,691-byte union of original bodies. Bulk pseudocode does not increase that byte metric. The denominator includes compiler/runtime code and says nothing directly about gameplay completeness.

The checkpoint export `checkpoints/global-00002022.bundle.json` contains **2,046 Evidence records**, occupies **97,778,601 bytes**, and has SHA-256:

```text
5a4279cbc0dffa68e9a820c894150adad6864126a3c0aecad0d02f1567ab0afe
```

The official **REA 6.4.0 `parseEvidenceBundle`** validation passed: 2,022 batches, 55,557 distinct requested selectors, 55,557 successes and zero errors. The older **737-record Native archive** remains separate and unchanged in the previous `2026-11-02-sframe-vector-owner-3e87a1c9` audit. The newer export does not replace that corpus or recreate its detailed dossiers.

An additional one-pass archive audit and targeted historical-file audit now confirm every retained text: **563 exact index IDs in the 737-record main archive plus 31 separately retained historical Native files**, giving 594 nonempty pseudocodes with matching target, address, profile, body and raw/normalized text. The index is not wholly contained in that archive. Six texts contain unrecovered-jumptable warnings and 24 value-flow graphs are truncated; these limitations remain separate from text availability and body-range accounting. The combined local proof `retained-native-text-combined-03.local.json` has SHA-256 `d6351968c26d90300660146f12545440093d4bd974443e178df61ac3f4613794`.

The previous published checkpoint 514 (8,224 new outputs) remains an immutable local baseline. Checkpoint 2022 adds 47,333 further outputs. Every internal entry in this provider census now has pseudocode text, either in the retained Native corpus or the bulk outputs. This closes the census attempt queue, not semantic review or the global decompilation goal: unidentified code, imperfect boundaries and other package modules remain outside this result.

## Package inventory and remaining scope

Static inspection found **41 PE files**, with zero static audit failures. Original files were unchanged and no target executable was run.

| Inventory dimension | Count |
|---|---|
| File kind | 9 EXE, 32 DLL |
| Architecture | 39 x86, 2 x86_64 |
| Analysis route | 38 native, 3 with managed headers |
| Provisional roles | 1 main client, 32 support libraries, 4 companion executable candidates, 4 maintenance executable candidates |

Role labels are inventory classifications, not proven behavior. Other modules need their own target identity, procedure inventory, appropriate native/managed analysis and coverage accounting. Their totals cannot be inferred from SFrame's percentage. The immutable `module-continuation.local-02.json` plan records the remaining 40 modules: 36 ordinary native files, one native bootloader with packaged Python content, and three managed assemblies. Its normal-import graph has 53 local edges and no cycles; 21 local dependencies are reachable from SFrame, including 11 direct imports. This is an analysis priority, not a proven runtime load order. The only observed local normal-import edge into `lua5.1.dll` comes from `uninstall.exe`; SFrame dynamic loading remains unknown.

A bounded static parse of `NoLimits_Launcher_3.exe` now identifies the **18,331,703-byte unmapped tail** as a MEI/CArchive-compatible package: its footer commits the same package extent, a Python 312 field and `python312.dll`; all **1,012 TOC entries** fit the package bounds and a PYZ header is present. No contents were decompressed, extracted or executed in that check. The 49 DLL-suffixed and 19 PYD-suffixed entry names are nested-module candidates, not confirmed PE inventories. The bootloader's native census alone will not cover the packaged Python code; its function census and runtime behavior remain unknown.

The plan has SHA-256 `3e46747c5efdfb624ebaa6ed6dee20131c136c8080ac111b911f285b416c278c` and explicitly supersedes an earlier local draft whose prose incorrectly suggested an import cycle.

## First independent module completed

The portable worker was then used on `Rappelz NoLimits/RappelzCmdLauncher.exe`, PE32 x86, **34,304 bytes**, SHA-256 `6230684e0b1eb096d39564f742ecf0d36f5ec8b2986917ceaab8db74cf75f1f6`. It was chosen as a small fresh-target pipeline validation, not as a proven gameplay priority. Its independent census contains **207 procedures: 146 internal, 61 external and one internal thunk**. With `retained_native: "none"`, all 146 internal entries produced pseudocode in three batches, with zero recorded errors, empty outputs or unresolved identities. No SFrame Native entry was transferred.

The five-record export is **234,519 bytes**, SHA-256 `8ee19d106325df68e56cf086763f8f8408c25d3bc570f542526fd0804e005a8c`. The official REA parser authenticated all three batches and 146 distinct selectors. The portable worker exported, saved its snapshot, closed successfully and confirmed the original target unchanged. Local proofs are in `client-data/audit/2026-11-02-module-rappelzcmdlauncher-89c21a84/`, particularly `publication-checkpoint.local.json`, `checkpoint-3-verification.local.json` and `verification.local.json`. This is real provider validation in addition to the 54 engine-free tool tests. Other module analysis continues under the active goal.

The next completed attempt census is `audiere.dll`, SHA-256 `90e59c4f60941a2838d46e20fb78ba0cf71e9cfd0782e291ca876645e3794a66`: **1,669 procedures, 1,576 internal and 93 external**. It produced **1,575 pseudocodes and one explicit decompiler error** (`0x100519b2`, Ghidra “Overlapping input varnodes”). The error remains an error, not a successful function. Its 25-batch /27-record export passes the official parser with 1,576 distinct selectors, 1,575 successes and one error; SHA-256 `60ee4e5fd9b3d0a3f028d035f4866efb59f76632008e6a1d30556443314fb71d`. Local source: `client-data/audit/2026-11-02-module-audiere-e7a1b240/`. A separate focused pass requests function metadata, assembly and references to diagnose the failure without repeating the unchanged pseudocode operation. No resolved behavior is claimed yet.

## Focused MAIN frontier remains unresolved

A separately frozen review of the first **1,796 batches / 41,120 new pseudocodes**, with no overlap with the 594 retained Native entries, establishes no new explicit call or qualified receiver for `0x918630`. Three functions (`0x73eef0`, `0x7516e0`, `0x8e7e10`) contain byte writes at object offset `+0x458`; none of their objects is proven to be MAIN. Offset matches and wider writes do not establish activation. The mode trigger remains unknown and requires qualified receiver/provenance evidence before implementation. This search result does not cover the later 14,437 bulk outputs.

The ignored local reports `new-global-main-frontier.local.json` and `new-global-main-review.local.json` have SHA-256 `cb4aa7bdeca38785a86914efaeffabb6f24f2ef3dc8414f8e5dc3ecc41c8e14d` and `fed74bcc25e8ddd8ce7ab6baa2744619d27180882829f0c3fc62ca519f745abf`, respectively.

A later disjoint review of batches **1797..2022 /14,437 texts** completes the bounded text search across all **55,557 new outputs** without rereading the first cohort. It establishes no additional recognized explicit calls to `0x918630` or `0x8fb010`, and no additional byte-write candidate at `+0x458`. This does not resolve indirect calls. Reports `new-global-main-tail-frontier.local.json` and `new-global-main-tail-review.local.json` are pinned by SHA-256 `c3bdc5b179c826aa6a72685ab865631fc2b42b61081d461a6de277a638497902` and `7c398f3bcce0cb89a5cddd82a2a453a62ce294b835089a01bc26d542358aa791`.

Original RTTI, complete-object locators, vtable slots/hierarchies and the six original byte stores now qualify the three earlier candidates on their original dispatch paths:

| Function | Qualified receiver |
|---|---|
| `0x73eef0` | `SUIAuctionTenderWnd` |
| `0x7516e0` | `SUIChattingOptionWnd` |
| `0x8e7e10` | `SUIRaidSiegeRotationSiegeWnd` |

None of these original RTTI hierarchies contains MAIN's type descriptor. They therefore do not establish MAIN activation; arbitrary or unresolved indirect delivery remains outside this conclusion. The local `new-global-three-receivers.local.json` report has SHA-256 `27223bed9423d6ad463d8a650c6c9e3f07639e1c7838ac653a8fc8a5a67a1b73`. Its bounded byte/instruction work creates no new complete Native dossier and does not change the retained body-coverage metric.

## Guarded Python application reconstruction

The [dedicated comparison](LAUNCHER_PYTHON_RECONSTRUCTION.md) retains both attempts on the same **90 application code objects /20,397 wordcode entries**, separately from the other 11,012 packaged code objects. Pass 01 emitted 42 syntax-only successes, 41 partial results, six failures and one timeout. Its timed-out child lacked a final guard receipt; the earlier aggregate therefore did not prove all 90 guard checks.

Pass 02 changes the adapter and classification profile after 12 authored static fixture checks. It retains iterator stack state, requires an unambiguous context-body/handler link, and bounds dispatch/range progress. It records **22 syntax-only successes, 61 partial results, seven failures and zero timeouts**; all 90 child receipts pass the same six execution-guard self-tests, with no unexpected guard event. The 21 earlier success labels now qualified as partial include missing definition context; the lower label count is not a tested functional regression. Both passes have 82 AST-valid fragments, which are not 82 complete decompilations.

`_fetch_news` now completes its bounded attempt after 106 dispatches and remains partial because outer exception handling is unresolved. `_resolve_chain` explicitly stops a nonprogress transfer. Generator suspension/resumption, exceptions, closures and parent-supplied defaults remain open. No target or reconstructed source was executed or compiled. Pass-02 report and verification SHA-256 are `366f2156c50b826bddbf360a733735230e5902d128f1e69ab0d5fa9bcf693988` and `0ff69faf57753e42a0f4899d1576e26a39b54f8da08c25d675ad05d6de5b171c`; the dedicated document preserves the immutable first-pass and obligation revisions.

## Targeted Python generator follow-up

[Pass 03](LAUNCHER_PYTHON_GENERATORS.md) attempts only selectors **3,25,28,2,27,29,30** under a changed, reviewed model for discarded resume values. Each ran once. All seven now produce **AST-valid partial fragments**, with no child failure or timeout, 14 authored-fixture checks and 42 passed repetitions of the same six execution guards. Eight nested-context occurrences represent six generator identities plus one ordinary caller; equal bytecode does not merge objects 25/28.

The other 83 records are inherited from pass 02. The labeled reference overlay is 22 syntax-only / 68 partial / 89 AST-valid fragments, including one inherited syntax-invalid fragment. It is not 90 new operations or a rebuilt application. Exceptional paths/PEP479, parent iterator timing, name/state cell bindings and 14 default-definition obligations remain unresolved. No target or reconstructed source was executed or compiled. Report SHA-256 is `4cf3d6dc890d915ee54790466a0edae7704a0cc079d2bb354eee98827df48418`; independent verification is `dadf011df362b9057244b9f72d4195e83cef837d5c6cbbfa85b27637b80b2738`.

## Supplemental launcher entry and return recovery

The [native launcher report](LAUNCHER_NATIVE_RECONSTRUCTION.md) establishes that `RappelzCmdLauncher.exe` starts **argv[1]**. `Launcher.exe` is an environment-marker value, not its hardcoded target. Four CRT table targets absent from the first inventory were created in a separate Ghidra profile: `0x4019e5`, `0x402562`, `0x402bef` and `0x405710`, covering **90 original bytes /34 instructions**. That profile moves from 146 to150 internal entries. These are four distinct additions overall, not four per revision, and do not silently change the frozen REA row or become new detailed Native dossiers.

V1/V2 recovered the four bodies but left main’s cookie-result semantics incorrect in the printed text. V3 inlining restored only the first normal result and failed at two other sites. V4 leaves inlining disabled and models the helper’s custom incoming `ECX:4` cookie, preserved `EAX:4` result and `EAX:4` return. Original instructions, helper storage and caller text jointly establish **0xffffffff for insufficient arguments, zero for successful CreateProcessA, and the same 32-bit GetLastError result on creation failure**, only when the cookie is valid. Printed `__fastcall` is not an ordinary compilable ECX/EDX declaration. The invalid-cookie destination and actual child use of inherited markers remain unresolved.

The V4 run profile is `d9f5b8458075ad0a334e38144b5be120180720c07b90c3990c1055638c3b6b06`, post-report SHA-256 `1c353cbf459d338e98e41248b618533358cddc3c86f424d660041dd394837f38`, and normal-return contract SHA-256 `1551df488f76ec2c3dd0b831c274b17db0cac6b36a1475a0e9cb13a23123f648`. The analysis process actually exited zero; the launcher itself was not executed. Its 589 guarded original bytes remain unchanged and its function count stays207 in this profile.

**Rollback correction:** earlier wording that analytical transactions “were rolled back” was too strong. `endTransaction(false)` proves the API request, while immediate V3/V4 probes still observe modified metadata. The original/staged hashes and absence of the isolated `.gpr/.rep` after read-only headless exit establish byte preservation and final project discard separately; they do not prove in-script metadata restoration.

## Automatic serial continuation

The local serial coordinator has 32 provider-free fixture checks. Each reviewed plan pins the original module bytes, helpers, runtime and resource options; it processes at most four modules sequentially, authenticates a finished export before advancing, skips previously verified hashes, and stops on ambiguous ownership, incomplete phases or observed exports at or above128MiB. This is an observed-size guard, not a hard cap during an in-flight write. It does not kill a worker or replay a producer after an observation timeout. Remaining original modules, packaged bootloader/tails, 68 embedded PE targets and reconstruction obligations remain on the active goal.

## New bounded configuration and entry frontiers

The [RappelzConfig report](RAPPELZ_CONFIG_RECONSTRUCTION.md) authenticates ten retained projections and five original-byte windows. Signed nonpositive width/height independently fall back to 1024/768; the integer-reader API uses default −1, and the write helper reaches WritePrivateProfileStringA. Full save/reset behavior, object ABI and renderer consumption remain open. Its corrected 77 textual key/offset pairs include two decimal offsets missed by the first hex-only extraction. No game setting or spelling was modified, and no Native dossier or complete-function metric was added.

The frozen CRT-table frontier identifies 581 entries absent from the initial SFrame census: ten C and 571 C++ targets. A later isolated ten-C V2 run, outside checkpoint 05, independently verifies ten new local entries and texts: **1,488 original bytes / 372 instructions**, with its local function count moving from 56,733 to 56,743. The source-pinned producer checked owners and disjoint bounded bodies; independent review checked original entry barriers, byte identity and pairwise bounds without reacquiring the full old body map in another engine. It does not alter the frozen 56,151-internal row or become REA Native Evidence. The 571 C++ candidates, indirect calls and MAIN+0x458 trigger remain unfinished.

The ten-C run retains **three embedded decompiler warnings** and unforced return metadata. Printed void at be4ea1 and a byte result at cd168b are insufficient descriptions of the normal EAX-consuming contract; these need a separate analytical repair. The analysis actually exited zero, originals/copy remained identical and the read-only project was absent afterward. Immediate metadata restoration after the rollback request was false. The requested private user.home was overridden by launch.bat; private project/input paths were observed, while full tool-preference isolation was not claimed. Future path-option correction is prepared without replaying these ten producers.

The distinct run profile is `e47d3ce94c57c15dec80d821dec0c2ab41ad4e73fa5e20a2c9b993220f9ace22`; its post-report SHA-256 is `4ef83112535eda935c716c426199a93c94fd4ca747530f56f97172762577e996`. The scoped independent receipt `sframe-crt-c-pilot-v2-fresh-01/verification.local.json` has SHA-256 `8417f5754acaaad60737ffc7d9a4f6e87f0cf605ae64e1679f10314adfd56fd9`. Success there proves the bounded source/process/byte/body checks, not complete semantics, game runtime or the global objective. These ten additions are counted once across profiles and add no detailed Native dossier.

Original `unins000.exe` has a qualified Inno-message appendix. A reviewed UPX decompression now produces a separately hashed maintenance PE; its native census is still pending. Packed-source/tool/derived lineage and the legacy non-exact PE reconstruction limit remain explicit.

## Recovery after the host restart

The confirmed restart at 2026-11-02 22:06:14 UTC ended the first xdelta inventory request before any inventory response, batch, checkpoint or retained export existed. Its original audit and request were preserved. After checking that the recorded processes were dead and ownership locks free, one new isolated audit recovered **836 internal entries**, with no item error. The original interrupted attempt is never marked complete or counted again.

After the reboot, **41 original PE hashes, 11 protected files, the installed WTF tree and the logical SQL backup** match the recorded baseline. The three local services have no listening ports and were not restarted for static analysis. These observations do not verify live SQL after the system crash. All 6,497 frozen checkpoint pins were independently rehashed. One recent preparation notice contains NUL bytes; its original is preserved, the notice is reconstructed separately, and the pinned scripts, plans and profiles passed integrity checks.

One reviewed UPX tool invocation decompressed an isolated copy to **3,805,696 bytes**, SHA-256 `5ef772bfd3c163011967a6c5f8eacb53df2f653337594b0529e548f2296e76d1`. The official tool's two owned processes ended zero under checked memory/CPU limits. The packed original and input copy remained unchanged. The derived PE has five bounded sections and no overlay; legacy PE13 reconstruction does not prove the old unpacked file byte for byte. No function census, independent uncompressed-stream checksum, semantic equivalence or target execution is claimed from this operation. Its later census must bind the packed-source/tool/derived lineage explicitly.

The fifteen-C++/two-known-C-return analysis uses a separate profile. Its first combined wrapper refused a valid Windows path before engine creation; that failure is retained. A corrected path guard is tested against the actual Windows PowerShell runtime before a changed attempt. The two known C entries add zero global discoveries. Results of later experiments remain outside checkpoint 06 until their own body, byte, process and metadata checks pass.

## Verified continuation checkpoint 15

The normally closed session has **233 acquired case views: 232 new Native acquisitions and one inherited source-only completion**. The inherited 0x47a1f5 primary, response, marker and Native ID remain unchanged. Source reads complete its original 4,774-byte body without another Native acquisition. The raw legacy comparison qualifies this inherited body and 177 newly acquired bodies, giving 178 qualified views.

The separately reviewed known19 notation model qualifies 54 already acquired identities across 258 exact sites and 41,368 unchanged body bytes. This gives **2,688 distinct qualified dossiers covering 900,691 deduplicated original executable body bytes** within 9,589,018 initialized executable bytes. The 54 original finished false projections, indices, source provenance and limits remain unchanged. Static qualification creates no Native producer, function or Evidence ID. Reported body qualification does not establish complete original functions or game semantics.

All 232 finished case views retain 97,769 authenticated body bytes and 34,598 instruction starts. Source history and eight portable checkpoints contain 929 references including the inherited primary. The sealed final 91,775,824-byte session export contains 928 local records. Source counts follow exact labels and body chunks. Across all 233 views there are two warning occurrences and four truncated value-flow graphs, all within the 232 finished views; the one pending view has no warning or truncated graph.

The acquired 0x492fee result remains source-deferred in this freeze, with a 1,505-byte body at ordinal 2095. The next reviewed plan prioritizes its existing Native identity and source obligations, then at most 1,023 uncaptured entries at ordinals 2096 through 3118, from 0x4935cf through 0x4ead57. Its frozen conservative baseline is 2,284 dossiers covering 625,604 bytes; its corrected baseline is 2,688 dossiers covering 900,691 bytes, including all 404 prior static qualifications. All results, launch observations, FIRST proof and mutable journals from that next slot are excluded from checkpoint 15.

The 45ef session ended normally: its coordinator actually exited zero, and an independent gate authenticated the closed records, original bytes, process closure and portable exports before admission. The two 0408 coordinator failures and failed receipts remain historical evidence in the preceding frozen checkpoint.

The bounded follow-up examined only the 232 newly acquired primaries for explicit MAIN+0x458/0x918630 text and typed outgoing calls: zero text matches and zero typed calls. The receiver, indirect callers and activation remain unresolved. Checkpoint 14, all its 28,056 pinned sources and both authored reports remain byte-identical. The 41 original PEs, 11 protected files, WTF settings tree and logical SQL backup match their preserved hashes. Static analysis did not start game/SQL services or verify live SQL contents. The six bulk failures, frozen CRT/UPX studies and separate version labels remain unchanged. Semantic reconstruction and uncensused scopes remain open. Raw code and private Evidence stay local.

## Evidence and batch limits

REA's public `batch_decompile` accepts explicit address selectors. Its implementation starts `Promise.all` without an input cap; the owned Ghidra bridge serializes actual requests through one decompiler. The controller therefore sends bounded batches, defaulting to **16 addresses**, rather than submitting the census at once.

Each batch yields one derived `rea-workflow` Evidence record, with workflow revision 2 and the upstream analysis profile committed in its contract. An original selector is distinct from a resolved canonical procedure entry. The controller checks exact requested selectors and order, canonical identity against the inventory, target identity, profile binding and nonempty pseudocode before counting success. A different provider identifier on this derived workflow does not permit a different subject or analysis profile.

The aggregate projection loses the primitive per-function limitation arrays, including possible no-return boundary assumptions. Warnings may survive in the text, but successful text alone does not prove a complete body or control-flow graph. Difficult functions still need direct primitive inspection and detailed Native evidence. External/bodyless procedures, null text, item errors and unknown identities have separate states; they must never be silently counted as decompiled.

Oversized completed transport responses may retain an Evidence reference. The worker recovers that observation through the public Evidence tools instead of repeating the producer. The controller allows a bounded recovery window of 300 seconds for that specific transport condition. Other failed operations stop with an explicit error rather than looping analysis retries.

## Persistent tools and recovery

Two authored, standard-library Python tools support continuation:

| Tool | Responsibility |
|---|---|
| [Queue controller](../tools/audit/rea_global_queue.py) | Inventory validation, canonical function states, bounded jobs, ingestion, checkpoints and progress summaries |
| [Public MCP worker](../tools/audit/rea_mcp_worker.py) | One serialized public REA stdio session, immutable responses, completion markers, exports and lifecycle cleanup |

The controller never starts a provider. Its SQLite queue uses WAL and FULL synchronization, with an operating-system file lock preventing a second writer. A job is committed before its unique label is appended to the plan. Resumption preserves outstanding jobs and ingests ready responses without creating duplicate jobs or counts. Immutable evidence files are not overwritten. Each completion marker binds the operation digest, target SHA-256, response filename, response SHA-256 and Evidence ID when present.

The Native index binding pins the target, analysis profile, index hash, source archive and proof hashes. Before queue initialization, a bounded structural reader decodes the canonical archive's first `artifacts` manifest within a 1 MiB prefix and requires its sole artifact to match the bound target. This prevents moving the 594 SFrame entries to another module merely by editing the binding's target hash. The proof must pass and match the index function count. New modules require independent bindings.

For a module without retained Native dossiers, the binding now explicitly declares `retained_native: "none"` and pins an index containing exactly `functions: []`, the module SHA-256 and its observed analysis-profile digest. This mode requires no invented archive or Native proof and rejects any `source_archive` or `proof` field. A nonempty index, unknown mode or change to an established queue binding is rejected. Omitting the mode preserves the existing archive/proof contract; `retained_native: "archive"` is its explicit form. Public inventory, subject, profile and completion-marker validation remains mandatory in both modes. The nine additional tests cover this path and its integrity failures.

The portable worker uses a unique epoch for initialization, recovery, final export, snapshot and error artifacts. Existing job labels retain their original pins across epochs. These tools launch the selected analysis runtime, never the target, and make no changes to game databases or character data. Configuration checks reject Node or launcher paths that identify the target itself, a hardlink to it or an identical-byte copy before a client can be constructed.

A real Windows `WinError 5` interrupted summary-file replacement after 177 batches. Mutable metadata replacement now retries sharing failures for at most **80 attempts / two seconds**, with 25 ms pauses. This retries file publication only. Ready job **178** was subsequently ingested without resubmission and its evidence pins remained unchanged. The portable worker retries its mutable-status replacement at most 20 times with 25 ms pauses; immutable outputs and producer operations are never retried.

**54 tests** passed across the controller and portable worker, including identity failures, mixed results, archive binding, resumability, immutable outputs and transient/permanent sharing failures. Tests started no provider and executed no target. The live controller and prototype public-MCP worker, using initial 16-address batches followed by 64-address batches, produced the 55,557 successes above. The SFrame prototype subsequently exported and closed normally; the portable worker then completed the independent 146-function module pass above.

## Continuation runbook

Keep a single worker and controller per audit directory. Resume the existing controller against its unchanged Native index and binding; `AUDIT`, `INDEX` and `BINDING` below stand for their existing local paths:

```powershell
python tools/audit/rea_global_queue.py --audit-dir AUDIT --native-index INDEX --native-index-binding BINDING --batch-size 64 --checkpoint-every 64 --max-batches 128
```

`--max-batches` means additional completed batches during that invocation. A phase exports a checkpoint and exits without sealing the worker, so the same command continues the queue. Successful functions are not reprocessed. Inspect explicit errors before deciding further work. Use `--seal-when-complete` only when all remaining internal functions have been attempted; error states remain errors.

For a future controlled restart, first confirm there is **no active worker**, preserve verified exports, and use the portable worker's six required paths:

```powershell
python tools/audit/rea_mcp_worker.py --audit-dir AUDIT --target ORIGINAL_SFRAME --node NODE_EXE --rea-launcher PUBLIC_REA_LAUNCHER_JS --ghidra-install GHIDRA_DIR --java-home JAVA_DIR
```

Supply `--snapshot SNAPSHOT_JSON` only when deliberately reopening a compatible saved session after the previous worker has stopped. Do not start this command alongside the warm worker. Validate the new session's identity/profile and exports before continuing the existing queue. The default disk floor is 5 GiB; request and idle timeouts default to 1,800 seconds.

Cumulative export growth is monitored. The planned **128 MiB** shard threshold requires coordinator-controlled export rotation at a phase boundary; the controller does not automatically rotate shards or restart an engine. Detailed Native archives remain separately pinned.

Original binaries, raw pseudocode, Evidence bundles and private audit data stay local. Only authored tools and documentation are intended for publication. Remaining work includes reviewing the complete SFrame text corpus for uncertain bodies and unresolved behavior, continuing the other independently bound native/managed modules, analyzing the identified launcher package and nested candidates, then converting verified behavior into tested core/client implementations. The continuous goal is still active.
