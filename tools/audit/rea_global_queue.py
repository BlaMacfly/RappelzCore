"""Persistent controller for an external, public-REA stdio worker.

This module never launches a provider or executes a target. Evidence files are
worker-owned immutable outputs; the controller owns only its SQLite queue,
summary and append-only operations in the worker's mutable plan.
"""
from __future__ import annotations

import argparse
import codecs
import contextlib
import hashlib
import json
import os
from pathlib import Path
import re
import sqlite3
import sys
import time
import uuid


class QueueError(RuntimeError):
    """An integrity or worker failure requiring external resolution."""


def read_json(path: Path):
    with path.open("r", encoding="utf-8") as stream:
        return json.load(stream)


def file_sha256(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def canonical_json(value) -> str:
    # This is the controller/worker operation encoding, not an RFC8785 Evidence
    # authenticator. Public REA remains the producer of authenticated Evidence.
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def operation_sha256(tool: str, arguments: dict) -> str:
    return hashlib.sha256(canonical_json({"tool": tool, "arguments": arguments}).encode("utf-8")).hexdigest()


def require(condition, message: str):
    if not condition:
        raise QueueError(message)


def archive_artifact_manifest(path: Path, maximum_bytes=1024 * 1024):
    """Read only the first canonical REA field, never the records array.

    The archive's complete file hash and prior proof are checked separately.
    This bounded structural check binds that pinned archive's own artifact
    manifest to the requested module without hydrating hundreds of megabytes.
    """
    decoder = json.JSONDecoder()
    utf8 = codecs.getincrementaldecoder("utf-8")()
    text = ""
    consumed = 0
    eof = False

    def whitespace(position):
        while position < len(text) and text[position] in " \t\r\n":
            position += 1
        if position == len(text):
            raise IndexError
        return position

    with path.open("rb") as stream:
        while True:
            try:
                position = whitespace(0)
                require(text[position] == "{", "Native archive is not a canonical REA bundle object")
                position = whitespace(position + 1)
                key, position = decoder.raw_decode(text, position)
                require(key == "artifacts", "Native archive first canonical field must be artifacts")
                position = whitespace(position)
                require(text[position] == ":", "Native archive artifact field lacks a colon")
                position = whitespace(position + 1)
                manifest, position = decoder.raw_decode(text, position)
                position = whitespace(position)
                require(text[position] in ",}", "Native archive artifact field delimiter is invalid")
                require(isinstance(manifest, list), "Native archive artifacts is not an array")
                return manifest
            except (IndexError, json.JSONDecodeError):
                require(not eof, "Native archive artifact manifest is incomplete")
                require(consumed < maximum_bytes, "Native archive artifact manifest exceeds bounded prefix limit")
                chunk = stream.read(min(65536, maximum_bytes - consumed))
                consumed += len(chunk)
                eof = not chunk
                text += utf8.decode(chunk, final=eof)


def address_key(value: str) -> str:
    require(isinstance(value, str), "Procedure address must be a string")
    match = re.fullmatch(r"(?:(?P<space>[A-Za-z0-9._~%\-]+):)?0x(?P<hex>[0-9a-f]+)", value)
    require(match is not None, f"Noncanonical procedure address: {value!r}")
    canonical = (match["space"] + ":" if match["space"] else "") + hex(int(match["hex"], 16))
    require(canonical == value, f"Noncanonical procedure address: {value!r}")
    return canonical


def atomic_json(path: Path, value):
    """Replace controller-owned mutable state; never call this for Evidence."""
    temporary = path.with_name(path.name + "." + uuid.uuid4().hex + ".tmp")
    try:
        with temporary.open("x", encoding="utf-8", newline="\n") as stream:
            json.dump(value, stream, ensure_ascii=False, indent=2)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        # Windows readers and virus scanners can briefly deny share-delete on
        # an otherwise valid mutable metadata file. Retry only this publication
        # operation; the producer job/Evidence is never submitted again.
        deadline = time.monotonic() + 2.0
        for attempt in range(80):
            try:
                os.replace(temporary, path)
                break
            except PermissionError:
                remaining = deadline - time.monotonic()
                if attempt == 79 or remaining <= 0:
                    raise
                time.sleep(min(0.025, remaining))
    finally:
        with contextlib.suppress(FileNotFoundError):
            temporary.unlink()


class ControllerLock:
    """OS-held lock; process exit/crash releases it without stale-PID guessing."""
    def __init__(self, path: Path):
        self.stream = path.open("a+b")
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
            raise QueueError("Another controller holds the audit queue lock") from error

    def close(self):
        if not self.stream.closed:
            if os.name == "nt":
                import msvcrt
                self.stream.seek(0)
                msvcrt.locking(self.stream.fileno(), msvcrt.LK_UNLCK, 1)
            self.stream.close()


class QueueController:
    def __init__(self, audit_dir, native_index, native_index_binding, *, batch_size=16,
                 checkpoint_every=64, max_batches=None, seal_when_complete=False):
        require(1 <= batch_size <= 256, "batch-size must be between1 and256")
        require(checkpoint_every > 0, "checkpoint-every must be positive")
        require(max_batches is None or max_batches > 0, "max-batches must be positive")
        self.audit = Path(audit_dir).resolve()
        require(self.audit.is_dir(), "audit-dir must already exist")
        self.lock = ControllerLock(self.audit / "controller.lock")
        self.db = None
        try:
            self.index_path = Path(native_index).resolve()
            self.binding_path = Path(native_index_binding).resolve()
            self.binding = self._load_binding()
            self.native = self._load_native()
            self.batch_size = batch_size
            self.checkpoint_every = checkpoint_every
            self.max_batches = max_batches
            self.seal_when_complete = seal_when_complete
            self.db = sqlite3.connect(self.audit / "queue.sqlite3")
            self.db.row_factory = sqlite3.Row
            self.db.execute("PRAGMA journal_mode=WAL")
            self.db.execute("PRAGMA synchronous=FULL")
            self.db.executescript("""
              CREATE TABLE IF NOT EXISTS metadata(key TEXT PRIMARY KEY,value TEXT NOT NULL);
              CREATE TABLE IF NOT EXISTS functions(
                address TEXT PRIMARY KEY,name TEXT NOT NULL,kind TEXT NOT NULL,
                state TEXT NOT NULL,batch_label TEXT,evidence_id TEXT,item_index INTEGER,
                pseudocode_sha256 TEXT,pseudocode_characters INTEGER,error_json TEXT);
              CREATE TABLE IF NOT EXISTS jobs(
                label TEXT PRIMARY KEY,kind TEXT NOT NULL,tool TEXT NOT NULL,
                arguments_json TEXT NOT NULL,operation_sha256 TEXT NOT NULL,
                ordinal INTEGER NOT NULL,state TEXT NOT NULL,response_sha256 TEXT,
                evidence_id TEXT,error_json TEXT);
              CREATE INDEX IF NOT EXISTS functions_state ON functions(state,address);
            """)
            fingerprint = hashlib.sha256(canonical_json(self.binding).encode()).hexdigest()
            previous = self._meta("binding_fingerprint")
            require(previous is None or previous == fingerprint, "Native binding changed on resume")
            self._set_meta("binding_fingerprint", fingerprint)
            self._check_completed_outputs()
            self.phase_start = self._completed_batches()
            self.last_state = "starting"
        except BaseException:
            self.close()
            raise

    def __enter__(self):
        return self

    def __exit__(self, *unused):
        self.close()

    def close(self):
        if self.db is not None:
            self.db.close()
            self.db = None
        self.lock.close()

    def _load_binding(self):
        binding = read_json(self.binding_path)
        for key in ("artifact_sha256", "analysis_profile_digest", "native_index_sha256"):
            require(isinstance(binding.get(key), str) and re.fullmatch(r"[0-9a-f]{64}", binding[key]),
                    f"Invalid native binding {key}")
        require(file_sha256(self.index_path) == binding["native_index_sha256"], "Native index hash mismatch")
        for key in ("source_archive", "proof"):
            reference = binding.get(key, {})
            require(isinstance(reference, dict) and "path" in reference and "sha256" in reference,
                    f"Missing native binding {key} reference")
            path = Path(reference["path"])
            if not path.is_absolute():
                path = self.binding_path.parent / path
            require(file_sha256(path) == reference["sha256"], f"Native {key} hash mismatch")
            if key == "source_archive":
                manifest = archive_artifact_manifest(path)
                require(len(manifest) == 1 and isinstance(manifest[0], dict), "Native archive must declare exactly one artifact")
                digest = manifest[0].get("digest")
                require(isinstance(digest, dict) and digest.get("sha256") == binding["artifact_sha256"],
                        "Native archive artifact manifest differs from binding target")
            if key == "proof":
                self.native_proof = read_json(path)
                require(self.native_proof.get("success") is True, "Native proof did not pass")
        return binding

    def _load_native(self):
        index = read_json(self.index_path)
        functions = index.get("functions")
        require(isinstance(functions, list), "Native index lacks functions")
        entries = {}
        for function in functions:
            address = address_key(function.get("address"))
            require(address not in entries, "Duplicate canonical native entry")
            require(re.fullmatch(r"ev_[0-9a-f]{64}", function.get("evidence_id", "")), "Invalid native Evidence ID")
            entries[address] = function["evidence_id"]
        require(self.native_proof.get("index_functions") == len(entries), "Native proof/index count mismatch")
        return entries

    def _meta(self, key, default=None):
        row = self.db.execute("SELECT value FROM metadata WHERE key=?", (key,)).fetchone()
        return default if row is None else json.loads(row[0])

    def _set_meta(self, key, value):
        with self.db:
            self.db.execute("INSERT OR REPLACE INTO metadata VALUES(?,?)", (key, canonical_json(value)))

    def _plan_snapshot(self):
        path = self.audit / "plan.json"
        require(path.exists(), "Worker plan.json is missing")
        data = path.read_bytes()
        plan = json.loads(data.decode("utf-8"))
        require(isinstance(plan.get("operations"), list) and isinstance(plan.get("sealed"), bool), "Invalid worker plan")
        labels = [op.get("label") for op in plan["operations"]]
        require(len(labels) == len(set(labels)) and all(isinstance(x, str) for x in labels), "Duplicate/invalid worker labels")
        return plan, hashlib.sha256(data).hexdigest()

    def _plan(self):
        return self._plan_snapshot()[0]

    def _worker_error(self, label):
        path = self.audit / (label + ".envelope.json")
        if not path.exists():
            return
        envelope = read_json(path)
        if not envelope.get("isError"):
            return
        error = None
        for item in envelope.get("content", []):
            if item.get("type") == "text":
                with contextlib.suppress(ValueError, TypeError):
                    error = json.loads(item["text"]).get("error")
        if not isinstance(error, dict):
            error = {"message": "Malformed worker tool-error envelope"}
        details = error.get("details", {})
        retained = details.get("reported_limits", {}).get("evidence_reference", {})
        if error.get("code") == "resource_constraint" and details.get("resource") == "transport" and retained.get("kind") == "retained-evidence":
            require(time.time() - path.stat().st_mtime < 300,
                    f"{label}: completed transport result recovery has not produced a marker within300s")
            return
        raise QueueError(f"{label}: worker tool error: {canonical_json(error)}")

    def _response(self, operation):
        label = operation["label"]
        marker = self.audit / (label + ".done.json")
        if not marker.exists():
            self._worker_error(label)
            return None
        done = read_json(marker)
        require(isinstance(done, dict), f"{label}: malformed completion marker")
        require(done.get("label") == label and done.get("tool") == operation["tool"], f"{label}: completion identity mismatch")
        require(done.get("operation_sha256") == operation_sha256(operation["tool"], operation["arguments"]),
                f"{label}: completion operation mismatch")
        require(done.get("artifact_sha256") == self.binding["artifact_sha256"], f"{label}: completion artifact mismatch")
        require(done.get("response_file") == label + ".json", f"{label}: unsafe/unexpected response filename")
        response_path = self.audit / done["response_file"]
        require(file_sha256(response_path) == done.get("response_sha256"), f"{label}: immutable response hash mismatch")
        response = read_json(response_path)
        require(isinstance(response, dict), f"{label}: malformed response object")
        require(done.get("evidence_id") == response.get("evidence_id"), f"{label}: Evidence ID marker mismatch")
        return response, done

    def _evidence(self, evidence, operation, *, workflow=False):
        require(isinstance(evidence, dict), "Expected Evidence object")
        require(evidence.get("operation") == operation["tool"], "Evidence operation mismatch")
        require(evidence.get("parameters") == operation["arguments"], "Evidence parameters differ from requested operation")
        require(re.fullmatch(r"ev_[0-9a-f]{64}", evidence.get("evidence_id", "")), "Malformed Evidence identifier")
        subject = evidence.get("subject")
        require(isinstance(subject, dict) and isinstance(subject.get("digest"), dict), "Evidence subject identity missing")
        require(subject["digest"].get("sha256") == self.binding["artifact_sha256"], "Evidence artifact mismatch")
        require(evidence.get("authority") == "shipped-artifact", "Unexpected Evidence authority")
        profile = evidence.get("analysis_profile") or {}
        require(isinstance(profile, dict) and isinstance(evidence.get("provider"), dict), "Evidence provider/profile missing")
        if workflow:
            expected_provider = {"id": "rea-workflow", "name": "REA composed investigation workflow", "version": "1"}
            require(evidence.get("provider") == expected_provider and evidence.get("confidence") == "derived", "Expected derived workflow Evidence")
            parameters = profile.get("parameters", {})
            require(isinstance(parameters, dict), "Workflow profile parameters missing")
            require(parameters.get("workflow_revision") == 2, "Unexpected batch/overview workflow revision")
            require(parameters.get("upstream_analysis_profile") == self._meta("upstream_profile"), "Workflow upstream profile mismatch")
            require(profile.get("provider") == evidence.get("provider"), "Workflow provider/profile mismatch")
        else:
            require(evidence.get("confidence") == "observed", "Expected observed inventory Evidence")
            require(profile.get("digest") == self.binding["analysis_profile_digest"], "Inventory analysis profile mismatch")
            require(profile.get("provider") == evidence.get("provider"), "Inventory provider/profile mismatch")
        return evidence["normalized_result"]

    def _initialize_inventory(self):
        plan = self._plan()
        by_label = {op["label"]: op for op in plan["operations"]}
        expected = {"inventory-procedures": "list_procedures", "inventory-overview": "binary_overview"}
        responses = {}
        for label, tool in expected.items():
            op = by_label.get(label)
            require(op is not None and op.get("tool") == tool and op.get("arguments") == {}, f"Missing/mismatched {label} operation")
            result = self._response(op)
            if result is None:
                return False
            responses[label] = result
        inventory = responses["inventory-procedures"][0]
        entries = self._evidence(inventory, by_label["inventory-procedures"])
        require(isinstance(entries, list), "Inventory is not an array")
        self._set_meta("upstream_profile", inventory["analysis_profile"])
        overview = self._evidence(responses["inventory-overview"][0], by_label["inventory-overview"], workflow=True)
        require(isinstance(overview, dict), "Overview is not an object")
        require(overview.get("procedure_count") == len(entries), "Overview and full inventory counts disagree")
        rows, addresses, thunks = [], set(), 0
        for item in entries:
            address = address_key(item.get("address"))
            require(address not in addresses, "Duplicate canonical inventory entry")
            addresses.add(address)
            facts = item.get("procedure", {})
            require(type(facts.get("external")) is bool and type(facts.get("thunk")) is bool, "Inventory function classification missing")
            external = facts["external"]
            kind = "external" if external else "thunk" if facts["thunk"] else "internal"
            if not external and facts["thunk"]:
                thunks += 1
            require(isinstance(item.get("value"), str), "Inventory procedure label missing")
            require(not (external and address in self.native), "Previously analyzed Native entry classified external")
            state = "external" if external else "native" if address in self.native else "pending"
            rows.append((address, item["value"], kind, state, self.native.get(address)))
        old_inventory = self._meta("inventory_response_sha256")
        inventory_hash = responses["inventory-procedures"][1]["response_sha256"]
        require(old_inventory is None or old_inventory == inventory_hash, "Inventory response changed on resume")
        if old_inventory is None:
            with self.db:
                self.db.executemany("INSERT INTO functions(address,name,kind,state,evidence_id) VALUES(?,?,?,?,?)", rows)
                values = {"inventory_response_sha256": inventory_hash, "overview_response_sha256": responses["inventory-overview"][1]["response_sha256"],
                          "inventory_thunks_internal": thunks, "native_missing_from_inventory": sorted(set(self.native) - addresses)}
                self.db.executemany("INSERT INTO metadata VALUES(?,?)", [(k, canonical_json(v)) for k, v in values.items()])
        return True

    def _completed_batches(self):
        return self.db.execute("SELECT count(*) FROM jobs WHERE kind='batch' AND state='done'").fetchone()[0]

    def _check_completed_outputs(self):
        # The SQLite record pins the original marker hash too: editing both a
        # response and its marker cannot silently change an ingested result.
        if self._meta("inventory_response_sha256") is not None:
            operations = {op["label"]: op for op in self._plan()["operations"]}
            for label, metadata_key in [("inventory-procedures", "inventory_response_sha256"),
                                        ("inventory-overview", "overview_response_sha256")]:
                require(label in operations, f"Completed inventory operation disappeared: {label}")
                result = self._response(operations[label])
                require(result is not None and result[1]["response_sha256"] == self._meta(metadata_key),
                        f"Completed inventory output changed or disappeared: {label}")
        for job in self.db.execute("SELECT * FROM jobs WHERE state='done'"):
            operation = {"label": job["label"], "tool": job["tool"], "arguments": json.loads(job["arguments_json"])}
            result = self._response(operation)
            require(result is not None and result[1]["response_sha256"] == job["response_sha256"],
                    f"Completed output changed or disappeared: {job['label']}")

    def _ensure_plan_job(self, job):
        plan, old_hash = self._plan_snapshot()
        existing = next((op for op in plan["operations"] if op["label"] == job["label"]), None)
        arguments = json.loads(job["arguments_json"])
        if existing is not None:
            require(operation_sha256(existing["tool"], existing["arguments"]) == job["operation_sha256"], "Outstanding plan operation changed")
            return existing
        require(not plan["sealed"], "Cannot append to sealed worker plan")
        operation = {"label": job["label"], "tool": job["tool"], "arguments": arguments,
                     "reason": "Persisted global pseudocode coverage queue; no target execution."}
        plan["operations"].append(operation)
        require(file_sha256(self.audit / "plan.json") == old_hash, "Worker plan changed during update")
        atomic_json(self.audit / "plan.json", plan)
        return operation

    def _new_job(self, kind, ordinal, arguments, addresses=()):
        label = f"global-{'pseudo' if kind == 'batch' else 'checkpoint'}-{ordinal:08d}"
        tool = "batch_decompile" if kind == "batch" else "export_evidence_bundle"
        require(not any(op["label"] == label for op in self._plan()["operations"]), "Plan label already exists outside persisted queue")
        require(not (self.audit / (label + ".json")).exists() and not (self.audit / (label + ".done.json")).exists(), "Refusing to overwrite existing job Evidence")
        with self.db:
            self.db.execute("INSERT INTO jobs(label,kind,tool,arguments_json,operation_sha256,ordinal,state) VALUES(?,?,?,?,?,?,'queued')",
                            (label, kind, tool, canonical_json(arguments), operation_sha256(tool, arguments), ordinal))
            for address in addresses:
                changed = self.db.execute("UPDATE functions SET state='queued',batch_label=? WHERE address=? AND state='pending'", (label, address)).rowcount
                require(changed == 1, "Function already scheduled or attempted")
        return self.db.execute("SELECT * FROM jobs WHERE label=?", (label,)).fetchone()

    def _ingest_batch(self, job, evidence, done):
        operation = {"label": job["label"], "tool": job["tool"], "arguments": json.loads(job["arguments_json"])}
        result = self._evidence(evidence, operation, workflow=True)
        require(isinstance(result, dict), "Batch result is not an object")
        items, requested = result.get("items"), operation["arguments"]["addresses"]
        require(isinstance(items, list) and len(items) == len(requested) and result.get("total") == len(requested), "Batch item count mismatch")
        require(all(isinstance(item, dict) for item in items), "Malformed batch item")
        require([item.get("address") for item in items] == requested, "Batch selectors/order differ from request")
        ok = sum(item.get("status") == "ok" for item in items)
        require(result.get("succeeded") == ok and result.get("failed") == len(items) - ok, "Batch aggregate counts disagree")
        updates = []
        for index, (address, item) in enumerate(zip(requested, items)):
            identity, state, error, text_hash, characters = item.get("procedure", {}), "error", item.get("error"), None, None
            require(isinstance(identity, dict), "Malformed batch procedure identity")
            require(item.get("status") in {"ok", "error"}, "Unknown batch item status")
            if identity.get("status") == "resolved":
                require(identity.get("address") == address, "Canonical batch identity aliases another inventory entry")
            elif identity.get("status") != "unknown":
                raise QueueError("Malformed batch procedure identity")
            if item["status"] == "ok":
                text = item.get("pseudocode")
                if not isinstance(text, str) or not text.strip():
                    state, error = "empty_pseudocode", {"reason": "Empty/null successful pseudocode"}
                elif identity.get("status") == "unknown":
                    state, error = "unresolved_identity", identity.get("error")
                else:
                    state, error = "pseudocode", None
                if isinstance(text, str):
                    text_hash, characters = hashlib.sha256(text.encode("utf-8")).hexdigest(), len(text)
            elif isinstance(error, dict):
                details = error.get("details", {})
                if (error.get("code") == "unreadable_output" and isinstance(details, dict)
                        and details.get("operation") == "procedure_pseudo_code"
                        and details.get("reason") == "provider returned empty pseudocode"):
                    # This is the actual batch contract for a null/bodyless
                    # provider result, not a recoverable execution failure.
                    state = "empty_pseudocode"
            updates.append((state, evidence["evidence_id"], index, text_hash, characters, canonical_json(error) if error is not None else None, address, job["label"]))
        with self.db:
            for update in updates:
                require(self.db.execute("UPDATE functions SET state=?,evidence_id=?,item_index=?,pseudocode_sha256=?,pseudocode_characters=?,error_json=? WHERE address=? AND batch_label=? AND state='queued'", update).rowcount == 1,
                        "Batch queue rows no longer match outstanding operation")
            self.db.execute("UPDATE jobs SET state='done',response_sha256=?,evidence_id=? WHERE label=?", (done["response_sha256"], evidence["evidence_id"], job["label"]))

    def _ingest_export(self, job, response, done):
        require(done.get("evidence_id") is None, "Checkpoint result unexpectedly claims Evidence identity")
        expected = Path(json.loads(job["arguments_json"])["path"]).resolve()
        result = response.get("result", {})
        require(isinstance(result, dict), "Malformed checkpoint response")
        require(Path(result.get("path", "")).resolve() == expected, "Checkpoint path mismatch")
        require(expected.is_file() and expected.stat().st_size == result.get("bytes"), "Checkpoint bytes/file mismatch")
        require(isinstance(result.get("records"), int) and result["records"] >= 1, "Checkpoint record count missing")
        checkpoint = {"batch": job["ordinal"], "path": str(expected), "sha256": file_sha256(expected), "bytes": result["bytes"], "records": result["records"]}
        with self.db:
            self.db.execute("UPDATE jobs SET state='done',response_sha256=? WHERE label=?", (done["response_sha256"], job["label"]))
            self.db.execute("INSERT OR REPLACE INTO metadata VALUES('last_checkpoint',?)", (canonical_json(checkpoint),))

    def _checkpoint(self, count):
        folder = self.audit / "checkpoints"
        folder.mkdir(exist_ok=True)
        require(folder.resolve().is_relative_to(self.audit), "Checkpoint directory resolves outside audit-dir")
        path = folder / f"global-{count:08d}.bundle.json"
        require(not path.exists(), "Checkpoint output already exists without a recorded completion")
        return self._new_job("export", count, {"path": str(path)})

    def summary(self, state=None):
        counts = {row[0]: row[1] for row in self.db.execute("SELECT state,count(*) FROM functions GROUP BY state")}
        internal = self.db.execute("SELECT count(*) FROM functions WHERE kind!='external'").fetchone()[0]
        completed = self._completed_batches()
        return {"state": state or self.last_state, "artifact_sha256": self.binding["artifact_sha256"],
                "inventory_total": sum(counts.values()), "inventory_internal": internal, "external_not_applicable": counts.get("external", 0),
                "internal_thunks": self._meta("inventory_thunks_internal", 0), "detailed_native_total": len(self.native),
                "detailed_native_in_inventory": counts.get("native", 0), "native_missing_from_inventory": self._meta("native_missing_from_inventory", []),
                "pseudocode_success": counts.get("pseudocode", 0), "decompiler_errors": counts.get("error", 0),
                "batch_item_errors": counts.get("error", 0),
                "empty_pseudocode": counts.get("empty_pseudocode", 0), "unresolved_identity": counts.get("unresolved_identity", 0),
                "pending_internal": counts.get("pending", 0), "queued_internal": counts.get("queued", 0),
                "completed_batches": completed, "phase_completed_batches": completed - self.phase_start,
                "last_checkpoint": self._meta("last_checkpoint"),
                "all_internal_attempted": self._meta("inventory_response_sha256") is not None and not (counts.get("pending", 0) or counts.get("queued", 0)),
                "metrics_limit": "Pseudocode availability is distinct from detailed Native dossiers, byte coverage and gameplay reconstruction."}

    def tick(self):
        try:
            return self._tick()
        except (QueueError, OSError, ValueError, sqlite3.DatabaseError) as error:
            self.last_state = "stopped_error"
            summary = self.summary()
            summary["error"] = str(error)
            atomic_json(self.audit / "controller-summary.json", summary)
            job = self.db.execute("SELECT label FROM jobs WHERE state IN ('queued','fatal') ORDER BY rowid LIMIT 1").fetchone()
            if job:
                with self.db:
                    self.db.execute("UPDATE jobs SET state='fatal',error_json=? WHERE label=?", (canonical_json({"message": str(error)}), job[0]))
            raise QueueError(str(error)) from error

    def _tick(self):
        if self._meta("inventory_response_sha256") is None:
            if not self._initialize_inventory():
                return self._publish("waiting_inventory")
        job = self.db.execute("SELECT * FROM jobs WHERE state IN ('queued','fatal') ORDER BY rowid LIMIT 1").fetchone()
        if job:
            operation = self._ensure_plan_job(job)
            response = self._response(operation)
            if response is None:
                require(job["state"] != "fatal", f"Outstanding job {job['label']} remains stopped: {job['error_json']}")
                return self._publish("waiting_" + job["kind"])
            if job["kind"] == "batch":
                self._ingest_batch(job, *response)
            else:
                self._ingest_export(job, *response)
            return self._publish("ingested_" + job["kind"])
        count = self._completed_batches()
        remaining = self.db.execute("SELECT count(*) FROM functions WHERE state='pending'").fetchone()[0]
        phase_end = self.max_batches is not None and count - self.phase_start >= self.max_batches
        checkpoint = self._meta("last_checkpoint")
        last_count = checkpoint["batch"] if checkpoint else None
        if (last_count is None or last_count < count) and (not remaining or phase_end or count > 0 and count % self.checkpoint_every == 0):
            self._ensure_plan_job(self._checkpoint(count))
            return self._publish("queued_checkpoint")
        if not remaining:
            if self.seal_when_complete:
                plan, old_hash = self._plan_snapshot()
                plan["sealed"] = True
                require(file_sha256(self.audit / "plan.json") == old_hash, "Worker plan changed during sealing")
                atomic_json(self.audit / "plan.json", plan)
            return self._publish("inventory_attempts_complete")
        if phase_end:
            return self._publish("phase_complete")
        addresses = [row[0] for row in self.db.execute("SELECT address FROM functions WHERE state='pending' ORDER BY address LIMIT ?", (self.batch_size,))]
        ordinal = self.db.execute("SELECT coalesce(max(ordinal),0)+1 FROM jobs WHERE kind='batch'").fetchone()[0]
        self._ensure_plan_job(self._new_job("batch", ordinal, {"addresses": addresses}, addresses))
        return self._publish("queued_batch")

    def _publish(self, state):
        self.last_state = state
        summary = self.summary()
        atomic_json(self.audit / "controller-summary.json", summary)
        return summary


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--audit-dir", required=True, type=Path)
    parser.add_argument("--native-index", required=True, type=Path)
    parser.add_argument("--native-index-binding", required=True, type=Path)
    parser.add_argument("--batch-size", type=int, default=16)
    parser.add_argument("--checkpoint-every", type=int, default=64)
    parser.add_argument("--max-batches", type=int, help="Additional completed batches during this invocation; checkpoint then exit without sealing.")
    parser.add_argument("--seal-when-complete", action="store_true", help="Seal only after every internal function was attempted; errors remain errors.")
    parser.add_argument("--poll-seconds", type=float, default=0.5)
    args = parser.parse_args(argv)
    try:
        require(args.poll_seconds > 0, "poll-seconds must be positive")
        with QueueController(args.audit_dir, args.native_index, args.native_index_binding,
                             batch_size=args.batch_size, checkpoint_every=args.checkpoint_every,
                             max_batches=args.max_batches, seal_when_complete=args.seal_when_complete) as controller:
            previous = None
            while True:
                summary = controller.tick()
                encoded = canonical_json(summary)
                if encoded != previous:
                    print(encoded, flush=True)
                    previous = encoded
                if summary["state"] in {"phase_complete", "inventory_attempts_complete"}:
                    return 0
                time.sleep(args.poll_seconds)
    except (QueueError, OSError, ValueError, sqlite3.DatabaseError) as error:
        print(canonical_json({"state": "stopped_error", "error": str(error)}), flush=True)
        return 1
    except KeyboardInterrupt:
        print(canonical_json({"state": "interrupted", "queue_preserved": True}), flush=True)
        return 130


if __name__ == "__main__":
    sys.exit(main())
