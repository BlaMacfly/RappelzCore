# Original client identity and reverse engineering

Audit date: 2026-10-30. The supplied original game was inspected statically;
it was not launched or connected to any server.

## Conclusion

The supplied `Rappelz NoLimits/SFrame.exe` is a **32-bit, customized Rappelz
client**. Its exact official Epic/patch release is **not established**. Three
different version identifiers coexist:

| Evidence | Observed value | Meaning and limit |
| --- | --- | --- |
| PE VERSIONINFO file/product versions | `9.1.0.0` / `9.1.0.0` | Embedded resource declaration; may be inherited or stale. |
| Embedded build/debug paths | `Source9.5.2` | A build directory label, not an authoritative official-release identifier. |
| Loose launcher `version.txt` | `1.3.60` | Launcher/package version; does not establish the game protocol or Epic. |
| Embedded network-version literal | `201507080` | Present in this exact image; prior decompiler notes identify it as both auth/world version payload. |

Do not collapse these into “Epic 9.5.2” or select a public emulator solely by
its advertised Epic. The compatibility target is the binary hash plus its
observed packet layouts. Older `docs/re/model_format.md` and `ui_format.md`
headings use “v1.3.60”; read that as the package label above.

## Reproducible identity

| Field | Fresh static observation |
| --- | --- |
| Relative path | `Rappelz NoLimits/SFrame.exe` |
| Size | 12,481,536 bytes |
| SHA-256 | `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d` |
| Format / COFF machine | PE32 / `0x014c` (x86) |
| Image base / entry VA | `0x00400000` / `0x00be512f` |
| Linker version declaration | `14.16` |
| COFF timestamp raw | `1785513614` |
| COFF timestamp interpreted as UTC | `2026-07-31T16:00:14+00:00` |
| Embedded compile-date/time strings | `Jul 31 2026`, `17:53:58` (timezone unknown) |
| Resource company / product | `GALALAB` / `Rappelz Client` |
| Version resource language | `1042` (Korean) |

Timestamps are embedded declarations and can be changed by build/patch tools.
They do not prove distribution date or authenticity. Architecture is established
by the PE headers, independently confirmed by REA `open_binary`.

The new standard-library-only audit can be rerun from the workspace root:

```powershell
python tools/audit/inspect_sframe.py "Rappelz NoLimits/SFrame.exe"
```

It reads the file, parses headers/normal imports/VERSIONINFO, hashes bytes,
and reports selected ASCII string locations. It does not load DLLs or execute
the target. PowerShell's `ConvertFrom-Json` can convert ISO dates to the local
timezone when displaying them; the script's raw UTC string is authoritative
for the interpretation recorded above.

## Fresh byte anchors

VAs below assume the declared image base; offsets refer to original file bytes.

| Value | File offset | VA | Evidentiary limit |
| --- | --- | --- | --- |
| `201507080` | `0x96f1e4` | `0x00d70be4` | Literal exists; use by the network comes from function/reference evidence. |
| `C:\Temp\net_debug.txt` | `0x96f444` | `0x00d70e44` | Debug logging path exists; no runtime write was observed. |
| `Jul 31 2026` | `0x991584` | `0x00d92f84` | Compile-date-like string. |
| `17:53:58` | `0x991590` | `0x00d92f90` | Compile-time-like string. |
| `source9.5.2\library\internal\source\network\xsyncstreamconnection.cpp : XSyncStreamConnection::Read` (full drive prefix in file) | `0x9fc3a8` | `0x00dfdda8` | Build/assertion path; does not recover original source. |
| `C:\Users\Administrator\Downloads\Source9.5.2\client\Game\output\Release_ME\SFrame.pdb` | `0xa29664` | `0x00e2b064` | Embedded PDB path; no matching PDB was supplied/found. |

The source-path label, private debug logging strings, launcher branding, and
the opcode differences recorded below support the **inference** of a custom
rebuild/patch. They do not establish source provenance or an official release.

## PE resources and imports

