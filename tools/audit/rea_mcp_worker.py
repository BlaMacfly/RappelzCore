"""A resumable, read-only native-analysis worker using REA's public MCP tools.

The worker launches the caller-selected Node/REA launcher, never the target.
It owns only its Node process and audit files. Original files and unrelated
processes/services are not changed. Each successful operation is committed by
an immutable response plus a hash-bound completion marker.
"""
from __future__ import annotations

import argparse
import contextlib
from dataclasses import dataclass
import hashlib
import json
import os
from pathlib import Path
import queue
import re
import shutil
import subprocess
import sys
import tempfile
import threading
import time
import uuid


LABEL = re.compile(r"[a-z0-9_-]{1,80}")
SHA256 = re.compile(r"[0-9a-f]{64}")
EVIDENCE_ID = re.compile(r"ev_[0-9a-f]{64}")
STATIC_TOOLS = frozenset({
    "binary_overview", "binary_session", "list_procedures", "search_procedures",
    "procedure_info", "procedure_pseudo_code", "procedure_assembly",
    "procedure_callers", "procedure_callees", "procedure_references",
    "resolve_containing_procedure", "list_segments", "list_strings",
    "search_strings", "read_bytes", "address_to_file_offset", "xrefs",
    "analyze_function", "batch_decompile", "inspect_native_api",
    "trace_native_values", "inspect_analysis_view", "get_evidence_bundle",
    "export_evidence_bundle",
})


class WorkerError(RuntimeError):
    """A failed operation or integrity constraint; no producer is retried."""


def require(condition, message):
    if not condition:
        raise WorkerError(message)


def file_sha256(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def read_json(path):
    with Path(path).open("r", encoding="utf-8") as stream:
        return json.load(stream)


def operation_digest(operation):
    value = {"tool": operation["tool"], "arguments": operation["arguments"]}
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":"),
                         ensure_ascii=False, allow_nan=False).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def audit_path(audit, path):
    audit = Path(audit).resolve(strict=True)
    path = Path(path)
    if not path.is_absolute():
        path = audit / path
    require(not path.is_symlink(), "Audit file symlinks are not admitted")
    resolved = path.resolve(strict=False)
    require(resolved.is_relative_to(audit) and resolved != audit,
            "Audit output escapes the audit directory")
    require(resolved.parent.is_dir(), "Audit output parent directory does not exist")
    return resolved


def atomic_json(path, value, *, mutable=False):
    """Commit immutable files without replacement, or replace mutable status."""
    path = Path(path)
    data = json.dumps(value, ensure_ascii=False, separators=(",", ":"),
                      allow_nan=False).encode("utf-8")
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(dir=path.parent, prefix=".worker-",
                                         suffix=".tmp", delete=False) as stream:
            temporary = Path(stream.name)
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        if mutable:
            # Windows readers can briefly deny replacement while observing
            # status. Retry this metadata rename only, never a producer or an
            # immutable file commit. The existing status remains intact if
            # the sharing conflict persists.
            for attempt in range(20):
                try:
                    os.replace(temporary, path)
                    break
                except PermissionError:
                    if attempt == 19:
                        raise
                    time.sleep(0.025)
        else:
            # A hard-link commit is atomic and fails if another file/symlink
            # already occupies the immutable name. No check/replace race.
            try:
                os.link(temporary, path)
            except FileExistsError as error:
                raise WorkerError("Refusing to overwrite immutable audit file: " + str(path)) from error
        if os.name != "nt":
            fd = os.open(path.parent, os.O_RDONLY)
            try:
                os.fsync(fd)
            finally:
                os.close(fd)
    finally:
        if temporary is not None:
            with contextlib.suppress(FileNotFoundError):
                temporary.unlink()


