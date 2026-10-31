"""Own an isolated MariaDB process, crash/restart it while the world stays alive.

No input PID is terminated. Data/config/log paths must resolve inside the dedicated
restart workspace, all database endpoints use its separate loopback service, and
normal InnoDB recovery is used on the SAME retained data directory.
"""
import hashlib
import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import time
from terrain_server_restart import validate,config_values,boot,write,wait_until,durable_reward_window


def database_settings(plan,probe):
    workspace,root,_,cfg,_=validate(plan)
    if plan.get('database_restart') is not True or plan.get('money_level') is not True:raise ValueError('explicit database currency-reward scenario required')
    spec=plan['database'];exe=Path(spec['executable']).resolve();ini=Path(spec['config']).resolve()
    if exe.name.lower()!='mariadbd.exe' or not ini.is_relative_to(workspace):raise ValueError('dedicated database config and MariaDB executable required')
    if hashlib.sha256(exe.read_bytes()).hexdigest()!=spec['executable_sha256']:raise ValueError('database executable changed')
    if any(line.lstrip().startswith('!') for line in ini.read_text(encoding='utf-8').splitlines()):raise ValueError('database include directives are forbidden')
    c={k.lower().replace('_','-'):v for k,v in config_values(ini).items()}
    allowed={'basedir','datadir','port','bind-address','innodb-buffer-pool-size','max-connections','log-error','pid-file','innodb-flush-log-at-trx-commit','innodb-force-recovery'}
    if set(c)-allowed or c.get('bind-address')!='127.0.0.1':raise ValueError('unsupported isolated database configuration')
    data=Path(c['datadir']).resolve();log=Path(c['log-error']).resolve();pid=Path(c['pid-file']).resolve();port=int(c['port'])
    if data!=workspace/'mariadb' or not (data/'mysql').is_dir() or not all(p.is_relative_to(workspace) for p in [data,log,pid]):raise ValueError('retained dedicated database directory/logs required')
    if Path(c['basedir']).resolve()!=exe.parent.parent or not 1024<=port<65536 or port==3310:raise ValueError('isolated database executable/endpoint required')
    if c.get('innodb-force-recovery','0')!='0' or c.get('innodb-flush-log-at-trx-commit','1')!='1':raise ValueError('normal durable InnoDB recovery required')
    values=config_values(cfg)
    for key in ['CharacterDatabaseInfo','LoginDatabaseInfo','WorldDatabaseInfo']:
        if values[key].split(';')[1]!=str(port):raise ValueError('all schemas must share the owned database')
    host,_,user,password,_=values['CharacterDatabaseInfo'].split(';');env=os.environ.copy();env['MYSQL_PWD']=password
    mysql=Path(probe['mysql_executable']).resolve()
    if mysql.parent!=exe.parent or mysql.name.lower()!='mariadb.exe':raise ValueError('matching MariaDB client required')
    command=[str(mysql),'--host='+host,'--port='+str(port),'--user='+user,'--batch','--skip-column-names']
    return exe,ini,data,log,port,command,env


def sql(command,env,query):
    p=subprocess.run([*command,'--connect-timeout=2','--execute='+query],env=env,capture_output=True,text=True,encoding='utf-8',timeout=4)
    if p.returncode:raise RuntimeError('isolated database query unavailable')
    return p.stdout.strip()