REA's portable `inspect_pe_resources` returned complete inventory coverage:
30 resource leaves, 68 directory entries, PE32 machine decimal 332 (`0x014c`).
Its Evidence ID is
`ev_ac5224ee2dff3438a30f214985b80099ec7f65a6ef34d711565d11d085e3d65b`.
The type-16/name-1/language-1042 VERSIONINFO payload is 728 bytes at file
offset `0xb7a070`, SHA-256
`dfca2b32e6cdf96a5068aeefd0d12d8aa90bdaf9bd6bd5a91c081c3602633386`.
The independent Python parser reaches the same payload through the resource
tree and reads the fixed file/product versions as `9.1.0.0`.

Normal imports name 30 DLLs:

```text
KERNEL32.dll, WS2_32.dll, IMM32.dll, VERSION.dll, d3d9.dll, DevIL.dll,
ILU.dll, libgobject-2.0-0.dll, libcairo-2.dll, libpango-1.0-0.dll,
libpangocairo-1.0-0.dll, USER32.dll, GDI32.dll, COMDLG32.dll,
ADVAPI32.dll, SHELL32.dll, ole32.dll, OLEAUT32.dll, WINMM.dll,
IPHLPAPI.DLL, freetype6.dll, libiconv2.dll, dbghelp.dll, WINTRUST.dll,
CRYPT32.dll, WININET.dll, SHLWAPI.dll, PSAPI.DLL, audiere.dll, mss32.dll
```

This confirms static dependencies on Direct3D 9, Winsock and the listed
rendering/audio libraries. It does not trace optional dynamic loading or prove
that each imported function runs. `lua5.1.dll` exists beside the executable but
is not in its normal import table; LuaVM build/assertion strings exist in the
executable. These facts alone do not establish how that DLL is used.

Selected supplied DLL identities, independently parsed:

| File | Architecture | Bytes | SHA-256 |
| --- | --- | --- | --- |
| `lua5.1.dll` | x86 | 325,960 | `12cb9b8f59c00ef40ea8f28bfc59a29f12dc28332bf44b1a5d8d6a8823365650` |
| `d3dx9_40.dll` | x86 | 4,379,984 | `16fd909aeb68d0d1aca8529dc7f78880b97d6649d70ce8d03a2c858bc28e216b` |
| `audiere.dll` | x86 | 512,000 | `90e59c4f60941a2838d46e20fb78ba0cf71e9cfd0782e291ca876645e3794a66` |
| `mss32.dll` | x86 | 386,048 | `44ceeffd9cb85845d1e068b45f7cc571e4f314c5328b00721d5bcfd3e26b45b8` |

These x86 libraries cannot simply be loaded by the new x86_64 Rust client.
The Rust addon runtime is a separate implementation; the original game's
internal Lua support does not provide the requested WoW-style addon framework.

## Recovered analysis and protocol target

Existing detailed notes remain the primary per-function ledger:

- `docs/re/network_protocol.md`: framing, stream cipher, authentication,
  lobby, world packets, gameplay structures, and observed/inferred field labels.
- `docs/re/data_archive.md`: archive index, name codec, shard selection, XOR.
- `docs/re/rdb_format.md`: fixed/grouped resource databases and field evidence.
- `docs/re/terrain_format.md`, `model_format.md`, `ui_format.md`: original
  terrain, model/animation and native-interface formats.

These notes explicitly name Ghidra 12.1.4 and the matching SFrame SHA-256.
Saved decompiler text and scan helpers survive under ignored
`client-data/proto-re/` (`parser.c`, `world.c`, `mgr.c`, `dialog.c`, `skill.c`,
`tmnames.c`, `tm.txt`, and JS scan scripts). These are decompiler/analyst
artifacts, not recovered original C/C++ source. No `.gpr`, `.gzf`, or REA
analysis snapshot was found in the workspace outside `target` at audit start.
The new REA session initially had no active binary or retained native analysis.

