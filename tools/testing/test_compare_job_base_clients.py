import copy
import unittest
from compare_job_base_clients import compare_bases


def fixtures():
    reports = {}
    for role, job in [('owner', 100), ('witness', 200)]:
        active = dict(player={'handle': job}, job_base=dict(job=job, id=job, attributes=[10]*7),
                      job=dict(base=[job, [10]*7]), display=f'Job base (raw): {job} / {job}\nSTR 10 VIT 10 DEX 10 AGI 10\nINT 10 WIS 10 LUCK 10')
        reports[role] = dict(steps=[active, dict(stage='FirstLobby', player=None, job_base=None), copy.deepcopy(active),
                                   dict(stage='Logout', player=None, job_base=None)],
                             messages=[f'JOB_BASE {job} {job} 10 10 10 10 10 10 10']*2,
                             events={'RAPPELZ_JOB_BASE_CHANGED': 2})
    return reports


class CompareJobBase(unittest.TestCase):
    def test_accepts_complete_private_lifecycles(self):
        self.assertTrue(compare_bases(fixtures())['success'])

    def test_rejects_foreign_base_missing_attribute_and_effective_stat_label(self):
        for mutation in [lambda r: r['owner']['steps'][0]['job_base'].update(job=200),
                         lambda r: r['owner']['steps'][0]['job_base']['attributes'].pop(),
                         lambda r: r['owner']['steps'][0].update(display='Effective stats')]:
            r=fixtures(); mutation(r); self.assertFalse(compare_bases(r)['success'])

    def test_rejects_duplicate_or_stale_callback(self):
        for line in ['JOB_BASE 100 100 10 10 10 10 10 10 10', 'JOB_BASE 200 200 10 10 10 10 10 10 10']:
            r=fixtures(); r['owner']['messages'].append(line); self.assertFalse(compare_bases(r)['success'])

    def test_rejects_stale_lobby_and_missing_event(self):
        r=fixtures(); r['witness']['steps'][1]['job_base']=r['witness']['steps'][0]['job_base']
        self.assertFalse(compare_bases(r)['success'])
        r=fixtures(); r['witness']['events'].clear(); self.assertFalse(compare_bases(r)['success'])
