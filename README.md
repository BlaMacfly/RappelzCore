<p align="center">
  <img src="docs/images/RappelzCore.png" alt="RappelzCore" width="320">
</p>

<p align="center">
  <a href="https://discord.gg/RPtfV6GjZz"><img src="https://img.shields.io/badge/Discord-Join%20the%20server-5865F2?style=for-the-badge&logo=discord&logoColor=white" alt="RappelzCore Discord"></a>
</p>

<p align="center"><b>English</b> · <a href="README.fr.md">Français</a></p>

# RappelzCore & RappelzRust

A **Rappelz** server emulator (RappelzCore) and game client (RappelzRust), written in Rust and
rebuilt by reverse engineering (REA / Ghidra) the **Rappelz NoLimits** client
(`Rappelz NoLimits\SFrame.exe`). The launcher label is 1.3.60; the executable's exact Epic/patch
identity remains unresolved (see [binary evidence](docs/CLIENT_REVERSE_ENGINEERING.md)).

Current continuation: a working **Lua 5.1 addon foundation**, native addon manager, XML subset and
persistent RappelzDebug demonstration are included. See [development status](docs/DEVELOPMENT_STATUS.md)
and the [API compatibility matrix](docs/LUA_API_COMPATIBILITY.md) for tested scope and remaining work.
The [native local-server regression](docs/LIVE_SERVER_VALIDATION.md) and
[two-client combat/XP run](docs/COMBAT_VALIDATION.md) verify real events, synchronized damage,
XP-driven level-up and original-data preservation. The [respawn follow-up](docs/RESPAWN_VALIDATION.md)
adds repeated kills and hidden-query checks. [Player death/revival](docs/REVIVAL_VALIDATION.md) now
verifies observer rendering/position and Lua transitions. [Movement/visibility](docs/VISIBILITY_VALIDATION.md)
now resumes late-visible players' routes and validates range removal. [Layer transitions](docs/LAYER_VALIDATION.md)
verify separation, hidden-interaction denial and saved character re-entry with two clients.
[Player groups](docs/PARTY_VALIDATION.md) add consented invitations, leadership, private chat and live roster/vitals,
verified through actual damage and layer separation. [Skill learning](docs/SKILL_LEARNING_VALIDATION.md) adds the native K window, real learning costs/prerequisites, durable private levels/JP and Lua diagnostics, verified with 40 two-client checks. The combined development workspace passes 124 Rust and 26 Python tests;
each branch keeps its own buildable server/client scope. See the [gameplay roadmap](docs/GAMEPLAY_ROADMAP.md) for remaining systems.

- **RappelzCore** uses **exactly the AzerothCore SQL schema** (`acore_auth`, `acore_characters`,
  `acore_world`, imported from the official repository, commit `3179ea3` of 2026-10-25). It also
  reuses AzerothCore's configuration format (`authserver.conf`, `worldserver.conf`, `AC_*`
  variables), its SRP6 password hashing and its console. An account created by AzerothCore works
  here, and the other way round.
- **RappelzRust** stores its assets **like a World of Warcraft 3.3.5 client**: **MPQ** archives
  (`Data/common.MPQ`, `Data/common-2.MPQ`, `Data/frFR/locale-frFR.MPQ`), **DBC** client databases
  (`DBFilesClient\*.dbc`), `realmlist.wtf`, `WTF/Config.wtf`.
- Both speak the **network protocol of the original Rappelz client**, so RappelzCore also targets
  the official NoLimits client (see "Known limitations").

## Branches

| Branch | Contents |
|---|---|
| `RappelzCore` (default) | **RappelzCore**, the server: `authserver`, `worldserver`, `rz-dbimport`, AzerothCore SQL schema (`sql/`), configuration (`conf/`), the shared crates it needs (protocol, database, Rappelz formats), and the `rz-extractor` (server data) and `rz-bot` (tests) tools. |
| `RappelzRust` | **RappelzRust**, the client: `RappelzRust.exe`, `rz-extractor` (Rappelz client → MPQ/DBC), `rz-bot`, the rendering and format crates (textures, terrain, models, interface) and the shared crates. |

