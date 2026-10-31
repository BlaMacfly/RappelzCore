"""Native two-client warm terrain streaming regression; reference/ disables only the CPU cache.

Usage: python tools/testing/compare_terrain_motion_clients.py <attempt-root>
"""
import hashlib
import json
from pathlib import Path
import statistics
import struct
import sys

ROLES = ("owner", "witness")
CPU_LIMIT = 128 * 1024 * 1024


def compare(reports, pictures, memory):
    checks, metrics = [], {}

    def check(name, passed):
        checks.append({"name": name, "passed": bool(passed)})

    for mode in ("reference", "optimized"):
        pair = reports[mode]
        a, b = pair["owner"].get("steps", []), pair["witness"].get("steps", [])
        check(mode + ": reciprocal peer identities", len(a) == len(b) == 6 and all(
            x.get("own") == y.get("peer") and x.get("peer") == y.get("own") for x, y in zip(a, b)))
        for role in ROLES:
            r = pair[role]
            label = mode + "/" + role
            steps, samples, trace = r.get("steps", []), r.get("motion_frames", []), r.get("trace", {})
            enabled = mode == "optimized"
            check(label + ": completed guarded motion scenario", r.get("success") is True
                  and not r.get("failure") and r.get("scenario") == "terrain-motion" and r.get("role") == role
                  and r.get("renderer") == "optimized" and r.get("motion") is True
                  and r.get("pixel_cache_enabled") is enabled
                  and [s.get("phase") for s in steps] == list(range(6)))
            check(label + ": clean network and gameplay unchanged", all(trace.get(k) == [] for k in
                  ("errors", "hits", "levels", "warps", "cast_events", "cast_results", "learn_results"))
                  and len(trace.get("logins", [])) == 1 and all(r.get("events", {}).get(k) == 1
                  for k in ("PLAYER_LOGIN", "PLAYER_ENTERING_WORLD", "PLAYER_LOGOUT")))
            y, peer_y = (7227, 7247) if role == "owner" else (7247, 7227)
            check(label + ": synchronized routes and unchanged stats", len(steps) == 6 and all(
                s.get("position") == [10540 if s.get("phase") == 4 else 8440, y]
                and s.get("peer_position") == [10540 if s.get("phase") == 4 else 8440, peer_y]
                and [s.get(k) for k in ("level", "hp", "mp", "xp_jp", "money", "layer")] == [1,100,50,[0,0],0,0]
                for s in steps))
            check(label + ": both authoritative movement routes", len(steps) == 6 and all(any(
                m.get("handle") == h and m.get("layer") == 0 and m.get("points") == [[x, ty]]
                for m in trace.get("moves", [])) for h, ty in
                ((steps[0]["own"], y), (steps[0]["peer"], peer_y)) for x in (10540,8440)))
            turn = [s for s in samples if s.get("phase") == 2]
            moving = [s for s in samples if s.get("phase") == 4]
            home = [s for s in samples if s.get("phase") == 5]
            check(label + ": twelve eight-frame views and moving samples", len(turn) == 96
                  and [s.get("frame") for s in turn] == list(range(96))
                  and [s.get("view") for s in turn] == [(i//8)%2 for i in range(96)]
                  and len(moving) >= 96 and bool(home) and len(samples) < 12000
                  and min(s["position"][0] for s in moving) < 8500
                  and max(s["position"][0] for s in moving) > 10500)
            caches = [s.get("cache", {}) for s in steps + samples]
            check(label + ": exact texture density and GPU visibility bound", bool(caches) and all(
                c.get("bake_px") == 1024 and c.get("bytes_per_chunk") == 4194304
                and c.get("reference") is False and 0 <= c.get("resident", -1) <= c.get("wanted", -1) <= 49
                and c.get("texture_payload_bytes") == c.get("resident", -1)*4194304
                for c in caches) and all(s["cache"].get("ready") is True for s in steps))
            check(label + ": bounded lossless CPU cache", bool(caches) and all(
                c.get("cpu_pixel_limit") == (CPU_LIMIT if enabled else 0)
                and 0 <= c.get("cpu_pixel_bytes", -1) <= (CPU_LIMIT if enabled else 0) for c in caches))
            check(label + ": bounded uploads and cold blends", bool(caches) and all(
                0 <= c.get("last_restores", -1) <= 4 and 0 <= c.get("last_bakes", -1) <= 1
                and not (c.get("last_restores", 0) and c.get("last_bakes", 0)) for c in caches))
            check(label + ": hot rotations reuse pixels without reblending" if enabled else label + ": control reblends pixels", bool(turn) and
                  (all(s["cache"].get("last_bakes") == 0 for s in turn)
                   and sum(s["cache"].get("last_restores", 0) for s in turn) > 0 if enabled else
                   sum(s["cache"].get("last_bakes", 0) for s in turn) > 0
                   and all(s["cache"].get("last_restores") == 0 for s in turn)))
            if turn:
                us = [s["cache"]["prepare_us"] for s in turn]
                metrics[label] = {"turn_frames":len(turn),
                    "missing_chunk_frames":sum(s["cache"]["missing"] for s in turn),
                    "prepare_total_us":sum(us),"prepare_p95_us":sorted(us)[int(len(us)*.95)],
                    "prepare_max_us":max(us),"cold_blends":sum(s["cache"]["last_bakes"] for s in turn),
                    "restores":sum(s["cache"]["last_restores"] for s in turn),
                    "cpu_pixel_peak_bytes":max(c["cpu_pixel_bytes"] for c in caches),
                    "gpu_texture_peak_bytes":max(c["peak_texture_payload_bytes"] for c in caches)}
        rows = memory.get(mode, [])
        check(mode + ": Windows CPU/GPU samples for both PIDs in all six views", all(
            sum(1 for r in rows if r.get("role") == role and r.get("phase") == phase
                and r.get("process_id",0)>0 and r.get("Private Bytes",0)>0 and r.get("Total Committed",0)>0)>=3
            for role in ROLES for phase in range(6)))
    for role in ROLES:
        before, after = metrics.get("reference/"+role), metrics.get("optimized/"+role)
        check(role + ": fewer unready requested chunks during identical rotations", before and after and
              after["missing_chunk_frames"] < before["missing_chunk_frames"])
        check(role + ": lower terrain preparation cost during rotations", before and after and
              after["prepare_total_us"] < before["prepare_total_us"])
        a, b = reports["reference"][role].get("steps", []), reports["optimized"][role].get("steps", [])
        check(role + ": unchanged settled GPU texture allocation", len(a) == len(b) == 6 and all(
            x["cache"]["texture_payload_bytes"] == y["cache"]["texture_payload_bytes"] for x,y in zip(a,b)))
        images = pictures.get(role, [])
        check(role + ": all six settled ground captures identical", [p.get("phase") for p in images] == list(range(6))
              and all(p.get("size") == [1280,720] and p.get("reference_sha256") == p.get("optimized_sha256") for p in images))
    windows = {}
    for mode, rows in memory.items():
        windows[mode] = {key:[sum(statistics.median(r[key] for r in rows if r["phase"]==p and r["role"]==role)
                                  for role in ROLES) for p in range(6)]
                         for key in ("Private Bytes","Working Set","Total Committed")}
    return {"success":all(c["passed"] for c in checks),"checks":checks,"motion":metrics,
            "windows_pair_bytes_by_phase":windows,"pictures":pictures}


def load_and_compare(root):
    root=Path(root)
    reports={mode:{role:json.loads((root/mode/f"{role}.json").read_text(encoding="utf-8-sig"))
                   for role in ROLES} for mode in ("reference","optimized")}
    memory={mode:json.loads((root/mode/"gpu-memory.json").read_text(encoding="utf-8-sig")) for mode in reports}
    pictures={}
    for role in ROLES:
        pictures[role]=[]
        for phase in range(6):
            data={mode:(root/mode/f"{role}-{phase}.ground.png").read_bytes() for mode in reports}
            sizes=[list(struct.unpack(">II",v[16:24])) if v[:16]==b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR" else [] for v in data.values()]
            pictures[role].append({"phase":phase,"size":sizes[0] if sizes[0]==sizes[1] else [],
                **{mode+"_sha256":hashlib.sha256(v).hexdigest() for mode,v in data.items()}})
    return compare(reports,pictures,memory)


if __name__=="__main__":
    root=Path(sys.argv[1]);result=load_and_compare(root)
    (root/"comparison.json").write_text(json.dumps(result,indent=2),encoding="utf-8")
    print(json.dumps(result,indent=2))
    raise SystemExit(0 if result["success"] else 1)
