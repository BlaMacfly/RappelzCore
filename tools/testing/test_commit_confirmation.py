import copy
import unittest
from test_money_level_reward import fixture as money_fixture
from compare_gameplay_restart_clients import step,checkpoint
from compare_commit_confirmation_clients import compare


def fixture():
    reports,old,source,pictures,memory=money_fixture();base=source['checkpoint'];committed=source['reward']['committed'];blocked=source['reward']['blocked'][0]
    blocked['waits'][0].update(connection=70,transaction='99')
    arm=dict(connection_id=70,transaction='99',unix_ms=1150)
    held=dict(event='commit_ok_held',unix_ms=1201,connection_id=70,sequence=1,status=2,packet_bytes=11,packet_sha256='a'*64,forwarded=False,commit_unix_ms=1200)
    reset=dict(event='commit_ok_discarded_connection_closed',unix_ms=2200,connection_id=70,forwarded=False,packet_bytes=11)
    observations=[dict(profiles=copy.deepcopy(committed),client=dict(xp_jp=[799,7],level=2,money=1000,unix_ms=n),unix_ms=n) for n in [1400,2100]]
    reentry=copy.deepcopy(committed);reentry['witness'].update(health=80,online=0)
    saved=copy.deepcopy(source['saved']);saved['witness']['health']=90
    proof=dict(success=True,scenario='ambiguous-commit',before=base,blocked=blocked,arm=arm,held=held,reset=reset,committed=committed,unacknowledged=observations,reentry_profiles=reentry,verified_unix_ms=18201)
    proxy=dict(faulted=True,overflow=0,packet_limit=1048576,connection_limit=16,active_connections=0,held=held,events=[dict(event='connected',connection_id=70,tls_advertised_after=False),dict(event='commit_forwarded',connection_id=70,sequence=0),held,reset])
    control=dict(success=True,scenario='ambiguous-commit',boots=old['boots'][:1],world_alive_after_sql_loss=True,world_pid_after_sql_loss=10,final_exit_code=0,proxy=proxy)
    for role,r in reports.items():
        owner=role=='owner';r['ambiguous_commit']=True;t=r['trace']
        if owner:
            i=t['experience_orders'].index(14);t['experience_orders'].pop(i);t['experience'].pop(i)
            t['gold'].pop(1);t['gold_orders'].pop(1)
        else:t['network_errors']=[]
        t['levels']=[];t['level_orders']=[];r['events'].pop('PLAYER_LEVEL_UP',None);r['events']['PLAYER_MONEY']=3
        for i,l in enumerate(t['logins']):
            if i>0:l['handle']+=2
        for move in t['moves']:
            if move['order']>=20:move['handle']+=2
        t['logins'][1]['hp']=140 if owner else 80;t['logins'][2]['hp']=140 if owner else 90
        lost=copy.deepcopy(step(r,'BeforeInterruptedSave'));lost.update(stage='CommitResponseLost',monster=536870912);r['steps'].append(lost)
        if not owner:r['steps'].append(dict(stage='PeerLobbySaved',player=None))
    db=dict(proof=proof,saved=saved,console_save_requests=1,request=source['request'])
    drops=dict(owner=dict(joined=True,unix_ms=2300),witness=dict(joined=True,normal_lobby=True,unix_ms=3000))
    database=dict(before_pid=100,after_pid=100,port=3331,created_unix_ms=500)
    return reports,control,db,pictures,memory,drops,database


class CommitConfirmationTests(unittest.TestCase):
    def test_real_commit_ok_lost_and_durable_state_discovered_without_gain_replay(self):
        r=compare(*fixture());self.assertTrue(r['success'],[c for c in r['checks'] if not c['passed']]);self.assertEqual(len(r['checks']),47)
    def test_forwarded_unrelated_or_failed_commit_response_cannot_pass(self):
        for kind in ['forwarded','thread','status','sequence','duplicate','database']:
            with self.subTest(kind=kind):
                reports,c,db,p,m,drops,database=fixture();q=db['proof']
                if kind=='forwarded':q['held']['forwarded']=True
                elif kind=='thread':q['arm']['connection_id']=71
                elif kind=='status':q['held']['status']=3
                elif kind=='sequence':q['held']['sequence']=2
                elif kind=='duplicate':c['proxy']['events'].append(copy.deepcopy(q['held']))
                else:database['after_pid']=101
                self.assertFalse(compare(reports,c,db,p,m,drops,database)['success'])
    def test_stale_disconnect_overwrite_premature_gain_and_private_leak_fail(self):
        for kind in ['overwrite','announced','currency','leak','raw_hp','level']:
            with self.subTest(kind=kind):
                reports,c,db,p,m,drops,database=fixture();q=db['proof'];r=reports['owner']
                if kind=='overwrite':q['reentry_profiles']['owner']['money']=1000
                elif kind=='announced':r['trace']['experience'].insert(1,dict(handle=1,xp=4,jp=10));r['trace']['experience_orders'].insert(1,14)
                elif kind=='currency':r['trace']['gold'][1]['gold']=1016
                elif kind=='leak':reports['witness']['trace']['gold'][1]['gold']=1008
                elif kind=='raw_hp':r['trace']['logins'][1]['hp']=120
                else:r['events']['PLAYER_LEVEL_UP']=1
                self.assertFalse(compare(reports,c,db,p,m,drops,database)['success'])
    def test_normal_peer_exit_liveness_and_actual_sql_confirmation_are_required(self):
        for kind in ['peer_error','peer_transport','stale','no_commit','no_handshake','wrong_save']:
            with self.subTest(kind=kind):
                reports,c,db,p,m,drops,database=fixture();q=db['proof']
                if kind=='peer_error':reports['witness']['trace']['network_errors']=['unexpected']
                elif kind=='peer_transport':drops['witness']['normal_lobby']=False
                elif kind=='stale':q['unacknowledged'][0]['client']['unix_ms']-=151
                elif kind=='no_commit':q['unacknowledged'][0]['profiles']['owner']['xp']=799
                elif kind=='no_handshake':c['proxy']['events'].append(dict(event='error'))
                else:db['saved']['owner']['money']=1000
                self.assertFalse(compare(reports,c,db,p,m,drops,database)['success'])

if __name__=='__main__':unittest.main()