Each branch builds on its own (`cargo build --release`). Game files (Rappelz client, converted
archives) are not versioned: you need a Rappelz NoLimits client to produce them.

## Layout

```
Cargo.toml                    workspace
shared/
  rz-common/                  .conf configuration (AzerothCore style), logging
  rz-mpq/                     MPQ v1: read, write, patch chain (WoW format)
  rz-dbc/                     WDBC: read, write, DBCfmt.h-style format strings
  rz-rappelz/                 original Rappelz formats: data.000-008, .rdb → .dbc, DBC by field name
  rz-proto/                   protocol: "XRC4" RC4, RSA/AES, TS_* messages (msg.rs, msg_game.rs)
  rz-database/                MySQL (sqlx) on the AzerothCore schema, SRP6
  rz-texture/                 textures: DDS (DXT1/3/5, uncompressed), TGA, JPEG, PNG, BMP
  rz-terrain/                 nFlavor maps: .nfm (heightmap, textures, props), .nfw, .nfa, .qpf, .nfs…
  rz-model/                   .nx3 models (BTRF/KNIB), .naf animations, .cob objects, assembly
  rz-ui/                      original interface: .spr sprites, .nui windows, drawing rules
  rz-addon/                   restricted Lua 5.1 addons, TOC/XML, UI/events, SavedVariables
RappelzCore/
  authserver/                 → authserver.exe
  worldserver/                → worldserver.exe (lobby, world, visibility, chat, GM commands, console)
  tools/rz-dbimport/          DBC → acore_world tables
RappelzRust/
  client/                     → RappelzRust.exe (macroquad)
  tools/rz-extractor/         Rappelz client → RappelzRust install (MPQ/DBC); server DBC extraction
  tools/rz-bot/               headless test client
sql/
  base/db_auth|db_characters|db_world/   AzerothCore schema (world: structure only)
  custom/                     RappelzCore additions (default realm)
  create/create_mysql.sql     user and databases (as in AzerothCore)
conf/                         authserver.conf.dist, worldserver.conf.dist
docs/re/                      reverse engineering: archive, RDB, protocol (+ Node validation scripts)
tools/                        AzerothCore schema import, RDB schema generation
```

## Building

```bash
cargo build --release
```

Binaries land in `target/release/`: `authserver`, `worldserver`, `RappelzRust`, `rz-extractor`,
`rz-dbimport`, `rz-bot`. Tests: `cargo test`. Some tests read the real client when it is present in
`Rappelz NoLimits/`.

## Getting started

### 1. Convert the Rappelz client into a RappelzRust client

```bash
target/release/rz-extractor client --client "Rappelz NoLimits" --out client-data/RappelzRust --locale frFR --codepage windows-1252
target/release/rz-extractor verify --client "Rappelz NoLimits" --data client-data/RappelzRust/Data
```

6.4 GB → 3.5 GB of MPQ in about 40 s; `verify` compares all 125,212 assets byte by byte.
Then copy `RappelzRust.exe` into `client-data/RappelzRust/`.

### 2. Server data (like AzerothCore's map_extractor)

```bash
target/release/rz-extractor dbc --data client-data/RappelzRust/Data --out <DataDir>
```

### 3. Databases

```bash
mariadb -uroot -p < sql/create/create_mysql.sql
tools/sql/import_all.sh 127.0.0.1 3306 acore acore
target/release/rz-dbimport --data-dir <DataDir> --apply "127.0.0.1;3306;acore;acore;acore_world"
```

From the DBC files, `rz-dbimport` fills:
- `item_template` (34,682);
- `creature_template` (+ `creature_template_model`);
- `creature` (1,497 placed NPCs);
- `quest_template` (986);
- `game_tele` (407).

### 4. Servers

Copy `conf/*.conf.dist` to `authserver.conf` / `worldserver.conf` and adjust `*DatabaseInfo` and
`DataDir`. Then run:

```bash
authserver -c authserver.conf      # port 4615 (NoLimits launcher auth_port)
worldserver -c worldserver.conf    # port 4514 (= realmlist.port)
```

