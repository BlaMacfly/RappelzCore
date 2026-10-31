# Skill 3212: archived corpse rule and two native clients

Verified **2026-10-31**, Europe/Paris. The final native pair exits **0/0** and passes **48/48 independent comparison checks**. Skill 3212 now requires a selected, currently visible dead monster. FIRE consumes that death's corpse once, restores the caster and leaves the original respawn timer intact. A second caster competing for the same body is canceled without cost or cooldown. This supersedes the corpse-free local profile in [SKILL_CASTING_VALIDATION.md](SKILL_CASTING_VALIDATION.md).

## Why the previous data disagreed

Two different Skill resources ship in the supplied client. The converter's `collect_rdb` intentionally loads archives first, then loose `Resource/` overrides. Conversion was accurate: the installed MPQ's Skill.dbc and server Skill.dbc are identical. The **loose row itself** changes 3212 into effect 502, self MP recovery, while leaving its client stage marked as a corpse target.

| Evidence | Archived resource | Loose resource |
|---|---|---|
| Skill RDB | data.002 entry offset 707800449, 2,588,642 bytes, 3,130 rows | Resource/db_skill.rdb, 2,742 rows |
| Full RDB SHA-256 | `1219008a3c8418569ab9c5b33ab48facda625f65865fb511c8be086c02732309` | `6029fbd6cc897a82e6de73c56024e31fd320b46b65c9950015007dc43a073c51` |
| Row 3212, zero-based index / decoded file offset | 171 / 141549 (`0x228ed`) | 159 / 131625 (`0x20229`) |
| Row SHA-256, 827 bytes | `787b76d336f5f64e01deb2eaca374519a22e4bf21cff92366599cf9d30193016` | `2fa129b6f64d465e21dbcf36bd67d184bc35360a3f6129011a2d93956e000726` |

Exactly **15 of 177 fields differ**, archive → loose:

| Field | Archive | Loose |
|---|---:|---:|
| elemental | 6 | 5 |
| is_harmful / is_need_target / is_corpse | 1 / 1 / 1 | 0 / 0 / 0 |
| cast_range | 16 | 0 |
| delay_cooltime / per skill level | 60 / −2 | 25 / 0 |
| uf_self / uf_enemy | 0 / 1 | 1 / 0 |
| tf_avatar / tf_summon | 0 / 0 | 1 / 1 |
| effect_type | **511** | **502** |
| var3 / var4 / var6 | **200 / 120 / 60** | **2000 / 1500 / 0** |

