# SFrame tooltip auxiliary provenance and an unbound mode-writer candidate

Verified **2026-11-01** on original x86 SFrame SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`, with **REA 6.4.0 / Ghidra 12.1.4 / Capstone 5.0.7**. This continues [selected remaining callbacks](SFRAME_REMAINING_CALLBACKS_RE.md). **18 new Native dossiers / 3,775 analyzed body bytes**, plus **four exact instruction observations**, qualify auxiliary construction, reference ownership and the retained 918630 candidate. **MAIN+0x458 activation remains unresolved.**

[Versions](VERSIONS.md) remain Core/RappelzRust **0.1.0 development**, NoLimits launcher **1.3.60**, original SFrame PE file/product **9.1.0.0**, embedded **Source9.5.2** build lineage and network literal/profile **201507080**. Exact official Epic/patch remains unconfirmed. The original client is not executed.

The [indirect-entry follow-up](SFRAME_918630_INDIRECT_ENTRY_RE.md) provider-confirms the 20 remaining JA overlaps and studies computed/data-relative carriers, preceding code, exception cleanup and 40 neighboring modules. It identifies no actual 918630 caller or MAIN receiver, with explicit model/runtime limits.

## Typed paths and remaining boundaries

| Field or candidate | Positive source evidence | Qualified result |
|---|---|---|
| control+0x25c | Six allocation→constructor→setter paths | KLazyTip or one of five positively bound derived classes; retained through a reference count. External inputs remain unchecked by the setter. |
| control+0x260 | Two inspected construction paths in 9a8260 | KUITipControl from the tooltip factory, with initial descriptor owner zero. Direct deleting callback, rather than the 25c reference-count release. |
| vector at control+0x264 | Four allocation→7d08d0→9a4a50 paths | Constructed rp::KLazyItemTooltip pointer entries; the selected in-place append retains one reference. Other inputs/growth remain bounded. |
| vector at control+0x27c | Two factory-return append sites in 9a8260 | Generated KUITipControl pointers, distinct from the receiver control and its owner. |
| 918630→8fb010 | Confirmed nonzero outer+0x458 projection | The outer receiver is still unbound. No actual caller or constructor route establishes MAIN. |

## Shared lazy-tooltip objects at+0x25c

Fresh **9a7080** retains a nonnull input by its object+4 counter, replaces control+0x25c and releases the old pointer through virtual0 when its count reaches zero. Six exact original allocation/constructor/PUSH-EAX/setter paths are checked against Native constructor/setter reference sites and their containing callers. These are positive constructions, not a cast inferred from a common offset.

| Exact RTTI name | Constructor | Allocation bytes | Vptr | Selected virtual0 |
|---|---|---:|---|---|
| .?AVKLazyCreatureSkillTooltip@rp@@ | 0x7d0880 | 0x30 | 0xd3fb64 | 0x7d0a60 |
| .?AVKLazyItemTooltip@rp@@ | 0x7d08d0 | 0x34 | 0xd3fb74 | 0x7d0a90 |
| .?AVKLazyCreatureSkillFundTooltip@rp@@ | 0x7d0820 | 0x38 | 0xd3fb54 | 0x7d0a30 |
| .?AVKLazySkillTooltip@rp@@ | 0x7d0920 | 0x30 | 0xd3fb44 | 0x7d0ac0 |
| .?AVKLazyTip@@ | 0x85d1f0 | 0x20 | 0xd3fb34 | 0x7d0b20 |
| .?AVKLazySkillTreeFundTooltip@rp@@ | 0x7d0970 | 0x40 | 0xd3fb84 | 0x7d0af0 |

The measured six-table family derives from **KLazyTip / rp::mixin::ref_counted**. All six selected deleting destructors converge on fresh **7d09d0**. For these newly allocated inputs the setter retains a net reference; count/payload operations apply to the auxiliary object, not control+0x20 or MAIN. Text generation and arbitrary object payloads remain outside this destructor-focused path.

The raw setter-discovery inventory has67 matches; only the six constructor paths above receive this positive subtype qualification. The other61 flows and possible external inputs are not promoted to the same proven subtype. The setter contains no runtime RTTI guard establishing that every supplied pointer belongs to this family.

Ghidra assigns **7d0b20** an automatic library-signature name resembling **MemMapReadOnly::scalar deleting destructor**. That name is not used as type proof. The original vptr, exact COL/TypeDescriptor hierarchy and constructor route bind this selected destructor to **KLazyTip**. No engine annotation or original executable change is made to hide the conflicting name.

## Generated tooltip controls at+0x260/+0x27c

Fresh **9a8260**, whose direct Native body is **2,300 bytes**, calls retained **b3ead0** using the exact class token **tooltip**, descriptor ID **tool_tip** and sprite **ui_frame.spr** on the inspected construction paths. The selected factory binding leads to fresh creator **b71c10**, a **0x2e8-byte KUITipControl** with exact vptr **d8e170**. Fresh **b71be0** is its selected deleting destructor.

The two checked descriptors explicitly supply **owner+0xb0=0**. That value is distinct from the factory/window-manager context and from the caller control's own parent at+0x20. Thus these selected constructions do not set MAIN as the new tooltip's owner. The base initializer/helper, arbitrary factory replacement, globals and later parent changes remain separate limits.

One route stores the generated object at control+0x260; two append sites supply factory results to control+0x27c. Scalar storage and vector storage are distinguished even though they share the same positive factory type. The observed cleanup calls deleting virtual0 on the supplied generated objects; this does not establish uniqueness or lifetime for arbitrary externally modified pointers.

Retained **9a72f0** invalidates the old 25c/260 state when tooltip text changes or clears, then calls retained **9a4d90**. The latter releases refcounted264 entries and deleting27c entries using different paths. Original nonnull/reference-count guards and exact virtual receivers are retained rather than interpreting every virtual0 as a MAIN callback.

## Reference-bearing vector inputs at+0x264

Four original producer paths call **7d08d0** on new 0x34-byte allocations at **799c45/799c96/799d43/799d94**, then pass the returned pointer to fresh **9a4a50** at **799c59/799caa/799d57/799da8**. Exact constructor/vptr/RTTI evidence identifies **rp::KLazyItemTooltip**. A field at payload+0x20 belongs to that auxiliary object; it is not the owner field control+0x20.

On its selected in-place branch 9a4a50 stores one pointer at vector end, advances end by four bytes, retains a net reference and invalidates generated tooltip state. Growth helper **9a3e10** is retained as an unclosed effect boundary; **9a3b10** is the corresponding unclosed 27c growth path. No full copy of MAIN or a control is established by these pointer appends. Getter/text-generation behavior, other caller inputs, allocation failure, reentrance and arbitrary payload aliases remain explicit limits.

## The nonzero+0x458 candidate still lacks a receiver

Retained **918630** has a complete 301-byte Native body. At **918720** it forms **EDI+0x454**, then calls retained **8fb010 at 918726**. The helper's+4 byte write projects to **outer+0x458=1**, and its+8 DWORD reset projects to **outer+0x45c=0**. This is a real nonzero projection; it is not yet a write proven to reach constructed MAIN.

The canonical incoming query is reused, without another identical producer call: **`ev_01b55707ea22285c9103b4c429d9a29add3b1840d29c21e3181b50964ab4a75e`**, observed reference count 0. A fresh original-byte inventory checks exact entry VA/RVA carriers, interior numeric candidates, relative transfer encodings and PE HIGHLOW entries. It finds no exact **918630/518630** literal, and no examined external relative CALL/JMP/Jcc/LOOP encoding into the 301-byte body. The relocation parser checks **210,912 HIGHLOW entries / 2,654 blocks**, including a known MAIN table relocation as a positive control.

There are **23 address-like interior VA values** and one apparent interior RVA. Absence of HIGHLOW is not treated as a rejection of an RVA carrier. Four new canonical instruction observations qualify the exceptional overlaps:

| Exact instruction | Provider decode | Evidence ID |
|---|---|---|
| 0x841f15 | `CALL 0x008470a0` | `ev_ee8ba9bc90409461e9681dcb73d269338dc11a6929cd7528d0f5ba6aa84a767e` |
| 0x946d92 | `MOVZX EAX,byte ptr [ESI + 0x91]` | `ev_47e268b178486c861a40f05e52ecbbbb73afd4c43427643313d5fde16713bb6b` |
| 0x98a2d4 | `MOVZX EAX,byte ptr [ESI + 0x91]` | `ev_e4cb974ebb6a05a1e13d8766fcc5d2d3804ad654f950bbab9aa09f66d7abbffa` |
| 0x98c286 | `CMP byte ptr [ESI + 0x91],AL` | `ev_f7d8aa83dc03003f2c06db1ada41f8a2d442622ab3f489f624b05b6c15fe7f53` |

The RVA value overlaps the opcode/displacement of the **CALL 8470a0**. Three VA values overlap the MOVZX/CMP opcode/ModRM/displacement sequences accessing+0x91; these observations do not identify a reference to 918630. The other twenty 0f879100 patterns remain a numeric inventory with local Capstone interpretation, without claiming their boundaries were all confirmed by Ghidra. Complete source bytes, offsets and limitations are preserved.

The final **virtual+0x78** is deliberately not used to exclude MAIN: exact MAIN+0x78 and the separately typed ArenaWaiting route both select retained **b3d4f0**, whose calling convention is compatible with the five pushed arguments. Shared formatting behavior does not type 918630 as either class. No actual caller, constructor, registration or reaching MAIN base is recovered. Computed/encoded/external entry remains unknown; no dead-code, unreachability or global no-writer conclusion follows.

## Evidence and graph limits

All new observations bind the original SHA and profile digest `b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17`. Full Native bodies and assembly remain separate from p-code coverage and the four instruction observations.

| Entry | Analyzed bytes | Evidence ID |
|---|---:|---|
| 0x9a7080 | 185 | `ev_41343ed1b5ad8341fa25b4e2a7373d6e71f8b7c7b411f2528cac96fcf8f6c470` |
| 0x9a8260 | 2300 | `ev_46d6504df08ac99d282ac13f74905122f6f445c0000b952027d92779111d8db2` |
| 0xb71c10 | 112 | `ev_e205c4e586cdc8c64173e8b8c3c5639d20c2b359b29a98885a2df6b21a785910` |
| 0xb71be0 | 44 | `ev_087a0117b004a5d6af346a4ae59e335aee8d04b5637b2a9ae14e106fd5b343ff` |
| 0x9a4a50 | 320 | `ev_9b35e205ba906680a93a59968b399df4abfbee95f4324a665e785b0fcc6be294` |
| 0x7d0820 | 88 | `ev_3804f261453ec6031ab827d2939d6307bea9add8f70b09e2fd8349da86991d96` |
| 0x7d0880 | 70 | `ev_276a84b5c7a9bc653f66b9e73c0321999e638e096a8a318ad17ef149adbacb2b` |
| 0x7d08d0 | 76 | `ev_916952d9a6be416caf67e127c095be0cf205230c55c5cd7dff3efb2dc679b9db` |
| 0x7d0920 | 70 | `ev_54c997c4e9fff9dd9eb3466b8c40fff728446ffdce925f6aa1da041848347beb` |
| 0x7d0970 | 94 | `ev_8e8e26559624be2f6d9a86dfdfe4c9927ab1b0a7ef75d75e32648b05e0493999` |
| 0x85d1f0 | 118 | `ev_dd015239d7e73f19b50d3b67a580dfce2e7c3f46258d7d41e1eafbb1b1bb114b` |
| 0x7d0a30 | 35 | `ev_ee19f04dd82ae2cf178bc4bbedca3cae1f179e0d2f211aa544f3bd0192de42ba` |
| 0x7d0a60 | 35 | `ev_6c680c53e3a840859782fcdd0bbcb27a1ecd74174c2053e32d33bf158a028a9e` |
| 0x7d0a90 | 35 | `ev_836671fae6752020923d8ad9598e6e26988b5361478b2559d048e1cacfe5bf1b` |
| 0x7d0ac0 | 35 | `ev_0dcc31be464b4573456ba7cda0573178e2d6b4594c00dfc544168ac354100e01` |
| 0x7d0af0 | 35 | `ev_0a138fbc2cf9e0f11c4ada477861cb3d556173b25ddea16dcf14c27711fd191e` |
| 0x7d0b20 | 35 | `ev_0cfbfb2f8308e8d6c738433a08f22b89adb76bb36e9e0e33586bc1d5b562589c` |
| 0x7d09d0 | 88 | `ev_95bb8b0e6cf09cc250e8bc63d96bac73ec9a11e2ef1235ed1fc898b4606e911f` |

**1 new Native graph reports truncation.**

| Entry | Known omitted edges | Omitted operations lower bound |
|---|---:|---:|
| 0x9a8260 | 14 | 1 |

For 9a8260, full assembly and the selected original instruction paths support the two construction/append facts; the truncated value-flow graph does not establish all possible callee effects. The constructor/deleting-function symbols are also kept separate from original RTTI proof.

| Selected reused Native source | Evidence ID |
|---|---|
| 0x9a72f0 | `ev_e6dba90ab6c719cca7693df5a90a2572a6a8be3fadfda8a6eaca577cdf9a91ef` |
| 0x9a4d90 | `ev_4bb1b18232b99f19fab20ae655bce55f4e1929cb780b629c002aaa8c8bfe66dc` |
| 0xb3ead0 | `ev_5f57d5eeeef5a105042b0ceec3f79df05293b720c9fe47ae03fe6cc106e0f02f` |
| 0x918630 | `ev_2e11babc974a7b4241ded6116bc72be4cbafa3d434f034fc40c37e79a04da7bc` |
| 0x8fb010 | `ev_6311d6edbba44308e24bffdc741ec9c6fbccbb16ca26c14b03969b080907bce5` |
| 0xb3d4f0 | `ev_ad8725bbe05f2378247b79f6c19fb6a39e2b4122eebe575c6ae63730f4a888d5` |

## Verification and preservation

Independent checks remap every new Native body to original PE bytes and confirm **1,044 new assembly boundaries**, **94 distinct selected ledger intervals** and **12 separately retained Native bodies**. The fields/vectors/writer receipts independently check their source records, actual allocation/setter paths, factory construction and the four probe results. Historical individual Native records outside the canonical bundle remain separately indexed.

The single owned Native session reuses a light 22-record/21-primitive snapshot, checkpoints new observations, officially imports the previous **526-record canonical bundle**, exports and closes successfully. Its output contains **548 canonical records / 39 cached primitives**. The 18 new Native dossiers and four instruction observations are 22 new Evidence records; the four probes are not extra functions or cached Native dossiers. Prior canonical records, full/selective snapshots and archived source records remain unchanged.

Ignored run: `client-data/audit/2026-11-01-sframe-auxiliary-provenance-2c8a64f1`. Canonical bundle **272,156,218 bytes**; closed snapshot **287,942,320 bytes**, SHA-256 **`a471e0a8d8afeb25192c3809b28015b41c20bc847b5a1f51b312fcfb9862a5c8`**. Complete Native/probe results, field/vector/writer ledgers, raw inventories, checkpoint, preservation and publication receipts stay local.

**Nine preservation checks pass.** Eleven baseline original/installed image, addon and settings hashes, saved-variable trees, the preserved database backup and primary listeners/service identities remain unchanged. Root Git index stays empty. Only seven authored Markdown files are synchronized through isolated Core/Client publication worktrees. Original binaries/resources, databases, credentials, settings and generated decompiler dumps are not published. No activation behavior is invented from an unbound receiver.

## Continuation boundary

The selected auxiliary callbacks now have positive tooltip construction/lifetime evidence. Their text generators, unqualified inputs, vector growth, base helper and mutable factory/global state remain boundaries. The 918630 receiver and MAIN's nonzero activation are still unresolved. Reuse this canonical bundle together with the historical Native index; require MAIN identity and a reaching nonzero byte write before reconstructing activation.
