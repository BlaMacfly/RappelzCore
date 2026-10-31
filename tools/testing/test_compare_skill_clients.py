import copy
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from compare_skill_clients import assess


def reports():
    old, new, peer = 2, 3, 1
    def catalog(target, jp, levels, job=200, jlv=3):
        a, b = dict(levels).get(1003, 0), dict(levels).get(1004, 0)
        cost = [4, 10, 62][a]
        def row(id, learned, maximum, price, why, j=1, prerequisites=None):
            return dict(id=id, learned=learned, next=learned+1, maximum=maximum, cost=price, reason=why, job_level=j,
                        player_level=0, prerequisites=prerequisites or [[0, 0]]*3, passive=False)
        return dict(target=target, job=job, job_level=jlv, player_level=1, jp=jp, status=0, entries=[
            row(1003, a, 3, cost, 3 if jlv < 3 else 2 if not b else 1 if jp < cost else 0, 3, [[1004, 1], [0, 0], [0, 0]]),
            row(1004, b, 1, None if b else 4, 5 if b else 1 if jp < 4 else 0), row(1801, 0, 1, 4, 3, 5)])
    learned = [[], [[1004, 1]], [[1003, 1], [1004, 1]], [[1003, 2], [1004, 1]], [[1003, 2], [1004, 1]]]
    balances, handles = [18, 14, 10, 0, 0], [old, old, old, old, new]
    cats = [catalog(h, jp, s) for h, jp, s in zip(handles, balances, learned)]
    def skill_list(h, levels): return dict(target=h, modification=0, levels=[dict(id=id, base=lv, current=lv) for id, lv in levels])
    def step(name, c, levels, owner):
        h = c["target"]
        return dict(stage=name, catalog=copy.deepcopy(c), learned=copy.deepcopy(levels), jp=c["jp"], xp_jp=[0, c["jp"]],
                    player=dict(handle=h, level=1, layer=0, hp=100, max_hp=100, mp=50, max_mp=50), position=[8440, 7227] if owner else [8420, 7240],
                    money=0, bag_revision=0, display=f"Player\nSkills: {len(levels)}  JP: {c['jp']}\nTarget")
    stages = [("Initial", 0), ("Denied", 0), ("FirstLearned", 1), ("DuplicatesDenied", 1), ("SecondLearned", 2), ("Upgraded", 3), ("Exhausted", 3), ("Reentered", 4)]
    empty = dict(hits=[], moves=[], warps=[], levels=[], errors=[])
    seq = [(peer, 1004, 0, 1), (old, 1004, 1, 1), (old, 1004, 0, -1), (old, 1004, 0, 2), (old, 2602, 0, 1),
           (old, 1801, 0, 1), (old, 1003, 0, 1), (old, 2147483647, 0, 1), (old, 1004, 0, 1), (old, 1004, 0, 1),
           (old, 1004, 0, 1), (old, 1003, 0, 1), (old, 1003, 0, 2), (old, 1003, 0, 3), (old, 1004, 0, 2)]
    owner = dict(scenario="skills", success=True, initial=old, own=new, other=peer,
                 requests=[dict(zip(("target", "id", "random", "level"), r)) for r in seq],
                 events=dict(PLAYER_LOGIN=2, PLAYER_LOGOUT=2, SKILL_LINES_CHANGED=5, RAPPELZ_JP_CHANGED=5),
                 steps=[step(name, cats[index], learned[index], True) for name, index in stages]+[dict(stage="Lobby"), dict(stage="Logout")],
                 messages=["périmée", "arbre actuel", "métier insuffisant", "préalable", "pas assez de JP", "maximal"]+
                          [f"EVENT RAPPELZ_JP_CHANGED {jp} value" for jp in balances],
                 trace={**copy.deepcopy(empty), "logins": [dict(handle=h, layer=0) for h in (old, new)], "skill_catalogs": cats,
                        "skill_lists": [skill_list(h, s) for h, s in zip(handles, learned)],
                        "experience": [dict(handle=h, jp=jp, xp=0) for h, jp in zip(handles, balances)],
                        "learn_results": [6]*8+[0]+[6]*2+[0, 0]+[6]*2})
    wc = catalog(peer, 0, [], 300, 1)
    witness = dict(scenario="skills", success=True, initial=peer, own=peer, initial_other=old, other=new,
                   requests=[dict(target=h, id=1004, random=0, level=1) for h in (peer, old)],
                   events=dict(PLAYER_LOGIN=1, PLAYER_LOGOUT=1, SKILL_LINES_CHANGED=1, RAPPELZ_JP_CHANGED=1),
                   steps=[step(name, wc, [], False) for name in ("Initial", "Denied", "Inspected", "FreshQuery", "PeerLogout")]+[dict(stage="Logout")],
                   trace={**copy.deepcopy(empty), "logins": [dict(handle=peer, layer=0)], "skill_catalogs": [wc], "skill_lists": [skill_list(peer, [])],
                          "experience": [dict(handle=peer, xp=0, jp=0)], "learn_results": [6, 6], "leaves": [old, new],
                          "enters": [dict(handle=h, kind="player") for h in (old, old, new, new)]})
    return owner, witness


class LearningComparisonTest(unittest.TestCase):
    def test_independent_valid_purchase_and_private_reentry_sample(self):
        result = assess(*reports())
        self.assertTrue(result["success"], result)

    def test_empty_success_flags_are_not_learning_evidence(self):
        self.assertFalse(assess({"success": True}, {"success": True})["success"])

    def test_charge_duplicate_privacy_persistence_cost_event_and_stat_mutations_fail(self):
        for case in ("charge", "duplicate", "privacy", "persistence", "cost", "event", "stats", "job"):
            with self.subTest(case=case):
                o, w = reports()
                if case == "charge": o["trace"]["experience"][1]["jp"] = 13
                elif case == "duplicate": o["trace"]["learn_results"][9] = 0
                elif case == "privacy": w["trace"]["skill_lists"].append(copy.deepcopy(o["trace"]["skill_lists"][-1]))
                elif case == "persistence": next(s for s in o["steps"] if s["stage"] == "Reentered")["learned"] = []
                elif case == "cost": o["steps"][0]["catalog"]["entries"][0]["cost"] = 3
                elif case == "event": o["events"]["SKILL_LINES_CHANGED"] = 6
                elif case == "stats": o["steps"][0]["player"]["hp"] = 120
                else: w["trace"]["skill_catalogs"][0]["job"] = 200
                self.assertFalse(assess(o, w)["success"])

    def test_cli_preserves_utf8_refusal_notices(self):
        with tempfile.TemporaryDirectory() as directory:
            for role, report in zip(("owner", "witness"), reports()):
                Path(directory, role+".json").write_text(json.dumps(report, ensure_ascii=False), encoding="utf-8")
            result = subprocess.run([sys.executable, str(Path(__file__).with_name("compare_skill_clients.py")), directory], capture_output=True)
            self.assertEqual(result.returncode, 0, result.stdout.decode("utf-8", errors="replace"))
            self.assertTrue(json.loads(Path(directory, "comparison.json").read_text(encoding="utf-8"))["success"])


if __name__ == "__main__": unittest.main()
