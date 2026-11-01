import copy
import unittest
from compare_soap_resilience import compare


def fixture():
    expired = dict(response={'error': 'RemoteDisconnected'}, elapsed=15.01, queue_observed=True, world_lock_observed=True)
    n = dict(legacy=True, cases=[dict(name='invalid_realm', response={'status': 200}, access_rows='3\t-1', before='a'*64, after='b'*64), dict(name='queued_gm', response={'status': 200}, target_count=1, world_lock_observed=True), dict(expired, name='expired_shutdown', exited_without_stdin=False, exit_code=0)])
    cases = [dict(name='invalid_arguments_'+str(i), response={'status': 500}, before='a'*64, after='a'*64) for i in range(7)]
    cases += [dict(name='queued_'+m, response={'status': 401 if m == 'password' else 403}, target_count=0, queue_observed=True, world_lock_observed=True) for m in ['gm', 'ban', 'ip', 'password']]
    cases += [dict(name='extra_create_argument', response={'status': 500}, target_count=0), dict(name='truncated_body', http='HTTP/1.1 413 Payload Too Large', target_count=0), dict(name='rotated_password', old_status=401, new_status=200), dict(expired, name='timed_out_mutation', count_before_release=0, target_count=1), dict(expired, name='expired_shutdown', exited_without_stdin=True, stdin_remained_open=True, exit_code=0), dict(name='competing_soap_bind', before='a'*64, after='a'*64, exit_code=1, online_characters=2)]
    p = dict(legacy=False, cases=cases)
    server = dict(success=True, exit_code=0)
    processes = [dict(role=r, exit_code=0) for r in ['owner', 'witness']]
    report = lambda count: dict(success=True, checks=[{'passed': True} for _ in range(count)])
    normal = dict(success=True, exit_code=0, response=dict(status=200, result='Arrêt du serveur demandé.'), stdin_remained_open=True)
    return [n, p, server, processes, report(44), report(25), normal]


class SoapResilienceComparison(unittest.TestCase):
    def test_accepts_complete_realistic_evidence(self):
        self.assertTrue(compare(*fixture())['success'])

    def test_rejects_a_negative_control_that_does_not_reproduce_the_bug(self):
        args=fixture();args[0]['cases'][1]['target_count']=0
        self.assertFalse(compare(*args)['success'])

    def test_rejects_queued_access_changes_with_any_side_effect(self):
        for mode in ['gm', 'ban', 'ip', 'password']:
            args=fixture();c=next(c for c in args[1]['cases'] if c['name']=='queued_'+mode);c['target_count']=1
            self.assertFalse(compare(*args)['success'])

    def test_rejects_missing_queue_or_lock_and_unexpired_response(self):
        for field, value in [('queue_observed', False), ('world_lock_observed', False), ('elapsed', 0.1), ('response', {'error': 'TimeoutError'})]:
            args=fixture();c=next(c for c in args[1]['cases'] if c['name']=='expired_shutdown');c[field]=value
            self.assertFalse(compare(*args)['success'])

    def test_rejects_stdin_assistance_online_flag_change_and_incomplete_evidence(self):
        for name, field, value in [('expired_shutdown', 'exited_without_stdin', False), ('competing_soap_bind', 'after', 'b'*64), ('competing_soap_bind', 'online_characters', 0)]:
            args=fixture();next(c for c in args[1]['cases'] if c['name']==name)[field]=value
            self.assertFalse(compare(*args)['success'])
        args=fixture();args[1]['cases'].pop();self.assertFalse(compare(*args)['success'])
        args=fixture();args[4]['checks'].pop();self.assertFalse(compare(*args)['success'])
