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

## Versions and game target

| Component | Version / target |
|---|---|
| Our RappelzRust client (Windows x64) | **0.1.0 — development** |
| RappelzCore server | **0.1.0 — development** |
| Original game | **Rappelz NoLimits**, customized x86 client |
| Original launcher/package | **1.3.60** |
| Original SFrame file/product metadata | **9.1.0.0** |
| Original build lineage / protocol | **Source9.5.2** / **201507080** |
| Exact official game Epic / patch | **Not established** |

See [version evidence and interpretation](docs/VERSIONS.md). The launcher, PE metadata and build-directory labels have different meanings; they do not establish an exact official release.

**Terrain pilot (2026-11-02):** [HD grass and shared GPU tiles](docs/TERRAIN_HD_ASSET_VALIDATION.md) preserve original scale/style and provide a separate optional texture pack. Two concurrent native offline clients pass 278 comparisons across four views. Experimental opt-in uniform tiles retain fine detail with 1.33 MiB original / 5.33 MiB HD terrain texture payload, replacing 24–84 MiB of per-chunk bakes in the selected area. Mixed-transition captures expose a visible detail boundary, so the baked renderer remains the default. The normal installation is preserved; online validation of this new path remains to be done.

**Previous validation (2026-11-01):** [SOAP shutdown with two active clients](docs/SOAP_SHUTDOWN_VALIDATION.md) now waits for every character save before exit. **65 checks** cover unsaved movement, a six-second SQL delay, an old-server failure control and restored characters after restart. **216 Rust + 138 Python tests pass** (one Rust ignored, 26 Lua included). Original data/images/settings and services are preserved after the PC reboot. [SOAP configuration](docs/SOAP_ADMINISTRATION.md) remains disabled by default; client/game versions remain explicit above.

The latest [interprocedural investigation](docs/SFRAME_INTERPROCEDURAL_RE.md) adds bounded call/return propagation and two new MAIN-chain helper decompilations. The selected MAIN and primitive effects are separated; child callbacks and rejected contexts remain explicit. **The receiver of 918630 and MAIN+0x458 trigger remain unknown.** Nine preservation checks pass.

The preceding [caller and descendant audit](docs/SFRAME_ENTRY_AND_DESCENDANTS_RE.md) examines 106 additional retained bodies and exact MAIN type-descriptor references. No computed caller or derived MAIN construction is established in those bounded sources. **The actual 918630 receiver and MAIN+0x458 trigger remain unknown.** No new decompilation or original execution; nine preservation checks pass.

The preceding [receiver-family investigation](docs/SFRAME_MODE_RECEIVER_FAMILY_RE.md) decompiles three additional Arena callback/helper functions and adds 22,308 modeled alias states. One old local cap is resolved; seven remain. **The actual 918630 receiver and MAIN+0x458 trigger remain unidentified.** Nine preservation checks pass.

The preceding [MAIN mode-transition audit](docs/SFRAME_MAIN_MODE_TRANSITION_AUDIT.md) revalidates the flag readers, explicit clears and the unbound 918630 receiver. **The alternate-mode trigger remains unidentified.** This audit reuses prior evidence with no new decompilation or runtime execution; nine preservation checks pass.

The preceding [MAIN request-producer investigation](docs/SFRAME_MAIN_REQUEST_PRODUCERS_RE.md) identifies compiled character-menu/taskbar Toggle requests, MAIN's close request and a result-button Show/tab request, with class and original resource bindings. Delivery is queued and retains its guards. **The alternate-mode trigger and receiver of 918630 remain unresolved.** Nine preservation checks pass; data and client/game versions are preserved.

The preceding [MAIN show/list-membership investigation](docs/SFRAME_MAIN_LIST_MEMBERSHIP_RE.md) links qualified internal requests to the actual MAIN registry value, render-list insertion/removal and its visibility callback. The ordinary callback explicitly clears the mode. **The actual input producer and alternate-mode trigger remain unresolved.** Nine preservation checks pass; data and client/game versions are preserved.

The preceding [MAIN render-entry investigation](docs/SFRAME_MAIN_RENDER_ENTRY_RE.md) links a constructed SBotManager/SGameViewPort path to interface drawing, window dispatch and the actual viewport drain. MAIN's membership in the rendered lists remains conditional. **The alternate-mode trigger and receiver of 918630 remain unresolved.** Nine preservation checks pass; data and client/game versions are preserved.

The preceding [MAIN retained-pointer consumer investigation](docs/SFRAME_MAIN_RENDER_CONSUMER_RE.md) identifies the primitive's clipping routine and its 16-byte rectangle read, plus a conditional viewport queue/dispatch path. **The actual renderer passed to MAIN, the receiver of 918630 and the alternate-mode trigger remain unresolved.** Nine preservation checks pass; existing data and client/game versions are preserved.

