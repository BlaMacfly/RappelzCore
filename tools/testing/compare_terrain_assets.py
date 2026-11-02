"""Compare offline native terrain captures; baseline equality precedes the edited-asset pair."""
import argparse
import hashlib
import json
from pathlib import Path
import statistics
import struct
import zlib


def png(path):
    data = path.read_bytes()
    assert data[:8] == b"\x89PNG\r\n\x1a\n", f"invalid PNG: {path}"
    offset, dimensions, channels, payload, ended = 8, None, None, bytearray(), False
    while offset < len(data):
        size = struct.unpack_from(">I", data, offset)[0]
        kind = data[offset + 4:offset + 8]
        content = data[offset + 8:offset + 8 + size]
        crc = struct.unpack_from(">I", data, offset + 8 + size)[0]
        assert zlib.crc32(kind + content) & 0xFFFFFFFF == crc, f"PNG CRC: {path}"
        if kind == b"IHDR":
            width, height, depth, color, compression, filter_method, interlace = struct.unpack(">IIBBBBB", content)
            assert depth == 8 and color in (2, 6) and compression == filter_method == interlace == 0
            assert 0 < width <= 8192 and 0 < height <= 8192
            dimensions, channels = (width, height), 3 if color == 2 else 4
        elif kind == b"IDAT":
            payload.extend(content)
        elif kind == b"IEND":
            ended = True
            offset += 12 + size
            break
        offset += 12 + size
    assert ended and offset == len(data) and dimensions
    decoded = zlib.decompress(payload)
    width, height = dimensions
    stride = width * channels + 1
    assert len(decoded) == height * stride
    assert all(decoded[y * stride] in range(5) for y in range(height))
    return dimensions, hashlib.sha256(data).hexdigest()


