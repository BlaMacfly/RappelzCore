# SFrame concrete MAIN pointer and message flow

Verified **2026-11-01** on original x86 SFrame SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`, with **REA 6.4.0 / Ghidra 12.1.4 / Capstone 5.0.7**. This continues [the pointer-delivery investigation](SFRAME_POINTER_DELIVERIES_RE.md) from actual constructed **SUICharacterInfoWnd** pointers, rather than untyped offset candidates. **68 new native dossiers / 14,797 analyzed body bytes** and **19 containment observations** qualify creation, registration, lookup, message delivery and geometry. **MAIN byte+0x458 activation remains unresolved.**

[Versions](VERSIONS.md) remain Core/RappelzRust **0.1.0 development**, NoLimits launcher **1.3.60**, original SFrame PE **9.1.0.0**, embedded build lineage **Source9.5.2**, network literal/profile **201507080**. Exact official Epic/patch remains unconfirmed. The original client is not executed.

## Reuse includes archived individual dossiers

The previous canonical bundle contains 378 records and 271 cached primitives. A SHA/profile-bound search also recovers older individual Native records outside that bundle. This pass verifies **39 reused Native functions**, including **10 whose records are outside the previous bundle**, against original body bytes and instruction boundaries. These archived records remain separately cited; they are not silently claimed as newly produced or imported into the current bundle. Reuse the historical index on continuation instead of assuming the latest snapshot contains every historical result.

## Construction and exact registry identity

The retained startup factory **0x9cb4e0** allocates **0x478 bytes** at 0x9cec3a and calls constructor **0x8a19f0** at 0x9cec60. Original bytes bind its vptr **0xd53af8** through exact RTTI to **SUICharacterInfoWnd**. The creation branch preserves the returned window in ESI, passes that pointer and key **0x2e**, and restores the factory context from its frame before **0x9c8920** at 0x9ceca1.

The constructor and resource initializer return their receiver in EAX even where recovered pseudocode displays void. The original return instructions and caller use establish the pointer; a pseudocode return type alone would lose this flow. The selected resource-initialization call is MAIN virtual+0xd0→**0x8a1ab0**, which creates Basic/Added children and stores MAIN in each child's+0x20 through **0xb3d9e0**.

Registration **0x9e2aa0**, lookup **0x9e4330** and visibility query **0x9d1280** all pass **context+0x24** to fresh **0x9e50b0**. Its Native body confirms bucket selection, sorted-bucket key search and found-value output. The selected registration path associates key 0x2e with the constructed MAIN pointer. This is a concrete path, conditional on initialization/registration success; arbitrary runtime replacement of a registry value remains unobserved.

The inspected lazy factory **0x9c56f0** has a distinct result for this key: ID 0x2e produces index 17, selector 64 and target **0x9c7c24**, which returns zero. It is not the startup factory. This proves the selected fallback does not recreate MAIN; it does not exclude other indirect/custom construction paths.

## Messages really reach MAIN

Retained **0x9dd9b0** calls lookup 0x9e4330, checks its result, restores that exact pointer to ECX, and calls virtual+0xcc. For the constructed registry entry this is **MAIN+0xcc→0x8a1dd0**. Message pointer and window identifier are stack inputs; the registry context is a separate receiver.

**Twelve send sites**, one visibility check and one direct geometry lookup are qualified. Ten sends occur in retained0x9d3db0. Known primary internal case labels for selected sends include 9,31,33,49,54,99,1030 and1290; one selected send's source case remains unassigned. These are internal dispatcher observations, not newly established network packet layouts. The large reused dispatcher graph is truncated with 240 known omitted edges; selected PUSH/CALL/register facts are checked directly.

Fresh **0x9e56e0**, reached under internal kind **40004**, forwards its original message pointer to key 0x2e at CALL 0x9e57ae. Fresh **0x9e5c90**, reached under **40003**, conditionally forwards its original message at CALL 0x9e5f33. The corresponding key PUSH sites are 0x9e57aa/0x9e5f2f; they are not CALL addresses. The separately constructed 0x426/refreshAllTitle command in 0x9e56e0 targets **Title window 0x36**, rather than MAIN. The MAIN handler's existing40003/40004 branch refreshes name-related display state via0x8a3740; no mode activation is recovered from these producers.

## Whole-object and interior-pointer effects

| Concrete handoff | Observation and boundary |
|---|---|
| 0x8a1a97 | Whole MAIN in a deleting-destructor path. This is object release, not activation of a live sheet. |
| 0x821355→0x8215d0 | MAIN is explicit argument 2, a geometry reference. The callee's 439-byte provider-described body is qualified on original bytes, not claimed as a new Native dossier. Rectangle reads use722270; target-record+8 moves through b3c8a0. Context virtual+0x10 receives a stack message; downstream effects remain bounded. |
| 0x8a215a→0xb402a0 | MAIN is an explicit argument to a distinct context receiver, for list/focus handling. The generic+0x13c flag is separate from mode+0x458. |
| 0x8a1b53/0x8a1be5→0xb3d9e0 | Child receiver plus explicit MAIN owner argument establishes the two child+0x20 links. |
| 0x9e14c9 lookup→0x9e14e8 move | Key PUSH at 0x9e14c5 selects MAIN; EAX→ESI preserves the returned pointer across rectangle reads on a different window. ECX=ESI then enters retained b3c8a0, which updates header rectangles and invokes virtual+0x20→b36a30. Indirect child effects are not globally excluded. |
| 722270 and b37d90 returns | On the qualified nonnull receiver paths, returned aliases are MAIN+0x108 and MAIN+0x2c4. The latter is passed to child virtual+0x2c/+0x34. Optional receiver redirection in b3c650 is kept separate. |

The 59 MAIN table bodies cover **10,111 bytes / 3,119 Native instructions**. Their whole-body literal+0x458 references are four reads in 0x8a1dd0 and two zero clears in 0x8a1d40. Constructor initialization is a separate known zero write. No nonzero activation follows from this bounded result. Eight functions reach flow-analysis caps; unknown callee effects and arbitrary aliases are not disproved.

The callout audit distinguishes register tags from proven receiver use. A prolog PUSH ECX can reserve a local slot; a later MOVSS[ESP] can overwrite a placeholder with a float argument. Neither is automatically a MAIN argument. Likewise 101 tagged ECX states are not 101 proven receiver deliveries. The five explicit pointer PUSH sites above are qualified separately.

Both **69-entry child tables** are fully accounted for: Basic 65 Native/4 raw slot occurrences, Added 63 Native/6 raw. Nine distinct short targets account for the ten raw occurrences. The parent-flow audit covers 170 contexts and retains four capped loop contexts. Its six established MAIN call sites remain mode/handle getters and geometry. Two recovered parent-pointer stores are Basic stack spills; constructor owner-pointer storage is the separate path above. No new mode-byte write is recovered in the selected child code.

Fresh Added routines also identify concrete display behavior: **0x898650** caches named gauge controls, **0x899e90** loads a Basic color sample, **0x898ed0** updates karma/PK/kill labels and gauge geometry, and **0x898780** formats Arena/Holic/rank displays. These selected paths do not load the constructed MAIN parent pointer. Full effective gameplay/stat formulas or server disclosure behavior are not inferred from labels alone.

## MAIN's complete measured table

The prefix has 59 executable entries, offsets 0..0xe8; +0xec is adjacent metadata. Source labels distinguish fresh dossiers from prior-bundle and separately archived evidence.

| Slot | Target | Source | Evidence ID |
|---|---|---|---|
| 0x0 | 0x8a1a80 | new | `ev_c65d58f897a013dba0a8639e4b4cffc362a41cba3ec1fda10be2c72747c47761` |
| 0x4 | 0xb36f70 | new | `ev_097dc11c65b82f8f726597ddb69709b71b54b9917034a8921190acddb3290bf7` |
| 0x8 | 0xb35d90 | new | `ev_5cb2886f4bc408c96a914bdfa775c377c41a9598aa01e14a748d9e0a310be6e9` |
| 0xc | 0xb3cea0 | new | `ev_4fdb962da5b78740691df8384ef06c71cf2421d17eeff2550d742506f2e9cac8` |
| 0x10 | 0x722220 | new | `ev_15844a853c40747ecdf8a2b3586f81c9c8e78421090a1b43eadcd7092f4c0d50` |
| 0x14 | 0xb376a0 | new | `ev_10d7bfd5420317fd7d23d357bf1455a5e746ddac4cb283d31e4b96f022822561` |
| 0x18 | 0x8a23d0 | archived individual | `ev_550dc95b0cabda6eff467bede9e41278f11fa94fcef1bb72d91e222a2bcbdcc0` |
| 0x1c | 0xb36cb0 | prior bundle | `ev_d1c78eb1e366ec54558ca555fc7916d89244e357a120a72e91d7234db3b3dd84` |
| 0x20 | 0xb36a30 | prior bundle | `ev_7b843fcf59711867a7cd81807a3bef5a97a6d5ea05b0a5da783ce97408935482` |
| 0x24 | 0xb363f0 | new | `ev_6f4e0a17cd43ad32da91d644e421a212ea02a4a1427e10dde52f78c9aa6752a9` |
| 0x28 | 0xb36310 | new | `ev_4d813a37cb1fa1dc648c80a13e1320f2817ca0fbf2bf51308a59df49650499de` |
| 0x2c | 0xb369d0 | new | `ev_3aaf84a16109bd298eaffe5b967a559b124faed14e77ba6cd6352bf25ad17bd1` |
| 0x30 | 0xb369b0 | new | `ev_ef9a1779a14b42f0bb1f2a56d004c82360ce63f857ed7f1a772af83b80ff34ba` |
| 0x34 | 0xb36990 | new | `ev_473449277177848912029720fcc11e9fc871e66b87b10cdb72791ed3581353c0` |
| 0x38 | 0xb3cd80 | new | `ev_80e068455a4edeba637bfb842c8341f539f145d282588234c5fe6d63caf6f6e7` |
| 0x3c | 0xb363c0 | new | `ev_bdb013920bd8a3e7c8a03725091c8a4b106020d121f3d7b1cc8a3b55768813f0` |
| 0x40 | 0xb3ca70 | new | `ev_2123589bd208228566e4412effb4e9fcbbf39c00006bcef4654dd065c39d9634` |
| 0x44 | 0xb3ca60 | new | `ev_10c7a4269bef55aca86519588370562ee306270c0f64e78139f7a0da74e5f754` |
| 0x48 | 0x7224b0 | prior bundle | `ev_11fbabb910c5fb04970359389da71ea1de0de35ff47fe813085c6a0aeda34e6d` |
| 0x4c | 0x7224c0 | prior bundle | `ev_27d1bc83e37f7fedad826443e9dfc6fd0f198ba5a294aefa925704e7302278eb` |
| 0x50 | 0x7224d0 | new | `ev_f909216fac083d52d9e5c358de82e7ada3f4313c3c129d37c016d0b6221d35e8` |
| 0x54 | 0x722460 | new | `ev_9e3574380b042a125cf62fea195f83e24195d50b4a6f2d11134462185fd41031` |
| 0x58 | 0x821140 | new | `ev_96d1a02e467350902a74b677aee36d55e4c5c3a781822d97f6c1a25aa36a649a` |
| 0x5c | 0xb36490 | new | `ev_54c6adbafc1a65d959f83a4ad0b69ff031bfeb73a66c874e6ca77c6e24b967f8` |
| 0x60 | 0x722250 | new | `ev_de6386e6b817e053c83495765a2d1fec96f3216645e49c7d9bb3eccd46abb2a4` |
| 0x64 | 0xb3c120 | new | `ev_bb706ce5042d2e1947d55351150a6fa9bb0e2452ea871b92394db8dc2e9dd0a7` |
| 0x68 | 0xb3d080 | new | `ev_051d64019fae344941e0fc417a32adfff3da93c9eda141b92c754c5e4025cd95` |
| 0x6c | 0xb3d030 | new | `ev_f9af724396bc2ecad78c1deb7861d9b1f37245aa38596aa566320266fdcf3f31` |
| 0x70 | 0xb3d360 | new | `ev_a72d27cc02c90b43d43c6956105d3cf7e4e3fd2d1359817c90f2dc6cd93f684e` |
| 0x74 | 0xb3d4c0 | prior bundle | `ev_dc757e2e5359bd61c41d5915bb6c0b0583e49fabc960484f2b63ab82306056cb` |
| 0x78 | 0xb3d4f0 | new | `ev_ad8725bbe05f2378247b79f6c19fb6a39e2b4122eebe575c6ae63730f4a888d5` |
| 0x7c | 0xb3d560 | new | `ev_f31d0cb200e2471119169aa83811068505bdf4d23851c8279905ec9a0fdddec6` |
| 0x80 | 0xb3d5c0 | prior bundle | `ev_d1b329ccac2f0c8aa70c09370d1e18e377ec379310b8a71b7c137859ab916624` |
| 0x84 | 0xb3d580 | new | `ev_988b44cc903a6c1c69cd55d0890ecb53a1fbe7d9b85171793c9d7e085d0fcade` |
| 0x88 | 0xb3d540 | new | `ev_313b86deb25d9f970d1cfacec301c2b6cf5a841fcdee63b9f3674fe8ba385a42` |
| 0x8c | 0xb3d680 | new | `ev_485db7f75d948eef45e409f49c7db42c1a80ac969f6a1176b5a954bb94bb6148` |
| 0x90 | 0xb3d5f0 | new | `ev_f87fffb7c8e0daebc35c80e5822ad8a290885420793affda14ff44abffadfb2f` |
| 0x94 | 0xb3d620 | new | `ev_675ab622891975d5f80c1b2924254818fb181cf9d768e61658b02d13d3fa58ef` |
| 0x98 | 0xb3d650 | new | `ev_c4b44c9f716523981cc71589c27c643f52a7c40454f6d04d5d22d9ed541e201f` |
| 0x9c | 0x722800 | new | `ev_bf37b1a03693f2ac622829d5bcb4577c03736b4cb552dcd59a9e19140bb01ed1` |
| 0xa0 | 0xb3d910 | new | `ev_c046eb44a9bd3f31ee0fb031d50104b2b58d127e3d22d6e61f87681d9a22f1f7` |
| 0xa4 | 0xb3d980 | new | `ev_63c6633db47538ccad71b53d382f244b565382db1791d5a1c8d7cef252dc8cd0` |
| 0xa8 | 0xb3c4e0 | new | `ev_fb091091e4a6731b2cab20b53248975163a36837c828e20469134808ce40d8ec` |
| 0xac | 0xb3c1d0 | new | `ev_fd7fc77cf7b42522280d6bb785b3a270f06f12bdd74393794f540b7295c75c14` |
| 0xb0 | 0xb3dae0 | new | `ev_9fd6c4b40506a817eb0497b5321900f31b8d3f19405b0022c3ff47b1a2182c5c` |
| 0xb4 | 0xb3d2e0 | new | `ev_b0d6e306aa35846f416143298c2b6dba107f8e9ba16a62db86e02a51c9c7f777` |
| 0xb8 | 0xb3cf40 | new | `ev_53f3adeb6dd18f56b65e4b976014c7c37b2883606779c3cccf094297b6e48f75` |
| 0xbc | 0xb3dab0 | new | `ev_57b8bb449a740a876706f5c04d48a6367f703290f5c153845a9fc93d2c19a545` |
| 0xc0 | 0xb3c700 | new | `ev_d75f5ca00522b32e85df1a3fa060e7dd097d616af1c3239ba0705f171c0c00ca` |
| 0xc4 | 0xb3bc20 | new | `ev_633d9455159f63d771900deb97946db3846b3597681e09bc2e298271f0991c4e` |
| 0xc8 | 0xb3c140 | new | `ev_9ce59e876543d722da6b9d674fced28d23c38dcaae09d11b56b215222802f927` |
| 0xcc | 0x8a1dd0 | archived individual | `ev_0c7097265f16bc7a09c0d3f3f17e1b34cc3261cd3e00af9dbe41055da9d6ee1e` |
| 0xd0 | 0x8a1ab0 | archived individual | `ev_6d832c9d0196d2b1898603d93fd01bcd02fc3ba1df5750e51a235611f561c24c` |
| 0xd4 | 0x8a1c20 | prior bundle | `ev_555e48d8e2d1429350dcb3ae8fe9855873fdb9f23d6e1fc0036fb0a1baf291b4` |
| 0xd8 | 0x8a1c60 | prior bundle | `ev_1e63597953a7299e94154316eae70632f3375c212f1d983c4ac390c4016f4b60` |
| 0xdc | 0x8a1d40 | archived individual | `ev_10e965f99af0876e0d108b0ba4bfd7820f456137fa2b9dfde61ae54ea385f824` |
| 0xe0 | 0x722530 | new | `ev_738352ec67766bfeefa034152dffa3fb5a20a4a02627027fd777daf1f0a99b46` |
| 0xe4 | 0x722900 | new | `ev_ca25f5fb6d7205e492ade12e1a5de1772dac68d54d274946f85cb2b8aa6d0e96` |
| 0xe8 | 0x722260 | new | `ev_456e8882442525095ec53eeb7347ad2755ef97c65e7479738c18050b2af44547` |

## New Native Evidence

| Entry | Analyzed body bytes | Evidence ID |
|---|---:|---|
| 0x8a1a80 | 38 | `ev_c65d58f897a013dba0a8639e4b4cffc362a41cba3ec1fda10be2c72747c47761` |
| 0xb36f70 | 1359 | `ev_097dc11c65b82f8f726597ddb69709b71b54b9917034a8921190acddb3290bf7` |
| 0xb35d90 | 754 | `ev_5cb2886f4bc408c96a914bdfa775c377c41a9598aa01e14a748d9e0a310be6e9` |
| 0xb3cea0 | 45 | `ev_4fdb962da5b78740691df8384ef06c71cf2421d17eeff2550d742506f2e9cac8` |
| 0x722220 | 3 | `ev_15844a853c40747ecdf8a2b3586f81c9c8e78421090a1b43eadcd7092f4c0d50` |
| 0xb376a0 | 243 | `ev_10d7bfd5420317fd7d23d357bf1455a5e746ddac4cb283d31e4b96f022822561` |
| 0xb363f0 | 156 | `ev_6f4e0a17cd43ad32da91d644e421a212ea02a4a1427e10dde52f78c9aa6752a9` |
| 0xb36310 | 168 | `ev_4d813a37cb1fa1dc648c80a13e1320f2817ca0fbf2bf51308a59df49650499de` |
| 0xb369d0 | 83 | `ev_3aaf84a16109bd298eaffe5b967a559b124faed14e77ba6cd6352bf25ad17bd1` |
| 0xb369b0 | 27 | `ev_ef9a1779a14b42f0bb1f2a56d004c82360ce63f857ed7f1a772af83b80ff34ba` |
| 0xb36990 | 22 | `ev_473449277177848912029720fcc11e9fc871e66b87b10cdb72791ed3581353c0` |
| 0xb3cd80 | 3 | `ev_80e068455a4edeba637bfb842c8341f539f145d282588234c5fe6d63caf6f6e7` |
| 0xb363c0 | 48 | `ev_bdb013920bd8a3e7c8a03725091c8a4b106020d121f3d7b1cc8a3b55768813f0` |
| 0xb3ca70 | 1 | `ev_2123589bd208228566e4412effb4e9fcbbf39c00006bcef4654dd065c39d9634` |
| 0xb3ca60 | 1 | `ev_10c7a4269bef55aca86519588370562ee306270c0f64e78139f7a0da74e5f754` |
| 0x7224d0 | 3 | `ev_f909216fac083d52d9e5c358de82e7ada3f4313c3c129d37c016d0b6221d35e8` |
| 0x722460 | 1 | `ev_9e3574380b042a125cf62fea195f83e24195d50b4a6f2d11134462185fd41031` |
| 0x821140 | 664 | `ev_96d1a02e467350902a74b677aee36d55e4c5c3a781822d97f6c1a25aa36a649a` |
| 0xb36490 | 100 | `ev_54c6adbafc1a65d959f83a4ad0b69ff031bfeb73a66c874e6ca77c6e24b967f8` |
| 0x722250 | 3 | `ev_de6386e6b817e053c83495765a2d1fec96f3216645e49c7d9bb3eccd46abb2a4` |
| 0xb3c120 | 23 | `ev_bb706ce5042d2e1947d55351150a6fa9bb0e2452ea871b92394db8dc2e9dd0a7` |
| 0xb3d080 | 43 | `ev_051d64019fae344941e0fc417a32adfff3da93c9eda141b92c754c5e4025cd95` |
| 0xb3d030 | 65 | `ev_f9af724396bc2ecad78c1deb7861d9b1f37245aa38596aa566320266fdcf3f31` |
| 0xb3d360 | 32 | `ev_a72d27cc02c90b43d43c6956105d3cf7e4e3fd2d1359817c90f2dc6cd93f684e` |
| 0xb3d4f0 | 69 | `ev_ad8725bbe05f2378247b79f6c19fb6a39e2b4122eebe575c6ae63730f4a888d5` |
| 0xb3d560 | 29 | `ev_f31d0cb200e2471119169aa83811068505bdf4d23851c8279905ec9a0fdddec6` |
| 0xb3d580 | 57 | `ev_988b44cc903a6c1c69cd55d0890ecb53a1fbe7d9b85171793c9d7e085d0fcade` |
| 0xb3d540 | 30 | `ev_313b86deb25d9f970d1cfacec301c2b6cf5a841fcdee63b9f3674fe8ba385a42` |
| 0xb3d680 | 30 | `ev_485db7f75d948eef45e409f49c7db42c1a80ac969f6a1176b5a954bb94bb6148` |
| 0xb3d5f0 | 38 | `ev_f87fffb7c8e0daebc35c80e5822ad8a290885420793affda14ff44abffadfb2f` |
| 0xb3d620 | 46 | `ev_675ab622891975d5f80c1b2924254818fb181cf9d768e61658b02d13d3fa58ef` |
| 0xb3d650 | 35 | `ev_c4b44c9f716523981cc71589c27c643f52a7c40454f6d04d5d22d9ed541e201f` |
| 0x722800 | 51 | `ev_bf37b1a03693f2ac622829d5bcb4577c03736b4cb552dcd59a9e19140bb01ed1` |
| 0xb3d910 | 104 | `ev_c046eb44a9bd3f31ee0fb031d50104b2b58d127e3d22d6e61f87681d9a22f1f7` |
| 0xb3d980 | 86 | `ev_63c6633db47538ccad71b53d382f244b565382db1791d5a1c8d7cef252dc8cd0` |
| 0xb3c4e0 | 33 | `ev_fb091091e4a6731b2cab20b53248975163a36837c828e20469134808ce40d8ec` |
| 0xb3c1d0 | 769 | `ev_fd7fc77cf7b42522280d6bb785b3a270f06f12bdd74393794f540b7295c75c14` |
| 0xb3dae0 | 147 | `ev_9fd6c4b40506a817eb0497b5321900f31b8d3f19405b0022c3ff47b1a2182c5c` |
| 0xb3d2e0 | 121 | `ev_b0d6e306aa35846f416143298c2b6dba107f8e9ba16a62db86e02a51c9c7f777` |
| 0xb3cf40 | 144 | `ev_53f3adeb6dd18f56b65e4b976014c7c37b2883606779c3cccf094297b6e48f75` |
| 0xb3dab0 | 44 | `ev_57b8bb449a740a876706f5c04d48a6367f703290f5c153845a9fc93d2c19a545` |
| 0xb3c700 | 11 | `ev_d75f5ca00522b32e85df1a3fa060e7dd097d616af1c3239ba0705f171c0c00ca` |
| 0xb3bc20 | 304 | `ev_633d9455159f63d771900deb97946db3846b3597681e09bc2e298271f0991c4e` |
| 0xb3c140 | 98 | `ev_9ce59e876543d722da6b9d674fced28d23c38dcaae09d11b56b215222802f927` |
| 0x722530 | 3 | `ev_738352ec67766bfeefa034152dffa3fb5a20a4a02627027fd777daf1f0a99b46` |
| 0x722900 | 16 | `ev_ca25f5fb6d7205e492ade12e1a5de1772dac68d54d274946f85cb2b8aa6d0e96` |
| 0x722260 | 7 | `ev_456e8882442525095ec53eeb7347ad2755ef97c65e7479738c18050b2af44547` |
| 0xb38840 | 325 | `ev_6e7e36dd33a10d7b099af960ed66e342b75b454a7263ede43002cd3496b11798` |
| 0xb3cdc0 | 51 | `ev_41fb54043d58d14cfa8eb6c8fc6bf19ae63ed52c62f7acd0557ef3b1a11c4722` |
| 0x898ed0 | 1011 | `ev_fa6a88d0c840a5aa5f4d7a18b12ab84884271435f5e97e60d2cdaf2150e0ab9c` |
| 0x898780 | 1856 | `ev_6e153864422127d51939a48e50443e5bf8116419c2f8d9f6ec7b8531b9866ddc` |
| 0x898650 | 64 | `ev_ce77224a9bdce933493c71144d6354eb7dd792575b3be79cffd71af8e4f1810e` |
| 0x899e90 | 493 | `ev_aaea4dd4578fcc670bca0974a0cdf8e68fff3b54ada4ce185392d1151e804dee` |
| 0x899d80 | 223 | `ev_6360a35da24e329a29052cc6c5147e77cc09c5b0d9976636b77a28ef38517423` |
| 0x8a1a50 | 44 | `ev_f53c86c39bb0eaa575ff5ff9e16aa50e09eb2b2271d70b77299f2e9b4979d0c4` |
| 0x820b40 | 242 | `ev_b4b3d1a005241a2f3a360b0a68595204e5d3bc1139f77efaa9d8bd29a168d387` |
| 0x9e50b0 | 154 | `ev_7f352aca262f6060b748c0b2382c24332373d752ed9e903be613570aa89f05ad` |
| 0x8213e0 | 267 | `ev_0f494ec60ebc657987950718f33eeb3ffd237f203c920163a1d952621fe3cfab` |
| 0xb3c660 | 111 | `ev_666b2d425f99f55ebaefa73584a86da66c18f086c8e8466d973a26499d848392` |
| 0xb3bd80 | 163 | `ev_16ef5222631567d20be112bb5da6081a3400129a31241977ed1d1ab7414d78e2` |
| 0xb37d90 | 127 | `ev_31d29c39cedaa7da1b61741a5e5538d7da58c0f5e0d7797243b29495429e10e3` |
| 0x8a27b0 | 432 | `ev_af38fa54230f2ee50372d98ec41edfab7044004abe7baa35772aaa0365a6c269` |
| 0x8a2a70 | 377 | `ev_0f8c16b15b0d7f885d82b781b7c8d401adc5b96b8b94c5e1fb03d00e6ab32433` |
| 0x8a2f80 | 361 | `ev_b7f0e11f4096b0dbea81ab2fbd794a88c0bb7b480e0e89bfb90de5d0b47871b8` |
| 0x8a2cf0 | 642 | `ev_e309645c4b48f2c4ce1257ad9fe2aabcab671a8d40666bae9b4fb92a6e5dcd54` |
| 0x9e1370 | 417 | `ev_2028fc157364a25a234fac35416a9fd48a6cf4dc4a9d95faa52aacdf074091ec` |
| 0x9e56e0 | 533 | `ev_275c9e1d85ff0e6ffeee1c3b8b89c024b92a77ee2041456df93365f065bc6c17` |
| 0x9e5c90 | 747 | `ev_b37ba0635457168db8939b465c3c51b20dc0c6b41dcf106d0898ce9621b90ab6` |

| Containment observation | Evidence ID |
|---|---|
| 0x8a19f0 | `ev_d02c28d88fd8ce8e38afc3b752b00d19d761a4e0e3680bb4ed4c2f60a3a1d23d` |
| 0x8a30f0 | `ev_1cb68c1265cc03a32da5bc642159b6e1981c0377d46145bb79b0d229ad297c4d` |
| 0x8a1d30 | `ev_bd4472d693932bd694f89c8086ce8986a8f1b27f792c7bb3de721215fb8fcb92` |
| 0x8a1c10 | `ev_12208633963b31fb178971b5727389fae642177e9457736a85f7d51057f406ed` |
| 0x9cec60 | `ev_c1f4321cc528f0bb24a32acd6c96962db89b45a8d51404f67e99e0ad244eef3f` |
| 0x9e14c5 | `ev_25b7fdf8c49cfb9377088b733eed0c718f8bc80f7a589f4e2c769490a252d181` |
| 0x9d41c6 | `ev_4f35d55c63527ddbe7d5402e916dc1cb0a3fa6bbafed9acb1946ddc39a3e9e0a` |
| 0x9d5007 | `ev_3f9003613ba6afec95aff342b4888cc3d844b62b4461fc2fbb5a350bfd421634` |
| 0x9d5ad2 | `ev_845beacc1bdc2465fc0ed3bfe316388c793c25f8a4dc77120a2b6ffca991f656` |
| 0x9d5be1 | `ev_cb60c78fb6e8c641e071cc19e11c43539ad20450fc55a5ea9f5309321140bccf` |
| 0x9d5dff | `ev_ab8b0a77290f31804fb27219779bf160d75a3f60677e1fb507a3cc64d9ca5aeb` |
| 0x9d5eb5 | `ev_eef3341bd44a83cc7f4a5f6ad1556a3c305d1032be4e904dc5c518d1472669ea` |
| 0x9d624f | `ev_1e49fa1788ac3b6556bdbc73fc6c6852039d10aa895f4a1272b45009da093535` |
| 0x9db9cd | `ev_9edac057923c408d7086e1cacd5ec2e91710a959f0ff35674b54a45a2b5dc209` |
| 0x9dc67b | `ev_dd455d6d07f67668184b3f5f3aef9c5ece3f6ab2e08fd1b670896efa426a914b` |
| 0x9dc85f | `ev_bbfcb9e47d7c045c49892309e07204ad088fb219b6a591ddb173cf18fe308857` |
| 0x9e57aa | `ev_a89fc1b8cdbc3dfe53b402212ca83b60453be87d048a4c39a1b99ab9fd55354d` |
| 0x9e5f2f | `ev_2a1d10ae9b75c686fa25a2648e14df4bdff3ceaac89e55419c8e55b0a326be51` |
| 0x9dc663 | `ev_43a541515a9496727e72be755e9c3404c82208fa77b6a315448ad19a85581504` |

## Separately reused MAIN roots

| Entry | Evidence ID |
|---|---|
| 0x8a19f0 | `ev_f3a4ff37e3949c13ed452102ee534729623efb381bd564fe0da8330bbded3cd2` |
| 0x8a1ab0 | `ev_6d832c9d0196d2b1898603d93fd01bcd02fc3ba1df5750e51a235611f561c24c` |
| 0x8a1dd0 | `ev_0c7097265f16bc7a09c0d3f3f17e1b34cc3261cd3e00af9dbe41055da9d6ee1e` |
| 0x8a1d40 | `ev_10e965f99af0876e0d108b0ba4bfd7820f456137fa2b9dfde61ae54ea385f824` |
| 0x8a1d30 | `ev_3cfd8199a45a65e80dd1d832beb282b912162f8c06beeac5a98a9a30e231812e` |
| 0x8a1c10 | `ev_849e554ef2e3225eef34ad284e75a062591ffc3ec4f63338b8dfbbe1e59ee72b` |
| 0x8a30f0 | `ev_295a9ba6abb1926edf4a6b5e4cc827f166f4a84d714ca3efa0c8d65301b94b1a` |
| 0x9cb4e0 | `ev_4275b7d64861bda9d3f1369468c58deec344a973e3c6e12679479663f89ad441` |
| 0x9c8920 | `ev_4a5435ee59e170130751d04fb39f47a481fa7b5c82322bcd854db8e838d4e2cc` |
| 0x9e2aa0 | `ev_8517cbb679aeec97719d6893e9c703edabbd32bf7ff31ca240d710d42bb50bcc` |
| 0x9dd9b0 | `ev_0574d2fb38e2b73df2742d3b8962198eef271497c7cdfe3897846e84f6655b23` |
| 0x9e4330 | `ev_9d7dc073b11c44636f3419900ec9739205ea7ec609bdc7d5ad6edc9e00bdbbb2` |
| 0x9d1280 | `ev_6cceeb250290ce65b7355bac5612359eebe83836819b19fcb27ecc3a1a98b7a8` |
| 0x9d3db0 | `ev_83e67f6199a1a76051df09eebc8044ed8a07305f6d8d4781f3f2f0314abbfc2f` |
| 0x722270 | `ev_aa161205dcef806f5b9b3434ddc52fb69da13c6ce4614059ad175a72880e02ad` |
| 0xb3c8a0 | `ev_66a8baba7a05710c97a380f5126355bec11a5e5b24f11067f67ac54031f51bc6` |

**Native graph limits:** 0 new value-flow graphs are truncated. Others are not marked truncated, but external/indirect effects remain separately bounded. Zero known omitted edges does not make a truncated graph complete.

| Entry | Known omitted edges | Omitted operations lower bound |
|---|---:|---:|

## Verification, recovery and preservation

Independent checks remap all new inclusive bodies to original PE bytes, confirm **4,641 assembly instruction boundaries**, and verify **11,100 selected byte intervals**, plus the separately reused Native body/source hashes. Exact MAIN/Basic/Added vtables, raw short entries, registry routes and the 439-byte geometry helper retain their own source/limits. This is static evidence, not observation of a message executing or arbitrary pointer analysis.

The first owned JVM failed before bridge startup, producing zero new observations. Its diagnostics are preserved ignored; one fresh-process retry succeeded. A large completed b3c660 dossier exceeded the response-size transport and was recovered from its retained canonical export without reanalysis. No target/runtime or global toolchain change is part of recovery.

The previous **378 canonical records** are retained unchanged; the new export contains **465 records**. The saved snapshot contains **339 cached primitives**: 271 retained plus 68 new Native analyses. The 19 metadata observations do not add primitive entries. Ten separately archived records remain outside this bundle and are independently verified. Owned analysis resources are closed after export; raw files remain ignored under `client-data/audit/2026-11-01-sframe-main-pointer-flow-fc73ad12/`.

All **nine original-data preservation checks** pass. Original SFrame/archive, installed images, addon/settings, backup and existing primary service listeners/processes remain unchanged. No original-client execution, server restart, migration, character reset or gameplay mutation is performed. Seven documentation files are published through isolated Core/client worktrees; the original combined root index stays empty. Client/game version distinctions are preserved. No unrelated Rust build/test rerun is needed for this documentation-only investigation.

**Still unresolved:** the concrete nonzero MAIN+0x458 writer, complete effects of indirect child/context callbacks and aliases outside the bounded models. Reuse both the latest snapshot and the historical-individual index. The proven registry/owner paths now provide actual MAIN pointer provenance; a future activation finding must additionally establish the reaching write and actual nonzero byte. Do not implement an invented activation or turn bounded negatives into a claim that this client can never activate the mode.
