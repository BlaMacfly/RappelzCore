"""Cross-check two native-client wire traces from the dedicated combat fixture.

The fixture starts at level 2 / 799 XP. A real monster kill awards 5 XP / 3 JP.
This script reads reports only; it cannot send gameplay commands or modify SQL.
"""
import json
from pathlib import Path
import sys


def damage_sequence(hits, starting_hp):
    if not hits:
        return False
    hp = starting_hp
    for hit in hits:
        damage = hit.get("damage", 0)
        if not isinstance(damage, int) or damage <= 0:
            return False
        hp = max(0, hp - damage)
        if hit.get("target_hp") != hp:
            return False
    return True


def combat_cycles(hits, actor, monster):
    """Split the ordered wire trace at each killing blow, retaining retaliation."""
    cycles, current = [], []
    for hit in hits:
        if (hit.get("attacker"), hit.get("target")) not in ((actor, monster), (monster, actor)):
            return []
        current.append(hit)
        if hit.get("attacker") == actor and hit.get("target_hp") == 0:
            cycles.append(current)
            current = []
    return [] if current else cycles


def assess(fighter, witness):
    checks = []

    def check(name, condition):
        checks.append({"name": name, "passed": bool(condition)})

    f = fighter.get("trace", {})
    w = witness.get("trace", {})
    actor, monster = fighter.get("fighter"), fighter.get("monster")
    hits, observed = f.get("hits", []), w.get("hits", [])
    outgoing = [h for h in hits if h.get("attacker") == actor and h.get("target") == monster]
    retaliation = [h for h in hits if h.get("attacker") == monster and h.get("target") == actor]
    kills = fighter.get("kills", 1)
    cycles = combat_cycles(hits, actor, monster)
    check("same supported kill count", kills in (1, 2) and witness.get("kills", 1) == kills)
    check("both native runs completed successfully", fighter.get("success") is True and witness.get("success") is True)
    check("distinct clients observed the same fighter and monster", actor and monster and fighter.get("own") == actor
          and witness.get("fighter") == actor and witness.get("monster") == monster and witness.get("own") != actor)
    check("all attack records match across clients", hits and hits == observed)
    check("outgoing damage sequence and monster death agree", len(cycles) == kills and all(
        damage_sequence([h for h in cycle if h.get("attacker") == actor], 105) for cycle in cycles))
    check("retaliation and fighter damage agree", len(cycles) == kills and all(
        damage_sequence([h for h in cycle if h.get("attacker") == monster], 120 if i == 0 else 140)
        and next(h for h in reversed(cycle) if h.get("attacker") == monster).get("target_hp", 0) > 0
        for i, cycle in enumerate(cycles)))
    fd = [p for p in f.get("hpmp", []) if p.get("add_hp", 0) < 0]
    wd = [p for p in w.get("hpmp", []) if p.get("add_hp", 0) < 0]
    check("all damage HPMP broadcasts match", fd and fd == wd)
    fl = [p for p in f.get("levels", []) if p.get("handle") == actor]
    wl = [p for p in w.get("levels", []) if p.get("handle") == actor]
    check("same real XP level-up broadcast", fl == [{"handle": actor, "level": 3, "job_level": 1}] and fl == wl)
    xp = [p for p in f.get("experience", []) if p.get("handle") == actor]
    check("actual XP gain crosses threshold and awards JP", len(xp) == kills + 1 and xp[0].get("xp") == 799
          and xp[0].get("jp") == 0 and xp[1].get("xp") == 4 and xp[1].get("jp") == 3
          and xp[1]["xp"] + 800 - xp[0]["xp"] == 5
          and (kills == 1 or xp[2].get("xp") == 9 and xp[2].get("jp") == 6))
    check("killer XP is not broadcast to unrelated witness", w.get("experience")
          and all(p.get("handle") == witness.get("own") and p.get("xp") == 0 and p.get("jp") == 0 for p in w["experience"]))
    expected = {"hp": 140, "max_hp": 140, "mp": 70, "max_mp": 70}
    for role, trace in [("fighter", f), ("witness", w)]:
        packets = [p for p in trace.get("hpmp", []) if p.get("handle") == actor]
        check(f"{role} received correct post-level vitals", any(all(p.get(k) == v for k, v in expected.items()) for p in packets))
        check(f"{role} trace has no decoding/overflow errors", trace.get("errors") == [])
    stages = {s.get("stage"): s for s in witness.get("steps", [])}
    damaged = stages.get("DamageQuery", {}).get("target") or {}
    target = stages.get("Query", {}).get("target") or {}
    own = stages.get("Query", {}).get("player") or {}
    check("query during combat preserves current damage", 0 < damaged.get("hp", 0) < 120 and damaged.get("mp") == 60)
    check("witness adapter and Lua display agree after level", target.get("level") == 3
          and all(target.get(k) == v for k, v in expected.items())
          and "Target level: 3" in stages.get("Query", {}).get("display", "")
          and "Target HP: 140/140 MP: 70/70" in stages.get("Query", {}).get("display", ""))
    check("witness own character is unchanged", own.get("level") == 1 and own.get("hp") == 100 and own.get("mp") == 50
          and stages.get("Query", {}).get("xp_jp") == [0, 0])
    check("witness received fighter leave and cleared target", actor in w.get("leaves", []) and "Leave" in stages
          and stages["Leave"].get("target") is None)
    check("both Lua logout events observed", fighter.get("events", {}).get("PLAYER_LOGOUT") == 1
          and witness.get("events", {}).get("PLAYER_LOGOUT") == 1)
    if kills == 2:
        for role, report, trace in [("fighter", fighter, f), ("witness", witness, w)]:
            stages_role = {s.get("stage"): s for s in report.get("steps", [])}
            lifecycle = [e for e in trace.get("lifecycle", []) if e.get("handle") == monster]
            check(f"{role} observed corpse leave then same-handle respawn", len(lifecycle) >= 3
                  and [e.get("event") for e in lifecycle[:3]] == ["enter", "leave", "enter"]
                  and all(e.get("hp") == 105 and e.get("mp") == 53 for e in [lifecycle[0], lifecycle[2]]))
            check(f"{role} hidden corpse query was rejected", "HiddenQuery" in stages_role
                  and (role != "fighter" or stages_role["HiddenQuery"].get("target") is None))
            respawn = stages_role.get("Respawn", {})
            target_respawn = respawn.get("target") or {}
            check(f"{role} Lua target reads respawn vitals", target_respawn.get("handle") == monster
                  and target_respawn.get("hp") == 105 and target_respawn.get("mp") == 53
                  and "Target HP: 105/105 MP: 53/53" in respawn.get("display", ""))
        repeated = stages.get("RepeatQuery", {})
        target_repeat = repeated.get("target") or {}
        last_hp = retaliation[-1].get("target_hp") if retaliation else None
        check("second kill vitals and query agree without another level-up", last_hp is not None
              and 0 < last_hp < 140 and target_repeat.get("hp") == last_hp
              and target_repeat.get("level") == 3 and target_repeat.get("mp") == 70
              and f"Target HP: {last_hp}/140 MP: 70/70" in repeated.get("display", ""))
    return {"success": all(c["passed"] for c in checks), "attack_records": len(hits),
            "kills": kills, "fighter_hits": len(outgoing), "monster_hits": len(retaliation), "checks": checks}


if __name__ == "__main__":
    root = Path(sys.argv[1]).resolve(strict=True)
    result = assess(json.loads((root / "fighter.json").read_text()), json.loads((root / "witness.json").read_text()))
    (root / "comparison.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
    print(json.dumps(result, indent=2))
    sys.exit(0 if result["success"] else 1)
