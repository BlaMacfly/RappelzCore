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

## Frozen checkpoint 514

The publication sources are local files in the ignored directory `client-data/audit/2026-11-02-global-decompilation-ec98a7d4/`: `publication-checkpoint.local.json`, `checkpoint-514-verification.local.json`, `coverage-baseline.local.json`, `artifact-inventory.local.json`, `tool-tests.local.json` and `resume-verification.local.json`. Mutable live summaries must not replace the following frozen counts.

| Measurement | At checkpoint 514 |
|---|---:|
| Provider-defined procedures | 56,733 |
| Internal functions | 56,151 |
| External procedures, excluded from internal attempts | 582 |
| Internal thunks, retained as eligible | 169 |
| Existing detailed Native dossiers, all matched to inventory | 594 |
| Additional functions with successful pseudocode | 8,224 |
| Combined functions with pseudocode text | **8,818 / 56,151 = 15.70408%** |
| Remaining internal functions awaiting an attempt | 47,333 |
| Completed bulk batches | 514 |
| Decompiler errors / batch item errors / empty text / unresolved identity | 0 / 0 / 0 / 0 |

The **15.70408%** measures text availability within this SFrame census. It does not measure reviewed behavior, gameplay implementation, semantic correctness, original source recovery, total executable bytes, all package modules or remaining effort. Decompiled text contains generated names and inferred types; it is not the original source.

Detailed Native coverage remains **594 functions**, covering **324,379 / 9,589,018 initialized executable bytes = 3.3828%**. Bulk pseudocode does not increase that byte metric. The denominator includes compiler/runtime code and says nothing directly about gameplay completeness.

The checkpoint export `checkpoints/global-00000514.bundle.json` contains **538 Evidence records**, occupies **40,498,583 bytes**, and has SHA-256:

```text
4ebdc09fb9195fa07d40727654904f7d6497180d778e9dbdfcf51c11c3095fe8
```

The official **REA 6.4.0 `parseEvidenceBundle`** validation passed: 514 batches, 8,224 distinct requested selectors, 8,224 successes and zero errors. The older **737-record Native archive** remains separate and unchanged in the previous `2026-11-02-sframe-vector-owner-3e87a1c9` audit. The newer export does not replace that corpus or recreate its detailed dossiers.

## Package inventory and remaining scope

Static inspection found **41 PE files**, with zero static audit failures. Original files were unchanged and no target executable was run.

| Inventory dimension | Count |
|---|---|
| File kind | 9 EXE, 32 DLL |
| Architecture | 39 x86, 2 x86_64 |
| Analysis route | 38 native, 3 with managed headers |
| Provisional roles | 1 main client, 32 support libraries, 4 companion executable candidates, 4 maintenance executable candidates |

Role labels are inventory classifications, not proven behavior. Other modules need their own target identity, procedure inventory, appropriate native/managed analysis and coverage accounting. Their totals cannot be inferred from SFrame's percentage. `NoLimits_Launcher_3.exe` has **18,331,703 bytes of unmapped file tail** whose format remains unresolved; this observation does not establish extracted contents or behavior.

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

The portable worker uses a unique epoch for initialization, recovery, final export, snapshot and error artifacts. Existing job labels retain their original pins across epochs. These tools launch the selected analysis runtime, never the target, and make no SQL or character-data changes. Configuration checks reject Node or launcher paths that identify the target itself, a hardlink to it or an identical-byte copy before a client can be constructed.

A real Windows `WinError 5` interrupted summary-file replacement after 177 batches. Mutable metadata replacement now retries sharing failures for at most **80 attempts / two seconds**, with 25 ms pauses. This retries file publication only. Ready job **178** was subsequently ingested without resubmission and its evidence pins remained unchanged. The portable worker retries its mutable-status replacement at most20 times with25 ms pauses; immutable outputs and producer operations are never retried.

**45 tests** passed across the controller and portable worker, including identity failures, mixed results, archive binding, resumability, immutable outputs and transient/permanent sharing failures. Tests started no provider and executed no target. The live controller and prototype public-MCP worker produced the 8,224 successes above. The portable worker has been tested without an engine but has **not yet replaced the intentionally warm live prototype session**.

## Continuation runbook

Keep a single worker and controller per audit directory. Resume the existing controller against its unchanged Native index and binding; `AUDIT`, `INDEX` and `BINDING` below stand for their existing local paths:

```powershell
python tools/audit/rea_global_queue.py --audit-dir AUDIT --native-index INDEX --native-index-binding BINDING --batch-size 16 --checkpoint-every 64 --max-batches 512
```

`--max-batches` means additional completed batches during that invocation. A phase exports a checkpoint and exits without sealing the worker, so the same command continues the queue. Successful functions are not reprocessed. Inspect explicit errors before deciding further work. Use `--seal-when-complete` only when all remaining internal functions have been attempted; error states remain errors.

For a future controlled restart, first confirm there is **no active worker**, preserve verified exports, and use the portable worker's six required paths:

```powershell
python tools/audit/rea_mcp_worker.py --audit-dir AUDIT --target ORIGINAL_SFRAME --node NODE_EXE --rea-launcher PUBLIC_REA_LAUNCHER_JS --ghidra-install GHIDRA_DIR --java-home JAVA_DIR
```

Supply `--snapshot SNAPSHOT_JSON` only when deliberately reopening a compatible saved session after the previous worker has stopped. Do not start this command alongside the warm worker. Validate the new session's identity/profile and exports before continuing the existing queue. The default disk floor is 5 GiB; request and idle timeouts default to 1,800 seconds.

Cumulative export growth is monitored. The planned **128 MiB** shard threshold requires coordinator-controlled export rotation at a phase boundary; the controller does not automatically rotate shards or restart an engine. Detailed Native archives remain separately pinned.

Original binaries, raw pseudocode, Evidence bundles and private audit data stay local. Only authored tools and documentation are intended for publication. Remaining work includes completing SFrame's attempted-function census, resolving explicit failures, analyzing the other inventoried modules and unresolved launcher tail, then converting verified behavior into tested core/client implementations. The continuous goal is still active.
