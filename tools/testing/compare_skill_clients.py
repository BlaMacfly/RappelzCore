"""Read-only comparison of native skill learning, durable costs and observer privacy."""
import json
from pathlib import Path
import sys


def assess(owner, witness):
    checks = []
    def check(name, value): checks.append({"name": name, "passed": bool(value)})
    old, new, peer = owner.get("initial"), owner.get("own"), witness.get("own")
    ot, wt = owner.get("trace", {}), witness.get("trace", {})
    os = {s.get("stage"): s for s in owner.get("steps", [])}
    ws = {s.get("stage"): s for s in witness.get("steps", [])}
    check("both native learning runs completed", owner.get("success") is True and witness.get("success") is True
          and owner.get("scenario") == witness.get("scenario") == "skills")
    check("distinct clients agree on old and reentered owner identity", old and new and peer and len({old, new, peer}) == 3
          and owner.get("other") == peer and witness.get("initial_other") == old and witness.get("other") == new)
    def levels(packet): return [[s.get("id"), s.get("base")] for s in packet.get("levels", [])]
    def entry(catalog, id): return next((e for e in catalog.get("entries", []) if e.get("id") == id), {})
    def offer(catalog, id, learned, next, cost, reason):
        e = entry(catalog, id)
        return e and [e.get(k) for k in ("learned", "next", "cost", "reason")] == [learned, next, cost, reason]
    for role, report, trace, stages, handles, position, expected_jp, cycles, job, jlv in [
            ("owner", owner, ot, os, [old, new], [8440, 7227], [18, 14, 10, 0, 0], 2, 200, 3),
            ("witness", witness, wt, ws, [peer], [8420, 7240], [0], 1, 300, 1)]:
        logins = trace.get("logins", [])
        check(f"{role} received expected real logins", [p.get("handle") for p in logins] == handles
              and all(p.get("layer") == 0 for p in logins))
        check(f"{role} has no combat movement reward-level or decoding errors", trace.get("hits") == [] and trace.get("moves") == []
              and trace.get("warps") == [] and trace.get("levels") == [] and trace.get("errors") == [])
        xp = trace.get("experience", [])
        check(f"{role} JP balances follow private actual EXP_UPDATE packets", [p.get("jp") for p in xp] == expected_jp
              and all(p.get("handle") in handles and p.get("xp") == 0 for p in xp))
        cats, lists = trace.get("skill_catalogs", []), trace.get("skill_lists", [])
        check(f"{role} catalogs are private and retain actual job mapping", cats and all(c.get("target") in handles
              and c.get("job") == job and c.get("job_level") == jlv and c.get("status") == 0 for c in cats))
        check(f"{role} skill lists are private complete replacements", lists and all(p.get("target") in handles and p.get("modification") == 0
              and all(s.get("base") == s.get("current") for s in p.get("levels", [])) for p in lists))
        live = [s for s in report.get("steps", []) if isinstance(s.get("player"), dict)]
        check(f"{role} adapter snapshots agree with actual private wire state", live and all(
            s.get("catalog") in cats and any(p.get("target") == s["player"].get("handle") and levels(p) == s.get("learned") for p in lists)
            and any(p.get("handle") == s["player"].get("handle") and p.get("jp") == s.get("jp") for p in xp) for s in live))
        check(f"{role} stats positions and possessions remain unchanged", live and all(s["player"].get("handle") in handles
              and [s["player"].get(k) for k in ("level", "layer", "hp", "max_hp", "mp", "max_mp")] == [1, 0, 100, 100, 50, 50]
              and s.get("position") == position and s.get("money") == 0 and s.get("bag_revision") == 0
              and s.get("xp_jp") == [0, s.get("jp")] for s in live))
        check(f"{role} Lua displays accepted skill count and JP", live and all(isinstance(s.get("learned"), list)
              and f"\nSkills: {len(s['learned'])}  JP: {s.get('jp')}\n" in s.get("display", "") for s in live))
        events = report.get("events", {})
        count = 5 if role == "owner" else 1
        check(f"{role} lifecycle skill and JP events occur exactly once", events.get("PLAYER_LOGIN") == events.get("PLAYER_LOGOUT") == cycles
              and events.get("SKILL_LINES_CHANGED") == events.get("RAPPELZ_JP_CHANGED") == count
              and all(events.get(e, 0) == 0 for e in ("PLAYER_DEAD", "PLAYER_ALIVE", "PLAYER_LEVEL_UP", "BAG_UPDATE", "RAPPELZ_LAYER_CHANGED")))
    check("owner spends only 4 then 4 then 10 JP", len(ot.get("skill_catalogs", [])) == 5
          and [c.get("jp") for c in ot["skill_catalogs"]] == [18, 14, 10, 0, 0])
    expected_levels = [[], [[1004, 1]], [[1003, 1], [1004, 1]], [[1003, 2], [1004, 1]], [[1003, 2], [1004, 1]]]
    check("actual learned lists contain only the three confirmed purchases", [levels(p) for p in ot.get("skill_lists", [])] == expected_levels)
    initial = os.get("Initial", {}).get("catalog") or {}
    check("initial quotes expose actual learning cost and prerequisite", offer(initial, 1004, 0, 1, 4, 0)
          and offer(initial, 1003, 0, 1, 4, 2) and [1004, 1] in entry(initial, 1003).get("prerequisites", []))
    check("job-level requirement is enforced before purchasing", entry(initial, 1801).get("reason") == 3
          and entry(initial, 1801).get("job_level") == 5)
    for name, learned, jp in [("Denied", [], 18), ("FirstLearned", [[1004, 1]], 14), ("DuplicatesDenied", [[1004, 1]], 14),
                              ("SecondLearned", [[1003, 1], [1004, 1]], 10), ("Upgraded", [[1003, 2], [1004, 1]], 0),
                              ("Exhausted", [[1003, 2], [1004, 1]], 0), ("Reentered", [[1003, 2], [1004, 1]], 0)]:
        check(f"owner progression is correct at {name}", name in os and os[name].get("learned") == learned and os[name].get("jp") == jp)
    check("learning unlocks and reprices the next level", offer(os.get("FirstLearned", {}).get("catalog") or {}, 1003, 0, 1, 4, 0)
          and offer(os.get("SecondLearned", {}).get("catalog") or {}, 1003, 1, 2, 10, 0)
          and offer(os.get("Upgraded", {}).get("catalog") or {}, 1003, 2, 3, 62, 1))
    requests, results = owner.get("requests", []), ot.get("learn_results", [])
    expected = [(peer, 1004, 0, 1), (old, 1004, 1, 1), (old, 1004, 0, -1), (old, 1004, 0, 2), (old, 2602, 0, 1),
                (old, 1801, 0, 1), (old, 1003, 0, 1), (old, 2147483647, 0, 1), (old, 1004, 0, 1), (old, 1004, 0, 1),
                (old, 1004, 0, 1), (old, 1003, 0, 1), (old, 1003, 0, 2), (old, 1003, 0, 3), (old, 1004, 0, 2)]
    check("real request sequence probes ownership type ranges tree prerequisites and duplicates", [tuple(p.get(k) for k in ("target", "id", "random", "level")) for p in requests] == expected)
    check("only the three valid next-level requests receive success", len(results) == 15
          and [i for i, code in enumerate(results) if code == 0] == [8, 11, 12])
    check("refusals report stale requests wrong job prerequisites and insufficient JP", all(any(word in m for m in owner.get("messages", []))
          for word in ("périmée", "arbre actuel", "métier insuffisant", "préalable", "pas assez de JP", "maximal")))
    check("witness cannot learn without JP or alter the owner's skills", len(wt.get("learn_results", [])) == 2
          and all(r != 0 for r in wt["learn_results"]) and len(witness.get("requests", [])) == 2
          and [r.get("target") for r in witness["requests"]] == [peer, old])
    check("witness never receives the owner's catalog or learned list", len(wt.get("skill_catalogs", [])) == len(wt.get("skill_lists", [])) == 1
          and levels(wt["skill_lists"][0]) == [] and all(ws.get(s, {}).get("learned") == [] and ws[s].get("jp") == 0 for s in ("Denied", "Inspected", "FreshQuery")))
    check("both old and reentered owner sessions are freshly queried then leave", all(
        sum(e.get("handle") == h and e.get("kind") == "player" for e in wt.get("enters", [])) >= 2 and wt.get("leaves", []).count(h) == 1 for h in (old, new)))
    check("saved learning loads into a new handle and logout completes", "Lobby" in os and "Reentered" in os and os["Reentered"].get("player", {}).get("handle") == new
          and "Logout" in os and "PeerLogout" in ws and "Logout" in ws)
    changes = [m for m in owner.get("messages", []) if "EVENT RAPPELZ_JP_CHANGED " in m]
    check("Lua JP event arguments follow actual balances including known zero", len(changes) == 5
          and all(f"EVENT RAPPELZ_JP_CHANGED {jp} " in message for jp, message in zip((18, 14, 10, 0, 0), changes)))
    return {"success": all(c["passed"] for c in checks), "checks": checks}


if __name__ == "__main__":
    directory = Path(sys.argv[1]).resolve(strict=True)
    result = assess(*[json.loads((directory / (role + ".json")).read_text(encoding="utf-8")) for role in ("owner", "witness")])
    (directory / "comparison.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
    print(json.dumps(result, indent=2))
    sys.exit(0 if result["success"] else 1)
