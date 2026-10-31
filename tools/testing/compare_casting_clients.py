"""Compare native self-restoration wire traces, interruption, privacy and real Lua state."""
import json
from pathlib import Path
import sys


def assess(owner, witness):
    checks = []
    def check(name, value): checks.append({"name": name, "passed": bool(value)})
    old, new, peer = owner.get("initial"), owner.get("own"), witness.get("own")
    ot, wt = owner.get("trace", {}), witness.get("trace", {})
    os = {p.get("stage"): p for p in owner.get("steps", [])}
    ws = {p.get("stage"): p for p in witness.get("steps", [])}
    events = ot.get("cast_events", [])
    expected = ["CASTING", "CANCEL", "CASTING", "FIRE"] * 2
    check("both guarded native runs completed", owner.get("success") is True and witness.get("success") is True and owner.get("scenario") == witness.get("scenario") == "casting")
    check("both clients agree on old fresh and witness identities", old and new and peer and len({old, new, peer}) == 3 and witness.get("initial_other") == old and witness.get("other") == new and owner.get("other") == peer)
    check("exact cast cancel fire sequence", [e.get("phase") for e in events] == expected)
    keys = ("id", "level", "caster", "target", "layer", "phase", "duration", "amount", "mp_cost", "caster_hp", "caster_mp")
    check("owner and observer received identical cast facts", len(events) == 8 and [[e.get(k) for k in keys] for e in events] == [[e.get(k) for k in keys] for e in wt.get("cast_events", [])])
    check("events refer only to real owning handles", len(events) == 8 and [e.get("caster") for e in events] == [old]*4+[new]*4 and all(e.get("caster") == e.get("target") and e.get("id") == 3212 and e.get("level") == 1 and e.get("layer") == 0 for e in events))
    check("actual casting duration is 145 centiseconds", len(events) == 8 and all(e.get("duration") == (145 if e.get("phase") == "CASTING" else 0) for e in events))
    check("MP costs occur only at the two fires", len(events) == 8 and all(e.get("mp_cost") == (5 if e.get("phase") == "FIRE" else 0) for e in events))
    check("nominal recovery is distinct from capped mana", len(events) == 8 and all(e.get("amount") == (3500 if e.get("phase") == "FIRE" else 0) and e.get("caster_hp") == 100 and (e.get("phase") != "FIRE" or e.get("caster_mp") == 50) for e in events))
    check("thirteen owner requests have exactly four accepted casts", len(owner.get("requests", [])) == 13 and len(ot.get("cast_results", [])) == 13 and [i for i, r in enumerate(ot.get("cast_results", [])) if r == 0] == [6, 7, 11, 12])
    check("unlearned and foreign witness casts were denied", len(witness.get("requests", [])) == 2 and len(wt.get("cast_results", [])) == 2 and all(r != 0 for r in wt.get("cast_results", [])))
    check("six initial owner probes precede learning", len(owner.get("requests", [])) == 13 and owner["requests"][0].get("caster") == peer and owner["requests"][1].get("target") == peer and owner["requests"][2].get("id") == 65535 and owner["requests"][4].get("id") == 63203 and owner["requests"][5].get("level") == 0)
    check("learning succeeds once before valid casting", ot.get("learn_results") == [0] and wt.get("learn_results") == [] and os.get("Initial", {}).get("jp") == 180 and os.get("Learned", {}).get("jp") == 0)
    lists = ot.get("skill_lists", [])
    def skill(packet): return next((s for s in packet.get("levels", []) if s.get("id") == 3212), {})
    check("five private owner lists include persisted base levels", len(lists) == 5 and [p.get("target") for p in lists] == [old]*3+[new]*2 and [sorted((s.get("id"), s.get("base")) for s in p.get("levels", [])) for p in lists] == [[(63203, 1)]] + [[(3212, 1), (63203, 1)]]*4)
    check("private cooldown packets survive fresh character load", len(lists) == 5 and skill(lists[1]).get("remain") == 0 and skill(lists[2]).get("remain") == 2500 and 0 < skill(lists[3]).get("remain", 0) <= 2500 and skill(lists[4]).get("remain") == 2500 and all(skill(p).get("total") == 2500 for p in lists[1:]))
    check("witness never receives foreign learned levels or cooldowns", len(wt.get("skill_lists", [])) == 1 and wt["skill_lists"][0].get("target") == peer and wt["skill_lists"][0].get("levels") == [])
    check("cooldown is restored then really expires", os.get("Reentered", {}).get("remaining", 0) > 0 and os.get("CooldownExpired", {}).get("remaining") == 0 and os.get("MoveCanceled", {}).get("remaining") == 0 and os.get("SecondFire", {}).get("remaining", 0) > 0)
    check("explicit and movement interruptions complete", os.get("CastingCancel", {}).get("casting") is True and os.get("Canceled", {}).get("casting") is False and os.get("CastingMove", {}).get("casting") is True and os.get("MoveCanceled", {}).get("casting") is False)
    check("both fires reach accepted actual mana state", all(os.get(name, {}).get("player", {}).get("mp") == 50 and os.get(name, {}).get("casting") is False for name in ("FirstFire", "Reentered", "SecondFire")))
    check("fresh owner is queried then leaves normally", "Lobby" in os and "Reentered" in os and "FreshQuery" in ws and sum(e.get("handle") == new for e in wt.get("enters", [])) >= 2 and wt.get("leaves", []).count(new) == 1 and "PeerLogout" in ws)
    for role, report, trace, stages, handles, position, cycles in [("owner", owner, ot, os, [old, new], [8440, 7227], 2), ("witness", witness, wt, ws, [peer], [8420, 7240], 1)]:
        check(f"{role} login logout lifecycle is real", [p.get("handle") for p in trace.get("logins", [])] == handles and report.get("events", {}).get("PLAYER_LOGIN") == cycles and report.get("events", {}).get("PLAYER_LOGOUT") == cycles and "Logout" in stages)
        check(f"{role} has no attack warp level or parser errors", trace.get("hits") == [] and trace.get("warps") == [] and trace.get("levels") == [] and trace.get("errors") == [])
        active = [p for p in stages.values() if isinstance(p.get("player"), dict)]
        check(f"{role} unchanged HP level layer money bag and position", len(active) >= 4 and all(p["player"].get("hp") == p["player"].get("max_hp") == 100 and p["player"].get("level") == 1 and p["player"].get("layer") == 0 and p.get("money") == p.get("bag_revision") == 0 and p.get("position") == position for p in active))
        check(f"{role} actual Lua receives eight cast phase events", report.get("events", {}).get("RAPPELZ_SKILL_EVENT") == 8 and len([m for m in report.get("messages", []) if "SKILL 3212 " in m]) == 8)
        check(f"{role} learns no fake XP or JP from casting", [e.get("xp") for e in trace.get("experience", [])] == [0]*len(handles) + ([0] if role == "owner" else []) and [e.get("jp") for e in trace.get("experience", [])] == ([180, 0, 0] if role == "owner" else [0]))
        check(f"{role} HPMP state remains bounded", bool(trace.get("hpmp")) and all(h.get("hp") == h.get("max_hp") == 100 and h.get("add_hp") == 0 and 0 <= h.get("mp", -1) <= h.get("max_mp", -1) == 50 for h in trace.get("hpmp", [])))
    moves = ot.get("moves", [])
    check("movement interruption uses normal same-position route", len(moves) == 1 and moves[0].get("handle") == new and moves[0].get("points") == [[8440, 7227]] and wt.get("moves") and wt["moves"][0].get("points") == [[8440, 7227]])
    check("witness keeps own empty skills JP and mana", all(p.get("learned") == {} and p.get("jp") == 0 and p.get("player", {}).get("mp") == 50 for p in ws.values() if isinstance(p.get("player"), dict)))
    messages=owner.get("messages", [])
    fires=[messages[i+1] for i,m in enumerate(messages[:-1]) if "SKILL 3212 FIRE " in m]
    check("Lua FIRE callbacks read post-effect mana before duplicate HPMP", len(fires)==2 and all("EVENT RAPPELZ_SKILL_EVENT 3212 " in m and "MP=50/50" in m for m in fires))
    return {"success": all(c["passed"] for c in checks), "checks": checks}


if __name__ == "__main__":
    directory = Path(sys.argv[1]).resolve(strict=True)
    result = assess(*[json.loads((directory / (role + ".json")).read_text(encoding="utf-8")) for role in ("owner", "witness")])
    (directory / "comparison.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
    print(json.dumps(result, indent=2))
    sys.exit(0 if result["success"] else 1)
