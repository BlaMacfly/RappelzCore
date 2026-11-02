# SFrame MAIN input, tab events and C+208 elements

Verified **2026-11-02**, following [text state and vector producers](SFRAME_TEXT_STATE_PRODUCERS_RE.md). **16 new Native dossiers / 4,471 original bytes / 1,454 instruction boundaries** qualify MAIN base-input calls, the selected resource controls' input handlers, two distinct tab-event destinations, and the nested C+208 rectangle consumer. **The actual receiver of 918630 and activation of MAIN+0x458 remain unidentified.** This is static analysis; the original client is not executed and no gameplay implementation is added by this pass.

Target: x86 SFrame SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`. REA **6.4.0**, Ghidra **12.1.4**, Capstone **5.0.7**, profile `b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17`. [Versions](VERSIONS.md) remain Core/RappelzRust **0.1.0 development**, NoLimits launcher **1.3.60**, SFrame file/product **9.1.0.0**, embedded **Source9.5.2**, network **201507080**. Exact official Epic/patch is unconfirmed.

Offsets, slots and event/message codes below are hexadecimal; sizes and counts are decimal.

## Concrete MAIN transfers, without expanding the frozen model

The 49 previously retained conditional MAIN contexts expose three remaining root targets that are deallocation, formatting or RTTI cases; residual ECX is not receiver proof there. Following already-retained callees instead qualifies three new whole-MAIN transfers:

| Selected original route | New boundary | Guard and result |
|---|---|---|
| MAIN virtual+d4 → 8a1c20 → 821040 | 821046 → **b3dc50** | The first callee in 821040 receives its unchanged entry ECX. It looks up `bounding_frame` through retained b3c660 and stores the possibly-null result at **MAIN+158**. |
| MAIN virtual+58 → 821140 → b36500 | b36544 → **b3cb10** | The selected 821184 branch admits messages 102/103/201..206 with MAIN+3e8 zero. b36500 explicitly restores its saved MAIN receiver after the diagnostic call and forwards three physical words. |
| MAIN virtual+dc → 8a1d40 → 8a3500 | Six calls → **b3d420** | For the selected visible branch, the same MAIN's +458/+45c are cleared before 8a3500. The calls pass the original Basic/Added/tab child-name strings according to tab+454=0 or1. |

These paths depend on the original table, constructed live objects, valid resource/registry state and ordinary completed calls. They do not observe a live invocation or globally close callback reachability. The frozen **555-root** interprocedural model, attribution script and ledger are unchanged; the new findings are separate source-qualified paths.

Fresh **b3d420** resolves a name, then requires the returned pointer to match a direct node in receiver+44. It unlinks/frees that 12-byte node, decrements the count, allocates a replacement with the same child payload through retained **7438f0**, and appends it before the sentinel, restoring the count. The helper copies the payload from the physical third argument's pointed-to word; it does not retain the caller's stack address. Its input ECX is not used as a list-object receiver. This is **child-list ordering**, not a direct visibility assignment. Fresh b3cb10 traverses this list from the tail using previous links, so the two routines qualify ordering versus input traversal under stable-list conditions. Allocation failure, corruption or reentrant list mutation is not a proved safe path.

## Input geometry, child callbacks and field identities

Fresh **b3cb10** returns2 if receiver+10 bit2 is set or visibility byte+0c is clear. Otherwise it visits children, calls fresh **b3c7b0**, updates a child's **+154** byte when the point-inside result changes, and invokes the parent's **virtual+50**. For the qualified original MAIN table this resolves to already-retained **7224d0, RET8**. Child **virtual+58** receives the three original words, or the original message plus two literal `ffff0001` words on the selected suppression branch. The first qualifying child result is selected using mask22==20; the routine also constructs result bits20/30 and4/8 according to its explicit branches. These are bit masks, not recovered event names.

On message201, it records coordinates at receiver+144/+148 and a selected child pointer at +140, then uses retained **b3d6f0** for focus changes. That setter updates the supplied child's **+13c** and can recurse; the selected MAIN/Basic/Added/six-resource tables' virtual+48 resolve to retained **7224b0, RET**. Focus13c, point-inside154, a selected-child pointer140 and MAIN mode458 remain different fields on different objects. The message200 drag branch additionally uses a loaded object at +138 and b40130; that separate object's full effects remain open.

Fresh **b3c710** tests signed half-open bounds using both receiver+108..114 and +118..124, adjusted by +24/+28/+2c/+30, and visibility+0c. Fresh **b3c7b0** searches children recursively and conditionally tests the receiver itself when flag+10 bit4 permits it. Both return their Boolean in **AL**; upper EAX bits are not a normalized Boolean or an object pointer. Signed arithmetic, integer wrap, descendant membership and valid structures retain their original bounds. Literal suppression coordinates are not assumed to miss every conceivable rectangle.

The selected resource classes bind these original virtual+58 entries:

| Resource class | Fresh handler |
|---|---|
| static | b67ae0 |
| simplebutton | 9ab740 |
| iconstatic | b67720 |
| tab_head | b6ce20 |
| tab_sheet / gauge | 9a6800 |

The four specialized handlers first call fresh 9a6800; tab_sheet and gauge use 9a6800 itself. The selected normal return contracts preserve that base result. Their additional hit/state/event branches keep their own type-specific guards. In particular, the retained static initializer sets control+10 bit2; without a later qualifying change, the base result blocks that static handler's event branch. The handlers do not make every label clickable. Optional tooltip branches of 9a6800 call 9a8260/9a9cd0 and retain a separate unresolved effect boundary.

## Physical event words and their actual destinations

Retained **7cd650 is a 37-byte name accessor with plain RET**. It reads entry ECX and returns the inline/heap buffer of child+4c, or its null fallback. It consumes none of the three event words already pushed by its caller. Pseudocode that displays those words as its formal arguments must not be used as the event ABI. The caller subsequently pushes returned EAX, making four physical words: **name, code, payload1, payload2**.

Retained initialization qualifies the owner link: b35d20 places the same parent in descriptor+b0; b3ead0 constructs/initializes the child and appends it to that parent's list; 9a5170→b3c510 writes descriptor+b0 into **child+20**. Selected wrappers preserve that route. Thus direct MAIN resource children really can deliver to the same constructed MAIN, while Basic/Added descendants retain their own subpanel owner. Five selected control virtual+18 entries use retained **9a6d30**, which loads child+20 and tail-jumps to that parent's virtual+18 without changing the four words. A null owner returns with RET10. Tab_sheet overrides this slot with fresh b6d530.

| Producer | Four physical words | Destination on the qualified route |
|---|---|---|
| simplebutton9ab740 | name, b or a, 0,0 | Immediate owner through 9a6d30; MAIN for a qualifying direct child. |
| staticb67ae0 | name, f or11, coordinate1,coordinate2 | Same owner route, subject to its base-result/flag guards. |
| iconstaticb67720 | name, c/f/10/11, branch-specific words | Same owner route. The shared call at b67862 admits **f,10 and11** from distinct branches; c has its separate call. |
| tab_head b6ce20 | name,47, **TH+2f8 index**,0; later name,a,index,0 | Immediate owner, including MAIN when directly attached. |
| tab_head b6d230 | name,47, **TH pointer**,0 | Separate peer **TS=[TH+2fc]**, then TS virtual18→b6d530. |
| tab_sheet b6de20 | sheetName,47,oldIndex,0; then sheetName,46,newIndex,0 | Directly to **[TS+20] virtual18**, including the qualified MAIN owner. |

The index and peer-sheet links are source-qualified through retained b6f800. **Same code47 does not mean same receiver or payload type.** Fresh b6d530 accepts code47 and a nonzero TH pointer, compares that pointer against TS+6d0..6d4, and calls fresh b6de20 with the matching index or -1. Despite its incomplete three-formal pseudocode, its physical return is **RET10**. It does not automatically forward to its parent.

Fresh **b6de20** checks the requested index, checks the current selection, updates header states and TS+2f4, and emits the two owner notifications above. For actual MAIN, retained **8a23d0→8214f0→b374c0** sends codes46/47 to the default **RET10** path. The already-retained code-a MAIN handler still selects Basic/Added via 8a3500; that is the known tab454 path. Neither notification establishes nonzero mode458.

Fresh **b6d230** returns its fourth physical argument, the saved base mask, in EAX on all selected normal return paths. Fresh b6ce20 returns that value after its call even though its pseudocode is void. Fresh **b6d830** reads only the low byte of its first word, prepares header caption text using the selected/inactive strings at TH+330/+348, strips the selected fixed-length `<#` spans, and calls retained **b3d150** on the loaded header object. It is a caption-formatting path, not a newly established pane-visibility operation. Complete markup grammar, arbitrary templates and all later caption effects are not reconstructed here; the selected caption hook remains qualified prior work.