The preceding [MAIN mode-reader and retained-alias investigation](docs/SFRAME_MAIN_MODE_READERS_RE.md) distinguishes the four mode tests, qualifies the actor/data lookup, and proves a renderer retains MAIN+0x118. **The receiver of 918630 and the alternate-mode trigger remain unresolved.** Nine preservation checks pass; existing data and versions are preserved.

The preceding [MAIN rectangle-producer investigation](docs/SFRAME_MAIN_RECT_PRODUCER_RE.md) identifies titlebar geometry→b377a0→vector growth/allocation→MAIN+0x328 and its selected custom-move geometry consumer. MOVE_BY_CUSTOM=0x10000 is separated from NOFRAME=0x100. Retained interior candidates and a bounded REL8 check yield no new 918630 caller. **Its receiver and alternate-mode trigger remain unidentified.** Nine preservation checks pass; data and client/game versions are preserved.

The preceding [MAIN trigger-provenance investigation](docs/SFRAME_MAIN_TRIGGER_PROVENANCE_RE.md) constrains two actual MAIN constructor/initializer paths, indexed rendering destinations and exact RTTI/symbol metadata. The primitive stride is clarified as 0x144 (324 bytes). **The 918630 receiver and alternate-mode trigger remain unidentified.** No matching original PDB exists in the bounded local inventory. Nine preservation checks pass; data and client/game versions are preserved.

The preceding [notification-action and countdown investigation](docs/SFRAME_NOTIFICATION_ACTIONS_RE.md) traces close-one/close-all button dispatch and cleanup, and confirms that initialization hides the disable control. Countdown state is compared with the main character sheet using actual receiver paths. **Alternate-mode activation remains unresolved.** Nine preservation checks pass; original data and client/game versions are preserved.

The preceding [notification-window investigation](docs/SFRAME_NOTIFICATION_WINDOW_RE.md) identifies window 0xd3 as SUINotificationWnd, its resource, message dispatcher, pushNotification callback and selected queue-to-caption path. The notification lists are distinct from the main character sheet's mode byte. **Alternate-mode activation remains unresolved.** Nine preservation checks pass; client/game versions are preserved.

The preceding [Colosseum warning investigation](docs/SFRAME_COLOSSEUM_RULES_RE.md) identifies four localized warnings, their packed records, selected predicates and two internal output paths. Nineteen new Native dossiers and three containment observations qualify related producers and a registered waiting-window callback. **The alternate character-sheet activation remains unresolved.** Nine preservation checks pass; evidence is saved, the owned engine is closed and client/game versions are preserved.

The preceding [918630 indirect-entry investigation](docs/SFRAME_918630_INDIRECT_ENTRY_RE.md) checks computed pointers, relative carriers, predecessor/exception paths and 40 neighboring modules. Three new Native dossiers and 28 other observations qualify the selected cases. **No indirect caller or alternate-mode activation is confirmed.** Nine preservation checks pass; evidence is saved, the owned engine is closed and client/game versions remain unchanged.

The preceding [auxiliary-provenance investigation](docs/SFRAME_AUXILIARY_PROVENANCE_RE.md) adds **18 Native dossiers and four instruction observations**. Positive construction paths identify shared lazy tooltips and generated tooltip controls. The actual nonzero-offset candidate still lacks a receiver bound to MAIN. **Alternate-mode activation remains unresolved.** Nine preservation checks pass; evidence is saved, the owned engine is closed and versions remain explicit.

The preceding [remaining-callback investigation](docs/SFRAME_REMAINING_CALLBACKS_RE.md) adds **14 Native dossiers**. List nodes and pointer payloads, the sheet's manager/focus route and selected control slots are qualified; known focus callbacks return directly. **Alternate-mode activation remains unresolved.** Nine preservation checks pass; evidence is saved, the owned engine is closed and client/game versions remain unchanged.

The preceding [caption and owner-callback investigation](docs/SFRAME_CAPTION_OWNER_RE.md) adds **25 Native dossiers**. The text callback is identified as caption localization, and original parser callbacks connect resource class tokens to child creation. Immediate MAIN and Basic/Added ownership are distinguished. **Alternate-mode activation remains unresolved.** Nine preservation checks pass; evidence is saved, the owned engine is closed and client/game versions remain explicit.

The preceding [indirect-callback investigation](docs/SFRAME_INDIRECT_CALLBACKS_RE.md) adds **22 Native dossiers**. Rectangle callbacks, the placement-message recipient and a child-creation descriptor carrying the sheet as owner are qualified; the static-control creator is bound to original resources. **Alternate-mode activation remains unresolved.** Nine preservation checks pass; both evidence generations are retained, the owned engine is closed and client/game versions remain unchanged.

