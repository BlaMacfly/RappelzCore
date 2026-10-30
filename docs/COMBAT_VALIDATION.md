# Combat and XP level-up: two native clients

Verified overnight **2026-10-30 → 2026-10-31**, Europe/Paris. Two separate RappelzRust processes connected simultaneously to the real local auth/world services. Both reports and the independent cross-client comparison pass. This extends [LIVE_SERVER_VALIDATION.md](LIVE_SERVER_VALIDATION.md), which covered GM level changes rather than an XP-driven kill.

## Fixture and data preservation

Before adding any combat-test data, a consistent logical dump of all three existing databases was taken: **17,219,602 bytes**, under ignored `client-data/validation/2026-10-30-combat-bfbcdfd4/logical-before.sql`. The earlier verified cold backup remains available. Existing services stayed bound to `127.0.0.1` (MariaDB 3310, auth 4615, world 4514).

Two **new** accounts and characters were used, both with GM level **0** during combat:

| Role | Character | GUID | Initial state |
|---|---|---:|---|
| Fighter | LuaHitBFBC | 4 | Level 2, XP 799, JP 0, HP 120, MP 60, position 8450/7227. |
| Witness | LuaSeeBFBC | 5 | Level 1, XP/JP 0, HP 100, MP 50, nearby at 8420/7240. |

The new fighter's XP was deliberately seeded **offline** to one point below the existing provisional level-2 threshold (800). The runner never sends a GM level/XP command; the tested level increase must come from a real kill and the server reward path. Preparation/reset SQL was restricted by new GUID, account, name and offline status. No existing character was used or reset.

Both native processes use separate temporary WTF/addon directories, with the existing MPQs opened through directory junctions. This avoids concurrent writes to the user's SavedVariables. The installed user's WTF backup was retained. The existing world spawn, Tortue de Ullr (Monster.dbc ID 711), was reused; no new world spawn, vendor, template or content import was written. Its supplied data is HP 105 / MP 53 / EXP 5 / JP 3 / gold 0.

The final `preservation.json` passes: all original account/access/realm-character records and character/settings/inventory/item records for IDs/GUIDs 1–3 match the pre-test dump. All existing world data INSERT statements also match. Aria, Brann **and the earlier LuaProbe** remain unchanged. New test accounts/characters are retained, not deleted. Operational realm/uptime metadata is outside this equality claim.

## Real gameplay result

The final pair observed **12 identical attack records**: seven fighter hits and five monster retaliations. Every shared damage HP/MP broadcast matches between clients. Damage sequences agree with each previous HP value, including the final hit reducing the monster to zero. The fighter takes genuine damage before the reward heals it through the existing level-up behavior.

The kill awards **5 EXP and 3 JP**. The fighter moves from level 2 / XP 799 to **level 3 / XP 4 / JP 3**, with **HP 140/140 and MP 70/70**. The same level and vitals arrive at the witness and appear through its Lua target API/UI. The witness remains level 1 / XP 0 / JP 0 / HP 100 / MP 50; fighter-only XP packets are not broadcast to it.

The witness additionally queries the fighter during damage and after level-up, checking fresh ENTER responses against live HP/MP and the final state. It sees the fighter's LEAVE, clears its selected target, then returns to the lobby. Both clients emit PLAYER_LOGOUT and persist offline character rows.

## Defects found and corrected

1. **Target mana discarded in the Rust client.** Initial ENTER/HPMP packets contained the correct MP/max MP, but entity-to-addon snapshots always returned nil for target mana. The first two-client baseline completed the kill but failed observer synchronization (wire: MP 70/70; target state: nil). Entities now retain known mana from player/NPC/monster ENTER and HPMP updates. A shared vitals setter/snapshot preserves genuine zero values and keeps unknown values nil. A regression test covers known, unknown and zero mana without requiring a graphics context.
2. **Fresh player ENTER used stale combat vitals.** After the mana correction, the witness query immediately reproduced ENTER HP 120 versus the most recent combat HPMP HP 104. `World::player_enter` now reads current HP/MP and maxima from the live player entity; the cached appearance record is not used as a vitals store. This applies to queries and newly visible players. A regression test verifies damaged and regenerated values as well as future visibility.

Both failing runs are retained as negative controls under `baseline/` and `after-mana/`. Final `fighter.json`, `witness.json` and `comparison.json` all report success. No synthetic packets, manually dispatched combat events, GM level command or extra XP reward were substituted for combat.

## Reproducible native runner

The same online validation host now supports `-validate-combat <plan.json>` as an alternative to `-validate-addons`. A plan contains:

```json
{"role":"fighter","peer":"LuaSeeBFBC","directory":"<fresh absolute attempt directory>"}
```

Use a second plan with role `witness` and the fighter's character name as peer. Launch two native clients with separate `-data` roots and explicit `-login`/`-enter`. The guard requires loopback and LUA*/Lua* test identities. Both plans must point to the same fresh attempt directory. The runner requires the fixture values above, an alive nearby monster with the corresponding supplied stats, and both characters visible. It uses existing gameplay commands for targeting, attacking, query and return-to-lobby. Local atomic barrier files coordinate the test; they do not supply game-state values.

The runner observes actual network events before production dispatch, then checks the production WorldState, Lua-rendered labels and logout. Packet traces are bounded and decoding/overflow errors fail validation. Mid-combat and post-level queries are distinct checks. The cross-client tool verifies all 18 conditions independently:

```powershell
python tools/testing/compare_combat_clients.py <attempt-directory>
python -m unittest discover -s tools/testing -p test_*.py -v
```

It rejects empty success reports and inconsistent damage/HP sequences. Raw report JSON, screenshots and logs remain local under `client-data/`; database dumps, credentials, original assets and executables are not publication inputs.

## Verification and boundary

- Combined development workspace: **81 Rust tests passed**, zero failed, one intentionally ignored model scan.
- **5 Python tests passed** for data-preservation and combat comparison tools.
- Final native pair: both process exit codes 0; comparison checks 18/18; seven fighter attacks/five retaliations; damage broadcasts identical; XP/JP gain, target vitals, fresh queries, private XP isolation and logout verified.
- Clippy completes with pre-existing style/large-enum warnings; no new combat validation warning was introduced.
- Evidence: `client-data/validation/2026-10-30-combat-bfbcdfd4/final/{fighter,witness,comparison}.json`, role PNGs/logs, and the run's preservation report/backups.
- Installed client SHA-256: `84518892E4A3D6257BAA37F8C36C729DE3F12E19B404D894BF97A753B779CBD1`; preceding executable and installed addon files were preserved before updating them.

The publication worktrees also pass independent workspace tests and optimized builds: `RappelzCore` **34 passed**, `RappelzRust` **74 passed / 1 ignored**. Shared tests run in both branches, so these counts are not additive unique coverage. Original-asset tests that require local game files return early in clean publication worktrees; the combined development run and the real two-client session used the supplied assets. Server/client Cargo membership and each branch's existing history/logo/license remain separate.

These tests validate the existing **provisional** combat/XP formulas and supplied monster data. They do not establish retail formula fidelity, group XP sharing, PvP, skills, pets, loot, party behavior, aggressive AI or original SFrame compatibility. Repeatable test fixture preparation must remain restricted to dedicated new/offline test records.
