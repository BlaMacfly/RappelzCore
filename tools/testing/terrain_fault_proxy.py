"""Opaque, bounded loopback relay for the dedicated native terrain-network scenario.

Never decodes or records payloads. Each direction holds at most one 64-KiB block
outside bounded asyncio/OS buffers. A pause stops forwarding; reset uses SO_LINGER
with zero timeout to exercise real remote connection failure, then accepts relogin.
Run: python terrain_fault_proxy.py <fresh-attempt-directory>
"""
import asyncio
import ipaddress
import json
import os
from pathlib import Path
import socket
import struct
import sys
import time

BLOCK=65536
ROLES=('owner','witness')

class Relay:
    def __init__(self,role,upstream=('127.0.0.1',4514)):
        if not ipaddress.ip_address(upstream[0]).is_loopback or not 0<upstream[1]<65536:
            raise ValueError('loopback upstream required')
        self.role=role;self.upstream=upstream;self.server=None;self.gate=asyncio.Event();self.gate.set()
        self.writers=[];self.tasks=set();self.events=[];self.overflow=0;self.generation=0
        self.counts={'up':0,'down':0};self.held={'up':0,'down':0};self.accepting=False
    def record(self,event,**fields):
        value=dict(event=event,role=self.role,connection=self.generation,ns=time.monotonic_ns(),bytes=self.counts.copy(),held=self.held.copy(),**fields)
        if len(self.events)<256:self.events.append(value)
        else:self.overflow+=1
        return value
    async def start(self):
        self.server=await asyncio.start_server(self.accept,'127.0.0.1',0,limit=BLOCK)
        return self.server.sockets[0].getsockname()[1]
    async def accept(self,reader,writer):
        if self.accepting:
            writer.close();await writer.wait_closed();self.record('rejected_concurrent');return
        self.accepting=True;self.generation+=1;self.counts={'up':0,'down':0};self.held={'up':0,'down':0};self.gate.set()
        connection=self.generation
        try:
            remote_r,remote_w=await asyncio.open_connection(*self.upstream,limit=BLOCK)
            self.writers=[writer,remote_w];self.record('connected')
            self.tasks={asyncio.create_task(self.pipe(reader,remote_w,'up')),asyncio.create_task(self.pipe(remote_r,writer,'down'))}
            await asyncio.wait(self.tasks,return_when=asyncio.FIRST_COMPLETED)
        except (ConnectionError,OSError) as e:self.record('connection_error',kind=type(e).__name__)
        finally:
            for t in self.tasks:t.cancel()
            await asyncio.gather(*self.tasks,return_exceptions=True)
            for w in self.writers:
                w.close()
                try:await w.wait_closed()
                except (ConnectionError,OSError):pass
            writer.close()
            self.tasks=set();self.writers=[];self.accepting=False
            self.record('closed',closed_connection=connection)
    async def pipe(self,reader,writer,direction):
        try:
            while True:
                await self.gate.wait()
                data=await reader.read(BLOCK)
                if not data:return
                self.held[direction]=len(data)
                await self.gate.wait()
                # No await between the gate check and write/accounting: pause has a
                # precise forwarding boundary on this single event loop.
                writer.write(data);self.counts[direction]+=len(data);self.held[direction]=0
                await writer.drain()
        except (ConnectionError,OSError) as e:self.record('pipe_error',direction=direction,kind=type(e).__name__)
        finally:self.held[direction]=0
    def command(self,action):
        if not self.writers:raise RuntimeError('live connection required')
        if action=='pause':self.gate.clear()
        elif action=='resume':self.gate.set()
        elif action=='reset':
            # Windows linger uses two unsigned shorts; Unix uses two ints.
            linger=struct.pack('HH' if os.name=='nt' else 'ii',1,0)
            for w in self.writers:
                transport_socket=w.get_extra_info('socket')
                transport_socket.setsockopt(socket.SOL_SOCKET,socket.SO_LINGER,linger)
                # Windows Proactor transport.abort() calls shutdown(SHUT_RDWR)
                # before close, producing FIN even with linger zero. Close the
                # pending I/O first, then close the owned socket synchronously
                # before its scheduled connection_lost callback can call
                # shutdown. The callback sees fileno -1 and cannot send FIN.
                w.transport.abort()
                transport_socket._sock.close()
        else:raise ValueError('unknown relay action')
        return self.record(action,abortive=action=='reset')
    async def close(self):
        if self.server:self.server.close();await self.server.wait_closed()
        self.gate.set()
        for w in self.writers:w.close()
        for _ in range(100):
            if not self.accepting:return
            await asyncio.sleep(.01)
        raise RuntimeError('relay connection did not close')
    def report(self):return dict(role=self.role,limit=256,overflow=self.overflow,block_limit=BLOCK,events=self.events)

def write(path,value):
    temporary=path.with_suffix(path.suffix+'.tmp');temporary.write_text(json.dumps(value,indent=2),encoding='utf-8');temporary.replace(path)

async def run(root):
    root=Path(root);assert root.is_dir() and not (root/'proxy-ready.json').exists(),'fresh directory required'
    relays={r:Relay(r) for r in ROLES};seen=set();completed=False
    try:
        ports={r:await relay.start() for r,relay in relays.items()};write(root/'proxy-ready.json',ports)
        deadline=time.monotonic()+240
        while not (root/'proxy-stop').exists():
            if time.monotonic()>deadline:raise TimeoutError('relay lifetime exceeded')
            for role,relay in relays.items():
                for cycle in (0,1):
                    for action in ('pause','resume','reset'):
                        name=f'{role}-{action}-{cycle}'
                        if name not in seen and (root/(name+'.request')).exists():
                            ack=relay.command(action);seen.add(name);write(root/(name+'.ack'),ack)
            await asyncio.sleep(.002)
        completed=True
    finally:
        await asyncio.gather(*(relay.close() for relay in relays.values()))
        write(root/'proxy.json',dict(success=completed,relays={r:relay.report() for r,relay in relays.items()}))

if __name__=='__main__':asyncio.run(run(sys.argv[1]))
