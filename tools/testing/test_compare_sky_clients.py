import json
from pathlib import Path
import struct
import tempfile
import unittest
import zlib

from compare_sky_clients import compare, png_pixels


SKY = (150, 185, 220, 255)
GRASS = (48, 77, 28, 255)
ORIGINAL = (90, 132, 211, 255)
EDITED = (100, 148, 215, 255)


def chunk(kind, content):
    return struct.pack(">I", len(content)) + kind + content + struct.pack(">I", zlib.crc32(kind + content) & 0xFFFFFFFF)


def png(pixels, channels=4):
    rows = b"".join(b"\0" + b"".join(bytes(p[:channels]) for p in pixels[y * 2:y * 2 + 2]) for y in range(2))
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", 2, 2, 8, 6 if channels == 4 else 2, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(rows)) + chunk(b"IEND", b""))


class SkyComparisonTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        for case_index, case in enumerate(("default", "control", "edited")):
            folder = self.root / case
            folder.mkdir()
            processes, owners, ended, samples = [], [], [], []
            enabled = case != "default"
            for role_index, role in enumerate(("left", "right")):
                pid = 100 + 10 * case_index + role_index
                birth = 133000000000000000 + pid
                variant = "edited" if case == "edited" and role == "right" else "original-b" if role == "right" else "original-a"
                args = ["-data", str(self.root / "clients" / variant), "-demo", "-pos", "8440,7227", "-validate-terrain-assets", str(folder / role)]
                if enabled:
                    args.append("-sky-gradient")
                processes.append({"role": role, "variant": variant, "pid": pid, "birth_filetime": birth, "exit_code": 0})
                owners.append({"role": role, "variant": variant, "pid": pid, "birth_filetime": birth, "arguments": args, "executable": str(self.root / "RappelzRust-sky-preview.exe")})
                ended.append({"role": role, "pid": pid, "birth_filetime": birth, "exit_code": 0, "has_exited": True})
                output = folder / role
                output.mkdir()
                views = []
                for phase in range(4):
                    cache = {"ready": True, "reference": False, "worker_error": None, "resident": 1, "wanted": 1,
                             "bake_px": 1024, "bytes_per_chunk": 4 * 1024**2, "texture_payload_bytes": 4 * 1024**2,
                             "cpu_pixel_bytes": 4 * 1024**2, "cpu_pixel_limit": 128 * 1024**2, "wanted_keys": [[phase, 0]],
                             "source_layers": {"grass.dds": {"width": 512, "height": 512, "rgba_bytes": 512**2 * 4}},
                             "source_rgba_bytes": 512**2 * 4, "direct_enabled": False, "direct_chunks": 0, "baked_chunks": 1, "direct_tiles": {}}
                    size = 1024 if case == "edited" and role == "right" else 256
                    cost = size * size * 4 if enabled else 0
                    sky = {"enabled": enabled, "loaded": enabled, "opaque": enabled, "used": enabled, "drawn": enabled,
                           "source": "Textures\\sky_zone001_sky01.dds" if enabled else None,
                           "width": size if enabled else 0, "height": size if enabled else 0, "rgba_bytes": cost,
                           "texture_payload_bytes": cost, "gpu_texture_payload_bytes": cost,
                           "owned_gpu_textures": int(enabled), "runtime_mip_levels": int(enabled), "retained_source_pixel_bytes": 0,
                           "fallback": False, "fallback_reason": None, "error": None, "status": "ready" if enabled else "disabled",
                           "camera": {"direction": [0, phase / 4, -1], "projection": "perspective"} if enabled else None}
                    views.append({"phase": phase, "cache": cache, "sky": sky, "character_cutout_enabled": False, "width": 2, "height": 2})
                    original_pixels = [SKY, SKY, GRASS, GRASS] if phase != 3 else [GRASS] * 4
                    pixels = [p if not enabled or p != SKY else EDITED if case == "edited" and role == "right" else ORIGINAL for p in original_pixels]
                    for part in ("ground", "scene"):
                        (output / f"view-{phase}-{part}.png").write_bytes(png(pixels))
                    for sample in range(3):
                        samples.append({"role": role, "pid": pid, "birth_filetime": birth, "phase": phase,
                                        "timestamp": f"2026-11-03T12:{phase:02}:{sample:02}Z",
                                        "Dedicated Usage": 11, "Shared Usage": 12, "Total Committed": 23, "working_set": 100, "private_bytes": 200})
                report = {"success": True, "offline": True, "network_login": False, "failure": None,
                          "input_locked": True, "normal_renderer": True, "audio_disabled": True,
                          "experimental_sky_gradient": enabled, "experimental_direct_tiles": False, "experimental_character_cutout": False, "views": views}
                self.write(output / "report.json", report)
            self.write(folder / "processes.json", processes)
            self.write(folder / "owned-processes.local.json", owners)
            self.write(folder / "closure.local.json", {"success": True, "phase_count": 4, "failure": None, "sky_gradient": enabled, "direct_terrain": False, "processes": ended})
            self.write(folder / "memory.json", samples)

    def write(self, path, value):
        path.write_text(json.dumps(value), encoding="utf-8")

    def change(self, path, edit):
        value = json.loads(path.read_text(encoding="utf-8"))
        edit(value)
        self.write(path, value)

    def check(self, legacy=None):
        return compare(self.root / "default", self.root / "control", self.root / "edited", legacy)

    def test_complete_comparison_covers_visible_and_fully_occluded_sky_and_legacy(self):
        legacy = self.root / "legacy" / "left"
        legacy.mkdir(parents=True)
        for phase in range(4):
            (legacy / f"view-{phase}-ground.png").write_bytes((self.root / "default/left" / f"view-{phase}-ground.png").read_bytes())
        result = self.check(legacy.parent)
        self.assertTrue(result["success"])
        self.assertTrue(result["legacy_default_compared"])
        self.assertEqual([v["sky_pixels"] for v in result["sky_masks"]], [2, 2, 2, 0])
        self.assertEqual(result["sky_payloads_per_view"], [{"default": 0, "original": 262144, "edited": 4194304}] * 4)
        (legacy / "view-3-ground.png").write_bytes(png([SKY] * 4))
        with self.assertRaises(AssertionError):
            self.check(legacy.parent)

    def test_png_decoder_handles_all_filters_and_rgb_without_changing_pixels(self):
        pixels = [(10, 20, 30, 255), (40, 50, 60, 255), (70, 80, 90, 255), (100, 110, 120, 255)]
        filtered_rows = [
            ([10,20,30,255,40,50,60,255], [70,80,90,255,100,110,120,255]),
            ([10,20,30,255,30,30,30,0], [70,80,90,255,30,30,30,0]),
            ([10,20,30,255,40,50,60,255], [60,60,60,0,60,60,60,0]),
            ([10,20,30,255,35,40,45,128], [65,70,75,128,45,45,45,0]),
            ([10,20,30,255,30,30,30,0], [60,60,60,0,30,30,30,0]),
        ]
        expected = b"".join(bytes(p) for p in pixels)
        for mode, rows in enumerate(filtered_rows):
            path = self.root / f"filter-{mode}.png"
            raw = bytes([mode]) + bytes(rows[0]) + bytes([mode]) + bytes(rows[1])
            path.write_bytes(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", 2, 2, 8, 6, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b""))
            self.assertEqual(png_pixels(path)[:2], ((2, 2), expected))
        path = self.root / "rgb.png"
        path.write_bytes(png(pixels, channels=3))
        self.assertEqual(png_pixels(path)[:2], ((2, 2), expected))

    def test_rejects_ground_changes_and_changes_in_no_sky_view(self):
        for phase in (0, 3):
            path = self.root / "edited/right" / f"view-{phase}-ground.png"
            before = path.read_bytes()
            pixels = [EDITED, EDITED, GRASS, (1, 2, 3, 255)] if phase == 0 else [GRASS, GRASS, GRASS, EDITED]
            path.write_bytes(png(pixels))
            with self.assertRaises(AssertionError):
                self.check()
            path.write_bytes(before)

    def test_rejects_stale_birth_and_incomplete_process_closure(self):
        path = self.root / "edited/memory.json"
        before = path.read_text(encoding="utf-8")
        self.change(path, lambda rows: rows[0].update(birth_filetime=1))
        with self.assertRaises(AssertionError):
            self.check()
        path.write_text(before, encoding="utf-8")
        self.change(self.root / "control/closure.local.json", lambda r: r["processes"][0].update(has_exited=False))
        with self.assertRaises(AssertionError):
            self.check()

    def test_rejects_fallback_or_network_report(self):
        path = self.root / "edited/right/report.json"
        before = path.read_text(encoding="utf-8")
        for edit in [lambda r: r["views"][0]["sky"].update(fallback=True), lambda r: r.update(network_login=True), lambda r: r["views"][0]["sky"].update(used=False), lambda r: r.update(experimental_character_cutout=True), lambda r: r["views"][0].update(character_cutout_enabled=True)]:
            self.change(path, edit)
            with self.assertRaises(AssertionError):
                self.check()
            path.write_text(before, encoding="utf-8")

    def test_rejects_hidden_gpu_cost_or_changed_terrain_route(self):
        path = self.root / "edited/right/report.json"
        before = path.read_text(encoding="utf-8")
        for edit in [lambda r: r["views"][0]["sky"].update(gpu_texture_payload_bytes=4194308), lambda r: r["views"][0]["sky"].update(runtime_mip_levels=11), lambda r: r["views"][0]["cache"].update(direct_enabled=True), lambda r: r["views"][0]["cache"].update(source_rgba_bytes=0)]:
            self.change(path, edit)
            with self.assertRaises(AssertionError):
                self.check()
            path.write_text(before, encoding="utf-8")

    def test_rejects_truncated_png_and_bad_crc(self):
        path = self.root / "edited/right/view-0-ground.png"
        before = path.read_bytes()
        for broken in (before[:-3], before[:30] + bytes([before[30] ^ 1]) + before[31:]):
            path.write_bytes(broken)
            with self.assertRaises((AssertionError, struct.error, zlib.error)):
                self.check()
        path.write_bytes(before)

    def test_rejects_nondeterministic_baseline_and_wrong_mode_flags(self):
        path = self.root / "default/right/view-0-ground.png"
        before = path.read_bytes()
        path.write_bytes(png([SKY, SKY, GRASS, EDITED]))
        with self.assertRaises(AssertionError):
            self.check()
        path.write_bytes(before)
        owners = self.root / "control/owned-processes.local.json"
        before = owners.read_text(encoding="utf-8")
        self.change(owners, lambda rows: rows[0]["arguments"].append("-terrain-direct-tiles"))
        with self.assertRaises(AssertionError):
            self.check()
        owners.write_text(before, encoding="utf-8")
        self.change(self.root / "default/left/report.json", lambda r: r.update(experimental_sky_gradient=True))
        with self.assertRaises(AssertionError):
            self.check()


if __name__ == "__main__":
    unittest.main()
