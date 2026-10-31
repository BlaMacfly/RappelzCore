"""Kill/restart only an owned, copied worldserver using separate loopback databases.

Run: python terrain_server_restart.py <plan.json>
The plan and executable/config must live under a terrain-restart-<token> workspace.
No PID from an input file or a process search is ever terminated.
"""
import hashlib
import json
import os
from pathlib import Path
import re
import socket
import subprocess
import sys
import time

def config_values(path):
    return {k.strip():v.strip().strip('"') for line in Path(path).read_text(encoding='utf-8').splitlines()
        if line.strip() and not line.lstrip().startswith(('#','[')) and '=' in line for k,v in [line.split('=',1)]}

def validate(plan):
    if plan.get('scenario','terrain') not in ('terrain','gameplay','save-rollback','reward-durable'):raise ValueError('unsupported restart scenario')
    workspace=Path(plan['workspace']).resolve();directory=Path(plan['directory']).resolve()
    match=re.search(r'terrain-restart-([a-f0-9]{8})$',workspace.name)
    if not match or not directory.is_relative_to(workspace):raise ValueError('dedicated restart workspace required')
    executable=Path(plan['executable']).resolve();config=Path(plan['config']).resolve()
    if not executable.is_relative_to(workspace) or not config.is_relative_to(workspace):raise ValueError('copied executable and config must stay in the isolated workspace')
    cfg=config_values(config);prefix='rz_crash_'+match[1]
    for key,kind in [('LoginDatabaseInfo','auth'),('CharacterDatabaseInfo','characters'),('WorldDatabaseInfo','world')]:
        parts=cfg.get(key,'').split(';')
        if len(parts)!=5 or parts[0]!='127.0.0.1' or not parts[1].isdigit() or not 1024<=int(parts[1])<65536 or int(parts[1])==3310 or parts[4]!=prefix+'_'+kind:
            raise ValueError('all databases must be isolated on a separate loopback database service')
    port=int(cfg.get('WorldServerPort','0'))
    if cfg.get('BindIP')!='127.0.0.1' or not 1024<=port<65536 or port==4514 or cfg.get('RealmID')!='1':raise ValueError('isolated world endpoint required')
    if hashlib.sha256(executable.read_bytes()).hexdigest()!=plan['executable_sha256']:raise ValueError('tested executable changed')
    return workspace,directory,executable,config,port

def candidates(root,now_ms):
    found={}
    for role in ('owner','witness'):
        try:value=json.loads((root/(role+'.candidate.json')).read_text(encoding='utf-8'))
        except (OSError,ValueError):return None
        if not isinstance(value,dict):return None
        if (value.get('role')!=role or not 0<=now_ms-value.get('unix_ms',0)<=8 or value.get('running') is not True
            or not 0<value.get('rows',0)<256 or value.get('id',0)<=0 or value.get('instance',0)<=0):return None
        found[role]=value
    return found

def gameplay_candidates(root,now_ms):
    found={}
    for role in ('owner','witness'):
        try:v=json.loads((root/(role+'.activity.json')).read_text(encoding='utf-8'))
        except (OSError,ValueError):return None
        if not isinstance(v,dict):return None
        if (v.get('role')!=role or not 0<=now_ms-v.get('unix_ms',0)<=32
            or v.get('moving') is not True or v.get('peer_moving') is not True or v.get('route_acks',0)<1
            or min(v.get('own_hits',0),v.get('peer_hits',0),v.get('retaliations',0))<1
            or not 0<v.get('monster_hp',0)<v.get('monster_max_hp',0) or v.get('own_hp',0)<=0
            or v.get('distance_from_checkpoint',0)<=3 or v.get('instance',0)<=0):return None
        found[role]=v
    a,b=found['owner'],found['witness']
    if a.get('own')!=b.get('peer') or b.get('own')!=a.get('peer') or a.get('monster',0)<=0 or a.get('monster')!=b.get('monster'):return None
    return found

def pending_save_window(root,pid):
    try:v=json.loads((root/'save-window-verified').read_text(encoding='utf-8'))
    except (OSError,ValueError):return None
    if (not isinstance(v,dict) or v.get('pid')!=pid or v.get('locks_held') is not True
        or not 0<=time.time_ns()//1_000_000-v.get('unix_ms',0)<=1500
        or len(set(v.get('waiting_transactions',[])))!=2):return None
    return v

