"""Acceptance regressions for real currency and level-up crash evidence."""
import copy
import unittest
from test_reward_commit_probe import fixture as plain_fixture
from compare_gameplay_restart_clients import compare,ROLES,step
from reward_commit_probe import withheld,awarded


def fixture():
    reports,s,db,p,m=plain_fixture();s['money_level']=True;q=db['reward'];q['money_level']=True
    before=copy.deepcopy(db['checkpoint'])
    before['owner'].update(level=2,xp=799,jp=7,money=1000,health=120,mp=60)
    before['owner']['settings']['jp']='7';before['witness']['money']=2000
    for v in before.values():v['position'][0]+=1000
    committed=copy.deepcopy(before);committed['owner'].update(level=3,xp=4,jp=10,money=1008,health=140,mp=70);committed['owner']['settings']['jp']='10'
    db['checkpoint']=before;db['down']=copy.deepcopy(committed);q.update(before=copy.deepcopy(before),committed=committed,after=copy.deepcopy(committed))
    q['window']['profiles']=copy.deepcopy(committed)
    for o in q['blocked']:
        o['committed']=copy.deepcopy(before);o['dirty']=copy.deepcopy(committed);o['dirty']['owner']['jp']=7;o['dirty']['owner']['settings']['jp']='7'
        o['client'].update(xp_jp=[799,7],level=2,money=1000)
    db['saved']=copy.deepcopy(committed)
    for role,v in db['saved'].items():v['online']=0
    db['saved']['witness']['position'][0]+=16
    db['monster_fixture']=dict(monster_id=651,hp=105,exp=5,jp=3,gold_min=8,gold_max=14,guid=1499,preexisting_max=1498,dbc_sha256='a'*64)
    early={'Joined','CheckpointArrived','CheckpointSaved','RewardWithheld','ActionsStarted'}
    for role in ROLES:
        r=reports[role];r['money_level']=True;t=r['trace'];owner=role=='owner'
        for st in r['steps']:
            if 'position' in st:st['position'][0]+=1000
            if st.get('player'):
                old=st['stage'] in early
                st['player'].update(level=2 if owner and old else 3 if owner else 1,hp=120 if owner and old else 140 if owner else 100,mp=60 if owner and old else 70 if owner else 50)
                st.update(money=1000 if owner and old else 1008 if owner else 2000,xp_jp=[799,7] if owner and old else [4,10] if owner else [0,0])
            if st['stage'] in ('CheckpointRestored','DurableReentry'):
                st['peer_unit']=dict(level=1 if owner else 3,max_hp=100 if owner else 140,max_mp=50 if owner else 70)
            if st['stage']=='RewardWithheld':st.update(xp_jp=[799,7],money=1000)
            if st['stage']=='RewardRetained':st.update(money=1008 if owner else 2000,xp_jp=[4,10] if owner else [0,0])
        for i,l in enumerate(t['logins']):
            l['x']+=1000;l.update(hp=120 if owner and i==0 else 140 if owner else 100,mp=60 if owner and i==0 else 70 if owner else 50,max_hp=120 if owner and i==0 else 140 if owner else 100,max_mp=60 if owner and i==0 else 70 if owner else 50)
        for mv in t['moves']:
            for pos in mv['points']:pos[0]+=1000
        for i,x in enumerate(t['experience']):
            if owner:x.update(xp=799 if i==0 else 4,jp=7 if i==0 else 10)
        t['gold']=[dict(gold=g,chaos=0) for g in ([1000,1008,1008,1008] if owner else [2000,2000,2000])]
        t['gold_orders']=[2,15,22,62] if owner else [2,22,62]
        t['levels']=[dict(handle=1,level=3,job_level=1)];t['level_orders']=[16]
        r['events']['PLAYER_MONEY']=4 if owner else 3
        if owner:r['events']['PLAYER_LEVEL_UP']=1
    return reports,s,db,p,m


class MoneyLevelCrashTests(unittest.TestCase):
    def test_positive_gold_rollover_and_peer_level_survive_three_entries(self):
        v=compare(*fixture());self.assertTrue(v['success'],[c for c in v['checks'] if not c['passed']]);self.assertEqual(len(v['checks']),57)
    def test_lost_or_duplicated_money_exp_and_level_never_pass(self):
        for kind in ['lost_money','rollover','double_gold','private_leak','level_loss','raw_mp']:
            with self.subTest(kind=kind):
                reports,s,db,p,m=fixture();r=reports['owner']
                if kind=='lost_money':db['down']['owner']['money']=1000
                elif kind=='rollover':db['reward']['committed']['owner']['xp']=804
                elif kind=='double_gold':r['trace']['gold'][2]['gold']=1016
                elif kind=='private_leak':reports['witness']['trace']['gold'][0]['gold']=1008
                elif kind=='level_loss':step(r,'CheckpointRestored')['player']['level']=2
                else:r['trace']['logins'][1]['mp']=60
                self.assertFalse(compare(reports,s,db,p,m)['success'])
    def test_probe_requires_atomic_level_heal_currency_and_withheld_notifications(self):
        _,_,db,_,_=fixture();q=db['reward'];o=q['blocked'][0]
        self.assertTrue(withheld(q['before'],o['committed'],o['dirty'],o['waits'],o['client'],o['unix_ms'],True))
        self.assertTrue(awarded(q['before'],q['committed'],True))
        for key,value in [('level',3),('money',1008),('xp_jp',[4,10])]:
            c=copy.deepcopy(o['client']);c[key]=value
            self.assertFalse(withheld(q['before'],o['committed'],o['dirty'],o['waits'],c,o['unix_ms'],True))
        for key,value in [('money',1007),('money',1015),('health',120),('mp',60),('jp',7)]:
            d=copy.deepcopy(q['committed']);d['owner'][key]=value;self.assertFalse(awarded(q['before'],d,True))
    def test_missing_peer_level_event_vitals_or_verified_monster_is_rejected(self):
        for kind in ['peer','peer_mp','event','max_hp','manifest','order','replay']:
            with self.subTest(kind=kind):
                reports,s,db,p,m=fixture();r=reports['owner']
                if kind=='peer':step(reports['witness'],'DurableReentry')['peer_unit']['level']=2
                elif kind=='peer_mp':step(reports['witness'],'DurableReentry')['peer_unit']['max_mp']=60
                elif kind=='event':r['events']['PLAYER_LEVEL_UP']=2
                elif kind=='max_hp':r['trace']['logins'][2]['max_hp']=120
                elif kind=='manifest':db['monster_fixture']['gold_max']=0
                elif kind=='order':r['trace']['level_orders'][0]=12
                else:r['trace']['levels'].append(dict(handle=1,level=3,job_level=1));r['trace']['level_orders'].append(30)
                self.assertFalse(compare(reports,s,db,p,m)['success'])

if __name__=='__main__':unittest.main()
