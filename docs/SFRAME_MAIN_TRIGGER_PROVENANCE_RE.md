# SFrame MAIN indexed destinations and 918630 receiver provenance

Verified **2026-11-01** on original x86 SFrame SHA-256 **`27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`**, using **REA 6.4.0 / Ghidra 12.1.4 / Capstone 5.0.7**. This follows [the notification/countdown investigation](SFRAME_NOTIFICATION_ACTIONS_RE.md). **The receiver of 0x918630 and the MAIN alternate-mode trigger remain unidentified. Two actual MAIN constructor/initializer paths constrain indexed-write destinations; exact RTTI and available symbol metadata supply no reaching MAIN receiver for the orphan function.** New evidence: **2 Native dossiers / 1,046 analyzed bytes** and **0 other observations**. These are static artifact observations; the original client is not executed.

[Versions](VERSIONS.md) stay Core/RappelzRust **0.1.0 development**, NoLimits launcher **1.3.60**, original SFrame PE file/product **9.1.0.0**, embedded **Source9.5.2** build lineage, and network literal/profile **201507080**. Exact official Epic/patch remains unconfirmed.

## Constructed MAIN and indexed destinations

Constructed MAIN is **SUICharacterInfoWnd**, ID **0x2e**, vptr **d53af8**. Retained constructor **8a19f0** preserves this object and calls **8208d0 at 8a19fd**; that constructor forwards the same receiver to new Native **b35070 at 8208ef**. The base constructor initializes **MAIN+0x328/+0x32c/+0x330** and the rendering pointer at **+0x3c8** to zero. The derived Main constructor subsequently clears **byte MAIN+0x458**; base initialization is therefore not presented as a surviving activation.

The exact MAIN **virtual+0x08** selects retained **b35d90**. Its **MOV ECX,ESI at b35f7b / CALL b38220 at b35f7d** provides actual MAIN receiver provenance for the new initializer, under **!(MAIN.flags14 & 0x100)** and the earlier successful-sprite branch. The original MAIN resource's textual style is not silently equated with this numeric bit; the selected bit mapping remains unproved.

New Native **b38220** releases the previous **[MAIN+0x3c8]** value under its cookie/virtual-deletion branches, computes an allocation of **4-byte cookie + n×0x144 bytes**, and stores the returned element pointer or null at **MAIN+0x3c8**. The construction loop uses **0x144 = 324 decimal bytes** per element. Indexed destination stores, including the dirty flag at **element+0x141**, are based on that loaded allocation pointer; they are not rewritten as stores based directly on MAIN. This selected allocation provenance assumes valid allocator/element contracts. Count and pointer are reloaded around downstream calls, so mutation, reentrancy, allocation failure and arbitrary aliases remain open; no global no-overlap theorem is claimed.

Retained **b386d0** has a separately checked inline write span **MAIN+0x334..0x3c3**: three loop iterations with stride 0x30 and the selected DWORD stores. The path's count==9 guard does not prove a universal range for all later counts. Retained **b36a30** also writes through begin/end loaded from **MAIN+0x328/+0x32c** with 16-byte entries. Their zero constructor state is now confirmed, but their later producer/mutations remain a distinct unresolved pointer-provenance boundary.

A historical notation is corrected in [the remaining-callback report](SFRAME_REMAINING_CALLBACKS_RE.md): its separate control+0x1f8 array also has stride **0x144 (324 bytes)**. The old 9a9050 construction remains bounded raw coverage, not a newly captured Native dossier. Original instructions **9a90b2 MOV EDX,144h** and **9a90f4 PUSH 144h** corroborate the clarification; the underlying primitive-constructor/RTTI proof is reused.

## Receiver constraints and authenticated symbol metadata

Exact RTTI gives **MAIN: d53af8→COL e0de70→CHD e0de84**, and **ArenaWaiting: d5c778→COL e0f320→CHD e0f334**. Each hierarchy has five entries: its own derived class followed by the four shared bases **SUIWnd→KUIGenWnd→KUIWnd→KObject**. The selected PMD triples are **(0,-1,0)**. Neither derived class appears in the other's base list. This establishes distinct classes with a common prefix and selected zero base adjustments; it supplies no reaching cast or receiver identity for 918630.

Retained **8fb010** has a positive typed caller in the registered ArenaWaiting callback **917aa0 at 917ca4**, and the separate orphan caller **918630 at 918726**. Reusing that state helper does not transfer the first caller's class to the second. **917810** does not explicitly capture its incoming ECX in its body; **917980** preserves an outer receiver but still lacks a reaching construction/registration proof. Warning text, neighboring function addresses, compatible fields and inherited formatting methods remain insufficient receiver evidence.

