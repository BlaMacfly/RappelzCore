# Player groups: implementation and native validation

Verified **2026-10-31**, Europe/Paris. Two native clients pass **37/37 independent comparison checks**, exit **0/0**, and exercise three distinct groups, actual monster damage, layer separation, leadership transfer, exclusion, voluntary departure and logout cleanup.

## Available gameplay

The new server-owned groups support invitations, explicit acceptance/refusal, leadership transfer, leader-only exclusion/disband, departure and automatic cleanup on disconnect. The local policy allows **eight online members**, expires invitations after **60 seconds**, permits one incoming invitation per player and at most seven outgoing invitations per inviter. Capacity and leadership are rechecked at acceptance; joining, leadership changes and disconnection revoke obsolete invitations. A departing leader of three or more members is replaced by the next member in join order. A one-member remainder dissolves. Group IDs are not reused during a world-service lifetime.

Inviting requires the other player to be currently visible in the same layer. Accepted members retain their private roster and group chat across distance/layer changes; this grants no world visibility, targeting or attack access. Outsiders receive no roster/chat. The group is temporary: lobby exit, disconnect and server restart remove membership. There is no group persistence, shared XP/loot, raid or dungeon allocation yet. These are implementation policies, not verified retail rules.

Client controls: **P** shows/hides the group panel. Select a visible player and use **Inviter**; the recipient can **Accepter** or **Refuser**. Member rows show name, leader, level, layer and actual HP/MP. The leader can transfer leadership, exclude or dissolve; members can leave. Clicking a visible member selects it through the existing guarded target path. **`/p message`** or **`/party message`** sends private group chat. The server also accepts:

```text
.party invite <player>
.party accept
.party decline
.party leader <player>
.party kick <player>
.party leave
.party disband
```

The native scenario uses the same command helper as panel buttons and the real network/gameplay adapter. Native screenshots were visually inspected. Physical-keyboard P and live mouse activation of each new button remain manual checks; the run does not claim to automate those inputs.

## Explicit native protocol extension

`shared/rz-proto/src/party.rs` defines an opt-in **Rust extension version 1**, not recovered SFrame layouts. `CS_PARTY_HELLO` **62000** carries a u16 version. The server sends `SC_PARTY_STATE` **62001** only after an authenticated in-world player opts into the supported version. Older clients receive no unsolicited extension packets; older servers leave party APIs unknown.

The state contains version, group ID, leader, bounded member count, members (handle/name/level/layer/current and maximum HP/MP), then an optional inviter/name/remaining-seconds tuple. A nonempty roster includes the recipient. Empty roster with zero IDs confirms solo status. Parsing rejects unsupported versions, oversized/one-member rosters, duplicate/zero handles, invalid identity/vitals/invites, truncation and trailing bytes. The client also rejects a roster omitting its own handle. The server recomputes state from live entities and sends only changed snapshots. The wire format and native group policy must be reconciled with original-client evidence before claiming interoperability.

## Lua connection and live defect

`GetNumPartyMembers()` now returns the accepted number of **other** members: nil until confirmation, zero when solo, 1–7 in a group. Existing UnitName/Level/Layer/Health/Mana APIs accept `party1`–`party7` in stable join order excluding the player. Unit fields remain known even when an admitted member is outside world visibility. `GetPartyInviteName()` is a read-only Rappelz extension.

Real events: `PARTY_INVITE_REQUEST(name)`, `PARTY_MEMBERS_CHANGED`, `UNIT_HEALTH/UNIT_MANA("partyN")`, and `CHAT_MSG_PARTY(message, sender)`. Rappelz extensions `RAPPELZ_PARTY_INVITE_CLOSED` and `RAPPELZ_PARTY_STATE_READY` signal invitation removal and initial authoritative roster discovery. Discovery does not fabricate a membership change; repeated snapshots and UI reload do not repeat invitations/changes. No Lua API can form a group or accept an invitation.

The first native run completed gameplay but its comparison exposed a genuine stale diagnostic: accepted solo count was 0 while RappelzDebug still displayed `?`, because no membership change had occurred. Added the explicit readiness event and a Lua regression. The final native run verifies initial known zero on both clients. The comparer also explicitly reads UTF-8, preserving accented server notices on Windows; an independent CLI test covers this.

## Two-client sequence and preservation

