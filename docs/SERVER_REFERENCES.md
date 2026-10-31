# Public server and format references

Checked on 2026-10-30. This is a source review, not an interoperability test. No external implementation or database content was copied into the Rust workspace during this review.

## Result and reference order

No exact compatible server has been established for the supplied NoLimits executable. The local binary observations in [the protocol investigation](re/network_protocol.md) remain the implementation authority. They describe a `source9.5.2` build lineage, wire version `201507080`, login packet 62 with a 280-byte body, and game-version packet 51 with a 20-byte body. A build-directory name or an Epic label alone does not establish a complete protocol profile.

For packet comparisons, `glandu2/rzu` is the most useful inspected reference because its definitions explicitly vary by Epic. For gameplay architecture, `NGemity/RZEmulator` has relevant systems and a 9.5.2 protocol target, but concrete layout discrepancies remain. The four repositories requested by the user serve different purposes; three are data tools rather than emulators.

## Requested repositories

| Repository and inspected revision | Observed scope | License evidence | Suitability |
|---|---|---|---|
| [rapmike1/Rappelz-Emulator](https://github.com/rapmike1/Rappelz-Emulator/tree/85792ca7efd230c646469e7488c7f0472eae8060) — `85792ca7efd230c646469e7488c7f0472eae8060` | README explicitly targets Epic 4. C++ emulator with separate `Mononoke` auth and `Chihiro` game services. README credits Skyfire and an unfinished C# emulator. | [Root LICENSE](https://github.com/rapmike1/Rappelz-Emulator/blob/85792ca7efd230c646469e7488c7f0472eae8060/LICENSE): GNU GPL version 3. | Historical architectural reference; not an exact modern-client match. README lists unfinished skills, mounts, sieges, guilds and monster roaming. |
| [rzLabs/Archimedes](https://github.com/rzLabs/Archimedes/tree/264930a4ebf4622a0db705ab8aae42771f9ee5ed) — `264930a4ebf4622a0db705ab8aae42771f9ee5ed` | Structure-driven RDB binary data engine; C#/.NET 5; project references MoonSharp. | [Root LICENSE](https://github.com/rzLabs/Archimedes/blob/264930a4ebf4622a0db705ab8aae42771f9ee5ed/LICENSE): GNU AGPL version 3. | RDB schema/tooling comparison. Its Lua dependency concerns data descriptions, not proof of a client addon framework. No server protocol compatibility established. |
| [rzLabs/DataCore](https://github.com/rzLabs/DataCore/tree/10b38380c1b3540a606d5a34f0534f6518f77436) — `10b38380c1b3540a606d5a34f0534f6518f77436` | .NET API for original `data.xxx` archive indexing, loading, saving and entry operations. README describes filename/hash, length, offset and archive number. | [Root LICENSE](https://github.com/rzLabs/DataCore/blob/10b38380c1b3540a606d5a34f0534f6518f77436/LICENSE): GNU GPL version 3. | Archive format cross-check; not a game server and not an MPQ implementation. Does not establish this client's entry count or key. |
| [rzLabs/Grimoire](https://github.com/rzLabs/Grimoire/tree/113a6626dcd32f6e38b5068bd06cd8cb09e5d726) — `113a6626dcd32f6e38b5068bd06cd8cb09e5d726` | Windows .NET 5 development tool for archives, RDBs, name hashes, SPR and XOR keys. Project references Archimedes and DataCore. | [Root LICENSE](https://github.com/rzLabs/Grimoire/blob/113a6626dcd32f6e38b5068bd06cd8cb09e5d726/LICENSE): GNU GPL version 3. | Format/tooling reference. It is not evidence of server gameplay, authentication or an addon renderer. |

All four repositories were publicly readable. Root license texts and GitHub metadata were checked at the revisions above. This establishes the published root declarations, not a complete copyright audit of every file, dependency or bundled dataset. The workspace currently declares `GPL-2.0-or-later`; do not assume copying GPLv3 or AGPLv3 code preserves that declaration unchanged. No such reuse was performed here.

## Additional relevant projects

### glandu2/rzu

Inspected revision: [`87c1e83bf84efe29bb6405e8e6da80349712f3fa`](https://github.com/glandu2/rzu/tree/87c1e83bf84efe29bb6405e8e6da80349712f3fa). The README describes auth, packet filtering and an incomplete game server. [PacketEpics.h](https://github.com/glandu2/rzu/blob/87c1e83bf84efe29bb6405e8e6da80349712f3fa/librzu/src/lib/Packet/PacketEpics.h) supplies explicit version constants, including 9.5.2 and later.

The recursive tracked-file tree contained no root or nested `LICENSE`/`COPYING` file, and the GitHub repository API returned no detected license. This bounded check does not prove that every source file lacks licensing, but reusable licensing/provenance is unresolved. Treat it as a protocol comparison reference; obtain clear licensing before copying code.

### NGemity/RZEmulator

Inspected revision: [`38ceb2c6065fabf6ff4ba71d52f955f362c6c839`](https://github.com/NGemity/RZEmulator/tree/38ceb2c6065fabf6ff4ba71d52f955f362c6c839). GitHub reports it archived on 2026-04-02. [README](https://github.com/NGemity/RZEmulator/blob/38ceb2c6065fabf6ff4ba71d52f955f362c6c839/README.md) credits Skyfire and TrinityCore and describes development as inactive; [LICENSE](https://github.com/NGemity/RZEmulator/blob/38ceb2c6065fabf6ff4ba71d52f955f362c6c839/LICENSE) is GPLv3. [PacketEpics.h](https://github.com/NGemity/RZEmulator/blob/38ceb2c6065fabf6ff4ba71d52f955f362c6c839/shared/Server/Packets/PacketEpics.h) sets the latest target to 9.5.2.

The tree includes `CalculateStat.cpp`, `Skills/`, `Entities/Summon/`, `Entities/NPC/`, `Quests/` and `Entities/Item/Inventory.*`, plus `Database/Auth.sql`, `Telecaster.sql` and `Arcadia.sql`. These establish areas for later study, not verified formulas, completeness or correctness. The SQL datasets' provenance was not audited and none were imported. Preserve the existing RappelzCore database design rather than replacing it from these files.

## Concrete protocol comparison

Local observations below refer to `docs/re/network_protocol.md` and current `shared/rz-proto`; they were not newly confirmed by a live original-client connection in this review.

| Area | Local NoLimits implementation/evidence | Public comparison | Consequence |
|---|---|---|---|
| Framing | Total size `u32 LE`, ID `u16 LE`, one checksum byte; seven-byte header | rzu [PacketBaseMessage.h](https://github.com/glandu2/rzu/blob/87c1e83bf84efe29bb6405e8e6da80349712f3fa/librzu/src/lib/Packet/PacketBaseMessage.h) has this layout and sums the first six bytes. | Useful agreement, not a complete protocol match. |
| Account authentication | ID 10010; `account[61]`, `password_size:u32`, encrypted password buffer 77 bytes, total 149 bytes for the documented country profile | rzu [TS_CA_ACCOUNT.h](https://github.com/glandu2/rzu/blob/87c1e83bf84efe29bb6405e8e6da80349712f3fa/librzu/src/packets/AuthClient/TS_CA_ACCOUNT.h) selects these sizes for applicable RSA/AES versions; regional additional information and later-version changes exist. | Match the actual country/profile; do not substitute legacy DES authentication. |
| Password protection | Ephemeral RSA-1024 public key; PKCS#1 v1.5 exchange of AES key/IV; AES-128-CBC | rzu [ClientAuthSession.cpp](https://github.com/glandu2/rzu/blob/87c1e83bf84efe29bb6405e8e6da80349712f3fa/librzu/src/lib/NetSession/ClientAuthSession.cpp), [RsaCipher.cpp](https://github.com/glandu2/rzu/blob/87c1e83bf84efe29bb6405e8e6da80349712f3fa/librzu/src/lib/Cipher/RsaCipher.cpp) and [AesPasswordCipher.cpp](https://github.com/glandu2/rzu/blob/87c1e83bf84efe29bb6405e8e6da80349712f3fa/librzu/src/lib/Cipher/AesPasswordCipher.cpp) implement that family. | Supports the protocol family, not the exact build. |
| Login ID/body | ID 62, `name[24]` plus 256 bytes, 287 total | rzu [TS_CS_LOGIN.h](https://github.com/glandu2/rzu/blob/87c1e83bf84efe29bb6405e8e6da80349712f3fa/librzu/src/packets/GameClient/TS_CS_LOGIN.h) agrees for 9.5.2 through pre-9.6.3. NGemity [TS_CS_LOGIN.h](https://github.com/NGemity/RZEmulator/blob/38ceb2c6065fabf6ff4ba71d52f955f362c6c839/shared/Server/Packets/GameClient/TS_CS_LOGIN.h) uses a 19-byte name plus four-byte field and 256 bytes: one byte shorter. | Same Epic name can still have incompatible layouts. |
| Move/login-result IDs | Move 63, result 64 | rzu [move definition](https://github.com/glandu2/rzu/blob/87c1e83bf84efe29bb6405e8e6da80349712f3fa/librzu/src/packets/GameClient/TS_CS_MOVE_REQUEST.h) and [login-result definition](https://github.com/glandu2/rzu/blob/87c1e83bf84efe29bb6405e8e6da80349712f3fa/librzu/src/packets/GameClient/TS_SC_LOGIN_RESULT.h) include these IDs in known version ranges. | The difference from the embedded legacy name table does not itself prove a private opcode modification. |
| Game-version packet | ID 51, 20-byte version string, 27 total | rzu [TS_CS_VERSION.h](https://github.com/glandu2/rzu/blob/87c1e83bf84efe29bb6405e8e6da80349712f3fa/librzu/src/packets/GameClient/TS_CS_VERSION.h) appends 256 bytes at 9.5.2, making 283 total. | Exact profile remains unresolved; preserve local evidence and verify the constructor/capture before changing lengths. |
| Stream cipher | Local KSA uses a cached previous `S[i]` value, key indexed by `j`, then drops 1013 bytes | rzu [RC4Cipher.cpp](https://github.com/glandu2/rzu/blob/87c1e83bf84efe29bb6405e8e6da80349712f3fa/librzu/src/lib/Cipher/RC4Cipher.cpp) uses `S[j]` in its modified KSA and also drops 1013 bytes. | Do not replace the local recurrence from a public reference without rechecking binary instructions or an authorized known-plaintext capture. |

RappelzCore's SRP6 code stores/checks an AzerothCore-style password verifier. Its Rappelz network login still decrypts a supplied password through RSA/AES and checks that verifier locally. It is not a WoW SRP challenge/proof exchange; reusing database organization does not imply a shared wire protocol.

## Remaining comparison work

Combat formulas, NPC script semantics, pet progression, skill effects, inventory slot rules and quest behavior have not been proven equal to any public project. Inspect each relevant function and compare against the supplied binary/data before implementing it. RDB tools help establish field shapes but cannot validate gameplay by themselves.

The next compatibility milestone is a reproducible packet fixture set for this executable: constructor bytes, full lengths, cipher vectors, encoding and country extensions. Keep version-string, build lineage and packet-layout evidence separate. Use the existing Rust bot and controlled local servers for integration tests; an original-client interoperability claim still requires an authorized actual client session or capture.

Search boundary: the four named GitHub repositories, glandu2/rzu, the archived NGemity emulator and primary GitHub source/API evidence. Public search also exposed alleged proprietary source releases; none were retrieved or treated as authorized implementation material. This is not an exhaustive survey of private repositories, unavailable projects or future forks.


## Skill-learning follow-up, 2026-10-31

Pinned primary sources were reviewed to separate learning costs from casting costs, without copying external implementation or SQL content. [SkillBase::GetNeedJobPoint](https://github.com/NGemity/RZEmulator/blob/38ceb2c6065fabf6ff4ba71d52f955f362c6c839/Chihiro/src/Skills/SkillBase.cpp#L96) reads the next-level learning array; GetCostJP concerns casting. [ObjectMgr](https://github.com/NGemity/RZEmulator/blob/38ceb2c6065fabf6ff4ba71d52f955f362c6c839/Chihiro/src/Globals/ObjectMgr.cpp#L1400) scales learning cost by the tree JP ratio with integer truncation. Mapping local Skill.dbc jp_01…jp_50 to that separate learning resource is a cross-format inference, checked against local row values, not proof of the original client's semantics.

[GameContent](https://github.com/NGemity/RZEmulator/blob/38ceb2c6065fabf6ff4ba71d52f955f362c6c839/Chihiro/src/Globals/GameContent.cpp#L239) applies job/player levels, skill caps and prerequisites; reference callers also consider previous jobs. The Rust slice intentionally handles current-job trees only, preserves duplicate tier alternatives, rejects ambiguous costs/unsupported conditions and leaves job advancement/inheritance unfinished.

The [rzu packet definition](https://github.com/glandu2/rzu/blob/87c1e83bf84efe29bb6405e8e6da80349712f3fa/librzu/src/packets/GameClient/TS_CS_LEARN_SKILL.h) and [NGemity definition](https://github.com/NGemity/RZEmulator/blob/38ceb2c6065fabf6ff4ba71d52f955f362c6c839/shared/Server/Packets/GameClient/TS_CS_LEARN_SKILL.h) corroborate the 14-byte request body for the applicable profile. The [reference handler](https://github.com/NGemity/RZEmulator/blob/38ceb2c6065fabf6ff4ba71d52f955f362c6c839/Chihiro/src/Network/GameNetwork/WorldSession.cpp#L906) computes the next level and leaves the requested-level comparison commented out. Rust requires the exact next level to prevent duplicate/stale double charges and restricts this slice to the player, without owned-summon/random skills. These are explicit local policies, not claims of identical reference behavior.

REA reported no open native session; existing constructor/decompiler records were reused, with no new native decompilation or original-client connection. [SKILL_LEARNING_VALIDATION.md](SKILL_LEARNING_VALIDATION.md) records data hashes, evidence boundaries, tests and remaining effects/progression work.


## Casting follow-up, 2026-10-31

The same pinned NGemity revision corroborates [the inverted active/passive byte](https://github.com/NGemity/RZEmulator/blob/38ceb2c6065fabf6ff4ba71d52f955f362c6c839/Chihiro/src/Skills/SkillBase.cpp#L132), [fixed MP recovery](https://github.com/NGemity/RZEmulator/blob/38ceb2c6065fabf6ff4ba71d52f955f362c6c839/Chihiro/src/Skills/SkillFunctor.h#L311), [cost/delay formulas](https://github.com/NGemity/RZEmulator/blob/38ceb2c6065fabf6ff4ba71d52f955f362c6c839/Chihiro/src/Skills/SkillBase.cpp#L79) and [SC_SKILL layout](https://github.com/NGemity/RZEmulator/blob/38ceb2c6065fabf6ff4ba71d52f955f362c6c839/shared/Server/Packets/GameClient/TS_SC_SKILL.h#L142). Costs/recharge occur at FIRE; nominal recovery differs from capped mana. The local subset avoids unimplemented MAtk/state/equipment modifiers.

Local 3212’s name mentions a corpse, yet its effect is 502, dispatched to generic MP recovery. Reference corpse absorption is a separate 511 path. A bounded review of Skill.cpp, Unit.cpp and SkillBase.cpp found no literal 3212/casting_type/casting_level rule; that absence does not prove original behavior. [SKILL_CASTING_VALIDATION.md](SKILL_CASTING_VALIDATION.md) preserves this unresolved difference, the compact-versus-padded wire distinction, native evidence and the intentionally local profile. No external code/data was copied, no new original-client capture/decompilation obtained, and no official corpse mechanic claimed.


## Archived corpse rule follow-up, 2026-10-31

Direct read-only comparison of supplied archives and Resource loose files found the missing distinction: archived3212 is effect511 with monster/corpse flags, while loose3212 changes15fields into effect502/self recovery. SkillStage's corpse flag and tooltip still match the archive. This corrects the previous bounded conclusion; the old search could not discover the archive profile because installed DBCs intentionally follow loose overrides.

[RappelzRDBTool SkillStage81](https://github.com/glandu2/RappelzRDBTool/blob/e1f261471ffec2ade1d966d06163c4477f92fb8c/Databases/SkillStage81Database.cpp#L11) corroborates distinct has_target/corpse_target fields. [NGemity CORPSE_ABSORB](https://github.com/NGemity/RZEmulator/blob/38ceb2c6065fabf6ff4ba71d52f955f362c6c839/Chihiro/src/Skills/Skill.cpp#L4595) requires a dead monster, removes it before recovery and emits HP then MP results to the beneficiary. These pinned primary behaviors guide the isolated Rust archive profile; no external implementation or SQL was copied. Target/range/cooldown limits, original-runtime uncertainty and the unsuccessful300sGhidra query are recorded in [SKILL_CORPSE_VALIDATION.md](SKILL_CORPSE_VALIDATION.md).
