import copy
import unittest

from compare_visibility_clients import assess


def reports():
    def state(stage, handle, x, visible=False, target=None, mover=None):
        return {"stage": stage, "player": {"handle": handle, "level": 1, "hp": 100, "mp": 50},
                "own_position": [x, 7167], "peer_visible": visible, "target": target, "mover_position": mover,
                "xp_jp": [0, 0], "money": 0, "display": "Target HP: 100/100 MP: 50/50"}
    first = {"handle": 1, "points": [[8500, 7167], [9050, 7167]], "time": 100, "speed": 120, "layer": 0, "order": 1}
    last = {**first, "points": [[8330, 7167]], "time": 705, "order": 9}
    trace = {"hits": [], "levels": [], "errors": [], "warps": [], "experience": [{"handle": 1, "xp": 0, "jp": 0}],
             "moves": [first, last], "enters": [{"handle": 2, "x": 9230, "y": 7167}, {"handle": 1, "x": 8330, "y": 7167}], "leaves": [2]}
    mover = {"scenario": "visibility", "success": True, "own": 1, "mover": 1, "trace": trace,
             "steps": [state("Join", 1, 8330), state("Arrived", 1, 9050, True), state("Home", 1, 8330)], "events": {"PLAYER_LOGOUT": 1}}
    witness = {"scenario": "visibility", "success": True, "own": 2, "mover": 1,
               "trace": {**copy.deepcopy(trace), "experience": [{"handle": 2, "xp": 0, "jp": 0}], "leaves": [1],
                         "enters": [{"handle": 1, "x": 8696, "y": 7167, "hp": 100, "mp": 50, "order": 2}],
                         "moves": [{**first, "points": [[9050, 7167]], "time": 405, "order": 4}, last]},
               "steps": [state("Join", 2, 9230), state("Arrived", 2, 9230, True, {"handle": 1}, [9050, 7167]),
                         state("RangeLeave", 2, 9230), state("HiddenQuery", 2, 9230)],
               "events": {"PLAYER_LOGOUT": 1, "PLAYER_TARGET_CHANGED": 2}}
    return mover, witness


class VisibilityComparisonTest(unittest.TestCase):
    def test_valid_independent_mid_route_entry_sample(self):
        result = assess(*reports())
        self.assertTrue(result["success"], result)

    def test_empty_success_flags_do_not_validate_movement(self):
        self.assertFalse(assess({"success": True}, {"success": True})["success"])

    def test_stalled_replayed_out_of_order_or_retained_target_is_rejected(self):
        for case in ("frozen", "backtrack", "order", "retained-target", "false-reward"):
            with self.subTest(case=case):
                m, w = reports()
                if case == "frozen": w["steps"][1]["mover_position"] = [8696, 7167]
                elif case == "backtrack": w["trace"]["moves"][0]["points"].insert(0, [8500, 7167])
                elif case == "order": w["trace"]["moves"][0]["order"] = 1
                elif case == "false-reward": m["steps"][1]["xp_jp"] = [5, 0]
                else: w["steps"][2]["target"] = {"handle": 1}
                self.assertFalse(assess(m, w)["success"])

    def test_predicted_home_without_authoritative_query_is_rejected(self):
        m, w = reports()
        m["trace"]["enters"] = [e for e in m["trace"]["enters"] if e["handle"] != 1]
        self.assertFalse(assess(m, w)["success"])


if __name__ == "__main__":
    unittest.main()
