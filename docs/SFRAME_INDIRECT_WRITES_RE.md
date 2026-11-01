# SFrame indirect character-sheet writes

**Follow-up:** [Implicit-copy bounds](SFRAME_IMPLICIT_COPIES_RE.md) resolves local lengths and DF-qualified spans for all 41 retained implicit seeds. Five long copies share a settings source; their selected system-window callers are traced, with one helper receiver still unbound. MAIN activation remains unresolved; untyped/interior bases retain their limits.

Verified **2026-11-01** on original x86 SFrame SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`, using **REA 6.4.0 / Ghidra 12.1.4**. This continues [the writer-provenance investigation](SFRAME_MODE_ACTIVATION_RE.md) with **25 new function dossiers / 19,426 analyzed body bytes**, ten focused reference/containment observations and a new bounded address-arithmetic scan. **Nonzero activation of SUICharacterInfoWnd+0x458 remains unresolved.** The concrete parent paths, caption callback and newly selected address escapes below do not identify that setter.

[Versions](VERSIONS.md) remain Core/RappelzRust **0.1.0 development**, NoLimits launcher **1.3.60**, original SFrame PE **9.1.0.0**, embedded build lineage **Source9.5.2**, network literal/profile **201507080**. The exact official Epic/patch remains unconfirmed. Original client execution and live object state are not observations of this static pass.

## Parent access is real, but the identified calls are reads

The retained initializer 0x8a1ab0 and setter 0xb3d9e0 bind each Basic/Added child+0x20 to MAIN. Fresh Basic virtual+0xf0/**0x89b7e0** and+0xf4/**0x89d9b0** load that parent and call the already retained getters:

| Getter | Returned value | Reused Evidence |
|---|---|---|
| 0x8a1d30 | BYTE MAIN+0x458 | `ev_3cfd8199a45a65e80dd1d832beb282b912162f8c06beeac5a98a9a30e231812e` |
| 0x8a1c10 | DWORD MAIN+0x45c | `ev_849e554ef2e3225eef34ad284e75a062591ffc3ec4f63338b8dfbbe1e59ee72b` |

Original instructions verify ECX=parent before each call, including the parent stack spill/reload. When the mode is nonzero and the world/actor/data lookups succeed, the selected branches copy actor data into local 16-byte/ 82-byte inputs. Otherwise they use their player-data fallback. These are consumers of an existing mode value. The getters return values, not addresses exposing the mode field. Other callees/indirect effects remain separate from this verified parent access; no server disclosure policy or wire layout follows from these internal copies.

## Child expansion and MAIN geometry use different fields

Basic virtual+0x110/**0x8a1110** and Added+0x110/**0x899960** store the supplied low byte at **child+0x448**. With a nonzero second argument they explicitly set ECX=[child+0x20] and call the retained **0x8a3650** geometry method on MAIN. Their named-control operations then use the child's virtual+0x80. The shared **0x8a1930** wrapper obtains child+0x448 and forwards it to that setter.

Fresh **0xb3d5c0**, the selected virtual+0x80 target, looks up the named control and writes its byte+0x0c. The separately decoded 73-byte helper **0x8a1950** clears that flag for `button_fold` and `button_unfold`, then sets it for one according to its argument. This binds the selected control-flag updates; the precise user-input action that changes the child state is still not established. Neither child+0x448 nor the returned MAIN rectangle updates are assignments to MAIN+0x458. Runtime aliases, resource overrides and uninspected downstream controls are not globally excluded.

## Named text routes to a control and a caption handler

MAIN virtual+0x74 is fresh **0xb3d4c0**. It finds the named control, explicitly switches ECX to the returned pointer, then calls fresh **0xb3d150(text,1,0)**. The latter updates the receiving control's string members, conditionally invokes the global caption object at **0xedd5a0** with the control's string member at+0x94, and invokes that control's virtual+0x3c. The original MAIN pointer is not the receiver of that selected setter call.

The caption object is now bound for the recovered resource-initialization branch:

- Fresh **0xb87450** assigns **0xedd5a0=&object 0xe99b44** at 0xb87ea9.
- Original data binds object 0xe99b44 to vtable **0xd92d84**, RTTI `myUICaptionHandler` local to `InitResource`, first slot **0xb90530**.
- Fresh **0xb90530** processes the supplied string, including matches for **`#@([0-9]+)@#`**, resolves numeric text keys and replaces the matched text. Its observed argument is a caption/string object.

The three retained xrefs to 0xedd5a0 contain two uses and this initialization write. This establishes the selected default caption handler, not every possible runtime reassignment or indirect write to the global. Complete effects of its lookup/replacement helpers and the control's virtual+0x3c remain bounded. No MAIN-mode activation is recovered from this path.

