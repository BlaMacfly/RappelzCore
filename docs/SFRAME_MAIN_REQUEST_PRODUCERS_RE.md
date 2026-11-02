# SFrame MAIN request producers and command bindings

Verified **2026-11-02** on original x86 SFrame SHA-256 **`27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`**, using **REA 6.4.0 / Ghidra 12.1.4 / Capstone 5.0.7**. This continues [the show/list-membership investigation](SFRAME_MAIN_LIST_MEMBERSHIP_RE.md). **Concrete compiled producers of MAIN requests are identified. Event 10 with button_character in SUIMainMenuSubWnd, or button_common_quick_character in SUITaskBarWnd, constructs SIMSG_TOGGLE_UIWINDOW/kind 0x3f8/ID 0x2e. MAIN's button_close handler requests visibility 0; SUIInstanceEntranceWnd button_result requests visibility 1 and queues a separate score request whose UI handling can select tab 1. The requests are queued, so actual pumping, destination and UI guards remain conditions. No nonzero MAIN+0x458 activation or reaching receiver of 918630 is established.** This pass adds **4 Native dossiers / 7,153 analyzed bytes** and **9 other observations**. Old message dispatch, constructors, renderer and pointer-consumer evidence are reused. No original-client execution is claimed.

[Versions](VERSIONS.md) remain Core/RappelzRust **0.1.0 development**, NoLimits launcher **1.3.60**, SFrame PE file/product **9.1.0.0**, embedded **Source9.5.2** lineage and network literal/profile **201507080**. Exact official Epic/patch remains unconfirmed.

## Typed message producers

Two exact message classes are kept separate: d31548 is SIMSG_SHOW_UIWINDOW, and d32154 is SIMSG_TOGGLE_UIWINDOW. The bounded inventories cover 778 Show and 31 Toggle literal-vptr carriers, selecting three Show and two Toggle constructions with literal window ID 0x2e. Show stores kind 0x3f9 and bytes17/18; Toggle stores kind 0x3f8 and has no inferred Show argument bytes. Generic/computed IDs are not counted as MAIN. The second result-button object is named SMSG_INSTANCE_GAME_SCORE_REQUEST and stores kind 0xa4/byte13=1; its name alone proves no network transport.

All five selected producers now have complete Native bodies. Original SUIInstanceEntranceWnd virtual+0x18=7cc060, event 10 with name button_result, queues Show ID2e with bytes(1,0), followed by the distinct A4 score-request object. Original MAIN SUICharacterInfoWnd virtual+0x18=8a23d0, event 10 and button_close, queues Show ID2e with(0,0). Helper 80a190 queues five Show(false) requests for IDs 0x1a/0x24/0x25/0x47/0x2e; its actual raw-qualified caller 80a830 selects it for kind 0x41d and byte17=0, but that receiver class remains unbound and is not called a user click. Original SUIMainMenuSubWnd virtual+0x18=91f400 uses event 10/button_character; SUITaskBarWnd virtual+0x18=920fb0 uses event 10/button_common_quick_character. Both queue Toggle ID2e. mainmenu_character is not the compared string in those two Toggle branches.

The handlers require their actual selected receiver/table and lexical event guards, normal completion of preceding helpers, valid managerpointer at receiver+0x3e8, and successful allocation. 9eeb40 marks message+0x0a=1 and enqueues its pointer in manager+0x44, with growth 9ed360; it does not directly dispatch to the UI. Eventual pumping, actual destination Interface, modal/routing guards and lookup2e identity remain conditions. On the known UI path, A4 is forwarded to visible window2e and MAIN.virtual+0xcc: nonzero byte13 selects 8a3500(1,1) when tab454 is not already 1, writing tab454=1. This is a conditional UI effect of the named score request; other pump recipients and transport remain unproved.

