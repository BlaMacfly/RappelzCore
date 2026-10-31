import copy
import hashlib
from pathlib import Path
import tempfile
import unittest
from test_money_level_reward import fixture as money_fixture
import test_terrain_server_restart as restart_tests
from compare_database_restart_clients import compare
from database_reward_probe import pending_reward_matches
from database_server_restart import database_settings,pending_window
from unittest.mock import patch
import json


def fixture():
    reports,old,source,pictures,memory=money_fixture();q=source['reward'];before=copy.deepcopy(q['committed']);q['database_restart']=True
    second=copy.deepcopy(before);second['owner'].update(xp=9,money=1016,health=90)
    pending=[dict(committed=copy.deepcopy(before),dirty=copy.deepcopy(second),waits=[dict(transaction='99',modified=1,query='REPLACE INTO character_settings (guid,source,data) VALUES (?, ?, ?)')],client=dict(xp_jp=[4,10],level=3,money=1008,unix_ms=n),unix_ms=n) for n in [1800,2400]]
    window=dict(pid=10,unix_ms=2410,locks_held=True,waiting_transactions=['99']);q.update(pending_reward=pending,pending_window=window,after=copy.deepcopy(before),transactions_gone=True)
    kill=dict(pid=100,world_pid=10,unix_ms=2420,method='owned database Popen.kill',graceful=False,exit_code=1,pending_reward=window)
    control=dict(success=True,scenario='database-reward',money_level=True,database_restart=True,boots=old['boots'][:1],database_boots=[dict(pid=100,data='isolated/mariadb',durability='1\t0\tON',unix_ms=900),dict(pid=101,data='isolated/mariadb',durability='1\t0\tON',unix_ms=8500)],database_kill=kill,
        first_reward=q['window'],world_alive_after_database_restart=True,world_pid_after_database_restart=10,final_exit_code=0,database_handed_off=True,recovery_log='InnoDB: Starting crash recovery from checkpoint LSN=123')
    second_hits=[];hp=105
    for attacker in [1,2,1,2,1,1,1]:
        hp-=15;second_hits.append(dict(attacker=attacker,target=536870912,damage=15,target_hp=hp))
    for role,r in reports.items():
        r['database_restart']=True;t=r['trace'];first=copy.deepcopy(t['hits']);t['hits']+=copy.deepcopy(second_hits);t['hit_orders']+=list(range(22,22+len(second_hits)))
        for values in [t['experience_orders'],t['gold_orders'],[]]:
            for i,n in enumerate(values):
                if n>=60:values[i]+=100
                elif n>=20:values[i]+=80
        for i,l in enumerate(t['logins']):
            if i==1:l['order']=100;l['handle']+=2
            elif i==2:l['order']=160;l['handle']+=2
        for m in t['moves']:
            if m['order']>=20:m['order']+=80;m['handle']+=2
        next(s for s in r['steps'] if s['stage']=='PostActionsStarted')['boundary']=104
        r['steps'].append(dict(stage='SecondActionsStarted',fight_boundary=20,monster=536870912))
        r['steps'].append(dict(stage='SecondRewardWithheld' if role=='owner' else 'SecondKillObserved',xp_jp=[4,10] if role=='owner' else [0,0],money=1008 if role=='owner' else 2000))
        if role=='witness':r['steps'].append(dict(stage='SaveDuringDatabaseOutage'))
    db=dict(reward=q,restored=copy.deepcopy(before),checkpoint=source['checkpoint'],saved=source['saved'],request=source['request'],monster_fixture=source['monster_fixture'],console_save_requests=1)
    drops=dict(owner=dict(joined=True,unix_ms=2500),witness=dict(joined=True,unix_ms=8000))
    return reports,control,db,pictures,memory,drops