def boot_database(settings,root,generation):
    exe,ini,data,log,port,command,env=settings
    with (root/f'database-{generation}.stdout.log').open('wb') as out:
        proc=subprocess.Popen([str(exe),'--defaults-file='+str(ini)],stdout=out,stderr=subprocess.STDOUT,creationflags=subprocess.CREATE_NO_WINDOW if os.name=='nt' else 0)
    def ready():
        if proc.poll() is not None:raise RuntimeError('owned database exited before readiness')
        try:return sql(command,env,'SELECT 1')=='1'
        except (RuntimeError,subprocess.TimeoutExpired):return False
    try:wait_until(ready,30,'owned database startup timed out')
    except BaseException:
        if proc.poll() is None:proc.kill();proc.wait(timeout=5)
        raise
    durable=sql(command,env,'SELECT @@innodb_flush_log_at_trx_commit,@@innodb_force_recovery,@@innodb_doublewrite')
    if durable not in ('1\t0\t1','1\t0\tON'):
        proc.kill();proc.wait(timeout=5);raise RuntimeError('durable normal InnoDB settings not active')
    info=dict(pid=proc.pid,generation=generation,unix_ms=time.time_ns()//1_000_000,port=port,data=str(data),durability=durable)
    write(root/f'database-up-{generation}.json',info);write(root.parent/'db-process.json',info)
    return proc,info


def pending_window(root,pid):
    if (root/'reward-commit.json').exists() and json.loads((root/'reward-commit.json').read_text(encoding='utf-8')).get('success') is False:raise RuntimeError('database reward probe failed')
    try:v=json.loads((root/'database-pending-verified').read_text(encoding='utf-8'))
    except (OSError,ValueError):return None
    return v if (v.get('pid')==pid and v.get('locks_held') is True and len(set(v.get('waiting_transactions',[])))==1
        and 0<=time.time_ns()//1_000_000-v.get('unix_ms',0)<=1500) else None


def run(plan,probe):
    _,root,exe,cfg,port=validate(plan);settings=database_settings(plan,probe);root.mkdir(exist_ok=True)
    if (root/'server-up-0.json').exists():raise ValueError('fresh attempt required')
    for p in [port,settings[4]]:
        with socket.socket() as s:s.bind(('127.0.0.1',p))
    world=db=None;record=dict(success=False,scenario='database-reward',money_level=True,database_restart=True,boots=[],database_boots=[],executable_sha256=plan['executable_sha256'])
    try:
        db,info=boot_database(settings,root,0);record['database_boots'].append(info)
        world,info=boot(exe,cfg,root,port,0);record['boots'].append(info)
        wait_until(lambda:all((root/(r+'.checkpoint-ready')).exists() for r in ['owner','witness']),70,'movement checkpoint missing')
        world.stdin.write(b'saveall\n');world.stdin.flush();write(root/'checkpoint-requested.json',dict(pid=world.pid,unix_ms=time.time_ns()//1_000_000,method='console saveall'))
        wait_until(lambda:(root/'checkpoint-verified').exists() and (root/'release-initial').exists() and (root/'reward-locks-ready').exists(),35,'checkpoint and reward locks missing')
        write(root/'actions-release',dict(ready=True))
        record['first_reward']=wait_until(lambda:durable_reward_window(root,world.pid),40,'committed first reward missing')
        chosen=wait_until(lambda:pending_window(root,world.pid),45,'second pending reward window missing')
        if world.poll() is not None or db.poll() is not None:raise RuntimeError('world/database stopped before controlled crash')
        killed=dict(pid=db.pid,world_pid=world.pid,unix_ms=time.time_ns()//1_000_000,method='owned database Popen.kill',graceful=False,pending_reward=chosen)
        db.kill();killed['exit_code']=db.wait(timeout=5)
        if killed['exit_code']==0:raise RuntimeError('database did not exit uncleanly')
        record['database_kill']=killed;write(root/'database-killed.json',killed)
        wait_until(lambda:all((root/(r+'.dropped')).exists() for r in ['owner','witness']) and time.time_ns()//1_000_000-killed['unix_ms']>=5000,55,'both failed-save connections did not close')
        if world.poll() is not None:raise RuntimeError('worldserver exited during database outage')
        offset=settings[3].stat().st_size;db,info=boot_database(settings,root,1);record['database_boots'].append(info)
        record['world_alive_after_database_restart']=world.poll() is None;record['world_pid_after_database_restart']=world.pid
        record['recovery_log'] = settings[3].read_bytes()[offset:].decode('utf-8',errors='replace')
        wait_until(lambda:(root/'server-stop').exists(),100,'recovered client sessions did not finish')
        if db.poll() is not None or world.poll() is not None:raise RuntimeError('recovered world/database exited')
        world.stdin.write(b'server shutdown\n');world.stdin.flush();record['final_exit_code']=world.wait(timeout=12)
        if record['final_exit_code']!=0:raise RuntimeError('final world shutdown failed')
        record['success']=True;record['database_handed_off']=True
    finally:
        if world and world.poll() is None:world.kill();world.wait(timeout=5);record['cleanup_kill']=True
        if db and db.poll() is None and not record['success']:db.kill();db.wait(timeout=5);record['cleanup_database_kill']=True
        write(root/'server-process.json',record)

if __name__=='__main__':run(*[json.loads(Path(p).read_text(encoding='utf-8-sig')) for p in sys.argv[1:3]])
