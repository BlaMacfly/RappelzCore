"""Observe a real interrupted character save on isolated, newly created fixtures.

Run with a server plan and a probe plan (mysql executable, fixtures path and
pre-existing GUID/account upper bounds). Credentials come from the isolated
server config and are passed to MariaDB only through MYSQL_PWD. No schema or
production save code is modified; the only write is an owned row-lock transaction
that is always rolled back. All queries and waits have a bounded lifetime.
"""
import json
import os
from pathlib import Path
import queue
import re
import subprocess
import sys
import threading
import time
from terrain_server_restart import validate,config_values,wait_until,write

ROLES=('owner','witness')

def guarded_fixtures(fixtures,existing_guid,existing_account):
    if set(fixtures)!=set(ROLES) or min(existing_guid,existing_account)<0:
        raise ValueError('two dedicated fixtures and baseline bounds required')
    for f in fixtures.values():
        if (type(f.get('guid')) is not int or f['guid']<=existing_guid or type(f.get('account')) is not int
            or f['account']<=existing_account or not re.fullmatch(r'Lua[A-Za-z0-9]+',f.get('character',''))):
            raise ValueError('only newly created Lua fixtures may be locked')
    if len({f['guid'] for f in fixtures.values()})!=2 or len({f['account'] for f in fixtures.values()})!=2:
        raise ValueError('distinct fixtures required')
    return {r:f['guid'] for r,f in fixtures.items()}

class Database:
    def __init__(self,plan,probe):
        self.workspace,self.root,_,cfg,_=validate(plan)
        if plan.get('scenario') not in ('save-rollback','reward-durable'):raise ValueError('save/reward probe plan required')
        fixture_path=Path(probe['fixtures']).resolve()
        if not fixture_path.is_relative_to(self.workspace):raise ValueError('isolated fixture file required')
        self.fixtures=json.loads(fixture_path.read_text(encoding='utf-8'))
        self.guids=guarded_fixtures(self.fixtures,probe['existing_character_max'],probe['existing_account_max'])
        host,port,user,password,self.schema=config_values(cfg)['CharacterDatabaseInfo'].split(';')
        self.env=os.environ.copy();self.env['MYSQL_PWD']=password
        self.command=[probe['mysql_executable'],'--protocol=tcp','--host='+host,'--port='+port,'--user='+user,
            '--batch','--unbuffered','--skip-column-names','--connect-timeout=3']
        self.flags=subprocess.CREATE_NO_WINDOW if os.name=='nt' else 0
        for role,guid in self.guids.items():
            f=self.fixtures[role]
            actual=self.query(f'SELECT account,name FROM {self.schema}.characters WHERE guid={guid}')
            if actual!=f"{f['account']}\t{f['character']}":raise ValueError('fixture database identity mismatch')
        if self.query(f"SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='{self.schema}' AND table_name IN ('characters','character_settings') AND engine='InnoDB'")!='2':
            raise ValueError('both save tables must be transactional')
    def query(self,query):
        result=subprocess.run([*self.command,'--execute='+query],env=self.env,creationflags=self.flags,
            capture_output=True,text=True,encoding='utf-8',timeout=6)
        if result.returncode:raise RuntimeError('isolated SQL probe failed (details intentionally omitted)')
        return result.stdout.strip()
    def profiles(self,dirty=False):
        # Both tables are read by ONE connection, with the requested isolation.
        columns='c.guid,c.level,c.xp,c.money,c.health,c.power1,c.position_x,c.position_y,c.online,s.data'
        values=self.query(('SET SESSION TRANSACTION ISOLATION LEVEL READ UNCOMMITTED;' if dirty else '')+
            f'SELECT {columns} FROM {self.schema}.characters c JOIN {self.schema}.character_settings s ON s.guid=c.guid '
            f"AND s.source='rappelz' WHERE c.guid IN ({','.join(map(str,self.guids.values()))}) ORDER BY c.guid")
        result={}
        for row in values.splitlines():
            a=row.split('\t');role=next(r for r,g in self.guids.items() if g==int(a[0]));settings=dict(p.split('=',1) for p in a[9].split(';'))
            result[role]=dict(level=int(a[1]),xp=int(a[2]),money=int(a[3]),health=int(a[4]),mp=int(a[5]),position=list(map(float,a[6:8])),online=int(a[8]),jp=int(settings['jp']),settings=settings)
        if set(result)!=set(ROLES):raise RuntimeError('missing fixture profiles')
        return result
    def waits(self,connection):
        value=self.query('SELECT DISTINCT r.trx_id,r.trx_mysql_thread_id,r.trx_rows_modified,r.trx_query '
            'FROM information_schema.INNODB_LOCK_WAITS w JOIN information_schema.INNODB_TRX r ON r.trx_id=w.requesting_trx_id '
            'JOIN information_schema.INNODB_TRX b ON b.trx_id=w.blocking_trx_id '
            f'WHERE b.trx_mysql_thread_id={int(connection)} ORDER BY r.trx_id')
        return [dict(transaction=a[0],connection=int(a[1]),modified=int(a[2]),query=a[3]) for a in (row.split('\t') for row in value.splitlines())]