The 778/31 domains are separate finite exact-vptr inventories with bounded field-expression rules, not all possible Show/Toggle producers. Copied/computed vptrs and IDs, distant stores, generic parameters, aliases and runtime replacements can lie outside the model. Constructed sender/resource bindings and preserved original tables remain conditional, with no live input observation. Queue capacity, allocator/copy contracts, callbacks, pointer lifetimes, ordering and reentrancy are not declared safe or pure. Visibility transitions invoke 8a1d40 and clear MAIN+0x458/+0x45c; an equal-state Show can bypass that callback. A4 selects tab454 and is not promoted to a nonzero458 writer or alternate-mode activation. No actual caller/receiver of 918630 or synchronous delivery is established. Four fresh Native graphs are not truncated; retained graph bounds and raw-only caller qualification remain distinct from complete original body checks.

## Resource controls and compiled command bindings

The original resource table used by 9c8920 maps UI registration ID 0x14 to window_main_menu_sub.nui, ID 0x89 to window_main_taskbar.nui, and request target 0x2e to window_player_character_info_main.nui. Four selected archive payloads equal their caches byte for byte. The first two contain the exact controls button_character and button_common_quick_character. Their geometry/type declarations alone are not the callback proof.

Two compiled producers are now tied to constructed types and resources. Native factory 9cb4e0 constructs SUIMainMenuSubWnd with vptr d6b8d4 and registers that same object as ID 0x14; its original slot +0x18 selects 91f400. The factory constructs SUITaskBarWnd through the provider-described 72-byte constructor 920af0, which installs d5d188; registration ID 0x89 and original slot +0x18 select 920fb0. Both require physical notification argument 2 equal to 10 and a zero c8f010 name comparison against their respective exact control IDs. No specific mouse gesture or active key assignment is inferred from notification 10 alone.

On those selected branches and successful 23-byte allocation, each handler builds SIMSG_TOGGLE_UIWINDOW with vptr d32154, internal kind 0x3f8 at offset +4 and window ID 0x2e at +0x13. Shared sites 91ff8c and 92193b enqueue through 9eeb40 using the menu context field +0x3e8. The known consumer routes an intact registered MAIN lookup to its visibility callback, which clears MAIN+0x458/0x45c on a transition. These are compiled conditional button bindings, not observed execution or a guarantee of delivery. Show d31548/0x3f9 remains a distinct message domain.

Construction/allocation, resource lookup, compatible live objects, unchanged registry/vptrs/names, normal ABI and deferred message processing are conditions. Active key maps, current mouse gestures, arbitrary aliases, indirect calls, exceptions and reentrance remain open. The reused factory 9cb4e0 value-flow graph is truncated; its complete original body and selected instruction boundaries are independently verified. The two new producer graphs are untruncated. Raw constructor/init spans are bounded evidence, not additional Native dossiers. No original runtime or data changes occur in this analysis scope; the previous .text/PDB/0x918630 inventories are not repeated.

## Mode boundary and delivery limits

The two menu handlers produce internal Toggle requests for key 0x2e under event/name guards; they do not directly store MAIN's mode. The result-button route first queues ShowMAIN(true), then kind 0xa4 with byte13=1, whose exact RTTI is SMSG_INSTANCE_GAME_SCORE_REQUEST. Retained routing checks MAIN visibility and forwards that request to MAIN.virtual+0xcc; the selected 8a1dd0 branch may call 8a3500(1,1) to choose tab+0x454. The class name does not prove a network transport or response. That UI-side operation must not be conflated with a nonzero byte+0x458 write. Ordinary visibility transitions retain the previously verified explicit mode/handle clears. Retained 918630 still has an unbound entry receiver and calls 8fb010 at 918726, projecting outer+0x458=1/outer+0x45c=0 without a typed MAIN reaching definition. Actual mouse/key execution, unconditional queue delivery, and alternate-mode activation are not observed.

## Evidence and verification