The other 162 fields agree. Archived/loose SkillStage row 3212 is byte-identical, SHA-256 `1feedf85a5d6339f2c376743fa9dc59b547d2ecb8a6ad0765b18f6e84c5f6956`: **has_target=1, corpse_target=1**, stage type 402. Independent raw parsing finds the corpse byte at loose RDB offset `0x558c` and converted DBC offset `0x61e8`. [RappelzRDBTool's pinned SkillStage schema](https://github.com/glandu2/RappelzRDBTool/blob/e1f261471ffec2ade1d966d06163c4477f92fb8c/Databases/SkillStage81Database.cpp#L11) corroborates the separate fields. Stage 402 is an animation/skill-stage selector, not a spatial layer.

Archived String resource key **40003212** also describes draining the selected corpse; its phrase begins at decoded offset 3069549. The converted tooltip retains that meaning with a replacement character in one word. Thus the target rule is supported by Skill flags, SkillStage and the tooltip, rather than inferred from the localized name alone.

The supplied SFrame hash remains `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`. REA opened that exact image, but its first string query exceeded the **300-second transport deadline** during Ghidra auto-analysis; no successful fresh native query was obtained and the session was closed. Read-only inspection with existing objdump identified the SkillStage loader at `0x673a50`, its 160-byte row read at `0x673ba0` and stage-type access at `+0x14`. This confirms layout, not runtime corpse consumption. The original SFrame was not executed; its loose/archive runtime precedence and complete interoperability remain unverified.

## Rule and isolated profile

The pinned primary emulator separately dispatches [effect 511 to corpse absorption](https://github.com/NGemity/RZEmulator/blob/38ceb2c6065fabf6ff4ba71d52f955f362c6c839/Chihiro/src/Skills/Skill.cpp#L4595): dead monster target, removal before recovery, caster beneficiary unless an invocation selector applies, ADD_HP followed by ADD_MP. For enhancement zero, its variable slots give HP = var1 + level×var2 and MP = var3 + level×var4. The archived supported row therefore gives **HP 0, nominal MP 320, cost 5 MP, cast 145 centiseconds at level 1**. It is not the loose profile's 3500 MP. Enhancement, invocation redirection, state/equipment/restoration-ratio modifiers remain unsupported. Nominal amounts are distinct from capped final vitals.

The Rust profile uses checked data arithmetic for cooldown base + per-level coefficient: **60−2×1 = 58 seconds**, group 103. Its spatial bound is **16×12 = 192 world units**, without the public reference's unit-radius/movement tolerances. The [reference range conversion](https://github.com/NGemity/RZEmulator/blob/38ceb2c6065fabf6ff4ba71d52f955f362c6c839/Chihiro/src/Skills/SkillBase.cpp#L329) supports the unit scale; the exact original cooldown computation and geometric tolerances are not claimed established. Matching archived target/effect data plus the primary effect implementation establishes this implemented profile, not a complete official-server equivalence.

`rz-extractor corpse-skill` reads **only archived entries**, selects one unambiguous 3212 row from Skill and SkillStage, validates their compatibility, then writes two companion DBCs under **CorpseSkill/**. Existing files are retained; identical output is idempotent, conflicting output is refused before either file is written. A real CLI conflict test confirms existing bytes remain unchanged and no second file is added. Full client conversion includes the companion profile when supported; DBC extraction transfers it separately.

Server location: `DataDir/dbc/CorpseSkill/{Skill.dbc,SkillStage.dbc}`. Client location: `Data/CorpseSkill/{Skill.dbc,SkillStage.dbc}`. The normal loose Skill.dbc, SkillStage.dbc, String.dbc, original Resource files and installed MPQs are **not replaced**. Missing/incompatible companion data disables this cast; it never silently falls back to corpse-free 3212. Learning remains on existing tables; its price/prerequisite data is unchanged by the 15-field difference.

Companion SHA-256: Skill `26f763b2606d12f762eb22e1ce1d2c1b293c206aa5c259195be727ef5a8bd33a`; SkillStage `007c66722d94e1259482b6e1515b30448c6d43177d931f41db6bf8fe9e51b455`. These locally derived files stay ignored and are not published.

## Authoritative lifecycle and presentation

The server checks owner identity, learned level, current life/MP/layer, announced visibility, monster type, zero HP, unconsumed body, range and cooldown. It binds a pending cast to the monster's **death generation**, preventing reuse of a respawned handle. FIRE rechecks all conditions under the existing world lock. Pending fires are ordered by deadline/handle; only one can consume the body. Invalidated competitors receive CANCEL with no debit/recharge. Expiration, movement, death/layer change and disconnect cancel normally. Queries cannot reveal expired/consumed corpses.

Consumed monsters stay in the spawn scheduler, with their original death time. Visibility sends one LEAVE, and normal respawn restores the same handle alive. No spawn row is deleted or modified; consumption grants no XP/JP/item reward. Real kills continue to award once.

CS_SKILL's selected target is the corpse. SC_SKILL's header also names it, while the two FIRE recovery results address the **caster**: ADD_HP amount 0, then ADD_MP amount 320. The compact frame is **83 bytes**; checked decoding also handles the public padded union. This separation replaces the old self-target assumption. Matching pending phases survive a corpse LEAVE if it arrives before recovery; unsolicited/mismatched phases remain ignored. Confirmed caster vitals update before Lua's existing RAPPELZ_SKILL_EVENT callback. No new Lua action API or opcode was introduced.

The K window requires a selected in-range dead monster and shows the archived cost/cast/recharge. Monster corpses remain rendered/selectable until server LEAVE. Physical controls are suppressed only in the opt-in automated corpse scenario, after a live attempt received a manual movement that correctly canceled its caster. Normal play retains its controls. The driver uses production selection/attack/casting helpers and real networking/rendering/Lua; it does not manufacture gameplay events.

## Real session and preservation

A consistent three-database backup precedes fixtures: **18,905,951 bytes**, SHA-256 `07ce37bd3a870cc2114f2ad6d91c9882f845d434252514b395907b803c38dcf4`. Pre-cycle records through account/character **19**, items through **4**, world INSERTs and installed WTF hashes are protected. New ordinary GM-0 fixtures **20 / LuaCrp050A** and **21 / LuaCrw050A** have seeded level 10, job 302/job level 35, skills 63203:1/3212:1, XP/JP/money zero and MP 20. Their setup is not claimed as earned progression. Only their exact new offline records may be reset between attempts.

1. Six owner/two witness self/player/living/unknown/NPC/foreign-owner probes are refused. The old server negative control accepted self targeting; both native processes exited 1 with that failure retained.
2. The owner kills the existing monster normally, begins a late incantation after about 2.05 seconds, then receives CANCEL when the three-second corpse expires. No cost, cooldown or recovery occurs; the witness's hidden query/cast is refused.
3. The same monster re-enters at its existing **15-second respawn**, HP **105/105**, MP **53/53**. After a second real kill, both clients start casts on the body. Only the owner's FIRE succeeds; the witness is canceled without cost/recharge. Both see immediate corpse LEAVE, and further queries/casts cannot recover it.
4. The owner returns to the lobby and loads under a fresh handle with its private recharge preserved. After the second original 15-second respawn, both refuse the living target. The witness kills it normally. The owner's fresh-corpse attempt is refused on cooldown; the witness successfully absorbs this new death generation.
5. Both receive exactly **eight matching phases**, with **two fires total**. Owner requests 6/7 are accepted among 11; witness requests 3/6 among 7. Nominal restoration is 320, actual MP caps at 140, and Lua callbacks read those confirmed values. Three kills give owner **XP 10 / JP 6**, witness **XP 5 / JP 3**; casting adds none. Both log out and the database matches their actual final vitals/positions/learned state.

The archived-rule negative control is in `negative-self-profile/`; `initial/` retains the manually interrupted attempt; **verified/** contains the final **48/48** pair and visually inspected screenshots. The native scenario also runs with the corrected terrain described in [TERRAIN_RENDERING_VALIDATION.md](TERRAIN_RENDERING_VALIDATION.md).

Preservation passes **9/9 comparisons**. All **19 pre-existing characters**, account/access/realm-character records, protected inventories/settings/items, world content and installed preferences compare unchanged. Original assets/MPQs remain intact; companion files are additive. Operational realm/uptime metadata is outside this equality claim. Worldserver was replaced gracefully once with no players; authentication/database services remain active.

## Verification and reproduction

Combined workspace: **140 Rust tests passed**, one intentionally ignored model scan, including **22 actual Lua integration tests**. Python tools pass **34 tests**, including an independent complete corpse fixture, empty/failure guards, mutations of self casts/double consumption/privacy/cooldown/rewards/movement and UTF-8 CLI output. Optimized builds succeed; Clippy completes with style warnings. Unit tests cover hybrid/missing-stage rejection, wire target/beneficiary offsets, expired/out-of-range/cross-layer corpses, competing fires, death-generation reuse, unchanged respawn timing and production client state ordering.

Separate publication workspaces pass **Core 73 Rust tests**, **Client 108 Rust tests / one ignored**, and **34 Python tests on each branch**, followed by successful optimized builds. Shared tests overlap between those branch totals and must not be added to the combined count. Cargo membership/lockfiles, licenses, history and README logos are preserved. All six optimized executables target AMD64.

Validated client SHA-256: `0F6928115EFC771EAEC620EB85EBEB342E51C6D17DCA2748DA4DFDB7E562CEB0`. Runtime evidence, generated profiles, credentials/backups and screenshots remain ignored under `client-data/validation/2026-10-31-corpse-skill-050a8eb7/`.

```powershell
# Adds isolated profiles, preserves existing files; no database import:
rz-extractor corpse-skill --client "Rappelz NoLimits" --out client-data/server-data/dbc
rz-extractor corpse-skill --client "Rappelz NoLimits" --out client-data/RappelzRust/Data
# Fresh role/peer/directory plans and new loopback LUA* fixtures:
# -validate-corpse-skill <plan.json>
python tools/testing/compare_corpse_skill_clients.py <attempt-directory>
python -m unittest discover -s tools/testing -p test_*.py -v
cargo test --workspace --all-targets --locked --offline -- --test-threads=4
```
