"""Own the copied worldserver and a narrowly armed local SQL response relay."""
import asyncio
import json
from pathlib import Path
import socket
import sys
from mysql_commit_proxy import CommitRelay,proxy_settings
from terrain_server_restart import validate,boot,write,wait_until


async def run(plan):
    _,root,exe,cfg,port=validate(plan);_,listen,upstream=proxy_settings(plan);root.mkdir(exist_ok=True)
    if (root/'server-up-0.json').exists():raise ValueError('fresh attempt required')
    with socket.socket() as probe:probe.bind(('127.0.0.1',port))
    relay=CommitRelay(root,upstream);world=None;record=dict(success=False,scenario='ambiguous-commit',boots=[],money_level=True,ambiguous_commit=True,executable_sha256=plan['executable_sha256'])
    try:
        await relay.start(listen[1]);write(root/'mysql-proxy-ready.json',dict(listen=list(listen),upstream=list(upstream)))
        world,info=await asyncio.to_thread(boot,exe,cfg,root,port,0);record['boots'].append(info)
        await asyncio.to_thread(wait_until,lambda:all((root/(r+'.checkpoint-ready')).exists() for r in ['owner','witness']),70,'checkpoint movement missing')
        world.stdin.write(b'saveall\n');world.stdin.flush();write(root/'checkpoint-requested.json',dict(pid=world.pid,unix_ms=__import__('time').time_ns()//1_000_000,method='console saveall'))
        await asyncio.to_thread(wait_until,lambda:(root/'checkpoint-verified').exists() and (root/'release-initial').exists() and (root/'reward-locks-ready').exists(),35,'checkpoint and lock verification missing')
        write(root/'actions-release',dict(ready=True))
        def verified():
            if (root/'ambiguous-commit.json').exists():
                value=json.loads((root/'ambiguous-commit.json').read_text(encoding='utf-8'))
                if not value.get('success'):raise RuntimeError('ambiguous COMMIT probe failed')
                return value
        record['proof']=await asyncio.to_thread(wait_until,verified,75,'successful-but-unacknowledged COMMIT not verified')
        if world.poll() is not None:raise RuntimeError('worldserver stopped')
        record['world_alive_after_sql_loss']=True;record['world_pid_after_sql_loss']=world.pid
        await asyncio.to_thread(wait_until,lambda:(root/'server-stop').exists(),100,'final client entries did not finish')
        world.stdin.write(b'server shutdown\n');world.stdin.flush();record['final_exit_code']=await asyncio.to_thread(world.wait,12)
        if record['final_exit_code']!=0:raise RuntimeError('world shutdown failed')
        record['success']=True
    finally:
        if world and world.poll() is None:world.kill();await asyncio.to_thread(world.wait,5);record['cleanup_kill']=True
        await relay.close();record['proxy']=relay.report();write(root/'server-process.json',record)

if __name__=='__main__':asyncio.run(run(json.loads(Path(sys.argv[1]).read_text(encoding='utf-8-sig'))))
