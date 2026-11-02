# SFrame text state, rectangle copies and vector producers

Follow-up **2026-11-02**: [MAIN input, tab events and C208 elements](SFRAME_MAIN_INPUT_AND_TABS_RE.md) qualifies actual parent/event delivery, selected input/tab handlers and the distinct C208 element→nested-record→leafL1c rectangle-copy contract. The C208 insertion/type/lifetime questions, actual 918630 receiver and nonzero MAIN458 activation remain open.

Verified **2026-11-02**, following [constructed receivers and MAIN call contexts](SFRAME_CONSTRUCTED_RECEIVERS_RE.md). **Ten new Native dossiers / 4,162 original body bytes / 1,318 instruction boundaries** qualify the selected auxiliary-state producer and its graphics helpers. **The actual receiver of 918630 and activation of MAIN+0x458 remain unidentified.** No original SFrame process is executed and no gameplay feature is added by this documentation pass.

Target: x86 SFrame SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`. REA **6.4.0**, Ghidra **12.1.4**, Capstone **5.0.7**, profile `b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17`. [Versions](VERSIONS.md): Core/RappelzRust **0.1.0 development**, NoLimits launcher **1.3.60**, SFrame file/product **9.1.0.0**, embedded **Source9.5.2**, network **201507080**. The exact official Epic/patch is unconfirmed.

## Object identities and a qualified numerical producer

The already-qualified allocation path constructs a separate **0x80-byte auxiliary A**, stores it at **C+0x104**, and returns A from 9a0f20. **A+0x20 is distinct from control owner C+0x20.** Retained control routine 9a7bd0 calls fresh **9a1230** with this A at four sites: 9a7d25, 9a80b8, 9a814d and 9a8197.

The new routine preserves entry ECX in EDI at **9a1250**, resets that A through 9a1ff0, prepares text and constructs a sprite primitive P through retained a8cfd0, storing its returned pointer at **A+0x50**. Original PE imports independently confirm the Pango/Cairo text-library names used in the Native listing. This supports a text preparation/layout/rasterization interpretation; a complete original source-level signature is not recovered.

The decisive stores are **9a15c8: [EDI+0x20]=EAX** and **9a15cb: [EDI+0x24]=ECX**, with EDI=A. Their source path includes numerical extent accumulation and comparisons in physical stack words **[EBP+0x20]** and **[EBP+0x1c]**. In the selected retained consumer, **9a81e8** loads A+20 into ECX, **9a81eb** stores the value into C+28c, **9a81f1** sign-extends the word at ed27b8, and **9a81f8..9a81fa** adds/stores that numerical value. **9a8200..9a8203** copies A+24 into C+290. This path does not dereference or invoke the loaded A+20 as an owner/callback pointer.

These stores close the specific missing later producer on this selected path. They do not prove that every entry/error/empty-text branch recomputes the fields, identify all replacements, or establish the business meaning of C+28c/+290 in every control. In particular, the numerical path is not a new MAIN-owner binding.

The original normal return of 9a1230 is **RET 0x20 at 9a1cec**, consuming eight physical stack words. The recovered pseudocode displays only two formal parameters and is insufficient for an ABI claim. Native value flow for this function is **truncated: 33 known omitted edges, zero known omitted inputs, omitted-operations lower bound 1**. The complete provider-defined body and assembly boundaries are independently matched to original bytes; this does not make the truncated def-use graph complete. Provider terminal-call assumptions at 9a1cf4/9a1cf9 are not independent proofs that those callees never return.

## The text rectangle is copied before retention

Retained 9a7bd0 passes **&C+0x118** to fresh **9a2a10** at 9a81b4, with ECX=A. The ten-instruction helper:

1. Loads the supplied rectangle pointer from its first physical stack word.
2. Forms **&A+0x54**, copies exactly **16 bytes** with MOVUPS, and replaces the physical argument with this new interior address.
3. Loads **P=[A+0x50]** into ECX and tail-jumps to retained **a8d370**.

The retained setter writes its pointer argument to **P+0x0c** and sets dirty byte **P+0x141** before RET4. Therefore, on the valid normal selected path, **P+0x0c=&A+0x54**, containing a copy of C+118. It does **not** directly retain &C+118. The previously qualified root-MAIN primitive path retains MAIN+118 directly; these two paths must remain distinct. The Native terminal-call caveat on the tail relay is retained; the original JMP and retained RET4 establish the selected ordinary forwarding path.

Fresh **9a2a30** records three floating words at A+34/+38/+3c, walks payloads in the A+68 list through b2bbc0, and conditionally updates the loaded P's positions **P+40/+44/+48**, setting **P+141**. Optional offsets are read through P+c8 and ab6140. Its caller's FLOAT3 words include converted C+128/+12c and C+fc; PUSH placeholders overwritten before the call are not final argument values. These are auxiliary/list/primitive effects, not writes to MAIN+458.

## Sprite scrolling setup uses an embedded receiver

Fresh **9a2fc0** reads and reapplies **A+70 byte, A+74 DWORD, A+78/+7c floating words**, then sends their four physical stack words to **a8de00** with P=[A+50]. It does not finalize A+20 dimensions.

Fresh **a8de00** forwards those four words, adds an explicit fifth argument **&P+0xd0**, and changes ECX to the embedded receiver **S=P+0x50** before calling fresh **a8e3f0**. The latter stores an enable byte at S, handles disabled/null-vertex branches, resets accumulated fields, records the two floating parameters at S+34/+38, and releases an old eight-byte strategy at S+4c. For selectors 0..3 its successful allocation branches install exact original RTTI-qualified strategy vptrs:

| Vptr | Original type name |
|---|---|
| d7f978 | sSpritePrimitiveScrollLeft |
| d7f988 | sSpritePrimitiveScrollRight |
| d7f998 | sSpritePrimitiveScrollUp |
| d7f9a8 | sSpritePrimitiveScrollDown |

It stores the selected strategy at S+4c and copies four coordinate pairs from fifth-argument offsets 14/18, 30/34, 4c/50 and 68/6c into S+4..+20. The **sprite scrolling setup** interpretation is supported by exact original type names and stores. Frame-update callbacks, precise speed/delay semantics and an observed animation remain outside this pass. Neither S nor the loaded P is silently treated as A, C or MAIN.

## Cleanup order and the copied rectangle's bounds

Fresh **9a1ff0**, called first by text preparation and by fresh **9a1180**, requests virtual+14 release on nonnull A+4c/A+64, clears those fields, requests virtual0(1) on nonnull A+50 and list payloads, clears A+50, relinks the A+68 sentinel, resets A+6c and frees old 12-byte nodes. These are explicit callback requests and normal-path stores; arbitrary callback completion, outside aliases and reentrant mutation are not proved.

Destructor **9a1180** first calls that reset, then forms **ECX=&A+0x68 at 9a11a0** and calls fresh **9a1120**. Thus the latter's receiver is the **list header L**, not whole A. It empties the list and frees its 12-byte sentinel. The destructor then handles possible remaining resources and string storage with capacity/alignment guards. The retained replacement caller 9a7bd0 subsequently frees A with size0x80. This qualifies the selected order of requests before freeing the allocation containing the copied rectangle; it is not a universal no-dangling-pointer or whole-program lifetime proof.

## A concrete C+1fc vector producer outside selected MAIN resources

The retained **b49090 and b4a560** construction paths call the control base constructor, construct an embedded primitive at **C+0x2ec**, and install exact RTTI-qualified **KUIControlColorBox vptr d8a330** and **KColorBoxPrimitive vptr d8a320**. They append R=&C+2ec to the pointer vector starting at **C+1fc**.

The capacity-available branch directly writes **[oldEnd]=R** and increments the end word at C+200 by four. The slow paths at **b49168/b4a653** call fresh **9a9f30** with **ECX=V=&C+1fc**, physical stack argument1=oldEnd and argument2=&a caller local containing R. **EDX is only the scratch address of that local**, not a separate formal position argument.

The new helper reads V+0/+4/+8 as begin/end/capacity, computes pointer-sized counts and a guarded growth allocation, inserts *argument2, invokes c5e9a0 for the selected byte copies, frees old storage and publishes the new header at V+0/+4/+8 before RET8. The selected slow callers insert at the old end. Large allocations include the original alignment/header checks. Error/terminal-call and allocation assumptions remain explicit.

The affected header occupies **[C+1fc,C+208)**; the end address is exclusive. This proves no producer at **C+208**. **ColorBox is absent from the selected MAIN resource classes**; its valid vector construction is not proof that MAIN owns such a child. A retained gauge hit is excluded because its receiver had already been rebased into embedded geometry. The bounded retained-corpus search still leaves the later C+208 vector producer and any relevant owner alias unresolved.

## Evidence and preservation

| Fresh entry | Bytes / instructions | Evidence ID |
|---|---:|---|
| 9a1230 | 2,766 / 845 | `ev_b23d43853c861afc941e7c0313733f53cf1af82603a507d52aac1f99346fdece` |
| 9a2fc0 | 69 / 22 | `ev_94ef9477ffb1a95c77de90cb879ea640dcfe60ba83a961a27b76ac2dad85fa3a` |
| 9a2a10 | 27 / 10 | `ev_4b94adbc073f872fc2aff2089759f3981191547dfb1431a97e28dc3749b8eddb` |
| 9a2a30 | 230 / 64 | `ev_c530471821b19e42af0cbea2d21dcb04742c5031c21fa9d95ae014c271a37573` |
| 9a9f30 | 345 / 131 | `ev_19f5dccd89d33c8f061c9ec9d98b33d1b6479ca53f3902a57df0571f85375bd8` |
| a8de00 | 52 / 15 | `ev_9d67677652ceb821a7f07f548282bfe66b1a3d30f54beae2022cc00312d9241f` |
| 9a1ff0 | 140 / 56 | `ev_97d18dcbb6d20a01248a2d487356f2e2a32a19903f6228eeb63fb92c6ec00804` |
| 9a1180 | 162 / 52 | `ev_5ae5a9b9575c19235f47a310f2d32998f23afb069e2cb88f61710bfdad3a7626` |
| a8e3f0 | 304 / 94 | `ev_366552cef3780890b9ef80b8627fb1452b1711f921bbad443041d9772c99533f` |
| 9a1120 | 67 / 29 | `ev_ae3a1367a5eae5e13dc5d990c1eab90c3581a6a4f5df820856da0784fa5b00a1` |

All ten complete provider-defined bodies match the original PE and independent Capstone instruction boundaries. Nine new graphs report no truncation; the 9a1230 limit above is retained. Original RTTI locators/type names, selected callsites and imported-library names are independently checked. Reused sources include a8d370, constructor/cleanup callers and the ColorBox constructors; they are not counted as new decompilations.

Ignored run: `client-data/audit/2026-11-02-sframe-state-producers-45c18b92`. Source contracts retain **100 selected instruction assertions / 112 byte intervals**, four scroll RTTI bindings and 50 Pango/Cairo import names; SHA-256 **`2a8da4ef47aaa3d0d6e08b7c1386eb4d59fbc27f9d146cb2795e7cd9b25a025c`**. Reused auxiliary and vector frontiers retain 42 and 27 byte intervals respectively; their SHA-256 values are **`3efb8fd211c35a02cff1f127fbdb517abdc847f8b5932ebc2d088eaf6d51de58`** and **`9b7ef8641440788ddfdff6c3677d1850963a47ac923d6bb49bc2242a18e08b6c`**. Overlapping intervals are not additional function counts.

The ten new dossiers were captured once and checkpointed before the initial full-file import of the prior export was rejected. The original prior archive remains unchanged (SHA-256 `4aada92515e14333999ace2b9e84951e2c9b15d501e6faa4001c4b8fab1417b8`). A diagnostic reconstruction contained an altered character and is retained locally but excluded from the final corpus. Recovery used **18 bounded raw-record copies** of the untouched original archive; the public REA importer authenticated every record. Original archives and failed responses remain local, and installed REA code was not changed. No successful Native dossier was repeated.

The evidence-only recovery session imports the checkpoint and prior records, exports **721 records**, preserves all **711 prior record fingerprints**, saves `sframe-state-producers.snapshot.json` and closes. The snapshot is **336,193,305 bytes / 21 cached Native primitives**, SHA-256 **`d586929da352d628933dcc7e6ddea8e22588487acb08ce23e00769ffb0d6a828`**. The complete restart index has **578 functions**, including the unchanged **31 separately retained historical sources**. Imported Evidence and cached provider primitives are different inventories; the index does not imply 578 cached queries. The frozen 555-root interprocedural model is unchanged. The bounded replay parts provide an additional authenticated resume path; the original large-import failure is not presented as a change to game logic. A separate evidence-only session successfully reopens this saved snapshot and closes without a new function-analysis request or execution of SFrame.

**Nine SQL/WTF data-preservation checks pass.** Eleven protected file hashes, the complete installed WTF tree, logical backup and primary service ownership remain unchanged. Only authored documentation is published on the two branches; game/terrain code, installed binaries/assets, schemas and versions are preserved. Raw Native exports, original resources, SQL and credentials remain local.

## What remains

| Priority | Unresolved | Required next evidence |
|---|---|---|
| 1 | Actual caller and constructed receiver of **918630** | A reaching call/registration with a qualified object receiver, rather than an opaque entry tag or matching offset. |
| 2 | Nonzero activation of the same MAIN instance's **+458** byte | A qualified writer or equivalent observed transition; 8fb010 alone still has no proved MAIN receiver. |
| 3 | Later **C+208** producers and relevant owner aliases | Follow a source-qualified vector producer/copy or embedded-object constructor. The ColorBox C+1fc producer and A+20 numerical path are now qualified within their stated scope. |
| 4 | Branch/callback reachability and eventual live activation | Qualify the next concrete reaching transfer beyond the unchanged frozen 555-root model, then observe an established original-client trigger. No usable trigger has yet been identified. |

The scrolling update callbacks are a distinct rendering frontier, not evidence for MAIN+0x458. Keep the selected caption-hook/resource-token paths, auxiliary constructor, sprite-array construction, key wrappers, numerical A+20 producer and rectangle-copy path as qualified prior work. Preserve the original packages/settings, archived raw Evidence and the optional terrain pilot. No whole-text/PDB rescan or repeated successful Native dossier is justified by this pass.
