import copy
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from compare_corpse_skill_clients import assess


def fixture():
    body=0x20000000
    def report(owner):
        return {'scenario':'corpse_skill','success':True,'initial':2 if owner else 1,'own':3 if owner else 1,'other':1 if owner else 3,'initial_other':1 if owner else 2,'body':body,
                'requests':[],'steps':[],'messages':[],'events':{'PLAYER_LOGIN':2 if owner else 1,'PLAYER_LOGOUT':2 if owner else 1,'RAPPELZ_SKILL_EVENT':8},
                'trace':{'cast_events':[],'cast_results':[6]*6+[0,0,6,6,6] if owner else [6,6,6,0,6,6,0],
                         'skill_lists':[],'learn_results':[],'hits':[],'experience':[{'xp':e,'jp':j} for e,j in ([(0,0),(5,3),(10,6),(10,6)] if owner else [(0,0),(5,3)])],
                         'enters':[{'handle':body,'kind':'monster','hp':105,'max_hp':105,'mp':53,'max_mp':53} for _ in range(3)],
                         'leaves':[body]*3,'logins':[{'handle':h} for h in ([2,3] if owner else [1])],'errors':[],'warps':[],'levels':[],'moves':[{'handle':body}]}}
    o,w=report(True),report(False)
    phases=['CASTING','CANCEL','CASTING','CASTING','FIRE','CANCEL','CASTING','FIRE'];casters=[2,2,2,1,2,1,1,1]
    for i,(phase,caster) in enumerate(zip(phases,casters)):
        event={'id':3212,'level':1,'caster':caster,'target':body,'layer':0,'phase':phase,'duration':145 if phase=='CASTING' else 0,'amount':320 if phase=='FIRE' else 0,'mp_cost':5 if phase=='FIRE' else 0,'caster_hp':270,'caster_mp':140 if phase=='FIRE' else 40,'corpse_fire':phase=='FIRE','order':i}
        for r,own in ((o,2),(w,1)):
            r['trace']['cast_events'].append(copy.deepcopy(event));r['messages'].append(f'[RappelzDebug] SKILL 3212 {phase} caster={caster} amount={event["amount"]}')
            r['messages'].append(f'[RappelzDebug] EVENT RAPPELZ_SKILL_EVENT 3212 MP={140 if phase=="FIRE" and caster==own else 40}/140')
    for role,r,labels,caster in [('owner',o,['self','player','living','unknown','npc','foreign_caster','late_expiring','race','consumed','respawn_alive','cooldown_fresh_body'],2),('witness',w,['self','living','expired','race','consumed','respawn_alive','fresh_corpse'],1)]:
        r['requests']=[{'label':label,'id':3212,'level':1,'caster':caster,'target':body,'layer':0} for label in labels]
    o['requests'][0]['target']=2;o['requests'][1]['target']=1;o['requests'][3]['target']=0x2fffffff;o['requests'][4]['target']=0x40000000;o['requests'][5]['caster']=1;w['requests'][0]['target']=1
    levels=[{'id':3212,'base':1,'current':1,'total':5800,'remain':0},{'id':63203,'base':1,'current':1,'total':0,'remain':0}]
    for r,rows in ((o,[(2,0),(2,5800),(3,5600)]),(w,[(1,0),(1,5800)])):
        for handle,remain in rows:
            row=copy.deepcopy(levels);row[0]['remain']=remain;r['trace']['skill_lists'].append({'target':handle,'levels':row})
        r['trace']['hits']=[{'attacker':h,'target':body,'target_hp':0,'damage':50} for h in (2,2,1)]
    owner_times=[('Initial',3000),('Refused',3300),('Kill1',7000),('LateCast',9050),('Expired',10000),('Respawn1',22000),('Kill2',25000),('Consumed',26500),('ConsumedQueryRefused',26700),('Reentered',29000),('Respawn2',40000),('AliveRefused',40100),('CooldownRefused',43500),('Saved',45100),('Logout',45150)]
    witness_times=[('Initial',3000),('ExpiredHidden',10000),('ExpiredQueryRefused',10100),('Racing',25100),('RaceLost',26600),('PeerReload',29100),('Respawn2',40100),('Kill3',43600),('SecondConsumed',45100),('Saved',45200),('Logout',45300)]
    for r,rows,owner in ((o,owner_times,True),(w,witness_times,False)):
        for name,time in rows:
            handle=(3 if time>=29000 else 2) if owner else 1
            s={'stage':name,'elapsed_ms':time,'player':None if name=='Logout' else {'handle':handle,'name':'Déva','hp':280,'max_hp':280,'mp':140 if name in ('Saved','Consumed','Reentered','SecondConsumed') else 40,'max_mp':140,'level':10,'layer':0},'target':None,'body':None,'remaining':0,'casting':False,'position':[8440,7227] if owner else [8440,7247],'money':0,'bag_revision':0}
            if name=='LateCast':s.update(body={'handle':body,'hp':0},body_renderable=True)
            if name in ('Reentered','CooldownRefused','SecondConsumed','Saved'):s['remaining']=40
            r['steps'].append(s)
    return o,w


class CorpseSkillComparisonTests(unittest.TestCase):
    def test_independent_complete_fixture_passes(self):
        result=assess(*fixture());self.assertTrue(result['success'],[c['name'] for c in result['checks'] if not c['passed']]);self.assertEqual(len(result['checks']),48)

    def test_empty_or_failed_scenarios_cannot_pass(self):
        self.assertFalse(assess({}, {})['success']);o,w=fixture();w['success']=False;self.assertFalse(assess(o,w)['success'])

    def test_mutations_reject_self_cast_double_consumption_missing_removal_and_rewards(self):
        mutations=[lambda o,w:o['trace']['cast_events'][4].update(target=2),lambda o,w:o['trace']['cast_events'][4].update(amount=3500),
                   lambda o,w:o['trace']['cast_events'][4].update(mp_cost=0),lambda o,w:o['trace']['cast_events'][4].update(corpse_fire=False),
                   lambda o,w:o['trace']['cast_results'].__setitem__(0,0),lambda o,w:w['trace']['cast_results'].__setitem__(4,0),
                   lambda o,w:w['trace']['leaves'].pop(),lambda o,w:o['trace']['skill_lists'][2]['levels'][0].update(remain=0),
                   lambda o,w:o['trace']['experience'][-1].update(jp=9),lambda o,w:o['steps'][7].update(elapsed_ms=29000),
                   lambda o,w:w['steps'][4].update(remaining=58),lambda o,w:o['trace']['moves'].append({'handle':2}),
                   lambda o,w:o['messages'].__setitem__(9,'[RappelzDebug] EVENT RAPPELZ_SKILL_EVENT 3212 MP=40/140')]
        for mutate in mutations:
            with self.subTest(mutation=mutate):o,w=fixture();mutate(o,w);self.assertFalse(assess(o,w)['success'])

    def test_cli_writes_utf8_comparison(self):
        with tempfile.TemporaryDirectory() as directory:
            p=Path(directory)
            for role,report in zip(('owner','witness'),fixture()):(p/f'{role}.json').write_text(json.dumps(report,ensure_ascii=False),encoding='utf-8')
            result=subprocess.run([sys.executable,str(Path(__file__).with_name('compare_corpse_skill_clients.py')),str(p)],capture_output=True)
            self.assertEqual(result.returncode,0,result.stderr);self.assertTrue(json.loads((p/'comparison.json').read_text(encoding='utf-8'))['success'])


if __name__=='__main__':unittest.main()
