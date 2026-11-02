# SFrame CRT initializers: verified local recovery

The continuous decompilation goal remains active. This report concerns native startup callbacks and analytical return models. These compiler/runtime entrypoints do not measure gameplay completion. Custom local Ghidra results remain separate from the original provider census and detailed REA Native dossiers.

The NoLimits package label is **1.3.60**. SFrame declares file/product **9.1.0.0**, contains **Source9.5.2** lineage and protocol **201507080**. The exact official Epic/patch is unconfirmed. Original SFrame SHA-256: `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`. See [versions](VERSIONS.md) and the [module checkpoint](GLOBAL_DECOMPILATION_MODULES.md).

## Completed finite startup frontier

The initial provider census has **56,151 internal / 582 external entries**. A subsequent table study identified **581 exact-entry candidates absent from that census**: ten C and 571 C++ callbacks. Entry-list absence alone was not proof of a safe separate function.

Completed experiments checked original bytes, table pointers/HIGHLOW relocations, exact and containing owners, inclusive instruction covers and bounded control flow:

| Experiment | New local entries | Successful texts | Original body bytes | Instructions |
|---|---:|---:|---:|---:|
| Ten C callbacks, V2 | 10 | 10 | 1,488 | 372 |
| Fifteen C++ callbacks, combined V2 | 15 | 15 | 357 | 106 |
| Remaining ordinary C++ phase | 555 | 555 | 12,706 | 3,279 |
| Separate rank-six case | 1 | 1 | 57 | 12 |
| **Distinct supplemental union** | **581** | **581** | **14,608** | **3,769** |

The entry sets and body ranges are pairwise disjoint. All 556 remaining candidates produced new functions and nonempty text in the completed experiment. Existing/refusal outcomes were checked rather than assumed absent. These identities are counted once; they add **zero formal REA Native dossiers** and **zero bytes to formal Native coverage**. The bulk-census row is unchanged. Formal Native progress is reported separately in the [module checkpoint](GLOBAL_DECOMPILATION_MODULES.md).

Independent verification checked 402 preparation pins and 575 runtime/source occurrences: **977 pin occurrences / 971 unique paths**. Original bytes, Capstone instruction boundaries, table bindings, owner probes, full body ranges, imports, durable results and the union with the prior 25 were checked. The verifier passed **35 focused fixtures**, including duplicate/truncated receipts, binding mismatches and range/flow errors. No producer or engine was replayed.

The 556 new texts contain **30 embedded warnings**: 29 overlapping-global-symbol warnings and one `__alloca_probe` injection warning at `0x52d6f0`. Earlier C10 warnings 3 and C++15 warning 1 remain separate historical scopes. Nonempty text and bounded bytes do not prove original source, exception behavior or callee equivalence.

## Indirect calls, cycle and rank six

The historical 529/27 static split was a preparation filter. The completed run included all 27 harder cases. Twenty-five callbacks contain 28 calls through six declared import slots; original declarations agree with existing read-only Ghidra external references:

| Slot | Declared API | Calls |
|---|---|---:|
| `0xd271e4` | KERNEL32.OpenMutexA | 1 |
| `0xd271e8` | KERNEL32.CreateMutexA | 1 |
| `0xd27400` | KERNEL32.InitializeCriticalSectionAndSpinCount | 3 |
| `0xd27408` | KERNEL32.InitializeCriticalSection | 20 |
| `0xd27424` | KERNEL32.GetLastError | 2 |
| `0xd276f8` | WINMM.timeGetTime | 1 |

Runtime implementation addresses, hooks and API effects remain unknown. Normal call fallthrough is conditional on the call returning; API return or exception behavior was not forced. The callback `0x5b18a0` has a closed nine-instruction  / 23-byte body containing a cycle. Qualification does not prove termination or require every path to return.

Rank-six entry `0x5b7ad4` jumps at `0x5b7ad9` to `0xbca73a`. Actual pre-creation exact/containing probes were null at both locations, with no overlapping function owner across the proposed body. One source function was created with two inclusive ranges:

| Range | Bytes | Instructions |
|---|---:|---:|
| `0x5b7ad4`–`0x5b7add` | 10 | 2 |
| `0xbca73a`–`0xbca768` | 47 | 10 |

The destination **is not an exact initial-census entry**, correcting an earlier narrative assumption. No owner was inferred from neighboring functions. No separate destination function, forced owner, merge or split was used. This is one 57-byte / 12-instruction identity.

## Two known returns: actual rendering change

Entries `0xbe4ea1` and `0xcd168b` were already counted in C10. Their bodies total **182 bytes  / 56 instructions**. The neighboring INT3 is guard context outside the first body. Reprojecting these known bodies adds **zero identities**.