On this pass Ghidra 12.1.4 was available and the hash-matched binary was opened,
but its initial `binary_overview` request exceeded the MCP client's 300-second
call timeout. The provider process was no longer running afterward; **no fresh
native decompilation/overview was returned**. The expensive request was not
repeated. This does not invalidate the fresh PE/resource observations or the
older hash-matched notes, but those older function claims remain distinguished
from this audit's results.

Available session evidence was exported, then the native session was closed:

| Retained file (ignored development data) | Contents and limit |
| --- | --- |
| `client-data/proto-re/sframe-2026-10-30-evidence.json` | One complete resource-inventory Evidence record, 26,181 bytes. |
| `client-data/proto-re/sframe-2026-10-30.snapshot.json` | Hash/profile-bound REA snapshot, 27,788 bytes; one Evidence record, **zero native primitive/workflow query answers**. It is not a completed Ghidra database and cannot replay a native analysis that timed out. |

Both files identify the SFrame SHA-256 recorded above. They retain resource
evidence without requiring that parser to be rerun. Future native work should
use a transport whose call deadline accommodates the initial Ghidra analysis,
then save focused successful query answers; do not treat this snapshot as a
completed disassembly.

Important **previously recorded** protocol findings (not a new packet capture):

| Feature | Existing evidence/implementation | Compatibility implication |
| --- | --- | --- |
| Header | `u32 size`, `u16 id`, `u8 checksum` (7 bytes) | Use the Rappelz framing crate, not WoW framing. |
| TCP cipher | Custom XRC4 KSA plus drop 1013; separate directional states | Ordinary RC4 with the same key is incompatible. |
| Auth exchange | RSA public key / encrypted AES key, encrypted account credentials | SRP6 is a separate optional service path; it is not this original client's handshake. |
| Auth/world version payloads | `201507080` in 20-byte fixed string fields | Retain an explicit protocol profile, independent of launcher/Epic labels. |
| Custom world opcodes | login `62`, movement `63`, login result `64` | Embedded debug name table gives older values for these; constructors/parser take precedence. |
| Character record / item record | 632 / 331 bytes | Compare field widths and offsets, not just packet names. |

The existing Rust `shared/rz-proto` reflects these values, including
`CLIENT_VERSION`, custom opcodes, cipher vectors, and structured messages.
See its tests and the current development status for fresh Rust validation.

## Archive corroboration

Fresh read-only command:

```powershell
node docs/re/tools/rappelz_data.js "Rappelz NoLimits" stats
```

Result: **125,282** entries. All eight `data.001`–`data.008` shards have exact
indexed coverage, zero overlaps and zero gap bytes. The index is 5,271,103
bytes. Counts include 40,285 DDS, 27,519 JPG, 22,072 NX3, 20,825 NAF,
4,312 COB, 136 NFM, 69 RDB, 485 NUI and 7 Lua resources. This freshly
corroborates the prior archive-count claim. It is not a claim that every model,
texture or script has been decoded or rendered successfully.

## Remaining uncertainty and next probes

1. Establish a protocol profile through byte-for-byte fixture comparisons and
   a controlled local login/movement exchange. No live packet capture of the
   original executable was performed in this audit.
2. Retain explicit field-level uncertainty from the existing notes, especially
   combat hit fields, quest status, learn-skill and market field semantics.
3. Obtain authoritative release/build provenance before assigning an exact
   official Epic or patch number. Neither metadata nor folder labels suffice.
4. Resolve original asset conversion/rendering failures using the existing
   parser tests and the hash-matched static entry points; preserve originals.

The original executable and supplied DLL/resource files remain unchanged.

## Progression follow-up, 2026-10-31

