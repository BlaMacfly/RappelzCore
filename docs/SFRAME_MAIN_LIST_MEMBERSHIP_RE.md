# SFrame MAIN show requests and render-list membership

Verified **2026-11-02** on original x86 SFrame SHA-256 **`27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`**, using **REA 6.4.0 / Ghidra 12.1.4 / Capstone 5.0.7**. This continues [the render-entry investigation](SFRAME_MAIN_RENDER_ENTRY_RE.md). **A qualifying internal request now links the actual MAIN registry value to the rendered window lists: ID 0x2e lookup, hidden-window insertion into KUIWndManager+0x18, visibility delivery and the known window-render callback. Toggle and explicit show/hide contracts are distinguished from initial mode0 registry creation. This is a selected static request path; its actual input producer and live invocation are not established. The alternate-mode trigger and reaching receiver of 918630 remain unresolved.** This pass adds **7 Native dossiers / 1,497 analyzed bytes** and **0 other observations**. Old message dispatch, constructors, renderer and pointer-consumer evidence are reused. No original-client execution is claimed.

[Versions](VERSIONS.md) remain Core/RappelzRust **0.1.0 development**, NoLimits launcher **1.3.60**, SFrame PE file/product **9.1.0.0**, embedded **Source9.5.2** lineage and network literal/profile **201507080**. Exact official Epic/patch remains unconfirmed.

## Request entry and visibility delivery

The retained 9d3db0 dispatcher maps internal kind 0x3f8 to 9d9694 (toggle) and 0x3f9 to 9d97a9 (explicit visibility), verified with original selector/target table bytes. For window key 0x2e and the normal message+0x18==0 branch, 0x3f9 calls 9d1710 with the requested byte from message+0x17. The alternate message+0x18 branch excludes key 0x2e. These are static internal UI paths, not recovered runtime gestures or network opcodes.

9d1710 looks up the window through 9e4330 and returns if it is absent or its visibility byte already matches the request. On a transition it calls Interface+4 removal b3ff50 or insertion b3e360(window, insertionFlag, 0), writes window+0xc through b3da20, then calls window.virtual+0xdc(newVisibility, 1). With successful intact selected registration for key 0x2e, this selects actual MAIN callback 8a1d40. The same dispatcher also forwards selected key 0x2e messages through 9dd9b0 to MAIN.virtual+0xcc.

MAIN callback 8a1d40 first invokes 8213e0 for rectangle clamping and associated-window hide messages, then clears MAIN+0x458 and MAIN+0x45c on both visibility branches. 9e1160 receives the window afterward, but it first excludes the exact name window_simple_summon and requires a matching name in Interface+0x108..0x10c. MAIN membership in that filter is unproven, so no actual MAIN capture in the separate Interface+0x100 bookkeeping list is claimed.

The new Native caller metadata is checked against five bounded original-byte bodies. Observed 9d1710 callers use IDs 0x2b (9e0c20) and 0x6a (9e22a0); 9e0c20 also constructs a 0x3f9 message specifically for ID 0x37, visibility 1. Postdelivery caller 9c4a80 uses IDs 0x15, 0x1f, 0x2c, 0x38 and 0x39. Wrapper 9d17d0 accepts a generic caller ID. These selected sites provide no new concrete MAIN ID 0x2e producer, and computed keys or aliases remain open. Raw-only node helper 9bdb00 stores the provided Window value at node+8 after an unexpanded allocation helper; MAIN capture remains conditional on the unresolved name filter.

All conclusions are static and conditional on successful registration, unchanged key mapping, compatible live objects, normal ABI and intact pointers. Callbacks, queued messages, aliases, reentrance, exceptions and later writes remain open. The reused 9d3db0 value-flow graph is truncated (240 known omitted edges, at least 1 omitted operation); complete selected original assembly/body checks remain separate. No original runtime, data or service changes occur in this analysis scope.

## From the MAIN registry value to manager lists

The actual CharacterInfo construction calls 8a19f0 and 9c8920 with ID 0x2e and mode0. Its virtual+0xd0 target 8a1ab0 returns the same preserved object; 9c8b47 passes it to 9e2aa0. The registry creates an 8-byte record {value=MAIN,key=0x2e} and stores a pointer to that record in the primary bucket at Interface+0x24. The same 9e50b0 helper used by 9e4330 compares record+0x04 with the requested key and returns record+0x00. Initial mode0 creation therefore proves registry binding under successful, nonduplicate insertion; it does not itself add MAIN to a manager render list.

A real first-membership route is identified under an explicit input contract. Once its modal prelude and preceding Interface callback permit routing, retained dispatcher 9d3db0 maps message kind+4=0x3f8 to 9d9694. It looks up DWORD message+0x13 with 9e4330. If that ID is 0x2e, the registry still identifies the selected MAIN, and MAIN byte+0x0c is zero, 9d96d7 calls b3e360 on [Interface+0x04] with physical arguments (MAIN, context_field228!=1, 0). b3e360 searches U+0x18 for the exact window pointer and inserts that pointer if absent, using a 12-byte node whose payload is copied at +0x08. The separate visibility helper is called afterward. No specific button producer or live input is inferred.

