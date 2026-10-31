# Skill learning: data, persistence and two native clients

Verified **2026-10-31**, Europe/Paris. The final `verified/` pair exits **0/0** and its independent comparison passes **40/40 checks**. Learning debits the actual next-level JP cost, enforces ownership/job/requirements, rejects duplicate or stale requests, persists before acknowledging success and reloads learned levels through the real character-lobby path. Casting and skill effects are separate unfinished systems.

## Data and evidence boundary

The typed loader reads the existing converted **Skill.dbc (2,742 rows / 177 fields)**, **SkillTree.dbc (9,275 rows / 16 fields)** and **Job.dbc (42 rows / 15 fields)**. Their SHA-256 digests are respectively:

```text
46804b3895bef4db2ef2d8db3317602545a375771aba3dabe52b03d21f28db2e
c1199982d9fb504cd6bee30fb23d832dec1bbbef226b03d6a49adafe85f61b81
c52c248827520db9f69cd739adacb5308eadce724a1b1e8a3791401b9632509c
```

An independent local decoder checked WDBC dimensions/fields and inspected the actual data. Job IDs **100/200/300** map through `skill_tree_id` to trees **10/1/19**; looking up a tree directly by job ID is incorrect. There are **1,435 repeated (tree, skill) pairs**, including overlapping player-level tiers and exact duplicate rows. The loader preserves every distinct rule and evaluates eligible alternatives instead of using `NamedDbc::by_id`'s last-row index for these tables.

Learning uses the **`jp_01`–`jp_50`** array for the requested next base level, multiplied by the applicable tree's `jp_ratio`, with integer truncation. It does not use **casting** `cost_jp`. These interpretations are corroborated by the pinned primary references linked in [SERVER_REFERENCES.md](SERVER_REFERENCES.md): [SkillBase.cpp](https://github.com/NGemity/RZEmulator/blob/38ceb2c6065fabf6ff4ba71d52f955f362c6c839/Chihiro/src/Skills/SkillBase.cpp#L96), [ObjectMgr.cpp](https://github.com/NGemity/RZEmulator/blob/38ceb2c6065fabf6ff4ba71d52f955f362c6c839/Chihiro/src/Globals/ObjectMgr.cpp#L1400) and [GameContent.cpp](https://github.com/NGemity/RZEmulator/blob/38ceb2c6065fabf6ff4ba71d52f955f362c6c839/Chihiro/src/Globals/GameContent.cpp#L239). Mapping the reference's separate SkillJPResource to the local array remains an explicit cross-format inference, consistent with the inspected data; it is not an original-client execution observation.

The reverse-engineering skill was used to reuse retained local schema/protocol evidence and review primary references. REA reported no open native session; no new SFrame decompilation/capture or original-client interoperability is claimed. No external implementation or dataset was copied.

## Learning policy and limits

The current implementation supports **the current job's tree, base levels 1–50, ordinary player learning, group 0 rules and enhancement ranges containing 0**. It checks character/job level, all declared prerequisite skills, next-level range and available JP. Negative/nonfinite/overflowing costs and conflicting eligible prices are refused. Zero-cost rules preserve known zero. Native costs/balances use checked/wide arithmetic; JP reward overflow saturates instead of wrapping negative. Lua 5.1 number precision beyond 2^53 remains a documented runtime limitation.

Minimum skill-level ranges and explicit next-level requests are enforced as native validation policies. Unsupported mastery/group/enhancement rules, random-pet learning, inherited previous-job trees, job-level advancement, skill effects/passive stat application and casting/cooldowns remain future work. Existing characters with an absent/unmapped job receive an unavailable catalog; no job is silently assigned or migrated. BasicStat/Exp are absent from the currently converted DBC set, so the older provisional combat/XP formulas remain unchanged.

## Protocol and durable state

