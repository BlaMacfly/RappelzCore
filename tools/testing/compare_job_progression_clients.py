"""Check real native job purchases against private packets, Lua and independent SQL."""
import json,re
from pathlib import Path
import sys
ROLES=('owner','witness')
def step(r,name):return next((s for s in r.get('steps',[]) if s.get('stage')==name),{})
def compare(reports,server,processes,sql,profile):
    checks=[]
    def check(name,value):checks.append(dict(name=name,passed=bool(value)))
    owner=reports.get('owner',{});witness=reports.get('witness',{});initial=owner.get('initial')
    check('one owned world exits normally after two native clients succeed',server.get('success') is True and server.get('scenario')=='job-progression' and server.get('world_pid',0)>0 and server.get('exit_code')==0 and not server.get('cleanup_clients') and not server.get('failed_attempt_shutdown')
          and len(processes)==2 and {p.get('role') for p in processes}==set(ROLES) and all(p.get('exit_code')==0 for p in processes))
    check('four independent SQL checkpoints gate the next purchase and real reentry',[p.get('name') for p in server.get('checkpoints',[])]==['probed','level2','level3','saved'] and set(sql)=={'probed','level2','level3','saved','final'})
    check('archive profiles are complete lossless and identical in server/client installs',profile.get('success') is True and profile.get('idempotent') is True and profile.get('conflict_preserved') is True
          and [(t.get('file'),t.get('rows'),t.get('fields')) for t in profile.get('tables',[])]==[('BasicStat.dbc',3923,8),('Exp.dbc',300,11),('SummonExp.dbc',200,7)] and all(t.get('lossless') is True and t.get('server_client_equal') is True for t in profile['tables']))
    check('both distinct players receive fresh handles after normal lobby saves',initial and initial==witness.get('initial_other') and owner.get('initial_other')==witness.get('initial') and len({owner.get('initial'),witness.get('initial'),owner.get('own'),witness.get('own')})==4
          and owner.get('own')==witness.get('other') and witness.get('own')==owner.get('other'))
    for role in ROLES:
        r=reports.get(role,{});t=r.get('trace',{});own=role=='owner';saved=step(r,'Saved');jlv=3 if own else 1;money=1000 if own else 2000;job=100 if own else 200;position=[9440,7227] if own else [9420,7247]
        check(role+': bounded driver completes two real Lua lifecycles',r.get('success') is True and not r.get('failure') and r.get('scenario')=='job-progression' and step(r,'Logout').get('elapsed_ms',10**9)<120000 and len(t.get('logins',[]))==2
              and all(r.get('events',{}).get(k)==2 for k in ['PLAYER_LOGIN','PLAYER_ENTERING_WORLD','PLAYER_LOGOUT']))
        check(role+': no network decoding combat cast learning or movement side effects',all(t.get(k)==[] for k in ['errors','network_errors','disconnects','hits','cast_events','learn_results','moves','warps']))
        check(role+': actual request labels and acceptance codes exclude foreign owners and overdraft',[q.get('label') for q in r.get('requests',[])]==(['foreign_owner','unknown_owner','level2','level3','insufficient_jp'] if own else ['foreign_owner','insufficient_jp'])
              and [q.get('target') for q in r.get('requests',[])]==([r.get('initial_other'),4294967295,r.get('initial'),r.get('initial'),r.get('initial')] if own else [r.get('initial_other'),r.get('initial')])
              and [q.get('kind') for q in r.get('requests',[])]==(['raw','raw','helper','helper','raw'] if own else ['raw','raw'])
              and len(t.get('job_results',[]))==(5 if own else 2) and all((v==0)==(own and i in (2,3)) for i,v in enumerate(t.get('job_results',[]))))
        check(role+': exact wide archive data is loaded by the native client',r.get('profile_counts')==[3923,300,200] and r.get('wide_exp300')=='272233208401492000')
        progress=t.get('job_progress',[]);expected=[(1,8),(2,5),(3,0)] if own else [(1,0)]
        sequence=[]
        for p in progress:
            value=(p.get('jlv'),p.get('jp'))
            if not sequence or value!=sequence[-1]:sequence.append(value)
        check(role+': only confirmed private JP and job levels are published',sequence==expected and len(progress)>1 and all(p.get('target') in [r.get('initial'),r.get('own')] and p.get('job')==job for p in progress)
              and progress[-1].get('target')==r.get('own') and progress[-1].get('jlv')==jlv and progress[-1].get('jp')==0)
        check(role+': server offers use the correct cost column and unchanged base attributes',all(p.get('base')==[job,[10]*7] and p.get('cost')==({1:3,2:5,3:8}.get(p.get('jlv')) if own else 3)
              and p.get('status')==(0 if own and p.get('jlv')<3 else 5) for p in progress))
        check(role+': XP and currency stay unchanged while only owner spends JP',[[p.get('xp'),p.get('jp')] for p in t.get('experience',[])]==([[0,8],[0,5],[0,0],[0,0]] if own else [[0,0],[0,0]])
              and [p.get('gold') for p in t.get('gold',[])]==[money,money] and saved.get('xp_jp')==[0,0] and saved.get('money')==money)
        check(role+': real Lua sees each committed job level and exact JP once',r.get('events',{}).get('RAPPELZ_JOB_LEVEL_CHANGED')==(4 if own else 2) and r.get('events',{}).get('RAPPELZ_JP_CHANGED')==(4 if own else 2) and r.get('events',{}).get('PLAYER_LEVEL_UP',0)==0
              and [tuple(map(int,m)) for line in r.get('messages',[]) for m in re.findall(r'EVENT RAPPELZ_JOB_LEVEL_CHANGED (\d+).* jp=(\d+) jlv=(\d+)',line)]==([(1,8,1),(2,5,2),(3,0,3),(3,0,3)] if own else [(1,0,1),(1,0,1)]))
        check(role+': job-only updates keep the player level and are not replayed on reentry',t.get('levels')==[dict(handle=initial,job_level=2,level=10),dict(handle=initial,job_level=3,level=10)]
              and len(t.get('level_orders',[]))==2 and len(t.get('logins',[]))==2 and all(v<t['logins'][1].get('order',0) for v in t['level_orders']))
        check(role+': reentry loads unchanged raw vitals coordinates and empty learned state',len(t.get('logins',[]))==2 and all([l.get(k) for k in ['hp','max_hp','mp','max_mp','x','y']]==[280,280,140,140,*position] for l in t['logins'])
              and saved.get('position')==position and saved.get('player',{}).get('level')==10 and saved.get('learned')=={} and saved.get('job',{}).get('jlv')==jlv and saved['job'].get('jp')==0)
        check(role+': learned skill catalog refreshes privately with the committed job state',[(p.get('job_level'),p.get('jp')) for p in t.get('skill_catalogs',[])]==([(1,8),(2,5),(3,0),(3,0)] if own else [(1,0),(1,0)])
              and all(p.get('target') in [r.get('initial'),r.get('own')] for p in t.get('skill_catalogs',[])))
        before=sql.get('probed',{}).get(role,{});settings=before.get('settings',{})
        for name,level,jp,online in [('probed',1,8 if own else 0,1),('level2',2 if own else 1,5 if own else 0,1),('level3',jlv,0,1),('saved',jlv,0,0),('final',jlv,0,0)]:
            p=sql.get(name,{}).get(role,{})
            check(role+': independent '+name+' SQL is complete and retains unrelated keys',p.get('settings')==dict(settings,jlv=str(level),jp=str(jp)) and [p.get(k) for k in ['level','xp','money','health','mp','online']]==[10,0,money,280,140,online] and p.get('position')==position)
    return dict(success=all(c['passed'] for c in checks),checks=checks)
def load_and_compare(directory):
    directory=Path(directory);read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
    return compare({r:read(directory/(r+'.json')) for r in ROLES},read(directory/'server-process.json'),read(directory/'processes.json'),{s:read(directory/(s+'-database.json')) for s in ['probed','level2','level3','saved','final']},read(directory.parent/'profile-audit.json'))
if __name__=='__main__':
    directory=Path(sys.argv[1]);r=load_and_compare(directory);(directory/'comparison.json').write_text(json.dumps(r,indent=2),encoding='utf-8');print(json.dumps(dict(success=r['success'],checks=len(r['checks']),failed=[c['name'] for c in r['checks'] if not c['passed']]),indent=2));raise SystemExit(0 if r['success'] else 1)