The constructed Interface+0x04 manager and SGameViewPort context were qualified in the preceding owner ledger. The new insertion specifically adds MAIN to U+0x18. Retained b400a0 visits that list, loads each window from node+0x08, and calls window.virtual+0x14 at b4010f when U+0x58 and MAIN+0x0c are nonzero, MAIN+0xf8 is zero, and MAIN+0x15c is zero. The selected original MAIN table d53af8 chooses b376a0; its context.virtual+0x28 handoff and the constructed SGameViewPort drain retain the previous qualified path. The membership and guards are conditional static facts, not a live-session observation.

The input must identify the actual preserved Interface and the same registry record; an existing ID 0x2e causes 9e2aa0 to reject the new binding. Allocations and container invariants must hold. Callback and focus side effects, reentrant list changes, pointer replacement, duplicate membership across different lists and object lifetime changes remain possible. Removal b3ff50 matches pointer equality and frees only selected 12-byte nodes; its name comparisons clean manager+0x3c and do not choose a different window to remove. It does not remove every possible list membership or directly destroy MAIN. Primary bucket growth preserves old elements under the c5e9a0 copy/move helper contract and valid buffer ownership; this is not an independent proof of that helper or allocator behavior. Native pcode bounds remain separate from complete original source-body checks. No runtime, data or service changes are performed.

## Mode boundary and unresolved producer

The ordinary visibility delivery reaches the constructed MAIN table's virtual+0xdc entry 8a1d40. Its two branches contain explicit clears of MAIN byte+0x458 and DWORD+0x45c; the show branch additionally invokes retained refresh helpers. This is not a nonzero mode setter or a proof that all callbacks are pure. A repeated explicit visibility request need not re-run the visibility callback when the visible byte already matches. No keyboard/button producer specifically creating a 0x3f8/0x3f9 request for ID 0x2e is identified here. The newly inspected direct callers produce other IDs or preserve a generic input ID; their bounded absence of 2e is not whole-program absence. Retained 918630 still has an unbound entry receiver and reaches 8fb010 at 918726, projecting outer+0x458=1/outer+0x45c=0 without a typed MAIN reaching definition. No alternate-mode activation or original-client execution is reconstructed.

## Evidence and verification

All new records bind the target SHA-256 above and profile **`b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17`**. New graph truncation: none. Reused graph truncation: 0x9cb4e0 (0 known omitted edges; omitted-operation lower bound 1), 0x9d3db0 (240 known omitted edges; omitted-operation lower bound 1). Complete body bytes do not imply complete high-pcode graphs, resolved aliases or whole-program effects.

| New Native entry | Analyzed bytes | Evidence ID |
|---|---:|---|
| 0xb3e360 | 195 | `ev_87d75375a4aa6c5af185257d3d40e95eb1ddac07ad2556f4acdca3d9d521e7a1` |
| 0xb41540 | 68 | `ev_074393d050b9f71f92cd4737217c42029bf4195465cc4399f2cba1640a317669` |
| 0xb3da20 | 25 | `ev_7a6e5bd724d51e89d5bd91d5a371f5b813d624cb2c2b6636b1dd37a41df7d2e4` |
| 0x9d1710 | 96 | `ev_ccb6e3c2166313ebebdb88ff09974a9d2fc9527191043e290b00632e4b8a6fa4` |
| 0xb3ff50 | 334 | `ev_3f2c9509fe93efca9a6e70c097dc37ab5d6c22a5473029ccf6a89b969e62f828` |
| 0x9e1160 | 433 | `ev_5c21a09d77994db51d1725dedf39905994c1bf8ccc1a6f86f7dc130ebc611a65` |
| 0x9be0a0 | 346 | `ev_e6938f42423b961f2cf7cb43601f990d1870da8c2446de0a133b84354007ed5b` |