Added **TS_CS_LEARN_SKILL (402)** with a 14-byte body / 21-byte framed packet: target u32, skill ID i32, random-base skill ID u32, requested level i16. Local constructor evidence establishes the size and primitive slots; field semantics/signedness are corroborated by pinned [rzu](https://github.com/glandu2/rzu/blob/87c1e83bf84efe29bb6405e8e6da80349712f3fa/librzu/src/packets/GameClient/TS_CS_LEARN_SKILL.h) and [NGemity](https://github.com/NGemity/RZEmulator/blob/38ceb2c6065fabf6ff4ba71d52f955f362c6c839/shared/Server/Packets/GameClient/TS_CS_LEARN_SKILL.h) definitions. They remain inferred for this exact SFrame build. The server requires the live owning player, random-base ID zero and the exact next level. A repeated old level cannot buy another level accidentally.

Successful learning sends the existing private EXP_UPDATE and a complete private **SKILL_LIST (403)**, followed by the result/notice. Its learned base/current levels are retained by the production client adapter. Foreign-target, duplicate, unsupported-modification and invalid-base-level lists are ignored by that adapter.

The learning catalog is an explicit **opt-in Rust extension version 1**: **62010** (hello) / **62011** (catalog). It contains owning handle, job/job level/player level, JP, availability and up to 512 sorted offers with actual cost, known level, next/max level, requirements and refusal reason. Strict decoding checks version, bounds, identity/order, prices/readiness, flags, truncation and trailing bytes. Only the requesting session receives it; only changed snapshots are sent. This extension is not a recovered retail packet. An older server leaves catalog information unavailable.

Learned levels use bounded canonical `learned_skill_<id>=<level>` keys in the existing `character_settings`, `source='rappelz'`. JP and the new level are committed together by the existing `characters::save` SQL transaction; no schema migration/import is required. A candidate snapshot is copied from live state, written durably, then assigned to session memory and published. An unreachable-database test proves failed learning leaves JP, learned fields and the previous session snapshot unchanged and emits no skill/XP success data. All unrelated extras remain intact. This does not claim complete duplicate-session/crash recovery for the wider server.

## Client and Lua

**K** opens the learning window: actual localized names/icons, levels, costs, requirements, refusal reasons, pagination and an **Apprendre** button. It sends the same validated learning request used by the native scenario and changes no local balance/learned level optimistically. The panel explicitly states that casting/effects are still to come. Expected learning refusals no longer open a fatal connection-error dialog. First-run screenshots exposed raw `<size:9>` name markup; the final window resolves it through the existing caption parser. The panel does not overlap the group window. Keyboard K and every new mouse button remain manual input checks; the automated run uses the production command helper and real network/state/rendering paths.

Read-only Rappelz Lua extensions: **GetJobPoints()**, **GetNumKnownSkills()**, **GetSkillLevel(id)**. Missing state returns nil; a confirmed empty list/count or zero JP remains zero; a known list returns zero for an unlearned positive ID. **SKILL_LINES_CHANGED** follows accepted list discovery/change; **RAPPELZ_JP_CHANGED(balance)** follows accepted private JP discovery/change. Initial discovery is not an earned reward, and duplicate snapshots/UI reload do not repeat changes. RappelzDebug displays/logs actual skill count and JP; no learning/casting Lua action API was exposed.

## Native sequence and preserved data

A consistent three-database backup was taken before fixtures: **18,902,503 bytes**, SHA-256 `01d4f694e8b59d551c386976f68f6f25bf9042bcebaa8aad7078d3dcad96d68c`. New ordinary GM-0 account/character **16 / LuaSklFA59** uses job 200, job level 3 and **18 seeded test JP**; **17 / LuaSkwFA59** uses job 300, job level 1 and zero JP. Both start level 1, HP 100, MP 50, XP/money 0, layer 0 at 8440/7227 and 8420/7240. Creation uses normal server/bot flows; only these exact new offline identities are configured/reset. No earned-JP/job-level progression is claimed for fixture preparation.

1. Both receive confirmed empty learned lists and their own catalog/JP. The owner probes foreign ownership, random-pet fields, negative/skipped levels, wrong-job skill, insufficient job level, missing prerequisite and unknown skill. All eight requests are denied without cost. The zero-JP witness's own purchase is also denied.
2. The owner learns **1004 level 1 for 4 JP** (18→14). Two repeated requests for that same base level are refused with no additional debit.
3. Skill 1003 is now unlocked: **level 1 costs 4 JP** (14→10), then **level 2 costs 10 JP** (10→0). Its next level costs 62 JP and is refused; skill 1004 above its tree cap is also refused. Only requests 8, 11 and 12 (zero-based) succeed among the owner's 15 requests.
4. The witness queries the owner and tries a foreign-target learning request. It retains its own empty list/catalog and zero JP; private owner skill/JP packets never leak.
5. The owner logs out to the lobby and re-enters through a fresh database character load/new handle. Learned levels **1003:2 / 1004:1** and **JP 0** persist. The witness queries the new handle without receiving private skill data. Both log out cleanly.

The owner receives five list/JP discoveries/changes across two logins and five JP snapshots **[18,14,10,0,0]**; the witness receives one of each with zero JP. No combat, movement, level/reward, item, layer or vital changes occur. The final row checks confirm the two new characters offline with exact positions/stats/jobs, JP 0 and the intended learned fields. Re-entry is a character-lobby cycle on the existing connection, not a full auth/client-process restart.

Preservation passes **9/9 checks**: pre-cycle accounts/access/realm-character records through ID 15, character/settings/inventory through GUID 15, items through GUID 4, all world INSERTs and installed WTF hashes. Operational realm/uptime metadata is outside this equality claim. No fixture is deleted, no original assets are changed, and backups/earlier evidence remain.

## Verification and reproduction

Combined workspace: **124 Rust tests passed**, one intentionally ignored model scan, including **21 real Lua integration tests**; **26 Python tests passed**. Coverage includes tier alternatives, distinct learning/casting costs, price/range/requirement guards, failed SQL commit, wide-JP reward saturation, strict catalog/request parsing, production adapter/privacy/no optimistic charge, event deduplication, nested trace capacity and independent comparison samples/mutations/UTF-8 CLI. Optimized server/client/extractor builds and Clippy pass with only the existing older warnings.

Separate publication workspaces also pass: **Core 60 Rust tests**, **Client 100 Rust tests / one ignored**, and **26 Python tests on each branch**, followed by successful optimized builds for both workspaces. Shared tests occur in both branch totals; those totals should not be added to the combined count. Branch Cargo membership, license, history and README logos are preserved; no local database/assets/binaries/credentials are included.

The installed client SHA-256 is `4BC98CD940D0F53BB4E5F54FF4AF3431E73CCDC25ABC91C2C444D96502ED999F`. Native reports/screenshots, local credentials, SQL backups/comparisons, data inspection and build/test logs stay under ignored `client-data/validation/2026-10-31-skills-fa59510d/`. `initial/` is the first passing logic run with visible markup; `verified/` is the final-code passing pair with corrected labels. None of these local artifacts is published.

```powershell
# Fresh owner/witness plans with role, raw Lua* peer name and shared directory:
# -validate-skills <plan.json>, separate native roots and loopback LUA* accounts.
python tools/testing/compare_skill_clients.py <attempt-directory>
python -m unittest discover -s tools/testing -p test_*.py -v
cargo test --workspace --all-targets --locked --offline -- --test-threads=4
```
