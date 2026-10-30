import unittest

from compare_combat_clients import assess, damage_sequence, combat_cycles


class CombatComparisonTest(unittest.TestCase):
    def test_empty_success_reports_cannot_validate_combat(self):
        result = assess({"success": True}, {"success": True})
        self.assertFalse(result["success"])
        self.assertEqual(result["attack_records"], 0)

    def test_damage_sequence_rejects_inconsistent_hp_and_accepts_overkill(self):
        self.assertTrue(damage_sequence([{"damage": 16, "target_hp": 104}, {"damage": 110, "target_hp": 0}], 120))
        self.assertFalse(damage_sequence([{"damage": 16, "target_hp": 120}], 120))
        self.assertFalse(damage_sequence([], 120))

    def test_repeat_trace_must_end_at_each_death_and_contain_only_combatants(self):
        hit = {"attacker": 1, "target": 2, "damage": 105, "target_hp": 0}
        reply = {"attacker": 2, "target": 1, "damage": 10, "target_hp": 130}
        self.assertEqual(combat_cycles([hit, reply, hit], 1, 2), [[hit], [reply, hit]])
        self.assertEqual(combat_cycles([hit, reply], 1, 2), [])
        self.assertEqual(combat_cycles([hit, {**hit, "attacker": 3}], 1, 2), [])


if __name__ == "__main__":
    unittest.main()
