import copy
import unittest
from compare_late_cast_clients import compare


def fixture():
    caster,viewer,body=90,91,0x20000001
    reports={};profiles={}
    for role,handle in [('owner',caster),('witness',viewer)]:
        own=role=='owner';money=1016 if own else 2000;mp=140 if own else 76;hp=263 if own else 280;pos=[9440 if own else 9950,7227]
        event=lambda phase,seq:dict(sequence=seq,skill=3212,phase=phase,caster=caster,amount=320 if phase=='FIRE' else 0)
        base=dict(player=dict(hp=hp,mp=mp),position=pos,money=money,xp_jp=[10,6] if own else [0,0],remaining=50 if own else 0,body=None,peer=dict(mp=140))
        def s(name,ms,**values):return dict(base,stage=name,elapsed_ms=ms)|values
        steps=[s('Initial',100,money=1000 if own else 2000,xp_jp=[0,0]),s('Canceled',7000,remaining=0,skill_event=event('CANCEL',2)),s('Consumed',26000,skill_event=event('FIRE',4)),s('Quiet',29000,skill_event=event('FIRE',4)),s('Saved',29100),s('Logout',29200)]
        if own:steps.extend([s('Kill1',6500,money=1008),s('Kill2',25000)])
        else:steps.extend([s('Hidden1',3000,peer=None,body=None,skill_event=None),s('Hidden2',8000,peer=None,body=None,skill_event=event('CANCEL',2)),s('HiddenAfter',27500,peer=None,body=None,skill_event=event('FIRE',4))])
        phases=[]
        for i,name in enumerate(['CASTING','CANCEL','CASTING','FIRE']):
            phases.append(dict(id=3212,level=1,caster=caster,target=body,layer=0,phase=name,order=(i+1)*100+(0 if own else 5),duration=(145 if own else 70) if name=='CASTING' else 0,amount=320 if name=='FIRE' else 0,mp_cost=5 if name=='FIRE' else 0,corpse_fire=name=='FIRE',caster_hp=266 if i<2 else 263,caster_mp=140 if i==3 else (20 if i<2 else 41)))
        trace={k:[] for k in ['errors','network_errors','disconnects','warps','denied_attacks','learn_results','levels','moves','hits','enters']}
        trace.update(logins=[dict(handle=handle)],cast_events=phases,gold=[dict(gold=v) for v in ([1000,1008,1016] if own else [2000])],experience=[dict(xp=x,jp=j) for x,j in ([(0,0),(5,3),(10,6)] if own else [(0,0)])])
        lists=[dict(target=handle,levels=[dict(id=3212,remain=0)])]
        if own:
            lists.append(dict(target=handle,levels=[dict(id=3212,remain=5800)]));trace['hits']=[dict(attacker=caster,target=body,target_hp=0)]*2
        else:
            trace['enters']=[dict(handle=h,hp=hp,order=order) for h,hp,order in [(caster,266,20),(body,0,21),(body,0,120),(body,0,220),(caster,263,290),(body,0,291)]]
            trace['moves']=[dict(handle=handle,points=[p]) for p in [[10040,7227],[9950,7227]]*3]
        trace['skill_lists']=lists
        reports[role]=dict(success=True,scenario='late-cast',role=role,own=handle,other=viewer if own else caster,body=body,near=[9950,7227],far=[10040,7227],steps=steps,events=dict(PLAYER_LOGIN=1,PLAYER_ENTERING_WORLD=1,PLAYER_LOGOUT=1,RAPPELZ_SKILL_EVENT=4),trace=trace)
        profiles[role]=dict(level=10,xp=10 if own else 0,money=money,health=hp,mp=mp,position=pos,online=0,settings=dict(job='302',jlv='35',layer='0',jp='6' if own else '0',learned_skill_3212='1',learned_skill_63203='1'))
    return reports,dict(success=True,scenario='late-cast',world_pid=123,exit_code=0),[dict(role=r,exit_code=0) for r in reports],dict(success=True,checks=[dict(offline=True,gm=0,progress=True,settings=True,position=True,vitals=True)]*2,profiles=profiles)


class LateCastAcceptanceTests(unittest.TestCase):
    def test_independent_synthetic_pair_passes(self):
        result=compare(*fixture());self.assertTrue(result['success'],[c['name'] for c in result['checks'] if not c['passed']]);self.assertEqual(len(result['checks']),27)
    def reject(self,mutate):
        values=copy.deepcopy(fixture());mutate(values);self.assertFalse(compare(*values)['success'])
    def test_rejects_full_or_zero_late_duration_and_replayed_or_missing_phase(self):
        for duration in [0,145,146]:self.reject(lambda v:v[0]['witness']['trace']['cast_events'][0].update(duration=duration))
        self.reject(lambda v:v[0]['witness']['trace']['cast_events'].append(v[0]['witness']['trace']['cast_events'][-1]))
        self.reject(lambda v:v[0]['witness']['trace']['cast_events'].pop(1))
    def test_rejects_hidden_target_unaccepted_lua_and_foreign_private_state(self):
        self.reject(lambda v:v[0]['witness']['trace'].update(enters=[]))
        self.reject(lambda v:v[0]['witness']['events'].update(RAPPELZ_SKILL_EVENT=3))
        self.reject(lambda v:v[0]['witness']['trace']['skill_lists'][0].update(target=90))
        self.reject(lambda v:next(s for s in v[0]['witness']['steps'] if s['stage']=='Hidden2').update(skill_event=dict(sequence=3)))
    def test_rejects_false_restore_duplicate_reward_and_wrong_sql_or_process(self):
        self.reject(lambda v:v[0]['owner']['trace']['cast_events'][3].update(caster_mp=139))
        self.reject(lambda v:v[0]['owner']['trace']['experience'].append(dict(xp=15,jp=9)))
        self.reject(lambda v:v[0]['owner']['trace']['gold'][1].update(gold=1015))
        self.reject(lambda v:v[3]['profiles']['owner'].update(money=1000))
        self.reject(lambda v:v[3]['checks'][0].update(gm=2))
        self.reject(lambda v:v[2][1].update(exit_code=1))
    def test_killing_blow_frame_can_precede_a_valid_private_currency_notification(self):
        values=fixture();next(s for s in values[0]['owner']['steps'] if s['stage']=='Kill1')['money']=1000
        self.assertTrue(compare(*values)['success'])

if __name__=='__main__':unittest.main()
