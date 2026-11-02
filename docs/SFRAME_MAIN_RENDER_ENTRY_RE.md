# SFrame MAIN render entry and context provenance

Verified **2026-11-02** on original x86 SFrame SHA-256 **`27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`**, using **REA 6.4.0 / Ghidra 12.1.4 / Capstone 5.0.7**. This continues [the retained-pointer consumer investigation](SFRAME_MAIN_RENDER_CONSUMER_RE.md). **A selected constructed frame path now identifies the rendering context as SGameViewPort. SBotManager creates and stores it at owner+0x40, passes the address of that slot to SGameManager, then uses the same object for interface drawing and queue draining. New b400a0/b41590 dispatchers forward that context to listed windows. MAIN's membership/visibility in those lists remains conditional; this is not a live-session observation or a proof of every MAIN invocation. No alternate-mode trigger or reaching receiver of 918630 is established.** This pass adds **11 Native dossiers / 9,529 analyzed bytes** and **0 other observations**. Reused Native records and independent raw-byte probes are identified separately. No original client execution is claimed.

[Versions](VERSIONS.md) remain Core/RappelzRust **0.1.0 development**, NoLimits launcher **1.3.60**, SFrame PE file/product **9.1.0.0**, embedded **Source9.5.2** lineage and network literal/profile **201507080**. Exact official Epic/patch remains unconfirmed.

## New dispatch inventory and selected sites

The original file-backed .text virtual extent is 9,589,018 bytes. Capstone x86 linear decoding covers 9,588,015 bytes in 3,369,149 instructions across 298 intervals; 1,003 undecoded bytes are recorded explicitly. It yields 1,139 candidate indirect memory CALL sites with displacement +0x14. This is a candidate inventory, not 1,139 typed windows or provider-confirmed code sites. Positive controls b3cff1 and b37735 match retained Native instructions.

The selected dispatchers are b400a0 (131 bytes: list fields +0x10/+0x18, calls b400d6/b4010f) and b41590 (150 bytes: list fields +0x30/+0x20/+0x28, calls b415b4/b415dd/b41602). They load ECX from node+8 and a call table from [ECX], then invoke table+0x14 under explicit window guards. Their real caller 9de940 (230 bytes) invokes both through entryReceiver+4 and also has seven fixed-field virtual+0x14 calls at 9de971, 9de989, 9de9a1, 9de9cb, 9de9e3, 9dea05 and 9dea1d.

Both list dispatchers preserve their first physical argument in EBX and forward it unchanged as the window-render first argument. 9de940 saves its first physical argument in EDI; when its second byte flag is zero it passes that EDI value and its third physical argument to b400a0/b41590. Its seven fixed-field calls use the same EDI first argument and literal second arguments 0 for the first five or 1 for the final conditional pair. The true incoming context and constructed receiver are handled by the independent owner evidence; fields and table offsets alone do not type them.

Static linear decode has embedded-data, overlap and alternative-entry limits; exact selected Native body verification is separate. Dynamic membership/vptrs, arbitrary aliases, lifetime changes, reentrance and indirect callee effects remain open. No duplicate 0x918630, pointer or PDB search and no original runtime/data changes occur in this scope.

## Constructed owner and rendering context

The constructed owner is SBotManager O: original constructor a6a030 installs d7c5b8 and calls a6adb0 on that owner. Its 277-byte body is qualified from original bytes and the caller metadata of Native a6adb0; it is not an additional Native capture. a6adb0 allocates 0x74c bytes, calls a17010(1,1), and stores the returned SGameViewPort C at O+0x40. a17010 installs d75540. The separate Bot B, SGameManager G, SGameInterface I and KUIWndManager U retain distinct actual constructor and field bindings; they are not identified from a shared offset.

The actual factory a6a4f0 passes address O+0x40 as argument7 and its newly constructed Interface I as argument2 to 9ed620. That SGameManager constructor stores I at G+0x0c and the slot address at G+0x3c. The actual 9de940 callers pass ECX=[G+0x0c] and context=*([G+0x3c]). Thus, on the preserved constructed path, 9de940 receives I and the stored SGameViewPort C, and loads the actual U=[I+0x04] created by 9bedc0/b3ddd0. b400a0/b41590 pass that same C as window.virtual+0x14 argument1. MAIN receives it only if the actual MAIN instance is in a visited manager list and passes its guards; that membership is not established here.

Native 9de940 has three real direct callers: 9eeb70 at 9eecef, 9efdb0 at 9efe23, and 9f0580 at 9f0606. The positive frame route is a6cb00 selecting B from O+0x20, loading B+0x0c=G and calling 9efdb0 at a6cbd0. Later its bit4 branch loads the same O+0x40=C and calls C.virtual+0x30(1,1) at a6ccbc. The actual SGameViewPort override is a18250; its a18aba call reaches a5c7c0, then the previously qualified queue dispatch aeab50 calls P.virtual+0x08. The earlier SViewPort a58390 route is no longer the only conditional drain model. This is a source-supported constructed frame path, with actual call guards and pointer-preservation conditions.