[Fresh job-level/resource validation](JOB_PROGRESSION_VALIDATION.md) supersedes the absence of successful native answers for this new cycle: REA6.4.0/Ghidra12.1.4 CLI returned function **0x006d7d50–0x006d7da6**, 87bytes, Evidence `ev_0277dcfe82d7a5652d55d27f9007df42e5b927a82e4033b81a113ce66e9072bc`, on the same SHA-256-bound SFrame. It constructs ID410, length11, normal checksum and one four-byte argument at +7. The MCP lookup still timed out at300seconds; its zero-answer snapshot is distinguished from the successful CLI snapshot/Evidence retained under the new ignored validation run. No SFrame runtime was executed. Archived BasicStat/Exp91/SummonExp are also recovered with exact schema/hash/row evidence and lossless companions. Current-job purchases are implemented and verified with two Rust clients; original cipher/version interoperability, cumulative character EXP, derived/job-history stats and exact official Epic remain unresolved. No decompiler output or original source/assets are published.

## Bounded job-growth and promotion investigation, 2026-10-31

The [character-stat report](CHARACTER_STATS_VALIDATION.md) records a read-only archive/text/reference investigation, without a new REA native session. Across125,282 index entries,69 named RDBs and89 candidate textfiles/400,576bytes, no named JobLevelBonus resource or promotion server-script body is identified. This bounded negative result does not exclude opaque constants elsewhere. Archived Job has45 records of292bytes; all42 common loose rows match, while extra105/115/125 have no BasicStat row and remain inactive. Original NPC contact strings identify promotion entry points, not their missing eligibility/reset/JP/history behavior. The pinned primary reference uses separate server SQL bonus coefficients. Actual current AP/DEF/speed can now be displayed and refreshed safely, but effective JLV bonuses, profession transitions and original SFrame interoperability remain unproved. Raw resource ledgers stay local/ignored; no proprietary payload or pseudocode is published.

## Fresh progression/property receivers, 2026-10-31

[SFRAME_PROGRESSION_RE.md](SFRAME_PROGRESSION_RE.md) records nine fresh hash-bound native dossiers: PROPERTY parser/own receiver/world handler, four current-job/JLV setter layers, own STAT_INFO consumer and party command handler. Both words of numeric PROPERTY are preserved; own JP is64-bit while job/JLV/three previous-job-level pairs are independent32-bit assignments. Stat snapshot dispatch and the party `CHANGE_JOB` false lead are documented with exact addresses, body extents, Evidence IDs and limits.37 original PE anchors independently match. The300-second integrated attempt is a zero-answer failure; a bounded longer-lived client uses the same installed public REA stdio interface and closes after successful export. No original executable execution or installed gameplay change is claimed. Server reset/JP/history/bonus rules and runtime interoperability remain unknown.

## Stat-bank receivers and direct UI consumers, 2026-11-01

[The next static pass](SFRAME_STAT_BANKS_RE.md) follows the three retained seeds through11 fresh REA6.4.0/Ghidra12.1.4 dossiers, including direct primary/derived plus-value consumers and two UI selectors. Type0 copies a main bank without clearing its partner; type1 copies a secondary contribution and subtracts selected fields, so duplicate/order effects cannot be ignored. Matching-handle/type0 actor gating, the actual four virtual slots and seven signed primary field positions are corroborated.54 PE anchors and seven original/archive/converted tooltip matches pass. Four large value-flow graphs are truncated; complete analyzed bodies/pseudocode/provider limits are retained separately. Resource decoding explicitly starts XOR phase0 at the selected archive entry. The old corpse-tooltip replacement-character statement is corrected from actual UTF8 bytes. No original execution, Rust gameplay change, service/data mutation, package-version bump or original server stat-formula claim is added.

## Queues, callback prepasses and window lifecycle, 2026-11-01

[The dispatch follow-up](SFRAME_DISPATCH_RE.md) retains 23 hash-bound native dossiers, including a completed large game-handler record recovered from its canonical export after a 10 MiB stdio response budget. It was not re-decompiled; two focused source-immutable sessions contain 13 + 10 distinct function queries. Constructor/visibility resets, Basic/Added RTTI, the selected-handle cache setter, local-vector dispatch, two priority deques, post-callback status paths and a kind-keyed prepass are documented. 123 PE anchors/seven typed links/five candidate interval hashes independently match. Literal pointer searches do not prove a concrete receiver bridge; nonzero-mode activation, callback registrations and global UI ordering remain unresolved. Both sessions close, raw Evidence stays ignored, and original/installed data/services/versions are preserved. No new runtime or original server-formula claim is introduced.