All new records bind the target SHA-256 above and profile **`b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17`**. New graph truncation: none. Reused graph truncation: 0x9cb4e0 (0 known omitted edges; omitted-operation lower bound 1), 0x9d3db0 (240 known omitted edges; omitted-operation lower bound 1). Complete body bytes do not imply complete high-pcode graphs, resolved aliases or whole-program effects.

| New Native entry | Analyzed bytes | Evidence ID |
|---|---:|---|
| 0x7cc060 | 1035 | `ev_40b42b6cc9a9efd676342dc60f7579de4be544e53f94068a657bf36d17d6b879` |
| 0x80a190 | 562 | `ev_220288460d693e8ac9a09dc0ec0f883f405156527def308f287a6cbd08e0b82a` |
| 0x91f400 | 3072 | `ev_a932f80d825c4f52e566d0cc908db4a4e20484fe28e145d7cd94fb5e9f18554d` |
| 0x920fb0 | 2484 | `ev_4054b57b824de1bfe936916c49f84388afa75ff65575757bffecbdcaaf12da34` |

| Other operation | Parameters | Evidence ID |
|---|---|---|
| resolve_containing_procedure | `{"address": "0x7cc3d9"}` | `ev_e6f56d11a4df56c43366a0282cb0b66ce16c32b52a7b0172db707fddc0173434` |
| resolve_containing_procedure | `{"address": "0x80a377"}` | `ev_d0eac984a5c597037f6d4b834f960288d96b9f4282702c9cf7f04e226c95a0fa` |
| xrefs | `{"address": "0xd5d11c"}` | `ev_f3aa51f5d362732de6cf55c07abed04586627497c34c892ac445d88289463eae` |
| xrefs | `{"address": "0xd50000"}` | `ev_abb3c69db4dcf56ca77989564a39f957fb85db2d73c307660aa2b7c7c51aca17` |
| xrefs | `{"address": "0xd5d364"}` | `ev_10e1d4f74d81d3d06d8c186ff9ceabb777fb6ff61f686e8439fb65d0cb2266aa` |
| resolve_containing_procedure | `{"address": "0x91f489"}` | `ev_7e0c3dbe154e1c2d2b2f1919308fa9c5cbda332ad763e10584e1e5e11935d2f3` |
| resolve_containing_procedure | `{"address": "0x921034"}` | `ev_196a0c09d614ac5d1d03b0c627ac43f15f2def2e173decb02f935e960e11b9ca` |
| resolve_containing_procedure | `{"address": "0x920107"}` | `ev_36d3ac321608cfbd88e9e2b0d29c733367ec89ae9dd4675b3a219eeffa5d14ba` |
| resolve_containing_procedure | `{"address": "0x9219cf"}` | `ev_7683a467789599a16f698cebd0dab15ff404eb37f839b4802d54ae159ffb926f` |

| Selected reused Native entry | Evidence ID |
|---|---|
| 0x820f70 | `ev_bf6a01b950ce19ea40f7d2a5709725e5431a157e569999caec2f85e087de96e0` |
| 0x8213e0 | `ev_0f494ec60ebc657987950718f33eeb3ffd237f203c920163a1d952621fe3cfab` |
| 0x8a19f0 | `ev_f3a4ff37e3949c13ed452102ee534729623efb381bd564fe0da8330bbded3cd2` |
| 0x8a1d40 | `ev_10e965f99af0876e0d108b0ba4bfd7820f456137fa2b9dfde61ae54ea385f824` |
| 0x8a1dd0 | `ev_0c7097265f16bc7a09c0d3f3f17e1b34cc3261cd3e00af9dbe41055da9d6ee1e` |
| 0x8a23d0 | `ev_550dc95b0cabda6eff467bede9e41278f11fa94fcef1bb72d91e222a2bcbdcc0` |
| 0x8a3500 | `ev_a4b453f0e7c50f8457c67db0e7c4379d6645e67044af83a62c0e95472cc3a20c` |
| 0x9c8920 | `ev_4a5435ee59e170130751d04fb39f47a481fa7b5c82322bcd854db8e838d4e2cc` |
| 0x9cb4e0 | `ev_4275b7d64861bda9d3f1369468c58deec344a973e3c6e12679479663f89ad441` |
| 0x9d1280 | `ev_6cceeb250290ce65b7355bac5612359eebe83836819b19fcb27ecc3a1a98b7a8` |
| 0x9d1710 | `ev_ccb6e3c2166313ebebdb88ff09974a9d2fc9527191043e290b00632e4b8a6fa4` |
| 0x9d3db0 | `ev_83e67f6199a1a76051df09eebc8044ed8a07305f6d8d4781f3f2f0314abbfc2f` |
| 0x9dd9b0 | `ev_0574d2fb38e2b73df2742d3b8962198eef271497c7cdfe3897846e84f6655b23` |
| 0x9eeb40 | `ev_f1dc736455bf4e490116d08bee86f74588317ce0d3d8eefdaa12e0e7415c9aee` |
| 0xb3da20 | `ev_7a6e5bd724d51e89d5bd91d5a371f5b813d624cb2c2b6636b1dd37a41df7d2e4` |