The table consumer `0xcaf895` calls the callback and tests the whole EAX result. Original instructions support these bounded normal-path relations:

| Entry | Supported relation | Remaining limit |
|---|---|---|
| `0xbe4ea1` | EAX is zero at the ordinary return through its TEST/branch guard. | Failure/exception and inherited fastfail behavior remain unqualified. |
| `0xcd168b` | `zeroExtend(AL) XOR 1`, range 0–255; zero exactly when callee AL is 1. | No Boolean callee assumption. |

The first model set `undefined4 / EAX:4`. Signature priority stayed DEFAULT; both rendered texts remained byte-identical, respectively void and byte. Installed Ghidra primary sources explain why undefined return types did not promote signature priority and left output unlocked. That historical result remains frozen.

The completed subsequent model uses an explicit four-byte unsigned integer, **TYPE_UINT(10)** and **SourceType.ANALYSIS**, solely through the return-type API:

| Observation | Actual result |
|---|---|
| Signature source | DEFAULT→ANALYSIS, both entries |
| Return metadata | Four bytes /EAX:4, both entries |
| HighFunction output | void/byte→uint/uint, **2/2 aligned** |
| Rendered text | **2/2 changed** |
| Parameters, convention, other ABI and inspected callee metadata | Unchanged |
| New global identities | **0** |

No USER_DEFINED signature, parameter, callee, custom storage, flow or nonreturn flag was forced. HighFunction's reported model changed from `__stdcall` to `unknown`; this is separate from the unchanged function-metadata convention. Unsigned interpretation is an analytical choice consistent with normal EAX relations; original source signedness/debug signatures remain unknown.

Nondefault signature priority also locks the existing empty input list in the primary marshaller. This analytical consequence is retained: unchanged empty parameter metadata does not prove an original prototype. The inherited nonreturn warning is a limitation, not a behavior repair.

## Durable results and closure limits

Each primary result was written immediately to a CREATE_NEW per-entry JSON file, bound to target/plan/profile/phase/ordinal and forced to disk. All **555 ordinary and one rank-six receipts** agree with final reports and hashes. Partial/truncated records or absent final reports cannot become completed phases. Retained partial success does not authorize automatic producer replay.

Observed phase-local counts were **56,733→57,288** (555), **56,733→56,734** (rank six) and **56,733→56,735** (two known bodies). These are separate phases, not one database union. Postscript time is distinct from cold import/startup time.

Immediate restoration probes after rollback requests reported **false in all three phases**. Later count resets do not prove restoration of the whole old entry set. Original and analysis-copy hashes matched, and the final read-only private project was absent. Read-only discard and immediate undo are separate facts.

Owned wrapper/batch-chain and completion-handle receipts, followed by a terminal snapshot with no owned outer processes and zero native Java processes, support closure. The earlier Java PID was **not recorded during activity** and is not invented retrospectively. This historical snapshot says nothing about later engines.

The corrected profiles observed private JVM home/tmp, a 1,536 MiB heap and two processors. Ghidra preferences/cache still resolve under AppData; complete preference isolation is not claimed. Verification did not reacquire the whole prior function-body map in a second engine: full owner/data-intersection guards rely on pinned producer source and observed probes, corroborated by original bytes/census barriers. No game or recovered implementation was launched or built.

## Evidence and remaining scope

Raw pseudocode stays local under `client-data/audit/2026-11-02-global-decompilation-ec98a7d4`.

| Evidence | SHA-256 |
|---|---|
| Original SFrame public Native profile | `b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17` |
| Earlier combined-run verification | `41e6d25c86cacbd5a3e4d3bbfb367df1fa3c7e487df77010456e7ec980829726` |
| Completed 556 independent verification | `2e7948c3fcaab70be6a3d7fcdc0b194a86cbce91945c2a06781b2a8ad9bbbbe2` |
| Completed 556 summary | `1b66295729dd5332668b59fb827b23d9966e40a6fd610819a66ee2743c09eeca` |
| Frozen 556 profile | `2ac5adea08b36bd534c3146c0ed5aacddbe2feadb2f1d5bb11d222c4564e79b0` |
| Explicit integer-return component | `585b443f4d276e934be5e0b811b0fbe4cc4aae3ff20b34101e61be281699461d` |

The finite 581-entry startup frontier is accounted for locally. Unidentified code outside these tables, remaining Native dossiers, warnings, original signatures, callee semantics and real-session behavior remain open. See the [global audit runbook](GLOBAL_DECOMPILATION.md).
