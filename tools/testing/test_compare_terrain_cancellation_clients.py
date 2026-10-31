import copy
import unittest
from compare_terrain_cancellation_clients import compare


def audit(instance,rotations=False,interrupted=False):
    events=[]
    def event(name,id,rows=0,running=False,produced=False,cancelled=False):
        events.append(dict(event=name,id=id,key=[id,id],rows=rows,running=running,produced=produced,cancelled=cancelled))
    for name in ("submitted","started","finished","polled_value","cache_accept","uploaded"):
        event(name,1,1024 if name not in ("submitted","started") else 0,name=="started",name in ("finished","polled_value","cache_accept","uploaded"))
    next_id=2
    if rotations:
        for id in range(2,8):
            event("submitted",id);event("started",id,running=True);event("cancel_view",id,100,True,False,True)
            event("finished",id,101,False,False,True);event("polled_cancelled",id,101,False,False,True)
        next_id=8
    if interrupted:
        event("submitted",next_id);event("started",next_id,running=True);event("shutdown_begin",next_id,80,True)
        event("cancel_shutdown",next_id,80,True,False,True);event("finished",next_id,81,False,False,True)
        event("joined",next_id,80)
    else:event("shutdown_begin",0);event("joined",0)
    renumber(events)
    return dict(instance=instance,limit=1024,overflow=0,events=events)


def renumber(events):
    for i,e in enumerate(events):e.update(order=i,us=(i+1)*10)


def fixture():
    reports={};pictures={};memory=[]
    handles={"owner":[1,3,4],"witness":[2,5,6]}
    names={"owner":"LuaOwner","witness":"LuaWitness"}
    for role in handles:
        other="witness" if role=="owner" else "owner";y,py=(7227,7247) if role=="owner" else (7247,7227)
        caches=dict(bake_px=1024,cpu_pixel_limit=134217728,cpu_pixel_bytes=4194304,async_enabled=True,
            worker_error=None,last_bakes=0,resident=15,wanted=15,texture_payload_bytes=15*4194304,
            last_restores=0,last_completed_uploads=0,submitted=1,completed=1,cancelled=0,job_active=False)
        def step(name,cycle,handle,elapsed):
            return dict(stage=name,cycle=cycle,elapsed_ms=elapsed,own=handle,player=dict(name=names[role],handle=handle,level=1,hp=100,mp=50),
                position=[8440,y],xp_jp=[0,0],money=0,layer=0,cache=copy.deepcopy(caches))
        steps=[step("InitialJoin",0,handles[role][0],0)]
        cycles=(0,1) if role=="owner" else (2,3)
        for i,cycle in enumerate(cycles):
            req=step("LobbyRequested" if i==0 else "TcpRequested",cycle,handles[role][i],10+i*10)
            req.update(closing_instance=i+1,closing_job=8 if i==0 else 2)
            req["cache"]["active_job"]=dict(running=True,rows=40)
            steps.extend([req,dict(stage="WorkerJoined",cycle=cycle,elapsed_ms=11+i*10),
                step("Reentered",cycle,handles[role][i+1],12+i*10)])
        steps.extend([dict(stage="ObservedLeave"),dict(stage="ObservedLeave")])
        trace={k:[] for k in ("errors","hits","levels","warps","cast_events","cast_results","learn_results")}
        trace.update(disconnects=["closed"],logins=[dict(handle=h,name=names[role],layer=0) for h in handles[role]],
            enters=[dict(handle=h,name=names[other],kind="player",x=8440,y=py,layer=0,hp=100,mp=50,level=1) for h in handles[other]],
            leaves=handles[other][:2])
        reports[role]=dict(scenario="terrain-cancellation",success=True,role=role,character=names[role],
            steps=steps,trace=trace,audits=[audit(1,True,True),audit(2,False,True),audit(3)],rotations=list(range(2,8)),
            events={k:3 for k in ("PLAYER_LOGIN","PLAYER_ENTERING_WORLD","PLAYER_LOGOUT")},samples=[copy.deepcopy(caches)])
        pictures[role]=dict(size=[1280,720],initial_sha256="a"*64,recovered_sha256="a"*64)
        memory.extend(dict(role=role,phase=p,process_id=1,**{"Private Bytes":100,"Working Set":80,"Total Committed":200})
            for p in ("initial","recovered") for _ in range(3))
    return reports,pictures,memory


class TerrainCancellationComparisonTests(unittest.TestCase):
    def test_actual_partial_cancellation_transport_reentry_and_joined_worlds_pass(self):
        r=compare(*fixture());self.assertTrue(r["success"]);self.assertEqual(len(r["checks"]),39)

    def test_stale_acceptance_worker_errors_or_reused_world_are_detected(self):
        for kind in ("stale","join","overflow","world"):
            with self.subTest(kind=kind):
                reports,images,memory=fixture();a=reports["owner"]["audits"][0]
                if kind=="stale":
                    e=next(copy.deepcopy(e) for e in a["events"] if e["event"]=="finished" and e["id"]==2)
                    e.update(event="cache_accept",produced=True,cancelled=False);a["events"].insert(-1,e);renumber(a["events"])
                elif kind=="join":a["events"][-1]["event"]="join_failed"
                elif kind=="overflow":a["overflow"]=1
                else:reports["owner"]["audits"][1]["instance"]=1
                self.assertFalse(compare(reports,images,memory)["success"])

    def test_completed_or_unstarted_work_cannot_count_as_partial_cancellation(self):
        for kind in ("rotation","shutdown","request"):
            with self.subTest(kind=kind):
                reports,images,memory=fixture();a=reports["witness"]["audits"][0]
                if kind=="rotation":
                    for e in a["events"]:
                        if e["event"]=="cancel_view":e["running"]=False
                elif kind=="shutdown":
                    for e in a["events"]:
                        if e["event"]=="shutdown_begin":e["running"]=False
                else:reports["witness"]["steps"][1]["closing_job"]=1
                self.assertFalse(compare(reports,images,memory)["success"])

    def test_missing_transport_peer_cleanup_or_changed_images_and_budgets_fail(self):
        for kind in ("tcp","peer","pixels","cpu","queue","counter"):
            with self.subTest(kind=kind):
                reports,images,memory=fixture();r=reports["owner"]
                if kind=="tcp":r["trace"]["disconnects"]=[]
                elif kind=="peer":r["trace"]["leaves"].pop()
                elif kind=="pixels":images["owner"]["recovered_sha256"]="b"*64
                elif kind=="cpu":r["samples"][0]["cpu_pixel_bytes"]=134217729
                elif kind=="queue":r["samples"][0]["submitted"]=2
                else:memory.pop()
                self.assertFalse(compare(reports,images,memory)["success"])


if __name__=="__main__":unittest.main()