The preceding [concrete MAIN pointer investigation](docs/SFRAME_MAIN_POINTER_FLOW_RE.md) adds **68 Native dossiers** and accounts for **59 MAIN entries and both 69-entry child tables**. Startup creation, registry lookup and 12 message sends are linked to the actual sheet; geometry and display effects are separated from mode activation. Alternate-mode activation remains unresolved. Nine preservation checks pass; evidence is saved, the owned engine is closed and versions remain unchanged.

The previous [pointer-delivery investigation](docs/SFRAME_POINTER_DELIVERIES_RE.md) reviews **all 102 retained sites**: 98 have provider procedure bodies and four remain raw-only. Thirteen ECX receiver tags are incidental; the concrete nonzero countdown path belongs to ArenaWaiting. MAIN alternate-mode activation remains unresolved. 102 native dossiers are added; nine preservation checks pass, the engine is closed and client/game versions remain unchanged.

The previous [helper-caller investigation](docs/SFRAME_HELPER_CALLERS_RE.md) finds no ordinary static caller or literal address carrier for 0x812a90. Its copy/update core matches the typed default-settings callback; this does not establish an incoming call. Main alternate-mode activation remains unresolved. Nine preservation checks pass, evidence is exported, the engine is closed and versions remain unchanged.

The previous [implicit-copy investigation](docs/SFRAME_IMPLICIT_COPIES_RE.md) bounds **all 41 retained copy seeds** and adds **13 native dossiers**. Five long copies share a settings source; inspected receivers lead to system settings, with one helper still unbound. Main alternate-mode activation remains unresolved. Nine preservation checks pass; evidence is exported, the owned engine is closed and versions remain unchanged.

The previous [indirect-write investigation](docs/SFRAME_INDIRECT_WRITES_RE.md) adds **25 native dossiers**. Constructed child-to-sheet paths read mode/handle or update geometry; the caption handler and new QuestReward/Arena/ColorBox address escapes are identified. Main activation remains unresolved. Nine preservation checks pass; the owned engine is closed and versions remain unchanged.

The previous [alternate-mode writer investigation](docs/SFRAME_MODE_ACTIVATION_RE.md) adds **21 native dossiers**. The concrete WORD writer belongs to a separately created gauge; panel and game-object copies are distinguished. Main activation remains unresolved, and the rectangle-count guard is not a universal bound. Nine preservation checks pass; evidence is saved and the owned engine is closed. Versions stay unchanged.

The previous [CharacterInfo callback investigation](docs/SFRAME_CHARACTER_CALLBACKS_RE.md) recovers **15 native function dossiers** after the PC restart. Name refresh, dimension caches, the distinct list recipient and child owner pointers are verified. Main alternate-mode activation and fold/unfold action binding remain unresolved. Nine preservation checks pass; existing services are resumed, the analysis session is closed, and versions remain unchanged.

The previous [loading-widget data investigation](docs/SFRAME_WIDGET_DATA_RE.md) adds **18 native function dossiers** and verifies the original archived loading-gauge resource. The previously untyped pointer transfer belongs to a saved loading-progress rectangle; its default static-control factory and virtual recipient are identified. Finite progress branches, source/destination geometry and resource-driven creation are documented. This path does not identify main CharacterInfo+0x458 activation. Nine preservation checks pass; original/installed files, data, services and versions above remain unchanged.

The previous [STAT_INFO routing-guard investigation](docs/SFRAME_STAT_ROUTING_RE.md) adds **27 native function dossiers**. Own-player and creature-slot UI gates are identified, separately from the looser storage predicate. The guarded normal world path processes stats before interface delivery and forwards an already handled message; handled is not a universal stop flag. Indirect mode-write candidates are reduced further, with main CharacterInfo+0x458 activation still unresolved. Nine preservation checks pass; original/installed files, data, services and versions above remain unchanged.

The previous [target-info and message-recipient investigation](docs/SFRAME_MODE_RECIPIENTS_RE.md) adds **36 original-client function dossiers**. The target-info button sends the selected handle to a separate window, then requests its display. Construction/RTTI evidence connects component messages through SGameWorld and SGameManager to SGameInterface; eight SHuntaHolicSystem callbacks are identified. The main character window’s nonzero alternate-mode setter remains unresolved after a broader bounded scan. Nine original preservation checks pass; installed files, services, data and versions above remain unchanged.

The previous [message/UI lifecycle investigation](docs/SFRAME_DISPATCH_RE.md) retains **23 function dossiers**, **123 original PE anchors** and seven typed vtable/RTTI links. Basic/Added tabs, mode/target resets, a local vector, two priority deque banks and callback prepasses are traced. A large completed dossier was recovered from its canonical export after the response-size limit. Some receiver bindings and alternate-mode activation remain unresolved. Installed gameplay, original data/services and versions above are preserved.

