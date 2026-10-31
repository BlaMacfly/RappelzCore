"""Bounded loopback-only MySQL relay that discards one selected real COMMIT OK.

Character-pool test traffic only. The backend handshake disables TLS/compression
on this isolated hop so framing can be observed; all other bytes pass unchanged.
Credentials/query payloads are never logged. Only a connection ID, exact COMMIT
and its actual server OK metadata are inspected; no response is synthesized.
"""
import asyncio
import hashlib
import json
from pathlib import Path
import struct
import time
from terrain_fault_proxy import abort_writers,write
from terrain_server_restart import validate,config_values

MAX_PACKET=1048576
MAX_CONNECTIONS=16


def proxy_settings(plan):
    _,root,_,cfg,world_port=validate(plan)
    if plan.get('ambiguous_commit') is not True or plan.get('money_level') is not True:raise ValueError('explicit isolated ambiguous currency scenario required')
    spec=plan['mysql_proxy'];listen=spec['listen_port'];upstream=spec['upstream_port']
    if any(type(v) is not int or not 1024<=v<65536 or v in (3310,4514,4615,world_port) for v in [listen,upstream]) or listen==upstream:raise ValueError('separate loopback database proxy ports required')
    c=config_values(cfg)
    for key,port in [('CharacterDatabaseInfo',listen),('LoginDatabaseInfo',upstream),('WorldDatabaseInfo',upstream)]:
        parts=c[key].split(';')
        if parts[0]!='127.0.0.1' or parts[1]!=str(port):raise ValueError('only the isolated character pool may use this proxy')
    return root,('127.0.0.1',listen),('127.0.0.1',upstream)


def handshake(payload):
    if not payload or payload[0]!=10:raise ValueError('MySQL v10 handshake required')
    end=payload.find(b'\0',1);cap=end+14
    if end<2 or len(payload)<cap+2:raise ValueError('truncated MySQL handshake')
    value=bytearray(payload);flags=int.from_bytes(value[cap:cap+2],'little')
    value[cap:cap+2]=(flags & ~(0x0800|0x0020)).to_bytes(2,'little')
    return bytes(value),int.from_bytes(payload[end+1:end+5],'little'),bool(flags&0x0800)


def is_commit(payload):return len(payload)<=64 and payload[:1]==b'\x03' and payload[1:].strip().rstrip(b';').strip().upper()==b'COMMIT'


def ok_status(payload):
    if not payload or payload[0]!=0:raise ValueError('actual COMMIT OK required')
    at=1
    for _ in range(2):
        if at>=len(payload):raise ValueError('truncated OK')
        first=payload[at];size=1 if first<251 else {252:3,253:4,254:9}.get(first,0)
        if not size or at+size>len(payload):raise ValueError('invalid length-encoded OK')
        at+=size
    if at+4>len(payload):raise ValueError('truncated OK status')
    status=int.from_bytes(payload[at:at+2],'little')
    if status&1:raise ValueError('COMMIT response still reports an open transaction')
    return status


async def read_packet(reader):
    header=await reader.readexactly(4);size=int.from_bytes(header[:3],'little')
    if not 0<size<=MAX_PACKET:raise ValueError('MySQL test packet limit exceeded')
    return header,await reader.readexactly(size)


class CommitRelay:
    def __init__(self,root,upstream):
        if upstream[0]!='127.0.0.1' or not 1024<=upstream[1]<65536 or upstream[1]==3310:raise ValueError('isolated loopback upstream required')
        self.root=Path(root);self.upstream=upstream;self.server=None;self.connections={};self.tasks=set();self.events=[];self.overflow=0;self.faulted=False;self.held=None
    def event(self,name,**fields):
        value=dict(event=name,unix_ms=time.time_ns()//1_000_000,**fields)
        if len(self.events)<256:self.events.append(value)
        else:self.overflow+=1
        return value
    async def start(self,port):
        self.server=await asyncio.start_server(self.accept,'127.0.0.1',port,limit=MAX_PACKET)
        return self.server.sockets[0].getsockname()[1]
    async def accept(self,reader,writer):
        if len(self.tasks)>=MAX_CONNECTIONS:
            writer.close();return
        task=asyncio.current_task();self.tasks.add(task);remote=None;pumps=[];connection=None
        try:
            remote_r,remote=await asyncio.open_connection(*self.upstream,limit=MAX_PACKET)
            header,payload=await read_packet(remote_r);payload,connection,tls=handshake(payload)
            if connection in self.connections:raise ValueError('duplicate backend connection ID')
            state=dict(pending=False,commit=None,writers=[writer,remote]);self.connections[connection]=state
            self.event('connected',connection_id=connection,tls_advertised_before=tls,tls_advertised_after=False)
            writer.write(header+payload);await writer.drain()
            async def upstream():
                while True:
                    h,p=await read_packet(reader)
                    if is_commit(p) and not self.faulted:
                        try:arm=json.loads((self.root/'commit-arm.json').read_text(encoding='utf-8'))
                        except (OSError,ValueError):arm={}
                        if arm.get('connection_id')==connection:
                            if h[3]!=0 or state['pending']:raise ValueError('unexpected COMMIT command sequence')
                            state['pending']=True;state['commit']=self.event('commit_forwarded',connection_id=connection,sequence=h[3])
                    remote.write(h+p);await remote.drain()
            async def downstream():
                while True:
                    h,p=await read_packet(remote_r)
                    if state['pending']:
                        if h[3]!=1:raise ValueError('unexpected COMMIT response sequence')
                        status=ok_status(p);self.held=self.event('commit_ok_held',connection_id=connection,sequence=h[3],status=status,packet_bytes=len(h)+len(p),packet_sha256=hashlib.sha256(h+p).hexdigest(),forwarded=False,commit_unix_ms=state['commit']['unix_ms'])
                        write(self.root/'commit-response-held.json',self.held)
                        deadline=time.monotonic()+25
                        while not (self.root/'discard-commit-response').exists():
                            if time.monotonic()>deadline:raise TimeoutError('held COMMIT response control timed out')
                            await asyncio.sleep(.002)
                        # The actual OK is discarded, never sent or replaced.
                        self.faulted=True;abort_writers(state['writers'])
                        event=self.event('commit_ok_discarded_connection_closed',connection_id=connection,forwarded=False,packet_bytes=len(h)+len(p))
                        write(self.root/'commit-response-reset.json',event);return
                    writer.write(h+p);await writer.drain()
            pumps=[asyncio.create_task(upstream()),asyncio.create_task(downstream())]
            done,_=await asyncio.wait(pumps,return_when=asyncio.FIRST_COMPLETED)
            for t in done:
                if t.exception():raise t.exception()
        except (asyncio.IncompleteReadError,ConnectionError,OSError):pass
        except Exception as error:self.event('error',kind=type(error).__name__)
        finally:
            for t in pumps:t.cancel()
            await asyncio.gather(*pumps,return_exceptions=True)
            for w in [writer,remote]:
                if w:
                    w.close()
                    try:await w.wait_closed()
                    except (ConnectionError,OSError):pass
            if connection is not None:self.connections.pop(connection,None)
            self.tasks.discard(task)
    async def close(self):
        if self.server:self.server.close();await self.server.wait_closed()
        for state in list(self.connections.values()):
            for writer in state['writers']:writer.close()
        for t in list(self.tasks):t.cancel()
        await asyncio.gather(*list(self.tasks),return_exceptions=True)
    def report(self):return dict(faulted=self.faulted,events=self.events,overflow=self.overflow,packet_limit=MAX_PACKET,connection_limit=MAX_CONNECTIONS,held=self.held,active_connections=len(self.connections))