## Concrete target-info activation and message recipients, 2026-11-01

[The recipient continuation](SFRAME_MODE_RECIPIENTS_RE.md) adds 36 REA/Ghidra function dossiers and four focused reference/string queries. Exact button-table bytes bind target_info_button to action 6 in SUITargetWndAddon; its producer posts SIMSG_UI_SEND_DATA(target_info_open, handle) and SHOW_UIWINDOW to ID 0x5d. The distinct target-window consumer, handle/selection guards, actor rendering and cached-stat fallback are traced. Constructor/RTTI/receiver-adjustment evidence closes component owner+4 → SGameWorld → SGameManager → SGameInterface. SHuntaHolicSystem has eight explicit constructor registrations, with registry insertion and virtual callback dispatch corroborated. Main-window +0x458 activation stays unresolved despite the broader literal/local-alias scan; external/generic writes and runtime registry contents are not excluded. Two oversized retained dossiers were recovered without reanalysis; two value-flow graphs remain truncated. The owned session is closed, raw Evidence stays ignored, and original/installed data, services and versions are preserved.

## STAT_INFO handle predicates and guarded call order, 2026-11-01

[The next static pass](SFRAME_STAT_ROUTING_RE.md) adds 27 native dossiers. The two supposed lookup seeds are short SUIDisplayInfo context getters; a pushed handle remains for the later predicate. SPlayerInfoMgr checks nonzero own-handle equality, while SCreatureSlotMgr checks the first matching record against six identifier fields. The separate storage predicate has weaker conditions and a zero-handle edge. The guarded normal world path calls core stats before interface forwarding, explicitly setting handled before that forwarding; no universal handled-stop contract follows. Constructed load, secondary-context and display-prehook kind 31 return paths are byte-verified. Alias tracking across calls finds seven more stores, all unrelated classes; main CharacterInfo+0x458 activation stays unresolved. Two new graphs are truncated. Raw Evidence remains ignored, the owned session closes, and original/installed data, services and explicit versions are preserved.

## Loading-gauge source rectangle and concrete virtual recipient, 2026-11-01

[The widget continuation](SFRAME_WIDGET_DATA_RE.md) adds 18 native dossiers and five focused metadata queries. The 82cb97 source belongs to SUILoadingProgressWnd’s saved loading_frame_02 rectangle, not a typed main-character mode field. Original NUI entry/index/cipher/plain/cache checks bind its static declaration; the default registration/factory/RTTI resolve KUIControlStatic virtual+0x1c to 9a6bb0 and base rectangle assignment. Native resource-name/descriptor creation is separate from cloning, template and programmatic-window consumers. Finite update cases reset, partially fill or set a 198-coordinate span, with copied rectangles and third-control repositioning. Main mode activation, full parser grammar/current overrides and complete indirect effects remain open. Two new graphs are truncated; the session is closed, raw evidence remains ignored, and original/installed data, services and versions are preserved.

## CharacterInfo callbacks and PC-restart recovery, 2026-11-01

[The callback continuation](SFRAME_CHARACTER_CALLBACKS_RE.md) retains 15 fresh native dossiers / 1,516 analyzed bytes after recovering the previous immutable snapshot. Name selection/refresh, main cached dimensions, the distinct context used for list reinsertion, child+0x20 owner assignment and shared child event tail-jump are bound to original bytes. The three archived character-sheet resources match their index/cipher/plain/cache identities. All 15 new graphs report untruncated; indirect/callee/alias limits remain. Main+0x458 activation and exact fold/unfold action binding stay unresolved. The owned engine closes with 115 Evidence records/ 106 primitives; nine preservation checks pass after existing services resume with recorded new post-reboot identities. Only authored documentation is published, with unchanged explicit versions.

## Alternate-mode writer provenance, 2026-11-01

