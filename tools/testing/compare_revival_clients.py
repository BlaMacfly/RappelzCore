"""Read-only comparison of two native -validate-revival reports.

The victim attacks once, receives actual lethal retaliation, and is revived by
the existing server death handler. No SQL or network commands are issued here.
"""
import json
from pathlib import Path
import re
import sys

from compare_combat_clients import damage_sequence


def assess(victim, witness):
    checks = []

    def check(name, value):
        checks.append({"name": name, "passed": bool(value)})

    actor, monster = victim.get("victim"), victim.get("monster")
    v, w = victim.get("trace", {}), witness.get("trace", {})
    hits = v.get("hits", [])
    outgoing = [h for h in hits if h.get("attacker") == actor and h.get("target") == monster]
    incoming = [h for h in hits if h.get("attacker") == monster and h.get("target") == actor]
    check("both native revival runs succeeded", victim.get("success") is True and witness.get("success") is True
          and victim.get("scenario") == witness.get("scenario") == "revival")
    check("distinct clients observed the same victim and monster", actor and monster and actor != monster
          and victim.get("own") == actor and witness.get("victim") == actor
          and witness.get("monster") == monster and witness.get("own") not in (None, actor, monster))
    check("all real attack records match", hits and hits == w.get("hits"))
    check("only the two combatants attacked", hits and len(hits) == len(outgoing) + len(incoming))
    check("one victim attack leaves the monster alive", len(outgoing) == 1 and damage_sequence(outgoing, 105)
          and outgoing[0].get("target_hp", 0) > 0)
    starting_hp = outgoing[0].get("attacker_hp", 0) if outgoing else 0
    check("real retaliation kills the victim", 0 < starting_hp <= 40 and damage_sequence(incoming, starting_hp)
          and incoming[-1].get("target_hp") == 0)
    vd = [h for h in v.get("hpmp", []) if h.get("add_hp", 0) < 0]
    wd = [h for h in w.get("hpmp", []) if h.get("add_hp", 0) < 0]
    check("shared damage HPMP records match", vd and vd == wd)
    for role, report, trace in [("victim", victim, v), ("witness", witness, w)]:
        vitals = [h for h in trace.get("hpmp", []) if h.get("handle") == actor]
        zero = [i for i, h in enumerate(vitals) if h.get("hp") == 0]
        check(f"{role} sees one death followed by full revival", len(zero) == 1 and zero[0] < len(vitals) - 1
              and all(vitals[-1].get(k) == n for k, n in {"hp": 100, "max_hp": 100, "mp": 50, "max_mp": 50}.items()))
        check(f"{role} has no trace errors or level changes", trace.get("errors") == [] and trace.get("levels") == [])
        experience = trace.get("experience", [])
        check(f"{role} receives only its unchanged private XP", experience
              and all(e.get("handle") == report.get("own") and e.get("xp") == 0 and e.get("jp") == 0 for e in experience))
        stages = {s.get("stage"): s for s in report.get("steps", [])}
        observed = stages.get("Observe", {})
        check(f"{role} sees revival at spawn beyond corpse timeout", observed.get("victim_position") == [8330.0, 7167.0]
              and observed.get("victim_renderable") is True and observed.get("observation_ms", 0) >= 3500)
        own = observed.get("player") or {}
        check(f"{role} own stats and rewards are unchanged", own.get("handle") == report.get("own")
              and own.get("level") == 1 and own.get("hp") == 100
              and own.get("mp") == 50 and observed.get("xp_jp") == [0, 0] and observed.get("money") == 0)
        events = report.get("events", {})
        expected = 1 if role == "victim" else 0
        check(f"{role} has correct death/alive/logout events", events.get("PLAYER_DEAD", 0) == expected
              and events.get("PLAYER_ALIVE", 0) == expected and events.get("PLAYER_LOGOUT") == 1)
    warps = v.get("warps", [])
    check("only the victim receives the spawn WARP", len(warps) == 1
          and all(warps[0].get(k) == value for k, value in {"x": 8330.0, "y": 7167.0, "z": 0.0}.items())
          and warps[0].get("layer", 0) == 0 and w.get("warps") == [])
    messages = victim.get("messages", [])
    dead = [i for i, m in enumerate(messages) if "EVENT PLAYER_DEAD " in m and " HP=0/100 " in m]
    alive = [i for i, m in enumerate(messages) if "EVENT PLAYER_ALIVE " in m and " HP=100/100 " in m]
    check("Lua observes dead then alive with real health", len(dead) == len(alive) == 1 and dead[0] < alive[0])
    target_health = [m for m in witness.get("messages", []) if "EVENT UNIT_HEALTH target " in m]
    zero = [i for i, m in enumerate(target_health) if re.search(r" targetHP=0(?:\s|$)", m)]
    full = [i for i, m in enumerate(target_health) if re.search(r" targetHP=100(?:\s|$)", m)]
    check("witness Lua observes target zero then restored health", len(zero) == 1 and full and zero[0] < full[-1])
    stages = {s.get("stage"): s for s in witness.get("steps", [])}
    joined, observed, queried = [stages.get(s, {}) for s in ("Join", "Observe", "Query")]
    check("automatic observer refresh precedes manual query", joined.get("victim_enter_count") == 1
          and observed.get("victim_enter_count") == 2 and queried.get("victim_enter_count") == 3)
    target = observed.get("target") or {}
    check("witness retains the revived target and correct Lua display", target.get("handle") == actor
          and target.get("hp") == 100 and target.get("mp") == 50
          and "Target HP: 100/100 MP: 50/50" in observed.get("display", ""))
    enters = [e for e in w.get("enters", []) if e.get("handle") == actor]
    check("automatic ENTER and query have correct spawn stats", len(enters) == 3 and all(
        all(e.get(k) == n for k, n in {"x": 8330.0, "y": 7167.0, "hp": 100, "max_hp": 100,
                                      "mp": 50, "max_mp": 50, "level": 1}.items()) for e in enters[1:]))
    check("peer logout clears witness target", w.get("leaves", []).count(actor) == 1
          and "Leave" in stages and stages["Leave"].get("target") is None)
    return {"success": all(c["passed"] for c in checks), "attack_records": len(hits), "checks": checks}


if __name__ == "__main__":
    root = Path(sys.argv[1]).resolve(strict=True)
    result = assess(*[json.loads((root / (role + ".json")).read_text()) for role in ("victim", "witness")])
    (root / "comparison.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
    print(json.dumps(result, indent=2))
    sys.exit(0 if result["success"] else 1)
