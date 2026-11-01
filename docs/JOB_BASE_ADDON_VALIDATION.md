# Read-only raw job bases in Lua

Verified **2026-11-01** with two native clients and **44 independent checks**, including all 36 existing progression checks. Addons read the raw seven-attribute job base already supplied by the private own-player progression preview. These values precede equipment, JLV, inherited-job and state modifiers. Provisional combat-packet primary attributes remain unexposed as effective totals.

| API/event | Contract |
|---|---|
| `GetJobBaseInfo()` | Confirmed job ID and BasicStat resource ID; two `nil` values when unavailable. |
| `GetJobBaseStat(field)` | Raw `strength`, `vitality`, `dexterity`, `agility`, `intelligence`, `wisdom`, or `luck`; unsupported/unavailable returns `nil`, known zero stays zero. |
| `RAPPELZ_JOB_BASE_CHANGED` | No arguments; newly committed complete state readable. Discovery, identity/value change or in-world loss emits; duplicate previews/JLV-only changes do not. |

The adapter uses only well-formed accepted previews for its own nonzero handle and valid job identity. Foreign/truncated previews cannot overwrite it; a confirmed missing base clears it. A known base remains usable when a purchase is unsupported/unaffordable. No fallback to provisional combat attributes is added.

`PLAYER_LOGOUT` retains the last snapshot during callbacks, then the lobby state is empty. Logout emits no fabricated base change. Fresh entry discovers the new private state. Reload preserves readable state without replaying a base event. RappelzDebug's three added lines are labeled **Job base (raw)** and show both IDs/all seven values; its enlarged frame fits the rows and counter. Both native screenshots were visually inspected.

Final GM-0 fixtures: jobs **100/200**, level **10**, JP **8/0**, money **1000/2000**, HP/MP **280/140**. Both selected resource rows contain seven tens; separate distinct-field/zero-value Lua regressions verify ordering. Owner buys JLV **1→2→3** for **3+5 JP**; witness stays JLV **1**. Each addon receives exactly **two** complete base events over two entries, with no extra event for purchases/foreign-owner probes. Both lobby snapshots clear it; reentry loads correct identities under fresh handles. Four SQL gates/final profiles retain XP, money, vitals, positions and unrelated settings. Clients/server exit **0**.

The same session verifies [SOAP administration](SOAP_ADMINISTRATION.md), including announcements in both actual Lua hosts and shutdown without stdin input. The first attempt's shutdown failure/data are retained; new characters are used for the successful run.

Coverage: real Lua fields/events/logout/reload, actual RappelzDebug, client privacy/malformed/unavailable previews, independent mutations for foreign/missing attributes, misleading labels, duplicate callbacks and stale lobby state. Combined **204 Rust / 127 Python** tests pass (one Rust ignored, 26 Lua included). Preservation **9/9 original**, **8/8 copied pre-existing**; installed images/WTF and original services unchanged. Credentials, clones, binaries, captures and backups stay ignored. Evidence: `client-data/validation/2026-11-01-job-base-addons-8d71c920/verified/`.

No new decompilation/effective stat formula is claimed. Resource provenance: [JOB_PROGRESSION_VALIDATION.md](JOB_PROGRESSION_VALIDATION.md); explicit software/game versions: [VERSIONS.md](VERSIONS.md).
