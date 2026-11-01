# SOAP shutdown with two active clients

Verified **2026-11-01**, RappelzCore/RappelzRust **0.1.0 development**. **65 independent checks pass** across a previous-server control, a normal shutdown and a shutdown with six seconds of blocked character saves. Both corrected scenarios save complete characters before process exit, then restart and restore them through actual native-client authentication. Original data is preserved.

## Defect and correction

The worldserver previously broadcast shutdown, slept **two seconds**, then exited without joining player sessions. A successful SOAP reply therefore did not establish that character saves had completed. The exact previous executable, SHA-256 `b8553e88c6c1d565df4a528028d049eaf8331b325d7cc20c7ba593fbeba62d42`, reproduces exit **0** while two real save transactions are still blocked. Their uncommitted movement rolls back and online flags remain set.

Sessions now run in a tracked `JoinSet`. Shutdown stops game/SOAP admission and the world/status tickers, marks the realm offline, and waits for each session's character saves, writer cleanup and account offline update. There is no fixed sleep standing in for save completion. A session starting after shutdown was signaled also observes the current signal value immediately. Writer cleanup has a bounded wait after saving, so a socket cannot hold shutdown indefinitely after its character commits.

`ShutdownTimeout = 30` in `conf/worldserver.conf.dist` controls the session-drain phase in seconds, clamped to **5..300**. A save failure, task failure or drain timeout produces a **nonzero process exit** and an incomplete-shutdown error. Other sessions are allowed to finish before an already-observed failure is reported; at the deadline the remaining futures are aborted/joined. No stale snapshot retry or successful-save claim is made after a failed/uncertain write. This is not a guarantee for every database/storage/host failure or a maximum duration for unrelated database operations outside that phase.

SOAP still returns **HTTP 200** for an accepted `server shutdown`. Its reply flushes before shutdown signaling; it acknowledges the command, not all later SQL commits. The successful process exit and the independent database/reentry checks establish completion here. The existing AC/TC envelopes and permission/expired-reply handling are retained.

## Real sessions and database proof

After the user's PC reboot, the existing primary database/auth/world services were resumed using their existing images/configurations. Original hashes and nine original-data/settings checks passed. Their new post-reboot PIDs were recorded and stayed unchanged throughout the isolated validation; pre-reboot PIDs are not claimed to survive a reboot.

Fresh copied schemas use MariaDB **3331**, auth **24615**, world **24514**, SOAP **27878**. Three pairs of new GM-0 characters are used; no pair is reset to rerun progress. Each starts at level **10**, EXP **0**, HP/MP **280/140**, money **1000/2000**, jobs **100/200**, JLV **1/1**, JP **8/0**. Owner buys JLV **2** for **3 JP**, leaving **5**; direct SQL verifies that durable checkpoint before movement.

Both clients then use production movement commands and stop twelve units east/eight units north. The owner is at **9452/7235**, witness **9432/7255**, with synchronized peer positions. SQL still holds **9440/7227** and **9420/7247** immediately before SOAP; neither client requests a lobby/save first, and the periodic interval is fifteen minutes. The shutdown therefore has genuine unsaved character state to persist. SOAP `server info` independently reports **two online players** immediately before the shutdown command.

| Scenario | Observed result |
|---|---|
| Previous server, saves held | Both actual transactions modify character rows, then wait on `REPLACE INTO character_settings`. The process exits 0 while locks remain held; movement is not saved. This is the expected failing product control. |
| Corrected server, normal SQL | Exit 0 after confirmed saves. Before restart, complete moved profiles and both character/account offline flags are verified directly. |
| Corrected server, delayed SQL | An independent transaction locks only the two new characters' settings rows for **six seconds**. The world stays alive until release, then exits 0 with both complete profiles saved. |

For both held-save cases, two distinct InnoDB transactions have modified rows and are observed waiting on the guarded settings lock. One committed-read query still sees the prior full checkpoint, while a dirty-read query sees both pending new positions. This proves actual in-flight saves rather than a controller sleep. The negative control's last committed checkpoint and all evidence are retained.

After each corrected shutdown, direct SQL verifies moved positions, EXP, wallets, HP/MP, JLV/JP, every unrelated settings key and offline flags **before restarting**. Both native clients then reauthenticate into a new world process and load the saved state. Numeric handles may be reused across server epochs; they are not treated as identity proof. A second SQL gate checks both restored online profiles, then the clients perform normal lobby saves and exit **0**. The second world also exits **0** through SOAP.

RappelzDebug observes one real shutdown logout and one final lobby logout per corrected client, with the last HP/MP/wallet/JLV/JP still readable. Reentry does not replay the purchase, EXP or money. Owner's JP/JLV callbacks count **3/3**, witness **2/2**, with no character-level-up event. Both reentry screenshots were visually inspected. The driver uses real network/game helpers and rendering; physical-keyboard interaction is not an acceptance claim.

## Checks and preservation

Combined **216 Rust tests** pass (one exhaustive scan ignored, 26 Lua tests included), and **138 Python tests** pass. Four new server regressions verify waiting for every session, finishing another save after a failure, reporting/cleaning up a timeout, and retaining a failure collected concurrently with the shutdown signal. Six independent comparer/mutation tests reject lost movement despite a success flag, missing offline commits, lost settings/wallets, a non-reproducing old control, unobserved SQL delay and replayed/synthetic reentry evidence.

Original data/WTF preservation **9/9** and copied pre-existing content **8/8** pass. All six new fixture profiles remain retained and GM-0; startup recovery clears the old control's stale online flags without changing its saved checkpoint. Installed images/configurations, original source assets and primary services remain unchanged during validation. Test services stop and their files/backups are retained. The first diagnostic attempt lacked packet tracing for the new test flag; it is retained as failed, corrected before any purchase/movement, and no progress reset was needed.

Only source/tests/docs are published. Credentials, copied databases, binaries, captures and backups stay ignored under `client-data/validation/2026-11-01-soap-player-shutdown-72fb3e40/`. See [SOAP setup](SOAP_ADMINISTRATION.md), [queued-call validation](SOAP_RESILIENCE_VALIDATION.md) and [explicit client/game versions](VERSIONS.md). Active combat/death at shutdown, database disappearance during drain, whole-host/storage failure and persistent request replay identity remain separate tests.