class WorkerLock:
    """A kernel-held lock released by process exit, without stale PID guessing."""
    def __init__(self, path):
        self.stream = Path(path).open("a+b")
        self.stream.seek(0, os.SEEK_END)
        if self.stream.tell() == 0:
            self.stream.write(b"\0")
            self.stream.flush()
        self.stream.seek(0)
        try:
            if os.name == "nt":
                import msvcrt
                msvcrt.locking(self.stream.fileno(), msvcrt.LK_NBLCK, 1)
            else:
                import fcntl
                fcntl.flock(self.stream.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        except OSError as error:
            self.stream.close()
            raise WorkerError("Another worker holds this audit directory's lock") from error

    def close(self):
        if not self.stream.closed:
            if os.name == "nt":
                import msvcrt
                self.stream.seek(0)
                msvcrt.locking(self.stream.fileno(), msvcrt.LK_UNLCK, 1)
            self.stream.close()

    def __enter__(self):
        return self

    def __exit__(self, *unused):
        self.close()


def validate_operation(operation, audit, available_tools=None):
    require(isinstance(operation, dict), "Operation must be an object")
    label, tool, arguments = (operation.get(k) for k in ("label", "tool", "arguments"))
    require(isinstance(label, str) and LABEL.fullmatch(label), "Unsafe operation label")
    require(isinstance(tool, str) and tool in STATIC_TOOLS, "Queued tool is not an admitted static analysis operation")
    require(isinstance(arguments, dict), "Operation arguments must be an object")
    operation_digest(operation)  # Also rejects non-JSON numeric values.
    if available_tools is not None:
        require(tool in available_tools, "Queued tool is not advertised by this REA server: " + tool)
    if tool == "export_evidence_bundle":
        require(isinstance(arguments.get("path"), str), "Export requires a file path")
        require(Path(arguments["path"]).is_absolute(), "Export paths must be absolute")
        require(arguments.get("overwrite") in (None, False), "Overwriting an Evidence export is forbidden")
        path = audit_path(audit, arguments["path"])
        require(not os.path.lexists(path), "Evidence export path already exists")
    return operation


def verify_completion(audit, operation, target_sha256):
    """Return a verified marker, None if absent, and never trust a filename alone."""
    label = operation["label"]
    marker_path = audit_path(audit, label + ".done.json")
    if not marker_path.exists():
        require(not os.path.lexists(marker_path), "Dangling completion-marker link")
        return None
    marker = read_json(marker_path)
    require(isinstance(marker, dict) and set(marker) == {
        "label", "tool", "operation_sha256", "artifact_sha256", "response_file",
        "response_sha256", "evidence_id"}, "Malformed completion marker")
    require(marker["label"] == label and marker["tool"] == operation["tool"], "Completion identity mismatch")
    require(marker["operation_sha256"] == operation_digest(operation), "Completion operation hash mismatch")
    require(marker["artifact_sha256"] == target_sha256, "Completion target hash mismatch")
    require(marker["response_file"] == label + ".json", "Unsafe completion response filename")
    require(isinstance(marker["response_sha256"], str) and SHA256.fullmatch(marker["response_sha256"]), "Malformed response hash")
    response_path = audit_path(audit, marker["response_file"])
    require(file_sha256(response_path) == marker["response_sha256"], "Immutable response hash mismatch")
    response = read_json(response_path)
    evidence_id = response.get("evidence_id") if isinstance(response, dict) else None
    require(marker["evidence_id"] == evidence_id, "Completion Evidence ID mismatch")
    require(evidence_id is None or isinstance(evidence_id, str) and EVIDENCE_ID.fullmatch(evidence_id), "Malformed Evidence ID")
    if evidence_id is not None:
        require(response.get("subject", {}).get("digest", {}).get("sha256") == target_sha256,
                "Completed Evidence target SHA-256 mismatch")
    return marker


class JsonValueStream:
    """Incremental complete JSON parsing, bounded per decoded value/record."""
    def __init__(self, stream, max_value_chars):
        self.stream, self.limit = stream, max_value_chars
        self.buffer, self.position, self.eof = "", 0, False
        self.decoder = json.JSONDecoder()

    def fill(self):
        chunk = self.stream.read(65536)
        self.buffer = self.buffer[self.position:] + chunk
        self.position = 0
        self.eof = not chunk

    def space(self):
        while True:
            while self.position < len(self.buffer) and self.buffer[self.position] in " \r\n\t":
                self.position += 1
            if self.position < len(self.buffer) or self.eof:
                return
            self.fill()

    def token(self, expected):
        self.space()
        require(self.position < len(self.buffer) and self.buffer[self.position] == expected,
                "Malformed exported Evidence bundle JSON")
        self.position += 1

    def value(self):
        self.space()
        while True:
            try:
                value, end = self.decoder.raw_decode(self.buffer, self.position)
                require(end - self.position <= self.limit, "Recovery record exceeds the configured memory bound")
                self.position = end
                return value
            except json.JSONDecodeError as error:
                require(not self.eof, "Incomplete or invalid exported Evidence JSON: " + str(error))
                require(len(self.buffer) - self.position <= self.limit,
                        "Recovery record exceeds the configured memory bound")
                self.fill()


def find_evidence_record(path, evidence_id, *, max_value_chars=128 * 1024 * 1024):
    """Validate all JSON, keeping at most one bundle record plus one match."""
    require(isinstance(evidence_id, str) and EVIDENCE_ID.fullmatch(evidence_id), "Invalid retained Evidence ID")
    matched, count, seen = None, 0, set()
    with Path(path).open("r", encoding="utf-8") as stream:
        parser = JsonValueStream(stream, max_value_chars)
        parser.token("{")
        while True:
            key = parser.value()
            require(isinstance(key, str) and key not in seen, "Duplicate/invalid bundle root key")
            seen.add(key)
            parser.token(":")
            if key == "records":
                parser.token("[")
                parser.space()
                while parser.position < len(parser.buffer) and parser.buffer[parser.position] != "]":
                    record = parser.value()
                    require(isinstance(record, dict), "Bundle record is not an Evidence object")
                    if record.get("evidence_id") == evidence_id:
                        matched, count = record, count + 1
                    del record
                    parser.space()
                    if parser.position < len(parser.buffer) and parser.buffer[parser.position] == "]":
                        break
                    parser.token(",")
                    parser.space()
                    require(parser.position < len(parser.buffer) and parser.buffer[parser.position] != "]", "Trailing bundle record comma")
                parser.token("]")
            else:
                parser.value()
            parser.space()
            if parser.position < len(parser.buffer) and parser.buffer[parser.position] == "}":
                parser.token("}")
                break
            parser.token(",")
        parser.space()
        require(parser.eof and parser.position == len(parser.buffer), "Trailing data in exported Evidence JSON")
    require("records" in seen and count == 1, "Retained Evidence ID is missing or duplicated in export")
    return matched


@dataclass
class WorkerOptions:
    audit_dir: Path
    target: Path
    node: Path
    rea_launcher: Path
    ghidra_install: Path
    java_home: Path
    snapshot: Path | None = None
    min_free_bytes: int = 5 * 1024 ** 3
    request_timeout: float = 1800
    idle_timeout: float = 1800
    poll_interval: float = 0.5
    shutdown_timeout: float = 15
    recovery_record_chars: int = 128 * 1024 * 1024


class McpClient:
    """Serialized JSON-RPC over the public Node REA stdio launcher."""
    def __init__(self, options, epoch, *, popen_factory=subprocess.Popen):
        self.options, self.epoch = options, epoch
        self.counter, self.responses = 0, queue.Queue()
        self.request_lock = threading.Lock()
        self.log = (options.audit_dir / (epoch + ".stderr.log")).open("xb")
        self.notifications = (options.audit_dir / (epoch + ".notifications.jsonl")).open("x", encoding="utf-8")
        environment = dict(os.environ)
        environment.update(GHIDRA_INSTALL_DIR=str(options.ghidra_install),
            JAVA_HOME=str(options.java_home), REA_ANALYSIS_PROVIDER="ghidra",
            REA_GHIDRA_STARTUP_TIMEOUT_MS="900000", JAVA_TOOL_OPTIONS="-XX:ActiveProcessorCount=2",
            GHIDRA_HEADLESS_MAXMEM="1536M", NODE_OPTIONS="--max-old-space-size=4096")
        environment["PATH"] = str(options.java_home / "bin") + os.pathsep + environment.get("PATH", "")
        flags = (subprocess.CREATE_NO_WINDOW | subprocess.BELOW_NORMAL_PRIORITY_CLASS) if os.name == "nt" else 0
        try:
            self.process = popen_factory([str(options.node), "--no-opt", str(options.rea_launcher), "--mcp"],
                env=environment, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=self.log,
                creationflags=flags, close_fds=True)
        except BaseException:
            self.log.close(); self.notifications.close()
            raise
        self.reader_thread = threading.Thread(target=self._read, name="rea-mcp-reader", daemon=True)
        self.reader_thread.start()

    def _read(self):
        try:
            for line in self.process.stdout:
                try:
                    self.responses.put(json.loads(line))
                except (ValueError, UnicodeError) as error:
                    self.responses.put(WorkerError("Invalid JSON on REA stdout: " + str(error)))
                    break
        finally:
            self.responses.put(None)

    def notify(self, method, params=None):
        value = {"jsonrpc": "2.0", "method": method}
        if params is not None:
            value["params"] = params
        self.process.stdin.write((json.dumps(value, ensure_ascii=False) + "\n").encode("utf-8"))
        self.process.stdin.flush()

    def request(self, method, params, timeout):
        with self.request_lock:
            self.counter += 1
            identity = self.counter
            data = {"jsonrpc": "2.0", "id": identity, "method": method, "params": params}
            self.process.stdin.write((json.dumps(data, ensure_ascii=False) + "\n").encode("utf-8"))
            self.process.stdin.flush()
            deadline = time.monotonic() + timeout
            while time.monotonic() < deadline:
                try:
                    item = self.responses.get(timeout=min(1, max(0.001, deadline - time.monotonic())))
                except queue.Empty:
                    continue
                if item is None:
                    raise WorkerError("REA stdio transport closed")
                if isinstance(item, Exception):
                    raise item
                if not isinstance(item, dict) or item.get("id") != identity:
                    self.notifications.write(json.dumps(item, ensure_ascii=False) + "\n")
                    self.notifications.flush()
                    continue
                if "error" in item:
                    raise WorkerError("REA JSON-RPC error: " + json.dumps(item["error"]))
                require("result" in item, "REA response has no result")
                return item["result"]
            raise WorkerError("REA request timed out; the producer will not be repeated")

    def close(self):
        with contextlib.suppress(OSError, ValueError):
            self.process.stdin.close()
        try:
            self.process.wait(timeout=self.options.shutdown_timeout)
        except subprocess.TimeoutExpired:
            self.process.terminate()
            try:
                self.process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait(timeout=5)
        self.reader_thread.join(timeout=5)
        self.process.stdout.close()
        self.log.close()
        self.notifications.close()
        require(not self.reader_thread.is_alive(), "Owned REA reader did not terminate")


def tool_error(envelope):
    if not envelope.get("isError"):
        return None
    for item in envelope.get("content", []):
        if item.get("type") == "text":
            with contextlib.suppress(ValueError, TypeError, AttributeError):
                error = json.loads(item["text"]).get("error")
                if isinstance(error, dict):
                    return error
    return {"message": "Malformed REA tool error envelope"}


def envelope_value(envelope):
    if "structuredContent" in envelope and envelope["structuredContent"] is not None:
        return envelope["structuredContent"]
    for item in envelope.get("content", []):
        if item.get("type") == "text":
            with contextlib.suppress(ValueError, TypeError):
                return json.loads(item["text"])
    raise WorkerError("REA response did not contain complete JSON")


class ReaWorker:
    def __init__(self, options, *, client=None):
        self.options = options
        self.audit = Path(options.audit_dir).resolve(strict=True)
        require(self.audit.is_dir(), "audit-dir must already exist")
        options.audit_dir = self.audit
        self.target = Path(options.target).resolve(strict=True)
        require(self.target.is_file(), "target must be a file")
        options.target = self.target
        self.target_sha = file_sha256(self.target)
        self.validate_runtime_paths(require_existing=client is None)
        self.epoch = "e-" + format(time.time_ns(), "x") + "-" + uuid.uuid4().hex[:8]
        self.client = client
        self.available, self.recoveries = set(), 0

    def validate_runtime_paths(self, *, require_existing):
        """Reject a target accidentally supplied as Node or the launcher.

        This is a configuration-confusion check, not publisher verification.
        Test clients can omit runtime files because they launch no process.
        """
        for role, value in (("Node executable", self.options.node),
                            ("REA launcher", self.options.rea_launcher)):
            candidate = Path(value).resolve(strict=require_existing)
            if not candidate.exists() and not require_existing:
                continue
            require(candidate.is_file(), role + " must be a file")
            require(candidate != self.target and not os.path.samefile(candidate, self.target),
                    role + " must not be the target or a hardlink to it")
            require(file_sha256(candidate) != self.target_sha,
                    role + " contains the target bytes; refusing to launch it")

    def path(self, name):
        return audit_path(self.audit, name)

    def status(self, phase, **fields):
        atomic_json(self.path("worker-status.json"), dict(epoch=self.epoch, phase=phase,
            artifact_sha256=self.target_sha, time=time.time(), **fields), mutable=True)

    def disk_guard(self):
        require(shutil.disk_usage(self.audit).free >= self.options.min_free_bytes,
                "Audit disk has less than the configured free-space reserve")

    def request_tool(self, name, arguments, label, *, recover=True, timeout=None):
        self.status("request", label=label, tool=name)
        try:
            envelope = self.client.request("tools/call", {"name": name, "arguments": arguments},
                                           self.options.request_timeout if timeout is None else timeout)
        except Exception as error:
            # This is explicitly a worker-side transport/request failure, not
            # forged producer Evidence. A controller can stop immediately
            # rather than waiting forever for a marker that cannot arrive.
            atomic_json(self.path(label + ".envelope.json"), {
                "isError": True, "worker_generated": True,
                "content": [{"type": "text", "text": json.dumps({"error": {
                    "code": "worker_client_failure", "message": str(error),
                    "producer_completion": "unknown", "producer_repeated": False}})}]})
            raise
        require(isinstance(envelope, dict), "REA tool response is not an envelope")
        atomic_json(self.path(label + ".envelope.json"), envelope)
        error = tool_error(envelope)
        if error:
            details = error.get("details") or {}
            reference = (details.get("reported_limits") or {}).get("evidence_reference") or {}
            retained = error.get("code") == "resource_constraint" and details.get("resource") == "transport" and reference.get("kind") == "retained-evidence"
            if not recover or not retained:
                raise WorkerError(name + " failed: " + json.dumps(error, ensure_ascii=False))
            value = self.recover_record(name, arguments, reference.get("evidence_id"), label)
        else:
            value = envelope_value(envelope)
        atomic_json(self.path(label + ".json"), value)
        self.status("response", label=label, tool=name)
        return value

    def recover_record(self, tool, arguments, identity, label):
        require(isinstance(identity, str) and EVIDENCE_ID.fullmatch(identity), "Invalid retained Evidence reference")
        require({"get_evidence_bundle", "export_evidence_bundle"} <= self.available,
                "This REA server does not advertise retained Evidence recovery tools")
        self.status("recovering", label=label, tool=tool, evidence_id=identity)
        self.recoveries += 1
        prefix = self.epoch + ".recovery-" + str(self.recoveries)
        self.request_tool("get_evidence_bundle", {"detail": "summary", "filters": {"evidence_id": identity}},
                          prefix + ".summary", recover=False)
        bundle = self.path(prefix + ".bundle.json")
        self.request_tool("export_evidence_bundle", {"path": str(bundle)}, prefix + ".export", recover=False)
        record = find_evidence_record(bundle, identity, max_value_chars=self.options.recovery_record_chars)
        require(record.get("operation") == tool and record.get("parameters") == arguments,
                "Retained Evidence operation does not match the producer request")
        require(record.get("subject", {}).get("digest", {}).get("sha256") == self.target_sha,
                "Retained Evidence belongs to another target")
        return record

    def load_plan(self):
        plan = read_json(self.path("plan.json"))
        require(isinstance(plan, dict) and isinstance(plan.get("sealed"), bool)
                and isinstance(plan.get("operations"), list), "Invalid plan.json")
        labels = []
        for operation in plan["operations"]:
            # An already completed export necessarily has an existing output;
            # the immutable marker is checked before its path nonexistence.
            label = operation.get("label") if isinstance(operation, dict) else None
            require(isinstance(label, str) and LABEL.fullmatch(label), "Unsafe operation label")
            labels.append(label)
        require(len(labels) == len(set(labels)), "Plan contains duplicate labels")
        return plan

    def run(self):
        opened, closed, error = False, False, None
        with WorkerLock(self.path("worker.lock")):
            try:
                self.disk_guard()
                if self.client is None:
                    # Recheck after acquiring the lock, immediately before
                    # process construction, as options/files may have changed.
                    self.validate_runtime_paths(require_existing=True)
                    launcher = Path(self.options.rea_launcher).resolve(strict=True)
                    require(launcher.suffix.casefold() in {".js", ".mjs", ".cjs"}, "Expected a public REA JavaScript launcher")
                    self.client = McpClient(self.options, self.epoch)
                init = self.client.request("initialize", {"protocolVersion": "2025-06-18",
                    "capabilities": {}, "clientInfo": {"name": "rappelz-static-worker", "version": "1"}},
                    self.options.request_timeout)
                atomic_json(self.path(self.epoch + ".initialize.json"), init)
                self.client.notify("notifications/initialized")
                catalog = self.client.request("tools/list", {}, self.options.request_timeout)
                require(isinstance(catalog, dict) and isinstance(catalog.get("tools"), list), "Invalid REA tool catalog")
                atomic_json(self.path(self.epoch + ".tools.json"), catalog)
                self.available = {tool["name"] for tool in catalog["tools"] if isinstance(tool, dict) and isinstance(tool.get("name"), str)}
                require({"open_binary", "export_evidence_bundle", "close_binary"} <= self.available,
                        "REA server lacks required lifecycle tools")
                arguments = {"path": str(self.target), "provider_id": "ghidra"}
                if self.options.snapshot is not None:
                    arguments["snapshot_path"] = str(Path(self.options.snapshot).resolve(strict=True))
                opened_value = self.request_tool("open_binary", arguments, self.epoch + ".open", recover=False)
                opened = True
                require(isinstance(opened_value, dict) and opened_value.get("result", {}).get("sha256") == self.target_sha,
                        "Opened target SHA-256 differs from the supplied file")
                idle = time.monotonic()
                completed = {}
                while True:
                    plan = self.load_plan()
                    pending = []
                    for operation in plan["operations"]:
                        previous = completed.get(operation["label"])
                        if previous is not None:
                            require(previous["operation_sha256"] == operation_digest(operation),
                                    "Completed operation changed within the current worker plan")
                            continue
                        done = verify_completion(self.audit, operation, self.target_sha)
                        if done is not None:
                            # Check tool admission too, without rejecting an
                            # already committed export because its file exists.
                            require(operation.get("tool") in STATIC_TOOLS and operation["tool"] in self.available,
                                    "Completed operation is not an admitted advertised static tool")
                            completed[operation["label"]] = done
                            continue
                        validate_operation(operation, self.audit, self.available)
                        pending.append(operation)
                    if not pending:
                        if plan["sealed"]:
                            # Fresh resumes always authenticate each marker;
                            # inside one process, avoid quadratic corpus reads.
                            # Revalidate the final immutable files before export.
                            for operation in plan["operations"]:
                                require(verify_completion(self.audit, operation, self.target_sha) is not None,
                                        "Completion disappeared before final export")
                            break
                        require(time.monotonic() - idle < self.options.idle_timeout,
                                "Unsealed queue exceeded its idle timeout")
                        time.sleep(self.options.poll_interval)
                        continue
                    operation = pending[0]
                    self.disk_guard()
                    require(file_sha256(self.target) == self.target_sha, "Target changed before queued operation")
                    label = operation["label"]
                    for suffix in (".request.json", ".envelope.json", ".json"):
                        require(not os.path.lexists(self.path(label + suffix)),
                                label + ": unmarked operation output exists; producer will not be repeated")
                    atomic_json(self.path(label + ".request.json"), dict(operation=operation,
                        operation_sha256=operation_digest(operation), artifact_sha256=self.target_sha))
                    value = self.request_tool(operation["tool"], operation["arguments"], label)
                    require(file_sha256(self.target) == self.target_sha, "Target changed during queued operation")
                    response = self.path(label + ".json")
                    identity = value.get("evidence_id") if isinstance(value, dict) else None
                    require(identity is None or isinstance(identity, str) and EVIDENCE_ID.fullmatch(identity), "Malformed producer Evidence ID")
                    if identity is not None:
                        require(value.get("subject", {}).get("digest", {}).get("sha256") == self.target_sha,
                                "Producer Evidence target SHA-256 mismatch")
                    marker = dict(label=label, tool=operation["tool"],
                        operation_sha256=operation_digest(operation), artifact_sha256=self.target_sha,
                        response_file=response.name, response_sha256=file_sha256(response), evidence_id=identity)
                    atomic_json(self.path(label + ".done.json"), marker)
                    completed[label] = marker
                    idle = time.monotonic()
                self.request_tool("export_evidence_bundle", {"path": str(self.path(self.epoch + ".final.bundle.json"))},
                                  self.epoch + ".final-export", recover=False)
                self.request_tool("close_binary", {"snapshot_path": str(self.path(self.epoch + ".snapshot.json"))},
                                  self.epoch + ".close", recover=False)
                closed = True
                require(file_sha256(self.target) == self.target_sha, "Target changed during worker session")
                self.status("complete", target_unchanged=True)
            except BaseException as caught:
                error = caught
                with contextlib.suppress(Exception):
                    self.status("error", error_type=type(caught).__name__, message=str(caught))
                    atomic_json(self.path(self.epoch + ".error.json"), dict(type=type(caught).__name__, message=str(caught)))
                raise
            finally:
                if opened and not closed:
                    with contextlib.suppress(Exception):
                        self.request_tool("export_evidence_bundle", {"path": str(self.path(self.epoch + ".partial.bundle.json"))},
                                          self.epoch + ".partial-export", recover=False,
                                          timeout=min(60, self.options.request_timeout))
                    with contextlib.suppress(Exception):
                        self.request_tool("close_binary", {}, self.epoch + ".partial-close", recover=False,
                                          timeout=min(60, self.options.request_timeout))
                if self.client is not None:
                    try:
                        self.client.close()
                    except BaseException:
                        if error is None:
                            raise
                if error is not None:
                    with contextlib.suppress(Exception):
                        self.status("error", error_type=type(error).__name__, message=str(error),
                                    target_unchanged=file_sha256(self.target) == self.target_sha)


def run_worker(options, *, client=None):
    return ReaWorker(options, client=client).run()


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("audit-dir", "target", "node", "rea-launcher", "ghidra-install", "java-home"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--snapshot", type=Path)
    parser.add_argument("--min-free-gib", type=float, default=5)
    parser.add_argument("--request-timeout", type=float, default=1800)
    parser.add_argument("--idle-timeout", type=float, default=1800)
    args = parser.parse_args(argv)
    try:
        require(args.min_free_gib >= 0 and args.request_timeout > 0 and args.idle_timeout > 0,
                "Invalid resource/time limits")
        options = WorkerOptions(audit_dir=args.audit_dir, target=args.target, node=args.node,
            rea_launcher=args.rea_launcher, ghidra_install=args.ghidra_install, java_home=args.java_home,
            snapshot=args.snapshot, min_free_bytes=int(args.min_free_gib * 1024 ** 3),
            request_timeout=args.request_timeout, idle_timeout=args.idle_timeout)
        run_worker(options)
    except (WorkerError, OSError, ValueError, KeyError) as error:
        print(type(error).__name__ + ": " + str(error), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
