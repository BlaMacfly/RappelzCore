"""Verify a real successful COMMIT whose database OK never reaches the game server."""
import hashlib
import json
from pathlib import Path
import struct
import sys
from compare_gameplay_restart_clients import epochs,step,damage_chain,checkpoint,final_position,ROLES
from compare_terrain_cancellation_clients import audit_facts


def compare(reports,control,db,pictures,memory,drops,database):
    checks=[];workers={}
    def check(name,value):checks.append(dict(name=name,passed=bool(value)))
    proof=db.get('proof',{});before=proof.get('before',{});committed=proof.get('committed',{});reentry=proof.get('reentry_profiles',{});saved=db.get('saved',{})
    arm=proof.get('arm',{});held=proof.get('held',{});reset=proof.get('reset',{});observations=proof.get('unacknowledged',[]);boots=control.get('boots',[]);proxy=control.get('proxy',{});events=proxy.get('events',[])
    check('same copied world process survives loss of SQL confirmation',control.get('success') is True and control.get('scenario')=='ambiguous-commit' and not control.get('cleanup_kill')
        and len(boots)==1 and control.get('world_alive_after_sql_loss') is True and control.get('world_pid_after_sql_loss')==boots[0].get('pid') and control.get('final_exit_code')==0)
    check('isolated database stays alive in the original process',database.get('before_pid',0)>0 and database.get('before_pid')==database.get('after_pid') and database.get('port')==3331
        and len(boots)==1 and database.get('created_unix_ms',10**20)<boots[0].get('unix_ms',0))
    check('single pre-combat console checkpoint save is independent of the reward',db.get('console_save_requests')==1 and len(boots)==1 and db.get('request',{}).get('pid')==boots[0].get('pid'))
    check('two exact healthy fixture checkpoints have different private wallets',set(before)==set(ROLES) and all(before[r].get('position')==checkpoint(r,True)
        and [before[r].get(k) for k in ['level','xp','jp','money','health','mp','online']]==([2,799,7,1000,120,60,1] if r=='owner' else [1,0,0,2000,100,50,1]) for r in ROLES))
    blocked=proof.get('blocked',{});waits=blocked.get('waits',[])
    check('selected connection belongs to a real row-modified waiting reward transaction',len(waits)==1 and waits[0].get('connection')==arm.get('connection_id') and waits[0].get('transaction')==arm.get('transaction')
        and waits[0].get('modified',0)>0 and 'replace into character_settings' in waits[0].get('query','').lower() and blocked.get('committed')==before
        and blocked.get('client',{}).get('xp_jp')==[799,7] and blocked['client'].get('money')==1000 and blocked['client'].get('level')==2)
    selected=[e for e in events if e.get('event')=='commit_forwarded'];held_events=[e for e in events if e.get('event')=='commit_ok_held'];closed=[e for e in events if e.get('event')=='commit_ok_discarded_connection_closed']
    check('one exact real COMMIT and one successful OK are correlated to the armed SQL connection',len(selected)==len(held_events)==len(closed)==1 and held_events[0]==held==proxy.get('held')
        and closed[0]==reset and selected[0].get('connection_id')==held.get('connection_id')==reset.get('connection_id')==arm.get('connection_id')
        and selected[0].get('sequence')==0 and held.get('sequence')==1 and held.get('status',1)&1==0 and held.get('packet_bytes')==11 and len(held.get('packet_sha256',''))==64)
    check('actual COMMIT OK is held then discarded without forwarding or replacement',proxy.get('faulted') is True and held.get('forwarded') is False and reset.get('forwarded') is False
        and held.get('commit_unix_ms',0)<=held.get('unix_ms',0)<reset.get('unix_ms',0) and arm.get('unix_ms',0)<=held.get('commit_unix_ms',0))
    check('SQL relay stays bounded and records no unexpected protocol error',proxy.get('overflow')==0 and proxy.get('active_connections')==0 and proxy.get('packet_limit')==1048576 and proxy.get('connection_limit')==16
        and not any(e.get('event')=='error' for e in events) and all(e.get('tls_advertised_after') is False for e in events if e.get('event')=='connected'))
    check('successful complete reward is independently readable before acknowledgement loss',set(committed)==set(ROLES) and committed.get('witness')==before.get('witness')
        and [committed['owner'].get(k) for k in ['level','xp','jp','health','mp']]==[3,4,10,140,70] and 8<=committed['owner'].get('money',0)-1000<=14
        and len(observations)>=2 and observations[-1].get('unix_ms',0)-observations[0].get('unix_ms',0)>=500
        and all(o.get('profiles')==committed and o.get('client',{}).get('xp_jp')==[799,7] and o['client'].get('level')==2 and o['client'].get('money')==1000
            and held.get('unix_ms',0)<=o.get('unix_ms',0)<reset.get('unix_ms',0) and 0<=o.get('unix_ms',0)-o['client'].get('unix_ms',0)<=150 for o in observations))
    check('failed COMMIT result and ordinary peer save never overwrite the durable owner',proof.get('success') is True and reentry.get('owner')==committed.get('owner')
        and reentry.get('witness',{}).get('position')==before.get('witness',{}).get('position') and [reentry['witness'].get(k) for k in ['level','xp','jp','money','mp','online']]==[1,0,0,2000,50,0]
        and 0<reentry['witness'].get('health',0)<=100 and reentry['witness'].get('settings')==before['witness'].get('settings'))
    check('reentry waits for natural monster respawn after both worlds are closed',proof.get('verified_unix_ms',0)-held.get('unix_ms',0)>=17000
        and all(drops.get(r,{}).get('joined') is True and reset.get('unix_ms',0)<=drops[r].get('unix_ms',0)<=proof.get('verified_unix_ms',0) for r in ROLES)
        and drops.get('witness',{}).get('normal_lobby') is True)
    check('later normal SQL saves retain private reward and only expected movement or regeneration',set(saved)==set(ROLES) and saved.get('owner')==dict(committed.get('owner',{}),online=0)
        and all(saved['witness'].get(k)==reentry.get('witness',{}).get(k) for k in ['level','xp','jp','money','mp','settings']) and saved['witness'].get('position')==final_position('witness',True)
        and reentry['witness'].get('health',0)<=saved['witness'].get('health',0)<=100 and saved['witness'].get('online')==0)
    fights={}
    for r in ROLES:
        report=reports[r];t=report.get('trace',{});ls=t.get('logins',[]);hits=epochs(t,'hits','hit_orders');xp=epochs(t,'experience','experience_orders');gold=epochs(t,'gold','gold_orders');lv=epochs(t,'levels','level_orders')
        fs=[audit_facts(a) for a in report.get('audits',[])];workers[r]=fs;owner=r=='owner';balance=committed.get(r,{}).get('money');expected=[4,10] if owner else [0,0]
        check(r+': real three-entry ambiguous COMMIT driver completes',report.get('success') is True and not report.get('failure') and report.get('ambiguous_commit') is True and report.get('money_level') is True
            and len(ls)==3 and all(report.get('events',{}).get(k)==3 for k in ['PLAYER_LOGIN','PLAYER_ENTERING_WORLD','PLAYER_LOGOUT']))
        check(r+': three distinct causal terrain workers all join',len(fs)==3 and len({f['instance'] for f in fs})==3 and all(not f['failures'] for f in fs))
        check(r+': new login identities and parallel packet histories stay aligned',len(ls)==3 and len({l.get('handle') for l in ls})==3 and [l.get('order') for l in ls]==sorted({l.get('order') for l in ls})
            and all(len(t.get(k,[]))==len(t.get(o,[])) for k,o in [('hits','hit_orders'),('experience','experience_orders'),('gold','gold_orders'),('levels','level_orders'),('hpmp','vitals_orders')]))
        arrived=step(report,'CheckpointArrived');stored=step(report,'CheckpointSaved')
        check(r+': real movement is acknowledged before checkpoint capture',len(ls)==3 and arrived.get('position')==stored.get('position')==checkpoint(r,True)
            and any(v.get('handle')==ls[0].get('handle') and v.get('points')==[checkpoint(r,True)] and ls[0].get('order',0)<v.get('order',0)<ls[1].get('order',0) for v in t.get('moves',[])))
        mh=step(report,'CommitResponseLost').get('monster');own=reports['owner'].get('trace',{}).get('logins',[{}])[0].get('handle');fights[r]=hits
        check(r+': one real owner killing blow and zero replayed combat after reconnect',len(hits)==3 and damage_chain(hits[0],mh)==0 and hits[1:]==[[],[]]
            and sum(h.get('target')==mh and h.get('target_hp')==0 for h in hits[0])==1 and any(h.get('target')==mh and h.get('target_hp')==0 and h.get('attacker')==own for h in hits[0]))
        check(r+': no reward update before confirmation loss and exactly one initial state in later worlds',len(xp)==3 and [[v.get('xp'),v.get('jp')] for v in xp[0]]==([[799,7]] if owner else [[0,0]])
            and all([[v.get('xp'),v.get('jp')] for v in xp[i]]==[expected] for i in (1,2)))
        check(r+': durable currency is discovered on reentry without duplicate or private gain',len(gold)==3 and [v.get('gold') for v in gold[0]]==([1000] if owner else [2000])
            and all([v.get('gold') for v in gold[i]]==[balance] for i in (1,2)) and report.get('events',{}).get('PLAYER_MONEY')==3)
        check(r+': no unacknowledged level-up packet or spurious Lua gain is published',len(lv)==3 and lv==[[],[],[]] and report.get('events',{}).get('PLAYER_LEVEL_UP',0)==0)
        restored=step(report,'CheckpointRestored');durable=step(report,'DurableReentry');profile=reentry.get(r,{})
        check(r+': second and third raw logins read actual saved vitals and level-derived maxima',len(ls)==3 and [ls[1].get(k) for k in ['hp','mp','max_hp','max_mp']]==[profile.get('health'),profile.get('mp'),140 if owner else 100,70 if owner else 50]
            and [ls[2].get(k) for k in ['hp','mp','max_hp','max_mp']]==[saved.get(r,{}).get('health'),profile.get('mp'),140 if owner else 100,70 if owner else 50]
            and all(s.get('player',{}).get('level')==(3 if owner else 1) and s.get('xp_jp')==expected and s.get('money')==balance for s in [restored,durable]))
        check(r+': returning peer sees durable level and maximum vitals through real ENTER',all(s.get('peer_unit',{}).get('level')==(1 if owner else 3)
            and s['peer_unit'].get('max_hp')==(100 if owner else 140) and s['peer_unit'].get('max_mp')==(50 if owner else 70) for s in [restored,durable]))
        quiet=step(report,'NoReplay');post=step(report,'PostActionsStarted')
        check(r+': quiet recovery contains no stale target movement combat or reward replay',quiet.get('stage_ms',0)>=3200 and quiet.get('hits')==quiet.get('moves')==[] and quiet.get('target') is None
            and quiet.get('monster_hp')==105 and quiet.get('position')==checkpoint(r,True) and post.get('hits')==post.get('moves')==[] and durable.get('hits')==durable.get('moves')==[])
        check(r+': new witness movement starts after both recovery observations',len(ls)==3 and any(v.get('handle')==reports['witness']['trace']['logins'][1]['handle'] and v.get('points')==[final_position('witness',True)]
            and post.get('boundary',999999)<v.get('order',0)<ls[2].get('order',0) for v in t.get('moves',[])))
        count=len(t.get('network_errors',[]))+len(t.get('disconnects',[]))
        check(r+': only the uncertain owner loses transport while witness saves on its live connection',count==(1 if owner else 0)
            and (step(report,'WorldDropped').get('player') is None if owner else bool(step(report,'PeerLobbySaved'))))
        check(r+': no decoding attack denial or other gameplay error',all(t.get(k)==[] for k in ['errors','warps','denied_attacks','cast_events','cast_results','learn_results']))
        samples=report.get('samples',[])
        check(r+': terrain work and memory remain bounded',0<len(samples)<16000 and all(v.get('worker_error') is None and v.get('async_enabled') is True and v.get('bake_px')==1024
            and 0<=v.get('cpu_pixel_bytes',-1)<=v.get('cpu_pixel_limit',-1)==134217728 and v.get('texture_payload_bytes')==v.get('resident',-1)*4194304
            and 0<=v.get('resident',-1)<=v.get('wanted',-1)<=49 and v.get('last_bakes')==0 and 0<=v.get('last_restores',-1)+v.get('last_completed_uploads',-1)<=4 for v in samples))
        picture=pictures.get(r,{})
        check(r+': recovered checkpoint ground is byte-identical',picture.get('size')==[1280,720] and len(picture.get('initial_sha256',''))==64 and picture.get('initial_sha256')==picture.get('recovered_sha256'))
        check(r+': real Windows CPU GPU counters cover both holds',all(sum(v.get('role')==r and v.get('phase')==p and v.get('Private Bytes',0)>0 and v.get('Total Committed',0)>0 for v in memory)>=3 for p in ['initial','recovered']))
    check('both observers receive the same actual damage sequence',fights.get('owner')==fights.get('witness'))
    return dict(success=all(c['passed'] for c in checks),checks=checks,workers=workers,pictures=pictures)


