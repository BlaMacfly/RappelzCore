"""Short-path publication plus the unchanged controller's 30 integrity tests."""
from pathlib import Path
import hashlib
import importlib.util
import json
import os
import tempfile
import unittest
from unittest import mock


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


TOOLS = Path(__file__).resolve().parents[1]
adapter = load("windows_queue_adapter_test", TOOLS / "audit/rea_global_queue_windows.py")
base_cases = load("original_queue_fixture_cases", Path(__file__).with_name("test_rea_global_queue.py"))
# These provider-free tests exercise the exact pinned controller with only its
# JSON publisher replaced, including ready-job ingestion after a real stop.
base_cases.queue.atomic_json = adapter.atomic_json
ControllerWithShortTempNames = base_cases.GlobalQueueTests


class ShortPublicationTests(unittest.TestCase):
    def test_actual_202_character_parent_keeps_both_paths_below_legacy_limit(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            parent = root / ("a" * (202 - len(str(root)) - 1))
            parent.mkdir()
            self.assertEqual(len(str(parent)), 202)
            destination = parent / "controller-summary.json"
            seen = []
            replace = os.replace
            def observe(source, target):
                seen.append((Path(source), Path(target)))
                return replace(source, target)
            with mock.patch.object(adapter.os, "replace", side_effect=observe):
                adapter.atomic_json(destination, {"state": "waiting_batch", "retained": "déjà acquis"})
            self.assertEqual(json.loads(destination.read_text(encoding="utf-8"))["state"], "waiting_batch")
            self.assertTrue(all(len(str(p)) < 260 for pair in seen for p in pair))
            self.assertEqual(list(parent.iterdir()), [destination])

    def test_serialization_is_byte_identical_to_original_writer(self):
        with tempfile.TemporaryDirectory() as directory:
            first, second = Path(directory) / "old.json", Path(directory) / "new.json"
            payload = {"unicode": "résumé", "items": [1, None, False], "state": "queued"}
            original = adapter.load_controller()
            # Capture the original publisher before the adapter installs it.
            pristine = load("pristine_queue_serialization", TOOLS / "audit/rea_global_queue.py")
            pristine.atomic_json(first, payload)
            original.atomic_json(second, payload)
            self.assertEqual(first.read_bytes(), second.read_bytes())

    def test_transient_sharing_denial_only_retries_metadata_rename(self):
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / "state.json"
            target.write_text('{"old": true}', encoding="utf-8")
            replace = os.replace
            attempts = []
            def delayed(source, destination):
                attempts.append(source)
                if len(attempts) < 3:
                    raise PermissionError("fixture reader holds mutable state")
                return replace(source, destination)
            with mock.patch.object(adapter.os, "replace", side_effect=delayed), mock.patch.object(adapter.time, "sleep"):
                adapter.atomic_json(target, {"new": True})
            self.assertEqual(len(attempts), 3)
            self.assertEqual(len(set(attempts)), 1)
            self.assertEqual(json.loads(target.read_text(encoding="utf-8")), {"new": True})

    def test_permanent_denial_preserves_existing_file_and_cleans_owned_temp(self):
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / "state.json"
            target.write_bytes(b"old-state")
            with mock.patch.object(adapter.os, "replace", side_effect=PermissionError("fixture")) as replace, mock.patch.object(adapter.time, "sleep"):
                with self.assertRaises(PermissionError):
                    adapter.atomic_json(target, {"new": True})
            self.assertLessEqual(replace.call_count, 80)
            self.assertEqual(target.read_bytes(), b"old-state")
            self.assertEqual(list(Path(directory).iterdir()), [target])

    def test_unrelated_rename_error_is_not_retried(self):
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / "state.json"
            target.write_bytes(b"old-state")
            with mock.patch.object(adapter.os, "replace", side_effect=OSError("fixture")) as replace:
                with self.assertRaises(OSError):
                    adapter.atomic_json(target, {"new": True})
            self.assertEqual(replace.call_count, 1)
            self.assertEqual(target.read_bytes(), b"old-state")

    def test_encoding_failure_keeps_previous_file(self):
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / "state.json"
            target.write_bytes(b"old-state")
            with self.assertRaises(TypeError):
                adapter.atomic_json(target, {"bad": object()})
            self.assertEqual(target.read_bytes(), b"old-state")
            self.assertEqual(list(Path(directory).iterdir()), [target])

    def test_fsync_failure_keeps_previous_file(self):
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / "state.json"
            target.write_bytes(b"old-state")
            with mock.patch.object(adapter.os, "fsync", side_effect=OSError("fixture flush failure")):
                with self.assertRaises(OSError):
                    adapter.atomic_json(target, {"new": True})
            self.assertEqual(target.read_bytes(), b"old-state")
            self.assertEqual(list(Path(directory).iterdir()), [target])

    def test_original_controller_hash_and_cli_preserved(self):
        source = TOOLS / "audit/rea_global_queue.py"
        with source.open("rb") as stream:
            self.assertEqual(hashlib.file_digest(stream, "sha256").hexdigest(), adapter.CONTROLLER_SHA256)
        implementation = adapter.load_controller()
        self.assertIs(implementation.atomic_json, adapter.atomic_json)
        self.assertTrue(callable(implementation.QueueController))
        self.assertTrue(callable(implementation.main))

    def test_changed_controller_is_rejected_before_import(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "wrong.py"
            source.write_text("raise AssertionError('must not import')\n", encoding="utf-8")
            with self.assertRaisesRegex(RuntimeError, "pinned coverage controller changed"):
                adapter.load_controller(source)


if __name__ == "__main__":
    unittest.main()