Independent checks bind each new body to original file bytes and verify **1,747 new assembly boundaries**, **171 selected provenance intervals**, and **18 reused Native bodies**. The producer and binding ledgers retain source-file hashes, exact request and pointer deliveries, original-byte checks and their limits. The prior .text dispatch inventory and constructed SGameViewPort route are reused, not repeated as new analysis.

The owned Native session checkpoints **13 new observations**, imports the preceding **681-record** bundle, exports and closes. All prior records remain equivalent as JSON values. The canonical bundle holds **694 records / 317,440,716 bytes**. The saved snapshot holds **25 cached primitives / 334,182,387 bytes**, SHA-256 **`e58370959b87a78f48f4873670731515a3292b845486cc7aa4cd36d71c5fa865`**. Provider close completed successfully. Historical Native records outside this canonical subset remain indexed separately.

Ignored local run: `client-data/audit/2026-11-02-sframe-main-request-producer-a7e4c913`. Complete records, both ledgers, source-byte checks, checkpoint, snapshot, preservation and publication receipts remain local.

**Nine preservation checks pass.** Eleven baseline executable, addon and settings hashes, the saved-variable tree, logical database backup and primary service identities remain unchanged. The root Git index stays empty. Only seven authored Markdown files are synchronized through isolated Core/Client worktrees. Gameplay code and versions are unchanged; original binaries/assets, databases, credentials, settings and generated decompiler output are not published.

## Restart point

Reuse this run's snapshot, producer/binding ledgers, separate Show and Toggle inventories, resource provenance and historical Native index. The two class-qualified menu commands, MAIN.button_close and the result-button Show/tab requests are closed under their explicit guards; do not repeat the producer search as wholly unknown. Any remaining physical keyboard binding, generic IDs, dynamic request copies and callback effects retain their bounds. Continue from an actual MAIN-bound nonzero mode writer or a reaching entry/receiver for 918630. Preserve queued-delivery versus synchronous-dispatch, tab454 versus mode458, cleanup-helper versus user-handler, resource/class construction, modal/map/list, lifetime and reentrance distinctions. The third legacy mainmenu_character control is not equated with the proven producer branches merely by its name. Do not repeat the old PDB, incoming, pointer, .textCALL14, viewport, list or clipping investigations.

## Follow-up: mode-transition audit, 2026-11-02

The [activation audit](SFRAME_MAIN_MODE_TRANSITION_AUDIT.md) revisits the remaining flag hypothesis. [MAIN mode-transition audit](SFRAME_MAIN_MODE_TRANSITION_AUDIT.md): retained typed reads/clears and the unbound 918630 receiver are revalidated. No nonzero MAIN activation or usable alternate-mode trigger is established; prior caps and the Arena counterexample remain explicit. Zero new Native analyses/queries; prior 694-record bundle and snapshot reused. Nine preservation checks pass. Documentation only.
