# Global decompilation: scope, checkpoint and continuation

The continuous decompilation goal remains active. The instruction is to work through the available client functions without requiring repeated requests to continue. This document freezes a verified publication point from **2026-11-02**; a later live phase has already continued beyond it. Completing this inventory will still require analysis, reconstruction and gameplay validation before it establishes functional compatibility.

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

Detailed Native coverage remains **594 functions**, covering **324,379 / 9,589,018 initialized executable bytes = 3.3828%**. Bulk pseudocode does not increase that byte metric. The denominator includes compiler/runtime code and says nothing directly about gameplay completeness.

The checkpoint export `checkpoints/global-00002022.bundle.json` contains **2,046 Evidence records**, occupies **97,778,601 bytes**, and has SHA-256:

```text
5a4279cbc0dffa68e9a820c894150adad6864126a3c0aecad0d02f1567ab0afe
```

The official **REA 6.4.0 `parseEvidenceBundle`** validation passed: 2,022 batches, 55,557 distinct requested selectors, 55,557 successes and zero errors. The older **737-record Native archive** remains separate and unchanged in the previous `2026-11-02-sframe-vector-owner-3e87a1c9` audit. The newer export does not replace that corpus or recreate its detailed dossiers.

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
