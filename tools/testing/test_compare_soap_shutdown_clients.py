import copy
import unittest
from compare_soap_shutdown_clients import compare


def fixture():
    cases = {}
    for index, label in enumerate(['negative', 'normal', 'slow']):
        negative = label == 'negative'; fs = {}
        for n, role in enumerate(['owner', 'witness']):
            fs[role] = dict(guid=34+index*2+n, account=34+index*2+n, character='Lua'+label+role,
                            money=1000 if n == 0 else 2000, position=[9440-n*20,7227+n*20],
                            settings=dict(job='100' if n == 0 else '200',jlv='1',jp='8' if n == 0 else '0',layer='0',marker='preserve'))
        def profiles(online, moved, progressed=True):
            result = {}
            for role, f in fs.items():
                settings=copy.deepcopy(f['settings'])
                if progressed: settings.update(jlv='2' if role == 'owner' else '1',jp='5' if role == 'owner' else '0')
                result[role]=dict(guid=f['guid'],account=f['account'],character=f['character'],level=10,xp=0,money=f['money'],health=280,mp=140,
                                  position=[f['position'][0]+(12 if moved else 0),f['position'][1]+(8 if moved else 0)],online=online,account_online=online,settings=settings)
            return result
        server=dict(success=True,negative=negative,first_pid=10,first_exit_code=0,stdin_remained_open=True,soap_response={'status':200},live_soap_info=dict(status=200,result='joueurs : 2'),clients=[dict(role=r,exit_code=0) for r in fs],saved_before_exit=not negative)
        c=dict(server=server,fixtures=fs,seeded=profiles(0,False,False),progress=profiles(1,False),before=profiles(1,False),after=profiles(1,False) if negative else profiles(0,True),reports={})
        if label != 'normal': server.update(save_waits=[dict(transaction=str(n),modified=1,settings_replace=True) for n in [1,2]],committed_while_blocked=profiles(1,False),dirty_while_blocked=profiles(1,True),held_seconds=6,alive_before_release=not negative,exit_seconds=6.1)
        if not negative: c.update(reentry=profiles(1,True),final=profiles(0,True));server.update(second_pid=20,second_exit_code=0,last_soap_response={'status':200})
        for role, f in fs.items():
            owner=role == 'owner';entries=1 if negative else 2;other='witness' if owner else 'owner'
            stages=['Initial','ProgressCommitted','MovedLive','Disconnected']+([] if negative else ['Reentered','SavedLobby'])
            live=dict(stage='MovedLive',elapsed_ms=1000,position=profiles(0,True)[role]['position'],peer_position=profiles(0,True)[other]['position'],peer={'name':fs[other]['character']},player=dict(handle=f['guid'],name=f['character'],level=10,hp=280,mp=140,max_hp=280,max_mp=140),money=f['money'],xp_jp=[0,5 if owner else 0],job=dict(job=100 if owner else 200,jlv=2 if owner else 1,jp=5 if owner else 0))
            steps=[dict(stage=s,elapsed_ms=2000) for s in stages];steps[2]=live
            if not negative: steps[4]=dict(copy.deepcopy(live),stage='Reentered',elapsed_ms=3000);steps[4]['player']['handle']+=100
            events={k:entries for k in ['PLAYER_LOGIN','PLAYER_ENTERING_WORLD','PLAYER_LOGOUT']};events.update(RAPPELZ_JP_CHANGED=entries+owner,RAPPELZ_JOB_LEVEL_CHANGED=entries+owner)
            trace=dict(errors=[],network_errors=['Déconnecté par le serveur (code 0).'],hits=[],cast_events=[],learn_results=[],job_results=[0] if owner else [],logins=[{}]*entries,gold=[{'gold':f['money']}]*entries,experience=[dict(xp=0,jp=p) for p in ([8,5]+([] if negative else [5]) if owner else [0]*entries)])
            c['reports'][role]=dict(success=True,negative=negative,scenario='soap-player-shutdown',steps=steps,events=events,trace=trace,messages=[f'EVENT PLAYER_LOGOUT money={f["money"]} jp={5 if owner else 0} jlv={2 if owner else 1}']*entries)
        cases[label]=c
    return cases


class ShutdownCharacterComparison(unittest.TestCase):
    def test_accepts_complete_three_case_evidence(self):
        self.assertTrue(compare(fixture())['success'])

    def test_saved_flag_cannot_hide_lost_position_or_missing_offline_commit(self):
        for field,value in [('position',[9440,7227]),('online',1),('account_online',1)]:
            cases=fixture();cases['slow']['after']['owner'][field]=value
            self.assertFalse(compare(cases)['success'])

    def test_rejects_lost_unrelated_settings_or_changed_money_after_restart(self):
        cases=fixture();del cases['normal']['reentry']['owner']['settings']['marker'];self.assertFalse(compare(cases)['success'])
        cases=fixture();cases['normal']['final']['witness']['money']+=1;self.assertFalse(compare(cases)['success'])

    def test_rejects_a_counter_control_that_does_not_reproduce_data_loss(self):
        cases=fixture();cases['negative']['server']['saved_before_exit']=True
        self.assertFalse(compare(cases)['success'])

    def test_rejects_unobserved_sql_delay_and_premature_exit(self):
        for field,value in [('alive_before_release',False),('held_seconds',1),('save_waits',[])]:
            cases=fixture();cases['slow']['server'][field]=value;self.assertFalse(compare(cases)['success'])
        cases=fixture();cases['slow']['server']['committed_while_blocked']=cases['slow']['server']['dirty_while_blocked'];self.assertFalse(compare(cases)['success'])

    def test_rejects_missing_logout_replayed_progress_or_synthetic_reentry(self):
        cases=fixture();cases['normal']['reports']['owner']['events']['PLAYER_LOGOUT']=1;self.assertFalse(compare(cases)['success'])
        cases=fixture();cases['normal']['reports']['owner']['trace']['experience'].append({'xp':0,'jp':8});self.assertFalse(compare(cases)['success'])
        cases=fixture();cases['normal']['reports']['witness']['steps'][4]['position']=[9420,7247];self.assertFalse(compare(cases)['success'])
