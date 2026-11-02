# SFrame 0x918630 caller search and MAIN descendant provenance

Verified **2026-11-02** on original x86 SFrame SHA-256 **`27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`**, following the [receiver-family investigation](SFRAME_MODE_RECEIVER_FAMILY_RE.md). **An actual caller/object for 0x918630 and the nonzero activation trigger of MAIN+0x458 are not identified.** This pass applies the computed-address model to **106 additional retained procedure bodies** and checks exact MAIN type-descriptor reuse through the retained RTTI tables and all declared PE HIGHLOW relocations.

This is a source-derived audit with **zero new Native decompilations and zero new REA queries**. No new engine session is opened, no original process is executed and no synthetic state is installed in the game. The existing canonical bundle remains at **697 records**, with its prior **24-primitive** snapshot reused in place.

[Versions](VERSIONS.md) remain Core/RappelzRust **0.1.0 development**, NoLimits launcher **1.3.60**, SFrame file/product **9.1.0.0**, embedded **Source9.5.2** lineage and network literal/profile **201507080**. Exact official Epic/patch remains unconfirmed. Retained evidence uses **REA 6.4.0 / Ghidra 12.1.4**, independently checked with **Capstone 5.0.7**, profile **`b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17`**.

## New question: the retained-body delta

The original computed-entry model definition prefix is reused without changing its transfer or sink semantics. Only the retained-index difference is fed to it; additional logging records original-image reads. The exact difference is 449 to 555 procedures: 106 added addresses, no removed addresses, and no body or Evidence ID changes among the 449 common procedures. Existing Native dossiers are reused individually; no Native decompilation is produced in this scope.

Only the 106 new-to-model procedure bodies are replayed, covering 106 provider ranges, 40990 range bytes and 11417 provider-aligned instructions. Both model scenarios have zero skip-data bytes and exactly match the provider instruction boundaries. Their bytes were already inside the previous heuristic whole-.text coverage; the added evidence is provider body/alignment provenance and state reset at each retained body range.

| Corpus check | Result |
|---|---:|
| Prior / current retained procedures | 449 / 555 |
| Common procedures with identical IDs and bodies | 449 |
| Additional bodies / ranges analyzed by this model | 106 / 106 |
| Additional provider-bounded bytes / instructions | 40,990 / 11,417 |
| Provider boundary mismatches / skip-data bytes | 0 / 0 |

These are **106 existing dossiers newly covered by the Native-boundary model**, not new decompilations or previously unexamined executable bytes. Earlier [whole-text heuristic coverage](SFRAME_918630_INDIRECT_ENTRY_RE.md) remains separate; it is not replayed or counted twice.

The generic unknown-callee scenario reports 0 computed values in 0x918630..0x91875c at modeled CALL, JMP, memory-store, pushed-argument or PUSH/RET sinks. Unknown calls clobber volatile registers and tracked memory.

The original CFG RET-0 conditional scenario reports 0 such candidates. It preserves registers and tracked memory at the original CFG check only under the explicit unchanged-slot/code condition: [0x00d279ac] = 0x00bc811c, bytes c2 00 00. This is a check, not an intended-target dispatcher; load-config data-directory size 64 and structure.Size 160 remain distinct.

Neither pass encounters a call to slot **d279ac** in these 106 bodies, so their counts and results coincide. Each processes **216 indirect calls, 4 indirect jumps and 149 returns** as instruction classes, not as resolved runtime destinations. The model also records **286 unknown writes** per scenario; those are residual precision limits, not 286 proved safe writes.

All **15 synthetic semantic controls** pass. The two scenario-specific positive records describe **one unique original computed jump**, **71e61f→71e626** in retained **71e610**, counted once per scenario. It is a decoder/model control, not an observed call to 918630. Synthetic controls are not client executions.

