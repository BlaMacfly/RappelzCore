import copy
import unittest
from compare_gameplay_restart_clients import compare,ROLES,checkpoint,final_position
from test_compare_terrain_cancellation_clients import audit

def fixture():
    reports={};pictures={};memory=[];cp={};saved={};activity={};monster=536870912
    first=[dict(attacker=1,target=monster,damage=12,target_hp=93),dict(attacker=2,target=monster,damage=12,target_hp=81),dict(attacker=monster,target=1,damage=18,target_hp=82)]
    post=[];hp=105
    for attacker in [1,2,1,2,1,2,1]:hp-=15;post.append(dict(attacker=attacker,target=monster,damage=15,target_hp=hp))
    post.insert(3,dict(attacker=monster,target=1,damage=20,target_hp=80))
    for i,role in enumerate(ROLES):
        other=ROLES[1-i];own=i+1;peer=2-i
        cp[role]=dict(level=1,xp=0,jp=0,money=0,health=100,mp=50,position=checkpoint(role),online=1)
        saved[role]=dict(level=1,xp=5 if i==0 else 0,jp=3 if i==0 else 0,money=0,health=80 if i==0 else 100,mp=50,position=final_position(role),online=0)
        activity[role]=dict(role=role,own=own,peer=peer,monster=monster,unix_ms=1100,moving=True,peer_moving=True,distance_from_checkpoint=10,checkpoint=checkpoint(role),
            route_acks=2,own_hits=1,peer_hits=1,retaliations=1,monster_hp=81,monster_max_hp=105,own_hp=82 if i==0 else 100)
        cache=dict(bake_px=1024,cpu_pixel_limit=134217728,cpu_pixel_bytes=4194304,worker_error=None,async_enabled=True,last_bakes=0,resident=1,wanted=1,
            texture_payload_bytes=4194304,last_restores=0,last_completed_uploads=1,submitted=1,completed=1,cancelled=0,job_active=False)
        def step(name,**extra):
            return dict(stage=name,own=own,monster=monster,player=dict(name='Lua'+role,level=1,hp=100,mp=50),position=checkpoint(role),xp_jp=[0,0],money=0,layer=0,
                target=None,motion=dict(moving=False),audit_instance=1,hits=[],moves=[],elapsed_ms=10,order=10)|extra
        steps=[step('Joined',position=[8440,7227 if i==0 else 7247]),step('CheckpointArrived',elapsed_ms=5),step('CheckpointSaved'),step('ActionsStarted',order=3),
            step('WorldDropped',player=None),step('DowntimeHeld',player=None,stage_ms=5100),step('CheckpointRestored',audit_instance=2),
            step('NoReplay',audit_instance=2,stage_ms=3200,monster_hp=105),step('PostActionsStarted',boundary=24),
            step('DurableReentry',xp_jp=[5,3] if i==0 else [0,0],position=final_position(role),audit_instance=3)]
        if role=='witness':steps.append(step('AssistStopped',own=2,monster_hp=15,hits=copy.deepcopy(post[:-1])))
        logins=[dict(handle=h,name='Lua'+role,layer=0,order=o,x=p[0],y=p[1],hp=health,mp=50) for h,o,p,health in
            [(own,1,[8440,7227 if i==0 else 7247],100),(own,20,checkpoint(role),100),(own+2,60,final_position(role),saved[role]['health'])]]
        xp=[dict(handle=own,xp=0,jp=0),dict(handle=own,xp=0,jp=0)];xp_orders=[2,21]
        if i==0:xp.append(dict(handle=own,xp=5,jp=3));xp_orders.append(45)
        xp.append(dict(handle=own+2,xp=5 if i==0 else 0,jp=3 if i==0 else 0));xp_orders.append(61)
        trace={k:[] for k in ['errors','warps','levels','denied_attacks','cast_events','cast_results','learn_results','disconnects','hpmp','vitals_orders']}
        trace.update(logins=logins,hits=copy.deepcopy(first+post),hit_orders=[5,6,7]+list(range(26,26+len(post))),experience=xp,experience_orders=xp_orders,network_errors=['server stopped'],
            moves=[dict(handle=own,points=[checkpoint(role)],order=2),dict(handle=own,points=[checkpoint(role)]*16,order=4),dict(handle=2,points=[final_position('witness')],order=25)])
        reports[role]=dict(role=role,character='Lua'+role,scenario='gameplay-restart',success=True,steps=steps,samples=[cache],audits=[audit(1),audit(2),audit(3)],trace=trace,
            events={k:3 for k in ['PLAYER_LOGIN','PLAYER_ENTERING_WORLD','PLAYER_LOGOUT']})
        pictures[role]=dict(size=[1280,720],initial_sha256='a'*64,recovered_sha256='a'*64)
        memory.extend(dict(role=role,phase=p,process_id=own,**{'Private Bytes':100,'Total Committed':100}) for p in ['initial','recovered'] for _ in range(3))
    server=dict(scenario='gameplay',success=True,boots=[dict(pid=10,unix_ms=1000),dict(pid=11,unix_ms=6500)],kill=dict(pid=10,unix_ms=1105,activity=activity,exit_code=1,graceful=False,method='owned Popen.kill'),final_exit_code=0)
    db=dict(checkpoint=cp,down=copy.deepcopy(cp),saved=saved,request=dict(pid=10,unix_ms=1070,method='console saveall'),restarted={r:dict(account=0,character=0) for r in ROLES}|dict(same_realm=0,other_realm=2))
    return reports,server,db,pictures,memory

