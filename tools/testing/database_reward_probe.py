"""Extend the reward probe with a second award interrupted by a real DB crash."""
import json
import time
from character_save_probe import SettingsLock
from terrain_server_restart import wait_until,write


def pending_reward_matches(before,committed,dirty,waits,client,now):
    if set(dirty)!=set(before) or 'owner' not in dirty:return False
    old=before['owner'];candidate=dirty['owner'];gain=candidate.get('money',0)-old['money']
    expected=dict(old,xp=9,money=candidate.get('money'),health=candidate.get('health'))
    return (committed==before and dirty['witness']==before['witness'] and candidate==expected
        and 8<=gain<=14 and 0<candidate.get('health',0)<=140
        and len(waits)==1 and waits[0].get('modified',0)>0 and 'replace into character_settings' in waits[0].get('query','').lower()
        and client.get('xp_jp')==[4,10] and client.get('level')==3 and client.get('money')==old['money']
        and 0<=now-client.get('unix_ms',0)<=150)


def finish(db,plan,result):
    root=db.root;before=result['committed'];result['database_restart']=True
    with SettingsLock(db) as lock:
        write(root/'second-actions-release',dict(ready=True))
        def observe():
            waits=db.waits(lock.connection);committed=db.profiles();dirty=db.profiles(True)
            # SQL CLI startup can exceed a frame lease: read native telemetry last.
            try:client=json.loads((root/'owner.pending-reward.json').read_text(encoding='utf-8'))
            except (OSError,ValueError):return None
            now=time.time_ns()//1_000_000
            value=dict(waits=waits,committed=committed,dirty=dirty,client=client,unix_ms=now)
            result['last_pending_observation']=value
            return value if pending_reward_matches(before,committed,dirty,waits,client,now) else None
        first=wait_until(observe,40,'second reward not held before database crash')
        observations=[first];end=time.monotonic()+.6
        while time.monotonic()<end:
            time.sleep(.05);value=observe()
            if not value:raise RuntimeError('second reward exposed before database crash')
            observations.append(value)
        server=json.loads((root/'server-up-0.json').read_text(encoding='utf-8'))
        marker=dict(pid=server['pid'],unix_ms=time.time_ns()//1_000_000,locks_held=True,waiting_transactions=[v['transaction'] for v in observations[-1]['waits']])
        result.update(pending_reward=observations,pending_window=marker)
        write(root/'database-pending-verified',marker)
        wait_until(lambda:(root/'database-killed.json').exists(),5,'owned database was not killed while reward waited')
    # The old CLI connection ended with the killed database. It must never commit.
    wait_until(lambda:(root/'database-up-1.json').exists(),65,'database did not restart')
    after=db.profiles()
    if after!=before:raise RuntimeError('database crash lost a committed reward or retained an incomplete award')
    ids=','.join("'"+v['transaction']+"'" for v in observations[-1]['waits'])
    if db.query(f'SELECT COUNT(*) FROM information_schema.INNODB_TRX WHERE trx_id IN ({ids})')!='0':raise RuntimeError('old pending reward transaction survived recovery')
    result.update(success=True,after=after,transactions_gone=True,verified_after_kill_unix_ms=time.time_ns()//1_000_000)
    write(root/'reward-commit.json',result);write(root/'database-reward-restored.json',after)
    write(root/'clients-reconnect',dict(verified=True))
