"""Verify real partial terrain cancellations, lobby/TCP reconnect and joined worker lifetimes.

Usage: python tools/testing/compare_terrain_cancellation_clients.py <attempt-directory>
"""
import hashlib
import json
from pathlib import Path
import statistics
import struct
import sys

ROLES=("owner","witness")


def audit_facts(a):
    events=a.get("events",[])
    by_id={}
    for e in events:
        if e.get("id",0)>0:by_id.setdefault(e["id"],[]).append(e)
    submitted=[e for e in events if e.get("event")=="submitted"]
    cancelled={e["id"] for e in events if e.get("event") in ("cancel_view","cancel_shutdown")}
    failures=[]
    if a.get("limit")!=1024 or a.get("overflow")!=0 or not events:failures.append("audit capacity/overflow")
    if [e.get("order") for e in events]!=list(range(len(events))):failures.append("event order")
    if [e.get("us") for e in events]!=sorted(e.get("us") for e in events):failures.append("event time order")
    if any(e.get("event") in ("failed","join_failed","submit_failed") for e in events):failures.append("worker error")
    if not events or events[-1].get("event")!="joined" or sum(e.get("event")=="joined" for e in events)!=1:failures.append("unjoined worker")
    if [e["id"] for e in submitted]!=list(range(1,len(submitted)+1)):failures.append("job identity sequence")
    if any(e.get("id") in cancelled and e.get("event") in ("cache_accept","uploaded") for e in events):failures.append("cancelled result accepted")
    if any(not 0<=e.get("rows",-1)<=1024 for e in events):failures.append("invalid progress")
    for job in by_id.values():
        starts=[e for e in job if e["event"]=="submitted"]
        finished=[e for e in job if e["event"]=="finished"]
        if len(starts)!=1 or len(finished)!=1 or starts[0]["order"]>=finished[0]["order"]:
            failures.append("missing/duplicate job completion");continue
        running=[e for e in job if e["event"]=="started"]
        if len(running)>1 or (running and not starts[0]["order"]<running[0]["order"]<finished[0]["order"]):
            failures.append("job started out of order")
        if (finished[0].get("produced") or finished[0].get("rows",0)>0) and len(running)!=1:failures.append("unstarted pixel work")
        if any(e["key"]!=starts[0]["key"] for e in job):failures.append("job changed key")
        uploaded=[e for e in job if e["event"]=="uploaded"]
        if len(uploaded)>1:failures.append("duplicate upload")
        for e in uploaded:
            polled=[p for p in job if p["event"]=="polled_value"]
            accepted=[p for p in job if p["event"]=="cache_accept"]
            if not finished[0].get("produced") or finished[0].get("cancelled") or len(polled)!=1 or len(accepted)!=1:
                failures.append("unproven upload")
            elif not finished[0]["order"]<polled[0]["order"]<accepted[0]["order"]<e["order"]:
                failures.append("upload before accepted result")
    partial_view=[];partial_shutdown=[];ready_view=[]
    for id,job in by_id.items():
        partial=any(e["event"]=="finished" and e.get("cancelled") is True and e.get("produced") is False
                    and 0<e.get("rows",0)<1024 for e in job)
        started=any(e["event"]=="started" for e in job)
        if (partial and started and any(e["event"]=="cancel_view" and e.get("running") is True and 0<e["rows"]<1024 for e in job)
            and any(e["event"]=="polled_cancelled" for e in job)):partial_view.append(id)
        if (partial and started and any(e["event"]=="shutdown_begin" and e.get("running") is True and 0<e["rows"]<1024 for e in job)
            and any(e["event"]=="cancel_shutdown" for e in job)):partial_shutdown.append(id)
        if (any(e["event"]=="finished" and e.get("produced") is True for e in job)
            and any(e["event"]=="cancel_view" and e.get("running") is False for e in job)
            and any(e["event"]=="polled_cancelled" for e in job)):ready_view.append(id)
    shutdown=next((e for e in events if e.get("event")=="shutdown_begin"),None)
    duration=events[-1]["us"]-shutdown["us"] if shutdown and events else None
    return {"instance":a.get("instance"),"failures":failures,"partial_view_ids":partial_view,
        "partial_shutdown_ids":partial_shutdown,"ready_view_ids":ready_view,"cancelled_ids":sorted(cancelled),"jobs":len(submitted),
        "events":len(events),"join_us":duration}


