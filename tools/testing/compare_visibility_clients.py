"""Read-only comparison of the native moving-player visibility regression."""
import json
from pathlib import Path
import sys


def near(position, x):
    return isinstance(position, (list, tuple)) and len(position) == 2 and abs(position[0] - x) < 0.5 and abs(position[1] - 7167) < 0.5


def assess(mover, witness):
    checks = []

    def check(name, value):
        checks.append({"name": name, "passed": bool(value)})

    actor, observer = mover.get("own"), witness.get("own")
    m, w = mover.get("trace", {}), witness.get("trace", {})
    ms = {s.get("stage"): s for s in mover.get("steps", [])}
    ws = {s.get("stage"): s for s in witness.get("steps", [])}
    check("both native visibility runs succeeded", mover.get("success") is True and witness.get("success") is True
          and mover.get("scenario") == witness.get("scenario") == "visibility")
    check("distinct clients agree on mover identity", actor and observer and actor != observer
          and mover.get("mover") == witness.get("mover") == actor)
    for role, report, trace, stages, start in [("mover", mover, m, ms, 8330), ("witness", witness, w, ws, 9230)]:
        joined = stages.get("Join", {})
        check(f"{role} starts outside peer visibility", near(joined.get("own_position"), start) and joined.get("peer_visible") is False)
        states = [s for s in report.get("steps", []) if s.get("player")]
        check(f"{role} retains real stats and rewards", states and all(s["player"].get("handle") == report.get("own")
              and s["player"].get("level") == 1 and s["player"].get("hp") == 100 and s["player"].get("mp") == 50
              and s.get("xp_jp") == [0, 0] and s.get("money") == 0 for s in states))
        check(f"{role} has no decode errors, combat, level change or teleport", trace.get("errors") == []
              and trace.get("hits") == [] and trace.get("levels") == [] and trace.get("warps") == [])
        experience = trace.get("experience", [])
        check(f"{role} receives only unchanged own XP", experience and all(e.get("handle") == report.get("own")
              and e.get("xp") == 0 and e.get("jp") == 0 for e in experience))
        events = report.get("events", {})
        check(f"{role} observes logout without death events", events.get("PLAYER_LOGOUT") == 1
              and events.get("PLAYER_DEAD", 0) == events.get("PLAYER_ALIVE", 0) == 0)
    mm = [p for p in m.get("moves", []) if p.get("handle") == actor]
    wm = [p for p in w.get("moves", []) if p.get("handle") == actor]
    we = [p for p in w.get("enters", []) if p.get("handle") == actor]
    check("mover sent the intended forward and return routes", len(mm) == 2
          and mm[0].get("points") == [[8500.0, 7167.0], [9050.0, 7167.0]] and mm[1].get("points") == [[8330.0, 7167.0]])
    check("late observer receives only unfinished forward waypoint", len(wm) == 2
          and wm[0].get("points") == [[9050.0, 7167.0]] and wm[1].get("points") == [[8330.0, 7167.0]])
    check("one actual ENTER occurs inside visibility while already moving", len(we) == 1
          and 8690 <= we[0].get("x", 0) < 9050 and we[0].get("y") == 7167 and we[0].get("hp") == 100 and we[0].get("mp") == 50)
    check("catch-up MOVE follows ENTER and uses the current clock", len(we) == 1 and len(wm) == len(mm) == 2
          and wm[0].get("order", 0) > we[0].get("order", 0) and wm[0].get("time", -1) > mm[0].get("time", -1))
    check("both views use the same speed and layer", len(wm) == len(mm) == 2
          and all(p.get("speed") == 120 and p.get("layer") == 0 for p in mm + wm))
    check("return movement is the same authoritative broadcast", len(wm) == len(mm) == 2
          and all(wm[1].get(k) == mm[1].get(k) for k in ("handle", "time", "speed", "layer", "points")))
    check("mover reaches destination while observer is visible", near(ms.get("Arrived", {}).get("own_position"), 9050)
          and ms.get("Arrived", {}).get("peer_visible") is True)
    arrival = ws.get("Arrived", {})
    target = arrival.get("target") or {}
    check("observer production state reaches destination without QUERY", near(arrival.get("mover_position"), 9050)
          and target.get("handle") == actor and "Target HP: 100/100 MP: 50/50" in arrival.get("display", ""))
    peer_enters = [e for e in m.get("enters", []) if e.get("handle") == observer]
    check("mover sees stationary observer then receives range LEAVE", len(peer_enters) == 1
          and peer_enters[0].get("x") == 9230 and peer_enters[0].get("y") == 7167 and m.get("leaves", []).count(observer) == 1)
    check("observer receives range LEAVE and clears target", w.get("leaves", []).count(actor) == 1
          and "RangeLeave" in ws and ws["RangeLeave"].get("target") is None and ws["RangeLeave"].get("mover_position") is None)
    check("query cannot reintroduce hidden mover", len(we) == 1 and "HiddenQuery" in ws
          and ws["HiddenQuery"].get("target") is None and ws["HiddenQuery"].get("mover_position") is None)
    check("observer never moves and Lua target changes twice", all(near(s.get("own_position"), 9230)
          for s in witness.get("steps", []) if s.get("player")) and witness.get("events", {}).get("PLAYER_TARGET_CHANGED") == 2)
    home = ms.get("Home", {})
    own_enter = [e for e in m.get("enters", []) if e.get("handle") == actor]
    check("return home is confirmed by server QUERY", near(home.get("own_position"), 8330) and home.get("peer_visible") is False
          and len(own_enter) == 1 and own_enter[0].get("x") == 8330 and own_enter[0].get("y") == 7167)
    return {"success": all(c["passed"] for c in checks), "checks": checks}


if __name__ == "__main__":
    directory = Path(sys.argv[1]).resolve(strict=True)
    result = assess(*[json.loads((directory / (role + ".json")).read_text()) for role in ("mover", "witness")])
    (directory / "comparison.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
    print(json.dumps(result, indent=2))
    sys.exit(0 if result["success"] else 1)
