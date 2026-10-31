"""Compare two native loopback clients, full-resolution ground images and Windows GPU counters.

Usage: python tools/testing/compare_terrain_clients.py <attempt-root>
The root contains reference/ and optimized/, each with owner/witness JSON, PNGs and GPU samples.
"""
import hashlib
import json
from pathlib import Path
import statistics
import struct
import sys

ROLES = ("owner", "witness")


def compare(reference, optimized, pictures, gpu):
    checks = []
    metrics = {}

    def check(name, passed):
        checks.append({"name": name, "passed": bool(passed)})

    for renderer, reports in (("reference", reference), ("optimized", optimized)):
        owner_steps = reports["owner"].get("steps", [])
        witness_steps = reports["witness"].get("steps", [])
        check(renderer + ": reciprocal peer identities", len(owner_steps) == len(witness_steps) == 6 and all(
            a.get("own") == b.get("peer") and a.get("peer") == b.get("own")
            for a, b in zip(owner_steps, witness_steps)))
        for role in ROLES:
            r = reports[role]
            label = f"{renderer}/{role}"
            steps = r.get("steps", [])
            trace = r.get("trace", {})
            check(label + ": completed all six views", r.get("success") is True and not r.get("failure")
                  and r.get("scenario") == "terrain" and r.get("renderer") == renderer and r.get("role") == role
                  and [s.get("phase") for s in steps] == list(range(6)))
            check(label + ": clean network and no combat/warp/reward", all(trace.get(k) == [] for k in
                  ("errors", "hits", "levels", "warps", "cast_events", "cast_results", "learn_results")))
            check(label + ": login/logout events", all(r.get("events", {}).get(k) == 1 for k in
                  ("PLAYER_LOGIN", "PLAYER_ENTERING_WORLD", "PLAYER_LOGOUT")) and len(trace.get("logins", [])) == 1)
            y = 7227 if role == "owner" else 7247
            peer_y = 7247 if role == "owner" else 7227
            check(label + ": peer synchronized through two chunk crossings and return", len(steps) == 6 and all(
                s.get("position") == [10540 if s.get("phase") == 4 else 8440, y]
                and s.get("peer_position") == [10540 if s.get("phase") == 4 else 8440, peer_y]
                and s.get("own") != s.get("peer") for s in steps))
            check(label + ": no stat/experience/money change", bool(steps) and all(
                [s.get(k) for k in ("level", "hp", "mp", "xp_jp", "layer", "money")] == [1, 100, 50, [0, 0], 0, 0]
                for s in steps))
            check(label + ": full resolution and all visible chunks ready", bool(steps) and all(
                s.get("cache", {}).get("bake_px") == 1024 and s["cache"].get("bytes_per_chunk") == 4194304
                and s["cache"].get("ready") is True and s["cache"].get("reference") == (renderer == "reference")
                and s["cache"].get("texture_payload_bytes") == s["cache"].get("resident", -1) * 4194304
                for s in steps))
            if len(steps) == 6:
                own, peer = steps[0]["own"], steps[0]["peer"]
                check(label + ": both movement routes received from server", all(any(
                    m.get("handle") == handle and m.get("points") == [[x, ty]] and m.get("layer") == 0
                    for m in trace.get("moves", []))
                    for handle, ty in ((own, y), (peer, peer_y)) for x in (10540, 8440)))
            else:
                check(label + ": both movement routes received from server", False)
    for role in ROLES:
        before, after = reference[role].get("steps", []), optimized[role].get("steps", [])
        check(role + ": invisible chunks evicted", len(after) == 6 and all(
            s["cache"].get("resident") == s["cache"].get("wanted") for s in after)
            and after[-1]["cache"].get("evicted", 0) > 0)
        check(role + ": lower terrain texture peak", len(before) == len(after) == 6 and
              max(s["cache"]["peak_texture_payload_bytes"] for s in after)
              < max(s["cache"]["peak_texture_payload_bytes"] for s in before))
        images = pictures.get(role, [])
        check(role + ": six exact matching ground captures", [p.get("phase") for p in images] == list(range(6))
              and all(p.get("reference_sha256") == p.get("optimized_sha256")
                      and p.get("size") == [1280, 720] for p in images))
        if before and after:
            b = max(s["cache"]["peak_texture_payload_bytes"] for s in before)
            a = max(s["cache"]["peak_texture_payload_bytes"] for s in after)
            metrics[role] = {"terrain_peak_before_bytes": b, "terrain_peak_after_bytes": a,
                             "terrain_peak_reduction_percent": 100 * (1 - a / b)}
    memory = {}
    for renderer in ("reference", "optimized"):
        rows = gpu.get(renderer, [])
        valid = all(sum(1 for r in rows if r.get("phase") == phase and r.get("role") == role
                        and r.get("renderer") == renderer and r.get("process_id", 0) > 0
                        and r.get("Dedicated Usage", -1) >= 0 and r.get("Shared Usage", -1) >= 0
                        and r.get("Total Committed", 0) > 0) >= 3
                    for phase in range(6) for role in ROLES)
        check(renderer + ": Windows GPU samples for both clients in every view", valid)
        if valid:
            # Sum per-client medians; this is whole-process driver accounting, not just terrain.
            memory[renderer] = [sum(statistics.median(r["Total Committed"] for r in rows
                                    if r["phase"] == phase and r["role"] == role) for role in ROLES)
                                for phase in range(6)]
    check("lower peak Windows GPU committed memory for the pair", len(memory) == 2 and
          max(memory["optimized"]) < max(memory["reference"]))
    return {"success": all(c["passed"] for c in checks), "checks": checks,
            "terrain": metrics, "pair_windows_gpu_committed_bytes_by_phase": memory,
            "pictures": pictures}


def load_and_compare(directory):
    directory = Path(directory)
    reports = {renderer: {role: json.loads((directory / renderer / f"{role}.json").read_text(encoding="utf-8-sig"))
                         for role in ROLES} for renderer in ("reference", "optimized")}
    pictures = {}
    for role in ROLES:
        pictures[role] = []
        for phase in range(6):
            data = {r: (directory / r / f"{role}-{phase}.ground.png").read_bytes()
                    for r in ("reference", "optimized")}
            sizes = []
            for value in data.values():
                if value[:16] != b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR":
                    raise ValueError("ground capture must be a PNG")
                sizes.append(list(struct.unpack(">II", value[16:24])))
            pictures[role].append({"phase":phase, "size":sizes[0] if sizes[0] == sizes[1] else [],
                **{f"{r}_sha256":hashlib.sha256(value).hexdigest() for r, value in data.items()}})
    gpu = {r: json.loads((directory / r / "gpu-memory.json").read_text(encoding="utf-8-sig"))
           for r in ("reference", "optimized")}
    return compare(reports["reference"], reports["optimized"], pictures, gpu)


if __name__ == "__main__":
    root = Path(sys.argv[1])
    result = load_and_compare(root)
    (root / "comparison.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
    print(json.dumps(result, indent=2))
    raise SystemExit(0 if result["success"] else 1)