## C+208 element path: a third rectangle-retention contract

All **70** original control-family virtual+24 entries are accounted for:66 select retained 9a6700 directly; four already-retained wrappers delegate to it, with separate sprite/interior effects. At **9a67c4**, that routine passes **ECX=Q**, loaded from the actual control's C+208 vector, and **&C+118** to fresh **b303a0**.

The new consumer walks a pointer range **Q+40/+44 → G**, then G+c/+10 as **16-byte records**. Only records whose word+4 is zero load **L=[record+0]** and forward the rectangle to fresh **b32c40**. This helper copies 16 bytes into **L+1c**, replaces the physical argument with &L+1c, loads **P=[L+18]**, and tail-jumps to retained **a8d370**. The latter writes **P+0c=&L+1c** and dirty byte P+141, then RET4.

This is distinct from the root MAIN's direct retained rectangle pointer and the earlier auxiliary copy at A+54. Q/G/L/P remain analyst labels: their original classes, constructors, insertion into C+208 and complete lifetime are not recovered. The other resolved b32c40 callers b2e7e0/b300a0/b30470 are future candidates only if their receiver can be qualified; a shared leaf helper alone is not a Q-instance binding. Bounds-diagnostic terminal-call assumptions are preserved rather than treated as independent nonreturn proofs.