The original PE has **no COFF symbol table and a zero export directory**. Its exact RSDS record identifies **SFrame.pdb**, GUID **1a63d302-5f70-474c-af94-63d0436732a7**, age **1**, under **Source9.5.2/client/Game/output/Release_ME**. All **723 inventoried local MSF7 PDBs** were parsed for GUID and age; none matches this original record. A reconstructed authserver EXE/PDB pair serves as a successful parser control. The inventory is bounded to the audited workspace and its stated exclusions; filenames alone are not symbol authentication.

The **POGO GCTL** payload is completely decoded: **53 contributions / 1,052 consumed bytes**. Both **918630 (RVA 0x518630)** and **8a19f0 (RVA 0x4a19f0)** fall in the generic **.text$mn** contribution, RVA **0x1b7dc0**, size **0x71c2f0**, VA **[0x5b7dc0,0xcd40b0)**. None of the contribution names attributes a method/class to 918630. The embedded PDB path remains build-lineage evidence, not a recovered source method, a receiver proof or an official Epic version.

## Continuing the alternate-mode investigation

The original positive projection is unchanged: **918630 captures entry ECX in EDI at 91864d**, forms **EDI+0x454 at 918720**, and calls **8fb010 at 918726**. This yields **byte outer+0x458=1** and **DWORD outer+0x45c=0**. It is a real write for a reaching compatible receiver. It is **not yet a write proven to reach constructed MAIN**.

| Required proof | Current status |
|---|---|
| A real direct, computed or registered transfer into 918630 | No confirmed reaching edge in the retained bounded observations. |
| Entry ECX is the constructed MAIN object or a source-backed adjustment to it | Unknown; RTTI common bases and field compatibility do not establish it. |
| A nonzero write to that same MAIN instance after its constructor/visibility clears | Not established. The new indexed allocation paths do not supply one. |
| The UI/message/action condition causing that write | Unknown; localized warning conditions are not a Main activation trigger. |

The earlier empty incoming query and bounded code/data/relative/module inventories are reused with their original limits, without replaying them as new progress. New Native bodies b35070/b38220 contain no observed direct CALL/data-reference/immediate target into 918630. Their indirect/transitive effects, later MAIN+0x328 producer and arbitrary runtime mutations remain unresolved. No activation is implemented or inferred from these gaps.

## Evidence and verification

All new records bind the original target SHA-256 and profile digest **`b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17`**. All new Native value-flow graphs report no truncation. Complete retained records, selected original-byte checks and limits remain distinct from complete runtime execution or interprocedural effects.

| New Native entry | Analyzed bytes | Evidence ID |
|---|---:|---|
| 0xb35070 | 526 | `ev_36abc928b928fda18fd4d45f80b6094ea5de35641e5c7b649b1fcc45cd005f18` |
| 0xb38220 | 520 | `ev_7a4a0a55c1c4240240d3ff1e6842ef4c2f77c8d3ef65cf6d24281f4d72bb507b` |

