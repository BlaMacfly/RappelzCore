# SOAP queue, interrupted calls and permission changes

Subsequent verification: [Two active clients at shutdown](SOAP_SHUTDOWN_VALIDATION.md) now covers actual unsaved movement, a six-second SQL delay, complete session draining and restored characters after restart. It fixes the previously untested two-second save grace with 65 checks and an old-server control. The tests below retain their original scope and counts.

Verified **2026-11-01**, RappelzCore **0.1.0 development**. **26 independent comparison checks pass**, together with all **25 existing SOAP checks** and **44 two-client progression/base checks**. The previous compiled server reproduces three concrete defects; the corrected server passes the corresponding real HTTP/SQL cases. No original game executable is run.

## Corrections

An authenticated command could wait behind another console command and then execute after its administrator access had been revoked. Each queued request now retains an authenticated account identity and salt/verifier proof, without the raw password. Immediately before console execution, the world loop reads the current account again and checks identity, credentials, realm/global administrator access, account/IP bans and IP lock. A queued permission/ban/IP change returns **403**, a changed password/account identity **401**, and unavailable authentication **503**. None of these refusals executes the console command. This is a dispatch-time check, not an atomic transaction combining arbitrary external administrator changes with the later command.

Shutdown previously depended on the HTTP callback reading the command result. Once its fifteen-second connection deadline expired, a later successful `server shutdown` could disappear with the closed reply channel. A shared completion state now records both successful shutdown execution and HTTP connection completion in either order. It signals once after both. Normal replies flush first; a dropped/expired connection still permits the accepted shutdown to complete. Cancellation drops the connection-completion guard, and failed/refused/non-shutdown commands never arm it. An already-stopped world also lets the SOAP listener exit immediately.

The shared console parser previously interpreted an invalid realm as global **-1**. It now accepts only the omitted/default global scope, explicit **-1**, or a positive signed-32-bit realm, and GM **0..3**. Malformed scopes, extra scope/create arguments and passwords over the existing sixteen-character limit fail without changing the target account/access rows. Local console and SOAP share these rules. The wire envelope, configuration defaults and game protocol remain unchanged.

## Controlled real failures

New copied schemas run on MariaDB **3331**, auth **24615**, world **24514**, SOAP **27878**. A separate SQL connection holds one **new fixture account row** while the world console's actual password UPDATE waits for that lock. This supplies an observed world-dispatch barrier without modifying existing content. SOAP's account-ID-only queue log confirms the corrected request is enqueued before the external change/release.

| Case | Previous server control | Corrected server |
|---|---|---|
| Realm text `invalid` | HTTP 200, new fixture granted GM 3 globally | HTTP 500; full account/access dump hashes unchanged. |
| Queued account create, then administrator demoted | HTTP 200; target account created | HTTP 403; target remains absent. |
| Queued request, then ban / IP lock / valid SRP password rotation | Separate current-proof regressions | HTTP 403 / 403 / 401; targets absent. New password works, old password fails. |
| Valid queued account creation after actual HTTP expiry | Timeout outcome may be ambiguous | No row before lock release; exactly one target row afterward. No driver retry. |
| Queued shutdown held beyond HTTP expiry | Connection closes around 15 seconds; server needs stdin assistance | Connection closes around 15 seconds; server exits 0 with stdin still open and no assistance. |
| Competing SOAP bind while two players are online | Existing startup ordering retained | Competing process fails; complete online/realm flag hashes and both online players remain unchanged. |

Seven invalid-argument cases preserve full hexadecimal account/access dump hashes. Extra create arguments and a truncated HTTP body create no account. A second ordinary shutdown run receives HTTP **200** with the full success result before exit **0**, while stdin remains open. The negative control uses the exact previously published executable hash and retains its intentionally demonstrated fixture changes.

Two ordinary native clients remain connected during the delayed-queue probes, then perform the existing JP **8→5→0**, JLV **1→2→3**, private-state checks and two normal lobby saves/reentries. Both exit **0**. Four direct SQL gates/final profiles preserve EXP, money **1000/2000**, HP/MP **280/140**, positions and unrelated settings. Both real Lua hosts receive the SOAP announcement. The expired shutdown is tested after their normal logout; this does not claim an active-character shutdown durability test.

Combined **212 Rust tests** pass (one exhaustive model scan ignored; 26 Lua tests included), and **132 Python tests** pass. Eight new server regressions cover both completion orders, dropped/aborted receivers, exact-once shutdown signaling, non-shutdown/refused results, credential-proof invalidation and strict realm parsing. Five independent comparison/mutation tests reject missing queue/lock evidence, unwanted effects, incomplete controls, premature timeouts, stdin assistance and changed online flags.

Original data/WTF preservation **9/9** and copied pre-existing rows **8/8** pass. Installed images/settings, original source assets, the three original service PIDs and original row maxima are unchanged. Copied services stop; their data/backups remain retained. A first diagnostic attempt stopped because the helper tried to decode binary account columns as Windows text. It is retained as failed; the corrected helper hashes hex-blob dumps, and no fixture progress reset was used for the successful rerun.

Evidence remains ignored under `client-data/validation/2026-11-01-soap-resilience-4a28e6b9/`, including `negative/`, `diagnostic/`, `verified/` and the normal-shutdown report. Source/tests/docs only are published. See [SOAP configuration](SOAP_ADMINISTRATION.md), [raw-base validation](JOB_BASE_ADDON_VALIDATION.md) and [explicit client/game versions](VERSIONS.md). Persistent idempotency/replay protection, atomic authorization fencing and active-player failure durability remain separate work; a timeout still requires inspecting state before retrying a modifying command.
