# SFrame notification actions and countdown state

Verified **2026-11-01** on original x86 SFrame SHA-256 **`27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`**, using **REA 6.4.0 / Ghidra 12.1.4 / Capstone 5.0.7**. This follows [the notification-window investigation](SFRAME_NOTIFICATION_WINDOW_RE.md). **Notification close-one and close-all input paths are identified. The original initialization hides the disable control; no persistent disable preference is established on these selected paths. The countdown comparison still does not identify MAIN alternate-mode activation.** New evidence: **11 Native dossiers / 1,785 analyzed bytes** and **2 other observations**. These are static artifact observations; the original client is not executed.

[Versions](VERSIONS.md) stay Core/RappelzRust **0.1.0 development**, NoLimits launcher **1.3.60**, original SFrame PE file/product **9.1.0.0**, embedded **Source9.5.2** build lineage, and network literal/profile **201507080**. Exact official Epic/patch remains unconfirmed.

## Input dispatch and registered actions

The previously identified **SUINotificationWnd / ID 0xd3 / vptr d5cb38** supplies **virtual+0x18→91c260** for the selected input path. This method consumes four stack arguments (**RET 0x10**), tests the second argument against **0x0a**, converts the first argument to a string key and searches the registry at **this+0x448**. A matching callback is invoked with the notification object as ECX and no additional explicit callback argument. The selected string registration through **91c6d0**, called by retained **91cbe0**, binds **`button_close_all_01`→91c9d0** and **`button_notify_close_01`→91d3e0** on the initialized path. This input registry is distinct from the kind-0x426 message registry at +0x450 and its pushNotification callback 91cf60.

The existing `window_main_notification.nui` resource has five buttons; their names alone do not establish five application actions. The selected direct construction passes the notification owner through descriptor+0xb0 and stores it at the created control's **+0x20**, using the retained **820f70→b35b60→b3ead0→virtual+8→9aad50→9a5170→b3c510** path. For a constructed simplebutton with the verified owner, the original input prefix at **9ab740..9ab7d0** checks input value **0x202**, button state and guard helpers before producing four physical arguments ending in event **0x0a** and the control name. New Native getter **7cd650** supplies that name. The exact simplebutton slot **+0x18→9a6d30** reloads the owner from control+0x20, requires it non-null and tail-transfers the same four arguments to owner.virtual+0x18. The button input prefix remains separately bounded raw coverage, rather than a complete new Native dossier. Successful control creation/lookup, the selected owner construction, unchanged parent/vptr/registry contents and input guards are conditions; arbitrary descendants or reparented controls are not globally typed. The original 9a6d30 tail-transfer opcode and register/stack preservation are checked separately from its decompiler warning about an unrecovered indirect jump; the selected owner binding does not resolve arbitrary dynamic owners.

## Close actions and hidden disable control

**Close all / 91c9d0** clears both notification lists at **+0x458 and +0x460** through **7df240**, and clears byte **+0x468**. The list helper resets the selected sentinel/count state and destroys the original string nodes; this is heap/list cleanup, not deletion of game characters or database items. If the notification visibility byte is set, the callback constructs an internal **SIMSG_SHOW_UIWINDOW / kind 0x3f9**, ID **0xd3**, state **0**, and queues it through retained **9eeb40**. This is a local hide request, not observed screen delivery or a network opcode.

**Close one / 91d3e0** removes the front node from the active list at **+0x458** through the separately anchored node destructor **7ddea0**. It updates the count and calls close-all when the active list was already empty or becomes empty; consequently that branch also clears any still-pending notifications. Otherwise it refreshes through retained **91d480** and resets the inspected notification timing state. Node/string lifetime, successful allocations, valid sentinels and downstream reentrancy remain explicit assumptions. Other input/animation branches are not inferred from the selected close paths.

Retained initializer **91bda0** supplies the exact name **`button_notify_disable_01`** and value **0** to inherited **b3d5c0**. That method finds the named control through b3c660 and writes its byte **+0x0c=0** on success. Thus this initialized path hides the disable control. The selected input map is initially empty and the inspected application registration supplies the two close bindings above; no disable callback or saved disable preference is established on this path. A hidden template control is not reconstructed as an implemented persistent setting. Runtime registry/parent replacement, other generic input handling and behavior outside the selected construction remain unknown. The raw 25-byte map initializer **91c380** and raw visibility helper **b3da20** remain explicitly separate from new Native dossiers.

## Continuing the alternate-mode investigation

