# ⚔️ RappelzCore & RappelzRust — Development Changelog

**📅 November 3, 2026**

**🚧 Status: Early Development / Pre-Alpha**

**🏷️ RappelzCore & RappelzRust: 0.1.0 — development**

Since our first update, development has focused on multiplayer consistency, reliable character saves, terrain rendering and a deeper understanding of the original client.

## 🖥️ RappelzCore — Server Emulator

- ✅ Improved two-client synchronization for combat, experience-driven leveling, player death, revival and layer changes.
- ✅ Added consent-based parties, leadership handling, private party chat and disconnect cleanup.
- ✅ Implemented skill learning with JP costs and prerequisites, plus supported local casting, cancellation and cooldowns. Skill support remains partial.
- ✅ Validated the archived skill 3212 profile: a selected monster corpse is required and consumed only once.
- ✅ Rewards now commit to the database before being announced. Controlled two-client tests cover retained experience, JP, currency and level-ups after server or database interruptions.
- ✅ Added authenticated SOAP administration inspired by AzerothCore/TrinityCore. It is disabled by default.
- ✅ Graceful shutdown now waits for character saves. Two-client tests cover delayed SQL writes, restart and restored character state.

## 🎮 RappelzRust — Game Client

- ✅ Sharper terrain blending preserves more of the original texture detail and style.
- ✅ Camera culling and texture eviction reduced the measured terrain-texture peak from **252 to 84 MiB per client** in the tested scene.
- ✅ Background terrain blending and bounded caching reduce foreground work. Cancellation is tested across camera changes, disconnections, teleports and map changes.
- ✅ Added an optional HD terrain-patch packaging and shared-texture prototype. This path remains experimental; the existing terrain baker stays the default.
- ✅ Expanded Lua/RappelzDebug integration for character stats, party state, skills and progression events.

## 🔬 Reverse Engineering

- ✅ Completed the initial SFrame function inventory and first pseudocode extraction for its **56,151 internal functions**.
- ✅ Reached **3,659 detailed, independently verified SFrame dossiers**, covering **1,193,745 deduplicated original body bytes**.
- ✅ Extended analysis of character statistics, interface callbacks and message routing, with resumable checkpoints and explicit coverage limits.

Extracted pseudocode still requires interpretation, reconstruction and testing. The character-sheet alternate-mode trigger remains unidentified, and analysis of the other inventoried modules is incomplete.

## 🧪 Validation & Data Preservation

Gameplay and recovery scenarios use two native clients and isolated test data. Existing records, settings and original assets are preserved. The latest static-analysis preservation checks also retain the original-file and logical SQL-backup hashes; live database contents were not inspected during that analysis.

## 🏷️ Original Game Reference

- **Rappelz NoLimits launcher/package:** 1.3.60.
- **Original SFrame file/product version:** 9.1.0.0.
- **Protocol reference:** 201507080.
- The exact official Epic/patch remains unconfirmed. These identifiers are separate from our software version.

## 🔧 Still in Development

Broader skill/effect coverage, advanced monster AI, quest progression, ground loot, pets/summons, remaining social/content systems, rendering improvements and full original-client compatibility.

The project remains **Pre-Alpha**. Tested development scenarios do not establish complete gameplay or production readiness.

## 🤝 Open Source & Contributions

Developers, reverse engineers, Rust programmers and testers are welcome!

🔗 [RappelzCore — Server Emulator](https://github.com/BlaMacfly/RappelzCore/tree/RappelzCore)

🔗 [RappelzRust — Game Client](https://github.com/BlaMacfly/RappelzCore/tree/RappelzRust)

Our goal remains to rebuild Rappelz with a modern, open-source Rust architecture while preserving the spirit of the original game.
