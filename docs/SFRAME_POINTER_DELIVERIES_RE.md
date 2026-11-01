# SFrame pointer-delivery provenance and alternate-mode candidates

**Follow-up:** [Concrete MAIN pointer flow](SFRAME_MAIN_POINTER_FLOW_RE.md) now links startup construction, registry lookup and actual message delivery to the sheet. Its 59-entry table and both 69-entry child tables are accounted for; geometry/display effects and archived evidence are qualified separately. MAIN activation remains unresolved.

Verified **2026-11-01** on original x86 SFrame SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`, using **REA 6.4.0 / Ghidra 12.1.4** and **Capstone 5.0.7**. This continues [the helper-caller investigation](SFRAME_HELPER_CALLERS_RE.md) across **all 102 retained pointer-delivery sites**. **98 sites are bound to provider procedure bodies; four remain original-byte-only.** This pass adds **102 new native function dossiers / 28,405 analyzed body bytes**, 74 containment observations and one incoming-reference query. **MAIN SUICharacterInfoWnd byte+0x458 activation remains unresolved.** No nonzero write with a demonstrated MAIN receiver is recovered.

[Versions](VERSIONS.md) remain Core/RappelzRust **0.1.0 development**, NoLimits launcher **1.3.60**, original SFrame PE **9.1.0.0**, embedded build lineage **Source9.5.2**, network literal/profile **201507080**. The exact official Epic/patch remains unconfirmed. No original target executes in this static pass.

## What the 102 sites actually represent

The source is the previous corrected affine scan, SHA-256 `91770d6c02286c4eb02f9a10f682c0df70c59d58ec58d80288703fc3ed7e8147`. It retains registers containing candidate adjusted addresses, along with nearby pushed values. The 102 entries are **candidate sites, not 102 MAIN pointer escapes**. There are 43 unique direct targets, with 39 newly analyzed and four retained dossiers reused; four other callsites use indirect destinations. Caller analysis and focused follow-ups bring the new native dossier total to 102. This is not a one-to-one correspondence between sites and functions.

| Qualification | Observed scope | Limit |
|---|---|---|
| Provider-contained sites | 98 sites in 57 distinct owning procedures, with every selected CALL checked against native assembly | Ownership does not itself identify the live receiver. |
| Raw-only sites | Ordinals 60,61,62,67; two bounded fragments | The provider returns not_in_procedure. No invented Native entry or decompiler result is substituted. |
| Incidental incoming ECX | 13 receiver tags: 1,54,55,59,63,65,68,69,77,81,82,92,93 | Some sites still pass a real pointer on the stack. This is not 13 wholly irrelevant calls. |
| Stack-local OS iteration | 0,1,2 in 0x5673e0 | Biased EBP frame and pointer destinations are verified; this is not a window base. |
| Generic virtual output pointers | 99,100 | ECX is the object base; the separately pushed address is base+0x45c. Virtual destinations and complete side effects remain unresolved. |

The raw instruction stream takes precedence over a recovered calling-convention label. For example, 0x530190 has an inferred __stdcall API label, but plain RET plus caller stack cleanup; it ignores incoming ECX while forwarding stack inputs through a global callback. Conversely, 0x5f3250 also has an inferred __stdcall label but explicitly reads the string object in ECX and searches with memchr. Treating either label alone as an ABI proof would misclassify these deliveries.

## Stack, strings and control aggregates

The prolog of **0x5673e0** establishes EBP as a biased stack frame. Its EBP+0x400 input is local storage. The two indirect targets are obtained through GetProcAddress names **Thread32First** and **Thread32Next**, and the direct 0x530190 forwards the same local buffer. These are observations of static lookup/call code, not execution of OS enumeration or a game-window pointer delivery.

Exact vptr/RTTI or receiver-preserving caller paths bind selected contexts to **SUICreaturePageWnd, SUICreatureSkillFundWnd, SUIHuntaHolicLobbyWnd, SUIHuntaHolicScoreBoardWnd, SUIInGameLogoutWnd, SUIMainFrameWnd, SUIDestructionWnd, SUIDurabilityWnd and SUITitleWnd**. The MainFrame class name is distinct from SUICharacterInfoWnd. The two apparently close CharacterInfo-area calls at 0x8a4a89/ 0x8a4cf9 are in **SUITitleWnd** teardown paths, with exact vptr stores. Their+0x454 container resets copy a pointer into outer+0x458; neither MAIN identity nor a known nonzero mode byte follows.

Fresh 0x81a9e0 writes the exact vtable for **sui::kui_updatable_window_array** into its receiving subobject. The selected HuntaHolic outer+0x458 address therefore holds an aggregate's vptr in that constructor path, rather than demonstrating MAIN's mode flag. Other helpers perform named-control lookup, vector/list insertion, tree callback registration or string construction/search. Private helpers without incoming receiver provenance retain that limit even when equivalent code is visible in a typed window.

## Real writes at the same offset have different meanings

| Selected site/path | Observed write/effect | Provenance and boundary |
|---|---|---|
| Ord45 / 0x8e7108→0x820240 | FLOAT subobject+4, i.e. outer+0x458, is animated/clamped0..1 and sent to a control | Exact SUIRaidSiegeProgressSiegeWnd path. Nonzero FLOAT/DWORD does not automatically mean its low byte is nonzero. |
| Ord63 / 0x8fe540 | Two DWORD increments of outer+0x458 after list appends | List-count behavior; the helper's incoming receiver remains unbound. Equivalent appends occur in a typed BingoBoard callback. |
| Ord90 / 0x916c20→0x8faf70 | Reads state byte at outer+0x458 | Exact SUIArenaWaitingWnd virtual+0x0c; later countdown update is separately decompiled. |
| Ord91 / 0x918630→0x8fb010 | BYTE outer+0x458=1; DWORD outer+0x45c=0 | A genuine nonzero byte projection, but 0x918630 has no recovered incoming reference and its receiver remains unidentified. |
| Additional callback 0x917aa0→0x8fb010 | Same reset, reached through a typed callback registry | Concrete SUIArenaWaitingWnd countdown path below; it does not prove the receiver of0x918630. |

### The typed countdown activation path

Original vtable **0xd5c778** and its exact RTTI bind **SUIArenaWaitingWnd**. The fresh native dossiers and original instructions establish:

1. Constructor **0x916550** installs the class vptr and initializes the state fields. The recovered object size 0x478 matches MAIN's allocation size, which demonstrates why size/offset matching alone is insufficient.
2. Class virtual+0xd8/**0x916940** calls **0x917500** with the same receiver.
3. Registration passes key **40004** from 0xd5c764 and callback **0x917aa0** to **0x917220**, with ECX=window+0x470. The insertion path stores the function at node+0x14; an existing-key path can retain a prior callback, so registry contents are conditional.
4. Class virtual+0xcc/**0x916bc0** looks up the message key and calls node+0x14 while restoring ECX=window.
5. Callback **0x917aa0** preserves that receiver, stores window+0x454 in a local, reloads it and calls **0x8fb010** at 0x917ca4. The callee writes BYTE state+4=1 and DWORD state+8=0.

Fresh **0x8faf70**, **0x8fafa0** and **0x992560** explain this state as a countdown: float value at state+0, active byte+4, previous tick value+8; the update subtracts a supplied tick difference scaled by a divisor of 1000, clamps the float at zero and clears the active byte on expiry. The getter checks the active byte and a float epsilon test. The callback renders the float countdown as an integer in the ArenaWaiting UI. This is static countdown behavior, not a wire-layout or server disclosure rule. Registration/dispatch reachability is conditional; no message was observed executing.

The separate 0x918630 helper contains corresponding state-reset/render operations, but its unknown receiver is not typed by proximity or by the shared helper. It is retained as an unbound candidate. No whole-program absence, unreachable-code or universal alias claim is made.

## Four sites remain outside provider procedures

The exact containment queries for **0x8fe514,0x8fe525,0x8fe536 and 0x906e44** return **found:false /not_in_procedure**. Their bounded original-byte fragments are retained separately:

| Fragment | Original bytes | Observed effects and limit |
|---|---:|---|
| 0x8fe4f0..0x8fe53c | 77 | Four calls to 0x8fe240 with key/callback pairs matching the typed BingoBoard registrations. All three selected sites lie in this straight-line fragment; receiver identity remains unknown. |
| 0x906e30..0x906e4a | 27 | Named lookup of area_control and insertion into receiver+0x458 via 0x81c2d0, matching a typed ArenaMenu fragment. This equivalence does not establish the standalone receiver. |

Both fragments end in RET and have independently checked INT3 separators. These facts bound the retained instruction fragments; they do not create provider function identities or establish all possible entry paths. The interpreted effects are original-byte analysis, not provider-generated pseudocode.

## Complete 102-site index

The owning procedure below is a provider observation. RAW means no containing procedure was returned. Exact class/ABI qualifications and byte records remain in the ignored three-part ledger; this index does not type an unbound owner by neighboring code.

| Ord | Original CALL | Owning entry | Destination |
|---:|---|---|---|
| 0 | 0x567a7d | 0x5673e0 | dword ptr [ebp - 0x4c] |
| 1 | 0x567aae | 0x5673e0 | 0x530190 |
| 2 | 0x567ac1 | 0x5673e0 | dword ptr [ebp - 0x48] |
| 3 | 0x656cf7 | 0x656200 | 0x622db0 |
| 4 | 0x763160 | 0x7630e0 | 0x5bec70 |
| 5 | 0x7684a6 | 0x768450 | 0x7683d0 |
| 6 | 0x768579 | 0x768520 | 0x7683d0 |
| 7 | 0x7c5156 | 0x7c50f0 | 0x81a9e0 |
| 8 | 0x7c52ac | 0x7c5280 | 0x81ad70 |
| 9 | 0x7c53f3 | 0x7c53c0 | 0x81ad70 |
| 10 | 0x7c58ef | 0x7c5490 | 0x81c1e0 |
| 11 | 0x7c5928 | 0x7c5490 | 0x81c1e0 |
| 12 | 0x7c66ef | 0x7c66c0 | 0x81b1a0 |
| 13 | 0x7c670b | 0x7c66c0 | 0x81b1a0 |
| 14 | 0x7c6722 | 0x7c66c0 | 0x81b1a0 |
| 15 | 0x7c6739 | 0x7c66c0 | 0x81b1a0 |
| 16 | 0x7c79e9 | 0x7c79a0 | 0x81a9e0 |
| 17 | 0x7c7b5d | 0x7c7b10 | 0x81ad70 |
| 18 | 0x7c7ca4 | 0x7c7c50 | 0x81ad70 |
| 19 | 0x7c855a | 0x7c8510 | 0x81b1a0 |
| 20 | 0x7c8571 | 0x7c8510 | 0x81b1a0 |
| 21 | 0x7c8588 | 0x7c8510 | 0x81b1a0 |
| 22 | 0x7c8e34 | 0x7c8e20 | 0x81c770 |
| 23 | 0x7c9428 | 0x7c93e0 | 0x81b1a0 |
| 24 | 0x7c943b | 0x7c93e0 | 0x81b1a0 |
| 25 | 0x7c944e | 0x7c93e0 | 0x81b1a0 |
| 26 | 0x7c9928 | 0x7c9890 | 0x5f3250 |
| 27 | 0x7d1c0f | 0x7d1ba0 | 0x5bec70 |
| 28 | 0x859a69 | 0x859a10 | 0x8594f0 |
| 29 | 0x85c30c | 0x85c2c0 | 0x859390 |
| 30 | 0x8a4a89 | 0x8a4a20 | 0x8a4920 |
| 31 | 0x8a4cf9 | 0x8a4c90 | 0x8a4920 |
| 32 | 0x8bbae2 | 0x8bbaa0 | 0x5bec40 |
| 33 | 0x8e6bd1 | 0x8e6bb0 | 0x81c320 |
| 34 | 0x8e6be2 | 0x8e6bb0 | 0x81c320 |
| 35 | 0x8e6bf3 | 0x8e6bb0 | 0x81c320 |
| 36 | 0x8e6c04 | 0x8e6bb0 | 0x81c320 |
| 37 | 0x8e6fb1 | 0x8e6f50 | 0x81c320 |
| 38 | 0x8e6fbe | 0x8e6f50 | 0x81c320 |
| 39 | 0x8e6fcb | 0x8e6f50 | 0x81c320 |
| 40 | 0x8e6fd8 | 0x8e6f50 | 0x81c320 |
| 41 | 0x8e6fe5 | 0x8e6f50 | 0x81c320 |
| 42 | 0x8e6ff2 | 0x8e6f50 | 0x81c320 |
| 43 | 0x8e6fff | 0x8e6f50 | 0x81c320 |
| 44 | 0x8e700c | 0x8e6f50 | 0x81c320 |
| 45 | 0x8e7108 | 0x8e70d0 | 0x820240 |
| 46 | 0x8e717f | 0x8e7160 | 0x81c320 |
| 47 | 0x8e718c | 0x8e7160 | 0x81c320 |
| 48 | 0x8e7199 | 0x8e7160 | 0x81c320 |
| 49 | 0x8e71a6 | 0x8e7160 | 0x81c320 |
| 50 | 0x8e71b3 | 0x8e7160 | 0x81c320 |
| 51 | 0x8e71c0 | 0x8e7160 | 0x81c320 |
| 52 | 0x8e71cd | 0x8e7160 | 0x81c320 |
| 53 | 0x8e71da | 0x8e7160 | 0x81c320 |
| 54 | 0x8fd915 | 0x8fd8b0 | 0x749890 |
| 55 | 0x8fd938 | 0x8fd8b0 | 0x8fe0e0 |
| 56 | 0x8fdc35 | 0x8fdbf0 | 0x8fe240 |
| 57 | 0x8fdc4a | 0x8fdbf0 | 0x8fe240 |
| 58 | 0x8fdc5f | 0x8fdbf0 | 0x8fe240 |
| 59 | 0x8fdc8c | 0x8fdbf0 | 0x8fd410 |
| 60 | 0x8fe514 | RAW | 0x8fe240 |
| 61 | 0x8fe525 | RAW | 0x8fe240 |
| 62 | 0x8fe536 | RAW | 0x8fe240 |
| 63 | 0x8fe572 | 0x8fe540 | 0x8fd410 |
| 64 | 0x901bc7 | 0x901bc0 | 0x901bd0 |
| 65 | 0x905e17 | 0x905d80 | 0x749890 |
| 66 | 0x906232 | 0x906190 | 0x81c2d0 |
| 67 | 0x906e44 | RAW | 0x81c2d0 |
| 68 | 0x908454 | 0x9083e0 | 0x908c80 |
| 69 | 0x908477 | 0x9083e0 | 0x908c60 |
| 70 | 0x90859b | 0x908580 | 0x909170 |
| 71 | 0x9085a6 | 0x908580 | 0x909170 |
| 72 | 0x9085c1 | 0x908580 | 0x9084d0 |
| 73 | 0x9087de | 0x9087c0 | 0x909170 |
| 74 | 0x9087e9 | 0x9087c0 | 0x909170 |
| 75 | 0x908804 | 0x9087c0 | 0x9084d0 |
| 76 | 0x909417 | 0x9093c0 | 0x908fd0 |
| 77 | 0x90aa3d | 0x90a9f0 | 0x90b350 |
| 78 | 0x90ab81 | 0x90ab40 | 0x90aab0 |
| 79 | 0x90adb4 | 0x90ad70 | 0x90aab0 |
| 80 | 0x90ba36 | 0x90b9e0 | 0x90b6a0 |
| 81 | 0x913864 | 0x9137f0 | 0x914020 |
| 82 | 0x913887 | 0x9137f0 | 0x914000 |
| 83 | 0x9139ab | 0x913990 | 0x914830 |
| 84 | 0x9139b6 | 0x913990 | 0x914830 |
| 85 | 0x9139d1 | 0x913990 | 0x9138e0 |
| 86 | 0x913bee | 0x913bd0 | 0x914830 |
| 87 | 0x913bf9 | 0x913bd0 | 0x914830 |
| 88 | 0x913c14 | 0x913bd0 | 0x9138e0 |
| 89 | 0x914bc6 | 0x914b70 | 0x914370 |
| 90 | 0x916c4d | 0x916c20 | 0x8faf70 |
| 91 | 0x918726 | 0x918630 | 0x8fb010 |
| 92 | 0x91b887 | 0x91b7f0 | 0x7dd470 |
| 93 | 0x91d25e | 0x91d200 | 0x7d33b0 |
| 94 | 0x9b126a | 0x9b1220 | 0xa8cfd0 |
| 95 | 0x9f501a | 0x9f4dd0 | 0xbe4695 |
| 96 | 0x9f53a1 | 0x9f5300 | 0xbe470e |
| 97 | 0x9f6b61 | 0x9f6b30 | 0x5bec70 |
| 98 | 0xb0e62a | 0xb0e280 | 0xb0abd0 |
| 99 | 0xb7270c | 0xb726f0 | dword ptr [edx + 0xec] |
| 100 | 0xb7276a | 0xb72730 | dword ptr [edx + 0xec] |
| 101 | 0xd06cc0 | 0xd06cae | 0xbe470e |

## New native Evidence

| Entry | Analyzed body bytes | Evidence ID |
|---|---:|---|
| 0x530190 | 111 | `ev_ac39658a61ae4e3d5750cbe0783ce839255a206190450049f785cfe2d29a88b3` |
| 0x5bec40 | 40 | `ev_95f3efeda8656751f614889ba961b51a6acafd4459cf4bc03e1e68bebcc40362` |
| 0x5bec70 | 290 | `ev_76bd09e77e3793886e56c5f7be2345c449e97dd07d2a28724de2d79799d97d5e` |
| 0x5f3250 | 262 | `ev_c0ae53a71209cdf5eaccd6a4223171fca5a2c65a5255a50e991fe2a2ab9e5057` |
| 0x622db0 | 39 | `ev_c0cd010c787ff6ac44d6c0e402b7b2aaee920009233368aa880e5c847ea798d8` |
| 0x7683d0 | 102 | `ev_6f55e3ebd63d0499e9c842d14cc9ee1bf857d9b0a1060269c5fc5e16420e2246` |
| 0x7d33b0 | 69 | `ev_39f595adc16318704854f252a105484702bf5473f00bec1ccae7dcd5762ec58b` |
| 0x81a9e0 | 40 | `ev_8f6d88fdd5bf81ac0f96d47f575ac09b8c6d9872484e24653f5b525582037b32` |
| 0x81ad70 | 161 | `ev_0ace970ec7f78827fee0ccb9ae89a97bd8a359c2e7fad033ce7e69526ae4b48d` |
| 0x81b1a0 | 83 | `ev_ac7215be48d5d4c600a3bbd905379545b7aa0f6466cf3eb0670e5ae3c0ae51d1` |
| 0x81c1e0 | 50 | `ev_3da19f075635433350eddef95d5ddbbf62a27c3c81961eb013f06b52ac982893` |
| 0x81c2d0 | 73 | `ev_663c5bf4c1af3e40cb73a15c565b7a08df9c21229c3d44a1a95f39655cf36656` |
| 0x81c320 | 67 | `ev_e3f077c30bfaa9999c8ca30f5aa00bae3bd5fefe406c35df403d58f9b5f3f213` |
| 0x81c770 | 84 | `ev_a0574c56611c52ed25e708c8f15bd8e1affc7b0b95cd5bda6ab7a5b6298b2ca7` |
| 0x820240 | 195 | `ev_d8b299690028339634eff7418fb3946d7cf71fe61a1a1e8461308ab6b9c08d8b` |
| 0x859390 | 348 | `ev_3011a383820435be59701ed12dd25c461ed26149c221158d5a8a23a016450804` |
| 0x8594f0 | 348 | `ev_43f191500f92cf704a2688f47d4ef2cd9951f8dfe1012c34a9c0a19aa1072350` |
| 0x8a4920 | 112 | `ev_0c6e6a8b523b5d149af4feb04889d7941ed6f4e0393675c5780fd2b01bf28990` |
| 0x8faf70 | 34 | `ev_2438ea2ed528ccdf2c4e457359892f32d22dab8e4567ce96ac284282fe718e33` |
| 0x8fb010 | 12 | `ev_6311d6edbba44308e24bffdc741ec9c6fbccbb16ca26c14b03969b080907bce5` |
| 0x8fd410 | 26 | `ev_44bbc5d275279961e6941444608a9dde2c65b7d74e59c17e999ab9e2489befeb` |
| 0x8fe0e0 | 25 | `ev_d96f22cdc4bd073e8569893b7f1715281f93c63325f4548858794785cc330587` |
| 0x8fe240 | 172 | `ev_25e011e7d0a984852608dadd824c32c93a4a07c7b65c4327a986b3c2f4a0d1c1` |
| 0x9084d0 | 134 | `ev_0dcb56e8f073cc413bfa091f60ed0474a34dd093c3657dff0c4278ad914c8d51` |
| 0x908c60 | 25 | `ev_6647ad51123bd6e6b410c458eeba89a2135d9329df1d026ea8acf3fcef8cf6c3` |
| 0x908c80 | 25 | `ev_f68b453b436d7a3f394d701aae9469e34015f08b5e877baeb928e28eef9742f8` |
| 0x908fd0 | 138 | `ev_f9df3b26ee66c2b5835462e31027b159b8301a4278418c8019d50d0ed5ab17cb` |
| 0x909170 | 120 | `ev_dfb23ef749a15be04e31a5b38d5299b376e97e8370ee75ba0566b8cfee220c95` |
| 0x90aab0 | 134 | `ev_68579fb68a5262ac9c28924e6fa027b47655e68290abd6130d1c9a324c22417d` |
| 0x90b6a0 | 138 | `ev_6f6de2c8ac82d7e20f590c1b71ec9a5d5e147a192733a01656b86c9237c301d6` |
| 0x9138e0 | 134 | `ev_6b9449a1037d3f7303f351bccc618e1c68d637e96558da3292a6678cd825f0c4` |
| 0x914000 | 25 | `ev_c3e2cea15de70458c0e79f9b34a263b22c39e9c1c75478bbf6ab896b99af4c1d` |
| 0x914020 | 25 | `ev_18dfbf3daab992ad8f11820c7dba59304e1e231376bff753d622f8e5e10e55a9` |
| 0x914370 | 138 | `ev_65e91261aab8df31943bb4f87ee51715aca2b92a381f8e6ce30e2e65c9abdfd7` |
| 0x914830 | 120 | `ev_25dde5dd5757b625f3a7bbad926302a0181bd5898267e3d566ca46398d812450` |
| 0xa8cfd0 | 361 | `ev_3b64e6e3f9bf7cf46f971bc223ef48c6d68de12071c462fa9d6cefd7d1dbf49d` |
| 0xb0abd0 | 345 | `ev_65d7487fb5ccaa29a059b40040353a93f838f8c54c27a3b509f382817b47d7d6` |
| 0xbe4695 | 90 | `ev_ad20336ddd0011b5481ab271fa343feeadc97eadc4eb6bc9afbcc4020fbbc753` |
| 0xbe470e | 105 | `ev_1629dac001f6ea84a9df33670cd87cf11046876cd522496a7e3e68ed9c826641` |
| 0x918630 | 301 | `ev_2e11babc974a7b4241ded6116bc72be4cbafa3d434f034fc40c37e79a04da7bc` |
| 0x916550 | 185 | `ev_596c3d917c90a0709cbb04e922e15feda7569b25cb26d777229825c4cde8e735` |
| 0x916940 | 203 | `ev_85b88ee032d7fbd1fc641566e16248d2e186783fccea7b76df953b9f7a705de8` |
| 0x917500 | 471 | `ev_30bc9b7b18b248bd3855f1f1c933d9f89ac652d78cc43105cecedda6548e1ee1` |
| 0x916bc0 | 82 | `ev_24d7024d1352d49ab5b8609e5cd1bca0e7242999e5cea6b8bb7d76ff6e2020f2` |
| 0x917220 | 172 | `ev_4a91334aea05853adaaa19f600f34a1424ea5dda05858e07416d0782959d6cae` |
| 0x917aa0 | 690 | `ev_5c5e2e76bb180562efa78127363e7cf4c687dc112ad925f6c515dde17249fb27` |
| 0x5673e0 | 2021 | `ev_8976ff267b904086b09f8c53e482c9a6e93eaf978e459a8d2d89463581da66cc` |
| 0x656200 | 3034 | `ev_0253038b39e9fba96114b4b738d80ba3cf9bd719cea252453ead601da38e8d5d` |
| 0x7630e0 | 152 | `ev_f21334bcab4a08bd978e0a13736e134672d5e0ad2e1705d41984c04486bccd8b` |
| 0x768450 | 110 | `ev_d848c886a7edffedc09e4373eb386627622dcf5b8b25378c12c6cb13a8643f38` |
| 0x768520 | 139 | `ev_aa16a7c0f9a29461efc0e0e90481bb5391edecd19e7a61530d29b94a67f9b20e` |
| 0x7c50f0 | 193 | `ev_ff627709c7839f4f6a05a2ea7e26c19a97c1a51e6bf72a0bba7b58156b234388` |
| 0x7d1ba0 | 142 | `ev_4d160bb393b3edcce3c657f75871d00c797329008370af42ef5e8c86d79eec4f` |
| 0x859a10 | 439 | `ev_8464c36e171c4619d51192248d44268d3bd591081d2ffa1e0639c957eb5f7889` |
| 0x85c2c0 | 719 | `ev_451b3490960dc81f21bf8d59373e39fbbc919b4f2be209380f039a79bd2f61f8` |
| 0x8a4a20 | 216 | `ev_b2a441e5603a9a07095db9956645a5addf2da233c33c5509cbbabf6ed31a4c0a` |
| 0x8a4c90 | 240 | `ev_87b343880a02aa2c1233532bf3ec4a70df0f49cecc0e03182f4b9b5983f49e04` |
| 0x8e6bb0 | 92 | `ev_8298d95564abbfc0d8d8482a89151960dd1e08c4bbfa119665f2f416095ee711` |
| 0x8e6f50 | 363 | `ev_f653874372b9c287f261c48d2b4d9512c202fc21e07673ecb8fa8f4db9a1835c` |
| 0x8e70d0 | 67 | `ev_7cf55f222bbc6306bfa773996a4fdeb1ef2d441d99749f1e34a7197e6728bebb` |
| 0x8e7160 | 130 | `ev_515cefe70f2eec2c83a41e774ecc0d8dfe2b285602af8f2c446b5bd9993f48e3` |
| 0x8fd8b0 | 163 | `ev_3f40f9ec6caa1cde8c092934ad276efe6556c9bb8d0bba3d7a73f52e5beb1903` |
| 0x8fdbf0 | 634 | `ev_90211c34c3c3c7a0b990af98599aa346f72513200c77f9d85ac0a285ad893c2b` |
| 0x905d80 | 178 | `ev_45c66b6503dfc108946532cb9727a464ee9c948493d86a93b6f508dd828ec232` |
| 0x906190 | 189 | `ev_ee6ba6706ceb9dbeac429848520fec36c4eb78fe140641377780f2b3b74c7452` |
| 0x9083e0 | 227 | `ev_6d73fc7e9764fd1472de1a80094bf65d6cedc1af3c39619363ba65f7c09d5d83` |
| 0x9093c0 | 924 | `ev_d2d0af221ff39b8c6887f6941783c4ecb73fb5f7d43223f4e1fcef39b6612e16` |
| 0x90a9f0 | 178 | `ev_0381c8ef9903775d9017e452981c3cc6670dc9b6a3b9f1c1f0b353cfec1c9c35` |
| 0x90b9e0 | 421 | `ev_37270d8c702aaf0c6463019f63fd25035fe66566c6af16609aa9621c181c1abb` |
| 0x9137f0 | 227 | `ev_6097c17cebe12fae0b6664bc7175b95cb43018b0f189de2e398a563c7a21e27a` |
| 0x914b70 | 1411 | `ev_038fa460d56a2688704fe7069bc328d030fc15cca57e39b88e8c59f80950b510` |
| 0x916c20 | 398 | `ev_f798fb6a88410dc78b49df4cd35c5c220e4e99768fdc15de5da1adfaa5039e62` |
| 0x91b7f0 | 343 | `ev_0665be855913f559ae733d854da05cb6873e4ecd16ab372d07f88f69f00284b1` |
| 0x91d200 | 248 | `ev_f06ff7300656e82b10e02abfcbaaf756f2509c4382b5147dabeb6052808774c7` |
| 0x9b1220 | 196 | `ev_349038411361640e1d549e90439f4f730d48ece9f4fa41836793fac92b280a24` |
| 0x9f5300 | 197 | `ev_6172083af895fa11876e3e32592b34e1af45386e1b58b2974eb4bb06919bf895` |
| 0x9f6b30 | 339 | `ev_915f187233c195fca24a7e2584f5eca8b6da8c23d24f7601de8b16d81ff1a66a` |
| 0xb0e280 | 1607 | `ev_62e72a836f3dbab46a515e77a951671d44d9f82d0e5f6a5a0fdd84a6a0de664e` |
| 0xb726f0 | 57 | `ev_7007fef636c08444ce1726847046301cad3652f019bf68e68d657782a8108f35` |
| 0xb72730 | 104 | `ev_dce60afb5cedde9eea59593b9bf8ea64306155969b5b9c9de273fb2e39a0e059` |
| 0xd06cae | 24 | `ev_ddd6b78c8a054965b98b32937d146fcb1340ba7ae2ae4e8537932d90bec44ede` |
| 0x7c5280 | 163 | `ev_04d6e806ec7eb720fe57ec449959c6edef873b53a8fa441ab61892884a30ebfe` |
| 0x7c53c0 | 194 | `ev_7077e316cbf400c1aab013871170d8062b196861ad1bc9e2969994611f4b6de9` |
| 0x7c5490 | 1237 | `ev_dc3a9c072e44742000f5465dbbfb46a481238e9d3515f7bffa248736aa38ffbf` |
| 0x7c66c0 | 149 | `ev_a145aae72af2b88eaa79dc5bc9e2f2bdc1a27bc1112a8133a88e8e8c50652924` |
| 0x7c79a0 | 172 | `ev_6f08c9fa9b8e53402e2fd47d2e575eb944a8457d986c738f85463dbc590be283` |
| 0x7c7b10 | 115 | `ev_f9fdfb13d981d990dab1f6e6c650659cc4a2c6931d9e98f0a4bccbcd9097791a` |
| 0x7c7c50 | 146 | `ev_020294eb115aed9eb93a3b29a83337cf8bf230be867b3c190b301b08870a7431` |
| 0x7c8510 | 148 | `ev_f643d4ec0df930ef17387229476e33b5d73a85d420f98ea27cb33a135d504020` |
| 0x7c8e20 | 122 | `ev_6ccbc255baf2595c0d77b38a9a8f79025e9c5732abf26d94132428b902f78188` |
| 0x7c93e0 | 127 | `ev_86bf4c7aef531acf7543d08dc2833dd6117ea645fa4cf52b3b1977fa58171254` |
| 0x7c9890 | 911 | `ev_9e19db34a2e1657818df8ad3fcc356a40ac258fe7c78629cd830f9c2abdffc73` |
| 0x8fe540 | 166 | `ev_b0885d5f8f85ac23192416740ac202bba25e824d1274d4f9a4f6f51160923821` |
| 0x908580 | 90 | `ev_ee457e861fb67aa5c551160e412642c0d3d7c8b253ca286c63fe53e75ca0e9c0` |
| 0x9087c0 | 119 | `ev_c374f162a31e6899ab290f04d5ab40788b49d6db4e3a6c173d1d455e0afe99dc` |
| 0x90ab40 | 79 | `ev_ba1dc279f03a338127a73ae4f4fc10f253eb33349a55a294285c2348dc0727ed` |
| 0x90ad70 | 108 | `ev_99df5e72a873297203e228c86620807d0e62ec0265eb710fd3dd29694e2ca6c6` |
| 0x913990 | 90 | `ev_bdf381f05161beb12e3bae884233c6370f0d7a296c66ed8bca42c372b542406f` |
| 0x913bd0 | 119 | `ev_54b07d4d53b61940c07f4d12643f17369ed35c2e169280dba3b2f673fd7adf8a` |
| 0x8fafa0 | 97 | `ev_da809073bd3cb75b6fe68756f7a1fe446bf0150b6c6bf804bd8db6c5b34c4433` |
| 0x992560 | 38 | `ev_3c1a474192eb87115f652b05bc150d0a4a9447bd06d3c582577c080a338c88b0` |
| 0x7c8670 | 989 | `ev_b925b597f8acc8d8e19f71ca96808334610782783b471f215fa9ccf63413c9c3` |

| Focused observation | Evidence ID |
|---|---|
| resolve_containing_procedure 0x567a7d | `ev_103f7a407d93c1defbac08344cd54df160028e365359c36c26e8264b86f575e8` |
| resolve_containing_procedure 0x8a4a89 | `ev_cb9bfe9c95fedacebdbd3a391df4ceb251c498046e87224e96f29c5e7563400b` |
| resolve_containing_procedure 0x8a4cf9 | `ev_c8a58022cbc7f968931d426ff599951821e02e16659518bf68c2b131dd1f05b1` |
| resolve_containing_procedure 0xb7270c | `ev_275941bdb9a8e240f13daf123d9e88d6f387edbbbf0e52d590bf0a71b440aac9` |
| resolve_containing_procedure 0xb7276a | `ev_3ca3510693d64c780a8c4f21a7d05fe8b992e1b47717c48e891e1df1e1d73097` |
| resolve_containing_procedure 0x7684a6 | `ev_69b5ed4181284be2898ce62f06e6ce5b15231ec0c92fddce65211489336986bd` |
| resolve_containing_procedure 0x768579 | `ev_73c4f774d2e5d0d8597ae78c8f1fd7225a4446825fd0046add994af2f9d10474` |
| resolve_containing_procedure 0x859a69 | `ev_776669ce96611d857ee32d739f147417a387eda00356cead5fb234eb08fdb4a2` |
| resolve_containing_procedure 0x85c30c | `ev_b898ae40cb52a14d170ed7267acec8b7b1ff5fd60d51f271655a3de075459797` |
| resolve_containing_procedure 0x7c5156 | `ev_17d04c3dd20d47844099e9981b9966f6880b345d5f2a08f6cbab716c82377987` |
| resolve_containing_procedure 0x656cf7 | `ev_735f0bd68f71a2919d491efe7c19e21e47d35d871f419ff0a713c64aa13ddfdb` |
| resolve_containing_procedure 0x763160 | `ev_1909e81cccaa8c9e194553997e581ed2676f0ea15a7589f2e7d70edb7c7121b8` |
| resolve_containing_procedure 0x7d1c0f | `ev_23ce0250648cd536d4aed615886b623662dc48c2fb15bd94acee8594aa9d1e98` |
| resolve_containing_procedure 0x8e6bd1 | `ev_78d7fd303b3cda0931c36025c66114ad2e47d3ee99750c0b1bdc4fbb71d62de3` |
| resolve_containing_procedure 0x8e6fb1 | `ev_018a741ea6abfca7ab881742ca9173e817975f1433722690f77409bce1bb38b6` |
| resolve_containing_procedure 0x8e7108 | `ev_cfcd11c40b2de5c91a2daad58fb91b7286e92067bd1fb81ef5f6c3569a49027e` |
| resolve_containing_procedure 0x8e717f | `ev_c2f1c3a5c9020c492f169284a5373c2ee1846fded8cd236671bcc204e21d31f7` |
| resolve_containing_procedure 0x8fd915 | `ev_d73513f8f2d50999ce51ce82e4848ef532772e4f07df4aad8879d45c04816eb1` |
| resolve_containing_procedure 0x8fdc35 | `ev_d42760a60d88b8c533d78c2bcbabaa14971e5e47fbcdfcce6940ce1baeebbc4a` |
| resolve_containing_procedure 0x8fe514 | `ev_3186a75f63e1c5798115c4894cca5d30a81c19ddbb43518fe20b086b7192da1b` |
| resolve_containing_procedure 0x905e17 | `ev_749cb421e3735af7dfd87eaea0c569042222d9954b1ddb8ce7dc032320fb050a` |
| resolve_containing_procedure 0x906232 | `ev_bfb0cd792a2a1518585f87bc312a794f76d1bb723d5545ac2ab473e011d84754` |
| resolve_containing_procedure 0x906e44 | `ev_698abc8433837bd2ae285468aa7269a999a9bacb5d36d0492b4cb8a3358bf306` |
| resolve_containing_procedure 0x908454 | `ev_b736fa47cac6949fe93a7595245ea5eb5cc94837356055079265ccde7c1bbede` |
| resolve_containing_procedure 0x909417 | `ev_1fba28bb7d878c0228bb03627ab756e10da704c15b98a0f16f3d7065482f3479` |
| resolve_containing_procedure 0x90aa3d | `ev_9b716de49b089ed29a350d00b245b25fb37e4e7ee69eca879fdfd6ae0a320a7e` |
| resolve_containing_procedure 0x90ba36 | `ev_8bb7e1aeb10e666da8583c78b8fc67cfea364ef1e1f299539315a3681172ef4a` |
| resolve_containing_procedure 0x913864 | `ev_f23db40542f2c65b6352da606d2eab829e84bd51658e72504e623b9c9c6d3aaa` |
| resolve_containing_procedure 0x914bc6 | `ev_ad8f7e84fa323331207bf08efc7d1b3a849289f5801574b492b588b77687f6fc` |
| resolve_containing_procedure 0x916c4d | `ev_c6b55a9d750e42ee5d13fb13a1f65a134df4b212adf7d594005d129e01255586` |
| resolve_containing_procedure 0x918726 | `ev_9b519e9d2ae8c39dec325bb36d7303ab4c18345fa4273d09a90a128073471793` |
| resolve_containing_procedure 0x91b887 | `ev_0f21b011481f67fd5df4816126245f981e35c02746a4a6caf86afa184eb1d3c9` |
| resolve_containing_procedure 0x91d25e | `ev_ffde937d8fa17de18fb53caf9839b1b8f770d63ee88f26b2dc3d9b89ed3ef254` |
| resolve_containing_procedure 0x9b126a | `ev_2ee0306ffac1316a025485227253c51a573d08e0fb540c0f1322ec3ff50a0bb9` |
| resolve_containing_procedure 0x9f53a1 | `ev_5b763348afb7e3ed5c94e3c45c719037e123754885c1819d755a6c4c959778a2` |
| resolve_containing_procedure 0x9f6b61 | `ev_6e8cf09d4f35bfb4fd88e93a2a101a4ab7fe06503209bdf9978205427cfc7540` |
| resolve_containing_procedure 0xb0e62a | `ev_070ef21ad764aeaa4e1faa2dc6d03b1a7491e188d61ffa9b2ae85cf3eca4a5b5` |
| resolve_containing_procedure 0xd06cc0 | `ev_ca32d98fb59236ac79cef9ecd1816a38dbaea903154436ea85aba41d6d03f3de` |
| procedure_references 0x918630 | `ev_01b55707ea22285c9103b4c429d9a29add3b1840d29c21e3181b50964ab4a75e` |
| resolve_containing_procedure 0x7c52ac | `ev_35b4d2f06a9c4a56ce245ba1c634662cac7cc403beea2d1dc37135ae2f218d3a` |
| resolve_containing_procedure 0x7c53f3 | `ev_bb0aa9e100dc6986a5b6fff3c48066d8793e9643538852cdc38171f936e0f863` |
| resolve_containing_procedure 0x7c58ef | `ev_0f5fe3db183eb6590d3b3bf966d55d2d3b8eec91f60430b8129e1da263e1ec65` |
| resolve_containing_procedure 0x7c5928 | `ev_09f870c677af91b56a615cd29784ddfa6794b0329f044c462037d52738948ad1` |
| resolve_containing_procedure 0x7c66ef | `ev_adc50cc514a3e9770f5037fb09660891d0b371b1facd0e7bb101786cd382e5bb` |
| resolve_containing_procedure 0x7c670b | `ev_4e3573423c2593487f8169b07ea1cc8b4ecc5772cef5f4adfb2b0a340a32e525` |
| resolve_containing_procedure 0x7c6722 | `ev_e248c174ee9cbe3d94117da60bf23bc51bd5040ea903cac7648de2d6ffa83135` |
| resolve_containing_procedure 0x7c6739 | `ev_fc29383f922fc1a722955613fc33814cec9930ee90ef0815275f8fdcf0bb09e1` |
| resolve_containing_procedure 0x7c79e9 | `ev_c5065c07401bcb0adcb9c2040cc425c0f55a65b4ad659b11916679af9de8eb2f` |
| resolve_containing_procedure 0x7c7b5d | `ev_9db9a49087c77d05b0ef0a1336ffd36e86b9c7c6a2edecd51a17d210ae8b0789` |
| resolve_containing_procedure 0x7c7ca4 | `ev_c8d5f6dd41d68f489e49509eb5bbbbbe6f3bc1a60c207c3968baf203cff43b9d` |
| resolve_containing_procedure 0x7c855a | `ev_cc6fe310c4f2ec8ba93106891684d4a6209f43f345e1baadbf4bfb04445c7b0c` |
| resolve_containing_procedure 0x7c8571 | `ev_05fd1d9d163a86ba9a9568c631d9748debd83796fd9d3da9c2499c1ce6fe2d0b` |
| resolve_containing_procedure 0x7c8588 | `ev_6b8b4b455888a71dd48eb82d55790925ccd82d4109cff93dffdcbb84cddc5bf5` |
| resolve_containing_procedure 0x7c8e34 | `ev_2a859f2a63237d78e91d900d8f51a5f3384b2f3d1b2ed6196213c85a24cbaf3d` |
| resolve_containing_procedure 0x7c9428 | `ev_2e883cf86fdf3372dca78ee304df463eb1cacf142f7e33e77efbf5f9b60eb585` |
| resolve_containing_procedure 0x7c943b | `ev_bb66eb44250d3dd9aa4572a9d705d5f5c9a5aeb99c696c20a97135240ccfcf58` |
| resolve_containing_procedure 0x7c944e | `ev_8deed7720a1a1d8dbfac187792a58062fc3c9e217616356e94a3f9900267fd4c` |
| resolve_containing_procedure 0x7c9928 | `ev_3e5f1b9692591cb7a839c085ffebfcd22c6b9da320581fbec60412b3344bef31` |
| resolve_containing_procedure 0x8fe525 | `ev_2e1996a9daf2e4d4d899f6de3a5fc5934e8024596f02f2c14e555189f702ecf3` |
| resolve_containing_procedure 0x8fe536 | `ev_8960815974dbbdcd2fa0dc92dfb03b6ccf0232d5ce7bce2c068417147d458f4c` |
| resolve_containing_procedure 0x8fe572 | `ev_3de6e4e21955010c29b88772c022a0fed66405438702852fb497f273961249e6` |
| resolve_containing_procedure 0x90859b | `ev_8f2e0ae080d7153b3bfdfd516ab233aa6a5af3bffc9f281873ea6b76ba8d910f` |
| resolve_containing_procedure 0x9085a6 | `ev_b50eec5cddbd31e799dbe29284d86678a3591f5c26ba83f50992fbb58328365c` |
| resolve_containing_procedure 0x9085c1 | `ev_4e071526e504f69b675f770418a71cbf0bd7bd0688ee43adcae1938cf8d4390f` |
| resolve_containing_procedure 0x9087de | `ev_5439cf3274af3bed3f4cf870482f35e3fefa199cfbb8ea73164cf4a627909be8` |
| resolve_containing_procedure 0x9087e9 | `ev_abb27f9143abc32d168e50cbe452ced279b1f31d1a5196dfd0aebd82f89d9e3e` |
| resolve_containing_procedure 0x908804 | `ev_9f00ef783b8058a4b55e3b0908610746b1caf3bb258d460618830bf0cf900da9` |
| resolve_containing_procedure 0x90ab81 | `ev_e9aad0cbd151804efb229223e923cf95e503dae383d4a814cddef9f8d22f5dc3` |
| resolve_containing_procedure 0x90adb4 | `ev_340f616afc07e775d6b537d93d5eb7d658e589136d4713cca80d572a5fdbd316` |
| resolve_containing_procedure 0x9139ab | `ev_7da9cf286d0a056746a54ae605bd01d36b42e89db1454e0a8506f51ad58e179c` |
| resolve_containing_procedure 0x9139b6 | `ev_62b770de982bc009527fbc78799a5de83c8b55e6d23198126e8627fe539873af` |
| resolve_containing_procedure 0x9139d1 | `ev_4523bf1a9531b9082f32981becfb058ca696803171a46da89d5679a6bb789e91` |
| resolve_containing_procedure 0x913bee | `ev_fa0e43cd827ad15ebf00e1e34f2bbdc42784e25f85b91eec2302faf94bddde44` |
| resolve_containing_procedure 0x913bf9 | `ev_e5c182df49999fb0fd62228c18d3e70ea05f3c5287b9da123eae4036fcf94ec4` |
| resolve_containing_procedure 0x913c14 | `ev_221bf4b2d14d52fdd24c589cba02cd6798436babc27f670f76438e04187b64f1` |

**Graph limits:** 1 of the 102 new native value-flow graphs are truncated. The remaining 101 are not marked truncated; indirect targets and external effects retain separate limits. Known omitted edges/lower bounds for every truncated dossier are recorded below. Zero known omitted edges does not mean a truncated graph is complete. Selected source/callee assertions use original instructions in addition to pseudocode.

| Entry | Known omitted edges | Omitted operations lower bound |
|---|---:|---:|
| 0x5673e0 | 1 | 1 |

## Verification, preservation and continuation

Independent verification checks all 102 inventory ordinals and their source digest, all new inclusive native body ranges, **8,533 assembly instruction boundaries** and **2,139 selected original byte intervals**. The 98 provider-contained selected calls are present at exact native assembly boundaries; the four negative containment answers remain negative. Exact selected RTTI prefixes/COLs are separately verified. ABI tags, source expressions and actual arguments retain their limits. This is not arbitrary alias analysis or a proof that every callee side effect is known.

The previous 201 Evidence records are preserved unchanged. The closed canonical bundle contains **378 records**; the closed snapshot contains **271 cached primitives**. The 102 new native functions add 102 primitive entries; 74 containment observations and one reference query add Evidence without adding primitive-cache entries. Raw outputs/snapshot remain ignored under `client-data/audit/2026-11-01-sframe-pointer-deliveries-5c2148d7/`. The one owned native engine is closed after export.

All **nine original-data preservation checks** pass. Original SFrame/archive, installed images, addon/settings, backup and existing service listeners/processes remain unchanged. No original-client execution, server restart, database migration, character reset or gameplay mutation is part of this pass. Seven documentation files are published through the isolated Core/client worktrees; the original combined root index remains empty. Client/game version distinctions remain explicit. This documentation-only investigation does not require unrelated Rust build/test reruns.

**Still unresolved:** MAIN's nonzero activation writer, the receivers/entry paths of several private helpers (including 0x918630), the four raw-only fragments and complete virtual-call effects at sites 99/100. Do not globally exclude them merely by a matching typed fragment. The next useful search should start from proven MAIN object/owner pointers and unadjusted argument/callback paths; the 102 affine candidates did not identify such a writer. Preserve the constructed child/MAIN links and require **MAIN identity, a reaching write and the actual nonzero byte at+0x458** before implementing alternate-mode activation.
