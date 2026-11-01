# SFrame implicit character-sheet copy candidates

**Follow-up:** [Caller investigation for 0x812a90](SFRAME_HELPER_CALLERS_RE.md) checks direct transfers, original literal pointer carriers and relocations, rejecting an apparent branch inside another CALL. A typed default-settings branch duplicates the helper’s selected core effects, without establishing a caller. MAIN activation and computed/runtime entry remain unresolved.

Verified **2026-11-01** on the original x86 SFrame SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`, using **REA 6.4.0 / Ghidra 12.1.4** and an independent **Capstone 5.0.7** decode. This continues [the indirect-write investigation](SFRAME_INDIRECT_WRITES_RE.md): **all 41 retained implicit MOVSD/STOSD seeds now have locally bounded counts and destination spans**, with **13 new native function dossiers / 11,342 analyzed body bytes** and five procedure-containment observations. **Activation of MAIN SUICharacterInfoWnd byte+0x458 remains unresolved.** No nonzero MAIN-typed writer was recovered.

[Versions](VERSIONS.md) remain Core/RappelzRust **0.1.0 development**, NoLimits launcher **1.3.60**, original SFrame PE **9.1.0.0**, embedded build lineage **Source9.5.2**, network literal/profile **201507080**. The exact official Epic/patch remains unconfirmed. This pass is static; no original-client execution or live object observation is claimed.

## Count and span coverage

The source inventory is the previous corrected affine scan, SHA-256 `91770d6c02286c4eb02f9a10f682c0df70c59d58ec58d80288703fc3ed7e8147`. Its 41 seeds are candidate instruction sites, not 41 exhaustive or disjoint copy sequences. Destination offsets retain that inventory's candidate base; an unidentified base may itself be an interior pointer. These bounds therefore do not prove the global absence of a MAIN writer.

All retained string operations use DWORD elements. For destination offset **d** and count **n**, the inclusive written span is **[d,d+4n−1]** with DF=0, or **[d−4(n−1),d+3]** with DF=1. The selected fragments contain no explicit CLD/STD; both directions are retained. Forward execution is conditional on DF=0, flat ES/data addressing, valid memory and successful completion. A repeated outer caller or a differently adjusted receiver requires its own proof.

**36 seeds do not reach relative+0x458 in either direction. Five REP MOVSD sites copy 85 DWORDs (340 bytes) and reach that relative byte only in the forward direction.** The adjacent single-instruction groups are also bounded: eight four-instruction groups (16 bytes) and two three-instruction groups (12 bytes), including the final STOSD group whose stores are separated by MOV ECX,ESI. No adjacent group changes the relative-overlap result.

| Ordinal | Original site | DWORDs at seed | Destination offset | DF=0 inclusive | DF=1 inclusive |
|---:|---|---:|---|---|---|
| 0 | 0x40c783 | 8 | 0xa0 | 0xa0..0xbf | 0x84..0xa3 |
| 1 | 0x4189f0 | 8 | 0xa0 | 0xa0..0xbf | 0x84..0xa3 |
| 2 | 0x451456 | 127 | 0x6c | 0x6c..0x267 | -0x18c..0x6f |
| 3 | 0x451460 | 8 | 0x278 | 0x278..0x297 | 0x25c..0x27b |
| 4 | 0x4a8f1b | 1 | 0x10 | 0x10..0x13 | 0x10..0x13 |
| 5 | 0x4a8f2b | 8 | 0x20 | 0x20..0x3f | 0x4..0x23 |
| 6 | 0x4aab54 | 8 | 0x10 | 0x10..0x2f | -0xc..0x13 |
| 7 | 0x4aab59 | 1 | 0x44 | 0x44..0x47 | 0x44..0x47 |
| 8 | 0x4aab85 | 8 | 0x10 | 0x10..0x2f | -0xc..0x13 |
| 9 | 0x4aaba7 | 1 | 0x44 | 0x44..0x47 | 0x44..0x47 |
| 10 | 0x4c13d0 | 16 | 0x2c4 | 0x2c4..0x303 | 0x288..0x2c7 |
| 11 | 0x4ee465 | 1 | 0x24 | 0x24..0x27 | 0x24..0x27 |
| 12 | 0x4fd5c3 | 7 | 0x1b4 | 0x1b4..0x1cf | 0x19c..0x1b7 |
| 13 | 0x4fd5ce | 7 | 0x1d4 | 0x1d4..0x1ef | 0x1bc..0x1d7 |
| 14 | 0x4fd5f4 | 1 | 0x164 | 0x164..0x167 | 0x164..0x167 |
| 15 | 0x617d07 | 256 | 0xc | 0xc..0x40b | -0x3f0..0xf |
| 16 | 0x61f6c1 | 256 | 0xc | 0xc..0x40b | -0x3f0..0xf |
| 17 | 0x7ffc9e | 50 | 0x60 | 0x60..0x127 | -0x64..0x63 |
| 18 | 0x80b9ce | 85 | 0x44c | 0x44c..0x59f | 0x2fc..0x44f |
| 19 | 0x80d974 | 85 | 0x44c | 0x44c..0x59f | 0x2fc..0x44f |
| 20 | 0x812aa9 | 85 | 0x44c | 0x44c..0x59f | 0x2fc..0x44f |
| 21 | 0x812b1e | 85 | 0x44c | 0x44c..0x59f | 0x2fc..0x44f |
| 22 | 0x812bb6 | 85 | 0x448 | 0x448..0x59b | 0x2f8..0x44b |
| 23 | 0xab6c50 | 25 | 0x158 | 0x158..0x1bb | 0xf8..0x15b |
| 24 | 0xbca6e5 | 1 | 0x8 | 0x8..0xb | 0x8..0xb |
| 25 | 0xbcab4b | 1 | 0x8 | 0x8..0xb | 0x8..0xb |
| 26 | 0xbcf713 | 11 | 0x18 | 0x18..0x43 | -0x10..0x1b |
| 27 | 0xbcf779 | 11 | 0x18 | 0x18..0x43 | -0x10..0x1b |
| 28 | 0xbd1775 | 11 | 0x8 | 0x8..0x33 | -0x20..0xb |
| 29 | 0xbd17ac | 11 | 0x8 | 0x8..0x33 | -0x20..0xb |
| 30 | 0xbd7e25 | 11 | 0x2c | 0x2c..0x57 | 0x4..0x2f |
| 31 | 0xbd7efb | 11 | 0x2c | 0x2c..0x57 | 0x4..0x2f |
| 32 | 0xbd7fcb | 11 | 0x8 | 0x8..0x33 | -0x20..0xb |
| 33 | 0xbd7fef | 11 | 0x8 | 0x8..0x33 | -0x20..0xb |
| 34 | 0xbd8038 | 1 | 0x8 | 0x8..0xb | 0x8..0xb |
| 35 | 0xbd804b | 11 | 0x18 | 0x18..0x43 | -0x10..0x1b |
| 36 | 0xbd8071 | 1 | 0x8 | 0x8..0xb | 0x8..0xb |
| 37 | 0xbd8084 | 11 | 0x18 | 0x18..0x43 | -0x10..0x1b |
| 38 | 0xbdf853 | 11 | 0x18 | 0x18..0x43 | -0x10..0x1b |
| 39 | 0xbe16b6 | 11 | 0x2c | 0x2c..0x57 | 0x4..0x2f |
| 40 | 0xcc61d7 | 1 | 0xc | 0xc..0xf | 0xc..0xf |

The single-instruction seeds at 0x4a8f1b,0x4aab59,0x4aaba7,0x4ee465,0xbca6e5,0xbcab4b,0xbd8038,0xbd8071 start four-store groups. Sites 0x4fd5f4 and 0xcc61d7 start three-store groups. The table describes each seed; the ignored ledgers retain the additional instructions and group intervals separately.

## Five long copies share a settings source

Fresh **0x9f5c80** returns the fixed global address **0xed5138**. Its TLS/static guard at 0xed569c conditionally calls fresh **0x9f4dd0** and registers a destructor. Fresh initializer **0x80b230** is called on the source base and base+0x154: these are two adjacent 340-byte blocks. The subsequent+0x2a8/+0x2ac/+0x2b0 fields are initialized as a vector's pointer triple, rather than another inferred block. Later initialization helpers can change values; the original file image and initializer alone do not establish the live copied byte.

The source's settings role is supported by its consumers: resolution/refresh choices in 0x80d620; brightness, terrain distance, texture mip bias and volume controls in 0x80e470; minimap controls in 0x812b80. This is a behavioral identification of the selected data, not a recovered C++ class name for the global.

| Copy site / containing entry | Source relative to 0xed5138 | Forward destination | Source byte corresponding to destination+0x458 | Receiver evidence and limit |
|---|---|---|---|---|
| 0x80b9ce /0x80b9b0 | +0x154 | +0x44c..+0x59f | global+0x160 | Typed Environment callback 0x80e470 passes its own preserved receiver to helper 0x80b9b0. |
| 0x80d974 /0x80d620 | +0x154 | +0x44c..+0x59f | global+0x160 | Environment vtable+0xd8; entry receiver spills to EBP−0x14 and is reloaded before the copy. |
| 0x812aa9 /0x812a90 | +0 | +0x44c..+0x59f | global+0x0c | No direct callers/incoming references returned. Actual receiver type and reachability remain unresolved. |
| 0x812b1e /0x812b00 | +0x154 | +0x44c..+0x59f | global+0x160 | SystemWnd 0x80e110 and tab helper 0x811b50 explicitly change ECX to stored pointer+0x450. Named environment child route; no claim that the original parent remains ECX. |
| 0x812bb6 /0x812b80 | +0x154 | +0x448..+0x59b | global+0x164 | Typed Play callbacks 0x80da10/0x810040 call with their own receiver; SystemWnd/tab routes use stored pointer+0x454. |

For the first four copies DF=1 writes+0x2fc..+0x44f; the fifth writes+0x2f8..+0x44b. These reverse spans exclude relative+0x458. Under DF=0 the long copies would extend beyond the known MAIN allocation size 0x478 if their receiver were the unadjusted MAIN base. That size mismatch is a type constraint, not evidence of a runtime overrun and not a substitute for receiver identity.

## Original vtable and receiver bindings

Each binding below is checked from the original vtable prefix, its complete-object-locator pointer and exact RTTI name. Every prefix entry through the selected slot is executable; a nearby class name alone is not used to type a function.

| Vtable | RTTI class | Selected slot → entry |
|---|---|---|
| 0xd45adc | SUISystemEnvironmentChild | +0xd8→0x80d620; +0x18→0x80e470 |
| 0xd45bcc | SUISystemPlayChild | +0xd8→0x80da10; +0x18→0x810040 |
| 0xd6be78 | SUISystemWnd | +0xcc→0x80e110 |

The Environment input callback recognizes button_def/button_apply/button_ok/button_cancel. The selected cancel path passes its own receiver to 0x80b9b0; its default branch separately copies the global base block. The Play callback's cancel path calls 0x812b80. The SystemWnd message handler and tab helper name window_system_environment/window_system_play/window_system_keymapping and reload their selected stored pointers. These parent/child transitions distinguish the inspected settings paths from MAIN; they do not identify every possible alias, resource override or indirect caller.

The remaining 18 late-address seeds contain one 100-byte REP MOVSD, twelve 44-byte REP MOVSD operations, four single MOVSD group starts and one STOSD group start. Selected constructor prefixes separately bind 0xbca6e5 to **std::ctype<char>**,0xbd1775 to **std::codecvt<unsigned short,char,_Mbstatet>**, and 0xbd17ac to **std::codecvt<wchar_t,char,_Mbstatet>**. These three positive bindings do not classify all 18 receivers. The final three-store STOSD group writes zero, as independently traced from EAX.

## Native Evidence

| Entry | Analyzed body bytes | Role | Evidence ID |
|---|---:|---|---|
| 0x9f5c80 | 135 | Guarded global settings source getter | `ev_2418c8a2ffe32ba0623ead4afc89702c06d6b4574e21e2098a02903434180644` |
| 0x80b9b0 | 49 | Environment callback copy helper | `ev_b213a26444c8c0f727641436554b2a0d3f1dc2de5d7b71ddc114579a05c23cf8` |
| 0x80d620 | 902 | Environment combo callback and copy | `ev_b59177ae5666b02daaa6717f293034a727abe70936a056d22dafe2f177582f0c` |
| 0x812a90 | 45 | Base-block copy helper; incoming receiver unresolved | `ev_f34a5f70a696cff7dd56f7095f25d579146778cf009b3f7f18fc2484bea1edb8` |
| 0x812b00 | 49 | Environment-pointer reload helper | `ev_2fd9d4dd42a2e50222497508aeee5e9801b20c35eec01629a988edc86b98c840` |
| 0x812b80 | 235 | Play settings copy/minimap refresh | `ev_76891bc69f86ec058d20bb1d5bd429dad01b51ba02c35b7f13555463abb155d0` |
| 0x9f4dd0 | 801 | Global source initialization; two 0x154-byte blocks | `ev_2d4f7b40669b6498210ea4749d49bc5bab5cd08fe7068681de5c1121869b3a44` |
| 0x80e470 | 4654 | Typed Environment input callback | `ev_ceaa757d7ece5998387f44e8349d8e0e796edca487caa5dbca98744ce5472f51` |
| 0x80e110 | 526 | Typed SystemWnd message receiver; switches child pointers | `ev_b815755ee322a54c2bf9835ea8006c130112bed457e623e849d406607e1b854e` |
| 0x811b50 | 205 | System settings tab selection and child reloads | `ev_7ce93a50afce2d0c046680328595ff8baaa9e67f9c0a4cf029e4ea4c1e115757` |
| 0x80da10 | 98 | Typed Play combo callback | `ev_2b1dbc1a52eb8b4f1b2e659abbb3b9a32094bb24453c8ef4b54884a8f85d9b3f` |
| 0x810040 | 2937 | Typed Play input callback | `ev_a635f17311bad3d89aa7d7ada1a38ecc29ac8e9a324d8f8823b7baf7fc25c68b` |
| 0x80b230 | 706 | Settings-block initializer, through DWORD index 0x54 | `ev_76d2fc9aaf25613aec350d25ae72da6840e65c1092380fa9c06045b5e43c7c6e` |

| Containment query | Evidence ID |
|---|---|
| 0x0080b9ce | `ev_34306be9f3c591f74f41b1da18719e778eddd76d787b1bdab3f70dc41ca10dd0` |
| 0x0080d974 | `ev_94832443dbe08484a3f6d9f8bf9e4b5b14f9f8bd186ed0cac7591e3363ea918f` |
| 0x00812aa9 | `ev_8799c125e747922f3a8c9c3c3007bcd5d6b2af1fce1b94b3e0f7451c5ac55a41` |
| 0x00812b1e | `ev_46687d1abcefc09ca95adb93904b3ec9756547aaae44b2d0433035a11782424c` |
| 0x00812bb6 | `ev_f9883892c9a527cb347f0b87f92956c745c181ccf344d46b9ae2c82742190fd7` |

**Graph limits:** new 0x80e470 and 0x810040 value-flow graphs are truncated, each with omitted-operations lower bound 1. Known omitted edges are 0 and 208 respectively. Zero known omitted edges does not mean a truncated graph is complete. The other eleven new graphs are not marked truncated; indirect call targets and helper side effects still retain their own limits. Selected receiver/copy statements above are checked against original instructions rather than inferred from the truncated graph alone.

The independent verifier remaps all 13 inclusive function bodies to the SHA-bound PE, confirms **2,872 native assembly instruction boundaries**, and checks **542 distinct selected byte intervals** across both span ledgers and the settings RTTI bindings. All 41 ordinals match the source inventory; repeat counts, forward/reverse bounds and adjacent group extents are checked. These are selected static-byte checks, not a global proof of dynamic reachability or memory safety.

## Preservation and continuation

The closed canonical bundle retains the previous 176 Evidence records unchanged and adds 18 observations, for **194 records**. The snapshot contains **165 cached primitives**: 152 retained entries plus 13 new function analyses; containment metadata queries do not add primitive-cache entries. The owned analysis engine is closed after export. Raw artifacts and the snapshot remain ignored under `client-data/audit/2026-11-01-sframe-implicit-copies-74be2d90/`.

All **nine original-data preservation checks** pass. Original SFrame/archive, installed images, addon/settings hashes, backup and existing service listeners/processes remain unchanged. No original target, server restart, database migration, character reset or gameplay mutation is part of this pass. Only seven documentation files are published through the isolated Core/client worktrees; the original combined workspace index remains empty. No Rust build/test rerun is needed for this documentation-only investigation.

Continue by establishing the incoming receiver/reachability of 0x812a90 and pursuing the retained **102 candidate pointer deliveries** with actual MAIN provenance. For the 36 nonreaching spans, do not discard an untyped base solely by its small offset: first rule out an interior MAIN pointer or repeated adjusted calls. A justified activation finding still requires **MAIN identity, a reaching write, and the actual nonzero byte at+0x458**. Source values after settings initialization, arbitrary aliases and live resource precedence remain unobserved. No alternate-mode behavior is added to Core/RappelzRust from an unproved setter.
