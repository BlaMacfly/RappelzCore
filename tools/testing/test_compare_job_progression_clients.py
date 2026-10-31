import copy
import unittest
from compare_job_progression_clients import compare

def fixture():
    reports={};sql={name:{} for name in ['probed','level2','level3','saved','final']}
    for role,initial,handle,other in [('owner',11,21,12),('witness',12,22,11)]:
        own=role=='owner';job=100 if own else 200;money=1000 if own else 2000;position=[9440,7227] if own else [9420,7247];jlv=3 if own else 1
        p=lambda level,jp,target:dict(target=target,job=job,jlv=level,jp=jp,cost={1:3,2:5,3:8}[level],status=0 if own and level<3 else 5,base=[job,[10]*7])
        progress=[p(1,8,initial),p(2,5,initial),p(3,0,initial),p(3,0,handle)] if own else [p(1,0,initial),p(1,0,handle)]
        labels=['foreign_owner','unknown_owner','level2','level3','insufficient_jp'] if own else ['foreign_owner','insufficient_jp'];targets=[other,4294967295,initial,initial,initial] if own else [other,initial];kinds=['raw','raw','helper','helper','raw'] if own else ['raw','raw']
        levels=[dict(handle=11,job_level=2,level=10),dict(handle=11,job_level=3,level=10)]
        trace={k:[] for k in ['errors','network_errors','disconnects','hits','cast_events','learn_results','moves','warps']}
        trace.update(logins=[dict(handle=h,hp=280,max_hp=280,mp=140,max_mp=140,x=position[0],y=position[1],order=o) for h,o in [(initial,1),(handle,100)]],job_results=[6,6,0,0,6] if own else [6,6],job_progress=progress,experience=[dict(xp=0,jp=j) for j in ([8,5,0,0] if own else [0,0])],gold=[dict(gold=money)]*2,levels=levels,level_orders=[20,40],skill_catalogs=[dict(target=v['target'],job_level=v['jlv'],jp=v['jp']) for v in progress])
        steps=[dict(stage='Saved',xp_jp=[0,0],money=money,position=position,player=dict(level=10),learned={},job=progress[-1]),dict(stage='Logout',elapsed_ms=12000)]
        messages=[f'[RappelzDebug] EVENT RAPPELZ_JOB_LEVEL_CHANGED {level} HP=280/280 jp={jp} jlv={level}' for level,jp in ([(1,8),(2,5),(3,0),(3,0)] if own else [(1,0),(1,0)])]
        reports[role]=dict(success=True,scenario='job-progression',initial=initial,own=handle,initial_other=other,other=22 if own else 21,profile_counts=[3923,300,200],wide_exp300='272233208401492000',trace=trace,steps=steps,requests=[dict(label=a,target=b,kind=c) for a,b,c in zip(labels,targets,kinds)],events=dict(PLAYER_LOGIN=2,PLAYER_ENTERING_WORLD=2,PLAYER_LOGOUT=2,RAPPELZ_JOB_LEVEL_CHANGED=4 if own else 2,RAPPELZ_JP_CHANGED=4 if own else 2),messages=messages)
        for name,level,jp,online in [('probed',1,8 if own else 0,1),('level2',2 if own else 1,5 if own else 0,1),('level3',jlv,0,1),('saved',jlv,0,0),('final',jlv,0,0)]:
            sql[name][role]=dict(level=10,xp=0,money=money,health=280,mp=140,online=online,position=position,settings=dict(job=str(job),jlv=str(level),jp=str(jp),unmapped='preserve-me'))
    control=dict(success=True,scenario='job-progression',world_pid=123,exit_code=0,checkpoints=[dict(name=s) for s in ['probed','level2','level3','saved']])
    profile=dict(success=True,idempotent=True,conflict_preserved=True,tables=[dict(file=f,rows=n,fields=c,lossless=True,server_client_equal=True) for f,n,c in [('BasicStat.dbc',3923,8),('Exp.dbc',300,11),('SummonExp.dbc',200,7)]])
    return reports,control,[dict(role=r,exit_code=0) for r in reports],sql,profile

class JobProgressionAcceptanceTests(unittest.TestCase):
    def reject(self,mutate):
        v=copy.deepcopy(fixture());mutate(v);self.assertFalse(compare(*v)['success'])
    def test_independent_complete_fixture_and_empty_input(self):
        r=compare(*fixture());self.assertTrue(r['success'],[c['name'] for c in r['checks'] if not c['passed']]);self.assertEqual(len(r['checks']),36);self.assertFalse(compare({}, {}, [], {}, {})['success'])
    def test_rejects_forged_success_wrong_request_target_and_overdraw(self):
        self.reject(lambda v:v[0]['owner']['trace']['job_results'].__setitem__(0,0))
        self.reject(lambda v:v[0]['owner']['requests'][2].update(target=12))
        self.reject(lambda v:v[0]['owner']['trace']['job_progress'][2].update(jp=-1))
    def test_rejects_private_leak_cost_column_error_and_inexact_wide_value(self):
        self.reject(lambda v:v[0]['witness']['trace']['job_progress'][0].update(target=11))
        self.reject(lambda v:v[0]['owner']['trace']['job_progress'][1].update(cost=30))
        self.reject(lambda v:v[0]['owner'].update(wide_exp300='272233208401491968'))
        self.reject(lambda v:v[4]['tables'][1].update(lossless=False))
    def test_rejects_premature_or_duplicate_lua_and_reentry_replay(self):
        self.reject(lambda v:v[0]['owner']['events'].update(RAPPELZ_JOB_LEVEL_CHANGED=5))
        self.reject(lambda v:v[0]['owner']['messages'].__setitem__(1,'EVENT RAPPELZ_JOB_LEVEL_CHANGED 2 jp=8 jlv=1'))
        self.reject(lambda v:v[0]['witness']['trace']['level_orders'].__setitem__(1,101))
    def test_rejects_partial_sql_unknown_key_loss_or_mixed_saved_profiles(self):
        self.reject(lambda v:v[3]['level2']['owner']['settings'].update(jp='8'))
        self.reject(lambda v:v[3]['final']['owner']['settings'].pop('unmapped'))
        self.reject(lambda v:v[3]['final']['witness'].update(health=281))
        self.reject(lambda v:v[2][0].update(exit_code=1))
if __name__=='__main__':unittest.main()