Retained **656200** is excluded as a producer anchor: its untyped entry X has 96 parallel vector headers, including matching displacements1fc/208. It is not source-qualified as KUIControl. A matching offset is insufficient. Its incoming negative is bounded to 555 catalogued listings; it is not global absence. The real later C+208 producer therefore remains open.

## Evidence and preservation

| Fresh entry | Bytes / instructions | Evidence ID |
|---|---:|---|
| b303a0 | 163 / 68 | `ev_cf7acf22cb64019146ef1a3199393aa69de6620b4464afdd05ca8360623bca4d` |
| b3dc50 | 21 / 7 | `ev_e888cb96c761851f38276a3f5ff2a2b519cbf70c9fa6f740b22d9ad3a9fd7e64` |
| b3cb10 | 622 / 191 | `ev_c2de3ffc73915d7ec5606f7202c33bcf06277fe453c52dd220214eb49f837201` |
| b3d420 | 144 / 62 | `ev_46eb05dc9df17e4cbd009d9d6233f3cc1b7c2f43011e38f511b62ca8c387487c` |
| b32c40 | 27 / 10 | `ev_aa4221cfe18fdf94369cfaa55f4be83a6a1b035d3b2b883833dd5670b8456aa7` |
| b3c7b0 | 126 / 57 | `ev_1223b270b951390896430f6dbc1f9f8714f3e2dec62ed5507546e1e517746371` |
| b3c710 | 146 / 55 | `ev_d3bbe141c369254adeae962a1ea2c9424e25bd3f6e6a0b983c7c34fa2373500d` |
| b67ae0 | 210 / 84 | `ev_a8f318447b108865d290898051d7ac01cbd7bd80331d286215885dd9c9dea1d7` |
| 9ab740 | 296 / 91 | `ev_9f54e9688951442f777a1e242b8b874c0143b2b4cb3038ba7b67ccd93eb14285` |
| b67720 | 458 / 147 | `ev_1bfe8ff18eab6220c85aeae847bb50c073465ed8713266c3e71ec63905b58f32` |
| b6ce20 | 452 / 130 | `ev_e9548b6187c7a06c2af04497f2d15d9cae6664328a0c386788e1b39d7e7bd460` |
| 9a6800 | 313 / 81 | `ev_74a9d9ad3734b034dbb7482908f89aacf479e9587a06f3b9db66a66e08f0cbed` |
| b6d230 | 177 / 55 | `ev_31927f5fb5e86ab6e7be906acd22d60a084bc46c8282184bc4258350334f9c41` |
| b6d530 | 100 / 45 | `ev_590cf81eb8bc7536afd14438268c94af97dc3663d67ee4be6ce9fd73e8e855d3` |
| b6de20 | 374 / 110 | `ev_adf70d72618021a683dfc62cd66fc5184bff703f1392938cdb6d375e1a8df0f4` |
| b6d830 | 842 / 261 | `ev_5b38b77ca81f4781c8f27d25fde8e1c750c2d385a40532f8645a07103f21c94f` |