The previous [stat-bank decompilation](docs/SFRAME_STAT_BANKS_RE.md) follows **11 original-client functions**: type 0 copies a stat bank; type 1 copies a separate contribution and subtracts selected fields, with order-sensitive behavior. Original UI/tooltips corroborate all seven primary attribute positions. **54 PE anchors**, seven archive/loose/DBC text comparisons and nine preservation checks pass. Versions above remain unchanged; original server bonus formulas remain unresolved.

The previous [static SFrame decompilation pass](docs/SFRAME_PROGRESSION_RE.md) documents numeric PROPERTY width, independently received current/previous job state, stat snapshot dispatch and the party `CHANGE_JOB` false lead. Nine fresh REA/Ghidra function dossiers and37 original byte anchors are retained; server promotion policy and bonus coefficients remain unresolved. Original files, installed gameplay, services and data are preserved.

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
verified through actual damage and layer separation. [Skill learning](docs/SKILL_LEARNING_VALIDATION.md) adds the native K window, real learning costs/prerequisites, durable private levels/JP and Lua diagnostics, verified with 40 two-client checks. [Archived skill3212](docs/SKILL_CORPSE_VALIDATION.md) now requires a selected monster corpse, consumed once under competing casts, with 48 native checks. [Ground rendering](docs/TERRAIN_RENDERING_VALIDATION.md) preserves detail while reducing the observed terrain texture peak from 252 to 84 MiB per client; 12 ground captures match exactly across the real two-client comparison. [Warm terrain streaming](docs/TERRAIN_STREAMING_VALIDATION.md) now reuses exact pixels during rapid camera turns, with a 128 MiB CPU-cache cap per client and 48 native checks. [Cold terrain blending](docs/TERRAIN_COLD_VALIDATION.md) now runs in one cancellable background worker, with 50 two-client checks and identical images. [Cancellation/reconnection](docs/TERRAIN_CANCELLATION_VALIDATION.md) now verifies actual partial blends during turns, lobby exit and TCP closure, with joined workers and preserved data. [Warp/map changes](docs/TERRAIN_WARP_VALIDATION.md) cancel pending work immediately on the accepted WARP, with ten real partial interruptions and exact ground captures across three maps. [Stalled/reset connections](docs/TERRAIN_NETWORK_VALIDATION.md) now verify real eight-second TCP holds, six partial view cancellations, two reset interruptions and fresh-handle reauthentication without changing ground pixels or existing data. [Forced server restart](docs/TERRAIN_RESTART_VALIDATION.md) now passes 37 two-client checks on isolated cloned data, including reused handles, exact recovered ground and realm-scoped startup recovery. [Movement/combat during restart](docs/GAMEPLAY_RESTART_VALIDATION.md) adds 45 checks: real active actions at kill, durable checkpoint recovery, no stale replay and a single saved reward; looped-route movement detection is corrected. [Interrupted-save recovery](docs/SAVE_ROLLBACK_VALIDATION.md) adds 48 checks with two real partially written transactions: process death rolls both back, then a fresh private reward survives normal save and re-entry. [Reward durability](docs/REWARD_DURABILITY_VALIDATION.md) now commits rewards before notifying the client; 49 native checks retain 5 EXP/3 JP after a real server crash with no later manual save. An old-server control rejects premature notifications. [Positive currency/level-up restart checks](docs/MONEY_LEVEL_RESTART_VALIDATION.md) add 57 native checks: +14 rupees, level 2→3, correct EXP rollover and saved 140/70 vitals survive process death and three entries with peer synchronization. [Actual database crash recovery](docs/DATABASE_RESTART_VALIDATION.md) adds 51 native checks: a committed award survives, a pending second award rolls back without announcement, and both clients reconnect to the same world process. Failed-save ghost/socket cleanup is corrected. [Lost successful-COMMIT confirmation](docs/COMMIT_CONFIRMATION_VALIDATION.md) adds 47 two-client checks: the real SQL OK is discarded, no premature gain is announced, and reentry finds level 3/money 1013 without stale overwrite or duplication; the witness saves normally. [Late skill observers](docs/LATE_CAST_VALIDATION.md) now receive the remaining cast after ENTER, then the matching cancel/effect; 27 real two-client checks preserve private rewards/cooldowns and reject old phase replay. [Recovered progression and job leveling](docs/JOB_PROGRESSION_VALIDATION.md) adds lossless BasicStat/Exp/SummonExp archive readers, fresh SFrame request-410 evidence and current-job purchases from the K window. Thirty-six native checks verify exact JP costs, saved JLV, private state and reentry; historical EXP/vitals stay intact. The combined development workspace passes 188 Rust and 118 Python tests;
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
188 passing Rust tests, 118 Python tests, the optimized build and 125,212 identical converted assets, are separated from
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
  - other casting/effects, job advancement, quests, ground loot and proactive monster aggression remain incomplete; temporary groups and current-job skill learning are implemented.
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
