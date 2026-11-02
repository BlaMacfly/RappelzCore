"""Engine-free integrity, lifecycle and retained-result tests for the worker."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock
from dataclasses import replace

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "audit"))
import rea_mcp_worker as worker


IDENTITY = "ev_" + "1" * 64
OPERATION = {"label": "batch-0001", "tool": "batch_decompile",
             "arguments": {"addresses": ["0x401000"]}, "reason": "Fixture"}


class FakeClient:
    def __init__(self, target, *, failure=None, mutate=False):
        self.target, self.failure, self.mutate = target, failure, mutate
        self.target_sha = worker.file_sha256(target)
        self.calls, self.closed = [], False
        self.record = {"evidence_id": IDENTITY, "subject": {"digest": {"sha256": self.target_sha}},
                       "operation": OPERATION["tool"], "parameters": OPERATION["arguments"],
                       "normalized_result": {"items": []}}

    def request(self, method, params, timeout):
        if method == "initialize":
            return {"serverInfo": {"name": "fake-rea", "version": "test"}}
        if method == "tools/list":
            names = {"open_binary", "close_binary", "export_evidence_bundle", "get_evidence_bundle",
                     "batch_decompile", "list_procedures", "binary_overview"}
            return {"tools": [{"name": n, "inputSchema": {"type": "object"}} for n in sorted(names)]}
        if method != "tools/call":
            raise AssertionError(method)
        name, args = params["name"], params["arguments"]
        self.calls.append((name, args))
        if name == "open_binary":
            return {"structuredContent": {"result": {"sha256": self.target_sha}}}
        if name == "close_binary":
            if "snapshot_path" in args:
                Path(args["snapshot_path"]).write_text("{}", encoding="utf-8")
            return {"content": [{"type": "text", "text": "{\"result\":null}"}]}
        if name == "export_evidence_bundle":
            path = Path(args["path"])
            path.write_text(json.dumps({"records": [self.record], "unknowns": []}), encoding="utf-8")
            return {"structuredContent": {"result": {"path": str(path), "records": 1, "bytes": path.stat().st_size}}}
        if name == "get_evidence_bundle":
            return {"structuredContent": {"result": {"evidence_id": IDENTITY, "records": 1}}}
        if name == "batch_decompile":
            if self.mutate:
                self.target.write_bytes(b"changed during fake analysis")
            if self.failure == "transport":
                error = {"code": "resource_constraint", "details": {"resource": "transport",
                    "reported_limits": {"evidence_reference": {"kind": "retained-evidence", "evidence_id": IDENTITY}}}}
                return {"isError": True, "content": [{"type": "text", "text": json.dumps({"error": error})}]}
            if self.failure:
                return {"isError": True, "content": [{"type": "text", "text": json.dumps({"error": {"code": "invalid_request", "message": "Fixture failure"}})}]}
            return {"structuredContent": self.record}
        raise AssertionError(name)

    def notify(self, method, params=None):
        if method != "notifications/initialized":
            raise AssertionError(method)

    def close(self):
        self.closed = True


class WorkerTest(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.audit = self.root / "audit"
        self.audit.mkdir()
        self.target = self.root / "fixture.bin"
        self.target.write_bytes(b"static fixture; never executed")
        self.options = worker.WorkerOptions(self.audit, self.target, Path("unused-node"),
            Path("unused-rea.mjs"), self.root, self.root, min_free_bytes=0,
            idle_timeout=0.01, poll_interval=0.001, shutdown_timeout=1)
        self.plan([OPERATION])

    def plan(self, operations, sealed=True):
        (self.audit / "plan.json").write_text(json.dumps({"sealed": sealed, "operations": operations}), encoding="utf-8")

    def completion(self, *, target_sha=None):
        response = {"evidence_id": IDENTITY,
                    "subject": {"digest": {"sha256": worker.file_sha256(self.target)}}}
        worker.atomic_json(self.audit / (OPERATION["label"] + ".json"), response)
        marker = {"label": OPERATION["label"], "tool": OPERATION["tool"],
            "operation_sha256": worker.operation_digest(OPERATION),
            "artifact_sha256": target_sha or worker.file_sha256(self.target),
            "response_file": OPERATION["label"] + ".json",
            "response_sha256": worker.file_sha256(self.audit / (OPERATION["label"] + ".json")),
            "evidence_id": IDENTITY}
        worker.atomic_json(self.audit / (OPERATION["label"] + ".done.json"), marker)
        return marker

    def test_operation_digest_matches_controller_contract(self):
        op = {"tool": "procedure_pseudo_code", "arguments": {"procedure": "étiquette"}, "label": "ignored"}
        expected = hashlib.sha256('{"arguments":{"procedure":"étiquette"},"tool":"procedure_pseudo_code"}'.encode("utf-8")).hexdigest()
        self.assertEqual(worker.operation_digest(op), expected)

    def runtime_options(self):
        node = self.root / "node-fixture.exe"
        launcher = self.root / "rea-fixture.mjs"
        node.write_bytes(b"Distinct fake Node runtime; not executed")
        launcher.write_text("// Distinct fake public launcher; not executed", encoding="utf-8")
        return replace(self.options, node=node, rea_launcher=launcher)

    def test_target_runtime_path_confusion_stops_before_client_construction(self):
        options = self.runtime_options()
        for field in ("node", "rea_launcher"):
            with self.subTest(field=field), mock.patch.object(worker, "McpClient") as client:
                confused = replace(options, **{field: self.target})
                with self.assertRaisesRegex(worker.WorkerError, "must not be the target"):
                    worker.run_worker(confused)
                client.assert_not_called()

    def test_target_hardlink_runtime_confusion_stops_before_client_construction(self):
        options = self.runtime_options()
        link = self.root / "linked-runtime.mjs"
        try:
            os.link(self.target, link)
        except OSError as error:
            self.skipTest("Hardlinks unavailable on this filesystem: " + str(error))
        for field in ("node", "rea_launcher"):
            with self.subTest(field=field), mock.patch.object(worker, "McpClient") as client:
                confused = replace(options, **{field: link})
                with self.assertRaisesRegex(worker.WorkerError, "hardlink"):
                    worker.run_worker(confused)
                client.assert_not_called()

    def test_target_copy_runtime_confusion_stops_before_client_construction(self):
        options = self.runtime_options()
        copied = self.root / "copied-runtime.mjs"
        copied.write_bytes(self.target.read_bytes())
        for field in ("node", "rea_launcher"):
            with self.subTest(field=field), mock.patch.object(worker, "McpClient") as client:
                confused = replace(options, **{field: copied})
                with self.assertRaisesRegex(worker.WorkerError, "contains the target bytes"):
                    worker.run_worker(confused)
                client.assert_not_called()

    def test_success_writes_exact_bound_marker_and_closes(self):
        client = FakeClient(self.target)
        worker.run_worker(self.options, client=client)
        marker = worker.verify_completion(self.audit, OPERATION, worker.file_sha256(self.target))
        self.assertEqual(marker["evidence_id"], IDENTITY)
        self.assertEqual(len(marker), 7)
        self.assertTrue(client.closed)
        self.assertEqual([n for n, _ in client.calls].count("batch_decompile"), 1)
        self.assertTrue(list(self.audit.glob("e-*.final.bundle.json")))
        self.assertEqual(worker.read_json(self.audit / "worker-status.json")["phase"], "complete")

    def test_verified_resume_skips_producer_and_never_rewrites_response(self):
        self.completion()
        response = self.audit / (OPERATION["label"] + ".json")
        original = (response.read_bytes(), response.stat().st_mtime_ns)
        client = FakeClient(self.target)
        worker.run_worker(self.options, client=client)
        self.assertNotIn("batch_decompile", [n for n, _ in client.calls])
        self.assertEqual((response.read_bytes(), response.stat().st_mtime_ns), original)

    def test_plan_cannot_change_an_operation_already_verified_in_memory(self):
        client = FakeClient(self.target)
        original = client.request
        def request(method, params, timeout):
            result = original(method, params, timeout)
            if method == "tools/call" and params["name"] == "batch_decompile":
                self.plan([dict(OPERATION, arguments={"addresses": ["0x402000"]})])
            return result
        client.request = request
        with self.assertRaisesRegex(worker.WorkerError, "changed within"):
            worker.run_worker(self.options, client=client)
        self.assertEqual([n for n, _ in client.calls].count("batch_decompile"), 1)
        self.assertTrue(client.closed)

    def test_wrong_target_hash_or_tampered_response_cannot_resume(self):
        self.completion(target_sha="0" * 64)
        with self.assertRaisesRegex(worker.WorkerError, "target hash"):
            worker.verify_completion(self.audit, OPERATION, worker.file_sha256(self.target))
        marker = self.audit / (OPERATION["label"] + ".done.json")
        data = worker.read_json(marker); data["artifact_sha256"] = worker.file_sha256(self.target)
        marker.write_text(json.dumps(data), encoding="utf-8")
        (self.audit / (OPERATION["label"] + ".json")).write_text("{}", encoding="utf-8")
        with self.assertRaisesRegex(worker.WorkerError, "response hash"):
            worker.verify_completion(self.audit, OPERATION, worker.file_sha256(self.target))

    def test_changed_operation_or_unsafe_response_filename_cannot_resume(self):
        self.completion()
        changed = dict(OPERATION, arguments={"addresses": ["0x402000"]})
        with self.assertRaisesRegex(worker.WorkerError, "operation hash"):
            worker.verify_completion(self.audit, changed, worker.file_sha256(self.target))
        marker = self.audit / (OPERATION["label"] + ".done.json")
        data = worker.read_json(marker); data["response_file"] = "../fixture.bin"
        marker.write_text(json.dumps(data), encoding="utf-8")
        with self.assertRaisesRegex(worker.WorkerError, "response filename"):
            worker.verify_completion(self.audit, OPERATION, worker.file_sha256(self.target))

    def test_matching_marker_hash_does_not_hide_wrong_evidence_target(self):
        self.completion()
        response = self.audit / (OPERATION["label"] + ".json")
        value = worker.read_json(response)
        value["subject"]["digest"]["sha256"] = "0" * 64
        response.write_text(json.dumps(value), encoding="utf-8")
        marker = self.audit / (OPERATION["label"] + ".done.json")
        value = worker.read_json(marker)
        value["response_sha256"] = worker.file_sha256(response)
        marker.write_text(json.dumps(value), encoding="utf-8")
        with self.assertRaisesRegex(worker.WorkerError, "Evidence target"):
            worker.verify_completion(self.audit, OPERATION, worker.file_sha256(self.target))

    def test_orphan_response_stops_without_repeating_producer(self):
        worker.atomic_json(self.audit / (OPERATION["label"] + ".json"), {"evidence_id": IDENTITY})
        client = FakeClient(self.target)
        with self.assertRaisesRegex(worker.WorkerError, "will not be repeated"):
            worker.run_worker(self.options, client=client)
        self.assertNotIn("batch_decompile", [n for n, _ in client.calls])
        self.assertTrue(client.closed)

    def test_producer_failure_is_preserved_and_not_retried_on_restart(self):
        client = FakeClient(self.target, failure="failure")
        with self.assertRaisesRegex(worker.WorkerError, "Fixture failure"):
            worker.run_worker(self.options, client=client)
        self.assertFalse((self.audit / (OPERATION["label"] + ".done.json")).exists())
        self.assertTrue(worker.read_json(self.audit / (OPERATION["label"] + ".envelope.json"))["isError"])
        self.assertTrue(list(self.audit.glob("e-*.partial.bundle.json")))
        second = FakeClient(self.target)
        with self.assertRaisesRegex(worker.WorkerError, "will not be repeated"):
            worker.run_worker(self.options, client=second)
        self.assertNotIn("batch_decompile", [n for n, _ in second.calls])

    def test_client_timeout_is_explicit_uncertainty_not_fabricated_evidence(self):
        client = FakeClient(self.target)
        original = client.request
        def request(method, params, timeout):
            if method == "tools/call" and params["name"] == "batch_decompile":
                client.calls.append(("batch_decompile", params["arguments"]))
                raise worker.WorkerError("Fixture request timed out")
            return original(method, params, timeout)
        client.request = request
        with self.assertRaisesRegex(worker.WorkerError, "timed out"):
            worker.run_worker(self.options, client=client)
        envelope = worker.read_json(self.audit / (OPERATION["label"] + ".envelope.json"))
        self.assertTrue(envelope["worker_generated"])
        error = worker.tool_error(envelope)
        self.assertEqual(error["code"], "worker_client_failure")
        self.assertEqual(error["producer_completion"], "unknown")
        self.assertEqual([n for n, _ in client.calls].count("batch_decompile"), 1)
        self.assertFalse((self.audit / (OPERATION["label"] + ".done.json")).exists())

    def test_transport_retention_recovers_without_repeating_producer(self):
        client = FakeClient(self.target, failure="transport")
        worker.run_worker(self.options, client=client)
        self.assertEqual([n for n, _ in client.calls].count("batch_decompile"), 1)
        self.assertEqual([n for n, _ in client.calls].count("get_evidence_bundle"), 1)
        self.assertEqual(worker.read_json(self.audit / (OPERATION["label"] + ".json")), client.record)
        self.assertEqual(worker.verify_completion(self.audit, OPERATION, client.target_sha)["evidence_id"], IDENTITY)
        self.assertTrue(worker.read_json(self.audit / (OPERATION["label"] + ".envelope.json"))["isError"])

    def test_retained_result_for_wrong_target_stops_without_reanalysis(self):
        client = FakeClient(self.target, failure="transport")
        client.record["subject"]["digest"]["sha256"] = "0" * 64
        with self.assertRaisesRegex(worker.WorkerError, "another target"):
            worker.run_worker(self.options, client=client)
        self.assertEqual([n for n, _ in client.calls].count("batch_decompile"), 1)
        self.assertFalse((self.audit / (OPERATION["label"] + ".done.json")).exists())
        self.assertTrue(client.closed)

    def test_target_change_during_analysis_prevents_completion(self):
        client = FakeClient(self.target, mutate=True)
        with self.assertRaisesRegex(worker.WorkerError, "changed during"):
            worker.run_worker(self.options, client=client)
        self.assertFalse((self.audit / (OPERATION["label"] + ".done.json")).exists())
        self.assertFalse(worker.read_json(self.audit / "worker-status.json")["target_unchanged"])

    def test_unsafe_labels_executable_tools_and_export_paths_are_rejected(self):
        for label in ("../escape", "..\\escape", "Name", "x.json", ""):
            with self.subTest(label=label), self.assertRaises(worker.WorkerError):
                worker.validate_operation(dict(OPERATION, label=label), self.audit)
        with self.assertRaises(worker.WorkerError):
            worker.validate_operation(dict(OPERATION, tool="capture_process_scenario"), self.audit)
        for arguments in ({"path": str(self.root / "escape.json")}, {"path": "relative.json"},
                          {"path": str(self.audit / "new.json"), "overwrite": True}):
            with self.subTest(arguments=arguments), self.assertRaises(worker.WorkerError):
                worker.validate_operation(dict(OPERATION, tool="export_evidence_bundle", arguments=arguments), self.audit)

    def test_immutable_output_and_disk_guard(self):
        path = self.audit / "immutable.json"
        worker.atomic_json(path, {"value": 1})
        with self.assertRaisesRegex(worker.WorkerError, "overwrite"):
            worker.atomic_json(path, {"value": 2})
        self.assertEqual(worker.read_json(path), {"value": 1})
        self.options.min_free_bytes = 10
        client = FakeClient(self.target)
        with mock.patch.object(worker.shutil, "disk_usage", return_value=type("Disk", (), {"free": 0})()):
            with self.assertRaisesRegex(worker.WorkerError, "free-space"):
                worker.run_worker(self.options, client=client)
        self.assertFalse(client.calls)
        self.assertTrue(client.closed)

    def test_mutable_status_retries_only_transient_replace_conflicts(self):
        path = self.audit / "status.json"
        worker.atomic_json(path, {"old": True})
        original_replace = worker.os.replace
        failures = [PermissionError("reader denies deletion"), PermissionError("still busy")]
        def replace(source, destination):
            if failures:
                raise failures.pop(0)
            return original_replace(source, destination)
        with mock.patch.object(worker.os, "replace", side_effect=replace) as rename, mock.patch.object(worker.time, "sleep") as sleep:
            worker.atomic_json(path, {"new": True}, mutable=True)
        self.assertEqual(rename.call_count, 3)
        self.assertEqual(sleep.call_count, 2)
        self.assertEqual(worker.read_json(path), {"new": True})

    def test_permanent_status_conflict_keeps_old_file_and_stops(self):
        path = self.audit / "status.json"
        worker.atomic_json(path, {"old": True})
        with mock.patch.object(worker.os, "replace", side_effect=PermissionError("busy")) as rename, mock.patch.object(worker.time, "sleep"):
            with self.assertRaises(PermissionError):
                worker.atomic_json(path, {"new": True}, mutable=True)
        self.assertEqual(rename.call_count, 20)
        self.assertEqual(worker.read_json(path), {"old": True})
        self.assertFalse(list(self.audit.glob(".worker-*.tmp")))
        with mock.patch.object(worker.os, "link", side_effect=PermissionError("busy")) as link, mock.patch.object(worker.time, "sleep") as sleep:
            with self.assertRaises(PermissionError):
                worker.atomic_json(self.audit / "immutable-new.json", {})
        self.assertEqual(link.call_count, 1)
        sleep.assert_not_called()

    def test_kernel_lock_excludes_second_worker_and_releases(self):
        path = self.audit / "worker.lock"
        with worker.WorkerLock(path):
            with self.assertRaisesRegex(worker.WorkerError, "Another worker"):
                worker.WorkerLock(path)
        with worker.WorkerLock(path):
            pass

    def test_kernel_lock_is_released_after_abrupt_process_exit(self):
        path = self.audit / "worker.lock"
        source = ("import sys,os; from pathlib import Path; "
                  "sys.path.insert(0,sys.argv[2]); "
                  "from rea_mcp_worker import WorkerLock; "
                  "lock=WorkerLock(Path(sys.argv[1])); os._exit(0)")
        subprocess.run([sys.executable, "-c", source, str(path), str(Path(worker.__file__).parent)],
                       check=True, timeout=10)
        with worker.WorkerLock(path):
            pass

    def test_bounded_recovery_validates_tail_and_uniqueness(self):
        path = self.audit / "bundle.json"
        record = {"evidence_id": IDENTITY, "text": "é" * 20000}
        path.write_text(json.dumps({"records": [record], "unknowns": []}, ensure_ascii=False), encoding="utf-8")
        self.assertEqual(worker.find_evidence_record(path, IDENTITY), record)
        with self.assertRaisesRegex(worker.WorkerError, "memory bound"):
            worker.find_evidence_record(path, IDENTITY, max_value_chars=128)
        path.write_text(json.dumps({"records": [record, record]}), encoding="utf-8")
        with self.assertRaisesRegex(worker.WorkerError, "duplicated"):
            worker.find_evidence_record(path, IDENTITY)
        path.write_text(json.dumps({"records": [{"evidence_id": IDENTITY}]}) + "garbage", encoding="utf-8")
        with self.assertRaisesRegex(worker.WorkerError, "Trailing data"):
            worker.find_evidence_record(path, IDENTITY)

    def test_fake_stdio_client_reader_terminates_and_environment_is_local(self):
        script = self.root / "fake_mcp.py"
        script.write_text("import sys,json\nfor line in sys.stdin:\n d=json.loads(line)\n if 'id' in d: print(json.dumps({'jsonrpc':'2.0','id':d['id'],'result':{'echo':d['method']}}),flush=True)\n", encoding="utf-8")
        original_environment = dict(os.environ)
        command_seen = []
        def launch(command, **kwargs):
            command_seen.append((command, kwargs["env"]))
            return subprocess.Popen([sys.executable, "-u", str(script)], **kwargs)
        client = worker.McpClient(self.options, "fake-epoch", popen_factory=launch)
        try:
            self.assertEqual(client.request("initialize", {}, 5), {"echo": "initialize"})
            client.notify("notifications/initialized")
        finally:
            client.close()
        self.assertFalse(client.reader_thread.is_alive())
        self.assertEqual(dict(os.environ), original_environment)
        self.assertEqual(command_seen[0][0], ["unused-node", "--no-opt", "unused-rea.mjs", "--mcp"])
        self.assertEqual(command_seen[0][1]["GHIDRA_HEADLESS_MAXMEM"], "1536M")
        self.assertEqual(command_seen[0][1]["NODE_OPTIONS"], "--max-old-space-size=4096")


if __name__ == "__main__":
    unittest.main()
