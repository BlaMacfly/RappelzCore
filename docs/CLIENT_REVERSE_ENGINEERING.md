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
