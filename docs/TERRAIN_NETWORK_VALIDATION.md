# Native terrain validation during stalled and reset TCP connections

Historical stalled/reset TCP cycle. The later [forced worldserver restart validation](TERRAIN_RESTART_VALIDATION.md) kills and restarts an actual isolated world process against copied persistent data. Counts, hashes and untested limits below belong to this earlier network cycle.

Verified **2026-10-31** with two Windows x64 clients, the existing loopback database/auth/world services and new ordinary GM-0 fixtures **32/33**. This follows [WARP/map-change validation](TERRAIN_WARP_VALIDATION.md). No server process is stopped, and no server/gameplay/schema/asset change is made.

## Scope and implementation

A new opt-in `-validate-terrain-network` scenario routes only its world connection through a separate local relay. Auth still uses the existing service and one-time-key login. The client accepts `-validate-world-proxy` only for this explicit scenario, with loopback auth/advertised-world/proxy endpoints and dedicated LUA*/Lua* identities. Ordinary sessions retain the advertised endpoint. Three Rust tests reject invalid routes, ordinary accounts, remote/non-numeric addresses and port zero.

The Python relay forwards opaque TCP bytes without decrypting, recording or modifying payloads. Two ephemeral loopback listeners isolate owner and witness; the upstream is the existing 127.0.0.1:4514 server. Each direction retains at most one **64-KiB relay block** outside asyncio/OS buffers and applies normal drain backpressure. Diagnostic history is bounded to **256 metadata events/role** and records connection generations, monotonic times, forwarded-byte counts and held-block sizes only. One role pauses at a time; the other stays connected to the same real server.

The existing client network-error/EOF path already drops the production world. This scenario proves that its terrain worker cancels and joins and that reauthentication produces a new world/handle. PacketTrace adds bounded private-probe and transport-error observations, sharing its existing 2048-entry budget. Decoder errors remain separate from deliberately induced transport errors. The test driver retains metadata audit handles across world teardown and binds cancellation events to the paused interval by audit cursor/instance. It does not add automatic reconnect, a heartbeat or an inactivity timeout to gameplay.

## Real two-client scenario

Both new players remain at **8440/7227** and **8440/7247**, level 1, HP 100/MP 50, zero XP/JP/money and layer 0. Separate client roots/preferences share original read-only Data resources.

1. Join normally, verify the peer and initial fully loaded view; capture ground/full-scene images and Windows counters.
2. While the owner's terrain blend is running, pause both forwarding directions of its real encrypted world socket. After relay acknowledgement, each player sends one private probe to the other.
3. Hold the pause for at least eight seconds. Both probes remain undelivered, no bytes are forwarded, and the peer remains visible with the same handle. The affected client keeps rendering and proves three actual partial blend cancellations during camera turns, without worker sleeps or held-result shortcuts.
4. Resume the stream. Both probes arrive exactly once, with no decoding errors, reconnect or lost cipher/framer state.
5. During another actual partial blend, remotely reset the owner's two relay sockets. The client receives a genuine transport error, drops/joins the world and returns to Login. The witness sees its old handle leave. Reauthenticate through the same relay, verify a fresh handle/worker and authoritative peer reappearance.
6. Perform the same stall/resume/reset/relogin sequence for the witness while the owner observes.
7. Restore both default views, capture recovered images/counters, and use normal ReturnLobby. Both processes exit 0.

There is no `NetCmd::Disconnect` in this driver. Commands crossing the real sockets are ordinary private chat/login/lobby requests. No gameplay state or packet is injected locally. Expected reset errors are captured before the test initiates login; unexpected errors in other phases fail validation.

## Correct reset semantics on Windows

The first retained relay run used `transport.abort()` with linger zero. Python's Windows Proactor transport calls `shutdown(SHUT_RDWR)` before closing, so native clients received clean EOF instead of an error. A real-socket regression now **requires ConnectionResetError/ConnectionAbortedError** and rejects EOF. The relay cancels pending transport I/O first, then closes its underlying owned linger-zero socket synchronously before the scheduled connection-lost callback can perform graceful shutdown. This also avoids invalid-handle overlapped-I/O warnings seen in an intermediate close-before-cancel attempt.

The final run has **one actual transport error and zero clean-disconnect events per client**, and empty relay/client stderr. The tests exercise the same Windows reset implementation and successful new-connection byte ordering. The relay currently relies on CPython's owned TransportSocket wrapper for this explicit diagnostic; this is verified on the installed Windows/Python runtime, not a portable transport-abort guarantee.

## Results

The independent comparison passes **41/41 checks**. Both final native processes exit 0, and the relay shuts down normally.

| Observation | Owner | Witness |
|---|---:|---:|
| Actual forwarding pause | 8047.0 ms | 8032.0 ms |
| Rendered frames during own pause | 424 | 385 |
| Partial camera cancellations inside pause | 3 | 3 |
| Progress at reset-triggered shutdown | 433 / 1024 rows | 490 / 1024 rows |
| World logins / Lua entry / Lua logout | 2 / 2 / 2 | 2 / 2 / 2 |
| Worker lifetimes joined | 2 | 2 |

