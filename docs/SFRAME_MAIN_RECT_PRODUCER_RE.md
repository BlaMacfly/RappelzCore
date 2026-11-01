# SFrame MAIN rectangle-vector producer and interior-entry checks

Verified **2026-11-01** on original x86 SFrame SHA-256 **`27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`**, using **REA 6.4.0 / Ghidra 12.1.4 / Capstone 5.0.7**. This follows [the MAIN provenance investigation](SFRAME_MAIN_TRIGGER_PROVENANCE_RE.md). **The actual producer of MAIN+0x328 is identified: MAIN's post-load method supplies its titlebar rectangle to b377a0 and the 9942d0/997330 vector-growth path. The vector also participates in the selected custom-move geometry test. No new caller or typed receiver of 918630, and no character-sheet alternate-mode trigger, is established.** New evidence: **8 Native dossiers / 5,512 analyzed bytes** and **1 other observations**. These are static artifact observations; the original client is not executed.

[Versions](VERSIONS.md) stay Core/RappelzRust **0.1.0 development**, NoLimits launcher **1.3.60**, original SFrame PE file/product **9.1.0.0**, embedded **Source9.5.2** build lineage, and network literal/profile **201507080**. Exact official Epic/patch remains unconfirmed.

## The rectangle producer on constructed MAIN

For constructed **SUICharacterInfoWnd / window ID 0x2e**, exact vptr **d53af8+0xd4 selects 8a1c20**. Its retained 49-byte body preserves the MAIN receiver, finds the named **`titlebar`** control through b3c660, obtains its geometry through **722270**, reloads **ECX=MAIN at 8a1c38** and calls **b377a0 at 8a1c3a**. This is an actual source-to-owner path; it does not rely on neighboring function addresses or a matching field offset.

Retained **b377a0** addresses **H=MAIN+0x328**, whose three DWORDs are begin/end/capacity. The selected branches are distinct: if nonempty, replace the first 16-byte rectangle without changing the header; if empty with spare capacity, copy a rectangle at end and advance end by 0x10; if empty/full, deliver **ECX=H**, insertion position and source-rectangle pointer to new Native **9942d0**. With no intervening header mutation, the zero state from the previously verified **b35070** constructor reaches the growth path at the selected first post-load insertion. Intervening mutation remains a condition of this initialization proof.

New Native **9942d0** calculates insertion offset and new size/capacity in **16-byte elements**, allocates through new **997330**, copies the incoming four DWORDs and retained old elements, releases old storage under its inspected contract, and writes **H+0/H+4/H+8 at 9943ec/9943f5/9943f8**. New **997330** converts element capacity to bytes with a shift by four; capacity 1 follows its small-allocation path for **16 bytes**. For the valid initially empty header, the successful selected path ends with one rectangle and one-element capacity. Larger allocations have separate alignment/overflow paths; allocator failures, malformed headers, aliases and mutations are not discharged globally.

This is a rectangle payload copy into vector storage, not a clone of MAIN. It closes the producer boundary left open in [the preceding report](SFRAME_MAIN_TRIGGER_PROVENANCE_RE.md). The separate renderer pointer at **MAIN+0x3c8** and its **0x144-byte primitives** remain distinct. MAIN+0x328 is an analytical offset label; **`SourceRectList` is not presented as a recovered property or source field name**.

## Descriptor, style and input-consumer path

The exact archived/cached **MAIN NUI** is revalidated: **8,871 bytes / 48 blocks** in this one resource, comprising one genwnd root, 37 static, two simplebutton, three gauge, one tab_sheet, two tab_head and two iconstatic blocks. Its root rectangle **(0,0,476,529)** feeds the root geometry path. The separate static child **titlebar**, rectangle **(1,2,474,27)** in the default resource, feeds child geometry **+0x108**, which 722270 supplies to b377a0. These are resource/default values and a constructed lookup path, not observed live window coordinates. The visible-label control **text_01**, code **6093 / 'Personnage'**, is a separate object; the titlebar's own caption is markup.

