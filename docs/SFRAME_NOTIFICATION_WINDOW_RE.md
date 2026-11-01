# SFrame notification window 0xd3 and its handler

Verified **2026-11-01** on original x86 SFrame SHA-256 **`27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`**, using **REA 6.4.0 / Ghidra 12.1.4 / Capstone 5.0.7**. This follows [the Colosseum warning investigation](SFRAME_COLOSSEUM_RULES_RE.md). **Window 0xd3 is constructed as SUINotificationWnd. Its message handler, registered pushNotification callback and selected queue-to-caption path are identified. MAIN alternate-mode activation remains unresolved.** New evidence: **8 Native dossiers / 2,590 analyzed bytes** and **0 other observations**. These are static artifact observations; the original client is not executed.

[Versions](VERSIONS.md) stay Core/RappelzRust **0.1.0 development**, NoLimits launcher **1.3.60**, original SFrame PE file/product **9.1.0.0**, embedded **Source9.5.2** build lineage, and network literal/profile **201507080**. Exact official Epic/patch remains unconfirmed.

## Registration, class and resource

Retained bootstrap **9cb4e0** pushes ID **0xd3 at 9d06d6** and calls **9beae0 at 9d06dd**. New Native 9beae0 allocates **0x488 bytes**, calls retained constructor **91b7f0 with [Interface+8]**, and passes the result and the caller-supplied ID 0xd3 to **9c8920 with mode 0**. The constructor installs vptr **d5cb38** at 91b81e; exact **vtable[-4]→COL e0f544→type descriptor eb1f9c** gives **`SUINotificationWnd`**. Slot **+0xcc points to 91c050**, not directly to 91cf60. Successful construction/registration and unchanged pointers are conditions of this path; an arbitrary pre-existing cached window is not identified by a factory allocation alone.

On the selected **mode-0 branch**, 9c8920 reads the ID-indexed resource table at **9c8b14**, invokes the constructed window's **virtual+0xd0 at 9c8b41**, and calls retained registration helper **9e2aa0 at 9c8b47**. The selected table slot **e979d4 = e97688+4×0xd3** points to literal **d692b8**, **`window_main_notification.nui`**. The exact archive member is **3,016 bytes** in **data.006 at offset 589850651**, index record at offset **3997256**. Decoding gives SHA-256 **`060739707596e240027f2dbc587720f71b528252757de8c90ec21a1f8c30dd7c`**, identical to the existing extracted cache. Template control names include `button_close_all_01`, `button_notify_close_01/02`, `button_notify_disable_01`, `button_notify_titlebar_01` and `text_notify_titlebar_01/02/03`. Control names establish resource contents, not button behavior. The mode-nonzero loader branch is not substituted for the actual mode-0 call.

## Handler and effects

The common loader/post-load path reaches window-specific initializer **91bda0**, which calls **91cbe0**. On the initial empty-registry/insertion path, this binds internal kind **0x426** to callback **91cf60** in the registry rooted at **this+0x450**, stored at **node+0x14**. If that key already exists, the selected branch preserves its prior callback. Native **91c050** looks up **message+4** in this registry and invokes the selected callback with **ECX=this** and the message on the stack. The preceding warning producer's **SIMSG_UI_SEND_DATA / kind 0x426 / window ID 0xd3 / `pushNotification`** therefore has a concrete constructed recipient path through the retained interface dispatcher and window lookup. Interface guards, allocation/registration failures, handled-state branches, absent windows and later pointer replacement remain explicit conditions.

**91cf60 marks message+8 handled before checking the command**, including an unmatched command. On the selected path it checks the exact **16-byte `pushNotification`** command and copies the supplied caption from **message+0x4b** into a local string. If the window's visibility byte is clear, it posts an **internal kind-0x3f9 request for window 0xd3** through retained 9eeb40, which enqueues a local message; this is a show request, not an observed visible window. It inserts the caption into the pending list at **this+0x460**, using retained node helper **7d33b0**, and updates its count at **+0x464**. Selected update method **91c0b0** reaches **91c760** under its animation/state guards; 91c760 transfers the pending front to the list at **this+0x458**, removes that pending entry, and calls **91d480**. The latter composes the retained prefix with the front caption and sends it through **b3d150** to the control stored at **this+0x47c**. Retained lookup helper **b3c660** binds that control to **`text_notify_titlebar_03`**. This reconstructs the selected queue-to-caption path; actual screen rendering, every timer/button branch, allocator/map-insertion internals and complete later fanout are not claimed. These internal message kinds are not newly recovered network opcodes. The retained caption setter null-guards a missing control, but also has an optional global caption hook and a child virtual+0x3c call; downstream effects and reentrancy remain open. The control binding depends on successful NUI child creation/lookup, and original queue helpers assume valid allocations; no universal null-safety is inferred.

## Continuing the alternate-mode investigation