class DatabaseRewardRestartTests(unittest.TestCase):
    def test_committed_award_survives_and_interrupted_award_disappears_with_two_real_sessions(self):
        v=compare(*fixture());self.assertTrue(v['success'],[c for c in v['checks'] if not c['passed']]);self.assertEqual(len(v['checks']),51)
    def test_lost_committed_gain_partial_recovery_phantom_connection_and_world_restart_fail(self):
        for kind in ['loss','partial','drop','world','directory','durability','duplicate','level']:
            with self.subTest(kind=kind):
                reports,c,db,p,m,drops=fixture()
                if kind=='loss':db['restored']['owner']['money']=1000
                elif kind=='partial':db['restored']['owner']['xp']=9
                elif kind=='drop':drops['witness']['unix_ms']=9000
                elif kind=='world':c['world_pid_after_database_restart']=20
                elif kind=='directory':c['database_boots'][1]['data']='other/directory'
                elif kind=='durability':c['database_boots'][0]['durability']='0\t0\tON'
                elif kind=='duplicate':reports['owner']['trace']['gold'][2]['gold']=1016
                else:reports['witness']['events']['PLAYER_LEVEL_UP']=1
                self.assertFalse(compare(reports,c,db,p,m,drops)['success'])
    def test_pending_probe_requires_fresh_unannounced_partial_second_reward(self):
        _,_,db,_,_,_=fixture();q=db['reward'];o=q['pending_reward'][0]
        self.assertTrue(pending_reward_matches(q['committed'],o['committed'],o['dirty'],o['waits'],o['client'],o['unix_ms']))
        for kind in ['stale','not_waiting','announced','wrong_jp','wrong_money']:
            value=copy.deepcopy(o)
            if kind=='stale':value['client']['unix_ms']-=151
            elif kind=='not_waiting':value['waits']=[]
            elif kind=='announced':value['client']['xp_jp']=[9,13]
            elif kind=='wrong_jp':value['dirty']['owner']['jp']=13
            else:value['dirty']['owner']['money']=1009
            self.assertFalse(pending_reward_matches(q['committed'],value['committed'],value['dirty'],value['waits'],value['client'],value['unix_ms']))
    def test_database_config_cannot_target_main_service_external_data_or_recovery_bypass(self):
        for kind in ['valid','main','outside','include','recovery','flush','hash','interface','directive']:
            with self.subTest(kind=kind),tempfile.TemporaryDirectory() as temp:
                base=Path(temp);plan=restart_tests.ServerRestartControlTests().plan(base);w=Path(plan['workspace']);data=w/'mariadb';(data/'mysql').mkdir(parents=True)
                binary=base/'MariaDB/bin';binary.mkdir(parents=True);exe=binary/'mariadbd.exe';exe.write_bytes(b'owned executable placeholder');(binary/'mariadb.exe').touch()
                cfg=w/'mariadb.ini';values=dict(basedir=str(binary.parent),datadir=str(data),port='3331',**{'bind-address':'127.0.0.1','log-error':str(w/'db.log'),'pid-file':str(w/'db.pid')})
                if kind=='main':values['port']='3310'
                elif kind=='outside':values['datadir']=str(base)
                elif kind=='include':values['init-file']='external.sql'
                elif kind=='recovery':values['innodb-force-recovery']='1'
                elif kind=='flush':values['innodb-flush-log-at-trx-commit']='0'
                elif kind=='interface':values['bind-address']='0.0.0.0'
                cfg.write_text('[mysqld]\n'+'\n'.join(k+'='+v for k,v in values.items())+('\n!include external.ini' if kind=='directive' else ''),encoding='utf-8')
                plan.update(database_restart=True,money_level=True,database=dict(executable=str(exe),config=str(cfg),executable_sha256='0'*64 if kind=='hash' else hashlib.sha256(exe.read_bytes()).hexdigest()))
                probe=dict(mysql_executable=str(binary/'mariadb.exe'))
                if kind=='valid':self.assertEqual(database_settings(plan,probe)[4],3331)
                else:
                    with self.assertRaises(ValueError):database_settings(plan,probe)
    def test_pending_kill_lease_rejects_wrong_world_stale_window_and_failed_probe(self):
        with tempfile.TemporaryDirectory() as temp,patch('database_server_restart.time.time_ns',return_value=2000_000_000):
            root=Path(temp);value=dict(pid=10,unix_ms=1000,locks_held=True,waiting_transactions=['99']);path=root/'database-pending-verified'
            path.write_text(json.dumps(value));self.assertIsNotNone(pending_window(root,10));self.assertIsNone(pending_window(root,11))
            value['unix_ms']=1;path.write_text(json.dumps(value));self.assertIsNone(pending_window(root,10))
            (root/'reward-commit.json').write_text(json.dumps(dict(success=False)))
            with self.assertRaises(RuntimeError):pending_window(root,10)

if __name__=='__main__':unittest.main()