A consistent three-database logical backup was taken **before** creating fixtures: **18,900,488 bytes**, SHA-256 `d24aad2c27736df4dd399d8428f88de00cb940adf2da825a120092f2a3498986`. New account/character **14 / LuaGrp950E** is the dedicated GM-2 leader; **15 / LuaMem950E** is an ordinary member. Both start at level 1, HP 100, MP 50, XP/JP/money 0, layer 0; positions 8440/7227 and 8420/7240. Accounts/characters were created through normal server/bot flows. Preparation is restricted to these exact new offline identities; all earlier fixtures remain intact.

1. Both receive actual solo state. The member sees an invitation without joining and declines it, then accepts a fresh invitation.
2. The leader attacks the visible monster and receives real retaliation. Both traces agree on attacks and HPMP. The leader stops attacking and enters layer 1: the member loses its world entity but retains the same damaged HP/MP/layer in its group roster and Lua `party1` state. The leader's return and existing GM `.level 1` path restore base-layer full vitals; no kill/reward occurs.
3. The ordinary member's disband request is denied. Both exchange two actual private group messages and observe leadership transfer. The new leader excludes the former leader; the two-person group dissolves. A group message from the now-solo former leader is refused and reaches neither group chat trace.
4. They form another group with a distinct ID. The member leaves and both become solo. A third group forms, then leader logout removes the member's roster and visible peer; the member logs out too.

The final process-local event counts include leader/member membership changes **6/7**, one readiness event each, four invitation/open-close events on the member, two chat events each and layer events **2/0**. No death/alive, XP/JP/reward or level-up occurs. Both new characters finish offline, layer 0, HP 100/MP 50 and unchanged level/XP/JP/money/position.

Preservation passes **9/9 checks**: original account/access/realm-character records through ID 13, character/settings/inventory through GUID 13, items through GUID 4, all existing world INSERTs and installed WTF hashes. No schema migration/content import was needed. Operational realm/uptime metadata is outside the equality claim. Existing data and all prior binaries/evidence remain; no fixture is deleted. World restarts were graceful at zero online players and reset only transient monster/group state.

## Verification and restart

The combined workspace passes **110 Rust tests**, one intentionally ignored model scan, including **19 actual Lua integration tests**, and **22 Python tests**. Coverage includes invitation/authority/capacity races, expiration/revocation/disconnect, opt-in/privacy/layer state, strict wire parsing/truncation, the graphics-independent production adapter, Lua known-zero/unknown/dead-member values, event deduplication and the independent report sample/mutations/UTF-8 CLI. The inventory audit also found that unsupported ground drops erased items without creating loot. The handler now refuses the operation; a unit test of the production Session handler with unreachable database pools proves existing stack/slot/GUID are retained for valid/invalid counts and handles, without SQL or destroy packets. Intentional erasure remains separate.

Optimized server/client/extractor builds and Clippy complete; remaining Clippy warnings are unchanged older style/large-enum warnings. The installed client SHA-256 is `819AB0AF9B1F5DA06C8327115E3D81124858148E60F6FC6176540C565719620B`. Installed addon files are current; prior executables/addon and preferences are preserved. Ignored evidence is under `client-data/validation/2026-10-31-party-950e67a3/`: `initial/` retains the diagnostic defect, `final/` is the first corrected pair and `verified/` reruns the final client/server code including the drop guard, with local credentials, backup, snapshots and preservation/build/test logs outside Git.

The separate publication worktrees pass complete locked/offline test suites and optimized workspace builds: **50 Rust tests** for the server branch and **89 Rust tests, one ignored** for the client branch, plus **22 Python tests on each**. Counts overlap through shared crates and are not additive. Asset-dependent tests skip absent resources in clean worktrees; the combined workspace and native sessions used the actual local assets.

```powershell
# Separate native roots, dedicated Lua* characters and fresh leader/member plans:
# -validate-party <plan.json>, role/peer/directory as in the preceding scenario.
python tools/testing/compare_party_clients.py <attempt-directory>
python -m unittest discover -s tools/testing -p test_*.py -v
cargo test --workspace --all-targets --locked --offline -- --test-threads=4
```

See [GAMEPLAY_ROADMAP.md](GAMEPLAY_ROADMAP.md) for the remaining game systems and their acceptance criteria.