The selected constructed bindings keep each object distinct:

| Field or slot | Source-supported binding |
|---|---|
| O+0x40 | C: SGameViewPort, created by a6adb0→a17010 |
| B+0x0c | G: SGameManager, created on the a6a4f0 path |
| B+0x14 | I: SGameInterface, created on that path |
| G+0x0c | I, from constructor argument2 |
| G+0x3c | Address O+0x40, from constructor argument7 |
| I+0x04 | U: KUIWndManager, created by 9bedc0→b3ddd0 |

These field contracts identify the selected context route; they do not establish MAIN membership in a drawn list or every possible runtime invocation.

Successful allocation and initialization, valid selected-Bot index and vector/list invariants, object and saved-pointer lifetimes, normal callee-saved ABI, and preserved slot O+0x40, G+0x3c, G+0x0c, I+0x04 and original tables are conditions. Intervening unknown calls or reentrant callbacks may replace these pointers, tables or lists; no nonaliasing or callback-purity claim is made. The actual frame requires O+0x78 and O+0xa8 enabled, O+0x70 bit1 for the selected-Bot call and bit4 for the viewport drain. UI calls additionally require their global and G+0x11c guards, and manager calls are skipped by 9de940 when its argument2 byte is nonzero. 9eeb70 locally sets G+0x11c=1 before its guarded UI branch; an intervening side effect would be required to make that branch pass. The selected MAIN mode0 creation records ID 0x2e in its registry path and does not itself prove insertion into the rendered lists. Native a6adb0 and a18250 have truncated high-pcode graphs with at least one omitted operation each; their complete original source bodies are separately checked. The other nine new pcode graphs are not truncated. No prior manager-local probe, executable sweep, target execution or independent engine is repeated.

## Alternate-mode boundary

The selected rendering path closes the earlier unbound viewport source for this constructed interface route. It does not prove MAIN's initial registration into a rendered list: the retained CharacterInfo factory uses key 0x2e and mode0 registration into the interface map, which must not be equated with the separate manager-list insertion branch. The new dispatch guards use visibility/style/list/render flags on their own objects and parameters; none is identified as MAIN byte+0x458. The preceding geometry/clipping consumer remains bounded to rectangle input. Retained 918630 still saves an unbound entry ECX in EDI and calls 8fb010 at 918726, projecting outer+0x458=1 and outer+0x45c=0 without a reaching definition binding that outer to constructed MAIN. No runtime activation, new 918630 caller or reconstructed trigger is claimed.

## Evidence and verification

All new records bind the target SHA-256 above and profile **`b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17`**. New graph truncation: 0xa6adb0 (0 known omitted edges; omitted-operation lower bound 1), 0xa18250 (0 known omitted edges; omitted-operation lower bound 1). Reused graph truncation: 0x9cb4e0 (0 known omitted edges; omitted-operation lower bound 1). Complete original body bytes do not imply complete high-pcode graphs or whole-program effects.

| New Native entry | Analyzed bytes | Evidence ID |
|---|---:|---|
| 0xb400a0 | 131 | `ev_b485af3279757c1071a0cb993659856106527f79285725ef98b7e3ca21c36305` |
| 0xb41590 | 150 | `ev_ea92db96d9d418749132de3860327db69b5c605d251b15af85bf6ef6e437451b` |
| 0x9de940 | 230 | `ev_04aea8c43a04adbf63cde4a6d612c06a9e28c54d9838fb9c06fc0bf5a3e8dae6` |
| 0x9eeb70 | 492 | `ev_f68f9ac04e9ab3679aa1b77927b4013543f07b4ea1c80b31327df257b564b9ae` |
| 0x9efdb0 | 152 | `ev_29306b5c546ef98b815c7bd25b9e23c174e5932afb12b983285aab251667365c` |
| 0x9f0580 | 199 | `ev_52f165d0df19924da794c0fda857684f56c8dec9ec1eaa7c2fd1ced23b251f2b` |
| 0xa6a4f0 | 473 | `ev_24f45b159dd255b07cb1e8468893a3dfc8e256ffd94cca5f348be2f115de0c48` |
| 0xa6adb0 | 3366 | `ev_b6ac86a183148248b392ec710b5f2946312e3c366952b6a721ba083b31018cbe` |
| 0xa17010 | 356 | `ev_c24303bc986cf9b54aa18bbf9469a2ead1226e1b0028e39d71f05ec18775cc13` |
| 0xa6cb00 | 618 | `ev_b8dcf42a8d940e307b2a02bb9ab6033991370fefc60fb63859f9f169f713a2d8` |
| 0xa18250 | 3362 | `ev_736a3dc237d5bd550d5ceaf44a7abd5bfd7f421c3cb08030a03b886aa52dc105` |

