"""Check simultaneous movement/combat at process death and durable, non-replayed recovery."""
import hashlib
import json
from pathlib import Path
import struct
import sys
from compare_terrain_cancellation_clients import audit_facts

ROLES=('owner','witness')
def checkpoint(role,money_level=False):return [9452 if money_level else 8452,7235 if role=='owner' else 7255]
def final_position(role,money_level=False):
    p=checkpoint(role,money_level);p[0]+=16 if role=='witness' else 0;return p
def epochs(trace,key,orders):
    logins=trace.get('logins',[]);values=trace.get(key,[]);positions=trace.get(orders,[])
    return [[v for v,o in zip(values,positions) if o>l.get('order',0) and (i+1==len(logins) or o<logins[i+1].get('order',0))] for i,l in enumerate(logins)]
def step(report,name):return next((s for s in report.get('steps',[]) if s.get('stage')==name),{})
def damage_chain(hits,monster):
    hp=105
    for h in hits:
        if h.get('target')!=monster:continue
        if h.get('damage',0)<=0 or h.get('target_hp')!=max(0,hp-h['damage']):return -1
        hp=h['target_hp']
    return hp

def compare(reports,server,database,pictures,memory):
    checks=[];facts={}
    def check(name,value):checks.append(dict(name=name,passed=bool(value)))
    rollback=server.get('scenario')=='save-rollback'
    reward_durable=server.get('scenario')=='reward-durable'
    prefight=rollback or reward_durable
    money_level=reward_durable and database.get('reward',{}).get('money_level') is True
    def baseline(r):
        return ([2,799,7,1000,120,60,1] if r=='owner' else [1,0,0,2000,100,50,1]) if money_level else [1,0,0,0,100,50,1]
    def progress(r):return ([4,10] if money_level else [5,3]) if r=='owner' else [0,0]
    def mp(r):return 70 if money_level and r=='owner' else 50
    def maxhp(r):return 140 if money_level and r=='owner' else 100
    boots=server.get('boots',[]);kill=server.get('kill',{});active=kill.get('activity',{})
    if prefight:
        active={r:dict(own=step(reports[r],'BeforeInterruptedSave').get('own'),peer=step(reports[r],'BeforeInterruptedSave').get('peer_handle'),monster=step(reports[r],'BeforeInterruptedSave').get('monster')) for r in ROLES}
    cp=database.get('checkpoint',{});saved=database.get('saved',{});request=database.get('request',{})
    check('real owned gameplay server kill and restart',server.get('scenario') in ('gameplay','save-rollback','reward-durable') and server.get('success') is True and not server.get('cleanup_kill')
        and len(boots)==2 and boots[0].get('pid')!=boots[1].get('pid') and kill.get('pid')==boots[0].get('pid') and kill.get('graceful') is False
        and kill.get('exit_code') not in (0,None) and kill.get('method')=='owned Popen.kill' and server.get('final_exit_code')==0)
    check('server outage lasts at least five seconds',len(boots)==2 and boots[1].get('unix_ms',0)-kill.get('unix_ms',0)>=5000)
    check('real console save request precedes active crash',len(boots)==2 and request.get('pid')==boots[0].get('pid') and request.get('method')=='console saveall'
        and 0<request.get('unix_ms',0)<kill.get('unix_ms',0))
    check('both durable checkpoints contain completed movement and healthy baseline',set(cp)==set(ROLES) and all(cp[r].get('position')==checkpoint(r,money_level)
        and [cp[r].get(k) for k in ('level','xp','jp','money','health','mp','online')]==baseline(r) for r in ROLES))
    reward=database.get('reward',{});reward_saved=reward.get('committed',{})
    check('durable reward or last committed checkpoint survives process death',database.get('down')==(reward_saved if reward_durable else cp))
    restarted=database.get('restarted',{})
    check('startup online recovery remains scoped',all(restarted.get(r)==dict(account=0,character=0) for r in ROLES) and restarted.get('same_realm')==0 and restarted.get('other_realm')==2)
    if reward_durable:
        blocked=reward.get('blocked',[]);window=reward.get('window',{})
        check('real reward write is blocked before notification for at least half a second',reward.get('success') is True and reward.get('method')=='reward COMMIT before notification'
            and len(blocked)>=2 and reward.get('held_ms',0)>=500 and blocked[-1].get('unix_ms',0)-blocked[0].get('unix_ms',0)>=500
            and all(v.get('committed')==cp and len(v.get('waits',[]))==1 and v['waits'][0].get('modified',0)>0
                and 'replace into character_settings' in v['waits'][0].get('query','').lower() and v.get('dirty',{}).get('owner',{}).get('xp')==progress('owner')[0]
                and v['dirty']['owner'].get('jp')==baseline('owner')[2] and v['dirty'].get('witness')==cp.get('witness')
                and v.get('client',{}).get('xp_jp')==baseline('owner')[1:3] and 0<=v.get('unix_ms',0)-v['client'].get('unix_ms',0)<=150 for v in blocked))
        check('reward commits XP JP and vitals together before owned process death',set(reward_saved)==set(ROLES) and reward.get('after')==reward_saved
            and reward_saved.get('witness')==cp.get('witness') and [reward_saved['owner'].get('xp'),reward_saved['owner'].get('jp')]==progress('owner')
            and reward_saved['owner'].get('position')==checkpoint('owner',money_level) and 0<reward_saved['owner'].get('health',0)<=maxhp('owner')
            and reward_saved['owner']==dict(cp.get('owner',{}),level=3 if money_level else 1,xp=progress('owner')[0],jp=progress('owner')[1],money=reward_saved['owner'].get('money') if money_level else 0,mp=mp('owner'),health=reward_saved['owner']['health'],settings=dict(cp.get('owner',{}).get('settings',{}),jp=str(progress('owner')[1]))))
        check('kill follows released reward commit without a later manual save',window==kill.get('reward_commit') and window.get('committed') is True and window.get('pid')==kill.get('pid')
            and window.get('profiles')==reward_saved and len(blocked)>=2 and blocked[-1].get('unix_ms',0)<=reward.get('released_unix_ms',0)<=window.get('unix_ms',0)
            and 0<=kill.get('unix_ms',0)-window.get('unix_ms',0)<=1500 and reward.get('verified_after_kill_unix_ms',0)>=kill.get('unix_ms',0)
            and database.get('console_save_requests')==1)
        withheld=step(reports['owner'],'RewardWithheld');trace=reports['owner'].get('trace',{});first=epochs(trace,'experience','experience_orders')
        check('owner observes real killing blow while reward stays unannounced',withheld.get('xp_jp')==baseline('owner')[1:3] and damage_chain(withheld.get('hits',[]),withheld.get('monster'))==0
            and len(first)==3 and any([v.get('xp'),v.get('jp')]==progress('owner') and o>withheld.get('order',0)
                for v,o in zip(trace.get('experience',[]),trace.get('experience_orders',[])) if o<trace['logins'][1].get('order',0)))
        check('both observers agree on pre-crash identities',active['owner'].get('own')==active['witness'].get('peer') and active['witness'].get('own')==active['owner'].get('peer')
            and active['owner'].get('monster',0)>0 and active['owner'].get('monster')==active['witness'].get('monster'))
    elif rollback:
        probe=database.get('rollback',{});window=probe.get('window',{});waits=probe.get('waits',[]);dirty=probe.get('dirty',{})
        check('real save requests have two row-modified transactions waiting at settings REPLACE',probe.get('success') is True
            and probe.get('method')=='owned InnoDB settings row locks' and len(waits)==2 and len({w.get('transaction') for w in waits})==2
            and all(w.get('modified',0)>0 and 'replace into character_settings' in w.get('query','').lower() for w in waits)
            and window==kill.get('pending_save') and window.get('locks_held') is True and window.get('pid')==kill.get('pid')
            and sorted(window.get('waiting_transactions',[]))==sorted(w.get('transaction','') for w in waits)
            and 0<=kill.get('unix_ms',0)-window.get('unix_ms',0)<=1500
            and request.get('unix_ms',0)<probe.get('request',{}).get('unix_ms',0)<=window.get('unix_ms',0)
            and probe.get('request',{}).get('pid')==kill.get('pid') and probe.get('request',{}).get('method')=='console saveall')
        check('partial uncommitted characters never leak to committed readers',probe.get('before')==probe.get('committed')==cp
            and set(dirty)==set(ROLES) and all(dirty[r].get('position')==final_position(r,money_level) and dirty[r].get('xp')==(5 if r=='owner' else 0)
                and dirty[r].get('jp')==0 and dirty[r].get('mp')==50 and 0<dirty[r].get('health',0)<=100
                and dirty[r]==dict(cp.get(r,{}),position=final_position(r,money_level),xp=5 if r=='owner' else 0,health=dirty[r].get('health')) for r in ROLES)
            and any(v.get('health',100)<100 for v in dirty.values()))
        check('killed transactions disappear and both complete checkpoints survive',probe.get('transactions_gone') is True and probe.get('after')==cp
            and probe.get('released_unix_ms',0)>=kill.get('unix_ms',0))
        check('both observers agree on pre-save identities',active['owner'].get('own')==active['witness'].get('peer')
            and active['witness'].get('own')==active['owner'].get('peer') and active['owner'].get('monster')==active['witness'].get('monster')
            and active['owner'].get('monster',0)>0)
    else:
        check('both moving combatants are contemporaneous at the kill',set(active)==set(ROLES) and all(
            a.get('role')==r and a.get('moving') is True and a.get('peer_moving') is True and a.get('distance_from_checkpoint',0)>3
            and a.get('checkpoint')==checkpoint(r,money_level) and 0<=kill.get('unix_ms',0)-a.get('unix_ms',0)<=45
            and a.get('route_acks',0)>=2 and min(a.get('own_hits',0),a.get('peer_hits',0),a.get('retaliations',0))>=1
            and 0<a.get('monster_hp',0)<a.get('monster_max_hp',0)==105 and 0<a.get('own_hp',0)<=100 for r,a in active.items()))
        check('cross-client active identities and real retaliation agree',set(active)==set(ROLES) and active['owner'].get('own')==active['witness'].get('peer')
            and active['witness'].get('own')==active['owner'].get('peer') and active['owner'].get('monster')==active['witness'].get('monster')
            and any(a.get('own_hp',100)<100 for a in active.values()))
    check('saved post-restart progress is private and singular',set(saved)==set(ROLES) and all(saved[r].get('position')==final_position(r,money_level)
        and [saved[r].get(k) for k in ('level','xp','jp','money','mp','online')]==[3 if money_level and r=='owner' else 1,*progress(r),reward_saved.get(r,{}).get('money') if money_level else 0,mp(r),0]
        and 0<saved[r].get('health',0)<=maxhp(r) for r in ROLES) and (reward_durable or any(p.get('health',100)<100 for p in saved.values())))
    hit_sets={}
    for i,r in enumerate(ROLES):
        report=reports[r];other=ROLES[1-i];t=report.get('trace',{});logins=t.get('logins',[]);audits=report.get('audits',[])
        fs=[audit_facts(a) for a in audits];facts[r]=fs;hits=epochs(t,'hits','hit_orders');xp=epochs(t,'experience','experience_orders');hit_sets[r]=hits
        check(r+': native scenario completed',report.get('success') is True and not report.get('failure') and report.get('scenario')==('reward-durable' if reward_durable else 'save-rollback' if rollback else 'gameplay-restart') and report.get('role')==r)
        check(r+': three bounded distinct world workers all joined',len(fs)==3 and len({f['instance'] for f in fs})==3 and all(not f['failures'] for f in fs))
        check(r+': ordered traces align to three login epochs',len(logins)==3 and [l.get('order',0) for l in logins]==sorted({l.get('order',0) for l in logins})
            and all(l.get('name')==report.get('character') and l.get('layer')==0 for l in logins)
            and all(len(t.get(k,[]))==len(t.get(o,[])) for k,o in [('hits','hit_orders'),('experience','experience_orders'),('hpmp','vitals_orders')]))
        arrived=step(report,'CheckpointArrived');stored=step(report,'CheckpointSaved');joined=step(report,'Joined')
        check(r+': real movement precedes checkpoint save',joined.get('position')!=checkpoint(r,money_level) and arrived.get('position')==stored.get('position')==checkpoint(r,money_level)
            and stored.get('elapsed_ms',0)>=arrived.get('elapsed_ms',0) and any(m.get('handle')==logins[0].get('handle') and m.get('points')==[checkpoint(r,money_level)] for m in t.get('moves',[])) if len(logins)==3 else False)
        if prefight:
            before=step(report,'BeforeInterruptedSave')
            check(r+': live position and advertised reward precede interrupted save',before.get('position')==final_position(r,money_level)
                and before.get('xp_jp')==progress(r) and before.get('target',{}).get('hp')==0
                and before.get('motion',{}).get('moving') is False)
        else:
            check(r+': active circuit is acknowledged in the original world',len(logins)==3 and any(m.get('handle')==logins[0].get('handle') and len(m.get('points',[]))==16
                and m['points'][-1]==checkpoint(r,money_level) and step(report,'ActionsStarted').get('order',0)<m.get('order',0)<logins[1].get('order',0) for m in t.get('moves',[])))
        monster=active.get(r,{}).get('monster');own=logins[0].get('handle') if logins else None;peer=active.get(r,{}).get('peer')
        check(r+': both players dealt damage and received retaliation before crash',len(hits)==3 and all(any(h.get('attacker')==p and h.get('target')==monster and h.get('target_hp',0)>0 for h in hits[0]) for p in (own,peer))
            and any(h.get('attacker')==monster and h.get('damage',0)>0 for h in hits[0]) and (damage_chain(hits[0],monster)==0 if prefight else damage_chain(hits[0],monster)>0))
        dropped=step(report,'WorldDropped');held=step(report,'DowntimeHeld');restored=step(report,'CheckpointRestored');quiet=step(report,'NoReplay')
        check(r+': world dropped and stayed absent through downtime',dropped.get('player') is None and held.get('player') is None and held.get('stage_ms',0)>=4500
            and dropped.get('audit_instance')!=restored.get('audit_instance') and len(t.get('network_errors',[]))+len(t.get('disconnects',[]))==1)
        if reward_durable:
            profile=reward_saved.get(r,{})
            check(r+': second real login retains the already committed reward and vitals',len(logins)==3 and [logins[1].get(k) for k in ('x','y','hp','mp')]==[*checkpoint(r,money_level),profile.get('health'),mp(r)]
                and restored.get('position')==checkpoint(r,money_level) and restored.get('xp_jp')==progress(r) and restored.get('target') is None)
        else:
            check(r+': last saved position and vitals restored by real second login',len(logins)==3 and [logins[1].get(k) for k in ('x','y','hp','mp')]==[*checkpoint(r,money_level),100,50]
                and restored.get('position')==checkpoint(r,money_level) and restored.get('xp_jp')==[0,0] and restored.get('target') is None)
        check(r+': no stale movement or combat during the quiet recovery period',quiet.get('stage_ms',0)>=3200 and quiet.get('hits')==[] and quiet.get('moves')==[]
            and quiet.get('monster_hp')==105 and quiet.get('target') is None and quiet.get('motion',{}).get('moving') is False and quiet.get('position')==checkpoint(r,money_level)
            and step(report,'PostActionsStarted').get('hits')==[] and step(report,'PostActionsStarted').get('moves')==[])
        post=step(report,'PostActionsStarted');finished=step(report,'CombatRecovered')
        owner=reports['owner'].get('trace',{}).get('logins',[]);owner_handle=owner[1].get('handle') if len(owner)==3 else None
        if reward_durable:
            check(r+': no fresh combat or replay after committed-reward restart',len(hits)==3 and hits[1]==[] and step(report,'RewardRetained').get('hits')==[]
                and step(report,'RewardRetained').get('xp_jp')==progress(r))
        else:
            check(r+': fresh post-restart combat has one owner killing blow',len(hits)==3 and sum(h.get('target')==monster and h.get('target_hp')==0 for h in hits[1])==1
                and any(h.get('attacker')==owner_handle and h.get('target')==monster and h.get('target_hp')==0 for h in hits[1]) and damage_chain(hits[1],monster)==0)
        witness_logins=reports['witness'].get('trace',{}).get('logins',[]);witness_handle=witness_logins[1].get('handle') if len(witness_logins)==3 else None
        check(r+': new witness movement is authoritative and observed',any(m.get('handle')==witness_handle and m.get('points')==[final_position('witness',money_level)]
            and post.get('boundary',999999)<m.get('order',0)<(logins[2].get('order',0) if len(logins)==3 else 0) for m in t.get('moves',[])))
        if reward_durable:
            expected=progress(r)
            check(r+': one awarded reward survives all later logins without duplication or private leak',len(xp)==3 and len(logins)==3
                and sum([v.get('xp'),v.get('jp')]==progress('owner') for v in xp[0])==(1 if r=='owner' else 0)
                and all(v.get('handle')==logins[0].get('handle') and [v.get('xp'),v.get('jp')] in ([baseline(r)[1:3],progress(r)] if r=='owner' else [[0,0]]) for v in xp[0])
                and all(len(xp[i])==1 and [xp[i][0].get('xp'),xp[i][0].get('jp')]==expected and xp[i][0].get('handle')==logins[i].get('handle') for i in (1,2)))
        else:
            check(r+(': uncommitted reward rolled back and fresh reward stays private' if rollback else ': exactly one private reward and no pre-crash XP'),len(xp)==3 and (sum([v.get('xp'),v.get('jp')]==[5,3] for v in xp[0])==(1 if r=='owner' else 0) and all(v.get('handle')==logins[0].get('handle') and [v.get('xp'),v.get('jp')] in ([[0,0],[5,3]] if r=='owner' else [[0,0]]) for v in xp[0]) if rollback else all(v.get('xp')==v.get('jp')==0 for v in xp[0]))
                and sum(v.get('xp')==5 and v.get('jp')==3 for v in xp[1])==(1 if r=='owner' else 0)
                and all(v.get('handle')==logins[1].get('handle') for v in xp[1]) and all([v.get('xp'),v.get('jp')]==progress(r) for v in xp[2]))
        durable=step(report,'DurableReentry')
        check(r+': third login reads saved combat vitals and position',len(logins)==3 and r in saved and [logins[2].get(k) for k in ('x','y','hp','mp')]==[*final_position(r,money_level),saved[r].get('health'),mp(r)]
            and durable.get('xp_jp')==progress(r) and durable.get('position')==final_position(r,money_level) and durable.get('target') is None
            and durable.get('motion',{}).get('moving') is False and durable.get('hits')==durable.get('moves')==[] and len(hits)==3 and hits[2]==[])
        check(r+': real Lua lifecycle and gameplay data remain coherent',all(report.get('events',{}).get(k)==3 for k in ('PLAYER_LOGIN','PLAYER_ENTERING_WORLD','PLAYER_LOGOUT'))
            and all(s.get('player',{}).get('level') in ([2,3] if money_level and r=='owner' else [1]) and (s.get('money') in [1000,reward_saved.get(r,{}).get('money')] if money_level and r=='owner' else s.get('money')==(2000 if money_level else 0)) and s.get('layer')==0 and 0<s['player'].get('hp',0)<=maxhp(r) for s in report.get('steps',[]) if s.get('player'))
            and all(t.get(k)==[] for k in (('errors','warps','denied_attacks','cast_events','cast_results','learn_results') if money_level else ('errors','warps','levels','denied_attacks','cast_events','cast_results','learn_results'))))
        if money_level:
            gold=epochs(t,'gold','gold_orders');levels=epochs(t,'levels','level_orders');expected_money=reward_saved.get(r,{}).get('money')
            check(r+': exact private positive currency gain remains stable across all logins',len(gold)==3 and len(t.get('gold',[]))==len(t.get('gold_orders',[]))
                and [v.get('gold') for v in gold[0]]==([1000,expected_money] if r=='owner' else [2000])
                and all([v.get('gold') for v in gold[i]]==[expected_money] for i in (1,2))
                and report.get('events',{}).get('PLAYER_MONEY')==(4 if r=='owner' else 3)
                and (r!='owner' or t['gold_orders'][1]>step(report,'RewardWithheld').get('order',0))
                and all(s.get('money')==expected_money for s in (restored,quiet,step(report,'RewardRetained'),durable)))
            first_owner=reports['owner']['trace']['logins'][0]['handle']
            check(r+': both observers see one real level-up and no repeated level gain after restart',len(levels)==3 and len(t.get('levels',[]))==len(t.get('level_orders',[]))
                and levels[0]==[dict(handle=first_owner,level=3,job_level=1)] and levels[1:]==[[],[]]
                and report.get('events',{}).get('PLAYER_LEVEL_UP',0)==(1 if r=='owner' else 0)
                and (r!='owner' or t['level_orders'][0]>step(report,'RewardWithheld').get('order',0)))
            check(r+': restored and third-entry level-derived vitals agree with authoritative peer',len(logins)==3
                and all([l.get('max_hp'),l.get('max_mp')]==[maxhp(r),mp(r)] for l in logins[1:])
                and all(s.get('player',{}).get('level')==(3 if r=='owner' else 1) for s in (restored,durable))
                and all(s.get('peer_unit',{}).get('level')==(1 if r=='owner' else 3) and s['peer_unit'].get('max_hp')==(100 if r=='owner' else 140) and s['peer_unit'].get('max_mp')==(50 if r=='owner' else 70) for s in (restored,durable)))
        samples=report.get('samples',[])
        check(r+': terrain resources remain bounded across three worlds',0<len(samples)<16000 and all(s.get('cpu_pixel_limit')==134217728 and 0<=s.get('cpu_pixel_bytes',-1)<=134217728
            and s.get('bake_px')==1024 and s.get('worker_error') is None and s.get('async_enabled') is True and s.get('last_bakes')==0
            and 0<=s.get('resident',-1)<=s.get('wanted',-1)<=49 and s.get('texture_payload_bytes')==s.get('resident',-1)*4194304
            and 0<=s.get('last_restores',-1)+s.get('last_completed_uploads',-1)<=4 and s.get('completed',0)+s.get('cancelled',0)+int(s.get('job_active',False))==s.get('submitted',0) for s in samples))
        p=pictures.get(r,{})
        check(r+': recovered ground equals the saved checkpoint view',p.get('size')==[1280,720] and len(p.get('initial_sha256',''))==64 and p.get('initial_sha256')==p.get('recovered_sha256'))
        check(r+': Windows CPU GPU samples cover both holds',all(sum(m.get('role')==r and m.get('phase')==phase and m.get('process_id',0)>0 and m.get('Private Bytes',0)>0 and m.get('Total Committed',0)>0 for m in memory)>=3 for phase in ('initial','recovered')))
    check('both observers saw identical damage sequences in both server lifetimes',all(len(v)==3 for v in hit_sets.values()) and hit_sets['owner'][:2]==hit_sets['witness'][:2])
    assist=step(reports['witness'],'BeforeSaveAssistStopped' if reward_durable else 'AssistStopped')
    check('witness stops after three assisting hits while monster is alive',assist.get('monster_hp',0)>0 and sum(h.get('attacker')==assist.get('own') and h.get('target')==assist.get('monster') for h in assist.get('hits',[]))==3)
    if reward_durable:
        t=reports['owner'].get('trace',{});hs=epochs(t,'hits','hit_orders');logins=t.get('logins',[])
        check('owner lands exactly one killing blow before durable reward',len(hs)==3 and len(logins)==3
            and sum(h.get('target_hp')==0 and h.get('target')==active['owner'].get('monster') for h in hs[0])==1
            and any(h.get('target_hp')==0 and h.get('target')==active['owner'].get('monster') and h.get('attacker')==logins[0].get('handle') for h in hs[0]))
    if rollback:
        assist=step(reports['witness'],'BeforeSaveAssistStopped')
        check('pre-save witness stops assisting before owner receives uncommitted reward',assist.get('monster_hp',0)>0
            and sum(h.get('attacker')==assist.get('own') and h.get('target')==assist.get('monster') for h in assist.get('hits',[]))==3)
    if money_level:
        monster=database.get('monster_fixture',{})
        check('verified positive-gold monster fixture and explicit mode agree',server.get('money_level') is True and all(reports[r].get('money_level') is True for r in ROLES)
            and [monster.get(k) for k in ('monster_id','hp','exp','jp','gold_min','gold_max')]==[651,105,5,3,8,14]
            and monster.get('guid',0)>monster.get('preexisting_max',0)>0 and len(monster.get('dbc_sha256',''))==64)
        check('committed level-up includes EXP rollover full vitals and random gold within source bounds',reward_saved.get('owner',{}).get('health')==140
            and reward_saved['owner'].get('mp')==70 and 8<=reward_saved['owner'].get('money',0)-1000<=14
            and all(o.get('client',{}).get('level')==2 and o['client'].get('money')==1000 and o.get('dirty',{}).get('owner',{}).get('level')==3
                and o['dirty']['owner'].get('health')==140 and o['dirty']['owner'].get('mp')==70
                and 8<=o['dirty']['owner'].get('money',0)-1000<=14 for o in reward.get('blocked',[])))
    return dict(success=all(c['passed'] for c in checks),checks=checks,workers=facts,pictures=pictures)

