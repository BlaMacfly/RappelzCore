import asyncio
import json
from pathlib import Path
import struct
import tempfile
import unittest
from unittest.mock import patch
from mysql_commit_proxy import CommitRelay,handshake,is_commit,ok_status,read_packet,MAX_PACKET,proxy_settings
import test_terrain_server_restart as restart_tests


def packet(payload,sequence=0):return len(payload).to_bytes(3,'little')+bytes([sequence])+payload

def hello(connection=7):return b'\x0a5.7-local-test\0'+struct.pack('<I',connection)+b'12345678\0'+b'\xff\xff'+b'\x21\x02\0\xff\xff\x15'+b'\0'*10+b'abcdefgh1234\0mysql_native_password\0'

class MySQLCommitPacketTests(unittest.TestCase):
    def test_handshake_changes_only_local_tls_compression_flags_and_extracts_real_id(self):
        original=hello();changed,connection,was_tls=handshake(original);end=original.find(b'\0',1);at=end+14
        self.assertEqual(connection,7);self.assertTrue(was_tls)
        self.assertEqual(original[:at],changed[:at]);self.assertEqual(original[at+2:],changed[at+2:])
        self.assertEqual(int.from_bytes(changed[at:at+2],'little'),0xffff & ~(0x0800|0x0020))
        for invalid in [b'',b'\x09old',b'\x0ano terminator',b'\x0aa\0']:
            with self.assertRaises(ValueError):handshake(invalid)
    def test_only_commit_and_complete_success_outside_transaction_are_accepted(self):
        for value in [b'\x03COMMIT',b'\x03 commit; ']:self.assertTrue(is_commit(value))
        for value in [b'\x03SELECT 1',b'\x03COMMIT;ROLLBACK',b'COMMIT',b'\x17COMMIT']:self.assertFalse(is_commit(value))
        self.assertEqual(ok_status(b'\0\0\0\x02\0\0\0'),2)
        for value in [b'',b'\xfferror',b'\0\0',b'\0\0\0\x03\0\0\0']:
            with self.assertRaises(ValueError):ok_status(value)
    def test_proxy_route_rejects_main_ports_shared_ports_remote_databases_and_wrong_pool(self):
        for kind in ['valid','main','shared','wrong_pool','remote']:
            with self.subTest(kind=kind),tempfile.TemporaryDirectory() as temp:
                p=restart_tests.ServerRestartControlTests().plan(Path(temp));cfg=Path(p['config']);s=cfg.read_text(encoding='utf-8')
                s=s.replace('CharacterDatabaseInfo = "127.0.0.1;3331;', 'CharacterDatabaseInfo = "127.0.0.1;3332;')
                p.update(ambiguous_commit=True,money_level=True,mysql_proxy=dict(listen_port=3332,upstream_port=3331))
                if kind=='main':p['mysql_proxy']['upstream_port']=3310
                elif kind=='shared':p['mysql_proxy']['listen_port']=3331
                elif kind=='wrong_pool':s=s.replace('WorldDatabaseInfo = "127.0.0.1;3331;', 'WorldDatabaseInfo = "127.0.0.1;3332;')
                elif kind=='remote':s=s.replace('127.0.0.1','192.168.1.2')
                cfg.write_text(s,encoding='utf-8')
                if kind=='valid':self.assertEqual(proxy_settings(p)[2],('127.0.0.1',3331))
                else:
                    with self.assertRaises(ValueError):proxy_settings(p)