def compare(reports,pictures,memory):
    checks=[];facts={}
    def check(name,passed):checks.append({"name":name,"passed":bool(passed)})
    for role in ROLES:
        r=reports[role];other=reports["witness" if role=="owner" else "owner"]
        steps,trace,audits=r.get("steps",[]),r.get("trace",{}),r.get("audits",[])
        fs=[audit_facts(a) for a in audits];facts[role]=fs
        check(role+": native scenario completed",r.get("success") is True and not r.get("failure") and r.get("scenario")=="terrain-cancellation" and r.get("role")==role)
        check(role+": exactly three distinct world workers",len(audits)==3 and len({a.get("instance") for a in audits})==3
            and [a.get("instance") for a in audits]==sorted(a.get("instance") for a in audits))
        for i,f in enumerate(fs):
            check(f"{role}/world-{i}: causal bounded audit and joined worker",not f["failures"])
        check(role+": six actual partial blends cancelled by rotation",len(fs)==3 and len(fs[0]["partial_view_ids"])>=6
            and len(set(r.get("rotations",[])))>=6 and set(r.get("rotations",[]))<=set(fs[0]["partial_view_ids"]))
        requests=[s for s in steps if s.get("stage") in ("LobbyRequested","TcpRequested")]
        cycles=(0,1) if role=="owner" else (2,3)
        check(role+": one lobby and one real TCP close requested",[s.get("cycle") for s in requests]==list(cycles)
            and [s.get("stage") for s in requests]==["LobbyRequested","TcpRequested"])
        check(role+": both teardown requests interrupted real pixel work",len(requests)==2 and len(fs)==3 and all(
            request.get("closing_instance")==fs[i]["instance"] and request.get("closing_job") in fs[i]["partial_shutdown_ids"]
            and request.get("cache",{}).get("active_job",{}).get("running") is True
            and 0<request["cache"]["active_job"].get("rows",0)<1024 for i,request in enumerate(requests)))
        check(role+": every cancelled job stayed out of CPU/GPU acceptance",len(fs)==3 and all("cancelled result accepted" not in f["failures"] for f in fs))
        logins=trace.get("logins",[]);peer_logins=other.get("trace",{}).get("logins",[])
        check(role+": three authentic world logins with new handles",len(logins)==3 and len({l.get("handle") for l in logins})==3
            and all(l.get("name")==r.get("character") and l.get("layer")==0 for l in logins))
        check(role+": one actual transport disconnect",len(trace.get("disconnects",[]))==1 and bool(trace["disconnects"][0]))
        check(role+": actual Lua login/entry/logout counts",all(r.get("events",{}).get(k)==3 for k in ("PLAYER_LOGIN","PLAYER_ENTERING_WORLD","PLAYER_LOGOUT")))
        y=7227 if role=="owner" else 7247;py=7247 if role=="owner" else 7227
        world_steps=[s for s in steps if s.get("player")]
        check(role+": positions and player data unchanged",bool(world_steps) and all(s.get("position")==[8440,y]
            and [s["player"].get(k) for k in ("name","level","hp","mp")]==[r.get("character"),1,100,50]
            and s.get("xp_jp")==[0,0] and s.get("money")==0 and s.get("layer")==0 for s in world_steps))
        check(role+": no gameplay/decoder errors",all(trace.get(k)==[] for k in ("errors","hits","levels","warps","cast_events","cast_results","learn_results")))
        check(role+": peer saw both intermediate departures",len(peer_logins)==3 and all(l["handle"] in trace.get("leaves",[]) for l in peer_logins[:2])
            and len([s for s in steps if s.get("stage")=="ObservedLeave"])==2)
        check(role+": peer saw all three authoritative appearances",len(peer_logins)==3 and all(any(
            e.get("handle")==l["handle"] and e.get("name")==other.get("character") and e.get("kind")=="player"
            and [e.get(k) for k in ("x","y","layer","hp","mp","level")]==[8440,py,0,100,50,1]
            for e in trace.get("enters",[])) for l in peer_logins))
        reentries=[s for s in steps if s.get("stage")=="Reentered"]
        joined=[s for s in steps if s.get("stage")=="WorkerJoined"]
        check(role+": workers joined before world re-entry",len(reentries)==len(joined)==2 and len(logins)==3 and all(
            j["cycle"]==e["cycle"] and j["elapsed_ms"]<=e["elapsed_ms"] and e["own"]==logins[i+1]["handle"]
            for i,(j,e) in enumerate(zip(joined,reentries))))
        samples=r.get("samples",[])
        check(role+": bounded CPU/GPU resources across sessions",0<len(samples)<12000 and all(
            s.get("bake_px")==1024 and s.get("cpu_pixel_limit")==134217728 and 0<=s.get("cpu_pixel_bytes",-1)<=134217728
            and s.get("async_enabled") is True and s.get("worker_error") is None and s.get("last_bakes")==0
            and 0<=s.get("resident",-1)<=s.get("wanted",-1)<=49
            and s.get("texture_payload_bytes")==s.get("resident",-1)*4194304
            and 0<=s.get("last_restores",-1)+s.get("last_completed_uploads",-1)<=4
            and s.get("completed",0)+s.get("cancelled",0)+int(s.get("job_active",False))==s.get("submitted",0) for s in samples))
        p=pictures.get(role,{})
        check(role+": recovered ground pixels exactly match initial view",p.get("size")==[1280,720] and p.get("initial_sha256")==p.get("recovered_sha256") and bool(p.get("initial_sha256")))
    check("Windows CPU/GPU samples for both clients before and after",all(sum(1 for r in memory
        if r.get("phase")==p and r.get("role")==role and r.get("process_id",0)>0 and r.get("Private Bytes",0)>0 and r.get("Total Committed",0)>0)>=3
        for p in ("initial","recovered") for role in ROLES))
    windows={k:{p:sum(statistics.median(r[k] for r in memory if r["phase"]==p and r["role"]==role) for role in ROLES)
                for p in ("initial","recovered")} for k in ("Private Bytes","Working Set","Total Committed")}
    return {"success":all(c["passed"] for c in checks),"checks":checks,"workers":facts,"pictures":pictures,"windows_pair_bytes":windows}


def load_and_compare(root):
    root=Path(root);reports={r:json.loads((root/f"{r}.json").read_text(encoding="utf-8-sig")) for r in ROLES};pictures={}
    for role in ROLES:
        data={p:(root/f"{role}-{p}.ground.png").read_bytes() for p in ("initial","recovered")}
        sizes=[list(struct.unpack(">II",v[16:24])) if v[:16]==b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR" else [] for v in data.values()]
        pictures[role]={"size":sizes[0] if sizes[0]==sizes[1] else [],**{p+"_sha256":hashlib.sha256(v).hexdigest() for p,v in data.items()}}
    memory=json.loads((root/"memory.json").read_text(encoding="utf-8-sig"))
    return compare(reports,pictures,memory)


if __name__=="__main__":
    root=Path(sys.argv[1]);r=load_and_compare(root)
    (root/"comparison.json").write_text(json.dumps(r,indent=2),encoding="utf-8")
    print(json.dumps(r,indent=2));raise SystemExit(0 if r["success"] else 1)
