"""Compare data statements in local mariadb-dump snapshots without executing SQL.

The filtered dumps must use the same options as logical-before.sql. This checks
all world INSERTs and the explicitly selected pre-existing account/character rows.
It does not assert that operational realm/uptime/session metadata is unchanged.
"""
import hashlib
import json
from pathlib import Path
import re
import sys


def inserts(path, default_database=None):
    database = default_database
    result = {}
    with Path(path).open(encoding="utf-8-sig") as source:
        for line in source:
            use = re.match(r"USE `([^`]+)`;", line)
            if use:
                database = use[1]
            insert = re.match(r"INSERT INTO `([^`]+)` VALUES(?:\s|$)", line)
            if insert:
                statement = [line.rstrip("\r\n")]
                while not statement[-1].endswith(";"):
                    statement.append(next(source).rstrip("\r\n"))
                result.setdefault((database, insert[1]), []).append("\n".join(statement))
    return result


def compare(run):
    before = inserts(run / "logical-before.sql")
    if not before.get(("acore_auth", "account")) or not before.get(("acore_characters", "characters")):
        raise ValueError("Baseline account/character INSERTs were not found; refusing an empty comparison")
    checks = []
    selected = [
        ("acore_auth", "account"), ("acore_auth", "account_access"),
        ("acore_auth", "realmcharacters"), ("acore_characters", "characters"),
        ("acore_characters", "character_settings"), ("acore_characters", "character_inventory"),
        ("acore_characters", "item_instance"),
    ]
    for database, table in selected:
        key = database, table
        actual = inserts(run / f"preserved-{table}.sql", database).get(key, [])
        expected = before.get(key, [])
        checks.append({"database": database, "table": table, "unchanged": actual == expected,
                       "before_sha256": hashlib.sha256("\n".join(expected).encode()).hexdigest(),
                       "after_sha256": hashlib.sha256("\n".join(actual).encode()).hexdigest()})
    world = inserts(run / "world-after.sql")
    old_world = {k: v for k, v in before.items() if k[0] == "acore_world"}
    if not old_world:
        raise ValueError("Baseline world INSERTs were not found; refusing an empty comparison")
    new_world = {k: v for k, v in world.items() if k[0] == "acore_world"}
    checks.append({"database": "acore_world", "table": "all data INSERTs",
                   "unchanged": old_world == new_world, "tables_with_data": len(old_world)})
    report = {"success": all(c["unchanged"] for c in checks), "checks": checks}
    (run / "preservation.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
    return report


if __name__ == "__main__":
    report = compare(Path(sys.argv[1]).resolve(strict=True))
    print(json.dumps(report, indent=2))
    sys.exit(0 if report["success"] else 1)