The selected property registration in retained **b39f10** uses **0x2e=rect**, **0x2f=pos**, **0x30=caption**, **0x32=style**. New **b3a960** selects the property ID/state; new **b39c00** consumes the selected value. The rectangle path populates **descriptor+0x94**, while style parsing calls new **b3aa50** and accumulates into **descriptor+0xa4**. Retained b3c510 supplies the object-side rectangle and style fields. Parser-relative bases and object-relative bases are kept separate; property ID 0x2e is not equated with window ID 0x2e merely because the numbers match.

The original CRT contribution and startup slot **d2818c→5b4980** select a separately bounded initializer wrapper calling new **b3ab30**. Its selected table pairs bind **KSTYLE_MOVE_BY_CUSTOM=0x10000** and **KSTYLE_NOFRAME=0x100**. New **b3b5e0** transfers the 35-pair, 0x1c-stride input table toward the selected style registry **edd564**; new b3aa50 looks up the selected entry and returns its value. Only the two needed mappings and the table handoff are claimed here. The insertion helpers/comparator, allocation contracts and possible later table replacement retain their limits. The complete b3ab30 body is preserved despite its truncated high-pcode value-flow graph.

Thus the original MAIN style token **MOVE_BY_CUSTOM** is distinct from the earlier **NOFRAME bit 0x100** guard; the two are not substituted for one another. Retained MAIN **virtual+0x58→821140**, with preserved MAIN passed at **8213bf/8213ca**, reaches new **b36500**. Under its **0x10000 style test**, it traverses the 16-byte rectangles from H, invokes the selected element/point-test helpers and can set **byte MAIN+0x31d** when its input/geometry predicates pass. This supplies a selected custom-move geometry/hit-test consumer. It is not a write to the distinct alternate-mode byte **+0x458**. Input guards, valid resources/owners, allocations, mutable headers and indirect/transitive effects remain conditional; no original gameplay or dragging was executed.

## Continuing the alternate-mode investigation

The search for incoming routes reuses the earlier **87 CODE/interior coincidence rows / 48 distinct DWORD slots** rather than running the old image sweep again. Thirty rows are internal to 918630 and 57 external. The two apparent E8 cases inside the body are bytes inside the MOV starting at **918656**, not fresh instruction boundaries. The real external CALLs **9cd58f / 9cd619** belong to retained **9cb4e0** and target **5bec70**. A new exact instruction probe confirms **b34586→5bec40**, within provider procedure b34070. These calls do not enter 918630. Unassigned source procedures for other numeric coincidences are not invented or typed as MAIN.

A separate **REL8** check is strictly bounded to possible opcode addresses **[L−129,H+126]=[0x9185af,0x9187da]** for target body **[L,H]=[0x918630,0x91875c]**. Across **255 exterior opcode positions**, ordinary short JMP/Jcc and LOOP/JCXZ encodings yield **zero targets in the body**, even before instruction-start validation. A 560-byte original capture and signed-displacement extreme controls preserve the exact range; this is a local encoding result, not a whole-program runtime reachability proof.

The installed **REA 6.4.0 ReaGhidraBridge.java** implementation is independently checked: incoming collection constructs a scope from **the whole function body plus entry**, iterates reference destinations in that scope, and reads their references; analyzeFunction separately excludes self-body sources and explicit entrypoint markers. Therefore the retained empty incoming result is not reinterpreted as an entry-only query. No duplicate caller query is counted as progress.

**918630 still receives an unbound entry ECX, saved in EDI, and its 8fb010 call projects byte outer+0x458=1 and DWORD outer+0x45c=0.** None of the new rectangle/parser paths or entry-encoding checks supplies the reaching MAIN receiver or the causal alternate-mode trigger. Dynamic targets, runtime-installed references, unmodeled encodings, arbitrary aliases and transitive effects remain unresolved. No activation is reconstructed from these gaps.

## Evidence and verification

All new records bind the original target SHA-256 and profile digest **`b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17`**. New graph truncation remains explicit: 0xb3ab30 (1 known omitted edges; omitted-operation lower bound 1). Reused graphs retain truncation: 0x9cb4e0 (0 known omitted edges; omitted-operation lower bound 1). Complete retained records, selected original-byte checks and limits remain distinct from complete runtime execution or interprocedural effects.

