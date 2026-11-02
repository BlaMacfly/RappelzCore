# SFrame MAIN mode readers and retained rendering aliases

Verified **2026-11-02** on original x86 SFrame SHA-256 **`27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`**, using **REA 6.4.0 / Ghidra 12.1.4 / Capstone 5.0.7**. This follows [the MAIN rectangle producer](SFRAME_MAIN_RECT_PRODUCER_RE.md) and refines [the actual MAIN pointer flow](SFRAME_MAIN_POINTER_FLOW_RE.md). **Mode reads have branch-specific effects; the actor lookup and data accessors are qualified, and a rendering primitive really retains MAIN+0x118. No typed receiver or new caller of 918630, and no causal MAIN alternate-mode activation, is established.** This pass adds **13 Native dossiers / 476 analyzed bytes**. The reused MAIN reader bodies are explicitly reused, not counted as new decompilations. This is static artifact analysis; no original client execution is claimed.

[Versions](VERSIONS.md) remain Core/RappelzRust **0.1.0 development**, NoLimits launcher **1.3.60**, SFrame PE file/product **9.1.0.0**, embedded **Source9.5.2** lineage, and network literal/profile **201507080**. Exact official Epic/patch remains unconfirmed.

## Four mode tests, distinct effects

The four Main+0x458 tests affect selected internal messages, not every update. Hex kinds 0x36, 0x31 and 0x406 choose the actor-oriented 8a30f0 refresh when the byte is nonzero; the 0x9/0x4c7 path only suppresses those input HP/MP setters. Other cache and 0x1f bank branches remain separately selected. Internal 0x31 (49 decimal) is distinct from 0x1f (31 decimal); neither is promoted into a new network opcode.

## From a selected handle to actor data

8a30f0 reads Main+0x45c, falling back to [Main+0x424]+0x208 only when that retained handle is zero. It requires a nonzero handle, a current world, a successful object lookup and nonnull object+0x36c data before retaining the handle and refreshing. The conditional SGameWorld slot +0xd0 selects a3b460, which forwards through world+0x3a4 to 6ad740. That wrapper rejects a nonzero manager+0x120 byte or a zero key, initializes its local output DWORD to zero, and calls 6b6440 on manager+0x20. The latter selects a hash bucket and performs a lower-bound search on unsigned node+4 keys. An exact match writes DWORD[node+0] into the output argument and returns AL=1; a miss returns AL=0 without replacing the initialized output. 6ad740 then returns the output DWORD as the object pointer, independently of the helper success byte. No fallback to +0x208 is retried after a failed nonzero Main+0x45c lookup, and no player-only or self!=target check is present in the selected refresh.

The saved object pointer is restored for each selected accessor. With D = [object+0x36c], seven numeric wrappers return (stored[D+offset] + 2*D + constant) modulo 2^32, with explicit null-data sentinels. The name wrapper selects inline or heap storage for the string at D+4, or the literal UnLoad Avatar when D is null. The final wrapper returns an opaque DWORD at D+0xec to Added-panel code. These are internal memory read contracts, not raw wire layouts or recovered server stat formulas.

The new accessor bodies load **D=[actor+0x36c]**. For nonnull D, each selected numeric result is **(DWORD[D+offset]+2×D+constant) mod 2³²**. The constants belong to this storage decoding contract; they are not gameplay bonuses.

| Entry | Data offset | Added constant | Result if D is null |
|---|---|---|---|
| 954480 | 0x30 | 0x784 | 0xffffffff |
| 9546a0 | 0x48 | 0x23a0 | 0xffffffff |
| 9546c0 | 0x44 | 0xa7 | 0 |
| 9546e0 | 0x4c | 0x16d | 0 |
| 954700 | 0x34 | 0x1b96 | 0xffffffff |
| 9547d0 | 0x38 | 0x82 | 0xffffffff |
| 9547f0 | 0x3c | 0x94 | 0xffffffff |

New 954950 returns the string at D+4 with its inline/heap choice based on the capacity DWORD at D+0x18, or the fixed pointer d60778 if D is null. New 954ed0 returns DWORD[D+0xec], or zero for null D. All nine use entry ECX as the actor receiver and return without explicit argument cleanup. These raw contracts preserve the distinction between actor and actor-data pointers.

