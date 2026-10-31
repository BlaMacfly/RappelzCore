import copy
import unittest
from compare_terrain_warp_clients import compare, destination, ROLES, PHASES
from test_compare_terrain_cancellation_clients import renumber

def fixture():
    reports={};pictures={};memory=[]
    for index,role in enumerate(ROLES):
        other=ROLES[1-index];own=index+1;peer=2-index
        cache=dict(bake_px=1024,cpu_pixel_limit=134217728,cpu_pixel_bytes=4194304,worker_error=None,
            async_enabled=True,last_bakes=0,resident=1,wanted=1,texture_payload_bytes=4194304,last_restores=0,
            last_completed_uploads=1,submitted=1,completed=1,cancelled=0,job_active=False,warp_count=5,map_switches=3)
        trace={k:[] for k in ('errors','hits','levels','cast_events','cast_results','learn_results','disconnects','enters','leaves','warps')}
        trace['logins']=[dict(handle=own,name='Lua'+role,layer=0)]
        steps=[];events=[]
        def event(name,id,key,rows=0,**extra):
            events.append(dict(event=name,id=id,key=key,rows=rows,running=False,produced=False,cancelled=False)|extra)
        def step(name,i,**extra):
            return dict(stage=name,leg=i,own=own,job=i*2+1,player=dict(name='Lua'+role,level=1,hp=100,mp=50),
                xp_jp=[0,0],money=0,layer=0,position=destination(role,i),cache=copy.deepcopy(cache))|extra
        for i in range(5):
            key=[v//1008 for v in destination(role,i)];job=i*2+1
            req=step('WarpRequested',i);req['cache']['active_job']=dict(id=job,key=key,running=True,rows=60)
            steps.append(req);event('submitted',job,key);event('started',job,key,running=True)
            event('warp',0,key);event('cancel_warp',job,key,80,running=True,cancelled=True)
            event('finished',job,key,81,cancelled=True);event('polled_cancelled',job,key,81,cancelled=True)
            trace['warps'].append(dict(x=destination(role,i)[0],y=destination(role,i)[1],z=0,layer=0))
            app=step('WarpApplied',i,warps=copy.deepcopy(trace['warps']));app['cache'].update(focus_map=[v//16128 for v in destination(role,i)],wanted_keys=[key]);steps.append(app)
            for name in ('submitted','started','finished','polled_value','cache_accept','uploaded'):
                event(name,job+1,key,1024 if name not in ('submitted','started') else 0,produced=name not in ('submitted','started'))
            trace['enters'].append(dict(handle=peer,name='<GM>Lua'+other,kind='player',x=destination(other,i)[0],y=destination(other,i)[1],layer=0,hp=100,mp=50,level=1))
            steps.append(step('PairReunited',i,peer_position=destination(other,i)))
            if role=='witness':
                steps.append(step('ObservedOwner',i,peer_position=destination('owner',0) if i==0 else None))
                if i>0:trace['leaves'].append(peer)
        event('shutdown_begin',0,[0,0]);event('joined',0,[0,0]);renumber(events)
        reports[role]=dict(scenario='terrain-warp',success=True,role=role,character='Lua'+role,steps=steps,trace=trace,
            audits=[dict(instance=1,limit=1024,overflow=0,events=events)],samples=[cache],
            events={k:1 for k in ('PLAYER_LOGIN','PLAYER_ENTERING_WORLD','PLAYER_LOGOUT')})
        pictures[role]={p:dict(size=[1280,720],sha256='a'*64) for p in PHASES}
        memory.extend(dict(role=role,phase=p,process_id=own,**{'Private Bytes':100,'Total Committed':200}) for p in PHASES for _ in range(3))
    return reports,pictures,memory

class TerrainWarpComparisonTests(unittest.TestCase):
    def test_partial_warps_maps_and_peer_refresh_pass(self):
        result=compare(*fixture());self.assertTrue(result['success'],[c for c in result['checks'] if not c['passed']])

    def test_missing_early_cancellation_or_stale_pixels_fail(self):
        for kind in ('missing','completed','zero','stale','ordering'):
            with self.subTest(kind=kind):
                reports,pictures,memory=fixture();ev=reports['owner']['audits'][0]['events']
                cancel=next(e for e in ev if e['event']=='cancel_warp')
                if kind=='missing':cancel['event']='cancel_view'
                elif kind=='completed':cancel['running']=False
                elif kind=='zero':cancel['rows']=0
                elif kind=='stale':
                    e=copy.deepcopy(cancel);e['event']='cache_accept';ev.insert(-1,e);renumber(ev)
                else:cancel['order']=999
                self.assertFalse(compare(reports,pictures,memory)['success'])

    def test_wrong_warp_map_or_missing_peer_state_fail(self):
        for kind in ('packet','map','upload','overlap','peer','login'):
            with self.subTest(kind=kind):
                reports,pictures,memory=fixture();r=reports['witness']
                if kind=='packet':r['trace']['warps'][2]['x']=0
                elif kind=='map':r['steps'][1]['cache']['focus_map']=[9,9]
                elif kind=='upload':
                    for e in r['audits'][0]['events']:
                        if e['id']==6:e['key']=[0,0]
                elif kind=='overlap':r['steps'][1]['cache']['wanted_keys']=[]
                elif kind=='peer':r['trace']['enters'].pop()
                else:r['events']['PLAYER_LOGIN']=2
                self.assertFalse(compare(reports,pictures,memory)['success'])

    def test_changed_ground_budgets_or_unjoined_worker_fail(self):
        for kind in ('pixels','cpu','queue','counter','join','overflow'):
            with self.subTest(kind=kind):
                reports,pictures,memory=fixture();r=reports['owner']
                if kind=='pixels':pictures['owner']['leg-4']['sha256']='b'*64
                elif kind=='cpu':r['samples'][0]['cpu_pixel_bytes']=134217729
                elif kind=='queue':r['samples'][0]['submitted']=2
                elif kind=='counter':memory.pop()
                elif kind=='join':r['audits'][0]['events'][-1]['event']='join_failed'
                else:r['audits'][0]['overflow']=1
                self.assertFalse(compare(reports,pictures,memory)['success'])

if __name__=='__main__':unittest.main()
