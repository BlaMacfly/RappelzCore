"""Check native WARP packets, peer visibility and causal partial terrain cancellation.

Usage: python tools/testing/compare_terrain_warp_clients.py <attempt-directory>
An optional second directory supplies the legacy-mode ground-image reference.
"""
import hashlib
import json
from pathlib import Path
import struct
import sys
from compare_terrain_cancellation_clients import audit_facts

ROLES=("owner","witness")
PHASES=("initial",)+tuple(f"leg-{i}" for i in range(5))
ROUTE=((8500,7227),(12000,7227),(72952,23355),(89080,23355),(8440,7227))

def destination(role,i):
    x,y=ROUTE[i];return [x,y+(20 if role=="witness" else 0)]

def compare(reports,pictures,memory):
    checks=[];facts={}
    def check(name,value):checks.append(dict(name=name,passed=bool(value)))
    for role in ROLES:
        r=reports[role];other=reports["witness" if role=="owner" else "owner"]
        t=r.get("trace",{});steps=r.get("steps",[]);audits=r.get("audits",[])
        fs=[audit_facts(a) for a in audits];facts[role]=fs
        check(role+": native route completed",r.get("scenario")=="terrain-warp" and r.get("success") is True and not r.get("failure") and r.get("role")==role)
        check(role+": one bounded causal joined world worker",len(fs)==1 and not fs[0]["failures"])
        check(role+": five authentic WARP packets",t.get("warps")==[dict(x=x,y=y,z=0,layer=0) for x,y in (destination(role,i) for i in range(5))])
        requests=[s for s in steps if s.get("stage")=="WarpRequested"]
        applied=[s for s in steps if s.get("stage")=="WarpApplied"]
        reunited=[s for s in steps if s.get("stage")=="PairReunited"]
        check(role+": five requests bound to actual partial jobs",len(requests)==5 and [s.get("leg") for s in requests]==list(range(5)) and len(fs)==1 and all(
            s.get("job") in fs[0]["partial_warp_ids"] and s.get("cache",{}).get("active_job",{}).get("id")==s.get("job")
            and s["cache"]["active_job"].get("running") is True and 0<s["cache"]["active_job"].get("rows",0)<1024 for s in requests))
        events=audits[0].get("events",[]) if len(audits)==1 else []
        warp_events=[e for e in events if e.get("event")=="warp"]
        check(role+": cancellation precedes completion at each WARP",len(warp_events)==5 and len(requests)==5 and len({s.get("job") for s in requests})==5 and all(
            any(c.get("event")=="cancel_warp" and c.get("id")==s.get("job") and e["order"]<c["order"]
                and (i==4 or c["order"]<warp_events[i+1]["order"])
                and any(f.get("event")=="finished" and f.get("id")==s["job"] and c["order"]<f["order"] for f in events) for c in events)
            and e.get("key")==[v//1008 for v in destination(role,i)] for i,(s,e) in enumerate(zip(requests,warp_events))))
        check(role+": new-map pixels uploaded in the destination namespace",len(warp_events)==5 and all(any(
            e.get("event")=="uploaded" and w["order"]<e["order"]<(warp_events[i+1]["order"] if i<4 else len(events))
            and [k//16 for k in e.get("key",[])]==[v//16128 for v in destination(role,i)] for e in events)
            for i,w in enumerate(warp_events) if i>=2))
        check(role+": old pending pixels never accepted",len(fs)==1 and all(not any(e.get("id")==s.get("job") and e.get("event") in ("cache_accept","uploaded") for e in events) for s in requests))
        check(role+": short WARP overlaps the former wanted chunk",len(requests)==len(applied)==5 and requests[0].get("cache",{}).get("active_job",{}).get("key") in applied[0].get("cache",{}).get("wanted_keys",[]))
        check(role+": actual target positions and map selection",len(applied)==5 and all(s.get("position")==destination(role,i)
            and s.get("cache",{}).get("focus_map")==[v//16128 for v in destination(role,i)] and len(s.get("warps",[]))==i+1 for i,s in enumerate(applied)))
        check(role+": pair reunited after every authoritative refresh",len(reunited)==5 and all(s.get("position")==destination(role,i)
            and s.get("peer_position")==destination("witness" if role=="owner" else "owner",i) for i,s in enumerate(reunited)))
        logins=t.get("logins",[]);pl=other.get("trace",{}).get("logins",[])
        check(role+": stable player handles and one normal login/logout",len(logins)==len(pl)==1 and all(s.get("own")==logins[0].get("handle") for s in steps)
            and all(r.get("events",{}).get(k)==1 for k in ("PLAYER_LOGIN","PLAYER_ENTERING_WORLD","PLAYER_LOGOUT")) and t.get("disconnects")==[])
        ph=pl[0].get("handle") if len(pl)==1 else None
        check(role+": server ENTER snapshots match peer in all destinations",ph is not None and all(any(e.get("handle")==ph and e.get("name")=="<GM>"+other.get("character","")
            and e.get("kind")=="player" and [e.get("x"),e.get("y")]==destination("witness" if role=="owner" else "owner",i)
            and [e.get(k) for k in ("layer","hp","mp","level")]==[0,100,50,1] for e in t.get("enters",[])) for i in range(5)))
        if role=="witness":
            seen=[s for s in steps if s.get("stage")=="ObservedOwner"]
            check("witness: near peer refresh then four departures",len(seen)==5 and seen[0].get("peer_position")==destination("owner",0)
                and all(s.get("peer_position") is None for s in seen[1:]) and t.get("leaves",[]).count(ph)>=4)
        check(role+": player stats and layer preserved",bool(steps) and all(s.get("player",{}).get("level")==1 and s["player"].get("hp")==100
            and s["player"].get("mp")==50 and s.get("xp_jp")==[0,0] and s.get("money")==0 and s.get("layer")==0 for s in steps))
        check(role+": no decoder or combat errors",all(t.get(k)==[] for k in ("errors","hits","levels","cast_events","cast_results","learn_results")))
        samples=r.get("samples",[])
        check(role+": bounded resources and three main-map changes",0<len(samples)<16000 and samples[-1].get("warp_count")==5 and samples[-1].get("map_switches")==3 and all(
            s.get("bake_px")==1024 and s.get("cpu_pixel_limit")==134217728 and 0<=s.get("cpu_pixel_bytes",-1)<=134217728
            and s.get("worker_error") is None and s.get("async_enabled") is True and s.get("last_bakes")==0
            and 0<=s.get("resident",-1)<=s.get("wanted",-1)<=49 and s.get("texture_payload_bytes")==s.get("resident",-1)*4194304
            and 0<=s.get("last_restores",-1)+s.get("last_completed_uploads",-1)<=4
            and s.get("completed",0)+s.get("cancelled",0)+int(s.get("job_active",False))==s.get("submitted",0) for s in samples))
        p=pictures.get(role,{})
        check(role+": six sharp ground captures and identical return",set(p)==set(PHASES) and all(p.get(x,{}).get("size")==[1280,720] and len(p[x].get("sha256",""))==64 for x in PHASES)
            and p.get("initial",{}).get("sha256")==p.get("leg-4",{}).get("sha256"))
        check(role+": sampled CPU/GPU at every destination",all(sum(m.get("role")==role and m.get("phase")==phase and m.get("process_id",0)>0
            and m.get("Private Bytes",0)>0 and m.get("Total Committed",0)>0 for m in memory)>=3 for phase in PHASES))
    return dict(success=all(c["passed"] for c in checks),checks=checks,workers=facts,pictures=pictures)

def load_and_compare(root,reference=None):
    root=Path(root);reports={r:json.loads((root/f"{r}.json").read_text(encoding="utf-8-sig")) for r in ROLES};pictures={}
    for role in ROLES:
        pictures[role]={}
        for phase in PHASES:
            data=(root/f"{role}-{phase}.ground.png").read_bytes()
            pictures[role][phase]=dict(sha256=hashlib.sha256(data).hexdigest(),size=list(struct.unpack(">II",data[16:24])) if data[:8]==b"\x89PNG\r\n\x1a\n" else [])
    r=compare(reports,pictures,json.loads((root/"memory.json").read_text(encoding="utf-8-sig")))
    if reference:
        for role in ROLES:
            for phase in PHASES:
                identical=(root/f"{role}-{phase}.ground.png").read_bytes()==(Path(reference)/f"{role}-{phase}.ground.png").read_bytes()
                r["checks"].append(dict(name=f"{role}/{phase}: legacy ground pixels identical",passed=identical))
        r["success"]=all(c["passed"] for c in r["checks"])
    return r

if __name__=="__main__":
    root=Path(sys.argv[1]);r=load_and_compare(root,sys.argv[2] if len(sys.argv)>2 else None)
    (root/"comparison.json").write_text(json.dumps(r,indent=2),encoding="utf-8")
    print(json.dumps({"success":r["success"],"checks":len(r["checks"]),"failed":[c["name"] for c in r["checks"] if not c["passed"]],"workers":r["workers"]},indent=2))
    raise SystemExit(0 if r["success"] else 1)
