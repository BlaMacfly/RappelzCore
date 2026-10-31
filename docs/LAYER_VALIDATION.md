# Layer transitions: two native clients

Verified **2026-10-31**, Europe/Paris. Two native clients use the existing local auth/world services. The final `verified/` pair exits **0/0** and its independent comparison passes **32/32 checks**: layer separation, blocked hidden interactions, persisted character re-entry and restoration of the base layer.

## Data preservation and fixtures

A consistent logical dump of all three databases was taken before creating fixtures: **18,898,538 bytes**, SHA-256 `a43a7aa82fc9fa6983da07c6573c418fe7cb5f2360204696bdededd3cc3df633`. Services remain bound to loopback (3310/4615/4514). There was no schema/content import or migration; the existing `character_settings` Rappelz `layer` key is reused.

| Role | Character | GUID/account | Dedicated access and offline preparation |
|---|---|---:|---|
| Operator | LuaLay51A9 | 12 | GM 2 on this new account only; layer 0, level 1, HP 100, MP 50, XP/JP/money 0, position 8440/7227. |
| Witness | LuaWit51A9 | 13 | Ordinary GM-0 account; layer 0, same stats, position 8420/7240. |

Both are new accounts/characters created through normal flows. Resetting attempts is guarded by new GUID/account, exact name and offline status, and affects only these fixtures. Each client has separate addon/WTF files and reads installed MPQs through a junction. World-service replacements were graceful with zero players online; prior binaries/addon files and all earlier evidence/backups remain.

The final preservation comparison passes **8/8 checks**: pre-cycle account/access/realm-character rows through ID 11, character/settings/inventory rows through GUID 11, items through GUID 4, and all existing world INSERTs match the backup. Installed WTF hashes also match. All eleven earlier characters are unchanged. Operational realm/uptime metadata is outside this equality claim. The new fixture rows finish offline in layer 0 with unchanged stats and positions; no fixture is deleted.

## Implemented transition path

The server previously had layer-filtered visibility but no controlled live transition command. Added **`.layer <0-255>`**, restricted to GM level 2 or higher, acting only on the issuing character. Invalid arguments are rejected. `World::change_layer` stops movement/pending attacks and remaining monster pursuit, changes the authoritative layer and uses the existing WARP/visibility refresh. Same-layer requests emit no new WARP and preserve the existing NPC context.

The native test uses this real authorized chat-command path. The ordinary witness also tries it and is refused. This is administrative phase control, not a public client packet, dungeon allocator or automatic party/instance feature.

The Rust client now retains the accepted login/WARP layer, clears previous-layer entities/target/NPC UI on a layer-changing WARP, and accepts server target acknowledgements only for known entities (zero clears the target). A graphics-independent adapter test covers these state/UI transitions.

## Defects found and negative controls

1. **Hidden NPC dialogue/market remained usable.** `npc_at` performed a global lookup, so the layer-1 operator could contact a layer-0 NPC by its old handle and reopen its dialogue/market. The `baseline/` report fails with `hidden NPC dialogue or market accepted across layers`; wire records include the extra dialogue and market. Hidden-player targeting also returned the old handle. NPC contact/dialogue/buy/sell paths now recheck current announced visibility, layer and range; invalid contexts are discarded. Target requests return zero when not visible. Allowed NPC dialogues are verified both before separation and after restoration.
2. **Current layer was not saved.** Once access checks were fixed, `after-access/` successfully isolated the clients, but the operator left in layer 1 and re-entered in layer 0: `saved layer was lost on relogin`. `Session::save` now copies the live layer into the existing Rappelz settings key (and live z into position_z). The final server login results are layers **[0, 1]**, with a new session handle on re-entry.

Character logout/re-entry here uses the actual lobby/server character-loading path on the existing connection. It loads the saved character again; a full auth/client-process restart is not claimed. Both failing attempts, an earlier passing `final/` run, and the final-code `verified/` run are retained locally.

## Actual two-client sequence