[The activation investigation](SFRAME_MODE_ACTIVATION_RE.md) adds 21 native dossiers / 8,563 analyzed bytes and five focused queries. The concrete WORD+0x458 writer is connected to SUIGaugeWnd via its 0x470 allocation, constructor and interface+0x74 registration/lookup; MAIN uses a distinct registered-window lookup at+0x24. Panel and game-object vector stores receive different typed instances. Main guild/height helpers, the raw child+0x448 getter and the non-clamping nine-rectangle initialization guard are retained. The935569 store belongs to 9354c0; neighboring 935480 does not establish a caller bridge. All 145 prior candidates are re-read with six wide zero-source copies verified, without global absence claims. Main activation remains unresolved. All 21 new graphs report untruncated; the session closes with 141 records/ 127 primitives. Nine preservation checks and unchanged service/image/settings/version identities pass; only authored documentation is published.

## Indirect character-sheet writes, 2026-11-01

[The indirect-write pass](SFRAME_INDIRECT_WRITES_RE.md) adds 25 native dossiers / 19,426 bytes and ten focused observations. Child+20→MAIN getter/geometry calls, child+448 and generic+13c writes are distinguished from mode+458. The named-control text route and default localized-caption handler are bound. A corrected scan covers affine addresses through calls and exact ADD forms; six new immediate writes have zero target bytes, while QuestReward/Arena/ColorBox receivers are typed. Three new graphs are truncated with explicit limits. Main activation remains unresolved; candidate pointer deliveries/implicit copy lengths are not a global absence proof. The closed snapshot holds 176 records/ 152 primitives; nine preservation checks pass and only documentation changes, with images/services/data/settings/versions preserved.


[The implicit-copy pass](SFRAME_IMPLICIT_COPIES_RE.md) adds 13 native dossiers / 11,342 bytes and five containment observations. All 41 retained implicit seeds now have counts and DF-qualified spans; 36 exclude relative+0x458 in both directions, while five 340-byte copies use a settings singleton. System Environment/Play callbacks and parent-to-child pointer changes are checked with exact RTTI and original instructions; helper 0x812a90 remains unbound. Adjacent single-store groups, untyped/interior bases and two truncated graphs retain explicit limits. MAIN activation is unresolved. Closed snapshot 194 records / 165 primitives; 542 byte intervals and 2,872 assembly boundaries verified. Nine preservation checks pass, with images/settings/services/data/versions unchanged; documentation only.

[The helper-caller pass](SFRAME_HELPER_CALLERS_RE.md) checks ordinary static references, every-byte relative encodings, 90 VA/RVA carrier patterns and 210,912 HIGHLOW entries for 0x812a90..0x812abc. No ordinary incoming caller is established; an apparent branch lies inside another CALL. A byte-checked equivalent button_def branch is typed as Environment settings, without proving an actual call to the helper. 4 new native dossiers / 5,487 bytes, with explicit graph limits; closed snapshot 201 records / 169 primitives. MAIN activation and computed/runtime entry remain unresolved. Nine preservation checks pass; original/installed files, data, services and versions unchanged.

[The pointer-delivery pass](SFRAME_POINTER_DELIVERIES_RE.md) reviews all 102 retained affine candidates with 102 new native dossiers / 28,405 bytes, 74 containment observations and one reference query. 98 selected sites are provider-contained; four remain raw-only. 13 ECX tags are incidental, while actual pushed pointers retain separate qualification. Other-class container/string/control and countdown paths are bound; no MAIN activation writer is demonstrated. A typed ArenaWaiting callback resets countdown byte+458, while 0x918630 remains receiver-unbound; virtual sites 99/100 remain limited. 378 records / 271 primitives are closed and saved, nine preservation checks pass, and installed files/data/services/versions are unchanged.

