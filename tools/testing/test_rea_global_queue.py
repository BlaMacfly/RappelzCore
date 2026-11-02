"""Integrity and crash/resume scenarios for the provider-free REA controller."""
from __future__ import annotations

import copy
import errno
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

MODULE = Path(__file__).resolve().parents[1] / "audit" / "rea_global_queue.py"
SPEC = importlib.util.spec_from_file_location("rea_global_queue", MODULE)
queue = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(queue)


def write_new(path, value):
    with Path(path).open("x", encoding="utf-8") as stream:
        json.dump(value, stream, ensure_ascii=False)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


class WorkerFixture:
    """Independent public-tool-shaped fixture, with no REA or target process."""
    def __init__(self, directory, addresses=("0x2000", "0x2001", "0x2002", "0x2003")):
        self.audit = Path(directory)
        self.artifact = "a" * 64
        self.profile = {"digest": "b" * 64, "provider": {"id": "ghidra", "name": "Ghidra", "version": "12.1.4"}, "parameters": {"seed": "fixture"}}
        self.index = self.audit / "native-index.json"
        self.binding = self.audit / "native-binding.json"
        archive = self.audit / "native-archive.json"
        proof = self.audit / "native-proof.json"
        write_new(self.index, {"functions": [{"address": "0x1000", "evidence_id": "ev_" + "1" * 64}]})
        write_new(archive, {"artifacts": [{"architecture": "x86", "digest": {"sha256": self.artifact}, "format": "pe"}],
                            "records": [], "fixture": "Already verified Native side archive"})
        write_new(proof, {"success": True, "index_functions": 1})
        write_new(self.binding, {"artifact_sha256": self.artifact, "analysis_profile_digest": self.profile["digest"], "native_index_sha256": sha(self.index),
                                "source_archive": {"path": str(archive), "sha256": sha(archive)}, "proof": {"path": str(proof), "sha256": sha(proof)}})
        plan = {"sealed": False, "operations": [{"label": "inventory-procedures", "tool": "list_procedures", "arguments": {}, "reason": "fixture"},
                                                  {"label": "inventory-overview", "tool": "binary_overview", "arguments": {}, "reason": "fixture"}]}
        write_new(self.audit / "plan.json", plan)
        self.entries = [{"address": "0x1000", "value": "already_verified", "procedure": {"external": False, "thunk": False, "thunk_target": None}},
                        {"address": "EXTERNAL:0x1", "value": "imported_function", "procedure": {"external": True, "thunk": False, "thunk_target": None}}]
        self.entries.extend({"address": address, "value": "function_" + address, "procedure": {"external": False, "thunk": i == 0, "thunk_target": "EXTERNAL:0x1" if i == 0 else None}}
                            for i, address in enumerate(addresses))

    def operation(self, label=None):
        operations = queue.read_json(self.audit / "plan.json")["operations"]
        return next(op for op in operations if op["label"] == label) if label else operations[-1]

    def evidence(self, operation, normalized, workflow=False):
        identity = hashlib.sha256((operation["label"] + operation["tool"]).encode()).hexdigest()
        result = {"evidence_id": "ev_" + identity, "operation": operation["tool"], "parameters": operation["arguments"],
                  "subject": {"digest": {"sha256": self.artifact}}, "authority": "shipped-artifact",
                  "confidence": "derived" if workflow else "observed", "provider": copy.deepcopy(self.profile["provider"]),
                  "analysis_profile": copy.deepcopy(self.profile), "normalized_result": normalized}
        if workflow:
            provider = {"id": "rea-workflow", "name": "REA composed investigation workflow", "version": "1"}
            result["provider"] = provider
            result["analysis_profile"] = {"digest": "c" * 64, "provider": provider, "parameters": {"workflow_revision": 2, "upstream_analysis_profile": copy.deepcopy(self.profile)}}
        return result

    def emit(self, operation, response):
        label = operation["label"]
        path = self.audit / (label + ".json")
        write_new(path, response)
        encoded = json.dumps({"tool": operation["tool"], "arguments": operation["arguments"]}, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()
        write_new(self.audit / (label + ".done.json"), {"label": label, "tool": operation["tool"], "operation_sha256": hashlib.sha256(encoded).hexdigest(),
                                                       "artifact_sha256": self.artifact, "response_file": label + ".json", "response_sha256": sha(path), "evidence_id": response.get("evidence_id")})

    def inventories(self, mutate=None):
        operation = self.operation("inventory-procedures")
        evidence = self.evidence(operation, self.entries)
        if mutate:
            mutate(evidence)
        self.emit(operation, evidence)
        operation = self.operation("inventory-overview")
        self.emit(operation, self.evidence(operation, {"procedure_count": len(self.entries)}, workflow=True))

    def success_items(self, operation=None):
        operation = operation or self.operation()
        return [{"address": address, "procedure": {"status": "resolved", "address": address, "name": "fixture"}, "status": "ok", "pseudocode": "void f() { return; }"}
                for address in operation["arguments"]["addresses"]]

    def batch(self, items=None, mutate=None):
        operation = self.operation()
        items = self.success_items(operation) if items is None else items
        succeeded = sum(item["status"] == "ok" for item in items)
        evidence = self.evidence(operation, {"items": items, "total": len(items), "succeeded": succeeded, "failed": len(items) - succeeded}, workflow=True)
        if mutate:
            mutate(evidence)
        self.emit(operation, evidence)

    def checkpoint(self):
        operation = self.operation()
        self.assert_checkpoint(operation)
        path = Path(operation["arguments"]["path"])
        write_new(path, {"fixture": "immutable public export", "records": []})
        self.emit(operation, {"result": {"path": str(path), "bytes": path.stat().st_size, "records": 3, "unknowns": 0}})

    @staticmethod
    def assert_checkpoint(operation):
        if operation["tool"] != "export_evidence_bundle":
            raise AssertionError("Expected queued public export")

    def controller(self, **options):
        return queue.QueueController(self.audit, self.index, self.binding, **options)


class GlobalQueueTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.fixture = WorkerFixture(self.temporary.name)

    def test_external_skipped_native_preserved_and_internal_thunk_scheduled(self):
        self.fixture.inventories()
        with self.fixture.controller(batch_size=2) as controller:
            summary = controller.tick()
            self.assertEqual(summary["state"], "queued_batch")
            self.assertEqual(summary["detailed_native_total"], 1)
            self.assertEqual(summary["detailed_native_in_inventory"], 1)
            self.assertEqual(summary["external_not_applicable"], 1)
            self.assertEqual(summary["internal_thunks"], 1)
            self.assertEqual(self.fixture.operation()["arguments"]["addresses"], ["0x2000", "0x2001"])

    def test_wrong_artifact_is_rejected_without_changing_worker_evidence(self):
        self.fixture.inventories(lambda e: e["subject"]["digest"].update(sha256="d" * 64))
        evidence = self.fixture.audit / "inventory-procedures.json"
        before = evidence.read_bytes()
        with self.fixture.controller() as controller:
            with self.assertRaisesRegex(queue.QueueError, "artifact mismatch"):
                controller.tick()
            self.assertEqual(controller.summary()["inventory_total"], 0)
        self.assertEqual(evidence.read_bytes(), before)

    def test_native_archive_cannot_be_retargeted_with_matching_new_inventory(self):
        # The index/proof hashes still match. Only the binding and new worker
        # responses claim a different module; the archive itself refutes it.
        self.fixture.artifact = "d" * 64
        binding = queue.read_json(self.fixture.binding)
        binding["artifact_sha256"] = self.fixture.artifact
        self.fixture.binding.write_text(json.dumps(binding), encoding="utf-8")
        self.fixture.inventories()
        before = (self.fixture.audit / "plan.json").read_bytes()
        with self.assertRaisesRegex(queue.QueueError, "archive artifact manifest differs"):
            self.fixture.controller()
        self.assertFalse((self.fixture.audit / "queue.sqlite3").exists())
        self.assertEqual((self.fixture.audit / "plan.json").read_bytes(), before)

    def test_native_manifest_parser_is_structural_and_prefix_bounded(self):
        archive = self.fixture.audit / "prefix-only.bundle.json"
        archive.write_bytes(b'{"artifacts": [ ' + b" " * 2048 + b" ],\"records\":[]}")
        with self.assertRaisesRegex(queue.QueueError, "bounded prefix limit"):
            queue.archive_artifact_manifest(archive, maximum_bytes=512)
        other = self.fixture.audit / "noncanonical.bundle.json"
        other.write_text('{"records":[],"artifacts":[]}', encoding="utf-8")
        with self.assertRaisesRegex(queue.QueueError, "first canonical field"):
            queue.archive_artifact_manifest(other)

    def test_mixed_results_remain_explicit_when_attempts_complete(self):
        self.fixture.inventories()
        with self.fixture.controller(batch_size=4, seal_when_complete=True) as controller:
            controller.tick()
            items = self.fixture.success_items()
            items[1] = {**items[1], "status": "error", "error": {"code": "decompile_failed"}}
            items[1].pop("pseudocode")
            items[2]["procedure"] = {"status": "unknown", "error": {"code": "identity_failed"}}
            items[3]["pseudocode"] = None
            self.fixture.batch(items)
            summary = controller.tick()
            self.assertEqual((summary["pseudocode_success"], summary["decompiler_errors"], summary["unresolved_identity"], summary["empty_pseudocode"]), (1, 1, 1, 1))
            self.assertEqual(controller.tick()["state"], "queued_checkpoint")
            self.fixture.checkpoint()
            controller.tick()
            final = controller.tick()
            self.assertEqual(final["state"], "inventory_attempts_complete")
            self.assertTrue(final["all_internal_attempted"])
            self.assertEqual(final["pseudocode_success"], 1)
            self.assertTrue(queue.read_json(self.fixture.audit / "plan.json")["sealed"])

    def test_alias_identity_rejects_entire_batch_without_partial_counts(self):
        self.fixture.inventories()
        with self.fixture.controller(batch_size=2) as controller:
            controller.tick()
            items = self.fixture.success_items()
            items[1]["procedure"]["address"] = "0x1000"
            self.fixture.batch(items)
            with self.assertRaisesRegex(queue.QueueError, "aliases"):
                controller.tick()
            summary = controller.summary()
            self.assertEqual(summary["pseudocode_success"], 0)
            self.assertEqual(summary["queued_internal"], 2)
            self.assertEqual(summary["completed_batches"], 0)

    def test_public_null_result_error_is_classified_empty_not_success_or_retry(self):
        self.fixture.inventories()
        with self.fixture.controller(batch_size=2) as controller:
            controller.tick()
            items = self.fixture.success_items()
            items[0].pop("pseudocode")
            items[0]["status"] = "error"
            items[0]["error"] = {"code": "unreadable_output", "details": {"operation": "procedure_pseudo_code", "reason": "provider returned empty pseudocode"}}
            self.fixture.batch(items)
            summary = controller.tick()
            self.assertEqual(summary["empty_pseudocode"], 1)
            self.assertEqual(summary["pseudocode_success"], 1)
            self.assertEqual(summary["batch_item_errors"], 0)
            controller.tick()
            self.assertNotIn(items[0]["address"], self.fixture.operation()["arguments"]["addresses"])

    def test_upstream_profile_parameters_cannot_change_behind_same_digest(self):
        self.fixture.inventories()
        with self.fixture.controller(batch_size=2) as controller:
            controller.tick()
            self.fixture.batch(mutate=lambda e: e["analysis_profile"]["parameters"]["upstream_analysis_profile"]["parameters"].update(seed="different"))
            with self.assertRaisesRegex(queue.QueueError, "upstream profile"):
                controller.tick()
            self.assertEqual(controller.summary()["pseudocode_success"], 0)

    def test_resume_ingests_existing_job_without_duplicate_work_then_new_phase(self):
        self.fixture.inventories()
        with self.fixture.controller(batch_size=2, max_batches=1) as controller:
            controller.tick()
            label = self.fixture.operation()["label"]
        self.fixture.batch()
        with self.fixture.controller(batch_size=2, max_batches=1) as controller:
            self.assertEqual(controller.tick()["completed_batches"], 1)
            self.assertEqual(controller.tick()["state"], "queued_checkpoint")
            self.fixture.checkpoint()
            controller.tick()
            self.assertEqual(controller.tick()["state"], "phase_complete")
            plan = queue.read_json(self.fixture.audit / "plan.json")
            self.assertFalse(plan["sealed"])
            self.assertEqual(sum(op["label"] == label for op in plan["operations"]), 1)
        with self.fixture.controller(batch_size=2, max_batches=1) as controller:
            self.assertEqual(controller.tick()["state"], "queued_batch")
            self.assertEqual(self.fixture.operation()["arguments"]["addresses"], ["0x2002", "0x2003"])

    def test_crash_after_sql_commit_before_plan_publish_preserves_job(self):
        self.fixture.inventories()
        real_atomic = queue.atomic_json
        def crash(path, value):
            if Path(path).name == "plan.json":
                raise SystemExit("simulated process crash")
            real_atomic(path, value)
        with self.fixture.controller(batch_size=2) as controller:
            with mock.patch.object(queue, "atomic_json", crash):
                with self.assertRaises(SystemExit):
                    controller.tick()
        self.assertEqual(len(queue.read_json(self.fixture.audit / "plan.json")["operations"]), 2)
        with self.fixture.controller(batch_size=2) as controller:
            self.assertEqual(controller.tick()["state"], "waiting_batch")
            self.assertEqual(len(queue.read_json(self.fixture.audit / "plan.json")["operations"]), 3)
            self.fixture.batch()
            self.assertEqual(controller.tick()["completed_batches"], 1)
            self.assertEqual(controller.summary()["pseudocode_success"], 2)

    def test_preexisting_output_never_overwritten_or_scheduled(self):
        self.fixture.inventories()
        output = self.fixture.audit / "global-pseudo-00000001.json"
        output.write_text("existing private observation", encoding="utf-8")
        with self.fixture.controller() as controller:
            with self.assertRaisesRegex(queue.QueueError, "overwrite"):
                controller.tick()
            self.assertEqual(controller.summary()["pending_internal"], 4)
        self.assertEqual(output.read_text(encoding="utf-8"), "existing private observation")

    def test_transient_busy_summary_publication_does_not_duplicate_worker_job(self):
        self.fixture.inventories()
        real_replace = os.replace
        summary_attempts = []
        def briefly_busy(source, destination):
            if Path(destination).name == "controller-summary.json":
                summary_attempts.append(1)
                if len(summary_attempts) <= 2:
                    raise PermissionError(errno.EACCES, "reader temporarily denies share-delete")
            return real_replace(source, destination)
        with self.fixture.controller(batch_size=2) as controller:
            with mock.patch.object(queue.os, "replace", briefly_busy), mock.patch.object(queue.time, "sleep") as sleep:
                self.assertEqual(controller.tick()["state"], "queued_batch")
            self.assertEqual(len(summary_attempts), 3)
            self.assertEqual(sleep.call_count, 2)
            plan = queue.read_json(self.fixture.audit / "plan.json")
            self.assertEqual(sum(op["tool"] == "batch_decompile" for op in plan["operations"]), 1)
            self.fixture.batch()
            self.assertEqual(controller.tick()["pseudocode_success"], 2)
            self.assertEqual(controller.summary()["completed_batches"], 1)

    def test_permanently_busy_metadata_preserves_old_file_and_bounds_retries(self):
        path = self.fixture.audit / "controller-summary.json"
        old = b'{"previous":"complete and valid"}\n'
        path.write_bytes(old)
        with mock.patch.object(queue.os, "replace", side_effect=PermissionError(errno.EACCES, "permanent deny")) as replace, mock.patch.object(queue.time, "sleep"):
            with self.assertRaises(PermissionError):
                queue.atomic_json(path, {"new": "must not replace previous content"})
        self.assertEqual(replace.call_count, 80)
        self.assertEqual(path.read_bytes(), old)
        self.assertEqual(list(self.fixture.audit.glob("controller-summary.json.*.tmp")), [])

    def test_unrelated_filesystem_error_is_not_retried(self):
        path = self.fixture.audit / "controller-summary.json"
        old = b'{"previous":true}'
        path.write_bytes(old)
        with mock.patch.object(queue.os, "replace", side_effect=OSError(errno.ENOSPC, "disk full")) as replace, mock.patch.object(queue.time, "sleep") as sleep:
            with self.assertRaises(OSError):
                queue.atomic_json(path, {"new": True})
        self.assertEqual(replace.call_count, 1)
        sleep.assert_not_called()
        self.assertEqual(path.read_bytes(), old)

    def test_single_controller_lock_released_after_context_exit(self):
        with self.fixture.controller() as first:
            with self.assertRaisesRegex(queue.QueueError, "Another controller"):
                self.fixture.controller()
            self.assertEqual(first.summary()["state"], "starting")
        with self.fixture.controller() as second:
            self.assertEqual(second.summary()["state"], "starting")

    def test_kernel_lock_is_released_after_abrupt_helper_exit(self):
        lock_path = self.fixture.audit / "controller.lock"
        script = "import runpy,os,sys; module=runpy.run_path(sys.argv[1]); held=module['ControllerLock'](module['Path'](sys.argv[2])); os._exit(17)"
        result = subprocess.run([sys.executable, "-c", script, str(MODULE), str(lock_path)], capture_output=True)
        self.assertEqual(result.returncode, 17, result.stderr.decode("utf-8", "replace"))
        with self.fixture.controller() as controller:
            self.assertEqual(controller.summary()["state"], "starting")

    def test_real_worker_error_stops_without_retry(self):
        self.fixture.inventories()
        with self.fixture.controller(batch_size=2) as controller:
            controller.tick()
            operation = self.fixture.operation()
            write_new(self.fixture.audit / (operation["label"] + ".envelope.json"), {"isError": True, "content": [{"type": "text", "text": json.dumps({"error": {"code": "invalid_request", "message": "bad selector"}})}]})
            with self.assertRaisesRegex(queue.QueueError, "worker tool error"):
                controller.tick()
            self.assertEqual(controller.summary()["completed_batches"], 0)
            self.assertEqual(len(queue.read_json(self.fixture.audit / "plan.json")["operations"]), 3)

    def test_retained_transport_error_waits_then_ingests_recovered_result(self):
        self.fixture.inventories()
        with self.fixture.controller(batch_size=2) as controller:
            controller.tick()
            operation = self.fixture.operation()
            error = {"code": "resource_constraint", "details": {"resource": "transport", "reported_limits": {"evidence_reference": {"kind": "retained-evidence", "evidence_id": "ev_" + "2" * 64}}}}
            write_new(self.fixture.audit / (operation["label"] + ".envelope.json"), {"isError": True, "content": [{"type": "text", "text": json.dumps({"error": error})}]})
            self.assertEqual(controller.tick()["state"], "waiting_batch")
            self.fixture.batch()
            self.assertEqual(controller.tick()["pseudocode_success"], 2)

    def test_retained_transport_recovery_has_finite_wait_without_reanalysis(self):
        self.fixture.inventories()
        with self.fixture.controller(batch_size=2) as controller:
            controller.tick()
            operation = self.fixture.operation()
            error = {"code": "resource_constraint", "details": {"resource": "transport", "reported_limits": {"evidence_reference": {"kind": "retained-evidence", "evidence_id": "ev_" + "2" * 64}}}}
            envelope = self.fixture.audit / (operation["label"] + ".envelope.json")
            write_new(envelope, {"isError": True, "content": [{"type": "text", "text": json.dumps({"error": error})}]})
            os.utime(envelope, (0, 0))
            with self.assertRaisesRegex(queue.QueueError, "within300s"):
                controller.tick()
            self.assertEqual(len(queue.read_json(self.fixture.audit / "plan.json")["operations"]), 3)
            self.assertEqual(controller.summary()["pseudocode_success"], 0)

    def test_completed_inventory_is_immutable_on_resume(self):
        self.fixture.inventories()
        with self.fixture.controller(batch_size=2) as controller:
            controller.tick()
        response = self.fixture.audit / "inventory-procedures.json"
        changed = queue.read_json(response)
        changed["normalized_result"][0]["value"] = "changed label"
        response.write_text(json.dumps(changed), encoding="utf-8")
        marker = self.fixture.audit / "inventory-procedures.done.json"
        changed_marker = queue.read_json(marker)
        changed_marker["response_sha256"] = sha(response)
        marker.write_text(json.dumps(changed_marker), encoding="utf-8")
        with self.assertRaisesRegex(queue.QueueError, "Completed inventory output changed"):
            self.fixture.controller()

    def test_completed_response_and_marker_tampering_detected_on_resume(self):
        self.fixture.inventories()
        with self.fixture.controller(batch_size=2) as controller:
            controller.tick()
            self.fixture.batch()
            controller.tick()
            label = self.fixture.operation()["label"]
        response = self.fixture.audit / (label + ".json")
        changed = queue.read_json(response)
        changed["normalized_result"]["items"][0]["pseudocode"] = "tampered"
        response.write_text(json.dumps(changed), encoding="utf-8")
        marker = self.fixture.audit / (label + ".done.json")
        changed_marker = queue.read_json(marker)
        changed_marker["response_sha256"] = sha(response)
        marker.write_text(json.dumps(changed_marker), encoding="utf-8")
        with self.assertRaisesRegex(queue.QueueError, "Completed output changed"):
            self.fixture.controller()


if __name__ == "__main__":
    unittest.main()