1. Both characters start nearby in layer 0. The witness selects the operator; the operator opens a real NPC dialogue. The witness's `.layer 1` is denied, and the operator's `.layer 256` is rejected without a WARP.
2. The operator changes **0→1** at the same coordinates. Its 18 dynamic entities disappear; target/dialogue/market clear. The witness stays in layer 0, retains its other 17 entities, receives the operator's LEAVE and clears its selected target.
3. Both send QUERY and targeting requests using the still-connected hidden peer's old handle. No extra player ENTER is received; targeting returns zero. The operator additionally sends an attack against the hidden base-layer monster, CONTACT against the old NPC and a market-dialogue request. Attack is explicitly denied, no hit occurs, and no hidden dialogue/market opens during the observation window.
4. The operator logs out to the lobby and re-enters. Its accepted login layer remains **1**, its handle changes, and no base-layer entities appear. The witness still does not see it.
5. The operator returns **1→0**. Both receive fresh visible player data. The witness selects the new handle and queries it; the operator can again select/query the witness and open the same NPC dialogue. Actual Lua target layer/vitals are 0 and 100/100 HP, 50/50 MP.
6. The operator logs out again, the witness receives the new handle's LEAVE/target clearing, then logs out. Both remain level 1, HP 100, MP 50, XP/JP/money 0, with no movement, combat or bag changes.

The final operator receives WARP layers **[1, 0]**, two player login/logout cycles and exactly two layer events. The ordinary witness receives no WARP and only its own unchanged private XP. Native screenshots for layer 1 and both restored views were visually inspected.

## Lua and diagnostics

Added read-only **`UnitLayer("player"/"target")`**, a Rappelz-specific extension. It returns accepted client-state data, nil for absent/unknown units/layers, and preserves known zero. **`RAPPELZ_LAYER_CHANGED(newLayer)`** fires only for a known layer change of the same player; initial discovery, another handle and UI reload do not fabricate it. It grants no layer-changing ability to addons.

RappelzDebug displays player/target layers and records the real event/state. The native operator displays layer 1 before and after re-entry, then layer 0 on return; the witness remains layer 0. Account-level event counters remain cumulative, while native reports count events emitted during each process. The XML frame was enlarged so all diagnostic rows remain visible.

## Reproduction and verification

Use **`-validate-layer <plan.json>`**, mutually exclusive with the existing validation flags. Each plan contains role `operator` or `witness`, the raw Lua* peer character name and a shared fresh report directory. The existing loopback/LUA* account/Lua* character guards remain. Use the documented new fixtures, an operator account with GM≥2, an ordinary witness and separate data roots. The driver handles the server's actual `<GM>` display prefix. It never writes SQL; fixture preparation remains a separate strictly scoped offline step.

```powershell
python tools/testing/compare_layer_clients.py <attempt-directory>
python -m unittest discover -s tools/testing -p test_*.py -v
cargo test --workspace --all-targets --locked --offline -- --test-threads=4
```

The combined workspace passes **100 Rust tests** (one intentionally ignored model scan), including **17 actual Lua integration tests**, plus **18 Python tests**. Tests cover stale NPC handles, layer/attack/visibility transitions, the real state adapter without graphics, known/unknown Lua layers, event deduplication and bounded login/WARP/dialogue/denial traces. The read-only comparator has an independent passing sample and rejects persistence, NPC, target, permission, event, adapter and reward mutations. The older revival comparer accepts the added WARP-layer metadata and rejects a nonzero revival layer.

The separate publication worktrees also pass their complete locked/offline suites and optimized workspace builds: **43 Rust tests** on the server branch and **84 Rust tests, one ignored** on the client branch, plus **18 Python tests on each**. These counts overlap through shared crates and are not additive. Asset-dependent tests in clean worktrees skip absent local resources; the combined workspace and native sessions used the actual installed local assets.

Optimized native client/server/extractor builds pass. Clippy completes with the same existing style/large-enum warnings and none in changed code; all six local release executables remain AMD64. Evidence, SQL dumps, credentials, screenshots, old binaries and build logs remain under ignored `client-data/validation/2026-10-31-layer-51a93c76/` and are excluded from publication.

This verifies two-client layer 0/1 separation and return. Same-nonzero-layer cohorts, additional observers, moving/combat-active phase changes, cross-layer chat policy, non-base NPC content, dungeon/party allocation and retail phase rules remain separate coverage.