class MySQLCommitSocketTests(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self):
        self.temp=tempfile.TemporaryDirectory();self.root=Path(self.temp.name);self.received=[];self.responses=[];self.remote=[];self.commit_error=False;self.greeting_gate=asyncio.Event();self.greeting_gate.set()
        async def backend(reader,writer):
            self.remote.append(writer)
            try:
                await self.greeting_gate.wait()
                writer.write(packet(hello(),0));await writer.drain()
                _,p=await read_packet(reader);self.received.append(p);writer.write(packet(b'\0\0\0\x02\0\0\0',2));await writer.drain()
                while True:
                    _,p=await read_packet(reader);self.received.append(p)
                    response=b'\xff\x15\x04error' if is_commit(p) and self.commit_error else b'\0\0\0\x02\0\0\0'
                    self.responses.append(response);writer.write(packet(response,1));await writer.drain()
            except (asyncio.IncompleteReadError,ConnectionError,OSError):pass
            finally:
                writer.close()
                try:await writer.wait_closed()
                except (ConnectionError,OSError):pass
        self.server=await asyncio.start_server(backend,'127.0.0.1',0)
        self.relay=CommitRelay(self.root,('127.0.0.1',self.server.sockets[0].getsockname()[1]));self.port=await self.relay.start(0);self.clients=[]
    async def connect(self):
        reader,writer=await asyncio.open_connection('127.0.0.1',self.port);self.clients.append(writer);_,h=await read_packet(reader)
        self.assertFalse(int.from_bytes(h[h.find(b'\0',1)+14:][:2],'little')&0x0800)
        writer.write(packet(b'unlogged authentication payload',1));await writer.drain();await read_packet(reader)
        return reader,writer
    async def asyncTearDown(self):
        for writer in self.clients:
            writer.close()
            try:await writer.wait_closed()
            except (ConnectionError,OSError):pass
        await self.relay.close();self.server.close();await self.server.wait_closed();self.temp.cleanup()
    async def test_real_selected_commit_ok_is_held_then_discarded_while_other_bytes_are_unchanged(self):
        reader,writer=await self.connect();query=b'\x03SELECT unlogged_payload';writer.write(packet(query));await writer.drain();self.assertEqual((await read_packet(reader))[1],b'\0\0\0\x02\0\0\0')
        (self.root/'commit-arm.json').write_text(json.dumps(dict(connection_id=7)))
        writer.write(packet(b'\x03COMMIT'));await writer.drain()
        with self.assertRaises(asyncio.TimeoutError):await asyncio.wait_for(reader.read(1),.08)
        self.assertTrue((self.root/'commit-response-held.json').exists());held=json.loads((self.root/'commit-response-held.json').read_text());self.assertEqual(held['connection_id'],7);self.assertFalse(held['forwarded'])
        (self.root/'discard-commit-response').write_text('verified')
        try:self.assertEqual(await asyncio.wait_for(reader.read(1),2),b'')
        except (ConnectionResetError,ConnectionAbortedError):pass
        self.assertEqual(self.received[-1],b'\x03COMMIT');self.assertEqual(self.responses[-1],b'\0\0\0\x02\0\0\0');self.assertTrue(self.relay.faulted)
        self.assertNotIn('unlogged',json.dumps(self.relay.report()))
    async def test_wrong_connection_commit_is_forwarded_and_server_error_is_never_an_ok(self):
        reader,writer=await self.connect();(self.root/'commit-arm.json').write_text(json.dumps(dict(connection_id=999)))
        writer.write(packet(b'\x03COMMIT'));await writer.drain();self.assertEqual((await read_packet(reader))[1],b'\0\0\0\x02\0\0\0');self.assertFalse(self.relay.faulted)
        self.commit_error=True;(self.root/'commit-arm.json').write_text(json.dumps(dict(connection_id=7)))
        writer.write(packet(b'\x03COMMIT'));await writer.drain()
        self.assertEqual(await asyncio.wait_for(reader.read(1),2),b'');self.assertFalse((self.root/'commit-response-held.json').exists())
        self.assertTrue(any(e['event']=='error' for e in self.relay.events))
    async def test_connection_budget_includes_handshakes_waiting_for_upstream(self):
        self.greeting_gate.clear()
        with patch('mysql_commit_proxy.MAX_CONNECTIONS',1):
            r1,w1=await asyncio.open_connection('127.0.0.1',self.port);self.clients.append(w1)
            for _ in range(100):
                if self.remote:break
                await asyncio.sleep(.01)
            self.assertEqual(len(self.relay.tasks),1);self.assertEqual(len(self.relay.connections),0)
            r2,w2=await asyncio.open_connection('127.0.0.1',self.port);self.clients.append(w2)
            self.assertEqual(await asyncio.wait_for(r2.read(1),1),b'');self.assertEqual(len(self.remote),1)
            self.greeting_gate.set();await asyncio.wait_for(read_packet(r1),1)
    async def test_oversized_packet_is_rejected_before_allocating_its_payload(self):
        reader=asyncio.StreamReader();reader.feed_data((MAX_PACKET+1).to_bytes(3,'little')+b'\0')
        with self.assertRaises(ValueError):await read_packet(reader)

if __name__=='__main__':unittest.main()
