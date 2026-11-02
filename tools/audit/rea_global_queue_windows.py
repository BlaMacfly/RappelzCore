"""Run the pinned REA coverage controller with short sibling temp filenames.

The original controller is immutable evidence in earlier audit checkpoints.
This compatibility entry point changes only mutable JSON publication: its
temporary basename no longer grows with the destination filename. It keeps
the same controller, queue, operation identities, sharing retries and CLI.
No analysis provider or target executable is launched by this module.
"""
from __future__ import annotations

import contextlib
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import sys
import tempfile
import time

CONTROLLER_SHA256 = "8b689d9cc512cb4480382905c722ca158e5e01691d175335f2ab8efd1cdb3732"


def atomic_json(path: Path, value):
    """Replace mutable controller state using a same-directory short temp file."""
    path = Path(path)
    descriptor, temporary_name = tempfile.mkstemp(prefix=".q-", suffix=".tmp", dir=path.parent)
    temporary = Path(temporary_name)
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8", newline="\n") as stream:
            json.dump(value, stream, ensure_ascii=False, indent=2)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
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


def load_controller(module_path=None):
    path = Path(module_path) if module_path is not None else Path(__file__).with_name("rea_global_queue.py")
    with path.open("rb") as stream:
        actual = hashlib.file_digest(stream, "sha256").hexdigest()
    if actual != CONTROLLER_SHA256:
        raise RuntimeError("The pinned coverage controller changed; review a new compatibility version first")
    name = "_rea_global_queue_short_temp_compatibility"
    spec = importlib.util.spec_from_file_location(name, path)
    implementation = importlib.util.module_from_spec(spec)
    sys.modules[name] = implementation
    spec.loader.exec_module(implementation)
    implementation.atomic_json = atomic_json
    return implementation


def main(argv=None):
    return load_controller().main(argv)


if __name__ == "__main__":
    raise SystemExit(main())
