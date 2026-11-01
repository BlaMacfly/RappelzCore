# SFrame caption handling, resource classes and child-owner callbacks

Verified **2026-11-01** on original x86 SFrame SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`, with **REA 6.4.0 / Ghidra 12.1.4 / Capstone 5.0.7**. This continues [the indirect-callback investigation](SFRAME_INDIRECT_CALLBACKS_RE.md). **25 new Native dossiers / 10,725 analyzed body bytes** identify the previously unbound caption handler, qualify resource-class descriptor flow and inspect selected callbacks after child ownership is stored. **MAIN byte+0x458 nonzero activation remains unresolved.**

[Versions](VERSIONS.md) remain Core/RappelzRust **0.1.0 development**, NoLimits launcher **1.3.60**, original SFrame PE file/product **9.1.0.0**, embedded **Source9.5.2** build lineage and network literal/profile **201507080**. Exact official Epic/patch remains unconfirmed. This pass does not execute the original client.

## Questions and results

| Question | Qualified result |
|---|---|
| Who receives MAIN+0x94 through global edd5a0? | The selected startup assignment binds a static myUICaptionHandler object. Its virtual0 is b90530, a caption-localization and marker-replacement routine. Runtime replacement is unobserved. |
| What does the caption callback write? | The selected route updates the caption string object through assignment/replacement helpers. Its 24-byte object, inline storage and separate heap buffer must be distinguished from the rest of MAIN. Bounds assume a valid string object; arbitrary corruption/ambient effects are not disproved. |
| How does a resource token become a factory class? | The original parser registers callbacks, copies an ordinary class token into the scratch descriptor, commits it to a ParseObject and copies it to the loader's descriptor. The loader separately injects owner/context. |
| Do child callbacks act on MAIN? | Selected receivers are typed controls. The tab-head path passes owner+0x44 as a source to a stack temporary; registry insertion is an earlier class-registration operation. Other downstream effects remain bounded. |
| What turns on the alternate sheet? | No reaching nonzero write to constructed MAIN+0x458 is demonstrated. This remains a reconstruction gap. |

## The caption callback has a concrete identity

The retained initialization routine **b87450** stores **e99b44** to **[edd5a0] at b87ea9**. This is the sole matched absolute store in the retained original-byte reference inventory, not a proof against computed/indirect stores. The object at e99b44 already contains original .data vptr **d92d84**. Exact COL **e1e388** and type descriptor **eb8b94** identify the local type **myUICaptionHandler** inside InitResource. Its selected virtual0 is retained **b90530**.

Retained **b3c510** from the preceding pass conditionally invokes that handler at **b3c628** with receiver+0x94. When the NUI root is applied directly to the existing character sheet, the argument is exactly **MAIN+0x94**; ECX is the caption-handler object, not MAIN. Retained b3d150 contains another caption-handler call after updating the same caption field on its own receiver.

Retained **b90530** returns for an empty string. Otherwise it resolves a localized caption and assigns it through retained **5bec70**. It then uses regex **#@([0-9]+)@#**, resolves the captured numeric key and calls fresh **b800c0** to replace a marker in that caption. Fresh **6e8f80** and **668490/668430/6682e0** qualify the selected regex/lookup route. Calling convention is checked on original PUSH/ECX/RET instructions: a PUSH before one helper may prepare a following call, so pseudocode argument lists alone do not prove consumption.

The replacement route is **b800c0→5c50d0→5c52e0** for the capacity-growth branch. 5c50d0 checks position, clamps the removed span, and distinguishes equal-size, shrinking and growing replacements; 5c52e0 checks the maximum length, chooses capacity, allocates a new character buffer, copies the selected ranges and updates the string's pointer/length/capacity. The -1 argument retained across the numeric lookup belongs to b800c0's maximum-replacement count, rather than an extra argument of the numeric converter. Getter 668490 returns without popping an argument; 668430/6682e0 return with RET4. Original stack behavior is kept alongside pseudocode.

The string object begins at MAIN+0x94, is **24 bytes**, has length/capacity at relative+0x10/+0x14 and uses the **15-byte inline-capacity threshold**. Inline character storage belongs to the object; longer character data uses a separate buffer pointer. Under the checked valid-string/capacity invariants, updates target that object and its selected buffer rather than MAIN+0x458. This is a typed local effect, not a universal claim about arbitrary heap aliases, corrupted capacities, exceptions or all callees.

## Original resource tokens reach descriptors

The three original archive resources are rechecked against decoded caches and parsed as **263 blocks**: MAIN one root/47 children, Basic one root/141 children, Added one root/72 children. The tokens and selected class registrations are the ones retained in the preceding pass; live override/resource precedence is not observed.

| Stage | Original path and receiver |
|---|---|
| Load | MAIN initializer 8a1ab0→820f70→b35b60 preserves the existing window. Basic/Added resources are loaded with their own window receivers. |
| Parse driver | KUIParser setup followed by b397a0→b33b00 drives the lexer. The original parser vtable selects b3a790 for an ordinary lexical token. |
| Reset | Registered begin→b3a2b0 resets scratch descriptor at parser+0x30. It does not select static by itself. |
| Class | b3a790 checks ordinary class-registry membership and copies the original token into the first string of the scratch descriptor. |
| Root token | Reserved genwnd→b3a680 creates a root ParseObject and records the genwnd descriptor. |
| Commit | Registered end→b3a4f0 treats genwnd as the active root; other classes create child ParseObjects. b399c0 copies descriptor data into ParseObject+4. |
| Descriptor copy | b35280 and short byte-checked getters connect the committed ParseObject descriptor to b35b60's stack descriptors. |
| Root application | At b35bff the root descriptor is passed to the existing receiver's virtual+8. The specialized MAIN object is not newly created by the class factory. A resource class string is not its RTTI identity. |
| Child application | Child descriptor base is EBP−0xc4. Store b35d20 writes receiver at EBP−0x14, exactly descriptor+0xb0. Store b35d0e writes factory context at descriptor+0xb4, before b3ead0 at b35d35. |
| Factory | b3ead0→b3ec90 uses the descriptor's first string as factory class; the new object's virtual+8 receives that descriptor. b3c510 stores class to child+0x64, owner to child+0x20 and context to child+0x138. |

This closes the selected **resource token→descriptor class→factory key→child class string** route on the inspected original parser path. It narrows the preceding report's token-translation gap. Full lexer/property semantics and arbitrary registry mutation remain outside this conclusion.

The loader injects its **current receiver**, not an unconditional MAIN pointer. A static control in MAIN's resource receives MAIN as immediate owner. A control inside Basic or Added receives that subpanel as immediate owner; reaching MAIN then requires a separately proven parent step. Basic and Added are separate allocated objects reached through pointers stored in **[MAIN+0x44c]** and **[MAIN+0x450]**, rather than subobjects at those offsets. The reserved genwnd root applies to each existing specialized object and does not replace it with a newly created generic root.

## Selected control callbacks and actual owners

The six selected control types and their original vtables are checked against the three resource contents. Generic initialization **9a5170** first calls b3c510 to persist the owner, then invokes the **new control's virtual+0xdc**. Fresh **9ad010, b682d0, b6f800, b6fce0, b52390**, plus the retained static-helper route **9a9030→9a7b90/9a7bd0**, qualify the selected bodies. These are child receivers, with other callback/callee effects explicitly retained as limits.

| Resource class | Exact RTTI name | Vptr | Virtual+0xdc |
|---|---|---|---|
| static | .?AVKUIControlStatic@@ | 0xd8d0f0 | 0x9a9030 |
| simplebutton | .?AVKUIControlSimpleButton@@ | 0xd664b0 | 0x9ad010 |
| iconstatic | .?AVKUIControlIconStatic@@ | 0xd8d1d4 | 0xb682d0 |
| tab_head | .?AVKUITabControlHead@@ | 0xd8deac | 0xb6f800 |
| tab_sheet | .?AVKUITabControlSheet@@ | 0xd8e074 | 0xb6fce0 |
| gauge | .?AVKUIControlGauge@@ | 0xd8b030 | 0xb52390 |

In tab-head **b6f800**, owner is loaded from child+0x20 at **b6f8c6**. It forms owner+0x44, PUSHes that source and calls fresh **80b130** at **b6f8d1**. ECX is a stack temporary at EBP−0x5c. This distinguishes the source owner-list address from the helper's receiver. 80b130 initializes that temporary and forwards the source to **80af50**, whose downstream effects remain a boundary. The selected tab-head path finds a peer with class **tab_sheet** and the expected sheet name, links it at head+0x2fc and adds the head to peerSheet+0x6d0; capacity growth calls b6abc0 on that peer collection. These are peer-control effects, not established MAIN mode writes. The tab-sheet's selected virtual+0x54 is retained **722460**, whose body is a single RET on the control receiver.

In the static-helper path 9a7bd0, **9a81e1 loads EDX=[control+0x104]**, then 9a81e8 loads ECX=[EDX+0x20]. That +0x20 belongs to a separately allocated auxiliary object installed at control+0x104, not to the control's owner field. The selected result is copied into control+0x28c. This prevents an incidental+0x20 access from being promoted to a MAIN recipient.

Fresh **9ac4d0** is separately bound to earlier class registration. The genwnd registration at 5b48f1 and gauge registration at 5b5977 insert class/creator pairs into the registry before these window children exist. MAIN or child+0x20 is not passed at those sites. Do not treat that upstream registry helper as a MAIN callback.

## New Native evidence and limits

All records use the original target digest and analysis-profile digest `b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17`. Inclusive body ranges, complete provider assembly and selected original instruction bytes remain local.

| Entry | Analyzed bytes | Evidence ID |
|---|---:|---|
| 0x9ad010 | 394 | `ev_0f44bced7aefb9e627e9b3d75cc291250c48ceeced2bacd381e07a36c4adfaf6` |
| 0xb682d0 | 46 | `ev_a773dea9fc5bb417673f6d807d4609773ba9651e29d82dccdf08c911d6f18b5a` |
| 0xb6f800 | 1237 | `ev_0b4dc9da8990d900ca1a8ddbc5c6fe5f82b83d6f59f72ce70687663f1b10b665` |
| 0xb6fce0 | 160 | `ev_9e554c4b6858d33d8576a6b4ae43089c88a760e9989d9e19fd77052693eab6fd` |
| 0xb52390 | 936 | `ev_fed44da61f094e9a86b05a055d0ee9a72eb3b8482b38b03602d00eaa1a9f29fc` |
| 0x9a7b90 | 60 | `ev_9f1ea700776ec2133a61359757f8d094f6510e5f56ba4536bc92e4bc258ba413` |
| 0x9a7bd0 | 1669 | `ev_9985e8beab0beff9120e8d26bdc72d32e66f8c6b445b641601a07ad279fa5a32` |
| 0x9ac4d0 | 546 | `ev_9263dd5d90c10177c7edc9683efdb77f91da5b2d76c797ca17d6837ce817058f` |
| 0xb800c0 | 380 | `ev_012dee3d9fce4ca1cdcce28c79a8ab0f0b181d63c560b2636264f2d2dff7c420` |
| 0x6e8f80 | 589 | `ev_f497043e896cb3516ab5fc6e8d9337e95daf4dce16048c4d5803093e8f734547` |
| 0x668490 | 441 | `ev_9b414244d3c9fe1767426262f2f36600274fa44af128a167d6c4ad76326864e4` |
| 0x668430 | 89 | `ev_a62f6f9db567f5ff0cc985723456a908f173b29b9b861a0d13332dd31779355e` |
| 0x6682e0 | 331 | `ev_eacf49e82d8bd52862eefdd63bc0d384f05f7f13628ede5c6a21ce3d7951f9ca` |
| 0xb397a0 | 34 | `ev_30fb166385a8e8e5322b25f41699985114d31517553b2a85a0317e184f5a1eb0` |
| 0xb33b00 | 50 | `ev_de3117b62e443219daf80939a7b05500a6994326fb5773b9f641edf2bebcad09` |
| 0xb3a790 | 464 | `ev_b574848ac50c819b06308d1097fc3254d7764106d21b3f25ad22fcd65eed77dc` |
| 0xb3a4f0 | 387 | `ev_944e3a1a144575dd7453f3c15ef857a7d87ebf9a936233bf17ee31e45ca1beca` |
| 0xb3a680 | 263 | `ev_b5bf248d561ac528022b668a8d11b633ae89dc00dc176608590aec66f15985f1` |
| 0xb399c0 | 273 | `ev_d3736f3cd834ccd4999dabd26324fe9acc18b95f4d2928e41414494c1a56fca0` |
| 0xb35280 | 230 | `ev_06bd0a6292706c32da0e795d0e78bc9b0105bf346fa8cc5ce0ee7129282eb707` |
| 0xb39f10 | 822 | `ev_08308fa8f2298a9f0c161fabeb69a66e276404c26b7c988fa7a8ad1069cd61ba` |
| 0xb3a2b0 | 492 | `ev_115b5e512d1258979bba2360b48cfe1e0ce9173de33bf0dfa48f42bad1ffedf9` |
| 0x80b130 | 114 | `ev_20be8738661f7b776a266840313e3cf8456fdc4a0785da3d3ac3d3c8d9daa805` |
| 0x5c50d0 | 349 | `ev_b6201b381926c9abcfe322bbd1c7891b6bb4329d4de85d43c79ed9d4932f5d45` |
| 0x5c52e0 | 369 | `ev_e08c446908633871d150538b3b9c2e7b3e12455431ad1c0f9df9d01e20fcbcad` |

All 25 new Native graphs report untruncated. The older b87450 graph retains its separate truncation limit.

Untruncated does not mean all indirect targets or ambient effects are resolved. The selected facts are checked on full body assembly and original bytes; graph caps remain explicit.

| Selected reused Native source | Evidence ID |
|---|---|
| 0xb90530 | `ev_ab147f4018c912b2f3dbc59cb6bd34f85204ad274476a32592bb4c3d1e87aa83` |
| 0xb87450 | `ev_55170aa159791c10c646fa3ef554fb6c5f5d01e9a6367af1709c90ef285824a6` |
| 0xb3c510 | `ev_6356bc8bbb36f8a8053edebbc871fdb724d632f03e1057a807c86f753eaca43d` |
| 0x5bec70 | `ev_76bd09e77e3793886e56c5f7be2345c449e97dd07d2a28724de2d79799d97d5e` |
| 0xb35b60 | `ev_780cd463050f07ecb6fc2124376fbcb2e95dd2a03b4f84435dcbface76076ff5` |
| 0xb3ead0 | `ev_5f57d5eeeef5a105042b0ceec3f79df05293b720c9fe47ae03fe6cc106e0f02f` |
| 0x9a5170 | `ev_ee83ed194b19c037d963a597ac72ac4ac55c2c284cd2f407cca0946112ec93fb` |
| 0x9a9030 | `ev_9e5ecf337e8acb93aa90b31101aba385d028b0b528920fe8e5e007dda03fa867` |

Historical graph limits and complete source hashes are retained in each worker ledger. In particular, a large result is not automatically truncated: the reused 5bec70 source reports untruncated, while b87450 reports truncation. Neither size nor a pseudocode type establishes full callee effects.

## Verification and preservation

Original-byte checks pass for **3,201 new Native assembly boundaries** and **264 distinct selected ledger intervals**. The common check also verifies **12 separately retained Native bodies**; caption/control/parser workers retain their own complete sources, source hashes, RTTI and coverage limits. Their successful verification receipts are kept with the ledger hashes.

The single owned Native session starts from the prior light snapshot containing 22 records/21 cached primitives, avoiding a full historical primitive-cache reload. Matching historical Native records are reused. Completed 668490/6682e0 results exceed the transport size and are recovered from retained canonical exports without reanalysis. New observations are checkpointed before the official import of the prior **487-record canonical bundle**; all prior records remain byte-equivalent as JSON values. The engine then exports and closes successfully with **512 canonical records / 46 cached primitives** (21 reused plus 25 new). The older full/selective snapshots and individual archived records remain untouched.

Ignored run: `client-data/audit/2026-11-01-sframe-caption-owner-4f68c231`. Canonical bundle **263,643,442 bytes**; closed snapshot **296,819,972 bytes**, SHA-256 **`aa3fdaa7989e0bfe5df6aa99d4cf9f4ca3e74e37c55e8550110a120451466306`**. Complete Native records, original-byte/type/parser/caption/control ledgers, canonical checkpoint, preservation and publication receipts stay local.

**Nine preservation checks pass.** Eleven baseline original/installed image, addon and settings hashes, the saved-variable tree, preserved database backup and primary service listeners/identities remain unchanged. The root Git index stays empty. Only seven authored Markdown files are synchronized through the isolated Core/Client publication worktrees. Original binaries/resources, databases, credentials, settings and generated decompiler outputs are not published. No game behavior is invented from the unresolved activation rule.

## Continuation

Reuse this canonical bundle, selective cache and the historical Native index together. Caption localization is now a typed route, and the selected NUI class translation is linked to the original parser. Continue only from proven owner/list deliveries and selected unresolved control callees; distinguish immediate Basic/Added ownership from MAIN. A mode implementation still requires an observed or statically proven nonzero write reaching the constructed MAIN byte+0x458.
