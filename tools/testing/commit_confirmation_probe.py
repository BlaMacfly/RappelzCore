"""Observe a real successful reward COMMIT whose actual SQL OK is lost."""
import json
from pathlib import Path
import sys
import time
from character_save_probe import Database,SettingsLock
from reward_commit_probe import withheld,awarded,pending
from terrain_server_restart import write,wait_until


def run(plan,probe):
    if plan.get('ambiguous_commit') is not True:raise ValueError('explicit ambiguous COMMIT scenario required')
    db=Database(plan,probe);root=db.root;result=dict(success=False,scenario='ambiguous-commit')
    try:
        wait_until(lambda:(root/'checkpoint-verified').exists() and (root/'release-initial').exists(),90,'checkpoint missing')
        before=db.profiles()
        if before!=json.loads((root/'checkpoint-database.json').read_text(encoding='utf-8')):raise RuntimeError('checkpoint changed')
        result['before']=before
        with SettingsLock(db) as lock:
            write(root/'reward-locks-ready',dict(connection=lock.connection))
            def blocked():
                waits=db.waits(lock.connection);committed=db.profiles();dirty=db.profiles(True);client=pending(root);now=time.time_ns()//1_000_000
                return dict(waits=waits,committed=committed,dirty=dirty,client=client,unix_ms=now) if withheld(before,committed,dirty,waits,client,now,True) else None
            first=wait_until(blocked,35,'real reward did not wait before COMMIT')
            arm=dict(connection_id=first['waits'][0]['connection'],transaction=first['waits'][0]['transaction'],unix_ms=time.time_ns()//1_000_000)
            result.update(blocked=first,arm=arm);write(root/'commit-arm.json',arm)
        held=wait_until(lambda:json.loads((root/'commit-response-held.json').read_text(encoding='utf-8')) if (root/'commit-response-held.json').exists() else None,8,'actual successful COMMIT response not held')
        if held['connection_id']!=arm['connection_id'] or held.get('forwarded') is not False:raise RuntimeError('wrong COMMIT response held')
        observations=[];end=None
        while end is None or time.monotonic()<end:
            profiles=db.profiles();client=pending(root);now=time.time_ns()//1_000_000
            if not awarded(before,profiles,True) or client.get('xp_jp')!=[799,7] or client.get('level')!=2 or client.get('money')!=1000 or not 0<=now-client.get('unix_ms',0)<=150:
                raise RuntimeError('successful COMMIT was not independently visible while reward remained unannounced')
            observations.append(dict(profiles=profiles,client=client,unix_ms=now))
            if end is None:end=time.monotonic()+.6
            time.sleep(.05)
        result.update(held=held,committed=observations[-1]['profiles'],unacknowledged=observations)
        write(root/'discard-commit-response',dict(verified=True))
        reset=wait_until(lambda:json.loads((root/'commit-response-reset.json').read_text(encoding='utf-8')) if (root/'commit-response-reset.json').exists() else None,5,'target SQL connection did not close')
        result['reset']=reset
        wait_until(lambda:all((root/(r+'.dropped')).exists() for r in ['owner','witness']),25,'owner error and ordinary witness lobby exit missing')
        profiles=db.profiles();owner=result['committed']['owner'];witness=profiles['witness']
        if profiles['owner']!=owner or witness['position']!=before['witness']['position'] or not 0<witness['health']<=100 or [witness[k] for k in ['level','xp','jp','money','mp','online']]!=[1,0,0,2000,50,0]:
            raise RuntimeError('stale disconnect save or peer save changed durable reward state')
        if witness['settings']!=before['witness']['settings']:raise RuntimeError('peer extras changed')
        result['reentry_profiles']=profiles
        write(root/'reward-committed.json',profiles)
        # Re-enter only after the existing monster's normal respawn, without resetting world.
        wait_until(lambda:time.time_ns()//1_000_000-held['unix_ms']>=17000,20,'ordinary respawn wait failed')
        result.update(success=True,verified_unix_ms=time.time_ns()//1_000_000)
        write(root/'clients-reconnect',dict(verified=True))
    except Exception as error:result['failure']=str(error);raise
    finally:write(root/'ambiguous-commit.json',result)

if __name__=='__main__':run(*[json.loads(Path(p).read_text(encoding='utf-8-sig')) for p in sys.argv[1:3]])
