"""Compare default/original-gradient/edited-gradient offline client captures and their scope."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import statistics
import struct
import zlib

from compare_terrain_assets import png as png_metadata


CLEAR = bytes((150, 185, 220, 255))
SOURCE = "Textures\\sky_zone001_sky01.dds"
MEMORY_KEYS = ("Dedicated Usage", "Shared Usage", "Total Committed", "working_set", "private_bytes")


def _paeth(left, above, upper_left):
    value = left + above - upper_left
    distances = (abs(value - left), abs(value - above), abs(value - upper_left))
    if distances[0] <= distances[1] and distances[0] <= distances[2]:
        return left
    return above if distances[1] <= distances[2] else upper_left


def png_pixels(path):
    """Validate the existing capture format/CRC, then unfilter to canonical RGBA bytes."""
    dimensions, digest = png_metadata(path)
    data = path.read_bytes()
    assert hashlib.sha256(data).hexdigest() == digest, "PNG changed during inspection"
    offset, channels, compressed = 8, None, bytearray()
    while offset < len(data):
        size = struct.unpack_from(">I", data, offset)[0]
        kind = data[offset + 4:offset + 8]
        content = data[offset + 8:offset + 8 + size]
        if kind == b"IHDR":
            assert channels is None, "duplicate PNG header"
            channels = 3 if content[9] == 2 else 4
        elif kind == b"IDAT":
            compressed.extend(content)
        offset += 12 + size
    assert channels in (3, 4)
    raw = zlib.decompress(compressed)
    width, height = dimensions
    stride = width * channels
    assert len(raw) == height * (stride + 1)
    result = bytearray(width * height * 4)
    previous = bytes(stride)
    for y in range(height):
        start = y * (stride + 1)
        filter_type = raw[start]
        row = bytearray(raw[start + 1:start + 1 + stride])
        if filter_type:
            for x in range(stride):
                left = row[x - channels] if x >= channels else 0
                above = previous[x]
                upper_left = previous[x - channels] if x >= channels else 0
                if filter_type == 1:
                    predictor = left
                elif filter_type == 2:
                    predictor = above
                elif filter_type == 3:
                    predictor = (left + above) // 2
                else:
                    predictor = _paeth(left, above, upper_left)
                row[x] = (row[x] + predictor) & 255
        if channels == 4:
            result[y * width * 4:(y + 1) * width * 4] = row
        else:
            for x in range(width):
                dst = (y * width + x) * 4
                result[dst:dst + 4] = row[x * 3:x * 3 + 3] + b"\xff"
        previous = row
    return dimensions, bytes(result), digest


def compare(default, control, edited, legacy_default=None):
    folders = {"default": Path(default).resolve(), "control": Path(control).resolve(), "edited": Path(edited).resolve()}
    checks, images, views, observations = [], {}, {}, {}
    process_identities = set()

    def need(ok, message):
        assert ok, message
        checks.append(message)

    def read(path):
        return json.loads(path.read_text(encoding="utf-8-sig"))

    def identity(record):
        return record["pid"], record["birth_filetime"]

    def terrain_identity(cache):
        return {key: cache[key] for key in ("wanted_keys", "wanted", "resident", "baked_chunks", "bake_px", "bytes_per_chunk", "texture_payload_bytes", "source_layers", "source_rgba_bytes")}

    for case, folder in folders.items():
        enabled = case != "default"
        processes = read(folder / "processes.json")
        owned = read(folder / "owned-processes.local.json")
        closure = read(folder / "closure.local.json")
        samples = read(folder / "memory.json")
        need(len(processes) == len(owned) == 2 and {p["role"] for p in processes} == {"left", "right"} and {p["role"] for p in owned} == {"left", "right"}, f"{case}: two owned native processes")
        need(closure["success"] is True and closure["phase_count"] == 4 and closure["failure"] is None, f"{case}: terminal four-view closure")
        need(closure["sky_gradient"] is enabled and closure["direct_terrain"] is False, f"{case}: closure modes")
        need(len(closure["processes"]) == 2 and {p["role"] for p in closure["processes"]} == {"left", "right"}, f"{case}: two terminal process records")
        need(len(samples) == 24, f"{case}: 24 memory observations")
        observations[case] = {}
        for role in ("left", "right"):
            proc = next(p for p in processes if p["role"] == role)
            owner = next(p for p in owned if p["role"] == role)
            ended = next(p for p in closure["processes"] if p["role"] == role)
            variant = "edited" if case == "edited" and role == "right" else "original-b" if role == "right" else "original-a"
            expected_args = ["-data", str(folder.parent / "clients" / variant), "-demo", "-pos", "8440,7227", "-validate-terrain-assets", str(folder / role)]
            if enabled:
                expected_args.append("-sky-gradient")
            need(proc["variant"] == owner["variant"] == variant and owner["arguments"] == expected_args, f"{case}/{role}: isolated offline data and exact mode arguments")
            need(Path(owner["executable"]) == folder.parent / "RappelzRust-sky-preview.exe", f"{case}/{role}: isolated preview executable")
            need(all(type(value) is int and value > 0 for value in identity(proc)), f"{case}/{role}: process PID and birth")
            need(identity(proc) == identity(owner) == identity(ended) and proc["exit_code"] == ended["exit_code"] == 0 and ended["has_exited"] is True, f"{case}/{role}: owned birth completed with exit zero")
            need(identity(proc) not in process_identities, f"{case}/{role}: fresh process lifetime")
            process_identities.add(identity(proc))
            report = read(folder / role / "report.json")
            need(report["success"] is True and report["offline"] is True and report["network_login"] is False and report["failure"] is None, f"{case}/{role}: successful offline report")
            need(report["input_locked"] is True and report["normal_renderer"] is True and report["audio_disabled"] is True, f"{case}/{role}: fixed preview controls")
            need(report["experimental_sky_gradient"] is enabled and report["experimental_direct_tiles"] is False, f"{case}/{role}: report modes")
            need(report["experimental_character_cutout"] is False, f"{case}/{role}: character experiment is separate")
            need([view["phase"] for view in report["views"]] == list(range(4)), f"{case}/{role}: four ordered views")
            for phase, view in enumerate(report["views"]):
                label = f"{case}/{role}/{phase}"
                cache, sky = view["cache"], view["sky"]
                need(view["character_cutout_enabled"] is False, f"{label}: normal character material retained")
                views[case, role, phase] = view
                need(cache["ready"] is True and cache["reference"] is False and cache["worker_error"] is None, f"{label}: settled normal terrain")
                need(cache["resident"] == cache["wanted"] > 0 and len(cache["wanted_keys"]) == cache["wanted"] and len({tuple(key) for key in cache["wanted_keys"]}) == cache["wanted"], f"{label}: only unique requested chunks")
                need(cache["direct_enabled"] is False and cache["direct_chunks"] == 0 and cache["direct_tiles"] == {} and cache["baked_chunks"] == cache["wanted"], f"{label}: baked terrain only")
                need(cache["bake_px"] == 1024 and cache["bytes_per_chunk"] == 4 * 1024**2 and cache["texture_payload_bytes"] == cache["baked_chunks"] * 4 * 1024**2, f"{label}: exact ordinary terrain payload")
                need(0 <= cache["cpu_pixel_bytes"] <= cache["cpu_pixel_limit"] == 128 * 1024**2, f"{label}: bounded CPU pixel cache")
                need(cache["source_rgba_bytes"] == sum(layer["rgba_bytes"] for layer in cache["source_layers"].values()) and all(layer["rgba_bytes"] == layer["width"] * layer["height"] * 4 for layer in cache["source_layers"].values()), f"{label}: decoded terrain source costs")
                need(sky["enabled"] is enabled and sky["fallback"] is False and sky["fallback_reason"] is None and sky["error"] is None, f"{label}: sky mode without fallback")
                if enabled:
                    size = 1024 if case == "edited" and role == "right" else 256
                    need(sky["loaded"] is True and sky["opaque"] is True and sky["drawn"] is True and sky["used"] is True and sky["status"] == "ready" and sky["source"] == SOURCE, f"{label}: requested opaque sky actually drawn")
                    need(sky["width"] == sky["height"] == size <= 2048 and sky["rgba_bytes"] == sky["texture_payload_bytes"] == sky["gpu_texture_payload_bytes"] == size * size * 4, f"{label}: exact top-level sky cost")
                    need(sky["owned_gpu_textures"] == sky["runtime_mip_levels"] == 1 and sky["retained_source_pixel_bytes"] == 0, f"{label}: one GPU level with no retained source pixels")
                    need(sky["camera"] is not None, f"{label}: explicit sky projection")
                else:
                    need(sky["loaded"] is False and sky["opaque"] is False and sky["drawn"] is False and sky["used"] is False and sky["status"] == "disabled" and sky["source"] is None, f"{label}: default sky disabled")
                    need(all(sky[key] == 0 for key in ("width", "height", "rgba_bytes", "texture_payload_bytes", "gpu_texture_payload_bytes", "owned_gpu_textures", "runtime_mip_levels", "retained_source_pixel_bytes")), f"{label}: disabled sky has zero image cost")
                ground = png_pixels(folder / role / f"view-{phase}-ground.png")
                scene = png_metadata(folder / role / f"view-{phase}-scene.png")
                need(ground[0] == scene[0] == (view["width"], view["height"]), f"{label}: complete matched capture dimensions")
                images[case, role, phase] = ground
                rows = [s for s in samples if s["role"] == role and s["phase"] == phase]
                need(len(rows) == 3 and all(identity(row) == identity(proc) for row in rows), f"{label}: current-birth memory samples")
                need(len({s["timestamp"] for s in rows}) == 3 and all(all(type(s[key]) in (int, float) and math.isfinite(s[key]) and s[key] >= 0 for key in MEMORY_KEYS) and s["working_set"] > 0 for s in rows), f"{label}: three valid memory measurements")
                observations[case][role, phase] = {key: statistics.median(s[key] for s in rows) for key in MEMORY_KEYS}

    masks, sky_costs = [], []
    for phase in range(4):
        reference = images["default", "left", phase]
        baseline = views["default", "left", phase]["cache"]
        for case, role in [(case, role) for case in folders for role in ("left", "right")]:
            need(images[case, role, phase][0] == reference[0], f"view{phase}/{case}/{role}: identical viewport")
            need(terrain_identity(views[case, role, phase]["cache"]) == terrain_identity(baseline), f"view{phase}/{case}/{role}: unchanged terrain geometry and sources")
        for case in ("default", "control"):
            need(images[case, "left", phase][1] == images[case, "right", phase][1], f"view{phase}/{case}: repeatable baseline pixels")
        need(images["edited", "left", phase][1] == images["control", "left", phase][1], f"view{phase}: original sky retained across runs")
        original_view = views["control", "left", phase]["sky"]
        for case, role in [("control", "right"), ("edited", "left"), ("edited", "right")]:
            need(views[case, role, phase]["sky"]["camera"] == original_view["camera"], f"view{phase}/{case}/{role}: unchanged sky camera")
        default_pixels, original_pixels, edited_pixels = reference[1], images["control", "left", phase][1], images["edited", "right", phase][1]
        sky_pixels = changed_original = changed_edit = escaped_original = escaped_edit = 0
        for offset in range(0, len(default_pixels), 4):
            before = default_pixels[offset:offset + 4]
            original = original_pixels[offset:offset + 4]
            after = edited_pixels[offset:offset + 4]
            if before == CLEAR:
                sky_pixels += 1
                changed_original += original != before
                changed_edit += after != original
            else:
                escaped_original += original != before
                escaped_edit += after != before
        need(escaped_original == escaped_edit == 0, f"view{phase}: sky changes do not escape the original background mask")
        if sky_pixels:
            need(changed_original > 0 and changed_edit > 0, f"view{phase}: original and edited sky visibly replace background")
        else:
            need(default_pixels == original_pixels == edited_pixels, f"view{phase}: no-sky image remains identical")
        masks.append({"phase": phase, "sky_pixels": sky_pixels, "original_changed_pixels": changed_original, "edited_changed_pixels": changed_edit, "outside_mask_changes": 0})
        sky_costs.append({"default": 0, "original": original_view["texture_payload_bytes"], "edited": views["edited", "right", phase]["sky"]["texture_payload_bytes"]})
        if legacy_default is not None:
            legacy = Path(legacy_default)
            if (legacy / "left").is_dir():
                legacy /= "left"
            old = png_pixels(legacy / f"view-{phase}-ground.png")
            need(old[:2] == reference[:2], f"view{phase}: legacy default pixels preserved")
    need(any(row["sky_pixels"] > 0 for row in masks), "at least one view exposes the sky")
    need(any(row["sky_pixels"] == 0 for row in masks), "at least one view has no sky")
    memory = {case: {role: {key: max(observations[case][role, phase][key] for phase in range(4)) for key in MEMORY_KEYS} for role in ("left", "right")} for case in folders}
    return {"success": True, "offline": True, "check_count": len(checks), "checks": checks,
            "runs": 3, "native_processes": 6, "matched_views": 4, "memory_samples_per_pair": 24,
            "legacy_default_compared": legacy_default is not None, "sky_masks": masks, "sky_payloads_per_view": sky_costs,
            "process_memory_peak_of_view_medians": memory,
            "limits": ["Checks prove background change and containment, not original sky shader or skybox fidelity", "No network/gameplay or original SFrame validation", "GPU image payload is distinct from total driver/process memory", "No runtime GPU mip-chain cost is assumed: this path uses one top level"]}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("default", "control", "edited"):
        parser.add_argument(name, type=Path)
    parser.add_argument("--legacy-default", type=Path, help="older default pair or its left capture directory")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = compare(args.default, args.control, args.edited, args.legacy_default)
    if args.output:
        args.output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({key: value for key, value in result.items() if key not in ("checks", "process_memory_peak_of_view_medians")}))