| New Native entry | Analyzed bytes | Evidence ID |
|---|---:|---|
| 0x9942d0 | 323 | `ev_034106ed856998e15badf92e17fb363050f0126a7b8f3a86c1823e441736ea2b` |
| 0xb39c00 | 571 | `ev_092e7b4ab5988b2f407f022115332d5d06acbd7cc4a88db22e64571b3d4fc264` |
| 0xb3aa50 | 221 | `ev_78f30310673e8b1603ec00b7aef30f9caba2ed9580a211ac13999494e4c61d5e` |
| 0xb3ab30 | 2723 | `ev_04182e64acff441675f01958e4bf312abda8e5e4a436b4ad9784e30cb9a15fc2` |
| 0xb3a960 | 79 | `ev_7a36794506b9fd5cbda44cc4efcc816e78b6978148d863d3e506c8b7f910883e` |
| 0xb3b5e0 | 363 | `ev_fd7b66db9f5aedef61f8ceada7eee90d71780961123106b1f63383a70912ad14` |
| 0xb36500 | 1134 | `ev_8a454b9675cc6a2451c3107adc5d92676324e6cb44ba4942d296b90de2823a13` |
| 0x997330 | 98 | `ev_2ddc068b8bb860e02730c7f9ea4edffe9db59fce5cb1e899543ba39352d21504` |

| Other operation | Parameters | Evidence ID |
|---|---|---|
| inspect_native_instruction | `{"address": "0xb34586"}` | `ev_d3d095101ec68b83374a4fba23c0ef70df7cd66648f5f8dba4afaa416ba82807` |

| Selected reused Native entry | Evidence ID |
|---|---|
| 0x722270 | `ev_aa161205dcef806f5b9b3434ddc52fb69da13c6ce4614059ad175a72880e02ad` |
| 0x8208d0 | `ev_2936d3aaf895e308c7373afcd56552ceab65af89dc7dc8ea3a6c3e04c83e0996` |
| 0x820f70 | `ev_bf6a01b950ce19ea40f7d2a5709725e5431a157e569999caec2f85e087de96e0` |
| 0x821140 | `ev_96d1a02e467350902a74b677aee36d55e4c5c3a781822d97f6c1a25aa36a649a` |
| 0x8a19f0 | `ev_f3a4ff37e3949c13ed452102ee534729623efb381bd564fe0da8330bbded3cd2` |
| 0x8a1ab0 | `ev_6d832c9d0196d2b1898603d93fd01bcd02fc3ba1df5750e51a235611f561c24c` |
| 0x8a1c20 | `ev_555e48d8e2d1429350dcb3ae8fe9855873fdb9f23d6e1fc0036fb0a1baf291b4` |
| 0x8fb010 | `ev_6311d6edbba44308e24bffdc741ec9c6fbccbb16ca26c14b03969b080907bce5` |
| 0x917500 | `ev_30bc9b7b18b248bd3855f1f1c933d9f89ac652d78cc43105cecedda6548e1ee1` |
| 0x917810 | `ev_85e0d1e5f82d8e0bbd61096a80d3e3c93a6b604c1d0a74ee0bae63959df29b08` |
| 0x918580 | `ev_aa32599fddde90584680130bd4bf1522a4f5e70355b17bdf6e60dc8702668644` |
| 0x918630 | `ev_2e11babc974a7b4241ded6116bc72be4cbafa3d434f034fc40c37e79a04da7bc` |
| 0x9a5170 | `ev_ee83ed194b19c037d963a597ac72ac4ac55c2c284cd2f407cca0946112ec93fb` |
| 0x9cb4e0 | `ev_4275b7d64861bda9d3f1369468c58deec344a973e3c6e12679479663f89ad441` |
| 0xb35070 | `ev_36abc928b928fda18fd4d45f80b6094ea5de35641e5c7b649b1fcc45cd005f18` |
| 0xb35280 | `ev_06bd0a6292706c32da0e795d0e78bc9b0105bf346fa8cc5ce0ee7129282eb707` |
| 0xb35b60 | `ev_780cd463050f07ecb6fc2124376fbcb2e95dd2a03b4f84435dcbface76076ff5` |
| 0xb35d90 | `ev_5cb2886f4bc408c96a914bdfa775c377c41a9598aa01e14a748d9e0a310be6e9` |
| 0xb36a30 | `ev_7b843fcf59711867a7cd81807a3bef5a97a6d5ea05b0a5da783ce97408935482` |
| 0xb377a0 | `ev_c2bb0efb0f52587df1ec388c291b55150d5dd0486fd5b9e36245d726f224d138` |
| 0xb399c0 | `ev_d3736f3cd834ccd4999dabd26324fe9acc18b95f4d2928e41414494c1a56fca0` |
| 0xb39f10 | `ev_08308fa8f2298a9f0c161fabeb69a66e276404c26b7c988fa7a8ad1069cd61ba` |
| 0xb3a2b0 | `ev_115b5e512d1258979bba2360b48cfe1e0ce9173de33bf0dfa48f42bad1ffedf9` |
| 0xb3a790 | `ev_b574848ac50c819b06308d1097fc3254d7764106d21b3f25ad22fcd65eed77dc` |
| 0xb3c510 | `ev_6356bc8bbb36f8a8053edebbc871fdb724d632f03e1057a807c86f753eaca43d` |
| 0xb3c660 | `ev_666b2d425f99f55ebaefa73584a86da66c18f086c8e8466d973a26499d848392` |
| 0xb3ead0 | `ev_5f57d5eeeef5a105042b0ceec3f79df05293b720c9fe47ae03fe6cc106e0f02f` |
| 0xb3ec90 | `ev_ddc5747a3d5b8e234861260a575d82bcbd6db31a663dc92f6dd55a7ae6e2c860` |
| 0xb670e0 | `ev_196628d23ba31ba6929cb63704a4344641b77d5c7cdb6d9bd4ca98e1ee54ad70` |