Worldserver console:
- `account create <account> <password>`
- `account set gmlevel <account> 3`
- `account set password …`
- `announce …`
- `server info`
- `saveall`
- `exit`

### 5. Play

```bash
RappelzRust.exe                      # from the install folder (Data/, WTF/)
RappelzRust.exe -demo                # offline world with the NPCs from NpcResource.dbc
```

Install the demonstration into an existing converted client with
`target/release/rz-extractor addons --out client-data/RappelzRust` (existing addon files are preserved).
Use the **AddOns** button/F9 for the manager and `/reload` in world chat to reload the addon UI.
See [LUA_ADDON_SYSTEM.md](docs/LUA_ADDON_SYSTEM.md) for the supported subset and validation limits.

Authserver address: `Data/frFR/realmlist.wtf` (`set realmlist 127.0.0.1[:port]`).
Volumes: `SET Sound_MusicVolume "0.5"` / `SET Sound_SFXVolume "0.7"` in `WTF/Config.wtf`.

The client displays the original assets:
- lobby and HUD `.nui` windows: login, servers, characters, gauges, chat, quick slots, menu,
  inventory, NPC dialogue;
- map heightmaps and ground textures;
- props (`TerrainPropInfo.cfg` → `.cob` → `.nx3`);
- animated characters, NPCs and monsters (`.nx3` + `.naf`, skinning);
- loading screens, music and sounds.

**In-game controls**

| Action | Input |
|---|---|
| Move | ZQSD/WASD, or right-click on the ground |
| Camera | left-drag; mouse wheel to zoom |
| Target a creature | left-click on it |
| Attack a monster / talk to an NPC | right-click on it |
| Inventory | `I`; right-click an item to equip/unequip, Shift+click to sell while a shop is open |
| Chat | Enter (`/w name msg`, `/y msg`, `.help`) |
| Close windows, then back to lobby | Escape |

GM commands (availability depends on the GM level in `account_access`):

| Command | Effect |
|---|---|
| `.gps`, `.online`, `.save`, `.server info` | position, online players, save, server status |
| `.tele <place>`, `.go x y`, `.appear`, `.summon` | teleports |
| `.announce`, `.level`, `.money` | announcement, level, rupees |
| `.additem <id> [n]` | adds an item |
| `.spawn <monster> [respawn]` | adds a spawn point to `creature` |
| `.vendor add <item>` | adds the item to the nearest NPC's shop (`npc_vendor` table) |

## Rappelz ↔ AzerothCore data mapping

| Rappelz | AzerothCore |
|---|---|
| account, password | `account` (upper-case username, SRP6 `salt`/`verifier`) |
| auth → game one-time key | `account.session_key` (first 8 bytes) |
| servers | `realmlist`, `realmcharacters` |
| character | `characters`: race (3 Gaia, 4 Deva, 5 Asura), `gender` = sex − 1, `class` = base class of the job, `level`, `xp`, `money` (rupees), position |
| job, job level, JP, appearance | `character_settings` (`source = 'rappelz'`) |
| items, monsters, NPCs, quests, places | `item_template`, `creature_template`, `creature`, `quest_template`, `game_tele` |
| inventory and equipment | `item_instance` + `character_inventory` (slot 0–27 = worn slot, ≥ 28 = bag) |
| NPC shops | `npc_vendor` (entry = 1,000,000 + NpcResource id) |
| monster spawns | `creature` (entry = Monster.dbc id), respawn `spawntimesecs` |
| client .rdb databases | `DBFilesClient\*.dbc` (WDBC), read by the worldserver from `DataDir/dbc` |

Constraints inherited from the schema:
- character names are at most 12 characters (`characters.name`);
- passwords are at most 16 characters (AzerothCore rule).

## Reverse engineering (docs/re)

- `data_archive.md`:
  - XOR-encrypted `data.000` index, file-name encoding, hashing to `data.001-008`;
  - extensions stored in clear;
  - Rust reader: `shared/rz-rappelz/src/archive.rs`.
- `rdb_format.md`:
  - 132-byte header, 23 table schemas validated down to the last byte;
  - `db_string`, `db_item.ref`, bit-permuted monster ids.