This is bounded linear, intra-procedure computed-value propagation, not a CFG fixed point or runtime observation. Literal-only values are not newly classified as computed entries. Branch joins, cross-function propagation, heap registrations, unknown dynamic inputs, modified guard/code/data and residual prior-corpus coverage remain outside the result. Value age is capped at 128 instructions, trace provenance at 32 instructions and tracked memory at 1024 bytes. Three reused Native high-p-code graphs are truncated (0xa18250, 0xa6adb0 and 0xb3ab30); the replay uses complete verified assembly/body ranges rather than those graphs. A zero candidate count does not establish unreachability or a dead feature.

| Reused graph with truncation | Known omitted edges | Omitted-operation lower bound |
|---|---:|---:|
| 0xa18250 | 0 | 1 |
| 0xa6adb0 | 0 | 1 |
| 0xb3ab30 | 1 | 1 |

The earlier source-bound incoming query **`ev_01b55707ea22285c9103b4c429d9a29add3b1840d29c21e3181b50964ab4a75e`** remains a bounded zero-reference result over the target body. It is reused, not rerun. The complete 301-byte 918630 dossier remains **`ev_2e11babc974a7b4241ded6116bc72be4cbafa3d434f034fc40c37e79a04da7bc`**. No direct or indirect reaching edge is manufactured from the new negative model result.

## New question: can a retained derived MAIN object supply the writer?

The exact MAIN RTTI chain is vptr d53af8 → COL e0de70 → TypeDescriptor eb14f8, whose bytes name SUICharacterInfoWnd. The COL points to CHD e0de84 and the five-entry BCD array e0de94. The self BCD e0deac names that exact TD at index0 with PMD (0,-1,0). Its other four entries are SUIWnd, KUIGenWnd, KUIWnd and KObject. This reuses the known own hierarchy; the new question is whether other retained hierarchies contain MAIN as a base.

All 2134 vtables from the retained RTTI inventory were rebound to their original COL, CHD, full BCD array and fixed BCD fields. Only the original MAIN table contains TypeDescriptor eb14f8, at its self entry. No different retained complete-object TD has a hierarchy containing that exact MAIN TD. This is a negative descendant result within those retained, source-qualified tables; it does not establish a general ban on derived objects.

The retained inventory contains **2,134 distinct COLs, 2,092 distinct CHDs/arrays, 2,186 distinct BCDs and 7,528 base entries**. The tested predicate is the **exact TD pointer eb14f8**, with decoded PMD adjustments, rather than a similar class name or overlapping member offset. Its only matching retained table is MAIN's own table; **zero other complete-object types** match.

An independent exact-pointer check parsed the complete PE-declared 445636-byte relocation directory: 2654 blocks, 210912 HIGHLOW slots and 1290 ABSOLUTE padding entries. All HIGHLOW slots were file-backed. Only two HIGHLOW pointers equal the MAIN TD: e0de7c in its own COL and e0deac in its own BCD. Targeted references to that BCD, CHD, array and COL close the same own structure; they do not introduce another descendant hierarchy. The directory is a static declared-pointer boundary, not evidence about loader or runtime behavior.

The only HIGHLOW pointer to the original MAIN vptr d53af8 is the immediate at 8a1a0f inside the complete retained constructor 8a19f0. That constructor captures entry ECX, installs this vptr and explicitly clears byte receiver+0x458. The retained original slot0 selects deleting destructor 8a1a80. No new derived constructor, override or concrete continuation to 918630 is justified by this RTTI/relocation result.

The inventory was originally limited to aligned, file-backed .rdata/.data table starts with validated COL/type names and a consecutive executable prefix. This pass reuses those starts rather than creating a new general vtable or field-offset scan. The independent pointer check covers declared HIGHLOW relocations; unrelocated values, synthesized pointers, memory copies, stripped or absent RTTI, tables outside the retained inventory, other modules, dynamic construction and runtime modifications remain outside the negative result. Exact-TD reuse is the tested predicate; equal names, offsets or resources do not prove type identity. Original-byte completeness and Native pcode limits remain separate. The metadata audit itself starts no engine or original process and changes no game data or service.

