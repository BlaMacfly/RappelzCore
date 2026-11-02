# SFrame receiver family and expanded MAIN alias coverage

Verified **2026-11-02**, following the [MAIN transition audit](SFRAME_MAIN_MODE_TRANSITION_AUDIT.md), on original x86 SFrame SHA-256 **`27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`**. **The actual receiver of 0x918630 and the trigger of MAIN+0x458 are not identified.** This pass adds **3 new Native decompilations / 469 body bytes**, positively qualifies registered Arena callbacks and their counter widget, and explores **22,308 additional modeled MAIN alias states**. It does not assume a usable alternate feature from the flag found in code.

[Versions](VERSIONS.md) remain Core/RappelzRust **0.1.0 development**, NoLimits launcher **1.3.60**, SFrame file/product **9.1.0.0**, embedded **Source9.5.2** lineage and network literal/profile **201507080**. Exact official Epic/patch remains unconfirmed. Tools: **REA 6.4.0 / Ghidra 12.1.4 / Capstone 5.0.7**, analysis profile **`b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17`**. The original client is not executed.

## Positive construction, widget and registration evidence

The reused constructed family is SUIArenaWaitingWnd, installed by 916550 with original vptr d5c778 and loaded as window ID d7 in 9e47b0. Its inherited 820f70 → 821040 initialization dispatch selects virtual+d8 = 916940. That initializer calls 917500 and binds its named children. This is an actual construction/table/initialization chain; it does not type the entry ECX of 918630.

A newly qualified pointer source is Arena+0x44c: 916940 asks b3c660 for the exact name text_arena_count_01 at d5c864 and stores its returned pointer at 9169d3. ID d7 selects window_instance_arena_sub_standby.nui through e97688, and the original selected archive payload declares that name as a static control. The 6438-byte decode matches the retained cache exactly. This binds a pointer stored in the constructed Arena family, subject to successful loading and intact child lists; it does not make the field an embedded control or guarantee a nonnull live pointer.

The selected original NUI payload is **6,438 bytes**, SHA-256 **`6d6e2a669afbae458c3c46f7f866c9647246e0a824ef588b72738dfd01a3cdc5`**, with byte-identical retained cache. The archival and cached agreement is resource evidence, not an observation of live loading or resource precedence.

The complete known registration body 917500 supplies five numeric-map registration pairs on Arena+0x470: 9c44 → 917aa0, 9c4e → 917e80, 9c42 → 917d80, 9c48 → 917d60 and 9c43 → 917e60. The helper stores the supplied callback on the insertion path, while its first existing-key branch returns without overwriting the existing callback. The reused dispatcher 916bc0 matches message+4, takes node+0x14 and calls it with ECX equal to the same outer Arena receiver and the message argument. The two 19-byte raw-only callbacks mark message+8 and call 9187f0 with that receiver. None of these five supplied targets or the selected 59-slot original Arena table selects 917810, 917980 or 918630. That is a bound on these concrete registrations, with intact/new-map conditions, not all possible registrations.

The registration helper 917220 preserves an existing key's callback rather than overwriting it. The target table below therefore describes successful insertion into the expected new/intact map; preexisting keys, later mutation, allocation and reentrance retain their conditions.

| Registered key | Selected callback | New coverage |
|---|---|---|
| 0x9c44 | 0x917aa0 | Prior typed Arena countdown producer, reused |
| 0x9c42 | 0x917d80 | New Native member-count/button/show handler |
| 0x9c4e | 0x917e80 | New Native member-count handler |
| 0x9c48 | 0x917d60 → 0x9187f0 | Complete 19-byte raw wrapper; new Native helper |
| 0x9c43 | 0x917e60 → 0x9187f0 | Complete 19-byte raw wrapper; same new helper |

The three new Native functions close the remaining substantive registered paths. 917d80 updates text_current_member_01, changes the stored button_arena_start_01 child through +0x294 and virtual+0x3c, and enqueues a show request for ID d7. 917e80 updates text_current_member_01, while 9187f0 sums two indexed values and formats the same named child for both forwarding callbacks. None contains an explicit 454/458/45c operand in its own complete body. Their callees, globals, hooks and virtual recipients can have other effects; this is not a claim of transitive purity.

The typed Arena callback 917aa0 is a real producer of FLOAT[Arena+0x454], and typed tick 916c20 uses that component and the newly bound counter child with the exact d5c88c font literal and d3a124 format. 917980 has a compatible countdown/text pattern and 918630 uses the same format and footprint. 917810 shares warning construction without an explicit entry-receiver use. These are family affinities only: no actual reaching transfer, table registration or constructed ECX for any of those orphan entries is established by this pass.

The shared reset helper still writes byte O+0x458=1 and DWORD O+0x45c=0 when called by 918630 with ECX=O+0x454. This is a concrete local effect with an **unbound O**, not a proved activation of constructed MAIN. Neither shared formatting literals nor compatible offsets establish the missing reaching transfer or object identity.

