# SFrame Colosseum warning records, predicates and recipients

Verified **2026-11-01** on original x86 SFrame SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`, using **REA 6.4.0 / Ghidra 12.1.4 / Capstone 5.0.7**. This follows [the indirect-entry investigation](SFRAME_918630_INDIRECT_ENTRY_RE.md). **The selected records carry Colosseum warnings and restrictions. No new caller of 918630, MAIN receiver, or alternate-character-sheet activation is established.** This pass adds **19 Native dossiers / 6,271 analyzed bytes** and **three containment observations**. It uses static evidence; the original client is not executed.

[Versions](VERSIONS.md) remain Core/RappelzRust **0.1.0 development**, NoLimits launcher **1.3.60**, original SFrame PE file/product **9.1.0.0**, embedded **Source9.5.2** build lineage, and network literal/profile **201507080**. Exact official Epic/patch remains unconfirmed.

## Corrected interpretation of the four records

The original producer 918630 constructs these records, in this order. The localized values describe full warnings, rather than four short menu choices. Semantic descriptions below are summaries, not server-policy proofs.

| Resource code | Record selector | Resource meaning | Selected predicate |
|---|---:|---|---|
| 2434 / 0x982 | 1 | Resurrection countdown, expulsion and penalty warning | `cSituationConcreterDeath`, 71ecc0 |
| 2435 / 0x983 | 2 | Unfinished-trade countdown, expulsion and penalty warning | `cSituationConcreterTrade`, 71eda0 |
| 2431 / 0x97f | 4 | Use prohibited in the Colosseum | `cSituationConcreterInstancePlaying`, 71ece0 |
| 2432 / 0x980 | 3 | Invitation prohibited for a player in the Colosseum | `cSituationConcreterSiegePlaying`, 71ed70 |

The archive, loose CP1252 resource and converted WDBC agree on all four selected values with **zero replacement characters**. All four codes are individually unique and group 0 in the loose resource. The independent full archive parser reaches exact EOF with **91,896 rows / 30,980 resolved prior-code references**. The loose and WDBC parsers each retain **150,632 rows**, including **36 extra duplicate-code rows** elsewhere; they are not collapsed into an assumed runtime precedence rule.

The resource checks preserve loose `db_string(windows-1252).rdb` SHA-256 **`563b1dcc7e08c0875eb341c8db89b3c95a9619b98081df146af1ac2548a9a5a4`**, decoded archive payload **`41676ad898f41ef232517093abd45442961059ac729ac615d7bbb3709110292e`**, and converted `String.dbc` **`cdad1dc4c1ddee59ebc2b94217a3fc471d338fe2ce08df97c843c199ac12b967`**. Archive index/XOR provenance is retained locally and continues [the earlier stat-resource work](SFRAME_STAT_BANKS_RE.md). Agreement does not establish the resource selected at runtime, language overlays, or actual invocation.

## Record ABI and predicate dispatch

668490 is a singleton getter with plain RET. 6682e0 consumes the numeric lookup argument with RET 4; its new callee 66aec0 loads the code, selects a bucket, compares the exact stored code, and returns the matching record pointer. 6682e0 then returns **the pointer loaded from [matched record+8]**; its diagnostic empty-string fallback remains possible. The decompiler's apparent arguments to the getter do not override these instruction and stack facts.

6ffe80 consumes **12 argument bytes** and constructs a packed **29-byte / 0x1d record**: **byte flag at +0, DWORD selector at +1, 24-byte string object at +5**. The selected producer supplies flag 1. In its **available-capacity branch**, 701c30 moves/appends the string and clears the temporary ownership; growth through 6ef150 remains outside the reconstructed branch. The aggregate has a 12-byte vector header followed by a DWORD of output flags at +0xc. These records contain **no original outer-window pointer**.

6dbac0 loads **ECX=[ECX+0x124]** and tail-transfers to 71e7d0 while preserving the aggregate argument. 71e7d0 advances by 0x1d, accepts selectors 0..10, selects a tiny handler object from the table installed by **71e610**, skips null entries and calls its **virtual slot 0**. Only a true low return byte reaches the output method; the consumer's own success return does not prove that any output occurred. Adjacent RTTI records are not invented additional virtual slots; **71e600 is data, not a constructor entry**.

The two predicate contexts are obtained independently of the producer: context 1 from **[global ecf5a4+0xa4]**, context 2 from **6c80d0(global ecf5a4)**, a seven-byte getter of context+0x94. For the four selected predicates, exact reads are:

| Predicate | Observed test |
|---|---|
| Death / 71ecc0 | Context 2 is non-null and its byte +0x630 is nonzero. |
| Trade / 71eda0 | Calls 6dbab0; loads the global context's +0x5c pointer and tests the pointed object's DWORD +0x20c for nonzero. |
| InstancePlaying / 71ece0 | Context 1 DWORD +0x650 equals 9. |
| SiegePlaying / 71ed70 | Context 1 DWORD +0x650 equals 2 or 3; caller tests the low return byte. |

RTTI names and these numeric state comparisons establish the local predicate path. They do not define all gameplay states, server rules, countdown timing, penalty calculation, permission checks or wire opcodes. The selector chooses the predicate; 71e860 does not itself read that selector to choose a menu action.

## Two internal output paths

After a successful predicate, 71e860 receives the record text, record flag and aggregate flags. The selected aggregate flag **1** takes a branch constructing **`SIMSG_UI_SEND_DATA`**, vptr **d2ea4c**, internal kind **0x426**, window ID **0xd3 at message+0x13**, command **`pushNotification`**, and caption from the record. It invokes context 1 **virtual+0x128** at 71e967. Other aggregate bits have distinct branches; they are not silently treated as the selected flag-1 path.

The shared context construction supplies an **SGameWorld** at +0xa4. **6c4310 installs its newly constructed context into global ecf5a4 only when that global was null**. Thus the source-backed construction path is conditional; an arbitrary already-installed context is not typed merely by reading this global. On the established SGameWorld path, the retained **d78708+0x128→9b75d0→SGameManager.virtual+0x10→9ee8c0→9d3db0** chain applies. The retained kind-0x426 switch passes message+0x13 to **9dd9b0**, looks up that window and calls its virtual+0xcc, subject to interface guards and successful lookup. [The earlier recipient report](SFRAME_MODE_RECIPIENTS_RE.md) preserves that forwarding proof and its limits. **Window ID 0xd3 differs from MAIN 0x2e. Its registered object, class and virtual+0xcc implementation remain untyped in this pass.**

71e860 also has a **second output after the conditional notification branch**: it invokes context 1 **virtual+0x104** with the text, formatting through 99ec80 when the selected record flag is 1. On the proven SGameWorld construction path, slot +0x104 targets new Native **a37020**. That method constructs another **internal message, kind 0x30**, RTTI **`SMSG_CHATTING`**, vptr **d325d4**, copies the text, loads the manager from world+0x230 and calls manager.virtual+0x10. This path is distinct from kind 0x426 and must not be omitted when describing effects. Complete kind-0x30 handling and runtime fanout remain outside this pass.

Both kinds are internal message fields. No new network packet, server request or automatic alternate-mode activation is inferred from these outputs. The packed record does not transport the unidentified EDI receiver of 918630 to either sink.

## Related producers and an actually registered callback

The three containment queries identify actual provider functions before Native analysis: **9178d0→917810 (243 bytes)**, **917a37→917980 (279 bytes)** and **6f5cd0→6f56a0 (1,775 bytes)**.

917810 builds the same four code/selector pairs before its consumer call. 917980 builds a **different two-record variant**, with text codes **0x985/0x986**, selectors 1/2 and flag 1; proximity to the four-record sites is not treated as equivalence. It tests the embedded object at outer+0x454 through 8faf70 and follows a separate formatting path. It does **not** call reset helper 8fb010. Neither producer is promoted to an active callback without registration evidence.

6f56a0 contains a concrete **`ERROR_IN_BATTLE_ARENA`** branch constructing the 0x97f warning through overload 6fff40 at 6f5cc1 and dispatching it through 6dbac0 at 6f5cd0. This partial variant uses **selector 0 / record flag 1**, rather than the four-record selector mapping above. Its **`ERROR_TARGET_IN_BATTLE_ARENA`** branch uses 0x980 through a different context+0xa4.virtual+0x104 path; those effects are kept distinct. Exact RTTI **[vtable d32d00−4]→COL e0480c→type descriptor eaccb8** binds vtable slot **+0x10 to 6f56a0** as **`SRaidMgr`**. Retained ctor 6f0980 writes this vptr at 6f098e and exports its receiver to ecf68c at 6f0a49. This is a constructed static error-handler path, not an observed server packet or gameplay trace, and it supplies no MAIN binding.

There is separately an actual registration route in constructed **`SUIArenaWaitingWnd`**: ctor **916550** installs vptr **d5c778**, initializer **916940** reaches **917500**, and **917654** registers callback **917aa0** with key **0x9c44** in the outer+0x470 registry. Handler **916bc0** selects by message+4 and calls the callback with the outer receiver. This qualifies that registered callback; it supplies no inverse edge into 918630.

New Native **9e47b0** calls ctor 916550 and registers that result as window ID **0xd7** through retained **9c8920**. The ID-indexed table at **e97688**, slot **e979e4**, points to **d69344**, literal **`window_instance_arena_sub_standby.nui`**, with an exact archive/index/payload check retained locally. The loader call uses virtual+0xd0 (**820f70**); **821040 is +0xd4**, and the window-specific initializer **916940 is +0xd8**. ID 0xd7 and this registered waiting-window path remain separate from the notification sink ID 0xd3 and MAIN ID 0x2e.

## Evidence identities and verification

Every new record binds original target SHA-256 and analysis-profile digest **`b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17`**. The 19 new Native value-flow graphs report no truncation; unknown indirect effects remain unknown independently of graph completeness. Reused large graphs **9d3db0 / 9cb4e0 retain their earlier truncation**, respectively **240 / 0 known omitted edges**, each with an omitted-operation lower bound of 1. Selected source instructions, arguments and the earlier switch mapping are separate from complete interprocedural effects.

| New Native entry | Analyzed bytes | Evidence ID |
|---|---:|---|
| 0x6dbac0 | 15 | `ev_942f724e6af58bdd09da6c669c93854a3a0ab9889c5940ebbd43e0977b769464` |
| 0x6ffe80 | 184 | `ev_1e4b82b2a705e0a35f48b3c88596d1c05c0c7e8370215efd4abc4861d2275558` |
| 0x66aec0 | 154 | `ev_aea6161e660bac87b2f82e4cab1226d394d6d7f3438c70932ca8e92a036b2127` |
| 0x71e7d0 | 134 | `ev_0b900ca30081dbc6effe6f77851f5e073499976f78e1e344f328a1f448b5d2ff` |
| 0x701c30 | 98 | `ev_bc3ad08d939dc23148f7d96e8f90566e693e8dd757c9a6a50c0afcf9878bd7ae` |
| 0x71e860 | 905 | `ev_1763269d0eeed3a5dfc8fd8d1e71120c985a1405ca7fb2e23751322e7b50791e` |
| 0x6fff40 | 183 | `ev_2f9aa2baa27277f67fc91dc8a7af937a6205c12b25b2e9823410b7e27558e013` |
| 0x71e610 | 317 | `ev_68700348ae01e110d4ecb20cd675afa939ffdb5ec30ce51cc8b8e2e29886ea42` |
| 0x9e47b0 | 1643 | `ev_7aaf53f690b1adb712ad0153b9adcd3f00d5dca97844e44cac215663b2d2ed27` |
| 0x71ecc0 | 31 | `ev_c978e7373b89681304a690982e408b2130be81f647744cb8878ecfc1cb9064a9` |
| 0x71eda0 | 19 | `ev_86bea4bafcda7b7e2689e613a7e947cea809886f143ac887fcf2f12c116d16f7` |
| 0x71ece0 | 20 | `ev_5f9377497b36bc774abd2391cd80c5cdf72532e2b2115dd44e4270c39f5c1f08` |
| 0x71ed70 | 34 | `ev_0e134d057618cab61d308047428469f51421f1a42bbdf117ddc3f7da0cedf175` |
| 0x6c80d0 | 7 | `ev_eccc6de837f48a2efcffa3192e4452211af684a8ae45b4c136614786a58ea5da` |
| 0x6dbab0 | 14 | `ev_67f6120f1066333c17cb4feff523a4e482e47e50d6e208e7f695f9da3118e555` |
| 0x917810 | 243 | `ev_85e0d1e5f82d8e0bbd61096a80d3e3c93a6b604c1d0a74ee0bae63959df29b08` |
| 0x917980 | 279 | `ev_6f667ccfb3e130926af3c52d22c1e224e43b001a72658002aaf68a586c4e976c` |
| 0x6f56a0 | 1775 | `ev_79ac47d33b757c541130faf2330253877d97763b7e8911d85ed98de8cda9e1f8` |
| 0xa37020 | 216 | `ev_8f793778c11a2f7a08b9182b83d693cd060e32f805d11c9f74b47b95295ffda2` |

| Containment address | Provider procedure | Evidence ID |
|---|---|---|
| 0x9178d0 | 0x917810 | `ev_12773ca6a7ddf46ae8aedd22642e5eaf444c787762674a1891067f1473dca0d8` |
| 0x917a37 | 0x917980 | `ev_08414badfa68c08065e61f92319727e96376a17c7940f2650a842b176ff396f9` |
| 0x6f5cd0 | 0x6f56a0 | `ev_0bb93fd3e179df1bda6297b5ea8a3aa4e67517896440f129ae348bce46e2c4ab` |

| Selected reused Native entry | Evidence ID |
|---|---|
| 0x6682e0 | `ev_eacf49e82d8bd52862eefdd63bc0d384f05f7f13628ede5c6a21ce3d7951f9ca` |
| 0x668490 | `ev_9b414244d3c9fe1767426262f2f36600274fa44af128a167d6c4ad76326864e4` |
| 0x6c4310 | `ev_df69fedf629870844905127b3500cc7f335f184565a4459d4fe21c271db59121` |
| 0x6c8220 | `ev_71675c853debf148612eaadfea8dfd388d7ae5979eb03195dd7d982a44dd6088` |
| 0x6f0980 | `ev_339f7ae5fef8ed7cce993dc82a09f79f89dd3c07d63e80c568db04482b439308` |
| 0x820f70 | `ev_bf6a01b950ce19ea40f7d2a5709725e5431a157e569999caec2f85e087de96e0` |
| 0x821040 | `ev_635168ea1ab1ba99e4dbbf1d9dbfd3b64975a7ca3476836b13603be71e77fa54` |
| 0x8fb010 | `ev_6311d6edbba44308e24bffdc741ec9c6fbccbb16ca26c14b03969b080907bce5` |
| 0x916550 | `ev_596c3d917c90a0709cbb04e922e15feda7569b25cb26d777229825c4cde8e735` |
| 0x916940 | `ev_85b88ee032d7fbd1fc641566e16248d2e186783fccea7b76df953b9f7a705de8` |
| 0x916bc0 | `ev_24d7024d1352d49ab5b8609e5cd1bca0e7242999e5cea6b8bb7d76ff6e2020f2` |
| 0x917220 | `ev_4a91334aea05853adaaa19f600f34a1424ea5dda05858e07416d0782959d6cae` |
| 0x917500 | `ev_30bc9b7b18b248bd3855f1f1c933d9f89ac652d78cc43105cecedda6548e1ee1` |
| 0x917aa0 | `ev_5c5e2e76bb180562efa78127363e7cf4c687dc112ad925f6c515dde17249fb27` |
| 0x918630 | `ev_2e11babc974a7b4241ded6116bc72be4cbafa3d434f034fc40c37e79a04da7bc` |
| 0x9b75d0 | `ev_5f908be21c2a229681371210103a617efda3b89b73dc9d21f94a396c33e01130` |
| 0x9c8920 | `ev_4a5435ee59e170130751d04fb39f47a481fa7b5c82322bcd854db8e838d4e2cc` |
| 0x9cb4e0 | `ev_4275b7d64861bda9d3f1369468c58deec344a973e3c6e12679479663f89ad441` |
| 0x9d3db0 | `ev_83e67f6199a1a76051df09eebc8044ed8a07305f6d8d4781f3f2f0314abbfc2f` |
| 0x9dd9b0 | `ev_0574d2fb38e2b73df2742d3b8962198eef271497c7cdfe3897846e84f6655b23` |
| 0x9e4330 | `ev_9d7dc073b11c44636f3419900ec9739205ea7ec609bdc7d5ad6edc9e00bdbbb2` |
| 0x9ee8c0 | `ev_f6401cb43b6dc2b1cea75674e56b68e3f3aedff971e8059551991d0fcd16b494` |
| 0xa3b7b0 | `ev_25139c79f85ef80ee96e9a4de4f4849082c61d73d3df6d5b933feabded10f843` |

Independent original-byte checks cover all new Native ranges and **1,895 assembly boundaries**, **245 distinct selected provenance intervals**, and **25 separately retained Native bodies**. Separate label, record/predicate/sink and producer/registration ledgers validate source identities, resource parsing and selected byte facts. These overlapping checks are not independent game executions.

The owned Native session checkpoints **22 new observations**, officially imports the preceding **579-record canonical bundle**, exports and closes. All preceding canonical records remain JSON-value equivalent. The new canonical bundle contains **601 records**, **279,746,353 bytes**. The closed snapshot holds **40 cached primitives**, **297,640,616 bytes**, SHA-256 **`6dc87e954179b428886da154f532aa441bdc5e15a4ba27e2377c72cd92a8ede6`**. The historical Native index retains complete older records outside this canonical subset.

Local ignored run: `client-data/audit/2026-11-01-sframe-context-menu-routing-4e8b1a63`. Its historical folder name records the initial hypothesis; the final result above uses the localized warnings and observed predicates. Complete Native/containment records, source bytes, resource digests, three ledgers, checkpoint, snapshot, preservation and publication receipts remain local.

**Nine data-preservation checks pass.** Eleven baseline original/installed executable, addon and settings hashes, saved-variable tree, logical database backup and primary service identities remain unchanged. The root Git index remains empty. Only seven authored Markdown files are published through the isolated Core/Client worktrees. Original resources/binaries, databases, credentials, settings and generated decompiler output are not published. Gameplay code and package versions are unchanged.

## Remaining question

The four warnings, selected predicates and output paths are answered statically; the registered ArenaWaiting callback is independently qualified. **918630's incoming registration/target and EDI receiver are still unknown.** Its call to 8fb010 still projects byte outer+0x458=1 and DWORD outer+0x45c=0, but no evidence binds that outer object to constructed `SUICharacterInfoWnd`. The next investigation must establish an actual reaching target and receiver; neither localized text, neighboring callbacks nor shared field offsets provide that proof. A separate concrete recipient frontier is the registered object for ID 0xd3: retained bootstrap 9cb4e0 references this ID through 9beae0 before the 9e47b0 phase. Follow that source-backed registration route and its actual handler without substituting the already-typed ID-0xd7 waiting window.