The notification object has **two 8-byte list headers at +0x458 and +0x460**, initialized on its retained constructor path. They are distinct from the **byte at MAIN+0x458** in constructed **SUICharacterInfoWnd**, ID **0x2e**, vptr **d53af8**, allocation **0x478**, resource `window_player_character_info_main.nui`. The notification caption/list path supplies no typed nonzero MAIN-mode assignment. Sharing a numeric offset or the b3d150 text helper does not bind an object to MAIN.

An independent review reuses **15 complete Native records** rather than repeating producers. **917810** builds the same warning recipe without explicitly capturing its incoming ECX in its own body; this provides no receiver provenance for 918630. **917980** preserves its entry receiver and uses +0x454, but its construction and registration remain unknown. The selected **virtual+0x78** calls pass the outer object on the stack while ECX holds the vptr. Under a MAIN hypothesis only, retained **b3d4f0** principally formats and assigns to the second physical argument; that compatibility neither proves nor excludes MAIN. The singleton getter's lazy initialization, transitive callees, reentrancy and unknown pointer flows are not promoted into universal no-effect claims.

**918630 still forms outer+0x454 for 8fb010, yielding byte outer+0x458=1 and DWORD outer+0x45c=0. Its incoming target/registration and EDI receiver remain unknown.** Identifying notification recipient 0xd3 answers the warning-destination question without identifying the receiver of that earlier reset call.

## Evidence and verification

All new records bind the original target SHA-256 and profile digest **`b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17`**. All new Native value-flow graphs report no truncation. Reused graphs retain truncation: 0x9cb4e0 (0 known omitted edges; omitted-operation lower bound 1), 0x9d3db0 (240 known omitted edges; omitted-operation lower bound 1). Complete retained records, selected original-byte checks and limits remain distinct from complete runtime execution or interprocedural effects.

| New Native entry | Analyzed bytes | Evidence ID |
|---|---:|---|
| 0x9beae0 | 202 | `ev_7e85fae44e2953a19c3737aef7ba23228a3a155e5c765af9a59d585c7d33bd67` |
| 0x91c050 | 82 | `ev_5935fb1acc149e4bcd4f4383bcd31c732d8f226ebb70f8ab6fcabbd086feaa11` |
| 0x91bda0 | 425 | `ev_1080538886678a091af237ab8d8b53da58fca355129b1d244723722404527adb` |
| 0x91cbe0 | 456 | `ev_cee916d8a7708806a8cebb30b2781eea8e9f227ed90c10e90fb3677af7bc9982` |
| 0x91cf60 | 410 | `ev_917a4e08f9a23e312d49594d1d5ed558f41e8ee89b7f8997f4f9ab0bb6a06e1d` |
| 0x91c0b0 | 429 | `ev_e3a99607bd2f6ee9491dc8fb53e83c657ca2c3006ec9e9e883f4f3ea54dca091` |
| 0x91c760 | 390 | `ev_ecf8f2eedb0b118331dca662e31e74fef706a60ea6f0501ad25033e19018d703` |
| 0x91d480 | 196 | `ev_6f3bd19e103d5979a85861e1ca295b2fd4aa45a98802f45c7d5d493ceb4bbc5f` |

