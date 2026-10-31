import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from test_character_save_probe import save_fixture
from compare_gameplay_restart_clients import compare,ROLES,checkpoint,step
from reward_commit_probe import withheld,awarded
from terrain_server_restart import durable_reward_window


def fixture():
    reports,s,db,p,m=save_fixture();s['scenario']='reward-durable';q=db.pop('rollback');before=q['before']
    profiles=copy.deepcopy(before);profiles['owner'].update(xp=5,jp=3,health=80);profiles['owner']['settings']={'jp':'3'}
    before['owner']['settings']={'jp':'0'};before['witness']['settings']={'jp':'0'}
    profiles['witness']['settings']={'jp':'0'};db['checkpoint']=copy.deepcopy(before);db['down']=copy.deepcopy(profiles)
    dirty=copy.deepcopy(before);dirty['owner'].update(xp=5,health=80)
    obs=[dict(waits=q['waits'][:1],committed=copy.deepcopy(before),dirty=copy.deepcopy(dirty),client=dict(xp_jp=[0,0],unix_ms=n,order=13),unix_ms=n) for n in [1100,1700]]
    window=dict(pid=10,unix_ms=1710,committed=True,profiles=profiles);s['kill'].pop('pending_save');s['kill'].update(reward_commit=window,unix_ms=1720);s['boots'][1]['unix_ms']=6800
    db['reward']=dict(success=True,method='reward COMMIT before notification',before=before,blocked=obs,held_ms=600,released_unix_ms=1701,committed=profiles,after=copy.deepcopy(profiles),window=window,verified_after_kill_unix_ms=1721)
    db['console_save_requests']=1
    for r in ROLES:
        report=reports[r];report['scenario']='reward-durable';t=report['trace'];t['hits']=t['hits'][:len(t['hits'])//2];t['hit_orders']=t['hit_orders'][:len(t['hits'])]
        if r=='owner':
            # First-world award remains; second world only has the ordinary initial-state packet.
            i=t['experience_orders'].index(45);t['experience_orders'].pop(i);t['experience'].pop(i)
            i=t['experience_orders'].index(21);t['experience'][i].update(xp=5,jp=3);t['logins'][1]['hp']=80
        step(report,'CheckpointRestored')['xp_jp']=[5,3] if r=='owner' else [0,0]
        report['steps'].append(dict(stage='RewardRetained',hits=[],xp_jp=[5,3] if r=='owner' else [0,0]))
    reports['owner']['steps'].append(dict(stage='RewardWithheld',xp_jp=[0,0],hits=copy.deepcopy(reports['owner']['trace']['hits']),monster=536870912,order=13))
    return reports,s,db,p,m


class DurableRewardTests(unittest.TestCase):
    def test_commit_then_real_crash_and_three_logins_preserve_one_reward(self):
        result=compare(*fixture());self.assertTrue(result['success'],[c for c in result['checks'] if not c['passed']]);self.assertEqual(len(result['checks']),49)
    def test_recipient_need_not_be_the_monster_retaliation_victim(self):
        reports,s,db,p,m=fixture()
        db['saved']['owner']['health']=100;reports['owner']['trace']['logins'][2]['hp']=100
        # The unsaved witness damage may roll back while the rewarded owner was never hurt.
        result=compare(reports,s,db,p,m)
        self.assertTrue(result['success'],[c for c in result['checks'] if not c['passed']])
    def test_early_notification_partial_commit_loss_duplicate_or_later_save_are_rejected(self):
        for kind in ['early','dirty','loss','duplicate','late_save','short_hold','restored','killer','settings']:
            with self.subTest(kind=kind):
                reports,s,db,p,m=fixture();q=db['reward']
                if kind=='early':q['blocked'][0]['client']['xp_jp']=[5,3]
                elif kind=='dirty':q['blocked'][0]['committed']['owner']['xp']=5
                elif kind=='loss':db['down']['owner']['xp']=0
                elif kind=='duplicate':reports['owner']['trace']['experience'].insert(3,dict(handle=1,xp=10,jp=6));reports['owner']['trace']['experience_orders'].insert(3,40)
                elif kind=='late_save':db['console_save_requests']=2
                elif kind=='short_hold':q['held_ms']=10
                elif kind=='restored':reports['owner']['trace']['logins'][1]['hp']=100
                elif kind=='killer':reports['owner']['trace']['hits'][-1]['attacker']=2
                else:q['committed']['owner']['settings']['jp']='0'
                self.assertFalse(compare(reports,s,db,p,m)['success'])
    def test_probe_accepts_only_blocked_unannounced_reward_and_complete_private_commit(self):
        _,_,db,_,_=fixture();q=db['reward'];o=q['blocked'][0]
        self.assertTrue(withheld(q['before'],o['committed'],o['dirty'],o['waits'],o['client'],o['unix_ms']))
        self.assertTrue(awarded(q['before'],q['committed']))
        for kind in ['visible','stale','no_wait','already_jp']:
            v=copy.deepcopy(o)
            if kind=='visible':v['client']['xp_jp']=[5,3]
            elif kind=='stale':v['client']['unix_ms']=1
            elif kind=='no_wait':v['waits']=[]
            else:v['dirty']['owner']['jp']=3
            self.assertFalse(withheld(q['before'],v['committed'],v['dirty'],v['waits'],v['client'],v['unix_ms']))
        value=copy.deepcopy(q['committed']);value['witness']['xp']=5;self.assertFalse(awarded(q['before'],value))
    def test_controller_refuses_stale_wrong_process_and_failed_sql_probe(self):
        with tempfile.TemporaryDirectory() as temp,patch('terrain_server_restart.time.time_ns',return_value=2000_000_000):
            root=Path(temp);v=dict(pid=10,unix_ms=1000,committed=True)
            def write(): (root/'reward-durable-verified').write_text(json.dumps(v),encoding='utf-8')
            write();self.assertIsNotNone(durable_reward_window(root,10));self.assertIsNone(durable_reward_window(root,11))
            v['unix_ms']=1;write();self.assertIsNone(durable_reward_window(root,10))
            (root/'reward-commit.json').write_text(json.dumps(dict(success=False)),encoding='utf-8')
            with self.assertRaises(RuntimeError):durable_reward_window(root,10)
    def test_preexisting_extras_and_currencies_cannot_change_with_an_award(self):
        _,_,db,_,_=fixture();q=db['reward'];q['before']['owner']['settings']['custom']='keep';q['committed']['owner']['settings']['custom']='keep'
        self.assertTrue(awarded(q['before'],q['committed']))
        for field,value in [('settings',{'jp':'3'}),('money',100),('position',[0,0]),('health',0)]:
            v=copy.deepcopy(q['committed']);v['owner'][field]=value;self.assertFalse(awarded(q['before'],v))

if __name__=='__main__':unittest.main()
