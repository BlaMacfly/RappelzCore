# Database architecture

Audit date: 2026-10-30. This describes the checked-in SQL and Rust queries. Existing databases and SQL files were not modified during the audit.

Follow-up: [LIVE_SERVER_VALIDATION.md](LIVE_SERVER_VALIDATION.md) supersedes the original offline audit boundary. Existing services were started after verified cold/logical backups; a dedicated account/character was added for validation. Full selected original account/character records and all existing world data INSERTs compare unchanged. No schema/bootstrap/content import was applied.

## Three logical databases

| Logical role | Default physical name | Configuration | Current responsibilities |
|---|---|---|---|
| auth | `acore_auth` | `LoginDatabaseInfo` | Accounts, password verifiers, bans, GM access, realm list/counts, one-time auth-to-world key, realm status and uptime. |
| characters | `acore_characters` | `CharacterDatabaseInfo` | Character identity/position/stats, Rappelz extras in settings, inventory and equipped items. |
| world | `acore_world` | `WorldDatabaseInfo` | Imported item/creature/quest/location templates, NPC shops, NPC/monster placements. |

The three names remain configurable; the logical separation is already present. `shared/rz-database` uses `sqlx` MySQL pools. The auth service needs the login database; the world service opens all three. Configuration follows `host;port;user;password;database`, with `AC_*` environment overrides. See `shared/rz-common/src/config.rs` and the distributed configuration files. Connection credentials are deliberately omitted here.

## SQL inventory and provenance

Fresh file/statement counts under `sql/base`:

| Directory | `.sql` files | `CREATE TABLE` | `DROP TABLE` | `INSERT INTO` statements |
|---|---:|---:|---:|---:|
| `db_auth` | 22 | 22 | 22 | 8 |
| `db_characters` | 108 | 108 | 108 | 8 |
| `db_world` | 309 | 309 | 309 | 0 |
| Total | 439 | 439 | 439 | 16 |

There are two additional SQL files under `sql/`: `create/create_mysql.sql`, and `custom/db_auth/0001_rappelzcore_realmlist.sql`. The latter updates the existing realm 1 entry, including world port 4514; it does not add a table. Its `gamebuild = 10360` is configuration data and must not be used as proof of this executable's actual wire version.

The README identifies upstream AzerothCore commit `3179ea3`. GitHub resolves it to [`3179ea3cc7483b885bd49e608728d6f439d47bdf`](https://github.com/azerothcore/azerothcore-wotlk/commit/3179ea3cc7483b885bd49e608728d6f439d47bdf), committed on 2026-10-25. A fresh sample comparison found exact text equality after newline normalization for upstream/local `db_auth/account.sql` and `db_characters/characters.sql`. `db_world/item_template.sql` matched after removing upstream INSERT statements using the importer's transformation. This corroborates the stated origin for three samples; all 439 files were not compared against upstream.

`tools/sql/import_acore_schema.js` copies auth/characters dumps, strips `updates` data, and strips world INSERTs. `tools/sql/import_all.sh` loads base dumps followed by custom SQL. These are initialization/import tools: base dumps contain DROP TABLE statements and must not be treated as incremental upgrades to a populated database.

## Authentication storage versus wire protocol

`shared/rz-database/src/srp6.rs` implements a 32-byte salted verifier using generator 7, the AzerothCore modulus, SHA-1, little-endian integer conversion and normalized credentials. Tests cover registration/checking, bad passwords and case normalization. They are not an independent cross-implementation interoperability test.

The Rappelz client does **not** perform a WoW SRP challenge/proof exchange. `rz-proto/secure.rs` and `authserver` exchange an RSA-protected AES key/IV, decrypt the submitted password, and recompute/check the stored verifier. Realm selection writes a 40-byte `account.session_key` whose first eight bytes hold the Rappelz one-time key. `worldserver/session.rs` checks those eight bytes and clears the stored key. See [NETWORK_PROTOCOL.md](NETWORK_PROTOCOL.md).

## Current gameplay mappings

| Rappelz data | Existing storage and convention |
|---|---|
| Character race, base class, sex, level, XP, rupees, HP/MP, position | `characters`; MP uses `power1`, sex uses the local gender conversion. |
| Exact job, job level, JP, layer, appearance/model fields | `character_settings`, `source = 'rappelz'`, a `key=value;...` serialization. |
| Items and ownership | `item_instance` joined through `character_inventory`. |
| Equipment versus bag | Local convention: `bag = 0`, slots 0–27 are Rappelz worn slots; slots 28 and above are bag positions. This is not WoW inventory behavior. |
| Monsters and NPCs | `creature_template`; NPC entries are `1_000_000 + NpcResource id` to avoid overlapping Rappelz IDs. Placements use `creature`, currently map 0. |
| NPC shops | `npc_vendor`, using the offset NPC entry. |
| Quests and teleport locations | `quest_template` and `game_tele` generated from selected DBC fields. |

The [layer validation](LAYER_VALIDATION.md) exposed that saving a character omitted its live layer. `Session::save` now copies it into the existing Rappelz settings `layer` key and also retains live z in `position_z`. Actual logout to the lobby and character re-entry confirmed layer 1 persisted. This required no schema migration or content import; all pre-cycle character/settings records and world data compare unchanged.

[Player groups](PARTY_VALIDATION.md) are ephemeral world state; they add no database tables or settings. Logout/disconnect/restart clears membership and pending invitations. Shared rewards and persistent groups remain future work. Ground-drop requests now retain inventory items while world loot is absent; intentional erasure remains separate. The group cycle's selected pre-existing records through ID/GUID 13 and all world INSERTs compare unchanged.

`rz-dbimport` reads `DataDir/dbc` and writes SQL for item templates, monster templates, NPC templates/placements, quests and world locations. Its generated SQL contains DELETE statements for those imported datasets; `--apply` is a data replacement/import operation, not a schema migration. Do not apply it to existing production data without reviewing the output and import behavior. This audit did not run it.

Character names currently inherit the 12-character schema limit. Rust storage also uses narrow numeric types, including `u8` level and `u32` rupees/XP; faithful late-game Rappelz limits need explicit evaluation. The settings blob is suitable for the existing small extras, but full pet, skill, quest-progress, party/guild, trade/economy and instance persistence are not established by the mere presence of WoW tables. Add explicit Rappelz extension tables/migrations when their semantics require them, preserving existing data.

## Validation boundary and next work

Historical files under `client-data` include MariaDB system/table files, generated imports, and server/bot logs from 2026-10-25/26. Logs show prior local auth/world sessions and a MariaDB 13.0.2 listener on port 3310. At this audit's process/listener check, no MariaDB process or listener on 3306/3310 was found; no live SQL queries, data import or database restart were performed. Historical logs do not prove the current schema state or every README claim.

The current development session's compilation/test results belong in [DEVELOPMENT_STATUS.md](DEVELOPMENT_STATUS.md). Next database work should establish reproducible integration fixtures, verifier vectors against upstream, migration tracking, and typed Rappelz persistence where generic settings cease to be adequate. Preserve the imported baseline and never use a fresh bootstrap import as an upgrade.
