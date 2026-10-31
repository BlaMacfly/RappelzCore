"""Validate opaque transport stalls, real resets and terrain recovery with two native clients."""
import hashlib
import json
from pathlib import Path
import struct
import sys
from compare_terrain_cancellation_clients import audit_facts

ROLES=('owner','witness')

def compare(reports,proxy,pictures,memory):
    checks=[];facts={}
    def check(name,value):checks.append(dict(name=name,passed=bool(value)))
    check('opaque relay completed normally',proxy.get('success') is True and set(proxy.get('relays',{}))==set(ROLES))
    for index,role in enumerate(ROLES):
        other=ROLES[1-index];r=reports[role];peer=reports[other];steps=r.get('steps',[]);trace=r.get('trace',{});audits=r.get('audits',[])
        fs=[audit_facts(a) for a in audits];facts[role]=fs
        check(role+': native scenario completed',r.get('scenario')=='terrain-network' and r.get('role')==role and r.get('success') is True and not r.get('failure'))
        check(role+': two causal bounded joined world workers',len(fs)==2 and len({f['instance'] for f in fs})==2 and all(not f['failures'] for f in fs))
        relay=proxy.get('relays',{}).get(role,{});events=relay.get('events',[])
        tagged=lambda name:[e for e in events if e.get('event')==name]
        pauses,resumes,resets=tagged('pause'),tagged('resume'),tagged('reset')
        check(role+': two real upstream connections with clean lifecycle',len(tagged('connected'))==len(tagged('closed'))==2
            and [e.get('connection') for e in tagged('connected')]==[1,2] and [e.get('connection') for e in tagged('closed')]==[1,2]
            and not any(e.get('event') in ('connection_error','rejected_concurrent') for e in events))
        check(role+': bounded relay metadata and held blocks',relay.get('limit')==256 and relay.get('overflow')==0 and relay.get('block_limit')==65536 and 0<len(events)<=256
            and [e.get('ns') for e in events]==sorted(e.get('ns') for e in events)
            and all(set(e.get('held',{}))=={'up','down'} and all(0<=n<=65536 for n in e['held'].values()) for e in events))
        check(role+': eight seconds with zero forwarded bytes in both directions',len(pauses)==len(resumes)==1 and pauses[0].get('connection')==resumes[0].get('connection')==1
            and 8_000_000_000<=resumes[0]['ns']-pauses[0]['ns']<=30_000_000_000
            and set(pauses[0].get('bytes',{}))=={'up','down'} and pauses[0]['bytes']==resumes[0].get('bytes')
            and all(resumes[0]['held'].get(k,0)>0 for k in ('up','down')))
        check(role+': abortive remote reset follows resumed forwarding',len(resets)==len(resumes)==1 and resets[0].get('abortive') is True
            and resets[0].get('connection')==1 and resets[0]['ns']>resumes[0]['ns'] and all(resets[0].get('bytes',{}).get(k,0)>resumes[0]['bytes'][k] for k in ('up','down')))
        begin=[s for s in steps if s.get('stage')=='StallBegin'];held=[s for s in steps if s.get('stage')=='StallHeld'];resumed=[s for s in steps if s.get('stage')=='Resumed']
        check(role+': rendering and peer identity survive both stalls',len(begin)==len(held)==len(resumed)==2 and [s.get('cycle') for s in begin]==[0,1] and all(
            b.get('cycle')==h.get('cycle')==s.get('cycle')==i and b.get('own')==h.get('own')==s.get('own')
            and b.get('peer')==h.get('peer')==s.get('peer') and bool(h.get('peer')) and h.get('stage_ms',0)>=8000
            and h.get('world_frame',0)-b.get('world_frame',0)>=120 and b.get('audit_instance')==h.get('audit_instance')
            and not b.get('loading') and not h.get('loading') for i,(b,h,s) in enumerate(zip(begin,held,resumed))))
        expected=[f'TerrainNet_{i}_'+('in' if i==index else 'out') for i in range(2)]
        chats=trace.get('chats',[])
        check(role+': each delayed private probe delivered exactly once',len(chats)==2 and [c.get('message') for c in chats]==expected
            and all(c.get('sender')==peer.get('character') and c.get('chat_type')==3 for c in chats))
        check(role+': no probe delivered before resume',len(held)==2 and all(not any(c.get('message')==expected[i] for c in h.get('chats',[])) for i,h in enumerate(held)))
        rotations=set(r.get('rotations',[]));actor_begin=next((s for s in begin if s.get('cycle')==index),{});actor_held=next((s for s in held if s.get('cycle')==index),{})
        current=next((a for a in audits if a.get('instance')==actor_begin.get('audit_instance')),{});ae=current.get('events',[])
        eligible=next((f['partial_view_ids'] for f in fs if f['instance']==actor_begin.get('audit_instance')),[])
        check(role+': three partial blends cancelled inside the paused interval',len(rotations)>=3 and rotations<=set(eligible) and all(
            any(e.get('id')==id and e.get('event')=='cancel_view' and actor_begin.get('audit_cursor',999999)<=e.get('order',-1)<actor_held.get('audit_cursor',0) for e in ae)
            and any(e.get('id')==id and e.get('event')=='polled_cancelled' and e.get('order',999999)<actor_held.get('audit_cursor',0) for e in ae) for id in rotations))
        req=[s for s in steps if s.get('stage')=='ResetRequested'];joined=[s for s in steps if s.get('stage')=='WorkerJoined'];reentry=[s for s in steps if s.get('stage')=='Reentered']
        check(role+': reset interrupts its actual running blend',len(req)==1 and req[0].get('cycle')==index and len(fs)==2 and req[0].get('closing_instance')==fs[0]['instance']
            and req[0].get('job') in fs[0]['partial_shutdown_ids'] and req[0].get('cache',{}).get('active_job',{}).get('id')==req[0].get('job')
            and req[0]['cache']['active_job'].get('running') is True and 0<req[0]['cache']['active_job'].get('rows',0)<1024)
        check(role+': genuine remote transport error without client disconnect request',len(trace.get('network_errors',[]))==1 and bool(trace['network_errors'][0]) and trace.get('disconnects')==[])
        logins=trace.get('logins',[]);pl=peer.get('trace',{}).get('logins',[])
        check(role+': worker joins before fresh-handle reauthentication',len(joined)==len(reentry)==1 and len(logins)==2 and logins[0].get('handle')!=logins[1].get('handle')
            and joined[0].get('elapsed_ms',999999)<reentry[0].get('elapsed_ms',0) and reentry[0].get('own')==logins[1].get('handle')
            and joined[0].get('player') is None and joined[0].get('error') and joined[0].get('audit_instance')!=reentry[0].get('audit_instance'))
        check(role+': two real Lua login/entry/logout cycles',all(r.get('events',{}).get(k)==2 for k in ('PLAYER_LOGIN','PLAYER_ENTERING_WORLD','PLAYER_LOGOUT'))
            and len(logins)==2 and all(l.get('name')==r.get('character') and l.get('layer')==0 for l in logins))
        y,py=(7227,7247) if role=='owner' else (7247,7227)
        check(role+': peer departure and authoritative reappearance',len(pl)==2 and pl[0].get('handle') in trace.get('leaves',[])
            and len([s for s in steps if s.get('stage')=='ObservedLeave'])==len([s for s in steps if s.get('stage')=='ObservedReentry'])==1
            and all(any(e.get('handle')==l.get('handle') and e.get('name')==peer.get('character') and [e.get(k) for k in ('x','y','layer','hp','mp','level')]==[8440,py,0,100,50,1]
                for e in trace.get('enters',[])) for l in pl))
        world_steps=[s for s in steps if s.get('player')]
        check(role+': character data and position unchanged',bool(world_steps) and all(s.get('position')==[8440,y] and [s['player'].get(k) for k in ('name','level','hp','mp')]==[r.get('character'),1,100,50]
            and s.get('xp_jp')==[0,0] and s.get('money')==0 and s.get('layer')==0 for s in world_steps))
        check(role+': no decoder/combat/warp errors',all(trace.get(k)==[] for k in ('errors','hits','levels','warps','cast_events','cast_results','learn_results')))
        samples=r.get('samples',[])
        check(role+': bounded CPU/GPU terrain throughout stalls and reconnect',0<len(samples)<16000 and all(
            s.get('bake_px')==1024 and s.get('cpu_pixel_limit')==134217728 and 0<=s.get('cpu_pixel_bytes',-1)<=134217728
            and s.get('worker_error') is None and s.get('async_enabled') is True and s.get('last_bakes')==0
            and 0<=s.get('resident',-1)<=s.get('wanted',-1)<=49 and s.get('texture_payload_bytes')==s.get('resident',-1)*4194304
            and 0<=s.get('last_restores',-1)+s.get('last_completed_uploads',-1)<=4
            and s.get('completed',0)+s.get('cancelled',0)+int(s.get('job_active',False))==s.get('submitted',0) for s in samples))
        p=pictures.get(role,{})
        check(role+': exact recovered ground pixels',p.get('size')==[1280,720] and len(p.get('initial_sha256',''))==64 and p.get('initial_sha256')==p.get('recovered_sha256'))
        check(role+': CPU/GPU counters at initial and recovered views',all(sum(m.get('role')==role and m.get('phase')==phase and m.get('process_id',0)>0 and m.get('Private Bytes',0)>0 and m.get('Total Committed',0)>0 for m in memory)>=3 for phase in ('initial','recovered')))
    return dict(success=all(c['passed'] for c in checks),checks=checks,workers=facts,pictures=pictures)

def load_and_compare(root):
    root=Path(root);reports={r:json.loads((root/f'{r}.json').read_text(encoding='utf-8-sig')) for r in ROLES};pictures={}
    for role in ROLES:
        data={phase:(root/f'{role}-{phase}.ground.png').read_bytes() for phase in ('initial','recovered')}
        sizes=[list(struct.unpack('>II',v[16:24])) if v[:8]==b'\x89PNG\r\n\x1a\n' else [] for v in data.values()]
        pictures[role]=dict(size=sizes[0] if sizes[0]==sizes[1] else [],**{p+'_sha256':hashlib.sha256(v).hexdigest() for p,v in data.items()})
    return compare(reports,json.loads((root/'proxy.json').read_text(encoding='utf-8-sig')),pictures,json.loads((root/'memory.json').read_text(encoding='utf-8-sig')))

if __name__=='__main__':
    root=Path(sys.argv[1]);r=load_and_compare(root);(root/'comparison.json').write_text(json.dumps(r,indent=2),encoding='utf-8')
    print(json.dumps(dict(success=r['success'],checks=len(r['checks']),failed=[c['name'] for c in r['checks'] if not c['passed']],workers=r['workers']),indent=2))
    raise SystemExit(0 if r['success'] else 1)
