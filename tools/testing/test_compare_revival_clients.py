import copy
import unittest

from compare_revival_clients import assess


def reports():
    """Minimal independent valid sample, including an overkill and unsolicited ENTER."""
    unit = {"handle": 1, "level": 1, "hp": 100, "mp": 50}
    target = {**unit}
    enter = {**unit, "max_hp": 100, "max_mp": 50, "x": 8330.0, "y": 7167.0}
    hits = [{"attacker": 1, "target": 3, "damage": 12, "target_hp": 93, "attacker_hp": 1},
            {"attacker": 3, "target": 1, "damage": 17, "target_hp": 0, "attacker_hp": 93}]
    hpmp = [{"handle": 3, "hp": 93, "add_hp": -12}, {"handle": 1, "hp": 0, "add_hp": -17},
            {"handle": 1, "hp": 100, "max_hp": 100, "mp": 50, "max_mp": 50, "add_hp": 0}]
    observed = {"stage": "Observe", "player": unit, "target": target, "xp_jp": [0, 0], "money": 0,
                "victim_position": [8330.0, 7167.0], "victim_renderable": True, "observation_ms": 3500,
                "victim_enter_count": 2, "display": "Target HP: 100/100 MP: 50/50"}
    victim = {"scenario": "revival", "success": True, "own": 1, "victim": 1, "monster": 3,
              "trace": {"hits": hits, "hpmp": hpmp, "levels": [], "errors": [],
                        "experience": [{"handle": 1, "xp": 0, "jp": 0}],
                        "warps": [{"x": 8330.0, "y": 7167.0, "z": 0.0}]},
              "steps": [observed], "events": {"PLAYER_DEAD": 1, "PLAYER_ALIVE": 1, "PLAYER_LOGOUT": 1},
              "messages": ["EVENT PLAYER_DEAD ? HP=0/100 MP=1/50", "EVENT PLAYER_ALIVE ? HP=100/100 MP=50/50"]}
    witness = copy.deepcopy(victim)
    witness["own"] = 2
    witness["events"] = {"PLAYER_LOGOUT": 1}
    witness["trace"].update({"warps": [], "leaves": [1],
                             "experience": [{"handle": 2, "xp": 0, "jp": 0}],
                             "enters": [{**enter, "x": 8450.0, "y": 7227.0}, enter, enter]})
    witness_observed = copy.deepcopy(observed)
    witness_observed["player"]["handle"] = 2
    witness["steps"] = [{"stage": "Join", "victim_enter_count": 1}, witness_observed,
                         {"stage": "Query", "victim_enter_count": 3}, {"stage": "Leave", "target": None}]
    witness["messages"] = ["EVENT UNIT_HEALTH target HP=100/100 targetHP=0",
                            "EVENT UNIT_HEALTH target HP=100/100 targetHP=100"]
    return victim, witness


class RevivalComparisonTest(unittest.TestCase):
    def test_independent_valid_sample_including_overkill(self):
        result = assess(*reports())
        self.assertTrue(result["success"], result)

    def test_empty_success_flags_do_not_validate_revival(self):
        self.assertFalse(assess({"success": True}, {"success": True})["success"])

    def test_render_position_events_and_unsolicited_refresh_are_required(self):
        for case in ("hidden", "old-position", "duplicate-event", "query-masks-refresh"):
            with self.subTest(case=case):
                v, w = reports()
                if case == "hidden": w["steps"][1]["victim_renderable"] = False
                elif case == "old-position": w["steps"][1]["victim_position"] = [8450, 7227]
                elif case == "duplicate-event": v["events"]["PLAYER_DEAD"] = 2
                else: w["steps"][1]["victim_enter_count"] = 1
                self.assertFalse(assess(v, w)["success"])

    def test_missing_death_or_false_rewards_are_rejected(self):
        v, w = reports()
        v["trace"]["hpmp"] = [h for h in v["trace"]["hpmp"] if h.get("hp") != 0]
        self.assertFalse(assess(v, w)["success"])
        v, w = reports()
        w["trace"]["experience"][0]["jp"] = 3
        self.assertFalse(assess(v, w)["success"])
        v, w = reports()
        v["trace"]["warps"][0]["layer"] = 1
        self.assertFalse(assess(v, w)["success"])


if __name__ == "__main__":
    unittest.main()
