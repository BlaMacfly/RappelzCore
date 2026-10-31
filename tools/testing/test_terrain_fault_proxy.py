import asyncio
import copy
import json
import unittest
from terrain_fault_proxy import Relay,BLOCK

class RelayTests(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self):
        self.remote=[]
        async def echo(reader,writer):
            self.remote.append(writer)
            try:
                while data:=await reader.read(BLOCK):writer.write(data);await writer.drain()
            except (ConnectionError,OSError):pass
            finally:
                writer.close()
                try:await writer.wait_closed()
                except (ConnectionError,OSError):pass
        self.server=await asyncio.start_server(echo,'127.0.0.1',0)
        self.relay=Relay('owner',('127.0.0.1',self.server.sockets[0].getsockname()[1]))
        self.port=await self.relay.start();self.clients=[]
    async def connect(self):
        reader,writer=await asyncio.open_connection('127.0.0.1',self.port);self.clients.append(writer)
        writer.write(b'hello');await writer.drain();self.assertEqual(await asyncio.wait_for(reader.readexactly(5),2),b'hello')
        return reader,writer
    async def asyncTearDown(self):
        for w in self.clients:
            w.close()
            try:await w.wait_closed()
            except (ConnectionError,OSError):pass
        await self.relay.close();self.server.close();await self.server.wait_closed()
    async def test_pause_holds_both_directions_then_preserves_byte_order(self):
        reader,writer=await self.connect();start=self.relay.command('pause')
        writer.write(b'A');await writer.drain();self.remote[-1].write(b'B');await self.remote[-1].drain()
        with self.assertRaises(asyncio.TimeoutError):await asyncio.wait_for(reader.read(1),.08)
        self.assertEqual(self.relay.counts,start['bytes']);self.assertTrue(all(0<=n<=BLOCK for n in self.relay.held.values()))
        self.relay.command('resume');self.assertEqual(await asyncio.wait_for(reader.readexactly(2),2),b'BA')
        self.assertEqual(self.relay.counts,dict(up=6,down=7))
    async def test_abortive_reset_closes_real_socket_and_fresh_stream_reconnects(self):
        reader,_=await self.connect();self.relay.command('reset')
        with self.assertRaises((ConnectionResetError,ConnectionAbortedError)):
            await asyncio.wait_for(reader.read(1),2)
        for _ in range(100):
            if not self.relay.accepting:break
            await asyncio.sleep(.01)
        self.assertFalse(self.relay.accepting)
        reader,writer=await self.connect();writer.write(b'new');await writer.drain()
        self.assertEqual(await asyncio.wait_for(reader.readexactly(3),2),b'new');self.assertEqual(self.relay.generation,2)
        self.assertEqual(self.relay.counts,dict(up=8,down=8))
    async def test_only_loopback_destinations_and_live_known_controls_are_allowed(self):
        for endpoint in [('0.0.0.0',4514),('192.168.1.1',4514),('127.0.0.1',0)]:
            with self.assertRaises(ValueError):Relay('owner',endpoint)
        with self.assertRaises(RuntimeError):self.relay.command('pause')
        await self.connect()
        with self.assertRaises(ValueError):self.relay.command('delete')
    async def test_audit_is_bounded_and_contains_metadata_only(self):
        for _ in range(260):self.relay.record('probe')
        r=self.relay.report();self.assertEqual(len(r['events']),256);self.assertEqual(r['overflow'],4)
        for event in r['events']:self.assertEqual(set(event),{'event','role','connection','ns','bytes','held'})
        self.assertNotIn('payload',json.dumps(r))

if __name__=='__main__':unittest.main()
