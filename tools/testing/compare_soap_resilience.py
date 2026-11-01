"""Independent checks for real queued SOAP calls, revocation and connection expiry."""
import json
from pathlib import Path
import re
import sys


def compare(negative, positive, server, processes, native, soap, normal):
    checks = []
    def check(name, value): checks.append(dict(name=name, passed=bool(value)))
    def cases(report): return {c.get('name'): c for c in report.get('cases', [])}
    def status(case): return case.get('response', {}).get('status')
    def unchanged(case):
        before = case.get('before', '')
        return bool(re.fullmatch('[a-f0-9]{64}', before)) and before == case.get('after')
    def expired(case):
        return case.get('response', {}).get('error') in ['RemoteDisconnected', 'ConnectionResetError'] and 14 <= case.get('elapsed', -1) < 19.5 and case.get('queue_observed') is True and case.get('world_lock_observed') is True
    n, p = cases(negative), cases(positive)
    check('old-server control has all three reproduced failures', negative.get('legacy') is True and set(n) == {'invalid_realm', 'queued_gm', 'expired_shutdown'} and len(negative.get('cases', [])) == 3)
    check('old server treats a malformed realm as a global administrator grant', status(n.get('invalid_realm', {})) == 200 and n['invalid_realm'].get('access_rows') == '3\t-1' and not unchanged(n['invalid_realm']))
    check('old server executes an already queued revoked command', status(n.get('queued_gm', {})) == 200 and n['queued_gm'].get('target_count') == 1 and n['queued_gm'].get('world_lock_observed') is True)
    check('old server loses shutdown after actual HTTP expiry', expired(n.get('expired_shutdown', {})) and n['expired_shutdown'].get('exited_without_stdin') is False and n['expired_shutdown'].get('exit_code') == 0)
    expected = {'invalid_arguments_'+str(i) for i in range(7)} | {'extra_create_argument', 'truncated_body', 'rotated_password', 'timed_out_mutation', 'competing_soap_bind', 'expired_shutdown'} | {'queued_'+v for v in ['gm', 'ban', 'ip', 'password']}
    check('corrected server runs all seventeen distinct cases', positive.get('legacy') is False and set(p) == expected and len(positive.get('cases', [])) == len(expected))
    for i in range(7):
        c = p.get('invalid_arguments_'+str(i), {})
        check('invalid administrative arguments '+str(i)+' fail without changing the full target account/access rows', status(c) == 500 and unchanged(c))
    c = p.get('extra_create_argument', {})
    check('extra create arguments cannot create an account', status(c) == 500 and c.get('target_count') == 0)
    c = p.get('truncated_body', {})
    check('incomplete HTTP body never reaches account creation', c.get('target_count') == 0 and c.get('http', '').startswith(('HTTP/1.1 413', 'HTTP/1.1 400', 'closed')))
    for mode in ['gm', 'ban', 'ip', 'password']:
        c = p.get('queued_'+mode, {})
        check('queued '+mode+' change is rechecked before any command side effect', status(c) == (401 if mode == 'password' else 403) and c.get('target_count') == 0 and c.get('queue_observed') is True and c.get('world_lock_observed') is True)
    c = p.get('rotated_password', {})
    check('rotated SRP password invalidates the old password and accepts the new one', c.get('old_status') == 401 and c.get('new_status') == 200)
    c = p.get('timed_out_mutation', {})
    check('valid already queued mutation completes once after HTTP expiry', expired(c) and c.get('count_before_release') == 0 and c.get('target_count') == 1)
    c = p.get('expired_shutdown', {})
    check('accepted shutdown survives expired response without any stdin assistance', expired(c) and c.get('exit_code') == 0 and c.get('exited_without_stdin') is True and c.get('stdin_remained_open') is True)
    c = p.get('competing_soap_bind', {})
    check('SOAP bind failure preserves both active players and every online/realm flag', c.get('exit_code') not in [None, 0] and unchanged(c) and c.get('online_characters') == 2)
    check('two ordinary clients and the copied world finish normally', server.get('success') is True and server.get('exit_code') == 0 and not server.get('cleanup_clients') and not server.get('failed_attempt_shutdown') and len(processes) == 2 and {c.get('role') for c in processes} == {'owner', 'witness'} and all(c.get('exit_code') == 0 for c in processes))
    for name, report, count in [('native progression/base', native, 44), ('existing SOAP', soap, 25)]:
        check(name+' checks remain complete and passing', report.get('success') is True and len(report.get('checks', [])) == count and all(c.get('passed') is True for c in report['checks']))
    response = normal.get('response', {})
    check('ordinary SOAP shutdown still flushes its success reply before exit', normal.get('success') is True and normal.get('exit_code') == 0 and response.get('status') == 200 and 'Arrêt du serveur' in (response.get('result') or '') and normal.get('stdin_remained_open') is True)
    return dict(success=all(c['passed'] for c in checks), checks=checks)


def load_and_compare(directory):
    directory = Path(directory)
    read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
    return compare(read(directory.parent/'negative/resilience.json'), read(directory/'resilience.json'), read(directory/'server-process.json'), read(directory/'processes.json'), read(directory/'job-base-comparison.json'), read(directory/'soap-comparison.json'), read(directory.parent/'normal-shutdown.json'))


if __name__ == '__main__':
    directory = Path(sys.argv[1]); result = load_and_compare(directory)
    (directory/'resilience-comparison.json').write_text(json.dumps(result, indent=2), encoding='utf-8')
    print(json.dumps(dict(success=result['success'], checks=len(result['checks']), failed=[c['name'] for c in result['checks'] if not c['passed']])))
    raise SystemExit(0 if result['success'] else 1)
