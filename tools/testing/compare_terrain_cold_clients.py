"""Compare synchronous and background cold terrain blending with two real native clients.

Usage: python tools/testing/compare_terrain_cold_clients.py <attempt-root>
"""
import hashlib
import json
from pathlib import Path
import statistics
import struct
import sys

ROLES=("owner","witness")
CPU_LIMIT=128*1024*1024


def compare(reports,pictures,memory):
    checks=[];metrics={}
    def check(name,passed):checks.append({"name":name,"passed":bool(passed)})
    for mode in ("reference","optimized"):
        pair=reports[mode]
        a,b=pair["owner"].get("steps",[]),pair["witness"].get("steps",[])
        check(mode+": reciprocal peer identities",len(a)==len(b)==6 and all(
            x.get("own")==y.get("peer") and x.get("peer")==y.get("own") for x,y in zip(a,b)))
        for role in ROLES:
            r=pair[role];label=mode+"/"+role;enabled=mode=="optimized"
            steps,samples,trace=r.get("steps",[]),r.get("motion_frames",[]),r.get("trace",{})
            check(label+": completed cold comparison",r.get("success") is True and not r.get("failure")
                and r.get("scenario")=="terrain-cold" and r.get("role")==role and r.get("renderer")=="optimized"
                and r.get("cold") is True and r.get("motion") is True and r.get("pixel_cache_enabled") is True
                and r.get("async_bake") is enabled and [s.get("phase") for s in steps]==list(range(6)))
            check(label+": clean network and actual login/logout",all(trace.get(k)==[] for k in
                ("errors","hits","levels","warps","cast_events","cast_results","learn_results"))
                and len(trace.get("logins",[]))==1 and all(r.get("events",{}).get(k)==1 for k in
                ("PLAYER_LOGIN","PLAYER_ENTERING_WORLD","PLAYER_LOGOUT")))
            y,py=(7227,7247) if role=="owner" else (7247,7227)
            check(label+": peer positions and gameplay preserved",len(steps)==6 and all(
                s.get("position")==[10540 if s.get("phase")==4 else 8440,y]
                and s.get("peer_position")==[10540 if s.get("phase")==4 else 8440,py]
                and [s.get(k) for k in ("level","hp","mp","xp_jp","money","layer")]==[1,100,50,[0,0],0,0]
                for s in steps))
            check(label+": both authoritative movement routes",len(steps)==6 and all(any(
                m.get("handle")==h and m.get("layer")==0 and m.get("points")==[[x,ty]]
                for m in trace.get("moves",[])) for h,ty in ((steps[0]["own"],y),(steps[0]["peer"],py))
                for x in (10540,8440)))
            check(label+": every phase sampled including moving cold entry",bool(samples) and len(samples)<12000
                and all([s.get("frame") for s in samples if s.get("phase")==phase]
                        ==list(range(len([s for s in samples if s.get("phase")==phase])))
                        and any(s.get("phase")==phase for s in samples) for phase in range(6))
                and min(s["position"][0] for s in samples if s["phase"]==4)<8500
                and max(s["position"][0] for s in samples if s["phase"]==4)>10500)
            caches=[s.get("cache",{}) for s in steps+samples]
            check(label+": full resolution and bounded visible GPU textures",bool(caches) and all(
                c.get("bake_px")==1024 and c.get("bytes_per_chunk")==4194304 and c.get("reference") is False
                and 0<=c.get("resident",-1)<=c.get("wanted",-1)<=49
                and c.get("texture_payload_bytes")==c.get("resident",-1)*4194304 for c in caches)
                and all(s["cache"].get("ready") is True for s in steps))
            check(label+": same bounded host cache in both modes",bool(caches) and all(
                c.get("cpu_pixel_limit")==CPU_LIMIT and 0<=c.get("cpu_pixel_bytes",-1)<=CPU_LIMIT for c in caches))
            check(label+": bounded uploads and no worker failure",bool(caches) and all(
                c.get("worker_error") is None and isinstance(c.get("job_active"),bool)
                and 0<=c.get("last_restores",-1)+c.get("last_completed_uploads",-1)<=4
                and 0<=c.get("last_submits",-1)<=1 and 0<=c.get("last_bakes",-1)<=1
                and not(c.get("last_bakes") and (c.get("last_restores") or c.get("last_completed_uploads")))
                and c.get("completed",0)+c.get("cancelled",0)+int(c.get("job_active",False))==c.get("submitted",0) for c in caches))
            check(label+": correct execution mode",bool(caches) and all(c.get("async_enabled") is enabled for c in caches)
                and (all(c.get("last_bakes")==0 for c in caches) if enabled else
                     all(c.get("last_submits")==0 and c.get("submitted")==0 and c.get("completed")==0 for c in caches)))
            active=[s for s in samples if (s["cache"].get("job_active") or s["cache"].get("last_submits")
                    or s["cache"].get("last_completed_uploads"))] if enabled else [s for s in samples if s["cache"].get("last_bakes")]
            check(label+": actual cold work in opposing view and new area",bool(active) and all(
                any(s["phase"]==p for s in active) for p in (1,4))
                and (max(c.get("completed",0) for c in caches)>0 if enabled else
                     sum(c.get("last_bakes",0) for c in caches)>0))
            if active:
                us=[s["cache"]["prepare_us"] for s in active]
                metrics[label]={"active_cold_frames":len(active),"prepare_total_us":sum(us),
                    "prepare_p95_us":sorted(us)[int(len(us)*.95)],"prepare_max_us":max(us),
                    "main_thread_blends":sum(s["cache"]["last_bakes"] for s in samples),
                    "submitted":max(c["submitted"] for c in caches),"completed":max(c["completed"] for c in caches),
                    "cancelled":max(c["cancelled"] for c in caches),"worker_us":max(c["worker_us"] for c in caches),
                    "cpu_pixel_peak_bytes":max(c["cpu_pixel_bytes"] for c in caches),
                    "gpu_texture_peak_bytes":max(c["peak_texture_payload_bytes"] for c in caches)}
        rows=memory.get(mode,[])
        check(mode+": Windows CPU/GPU samples for both clients in every view",all(sum(1 for r in rows
            if r.get("phase")==p and r.get("role")==role and r.get("process_id",0)>0
            and r.get("Private Bytes",0)>0 and r.get("Total Committed",0)>0)>=3 for p in range(6) for role in ROLES))
    for role in ROLES:
        a,b=metrics.get("reference/"+role),metrics.get("optimized/"+role)
        check(role+": lower cold-related preparation p95",a and b and b["prepare_p95_us"]<a["prepare_p95_us"])
        steps_a,steps_b=reports["reference"][role].get("steps",[]),reports["optimized"][role].get("steps",[])
        check(role+": identical settled GPU allocation",len(steps_a)==len(steps_b)==6 and all(
            x["cache"]["texture_payload_bytes"]==y["cache"]["texture_payload_bytes"] for x,y in zip(steps_a,steps_b)))
        images=pictures.get(role,[])
        check(role+": six exact settled ground captures",[p.get("phase") for p in images]==list(range(6))
            and all(p.get("size")==[1280,720] and p.get("reference_sha256")==p.get("optimized_sha256") for p in images))
    windows={m:{k:[sum(statistics.median(r[k] for r in rows if r["phase"]==p and r["role"]==role)
        for role in ROLES) for p in range(6)] for k in ("Private Bytes","Working Set","Total Committed")}
        for m,rows in memory.items()}
    return {"success":all(c["passed"] for c in checks),"checks":checks,"cold":metrics,
        "windows_pair_bytes_by_phase":windows,"pictures":pictures}


def load_and_compare(root):
    root=Path(root)
    reports={m:{r:json.loads((root/m/f"{r}.json").read_text(encoding="utf-8-sig")) for r in ROLES}
             for m in ("reference","optimized")}
    memory={m:json.loads((root/m/"gpu-memory.json").read_text(encoding="utf-8-sig")) for m in reports}
    pictures={}
    for role in ROLES:
        pictures[role]=[]
        for phase in range(6):
            data={m:(root/m/f"{role}-{phase}.ground.png").read_bytes() for m in reports}
            sizes=[list(struct.unpack(">II",v[16:24])) if v[:16]==b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR" else [] for v in data.values()]
            pictures[role].append({"phase":phase,"size":sizes[0] if sizes[0]==sizes[1] else [],
                **{m+"_sha256":hashlib.sha256(v).hexdigest() for m,v in data.items()}})
    return compare(reports,pictures,memory)


if __name__=="__main__":
    root=Path(sys.argv[1]);result=load_and_compare(root)
    (root/"comparison.json").write_text(json.dumps(result,indent=2),encoding="utf-8")
    print(json.dumps(result,indent=2))
    raise SystemExit(0 if result["success"] else 1)
