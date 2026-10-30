import unittest

from compare_combat_clients import assess, damage_sequence


class CombatComparisonTest(unittest.TestCase):
    def test_empty_success_reports_cannot_validate_combat(self):
        result = assess({"success": True}, {"success": True})
        self.assertFalse(result["success"])
        self.assertEqual(result["attack_records"], 0)

    def test_damage_sequence_rejects_inconsistent_hp_and_accepts_overkill(self):
        self.assertTrue(damage_sequence([{"damage": 16, "target_hp": 104}, {"damage": 110, "target_hp": 0}], 120))
        self.assertFalse(damage_sequence([{"damage": 16, "target_hp": 120}], 120))
        self.assertFalse(damage_sequence([], 120))


if __name__ == "__main__":
    unittest.main()
