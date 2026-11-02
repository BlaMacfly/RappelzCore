# SFrame MAIN mode transition: unresolved activation audit

Verified **2026-11-02** on original x86 SFrame SHA-256 **`27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`**. This follows [the request-producer investigation](SFRAME_MAIN_REQUEST_PRODUCERS_RE.md), reusing **REA 6.4.0 / Ghidra 12.1.4** Evidence and independent **Capstone 5.0.7** byte checks. **The trigger of MAIN byte+0x458 is not identified. The typed readers presuppose a nonzero flag, the known typed writes clear it, and the receiver of 918630 is still unbound. A usable alternate-mode feature is not presumed: the user confirms this investigation concerns the flag found in code, not an observed client behavior.**

**This is a verification audit of retained evidence: zero new Native decompilations, zero new REA queries and no original-client execution.** No new engine session is opened. No duplicate incoming-reference, PDB, carrier, .text, clipping, list or menu-producer analysis is counted as progress.

[Versions](VERSIONS.md) remain Core/RappelzRust **0.1.0 development**, NoLimits launcher **1.3.60**, SFrame PE file/product **9.1.0.0**, embedded **Source9.5.2** lineage and network literal/profile **201507080**. Exact official Epic/patch remains unconfirmed.

## The actual MAIN state contracts

The scope concerns the code flag MAIN+0x458; the user reports no observed activation. The selected typed entries establish initialization, resets and reads. They do not establish a usable trigger or a currently active game feature.

Four tests in 8a1dd0 consume the existing byte at MAIN+0x458. Three branches (internal kinds 0x36, 0x31 and 0x406) reach two actual CALL sites for 8a30f0; the fourth (0x9/0x4c7) only skips selected HP/MP setters. 8a30f0 does not test or set the flag itself. It refreshes a selected actor and may commit its handle to MAIN+0x45c; committing that handle is not flag activation.

Constructor 8a19f0 initializes MAIN+0x454, MAIN+0x458 and MAIN+0x45c to zero. Visibility callback 8a1d40 clears 0x458/0x45c after 8213e0 returns. The 8a3500 write is a DWORD at MAIN+0x454..0x457, ending before the flag. Parent accessors return values in AL/EAX, not field pointers, and the selected core bodies form no +0x454/+0x458 address with LEA. The indexed child pointer and visibility setter remain an arbitrary-input alias boundary, not evidence of a legitimate activation contract.

The eight old MAIN59 flow caps were reconciled without rerunning an inventory. Every row visited all Native instructions and had an untruncated Native graph, while local alias-state variants were capped. None of those recorded states supplies a MAIN+0x454/0x458 address delivery or a write covering the flag. This is a bounded observation, not an exhaustive client-wide no-writer theorem.

A nonzero flag is a precondition of the selected reader branches; its actual transition remains unknown. Neither a live feature nor dead code is established. Indirect calls, heap aliases, omitted state variants, arbitrary corruption, reentrance, exceptions and later writes remain open. No new writer or receiver identity is established by this audit.

## Why the remaining 918630 hypothesis is not an activation proof

The retained 301-byte body of 918630 captures entry ECX as O in EDI. It passes O+0x454 to 8fb010, which sets byte O+0x458=1 and DWORD O+0x45c=0 without setting O+0x454. It then converts FLOAT[O+0x454] to an integer and invokes original virtual+0x78 with stack arguments including O and [O+0x44c]. These exact local effects remain conditional on a valid receiver and normal completion; no constructed MAIN receiver is loaded by this body.

The explicit callee contracts do not identify a derived class. The localization getter 668490 does not read its entry ECX in its own body; the warning vector contains record IDs, localized text pointers and flags, and its consumer receives a separate global context. The sole explicit whole-O handoff is physical argument 1 of the variadic formatter. Both original MAIN and Arena virtual+0x78 select b3d4f0, whose 69-byte body ignores argument 1 and calls b3d150 on argument 2=[O+0x44c]. That field is a recipient object pointer, not a recovered string name. These calls provide no type discriminator for O.

The other real caller of 8fb010, 917aa0, already has a constructed/registered SUIArenaWaitingWnd provenance. It explicitly writes a FLOAT through Arena+0x454, then resets and reads the same component. In contrast, typed MAIN 8a3500 stores an integer tab selector at MAIN+0x454 and uses it to index separate Basic/Added child pointers. This invalidates using the shared helper or footprint as evidence of MAIN identity. It neither identifies 918630 as Arena nor logically falsifies MAIN: state-dependent field reuse, casts and aliases are not excluded.

No new indispensable Native continuation is justified by these explicit callee inputs. The receiver of 918630 remains unbound, and 918630 must not be reported as an established MAIN flag activation. No usable alternate-mode feature or prior observed mode is presumed. The missing code evidence is an actual reaching transfer/registration to this entry together with the origin and state of its ECX, or a separately qualified direct/alias writer to MAIN+0x458 on the same constructed instance. Earlier bounded incoming/carrier/model results are reused and are not global unreachability proofs.

All evidence is static and source-bound. Original tables, valid objects, successful helper completion, normal ABI/callee-saved registers, pointer lifetime and preserved alias/state are conditions. Global callbacks or runtime replacement may change behavior. No claim of callback purity, universal nonaliasing, inaccessible code or impossible MAIN execution is made. Complete original Native body checks remain separate from retained pcode truncation bounds. This receiver audit reuses retained evidence without repeating scans, running the original client or performing new Native analysis.