Independent checks bind every new Native body to original file bytes, verify **1,397 new assembly boundaries**, **218 selected provenance intervals**, and **32 separately retained Native bodies**. The three producer, descriptor and interior-entry ledgers preserve their own hashes and limits. Overlapping byte checks are corroboration, not separate game executions.

The owned Native session checkpoints **9 new observations**, officially imports the preceding **624-record canonical bundle**, exports and closes. All preceding canonical records remain equivalent as JSON values. The final bundle holds **633 records / 290,304,818 bytes**. The closed snapshot holds **29 cached primitives / 307,067,289 bytes**, SHA-256 **`fea3bc0d0a66e5621354fd0b7eadd479730348b14d729167e7a1ae77a3bc2518`**. The historical Native index retains complete older records outside this canonical subset.

Ignored local run: `client-data/audit/2026-11-01-sframe-main-rect-producer-3b2a7e90`. Complete records, original-byte/resource provenance, three ledgers, checkpoint, snapshot, preservation and publication receipts stay local.

**Nine preservation checks pass.** Eleven baseline executable, addon and settings hashes, the saved-variable tree, logical database backup and primary service identities remain unchanged. The root Git index stays empty. Only seven authored Markdown files are synchronized through isolated Core/Client worktrees; original assets/binaries, databases, credentials, settings and generated decompiler output are not published. Gameplay code and package versions are unchanged.

## Restart point

The MAIN+0x328 producer is now answered on the selected constructor/post-load path: titlebar geometry→b377a0→9942d0→997330→header writeback. The style/input consumer and resource distinction are source-backed. Do not carry the old 'producer unknown' boundary forward. The remaining objective is an actual reaching transfer into 918630 with entry ECX tied to constructed MAIN, or a different typed MAIN nonzero mode write and its trigger. Reuse the new records and the interior ledger; do not repeat the old entry/body query, 87-row classification, local REL8 range or symbol/PDB inventory as new discoveries. No original client was executed.