The constructed Basic child selects a 16-byte block at object data+0x80 and an 82-byte block at object data+0x90 only under its parent-mode, world, retained-handle and data guards. Failed guards use its own cached player-state banks. The child lookup uses Main+0x45c without implementing the +0x208 fallback itself. The explicit actor refresh creates zero-filled local 16-byte and 82-byte argument blocks for the contribution renderers and displays dashes for EXP/JP; these are UI call inputs, not writes clearing actor data or resetting its stats. Repeated child lookups and other ungated messages do not establish an atomic global mode snapshot or a privacy policy.

Static reader semantics identify an actor-oriented display branch, but they do not establish what activates it, which object reaches 918630, or any server disclosure permission. The selected SGameWorld virtual target remains conditional on that actual dynamic-world path. Container invariants, borrowed object lifetime, mutable aliases and interprocedural effects remain open. The reused 89b7e0 and 89d9b0 value-flow graphs are each truncated, with zero known omitted edges and an omitted-operation lower bound of one; selected original instructions are verified independently.

## An actual persistent MAIN interior pointer

A new real escape is qualified outside the previously closed MAIN+0x328 buffer. Original MAIN virtual+0x24 selects b363f0. It passes the address MAIN+0x118 to a8d370 at b3641b and b36450, then has a third tail handoff at b36481. New Native a8d370 is exactly 20 bytes: it loads physical argument1, stores that pointer at [entryECX+0x0c], sets byte [entryECX+0x141]=1 and returns with RET4. It retains an interior MAIN pointer; it does not copy a 16-byte rectangle or write MAIN+0x458 in this body.

The first two receiver expressions are [MAIN+0x3c8]+i*0x144 and [MAIN+0x3d4]+i*0x144, under their actual count guards. The third receiver is the separate object pointer [MAIN+0x3d8]. On the selected MAIN virtual+8 initialization path, b35d90 conditionally allocates 0x144 bytes, calls retained constructor a8cfd0(1), and stores its result at MAIN+0x3d8. That constructor installs vptr d7f910; its exact COL/type descriptor identifies KSpritePrimitive, and its +0x0c initially holds zero. Thus this third constructed receiver has an actual type binding. The array receivers are not assigned that type from their equal stride alone.

The selected MAIN virtual+0x14 method b376a0 passes those primitive pointers to virtual+0x28 of its first physical argument, under the visibility guard. This is the next concrete consumer handoff. The rendering-context argument has no constructor/vptr/target binding in this ledger. No later read or write through the saved primitive+0x0c pointer, no reconstruction of MAIN from it, and no call into 918630 are established. A nearby primitive method or a compatible field offset cannot supply that missing consumer.

The capture is conditional static reachability: the selected MAIN instance, successful allocations, valid array counts and storage, callee-saved ESI ABI, pointer lifetime and unchanged vptrs/owners are assumptions. The saved pointer may outlive MAIN or be replaced. The old 59-slot callout inventory and its state caps are reused, not promoted into an exhaustive escape proof. MAIN+0x448 is a value copied from the factory context; [MAIN+0x450] is a separate Added object pointer, not an embedded MAIN subobject. Existing caption, parent and manager investigations are reused. No MAIN nonzero mode writer or actual target/receiver for 918630 is identified; no additional Native target is justified before the rendering-context or another actual saved-pointer consumer is bound.

This qualifies a persistent alias omitted from a receiver-only reading of the earlier callout audit. It is not evidence that the stored pointer reaches MAIN+0x458 or invokes 918630. The rectangle vector at MAIN+0x328 and the primitive-retained pointer to MAIN+0x118 have separate storage and contracts.

## What remains unresolved

Retained **918630** still saves an unbound entry ECX in EDI. Its **918726→8fb010** call projects **byte outer+0x458=1 / DWORD outer+0x45c=0**, but no reaching definition binds that outer receiver to the constructed SUICharacterInfoWnd. Existing zero initialization/visibility clears and these mode readers do not supply the activation event. A matching offset is insufficient to identify an object.

The previous incoming-reference result, interior-entry checks, pointer/address scans, symbol audit and MAIN+0x328 producer are reused. No repeated empty caller query, missing-PDB request, synthetic activation or runtime observation is counted as progress. Callback installation, unresolved indirect receiver dispatch, arbitrary aliases and transitive writes retain their limits.

## Evidence and verification

All records bind the target SHA-256 above and analysis profile **`b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17`**. New value-flow graph truncation: none. Reused value-flow graph truncation: 0x89b7e0 (0 known omitted edges; omitted-operation lower bound 1), 0x89d9b0 (0 known omitted edges; omitted-operation lower bound 1), 0x9cb4e0 (0 known omitted edges; omitted-operation lower bound 1). Complete body bytes and retained Evidence records do not imply complete high-pcode graphs or whole-program effects.

