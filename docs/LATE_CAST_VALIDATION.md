# Skill synchronization for late observers

Verified **2026-10-31** with **27/27 independent checks**, two native Windows x64 clients and the existing archived corpse profile of skill **3212**. An ordinary player entering visibility during an incantation now receives its **remaining** duration, followed by the matching cancellation or effect. Private skill lists, cooldowns and rewards stay with their owner. This extends [corpse-rule validation](SKILL_CORPSE_VALIDATION.md).

## Defect and implementation

Previously `PendingCast.audience` was frozen at the beginning. A player entering range during the cast received ENTER but no CASTING and no terminal phase. The new server regression fails against that implementation with `late observer missed active cast`.

After all visibility ENTER/LEAVE packets are sent, the server reconciles each pending cast's audience. A new observer must currently see both the caster and its corpse target, in the correct layer; CASTING carries only the positive unelapsed duration. Repeated visibility updates and QUERY replies do not restart the incantation. Invalidated or already finished casts are not introduced to new viewers. Leaving the caster's visibility uses normal LEAVE cleanup; losing only the target sends CANCEL for the previously accepted phase. Returning while it is still active can subscribe again.

FIRE rechecks recipient visibility **before** consuming the corpse. An observer crossing only the target boundary between the visibility and FIRE ticks receives CANCEL instead of the now-hidden effect. The caster's actual valid spell still fires once. Consuming a body may cancel a competing pending cast during visibility reconciliation; the completion loop tolerates that removal. Audience membership comparisons use sets. No SQL runs under the world lock, and no packet, opcode, amount, cost, respawn rule or cooldown persistence policy is changed.

The existing client already accepts the same checked CASTING/CANCEL/FIRE subset. A new client regression verifies a short remote duration, duplicate-start suppression, exact matching terminal phases, post-LEAVE corpse recovery and no local mana/cooldown grant. The native test is an explicit `late_observer: true` mode of `-validate-corpse-skill`; normal controls and gameplay are unchanged.

## Actual session

Original database/auth/world services on **3310/4615/4514** keep their PIDs, executable and data. A fresh isolated MariaDB **3331**, auth **24615** and copied world **24514** use cloned schemas. Only new clone fixtures **34/35**, sentinels **36/37** and additive spawn **1499** are created. Original world rows, including spawn 1498, remain exact. Monster 651's converted HP 105, EXP 5, JP 3 and gold 8–14 are unchanged. Both ordinary GM-0 fixtures are seeded at level 10, job 302/job level 35, learned skills 63203:1 and 3212:1, MP 20; these are fixture prerequisites, not earned progression.

1. Owner starts at **9440/7227**, witness at **9950/7227**, using the existing 540-unit visibility distance. Witness moves normally to **10040/7227**, hiding both owner and monster before combat. No privileged layer command, fabricated phase, special spell duration or client teleport is used.
2. Owner kills the real monster and starts its normal **145-centisecond** corpse cast. Witness walks back into range and receives the dead target/caster ENTERs followed by CASTING with **86 centiseconds** left. It queries both objects twice. No extra CASTING is sent. Owner cancels normally; both receive CANCEL, with unchanged caster mana and no cooldown.
3. Witness leaves again. The monster respawns on its unchanged ordinary timer; owner kills it again and casts. Witness's second arrival gets **88 centiseconds** left. Both receive one matching corpse FIRE: cost **5 MP**, nominal restoration **320**, actual caster MP capped at **140**. Corpse consumption and the owner's private **5800-centisecond** recharge occur once. Witness gets neither the skill list update nor that recharge.
4. Witness leaves and returns after the effect. The caster's ENTER now contains confirmed MP 140, with no old CASTING/FIRE replay. Both finish with normal lobby saves and exit **0/0**. Copied world PID **32596** exits normally after the clients; no production service is restarted.

Both raw histories contain exactly **CASTING → CANCEL → CASTING → FIRE**. Each real Lua runtime accepts exactly **four RAPPELZ_SKILL_EVENT callbacks**, plus one login/entry/logout lifecycle. The witness's final RappelzDebug screenshot was inspected showing FIRE from caster 1, amount 320, while its own money remains **2000**, JP **0**, MP **76/140** from ordinary regeneration. Owner saves level 10, EXP **10**, JP **6**, money **1023** (**+10/+13** from the two kills), HP/MP **263/140**; witness saves EXP/JP 0/0 and HP/MP **280/76**. Both retain their exact unrelated settings, learned levels and final positions. No native geometry, hardware performance, keyboard or broad-latency acceptance is claimed.

The initial successful pair is retained before the final boundary-at-FIRE review. The final native pair also succeeded immediately; its comparer initially assumed the killing-blow frame already contained the wallet update. That frame can precede the durable reward packet. The corrected check independently requires exactly two private gold increments, each 8–14, and an exact final SQL balance. A regression covers that ordering; raw reports and the rejected comparison remain retained.

## Checks and preservation

- Combined workspace: **170 Rust tests pass**, one exhaustive model scan ignored; **22 Lua tests included**. **113 Python tests pass**.
- Publication branches: Core **80 Rust**; client **131 Rust**, one ignored; **113 Python tests each**, plus six optimized AMD64 executable types built.
- Four new server regressions cover ENTER-before-remaining-CASTING, no QUERY replay, target-only loss/resubscription/cancel, expired-cast exclusion and the between-ticks FIRE boundary. One client regression covers acceptance/private state; five Python tests cover independent acceptance and mutations of durations, phases, hidden entities, Lua counts, foreign lists, duplicate rewards, SQL profiles and process failures.
- Original selected accounts/characters through **33**, items through **4**, world INSERTs and installed WTF pass **9 preservation checks**. Copied pre-existing content plus the exact additive spawn pass **9**. Original high-water marks and service PIDs remain unchanged; fixtures are offline GM-0.
- Logical backup: **18,917,362 bytes**, SHA-256 `5203672ea4722a1014f3611807250bb5f5441712bdee0e1e930fd618439dd9d4`.
- Native-tested installed client SHA-256 `089ad5aa456111be7236e75306dc5fd7a931fe32121bc8887cc561f16d773e59`; its predecessor is retained. Tested copied worldserver SHA-256 `b94852efc9bacb5bce3e602a523f44d720f9e646ea34cd1016861abe663a43cc`.

The original running server image remains `3ecfcb979e6dbffa4709945ca6403a931b2ef5e4cad743a79dfc064ce125a37c`; its replacement/restart is not claimed. No migration, original asset replacement or production content import is added. Source/tests/docs are published on both branches. Credentials/config, backup, assets, database files, copied executables, screenshots and raw reports stay ignored under `client-data/validation/2026-10-31-late-cast-97acec84/`; `verified/` is the final evidence. Additional test services stop after final checks and persistent files are retained.

## Reproduction and remaining scope

Use a fresh isolated clone and the existing additive corpse companion resources, separate auth/world/database endpoints, new offline fixtures and a fresh attempt directory. Give both native plans `late_observer: true`, matching role/peer/directory and the near/far points above. Start the copied world, run both clients with `-validate-corpse-skill`, then compare raw histories, Lua state and independently read final SQL profiles using `compare_late_cast_clients.py`. Run original/clone preservation and both branch builds before publication.

The unit tests cover both range and layer boundaries; this native session covers ordinary range movement. More simultaneous observers, higher latency/load, casting-speed/state/equipment modifiers, other effects, restart-persistent cooldowns and original SFrame protocol equivalence remain separate work.
