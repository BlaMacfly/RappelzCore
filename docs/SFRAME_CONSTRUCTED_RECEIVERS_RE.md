# SFrame constructed graphics receivers and MAIN call contexts

Verified **2026-11-02**, following [MAIN child opacity callbacks](SFRAME_MAIN_OPACITY_CALLBACKS_RE.md). **Seven new Native dossiers / 1,606 analyzed bytes / 373 instruction boundaries** qualify control/auxiliary construction, sprite-array creation, a MAIN sprite-query adapter and focused-child keyboard dispatch. The actual receiver of **918630** and nonzero activation of **MAIN+0x458 remain unidentified**. No original SFrame process is executed.

Target: x86 SFrame SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`. REA **6.4.0**, Ghidra **12.1.4**, Capstone **5.0.7**, profile `b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17` are retained. [Versions](VERSIONS.md) remain Core/RappelzRust **0.1.0 development**, NoLimits launcher **1.3.60**, SFrame file/product **9.1.0.0**, embedded **Source9.5.2** and network **201507080**. Exact official Epic/patch remains unconfirmed.

## Passive call-context attribution

The earlier frozen interprocedural model recorded several eligibility rejections before recording the callsite's bound registers/stack. A local extension now snapshots **ECX, ESP and 12 already-tracked physical stack-word positions before those checks**. It does not call the modeled memory reader, read additional original-image words, admit rejected callees, increase quotas or alter graph semantics.

The frozen **555 roots / 89,946 instruction boundaries** are replayed unchanged. After stripping the single new metadata field, old attempts, byte-proof IDs, rejections, summaries, returned-value observations and per-root results compare **exactly equal**. Results remain **9,167 attempted resolved transfers, 329 evaluated contexts, 3,060 callee instructions, 51 summaries, zero target candidates**. Nine new passive-attribution controls, 22 old interprocedural controls and 15 legacy controls pass. Controls cover missing/branched/oversize/quota-rejected callees, overwritten FLOAT placeholders, unknown words and nonmutation of model state.

The compact priority list contains **49 conditional MAIN-related contexts** and distinguishes **1,511 opaque entry-ECX contexts**. These are model labels, not 49 newly proved MAIN deliveries. Qualification requires the original MAIN-table entry, a valid receiver and first uninterrupted entry epoch; the inherited linear root traversal does not establish branch reachability or execution. Physical stack positions do not prove callee arity: a later word can be a caller local. No new MAIN+454/+458 binding or 918630 candidate is established.

Two absent callees have stronger exact receiver provenance and are captured below: **b35dc9→b3dbe0**, with ECX restored from the preserved initializer receiver, and **b3649f→b3cad0**, with the preserved key-handler receiver and two forwarded parameters. Deallocation be3f66, formatting-like adapter 5ba9f0 and RTTI-cast helper c60046 are kept separate; a residual ECX tag alone is not their implicit object receiver.

## Constructed control and separate auxiliary object

The retained static factory b67110 allocates **0x2ec**, invokes fresh **9a4050** and installs derived static vptr d8d0f0. The new base constructor installs **KUIControl vptr d66314**, initializes C+1f4/C+1f8 to zero, and initializes both C+1fc..C+204 and C+208..C+210 pointer-vector headers to zero. Its normal return physically sets **EAX=C**, independently of the recovered pseudocode's void signature.

The retained 9a7bd0 allocation route passes a separately allocated **0x80-byte object A** to fresh **9a0f20**, then stores the returned pointer at C+104. The new constructor preserves A in EDI and sets **EAX=EDI at 9a0fad**, proving the return-to-allocation identity on normal completion. It initializes dimensions at A+18/+1c, an input property at A+28, several container members, and **A+20/A+24 to zero**. At A+0/+10/+14, the described direct writes have a string-shaped layout. No unsupported original class name is assigned to A.

Thus the later opacity write to **[C+104]+48** is bound to this separately constructed object on the selected allocation path. **A+20 is not the control's owner C+20**. Its later value and replacement/lifetime paths are not inferred from the zero initializer. The two empty vector initializers likewise do not prove their later element producers or types.

## The C+1f8 array has a concrete construction route

Retained static resource processing **9a9480** calls fresh **9a9050** at five actual sites with the same control C. The new producer releases/replaces the old C+1f8 array, reads count C+1f4, computes **count×0x144 plus a four-byte cookie** with overflow-saturating arithmetic, allocates storage, writes the count cookie, constructs the elements and stores the payload pointer at C+1f8. Normal type/lifetime conclusions require successful, valid, distinct live allocations and consistent counts; this is not a proof of whole-program memory safety.

The producer passes code pointer **9a49e0** to retained EH-vector constructor iterator **be4695**. The provider explicitly reports **no procedure exists at 9a49e0**. The complete negative response is retained; this is not a no-code finding. An independently decoded **eight-byte original fragment** is `PUSH 1; CALL a8cfd0; RET`. It is recorded as a fragment, **not counted as a fresh Native dossier or provider-defined body**.

The actual constructor **a8cfd0 is already retained**, Evidence `ev_3b64e6e3f9bf7cf46f971bc223ef48c6d68de12071c462fa9d6cefd7d1dbf49d`. Its full 361-byte original body installs exact **KSpritePrimitive vptr d7f910**, initializes the rectangle pointer at R+0xc to zero and sets the known scalar/color members. The raw relay passes flag1 and preserves the supplied element receiver. This binds the selected array's elements to constructed sprite primitives rather than typing them merely from stride0x144. Constructor registration/global effects and later replacements remain conditional.

One new containment observation confirms the existing a8cfd0 entry/body, Evidence `ev_9cb267f0516bac15b4b367c1636599d6dba43a9cf705a6063cf2163e1ddd5968`; it is not another constructor decompilation.

## MAIN name lookup and keyboard dispatch

Fresh **b3dbe0** extracts the 24-byte string at receiver+dc, choosing inline or heap character storage by the capacity-at+14 threshold16. It pushes that character pointer, loads a **different receiver from global edbf88**, and calls ac18f0. In the qualified MAIN initializer, this is a MAIN+dc name-buffer handoff, not a whole-MAIN implicit receiver at the lookup. The caller's sprite diagnostic supports the lookup interpretation; downstream effects and valid-string invariants retain their bounds.

Fresh **b3cad0** tests receiver+10 bit1 and byte+0xc, then calls retained **b3c6d0**, Evidence `ev_83f716555d42cd9f13743ff6b55472747f327f91d84b23064d4a051e8980a112`. The latter walks receiver+44 and returns the first child with nonzero **child+13c**, or zero. The handler forwards its two original physical parameters through that child's **virtual+0x5c**. It returns **4** on the admitted branch and **2** on the other branch; it does not return the child's result. Focus13c and mode458 remain distinct.

Original selected slots narrow the next dispatch:

| Receiver candidate | virtual+0x5c |
|---|---|
| MAIN / Basic / Added | retained b36490 |
| static / iconstatic / tab_sheet / gauge | b3cad0 |
| simplebutton | fresh 9ab310 |
| tab_head | fresh b6cd30 |

The two fresh nine-byte wrappers **9ab310 and b6cd30 both tail-jump to b3cad0**, preserving the entry receiver and parameters. They add no separate key handling. Retained b36490 includes the earlier Enter/Escape handling after the base call. Actual focus choice, recursive descendant membership, event production and runtime state are not observed; these slots do not establish a mode activation.

## Closed paths reconciled with older evidence

The edd5a0 caption hook was already resolved by [caption/owner analysis](SFRAME_CAPTION_OWNER_RE.md): the selected startup store binds e99b44/myUICaptionHandler, virtual0→b90530, and a valid 24-byte MAIN+94 string object and its buffer. No fresh Native target is justified there. The same report already qualifies the selected ordinary resource-token→descriptor→factory-key translation. Complete grammar, runtime overrides and arbitrary mutation are separate limits; the selected translation must not be reopened as wholly unknown.

## New Native records and retained proof

| Fresh entry | Bytes / instructions | Evidence ID |
|---|---:|---|
| 9a4050 | 539 / 86 | `ev_d79afc24485d5904a9b4696c982ceb58399c4651344fc154ca5ae5aa829089f0` |
| 9a0f20 | 211 / 52 | `ev_896b291ea4ca9e706799e3997e08f33ba8d8ea2c547d9f0374bdf28c5579de16` |
| 9a9050 | 756 / 199 | `ev_7cd8abc6813725d55ded8e11e6c614a2a646071202ef73cfe46cd1329c4edcbc` |
| b3dbe0 | 27 / 8 | `ev_cd32f5729c2d3fc64beaf7d9d3922dd2f67968490d271b061014f8287c5ceb78` |
| b3cad0 | 55 / 20 | `ev_66da27a576e9b8178c683cdc79f1f307d52276283fa2be2a76dff001c415a085` |
| 9ab310 | 9 / 4 | `ev_38eebb31c5ada66e1cf8b80e90284056a70ca279e33a3e1b1377433dfa21553e` |
| b6cd30 | 9 / 4 | `ev_67664a8a16511fcaef20c364933528b636dffeb9515fba426ded8b7837c3bf9f` |

All seven new value-flow graphs report no truncation. Inclusive provider bodies and all 373 instruction boundaries match original PE bytes independently. These Native graphs do not close arbitrary interprocedural, exception, alias or runtime behavior.

Ignored run: `client-data/audit/2026-11-02-sframe-owner-contexts-6d47e2a1`. Source contracts retain **45 instruction assertions / 59 byte intervals**, three reused constructor/iterator/focus bodies, exact RTTI and slot bindings; SHA-256 **`8cfa58d4392de0bfebd6be083215aebbfe347dca749c5cda1a965e46ffdb8164`**. The passive call ledger SHA-256 is **`10f7f0b0d16c8c092eecd5ed1637cf6b6b89194a877fd5babceae6ede012a9d4`**. Separate frontiers verify 28 caption-hook, 51 allocation and 47 callsite byte intervals; interval counts overlap and are not new function counts.

The first owned Native session retains five dossiers before the unrecognized relay request and closes with a preserved evidence export. A second sequential session captures the two key wrappers and containment metadata, imports those first-session records and the **703 prior canonical records unchanged**, exports **711 records**, saves `sframe-owner-contexts.snapshot.json` and closes. The complete restart index has **568 functions** across the central bundle and the unchanged 31 separately retained historical sources. The snapshot is **331,029,653 bytes / 23 cached Native primitives**, SHA-256 **`92e544876a1b53fe1d50090b6b845cbe51bd63afb223ce063493a2b0d8f83747`**. The frozen555 model is preserved. No successful Native dossier is repeated; the failed address is not silently promoted to a function.

**Nine data-preservation checks pass.** Eleven protected file hashes, installed WTF settings, logical backup and primary service ownership are unchanged. Original SFrame is not executed. Only authored documentation is published to the Core/Client branches; game/terrain code, installed binaries/assets, schemas and versions remain unchanged. Raw resources, credentials and Native exports stay local.

## What remains

| Priority | Still unresolved | Evidence needed next |
|---|---|---|
| 1 | **Actual caller and constructed receiver of 918630** | A reaching transfer/registration binding entry ECX to a real object, not an opaque tag or coincident field offset. |
| 2 | **Nonzero activation of the same MAIN instance's +458 byte** | A qualified writer or equivalent observed transition; 8fb010 is insufficient until its receiver is bound. |
| 3 | **Remaining loaded-state origins and effects** | Later A+20 assignment, producers/replacements of C+1fc..200 and C+208..20c, and a concrete owner/alias link where relevant. Constructor zeros do not close later state. |
| 4 | **Reaching branches, recursive callbacks and live activation** | Source-qualified successors beyond the bounded linear/thin model, then an appropriate controlled observation of any identified activation. No usable original-client trigger is currently established. |

Do not repeat the closed selected caption/token path, array construction, selected key wrappers, or old PDB/incoming/whole-text inventories. Runtime replacement and arbitrary corruption are limitations, not fresh static decompilation targets by themselves. The next cycle should follow a concrete receiver/write or one of the recorded remaining producers.