| New Native entry | Analyzed bytes | Evidence ID |
|---|---:|---|
| 0xa3b460 | 15 | `ev_077278bf961f2758c14630014a4bb55987935c8941c86811f4776bd553cefc12` |
| 0x954480 | 26 | `ev_460a38e8c19dd7ca9fcaff27492eb4b3cd9c4bbfd9cbf2bf1fe32576a2f7ebfe` |
| 0x9546a0 | 26 | `ev_fc3a3594292fb80a89af890d30641bef7a197cfb90002cbc2669a9f08c9c5337` |
| 0x9546c0 | 25 | `ev_7b589af134c68822ed0980e8caffccff059671175f8ee1527ba281709e620453` |
| 0x9546e0 | 25 | `ev_a24417be55b7a3af380dcbbf42b5b6ae2d7f6cc13c82d4457e66777ddca51545` |
| 0x954700 | 26 | `ev_d731c236735e321c6d48edf773e923517707c4b63f436dc3075b04f6c3908edb` |
| 0x9547d0 | 26 | `ev_79fac1bdaee86857a259fb59a5570f4f273aa57052be558d27bb2ecc79c4c63c` |
| 0x9547f0 | 26 | `ev_a00fc9defcd028ece5eb31741ed6ce3d879511e14e96f5f76605f577b9ae28b0` |
| 0x954950 | 28 | `ev_ddd574c26c1722b482875b1091d5f0b42f2e407f6e064db3b3a226311c1e8a7a` |
| 0x954ed0 | 20 | `ev_d963ebfb58b02287e551986b79bd02b90eed7bcda4b0b1866d872972e9766f0c` |
| 0x6ad740 | 59 | `ev_4064294da0c100daa767c522f9fb1079c9a9297b16d1defeacf972a0527a72e2` |
| 0xa8d370 | 20 | `ev_83932bb15621cc2b6979b3e7970d570883e86589796b140d84e329518b8259e3` |
| 0x6b6440 | 154 | `ev_68296f0335579c55ac6ce0450b2937c5256b64bba8474ef3a592bb874438e12e` |

| Selected reused Native entry | Evidence ID |
|---|---|
| 0x722270 | `ev_aa161205dcef806f5b9b3434ddc52fb69da13c6ce4614059ad175a72880e02ad` |
| 0x89b7e0 | `ev_7473b0829fc541429e384401828478f30489f08f95d67d578d1f85e337b40771` |
| 0x89d9b0 | `ev_123d66412e1f03dfd14417cc7ed2ad8cb2a27987c0cfd3a694114a66f9611b7f` |
| 0x8a19f0 | `ev_f3a4ff37e3949c13ed452102ee534729623efb381bd564fe0da8330bbded3cd2` |
| 0x8a1ab0 | `ev_6d832c9d0196d2b1898603d93fd01bcd02fc3ba1df5750e51a235611f561c24c` |
| 0x8a1c10 | `ev_849e554ef2e3225eef34ad284e75a062591ffc3ec4f63338b8dfbbe1e59ee72b` |
| 0x8a1d30 | `ev_3cfd8199a45a65e80dd1d832beb282b912162f8c06beeac5a98a9a30e231812e` |
| 0x8a1d40 | `ev_10e965f99af0876e0d108b0ba4bfd7820f456137fa2b9dfde61ae54ea385f824` |
| 0x8a1dd0 | `ev_0c7097265f16bc7a09c0d3f3f17e1b34cc3261cd3e00af9dbe41055da9d6ee1e` |
| 0x8a25f0 | `ev_284a3a99269481f6027352d3f98d133b6b38cb516dce5c2df16ddf02ab102153` |
| 0x8a27b0 | `ev_af38fa54230f2ee50372d98ec41edfab7044004abe7baa35772aaa0365a6c269` |
| 0x8a2a70 | `ev_0f8c16b15b0d7f885d82b781b7c8d401adc5b96b8b94c5e1fb03d00e6ab32433` |
| 0x8a2cf0 | `ev_e309645c4b48f2c4ce1257ad9fe2aabcab671a8d40666bae9b4fb92a6e5dcd54` |
| 0x8a2f80 | `ev_b7f0e11f4096b0dbea81ab2fbd794a88c0bb7b480e0e89bfb90de5d0b47871b8` |
| 0x8a30f0 | `ev_295a9ba6abb1926edf4a6b5e4cc827f166f4a84d714ca3efa0c8d65301b94b1a` |
| 0x9cb4e0 | `ev_4275b7d64861bda9d3f1369468c58deec344a973e3c6e12679479663f89ad441` |
| 0x9ee690 | `ev_143f87162b40cbf34da5ba2c9db9448e158f72b5448081ac747dc5ac9a1b7573` |
| 0xa8cfd0 | `ev_3b64e6e3f9bf7cf46f971bc223ef48c6d68de12071c462fa9d6cefd7d1dbf49d` |
| 0xb35d90 | `ev_5cb2886f4bc408c96a914bdfa775c377c41a9598aa01e14a748d9e0a310be6e9` |
| 0xb363f0 | `ev_6f4e0a17cd43ad32da91d644e421a212ea02a4a1427e10dde52f78c9aa6752a9` |
| 0xb376a0 | `ev_10d7bfd5420317fd7d23d357bf1455a5e746ddac4cb283d31e4b96f022822561` |
| 0xb37d90 | `ev_31d29c39cedaa7da1b61741a5e5538d7da58c0f5e0d7797243b29495429e10e3` |
| 0xb3d9e0 | `ev_390fd092447e2b311ba69d0267e43c7383739030c06d243b2095e9e78977387e` |
| 0xb402a0 | `ev_5b4ce74ae2be7bf8b7c03aa2b155bf6ed5d5d28ebf85964e05c061ce6c353005` |