Across both clients, **eight invalidated jobs** have no CPU cache acceptance or GPU upload: six actual partial view cancellations during the paused intervals and two partial shutdown interruptions after remote reset. All **four world workers join**; observed shutdown-begin-to-join intervals are **0.155–1.670 ms**, not end-to-end reconnect latency. Relay metadata proves constant up/down forwarded-byte totals throughout each pause, nonempty held blocks in both directions and resumed forwarding before reset. Connection generations 1/2 have matched connected/closed events.

Both recovered ground images are **byte-identical** to initial images. All **four initial/recovered captures** also match the corresponding home view from the previously installed client. The recovered full scene was visually inspected, with both players and live RappelzDebug state. Vitals, XP/JP/money, position and layer remain unchanged; actual server ENTER/LEAVE snapshots prove old-handle cleanup and fresh-handle reappearance. Both private probes per client arrive once after their corresponding resume.

The checker rejects a short/leaking pause, early or duplicate probes, stalled rendering, cancellations outside the held interval, clean EOF misreported as reset, missing partial shutdown, reused/unjoined workers, incorrect peer re-entry, changed images and resource/counter overflow. Four relay tests cover real bidirectional holding/order, required reset errors plus fresh connections, loopback/control guards and bounded metadata. Four additional Rust tests cover route guards and trace capacity. Four comparison tests cover success and failure mutations.

## Memory, data and publication

The existing limits remain **128 MiB cached CPU pixels/client**, one outstanding blend, at most one **4 MiB** working/result image and four GPU uploads/frame. Observed CPU cached pixels peak at **64 MiB/client** and GPU terrain payload at **60 MiB/client** in this stationary scene. These payloads exclude Assets/model/map caches, asyncio/OS buffers and graphics-driver allocations.

Twelve Windows samples were retained across initial/recovered holds. Sums of each role's per-phase medians, in MiB:

| Phase | Working Set | Private Bytes | GPU Total Committed |
|---|---:|---:|---:|
| initial | 839.98 | 9279.90 | 8522.42 |
| recovered | 844.49 | 9278.30 | 8549.42 |

These are whole-process/driver observations, not leak thresholds, total physical VRAM or an FPS benchmark. The driver records hundreds of frames during each pause, but does not benchmark whole-frame latency or gameplay under packet loss.

The pre-run SQL backup contains **18,915,756 bytes**, SHA-256 `6ae8c053a9082f4630ac7ed92cecedeb393fcee29370a44e99b22bd3f6a7c3f8`. Selected pre-existing account/character records through **31**, items through **4**, all world INSERTs and installed WTF compare unchanged: **9/9 preservation checks**. Fixtures 32/33 are retained offline at home with unchanged stats/settings and **GM-0 throughout**. Existing MariaDB/auth/world processes, server binary, schemas, original assets/MPQs and normal/companion DBCs are preserved. No original character or earlier fixture is reset.

Combined verification: **158 Rust passed, 0 failed, 1 ignored**, including **22 Lua tests**; **62 Python tests passed**. Independently tested publication workspaces: Core **73 Rust/62 Python**, client **126 Rust/62 Python** (one ignored Rust test). Six optimized executables are AMD64. Core shares diagnostic tools/docs; client adds only the guarded routing/driver and bounded trace observations. The existing production teardown path required no behavioral change. Installed native-tested client SHA-256: `f911647a3b70b1f4f8519c8487cb0c2c3b3aacd535397fcb75d29020185e5b03`.

## Reproduce and limits

```powershell
python tools/testing/terrain_fault_proxy.py <fresh-attempt-directory>
RappelzRust.exe -data <independent-root> -login <dedicated-account:password> -enter <dedicated-character> -validate-terrain-network <plan.json> -validate-world-proxy 127.0.0.1:<role-port>
python tools/testing/compare_terrain_network_clients.py <attempt-directory>
python -m unittest discover -s tools/testing -p 'test_*.py' -v
```

The relay writes owner/witness ports to `proxy-ready.json`. Each native plan supplies `role`, `peer` and the absolute shared `directory`. Clients request role-specific pause/resume/reset files, and the relay atomically acknowledges them with metadata. The runner waits for initial/recovered holds, takes counters, writes matching release files and writes `proxy-stop` after both clients exit. Lifetimes are bounded to 220 seconds/client and 240 seconds/relay, with bounded packet/frame/worker/proxy histories. No account passwords or payloads are stored in relay logs or published sources.

Final evidence is local and ignored under `client-data/validation/2026-10-31-terrain-network-c989e6bb/verified/`. Initial clean-EOF and intermediate reset experiments are retained separately, with their limitations above; they are not the final verified measurements. Credentials, original-asset screenshots, SQL/executable backups and runtime reports remain local.

This covers eight-second bidirectional forwarding stalls, opaque-stream resumption, forced per-connection TCP reset and reauthentication of both clients while the real server stays alive. It does not establish behavior after server crash/process kill, permanent silent black holes, heavy traffic/backpressure, active combat/movement during stalls, packet loss/reordering outside TCP, all maps/hardware, automatic timeout/reconnect or original-SFrame compatibility. Those remain separate work.