| Selected reused Native entry | Evidence ID |
|---|---|
| 0x7438f0 | `ev_9c938740e30b1b4870dba81627eb9b2dc889056a8d1a8fc3dcbef9a26e5d8d3c` |
| 0x820f70 | `ev_bf6a01b950ce19ea40f7d2a5709725e5431a157e569999caec2f85e087de96e0` |
| 0x8a19f0 | `ev_f3a4ff37e3949c13ed452102ee534729623efb381bd564fe0da8330bbded3cd2` |
| 0x8a1ab0 | `ev_6d832c9d0196d2b1898603d93fd01bcd02fc3ba1df5750e51a235611f561c24c` |
| 0x9bedc0 | `ev_b705a3a4675c9c800473191bafa27ef01bf809be1ed93ecaf380812277799d31` |
| 0x9c8920 | `ev_4a5435ee59e170130751d04fb39f47a481fa7b5c82322bcd854db8e838d4e2cc` |
| 0x9cb4e0 | `ev_4275b7d64861bda9d3f1369468c58deec344a973e3c6e12679479663f89ad441` |
| 0x9d0710 | `ev_2a1a76c68e4ba6016bc3f7c437712c7ff656d4675c07abe738c1990e1094af78` |
| 0x9e2aa0 | `ev_8517cbb679aeec97719d6893e9c703edabbd32bf7ff31ca240d710d42bb50bcc` |
| 0x9ed620 | `ev_ecbb5cfdb6e09384fdc6748777d17c2475a307dd7b0cb669b61912e7bc9cf012` |
| 0xa5c7c0 | `ev_6f1a2f7ca0c2443acf19e9db4cbbf7e2d1666267dfa627e917127f5a4aa74921` |
| 0xa69ed0 | `ev_e1cd440ad8e2b62240e0459982853a329e83a353a8045646d0e2af461b95c55d` |
| 0xaea340 | `ev_554d8a9d214da9115ca6456a5dcb3c5f8df255f6f9a676c7e8e753bbddb17079` |
| 0xaeab50 | `ev_5a9079aba7c6c1642b5b5e61e7022336add70c6309ff8573fb1651790edd00c1` |
| 0xb35b60 | `ev_780cd463050f07ecb6fc2124376fbcb2e95dd2a03b4f84435dcbface76076ff5` |
| 0xb376a0 | `ev_10d7bfd5420317fd7d23d357bf1455a5e746ddac4cb283d31e4b96f022822561` |
| 0xb3cfd0 | `ev_92f6b6976a4d91f0e764268ade2d2a4f627940148d42a2def4715098ba36a362` |
| 0xb3ddd0 | `ev_e0c3e4a3e323b6632c1d0ba389838022ed3efb557a2a5cfba2b64859d1105c3e` |
| 0xb414f0 | `ev_d1e9b5405ec84ed176a6b008bf5534eb0e42862462e89cfc5380373b05f84c2d` |

Independent checks bind each new body to original file bytes and verify **2,629 new assembly boundaries**, **2,289 selected provenance intervals**, and **22 reused Native bodies**. The owner and dispatch ledgers retain file hashes, original-byte provenance and selected path/ABI limits. The whole-text linear dispatch inventory is ordinary independent Capstone evidence, with its own decoding coverage and unknowns; it is not a provider-certified whole-program call graph.

The owned Native session checkpoints **11 new observations**, imports the preceding **663-record** bundle, exports and closes. All preceding records remain equivalent as JSON values. The canonical bundle holds **674 records / 305,637,528 bytes**. The saved snapshot holds **32 cached primitives / 326,283,245 bytes**, SHA-256 **`7fe3e2cd4736533c09d9f20048fb9aafb7eb03d5b44a1d1440cfa5f63957939f`**. Provider close completed successfully. Older Native records outside this subset remain in the historical source index.

Ignored local run: `client-data/audit/2026-11-02-sframe-main-render-entry-4f8c2a61`. Complete records, dispatch inventory, both ledgers, source-byte checks, snapshot, preservation and publication receipts remain local.

**Nine preservation checks pass.** Eleven baseline executable, addon and settings hashes, the saved-variable tree, logical database backup and primary service identities remain unchanged. The root Git index stays empty. Only seven authored Markdown files are synchronized through isolated Core/Client worktrees. Gameplay code and package versions are unchanged; original binaries/assets, databases, credentials, settings and generated decompiler output are not published.

## Restart point

Reuse this run's snapshot, owner/dispatch ledgers, complete linear .text candidate inventory and historical Native index. The selected owner+0x40 viewport producer, constructor argument-slot chain, frame drawing and actual viewport drain are closed under their explicit guards; do not repeat them as unknown. Continue from a concrete MAIN show/list-registration path, another demonstrated entry reaching a MAIN-bound nonzero mode write, or actual receiver provenance for 918630. Initial map mode0 is not list membership. Keep the whole-text inventory's undecoded holes, linear-boundary/targetless-call limits, the two new truncated high-pcode graphs, dynamic table/lifetime/mutation and callback effects explicit. Do not rerun the old empty incoming/PDB/pointer/clipping probes or infer activation from shared offsets or a forced call.