## The list helper writes a generic flag at+0x13c

The previously retained **0xb402a0** passes MAIN as the receiver of fresh **0xb3d6f0(1)**, separately from its list-owning context. This helper changes **receiver byte+0x13c**, follows selected children and calls receiver virtual+0x48. Fresh **0xb3f480** uses the last object of each selected context list, invokes its virtual+0x4c and calls 0xb3d6f0(0) on that object. The last object need not be MAIN at runtime.

For the MAIN, Basic and Added tables, **virtual+0x48→0x7224b0** and **+0x4c→0x7224c0** are each a one-byte **RET**, independently checked and retained as native dossiers. Those selected callbacks do not activate MAIN's mode. Other child types and dynamically populated lists keep their own limits. Short inherited MAIN entries at 0x722530/0x722900/0x722260 are also byte-checked separately: RET4, argument→receiver+0x3ec, and receiver+0x3ec→return, respectively. These short raw checks are not additional native function dossiers.

## Scalar candidates have different receivers

Fresh **0x8891d0** copies twelve current/previous DWORD pairs, including receiver+0x454→+0x458. Fresh caller **0x8899e0** is bound by original **SUISkillWnd** vtable0xd5202c+0xd8 and passes its own receiver to that helper. This selected skill-window path is not a MAIN writer.

Fresh **0xb47640** copies two three-component values into receiver+0x44c..+0x460 and clears byte+0x518. In fresh caller **0x85da10**, the relevant receiver is explicitly the object at the enclosing equipment object's+0x4b0, with local coordinate inputs. Merely finding **KUIControl3DStatic** RTTI nearby does not prove every live receiver's class; this control-pointer transition is retained separately. No MAIN provenance is established for this scalar copy.

## New address-arithmetic coverage

The new **Capstone 5.0.7** scan reads **9,589,018 executable-section bytes**, producing **3,370,152 instruction/skip-data records**. It tracks bounded affine register expressions through calls using MSVC preserved/clobbered registers, includes ADD/SUB forms, and accounts for vector stores whose operand-access flags alone miss their memory destination. It is sequential decoding with branch resets, not whole-program CFG recovery or arbitrary pointer analysis.

| Retained candidate set | Result and boundary |
|---|---|
| 29 exact arithmetic sites | 13 ADD0x454,13 ADD0x458,3 ADD0x45c; no exact SUB/negative-immediate case. First transfers include 5 CALL,7 conditional branches and17 unconditional jumps. |
| 24 affine writes | Six are new after-call candidates relative to the older alias inventory; each has a known final **BYTE+0x458=0**. |
| 102 pointer deliveries | Candidate bases/arguments, not 102 proven MAIN pointer escapes. |
| 41 implicit bulk candidates | Neither MAIN identity nor a copy length reaching+0x458 is generally proved. |

Two new after-call stores illustrate the required byte projection. Fresh **0xb49090/0xb4a560** construct **KUIControlColorBox**, with a **KColorBoxPrimitive** subobject at+0x2ec. Their DWORD store at subobject+0x16c reaches outer+0x458 and contains **0x3f800000**. In little-endian bytes that is **00 00 80 3f**: float 1.0 is nonzero, but the actual mode-position byte is zero. It is not a nonzero-byte activation, and the receiver type is separately bound.

The newly selected vector store **0x8bbfad**, contained in fresh **0x8bbaa0**, moves a 24-byte string representation into receiver+0x454 and resets the source string. Original **SUIQuestRewardWnd** vtable0xd6a608 binds its callers at+0xdc/+0xcc/+0x18; receiver-preserving instructions connect those quest callbacks to the helper. This is a quest-reward string member, not a demonstrated MAIN mode field. The other newly selected vector store 0x773d5e receives an explicitly zero XMM0, with its original instruction interval retained.

Two ADD wrappers are also now typed. **0x901bc0** passes its parent and calls **0x901bd0** with ECX=parent+0x458; that helper fills control pointers and a list. **0x9029d0** tail-jumps to **0x9029e0** with the same receiver adjustment; the latter stores a byte at subobject+0x14, i.e. **parent+0x46c**, and changes control state. Original **SUIArenaGameWnd** vtable0xd5ae14 binds these wrappers at+0xf8/+0x100, while **SUIArenaGameBingoWnd** supplies the typed caller of the latter. This is an Arena control subobject at+0x458, distinct from MAIN's byte at that offset.

## New Evidence and verification