def load_and_compare(root):
    root=Path(root);read=lambda n:json.loads((root/n).read_text(encoding='utf-8-sig'));pictures={}
    for r in ROLES:
        data={p:(root/f'{r}-{p}.ground.png').read_bytes() for p in ('initial','recovered')};sizes=[list(struct.unpack('>II',v[16:24])) if v[:8]==b'\x89PNG\r\n\x1a\n' else [] for v in data.values()]
        pictures[r]=dict(size=sizes[0] if sizes[0]==sizes[1] else [],**{p+'_sha256':hashlib.sha256(v).hexdigest() for p,v in data.items()})
    db={k:read(n) for k,n in [('checkpoint','checkpoint-database.json'),('down','down-profiles.json'),('saved','saved-database.json'),('restarted','restarted-database.json'),('request','checkpoint-requested.json')]}
    if (root/'reward-commit.json').exists():
        db['reward']=read('reward-commit.json')
        db['console_save_requests']=sum((root/f'world-{i}.log').read_text(encoding='utf-8').count('Sauvegarde de tous les joueurs demandée.') for i in (0,1))
    if (root/'monster-fixture.json').exists():db['monster_fixture']=read('monster-fixture.json')
    if (root/'save-rollback.json').exists():db['rollback']=read('save-rollback.json')
    return compare({r:read(r+'.json') for r in ROLES},read('server-process.json'),db,pictures,read('memory.json'))

if __name__=='__main__':
    root=Path(sys.argv[1]);r=load_and_compare(root);(root/'comparison.json').write_text(json.dumps(r,indent=2),encoding='utf-8')
    print(json.dumps(dict(success=r['success'],checks=len(r['checks']),failed=[c['name'] for c in r['checks'] if not c['passed']],workers=r['workers']),indent=2));raise SystemExit(0 if r['success'] else 1)
