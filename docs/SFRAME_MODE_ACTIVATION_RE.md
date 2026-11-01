# SFrame alternate-mode writer provenance

Investigated **2026-11-01** on original x86 SFrame SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`, using **REA 6.4.0 / Ghidra 12.1.4**. **21 new function dossiers / 8,563 analyzed body bytes**, plus five focused search/containment observations, are retained. This continues [the CharacterInfo callback pass](SFRAME_CHARACTER_CALLBACKS_RE.md). **A concrete nonzero activation of SUICharacterInfoWnd+0x458 remains unidentified.** The strongest newly followed literal writer belongs to a separately constructed **SUIGaugeWnd**; the selected vector-copy path belongs to **KUIControlPanel** descendants. Neither is a proven activation of the main character sheet.

[Versions](VERSIONS.md) stay explicit: Core/RappelzRust **0.1.0 development**, original NoLimits launcher **1.3.60**, original SFrame PE **9.1.0.0**, embedded build lineage **Source9.5.2**, network literal/profile **201507080**. The exact official Epic and patch remain unconfirmed. This pass executes no original SFrame process and changes no gameplay implementation.

## The WORD writer is on a gauge-window path

The candidate at **0x7460db** writes **the 16-bit value 1 at receiver+0x458**, overlapping the byte offset under investigation. Fresh **0x7460c0** requires receiver+0x44c to be nonnull. It selects `common_gauge_titanium_bow_cast_cool`, updates the referenced control, requests a rectangle with height 9, stores its argument at receiver+0x46c and copies receiver+0x45c to+0x468. No duration unit or live animation is inferred from those values.

Receiver provenance is the decisive distinction:

1. In the reused interface handler, **0x9d796f–0x9d7989** supplies internal message+0x13 to **0x9e3f40**, checks its returned pointer, loads the WORD at message+0x1d, sets **ECX to that returned pointer**, then calls 0x7460c0. These offsets describe internal objects, not newly recovered network fields.
2. **0x9e3f40** adjusts the interface receiver by **+0x74** and calls **0x9e5010**, a key-based bucket/ordered-vector lookup returning the stored payload. This is distinct from **0x9e4330**, whose registered-window lookup first uses **interface+0x24**.
3. Fresh creation **0x9c7eb0** checks the +0x74 registry, allocates **0x470 bytes**, calls **0x744560**, initializes a resource named `gaugewnd`/`_gauge_wnd`, and passes the returned object and key to **0x9e2840**.
4. **0x744560** installs vtable **0xd35f88**, whose original RTTI is **SUIGaugeWnd**, and initializes its WORD+0x458 to zero. **0x9e2840** inserts a payload containing the supplied object pointer and key into the same +0x74 registry and tracks the pointer in the interface vector at+0x90. Its retained direct caller set contains 0x9c7eb0.

Thus a concrete creation→insertion→lookup→writer path is established for **SUIGaugeWnd**, separately from MAIN's **0x478-byte SUICharacterInfoWnd**, vtable **0xd53af8**, registered window ID **0x2e**. The two-byte gauge state must not be renamed as MAIN's one-byte mode flag. Runtime registry mutation, indirect registrations and arbitrary pointer substitution remain outside this static construction proof.

The separately analyzed **0x9c4d20** allocates **0x608 bytes** and constructs **SUICharacterWnd** through 0x743e00, then uses the **+0x9c** registry and+0xb8 tracking structure. That character-overlay construction alone does not type the +0x74 lookup. Original RTTI/constructor bytes keep **SUICharacterWnd**, **SUIGaugeWnd** and **SUICharacterInfoWnd** distinct.

## The vector writer receives panel rectangles

Fresh **0xb725d0** performs two original **MOVUPS 16-byte stores**, to receiver+0x44c and+0x45c, then calls 0xb72170(0). The first interval includes+0x458; its last DWORD is the supplied first rectangle's fourth component. Recovered pseudocode expands these vector stores into scalar assignments, so an apparent DWORD+0x458 store cannot be attributed without its receiver and source.

The retained incoming references identify nine direct call sites. The fresh selected caller **0x7c27f0** finds **`trim_titlebar_open`**, builds a local rectangle from that child's height and the panel's geometry, and supplies it together with the panel rectangle to 0xb725d0. Original vtable **0xd4ca58+0xd4** binds that method to **SUIGuildDonatePanelWnd**. Its hierarchy includes **KUIControlPanel**; a second selected caller is bound to **SUIAdministraionPanelWnd**, vtable **0xd4ae18**, with the same base. MAIN's hierarchy contains **SUIWnd/KUIGenWnd/KUIWnd/KObject**, without KUIControlPanel. The compiled class spelling `Administraion` is retained.

These positively typed panel callers do not establish a vector-store activation of MAIN. Their rectangle-copy helper's downstream/indirect effects are not globally excluded. No whole-program absence conclusion follows from the nine direct references.

## Remaining main-sheet helpers

Three fresh main-adjacent helpers close previously uninspected direct effects:

| Entry | Observed behavior |
|---|---|
| 0x8a29c0 | Read the string at `[MAIN+0x40c]+0xa4`, format it and call MAIN virtual+0x74 for `character_placeguildname_text`. |
| 0x8a2970 | Find `character_guild_icon`; choose `common_mark_icon_none` for an empty supplied string or the supplied string otherwise, then call the child sprite helper with `ui_frame.spr`. |
| 0x8a3650 | Use a **byte argument**, with a 12-byte stride, to read cached heights at MAIN+0x460/+0x464/+0x468; update local rectangles for `inframe`, `outframe` and MAIN through 0xb3d010; request bounds correction if the byte is nonzero. |

The reused caller **0x8a3500** obtains that selection byte from the active child's **virtual+0x104**. Both child tables bind this slot to fresh **0x89a370**, which returns the raw byte at **child+0x448**. Values 0/1 select the two cached height triples; the getter itself does not clamp the byte to that set. This child flag is distinct from MAIN+0x458. These paths do not recover a nonzero main-mode assignment. Child setters, inherited virtual calls and arbitrary aliases retain separate coverage limits.

## Nine inline rectangles do not prove a global count bound

The reused main/inherited geometry method **0xb36cb0** reads rectangle entries at receiver+0x334+i×16, bounded by count+0x3c4. A hypothetical nineteenth entry starts at+0x454 and its second DWORD reaches+0x458. The earlier report correctly left cardinality unresolved.

Fresh **0xb386d0** stores the resource-count result of 0xac1780 into receiver+0x3c4. Its **comparison to 9 guards rectangle initialization**, which fills nine rectangles in three iterations. It does **not clamp the stored count to 9**: a different positive result survives that guard in the inspected body. A universal upper bound of nine therefore cannot be claimed. The actual selected MAIN resource count at runtime is still unobserved; this is not evidence that a count of 19 occurs or that this read activates the mode.

Fresh **0xb38530** and **0xb37fe0** handle separate tiled geometry/storage through receiver+0x3cc/+0x3d0/+0x3d4, including dynamically allocated elements. Those separate arrays do not by themselves bound the inline count+0x3c4. The inherited rectangle operation's indirect child callbacks and externally referenced storage remain distinct from MAIN's mode byte.

## Named-string search boundary

A focused provider-defined string search for **`character_info`** returns six objects: the main, Basic, Added and target-player resource names plus the two subwindow names. No extra named activation command appears in that returned set. This covers Ghidra-defined string objects, not arbitrary packed/undefined text, dynamically constructed commands or all script/runtime behavior. Original resource decoding from the previous pass remains the archived/default resource evidence; no new precedence claim is made here.

## Further copy candidates and procedure boundaries

Provider containment places the write **0x935569** in **0x9354c0**, not the nearby 0x935480 function used by the earlier nearest-entry heuristic. The fresh body finds `storage_scroll`, adjusts its rectangle, then copies the receiving object's current rectangle to **+0x454..+0x463**, overlapping+0x458. Its retained incoming-reference set is empty. The separately analyzed storage event/reset helpers **0x933ff0/0x935480** do not establish a caller bridge to 0x9354c0. MAIN receiver provenance for this copy remains unresolved; neither an adjacent storage method nor the shared field offset supplies that proof.

The other copy at **0x960b01** belongs to **0x960760**. It copies four 16-byte blocks from receiver+0x64/+0x74/+0x84/+0x94 to+0x440/+0x450/+0x460/+0x470 and sets byte+0x480. Fresh caller **0x95baf0** keeps its incoming receiver in ESI and explicitly restores ECX=ESI at **0x95bca3** before the call. Original vtable/RTTI bind that caller to game-object classes including **SGameCreature**, rather than a character-sheet callback. The same body accesses a larger object layout through+0x112c and beyond. This typed game-object path does not establish a MAIN-mode writer; arbitrary invalid pointer substitutions are not ruled out globally.

## New Evidence and verification

| Entry | Analyzed body bytes | Role | Evidence ID |
|---|---:|---|---|
| 0x8a29c0 | 165 | Guild-name label update | `ev_41852a085c13383ec31fa0eeb8dd89f372bb14f7541fc89ffabf33ffbddb4754` |
| 0xb725d0 | 40 | Two panel rectangle stores | `ev_f9aa8138ca1a3cfb14c17cc05560120cf804699278826ad9ebcfd4c7f0bc387a` |
| 0x7460c0 | 136 | Gauge WORD-state writer | `ev_e12ded398444664a05f975cf0228dccae681983389ac055afa900cf8d12d9dc5` |
| 0x8a2970 | 67 | Guild-icon child update | `ev_ba6ae9a8e11a396363b4c97bbaea1426cb12012725e0aa4087706088060e1167` |
| 0x8a3650 | 226 | Height selection and geometry update | `ev_067923c67bfa44bf0c792ce0edd4d48a143064bfd1b471510246c2162674e2b9` |
| 0x7c27f0 | 237 | Typed GuildDonatePanel caller | `ev_9a730a2b2b324d41d6768c55155aed782421e88297ef333f32e98f256253e405` |
| 0x9e3f40 | 49 | Gauge registry lookup wrapper | `ev_dc6faa56bf20734afe1a495e2a1e432b85903006b5f665d9008a219ddbb7b3e2` |
| 0x9e5010 | 154 | Keyed bucket/vector lookup | `ev_dba3d6986bb8edd10b3a412b2ce184febf99bd1a417d4afd9539052cfe3c0bd9` |
| 0x9c4d20 | 630 | Separate character-overlay creation | `ev_3eff1536aec1962490842bbcb045c8990cce0b93ac2f34e9fd4140446625a55d` |
| 0xb38530 | 415 | Tiled rectangle allocation | `ev_979b8ad60ce29de7bd654682d195e394f05251ba52b714279f083b3a055bd50a` |
| 0xb37fe0 | 570 | Separate dynamic control array | `ev_c228333fa818dfff157c8c2383793f5c30d892dcf55b5a090165a2cf990c9ff1` |
| 0xb386d0 | 363 | Skin count and nine-rectangle initialization | `ev_d95c69afe13991fd427fc19cfe36fd3570473a545c2cf7ec7417f6fc1c9883e2` |
| 0x9c7eb0 | 362 | Gauge construction/registration path | `ev_45b57f55d49b01430c7070a20b355da95d73f3fb8be937757b7a259de3935628` |
| 0x9e2840 | 498 | Gauge registry insertion | `ev_04dacfb1ab61a9605405f4280a864e38284662c9a50888053a15d6c4abef8d2c` |
| 0x744560 | 131 | SUIGaugeWnd constructor | `ev_17e37325871a479ef9d357bbc2d641853f8681743116cce25ddd9d2521a56f7b` |
| 0x9354c0 | 182 | Storage-scroll rectangle copy; receiver unresolved | `ev_1e3565c9b7a5b35c8c933a42fe8af9f3e346c5c405e862772e1b92c461656f23` |
| 0x960760 | 996 | Game-object matrix copy | `ev_47d30cb4dd57bb82c7cba230d60578e0fca81bd815e964394447b85441e07fec` |
| 0x89a370 | 7 | Raw child-byte getter | `ev_94c9f7bf35040b9f0062177a4a5ef3e397a369516979cba48cae6a82f6629d77` |
| 0x935480 | 64 | Separate storage reset helper | `ev_cd1015885a1b985abf41b6f8de047394f15ea39c6e745f6ed58e58d8c9c8b3fa` |
| 0x95baf0 | 552 | Typed game-object update caller | `ev_a864c8b06cec5fcda0a8339f99ee07cf207e344aa3792d3596efd1c698da0ebf` |
| 0x933ff0 | 2719 | Storage event helper; no proven bridge to 0x9354c0 | `ev_8cbd5baa859135382084ee8a34e9408f40417ee9168232ef2c5594439b2a68df` |

| Focused query | Evidence ID |
|---|---|
| character-info-strings | `ev_d45b645c3dbf7eab5e78782e63f938c0947df44bd6a0e14624557e95cf4e692b` |
| contains-009e285b | `ev_abce2d6dbd049fd8bc0ab8670a6138a0385307be1895714f2cab642450a13564` |
| contains-009c7ee7 | `ev_bb92430bbcf20ddbf20c83a621cd8f9537df7903e5e0d18fed156695622be89f` |
| contains-00935569 | `ev_f792f82fc216dfc879513467e9917d696b51e94a923801da0ac6e5c5f7f75946` |
| contains-00960b01 | `ev_031073c61f28c9694fb600eb41f1e125466c3421f8e338941895f3b99300774f` |

All **21 new native value-flow graphs report untruncated**, with zero reported omitted-operation lower bounds/known omitted edges. Every new record binds to the original SHA and profile digest `b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17`. The reused large interface handler retains its previous truncated graph; its selected key/WORD/receiver call interval is independently checked in original bytes. New untruncated graphs do not resolve every indirect call or runtime alias.

The previous bounded inventory of **145 candidates with nonzero or unknown source** was re-read: **6 BYTE, 1 WORD, 121 DWORD, 1 QWORD and 16 vector stores**. These are candidate displacements, not 145 proven activations. Six selected wide stores are now independently shown to receive zero from a straight-line `XORPS XMM0,XMM0` sequence without intervening calls, branches or writes to XMM0. Remaining scalar values, arbitrary interprocedural aliases, undefined instructions and runtime writers are not globally resolved. Nearest-entry/RTTI grouping is triage, not proof of receiver identity; the 935480/9354c0 boundary demonstrates that limit.

Verification matches **247 distinct selected original byte intervals**, **eight RTTI/class hierarchies**, and **2,443 native assembly instruction entry boundaries** decoded independently with the locally available **Capstone 5.0.7**. All 21 provider-reported inclusive function body ranges are separately mapped and hashed against original file bytes. MOVUPS memory destinations are checked from their original store encoding; generic operand-access flags alone are not used to classify them. These overlapping checks do not constitute whole-program CFG recovery. The final canonical bundle is **98,413,465 bytes / 141 Evidence records**, preserving 115 imported records unchanged; the **196,526,561-byte snapshot** retains 127 primitives. The owned session/processes are closed, with a first-function canonical checkpoint retained locally.

All **nine original data/settings preservation checks** pass. Original/installed executable, addon/WTF hashes and existing primary service identities match the baseline; no service restart was needed in this pass. The single owned analysis engine used a 1536 MiB Java heap, two active processors, a 3072 MiB Node old-space ceiling and reduced process priority. No original client execution, gameplay/SQL/schema change, installed binary/asset replacement or package-version bump occurred. Only authored documentation is published; raw code, resources, configuration and analysis output remain ignored. Historical gameplay test totals are not new runtime validation for this pass.

## Restart point

| Question | Result | Remaining boundary |
|---|---|---|
| Nonzero MAIN+0x458 activation | **Unresolved** | Still requires a concrete writer with MAIN receiver/caller provenance or controlled original-runtime evidence. |
| WORD and selected wide stores | **Answered for constructed/selected paths** | SUIGaugeWnd, KUIControlPanel and game-object receivers are distinguished; arbitrary registrations/aliases remain separate. |
| Main guild/height helpers | **Answered for inspected bodies** | Child+0x448 is a raw byte, distinct from MAIN mode; complete indirect child effects remain open. |
| Inline rectangle cardinality | **Partially answered** | The nine guard initializes rectangles, not a universal count clamp. Actual selected runtime count remains unobserved. |
| Storage rectangle candidate | **Unresolved receiver provenance** | No retained incoming edge to 9354c0; nearby reset/event code does not prove a bridge. |

Continue from the closed snapshot in ignored audit run `2026-11-01-sframe-mode-activation-c381e4a9`. Reuse the gauge creation/registry proof, the correct 0x9354c0 containment and the raw child-byte getter. Further work needs real MAIN-instance flow through a remaining scalar/property/copy writer or original runtime evidence. Do not turn the absence of a recovered setter into a claim that this version cannot activate the mode, and do not implement an invented activation in RappelzRust/Core.