def load_and_compare(root):
    root=Path(root);read=lambda n:json.loads((root/n).read_text(encoding='utf-8-sig'));pictures={}
    for r in ROLES:
        data={p:(root/f'{r}-{p}.ground.png').read_bytes() for p in ['initial','recovered']};sizes=[list(struct.unpack('>II',v[16:24])) if v[:8]==b'\x89PNG\r\n\x1a\n' else [] for v in data.values()]
        pictures[r]=dict(size=sizes[0] if sizes[0]==sizes[1] else [],**{p+'_sha256':hashlib.sha256(v).hexdigest() for p,v in data.items()})
    db=dict(proof=read('ambiguous-commit.json'),saved=read('saved-database.json'),request=read('checkpoint-requested.json'),console_save_requests=(root/'world-0.log').read_text(encoding='utf-8').count('Sauvegarde de tous les joueurs demandée.'))
    return compare({r:read(r+'.json') for r in ROLES},read('server-process.json'),db,pictures,read('memory.json'),{r:read(r+'.dropped') for r in ROLES},read('database-process-preserved.json'))

if __name__=='__main__':
    root=Path(sys.argv[1]);v=load_and_compare(root);(root/'comparison.json').write_text(json.dumps(v,indent=2),encoding='utf-8');print(json.dumps(dict(success=v['success'],checks=len(v['checks']),failed=[c['name'] for c in v['checks'] if not c['passed']]),indent=2));raise SystemExit(0 if v['success'] else 1)
