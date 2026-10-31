import copy
import unittest
from compare_terrain_network_clients import compare,ROLES
from test_compare_terrain_cancellation_clients import audit

def fixture():
    reports={};pictures={};memory=[];relays={};handles={'owner':[1,3],'witness':[2,4]}
    for index,role in enumerate(ROLES):
        other=ROLES[1-index];y=7227 if role=='owner' else 7247;py=7247 if role=='owner' else 7227
        cache=dict(bake_px=1024,cpu_pixel_limit=134217728,cpu_pixel_bytes=4194304,worker_error=None,async_enabled=True,
            last_bakes=0,resident=1,wanted=1,texture_payload_bytes=4194304,last_restores=0,last_completed_uploads=1,
            submitted=1,completed=1,cancelled=0,job_active=False)
        def step(stage,cycle,**extra):
            me=handles[role][int(cycle>index)];peer=handles[other][int(cycle>1-index)]
            return dict(stage=stage,cycle=cycle,own=me,peer=peer,player=dict(name='Lua'+role,level=1,hp=100,mp=50),position=[8440,y],
                xp_jp=[0,0],money=0,layer=0,cache=copy.deepcopy(cache),loading=False,world_frame=20,audit_instance=int(cycle>index)+1,audit_cursor=6,
                stage_ms=0,elapsed_ms=1,chats=[])|extra
        steps=[]
        for cycle in range(2):
            steps.extend([step('StallBegin',cycle),step('StallHeld',cycle,world_frame=500,stage_ms=8000,audit_cursor=36),step('Resumed',cycle)])
        req=step('ResetRequested',index,job=8,closing_instance=1);req['cache']['active_job']=dict(id=8,running=True,rows=80)
        steps.extend([req,step('WorkerJoined',index,player=None,position=None,cache=None,error='remote reset',elapsed_ms=10),
            step('Reentered',index,own=handles[role][1],audit_instance=2,elapsed_ms=20),step('ObservedLeave',1-index),step('ObservedReentry',1-index)])
        trace={k:[] for k in ('disconnects','errors','hits','levels','warps','cast_events','cast_results','learn_results')}
        trace.update(network_errors=['remote reset'],chats=[dict(message=f'TerrainNet_{i}_'+('in' if i==index else 'out'),sender='Lua'+other,chat_type=3) for i in range(2)],
            logins=[dict(handle=h,name='Lua'+role,layer=0) for h in handles[role]],leaves=[handles[other][0]],
            enters=[dict(handle=h,name='Lua'+other,x=8440,y=py,layer=0,hp=100,mp=50,level=1) for h in handles[other]])
        reports[role]=dict(scenario='terrain-network',role=role,character='Lua'+role,success=True,steps=steps,trace=trace,
            audits=[audit(1,True,True),audit(2)],rotations=[2,3,4],samples=[cache],events={k:2 for k in ('PLAYER_LOGIN','PLAYER_ENTERING_WORLD','PLAYER_LOGOUT')})
        ev=[]
        for name,connection,ns,n,held in [('connected',1,0,0,0),('pause',1,1_000_000_000,100,0),('resume',1,9_000_000_000,100,20),
            ('reset',1,9_100_000_000,120,0),('closed',1,9_200_000_000,120,0),('connected',2,10_000_000_000,0,0),('closed',2,11_000_000_000,50,0)]:
            ev.append(dict(event=name,connection=connection,ns=ns,bytes=dict(up=n,down=n),held=dict(up=held,down=held),abortive=name=='reset'))
        relays[role]=dict(limit=256,overflow=0,block_limit=65536,events=ev)
        pictures[role]=dict(size=[1280,720],initial_sha256='a'*64,recovered_sha256='a'*64)
        memory.extend(dict(role=role,phase=p,process_id=index+1,**{'Private Bytes':100,'Total Committed':100}) for p in ('initial','recovered') for _ in range(3))
    return reports,dict(success=True,relays=relays),pictures,memory

class TerrainNetworkComparisonTests(unittest.TestCase):
    def test_real_stalls_reset_reentry_and_exact_images_pass(self):
        r=compare(*fixture());self.assertTrue(r['success'],[c for c in r['checks'] if not c['passed']]);self.assertEqual(len(r['checks']),41)
    def test_short_or_leaking_stalls_early_probes_or_frozen_rendering_fail(self):
        for kind in ('short','forward','held','probe','frames','cursor'):
            with self.subTest(kind=kind):
                reports,proxy,pictures,memory=fixture();ev=proxy['relays']['owner']['events'];s=reports['owner']['steps'][1]
                if kind=='short':ev[2]['ns']-=1
                elif kind=='forward':ev[2]['bytes']['up']+=1
                elif kind=='held':ev[2]['held']['down']=0
                elif kind=='probe':s['chats']=[reports['owner']['trace']['chats'][0]]
                elif kind=='frames':s['world_frame']=21
                else:s['audit_cursor']=6
                self.assertFalse(compare(reports,proxy,pictures,memory)['success'])
    def test_orderly_eof_missing_partial_work_or_bad_peer_reentry_fail(self):
        for kind in ('eof','partial','join','identity','peer','abortive'):
            with self.subTest(kind=kind):
                reports,proxy,pictures,memory=fixture();r=reports['witness']
                if kind=='eof':r['trace'].update(network_errors=[],disconnects=['closed'])
                elif kind=='partial':next(s for s in r['steps'] if s['stage']=='ResetRequested')['job']=1
                elif kind=='join':r['audits'][0]['events'][-1]['event']='join_failed'
                elif kind=='identity':r['trace']['logins'][1]['handle']=r['trace']['logins'][0]['handle']
                elif kind=='peer':r['trace']['leaves']=[]
                else:proxy['relays']['witness']['events'][3]['abortive']=False
                self.assertFalse(compare(reports,proxy,pictures,memory)['success'])
    def test_stale_pixels_duplicate_probes_overflow_or_budget_errors_fail(self):
        for kind in ('pixels','duplicate','overflow','cpu','queue','memory'):
            with self.subTest(kind=kind):
                reports,proxy,pictures,memory=fixture();r=reports['owner']
                if kind=='pixels':pictures['owner']['recovered_sha256']='b'*64
                elif kind=='duplicate':r['trace']['chats'].append(r['trace']['chats'][0])
                elif kind=='overflow':proxy['relays']['owner']['overflow']=1
                elif kind=='cpu':r['samples'][0]['cpu_pixel_bytes']=134217729
                elif kind=='queue':r['samples'][0]['submitted']=2
                else:memory.pop()
                self.assertFalse(compare(reports,proxy,pictures,memory)['success'])

if __name__=='__main__':unittest.main()