The retained construction and actual callback registration bind the selected static **917aa0** path to **SUIArenaWaitingWnd**, separately registered as ID **0xd7**. At 917afa it forms outer+0x454; **DIVSS at 917b17** uses original float constant **100.0 at dffed0**, bytes **0000c842**, and **MOVSS at 917b1f** stores the computed remaining value. The source is the selected DWORD difference or original fallback, not a newly recovered network-time specification. New clock wrapper **613c30** computes an unsigned divide-by-10 of the result of **602280**, then adds global **ee9060** with DWORD arithmetic. The lower clock helper, global replacement, external clock rollover policy and external units remain unresolved; a divide constant alone is not a unit proof.

Retained updater **8fafa0** loads its first stack argument as a **DWORD from [EBP+8]**, maintains the previous value at **state+8**, and subtracts the unsigned difference divided by **1000.0**, original constant **d2b414**, bytes **00007a44**, from the float remaining value. Its recovered pseudocode's float argument type is not adopted as an ABI fact. Retained **8fb010** writes **byte state+4=1** and **DWORD state+8=0**; it leaves state+0 unchanged. The construction and update scales, 100 versus 1000, are kept distinct rather than silently normalized.

Containment **0x008faf40** returns entry **0x8fae50** (`ev_1a1d5961f208cbf221dff18eb0d14598b0cd4051060152390f183c1df23ee19e`). Containment **0x008faf60** returns entry **0x8faf60** (`ev_1bb7060180d84dc52b70e14181e0e601caed4bafa3c3b9da53f76c69da60a289`). The separately captured clear entry **8faf60** writes DWORD state+0=0 and byte state+4=0; it does **not** assign state+8.

The original span around **8faf40** contains a loop freeing **12-byte nodes** and a RET 4; it is not promoted into a countdown-constructor Native entry by proximity. Provider containment and independently checked bytes remain separate observations.

On the proven MAIN path, retained **8a3500** writes the supplied DWORD at **MAIN+0x454 at 8a354a** and indexes the child-pointer table **MAIN+0x44c+4×index at 8a3550**. This is a selected tab/control operation; the store/index occur before the later 0/1 branches, so a universal two-value index range is not inferred. The countdown float and MAIN selection DWORD are observed on different typed paths. Later representation changes, unions, aliasing or an unknown caller are not excluded merely because the field layouts are compatible.

**918630 still projects 8fb010 onto outer+0x458=1 and outer+0x45c=0, but its actual incoming target/registration and EDI receiver remain unbound.** The selected registered countdown callback and the equivalent warning text do not supply an inverse edge into this function. No new typed MAIN mode activation follows from the selected notification or clock/clear paths.

New Native **917f00** supplies the exact incoming clear call **917fc7**, with ECX formed from its preserved outer receiver+0x454, **only in the selected branch where 71f5b0 returns AL=0**. Its direct caller is new Native **916a60**, through **CALL at 916ab5** on the nonzero-argument path. Exact original table slot **d5c854 = d5c778+0xdc** selects 916a60 for the constructed **SUIArenaWaitingWnd**. Thus this selected clear path has actual class/slot/call provenance, instead of a class assignment by neighboring address. The nonzero branch also constructs internal kind **0x524** with ID **0xd3**, RTTI **SIMSG_UI_LIST_MOVE_FORCE_BACK**, and the inspected continuation queues kind **0x9c41**, RTTI **SIMSG_UPDATE_MAIN_MENU_BUTTON**. Their later dispatch effects remain unclosed; these are local internal-message observations, not new server opcodes or a MAIN character-sheet assignment. The two new caller bodies supply no observed direct CALL/data-reference/immediate target into 918630; indirect, mutable and transitive effects remain open.

## Evidence and verification

All new records bind the original target SHA-256 and profile digest **`b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17`**. All new Native value-flow graphs report no truncation. Complete retained records, selected original-byte checks and limits remain distinct from complete runtime execution or interprocedural effects.

