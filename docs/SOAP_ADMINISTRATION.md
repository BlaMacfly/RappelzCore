# SOAP remote administration

Implemented and tested **2026-11-01**, RappelzCore **0.1.0 development**. The worldserver accepts **SOAP 1.1 `executeCommand(command)`** over HTTP Basic authentication, with **`urn:AC`** and **`urn:TC`** namespaces. This exposes RappelzCore console commands; it does not import the other projects' commands/game systems.

The interface follows primary [AzerothCore documentation](https://github.com/azerothcore/wiki/blob/master/docs/remote-access.md) and the [TrinityCore implementation](https://github.com/TrinityCore/TrinityCore/blob/3.3.5/src/server/worldserver/TCSoap/TCSoap.cpp): authenticated command RPC, console output in `result`, SOAP Fault on failure, administrator access. Authentication uses existing SRP salt/verifier and current realm/global GM access on every request. No schema or credential storage is added.

## Enable

Add to your own `worldserver.conf`, then restart that worldserver:

```ini
SOAP.Enabled = 1
SOAP.IP = "127.0.0.1"
SOAP.Port = 7878
```

Defaults in `conf/worldserver.conf.dist` are **disabled**, loopback, port **7878**. Environment overrides: `AC_SOAP_ENABLED`, `AC_SOAP_IP`, `AC_SOAP_PORT`. An invalid IP/port or occupied enabled SOAP listener fails startup **before any online flags are reset**. Disabled SOAP opens no listener. Installed configuration is not automatically changed.

Use an existing administrator or create a dedicated account in the local console. Grant access with `account set gmlevel <account> 3 <realm-id>` or `account set gmlevel <account> 3 -1`. GM 0/1/2 and other-realm grants are refused. Account/IP bans and account IP locks apply. Revocation/password changes take effect on the next request. SOAP does not create a game session or change account online/session flags.

HTTP Basic has no transport encryption. Keep loopback for local tools; use HTTPS termination or a secure tunnel for remote access. The built-in listener does not provide TLS.

## Request

Send HTTP **POST**, authenticate with the administrator account, and use this UTF-8 body:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<SOAP-ENV:Envelope xmlns:SOAP-ENV="http://schemas.xmlsoap.org/soap/envelope/"
                   xmlns:ns1="urn:AC">
  <SOAP-ENV:Body>
    <ns1:executeCommand><command>server info</command></ns1:executeCommand>
  </SOAP-ENV:Body>
</SOAP-ENV:Envelope>
```

Replies contain `executeCommandResponse/result` with the matching AC/TC namespace and XML-escaped output. Content-Length/chunked requests and an optional command-leading dot work. SOAPAction is optional; the validated body selects the operation.

Commands: `help`, `server info`, `announce <text>`, `saveall`, `account create`, `account set password`, `account set gmlevel`, `server shutdown`/`exit`/`quit`. `saveall` acknowledges a request, not completion of every SQL commit. Player/target-dependent chat commands are not console commands. SOAP and local console execution share the serial world-loop dispatcher; no command shell is started.

| HTTP | Meaning |
|---|---|
| 200 | Successful command, output in `result`. |
| 401 / 403 | Missing/invalid credentials with Basic challenge / access denied. |
| 405 / 413 | Unsupported HTTP method / body too large. |
| 500 | Invalid SOAP or unsuccessful command, with SOAP Fault. |
| 503 | Authentication/console unavailable or queue full. |

SOAP 1.1 requires one supported `executeCommand` and one text `command`. SOAP 1.2, WSDL discovery, arbitrary SOAP headers, DTD/entities and nested command markup are unsupported. This is not full gSOAP. Limits: eight connections and eight queued commands; 32 headers with a 16 KiB buffer; 16 KiB body; 4096-byte command; five-second header and fifteen-second connection deadlines. SOAP logs exclude credentials/command arguments. A timeout after queueing can leave a modifying command's result uncertain: inspect state before retrying.

## Validation

Isolated copied world **24514**, SOAP **27878**, auth **24615**, MariaDB **3331**, two native clients: **25 HTTP/SOAP checks pass**, plus graceful SOAP shutdown. Both clients receive the same XML-escaped announcement as real Lua `CHAT_MSG_SYSTEM`; both namespaces report two players. Checks cover missing/bad credentials, player/other-realm refusal, bans/IP lock, self-demotion and immediate revocation, chunked/malformed/duplicate/oversized requests, unknown commands, real new-account creation/privilege/password changes, old-password refusal and new-password success. No original account is modified.

The first run exposed a real shutdown defect: Tokio's pending blocking stdin read kept the process alive after its SOAP success reply. A bounded channel from a detached standard-input thread fixes it. The final run keeps stdin open, receives the reply, and exits **0** without another newline. Failed-run evidence/data remain retained; the rerun uses new characters.

The same session passes [44 job-base/progression checks](JOB_BASE_ADDON_VALIDATION.md). Combined **204 Rust** tests pass (one scan ignored, 26 Lua included) and **127 Python** pass. An existing socket-timing test failed under concurrent load; its focused seven-test suite and complete rerun pass. Original preservation **9/9**, copied pre-existing content **8/8**. Test services stop; files/backups remain ignored under `client-data/validation/2026-11-01-job-base-addons-8d71c920/`. Installed images/configuration and original services are preserved. [VERSIONS.md](VERSIONS.md) distinguishes our software and the original game.
