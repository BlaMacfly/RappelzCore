import copy
import tempfile
from pathlib import Path
import unittest
from character_save_probe import guarded_fixtures,window_matches
from terrain_server_restart import pending_save_window
from unittest.mock import patch
from test_compare_gameplay_restart_clients import fixture
from compare_gameplay_restart_clients import compare,step,ROLES,final_position

def save_fixture():
    reports,server,db,p,m=fixture();server['scenario']='save-rollback'
    post=copy.deepcopy(reports['owner']['trace']['hits'][3:])
    for i,r in enumerate(ROLES):
        report=reports[r];report['scenario']='save-rollback';t=report['trace']
        t['hits']=copy.deepcopy(post+post);t['hit_orders']=list(range(5,5+len(post)))+list(range(26,26+len(post)))
        if r=='owner':t['experience'].insert(1,dict(handle=1,xp=5,jp=3));t['experience_orders'].insert(1,14)
        s=copy.deepcopy(step(report,'CheckpointSaved'));s.update(stage='BeforeInterruptedSave',own=i+1,peer_handle=2-i,monster=536870912,
            position=final_position(r),xp_jp=[5,3] if r=='owner' else [0,0],target=dict(hp=0))
        report['steps'].append(s)
        if r=='witness':
            assist=copy.deepcopy(step(report,'AssistStopped'));assist['stage']='BeforeSaveAssistStopped';report['steps'].append(assist)
        t['moves'][1]=dict(handle=2,points=[final_position('witness')],order=4)
    server['kill'].pop('activity');window=dict(pid=10,unix_ms=1101,locks_held=True,waiting_transactions=['80','81']);server['kill']['pending_save']=window
    dirty=copy.deepcopy(db['checkpoint'])
    for r in ROLES:dirty[r].update(position=final_position(r),xp=5 if r=='owner' else 0,health=80 if r=='owner' else 100)
    db['rollback']=dict(success=True,method='owned InnoDB settings row locks',window=window,waits=[dict(transaction=str(80+i),connection=100+i,modified=1,query='REPLACE INTO character_settings (guid, source, data) VALUES (?, ?, ?)') for i in range(2)],
        before=copy.deepcopy(db['checkpoint']),committed=copy.deepcopy(db['checkpoint']),dirty=dirty,after=copy.deepcopy(db['checkpoint']),transactions_gone=True,released_unix_ms=1106,
        request=dict(pid=10,unix_ms=1100,method='console saveall'))
    return reports,server,db,p,m

class SaveRollbackTests(unittest.TestCase):
    def test_complete_interrupted_save_and_two_native_recoveries_pass(self):
        value=compare(*save_fixture());self.assertTrue(value['success'],[c for c in value['checks'] if not c['passed']]);self.assertEqual(len(value['checks']),48)
    def test_committed_partial_missing_transaction_and_lost_rollback_are_rejected(self):
        for kind in ['committed','after','wait','dirty_jp','lease','unreleased','reward','private']:
            with self.subTest(kind=kind):
                reports,s,db,p,m=save_fixture();q=db['rollback']
                if kind=='committed':q['committed']['owner']['xp']=5
                elif kind=='after':q['after']['owner']['xp']=5
                elif kind=='wait':q['waits'][1]['query']='UPDATE characters'
                elif kind=='dirty_jp':q['dirty']['owner']['jp']=3
                elif kind=='lease':s['kill']['unix_ms']=4000
                elif kind=='unreleased':q['transactions_gone']=False
                elif kind=='reward':reports['owner']['trace']['experience'][1]['xp']=0
                else:reports['witness']['trace']['experience'][0]['xp']=10
                self.assertFalse(compare(reports,s,db,p,m)['success'])
    def test_probe_window_requires_two_actual_partial_updates_and_invisible_commit(self):
        _,_,db,_,_=save_fixture();q=db['rollback']
        self.assertTrue(window_matches(q['before'],q['committed'],q['dirty'],q['waits']))
        for field,value in [('modified',0),('query','SELECT 1')]:
            waits=copy.deepcopy(q['waits']);waits[0][field]=value
            self.assertFalse(window_matches(q['before'],q['committed'],q['dirty'],waits))
        self.assertFalse(window_matches(q['before'],q['dirty'],q['dirty'],q['waits']))
    def test_probe_cannot_lock_preexisting_or_duplicate_fixture_identities(self):
        fixtures={r:dict(guid=34+i,account=34+i,character='Lua'+r) for i,r in enumerate(ROLES)}
        self.assertEqual(guarded_fixtures(fixtures,33,33),dict(owner=34,witness=35))
        for field,value in [('guid',33),('account',33),('guid',35),('character','Existing')]:
            f=copy.deepcopy(fixtures);f['owner'][field]=value
            with self.assertRaises(ValueError):guarded_fixtures(f,33,33)
    def test_controller_rejects_expired_unheld_or_wrong_process_save_window(self):
        import json
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);v=dict(pid=10,unix_ms=1000,locks_held=True,waiting_transactions=['1','2'])
            def write(): (root/'save-window-verified').write_text(json.dumps(v),encoding='utf-8')
            with patch('terrain_server_restart.time.time_ns',return_value=2000_000_000):
                write();self.assertIsNotNone(pending_save_window(root,10));self.assertIsNone(pending_save_window(root,11))
                for field,value in [('unix_ms',1),('locks_held',False),('waiting_transactions',['1','1'])]:
                    original=v[field];v[field]=value;write();self.assertIsNone(pending_save_window(root,10));v[field]=original

if __name__=='__main__':unittest.main()
