"""Independent SQL/native checks for SOAP shutdown with two active characters."""
import json
from pathlib import Path
import sys

ROLES = ('owner', 'witness')


def expected_profile(f, role, online, moved, progressed=True):
    settings = dict(f['settings'])
    if progressed:
        settings.update(jlv='2' if role == 'owner' else '1', jp='5' if role == 'owner' else '0')
    return dict(guid=f['guid'], account=f['account'], character=f['character'], level=10, xp=0,
                money=f['money'], health=280, mp=140,
                position=[f['position'][0]+(12 if moved else 0), f['position'][1]+(8 if moved else 0)],
                online=online, account_online=online, settings=settings)


def compare(cases):
    checks = []
    def check(name, value): checks.append(dict(name=name, passed=bool(value)))
    check('old control and both corrected scenarios are present', set(cases) == {'negative', 'normal', 'slow'})
    for label in ['negative', 'normal', 'slow']:
        c = cases.get(label, {}); s = c.get('server', {}); f = c.get('fixtures', {}); negative = label == 'negative'
        check(label+': only two distinct new fixture identities', set(f) == set(ROLES) and all(f[r].get('guid', 0)>33 and f[r].get('account', 0)>33 for r in ROLES) and len({v['guid'] for v in f.values()}) == 2 and len({v['account'] for v in f.values()}) == 2)
        if set(f) != set(ROLES): continue
        profiles = lambda online, moved, progressed=True: {r: expected_profile(f[r], r, online, moved, progressed) for r in ROLES}
        check(label+': seeded characters have complete expected profiles', c.get('seeded') == profiles(0, False, False))
        check(label+': one durable job purchase precedes movement', c.get('progress') == profiles(1, False))
        check(label+': SQL still holds old positions when SOAP is requested', c.get('before') == profiles(1, False))
        check(label+': SOAP observes two online players and replies before unattended exit', s.get('success') is True and s.get('negative') == negative and s.get('first_exit_code') == 0 and s.get('stdin_remained_open') is True and s.get('soap_response', {}).get('status') == 200 and s.get('live_soap_info', {}).get('status') == 200 and 'joueurs : 2' in (s.get('live_soap_info', {}).get('result') or '') and not s.get('assisted_clients') and not s.get('assisted_shutdown'))
        processes = s.get('clients', [])
        check(label+': both ordinary native clients complete successfully', len(processes) == 2 and {p.get('role') for p in processes} == set(ROLES) and all(p.get('exit_code') == 0 for p in processes))
        if label != 'normal':
            waits = s.get('save_waits', [])
            check(label+': two real modified transactions wait on settings REPLACE', len(waits) == 2 and len({w.get('transaction') for w in waits}) == 2 and all(w.get('modified', 0)>0 and w.get('settings_replace') is True for w in waits))
            check(label+': committed readers see old state while dirty readers see pending movement', s.get('committed_while_blocked') == c.get('before') and s.get('dirty_while_blocked') == profiles(1, True) and s.get('held_seconds', 0)>=6)
        if negative:
            check('old server exits during blocked saves and leaves the last checkpoint intact', s.get('alive_before_release') is False and s.get('saved_before_exit') is False and c.get('after') == c.get('before'))
        else:
            check(label+': all characters and accounts are offline with complete new state at exit', s.get('saved_before_exit') is True and c.get('after') == profiles(0, True))
            check(label+': restart restores full state before normal lobby save', c.get('reentry') == profiles(1, True) and c.get('final') == profiles(0, True))
            check(label+': second world exits through SOAP normally', s.get('second_pid') not in [None, s.get('first_pid')] and s.get('second_exit_code') == 0 and s.get('last_soap_response', {}).get('status') == 200)
        if label == 'slow': check('corrected server stays alive until delayed saves can commit', s.get('alive_before_release') is True and s.get('exit_seconds', 0)>=s.get('held_seconds', 6))
        for role in ROLES:
            report = c.get('reports', {}).get(role, {}); trace = report.get('trace', {}); events = report.get('events', {})
            stage = lambda name: next((v for v in report.get('steps', []) if v.get('stage') == name), {})
            steps = ['Initial', 'ProgressCommitted', 'MovedLive', 'Disconnected'] + ([] if negative else ['Reentered', 'SavedLobby'])
            check(label+' '+role+': real bounded lifecycle contains the expected stages', report.get('success') is True and not report.get('failure') and report.get('scenario') == 'soap-player-shutdown' and report.get('negative') == negative and [v.get('stage') for v in report.get('steps', [])] == steps and stage(steps[-1]).get('elapsed_ms', 999999)<150000)
            expected = profiles(0, True)[role]; live = stage('MovedLive'); player = live.get('player', {})
            other = 'witness' if role == 'owner' else 'owner'; peer_position = profiles(0, True)[other]['position']
            check(label+' '+role+': live unsaved position and synchronized peer are confirmed', live.get('position') == expected['position'] and live.get('peer_position') == peer_position and live.get('peer', {}).get('name') == f[other]['character'] and [player.get(k) for k in ['name','level','hp','mp','max_hp','max_mp']] == [f[role]['character'],10,280,140,280,140] and live.get('money') == f[role]['money'] and live.get('xp_jp') == [0,5 if role == 'owner' else 0])
            entries = 1 if negative else 2; jp_events = entries+(role == 'owner')
            check(label+' '+role+': real Lua logout keeps readable progress without replay', all(events.get(k) == entries for k in ['PLAYER_LOGIN','PLAYER_ENTERING_WORLD','PLAYER_LOGOUT']) and events.get('RAPPELZ_JP_CHANGED') == jp_events and events.get('RAPPELZ_JOB_LEVEL_CHANGED') == jp_events and events.get('PLAYER_LEVEL_UP', 0) == 0 and len([m for m in report.get('messages', []) if 'EVENT PLAYER_LOGOUT ' in m and f'money={f[role]["money"]}' in m and f'jp={5 if role == "owner" else 0} jlv={2 if role == "owner" else 1}' in m]) == entries)
            errors = trace.get('network_errors', [])
            check(label+' '+role+': only the server shutdown descriptor interrupts transport', trace.get('errors') == [] and len(errors) == 1 and errors[0].endswith('(code 0).') and trace.get('hits') == [] and trace.get('cast_events') == [] and trace.get('learn_results') == [] and trace.get('job_results') == ([0] if role == 'owner' else []))
            check(label+' '+role+': reconnect does not duplicate money or earned progress', len(trace.get('logins', [])) == entries and [v.get('gold') for v in trace.get('gold', [])] == [f[role]['money']]*entries and [(v.get('xp'),v.get('jp')) for v in trace.get('experience', [])] == ([(0,8),(0,5)]+([] if negative else [(0,5)]) if role == 'owner' else [(0,0)]*entries))
            if not negative:
                restored = stage('Reentered')
                check(label+' '+role+': actual reauthentication loads the saved position and state', restored.get('position') == live.get('position') and restored.get('peer_position') == peer_position and restored.get('money') == live.get('money') and restored.get('xp_jp') == live.get('xp_jp') and restored.get('job') == live.get('job') and {k:v for k,v in restored.get('player', {}).items() if k != 'handle'} == {k:v for k,v in player.items() if k != 'handle'})
    return dict(success=all(c['passed'] for c in checks), checks=checks)


def load_and_compare(directory):
    directory = Path(directory); read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
    cases = {}
    for label in ['negative', 'normal', 'slow']:
        folder = directory/label
        cases[label] = dict(server=read(folder/'server-process.json'), fixtures=read(folder/'fixtures.json'),
                            seeded=read(folder/'seeded.json'), progress=read(folder/'progress-database.json'),
                            before=read(folder/'before-shutdown.json'), after=read(folder/'after-shutdown.json'),
                            reports={r:read(folder/(r+'.json')) for r in ROLES})
        if label != 'negative': cases[label].update(reentry=read(folder/'reentry-database.json'), final=read(folder/'final-database.json'))
    return compare(cases)


if __name__ == '__main__':
    directory = Path(sys.argv[1]); result = load_and_compare(directory)
    (directory/'comparison.json').write_text(json.dumps(result, indent=2), encoding='utf-8')
    print(json.dumps(dict(success=result['success'], checks=len(result['checks']), failed=[c['name'] for c in result['checks'] if not c['passed']])))
    raise SystemExit(0 if result['success'] else 1)
