import copy
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from compare_casting_clients import assess


def fixture():
    def report(role):
        owner=role=='owner';handle=3 if owner else 1
        return {'scenario':'casting','success':True,'initial':2 if owner else 1,'own':handle,
                'other':1 if owner else 3,'initial_other':1 if owner else 2,
                'requests':[], 'steps':[], 'messages':[],
                'events':{'RAPPELZ_SKILL_EVENT':8,'PLAYER_LOGIN':2 if owner else 1,'PLAYER_LOGOUT':2 if owner else 1},
                'trace':{'hits':[],'warps':[],'levels':[],'errors':[], 'cast_events':[],
                         'cast_results':[6]*6+[0,0,6,6,6,0,0] if owner else [6,6],
                         'learn_results':[0] if owner else [],'skill_lists':[],
                         'logins':[{'handle':2},{'handle':3}] if owner else [{'handle':1}],
                         'experience':[{'xp':0,'jp':n} for n in ([180,0,0] if owner else [0])],
                         'hpmp':[{'handle':handle,'hp':100,'max_hp':100,'mp':50,'max_mp':50,'add_hp':0}],
                         'enters':[] if owner else [{'handle':3},{'handle':3}],
                         'leaves':[] if owner else [2,3],
                         'moves':[{'handle':3,'points':[[8440,7227]]}]}}
    o,w=report('owner'),report('witness')
    phases=['CASTING','CANCEL','CASTING','FIRE']*2
    for index,phase in enumerate(phases):
        event={'id':3212,'level':1,'caster':2 if index<4 else 3,'target':2 if index<4 else 3,
               'layer':0,'phase':phase,'duration':145 if phase=='CASTING' else 0,
               'amount':3500 if phase=='FIRE' else 0,'mp_cost':5 if phase=='FIRE' else 0,
               'caster_hp':100,'caster_mp':12 if index<3 else 50,'order':index}
        for r in (o,w):
            r['trace']['cast_events'].append(copy.deepcopy(event));r['messages'].append(f'[RappelzDebug] SKILL 3212 {phase} caster={event["caster"]} amount={event["amount"]}')
            r['messages'].append(f'[RappelzDebug] EVENT RAPPELZ_SKILL_EVENT 3212 HP=100/100 MP={event["caster_mp"]}/50')
    def request(handle):return {'id':3212,'level':1,'caster':handle,'target':handle,'layer':0}
    o['requests']=[request(2) for _ in range(10)]+[request(3) for _ in range(3)]
    o['requests'][0]['caster']=1;o['requests'][1]['target']=1;o['requests'][2]['id']=65535;o['requests'][4]['id']=63203;o['requests'][5]['level']=0
    w['requests']=[request(1),request(3)]
    base={'id':63203,'base':1,'current':1,'total':0,'remain':0}
    o['trace']['skill_lists']=[{'target':2,'levels':[base]}]
    for handle,remain in [(2,0),(2,2500),(3,2490),(3,2500)]:o['trace']['skill_lists'].append({'target':handle,'levels':[{'id':3212,'base':1,'current':1,'total':2500,'remain':remain},base]})
    w['trace']['skill_lists']=[{'target':1,'levels':[]}]
    for name in ('Initial','Refused','Learned','CastingCancel','Canceled','FirstFire','Lobby','Reentered','CooldownExpired','CastingMove','MoveCanceled','SecondFire','Logout'):
        r={'stage':name,'player':None,'remaining':0,'casting':False}
        if name not in ('Lobby','Logout'):
            r.update(player={'handle':3 if name in ('Reentered','CooldownExpired','CastingMove','MoveCanceled','SecondFire') else 2,'hp':100,'max_hp':100,'mp':50,'level':1,'layer':0,'name':'Déva'},position=[8440,7227],money=0,bag_revision=0,jp=180 if name in ('Initial','Refused') else 0)
        if name in ('FirstFire','Reentered','SecondFire'):r['remaining']=24.9
        if name in ('CastingCancel','CastingMove'):r['casting']=True
        o['steps'].append(r)
    for name in ('Initial','ReenteredPeer','FreshQuery','Observed','PeerLogout','Logout'):
        w['steps'].append({'stage':name,'player':None if name=='Logout' else {'handle':1,'hp':100,'max_hp':100,'mp':50,'level':1,'layer':0},'position':[8420,7240],'money':0,'bag_revision':0,'learned':{},'jp':0})
    return o,w


class CastingComparisonTests(unittest.TestCase):
    def test_independent_valid_pair_passes(self):
        result=assess(*fixture());self.assertTrue(result['success'],[c['name'] for c in result['checks'] if not c['passed']]);self.assertEqual(len(result['checks']),34)

    def test_empty_or_incomplete_reports_fail(self):
        self.assertFalse(assess({}, {})['success'])
        o,w=fixture();o['success']=False;self.assertFalse(assess(o,w)['success'])

    def test_mutations_of_wire_privacy_cost_cooldown_and_real_state_fail(self):
        mutations=[
            lambda o,w:o['trace']['cast_events'][3].update(mp_cost=0),
            lambda o,w:o['trace']['cast_events'][0].update(duration=150),
            lambda o,w:w['trace']['cast_events'][3].update(amount=50),
            lambda o,w:o['trace']['cast_results'].__setitem__(8,0),
            lambda o,w:w['trace']['skill_lists'][0]['levels'].append({'id':3212,'base':1}),
            lambda o,w:o['trace']['skill_lists'][3]['levels'][0].update(remain=0),
            lambda o,w:o['events'].update(RAPPELZ_SKILL_EVENT=0),
            lambda o,w:o['steps'][5]['player'].update(mp=49),
            lambda o,w:o['trace']['moves'][0].update(points=[[9000,9000]]),
            lambda o,w:w['steps'][2].update(jp=180),
            lambda o,w:o['trace']['hpmp'][0].update(hp=0),
            lambda o,w:o['messages'].__setitem__(7,'[RappelzDebug] EVENT RAPPELZ_SKILL_EVENT 3212 MP=12/50'),
        ]
        for mutate in mutations:
            with self.subTest(mutation=mutate):
                o,w=fixture();mutate(o,w);self.assertFalse(assess(o,w)['success'])

    def test_utf8_cli_writes_reviewable_comparison(self):
        with tempfile.TemporaryDirectory() as directory:
            p=Path(directory)
            for role,report in zip(('owner','witness'),fixture()):(p/f'{role}.json').write_text(json.dumps(report,ensure_ascii=False),encoding='utf-8')
            result=subprocess.run([sys.executable,str(Path(__file__).with_name('compare_casting_clients.py')),str(p)],capture_output=True)
            self.assertEqual(result.returncode,0,result.stderr);self.assertTrue(json.loads((p/'comparison.json').read_text(encoding='utf-8'))['success'])


if __name__=='__main__':unittest.main()
