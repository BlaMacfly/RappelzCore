# SFrame indirect-entry search for 0x918630

Verified **2026-11-01** on original x86 SFrame SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`, with **REA 6.4.0 / Ghidra 12.1.4 / Capstone 5.0.7**. This continues [the auxiliary/writer investigation](SFRAME_AUXILIARY_PROVENANCE_RE.md). **No indirect caller of 918630 is confirmed in the studied routes. Its receiver and MAIN+0x458 activation remain unresolved.** New evidence consists of **3 Native dossiers / 203 analyzed bytes**, **24 exact instruction observations** and **four containment observations**, alongside bounded code/data/module inventories.

[Versions](VERSIONS.md) remain Core/RappelzRust **0.1.0 development**, NoLimits launcher **1.3.60**, original SFrame PE file/product **9.1.0.0**, embedded **Source9.5.2** build lineage and network literal/profile **201507080**. Exact official Epic/patch remains unconfirmed. The original client is not executed.

## Search questions and boundaries

| Route studied | Observation | Remaining boundary |
|---|---|---|
| Preceding function / alternate entry | 918580 exits through RET, followed by 11 INT3 bytes before 918630 | No ordinary fallthrough; computed/debugger/external entry is not disproved. |
| Registered exception handler | The SEH chain goes from an already-entered 918630 frame to cleanup of a local container | No reverse caller or initial entry is established; generic runtime effects are not fully reconstructed. |
| Computed code pointers | Two propagation scenarios find no candidate in 449 retained Native bodies and the executable byte sweep | Supported operations/paths only; heuristic alignment and unknown values are not complete interprocedural execution. |
| Relative/reconstructed table carriers | No examined DATA carrier targets entry/body; code coincidences are separated | Other encodings, compressed tables and runtime writes remain unknown. |
| Neighboring executables/libraries | No selected exact entry byte/string pattern in 40 neighboring files | Read-only literal inventory, not proof about loaded modules or runtime decoding. |

The retained 301-byte body still projects a real nonzero write:918720 forms EDI+0x454 and 918726 calls 8fb010, giving **BYTE outer+0x458=1** and **DWORD outer+0x45c=0**. No new observation binds that outer object to constructed MAIN. The prior canonical incoming query with zero observed references is reused, not repeated. Main/ArenaWaiting sharing formatter b3d4f0 at virtual+0x78 remains compatible with the caller's stack arguments and does not type 918630.

## Predecessor and exception path

New Native **918580..918624,165 bytes**, has return instructions at **918611/918624**. Original bytes **918625..91862f** are eleven **CC/INT3** bytes. This qualifies the missing ordinary predecessor-flow hypothesis; adjacency is not used as a class/receiver proof.

Inside 918630 the registered frame handler is **cfd408**. Its original bytes are **MOV EAX,e5da3c**, then opcode **JMP c5e93f**. The exact containment query returns **found:false/not_in_procedure** for cfd408, while both instructions are decoded by the provider. The **10-byte handler is therefore instruction/raw coverage, not a fabricated fourth Native function**. The supplier's flow classification **CALL_TERMINATOR** is retained separately from the E9/JMP opcode and disassembly.

The byte-bound metadata path is:

**918630 prologue→cfd408→FuncInfo e5da3c→UnwindMap e5da34→cfd400→6f1920**.

FuncInfo has magic **0x19930522**, one unwind state/action **(-1,cfd400)** and no encoded try/catch or IP-state continuation table in this selected metadata. New Native **cfd400, 8 bytes** forms ECX=&frame-local-container at EBP−0x1c, then tail-transfers to new Native **6f1920, 30 bytes**. The cleanup uses the same local-container operations as the normal path. Its receiver is not the outer window. State transitions **-1→0→-1**, including restoration before the 918726 reset, remain byte-checked.

This is an **outgoing cleanup path after frame installation**, not an incoming callback registration for 918630. It does not establish an inverse edge into the function, a MAIN receiver, or universal absence of exception/runtime redirection.

## Code-side address propagation

The scan reuses **449 Native bodies / 272,669 analyzed bytes / 78,529 instructions**, with complete source identities and original-body checks. A separate executable-section sweep consumes **9,589,248 bytes** as **3,370,267 decoded units**, including **1,003 bytes handled as skipdata**. The latter is complete in consumed bytes, **heuristic in instruction alignment and paths**; it is not promoted to millions of provider-confirmed instructions.

The bounded model propagates values from actual instruction sequences through supported MOV/LEA/arithmetic/bit operations, 8/16-bit partial registers, small memory writes and module/PC-based constructions. It looks for values reaching indirect call/jump, store and PUSH/RET-related sinks, rather than pairing arbitrary constants or repeating the old VA/RVA word scan. Unsupported operations, unknown memory, cross-function values and control-flow merges retain explicit losses of precision.

Both the generic-call scenario and the original-CFG-slot scenario produce **zero candidates into 918630..91875c** in the studied model. **15 synthetic model controls** and **10 distinct positive controls from original code** pass; synthetic controls are labeled as such and do not stand in for game execution.

The conditional scenario preserves registers across 433 modeled CFG checks **only if the original slot remains unchanged**. The generic scenario keeps its normal call clobbers. Dynamic slot replacement, unknown loader behavior, encoded/decrypted pointers, unmodeled arithmetic and external/runtime sources remain open. No whole-program no-caller or dead-code conclusion follows.

## DATA, vtable and relative-carrier search

New relative-carrier models examine **12,481,515 DWORD positions across the mapped scan ranges**, including **2,892,270 DATA positions**, at all studied byte alignments. They test signed/modulo32 forms based on the source slot, slot+4 and the six measured section/image bases. No examined DATA result reaches the entry or 301-byte body; no studied form reaches the exact entry. **87 new interior coincidences are all in .text**, without an established consuming table. The previously observed RVA overlap is retained separately, rather than counted again as a new discovery.

The retained table inventory checks **2,134 vtable prefixes / 40,833 fields**. Known CALL-relative and vtable/HIGHLOW cases serve as positive controls. This is a bounded search for source-supported carriers, not proof against every split, encrypted or dynamically populated pointer. The original TLS callback list begins with a null entry, and .fptable is zero on disk; no assertion that those locations stay null at runtime is made.

The new PE control evidence also preserves a layout discrepancy: **LoadConfig directory size 64 bytes, internal Size 160 bytes**. Local SDK layout offsets are checked explicitly. Within the internal-size span, the original GuardCFCheckFunctionPointer is **d279ac**, its slot points to **bc811c**, and **433 raw CALL[d279ac] encodings** are counted. The selected call instruction and stub are provider-confirmed; the 433 encodings are not all silently promoted to executed calls. Original SafeSEH/GuardCF function-table fields/counts are zero, and GuardFlags is 0x100. Loader interpretation of the 64/160 discrepancy remains unknown.

The selected **CALL[d279ac]→bc811c→RET** is a check/stub control, not a dispatcher to 918630. This distinction supports the two code-propagation scenarios without treating guard checks as candidate callers.

## Numeric overlaps and neighboring modules

All twenty previously raw-only **0f879100** overlaps are now provider-decoded **JA** instructions to other destinations. They are opcode/displacement bytes, not established pointers to 91870f. Together with the four earlier MOVZX/CMP/CALL observations, this strengthens the original numeric-carrier qualification without inventing a route to 918630.

A separate read-only inventory hashes **40 neighboring EXE/DLL/SYS/OCX/ASI files, 47,503,277 bytes total**, excluding SFrame itself. None matches the selected exact 32-bit entry VA, SFrame RVA, ASCII/UTF16 hexadecimal or decimal-entry patterns. All paths/hashes and zero-match results remain local. This covers selected physical files/patterns; it does not establish which modules load, decompile their code, or exclude split/decoded addresses and other resources.

## New observations

All new records bind the original target SHA and profile digest `b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17`. The three new Native graphs are untruncated; unknown runtime/callee effects remain separate.

| Native entry | Analyzed bytes | Evidence ID |
|---|---:|---|
| 0x918580 | 165 | `ev_aa32599fddde90584680130bd4bf1522a4f5e70355b17bdf6e60dc8702668644` |
| 0xcfd400 | 8 | `ev_4ed1384579eea83afdb028e0658915b7fc4a91a765db0e4f3cb086076e6dba25` |
| 0x6f1920 | 30 | `ev_7074deaa274f2c305aaf8ada29328d7aab2396b184be468b11a644a957fe629d` |

| Containment query | Provider procedure result | Evidence ID |
|---|---|---|
| 0x918580 | 0x918580 | `ev_9eb7ca2348cfc4b29eb7a646ff9988bf4e020d0d62deb08a1be242b8d63e3945` |
| 0xcfd408 | not_in_procedure | `ev_a4b509794fb19a72c62893b7234515ee3bcf6dbe5ec8912889b3a3efab1e12c9` |
| 0xcfd400 | 0xcfd400 | `ev_31d0782b5cd15ae59da43ffd001588006df4953c513c65fb69b735b1d88d920e` |
| 0x6f1920 | 0x6f1920 | `ev_3ebcac830d367dfefb00021e7d0b7e5b3df0e5fcf4c25415c6b6cecaec9f1601` |

| Exact instruction | Provider decode | Evidence ID |
|---|---|---|
| 0x52fccd | `JA 0x0052fd64` | `ev_c938236e0a066efa004ec29e679dcf90486bba7ac8bda81c3fb637746b49753d` |
| 0x5dc164 | `JA 0x005dc1fb` | `ev_b541c9bc70a7b1b7892266bf185c311e9c9dc61aa2381e8bd81a19b4f396b4a5` |
| 0x610cf0 | `JA 0x00610d87` | `ev_38246425911814b3cfdd077453229ce0d8ece3e8cc293fde74eb9a61c035d598` |
| 0x6c1d62 | `JA 0x006c1df9` | `ev_307ce243a9b0e9c2938fe40dba594d51ebef4d7246490d1a9e93a9f4311d4a70` |
| 0x6c1f02 | `JA 0x006c1f99` | `ev_aac90290a6a51f439c2ae0e8f707b191b2e9cf35ea67cca1378c23b923ea8c86` |
| 0x808694 | `JA 0x0080872b` | `ev_8e516c57f48a8c558866ec3cedd87b382b0c3f45f5fa4006cd4dca0529565624` |
| 0x82956f | `JA 0x00829606` | `ev_563c9a41ef621c5bb67ab02caa2da378c1dd6c918a0a9c7d09d5c1ddedec1620` |
| 0x859c8a | `JA 0x00859d21` | `ev_0a3256e43697a429786c26ca367f18a3b31ace02d0104b63399692c4a942d3d0` |
| 0x88b5e0 | `JA 0x0088b677` | `ev_34c7c793cd33e67c33ea5bbb0f894bded540d8dc8c3cfaef251187669f8844fd` |
| 0x88b720 | `JA 0x0088b7b7` | `ev_8dbe1bf98e2dd9adf16adbb54202213de162e454898ea2c4a2b57df3ab045127` |
| 0x89972a | `JA 0x008997c1` | `ev_00973a9c52c2c9d67fec6b8fc66f07e27b4f302c83632ffece707b5738fb360b` |
| 0x8e24bd | `JA 0x008e2554` | `ev_1fde0fa09a0ac19b261891550917d0232632f3e9ea0aa253d5473358e11e6269` |
| 0x8e5875 | `JA 0x008e590c` | `ev_7e37fef5fdedbc1f16185e123bc7998a95af91a6cdf21a27e849bd51fb08f444` |
| 0x8e5b25 | `JA 0x008e5bbc` | `ev_f1403296b5a5e9290986e0f0c96b5dfb2abb42ef3894dd6daf634370a4219c07` |
| 0x9964cb | `JA 0x00996562` | `ev_259139eac707d63979b8bb855a11ad0bdf336e273e106c9cd1570ce01be2570b` |
| 0x99689b | `JA 0x00996932` | `ev_00664eb06d446a3db7091706d8f3cb4f5f755f606acffe79a00da4eca7c0a7f0` |
| 0x9c141c | `JA 0x009c14b3` | `ev_98d8a219bef9881001c911c34a8d93dd29ce3e193ecf4033007ce720029b5887` |
| 0xb52adc | `JA 0x00b52b73` | `ev_8f9adcb5ee29629ead353b22999a5dbdea6247d470688f928c6e722fb99dfab5` |
| 0xbadae6 | `JA 0x00badb7d` | `ev_091d1a71eed6e8e12960fd365a980c991a60303c23f70fac7da72488396c9728` |
| 0xbc7467 | `JA 0x00bc74fe` | `ev_4d76992d8226f5141c4103ed786788d6d93605f9af346d43dc3150c3684dc9f5` |
| 0xb97f5a | `CALL dword ptr [0x00d279ac]` | `ev_05eb1717498843503a6351077ec6d3150140b6196e1631cd64e4826cadeea14b` |
| 0xbc811c | `RET 0x0` | `ev_a8872a183d89f91af248411eff6e370f144ceb5bd8fd7eb1597dee62882040be` |
| 0xcfd408 | `MOV EAX,0xe5da3c` | `ev_32bfa1867fdd334d7ee5a331eb4bf6a20ba1acef675e8d1dce9a85c643e3ea69` |
| 0xcfd40d | `JMP 0x00c5e93f` | `ev_422471790dd6eaf691422cd215ddf38ff8e53f57c3202576cfce26cf77a04822` |

| Selected reused Native source | Evidence ID |
|---|---|
| 0x918630 | `ev_2e11babc974a7b4241ded6116bc72be4cbafa3d434f034fc40c37e79a04da7bc` |
| 0x8fb010 | `ev_6311d6edbba44308e24bffdc741ec9c6fbccbb16ca26c14b03969b080907bce5` |
| 0xb3d4f0 | `ev_ad8725bbe05f2378247b79f6c19fb6a39e2b4122eebe575c6ae63730f4a888d5` |

The original incoming query **ev_01b55707ea22285c9103b4c429d9a29add3b1840d29c21e3181b50964ab4a75e** and the older exact-entry/relative searches are reused with their existing limits. Prior-producer repetition is not counted as progress.

## Verification and preservation

Independent original-byte checks confirm all new Native ranges, **64 new Native assembly boundaries**, **81 distinct selected ledger intervals** and **7 separately retained Native bodies**. Entry/computed/carrier receipts separately validate source hashes, the two propagation scenarios, metadata/control cases and canonical probes. The 40 external file hashes are checked again before publication.

The single owned Native session checkpoints these 31 observations, officially imports the prior **548-record canonical bundle**, exports and closes successfully. Every prior Evidence record remains equivalent as a JSON value. The output holds **579 canonical records / 24 cached primitives**, beginning from the retained light 21-primitive snapshot. Three Native functions and 28 other observations are kept distinct; archival Native records outside the canonical bundle retain their historical index.

Ignored run: `client-data/audit/2026-11-01-sframe-918630-indirect-entry-79d6b204`. Canonical bundle **272,584,995 bytes**; closed snapshot **283,637,940 bytes**, SHA-256 **`b16ffa51ccb4538de78ff7a45194a711e4c3f62ed93c429e1ff5b282e67e4c7b`**. Complete Native/probe/containment records, code/data/entry ledgers, model controls, module hashes, checkpoint, preservation and publication receipts remain local.

**Nine preservation checks pass.** Eleven baseline original/installed image, addon and settings hashes, saved-variable trees, preserved database backup and primary listeners/service identities stay unchanged. Root Git index remains empty. Only seven authored Markdown files are synchronized through isolated Core/Client publication worktrees; no original binaries/resources, databases, credentials, settings or generated decompiler output is published. No mode activation is reconstructed from an unbound receiver.

## Unresolved result

The selected ordinary predecessor, SEH, numeric, relative-table and modeled computed-pointer routes provide no confirmed indirect caller. The identity of EDI in 918630 and its relation to MAIN remain unknown. Future evidence must supply an actual computed/registered target and receiver provenance, or a controlled original-runtime observation with its own scope. These bounded negative results do not prove 918630 unreachable, nor do they identify MAIN's activation writer.