def window_matches(before,committed,dirty,waits):
    return (committed==before and set(dirty)==set(ROLES) and len(waits)==2 and len({w['transaction'] for w in waits})==2
        and all(w['modified']>0 and 'replace into character_settings' in w['query'].lower() for w in waits)
        and all(dirty[r]['position']==[8452+(16 if r=='witness' else 0),7235 if r=='owner' else 7255]
            and dirty[r]['xp']==(5 if r=='owner' else 0) and dirty[r]['jp']==0 and dirty[r]['mp']==50
            and dirty[r]['level']==1 and dirty[r]['money']==0 and 0<dirty[r]['health']<=100 for r in ROLES)
        and any(dirty[r]['health']<100 for r in ROLES))


class SettingsLock:
    """One bounded, owned CLI transaction locking only guarded new fixture rows."""
    def __init__(self,db):self.db=db;self.proc=None
    def __enter__(self):
        db=self.db
        try:
            with (db.root/'save-lock.stderr.log').open('wb') as err:
                self.proc=subprocess.Popen(db.command,env=db.env,creationflags=db.flags,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=err,text=True,encoding='utf-8',bufsize=1)
            rows=queue.Queue()
            def consume():
                for row in self.proc.stdout:rows.put(row.strip())
                rows.put(None)
            self.reader=threading.Thread(target=consume,daemon=True);self.reader.start()
            self.proc.stdin.write('SET SESSION wait_timeout=45; SET SESSION innodb_lock_wait_timeout=5; START TRANSACTION; '+
                f"SELECT guid FROM {db.schema}.character_settings WHERE source='rappelz' AND guid IN ({','.join(map(str,db.guids.values()))}) ORDER BY guid FOR UPDATE; SELECT 'probe-locked'; SELECT CONNECTION_ID();\n")
            self.proc.stdin.flush();end=time.monotonic()+7;seen=[]
            while True:
                try:row=rows.get(timeout=max(.01,end-time.monotonic()))
                except queue.Empty:raise TimeoutError('settings row lock acquisition')
                if row is None or time.monotonic()>end:raise RuntimeError('lock holder exited')
                if seen and seen[-1]=='probe-locked':self.connection=int(row);break
                seen.append(row)
            if sorted(seen[:-1])!=sorted(map(str,db.guids.values())):raise RuntimeError('wrong locked rows')
            return self
        except BaseException:
            self.release();raise
    def release(self):
        proc=self.proc
        if proc and proc.poll() is None:
            try:
                proc.stdin.write('ROLLBACK;\n');proc.stdin.flush();proc.stdin.close();proc.wait(timeout=3)
            except (OSError,ValueError,subprocess.TimeoutExpired):
                if proc.poll() is None:proc.kill()
                proc.wait(timeout=3)
        if proc and hasattr(self,'reader'):self.reader.join(timeout=2)
    def __exit__(self,*_):self.release()


def run(plan,probe):
    db=Database(plan,probe);root=db.root;lock=None;result=dict(success=False,method='owned InnoDB settings row locks')
    try:
        wait_until(lambda:all((root/(r+'.save-ready')).exists() for r in ROLES),100,'pre-save combat missing')
        before=db.profiles();checkpoint=json.loads((root/'checkpoint-database.json').read_text(encoding='utf-8'))
        if before!=checkpoint:raise RuntimeError('unexpected save before controlled window')
        lock=SettingsLock(db).__enter__();connection=lock.connection
        write(root/'save-locks-ready',dict(connection=connection,guids=db.guids))
        request=wait_until(lambda:json.loads((root/'pending-save-requested.json').read_text(encoding='utf-8')) if (root/'pending-save-requested.json').exists() else None,12,'save request missing')
        def observe():
            waits=db.waits(connection);committed=db.profiles();dirty=db.profiles(True)
            return dict(waits=waits,committed=committed,dirty=dirty) if window_matches(before,committed,dirty,waits) else None
        observation=wait_until(observe,10,'two partial writes invisible to committed readers not observed')
        window=dict(pid=request['pid'],unix_ms=time.time_ns()//1_000_000,locks_held=True,waiting_transactions=[w['transaction'] for w in observation['waits']])
        result.update(before=before,request=request,window=window,**observation)
        write(root/'save-window-verified',window)
        wait_until(lambda:(root/'server-killed.json').exists(),5,'server kill missing while locks held')
        lock.release()
        if lock.proc.returncode!=0:raise RuntimeError('lock holder did not exit cleanly')
        ids=','.join("'"+w['transaction']+"'" for w in observation['waits'])
        wait_until(lambda:db.query(f'SELECT COUNT(*) FROM information_schema.INNODB_TRX WHERE trx_id IN ({ids})')=='0',8,'killed server save transactions still alive')
        after=db.profiles()
        if after!=before:raise RuntimeError('partial character/settings save survived killed connection')
        result.update(success=True,after=after,transactions_gone=True,released_unix_ms=time.time_ns()//1_000_000)
        write(root/'save-rollback.json',result);write(root/'save-rollback-complete',dict(success=True))
    finally:
        if lock:lock.release()
        write(root/'save-rollback.json',result)

if __name__=='__main__':run(*[json.loads(Path(p).read_text(encoding='utf-8-sig')) for p in sys.argv[1:3]])
