import copy
import unittest
from compare_terrain_restart_clients import compare,ROLES
from test_compare_terrain_cancellation_clients import audit

def fixture():
    reports={};pictures={};memory=[];jobs={}
    live={r:dict(account=1,character=1) for r in ROLES}|dict(same_realm=0,other_realm=2,population=.375)
    restarted={r:dict(account=0,character=0) for r in ROLES}|dict(same_realm=0,other_realm=2,population=0)
    for index,role in enumerate(ROLES):
        other=ROLES[1-index];own=index+1;peer=2-index;y=7227 if index==0 else 7247;py=7247 if index==0 else 7227
        cache=dict(bake_px=1024,cpu_pixel_limit=134217728,cpu_pixel_bytes=4194304,worker_error=None,async_enabled=True,
            last_bakes=0,resident=1,wanted=1,texture_payload_bytes=4194304,last_restores=0,last_completed_uploads=1,submitted=1,completed=1,cancelled=0,job_active=False)
        def step(stage,**extra):
            return dict(stage=stage,own=own,peer=peer,job=2,closing_instance=1,audit_instance=1,player=dict(name='Lua'+role,level=1,hp=100,mp=50),
                position=[8440,y],xp_jp=[0,0],money=0,layer=0,cache=copy.deepcopy(cache),elapsed_ms=0,events={})|extra
        steps=[step('Joined'),step('ServerKilled'),step('WorkerJoined',player=None,peer=None,elapsed_ms=100),
            step('DowntimeHeld',player=None,peer=None,stage_ms=5100,events=dict(PLAYER_LOGOUT=1)),
            step('Reentered',audit_instance=2,elapsed_ms=5500),step('PairReunited',audit_instance=2)]
        trace={k:[] for k in ('errors','hits','levels','warps','cast_events','cast_results','learn_results','disconnects')}
        trace.update(network_errors=['server terminated'],logins=[dict(handle=own,name='Lua'+role,layer=0,order=o) for o in (1,20)],
            enters=[dict(handle=peer,name='Lua'+other,kind='player',x=8440,y=py,layer=0,hp=100,mp=50,level=1,order=o) for o in (2,21)])
        reports[role]=dict(scenario='terrain-restart',role=role,character='Lua'+role,success=True,steps=steps,samples=[cache],audits=[audit(1,False,True),audit(2)],trace=trace,
            events={k:2 for k in ('PLAYER_LOGIN','PLAYER_ENTERING_WORLD','PLAYER_LOGOUT')})
        jobs[role]=dict(role=role,running=True,rows=50,id=2,instance=1,unix_ms=1100)
        pictures[role]=dict(size=[1280,720],initial_sha256='a'*64,recovered_sha256='a'*64)
        memory.extend(dict(role=role,phase=p,process_id=own,**{'Private Bytes':100,'Total Committed':100}) for p in ('initial','recovered') for _ in range(3))
    server=dict(success=True,boots=[dict(pid=10,generation=0,port=24514,unix_ms=1000),dict(pid=11,generation=1,port=24514,unix_ms=6500)],
        kill=dict(pid=10,unix_ms=1105,jobs=jobs,method='owned Popen.kill',graceful=False,exit_code=1),final_exit_code=0)
    database=dict(duplicate=dict(exit_code=1,before=copy.deepcopy(live),after=copy.deepcopy(live)),down=copy.deepcopy(live),restarted=restarted)
    return reports,server,database,pictures,memory

class TerrainRestartComparisonTests(unittest.TestCase):
    def test_process_kill_recovery_and_reused_handles_pass(self):
        r=compare(*fixture());self.assertTrue(r['success'],[c for c in r['checks'] if not c['passed']]);self.assertEqual(len(r['checks']),37)
    def test_fake_kill_stale_jobs_or_uncleared_and_foreign_flags_fail(self):
        for kind in ('graceful','pid','downtime','job','current','foreign','duplicate'):
            with self.subTest(kind=kind):
                reports,server,db,images,memory=fixture()
                if kind=='graceful':server['kill']['exit_code']=0
                elif kind=='pid':server['kill']['pid']=999
                elif kind=='downtime':server['boots'][1]['unix_ms']=1200
                elif kind=='job':server['kill']['jobs']['owner']['unix_ms']=1000
                elif kind=='current':db['restarted']['owner']['account']=1
                elif kind=='foreign':db['restarted']['other_realm']=0
                else:db['duplicate']['after']['owner']['character']=0
                self.assertFalse(compare(reports,server,db,images,memory)['success'])
    def test_completed_work_unjoined_world_or_old_session_peer_fail(self):
        for kind in ('partial','join','worker','enter','logout','phantom'):
            with self.subTest(kind=kind):
                reports,server,db,images,memory=fixture();r=reports['owner']
                if kind=='partial':server['kill']['jobs']['owner']['id']=1
                elif kind=='join':r['audits'][0]['events'][-1]['event']='join_failed'
                elif kind=='worker':r['steps'][4]['audit_instance']=1
                elif kind=='enter':r['trace']['enters'][1]['order']=3
                elif kind=='logout':r['events']['PLAYER_LOGOUT']=1
                else:r['steps'][3]['peer']=2
                self.assertFalse(compare(reports,server,db,images,memory)['success'])
    def test_changed_images_budget_accounting_and_missing_counters_fail(self):
        for kind in ('image','cpu','queue','counters','reuse'):
            with self.subTest(kind=kind):
                reports,server,db,images,memory=fixture();r=reports['witness']
                if kind=='image':images['witness']['recovered_sha256']='b'*64
                elif kind=='cpu':r['samples'][0]['cpu_pixel_bytes']=134217729
                elif kind=='queue':r['samples'][0]['submitted']=2
                elif kind=='reuse':r['trace']['logins'][1]['handle']=99
                else:memory.pop()
                self.assertFalse(compare(reports,server,db,images,memory)['success'])

if __name__=='__main__':unittest.main()
