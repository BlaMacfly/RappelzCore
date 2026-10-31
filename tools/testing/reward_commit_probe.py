"""Hold a real reward save, prove notifications wait for COMMIT, then check crash retention.

Uses the same isolated database/fixture guards and owned settings lock as the
interrupted-save probe. No manual save is requested after combat.
"""
import json
from pathlib import Path
import sys
import time
from character_save_probe import Database,SettingsLock,ROLES
from terrain_server_restart import wait_until,write


def pending(root):
    try:return json.loads((root/'owner.pending-reward.json').read_text(encoding='utf-8'))
    except (OSError,ValueError):return {}


def withheld(before,committed,dirty,waits,client,now_ms,money_level=False):
    if money_level:
        if set(dirty)!=set(ROLES):return False
        d=dirty["owner"];gain=d.get("money",0)-before["owner"]["money"]
        expected=dict(before["owner"],level=3,xp=4,money=d.get("money"),health=140,mp=70)
        return (committed==before and len(waits)==1 and waits[0].get("modified",0)>0
            and "replace into character_settings" in waits[0].get("query","").lower()
            and client.get("xp_jp")==[799,7] and client.get("level")==2 and client.get("money")==1000
            and 0<=now_ms-client.get("unix_ms",0)<=150 and 8<=gain<=14 and d==expected
            and dirty["witness"]==before["witness"])
    return (committed==before and len(waits)==1 and waits[0].get('modified',0)>0
        and 'replace into character_settings' in waits[0].get('query','').lower()
        and client.get('xp_jp')==[0,0] and 0<=now_ms-client.get('unix_ms',0)<=150
        and dirty.get('witness')==before.get('witness')
        and dirty.get('owner',{}).get('xp')==5 and dirty['owner'].get('jp')==0
        and 0<dirty['owner'].get('health',0)<=100)


def awarded(before,profiles,money_level=False):
    if money_level:
        if set(profiles)!=set(ROLES) or profiles['witness']!=before['witness']:return False
        d=profiles['owner'];expected=dict(before['owner'],level=3,xp=4,jp=10,money=d.get('money'),health=140,mp=70)
        expected['settings']=dict(before['owner']['settings'],jp='10')
        return d==expected and 8<=d.get('money',0)-before['owner']['money']<=14
    if set(profiles)!=set(ROLES) or profiles['witness']!=before['witness']:return False
    expected=dict(before['owner'],xp=5,jp=3,health=profiles['owner'].get('health'))
    expected['settings']=dict(before['owner']['settings'],jp='3')
    return profiles['owner']==expected and 0<profiles['owner'].get('health',0)<=100


def run(plan,probe):
    if plan.get('scenario')!='reward-durable':raise ValueError('reward-durable scenario required')
    db=Database(plan,probe);root=db.root;money_level=plan.get('money_level') is True;result=dict(success=False,method='reward COMMIT before notification',money_level=money_level)
    try:
        wait_until(lambda:(root/'checkpoint-verified').exists() and (root/'release-initial').exists(),100,'durable baseline and initial capture missing')
        before=db.profiles();checkpoint=json.loads((root/'checkpoint-database.json').read_text(encoding='utf-8'))
        if before!=checkpoint:raise RuntimeError('pre-combat checkpoint changed')
        result['before']=before
        with SettingsLock(db) as lock:
            write(root/'reward-locks-ready',dict(connection=lock.connection,guids=db.guids))
            def observe():
                if (root/'owner.save-ready').exists():
                    result['premature_notification']=dict(locks_held=True,connection=lock.connection,committed=db.profiles())
                    raise RuntimeError('reward announced before the locked save could commit')
                waits=db.waits(lock.connection);committed=db.profiles();dirty=db.profiles(True);client=pending(root);now=time.time_ns()//1_000_000
                return dict(waits=waits,committed=committed,dirty=dirty,client=client,unix_ms=now) if withheld(before,committed,dirty,waits,client,now,money_level) else None
            first=wait_until(observe,35,'reward not held before notification')
            observations=[first];end=time.monotonic()+.6
            while time.monotonic()<end:
                time.sleep(.05);v=observe()
                if not v:raise RuntimeError('reward became visible while commit was blocked')
                observations.append(v)
            result.update(before=before,blocked=observations,held_ms=observations[-1]['unix_ms']-first['unix_ms'])
        result['released_unix_ms']=time.time_ns()//1_000_000
        wait_until(lambda:all((root/(r+'.save-ready')).exists() for r in ROLES),10,'reward not received after lock release')
        def committed_reward():
            profiles=db.profiles()
            return profiles if awarded(before,profiles,money_level) else None
        profiles=wait_until(committed_reward,8,'announced reward not fully committed')
        write(root/'reward-committed.json',profiles)
        server=json.loads((root/'server-up-0.json').read_text(encoding='utf-8'))
        marker=dict(pid=server['pid'],unix_ms=time.time_ns()//1_000_000,committed=True,profiles=profiles)
        result.update(committed=profiles,window=marker)
        write(root/'reward-durable-verified',marker)
        wait_until(lambda:(root/'server-killed.json').exists(),5,'real crash after acknowledged reward missing')
        after=db.profiles()
        if after!=profiles:raise RuntimeError('committed reward changed during process crash')
        result.update(success=True,after=after,verified_after_kill_unix_ms=time.time_ns()//1_000_000)
        write(root/'reward-commit.json',result);write(root/'reward-crash-verified',dict(success=True))
    except Exception as error:
        result['failure']=str(error);raise
    finally:write(root/'reward-commit.json',result)

if __name__=='__main__':run(*[json.loads(Path(p).read_text(encoding='utf-8-sig')) for p in sys.argv[1:3]])
