"""Check process death/reboot, database recovery and fresh terrain worlds with reused handles."""
import hashlib
import json
from pathlib import Path
import struct
import sys
from compare_terrain_cancellation_clients import audit_facts

ROLES=('owner','witness')

def compare(reports,server,database,pictures,memory):
    checks=[];facts={}
    def check(name,value):checks.append(dict(name=name,passed=bool(value)))
    boots=server.get('boots',[]);kill=server.get('kill',{})
    check('owned isolated server stopped forcibly and restarted normally',server.get('success') is True and not server.get('cleanup_kill')
        and len(boots)==2 and [b.get('generation') for b in boots]==[0,1] and all(b.get('pid',0)>0 and b.get('port')!=4514 for b in boots)
        and boots[0]['pid']!=boots[1]['pid'] and kill.get('pid')==boots[0]['pid'] and kill.get('method')=='owned Popen.kill'
        and kill.get('graceful') is False and kill.get('exit_code') not in (None,0) and server.get('final_exit_code')==0)
    check('server remained down at least five seconds',len(boots)==2 and boots[0].get('unix_ms',0)<kill.get('unix_ms',0)
        and boots[1].get('unix_ms',0)-kill.get('unix_ms',0)>=5000)
    duplicate=database.get('duplicate',{});before=duplicate.get('before',{});after=duplicate.get('after',{})
    check('failed duplicate bind preserves live session flags and population',duplicate.get('exit_code') not in (None,0) and before==after
        and all(before.get(r)==dict(account=1,character=1) for r in ROLES) and before.get('population')==.375)
    down=database.get('down',{});restarted=database.get('restarted',{})
    check('force kill leaves unsaved online flags until restart',all(down.get(r)==dict(account=1,character=1) for r in ROLES))
    check('successful restart clears current realm accounts and character flags',all(restarted.get(r)==dict(account=0,character=0) for r in ROLES)
        and restarted.get('same_realm')==0)
    check('other realm account survives all startup recovery',all(v.get('other_realm')==2 for v in (before,after,down,restarted)))
    for index,role in enumerate(ROLES):
        other=ROLES[1-index];r=reports[role];peer=reports[other];steps=r.get('steps',[]);trace=r.get('trace',{});audits=r.get('audits',[])
        fs=[audit_facts(a) for a in audits];facts[role]=fs
        check(role+': native restart scenario completed',r.get('scenario')=='terrain-restart' and r.get('success') is True and not r.get('failure') and r.get('role')==role)
        check(role+': two distinct bounded causal joined world workers',len(fs)==2 and len({f['instance'] for f in fs})==2 and all(not f['failures'] for f in fs))
        chosen=kill.get('jobs',{}).get(role,{})
        check(role+': process kill bound to a fresh positive partial job',chosen.get('role')==role and chosen.get('running') is True and 0<chosen.get('rows',0)<256
            and 0<=kill.get('unix_ms',0)-chosen.get('unix_ms',0)<=12 and len(fs)==2 and chosen.get('instance')==fs[0]['instance']
            and chosen.get('id') in fs[0]['partial_shutdown_ids'])
        killed=[s for s in steps if s.get('stage')=='ServerKilled'];joined=[s for s in steps if s.get('stage')=='WorkerJoined'];held=[s for s in steps if s.get('stage')=='DowntimeHeld'];reentry=[s for s in steps if s.get('stage')=='Reentered']
        check(role+': exact selected job cancelled and joined before recovery',len(killed)==len(joined)==len(reentry)==1
            and killed[0].get('job')==joined[0].get('job')==chosen.get('id') and joined[0].get('closing_instance')==chosen.get('instance')
            and joined[0].get('elapsed_ms',999999)<reentry[0].get('elapsed_ms',0) and joined[0].get('player') is None and joined[0].get('peer') is None
            and joined[0].get('audit_instance')!=reentry[0].get('audit_instance'))
        check(role+': no phantom world during server downtime',len(held)==1 and held[0].get('player') is None and held[0].get('peer') is None
            and held[0].get('stage_ms',0)>=4500 and held[0].get('events',{}).get('PLAYER_LOGOUT')==1)
        check(role+': one terminal transport event from actual server death',len(trace.get('network_errors',[]))+len(trace.get('disconnects',[]))==1)
        logins=trace.get('logins',[]);pl=peer.get('trace',{}).get('logins',[])
        check(role+': two authoritative ordered logins independent of numeric handle reuse',len(logins)==2 and logins[0].get('order',0)>0
            and logins[1].get('order',0)>logins[0]['order'] and all(l.get('name')==r.get('character') and l.get('layer')==0 for l in logins)
            and len(reentry)==1 and reentry[0].get('own')==logins[1].get('handle'))
        check(role+': Lua login entry and logout each occur twice',all(r.get('events',{}).get(k)==2 for k in ('PLAYER_LOGIN','PLAYER_ENTERING_WORLD','PLAYER_LOGOUT')))
        y,py=(7227,7247) if role=='owner' else (7247,7227)
        check(role+': both peer appearances belong to the correct new session',len(logins)==len(pl)==2 and all(any(e.get('name')==peer.get('character') and e.get('handle')==pl[i].get('handle')
            and e.get('order',0)>logins[i].get('order',0) and (i==1 or e['order']<logins[1].get('order',0))
            and [e.get(k) for k in ('kind','x','y','layer','hp','mp','level')]==['player',8440,py,0,100,50,1] for e in trace.get('enters',[])) for i in range(2)))
        pair=[s for s in steps if s.get('stage')=='PairReunited']
        check(role+': peer correctly rebound in the replacement world',len(pair)==1 and len(pl)==2 and pair[0].get('peer')==pl[1].get('handle') and pair[0].get('audit_instance')==fs[-1]['instance'])
        world_steps=[s for s in steps if s.get('player')]
        check(role+': stats position money and layer unchanged',bool(world_steps) and all(s.get('position')==[8440,y] and [s['player'].get(k) for k in ('name','level','hp','mp')]==[r.get('character'),1,100,50]
            and s.get('xp_jp')==[0,0] and s.get('money')==0 and s.get('layer')==0 for s in world_steps))
        check(role+': no decoder combat or warp errors',all(trace.get(k)==[] for k in ('errors','hits','levels','warps','cast_events','cast_results','learn_results')))
        samples=r.get('samples',[])
        check(role+': bounded CPU GPU and job accounting across reboot',0<len(samples)<12000 and all(
            s.get('bake_px')==1024 and s.get('cpu_pixel_limit')==134217728 and 0<=s.get('cpu_pixel_bytes',-1)<=134217728
            and s.get('worker_error') is None and s.get('async_enabled') is True and s.get('last_bakes')==0
            and 0<=s.get('resident',-1)<=s.get('wanted',-1)<=49 and s.get('texture_payload_bytes')==s.get('resident',-1)*4194304
            and 0<=s.get('last_restores',-1)+s.get('last_completed_uploads',-1)<=4
            and s.get('completed',0)+s.get('cancelled',0)+int(s.get('job_active',False))==s.get('submitted',0) for s in samples))
        p=pictures.get(role,{})
        check(role+': recovered ground pixels exactly match initial',p.get('size')==[1280,720] and len(p.get('initial_sha256',''))==64 and p.get('initial_sha256')==p.get('recovered_sha256'))
        check(role+': Windows CPU GPU counters sampled before and after',all(sum(m.get('role')==role and m.get('phase')==phase and m.get('process_id',0)>0 and m.get('Private Bytes',0)>0 and m.get('Total Committed',0)>0 for m in memory)>=3 for phase in ('initial','recovered')))
    before_handles={r:reports[r].get('trace',{}).get('logins',[]) for r in ROLES}
    check('numeric player handles are actually reused across server boots',all(len(v)==2 for v in before_handles.values())
        and {v[0].get('handle') for v in before_handles.values()}=={v[1].get('handle') for v in before_handles.values()} and len({v[0].get('handle') for v in before_handles.values()})==2)
    return dict(success=all(c['passed'] for c in checks),checks=checks,workers=facts,pictures=pictures)

