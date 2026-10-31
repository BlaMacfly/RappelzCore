import copy
import unittest
from compare_character_stats_clients import compare


def fixture():
    reports = {}
    sql = {s: {} for s in ['queried', 'saved', 'final']}
    negative = {}
    hits = [dict(attacker=11, target=100, damage=20, target_hp=85),
            dict(attacker=100, target=11, damage=15, target_hp=105),
            dict(attacker=11, target=100, damage=20, target_hp=0)]
    for role, initial, handle, peer, new_peer in [('owner', 11, 21, 12, 22), ('witness', 12, 22, 11, 21)]:
        own = role == 'owner'
        before = dict(attack=20 if own else 16, defence=8 if own else 6, speed=120)
        after = dict(attack=24 if own else 16, defence=10 if own else 6, speed=120)
        position = [9440, 7227 if own else 7247]
        hp, mp, money = (140, 70, 1012) if own else (100, 50, 2000)
        stats = [dict(target=h, kind=0, bytes=110, order=o, **v) for h, o, v in
                 [(initial, 2, before), (initial, 4, before), (initial, 5, before)] +
                 ([(initial, 20, after)] if own else []) + [(handle, 101, after), (handle, 104, after)]]
        trace = {k: [] for k in ['errors', 'network_errors', 'disconnects', 'cast_events', 'learn_results', 'job_results', 'warps', 'moves']}
        trace.update(stats=stats, hits=hits, levels=[dict(handle=11, job_level=1, level=3)], level_orders=[19],
                     logins=[dict(handle=h, order=o, hp=a, max_hp=a, mp=b, max_mp=b, x=position[0], y=position[1])
                             for h, o, a, b in [(initial, 1, 120 if own else 100, 60 if own else 50), (handle, 100, hp, mp)]],
                     experience=[dict(xp=x, jp=j) for x, j in ([(799, 7), (4, 10), (4, 10)] if own else [(0, 0), (0, 0)])],
                     gold=[dict(gold=g) for g in ([1000, money, money] if own else [money, money])])
        steps = [dict(stage=s, stats=v, display=f"AP: {v['attack']}  DEF: {v['defence']}", player=dict(level=3 if own else 1),
                      position=position, xp_jp=[4, 10] if own else [0, 0], money=money,
                      peer=dict(level=3, max_hp=140, max_mp=70)) for s, v in
                 [('Initial', before), ('Queried', before), ('Leveled' if own else 'Observed', after),
                  ('BeforeLobby', after), ('Reentered', after), ('Saved', after)]]
        steps.append(dict(stage='Logout', elapsed_ms=20000))
        reports[role] = dict(success=True, scenario='character-stats', initial=initial, own=handle, initial_other=peer, other=new_peer,
                             body=100, trace=trace, steps=steps,
                             requests=[dict(label=l, target=h, helper=b) for l, h, b in
                                       [('refresh1', initial, True), ('refresh2', initial, True), ('peer_query', peer, False),
                                        ('peer_query', peer, False), ('reentry_refresh', handle, True)]],
                             events=dict(PLAYER_LOGIN=2, PLAYER_ENTERING_WORLD=2, PLAYER_LOGOUT=2,
                                         PLAYER_LEVEL_UP=1 if own else 0, RAPPELZ_COMBAT_STATS_CHANGED=3 if own else 2),
                             messages=[f"[RappelzDebug] EVENT RAPPELZ_COMBAT_STATS_CHANGED attack={v['attack']} defence={v['defence']} speed=120"
                                       for v in ([before, after, after] if own else [before, after])])
        for name in sql:
            seed = name == 'queried'
            sql[name][role] = dict(level=2 if own and seed else 3 if own else 1,
                                   xp=799 if own and seed else 4 if own else 0,
                                   money=1000 if own and seed else money,
                                   health=120 if own and seed else hp, mp=60 if own and seed else mp,
                                   online=1 if seed else 0, position=position,
                                   settings=dict(job='100' if own else '200', jlv='1', jp=str(7 if own and seed else 10 if own else 0), unknown='retain'))
        negative[role] = dict(success=False, requests=[], failure='initial combat stats disagree with real server combat',
                              trace=dict(stats=[dict(attack=14 if own else 12, defence=7 if own else 6, speed=120, kind=0, bytes=110)]))
    negative.update(server=dict(negative_control=True, expected_rejection=True, success=False, exit_code=0,
                                executable_sha256='7afad0417e19f255426d2faf3c4ba06be3a60616358938add36136df1a92d380'),
                    processes=[dict(role=r, exit_code=1) for r in reports],
                    sql={r: dict(p, online=0) for r, p in sql['queried'].items()})
    server = dict(success=True, scenario='character-stats', world_pid=90, exit_code=0, executable_sha256='new-server',
                  checkpoints=[dict(name=n) for n in ['queried', 'saved']])
    return reports, server, [dict(role=r, pid=i + 91, exit_code=0) for i, r in enumerate(reports)], sql, negative


class CharacterStatsAcceptanceTests(unittest.TestCase):
    def reject(self, mutate):
        values = copy.deepcopy(fixture())
        mutate(values)
        self.assertFalse(compare(*values)['success'])

    def test_complete_independent_fixture_and_empty_input(self):
        result = compare(*fixture())
        self.assertTrue(result['success'], [c['name'] for c in result['checks'] if not c['passed']])
        self.assertEqual(len(result['checks']), 35)
        self.assertFalse(compare({}, {}, [], {}, {})['success'])

    def test_rejects_old_wrong_stats_private_leak_delta_and_stale_identity(self):
        self.reject(lambda v: v[0]['owner']['trace']['stats'][0].update(attack=14))
        self.reject(lambda v: v[0]['witness']['trace']['stats'][2].update(target=11))
        self.reject(lambda v: v[0]['owner']['trace']['stats'][1].update(kind=1))
        self.reject(lambda v: v[0]['owner']['requests'][-1].update(target=11))

    def test_rejects_duplicate_lua_partial_tuple_and_reentry_replay(self):
        self.reject(lambda v: v[0]['owner']['events'].update(RAPPELZ_COMBAT_STATS_CHANGED=4))
        self.reject(lambda v: v[0]['owner']['messages'].__setitem__(1, 'EVENT RAPPELZ_COMBAT_STATS_CHANGED attack=24'))
        self.reject(lambda v: v[0]['witness']['trace']['level_orders'].__setitem__(0, 103))
        self.reject(lambda v: v[0]['owner']['trace']['stats'][-1].update(order=99))

    def test_rejects_partial_sql_unknown_key_loss_private_gain_and_mismatched_combat(self):
        self.reject(lambda v: v[3]['saved']['owner']['settings'].update(jp='7'))
        self.reject(lambda v: v[3]['final']['witness']['settings'].pop('unknown'))
        self.reject(lambda v: v[0]['witness']['trace']['experience'][0].update(jp=10))
        self.reject(lambda v: v[0]['witness']['trace'].update(hits=[]))

    def test_rejects_forged_control_process_failure_and_unproven_regression(self):
        self.reject(lambda v: v[2][0].update(exit_code=1))
        self.reject(lambda v: v[4]['server'].update(expected_rejection=False))
        self.reject(lambda v: v[4]['owner']['trace']['stats'][0].update(attack=20))
        self.reject(lambda v: v[4]['sql']['owner'].update(xp=4))


if __name__ == '__main__':
    unittest.main()