| Entry | Analyzed body bytes | Role | Evidence ID |
|---|---:|---|---|
| 0x89b7e0 | 2764 | Basic16-byte display path and parent reads | `ev_7473b0829fc541429e384401828478f30489f08f95d67d578d1f85e337b40771` |
| 0x89d9b0 | 5756 | Basic82-byte display path and parent reads | `ev_123d66412e1f03dfd14417cc7ed2ad8cb2a27987c0cfd3a694114a66f9611b7f` |
| 0xb3d4c0 | 33 | Named-control text receiver transition | `ev_dc757e2e5359bd61c41d5915bb6c0b0583e49fabc960484f2b63ab82306056cb` |
| 0x8a1110 | 684 | Basic state/geometry/control updates | `ev_716a91d0ede21fa943e67a964317d4c2c00476c1f0352874d99960edd5ddd9c3` |
| 0x899960 | 347 | Added state/geometry/control updates | `ev_b034e31efa50c84b840a34392e21893c61e8b21eadb836f1024b46462d7c2f9c` |
| 0x8a1930 | 26 | Child virtual+110 wrapper | `ev_26455e5f8431311210bf4bcbc86c81c1b0d4a347416ffa8a9c74067fa5bb3e39` |
| 0xb3d6f0 | 206 | Generic byte+13c update and child callbacks | `ev_2ebabcb0d6249542f9fe97303c67737aa93a0d3bbf02a149750096e95d8594f3` |
| 0xb3f480 | 77 | Context-list last-object callbacks | `ev_da7af6d0d124ab2b3eabe6af4685113f4b19781bcf075039b0f3e1bc59f9169b` |
| 0xb3d5c0 | 34 | Named control byte+0c setter | `ev_d1b329ccac2f0c8aa70c09370d1e18e377ec379310b8a71b7c137859ab916624` |
| 0x7224b0 | 1 | Selected virtual+48 RET | `ev_11fbabb910c5fb04970359389da71ea1de0de35ff47fe813085c6a0aeda34e6d` |
| 0x7224c0 | 1 | Selected virtual+4c RET | `ev_27d1bc83e37f7fedad826443e9dfc6fd0f198ba5a294aefa925704e7302278eb` |
| 0x8891d0 | 145 | Current/previous DWORD snapshots | `ev_3383931c7a69a5d2357c54816fa39697e9f76b064d8b351ba2abb7421a7dee10` |
| 0xb47640 | 94 | Two coordinate triples on a control | `ev_866db017c6b53e90de4e56229b26bf5f8c72d2493f9331cdf1848770ac4ef7eb` |
| 0xb3d150 | 397 | Control string/caption update | `ev_b3f2a3d86fd5d7754a07cd1d8d79d1787614f149263c7cbf6b6b608809451baa` |
| 0xb49090 | 277 | ColorBox constructor with subobject | `ev_3bc7c5ae0559e12e2a4f9b3db5fb9d549cc08f2ec1e5506eea7a71ee7ade0d4a` |
| 0xb4a560 | 321 | ColorBox factory and float initialization | `ev_3012f8dd4325eec5e7549b967454b78998e20308ffaeea6e49ffe7f450d6c908` |
| 0x8899e0 | 173 | Typed SkillWnd callback | `ev_76320cda4cfd3afa5b637392df75a744bfa51084b1c5e06507a927ca984d50e7` |
| 0x85da10 | 216 | Equipment/control receiver transition | `ev_0a728e289d5936e7f767f80dd4d09324c22cf607b02db90777624619a24204ee` |
| 0xb87450 | 3403 | Resource-init caption-object binding | `ev_55170aa159791c10c646fa3ef554fb6c5f5d01e9a6367af1709c90ef285824a6` |
| 0xb90530 | 255 | Localized caption handler | `ev_ab147f4018c912b2f3dbc59cb6bd34f85204ad274476a32592bb4c3d1e87aa83` |
| 0x8bbaa0 | 2981 | QuestReward string move | `ev_ef53218ace086894b0adfc3e58ce2e6001e9d1d844efaa39e6625fa171b23e8f` |
| 0x901bc0 | 13 | Arena subobject initializer wrapper | `ev_5db777db3f754f5c6f173c37a5aba3620ca1931932722e5923cc792d7bea654b` |
| 0x901bd0 | 1114 | Arena control/list initialization | `ev_f1d7c4ac9a538b6e77d8d30fe3bf53a4918eeda0dab86e460070adce1c1d26e1` |
| 0x9029d0 | 15 | Arena state wrapper | `ev_11f4a2b375724a5d8f5e0b27e220655269af5de8f6327926916198998c711f0e` |
| 0x9029e0 | 93 | Arena subobject state/control updates | `ev_dd67da6607c71ec0b51cd5933b3b879fab1b77ae7de9437c7cac8b81950c52ee` |