def load_and_compare(root):
    root=Path(root);reports={r:json.loads((root/f'{r}.json').read_text(encoding='utf-8-sig')) for r in ROLES};pictures={}
    for role in ROLES:
        data={p:(root/f'{role}-{p}.ground.png').read_bytes() for p in ('initial','recovered')}
        sizes=[list(struct.unpack('>II',v[16:24])) if v[:8]==b'\x89PNG\r\n\x1a\n' else [] for v in data.values()]
        pictures[role]=dict(size=sizes[0] if sizes[0]==sizes[1] else [],**{p+'_sha256':hashlib.sha256(v).hexdigest() for p,v in data.items()})
    read=lambda name:json.loads((root/name).read_text(encoding='utf-8-sig'))
    database=dict(duplicate=read('duplicate-start.json'),down=read('down-database.json'),restarted=read('restarted-database.json'))
    return compare(reports,read('server-process.json'),database,pictures,read('memory.json'))

if __name__=='__main__':
    root=Path(sys.argv[1]);r=load_and_compare(root);(root/'comparison.json').write_text(json.dumps(r,indent=2),encoding='utf-8')
    print(json.dumps(dict(success=r['success'],checks=len(r['checks']),failed=[c['name'] for c in r['checks'] if not c['passed']],workers=r['workers']),indent=2));raise SystemExit(0 if r['success'] else 1)