| Selected reused Native entry | Evidence ID |
|---|---|
| 0x668490 | `ev_9b414244d3c9fe1767426262f2f36600274fa44af128a167d6c4ad76326864e4` |
| 0x7d33b0 | `ev_39f595adc16318704854f252a105484702bf5473f00bec1ccae7dcd5762ec58b` |
| 0x7dd470 | `ev_36f5829729c777133146544d87c6634be99bd801dfa401d8b91cbe91347a0f4d` |
| 0x820f70 | `ev_bf6a01b950ce19ea40f7d2a5709725e5431a157e569999caec2f85e087de96e0` |
| 0x821040 | `ev_635168ea1ab1ba99e4dbbf1d9dbfd3b64975a7ca3476836b13603be71e77fa54` |
| 0x821070 | `ev_32e0fe19221451a38737df4448def71f0b0cfb20613d5b4b9616348c9208b5a8` |
| 0x8a19f0 | `ev_f3a4ff37e3949c13ed452102ee534729623efb381bd564fe0da8330bbded3cd2` |
| 0x8a1d40 | `ev_10e965f99af0876e0d108b0ba4bfd7820f456137fa2b9dfde61ae54ea385f824` |
| 0x8a1dd0 | `ev_0c7097265f16bc7a09c0d3f3f17e1b34cc3261cd3e00af9dbe41055da9d6ee1e` |
| 0x8a2970 | `ev_ba6ae9a8e11a396363b4c97bbaea1426cb12012725e0aa4087706088060e1167` |
| 0x8a29c0 | `ev_41852a085c13383ec31fa0eeb8dd89f372bb14f7541fc89ffabf33ffbddb4754` |
| 0x8faf70 | `ev_2438ea2ed528ccdf2c4e457359892f32d22dab8e4567ce96ac284282fe718e33` |
| 0x8fafa0 | `ev_da809073bd3cb75b6fe68756f7a1fe446bf0150b6c6bf804bd8db6c5b34c4433` |
| 0x8fb010 | `ev_6311d6edbba44308e24bffdc741ec9c6fbccbb16ca26c14b03969b080907bce5` |
| 0x917810 | `ev_85e0d1e5f82d8e0bbd61096a80d3e3c93a6b604c1d0a74ee0bae63959df29b08` |
| 0x917980 | `ev_6f667ccfb3e130926af3c52d22c1e224e43b001a72658002aaf68a586c4e976c` |
| 0x918630 | `ev_2e11babc974a7b4241ded6116bc72be4cbafa3d434f034fc40c37e79a04da7bc` |
| 0x91b7f0 | `ev_0665be855913f559ae733d854da05cb6873e4ecd16ab372d07f88f69f00284b1` |
| 0x9c8920 | `ev_4a5435ee59e170130751d04fb39f47a481fa7b5c82322bcd854db8e838d4e2cc` |
| 0x9cb4e0 | `ev_4275b7d64861bda9d3f1369468c58deec344a973e3c6e12679479663f89ad441` |
| 0x9d3db0 | `ev_83e67f6199a1a76051df09eebc8044ed8a07305f6d8d4781f3f2f0314abbfc2f` |
| 0x9dd9b0 | `ev_0574d2fb38e2b73df2742d3b8962198eef271497c7cdfe3897846e84f6655b23` |
| 0x9e2aa0 | `ev_8517cbb679aeec97719d6893e9c703edabbd32bf7ff31ca240d710d42bb50bcc` |
| 0x9e4330 | `ev_9d7dc073b11c44636f3419900ec9739205ea7ec609bdc7d5ad6edc9e00bdbbb2` |
| 0x9e50b0 | `ev_7f352aca262f6060b748c0b2382c24332373d752ed9e903be613570aa89f05ad` |
| 0x9eeb40 | `ev_f1dc736455bf4e490116d08bee86f74588317ce0d3d8eefdaa12e0e7415c9aee` |
| 0xb3c660 | `ev_666b2d425f99f55ebaefa73584a86da66c18f086c8e8466d973a26499d848392` |
| 0xb3d150 | `ev_b3f2a3d86fd5d7754a07cd1d8d79d1787614f149263c7cbf6b6b608809451baa` |
| 0xb3d4f0 | `ev_ad8725bbe05f2378247b79f6c19fb6a39e2b4122eebe575c6ae63730f4a888d5` |

Independent checks bind every new Native body to original file bytes, verify **759 new assembly boundaries**, **130 selected provenance intervals**, and **29 separately retained Native bodies**. The three registration/handler, resource and mode ledgers preserve their own hashes and limits. Overlapping byte checks are corroboration, not separate game executions.

The owned Native session checkpoints **8 new observations**, officially imports the preceding **601-record canonical bundle**, exports and closes. All preceding canonical records remain equivalent as JSON values. The final bundle holds **609 records / 281,643,161 bytes**. The closed snapshot holds **29 cached primitives / 294,302,726 bytes**, SHA-256 **`a9973bdee1d496ecbfa28409652c3363fb2611399cf53ad741dc7f12c626d2fe`**. The historical Native index retains complete older records outside this canonical subset.

Ignored local run: `client-data/audit/2026-11-01-sframe-notification-window-0a4f3b72`. Complete records, original-byte/resource provenance, three ledgers, checkpoint, snapshot, preservation and publication receipts stay local.

**Nine preservation checks pass.** Eleven baseline executable, addon and settings hashes, the saved-variable tree, logical database backup and primary service identities remain unchanged. The root Git index stays empty. Only seven authored Markdown files are synchronized through isolated Core/Client worktrees; original assets/binaries, databases, credentials, settings and generated decompiler output are not published. Gameplay code and package versions are unchanged.

## Restart point

Registration/class/resource, the virtual+0xcc registry dispatcher, its kind-0x426 callback and the selected pending-list-to-caption path are answered statically. Complete runtime registration lifetime, all notification actions and the original character-sheet mode trigger remain unresolved. Reuse the complete new functions and three ledgers. Continue from an actual reaching registration/computed target and receiver for 918630, or a source-backed typed MAIN pointer reaching a nonzero mode write. Do not repeat the empty caller query, the old numeric sweeps or retained formatters, and do not use notification list headers or warning text to type MAIN.

## Follow-up: notification actions and countdown, 2026-11-01

The [action/countdown investigation](SFRAME_NOTIFICATION_ACTIONS_RE.md) continues from the identified recipient and mode-writer candidate. Notification close-one/close-all dispatch and cleanup are traced; initialization hides the disable control and no saved disable option is established on the selected path. Typed countdown state and time conversions are separated from MAIN's tab/mode fields; the 918630 receiver and alternate-mode activation remain unresolved. Data, settings, services and versions remain preserved.