| New Native entry | Analyzed bytes | Evidence ID |
|---|---:|---|
| 0x91c260 | 253 | `ev_b60e936ac6b178ac83ee661a1c194fd67fbe1fa2fc9a2075e96d87b6b3bd79fe` |
| 0x91c6d0 | 138 | `ev_c12df1cb07b7a3401b1f455a01d0b42606f161a94844431ba2e9c2439bd7744e` |
| 0x91c9d0 | 145 | `ev_fe7ec44b8aecb3b1c237e2278842435eccc85c1e3940473747675186f88be77a` |
| 0x91d3e0 | 159 | `ev_8f01f8e1d8866d8b6ba208e0840fa708cb0dd1ae23ad8dd0b2e3b6ebc9100695` |
| 0x7df240 | 126 | `ev_468b795f7b6a05038c9df72fc3405a334f8b6b8c6680fe8c411ec2a7a5e152e1` |
| 0x613c30 | 26 | `ev_0322c62607ab8f873abcb6a7daa9b3822688caafd20e73b52cbe8d28af7325cb` |
| 0x9a6d30 | 20 | `ev_da28020bc8990a583695f5db99fa2520c3a4a7a6800e4676f11667dfed0b70d2` |
| 0x7cd650 | 37 | `ev_d6ba758db3b061f7d151fb3434ddf5dadf748b1b48b3de1ea73aa2bd5b7d20f9` |
| 0x8faf60 | 11 | `ev_b68b9eaac95cf0f480f057f291993e443d9e54cfc10c7bbed8ae2f8beef7cbe3` |
| 0x917f00 | 519 | `ev_f8164952ca2b6eca04668d8cdf93d49dd7341df5d04ba07cca54d9fdcf5d576d` |
| 0x916a60 | 351 | `ev_008fb1f8d774e53f8e18a90c323e9867ddeeadab0024d0854c2e4dbdaf70e12d` |

| Other operation | Parameters | Evidence ID |
|---|---|---|
| resolve_containing_procedure | `{"address": "0x8faf40"}` | `ev_1a1d5961f208cbf221dff18eb0d14598b0cd4051060152390f183c1df23ee19e` |
| resolve_containing_procedure | `{"address": "0x8faf60"}` | `ev_1bb7060180d84dc52b70e14181e0e601caed4bafa3c3b9da53f76c69da60a289` |

| Selected reused Native entry | Evidence ID |
|---|---|
| 0x820f70 | `ev_bf6a01b950ce19ea40f7d2a5709725e5431a157e569999caec2f85e087de96e0` |
| 0x8a19f0 | `ev_f3a4ff37e3949c13ed452102ee534729623efb381bd564fe0da8330bbded3cd2` |
| 0x8a1dd0 | `ev_0c7097265f16bc7a09c0d3f3f17e1b34cc3261cd3e00af9dbe41055da9d6ee1e` |
| 0x8a23d0 | `ev_550dc95b0cabda6eff467bede9e41278f11fa94fcef1bb72d91e222a2bcbdcc0` |
| 0x8a3500 | `ev_a4b453f0e7c50f8457c67db0e7c4379d6645e67044af83a62c0e95472cc3a20c` |
| 0x8faf70 | `ev_2438ea2ed528ccdf2c4e457359892f32d22dab8e4567ce96ac284282fe718e33` |
| 0x8fafa0 | `ev_da809073bd3cb75b6fe68756f7a1fe446bf0150b6c6bf804bd8db6c5b34c4433` |
| 0x8fb010 | `ev_6311d6edbba44308e24bffdc741ec9c6fbccbb16ca26c14b03969b080907bce5` |
| 0x916550 | `ev_596c3d917c90a0709cbb04e922e15feda7569b25cb26d777229825c4cde8e735` |
| 0x916940 | `ev_85b88ee032d7fbd1fc641566e16248d2e186783fccea7b76df953b9f7a705de8` |
| 0x916bc0 | `ev_24d7024d1352d49ab5b8609e5cd1bca0e7242999e5cea6b8bb7d76ff6e2020f2` |
| 0x916c20 | `ev_f798fb6a88410dc78b49df4cd35c5c220e4e99768fdc15de5da1adfaa5039e62` |
| 0x917220 | `ev_4a91334aea05853adaaa19f600f34a1424ea5dda05858e07416d0782959d6cae` |
| 0x917500 | `ev_30bc9b7b18b248bd3855f1f1c933d9f89ac652d78cc43105cecedda6548e1ee1` |
| 0x917980 | `ev_6f667ccfb3e130926af3c52d22c1e224e43b001a72658002aaf68a586c4e976c` |
| 0x917aa0 | `ev_5c5e2e76bb180562efa78127363e7cf4c687dc112ad925f6c515dde17249fb27` |
| 0x918630 | `ev_2e11babc974a7b4241ded6116bc72be4cbafa3d434f034fc40c37e79a04da7bc` |
| 0x91b7f0 | `ev_0665be855913f559ae733d854da05cb6873e4ecd16ab372d07f88f69f00284b1` |
| 0x91bda0 | `ev_1080538886678a091af237ab8d8b53da58fca355129b1d244723722404527adb` |
| 0x91c050 | `ev_5935fb1acc149e4bcd4f4383bcd31c732d8f226ebb70f8ab6fcabbd086feaa11` |
| 0x91c0b0 | `ev_e3a99607bd2f6ee9491dc8fb53e83c657ca2c3006ec9e9e883f4f3ea54dca091` |
| 0x91c760 | `ev_ecf8f2eedb0b118331dca662e31e74fef706a60ea6f0501ad25033e19018d703` |
| 0x91cbe0 | `ev_cee916d8a7708806a8cebb30b2781eea8e9f227ed90c10e90fb3677af7bc9982` |
| 0x91cf60 | `ev_917a4e08f9a23e312d49594d1d5ed558f41e8ee89b7f8997f4f9ab0bb6a06e1d` |
| 0x91d480 | `ev_6f3bd19e103d5979a85861e1ca295b2fd4aa45a98802f45c7d5d493ceb4bbc5f` |
| 0x992560 | `ev_3c1a474192eb87115f652b05bc150d0a4a9447bd06d3c582577c080a338c88b0` |
| 0x9a5170 | `ev_ee83ed194b19c037d963a597ac72ac4ac55c2c284cd2f407cca0946112ec93fb` |
| 0x9aad50 | `ev_9f5893ba70700d907047002b420f9928393cab547a6105c8fd6c9d274bc16aba` |
| 0x9e47b0 | `ev_7aaf53f690b1adb712ad0153b9adcd3f00d5dca97844e44cac215663b2d2ed27` |
| 0x9eeb40 | `ev_f1dc736455bf4e490116d08bee86f74588317ce0d3d8eefdaa12e0e7415c9aee` |
| 0xb35b60 | `ev_780cd463050f07ecb6fc2124376fbcb2e95dd2a03b4f84435dcbface76076ff5` |
| 0xb3c510 | `ev_6356bc8bbb36f8a8053edebbc871fdb724d632f03e1057a807c86f753eaca43d` |
| 0xb3c660 | `ev_666b2d425f99f55ebaefa73584a86da66c18f086c8e8466d973a26499d848392` |
| 0xb3d5c0 | `ev_d1b329ccac2f0c8aa70c09370d1e18e377ec379310b8a71b7c137859ab916624` |
| 0xb3ead0 | `ev_5f57d5eeeef5a105042b0ceec3f79df05293b720c9fe47ae03fe6cc106e0f02f` |
| 0xb3ec90 | `ev_ddc5747a3d5b8e234861260a575d82bcbd6db31a663dc92f6dd55a7ae6e2c860` |