## Evidence and preservation

All selected records retain target SHA-256 and profile **`b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17`**. None of the selected reused Native graphs is truncated. Complete source bodies, selected machine instructions and bounded analysis states are distinguished from whole-program effects.

| Reused Native entry | Evidence ID |
|---|---|
| 0x6682e0 | `ev_eacf49e82d8bd52862eefdd63bc0d384f05f7f13628ede5c6a21ce3d7951f9ca` |
| 0x668490 | `ev_9b414244d3c9fe1767426262f2f36600274fa44af128a167d6c4ad76326864e4` |
| 0x6dbac0 | `ev_942f724e6af58bdd09da6c669c93854a3a0ab9889c5940ebbd43e0977b769464` |
| 0x6ffe80 | `ev_1e4b82b2a705e0a35f48b3c88596d1c05c0c7e8370215efd4abc4861d2275558` |
| 0x8a19f0 | `ev_f3a4ff37e3949c13ed452102ee534729623efb381bd564fe0da8330bbded3cd2` |
| 0x8a1ab0 | `ev_6d832c9d0196d2b1898603d93fd01bcd02fc3ba1df5750e51a235611f561c24c` |
| 0x8a1c10 | `ev_849e554ef2e3225eef34ad284e75a062591ffc3ec4f63338b8dfbbe1e59ee72b` |
| 0x8a1d30 | `ev_3cfd8199a45a65e80dd1d832beb282b912162f8c06beeac5a98a9a30e231812e` |
| 0x8a1d40 | `ev_10e965f99af0876e0d108b0ba4bfd7820f456137fa2b9dfde61ae54ea385f824` |
| 0x8a1dd0 | `ev_0c7097265f16bc7a09c0d3f3f17e1b34cc3261cd3e00af9dbe41055da9d6ee1e` |
| 0x8a23d0 | `ev_550dc95b0cabda6eff467bede9e41278f11fa94fcef1bb72d91e222a2bcbdcc0` |
| 0x8a30f0 | `ev_295a9ba6abb1926edf4a6b5e4cc827f166f4a84d714ca3efa0c8d65301b94b1a` |
| 0x8a3500 | `ev_a4b453f0e7c50f8457c67db0e7c4379d6645e67044af83a62c0e95472cc3a20c` |
| 0x8fb010 | `ev_6311d6edbba44308e24bffdc741ec9c6fbccbb16ca26c14b03969b080907bce5` |
| 0x916550 | `ev_596c3d917c90a0709cbb04e922e15feda7569b25cb26d777229825c4cde8e735` |
| 0x917980 | `ev_6f667ccfb3e130926af3c52d22c1e224e43b001a72658002aaf68a586c4e976c` |
| 0x917aa0 | `ev_5c5e2e76bb180562efa78127363e7cf4c687dc112ad925f6c515dde17249fb27` |
| 0x918630 | `ev_2e11babc974a7b4241ded6116bc72be4cbafa3d434f034fc40c37e79a04da7bc` |
| 0xb3d150 | `ev_b3f2a3d86fd5d7754a07cd1d8d79d1787614f149263c7cbf6b6b608809451baa` |
| 0xb3d4f0 | `ev_ad8725bbe05f2378247b79f6c19fb6a39e2b4122eebe575c6ae63730f4a888d5` |
| 0xb3da20 | `ev_7a6e5bd724d51e89d5bd91d5a371f5b813d624cb2c2b6636b1dd37a41df7d2e4` |

Independent revalidation checks **21 retained Native bodies**, source-file hashes, instruction starts and **199 distinct selected provenance intervals**. The transition and receiver ledgers remain local, with their original-byte checks and exact negative boundaries. They introduce no new canonical REA record.

The prior **694-record** canonical bundle and **25-primitive** snapshot are reused in place. Snapshot: `2026-11-02-sframe-main-request-producer-a7e4c913/sframe-main-request-producer.snapshot.json`, **334,182,387 bytes**, SHA-256 **`e58370959b87a78f48f4873670731515a3292b845486cc7aa4cd36d71c5fa865`**. That provider session was already closed; no new snapshot or freshly closed session is claimed.

Ignored local audit: `client-data/audit/2026-11-02-sframe-main-mode-transition-2d8a6f31`. **Nine preservation checks pass.** Eleven baseline file hashes, saved-variable tree, logical database backup and primary service identities remain unchanged. Root Git index empty; only seven authored Markdown files are synchronized through isolated Core/Client worktrees. Gameplay code, versions, original binaries/assets, databases and settings remain unchanged.

## Required next evidence

A real activation requires a source-bound write of a nonzero value to +0x458 on the same constructed MAIN instance, with its causal entry and state. Alternatively, a reaching transfer to 918630 and the origin/type/state of its entry ECX could establish or reject that candidate. Matching offsets, a shared reset helper, float/int role differences, retained state caps or a synthetic forced state do not supply that evidence. Available native-call runtime tooling advertises Mach-O/LLDB, not a Windows PE data-watchpoint workflow; no original SFrame process or debugger session is assumed. Further static calls need a new concrete anchor; the audited contracts do not currently justify another generic callee sweep. The activation question remains unresolved.
