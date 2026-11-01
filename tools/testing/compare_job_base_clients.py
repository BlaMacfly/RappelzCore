"""Compare real Lua/raw-job-base state separately from effective combat attributes."""
import json
import re
from pathlib import Path
import sys
from compare_job_progression_clients import load_and_compare as compare_progression


def compare_bases(reports):
    checks = []
    def check(name, value):
        checks.append(dict(name=name, passed=bool(value)))
    for role, job in [('owner', 100), ('witness', 200)]:
        report = reports.get(role, {})
        steps = report.get('steps', [])
        active = [s for s in steps if s.get('player')]
        expected = dict(job=job, id=job, attributes=[10]*7)
        check(role+': every active adapter snapshot holds its own confirmed raw base',
              bool(active) and all(s.get('job_base') == expected and s.get('job', {}).get('base') == [job, [10]*7] for s in active))
        check(role+': real Lua displays all seven attributes as raw job base', bool(active) and all(
            all(text in s.get('display', '') for text in [f'Job base (raw): {job} / {job}', 'STR 10 VIT 10 DEX 10 AGI 10', 'INT 10 WIS 10 LUCK 10']) for s in active))
        callbacks = [list(map(int, m)) for line in report.get('messages', [])
                     for m in re.findall(r'JOB_BASE (\d+) (\d+) ((?:\d+ ){6}\d+)', line)
                     for m in [m[:2]+tuple(m[2].split())]]
        check(role+': Lua reads one complete base per world entry without JLV or foreign duplicates',
              callbacks == [[job, job]+[10]*7]*2 and report.get('events', {}).get('RAPPELZ_JOB_BASE_CHANGED') == 2)
        lobby = [s for s in steps if s.get('stage') in ['FirstLobby', 'Logout']]
        check(role+': lobby clears the complete base after each real logout',
              len(lobby) == 2 and all(s.get('player') is None and 'job_base' in s and s['job_base'] is None for s in lobby))
    return dict(success=all(c['passed'] for c in checks), checks=checks)


def load_and_compare(directory):
    directory = Path(directory)
    prior = compare_progression(directory)
    bases = compare_bases({r: json.loads((directory/(r+'.json')).read_text(encoding='utf-8-sig')) for r in ['owner', 'witness']})
    checks = prior['checks']+bases['checks']
    return dict(success=all(c['passed'] for c in checks), checks=checks)


if __name__ == '__main__':
    directory = Path(sys.argv[1]); result = load_and_compare(directory)
    (directory/'job-base-comparison.json').write_text(json.dumps(result, indent=2), encoding='utf-8')
    print(json.dumps(dict(success=result['success'], checks=len(result['checks']), failed=[c['name'] for c in result['checks'] if not c['passed']])))
    raise SystemExit(0 if result['success'] else 1)
