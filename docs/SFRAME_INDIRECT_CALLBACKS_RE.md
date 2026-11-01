# SFrame indirect callbacks rooted in the character sheet

Verified **2026-11-01** on original x86 SFrame SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`, with **REA 6.4.0 / Ghidra 12.1.4 / Capstone 5.0.7**. This continues [concrete MAIN pointer flow](SFRAME_MAIN_POINTER_FLOW_RE.md). **22 new Native dossiers / 14,107 analyzed body bytes** qualify rectangle recipients, the placement message and a descriptor carrying MAIN as owner. **No nonzero write activating MAIN byte+0x458 is identified.** This remains an unresolved reconstruction requirement.

[Versions](VERSIONS.md) remain Core/RappelzRust **0.1.0 development**, NoLimits launcher **1.3.60**, original SFrame PE file/product **9.1.0.0**, embedded **Source9.5.2** build lineage and network literal/profile **201507080**. Exact official Epic/patch remains unconfirmed. These are static observations; the original executable is not run.

The [caption/parser/control follow-up](SFRAME_CAPTION_OWNER_RE.md) identifies the selected global caption handler, qualifies the original class-token translation path and distinguishes immediate subpanel owners. The caption callback is no longer an unidentified recipient; remaining effects and MAIN activation retain explicit limits.

## Questions and coverage

| Question | Result and boundary |
|---|---|
| Who receives MAIN rectangle borrows? | The selected original RTTI family has 315 primary KUIWnd table prefixes; +0x2c/+0x34 each resolve to three bodies and +0x24 to seven. Constructed Basic/Added membership and rectangle provenance are separately checked. Arbitrary runtime/custom vptrs are not observed. |
| Who receives the placement message through context virtual+0x10? | The constructed SGameManager route forwards SIMSG_GET_ADDON_POSITION, kind 0x48f, to Interface. The message contains copied rectangles and a coordinate result, without a MAIN pointer. |
| Does another callback receive the whole sheet? | The child-creation descriptor embeds MAIN as owner at+0xb0. It is distinct from the newly created callback receiver and KUIWndManager at+0xb4. Selected static-control creation reaches the generic initializer. Other class creators and arbitrary callee effects remain bounded. |
| What activates the alternate mode? | Unresolved. Known initialization/visibility clears and mode reads remain; the selected callbacks supply no demonstrated nonzero reaching write. |

## Rectangle callbacks: arguments matter

Exact original COL/RTTI hierarchy filtering yields **315 file-backed primary KUIWnd-base tables**. Three secondary vptrs at complete-object offset 448 are excluded because their KUIWnd PMD is 0. This closes the measured static family, not every possible runtime receiver. MAIN is SUICharacterInfoWnd→SUIWnd→KUIGenWnd→KUIWnd→KObject.

| Slot | Family distribution | Qualified effect |
|---|---|---|
| +0x2c | b369d0:244,9a6990:70,b3cdb0:1 | b369d0 ignores the first rectangle argument and passes only deltas/flags to b3c900. 9a6990 forwards all four arguments to list children before the delta update. b3cdb0 returns and pops 16 bytes. |
| +0x34 | b36990:244,9a6960:70,b3cd90:1 | b36990 conditionally tail-jumps b3c1b0 with the original rectangle argument. 9a6960 tail-jumps its own virtual+0x24; b3cd90 returns and pops 4 bytes. |
| +0x24 | b363f0:244,9a6700:66; b3ca90,b488d0,b5bdb0,b643b0,b643e0:one each | The full bodies qualify copied/read rectangle spans and further callbacks; 9a6700's cold bounds-failure tails are included, rather than stopping at its first RET. |

The enabled **b36990→b3c1b0** tail jump is especially important: pseudocode that displays a void/no-argument helper would lose the original stack argument. b3c1b0 forwards it to each child virtual+0x24 and returns **receiver+0x118**. b3ca90 reads a 16-byte rectangle into its own+0x118 and passes the original pointer to child virtual+0x34. After that base call, 9a6700 uses receiver+0x118 for its other geometry callbacks; its two cold tails push the bounds-error literal. b643e0 explicitly replaces the tailcall's argument with **receiver+0x2f0**, although pseudocode omits that argument. This qualifies the borrowed bytes locally; it does not exclude arbitrary receiver/global/virtual-callee effects.

Retained b36cb0 supplies **MAIN+0x108** to vector children and **MAIN+0x2c4** to list children. Retained b36a30 supplies **MAIN+0x118** to vector children. The latter alias extends the preceding report. MAIN initializer 8a1ab0 calls b3bd80 at 8a1b61/8a1bf3 to insert the constructed Basic/Added children into MAIN.list+0x44. Their exact tables d53448/d53be8 select the generic callbacks. No qualified local rectangle span reaches MAIN+0x458; this statement is limited to the proven borrow/family paths.

## Placement message: exact manager and layout

The constructed path establishes **MAIN+0x3e8 = SGameManager***. Application a69ed0 constructs 9ed620, passes that manager to Interface 9d0710, and 9d0710 stores its argument to **Interface+8 at 9d0758**. Interface 9c8e50 reaches the retained startup factory 9cb4e0. That factory passes Interface+8 to MAIN constructor 8a19f0→base 8208d0; the base stores it at MAIN+0x3e8 at **820a9a**. SGameManager vptr **d70c0c**, exact COL e140b4/type descriptor eb4014, has virtual+0x10→**9ee8c0→Interface9d3db0**.

Fresh **8215d0** constructs **SIMSG_GET_ADDON_POSITION** through **820870**. The constructor's complete body is **820870..8208c0,81 bytes**, including the three-byte RET0x24. It installs vptr**d47804**, kind **0x48f**, selector at+0x13, zero-initialized result at+0x17, and two 16-byte rectangle copies at+0x1b/+0x2b. It does not put MAIN into this stack message.

Interface dispatcher 9d3db0's original jump table at **9d9581** selects case0x48f at **9dc4ab**. Its selectors 1..10 calculate a coordinate from those copied rectangles and store message+0x17. Sender 8215d0 reads that result and moves the target from mapping-record+8 through retained b3c8a0. When Interface+0x190 is nonzero and byte[object+0xc] is nonzero, the modal guard returns for kind 0x48f before the hook/case, leaving this fresh message's initialized result at zero. Delivery at runtime is not asserted.

The pre-dispatch hook is also typed on this constructed path. Interface constructor 9bedc0 installs **SUIDisplayInfo** at Interface+0x10 (776780 at 9bee37, store 9bee40), vptr**d3ae5c**, COL e07ee8, type descriptor eae86c. Its virtual+4 is retained **794a90**. Original branch instructions for kind 0x48f reach its epilogue without calls or message writes. Later replacement of this receiver, manager replacement and arbitrary runtime hooks remain unobserved.

**Interface+4 is a different object: KUIWndManager***. Constructor9bedc0 allocates 0x60, calls fresh b3ddd0 and stores the result at 9bee18. b3ddd0 installs exact vptr **d89d34**, COL e1b624/type descriptor eb75fc. Retained 9c8920 passes Interface+4 as argument 2 to window virtual+0xd0. The constructed MAIN fields are **+0x138 KUIWndManager***, **+0x3e8 SGameManager*** and **+0x448 SUIDisplayInfo***; mode byte+0x458 is separate.

## Embedded owner descriptor and indirect creation

The other-callback audit covers **38 recorded states /36 distinct sites**, with six actual MAIN-self call sites. The constructed MAIN table resolves selected self slots+0x18/+0x28/+0x94/+0x40/+0xac to8a23d0/b36310/b3d620/b3ca70/b3c1d0. Named lookup b3c660 searches list[this+0x44] by child+0x4c and returns a child or zero; there is no self/ancestor special case. Renderer callbacks in b376a0 receive an explicit renderer argument, rather than MAIN. Its MAIN+0x3d8 primitive is a separately constructed KSpritePrimitive (vptrd7f910).

There is nevertheless a genuine **whole-MAIN handoff** in self virtual+0xac/**b3c1d0**. The descriptor starts at **EBP−0xec**; **MAIN is stored at EBP−0x3c byb3c436**, hence descriptor+0xb0. EBP−0x40 at b3c494 is a different field. The class string comes from an existing named member+0x64. MAIN+0x138 supplies the separate KUIWndManager receiver to b3ead0, which writes descriptor+0xb4.

Retained **b3ead0→b3ec90** selects a creator through global registry **ed27e0**. **CALL ESI at b3ede7** calls that creator without an explicit descriptor argument. The returned **new object's virtual+8** then receives the descriptor. Fresh **b3c510** stores descriptor+0xb0 into the new child+0x20 and descriptor+0xb4 into new child+0x138. Afterwards b3ead0 restores owner from descriptor+0xb0 and calls b3bd80 to register the new child under MAIN. MAIN is the owner/list receiver; it is not the newly created object.

The initial NUI loader **b35b60** supplies the same owner path: MAIN initializer 8a1ab0 calls 820f70 at 8a1adc, and 820f70 calls b35b60 at 820f9d while preserving MAIN. It preserves its MAIN receiver in EDI at b35b7f; the controls descriptor is EBP−0xc4, and **b35d20 stores EDI at EBP−0x14 = descriptor+0xb0** before CALL b3ead0 at b35d35. The resource root genwnd first receives virtual+8 directly on MAIN at b35bff; the 47 MAIN child blocks use the factory. This establishes the descriptor handoff during initialization as well as the clone path. Complete source bytes and the initializer-to-loader receiver chain are retained in the other-callback ledger.

Format-aware resources narrow a frequent branch. The original archive index, encoded names, continuous index XOR and phase-zero content XOR are checked again; all three decoded contents equal the retained caches. Complete block parsing gives MAIN one root/47 children, Basic one root/141 children and Added one root/72 children. MAIN contains genwnd1/static37/simplebutton2/gauge3/tab_sheet1/tab_head2/iconstatic2; Basic's children are static139/simplebutton2, Added's children static70/simplebutton2. The original registration at **5b67a0** binds class string **static at d36180** to retained creator **b67110**, which allocates 0x2ec and installs **KUIControlStatic vptr d8d0f0** at b6714e. Its virtual+8 is fresh **b670e0**, which forwards the descriptor to fresh **9a5170**. The original parser's full token-to-descriptor translation is not reconstructed; matching class-token routes remain conditional on that translation and selected registry contents. These facts qualify the static-control branch without promoting every generic KUIWnd type to an actual MAIN child.

The measured KUIWnd family still has **17 distinct virtual+8 targets**. Generic initializers 9a5170/9aad50 and b3c510 are retained this pass. Important remaining callouts are explicit: b3c510 conditionally invokes global **[edd5a0].virtual0** with receiver+0x94, which is **MAIN+0x94** on the root path; 9a5170 calls its receiver's virtual+0xdc and deletes an optional separate receiver+0x288 object through virtual0. These callback effects, other class-specific creators, mutable registry/list contents and recursive/reentrant effects remain separate gaps. Resource equality and class registration establish a static route, not live resource precedence or execution of the clone callback. No MAIN+0x458 nonzero write follows from the selected path.

## New Native evidence

Each record is bound to the original target SHA and analysis-profile digest `b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17`. Inclusive provider body ranges and original PE instruction boundaries are retained locally.

| Entry | Analyzed bytes | Evidence ID |
|---|---:|---|
| 0x8208d0 | 497 | `ev_2936d3aaf895e308c7373afcd56552ceab65af89dc7dc8ea3a6c3e04c83e0996` |
| 0xb3c900 | 70 | `ev_1e64496cf4f106b6dd9f05308a6c664c9494ca1e94e09ba3a2833cbae2bf6c91` |
| 0xb3c1b0 | 25 | `ev_05e433f4d1a1545a0d67ddca77324621de5ceffd3d137d8fd62d9a75d422a466` |
| 0x8215d0 | 439 | `ev_2959bab99f40d275c9fd01f158d9a4afa119699b26227f3d92c631d9504800a4` |
| 0x9a6990 | 113 | `ev_5fcd3067cf2b236988be110a1219f542ce453cd65efb18f943806a567be83bf1` |
| 0x9a6960 | 9 | `ev_345da3219d1c00bea9d4357af21e684a3eb6d8eb1408dea5ce73de04fd257039` |
| 0xb3cdb0 | 3 | `ev_0a9709187097b3f9fb98e770e985d11d58eda704df9250df7d837bd4f6038814` |
| 0xb3cd90 | 3 | `ev_4301bfc229180986fd63b9ccb7915546d937d0ecdaeda8fb50c7a2363281249e` |
| 0x9d0710 | 2297 | `ev_2a1a76c68e4ba6016bc3f7c437712c7ff656d4675c07abe738c1990e1094af78` |
| 0x820870 | 81 | `ev_84ead7ee0328cf06bb4e895dcc6433730530b24a28f8b6a10f5263c6186da7ff` |
| 0x9a6700 | 242 | `ev_5f7818df5224d2f18b3a5bf05e8a34fa2b107067d30ef4ac366d374e075c373f` |
| 0xb3ca90 | 62 | `ev_2be5ebc7010eac9b395c9b31ba4e84bdcf3326b4c7c5b4841db34b12b05a9a70` |
| 0xb488d0 | 55 | `ev_4d73135b6ff4c003d6647881c59a6f5cd785f880d3629d2dbec90a709b3ec2bc` |
| 0xb5bdb0 | 9 | `ev_ceb69bf0d6b7a92db0f7744c1b47b723bc1e4afb712bda9ccceebdf24a13b6ba` |
| 0xb643b0 | 46 | `ev_1acaf828f68ca19171751a167c5d9142b584a19e552b44fc6c9295be0540ea5e` |
| 0xb643e0 | 54 | `ev_3a9aa141f39229b6af7dfc3f0268d5e2022b4d09d9ed04ff78ed558d35bced25` |
| 0xb3c510 | 318 | `ev_6356bc8bbb36f8a8053edebbc871fdb724d632f03e1057a807c86f753eaca43d` |
| 0xb3ddd0 | 341 | `ev_e0c3e4a3e323b6632c1d0ba389838022ed3efb557a2a5cfba2b64859d1105c3e` |
| 0x9c8e50 | 8700 | `ev_204a6f72e731f1b7238b53678677ff1e2f7c3a80b0097e25efaf712356ea22b2` |
| 0x9a5170 | 692 | `ev_ee83ed194b19c037d963a597ac72ac4ac55c2c284cd2f407cca0946112ec93fb` |
| 0x9aad50 | 9 | `ev_9f5893ba70700d907047002b420f9928393cab547a6105c8fd6c9d274bc16aba` |
| 0xb670e0 | 42 | `ev_196628d23ba31ba6929cb63704a4344641b77d5c7cdb6d9bd4ca98e1ee54ad70` |

**Graph limits:** One new value-flow graph reports truncation. A graph not marked truncated does not resolve unknown indirect targets or callee effects. Selected instruction-level calling convention and receiver facts are checked independently.

| Entry | Known omitted edges | Omitted operations lower bound |
|---|---:|---:|
| 0x9c8e50 | 0 | 1 |

The reused Interface dispatcher 9d3db0 also reports a truncated graph with 240 known omitted edges/operations lower bound 1; constructor 776780 is truncated with one known omitted edge. The full body/assembly and selected original instructions support the narrow facts above; they do not restore full graph coverage.

| Selected reused Native source | Evidence ID |
|---|---|
| 0x794a90 | `ev_0708ab16944356e6ed57fc4c4884faa18ecd36ec8349113c54b89532625ee641` |
| 0x776780 | `ev_cc4f5fb843017a46cffc4f840ffcc2d37aeb4e48178a67b4e328390c260f6aa6` |
| 0x9d3db0 | `ev_83e67f6199a1a76051df09eebc8044ed8a07305f6d8d4781f3f2f0314abbfc2f` |
| 0x9ee8c0 | `ev_f6401cb43b6dc2b1cea75674e56b68e3f3aedff971e8059551991d0fcd16b494` |
| 0xb3ead0 | `ev_5f57d5eeeef5a105042b0ceec3f79df05293b720c9fe47ae03fe6cc106e0f02f` |
| 0xb3ec90 | `ev_ddc5747a3d5b8e234861260a575d82bcbd6db31a663dc92f6dd55a7ae6e2c860` |
| 0xb35b60 | `ev_780cd463050f07ecb6fc2124376fbcb2e95dd2a03b4f84435dcbface76076ff5` |
| 0xb67110 | `ev_7ebc0497915501cf0499b1a452b680fafe0fcde49f19edb9452a99a8e61b6c2b` |

## Verification and retained state

Independent original-byte checks pass for **4,201 new Native assembly instruction boundaries** and **3,195 distinct selected ledger intervals**. The root reuse check also verifies **14 retained Native functions**; the workers separately retain their complete source IDs, source hashes, type bindings and local coverage limits. Historical individual Native records remain available outside the previous canonical bundle; that archival index is still necessary.

The first owned session returned constructor 8208d0, then its export transport closed unexpectedly. Diagnostics show no established cause. The complete returned Evidence is preserved and authenticated through REA's official bundle builder/import; its producer is not repeated. A fresh static session saves the 22 new records and 21 newly cached primitives. A separate data-only REA session then imports that light snapshot and the prior 465-record canonical bundle, exports and closes successfully, without new producer calls. A completed 9c8e50 response exceeding the transport size is recovered from its retained canonical export without reanalysis. The previous **339-primitive full snapshot remains untouched**. The new snapshot has **487 canonical records / 21 freshly cached primitives**; imported Evidence is not falsely counted as an old primitive cache.

Ignored run: `client-data/audit/2026-11-01-sframe-indirect-callbacks-6e9d302a`. Its canonical bundle is **241,203,268 bytes**; closed snapshot **251,978,449 bytes**, SHA-256 **`9591a8d15e279d248a0ac243e91d6111c9d15bbee8122b8a333995693b1b976a`**. Full Native records, rectangle-family/provenance, context and other-callback ledgers, resource/class bindings, transport diagnostics, verification and publication receipts stay local.

Nine original-data preservation checks pass. Eleven baseline original/installed image, addon and settings hashes, saved-variable trees, the preserved backup and primary service listeners/identities remain unchanged. Root Git index stays empty. Only seven authored Markdown files are synchronized through the isolated Core/Client publication worktrees; no original binary, resources, database, configuration, credentials or generated decompiler output is published. No Rust behavior is added from an unresolved activation rule.

## Continuation

Reuse this canonical bundle, its selective primitive snapshot and the prior historical Native index together. Continue from the descriptor's proven owner field and actual resource class registrations, keeping newly created receivers separate from MAIN. Finish remaining class-specific creation/recursive callbacks only when a concrete owner path reaches them. If static evidence still cannot bind activation, a controlled original-runtime write observation would be a separate evidence step, with explicit host and data-preservation boundaries. Require a demonstrated nonzero write reaching the constructed MAIN byte+0x458 before reconstructing activation.