- `network_protocol.md`:
  - 7-byte header + checksum, RC4 with a non-standard KSA (drop 1013);
  - RSA-1024 + AES-128-CBC for the password and the one-time key;
  - login, world and chat packets, ENTER (scrambled ids), STAT_INFO;
  - §9: 52 gameplay messages (inventory, equipment, NPCs, shop, skills, quests, combat).
- `terrain_format.md`:
  - `.nfm` v16/v22 (384×384 heightmap, texture layers, props, grass), sidecar files and `.cfg`;
  - world coordinates verified (seamless edges, NPC positions).
- `model_format.md`:
  - BTRF/KNIB containers and `nx3.tml` templates;
  - meshes, materials, skeletons, animations (4800 ticks/s), `.cob`;
  - character assembly (`db_DefaultModelRes`, `db_item`).
- `ui_format.md`:
  - text/binary `.spr`, `.nui`;
  - drawing rules (1/3/9 images, states), `<$id>` captions, the windows used on each screen.

## Earlier development verification

The following includes retained results from earlier work. Current combined-workspace checks, including
124 passing Rust tests, 26 Python tests, the optimized build and 125,212 identical converted assets, are separated from
historical online/database claims in [DEVELOPMENT_STATUS.md](docs/DEVELOPMENT_STATUS.md).

- **Unit tests**: RC4 and checksum vectors taken from the binary, sizes of the 20 observed packets,
  MPQ/StormLib, DBC, SRP6, RDB on the real files.
- **Client conversion**: full conversion and byte-by-byte verification.
- **Databases**: AzerothCore schema imported into MariaDB 13 without errors; world data imported.
- **Full login flow**, with two `rz-bot` running at the same time and with the graphical client:
  - authentication, realm, character creation, entering the world;
  - player/NPC visibility, movement, chat, GM commands;
  - saving and logging out;
  - data checked in the database.
- **Gameplay with `rz-bot`**:
  - `.additem` → inventory;
  - equipment (`ITEM_WEAR_INFO` + `WEAR_INFO`);
  - NPC dialogue → shop → purchase (rupees deducted);
  - monster placed with `.spawn` → combat both ways → monster death → Monster.dbc rewards;
  - items, shop and monsters persist across restarts.
- **Rendering**, checked with screenshots (`-screenshot`):
  - original lobby and HUD;
  - Laksy in 3D (buildings, paving, animated NPCs);
  - a monster sent by the server shown with its model.
- **Formats**:
  - 22,061/22,072 nx3, 20,825 naf, 4,311/4,312 cob, 106 maps and 485 `.nui` files read back;
  - DDS/TGA textures decoded without failure on the tested sample.

## Known limitations

- **Rendering**:
  - no water (`.nfw`), procedural grass, effects (`rcfx`, `.spt`), sky or fog yet;
  - weapons and accessories are not attached to bones yet;
  - lighting on models is simplified;
  - armour pieces replace naked body parts based on their file name.
- **Interface**:
  - the windows are the game's own, but RappelzRust chooses their on-screen positions (the
    original client sets them in code);
  - character creation, the shop and the detailed equipment view still use simple panels.
- **Server gameplay**:
  - melee combat with provisional formulas (until db_BasicStat / db_Exp are decoded);
  - EXP/JP/rupee rewards, death and respawn;
  - skill casting/effects, job advancement, quests, ground loot and proactive monster aggression remain incomplete; temporary groups and current-job skill learning are implemented.
- **Original NoLimits client**:
  - not tested against RappelzCore yet;
  - it must be started by its launcher (`<process>_RUNNER`);
  - the RC4 KSA was read from assembly, with no real capture to confirm it;
  - several gameplay packet fields are deduced (marked [D] in the docs).
- **RDB fields**: the meaning of several fields is deduced (their positions are proven, their
  meaning sometimes unknown).

## License

GPL-2.0-or-later. The SQL schema in `sql/base` comes from
[AzerothCore](https://github.com/azerothcore/azerothcore-wotlk) (GPL-2.0). Rappelz is a trademark of
its respective owners; no game file is distributed.
