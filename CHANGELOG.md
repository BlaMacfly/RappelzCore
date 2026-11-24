# ⚔️ RappelzCore & RappelzRust — Development Changelog

**📅 November 24, 2026**

**🚧 Status: Early Development / Pre-Alpha**

**🏷️ RappelzCore & RappelzRust: 0.1.0 — development**

Since our November 3 update, development has focused on decompiling and rebuilding core gameplay systems: quests, storage, player trade, ground loot, creature summons and the first active skills. Each system is first documented from the original client, then implemented on the server and tested.

## 🔒 Security Fix — please update

- ⚠️ **Gold duplication in player trade.** Since the trade system was first published (commit `8185dcf`, November 23), a player who disconnected right after validating a trade kept the gold they had given away. This was reproduced in 10 out of 10 attempts. It is fixed in commit `29a9db1`: 17 trades were executed afterwards, 13 of them with abrupt disconnects, and the total gold never changed. **Any server running a version from `8185dcf` onwards should update.**
- ✅ Dropped items are now only visible on their own layer.

## 🖥️ RappelzCore — Server Emulator

- ✅ **Quests**: NPCs now offer, track and complete quests from the original quest data (986 quests, 1,306 NPC links). Supported objectives:
  - kill monsters;
  - collect items;
  - hunt quest items;
  - talk to an NPC;
  - reach a job level.

  Rewards and quest completion are saved together in a single database transaction. Random, script and skill quests are not offered yet.
- ✅ **Storage**: banker NPCs open a personal storage for items and rupees. Partial stacks can be deposited and withdrawn.
- ✅ **Player trade**: full negotiation is supported: request, items, rupees, lock and confirmation. The exchange runs in a single database transaction that re-checks every item before anything moves. Inventory actions are blocked during a trade, and disconnecting cancels it.
- ✅ **Ground loot**:
  - monsters drop loot around their corpse, reserved to the killer for 30 seconds and removed after 3 minutes;
  - players can drop items and pick them up;
  - if saving a picked-up item fails, the item returns to the ground.
- ✅ **Loot tables**: the client's drop groups are imported into the standard AzerothCore loot tables (31,657 rows). Per-monster loot is not included in the client, so server administrators must add it.
- ✅ **Creature summons**: creature cards hold a creature stored in the standard AzerothCore pet table. Summon, dismiss, formation and following the master work, and a creature moves with its card when traded. Summoned creatures do not fight or gain experience yet.
- ✅ **Active skills**: 47 skills are available: single-target physical and magical damage, heals, and summoning with its real cast time. Damage follows the formulas shown by the original client; defence is still provisional.
- ✅ New GM commands: `.quest add`, `.quest remove` and `.learn`, as in AzerothCore.
- ℹ️ Servers need to run `rz-extractor` again to get `Creature.dbc`. Without it the server starts with summons disabled.
- 🔄 **Databases renamed** to `rcore_auth`, `rcore_characters` and `rcore_world`, with the default user `rcore`. Table names and structure are unchanged and remain identical to AzerothCore. Existing `acore_*` installations keep working with their current configuration lines.
- ✅ The test bot can now run quests, storage, two-player trades, looting, item drops, summons and skills end to end.

## 🎮 RappelzRust — Game Client

- No new client features in this update. The client branch will be resynchronized with the new shared protocol code in a future update.

## 🔬 Reverse Engineering

- ✅ Documented the original client's **quest protocol**:
  - quest list and progress packets;
  - abandoning a quest;
  - NPC quest dialogs, including the client-built reward choice;
  - how progress text is displayed.
- ✅ Documented **storage and player trade**: storage opening, deposit and withdrawal modes, stored rupees, and the 12 modes of the trade message.
- ✅ Documented **ground items**: item appearance, rupees on the ground, pickup range, results and messages, and pet auto-loot. The client never removes ground items itself, so the server must.
- ✅ Documented **creature summons**: cards, formation, the summon packets and the creature data table (3,543 creatures). Summoning goes through skills; no sender was found for the dedicated summon packet.
- ✅ Documented **skills and states**: cast phases, hit types, state rules, cooldowns and the damage formulas stored in the client texts.
- ✅ Established that the client reads loose `Resource` files before its archives, and that it overwrites 100 skill values after loading them.
- ✅ Verified a quest data field offset against the real data files; the existing format description was correct.
- ✅ Reviewed the client's embedded PNG writer routine (November 4).

All findings are published as documents under `docs/re/`. They mark what was observed in the client and what is still deduced, and they list open questions.

## 🧪 Validation & Data Preservation

The new systems were tested on isolated test databases with the test bot, including two simultaneous players for trading; heals and some refusal cases are covered by unit tests only. A fresh installation was also checked from scratch: database creation, schema import (22 / 108 / 309 tables, as in AzerothCore), world data import, server start, account and character creation and world entry. These systems have not yet been tested with the original graphical client.

## 🏷️ Original Game Reference

- **Rappelz NoLimits launcher/package:** 1.3.60.
- **Original SFrame file/product version:** 9.1.0.0.
- **Protocol reference:** 201507080.
- The exact official Epic/patch remains unconfirmed. These identifiers are separate from our software version.

## 🔧 Still in Development

Still missing:
- area skills, states, toggles and passives;
- creature combat, experience and evolution;
- advanced monster AI;
- random and scripted quests and quest timers;
- per-monster loot data;
- pets and mounts;
- guilds and other social systems;
- client windows for the new systems;
- rendering improvements;
- full original-client compatibility.

The project remains **Pre-Alpha**. Tested development scenarios do not establish complete gameplay or production readiness.

## 🤝 Open Source & Contributions

Developers, reverse engineers, Rust programmers and testers are welcome!

🔗 [RappelzCore — Server Emulator](https://github.com/BlaMacfly/RappelzCore/tree/RappelzCore)

🔗 [RappelzRust — Game Client](https://github.com/BlaMacfly/RappelzCore/tree/RappelzRust)

📖 [Wiki](https://github.com/BlaMacfly/RappelzCore/wiki)

Our goal remains to rebuild Rappelz with a modern, open-source Rust architecture while preserving the spirit of the original game.
