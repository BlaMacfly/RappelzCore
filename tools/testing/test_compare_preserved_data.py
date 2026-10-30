import tempfile
import unittest
from pathlib import Path

from compare_preserved_data import compare, inserts


class SnapshotComparisonTest(unittest.TestCase):
    def test_multiline_mariadb_13_and_one_line_inserts(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "dump.sql"
            path.write_text("USE `auth`;\nINSERT INTO `account` VALUES\n(1,'old;account'),\n(2,'other');\n"
                            "INSERT INTO `access` VALUES (1,3);\n", encoding="utf-8")
            parsed = inserts(path)
            self.assertIn("(2,'other');", parsed[("auth", "account")][0])
            self.assertEqual(parsed[("auth", "access")], ["INSERT INTO `access` VALUES (1,3);"])

    def test_empty_unrecognized_baseline_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "logical-before.sql").write_text("-- no recognized data\n", encoding="utf-8")
            with self.assertRaises(ValueError):
                compare(root)

    def test_changed_legacy_data_is_detected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            account = "INSERT INTO `account` VALUES\n(1,'original');\n"
            character = "INSERT INTO `characters` VALUES\n(1,'Aria');\n"
            world = "USE `acore_world`;\nINSERT INTO `creature` VALUES\n(1,711);\n"
            (root / "logical-before.sql").write_text("USE `acore_auth`;\n" + account +
                "USE `acore_characters`;\n" + character + world, encoding="utf-8")
            for table in ["account", "account_access", "realmcharacters", "characters", "character_settings", "character_inventory", "item_instance"]:
                (root / f"preserved-{table}.sql").write_text(
                    account if table == "account" else character if table == "characters" else "", encoding="utf-8")
            (root / "world-after.sql").write_text(world, encoding="utf-8")
            self.assertTrue(compare(root)["success"])
            (root / "preserved-account.sql").write_text(account.replace("original", "changed"), encoding="utf-8")
            self.assertFalse(compare(root)["success"])


if __name__ == "__main__":
    unittest.main()