Independent checks bind all new Native bodies to original file bytes, verify **184 new assembly boundaries**, **393 selected provenance intervals**, and **30 reused Native bodies**. Both reader and escape ledgers preserve source-file hashes, original-byte provenance, selected ABI paths and limits. Overlapping checks are corroboration, not separate game sessions.

The owned Native session checkpoints **13 new observations** and imports the preceding **633-record** canonical bundle. All preceding records remain equivalent as JSON values. The new canonical bundle contains **646 records / 291,827,290 bytes**. The saved snapshot contains **34 cached primitives / 304,104,754 bytes**, SHA-256 **`7133c5f70c03ec35d5e6b170a0b4d2466c9e61bc329a7fffe0d9044206484060`**. Older Native records outside this canonical subset remain in the historical source index.

Provider close wrote the complete snapshot, then reported **cleanup_incomplete** because a private cached database file was still locked. The original failure envelopes are retained. After the owned provider/launcher processes exited, independent verification found all 646 snapshot records equal to the exported canonical bundle and all 34 Native cache entries present. A separate PowerShell cleanup verified the exact run/target ownership, absent owned processes and absence of reparse points before removing only that private temporary directory. Its absence and the preserved unrelated process are recorded in `close-recovery.local.json`. No analysis producer was rerun and no failed close is represented as a successful REA response.

Ignored local run: `client-data/audit/2026-11-02-sframe-main-mode-reader-6c4a9e21`. Complete records, both ledgers, selected byte checks, checkpoint, snapshot and preservation/publication receipts remain local.

**Nine preservation checks pass.** Eleven baseline executable, addon and settings hashes, the saved-variable tree, logical database backup and primary service identities remain unchanged. The root Git index stays empty. Only seven authored Markdown files are synchronized through isolated Core/Client worktrees. Original assets, binaries, databases, credentials, settings and generated decompiler output are not published. Gameplay code and package versions are unchanged.

## Restart point

Reuse the closed snapshot, full source index and these two ledgers. Continue from an actual typed consumer of the retained primitive pointer, or another demonstrated reaching definition that could carry constructed MAIN to the unresolved receiver. Preserve the distinction between parent selected-handle fallback, child raw-handle lookup and per-message mode guards. Require a concrete MAIN-bound nonzero write and its causal entry event before reconstructing activation. Do not repeat the already bounded empty reference searches or infer activation from display labels.

## Follow-up: retained-pointer consumer, 2026-11-02

The [render-consumer investigation](SFRAME_MAIN_RENDER_CONSUMER_RE.md) follows the pointer retained at primitive+0x0c. KSpritePrimitive's a8d2c0→a8d770 route consumes the retained MAIN+0x118 rectangle as 16 bytes of clipping input. Its direct coordinate reads and helper write bounds are verified. Viewport enqueue/queue/virtual+8 dispatch is traced conditionally; the actual MAIN rendering context, 918630 receiver and mode activation remain unresolved. The earlier mode guards, child fallbacks and graph limits remain applicable. Data and versions stay preserved.