def durable_reward_window(root,pid):
    if (root/'reward-commit.json').exists():
        result=json.loads((root/'reward-commit.json').read_text(encoding='utf-8'))
        if result.get('success') is False:raise RuntimeError('reward SQL probe failed; inspect retained evidence')
    try:v=json.loads((root/'reward-durable-verified').read_text(encoding='utf-8'))
    except (OSError,ValueError):return None
    if (not isinstance(v,dict) or v.get('pid')!=pid or v.get('committed') is not True
        or not 0<=time.time_ns()//1_000_000-v.get('unix_ms',0)<=1500):return None
    return v

def write(path,value):
    temporary=path.with_suffix(path.suffix+'.tmp');temporary.write_text(json.dumps(value,indent=2),encoding='utf-8');temporary.replace(path)

def wait_until(predicate,seconds,label):
    end=time.monotonic()+seconds
    while time.monotonic()<end:
        result=predicate()
        if result:return result
        time.sleep(.002)
    raise TimeoutError(label)

def server_environment():
    env={k:v for k,v in os.environ.items() if not k.startswith('AC_')}
    env['RUST_LOG']='info'
    return env

def boot(executable,config,root,port,generation):
    env=server_environment()
    with (root/f'world-{generation}.log').open('wb') as out,(root/f'world-{generation}.stderr.log').open('wb') as err:
        proc=subprocess.Popen([str(executable),'--config',str(config)],stdin=subprocess.PIPE,stdout=out,stderr=err,cwd=root,env=env,
            creationflags=subprocess.CREATE_NO_WINDOW if os.name=='nt' else 0)
    def ready():
        if proc.poll() is not None:raise RuntimeError('isolated worldserver exited before readiness')
        return 'Worldserver prêt' in (root/f'world-{generation}.log').read_text(encoding='utf-8',errors='replace')
    try:wait_until(ready,25,'world startup timed out')
    except BaseException:
        if proc.poll() is None:proc.kill();proc.wait(timeout=5)
        raise
    info=dict(pid=proc.pid,generation=generation,port=port,unix_ms=time.time_ns()//1_000_000)
    write(root/f'server-up-{generation}.json',info)
    return proc,info

def run(plan):
    _,root,exe,cfg,port=validate(plan);root.mkdir(exist_ok=True)
    if (root/'server-up-0.json').exists():raise ValueError('fresh attempt required')
    # Do not start on a port owned by any existing process.
    with socket.socket() as probe:probe.bind(('127.0.0.1',port))
    scenario=plan.get('scenario','terrain')
    proc=None;record=dict(success=False,boots=[],executable_sha256=plan['executable_sha256'],scenario=scenario)
    try:
        proc,info=boot(exe,cfg,root,port,0);record['boots'].append(info)
        if scenario in ('gameplay','save-rollback','reward-durable'):
            wait_until(lambda:all((root/(r+'.checkpoint-ready')).exists() for r in ('owner','witness')),70,'movement checkpoint missing')
            proc.stdin.write(b'saveall\n');proc.stdin.flush()
            write(root/'checkpoint-requested.json',dict(pid=proc.pid,unix_ms=time.time_ns()//1_000_000,method='console saveall'))
            wait_until(lambda:(root/'checkpoint-verified').exists() and (root/'release-initial').exists(),35,'durable movement checkpoint and initial capture not verified')
            if scenario=='reward-durable':wait_until(lambda:(root/'reward-locks-ready').exists(),10,'pre-reward locks missing')
            write(root/'actions-release',dict(ready=True))
            if scenario=='reward-durable':
                chosen=wait_until(lambda:durable_reward_window(root,proc.pid),45,'committed private reward not verified')
            elif scenario=='save-rollback':
                wait_until(lambda:(root/'save-locks-ready').exists(),40,'save row locks missing')
                proc.stdin.write(b'saveall\n');proc.stdin.flush()
                write(root/'pending-save-requested.json',dict(pid=proc.pid,unix_ms=time.time_ns()//1_000_000,method='console saveall'))
                chosen=wait_until(lambda:pending_save_window(root,proc.pid),15,'two partially written save transactions not verified')
            else:
                chosen=wait_until(lambda:gameplay_candidates(root,time.time_ns()//1_000_000),15,'two moving combatants not observed together')
        else:
            chosen=wait_until(lambda:candidates(root,time.time_ns()//1_000_000) if (root/'release-initial').exists() else None,90,'two fresh partial jobs not observed together')
        # The Popen object owns the exact process handle. It cannot target the
        # user's running server even if a numeric PID is later reused.
        if proc.poll() is not None:raise RuntimeError('world already stopped')
        killed=dict(pid=proc.pid,unix_ms=time.time_ns()//1_000_000,**{('activity' if scenario=='gameplay' else 'pending_save' if scenario=='save-rollback' else 'reward_commit' if scenario=='reward-durable' else 'jobs'):chosen},method='owned Popen.kill',graceful=False)
        proc.kill();killed['exit_code']=proc.wait(timeout=5);record['kill']=killed
        if killed['exit_code']==0:raise RuntimeError('force-killed process unexpectedly exited cleanly')
        write(root/'server-killed.json',killed)
        wait_until(lambda:all((root/(r+'.dropped')).exists() for r in ('owner','witness')) and (root/'allow-restart').exists()
            and time.time_ns()//1_000_000-killed['unix_ms']>=5000,30,'clients/drop and database snapshot missing')
        proc,info=boot(exe,cfg,root,port,1);record['boots'].append(info)
        wait_until(lambda:(root/'server-stop').exists(),100,'final client session did not finish')
        proc.stdin.write(b'server shutdown\n');proc.stdin.flush();record['final_exit_code']=proc.wait(timeout=10)
        if record['final_exit_code']!=0:raise RuntimeError('final graceful shutdown failed')
        record['success']=True
    finally:
        if proc and proc.poll() is None:
            proc.kill();proc.wait(timeout=5);record['cleanup_kill']=True
        write(root/'server-process.json',record)

if __name__=='__main__':run(json.loads(Path(sys.argv[1]).read_text(encoding='utf-8-sig')))