All sixteen complete provider-defined bodies and all 1,454 assembly boundaries match the original PE independently. All sixteen new value-flow graphs report no truncation; interprocedural aliases, callback side effects, allocation/exception paths and actual runtime reachability retain their limits. This is not a whole-program proof.

Ignored run: `client-data/audit/2026-11-02-sframe-vector-owner-3e87a1c9`. Six local proof ledgers verify **50 distinct reused/current Native sources and eight input files**:

| Local proof | Byte intervals | SHA-256 |
|---|---:|---|
| main-frontier.local.json | 103 | `9ea9680751d99372857de9ccde68c18c51887052cec4f350b54f1ca5d8d2aca3` |
| c208-frontier.local.json | 106 | `ef52de48210779aee92b4f841912ae5050f505a12a582875d0a41ccfdaa6b329` |
| main-callback-frontier.local.json | 75 | `a85c4f6895b0073e3b482d26dee63b265de2b2b82b329895124f67d5774ab396` |
| main-event-frontier.local.json | 67 | `2746f7745d79e88c68e1f2be773b89848264e3fc032d6decd3cf4c0f5f4f27e1` |
| c208-element-frontier.local.json | 29 | `9888a89a9604cdf0801f2f1249a8e25bb376b653db5e2c3ec307a43ad02e415d` |
| new-effects.local.json | 214 | `7bdb5bd0206151b0c4b3a461f3c556a0dc1a41d3b7fcd31aa6d29e652ba4b789` |

Intervals overlap; their sum is not another function count or global coverage measurement. New effects retain 213 selected instruction assertions; event proofs include 13 branch-aware physical paths. No successful Native dossier is repeated. A single owned public REA session restores the prior 721-record snapshot, captures 16 new functions, exports **737 records**, preserves all **721 prior fingerprints**, saves `sframe-vector-owner.snapshot.json` and closes. The complete index has **594 functions**, including 31 separately retained historical sources unchanged. The snapshot is **344,922,249 bytes / 37 cached Native primitives**, SHA-256 **`d036f4294a84ee0ba9d9f9cce4bf9d653b2f10d9d900de23b342ca2792f3b355`**. Evidence records, cached queries and indexed functions are separate inventories.

**Nine SQL/WTF preservation checks pass.** Eleven protected file hashes, the complete installed WTF tree, logical backup and primary service ownership remain unchanged. The root Git index remains empty. Only authored documentation is published on the two branches; installed packages/assets, game/terrain code, schemas and versions are preserved. Raw resources, SQL, credentials and Native exports stay local.

## What remains

1. Establish an actual reaching caller/registration and constructed receiver for **918630**.
2. Establish a qualified nonzero write to **the same MAIN instance's +458 byte**; the ordinary input/tab paths above do not provide one. The unbound 8fb010 writer still needs its receiver.
3. Find the actual **C+208 insertion/construction route** and any relevant owner alias. Its selected element/copy consumer is now qualified and should not be repeated as wholly unknown.
4. Follow only a concrete next branch: optional 9a8260/9a9cd0 tooltip effects, loaded-object b40130 drag effects, or another source-qualified callback/producer. Preserve descendant, lifetime, reentrancy and resource guards; these branches are not claimed to activate MAIN mode. No usable original-client activation trigger is yet established.

Keep the selected caption/token paths, numerical A+20 producer, three distinct rectangle contracts, key wrappers and the input/tab contracts above as qualified prior work. Do not repeat the old PDB/incoming/whole-text inventories or assign original classes from coincident offsets.
