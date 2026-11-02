import copy
import json
from pathlib import Path
import struct
import tempfile
import unittest
import zlib

from compare_terrain_assets import compare


def png(value):
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", 2, 2, 8, 6, 0, 0, 0)) + chunk(b"IDAT", zlib.compress((b"\0" + bytes([value, 70, 20, 255]) * 2) * 2)) + chunk(b"IEND", b"")


class AssetComparisonTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        for kind in ["control", "hd"]:
            folder = self.root / kind
            folder.mkdir()
            processes = [{"role": role, "pid": pid, "exit_code": 0} for role, pid in [("left", 101), ("right", 102)]]
            (folder / "processes.json").write_text(json.dumps(processes))
            samples = []
            for role, pid in [("left", 101), ("right", 102)]:
                edited = kind == "hd" and role == "right"
                output = folder / role
                output.mkdir()
                side = 1024 if edited else 512
                cache = {"ready": True, "reference": False, "worker_error": None, "resident": 15, "wanted": 15,
                         "bake_px": 1024, "bytes_per_chunk": 4 * 1024**2, "texture_payload_bytes": 60 * 1024**2,
                         "cpu_pixel_bytes": 60 * 1024**2, "cpu_pixel_limit": 128 * 1024**2, "wanted_keys": [[1, i] for i in range(15)],
                         "source_layers": {"zone005_grass@normal002.DDS": {"width": side, "height": side, "rgba_bytes": side**2 * 4}},
                         "source_rgba_bytes": side**2 * 4}
                report = {"success": True, "offline": True, "network_login": False, "input_locked": True, "normal_renderer": True, "audio_disabled": True,
                          "views": [{"phase": i, "cache": copy.deepcopy(cache)} for i in range(4)]}
                (output / "report.json").write_text(json.dumps(report))
                for phase in range(4):
                    for part in ["ground", "scene"]:
                        (output / f"view-{phase}-{part}.png").write_bytes(png(40 if edited else 30))
                    for _ in range(3):
                        samples.append({"role": role, "pid": pid, "phase": phase, "Dedicated Usage": 1, "Shared Usage": 1, "Total Committed": 2, "working_set": 100, "private_bytes": 200})
            (folder / "memory.json").write_text(json.dumps(samples))

    def change(self, kind, role, edit):
        p = self.root / kind / role / "report.json"
        v = json.loads(p.read_text())
        edit(v)
        p.write_text(json.dumps(v))

    def check(self):
        return compare(self.root / "control", self.root / "hd")

    def test_complete_comparison_keeps_source_cost_separate_from_gpu_payload(self):
        result = self.check()
        self.assertTrue(result["success"])
        self.assertEqual(result["source_rgba_delta_bytes_per_view"], [3 * 1024**2] * 4)

    def test_rejects_missing_asset_use_or_nondeterministic_baseline(self):
        p = self.root / "hd/right/view-0-ground.png"
        p.write_bytes(png(30))
        with self.assertRaises(AssertionError): self.check()
        p.write_bytes(png(40))
        (self.root / "control/right/view-0-ground.png").write_bytes(png(31))
        with self.assertRaises(AssertionError): self.check()

    def test_rejects_network_mode_wrong_dimensions_and_changed_gpu_budget(self):
        for edit in [lambda r: r.update(network_login=True),
                     lambda r: r["views"][0]["cache"].update(texture_payload_bytes=61 * 1024**2),
                     lambda r: r["views"][0]["cache"]["source_layers"]["zone005_grass@normal002.DDS"].update(width=512)]:
            p = self.root / "hd/right/report.json"
            original = p.read_text()
            self.change("hd", "right", edit)
            with self.assertRaises(AssertionError): self.check()
            p.write_text(original)

    def test_rejects_stale_memory_process_and_truncated_capture(self):
        p = self.root / "hd/memory.json"
        original = p.read_text()
        rows = json.loads(original)
        rows[0]["pid"] = 999
        p.write_text(json.dumps(rows))
        with self.assertRaises(AssertionError): self.check()
        p.write_text(original)
        p = self.root / "hd/right/view-0-ground.png"
        p.write_bytes(p.read_bytes()[:-4])
        with self.assertRaises((AssertionError, struct.error)): self.check()

    def test_shared_tiles_account_for_mips_and_reject_a_hidden_cost(self):
        for kind in ["control", "hd"]:
            for role in ["left", "right"]:
                def route(report):
                    report["experimental_direct_tiles"]=True
                    for view in report["views"]:
                        cache=view["cache"]
                        size=1024 if kind=="hd" and role=="right" else 512
                        cost=sum((size >> level)**2*4 for level in range(size.bit_length()))
                        cache.update(direct_enabled=True,direct_chunks=15,baked_chunks=0,direct_tiles={"zone005_grass@normal002.DDS":cost},texture_payload_bytes=cost)
                self.change(kind, role, route)
        result=self.check()
        self.assertFalse(result["gpu_terrain_payload_unchanged"])
        self.change("hd","right",lambda r:r["views"][0]["cache"]["direct_tiles"].update({"zone005_grass@normal002.DDS":4*1024**2}))
        with self.assertRaises(AssertionError): self.check()


if __name__ == "__main__":
    unittest.main()
