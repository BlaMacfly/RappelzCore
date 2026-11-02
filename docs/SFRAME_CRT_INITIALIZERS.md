# SFrame CRT initializers: verified additions and remaining scope

The continuous decompilation goal remains active. This report concerns callbacks in SFrame's native startup tables. These are compiler/runtime entrypoints; their count is not a gameplay completion measure. The original SFrame census and the later local experiments have separate profiles and evidence.

The supplied NoLimits package label is **1.3.60**. SFrame declares file/product version **9.1.0.0**, contains **Source9.5.2** lineage and protocol literal **201507080**. The exact official Epic/patch remains unconfirmed. The audited original has SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`. See [version labels](VERSIONS.md) and the [module checkpoint](GLOBAL_DECOMPILATION_MODULES.md).

## What has been verified

The initial public-provider census contains **56,151 internal entries and 582 external entries**. The retained table study then found **581 exact-entry candidates absent from that initial census**: ten C callbacks and 571 C++ callbacks. An absent entry was a candidate, rather than proof that a separate function could safely be created.

Two subsequent experiments established disjoint local bodies after checking the original bytes, table pointers, exact/containing owners, instruction coverage and bounded control flow:

| Experiment | Newly observed entries | Successful texts | Original body bytes | Instructions |
|---|---:|---:|---:|---:|
| Ten C callbacks, V2 | 10 | 10 | 1,488 | 372 |
| Fifteen C++ callbacks, combined V2 | 15 | 15 | 357 | 106 |
| **Distinct supplemental union** | **25** | **25** | **1,845** | **478** |

The fifteen C++ bodies were independently checked against the ten earlier C bodies: their entry sets and body ranges do not overlap. These 25 identities are counted once. They do not alter the frozen SFrame provider-census row, become REA Native dossiers or enlarge its historical detailed-body metric.

The ten-C profile observed a local function count of **56,733 → 56,743**. The later C++ phase observed **56,733 → 56,748**. These are observations in separate experiment phases, not a persistent combined database count. The formal detailed Native corpus remains **594 dossiers**, with its separately verified body union of **324,379 bytes**.

The ten-C output retains **three embedded warnings**. The C++ output retains one global-symbol-overlap warning at `0x5b7a9a`. Nonempty text and a verified bounded body do not establish full source recovery, complete exception behavior or semantic equivalence of the callees. Callback return types and conventions were not forced in those discovery experiments.

## Two known C returns: metadata and rendered text differ

The combined run also revisited **two already counted C identities**, `0xbe4ea1` and `0xcd168b`. Their original bodies total **182 bytes / 56 instructions**. The neighboring `INT3` used as guard context is outside the first 170-byte body; it is not a 183rd body byte. This return-model phase adds **zero new global entries**.

The table consumer at `0xcaf895` calls the callback and tests the whole EAX result. Independent original-byte checks establish these bounded normal-path relations:

| Known entry | Relation supported by original instructions | Remaining limit |
|---|---|---|
| `0xbe4ea1` | EAX is zero at the ordinary return reached through its `TEST EAX` / conditional-branch guard. | Failure/exception paths and inherited fastfail behavior remain unqualified. |
| `0xcd168b` | The result is `zeroExtend(AL) XOR 1`, in the range 0–255; it is zero exactly when the callee's AL is 1. | The callee is not assumed to return only Boolean values. |

The first analytical model changed each database return declaration from `undefined / <UNASSIGNED>` to **`undefined4 / EAX:4`**. Body bytes, instructions, parameters, convention, other ABI metadata and the inspected callee metadata remained unchanged. The two known bodies were projected into that fresh copy, with a phase-local count of **56,733 → 56,735**; these projections were not new discoveries.

However, **both C texts remained byte-identical before and after the metadata change**: `0xbe4ea1` still printed a void result and `0xcd168b` still printed a byte result. The first retained its nonreturn warning in both renderings. Those are repeated views of the same warning, rather than additional recovered functions or independent defects. The pseudocode rendering was not corrected by that experiment.

The installed Ghidra 12.1.4 primary sources explain the observed result:

1. `Undefined4DataType` inherits from `Undefined`; its decompiler metatype is `TYPE_UNKNOWN`.
2. `FunctionDB.updateSignatureSourceAfterVariableChange` skips undefined types. The two observed signatures consequently remained **`DEFAULT → DEFAULT`**.
3. `FunctionPrototype.grabFromFunction` leaves the output type unlocked when the signature source remains `DEFAULT`. Width/storage metadata alone therefore did not force the rendered return type.

This is a source-supported explanation for the observed profile, not a claim that every Undefined4 declaration is always unlocked or that a stale cache caused the result.

## Changed explicit-integer model: prepared, not executed

A separate, compiled analysis component proposes an **explicit four-byte unsigned integer** with **`SourceType.ANALYSIS`**. The source-priority change is performed only through the return-type API; it does not request USER_DEFINED priority or directly change signature source, parameters, convention, callees, custom storage or nonreturn flags. A preexisting signature with another priority is refused.

The proposed unsigned interpretation fits the verified normal EAX relations. It does **not** establish the original source signedness or recover a missing debug signature. The program's integer type must actually have length four and metatype TYPE_UINT; those properties are checked rather than assumed. The future report will separately observe database metadata, the return received by HighFunction and the rendered text.

The primary marshaller also locks an existing empty input list when signature priority becomes nondefault. This analytical consequence is documented; unchanged empty parameter metadata is not proof of the original source prototype.

Preparation checks passed: analysis-class compilation with annotation processors disabled, **99 original-byte/table/return assertions**, **ten evidence fixtures** and **19 copied primary source files** with lineage. The component launches no engine. Its future runtime and rendering outcomes remain unobserved, and its two identities must still add **zero** global discoveries.

## Remaining 556 C++ candidates

The fifteen verified additions leave **556 candidates** from the original 571-entry missing-C++ set. A later static study accounts for all of them, reusing the earlier excluded rank-six decode rather than repeating it:

| Static classification | Candidates | Meaning |
|---|---:|---|
| Simple bounded structures selected for future owner checks | 529 | Candidate instruction covers; no function creation has yet been observed. |
| Indirect-call dependencies | 25 | Import declarations alone do not establish complete runtime callback behavior. |
| Local cycle | 1 | Requires separate flow qualification. |
| Previously excluded rank six | 1 | Its jump to `0xbca73a` needs separate shared-body and owner checks. |

The later study corrects an earlier assumption: **`0xbca73a` is not an exact entry in the initial census**. A containing function is a different question. The static 529/27 split, read caps and proposed covers do not establish 556 new functions or executable-body coverage.

A full-remaining-set profile is prepared to treat 555 candidates, the separate rank-six case and the two known returns in independent phases within one owned import. Its compilation, 30 fixtures, 402 source pins and Windows PowerShell preflight have been checked; it has not produced a new function or text result in this checkpoint. Every candidate still needs actual owner/body checks; refused or existing entries must remain distinct from new successes.

The rank-six preparation decodes a ten-byte source fragment and a 47-byte destination fragment, totaling 57 bytes / 12 instructions in two ranges. Their owners remain unknown. These are static fragments, rather than an additional recovered function or an addition to body coverage.

## Closure, rollback and configuration limits

The original and analysis-copy hashes remained identical in the completed experiments. Their owned analysis processes ended, and the requested read-only private project files were absent afterward. The immediate metadata probes after rollback requests nevertheless reported **false** restoration. A requested rollback, its immediate observation and the final discarded project are separate facts.

The C++ phase ended with 56,748 functions, while the following known-return phase began with 56,733. That later count does not by itself prove restoration of the complete original entry set; the recorded immediate probes remain authoritative for their stated scope.

The earlier ten-C run's requested private `user.home` was overridden by the launcher. The corrected combined V2 run did observe the intended JVM `user.home` and temporary path, a **1,536 MiB heap** and **two processors**. Ghidra's settings and cache directories still resolved under AppData, so full tool-preference isolation is not claimed. Game settings and database contents are separate preservation scopes.

Independent review matched original bytes, complete retained instruction boundaries, pairwise body bounds and original census entry barriers. It also checked the reported exact/containing probes. It did not reacquire the entire old function-body map in a second engine; complete owner/data-intersection guards remain supported by the pinned producer source and observed outcomes. No game executable or reconstructed implementation was launched or built.

## Retained evidence

The private audit folder is `client-data/audit/2026-11-02-global-decompilation-ec98a7d4`. Raw pseudocode and original artifacts remain local. These are distinct profiles and authored verification records:

| Evidence | Digest or identity |
|---|---|
| Original public SFrame Native profile | `b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17` |
| Ten-C V2 session profile | `e47d3ce94c57c15dec80d821dec0c2ab41ad4e73fa5e20a2c9b993220f9ace22` |
| Combined V2 session profile | `0db147a0d8557e7a5c57415c569bc8078163bda9408c666e81499470f62f9347` |
| Independent combined-run verification | `41e6d25c86cacbd5a3e4d3bbfb367df1fa3c7e487df77010456e7ec980829726` |
| Independent union, rendering and closure supplement | `b90b3e439442f2fb477f16c04a4178b7249d5362af3bd90fbf036099676a2a28` |
| Primary-source explanation of the weak return model | `9d01842ff00de124bc397aaca00c74f7ef2b069bf74280c21f8b546753b8fc35` |
| Prepared explicit-integer return component, not a runtime result | `585b443f4d276e934be5e0b811b0fbe4cc4aae3ff20b34101e61be281699461d` |

The [global audit runbook](GLOBAL_DECOMPILATION.md) explains the continuous workflow. The 25 supplemental entries, prepared follow-ups and provider-census availability must remain separate from semantic reconstruction and validation in a real game session.
