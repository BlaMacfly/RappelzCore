"""Independent evidence checks for committed and interrupted rewards at DB process death."""
import hashlib
import json
from pathlib import Path
import struct
import sys
from compare_gameplay_restart_clients import epochs,step,damage_chain,checkpoint,final_position,ROLES
from compare_terrain_cancellation_clients import audit_facts


def compare(reports,control,db,pictures,memory,drops):
    checks=[];workers={}
    def check(name,value):checks.append(dict(name=name,passed=bool(value)))
    boots=control.get('boots',[]);dbboots=control.get('database_boots',[]);kill=control.get('database_kill',{});proof=db.get('reward',{})
    cp=db.get('checkpoint',{});committed=proof.get('committed',{});saved=db.get('saved',{});pending=proof.get('pending_reward',[]);window=proof.get('pending_window',{})
    check('owned database is force-killed and restarted on the identical persistent directory',control.get('success') is True and control.get('scenario')=='database-reward'
        and not control.get('cleanup_kill') and not control.get('cleanup_database_kill') and not control.get('kill')
        and len(dbboots)==2 and dbboots[0].get('pid')==kill.get('pid')!=dbboots[1].get('pid') and dbboots[0].get('data')==dbboots[1].get('data')
        and kill.get('method')=='owned database Popen.kill' and kill.get('graceful') is False and kill.get('exit_code') not in (0,None))
    check('world stays alive in the same process throughout database failure',len(boots)==1 and kill.get('world_pid')==boots[0].get('pid')==control.get('world_pid_after_database_restart')
        and control.get('world_alive_after_database_restart') is True and control.get('final_exit_code')==0 and control.get('database_handed_off') is True)
    check('durable normal InnoDB operation and actual crash recovery are recorded',len(dbboots)==2 and all(b.get('durability') in ('1\t0\tON','1\t0\t1') for b in dbboots)
        and 'crash recovery' in control.get('recovery_log','').lower())
    check('database outage lasts at least five seconds and both real connections close before restart',len(dbboots)==2 and dbboots[1].get('unix_ms',0)-kill.get('unix_ms',0)>=5000
        and all(drops.get(r,{}).get('joined') is True and kill.get('unix_ms',0)<=drops[r].get('unix_ms',0)<=dbboots[1].get('unix_ms',0) for r in ROLES))
    check('only the pre-combat console checkpoint save was requested',db.get('console_save_requests')==1 and len(boots)==1 and db.get('request',{}).get('pid')==boots[0].get('pid'))
    check('two dedicated healthy checkpoints have distinct original wallets',set(cp)==set(ROLES) and all([cp[r].get(k) for k in ('level','xp','jp','money','health','mp','online')]
        ==([2,799,7,1000,120,60,1] if r=='owner' else [1,0,0,2000,100,50,1]) and cp[r].get('position')==checkpoint(r,True) for r in ROLES))
    first=proof.get('blocked',[])
    check('first complete reward is withheld while its transaction waits at settings',len(first)>=2 and proof.get('held_ms',0)>=500
        and all(v.get('committed')==cp and v.get('client',{}).get('xp_jp')==[799,7] and v['client'].get('level')==2 and v['client'].get('money')==1000
            and len(v.get('waits',[]))==1 and v['waits'][0].get('modified',0)>0 for v in first))
    check('positive first currency and level-up are completely committed before the database crash',set(committed)==set(ROLES) and committed.get('witness')==cp.get('witness')
        and [committed['owner'].get(k) for k in ('level','xp','jp','health','mp')]==[3,4,10,140,70] and 8<=committed['owner'].get('money',0)-1000<=14
        and control.get('first_reward',{}).get('profiles')==committed and control['first_reward'].get('unix_ms',0)<kill.get('unix_ms',0))
    check('second reward has a row-modified transaction but no announced EXP JP level or currency gain',len(pending)>=2 and pending[-1].get('unix_ms',0)-pending[0].get('unix_ms',0)>=500
        and all(v.get('committed')==committed and v.get('dirty',{}).get('witness')==committed.get('witness')
            and [v['dirty']['owner'].get(k) for k in ('level','xp','jp','mp')]==[3,9,10,70] and 8<=v['dirty']['owner'].get('money',0)-committed['owner']['money']<=14
            and v.get('client',{}).get('xp_jp')==[4,10] and v['client'].get('level')==3 and v['client'].get('money')==committed['owner']['money']
            and len(v.get('waits',[]))==1 and v['waits'][0].get('modified',0)>0 and 'replace into character_settings' in v['waits'][0].get('query','').lower() for v in pending))
    check('database death occurs inside the verified locked second-save window',window==kill.get('pending_reward') and window.get('locks_held') is True and len(boots)==1
        and window.get('pid')==boots[0].get('pid') and 0<=kill.get('unix_ms',0)-window.get('unix_ms',0)<=1500
        and len(pending)>=2 and window.get('waiting_transactions')==[w['transaction'] for w in pending[-1]['waits']])
    check('crash recovery retains committed award and rolls back the incomplete second award',proof.get('success') is True and proof.get('database_restart') is True
        and proof.get('after')==committed==db.get('restored') and proof.get('transactions_gone') is True)
    check('final normal saves retain the same award and only new witness movement',set(saved)==set(ROLES) and all(saved[r]==dict(committed[r],online=0,position=final_position(r,True)) for r in ROLES))
    monster=db.get('monster_fixture',{})
    check('positive-gold fixture uses the unchanged verified monster definition', [monster.get(k) for k in ('monster_id','hp','exp','jp','gold_min','gold_max')]==[651,105,5,3,8,14]
        and monster.get('guid',0)>monster.get('preexisting_max',0)>0 and len(monster.get('dbc_sha256',''))==64)
    fights={}
    for r in ROLES:
        report=reports[r];t=report.get('trace',{});logins=t.get('logins',[]);hit=epochs(t,'hits','hit_orders');xp=epochs(t,'experience','experience_orders');gold=epochs(t,'gold','gold_orders');lv=epochs(t,'levels','level_orders')
        fs=[audit_facts(a) for a in report.get('audits',[])];workers[r]=fs;balance=committed.get(r,{}).get('money');owner=r=='owner';expected=[4,10] if owner else [0,0]
        check(r+': real three-entry database scenario completes',report.get('success') is True and not report.get('failure') and report.get('database_restart') is True
            and report.get('money_level') is True and len(logins)==3 and all(report.get('events',{}).get(k)==3 for k in ['PLAYER_LOGIN','PLAYER_ENTERING_WORLD','PLAYER_LOGOUT']))
        check(r+': three distinct bounded world workers all join',len(fs)==3 and len({f['instance'] for f in fs})==3 and all(not f['failures'] for f in fs))
        check(r+': login epochs and parallel packet histories align',len(logins)==3 and [l.get('order') for l in logins]==sorted({l.get('order') for l in logins})
            and logins[0].get('handle')!=logins[1].get('handle') and all(len(t.get(k,[]))==len(t.get(o,[])) for k,o in [('hits','hit_orders'),('experience','experience_orders'),('gold','gold_orders'),('levels','level_orders'),('hpmp','vitals_orders')]))
        arrived=step(report,'CheckpointArrived');stored=step(report,'CheckpointSaved');joined=step(report,'Joined')
        check(r+': real acknowledged movement precedes the independently saved checkpoint',joined.get('position')!=checkpoint(r,True)
            and arrived.get('position')==stored.get('position')==checkpoint(r,True) and stored.get('elapsed_ms',0)>=arrived.get('elapsed_ms',0)
            and len(logins)==3 and any(v.get('handle')==logins[0].get('handle') and v.get('points')==[checkpoint(r,True)] and logins[0].get('order',0)<v.get('order',0)<logins[1].get('order',0) for v in t.get('moves',[])))
        started=step(report,'SecondActionsStarted');boundary=started.get('fight_boundary',0);mh=started.get('monster')
        a=[h for h,o in zip(t.get('hits',[]),t.get('hit_orders',[])) if o<boundary];b=[h for h,o in zip(t.get('hits',[]),t.get('hit_orders',[])) if boundary<o<(logins[1].get('order',0) if len(logins)==3 else 0)];fights[r]=[a,b]
        own0=reports['owner'].get('trace',{}).get('logins',[{}])[0].get('handle')
        check(r+': both real kills belong to the owner with exact damage chains',boundary>0 and all(damage_chain(v,mh)==0 and sum(h.get('target')==mh and h.get('target_hp')==0 for h in v)==1
            and any(h.get('target')==mh and h.get('target_hp')==0 and h.get('attacker')==own0 for h in v) for v in [a,b]))
        wait=step(report,'SecondRewardWithheld') if owner else step(report,'SecondKillObserved')
        check(r+': second kill is observed without a second private reward',wait.get('xp_jp')==expected and wait.get('money')==balance and len(hit)==3 and hit[1:]==[[],[]])
        dropped=step(report,'WorldDropped');held=step(report,'DowntimeHeld')
        check(r+': failed connection drops and joins before real reauthentication',dropped.get('player') is None and held.get('player') is None
            and len(t.get('network_errors',[]))+len(t.get('disconnects',[]))==1 and len(logins)==3 and dropped.get('audit_instance')!=step(report,'CheckpointRestored').get('audit_instance'))
        check(r+': one first award and no uncommitted or duplicate EXP JP update',len(xp)==3 and len(logins)==3
            and [[v.get('xp'),v.get('jp')] for v in xp[0]]==([[799,7],[4,10]] if owner else [[0,0]])
            and all([[v.get('xp'),v.get('jp')] for v in xp[i]]==[expected] for i in (1,2)))
        check(r+': exact private currency survives database restart and normal re-entry',len(gold)==3 and [v.get('gold') for v in gold[0]]==([1000,balance] if owner else [2000])
            and all([v.get('gold') for v in gold[i]]==[balance] for i in (1,2)) and report.get('events',{}).get('PLAYER_MONEY')==(4 if owner else 3))
        check(r+': one synchronized level-up and no repeated Lua level gain',len(lv)==3 and lv[0]==[dict(handle=own0,level=3,job_level=1)] and lv[1:]==[[],[]]
            and report.get('events',{}).get('PLAYER_LEVEL_UP',0)==(1 if owner else 0))
        restored=step(report,'CheckpointRestored');durable=step(report,'DurableReentry');profile=committed.get(r,{})
        check(r+': second and third raw login load committed level-derived vitals',len(logins)==3 and all([l.get(k) for k in ('hp','mp','max_hp','max_mp')]==[profile.get('health'),profile.get('mp'),140 if owner else 100,70 if owner else 50] for l in logins[1:])
            and all(s.get('player',{}).get('level')==(3 if owner else 1) and s.get('xp_jp')==expected and s.get('money')==balance for s in [restored,durable]))
        check(r+': peer level and maximum vitals remain authoritative after recovery',all(s.get('peer_unit',{}).get('level')==(1 if owner else 3)
            and s['peer_unit'].get('max_hp')==(100 if owner else 140) and s['peer_unit'].get('max_mp')==(50 if owner else 70) for s in [restored,durable]))
        quiet=step(report,'NoReplay');post=step(report,'PostActionsStarted')
        check(r+': no stale target combat movement or reward after database recovery',quiet.get('stage_ms',0)>=3200 and quiet.get('hits')==quiet.get('moves')==[] and quiet.get('target') is None
            and quiet.get('monster_hp')==105 and quiet.get('position')==checkpoint(r,True) and quiet.get('motion',{}).get('moving') is False and post.get('hits')==post.get('moves')==[] and durable.get('hits')==durable.get('moves')==[])
        check(r+': actual new movement follows both recovery observations',len(logins)==3 and any(v.get('handle')==reports['witness']['trace']['logins'][1]['handle'] and v.get('points')==[final_position('witness',True)] and post.get('boundary',999999)<v.get('order',0)<logins[2].get('order',0) for v in t.get('moves',[])))
        check(r+': no decoding addon combat denial or warp failure',all(t.get(k)==[] for k in ['errors','warps','denied_attacks','cast_events','cast_results','learn_results']))
        samples=report.get('samples',[])
        check(r+': terrain memory and upload work remain bounded',0<len(samples)<16000 and all(v.get('worker_error') is None and v.get('async_enabled') is True and v.get('bake_px')==1024
            and 0<=v.get('cpu_pixel_bytes',-1)<=v.get('cpu_pixel_limit',-1)==134217728 and v.get('texture_payload_bytes')==v.get('resident',-1)*4194304
            and 0<=v.get('resident',-1)<=v.get('wanted',-1)<=49 and v.get('last_bakes')==0 and 0<=v.get('last_restores',-1)+v.get('last_completed_uploads',-1)<=4 for v in samples))
        picture=pictures.get(r,{})
        check(r+': recovered checkpoint ground stays byte-identical',picture.get('size')==[1280,720] and len(picture.get('initial_sha256',''))==64 and picture.get('initial_sha256')==picture.get('recovered_sha256'))
        check(r+': real Windows CPU GPU counters cover both settled holds',all(sum(v.get('role')==r and v.get('phase')==phase and v.get('Total Committed',0)>0 and v.get('Private Bytes',0)>0 for v in memory)>=3 for phase in ['initial','recovered']))
    check('both observers receive identical first and second damage sequences',fights.get('owner')==fights.get('witness'))
    check('witness exercises a failing real save while the database is down',bool(step(reports['witness'],'SaveDuringDatabaseOutage')))
    return dict(success=all(c['passed'] for c in checks),checks=checks,workers=workers,pictures=pictures)