## Changed alias-analysis input and measured coverage

A state variant is the exact tuple of tracked register aliases, stack spills and fresh pushes at one instruction. The 16-variant/4,096-state replay exactly reproduces the old observations. Only caps, selected targets and diagnostic logging change; caller snapshots, ABI cleanup, instruction semantics and branch treatment stay fixed.

| Variants per instruction | Per-function state budget | Visited state variants |
|---:|---:|---:|
| 16 | 4,096 | 12,582 |
| 32 | 16,384 | 20,410 |
| 64 | 32,768 | 34,890 |

At 32 variants the model visits 20,410 states; at 64 it visits 34,890, compared with 12,582 originally. The 64 pass adds 22,308 states, admits 472 of the 496 explicitly omitted old variants (95.2%), and preserves every previously visited state. No global per-function state budget is reached. The local cap for b3c1d0 is resolved within the model; seven local caps remain, including 24 old frontier variants.

Across the expanded states there is no new unique MAIN-address callout or MAIN write, no write covering MAIN+0x458, and no tracked MAIN+0x454/0x458 address in admitted states or logged omitted boundaries. This is new bounded coverage, not a client-wide absence proof. No additional Native capture is indispensable from this comparison.

The model does not prove conditional-edge feasibility or arbitrary memory/heap aliases and preserves indirect-call, partial-register, exception, ABI and reentrance limits. At 64 the remaining frontier contains 950 unique variants while the effect sets have not changed, so further blind cap increases are stopped. Activation of MAIN+0x458 remains unknown; no live feature, dead code or 0x918630 receiver is inferred.

The **24 old omissions** and **950 unique frontier variants at the new limit** describe different sets. Admitting 95.2% of the old omissions does not measure whole-program coverage or prove that those modeled paths can execute. The selected eight bodies remain the entire changed analysis scope.

## New Native evidence and retained sources

| New Native entry | Analyzed bytes | Evidence ID |
|---|---:|---|
| 0x917d80 | 219 | `ev_22348c7435efb8502ae54c85858842b2c8f1c351a0b935a386a3cf16660112e9` |
| 0x917e80 | 118 | `ev_68d55fed6eba187a992d72f4c740efd0314b9d58682a1ab575fdb9983143cf59` |
| 0x9187f0 | 132 | `ev_767ce1d09dbd0872526e18c680cc6c23b96420a4f127436dd32dd8404b361341` |

All three new Native graphs report no truncation. Complete original body/assembly checks remain distinct from pcode completeness, indirect callees and runtime effects. The two 19-byte wrappers are **raw-only coverage**, not extra Native dossiers. This pass does not repeat the earlier empty incoming query, entry/body carrier sweeps, PDB inventory or neighboring-module search.

