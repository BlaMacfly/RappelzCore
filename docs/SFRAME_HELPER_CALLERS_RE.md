# SFrame caller search for the alternate-mode copy candidate

**Follow-up:** [All 102 pointer deliveries](SFRAME_POINTER_DELIVERIES_RE.md) are now reviewed with 98 provider-contained sites and four raw-only fragments. ABI artifacts, typed container paths and an ArenaWaiting countdown reset are distinguished from MAIN mode. MAIN activation and several helper/virtual receiver effects remain unresolved.

Verified **2026-11-01** on original x86 SFrame SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`, using **REA 6.4.0 / Ghidra 12.1.4** and **Capstone 5.0.7**. This follows [the implicit-copy investigation](SFRAME_IMPLICIT_COPIES_RE.md) and focuses on the unbound **0x812a90** entry and its inclusive **0x812a90..0x812abc** body. **No ordinary static caller was recovered in the examined forms. MAIN SUICharacterInfoWnd byte+0x458 activation remains unresolved.**

[Versions](VERSIONS.md) remain Core/RappelzRust **0.1.0 development**, NoLimits launcher **1.3.60**, original SFrame PE **9.1.0.0**, embedded build lineage **Source9.5.2**, network literal/profile **201507080**. The exact official Epic/patch remains unconfirmed. This pass performs no original-target execution or live object observation.

## Incoming-reference and pointer boundaries

The retained 45-byte helper dossier `ev_f34a5f70a696cff7dd56f7095f25d579146778cf009b3f7f18fc2484bea1edb8` already reported no direct callers/incoming references. The new focused incoming-reference query is checked separately against original control-transfer encodings and initialized pointer carriers. A provider's negative reference result is not whole-program alias analysis.

| Search | Measured coverage | Result and limit |
|---|---|---|
| Linear executable decode | 9,589,018 file-backed executable bytes; 3,370,152 instruction/skip-data entries; 530,544 control transfers | No decoded direct transfer into the inclusive helper body. Sequential section decoding is not exhaustive CFG or instruction-boundary recovery. |
| Relative-encoding scan at every executable byte | E8/E9 rel32, EB/short conditional/LOOP forms and 0F8x rel32 | One apparent match, at 0x812ad1, lies inside a separately decoded CALL's displacement. Prefix-dependent rel16/other execution modes and computed destinations retain their limits. |
| Far 9A/EA ptr16:32 offsets | Every executable byte with enough original bytes for the encoding | No matching offset. Selector/runtime mapping is not established by a raw offset search. |
| Exact initialized address carriers | All 12,481,536 file bytes; 90 DWORD patterns: VA and image-relative RVA of each of 45 body addresses, including unaligned positions | Zero occurrences. This covers exact literal 32-bit pointer carriers in this original file; computed, split, encoded, relative or runtime-created pointers remain possible. |
| PE base relocations | Entire 445,636-byte directory; 2,654 blocks / 212,202 entries: 1,290 ABSOLUTE padding and 210,912 HIGHLOW | All HIGHLOW targets file-backed; no stored value targets the body. Relocation presence alone would not establish an indirect call consumer. |

No absolute function-pointer/vtable slot initialized with one of these exact addresses is present in the original file. This bounded result does not prove the function is globally unused, and does not rule out dynamic patching, relocated/generated code, runtime pointer arithmetic, exception-driven entry or externally supplied data.

## The apparent branch is inside another instruction

At original address **0x812ad1**, decoding the two bytes **7b e0** in isolation yields **JNP 0x812ab3**, inside the helper. The actual surrounding instruction starts one byte earlier:

| Address | Original bytes | Decode |
|---|---|---|
| 0x812ad0 | e8 7b e0 23 00 | CALL 0xa50b50 |
| 0x812ad1 | 7b e0 | Apparent JNP 0x812ab3 from a displacement byte |

The neighbor's provider body and assembly bind the normal instruction boundary at 0x812ad0. There is no provider instruction boundary at 0x812ad1. Thus this raw byte hit is not evidence of an ordinary decoded incoming branch. Deliberate overlapping entry is not globally excluded without a reaching control-flow path.

The preceding code ends with **0x812a7f: JMP 0x812fb0**, followed by **12 INT3 bytes** at 0x812a84..0x812a8f. The helper's last instruction tail-jumps to 0x80bc80; **three INT3 bytes** at 0x812abd..0x812abf separate it from0x812ac0. These bytes and provider extents rule out ordinary fallthrough from the inspected neighbor, rather than proving all forms of runtime entry impossible.

## A typed default-settings branch duplicates the core effects

The prior **0x80e470** dossier `ev_ceaa757d7ece5998387f44e8349d8e0e796edca487caa5dbca98744ce5472f51` is bound by original **SUISystemEnvironmentChild vtable 0xd45adc+0x18**. The selected input-dispatch path is now checked directly from original bytes:

1. Entry preserves its actual ECX receiver in EBX at 0x80e499.
2. Callback parameter 2=10 selects byte-table index 0 and jump-table target **0x80eb3b**.
3. The branch compares the supplied control name with the exact literal **button_def** at 0xd45ef8. A zero comparison result reaches 0x80eb61.
4. **Nine corresponding core instructions** in 0x80eb61..0x80eb7e and 0x812a95..0x812ab2 have identical decoded instruction effects: call source getter 0x9f5c80, set destination receiver+0x44c, copy 85 DWORDs, then pass the destination to 0x811c20 with ECX restored to the receiver.
5. The inline branch calls 0x80bc80 with ECX=the same receiver. The out-of-line helper restores its preserved registers and tail-jumps to that same target.

This is a concrete default-settings callback path with equivalent selected copy/update effects. It is **not an incoming call to 0x812a90**. Compiler inlining, source-level identity and why an unreferenced out-of-line form remains in this binary are inferences, not recovered build facts.

Fresh **0x811c20** consumes the supplied block to update named resolution/refresh combos, checkboxes and sliders. Fresh **0x80bc80** examines minimum_system/maximum_system settings and updates control enable/color state. These selected operations further support the settings interpretation; both new value-flow graphs are truncated, so their complete downstream effects are not claimed. Fresh neighboring **0x812540** ends at 0x812a83 with the checked tail jump. Fresh **0x812ac0** has a 32-byte body ending at 0x812adf and contains the actual CALL at 0x812ad0; its separate byte write at receiver+0x2b28 is not a demonstrated MAIN+0x458 write.

The source remains the fixed settings global 0xed5138, with the base block copied here rather than the+0x154 block used by other settings reload paths. The forward span+0x44c..+0x59f includes relative byte+0x458, but the actual receiver in the typed callback is the settings child. The known MAIN allocation and constructor are distinct; a matching member offset alone does not establish MAIN provenance. DF, the live source byte and arbitrary aliases remain unobserved.

## New native observations

This pass adds **4 function dossiers / 5,487 analyzed body bytes** and **3 reference/containment observations**, without re-decompiling the already retained 0x812a90 or 0x80e470 dossiers.

| Entry | Analyzed body bytes | Evidence ID |
|---|---:|---|
| 0x811c20 | 1359 | `ev_d63f1b3e376f64a0fbc2d0f0192e2f5cf1d482abb516cc5018d5a73cbe110e70` |
| 0x80bc80 | 2748 | `ev_ca38a333b83a6c88f06c523a687c59b807d4de71b9ea79d4d3c3637f99429acd` |
| 0x812540 | 1348 | `ev_20aa89a4579ccc0b237fd81422afbc4f29226c10dc12560312348dbaeb29ef8a` |
| 0x812ac0 | 32 | `ev_aee6a68492149fb749e90dc02ae0b0a06b65517017c2b5fecf232299bf92a065` |

| Focused query | Evidence ID |
|---|---|
| procedure_references 0x812a90 | `ev_dc2574ed5a936c24c4682fad8c0967c366b30e29e9adaa720c92dcbad40dcf9d` |
| resolve_containing_procedure 0x812a7f | `ev_089f9e27951c07d8d00bd58389cca8bcb9e3ef47c64da3669251f8584186f663` |
| resolve_containing_procedure 0x812ad1 | `ev_cbf0f11c3890c21d3ff6a339015d79a3d6f645f3c693c84aacad132905885fba` |

Graph completeness for the new dossiers is retained explicitly:

| Entry | Graph truncated | Known omitted edges | Omitted operations lower bound |
|---|---|---:|---:|
| 0x811c20 | True | 0 | 1 |
| 0x80bc80 | True | 0 | 1 |
| 0x812540 | False | 0 | 0 |
| 0x812ac0 | False | 0 | 0 |

The reused 0x80e470 graph is truncated, with omitted-operations lower bound 1 and known omitted edges 0; zero known edges does not make that graph complete. The selected dispatch, comparison and copied instructions above are independently checked against original bytes and provider assembly boundaries.

## Verification, preservation and continuation

Independent checks remap every new inclusive native body to original PE bytes and confirm **1,669 assembly instruction boundaries**. Selected source/provenance checks cover **207 distinct byte intervals**, plus complete relocation-entry/block validation and all 90 exact pointer-carrier searches. The control-scan sidecar independently verifies 197 intervals, the 45-byte body SHA-256 `0fe800914eb62f90e1b4db5ee82dd250c01f273240d19a270799f8daeeb7d003`, the false-candidate containment and padding. These are bounded static checks, not execution or a memory-safety proof.

The previous **194 Evidence records** are retained unchanged. The closed canonical bundle contains **201 records**; the closed snapshot contains **169 cached primitives**. Reference/containment metadata queries do not add primitive-cache entries. The owned engine is closed after export. Raw outputs remain ignored under `client-data/audit/2026-11-01-sframe-helper-callers-e6ab819c/`.

All **nine original-data preservation checks** pass. Original SFrame/archive, installed images, addon/settings, backup and primary service listeners/processes remain unchanged. No original client execution, server restart, migration, character reset or gameplay mutation occurs. Seven documentation files are published through the isolated Core/client worktrees; the combined root workspace index remains empty. Existing client/game version distinctions are preserved. No Rust behavior changes or unrelated build/test reruns are part of this documentation-only pass.

**Unresolved:** a runtime/indirect caller of 0x812a90, its receiver outside the demonstrated equivalent settings branch, and the actual nonzero MAIN+0x458 writer. The ordinary static caller hypothesis is now narrowed; do not keep treating this helper as a proven MAIN path. Continue with the retained 102 pointer deliveries that can be tied to the constructed MAIN object, including interior-pointer adjustments and indirect callbacks. Any activation implementation still requires MAIN identity, a reaching write and the actual nonzero mode byte.
