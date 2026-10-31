"""Independent packet/state/Lua/SQL checks for real late-observer corpse casts."""
import json
from pathlib import Path
import sys

ROLES=('owner','witness')
def step(report,name):return next((s for s in report.get('steps',[]) if s.get('stage')==name),{})
def compare(reports,server,processes,fixtures):
    checks=[]
    def check(name,value):checks.append(dict(name=name,passed=bool(value)))
    owner=reports.get('owner',{});witness=reports.get('witness',{});caster=owner.get('own');body=owner.get('body')
    check('one owned world exits normally after two native clients pass',server.get('success') is True and server.get('scenario')=='late-cast' and server.get('world_pid',0)>0 and server.get('exit_code')==0
          and not server.get('cleanup_clients') and not server.get('failed_attempt_shutdown') and len(processes)==2 and {v.get('role') for v in processes}==set(ROLES) and all(v.get('exit_code')==0 for v in processes))
    check('both observers identify the same distinct ordinary players and monster',caster and caster==witness.get('other') and owner.get('other')==witness.get('own') and caster!=witness.get('own') and body==witness.get('body') and 0x20000000<=body<0x40000000)
    check('independent SQL fixture verification passes for both offline GM0 characters',fixtures.get('success') is True and len(fixtures.get('checks',[]))==2 and all(v.get('offline') is True and v.get('gm')==0 and all(v.get(k) is True for k in ['progress','settings','position','vitals']) for v in fixtures['checks']))
    events={}
    for role in ROLES:
        r=reports.get(role,{});t=r.get('trace',{});e=t.get('cast_events',[]);events[role]=e;own=role=='owner';initial=step(r,'Initial');saved=step(r,'Saved');canceled=step(r,'Canceled');consumed=step(r,'Consumed');quiet=step(r,'Quiet');sql=fixtures.get('profiles',{}).get(role,{})
        check(role+': bounded native driver and one real Lua lifecycle complete',r.get('success') is True and not r.get('failure') and r.get('scenario')=='late-cast' and step(r,'Logout').get('elapsed_ms',10**9)<120000
              and len(t.get('logins',[]))==1 and all(r.get('events',{}).get(k)==1 for k in ['PLAYER_LOGIN','PLAYER_ENTERING_WORLD','PLAYER_LOGOUT']))
        check(role+': no protocol transport or denied-action error',all(t.get(k)==[] for k in ['errors','network_errors','disconnects','warps','denied_attacks','learn_results','levels']))
        check(role+': only the two matching starts and one cancel/fire are received',len(e)==4 and [v.get('phase') for v in e]==['CASTING','CANCEL','CASTING','FIRE'] and [v.get('order') for v in e]==sorted({v.get('order') for v in e})
              and all([v.get(k) for k in ['id','level','caster','target','layer']]==[3212,1,caster,body,0] for v in e))
        check(role+': each received phase is accepted once by real Lua state',r.get('events',{}).get('RAPPELZ_SKILL_EVENT')==4 and canceled.get('skill_event',{}).get('sequence')==2 and canceled['skill_event'].get('phase')=='CANCEL'
              and consumed.get('skill_event',{}).get('sequence')==quiet.get('skill_event',{}).get('sequence')==4 and consumed['skill_event'].get('phase')==quiet['skill_event'].get('phase')=='FIRE')
        check(role+': cancel has no debit and the sole real fire restores the caster',len(e)==4 and e[1].get('mp_cost')==0 and e[1].get('caster_mp')==e[0].get('caster_mp') and e[1].get('caster_hp')==e[0].get('caster_hp')
              and e[3].get('mp_cost')==5 and e[3].get('amount')==320 and e[3].get('corpse_fire') is True and e[3].get('caster_mp')==140)
        gold=[v.get('gold') for v in t.get('gold',[])]
        # A killing-blow frame can precede its durable reward notification. Validate
        # the two actual private currency packets, not that earlier rendered frame.
        wallet=(len(gold)==3 and gold[0]==1000 and gold[-1]==saved.get('money') and all(8<=gold[i+1]-gold[i]<=14 for i in (0,1))) if own else gold==[2000] and saved.get('money')==2000
        check(role+': existing private wallets and progression stay isolated',initial.get('xp_jp')==[0,0] and initial.get('money')==(1000 if own else 2000) and saved.get('xp_jp')==([10,6] if own else [0,0]) and wallet)
        check(role+': final normal save matches durable profile and unrelated learned settings',sql.get('level')==10 and sql.get('xp')==(10 if own else 0) and sql.get('money')==saved.get('money') and sql.get('online')==0 and sql.get('position')==saved.get('position')
              and saved.get('player',{}).get('hp',10**9)<=sql.get('health',-1)<=280 and saved['player'].get('mp',10**9)<=sql.get('mp',-1)<=140
              and all(sql.get('settings',{}).get(k)==v for k,v in [('job','302'),('jlv','35'),('layer','0'),('jp','6' if own else '0'),('learned_skill_3212','1'),('learned_skill_63203','1')]))
        lists=t.get('skill_lists',[])
        check(role+': skill/cooldown lists remain private and cancellation grants no recharge',len(lists)==(2 if own else 1) and all(v.get('target')==r.get('own') for v in lists)
              and all(s.get('remain')==0 for s in lists[0].get('levels',[])) and canceled.get('remaining')==0 and (consumed.get('remaining',0)>0 if own else consumed.get('remaining')==0))
    a=events['owner'];b=events['witness']
    check('late starts use remaining time instead of restarting the full incantation',len(a)==len(b)==4 and all(a[i].get('duration')==145 and 0<b[i].get('duration',0)<145 and b[i].get('caster_mp')==a[i].get('caster_mp') for i in (0,2)))
    check('both observers receive identical terminal events independent of packet order numbers',len(a)==len(b)==4 and all({k:v for k,v in a[i].items() if k!='order'}=={k:v for k,v in b[i].items() if k!='order'} for i in (1,3)))
    t=witness.get('trace',{});enters=t.get('enters',[])
    check('dead target and live caster ENTER precede each late CASTING',len(b)==4 and all(all(any(v.get('handle')==h and v.get('order',10**9)<b[i].get('order',0) and v.get('hp')==(0 if h==body else b[i].get('caster_hp')) for v in enters) for h in [caster,body]) for i in (0,2)))
    check('observer is hidden before both starts and receives no hidden phase',step(witness,'Hidden1').get('peer') is None and step(witness,'Hidden1').get('body') is None and step(witness,'Hidden1').get('skill_event') is None
          and step(witness,'Hidden2').get('peer') is None and step(witness,'Hidden2').get('body') is None and step(witness,'Hidden2').get('skill_event',{}).get('sequence')==2)
    check('observer crosses the boundary with real production movement',len(t.get('moves',[]))>=6 and sum(v.get('handle')==witness.get('own') and v.get('points')==[list(witness.get('far',[]))] for v in t['moves'])==3
          and sum(v.get('handle')==witness.get('own') and v.get('points')==[list(witness.get('near',[]))] for v in t['moves'])==3)
    check('reentering range after completion replays no old cast or effect',step(witness,'HiddenAfter').get('peer') is None and step(witness,'HiddenAfter').get('body') is None and step(witness,'Quiet').get('peer',{}).get('mp')==140
          and step(witness,'Quiet').get('elapsed_ms',0)-step(witness,'HiddenAfter').get('elapsed_ms',0)>=1000 and step(witness,'Quiet').get('skill_event',{}).get('sequence')==4)
    kills=[h for h in owner.get('trace',{}).get('hits',[]) if h.get('target')==body and h.get('target_hp')==0]
    check('two real owner kills award once each while hidden witness receives no combat',len(kills)==2 and all(h.get('attacker')==caster for h in kills) and witness.get('trace',{}).get('hits')==[]
          and [[v.get('xp'),v.get('jp')] for v in owner.get('trace',{}).get('experience',[])]==[[0,0],[5,3],[10,6]])
    check('queries do not duplicate casting and consumed corpse leaves both clients',len(b)==4 and sum(v.get('handle')==body for v in enters)>=4 and step(witness,'Consumed').get('body') is None and step(owner,'Consumed').get('body') is None)
    return dict(success=all(c['passed'] for c in checks),checks=checks)

def load_and_compare(directory):
    directory=Path(directory);read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
    return compare({r:read(directory/(r+'.json')) for r in ROLES},read(directory/'server-process.json'),read(directory/'processes.json'),read(directory.parent/'fixture-status.json'))
if __name__=='__main__':
    directory=Path(sys.argv[1]);result=load_and_compare(directory);(directory/'comparison.json').write_text(json.dumps(result,indent=2),encoding='utf-8');print(json.dumps(dict(success=result['success'],checks=len(result['checks']),failed=[c['name'] for c in result['checks'] if not c['passed']]),indent=2));raise SystemExit(0 if result['success'] else 1)