| Selected Native source | Evidence ID |
|---|---|
| 0x820f70 | `ev_bf6a01b950ce19ea40f7d2a5709725e5431a157e569999caec2f85e087de96e0` |
| 0x821040 | `ev_635168ea1ab1ba99e4dbbf1d9dbfd3b64975a7ca3476836b13603be71e77fa54` |
| 0x821140 | `ev_96d1a02e467350902a74b677aee36d55e4c5c3a781822d97f6c1a25aa36a649a` |
| 0x8fb010 | `ev_6311d6edbba44308e24bffdc741ec9c6fbccbb16ca26c14b03969b080907bce5` |
| 0x916550 | `ev_596c3d917c90a0709cbb04e922e15feda7569b25cb26d777229825c4cde8e735` |
| 0x916940 | `ev_85b88ee032d7fbd1fc641566e16248d2e186783fccea7b76df953b9f7a705de8` |
| 0x916bc0 | `ev_24d7024d1352d49ab5b8609e5cd1bca0e7242999e5cea6b8bb7d76ff6e2020f2` |
| 0x916c20 | `ev_f798fb6a88410dc78b49df4cd35c5c220e4e99768fdc15de5da1adfaa5039e62` |
| 0x917220 | `ev_4a91334aea05853adaaa19f600f34a1424ea5dda05858e07416d0782959d6cae` |
| 0x917500 | `ev_30bc9b7b18b248bd3855f1f1c933d9f89ac652d78cc43105cecedda6548e1ee1` |
| 0x917810 | `ev_85e0d1e5f82d8e0bbd61096a80d3e3c93a6b604c1d0a74ee0bae63959df29b08` |
| 0x917980 | `ev_6f667ccfb3e130926af3c52d22c1e224e43b001a72658002aaf68a586c4e976c` |
| 0x917aa0 | `ev_5c5e2e76bb180562efa78127363e7cf4c687dc112ad925f6c515dde17249fb27` |
| 0x917d80 | `ev_22348c7435efb8502ae54c85858842b2c8f1c351a0b935a386a3cf16660112e9` |
| 0x917e80 | `ev_68d55fed6eba187a992d72f4c740efd0314b9d58682a1ab575fdb9983143cf59` |
| 0x918580 | `ev_aa32599fddde90584680130bd4bf1522a4f5e70355b17bdf6e60dc8702668644` |
| 0x918630 | `ev_2e11babc974a7b4241ded6116bc72be4cbafa3d434f034fc40c37e79a04da7bc` |
| 0x9187f0 | `ev_767ce1d09dbd0872526e18c680cc6c23b96420a4f127436dd32dd8404b361341` |
| 0x9c8920 | `ev_4a5435ee59e170130751d04fb39f47a481fa7b5c82322bcd854db8e838d4e2cc` |
| 0x9e47b0 | `ev_7aaf53f690b1adb712ad0153b9adcd3f00d5dca97844e44cac215663b2d2ed27` |
| 0xb36310 | `ev_4d813a37cb1fa1dc648c80a13e1320f2817ca0fbf2bf51308a59df49650499de` |
| 0xb363f0 | `ev_6f4e0a17cd43ad32da91d644e421a212ea02a4a1427e10dde52f78c9aa6752a9` |
| 0xb36a30 | `ev_7b843fcf59711867a7cd81807a3bef5a97a6d5ea05b0a5da783ce97408935482` |
| 0xb36cb0 | `ev_d1c78eb1e366ec54558ca555fc7916d89244e357a120a72e91d7234db3b3dd84` |
| 0xb36f70 | `ev_097dc11c65b82f8f726597ddb69709b71b54b9917034a8921190acddb3290bf7` |
| 0xb376a0 | `ev_10d7bfd5420317fd7d23d357bf1455a5e746ddac4cb283d31e4b96f022822561` |
| 0xb3c1d0 | `ev_fd7fc77cf7b42522280d6bb785b3a270f06f12bdd74393794f540b7295c75c14` |
| 0xb3c660 | `ev_666b2d425f99f55ebaefa73584a86da66c18f086c8e8466d973a26499d848392` |
| 0xb3d4f0 | `ev_ad8725bbe05f2378247b79f6c19fb6a39e2b4122eebe575c6ae63730f4a888d5` |

Independent checks bind all selected Native bodies to original bytes and instruction boundaries, and verify **1,515 distinct selected provenance intervals**. The ignored local family and alias ledgers retain their exact source hashes, coverage, original-byte proofs and remaining unknowns.

One owned analysis session checkpoints the three new observations, imports the previous **694 canonical records** without changes, exports **697 canonical records** and closes successfully. Snapshot: `2026-11-02-sframe-mode-receiver-family-84c1d9a7/sframe-mode-receiver-family.snapshot.json`, **328,815,816 bytes / 24 cached primitives**, SHA-256 **`0421d455a4468c1e98f6563a57c587e8749d0e9ed1e44c7821560728ff47ee74`**. The older request-producer snapshot and canonical sources remain unchanged. Complete export comparison validates one decoded record at a time to bound verification memory.

**Nine data-preservation checks pass.** Eleven baseline file hashes, installed saved-variable tree, logical database backup and primary service identities remain unchanged. The root index stays empty. Only seven authored Markdown files are published through the isolated Core/Client worktrees; no original binaries/resources, dumps, credentials, settings or databases are published. Gameplay code and package versions are unchanged.

## Remaining identification requirement

MAIN remains SUICharacterInfoWnd, a separate constructed window ID 2e. No new reaching transfer to 918630 with MAIN in ECX, nor a separate qualified nonzero store to byte MAIN+0x458, is established. The code investigation remains open; no usable or previously observed alternate-mode feature is presumed.

The next decisive evidence must be an actual reaching transfer/registration to 918630 with the origin/type/state of entry ECX, or a separate nonzero write to +0x458 on the same constructed MAIN instance. These Arena callback and widget paths are now qualified under their explicit guards; they must not be reclassified as an unknown family or counted again as a new discovery. Original tables, successful construction/loading, valid objects, normal ABI, preserved registrations, lifetime and non-reentrant state are conditions. Aliases, hooks, runtime replacements and downstream effects remain unknown. No class assignment for the orphan, universal no-writer theorem, dead-code claim or usable alternate feature is established.

## Follow-up: computed-entry delta and MAIN descendants

[Computed-entry delta and MAIN descendants](SFRAME_ENTRY_AND_DESCENDANTS_RE.md): the 106-body retained-corpus delta adds 40,990 provider-bounded bytes / 11,417 instructions to the address-propagation model, with no target candidate. Exact MAIN TD reuse in 2,134 retained RTTI tables and 210,912 declared HIGHLOW slots identifies only its own hierarchy/constructor. The actual 918630 caller/receiver and MAIN+0x458 trigger remain unknown. Source-derived audit only; zero new Native analyses or original execution. Nine preservation checks pass; versions unchanged.
