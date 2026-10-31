"""Compare two native clients against archived 3212 corpse targeting/consumption facts."""
import json
from pathlib import Path
import sys


def assess(owner, witness):
    checks=[]
    def check(name,value):checks.append({'name':name,'passed':bool(value)})
    old,new,peer,body=owner.get('initial'),owner.get('own'),witness.get('own'),owner.get('body')
    ot,wt=owner.get('trace',{}),witness.get('trace',{})
    os={p.get('stage'):p for p in owner.get('steps',[])};ws={p.get('stage'):p for p in witness.get('steps',[])}
    def player(s):return s.get('player') or {}
    def skill(p):return next((s for s in p.get('levels',[]) if s.get('id')==3212),{})
    events=ot.get('cast_events',[])
    check('both native corpse scenarios completed',owner.get('success') is True and witness.get('success') is True and owner.get('scenario')==witness.get('scenario')=='corpse_skill')
    check('distinct old fresh and observer identities agree',old and new and peer and len({old,new,peer})==3 and owner.get('other')==peer and witness.get('initial_other')==old and witness.get('other')==new)
    check('both clients use the same real monster handle',isinstance(body,int) and 0x20000000<=body<0x40000000 and witness.get('body')==body)
    phases=['CASTING','CANCEL','CASTING','CASTING','FIRE','CANCEL','CASTING','FIRE']
    check('eight exact expiration race and fresh-body phases', [e.get('phase') for e in events]==phases)
    check('winner and canceled competitor are correctly distinguished', [e.get('caster') for e in events]==[old,old,old,peer,old,peer,peer,peer])
    keys=('id','level','caster','target','layer','phase','duration','amount','mp_cost','caster_hp','caster_mp','corpse_fire')
    check('both clients receive identical skill facts',len(events)==8 and [[e.get(k) for k in keys] for e in events]==[[e.get(k) for k in keys] for e in wt.get('cast_events',[])])
    check('selected corpse is distinct from mana beneficiary',len(events)==8 and all(e.get('target')==body and e.get('target')!=e.get('caster') and e.get('id')==3212 and e.get('level')==1 and e.get('layer')==0 for e in events))
    check('only two fires debit five mana',len(events)==8 and all(e.get('mp_cost')==(5 if e.get('phase')=='FIRE' else 0) for e in events))
    check('archive nominal recovery is 320 not loose 3500',len(events)==8 and all(e.get('amount')==(320 if e.get('phase')=='FIRE' else 0) for e in events))
    check('cast duration remains 145 centiseconds',len(events)==8 and all(e.get('duration')==(145 if e.get('phase')=='CASTING' else 0) for e in events))
    check('fires contain checked corpse HP then MP results',len(events)==8 and all(e.get('corpse_fire')==(e.get('phase')=='FIRE') for e in events) and all(e.get('caster_mp')==140 and 0<e.get('caster_hp',0)<=280 for e in events if e.get('phase')=='FIRE'))
    check('eleven owner requests accept only the two corpse starts',len(owner.get('requests',[]))==11 and len(ot.get('cast_results',[]))==11 and [i for i,r in enumerate(ot.get('cast_results',[])) if r==0]==[6,7])
    check('seven witness requests accept race then fresh corpse',len(witness.get('requests',[]))==7 and len(wt.get('cast_results',[]))==7 and [i for i,r in enumerate(wt.get('cast_results',[])) if r==0]==[3,6])
    check('initial self player living unknown NPC and ownership probes are real',[r.get('label') for r in owner.get('requests',[])[:6]]==['self','player','living','unknown','npc','foreign_caster'] and len(owner.get('requests',[]))==11 and owner['requests'][0].get('target')==old and owner['requests'][1].get('target')==peer and owner['requests'][2].get('target')==body and owner['requests'][5].get('caster')==peer)
    check('witness probes expired consumed and living respawn targets',[r.get('label') for r in witness.get('requests',[])]==['self','living','expired','race','consumed','respawn_alive','fresh_corpse'])
    check('late cast starts on a still-rendered corpse',os.get('LateCast',{}).get('body_renderable') is True and (os.get('LateCast',{}).get('body') or {}).get('hp')==0 and os.get('LateCast',{}).get('elapsed_ms',0)-os.get('Kill1',{}).get('elapsed_ms',0)>=2000)
    check('expiry cancels before full incantation without cooldown','Expired' in os and os['Expired'].get('body') is None and os['Expired'].get('target') is None and os['Expired'].get('remaining')==0 and 0<os['Expired'].get('elapsed_ms',0)-os.get('LateCast',{}).get('elapsed_ms',0)<1450)
    check('expired query cannot recover a removed corpse','ExpiredQueryRefused' in ws and ws['ExpiredQueryRefused'].get('body') is None and ws['ExpiredQueryRefused'].get('target') is None)
    check('first consumption precedes natural three-second corpse removal','Consumed' in os and os['Consumed'].get('body') is None and 1300<=os['Consumed'].get('elapsed_ms',0)-os.get('Kill2',{}).get('elapsed_ms',0)<2800)
    check('canceled competitor receives no restoration or cooldown','RaceLost' in ws and ws['RaceLost'].get('body') is None and ws['RaceLost'].get('target') is None and ws['RaceLost'].get('casting') is False and ws['RaceLost'].get('remaining')==0)
    check('consumed corpse query stays hidden','ConsumedQueryRefused' in os and os['ConsumedQueryRefused'].get('body') is None and os['ConsumedQueryRefused'].get('target') is None)
    check('both original fifteen-second respawns remain intact',all(14500<=os.get(respawn,{}).get('elapsed_ms',0)-os.get(kill,{}).get('elapsed_ms',0)<21000 for kill,respawn in [('Kill1','Respawn1'),('Kill2','Respawn2')]))
    check('fresh-handle owner retains recharge and denies a fresh corpse','Reentered' in os and player(os['Reentered']).get('handle')==new and os['Reentered'].get('remaining',0)>0 and os.get('CooldownRefused',{}).get('remaining',0)>0)
    check('competitor can use later fresh corpse after canceled race','SecondConsumed' in ws and player(ws['SecondConsumed']).get('mp')==140 and ws['SecondConsumed'].get('remaining',0)>0 and ws['SecondConsumed'].get('body') is None)
    owner_lists,witness_lists=ot.get('skill_lists',[]),wt.get('skill_lists',[])
    check('private owner lists contain only owning handles',len(owner_lists)==3 and [p.get('target') for p in owner_lists]==[old,old,new])
    check('private observer lists contain only own handle',len(witness_lists)==2 and all(p.get('target')==peer for p in witness_lists))
    check('archive recharge packets survive owner character reload',len(owner_lists)==3 and [skill(p).get('total') for p in owner_lists]==[5800]*3 and skill(owner_lists[0]).get('remain')==0 and skill(owner_lists[1]).get('remain')==5800 and 0<skill(owner_lists[2]).get('remain',0)<=5800)
    check('canceled competitor receives recharge only after its own fire',len(witness_lists)==2 and [skill(p).get('remain') for p in witness_lists]==[0,5800] and all(skill(p).get('total')==5800 for p in witness_lists))
    check('skill levels never change or leak from casts',len(owner_lists)==3 and len(witness_lists)==2 and all(sorted((s.get('id'),s.get('base')) for s in p.get('levels',[]))==[(3212,1),(63203,1)] for p in owner_lists+witness_lists) and ot.get('learn_results')==wt.get('learn_results')==[])
    check('both clients agree on every real attack hit',bool(ot.get('hits')) and ot.get('hits')==wt.get('hits'))
    kills=[h for h in ot.get('hits',[]) if h.get('target')==body and h.get('target_hp')==0]
    check('exactly three real kills by owner owner then witness',len(kills)==3 and [h.get('attacker') for h in kills]==[old,old,peer])
    ox,wx=ot.get('experience',[]),wt.get('experience',[])
    e=ox[1].get('xp',0) if len(ox)>1 else 0;j=ox[1].get('jp',0) if len(ox)>1 else 0
    check('only actual kills award XP JP once each',len(ox)==4 and len(wx)==2 and e>0 and j>0 and [p.get('xp') for p in ox]==[0,e,2*e,2*e] and [p.get('jp') for p in ox]==[0,j,2*j,2*j] and [(p.get('xp'),p.get('jp')) for p in wx]==[(0,0),(e,j)])
    for role,report,trace,stages,handles,position,cycles in [('owner',owner,ot,os,[old,new],[8440,7227],2),('witness',witness,wt,ws,[peer],[8440,7247],1)]:
        enters=[p for p in trace.get('enters',[]) if p.get('handle')==body]
        check(f'{role} receives three living respawns and no hidden-query ENTER',len(enters)==3 and all(p.get('kind')=='monster' and p.get('hp')==p.get('max_hp')==105 and p.get('mp')==p.get('max_mp')==53 for p in enters))
        check(f'{role} removes the corpse once per death',trace.get('leaves',[]).count(body)==3)
        check(f'{role} completes real login logout cycles',[p.get('handle') for p in trace.get('logins',[])]==handles and report.get('events',{}).get('PLAYER_LOGIN')==cycles and report.get('events',{}).get('PLAYER_LOGOUT')==cycles and 'Logout' in stages)
        check(f'{role} receives eight real Lua phase diagnostics',report.get('events',{}).get('RAPPELZ_SKILL_EVENT')==8 and len([m for m in report.get('messages',[]) if 'SKILL 3212 ' in m])==8)
        own_caster=old if role=='owner' else peer;messages=report.get('messages',[])
        fire=[messages[i+1] for i,m in enumerate(messages[:-1]) if f'SKILL 3212 FIRE caster={own_caster} amount=320' in m]
        check(f'{role} own FIRE Lua callback reads confirmed mana',len(fire)==1 and 'EVENT RAPPELZ_SKILL_EVENT 3212 ' in fire[0] and 'MP=140/140' in fire[0])
        active=[p for p in stages.values() if player(p)]
        check(f'{role} keeps level layer positions inventory and money',len(active)>=6 and all(player(p).get('level')==10 and player(p).get('layer')==0 and player(p).get('max_hp')==280 and player(p).get('max_mp')==140 and 0<player(p).get('hp',0)<=280 and 0<=player(p).get('mp',-1)<=140 and p.get('position')==position and p.get('money')==p.get('bag_revision')==0 for p in active))
        check(f'{role} has no parser errors warp level change or manual player movement',trace.get('errors')==trace.get('warps')==trace.get('levels')==[] and all(p.get('handle')==body for p in trace.get('moves',[])) and not any(report.get('events',{}).get(event,0) for event in ('PLAYER_DEAD','PLAYER_ALIVE','PLAYER_LEVEL_UP')))
        check(f'{role} saves actual final restored state','Saved' in stages and player(stages['Saved']).get('mp')==140 and stages['Saved'].get('body') is None and stages['Saved'].get('target') is None and stages['Saved'].get('casting') is False)
    return {'success':all(c['passed'] for c in checks),'checks':checks}


if __name__=='__main__':
    directory=Path(sys.argv[1]).resolve(strict=True)
    result=assess(*[json.loads((directory/f'{role}.json').read_text(encoding='utf-8')) for role in ('owner','witness')])
    (directory/'comparison.json').write_text(json.dumps(result,indent=2),encoding='utf-8')
    print(json.dumps(result,indent=2));sys.exit(0 if result['success'] else 1)