class GameplayRestartComparisonTests(unittest.TestCase):
    def test_checkpoint_live_actions_no_replay_and_durable_reward_pass(self):
        r=compare(*fixture());self.assertTrue(r['success'],[c for c in r['checks'] if not c['passed']]);self.assertEqual(len(r['checks']),45)
    def test_stationary_stale_dead_or_unacknowledged_candidates_fail(self):
        for key,value in [('moving',False),('peer_hits',0),('unix_ms',1000),('monster_hp',0),('route_acks',0)]:
            with self.subTest(key=key):
                reports,server,db,p,m=fixture();server['kill']['activity']['owner'][key]=value;self.assertFalse(compare(reports,server,db,p,m)['success'])
    def test_old_action_replay_wrong_checkpoint_or_broken_damage_chain_fail(self):
        for kind in ['saved','replay','raw_hp','chain','loop','epoch']:
            with self.subTest(kind=kind):
                reports,s,db,p,m=fixture();r=reports['owner']
                if kind=='saved':db['down']['owner']['xp']=5
                elif kind=='replay':next(v for v in r['steps'] if v['stage']=='NoReplay')['hits']=[{}]
                elif kind=='raw_hp':r['trace']['logins'][2]['hp']=100
                elif kind=='chain':r['trace']['hits'][3]['target_hp']=99
                elif kind=='loop':r['trace']['moves'][1]['points']=[]
                else:r['trace']['hit_orders'][0]=30
                self.assertFalse(compare(reports,s,db,p,m)['success'])
    def test_duplicate_rewards_private_leaks_images_and_resource_failures_fail(self):
        for kind in ['duplicate','private','pixels','cpu','join','counter']:
            with self.subTest(kind=kind):
                reports,s,db,p,m=fixture();r=reports['owner']
                if kind=='duplicate':r['trace']['experience'].insert(3,dict(handle=1,xp=5,jp=3));r['trace']['experience_orders'].insert(3,46)
                elif kind=='private':db['saved']['witness']['xp']=5
                elif kind=='pixels':p['owner']['recovered_sha256']='b'*64
                elif kind=='cpu':r['samples'][0]['cpu_pixel_bytes']=134217729
                elif kind=='join':r['audits'][0]['events'][-1]['event']='join_failed'
                else:m.pop()
                self.assertFalse(compare(reports,s,db,p,m)['success'])

if __name__=='__main__':unittest.main()