def load_and_compare(root):
    root=Path(root);read=lambda n:json.loads((root/n).read_text(encoding='utf-8-sig'));pictures={}
    for r in ROLES:
        data={p:(root/f'{r}-{p}.ground.png').read_bytes() for p in ['initial','recovered']}
        sizes=[list(struct.unpack('>II',v[16:24])) if v[:8]==b'\x89PNG\r\n\x1a\n' else [] for v in data.values()]
        pictures[r]=dict(size=sizes[0] if sizes[0]==sizes[1] else [],**{p+'_sha256':hashlib.sha256(v).hexdigest() for p,v in data.items()})
    db={k:read(n) for k,n in [('reward','reward-commit.json'),('restored','database-reward-restored.json'),('checkpoint','checkpoint-database.json'),('saved','saved-database.json'),('request','checkpoint-requested.json'),('monster_fixture','monster-fixture.json')]}
    db['console_save_requests']=(root/'world-0.log').read_text(encoding='utf-8').count('Sauvegarde de tous les joueurs demandée.')
    return compare({r:read(r+'.json') for r in ROLES},read('server-process.json'),db,pictures,read('memory.json'),{r:read(r+'.dropped') for r in ROLES})

if __name__=='__main__':
    root=Path(sys.argv[1]);value=load_and_compare(root);(root/'comparison.json').write_text(json.dumps(value,indent=2),encoding='utf-8')
    print(json.dumps(dict(success=value['success'],checks=len(value['checks']),failed=[v['name'] for v in value['checks'] if not v['passed']]),indent=2));raise SystemExit(0 if value['success'] else 1)