Independent checks bind every new Native body to original file bytes, verify **578 new assembly boundaries**, **188 selected provenance intervals**, and **36 separately retained Native bodies**. The two action and countdown ledgers preserve their own hashes and limits. Overlapping byte checks are corroboration, not separate game executions.

The owned Native session checkpoints **13 new observations**, officially imports the preceding **609-record canonical bundle**, exports and closes. All preceding canonical records remain equivalent as JSON values. The final bundle holds **622 records / 283,606,320 bytes**. The closed snapshot holds **32 cached primitives / 296,319,071 bytes**, SHA-256 **`7bb179a4529053914e74453daa2cd205f6794fe27550a3230d0419034b4962b3`**. The historical Native index retains complete older records outside this canonical subset.

Ignored local run: `client-data/audit/2026-11-01-sframe-notification-actions-8f6d2c41`. Complete records, original-byte/resource provenance, two ledgers, checkpoint, snapshot, preservation and publication receipts stay local.

**Nine preservation checks pass.** Eleven baseline executable, addon and settings hashes, the saved-variable tree, logical database backup and primary service identities remain unchanged. The root Git index stays empty. Only seven authored Markdown files are synchronized through isolated Core/Client worktrees; original assets/binaries, databases, credentials, settings and generated decompiler output are not published. Gameplay code and package versions are unchanged.

## Restart point

Notification input dispatch, the two selected close callbacks, their cleanup and the initialized hidden disable control are answered statically. The countdown fields are compared only on proven receiver paths. MAIN activation still requires an actual reaching target and receiver for 918630 or a typed MAIN pointer reaching a nonzero mode write. Reuse complete new records and the two ledgers; do not repeat the empty caller query, earlier broad pointer/carrier scans or retained producers. Preserve boundary, ownership, mutable-map and time-unit limitations.
