# SFrame selected remaining list, control and window-manager callbacks

Verified **2026-11-01** on original x86 SFrame SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`, with **REA 6.4.0 / Ghidra 12.1.4 / Capstone 5.0.7**. This continues [caption and owner callbacks](SFRAME_CAPTION_OWNER_RE.md). **14 new Native dossiers / 3,264 analyzed body bytes** qualify the remaining selected list operations, window-manager route and control callbacks. **No reaching nonzero write activating MAIN byte+0x458 is identified.** Activation remains unresolved; this is not a whole-program absence proof.

[Versions](VERSIONS.md) remain Core/RappelzRust **0.1.0 development**, NoLimits launcher **1.3.60**, original SFrame PE file/product **9.1.0.0**, embedded **Source9.5.2** build lineage and network literal/profile **201507080**. Exact official Epic/patch remains unconfirmed. The original client is not executed.

The [auxiliary-provenance follow-up](SFRAME_AUXILIARY_PROVENANCE_RE.md) identifies selected25c/264 sources as lazy tooltips and260/27c factory products as tooltip controls. It retains external-input and truncated-flow limits, and rechecks the unbound918630 nonzero projection without identifying MAIN activation.

## What is now qualified

| Proven input or receiver | Selected operation | Result and boundary |
|---|---|---|
| MAIN+0x44 as source to a stack temporary | 80b130→80af50→7438f0 | New list nodes contain copied child-pointer DWORDs; MAIN object bytes are not copied by this selected operation. Payload aliases/lifetime are retained as limits. |
| peerSheet+0x6d0 | b6abc0 | Inserts a tabHead pointer into the peer sheet's vector and updates its vector storage; the receiver is not MAIN. |
| Whole MAIN passed to [MAIN+0x138] manager | b402a0→b3f480/b3d6f0 | Reorders window-pointer nodes, changes generic byte+0x13c, calls selected focus callbacks; +0x13c is distinct from mode+0x458. |
| Six selected control classes | virtual+0xd4→9a7bd0 and selected self calls | Exact vtables resolve the selected control slots. Other helper/auxiliary effects remain bounded. |
| Constructed sprite-array element | KSpritePrimitive virtual0→9a4940 | Deletion receives the primitive element, not the owning control or MAIN. The remaining auxiliary types are explicitly unknown. |

## List nodes, payload pointers and cleanup

Retained **b6f800** loads its owner from child+0x20, forms owner+0x44 and passes it to retained **80b130** with a stack temporary as ECX. On the proven MAIN tab-head path, that source is MAIN+0x44. The PUSH EDI at b6f8cc is a late saved register, restored after the loop, rather than a second source argument: 80b130 ends with **RET4**. At its call to **80af50**, four actual stack slots are consumed by **RET0x10**, even though pseudocode names fewer arguments.

Fresh **80af50** walks source nodes and creates destination nodes through fresh **7438f0**, which calls retained **749890**. The node layout is **12 bytes: next, previous, payload DWORD**. 7438f0 reads one DWORD through its third stack argument and stores it at newNode+8. It does not consume entry ECX as a hidden receiver. EAX retains the new node, which the callers consume despite a recovered void pseudocode return type.

On the copy path the DWORD is a **child-window pointer** from sourceNode+8. On the manager path the same helper copies the **MAIN pointer value** from an argument slot. Neither use is a clone of the pointed-to window. Destination nodes are separate allocations, while payload objects remain shared; this does not guarantee their lifetime or exclude mutations during callbacks.

The direct Native dossier for 80af50 contains **151 analyzed bytes in two noncontiguous ranges: 80af50..80afc9 and 80afec..80b008**, spanning 185 bytes. The preceding callee metadata had flattened 151 bytes into a single interval, which is unsuitable as whole-body coverage. All Native instructions in the two actual ranges are byte-checked. Separately anchored exception funclet **80afca..80afeb** and resume fragment **80b009..80b011** remain raw coverage, not silently added to the Native body. Original handler/metadata bytes connect the funclet to this frame. Fresh rollback **813490** detaches and frees a 12-byte destination node and adjusts its count; it does not dereference the payload as a window.

Fresh **b6abc0** receives the **peer sheet's vector subobject at+0x6d0**. It allocates/grows pointer storage, inserts one tabHead DWORD and replaces the vector's three storage pointers. The earlier tab-head route stores its peer at head+0x2fc. These are node/vector and peer-control effects, with allocation/copy contracts and runtime aliases retained as limits.

The temporary's normal teardown at **b6fc60** passes ECX=&stackTemporary to **7445f0**. Its complete **67-byte raw-only interval** has two branches, two calls to be3f66 with size12 and one RET; it frees the copied nodes/sentinel without reading their payload+8. This is separately verified original-byte coverage, not a fifteenth Native dossier. Its conclusion assumes the normal deallocation contract and does not establish payload lifetime or rule out reentrance/ambient effects.

## Whole MAIN through the window manager

For internal kind **0x41e**, retained MAIN handler 8a1dd0 loads ECX from **[MAIN+0x138] at 8a2153**, PUSHes MAIN at 8a2159 and calls retained **b402a0 at 8a215a**. The context is the previously constructed **KUIWndManager**; MAIN is an explicit argument.

The selected route is:

1. Locate the MAIN payload in manager list+0x18 or+0x28, require MAIN+0x13c==0, unlink and free its **node**, preserving MAIN.
2. Call **b3f480(manager)**. It treats former tops in list+0x28, then+0x18: virtual+0x4c followed by b3d6f0(window,0).
3. Call **b3d6f0(MAIN,1)**, which updates byte **MAIN+0x13c** when changed and invokes MAIN virtual+0x48.
4. Create a new node holding MAIN via 7438f0, or the wrapper **b414f0**, and reinsert it.

The two lists are kept as measured offsets; no unsupported modal/nonmodal classification is assigned. A former top is a Window* loaded from Node+8. Its identity as MAIN is conditional. **The manager reloads tops after callbacks**, so callback and following focus-update receivers may differ after reentrance. Other windows' dynamic types and callbacks remain unbound rather than being relabeled MAIN.

Fresh **b3c6d0** qualifies the direct-child+0x13c search used by b3d6f0. For focus1, the fallback compares class bytes to the exact literal **edit\0** at d46ee8, using an equality comparison over four bytes. Revalidated original MAIN/Basic/Added resources have no edit class: MAIN uses static/simplebutton/gauge/tab_sheet/tab_head/iconstatic; the two specialized subpanel roots receive class string genwnd. This selected fallback therefore chooses no edit child from those unchanged resources.

For MAIN, Basic, Added and the six loaded control families, exact vtables select **virtual+0x48→7224b0** and **virtual+0x4c→7224c0**, both single **RET** bodies. The selected focus0 recursion into known child types reaches the same callback48 stub. This qualifies those types and original resource memberships; generated/reparented children, class/vptr replacement, changed lists and deeper unknown descendants remain limits. These facts do not establish a mode+0x458 write.

## Selected control slots and separate auxiliary receivers

All six measured control families share **virtual+0xd4→retained 9a7bd0**. The selected self-control call sites are checked against their exact vtables, not against all 315 possible KUIWnd-derived tables. The static route's self+0xd0 is a distinct resource-loading operation; known self+0x54 targets select retained 722460, a RET.

The new iconstatic continuation is **b67cb0→b3d0b0→self+0x44→b673b0**. The exact iconstatic vptr **d8d1d4** selects b673b0. Its selected animation/resource branches and cache-equality guards remain conditions; downstream sprite/animation helper effects are not globally excluded. The new 936-byte **9a9480**, 441-byte **9a72f0** and 131-byte **9a6100** bodies qualify control-local geometry/auxiliary operations and the class-string alias, rather than another MAIN receiver.

Seven selected self-control virtual sites are resolved under those type assumptions. **Seven auxiliary virtual sites are kept separate**. One conditional positive binding comes from array construction 9a9050 and wrapper 9a49e0→retained a8cfd0: the array at control+0x1f8 has stride 144 and **KSpritePrimitive vptr d7f910**. The destructor site receives the first element and flag 3; fresh **9a4940** is its selected virtual0. Its downstream iterator/destructor callback **a8d150** remains an explicit effect boundary.

The other six auxiliary sites involve fields **+0x25c/+0x260** or elements of vectors **+0x264/+0x27c**. Fresh **9a4d90** qualifies the vector cleanup helper, but their element subtypes are not established. Their receivers are separately loaded auxiliary objects/elements; they are not automatically the control, its owner or MAIN. Runtime replacement, arbitrary aliases and effects of these unknown dynamic receivers remain unresolved.

## Native evidence and limits

All new records bind the original target SHA and profile digest `b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17`. Inclusive body address sets, complete provider assembly and separately anchored raw intervals are retained locally.

| Entry | Analyzed bytes | Evidence ID |
|---|---:|---|
| 0x80af50 | 151 | `ev_8362c2ed14abb8093b78d18185f66b235fe8f08c57cd9c7cdfae40df8d0c7959` |
| 0xb6abc0 | 345 | `ev_e84cfd165c3bcf7b57eaa0cef27ff24da55be853d7927556d72ee0a4d8c13773` |
| 0xb3c6d0 | 40 | `ev_83f716555d42cd9f13743ff6b55472747f327f91d84b23064d4a051e8980a112` |
| 0x7438f0 | 26 | `ev_9c938740e30b1b4870dba81627eb9b2dc889056a8d1a8fc3dcbef9a26e5d8d3c` |
| 0xb414f0 | 66 | `ev_d1e9b5405ec84ed176a6b008bf5534eb0e42862462e89cfc5380373b05f84c2d` |
| 0xb67cb0 | 157 | `ev_f0f154f46a4b2e3bf7a890941c404b39fa69b361621c172b903dcc0e03e13162` |
| 0x9a9480 | 936 | `ev_aafca74ab3458f28bca382e818536522c0d34a0730877fe2a704094a88f384d7` |
| 0x9a72f0 | 441 | `ev_e6dba90ab6c719cca7693df5a90a2572a6a8be3fadfda8a6eaca577cdf9a91ef` |
| 0x9a6100 | 131 | `ev_ac77ee268e302aea7039601589ed272144b201468cfe28332169ae65f6bb4faf` |
| 0x813490 | 45 | `ev_1edf440beda9c0e0612e5af79e6c5233e1c8edf672f4a2e9348f9b99bf46ea69` |
| 0xb3d0b0 | 156 | `ev_26d46979a7e3085648ecffaf1699e42d52f246200d3635c97e5f7c702d624e7b` |
| 0xb673b0 | 448 | `ev_042632b9f480e741bd7a3d724cb58164346040f356360a11e66e59a771565a30` |
| 0x9a4940 | 149 | `ev_93d09db6a3a5833fa364cfdd96ad74b88bfa535f00c5d22579169860a3f725e0` |
| 0x9a4d90 | 173 | `ev_4bb1b18232b99f19fab20ae655bce55f4e1929cb780b629c002aaa8c8bfe66dc` |

All 14 new Native graphs report untruncated. Unknown callees, dynamic types, aliases and runtime mutations remain separate limits.

| Selected reused Native source | Evidence ID |
|---|---|
| 0x80b130 | `ev_20be8738661f7b776a266840313e3cf8456fdc4a0785da3d3ac3d3c8d9daa805` |
| 0xb6f800 | `ev_0b4dc9da8990d900ca1a8ddbc5c6fe5f82b83d6f59f72ce70687663f1b10b665` |
| 0xb402a0 | `ev_5b4ce74ae2be7bf8b7c03aa2b155bf6ed5d5d28ebf85964e05c061ce6c353005` |
| 0xb3f480 | `ev_da7af6d0d124ab2b3eabe6af4685113f4b19781bcf075039b0f3e1bc59f9169b` |
| 0xb3d6f0 | `ev_2ebabcb0d6249542f9fe97303c67737aa93a0d3bbf02a149750096e95d8594f3` |
| 0x9a7bd0 | `ev_9985e8beab0beff9120e8d26bdc72d32e66f8c6b445b641601a07ad279fa5a32` |
| 0x7224b0 | `ev_11fbabb910c5fb04970359389da71ea1de0de35ff47fe813085c6a0aeda34e6d` |
| 0x7224c0 | `ev_27d1bc83e37f7fedad826443e9dfc6fd0f198ba5a294aefa925704e7302278eb` |
| 0x749890 | `ev_5a50426fad3bfcecc56f1c09d15e07368e2296cb446ea921b1ea77d9ae37cfb2` |
| 0xa8cfd0 | `ev_3b64e6e3f9bf7cf46f971bc223ef48c6d68de12071c462fa9d6cefd7d1dbf49d` |

Complete reused-record source hashes and all historical graph limits remain in the three worker ledgers. Native body availability and an untruncated graph are not substitutes for resolving every callee, exception path or runtime target.

## Verification and preservation

Independent checks remap every new Native body to original PE bytes and confirm **1,056 new assembly instruction boundaries**, **179 distinct selected ledger intervals** and **18 separately retained Native functions**. Worker receipts separately cover node/exception metadata, exact vtables, original archive/index/XOR/resource equality and the control/manager receiver distinctions. All Native requests in this pass are complete; the activation question remains unresolved.

The single owned Native session reuses the prior light snapshot's 22 records/21 cached primitives, checkpoints new observations, imports the prior **512-record canonical bundle**, exports and closes successfully. All prior Evidence records remain equivalent JSON values. The new state has **526 canonical records / 35 cached primitives** (21 reused plus 14 new). Historical full/selective snapshots and archived individual Native records remain untouched.

Ignored run: `client-data/audit/2026-11-01-sframe-residual-callbacks-9b7e210c`. Canonical bundle **267,095,939 bytes**; closed snapshot **281,301,875 bytes**, SHA-256 **`3bd82c2a577a5afd4a2a37914f4dd846c650139e56228e4f9a4f4bf054f17d2d`**. Full Native results, list/slot/manager ledgers, raw extensions, canonical checkpoint, preservation and publication receipts remain local.

**Nine preservation checks pass.** Eleven baseline original/installed image, addon and settings hashes, the saved-variable tree, preserved database backup and primary listeners/service identities stay unchanged. Root Git index remains empty. Only seven authored Markdown files are synchronized through isolated Core/Client publication worktrees; original binaries/resources, databases, credentials, settings and generated decompiler dumps are not published. No activation behavior is implemented from an unresolved rule.

## Continuation boundary

The selected MAIN-list, manager and control-slot paths now have explicit receivers and local effects. Remaining questions concern auxiliary dynamic types, downstream helpers, reentrant mutations, generated children and other proven MAIN aliases. Reuse the canonical Evidence and historical Native index together, retain the noncontiguous-body correction, and require a nonzero write reaching the constructed MAIN+0x458 byte before reconstructing activation. These bounded callback results do not prove that activation is globally unreachable.
