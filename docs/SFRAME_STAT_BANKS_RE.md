# SFrame stat banks, secondary contributions and character UI

Investigation started **2026-10-31**, completed **2026-11-01**. Fresh **REA 6.4.0 / Ghidra 12.1.4** dossiers cover **11 selected functions / 23492 analyzed body bytes** of the supplied original SFrame. This follows the three next seeds retained by the [previous progression investigation](SFRAME_PROGRESSION_RE.md). The original executable was inspected statically, without being launched or modified.

The [version identity](VERSIONS.md) stays explicit: our RappelzRust/Core **0.1.0 development**; original **Rappelz NoLimits**, launcher **1.3.60**, SFrame PE metadata **9.1.0.0**, build-path label **Source9.5.2**, network literal **201507080**. The exact official Epic/patch and full original-client interoperability remain unestablished. Every native dossier binds to original SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`.

## What the two stat types actually do

The inspected own writer **0x007056e0** receives a 16-byte primary block, an 82-byte derived block and a type. It contains copies and subtractions, not server stat-generation formulas. The first primary word remains opaque in this pass; the historical name `stat_id` is conventional rather than newly proved by these consumers.

| Type | Own cached state | Classified-entity path | Inspected actor wrapper / UI |
|---|---|---|---|
| 0 | Replace bank A's full 98 bytes. Retain bank B. | Same operation after finding the supplied handle in its list. | Actor gate permits the update; UI refreshes primary and derived values. |
| 1 | Replace bank B's full 98 bytes, then subtract selected incoming values from the current bank A. | Equivalent width/field operations on separate banks. | Actor gate refuses the creature-data update; UI refreshes main values, then primary/derived `plus_value` controls. |
| Other values | No write in these two bank writers. | No write in the inspected writer. | Actor gate refuses; the inspected UI stat case does not run the type 0/type 1 refresh calls. |

The own banks occupy **[+0x11c,+0x17e)** and **[+0x17e,+0x1e0)**. The separately classified entity/summon bookkeeping path **0x007116d0** uses **[+0x98,+0xfa)** and **[+0x10c,+0x16e)** after its handle lookup. These are **internal object offsets**, not network offsets or proposed Rust struct layouts. The exact runtime class/lifecycle of that list is not reconstructed here.

Type1 subtracts **seven primary 16-bit fields**, **six derived 32-bit fields** and **27 derived 16-bit fields**. It does not subtract the opaque first primary word or the derived words at relative **+0x2e/+0x30** (network **+0x49/+0x4b**). The older layout calls those two fields max_weight/casting_speed; that naming is distinguished from the newly observed arithmetic. Both excluded words are still copied into bank B. Bank A therefore must not be described as a uniformly recomputed “base” across all 98 bytes. No clamp is present in these selected arithmetic paths.

This operation is **order-sensitive and not idempotent**. For an affected field, a type 0 value20 followed by type 1 value3 leaves17; another identical type 1 leaves14. This is an illustration derived from the observed assignments, not an original runtime experiment. Type0 alone does not clear a previous secondary bank. The originating server's legal send order, refresh pairs and replay guarantees remain unknown. Type1 is a copied secondary contribution with subsequent subtraction, rather than a generic incremental network delta. Its exclusive origin in equipment is **not established**; job/state/equipment bonus coefficients cannot be recovered from these copies alone.

## Actor data and UI source selection

The inspected wrapper **0x0095db50** requires a matching actor handle, type 0 and a nonnull creature-data pointer before forwarding to **0x00940f90**. That receiver's internal stat case copies the full 98 bytes to data **[+0x80,+0xe2)**. Its type gate belongs to this wrapper, so it is not asserted as a universal rule for every possible caller of the deeper receiver.

The primary display **0x0089b7e0** can read the selected actor's 16-byte block at **data+0x80**, or fall back to the own cache at **+0x11c**. The two additional getters return the UI mode byte **+0x458** and selected handle **+0x45c**. This establishes conditional data-source selection, not the full lifecycle/meaning of every UI mode. The derived display **0x0089d9b0** follows the corresponding derived blocks.

The character-window message consumer **0x008a1dd0**, internal kind **0x1f**, selects four virtual slots in table **0x00d53448**:

| Slot / actual function | Type0 | Type1 |
|---|---|---|
| +0xf0 → 0x0089b7e0 | Main primary refresh | Main primary refresh |
| +0xf4 → 0x0089d9b0 | Main derived refresh | Main derived refresh |
| +0xf8 → 0x0089a390 | Not called in this case | Primary `plus_value`, input message+0x17 |
| +0xfc → 0x0089c2c0 | Not called in this case | Derived `plus_value`, input message+0x27 |

The type 1 calls occur in the listed order. Message+0x79 is the **internal** type byte, distinct from the wire type at+0x6d in the 110-byte network frame. The primary plus renderer hides a zero contribution and chooses different styles for positive/negative contributions while formatting the magnitude. Its name and behavior establish a separate UI contribution, not a server formula. There is no new own-handle comparison in this UI stat case itself; upstream message-bus filtering and global callback order are outside the examined paths. Rust's existing own-handle/type 0 projection remains unchanged.

## Seven primary fields corroborated by the original UI and resources

The primary renderer and plus renderer use the same permutation. Signed16-bit reads, tooltip IDs and identical texts across archive/loose/converted sources corroborate this mapping:

| Wire offset | Primary-block offset | Label | UI status number | Tooltip code |
|---|---|---|---:|---:|
| +0x0b | +0x00 | Opaque first word; not displayed/subtracted here | — | — |
| +0x0d | +0x02 | Force / strength | 01 | 1410 |
| +0x0f | +0x04 | Vitalité / vitality | 05 | 1411 |
| +0x11 | +0x06 | Dextérité / dexterity | 06 | 1415 |
| +0x13 | +0x08 | Agilité / agility | 02 | 1412 |
| +0x15 | +0x0a | Intelligence | 03 | 1413 |
| +0x17 | +0x0c | Sagesse / mentality | 07 | 1414 |
| +0x19 | +0x0e | Chance / luck | 04 | 1416 |

`static_statusNN_value` and `static_statusNN_plus_value` identify the controls. Their display order differs from memory order; blindly numbering UI controls as consecutive packet fields would mis-map several attributes.

The seven group 99 tooltip values are byte-identical between the decoded archive payload and the loose CP1252 resource, then equal as Unicode to the converted server-side String.dbc. This does not depend on assuming which original resource wins at runtime. Their descriptive coefficients are tooltips, not proof that the client calculates server combat or job growth from them.

| Resource | Size | SHA-256 |
|---|---:|---|
| Archived db_string(windows-1252).rdb, decoded | 6,910,604 | `41676ad898f41ef232517093abd45442961059ac729ac615d7bbb3709110292e` |
| Loose Resource/db_string(windows-1252).rdb | 14,803,699 | `563b1dcc7e08c0875eb341c8db89b3c95a9619b98081df146af1ac2548a9a5a4` |
| Converted client-data/server-data/dbc/String.dbc | 9,441,340 | `cdad1dc4c1ddee59ebc2b94217a3fc471d338fe2ce08df97c843c199ac12b967` |

The archive resource is the data.008 slice at **821035141**, then XOR-decoded with the original 256-byte key at SFrame VA **0x00dfb1a8** (file+0x9f97a8), phase restarted at0 per resource. The ciphertext hash is `9014bb9bda638af488ce9985fc4d70fec81cd7d1faa8fde584697b02d03e2f38`, key hash `2e379ac7b5317f094ad5b003d9f6a1ea79ec95bb05e06bfe7e6199489a1dd302`. An independent verifier reads that key directly from the SHA-bound PE, decodes this exact resource and checks the hashes/selected values. The archive check is bounded to selected code/group/length anchors, not a claimed full parse of its older String format; the loose parser consumes all 150,632 records exactly.

The same verification corrects one historical statement in [the corpse-skill report](SKILL_CORPSE_VALIDATION.md): converted tooltip 40003212 is valid UTF8, has **zero U+FFFD**, and retains `ciblé` at file offset **8253297** as bytes **63 69 62 6c c3 a9**. The earlier replacement glyph came from Python output encoding, not the original or converted files. The corpse targeting conclusion is unchanged.

## Native Evidence and limits

| Function / inclusive analyzed body | Bytes | Role | Evidence ID |
|---|---:|---|---|
| 0x7056e0–0x705951 | 626 | Own stat-bank writer | `ev_c218ba00ef0f424c4ca74953b41f716f7e2620fe82a99bf4aa472c08ef939f18` |
| 0x7116d0–0x711981 | 690 | Classified-entity stat-bank writer | `ev_c999f80fad8f2ab5eb1c8924b65a4856b2e921f3c63d17691f7498a5c4fbc7e6` |
| 0x95db50–0x95db7a | 43 | Actor handle/type gate | `ev_90d824001ebcfe1d84cbb69d631f43235822e3b49fc03aa5198d07d5729a51a4` |
| 0x940f90–0x94148d | 1278 | Creature-data message receiver | `ev_d60db15b693823b12aa5aabbef9625e98c1949aa4e507b17e9b84fd4232081cc` |
| 0x89d9b0–0x89f02b | 5756 | Primary derived-stat display | `ev_123d66412e1f03dfd14417cc7ed2ad8cb2a27987c0cfd3a694114a66f9611b7f` |
| 0x89b7e0–0x89c2ab | 2764 | Seven primary-attribute display | `ev_7473b0829fc541429e384401828478f30489f08f95d67d578d1f85e337b40771` |
| 0x89a390–0x89b7c7 | 5176 | Primary plus-value display | `ev_ab2b4c4c98bdad63d590fdbd1525d4be3e1172e567c974942da418cbeca83039` |
| 0x8a1dd0–0x8a22db | 1292 | Character UI message dispatch | `ev_0c7097265f16bc7a09c0d3f3f17e1b34cc3261cd3e00af9dbe41055da9d6ee1e` |
| 0x89c2c0–0x89d99c | 5853 | Derived plus-value display | `ev_6e0670d85763c57df0aef8b204ae8a4de8d695537fa890d9cc789324e5ab05a9` |
| 0x8a1d30–0x8a1d36 | 7 | UI mode-byte getter | `ev_3cfd8199a45a65e80dd1d832beb282b912162f8c06beeac5a98a9a30e231812e` |
| 0x8a1c10–0x8a1c16 | 7 | UI selected-handle getter | `ev_849e554ef2e3225eef34ad284e75a062591ffc3ec4f63338b8dfbbe1e59ee72b` |

All records use analysis-profile digest `b9d116ea480a02cc8773b19372074a3caf77785cbc080b1ac7133b5110e40b17`. Complete Ghidra-analyzed body ranges, instructions, pseudocode and provider warnings are retained. The high-p-code graphs for **0x89d9b0,0x89b7e0,0x89a390,0x89c2c0** are **truncated**, each with an omitted-operation lower bound of1; their known omitted-edge counts are **0,0,2,1** respectively. The other seven value-flow graphs report untruncated. This is not a claim of complete value-flow proof or native source recovery. Indirect calls, alias semantics, terminal-flow flags and absent original runtime observations remain explicit limits.

The same installed public REA launcher was used over stdio with a bounded 900-second request/initialization deadline, avoiding the already diagnosed integrated 300-second initialization limit. The initial query took **506.03 seconds**, including default Ghidra analysis. The nine initial and two adaptively selected getter queries reused that same Program; no duplicate original import was required for the follow-ups. The closed snapshot retains **11 primitive entries /11 Evidence records**, **41,909,502 bytes**. No new integrated-timeout attempt or original process execution occurred in this pass.

An independent original-PE verifier checks **54 instruction/vtable anchors**, plus the seven cross-source tooltip matches and corrected UTF8 value. All **nine original preservation checks** pass. Original SFrame/archive index, installed client/extractor/world images, addon/WTF and primary service identities are unchanged. The previous runtime counts in [character-stat validation](CHARACTER_STATS_VALIDATION.md) remain historical; no new gameplay test or Rust behavior change is claimed here.

Raw dossiers, snapshots, resource/byte checks and finding ledgers stay ignored under `client-data/audit/2026-10-31-sframe-stat-banks-11a61dfb/`. Only documentation is published to both branches. Current package/game versions remain visible in both READMEs and VERSIONS.md.

## Remaining work

The five scoped storage/actor/UI/attribute questions are answered for the inspected paths. Server contribution sources/coefficients, valid type 0/type 1 send sequencing, UI mode lifecycle and original runtime interoperability remain unresolved. A future implementation should retain authoritative totals and secondary contributions independently, define ordering/reset behavior from verified server evidence, and preserve malformed/foreign/type guards. Do not copy the native cache's order-sensitive subtraction into the server's combat formulas or treat descriptive tooltip coefficients as recovered server rules. Job promotion/history and actual equipment/state/job bonuses remain separate investigations.

## Follow-up: UI lifecycle and local message routing, 2026-11-01

[SFRAME_DISPATCH_RE.md](SFRAME_DISPATCH_RE.md) follows 23 additional selected functions through construction/reset, Basic/Added tab events, a local vector, two priority deque banks, kind-keyed callbacks and selected tracking/reinsertion/cleanup methods. Alternate-view refresh clears contribution displays and uses a dash for EXP/JP. The direct nonzero alternate-mode setter remains unidentified within the bounded method region. A callback prepass precedes the explicit stat branch, but its registered recipients and complete UI/world routing remain unresolved; no blanket UI-after-bank, global FIFO or exactly-once claim follows. The original server’s legal type0/type1 sequence and bonus coefficients remain open. The earlier scope/limits above remain historical and are not restated as runtime evidence.
