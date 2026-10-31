import copy
import unittest

from compare_layer_clients import assess


def reports():
    def state(stage, handle, layer, position, entities=18, target=None, dialog=False):
        return {"stage": stage, "player": {"handle": handle, "layer": layer, "level": 1, "hp": 100, "mp": 50},
                "layer": layer, "entities": entities, "target": target, "dialog_open": dialog, "position": position,
                "xp_jp": [0, 0], "money": 0, "bag_revision": 0,
                "display": f"Player: Fixture\nLayer: {layer}\nTarget layer: 0"}
    def target(handle):
        return {"handle": handle, "layer": 0, "hp": 100, "mp": 50}
    position, other_position = [8440, 7227], [8420, 7240]
    trace = {"logins": [{"handle": 2, "layer": 0}, {"handle": 3, "layer": 1}],
             "hits": [], "moves": [], "levels": [], "errors": [], "experience": [{"handle": 2, "xp": 0, "jp": 0}, {"handle": 3, "xp": 0, "jp": 0}],
             "warps": [{"layer": layer, "x": 8440, "y": 7227, "z": 0} for layer in (1, 0)],
             "dialogs": [{"npc": 4, "kind": "dialog"}, {"npc": 4, "kind": "dialog"}],
             "denied_attacks": [{"attacker": 2, "target": 5, "reason": 1}], "targets": [4, 0, 1],
             "enters": [{"kind": "player", "handle": 1, "layer": 0} for _ in range(3)], "leaves": [1]}
    operator = {"scenario": "layer", "success": True, "old_operator": 2, "operator": 3, "own": 3, "npc": 4, "monster": 5,
                "trace": trace, "events": {"PLAYER_LOGIN": 2, "PLAYER_LOGOUT": 2, "RAPPELZ_LAYER_CHANGED": 2},
                "messages": ["Usage : .layer <0-255>", "EVENT RAPPELZ_LAYER_CHANGED 1 layer=1", "EVENT RAPPELZ_LAYER_CHANGED 0 layer=0"],
                "steps": [state("Join", 2, 0, position), state("InitialDialog", 2, 0, position, dialog=True),
                          state("InvalidLayerDenied", 2, 0, position), state("Away", 2, 1, position, 0),
                          state("Isolated", 2, 1, position, 0), {"stage": "Disconnected"},
                          state("Reentered", 3, 1, position, 0), state("Restored", 3, 0, position, target=target(1), dialog=True)]}
    witness = {"scenario": "layer", "success": True, "old_operator": 2, "operator": 3, "own": 1,
               "trace": {**copy.deepcopy(trace), "logins": [{"handle": 1, "layer": 0}], "warps": [], "dialogs": [],
                         "experience": [{"handle": 1, "xp": 0, "jp": 0}], "targets": [2, 0, 3], "denied_attacks": [],
                         "enters": [{"kind": "player", "handle": handle, "layer": 0} for handle in (2, 3, 3)], "leaves": [2, 3]},
               "events": {"PLAYER_LOGIN": 1, "PLAYER_LOGOUT": 1, "PLAYER_TARGET_CHANGED": 4},
               "messages": ["Vous n'avez pas le niveau requis"],
               "steps": [state("Join", 1, 0, other_position), state("PermissionDenied", 1, 0, other_position),
                         state("Isolated", 1, 0, other_position, 17), state("Restored", 1, 0, other_position, target=target(3)),
                         state("Queried", 1, 0, other_position, target=target(3)), state("PeerLogout", 1, 0, other_position)]}
    return operator, witness


class LayerComparisonTest(unittest.TestCase):
    def test_independent_valid_layer_roundtrip_and_relogin(self):
        result = assess(*reports())
        self.assertTrue(result["success"], result)

    def test_empty_success_flags_cannot_validate_layers(self):
        self.assertFalse(assess({"success": True}, {"success": True})["success"])

    def test_persistence_npc_target_and_permission_leaks_are_rejected(self):
        for case in ("persist", "npc", "target", "permission", "duplicate-event"):
            with self.subTest(case=case):
                o, w = reports()
                if case == "persist": o["trace"]["logins"][1]["layer"] = 0
                elif case == "npc": o["trace"]["dialogs"].append({"npc": 4, "kind": "market"})
                elif case == "target": w["trace"]["targets"][1] = 2
                elif case == "permission": w["trace"]["warps"] = [{"layer": 1}]
                else: o["events"]["RAPPELZ_LAYER_CHANGED"] = 3
                self.assertFalse(assess(o, w)["success"])

    def test_fabricated_zero_layer_adapter_and_false_rewards_are_rejected(self):
        o, w = reports()
        next(s for s in o["steps"] if s["stage"] == "Isolated")["player"]["layer"] = 0
        self.assertFalse(assess(o, w)["success"])
        o, w = reports()
        w["trace"]["experience"][0]["jp"] = 3
        self.assertFalse(assess(o, w)["success"])


if __name__ == "__main__":
    unittest.main()