def compare(control, edited, texture="zone005_grass@normal002.dds", size=1024):
    control, edited = Path(control), Path(edited)
    checks = []

    def need(ok, name):
        assert ok, name
        checks.append(name)

    def read(path):
        return json.loads(path.read_text(encoding="utf-8-sig"))

    reports = {}
    processes = {}
    measurements = {}
    images = {}
    for kind, folder in [("control", control), ("edited", edited)]:
        procs = read(folder / "processes.json")
        need({p["role"] for p in procs} == {"left", "right"} and len(procs) == 2, f"{kind}: two native processes")
        need(all(p["exit_code"] == 0 for p in procs), f"{kind}: both exits zero")
        processes[kind] = {p["role"]: p["pid"] for p in procs}
        samples = read(folder / "memory.json")
        need(len(samples) == 24, f"{kind}: 24 process-memory observations")
        measurements[kind] = {}
        for role in ["left", "right"]:
            report = read(folder / role / "report.json")
            reports[kind, role] = report
            need(report["success"] and report["offline"] and not report["network_login"], f"{kind}/{role}: successful offline session")
            need(report["input_locked"] and report["normal_renderer"] and report["audio_disabled"], f"{kind}/{role}: fixed preview controls")
            need([v["phase"] for v in report["views"]] == list(range(4)), f"{kind}/{role}: four ordered views")
            for phase, view in enumerate(report["views"]):
                cache = view["cache"]
                need(cache["ready"] and not cache["reference"] and cache["worker_error"] is None, f"{kind}/{role}/{phase}: settled normal renderer")
                need(cache["resident"] == cache["wanted"] > 0, f"{kind}/{role}/{phase}: only requested terrain")
                need(len(cache["wanted_keys"]) == cache["wanted"] and len({tuple(k) for k in cache["wanted_keys"]}) == cache["wanted"], f"{kind}/{role}/{phase}: unique requested chunk identities")
                need(cache["bake_px"] == 1024 and cache["bytes_per_chunk"] == 4 * 1024**2, f"{kind}/{role}/{phase}: bake detail unchanged")
                direct = cache.get("direct_chunks", 0)
                baked = cache.get("baked_chunks", cache["resident"])
                tiles = cache.get("direct_tiles", {})
                need(not direct or cache.get("direct_enabled", False) and report.get("experimental_direct_tiles", False), f"{kind}/{role}/{phase}: direct rendering is an explicit experiment")
                need(direct + baked == cache["resident"], f"{kind}/{role}/{phase}: chunk routes accounted")
                need(cache["texture_payload_bytes"] == baked * 4 * 1024**2 + sum(tiles.values()), f"{kind}/{role}/{phase}: GPU payload accounting")
                need(cache["texture_payload_bytes"] <= cache["wanted"] * 4 * 1024**2, f"{kind}/{role}/{phase}: no growth beyond baked terrain budget")
                for name, cost in tiles.items():
                    layer = cache["source_layers"][name]
                    width, height = layer["width"], layer["height"]
                    expected = 0
                    while True:
                        expected += width * height * 4
                        if width == height == 1: break
                        width, height = max(1, width // 2), max(1, height // 2)
                    need(cost == expected, f"{kind}/{role}/{phase}: full shared mip cost {name}")
                need(cache["cpu_pixel_bytes"] <= cache["cpu_pixel_limit"] == 128 * 1024**2, f"{kind}/{role}/{phase}: bounded CPU chunk cache")
                ground = png(folder / role / f"view-{phase}-ground.png")
                scene = png(folder / role / f"view-{phase}-scene.png")
                need(ground[0] == scene[0], f"{kind}/{role}/{phase}: complete capture dimensions")
                images[kind, role, phase] = ground
                rows = [s for s in samples if s["role"] == role and s["phase"] == phase]
                need(len(rows) == 3 and all(s["pid"] == processes[kind][role] for s in rows), f"{kind}/{role}/{phase}: current-PID memory samples")
                need(all(all(s[k] >= 0 for k in ["Dedicated Usage", "Shared Usage", "Total Committed", "working_set", "private_bytes"]) and s["working_set"] > 0 for s in rows), f"{kind}/{role}/{phase}: memory values available")
                measurements[kind][role, phase] = {key: statistics.median(s[key] for s in rows) for key in ["Dedicated Usage", "Shared Usage", "Total Committed", "working_set", "private_bytes"]}

    source_delta = []
    terrain_payloads = []
    for phase in range(4):
        base = reports["control", "left"]["views"][phase]["cache"]
        original = reports["edited", "left"]["views"][phase]["cache"]
        hd = reports["edited", "right"]["views"][phase]["cache"]
        need(images["control", "left", phase] == images["control", "right", phase], f"view{phase}: repeatable baseline pixels")
        need(images["control", "left", phase] == images["edited", "left", phase], f"view{phase}: original rendering retained across runs")
        need(images["edited", "left", phase][0] == images["edited", "right", phase][0], f"view{phase}: matched edited viewport")
        need(images["edited", "left", phase][1] != images["edited", "right", phase][1], f"view{phase}: edited tile actually changes ground pixels")
        need(base["wanted_keys"] == original["wanted_keys"] == hd["wanted_keys"], f"view{phase}: identical visible chunk identities")
        need(base["texture_payload_bytes"] == original["texture_payload_bytes"], f"view{phase}: repeatable original GPU terrain payload")
        need(original.get("direct_chunks", 0) == hd.get("direct_chunks", 0) and original.get("baked_chunks") == hd.get("baked_chunks"), f"view{phase}: same rendering routes")
        terrain_payloads.append({"original": original["texture_payload_bytes"], "hd": hd["texture_payload_bytes"], "baked_equivalent": hd["wanted"] * 4 * 1024**2})
        need(base["source_layers"] == original["source_layers"], f"view{phase}: unchanged original source cache")
        need(set(original["source_layers"]) == set(hd["source_layers"]), f"view{phase}: same named source layers")
        targets = [n for n in original["source_layers"] if n.lower() == texture.lower()]
        need(len(targets) == 1, f"view{phase}: selected original tile resolved")
        name = targets[0]
        src, dst = original["source_layers"][name], hd["source_layers"][name]
        need(src["width"] == src["height"] == 512 and src["rgba_bytes"] == 1024**2, f"view{phase}: original512 decoded")
        need(dst["width"] == dst["height"] == size and dst["rgba_bytes"] == size * size * 4, f"view{phase}: HD source decoded through MPQ")
        for other in original["source_layers"]:
            if other != name:
                need(original["source_layers"][other] == hd["source_layers"][other], f"view{phase}: unrelated layer {other}")
        delta = hd["source_rgba_bytes"] - original["source_rgba_bytes"]
        need(delta == dst["rgba_bytes"] - src["rgba_bytes"], f"view{phase}: exact source-pixel cost")
        source_delta.append(delta)

    # Driver/process counters are observations. A texture-payload invariant does
    # not imply a whole-process committed-memory invariant.
    memory = {}
    for kind in ["control", "edited"]:
        memory[kind] = {role: {key: max(measurements[kind][role, phase][key] for phase in range(4)) for key in ["Dedicated Usage", "Shared Usage", "Total Committed", "working_set", "private_bytes"]} for role in ["left", "right"]}
    return {"success": True, "checks": checks, "check_count": len(checks), "offline": True,
            "matched_views": 4, "baseline_image_pairs_equal": 4, "edited_ground_views_different": 4,
            "source_rgba_delta_bytes_per_view": source_delta, "terrain_payloads_per_view": terrain_payloads,
            "gpu_terrain_payload_unchanged": all(p["original"] == p["hd"] for p in terrain_payloads),
            "process_memory_peak_of_view_medians": memory,
            "limits": ["No original SFrame or network/gameplay interoperability validation", "Image changes establish asset use; artistic quality requires visual inspection", "Memory counters are whole-process driver observations, distinct from terrain payload", "Lower DDS mips are prepared but current terrain decode uses the top mip"]}


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("control", type=Path)
    p.add_argument("edited", type=Path)
    p.add_argument("--output", type=Path)
    args = p.parse_args()
    result = compare(args.control, args.edited)
    text = json.dumps(result, indent=2)
    if args.output:
        args.output.write_text(text + "\n", encoding="utf-8")
    print(json.dumps({k: v for k, v in result.items() if k not in ["checks", "process_memory_peak_of_view_medians"]}))
