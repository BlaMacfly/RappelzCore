# SFrame CharacterInfo callbacks and restart recovery

Verified **2026-11-01** against original x86 SFrame SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`, using **REA 6.4.0 / Ghidra 12.1.4**. This continues [the loading-widget investigation](SFRAME_WIDGET_DATA_RE.md) with **15 new function dossiers / 1,516 analyzed body bytes**. The concrete callbacks recover name selection/refresh, cached dimensions, base event forwarding, a distinct list recipient and child owner-pointer assignment. Main alternate-mode activation remains unresolved. All observations are static; the original SFrame executable was not launched.

[Versions](VERSIONS.md) remain RappelzRust/Core **0.1.0 development**, NoLimits launcher **1.3.60**, original SFrame PE **9.1.0.0**, embedded build lineage **Source9.5.2**, network profile/literal **201507080**. The exact official Epic and patch remain **unconfirmed**.

## Recovery boundary

The PC restarted during the first attempted continuation. That attempt had imported the previous immutable snapshot but had saved **zero new native function dossiers**. Its files remain under the ignored `client-data/audit/2026-11-01-sframe-main-data-6872ab31/`; they are not counted as completed decompilation. The previous snapshot and its 100 Evidence records remain intact.

The new run uses a separate ignored directory, `client-data/audit/2026-11-01-sframe-main-resume-b47de812/`. One owned Ghidra process uses two active processors and an explicit **1536 MiB Java heap limit**. This is a resource bound, not a diagnosis or guarantee against another PC restart. The cause of the restart is not established.

Existing primary database/auth/world services were restarted from their existing executables and configurations. Their new process identities are recorded separately from pre-reboot identities. Nine original-record/settings preservation checks pass after recovery; no restore, schema migration or test-character preparation was performed.

## Main and child table boundaries

Original PE bytes bind main vtable **0xd53af8** through its complete-object locator and type descriptor to **SUICharacterInfoWnd**. Its measured prefix contains **59 consecutive executable entries**, ending before offset **0xec**. Relative to the corresponding SUIWnd prefix, the differing entries are:

| Main slot | Entry | Role in this investigation |
|---|---|---|
| +0 | 0x8a1a80 | Existing main override |
| +0x18 | 0x8a23d0 | Previously retained UI event callback |
| +0xcc | 0x8a1dd0 | Previously retained internal-message callback |
| +0xd0 | 0x8a1ab0 | Previously retained resource/child initialization |
| +0xd4 | 0x8a1c20 | Fresh lifecycle callback |
| +0xd8 | 0x8a1c60 | Fresh lifecycle callback |
| +0xdc | 0x8a1d40 | Previously retained refresh entry |

The separate **SUICharacterInfoSubBasicWnd** table **0xd53448** and **SUICharacterInfoSubAddedWnd** table **0xd53be8** both bind **+0x18 to 0x8a1920**. Reading **vtable 0xd53af8+0x108** crosses its measured main-table prefix into adjacent table data; it cannot establish a main-window virtual slot. These table boundaries are part of the byte ledger.

The reused main initializer **0x8a1ab0**, Evidence `ev_6d832c9d0196d2b1898603d93fd01bcd02fc3ba1df5750e51a235611f561c24c`, creates the Basic/Added children with IDs **0x2f/0x30**, stores their pointers at main+0x44c/+0x450, and calls 0xb3d9e0 with the main pointer on each child. Its callback edges are assessed separately from the resources' lexical contents.

## Original resource cross-check

Each selected name has exactly one entry in the original data.000 index. The original SFrame key at raw offset **0x9f97a8**, encrypted index row, archive range, phase0 resource transform, decoded bytes and existing cache are checked independently. The selected resources total **53,036 bytes**:

| Resource suffix after `window_player_character_info_` | Original range | Bytes | Decoded SHA-256 |
|---|---|---:|---|
| `main.nui` | data.004 @565346988 | 8,871 | `b3d48aa242acd4dd878bc2e145e5ce59608aa24ac5ec7a620b17556f7b5c5c64` |
| `sub_basic.nui` | data.007 @597972807 | 28,884 | `520b988ef8a2ed846accc65c431fcf92e8e51d5d7eafd86e396b88f272f46a3d` |
| `sub_added.nui` | data.002 @550705144 | 15,281 | `57c3bbf7807bf83c42d435e552a5c41466e503b95203212e00589ae681bb9a30` |

The main resource has `button_close` and `button_equip`; each child resource has `button_fold` and `button_unfold`. A bounded whole-text search finds no standalone ASCII `mode`, `target`, `script`, `command`, `event` or `visible` in these three decoded payloads. This lexical result neither establishes complete parser semantics nor excludes compiled, aliased or runtime activation. Loose-resource precedence and current live instances remain unobserved.

## Internal messages and concrete recipients

The reused main handler **0x8a1dd0**, Evidence `ev_0c7097265f16bc7a09c0d3f3f17e1b34cc3261cd3e00af9dbe41055da9d6ee1e`, selects **0x8a3740** for internal kinds **0x9c43/0x9c44**. These are internal message values, not newly identified network packet opcodes. The fresh 37-byte callee performs a name refresh:

1. Load the context pointer from global **0xecf5a4** and call **0x6db8c0**, which returns its receiver+0x128.
2. Use that returned context as the receiver of **0x71f480**, with **main+0x424** as the explicit argument.
3. Pass the returned character pointer to **0x8a25f0**, restoring MAIN as that call's receiver. This formats the supplied text and invokes MAIN virtual+0x74 for **`static_character_name`**.

0x71f480 chooses the argument's string object at **+0xec** when context word0 equals **0x9c5**, context+0x18 is zero, or argument+0xd0 is zero. Otherwise it chooses **+0xc0**. It returns inline storage for capacity below16 and a heap pointer otherwise. The semantic meaning of the two name sources and those context flags is not established by this branch alone. Register-transfer bytes independently bind the context, explicit argument and MAIN receiver; the pseudocode's implicit `in_ECX` alone would not express all those links.

For internal kind **0x41e**, original instructions at **0x8a2153** load **ECX from main+0x138**, push **MAIN**, then call **0xb402a0**. The main pointer is the explicit argument; it is not the helper's receiver. That helper searches linked lists at the distinct context's **+0x18/+0x28**, with counts **+0x1c/+0x2c**. For a matching window whose byte+0x13c is zero, it unlinks and frees the list node, invokes additional helpers, then reinserts the window at a list tail. Successful first-list reinsertion returns before the second-list path. This supports list reordering in the selected context, rather than a direct assignment to the main mode byte. Complete effects of the called helpers and the concrete runtime class of context main+0x138 remain unobserved.

The fresh **0xb3d9e0** body stores its argument at **receiver+0x20**. Retained initialization bytes show MAIN in EBX, child in EDI, then `PUSH EBX` with `ECX=EDI` at both calls. Thus both constructed child+0x20 pointers receive MAIN. This owner-pointer relation does not prove that a particular button event reaches MAIN through that field.

## Lifecycle and base event behavior

Main virtual+0xd4, **0x8a1c20**, finds **`titlebar`**, obtains that child's rectangle through the already retained nullable accessor **0x722270**, restores MAIN as receiver, and passes the rectangle to **0xb377a0**. The latter replaces the first 16-byte item of a nonempty rectangle vector at receiver+0x328/+0x32c, appends into available capacity for an empty vector, or delegates growth to another helper. The source rectangle and vector destinations remain distinct; arbitrary destination aliases are not globally excluded. The subsequent **0x821040** invokes two base helpers, calls receiver virtual+0xd8(0), then returns AL=1.

Main virtual+0xd8, **0x8a1c60**, first calls 0x8a29c0, whose effects are not reconstructed in this pass, and caches six signed height expressions, each using rectangle bottom−top:

| MAIN field | Source/expression |
|---|---|
| +0x460 | `inframe` height |
| +0x464 | `outframe` height |
| +0x468 | MAIN height |
| +0x46c | `inframe_01` height |
| +0x470 | `outframe_01` height |
| +0x474 | MAIN height − cached outframe height + cached outframe_01 height |

It then calls **0x821070**. When receiver byte+0x3f4 is nonzero, that base routine iterates the list at receiver+0x44 and calls 0x9a5430(0) on each payload; both branches return AL=1. The full meaning/effects of that downstream control helper remain outside this pass. These six direct writes are dimensions at **+0x460..+0x474**; they do not identify a nonzero write to MAIN+0x458.

The reused main event callback **0x8a23d0**, Evidence `ev_550dc95b0cabda6eff467bede9e41278f11fa94fcef1bb72d91e222a2bcbdcc0`, calls **0x8210b0(0,0,0,0)** for event0x51, then forwards events to **0x8214f0**. The fresh geometry helper computes corrections against the receiver rectangle and two global bounds **0xe98c08/+4**; it requests relative movement through 0xb3c900 only if either correction is nonzero. With general arguments, it first corrects left/top underflow, then adds any right/bottom overflow correction. No live screen size or rendered position is inferred from the globals.

Both child event slots resolve to the **nine-byte 0x8a1920 wrapper**, which tail-jumps to **0x8214f0** with the existing arguments/receiver. That base callback requests **`ui_button_click.wav`** for event10 when receiver+0x3e8 is nonnull, then forwards to **0xb374c0**. The latter handles selected keyboard actions, tab traversal and `_Resize` press/release state. It can dispatch synthetic event10 through receiver virtual+0x18; indirect recursion/overrides remain relevant. Its inspected body has no named `button_fold`/`button_unfold` branch. The exact fold/unfold action binding remains unresolved; neither a shared slot nor the button names alone establish such a path.

## New native Evidence

| Entry | Analyzed body bytes | Role | Evidence ID |
|---|---:|---|---|
| 0x8a3740 | 37 | Name-refresh message callee | `ev_7b543ecc69afccdfc25a01054f9d9865db495da20e0cfd6bc81d9489954b3c2b` |
| 0x8a1c20 | 49 | Main titlebar/rectangle lifecycle callback | `ev_555e48d8e2d1429350dcb3ae8fe9855873fdb9f23d6e1fc0036fb0a1baf291b4` |
| 0x8a1c60 | 200 | Main cached-height lifecycle callback | `ev_1e63597953a7299e94154316eae70632f3375c212f1d983c4ac390c4016f4b60` |
| 0x8210b0 | 134 | Rectangle-bound correction | `ev_da362292f0e7f3678fc923bc799e3119ae522495262aac4a92bbe69eb1975a84` |
| 0x8214f0 | 60 | Base click sound and event forwarding | `ev_407003e4ea5171c09f9ea0798705d8b582ae97f26821e7be85a46c34d824c0c4` |
| 0xb402a0 | 255 | Context list search/reinsertion | `ev_5b4ce74ae2be7bf8b7c03aa2b155bf6ed5d5d28ebf85964e05c061ce6c353005` |
| 0x8a1920 | 9 | Shared child event tail-jump | `ev_80dfd3ef6c07cf9cbcd960b54634265e0e4bb7328f8b4460b5f638e114d3943c` |
| 0xb3d9e0 | 13 | Child owner-pointer setter | `ev_390fd092447e2b311ba69d0267e43c7383739030c06d243b2095e9e78977387e` |
| 0x8a25f0 | 150 | Named character-label formatting | `ev_284a3a99269481f6027352d3f98d133b6b38cb516dce5c2df16ddf02ab102153` |
| 0x71f480 | 53 | Conditional received-name selection | `ev_7140a65b1f384ac83242bad14b9aa045815b9b08fa77842be1b0e23b12b2153f` |
| 0x6db8c0 | 7 | Context+0x128 accessor | `ev_00013585555fdf85086fb3505e8f579ec55d0608d2eb4b765a37840a7b593a28` |
| 0xb374c0 | 373 | Generic keyboard/tab/resize events | `ev_f36a19d7e7b9b0ee9e2fd51793a98b9db8a22e3ab4b9b704c8358af9210a3f99` |
| 0x821040 | 43 | Base lifecycle and virtual+d8 call | `ev_635168ea1ab1ba99e4dbbf1d9dbfd3b64975a7ca3476836b13603be71e77fa54` |
| 0x821070 | 52 | Guarded child-list iteration | `ev_32e0fe19221451a38737df4448def71f0b0cfb20613d5b4b9616348c9208b5a8` |
| 0xb377a0 | 81 | Rectangle-vector update | `ev_c2bb0efb0f52587df1ec388c291b55150d5dd0486fd5b9e36245d726f224d138` |

All 15 new native value-flow graphs report **untruncated**, with zero reported omitted-operation lower bounds and known omitted edges. This does not resolve indirect callees, runtime types, vector aliases or external writes. Dossiers match the original SHA and analysis-profile digest `b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17`. The first call took **586.36 seconds**, including cold initialization; subsequent focused calls reused that engine. No producer was rerun to recover a truncated result.

The canonical bundle contains **115 Evidence records**: 100 imported unchanged and 15 new. Its size is **89,425,042 bytes**. The final **178,604,021-byte snapshot** retains **106 primitive entries / 115 records**. The first-function checkpoint is also retained. The owned native session and its analysis processes are closed. Raw dossiers, pseudocode, archives and local configuration remain ignored; only authored documentation is published.

Verification compares **387 table/RTTI byte records plus 26 selected instruction/operand intervals** to the original PE (**413 distinct selected records**). All 15 inclusive function body ranges are separately mapped and hashed against original bytes; assembly entry addresses are checked within those provider-reported ranges. The selected checks explicitly bind MAIN/child/context registers, the shared tail-jump, context/name calls and six dimension destinations. These counts are overlapping byte checks, not 413 independently recovered functions or a whole-program decompilation claim. Original encrypted index rows, archive ranges, the original key, phase0 transforms and all three cache matches pass.

All **nine original-record/settings preservation checks** pass again after the analysis. Original/installed images, addon/WTF contents and the primary service identities established after recovery are preserved. Pre-reboot PIDs are not claimed to have survived. Versions remain explicit and unchanged. No Rust behavior, schema, installed asset/binary or package version changes; earlier gameplay test counts are historical and are not new runtime validation for this static pass.

## Checklist and restart point

| Question | Result | Remaining boundary |
|---|---|---|
| Main alternate-mode activation | **Unresolved** | Still needs a concrete nonzero MAIN+0x458 writer with receiver provenance, or corresponding controlled original-runtime evidence. |
| Main lifecycle and base callbacks | **Answered for inspected bodies** | Dimensions, name refresh, list recipient and owner assignment are retained; uninspected callees and indirect effects remain open. |
| Resource/button callback binding | **Partially answered** | Archived resources and the shared child event delegation match; the exact fold/unfold action binding remains unresolved. |

Reuse this closed snapshot and the new function dossiers. Do not reinterpret 0x9c43/0x9c44 name refresh, the +0x460..+0x474 dimension caches, the distinct +0x138 list context or the adjacent child table as a proven MAIN+0x458 activation path. Future work should follow a concrete remaining property/copy/action binding or an identified original-runtime observation, while preserving data and the separate context/receiver identities.

The next [alternate-mode writer investigation](SFRAME_MODE_ACTIVATION_RE.md) adds 21 functions and five focused queries. It identifies the concrete WORD writer as SUIGaugeWnd through creation/registration/lookup, distinguishes further panel/game-object copies, and resolves the previously uninspected guild helpers and child-byte getter. The nine-rectangle guard does not prove a global cardinality bound. MAIN activation and the storage-copy receiver remain unresolved; reuse its newer closed snapshot.