## Evidence checks and preservation

| Reused MAIN Native entry | Evidence ID |
|---|---|
| 0x8a19f0 | `ev_f3a4ff37e3949c13ed452102ee534729623efb381bd564fe0da8330bbded3cd2` |
| 0x8a1a80 | `ev_c65d58f897a013dba0a8639e4b4cffc362a41cba3ec1fda10be2c72747c47761` |

The exact 106-entry delta, all source Evidence IDs, original paths/hashes, body ranges, per-function results, model derivation, controls and unknown operations remain in the ignored local **entry ledger**, SHA-256 **`64a3b3c631e3e76d2bddc22c3d5c491db2b450b3ddd283776555010176cb243d`**. The separate **descendant ledger** retains the table structures, complete declared relocation coverage and original-byte intervals, SHA-256 **`1935abac9f01eff20eb300f3c9282ae7228e00d768cd85cd70fa105358d356ce`**. These local ledgers are new source-derived audit records, not new canonical REA observations.

Independent checks verify **108 distinct retained Native sources** and **8,651 distinct selected provenance intervals**, remapped to the original PE and checked against complete decoded body boundaries. Native body completeness, pcode truncation metadata and the computed model's precision limits remain separate. No new whole-text, generic pointer-carrier, PDB or neighboring-module scan is claimed.

The prior **697-record / 317,743,005-byte** canonical bundle and **24-primitive / 328,815,816-byte** snapshot are preserved and compared as complete JSON values, one decoded record at a time. Snapshot SHA-256 **`0421d455a4468c1e98f6563a57c587e8749d0e9ed1e44c7821560728ff47ee74`**, at `2026-11-02-sframe-mode-receiver-family-84c1d9a7/sframe-mode-receiver-family.snapshot.json`. That session was closed in the previous pass; this audit creates no duplicate snapshot or new close receipt.

Ignored local run: `client-data/audit/2026-11-02-sframe-entry-descendants-57b8c204`. **Nine data-preservation checks pass.** Eleven baseline file hashes, installed saved-variable tree, logical database backup and primary service identities remain unchanged. The root index stays empty; only seven authored Markdown files are published through isolated Core/Client worktrees. Gameplay code, original binaries/assets, databases, settings and package versions are unchanged. No raw dumps or credentials are published.

## Remaining identification requirement

The replay provides no new source-supported computed-entry anchor for 0x918630 and does not establish a nonzero write to typed Main+0x458. The receiver of 0x918630 and the flag activation trigger remain unknown.

This pass does not establish any nonzero write to MAIN+0x458 or a constructed receiver for 918630. A derived MAIN object is not supplied by the selected RTTI or targeted relocated-pointer sources. The activation question remains unresolved, and no usable or previously observed alternate mode is presumed.

A decisive continuation still needs an actual reaching transfer or registration to 918630 with the origin/type/state of entry ECX, or a separately qualified nonzero write to +0x458 on the same constructed MAIN object. The typed Arena paths, changed 106-body coverage and exact-TD descendant question now have explicit source bounds. They do not establish universal absence of callers, derived objects or writers, dead code, or a usable alternate feature. The user's request remains an investigation of the code flag, not an already observed alternate mode.

## Follow-up: interprocedural arguments, returns and MAIN helpers

[Interprocedural values and MAIN contracts](SFRAME_INTERPROCEDURAL_RE.md): new bounded argument/EAX-return propagation on 555 retained roots evaluates 329 callee contexts and retains 51 conditional summaries, with one source-bound consumed return and 22 new semantic controls. Two new Native helpers / 160 bytes qualify MAIN.virtual28→b36310→MAIN/loaded-primitive effects; child.virtual38 remains open. No 918630 reaching receiver or MAIN+458 trigger is established. Nine preservation checks pass; versions unchanged.
