import hashlib
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch,MagicMock
from terrain_server_restart import validate,candidates,gameplay_candidates,server_environment,boot

class ServerRestartControlTests(unittest.TestCase):
    def activity(self,root):
        for role,own,peer,y in [('owner',1,2,7235),('witness',2,1,7255)]:
            value=dict(role=role,own=own,peer=peer,unix_ms=1000,moving=True,peer_moving=True,route_acks=2,own_hits=1,peer_hits=1,retaliations=1,
                monster=536870912,monster_hp=80,monster_max_hp=105,own_hp=82,instance=1,distance_from_checkpoint=10,position=[8462,y],checkpoint=[8452,y])
            (root/(role+'.activity.json')).write_text(json.dumps(value),encoding='utf-8')
    def test_live_gameplay_candidates_require_both_real_moving_combatants(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);self.activity(root);self.assertEqual(set(gameplay_candidates(root,1032)),{'owner','witness'})
    def test_stale_stationary_unacknowledged_or_dead_combat_is_rejected(self):
        for key,value in [('unix_ms',990),('moving',False),('peer_moving',False),('route_acks',0),('own_hits',0),('peer_hits',0),('retaliations',0),('monster_hp',0),('own_hp',0),('distance_from_checkpoint',0),('peer',999)]:
            with self.subTest(key=key),tempfile.TemporaryDirectory() as temp:
                root=Path(temp);self.activity(root);p=root/'owner.activity.json';v=json.loads(p.read_text());v[key]=value;p.write_text(json.dumps(v),encoding='utf-8');self.assertIsNone(gameplay_candidates(root,1032))
    def test_unknown_scenario_cannot_fall_back_to_a_kill_strategy(self):
        with tempfile.TemporaryDirectory() as temp:
            p=self.plan(Path(temp));p['scenario']='unknown'
            with self.assertRaises(ValueError):validate(p)
    def plan(self,root):
        workspace=root/'terrain-restart-deadbeef';workspace.mkdir();directory=workspace/'attempt';directory.mkdir()
        exe=workspace/'worldserver.exe';exe.write_bytes(b'unit-test executable placeholder')
        cfg=workspace/'world.conf';lines={k:f'127.0.0.1;3331;test;placeholder;rz_crash_deadbeef_{kind}' for k,kind in
            [('LoginDatabaseInfo','auth'),('CharacterDatabaseInfo','characters'),('WorldDatabaseInfo','world')]}
        lines.update(BindIP='127.0.0.1',RealmID='1',WorldServerPort='24514')
        cfg.write_text('\n'.join(k+' = "'+v+'"' for k,v in lines.items()),encoding='utf-8')
        return dict(workspace=str(workspace),directory=str(directory),executable=str(exe),config=str(cfg),executable_sha256=hashlib.sha256(exe.read_bytes()).hexdigest())
    def test_only_scoped_copied_binaries_configs_and_separate_databases_are_allowed(self):
        for kind in ('valid','outside','main_db','source_schema','main_port','remote','hash'):
            with self.subTest(kind=kind),tempfile.TemporaryDirectory() as temp:
                p=self.plan(Path(temp));cfg=Path(p['config']);s=cfg.read_text(encoding='utf-8')
                if kind=='outside':p['directory']=temp
                elif kind=='main_db':s=s.replace(';3331;',';3310;')
                elif kind=='source_schema':s=s.replace('rz_crash_deadbeef_auth','acore_auth')
                elif kind=='main_port':s=s.replace('24514','4514')
                elif kind=='remote':s=s.replace('127.0.0.1','0.0.0.0')
                elif kind=='hash':p['executable_sha256']='0'*64
                cfg.write_text(s,encoding='utf-8')
                if kind=='valid':self.assertEqual(validate(p)[-1],24514)
                else:
                    with self.assertRaises(ValueError):validate(p)
    def test_candidate_leases_reject_idle_stale_future_or_nonpartial_work(self):
        for kind in ('valid','null','list','stale','future','zero','done','not_running','malformed'):
            with self.subTest(kind=kind),tempfile.TemporaryDirectory() as temp:
                root=Path(temp)
                for role in ('owner','witness'):
                    value=dict(role=role,unix_ms=1000,running=True,rows=50,id=1,instance=1)
                    if role=='witness':
                        if kind=='null':value=None
                        elif kind=='list':value=[]
                        elif kind=='stale':value['unix_ms']=990
                        elif kind=='future':value['unix_ms']=1010
                        elif kind=='zero':value['rows']=0
                        elif kind=='done':value['rows']=1024
                        elif kind=='not_running':value['running']=False
                    (root/(role+'.candidate.json')).write_text('{' if kind=='malformed' and role=='witness' else json.dumps(value),encoding='utf-8')
                self.assertEqual(candidates(root,1008) is not None,kind=='valid')
    def test_child_environment_cannot_override_isolation_or_hide_readiness(self):
        with patch.dict(os.environ,{'AC_LOGIN_DATABASE_INFO':'main-db','AC_WORLD_SERVER_PORT':'4514','RUST_LOG':'warn'}):
            env=server_environment();self.assertFalse(any(k.startswith('AC_') for k in env));self.assertEqual(env['RUST_LOG'],'info')
            self.assertEqual(os.environ['RUST_LOG'],'warn');self.assertIn('AC_LOGIN_DATABASE_INFO',os.environ)
    def test_failed_readiness_cleans_up_the_exact_owned_process(self):
        with tempfile.TemporaryDirectory() as temp:
            proc=MagicMock();proc.poll.return_value=None
            with patch('terrain_server_restart.subprocess.Popen',return_value=proc),patch('terrain_server_restart.wait_until',side_effect=TimeoutError('readiness')):
                with self.assertRaises(TimeoutError):boot(Path('unused.exe'),Path('unused.conf'),Path(temp),24514,0)
            proc.kill.assert_called_once_with();proc.wait.assert_called_once_with(timeout=5)

if __name__=='__main__':unittest.main()