| Selected reused Native entry | Evidence ID |
|---|---|
| 0x8208d0 | `ev_2936d3aaf895e308c7373afcd56552ceab65af89dc7dc8ea3a6c3e04c83e0996` |
| 0x820f70 | `ev_bf6a01b950ce19ea40f7d2a5709725e5431a157e569999caec2f85e087de96e0` |
| 0x8a19f0 | `ev_f3a4ff37e3949c13ed452102ee534729623efb381bd564fe0da8330bbded3cd2` |
| 0x8a1ab0 | `ev_6d832c9d0196d2b1898603d93fd01bcd02fc3ba1df5750e51a235611f561c24c` |
| 0x8a1d40 | `ev_10e965f99af0876e0d108b0ba4bfd7820f456137fa2b9dfde61ae54ea385f824` |
| 0x8a1dd0 | `ev_0c7097265f16bc7a09c0d3f3f17e1b34cc3261cd3e00af9dbe41055da9d6ee1e` |
| 0x8a3500 | `ev_a4b453f0e7c50f8457c67db0e7c4379d6645e67044af83a62c0e95472cc3a20c` |
| 0x8faf60 | `ev_b68b9eaac95cf0f480f057f291993e443d9e54cfc10c7bbed8ae2f8beef7cbe3` |
| 0x8faf70 | `ev_2438ea2ed528ccdf2c4e457359892f32d22dab8e4567ce96ac284282fe718e33` |
| 0x8fafa0 | `ev_da809073bd3cb75b6fe68756f7a1fe446bf0150b6c6bf804bd8db6c5b34c4433` |
| 0x8fb010 | `ev_6311d6edbba44308e24bffdc741ec9c6fbccbb16ca26c14b03969b080907bce5` |
| 0x916550 | `ev_596c3d917c90a0709cbb04e922e15feda7569b25cb26d777229825c4cde8e735` |
| 0x916a60 | `ev_008fb1f8d774e53f8e18a90c323e9867ddeeadab0024d0854c2e4dbdaf70e12d` |
| 0x916c20 | `ev_f798fb6a88410dc78b49df4cd35c5c220e4e99768fdc15de5da1adfaa5039e62` |
| 0x917810 | `ev_85e0d1e5f82d8e0bbd61096a80d3e3c93a6b604c1d0a74ee0bae63959df29b08` |
| 0x917980 | `ev_6f667ccfb3e130926af3c52d22c1e224e43b001a72658002aaf68a586c4e976c` |
| 0x917aa0 | `ev_5c5e2e76bb180562efa78127363e7cf4c687dc112ad925f6c515dde17249fb27` |
| 0x917f00 | `ev_f8164952ca2b6eca04668d8cdf93d49dd7341df5d04ba07cca54d9fdcf5d576d` |
| 0x918630 | `ev_2e11babc974a7b4241ded6116bc72be4cbafa3d434f034fc40c37e79a04da7bc` |
| 0x9a9480 | `ev_aafca74ab3458f28bca382e818536522c0d34a0730877fe2a704094a88f384d7` |
| 0xb35b60 | `ev_780cd463050f07ecb6fc2124376fbcb2e95dd2a03b4f84435dcbface76076ff5` |
| 0xb35d90 | `ev_5cb2886f4bc408c96a914bdfa775c377c41a9598aa01e14a748d9e0a310be6e9` |
| 0xb36a30 | `ev_7b843fcf59711867a7cd81807a3bef5a97a6d5ea05b0a5da783ce97408935482` |
| 0xb36cb0 | `ev_d1c78eb1e366ec54558ca555fc7916d89244e357a120a72e91d7234db3b3dd84` |
| 0xb386d0 | `ev_d95c69afe13991fd427fc19cfe36fd3570473a545c2cf7ec7417f6fc1c9883e2` |
| 0xb3d4f0 | `ev_ad8725bbe05f2378247b79f6c19fb6a39e2b4122eebe575c6ae63730f4a888d5` |

Independent checks bind every new Native body to original file bytes, verify **249 new assembly boundaries**, **176 selected provenance intervals**, and **26 separately retained Native bodies**. The three indexed-destination, receiver and symbol ledgers preserve their own hashes and limits. Overlapping byte checks are corroboration, not separate game executions.

The owned Native session checkpoints **2 new observations**, officially imports the preceding **622-record canonical bundle**, exports and closes. All preceding canonical records remain equivalent as JSON values. The final bundle holds **624 records / 284,299,686 bytes**. The closed snapshot holds **23 cached primitives / 295,765,127 bytes**, SHA-256 **`6e6576e2de3d387c4514e5f39d156566dc52e13a7585fb6fe6727838280ea0ae`**. The historical Native index retains complete older records outside this canonical subset.

Ignored local run: `client-data/audit/2026-11-01-sframe-main-trigger-provenance-c2d7e8a5`. Complete records, original-byte/resource provenance, three ledgers, checkpoint, snapshot, preservation and publication receipts stay local.

**Nine preservation checks pass.** Eleven baseline executable, addon and settings hashes, the saved-variable tree, logical database backup and primary service identities remain unchanged. The root Git index stays empty. Only seven authored Markdown files are synchronized through isolated Core/Client worktrees; original assets/binaries, databases, credentials, settings and generated decompiler output are not published. Gameplay code and package versions are unchanged.

## Restart point

The requested object identity and activation trigger remain unresolved. Continue from a concrete reaching target plus ECX provenance for 918630, or a typed MAIN pointer reaching a nonzero mode write. The newly explicit static boundary is the later producer of the rectangle-vector header at MAIN+0x328, separately from the now-qualified renderer allocation at +0x3c8; do not invent its alias or redo the old broad address sweeps. If a future controlled original-runtime investigation becomes available, an entry trace must retain the caller, ECX, vptr and object lifetime, while a MAIN-byte watchpoint must be tied to its actual constructor/registry instance. A forced call with an analyst-selected object would not identify the original receiver. No original client is executed in this pass.

## Follow-up: MAIN rectangle producer, 2026-11-01

The [rectangle-producer investigation](SFRAME_MAIN_RECT_PRODUCER_RE.md) identifies the previously open MAIN+0x328 producer and checks retained interior-entry candidates. The actual MAIN+0x328 producer is identified from the typed post-load method through titlebar geometry, setter, vector growth and allocation, with its selected custom-move input consumer. Descriptor/style mapping separates MOVE_BY_CUSTOM 0x10000 from NOFRAME 0x100. The 87 retained coincidence rows and exact 255-position REL8 range add no caller of 918630; whole-body incoming coverage is confirmed. Its receiver and alternate-mode trigger remain unidentified. Data, settings, services and versions remain preserved.
