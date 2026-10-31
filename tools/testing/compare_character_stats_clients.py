"""Independently check private combat stats, actual XP combat and SQL reentry."""
import json
import re
import sys
from pathlib import Path

ROLES = ('owner', 'witness')


def step(report, name):
    return next((s for s in report.get('steps', []) if s.get('stage') == name), {})


def compare(reports, server, processes, sql, negative):
    checks = []

    def check(name, value):
        checks.append(dict(name=name, passed=bool(value)))

    owner = reports.get('owner', {})
    witness = reports.get('witness', {})
    check('owned world and both native clients exit normally',
          server.get('success') is True and server.get('scenario') == 'character-stats'
          and server.get('world_pid', 0) > 0 and server.get('exit_code') == 0
          and not server.get('cleanup_clients') and not server.get('failed_attempt_shutdown')
          and len(processes) == 2 and {p.get('role') for p in processes} == set(ROLES)
          and all(p.get('exit_code') == 0 and p.get('pid', 0) > 0 for p in processes))
    check('independent SQL gates precede combat and persisted reentry',
          [c.get('name') for c in server.get('checkpoints', [])] == ['queried', 'saved']
          and set(sql) == {'queried', 'saved', 'final'})
    handles = [owner.get('initial'), witness.get('initial'), owner.get('own'), witness.get('own')]
    check('both peers have distinct fresh handles', all(handles) and len(set(handles)) == 4
          and owner.get('initial') == witness.get('initial_other')
          and witness.get('initial') == owner.get('initial_other')
          and owner.get('own') == witness.get('other') and witness.get('own') == owner.get('other'))
    hits = owner.get('trace', {}).get('hits', [])
    body = owner.get('body')
    check('both clients observe the same real combat and exactly one kill',
          body and body == witness.get('body') and len(hits) >= 2
          and hits == witness.get('trace', {}).get('hits')
          and {h.get('attacker') for h in hits} == {owner.get('initial'), body}
          and all(h.get('damage', 0) > 0 for h in hits)
          and sum(h.get('attacker') == owner.get('initial') and h.get('target') == body
                  and h.get('target_hp') == 0 for h in hits) == 1)
    control = negative.get('server', {})
    check('old backend is a genuine negative control with normal shutdown',
          control.get('negative_control') is True and control.get('expected_rejection') is True
          and control.get('success') is False and control.get('exit_code') == 0
          and control.get('executable_sha256') == '7afad0417e19f255426d2faf3c4ba06be3a60616358938add36136df1a92d380'
          and server.get('executable_sha256') != control.get('executable_sha256')
          and len(negative.get('processes', [])) == 2
          and {p.get('role') for p in negative['processes']} == set(ROLES)
          and all(p.get('exit_code') == 1 for p in negative['processes']))
    for role in ROLES:
        r = reports.get(role, {})
        t = r.get('trace', {})
        own = role == 'owner'
        initial, handle = r.get('initial'), r.get('own')
        before = dict(attack=20 if own else 16, defence=8 if own else 6, speed=120)
        after = dict(attack=24 if own else 16, defence=10 if own else 6, speed=120)
        stats = t.get('stats', [])
        expected = [(initial, before)] * 3 + ([(initial, after)] if own else []) + [(handle, after)] * 2
        check(role + ': exact full private wire snapshots include duplicate queries and reentry',
              len(stats) == len(expected) and all(
                  s.get('target') == h and {k: s.get(k) for k in v} == v
                  and s.get('kind') == 0 and s.get('bytes') == 110 for s, (h, v) in zip(stats, expected)))
        logins = t.get('logins', [])
        orders = [s.get('order', 0) for s in stats]
        check(role + ': packets are ordered after login and the committed public level',
              len(logins) == 2 and len(stats) == len(expected)
              and logins[0].get('order', 0) < orders[0] < orders[1] < orders[2] < logins[1].get('order', 0) < orders[-2] < orders[-1]
              and (not own or len(t.get('level_orders', [])) == 1 and orders[2] < t['level_orders'][0] < orders[3] < logins[1]['order']))
        check(role + ': production self refreshes and foreign probes use the intended identity',
              r.get('requests') == [dict(label=label, target=target, helper=helper) for label, target, helper in
                                   [('refresh1', initial, True), ('refresh2', initial, True),
                                    ('peer_query', r.get('initial_other'), False), ('peer_query', r.get('initial_other'), False),
                                    ('reentry_refresh', handle, True)]])
        check(role + ': bounded driver completes two real Lua lifecycles',
              r.get('success') is True and not r.get('failure') and r.get('scenario') == 'character-stats'
              and step(r, 'Logout').get('elapsed_ms', 10**9) < 120000
              and all(r.get('events', {}).get(k) == 2 for k in ['PLAYER_LOGIN', 'PLAYER_ENTERING_WORLD', 'PLAYER_LOGOUT']))
        check(role + ': no decoding errors casts learning warps or player movement',
              all(t.get(k) == [] for k in ['errors', 'network_errors', 'disconnects', 'cast_events', 'learn_results', 'job_results', 'warps'])
              and all(m.get('handle') == body for m in t.get('moves', [])))
        check(role + ': exactly one public level update is observed before reentry',
              t.get('levels') == [dict(handle=owner.get('initial'), job_level=1, level=3)]
              and len(t.get('level_orders', [])) == 1 and len(logins) == 2
              and t['level_orders'][0] < logins[1].get('order', 0))
        tuples = [tuple(map(int, m)) for line in r.get('messages', []) for m in
                  re.findall(r'EVENT RAPPELZ_COMBAT_STATS_CHANGED .*?attack=(\d+) defence=(\d+) speed=(\d+)', line)]
        check(role + ': Lua emits complete changed tuples once and suppresses refresh duplicates',
              r.get('events', {}).get('RAPPELZ_COMBAT_STATS_CHANGED') == (3 if own else 2)
              and tuples == [(v['attack'], v['defence'], v['speed']) for v in ([before, after, after] if own else [before, after])]
              and r.get('events', {}).get('PLAYER_LEVEL_UP', 0) == (1 if own else 0))
        check(role + ': own adapter and rendered addon agree at every checkpoint',
              all(step(r, s).get('stats') == v and f"AP: {v['attack']}  DEF: {v['defence']}" in step(r, s).get('display', '')
                  for s, v in [('Initial', before), ('Queried', before), ('BeforeLobby', after), ('Reentered', after), ('Saved', after)]))
        position = [9440, 7227 if own else 7247]
        hp, mp = (140, 70) if own else (100, 50)
        money = step(owner, 'Saved').get('money', 0) if own else 2000
        check(role + ': raw reentry vitals and position retain the saved level',
              len(logins) == 2 and [l.get('handle') for l in logins] == [initial, handle]
              and all([l.get(k) for k in ['hp', 'max_hp', 'mp', 'max_mp', 'x', 'y']] == [h, h, m, m, *position]
                      for l, h, m in zip(logins, [120 if own else 100, hp], [60 if own else 50, mp]))
              and step(r, 'Saved').get('player', {}).get('level') == (3 if own else 1)
              and step(r, 'Saved').get('position') == position)
        check(role + ': XP JP and currency remain private and are not replayed',
              [(p.get('xp'), p.get('jp')) for p in t.get('experience', [])] == ([(799, 7), (4, 10), (4, 10)] if own else [(0, 0), (0, 0)])
              and [p.get('gold') for p in t.get('gold', [])] == ([1000, money, money] if own else [2000, 2000])
              and (1008 <= money <= 1014 if own else money == 2000)
              and step(r, 'Saved').get('xp_jp') == ([4, 10] if own else [0, 0]))
        checkpoint = step(r, 'Leveled' if own else 'Observed')
        check(role + ': post-combat peer state keeps own stats independent',
              checkpoint.get('stats') == after and (own or checkpoint.get('peer', {}).get('level') == 3
              and checkpoint['peer'].get('max_hp') == 140 and checkpoint['peer'].get('max_mp') == 70))
        settings = sql.get('queried', {}).get(role, {}).get('settings', {})
        for name in ['queried', 'saved', 'final']:
            seed = name == 'queried'
            p = sql.get(name, {}).get(role, {})
            check(role + ': independent ' + name + ' SQL retains complete unrelated settings',
                  settings.get('job') == ('100' if own else '200') and settings.get('jlv') == '1'
                  and p.get('settings') == dict(settings, jp=str(7 if own and seed else 10 if own else 0))
                  and [p.get(k) for k in ['level', 'xp', 'money', 'health', 'mp', 'online']] ==
                  [2 if own and seed else 3 if own else 1, 799 if own and seed else 4 if own else 0,
                   1000 if own and seed else money, 120 if own and seed else hp, 60 if own and seed else mp, 1 if seed else 0]
                  and p.get('position') == position)
        n = negative.get(role, {})
        check(role + ': old backend reproduces the wrong stat packet without requests or SQL mutation',
              n.get('success') is False and n.get('requests') == []
              and ('initial combat stats disagree with real server combat' in (n.get('failure') or '') or n.get('failure') == 'peer failed')
              and [{k: s.get(k) for k in ['attack', 'defence', 'speed', 'kind', 'bytes']} for s in n.get('trace', {}).get('stats', [])] ==
                  [dict(attack=14 if own else 12, defence=7 if own else 6, speed=120, kind=0, bytes=110)]
              and negative.get('sql', {}).get(role) == dict(sql.get('queried', {}).get(role, {}), online=0))
    return dict(success=all(c['passed'] for c in checks), checks=checks)


def load_and_compare(directory):
    directory = Path(directory)
    read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
    negative = directory.parent / 'negative'
    n = {r: read(negative / (r + '.json')) for r in ROLES}
    n.update(server=read(negative / 'server-process.json'), processes=read(negative / 'processes.json'), sql=read(negative / 'unchanged-database.json'))
    return compare({r: read(directory / (r + '.json')) for r in ROLES}, read(directory / 'server-process.json'),
                   read(directory / 'processes.json'), {s: read(directory / (s + '-database.json')) for s in ['queried', 'saved', 'final']}, n)


if __name__ == '__main__':
    directory = Path(sys.argv[1])
    result = load_and_compare(directory)
    (directory / 'comparison.json').write_text(json.dumps(result, indent=2), encoding='utf-8')
    print(json.dumps(dict(success=result['success'], checks=len(result['checks']), failed=[c['name'] for c in result['checks'] if not c['passed']]), indent=2))
    raise SystemExit(0 if result['success'] else 1)
