# SFrame progression properties, history and stat dispatch

Static investigation completed **2026-10-31** on the supplied **PE32 x86** SFrame, SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`. Fresh **REA 6.4.0 / Ghidra 12.1.4** dossiers cover nine selected functions. The original executable was inspected without being launched or modified. Recovered pseudocode is decompiler output, not original source.

The nine selected analyzed bodies cover **5708 bytes**, not the whole program. The pass establishes how the client receives current job, job level, JP and three previous job/level pairs, and how it dispatches stat snapshots. It also separates a party `CHANGE_JOB` command from personal profession promotion. These observations refine the reconstruction boundary; they do not supply the missing original server promotion policy or JobLevelBonus coefficients.

## Questions and outcomes

| Question | Result |
|---|---|
| Is a numeric PROPERTY truncated to one 32-bit word on receipt? | Answered: both halves of the wire i64 are used to create its internal decimal string. |
| Are current job/JLV/JP and history one implicit promotion operation? | Answered for the inspected receiver: they have separate typed assignments; there are three separate previous-job/level pairs. |
| How do actor job/JLV properties propagate? | Answered for the inspected handlers/setters; wider visual/runtime behavior remains outside static evidence. |
| What does the own STAT_INFO handler do with type? | Partially answered: own/other routing, type forwarding and type 0 cache replacement are established; the deeper stat-bank/type semantics remain unexamined in this pass. |
| Does the `CHANGE_JOB` token prove a personal promotion entry point? | Answered for this seed: its record-update behavior and surrounding context support party-member command processing. It is not evidence of the missing NPC promotion operation. |
| Are original promotion reset/JP/history rules or JLV stat coefficients established? | Unresolved. Independent client storage paths do not establish the originating server operation. |

## PROPERTY: wire value and independent state

The numeric wire path in **0x00a29150** reads both dwords at packet **+0x1c/+0x20**, formats the value with `%I64d`, and transports a decimal string in internal message kind **0x36**. These are wire offsets including the seven-byte header. The property identifier is resolved separately from its name.

The mapping of property names to internal indices is retained in the earlier **0x00a1f9f0** constructor analysis (`client-data/proto-re/tmnames.c`, lines328–370). Its exact name strings and the selected consumer instructions were independently checked against the current original PE. This constructor text is historical analysis; it is distinguished from the nine fresh dossiers below.

The own-player branch in **0x006cdd70** compares the property handle with the current player before applying these fields. JP is parsed/stored through a 64-bit path. Job identifiers, JLV and each history component use separate 32-bit decimal parses/stores:

| Property name | Internal index | Internal player-state offset | Width |
|---|---:|---:|---:|
| `jp` | 9 | +0x30 / +0x34 | 64 bits |
| `job_level` | 10 | +0x28 | 32 bits |
| `job` | 11 | +0x38 | 32 bits |
| `job_0` / `jlv_0` | 12 / 13 | +0x3c / +0x48 | 32 bits each |
| `job_1` / `jlv_1` | 14 / 15 | +0x40 / +0x4c | 32 bits each |
| `job_2` / `jlv_2` | 16 / 17 | +0x44 / +0x50 | 32 bits each |

These player-state offsets are **internal layout facts, not packet offsets or a proposed Rust memory layout**. The history assignments are independent. Their names/slots do not establish oldest/newest ordering, when to shift/backfill, initial sentinel values or how a profession change should alter current JLV/JP. Overflow behavior for out-of-range 32-bit strings was not executed or established.

The world PROPERTY handler **0x006afd90** routes current JLV to **0x0095cfa0** and current job to **0x0095cf80**. Those wrappers lead to **0x00941780** and **0x00941760**, respectively. The deepest setters use object-address-dependent encoding for their own field. The JLV setter distinguishes initial/unchanged state from a subsequent changed value; the world handler conditionally creates internal notifications. Those local assignments contain no JP debit or history shift. The additional job-classification/notification callees are not all decompiled, so their wider effects remain unknown. The inspected world handler has no history-index12–17 branches; the separate own-player receiver retains those fields.

## STAT_INFO and the CHANGE_JOB false lead

The fresh **0x006d2b70** consumer checks a nonzero own handle before calling **0x007056e0** with two internal data pointers (**message+0x17/+0x27**) and the type byte (**message+0x79**). Its separate classified-entity branch calls **0x007116d0**; an existing actor can also receive **0x0095db50**. For **type 0 only**, it replaces a cached record with the same handle and appends a **122-byte internal message record**. These are internal-message offsets/size, distinct from the 110-byte network frame and its type at+0x6d. The three deeper callees are retained as concrete next seeds. Their stat-bank calculations and the meaning of every nonzero type are **not established** by this dispatch function; the earlier suggested type 1 “bonus/equipment” meaning remains an inference. The Rust own type 0 projection is unchanged.

The function at **0x006f5360** compares `CHANGE_JOB` alongside party command vocabulary. It requires at least three tokens, looks up a record using the second token and parses the third as32 bits into record+0x20; CHANGE_LEVEL uses+0x24. The same receiver includes CHANGE_NAME and PINFO, with an internal UI-window notification in the PINFO path. The nearby original SPartyMgr context corroborates the **party-member metadata interpretation**; the exact original class/runtime exchange is not reconstructed here. This seed cannot justify implementing an NPC profession reset or changing durable history. The earlier original NPC contact strings `NPC_JobChange_contact()` and `NPC_Tutorial_Job_Change_contact()` identify different entry points whose server bodies remain missing.

## Fresh native Evidence

| Address / analyzed inclusive body | Bytes | Role | Fresh Evidence ID |
|---|---:|---|---|
| 0x6cdd70–0x6ce81a | 2731 | Own player PROPERTY receiver | `ev_f1a5bbf3822c95017cf6757ff510f8cbe0220605ea2298f4e68a9694916590a6` |
| 0xa29150–0xa2942b | 732 | Wire PROPERTY parser | `ev_78c57e95fb4d0ed305027ac2545286cd5a531b66ea9dfb5fbf2d21a48c3d9901` |
| 0x6afd90–0x6affe6 | 599 | World actor PROPERTY handler | `ev_ab81b0330fa628683347e0c4e6a291ca81028511065868e93c7f4df9aac1f95c` |
| 0x95cf80–0x95cf96 | 23 | Actor current-job wrapper | `ev_f2d40ac830f1660dc5a65139139e542cf0cc2a9fffd6421ac7a3be6f6c2e7359` |
| 0x95cfa0–0x95cfb8 | 25 | Actor current-JLV wrapper | `ev_667fb7986c770c84dd6b1976fe95942140eb8d0dc0387a82e2cd723a42c2ffc8` |
| 0x941760–0x941775 | 22 | Encoded current-job field setter | `ev_951640c6f9b77a758d488ef634b0c94866e22ed1024ea9e959fd07c69e1ac2e8` |
| 0x941780–0x9417b5 | 54 | Encoded current-JLV field setter/change indication | `ev_2bd6570685f03b441a582d23da6293c4ea3119c4088a31f7038958c8480128bb` |
| 0x6d2b70–0x6d2e27 | 696 | STAT_INFO dispatch/cache | `ev_3ed427a22391a30e210e79f6336793e1bf740b28f5cb257d4d27cb20de1fee69` |
| 0x6f5360–0x6f5699 | 826 | Member command parser including CHANGE_JOB | `ev_ca6cc9fea9783bb2259a70a6b1e39597db9efd05ecaf17ac29e9bb5a3124f39f` |

All nine complete dossier records bind to the target SHA and the provider analysis profile. Complete analyzed body ranges and raw instructions are retained; this does not make every inferred type/name authoritative. Native value-flow can be bounded/truncated independently of the complete body and pseudocode. All nine returned value-flow records report `truncated=false` and no known omitted edges/operations within their represented graphs. This does not resolve targetless indirect calls. Ghidra marks terminal flows in the PROPERTY parser to 0xc6ad3d and in both actor wrappers; those provider flags are not independent proof that a callee cannot return. The own-property pseudocode also reports removed unreachable blocks in unrelated paths. The raw complete dossiers retain these warnings. Ghidra default analysis, indirect calls, alias semantics and the absence of original runtime observations remain explicit limitations. No full original-client interoperability or official Epic/patch equivalence is inferred.

The integrated MCP attempt hit its fixed **300-second** tools/call deadline during initial default Ghidra analysis and produced **zero native Evidence records**. It was closed with its zero-answer snapshot. Recovery used the **same installed public REA launcher**, its supported stdio tools and a bounded **900-second** initialization/request deadline, preserving the original target and analysis preset. The successful initial function query took **616.73 seconds**, including default auto-analysis; the remaining eight selected functions reused that live Program and each completed in less than one second. The successful dossiers, bundle and final snapshot are separate from the failed attempt. The final snapshot retains **nine primitive entries and nine Evidence records** (11,410,054 bytes), with analysis-profile digest `b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17`. The owned analysis process/session was closed after export.

## Preservation, publication and next probes

An independent PE check confirms **37 exact byte/string anchors** and their VA/file-offset mappings. All **nine original preservation checks** pass. Original SFrame/archive index, installed client/extractor/world images, addon code, WTF and primary listener identities remain unchanged. This pass writes analysis evidence and documentation; it adds no server behavior, schema/data migration or installed gameplay change. The previous [character-stat runtime validation](CHARACTER_STATS_VALIDATION.md) remains a separate historical result; its Rust/Python/native counts are not rerun or reattributed here.

Raw Evidence, snapshots, instruction/pseudocode output and the private ledgers stay ignored under `client-data/audit/2026-10-31-sframe-progression-bc790366/`. The supplementary byte ledger is `client-data/audit/2026-11-01-job-properties-static/ledger.json`; that directory's date is a label, while this investigation ran on 2026-10-31. Only documentation is published to both branches.

Next probes: recover the actual server promotion scripts/bonus dataset or equivalent authorized runtime evidence; determine history chronology/defaults and promotion transactions before exposing a durable promotion operation; follow 0x007056e0, 0x007116d0 and 0x0095db50 to establish any additional stat-type semantics before extending the current own type 0 projection. Preserve existing JP/JLV/history/EXP and source provenance throughout.

## Follow-up: stat banks and UI, 2026-11-01

The three stat seeds left open above are now examined in [SFRAME_STAT_BANKS_RE.md](SFRAME_STAT_BANKS_RE.md), with11 fresh native dossiers including direct UI consumers and two selectors. Type0 replaces one98-byte bank; type1 replaces a second and subtracts selected fields from the first. The cache is order-sensitive, with an opaque skipped word and two derived-word exclusions. Actor creature-data stays behind the inspected type0/handle gate. The original UI dispatch and all seven primary field positions are corroborated; four large high-p-code graphs remain explicitly truncated. Server contribution origins/coefficients, legal send ordering and original runtime behavior remain unresolved. The earlier partial result is preserved as the scope of that pass, not a reason to repeat those same three queries.
