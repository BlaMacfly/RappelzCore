# Network protocol implementation

Audit date: 2026-10-30. The detailed packet/function evidence remains in [re/network_protocol.md](re/network_protocol.md). [CLIENT_REVERSE_ENGINEERING.md](CLIENT_REVERSE_ENGINEERING.md) records executable identity and the limits of its competing version labels; [SERVER_REFERENCES.md](SERVER_REFERENCES.md) compares public implementations.

## Compatibility target and evidence

The implemented profile targets the supplied NoLimits SFrame SHA-256 `27f42fc8f6b08df8c239640b938d601aeaebb7b3db0c89fc47755325a98a410d`, with the version payload `201507080`. PE metadata, source-directory labels and launcher package labels do not establish an exact official Epic/patch version.

The detailed investigation distinguishes directly observed offsets/constructors from inferred field meanings. Preserve those distinctions when extending messages: a matching size does not validate every field's gameplay meaning. No original-client connection or packet capture was made during this audit. Historical Rust bot logs demonstrate the local Rust implementations communicating with one another, not proof of interoperability with the original game.

## Shared Rust implementation

`shared/rz-proto` is used by both services, the graphical client and `rz-bot`. It contains stream encryption, TCP reader/writer, framing, encoding and messages in `msg.rs` / `msg_game.rs`. The client runs its Tokio network session separately from rendering and forwards decoded events through channels.

| Layer | Current behavior |
|---|---|
| Transport | TCP; separate send/receive cipher state per connection, recreated for auth and world. |
| Header | Seven bytes: total size `u32 LE`, message ID `u16 LE`, checksum `u8`. |
| Outgoing checksum | Sum of the first six header bytes modulo 256. |
| Incoming framing | Handles incomplete/concatenated packets, rejects total sizes outside 7–1,048,576 bytes, skips dummy ID 9999. The current framer does not verify received checksums. |
| Stream cipher | Local XRC4 reconstruction with modified KSA and 1013-byte discard; entire packets, including headers, are encrypted. |
| Password/key protection | RSA-1024 public key exchange with PKCS#1 v1.5 protection of AES key/IV, then AES-128-CBC with PKCS#7 padding. IV is restored per protected message. |
| Strings | Fixed-width/variable byte strings, configurable legacy code page; default Windows-1252. |

The public rzu KSA differs from the local reconstructed recurrence. Do not switch algorithms based on a similar Epic label. Byte-level binary evidence or an authorized capture is needed to resolve that difference.

## Login sequence

```text
Client -> auth:  VERSION(10001), RSA_PUBLIC_KEY(71)
Auth   -> client: AES_KEY_IV(72)
Client -> auth:  ACCOUNT(10010)
Auth   -> client: RESULT(10000)
Client -> auth:  SERVER_LIST(10021)
Auth   -> client: SERVER_LIST(10022)
Client -> auth:  SELECT_SERVER(10023)
Auth   -> client: SELECT_SERVER(10024), AES-protected one-time key

Client opens chosen world connection:
Client -> world: VERSION(51), ACCOUNT_WITH_AUTH(2005)
World  -> client: RESULT(0)
Client -> world: CHARACTER_LIST(2001)
World  -> client: CHARACTER_LIST(2004)
Client -> world: LOGIN(62)
World  -> client: LOGIN_RESULT(64), state/visibility updates, time sync
```

Authentication checks stored salted verifiers after decrypting the supplied password. This is not a WoW SRP proof exchange. The world service validates and clears the auth-to-world key through `acore_auth`; see [DATABASE_ARCHITECTURE.md](DATABASE_ARCHITECTURE.md).

Selected implemented total packet sizes, backed by `msg::tests::observed_sizes` and the prior constructor investigation:

| Message | ID | Total bytes |
|---|---:|---:|
| Auth VERSION | 10001 | 27 |
| ACCOUNT, documented country profile | 10010 | 149 |
| Auth SERVER_LIST | 10022 | `11 + 302 * server_count` |
| Auth SELECT_SERVER result | 10024 | 41 |
| World ACCOUNT_WITH_AUTH | 2005 | 76 |
| CREATE_CHARACTER | 2002 | 639 |
| LOGIN | 62 | 287 |
| LOGIN_RESULT | 64 | 97 |
| MOVE_REQUEST | 63 | `26 + 8 * point_count` |

The embedded legacy message-name table uses some older IDs; observed constructors/parsers take precedence. Public version-aware definitions also use 62/63/64, so these differences alone do not establish private opcode modifications. The 27-byte world-version packet differs from some public 9.5.2 definitions, which append a checksum buffer; exact version compatibility remains unresolved.

## Gameplay coverage and limits

The source has message definitions for inventory/equipment, NPC dialogs and shops, skills/skill lists, quests, stats, combat and additional gameplay. `worldserver/session.rs`, `gameplay.rs` and `commands.rs` implement selected behavior; message existence is not proof that the corresponding full game system works. Decoding support and server simulation coverage must be tracked separately.

The [respawn regression](RESPAWN_VALIDATION.md) additionally verifies ordered monster LEAVE/ENTER and a second real kill with two clients. QUERY now requires an announced object that remains visible by live distance/layer/corpse state. Pending player attacks recheck that same visibility, and monster retaliation stops on layer changes. The native pair verifies hidden-corpse query denial; range/layer transitions currently have server-test coverage.

Existing protocol tests cover cipher vectors, framing/dummy packets, fixed strings, observed packet lengths, message round trips, entity IDs and RSA/AES round trips. Their latest executed results are recorded in [DEVELOPMENT_STATUS.md](DEVELOPMENT_STATUS.md). A round trip between two Rust endpoints can share the same mistake; retain independent binary offsets/vectors and add captured fixtures when available.

Historical `client-data/bot1.log` and `bot2.log` show simultaneous local login, world entry, nearby NPC/player visibility, movement, chat and clean disconnect. They are useful restart evidence, not a fresh test run or coverage of all gameplay. Further work includes original-client interoperability evidence, explicit protocol-profile fixtures, parser robustness checks and verification of inferred gameplay fields.