| Focused query | Evidence ID |
|---|---|
| contains-00b490e6 | `ev_14bd4b00f25e34dd905d95da5d0c1e723c5b782058950c6f5902b2becae6ee1f` |
| contains-00b4a5d1 | `ev_dafe1871e9d436f3c6643974df5c2033a4f24fe17e93e2a33c09512f7ad3f306` |
| text-callback-xrefs | `ev_a3e961a5af0994d8d75940e10a232ff877ca8bf7a45f677c97ccb5578bfa2994` |
| contains-00b87ea9 | `ev_0ea1521ffffcbbddc96de6b761be23ff070e4b77e89529ea2b6beccff22fed25` |
| contains-00889a35 | `ev_56821ed62a9bad2b120f453d825fb074ad6d82ee8170e7b1c786a69fb10f178c` |
| contains-0085dac1 | `ev_5ae2fb6d36da3d045e63a71bbcebdc76f9488c88c87a117603fa86885ad123e1` |
| contains-text-handler | `ev_6587c581bfb7eb684d840da24f023e5edb3c4e7eb99f48e48209a5c1c998d66d` |
| contains-008bbfad | `ev_5a79b12866959a2a82c177a22cbb1a7db03ce865f12c493168fd0c3525a09a8e` |
| contains-00901bc1 | `ev_047f25de97d06e6b0b4f82ce6216307a2ba6072255459f43a2dc062102a12edb` |
| contains-009029d3 | `ev_60ec2dca5bbbf3ff0ff2927776c413dec90f8b9315ba78948c8e0b8cb50fcb04` |

Three new value-flow graphs are **truncated**: **0x89b7e0** and **0x89d9b0** each report 0 known omitted edges, while **0xb87450** reports 219; all three report omitted-operation lower bound 1. The other 22 new graphs report untruncated. Complete retained body ranges and pseudocode do not turn these graphs or indirect callees into complete interprocedural coverage. Selected parent transfers, callback registration, string/subobject receivers and actual destination bytes are checked separately. All new records match the original SHA and profile digest `b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17`.

Verification compares **1,537 distinct selected original byte records**, **11 RTTI/class hierarchies**, and **5,107 native assembly instruction entry boundaries** decoded independently with Capstone 5.0.7. Each of the 25 new inclusive function body ranges is separately mapped and hashed against original bytes. The scan follow-up checks 1,314 distinct instruction records before the additional receiver/byte proofs. These overlapping counts are not independent recovered functions or a whole-program CFG. The corrected v2 scanner and its limits are retained; v1 remains a diagnostic, not the current result.

The final canonical bundle is **120,965,527 bytes / 176 Evidence records**: 141 imported unchanged and 35 new observations. The **241,549,986-byte snapshot** retains **152 primitive entries**. The first-function canonical checkpoint is retained and the owned session/processes are closed. Cold initialization was included in the first 378.83-second call; later requests reused that engine. No producer was repeated to recover a truncated response.

All **nine original data/settings preservation checks** pass. Original/installed images, addon/WTF hashes and primary service identities match the baseline. Only authored documentation is published; no original client process was launched, and no gameplay/schema/data/installed-asset/binary/package-version change occurred. The engine remained bounded to 1536 MiB Java heap/two processors and 3072 MiB Node old-space, at reduced priority. Earlier gameplay test counts are historical, not new runtime results for this static pass.

## Checklist and restart point

| Question | Result | Remaining boundary |
|---|---|---|
| Nonzero MAIN+0x458 activation | **Unresolved** | Needs a concrete writer with MAIN provenance or controlled original-runtime evidence. |
| Constructed child→MAIN paths | **Answered for selected calls** | Mode/handle getters are reads; expansion returns through MAIN geometry. Arbitrary callees and aliases remain separate. |
| MAIN text/callback route | **Partially answered** | Named receiver transition and default caption handler are bound; live overrides and downstream virtual effects are not observed. |
| Focus/list route | **Answered for selected MAIN callbacks** | Generic flag+13c is distinct; selected MAIN/Basic/Added callbacks RET. Other list objects remain dynamic. |
| New calculated-address candidates | **Partially answered** | The ColorBox byte at the target position is zero; QuestReward/Arena receivers are typed. 102 candidate deliveries/41 implicit bulk operations still lack general MAIN/length proof. |

Continue from the closed snapshot in ignored audit run `2026-11-01-sframe-indirect-mode-9d6a24f3`. Inspect a remaining candidate only after establishing a MAIN base and, for bulk operations, a destination/length that can reach its mode byte. Reuse the corrected address-escape v2 ledger and the typed child, caption, QuestReward, Arena and ColorBox paths. Do not reinterpret child+0x448, generic+0x13c, a nonzero DWORD with zero low byte, or a different class’s+0x458 as activation of MAIN. No global claim that the feature is inactive follows from this pass.