| Selected reused Native entry | Evidence ID |
|---|---|
| 0x7438f0 | `ev_9c938740e30b1b4870dba81627eb9b2dc889056a8d1a8fc3dcbef9a26e5d8d3c` |
| 0x749890 | `ev_5a50426fad3bfcecc56f1c09d15e07368e2296cb446ea921b1ea77d9ae37cfb2` |
| 0x8213e0 | `ev_0f494ec60ebc657987950718f33eeb3ffd237f203c920163a1d952621fe3cfab` |
| 0x8a19f0 | `ev_f3a4ff37e3949c13ed452102ee534729623efb381bd564fe0da8330bbded3cd2` |
| 0x8a1ab0 | `ev_6d832c9d0196d2b1898603d93fd01bcd02fc3ba1df5750e51a235611f561c24c` |
| 0x8a1d40 | `ev_10e965f99af0876e0d108b0ba4bfd7820f456137fa2b9dfde61ae54ea385f824` |
| 0x9c8920 | `ev_4a5435ee59e170130751d04fb39f47a481fa7b5c82322bcd854db8e838d4e2cc` |
| 0x9cb4e0 | `ev_4275b7d64861bda9d3f1369468c58deec344a973e3c6e12679479663f89ad441` |
| 0x9d3db0 | `ev_83e67f6199a1a76051df09eebc8044ed8a07305f6d8d4781f3f2f0314abbfc2f` |
| 0x9dd9b0 | `ev_0574d2fb38e2b73df2742d3b8962198eef271497c7cdfe3897846e84f6655b23` |
| 0x9e2aa0 | `ev_8517cbb679aeec97719d6893e9c703edabbd32bf7ff31ca240d710d42bb50bcc` |
| 0x9e4330 | `ev_9d7dc073b11c44636f3419900ec9739205ea7ec609bdc7d5ad6edc9e00bdbbb2` |
| 0x9e50b0 | `ev_7f352aca262f6060b748c0b2382c24332373d752ed9e903be613570aa89f05ad` |
| 0xb376a0 | `ev_10d7bfd5420317fd7d23d357bf1455a5e746ddac4cb283d31e4b96f022822561` |
| 0xb3c510 | `ev_6356bc8bbb36f8a8053edebbc871fdb724d632f03e1057a807c86f753eaca43d` |
| 0xb3d080 | `ev_051d64019fae344941e0fc417a32adfff3da93c9eda141b92c754c5e4025cd95` |
| 0xb3d6f0 | `ev_2ebabcb0d6249542f9fe97303c67737aa93a0d3bbf02a149750096e95d8594f3` |
| 0xb3f480 | `ev_da7af6d0d124ab2b3eabe6af4685113f4b19781bcf075039b0f3e1bc59f9169b` |
| 0xb400a0 | `ev_b485af3279757c1071a0cb993659856106527f79285725ef98b7e3ca21c36305` |
| 0xb402a0 | `ev_5b4ce74ae2be7bf8b7c03aa2b155bf6ed5d5d28ebf85964e05c061ce6c353005` |
| 0xb414f0 | `ev_d1e9b5405ec84ed176a6b008bf5534eb0e42862462e89cfc5380373b05f84c2d` |
| 0xb41590 | `ev_ea92db96d9d418749132de3860327db69b5c605d251b15af85bf6ef6e437451b` |

Independent checks bind each new body to original file bytes and verify **586 new assembly boundaries**, **481 selected provenance intervals**, and **26 reused Native bodies**. The membership and show ledgers retain source-file hashes, exact request and pointer deliveries, original-byte checks and their limits. The prior .text dispatch inventory and constructed SGameViewPort route are reused, not repeated as new analysis.

The owned Native session checkpoints **7 new observations**, imports the preceding **674-record** bundle, exports and closes. All prior records remain equivalent as JSON values. The canonical bundle holds **681 records / 311,431,803 bytes**. The saved snapshot holds **28 cached primitives / 327,990,388 bytes**, SHA-256 **`b1d3803af8fe170d54752e5c0d8c7f30f256bc8c47854b9dae7a4316607e3cea`**. Provider close completed successfully. Historical Native records outside this canonical subset remain indexed separately.

Ignored local run: `client-data/audit/2026-11-02-sframe-main-list-membership-c54a7d20`. Complete records, both ledgers, source-byte checks, checkpoint, snapshot, preservation and publication receipts remain local.

**Nine preservation checks pass.** Eleven baseline executable, addon and settings hashes, the saved-variable tree, logical database backup and primary service identities remain unchanged. The root Git index stays empty. Only seven authored Markdown files are synchronized through isolated Core/Client worktrees. Gameplay code and versions are unchanged; original binaries/assets, databases, credentials, settings and generated decompiler output are not published.

## Restart point

Reuse this run's snapshot, membership/show ledgers and historical Native index. The map→MAIN→qualifying request→list insertion/removal→visibility callback route is closed under its guards; do not repeat it as unknown or equate map mode0 with immediate list membership. Continue from an actual producer targeting ID 0x2e, a source-backed nonzero MAIN mode write, or a reaching definition of 918630's receiver. Retain duplicate-key/free-key conditions, map/record identity, modal/display callback guards, list/allocator/copy/lifetime and reentrance limits. The post-show name history has a separate whitelist; MAIN's inclusion is not proven and must not become an asserted pointer capture. Other caller IDs and the rejected other-list show variant are bounded negative results, not missing MAIN producers. The prior viewport producer/frame, .text inventory, PDB/incoming/pointer/clipping searches are reused rather than repeated.

## Follow-up: MAIN request producers, 2026-11-02

The [request-producer investigation](SFRAME_MAIN_REQUEST_PRODUCERS_RE.md) identifies source-backed requests and their command bindings. Class/resource-qualified menu and taskbar handlers produce ToggleMAIN requests; MAIN's close handler and an instance-result handler produce explicit visibility requests. The instance-result continuation selects tab454=1, distinct from mode458. Queued delivery and intact receiver/resource conditions remain explicit. The alternate-mode trigger and 918630 receiver remain unresolved. The earlier registry/list, viewport and indirect callback limits remain applicable. Data and versions remain preserved.