[The concrete MAIN flow pass](SFRAME_MAIN_POINTER_FLOW_RE.md) adds 68 Native dossiers / 14,797 body bytes and 19 containment queries. All 59 MAIN entries and both 69-entry child tables are covered by Native or short-entry byte proofs. Startup construction, registry context+24, key 2e lookup and 12 actual message sends bind real MAIN pointers. Lazy fallback 2e returns zero; geometry/interior aliases and karma/PK/Arena display paths do not identify activation. Ten archived individual Native records are reused separately from the prior bundle. MAIN mode remains unresolved with explicit graph/flow/indirect limits; 465 canonical records / 339 primitives saved, nine preservation checks pass, original/installed files/settings/services/data/versions unchanged.

[Indirect callbacks](SFRAME_INDIRECT_CALLBACKS_RE.md) adds 22 Native dossiers / 14,107 body bytes. The 315-table static KUIWnd family closes selected rectangle slots; constructed SGameManager/Interface kind 0x48f carries copied rectangles, while a separate creation descriptor carries MAIN as owner. Static resources bind KUIControlStatic creation and descriptor forwarding. MAIN+458 activation remains unresolved; runtime replacement, caption callback, remaining creators/callee effects stay bounded. 487 canonical records / 21 fresh primitives saved; old 339-primitive snapshot and historical Native index preserved. Nine preservation checks pass, original/installed data/settings/images/services/versions unchanged; documentation only.

[Caption and child-owner callbacks](SFRAME_CAPTION_OWNER_RE.md) adds 25 Native dossiers / 10,725 body bytes. The constructed caption handler translates MAIN+94 string data; original parser callbacks bind resource tokens to class descriptors and factory keys. Immediate MAIN vs Basic/Added owners, selected child virtual+dc calls, tab-head owner-list source and upstream registry insertion remain distinct. MAIN+458 activation remains unresolved with explicit graph/string/indirect limits. 512 canonical records / 46 primitives saved; older snapshots and archival Native records preserved. Nine preservation checks pass, original/installed data/settings/images/services/versions unchanged; documentation only.

[Selected remaining callbacks](SFRAME_REMAINING_CALLBACKS_RE.md) adds 14 Native dossiers / 3,264 body bytes. MAIN-list copying stores pointer payloads in separate nodes, with raw-only temporary cleanup; the manager reorders MAIN and changes generic byte+13c, while exact selected virtual48/4c targets RET. Six control families share virtual+d4; iconstatic self+44 and a conditional KSpritePrimitive destructor are bound. Unknown auxiliary subtypes and reentrant/downstream effects remain explicit. MAIN+458 activation is unresolved. 526 canonical records / 35 primitives saved; older snapshots and archived Native preserved. Nine preservation checks pass, original/installed data/settings/images/services/versions unchanged; documentation only.

[Tooltip auxiliary provenance](SFRAME_AUXILIARY_PROVENANCE_RE.md) adds 18 Native dossiers / 3,775 body bytes and four instruction observations. Six scalar25c constructions bind KLazyTip-family objects; generated260/27c objects bind KUITipControl with initial owner0; four264 input paths bind rp::KLazyItemTooltip. Reference-counted and direct-deletion paths stay distinct. The918630 projection writes outer458=1 but remains receiver-unbound; entry/carrier searches and compatible virtual78 retain computed/runtime limits. MAIN activation remains unresolved. 548 canonical records / 39 primitives saved; all older evidence preserved. Nine preservation checks pass, original/installed data/settings/images/services/versions unchanged; documentation only.

[918630 indirect entry](SFRAME_918630_INDIRECT_ENTRY_RE.md) adds three Native dossiers / 203 bytes and 28 other observations. Predecessor RET/INT3, outgoing SEH cleanup, relative DATA/table models, 40 external files and two computed-code scenarios yield no confirmed caller. Twenty old numeric JA overlaps are now provider-confirmed; CFG check/stub is separate from actual dispatch. Both code scenarios cover 449 Native bodies and 9.59MB heuristically, with explicit synthetic-control and runtime limits. MAIN receiver and mode activation remain unresolved. 579 canonical records / 24 primitives saved; older evidence preserved. Nine preservation checks pass, original/installed data/settings/images/services/versions unchanged; documentation only.
