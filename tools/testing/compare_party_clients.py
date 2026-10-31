"""Compare real -validate-party reports; reads gameplay reports, never databases."""
import json
from pathlib import Path
import sys


def assess(leader, member):
    checks = []
    def check(name, value):
        checks.append({"name": name, "passed": bool(value)})
    l, m = leader.get("own"), member.get("own")
    lt, mt = leader.get("trace", {}), member.get("trace", {})
    ls = {s.get("stage"): s for s in leader.get("steps", [])}
    ms = {s.get("stage"): s for s in member.get("steps", [])}
    check("both real native party runs succeeded", leader.get("success") is True and member.get("success") is True
          and leader.get("scenario") == member.get("scenario") == "party")
    check("distinct clients agree on membership identity", l and m and l != m and leader.get("other") == m and member.get("other") == l)
    ids = []
    for role, report, trace, stages, own, peer in [("leader", leader, lt, ls, l, m), ("member", member, mt, ms, m, l)]:
        wire = trace.get("party_states", [])
        states = [s for s in report.get("steps", []) if isinstance(s.get("party"), dict)]
        groups = list(dict.fromkeys(s.get("id") for s in wire if s.get("members")))
        ids.append(groups)
        check(f"{role} received a real login and decoded bounded traces", len(trace.get("logins", [])) == 1
              and trace["logins"][0].get("handle") == own and trace["logins"][0].get("layer") == 0 and trace.get("errors") == [])
        e = report.get("events", {})
        check(f"{role} receives one actual party state discovery", e.get("RAPPELZ_PARTY_STATE_READY") == 1)
        check(f"{role} lifecycle and roster event counts match", e.get("PLAYER_LOGIN") == e.get("PLAYER_LOGOUT") == 1
              and e.get("PLAYER_DEAD", 0) == e.get("PLAYER_ALIVE", 0) == 0
              and e.get("RAPPELZ_LAYER_CHANGED", 0) == (2 if role == "leader" else 0)
              and e.get("PARTY_MEMBERS_CHANGED") == (6 if role == "leader" else 7))
        check(f"{role} begins with confirmed solo state", stages.get("Solo", {}).get("party_count") == 0
              and stages.get("Solo", {}).get("party") == {"id": 0, "leader": 0, "members": [], "invite": None})
        check(f"{role} declined invitation never creates a group", "Declined" in stages
              and stages["Declined"].get("party_count") == 0 and stages["Declined"].get("party", {}).get("members") == [])
        check(f"{role} adapter snapshots come from actual wire states", states and all(s["party"] in wire for s in states))
        def adapter(s):
            roster = s["party"].get("members", [])
            others = [u for u in roster if u.get("handle") != own]
            units = s.get("party_units")
            keys = ("handle", "name", "level", "layer", "hp", "max_hp", "mp", "max_mp")
            return (s.get("party_count") == len(others) and isinstance(units, list) and len(units) == len(others)
                    and all(all(a.get(k) == b.get(k) for k in keys) for a, b in zip(units, others))
                    and f"\nParty: {len(others)} " in s.get("display", ""))
        check(f"{role} Lua roster units and displayed counts reflect adapter", states and all(adapter(s) for s in states))
        check(f"{role} sees exactly three unique consented groups", len(groups) == 3 and all(groups)
              and all(len(s.get("members", [])) == 2 and {u.get("handle") for u in s["members"]} == {l, m}
                      and s.get("leader") in (l, m) for s in wire if s.get("members")))
        check(f"{role} receives only unchanged private XP", trace.get("experience") and all(
            p.get("handle") == own and p.get("xp") == p.get("jp") == 0 for p in trace["experience"]))
        for stage in ("Kicked", "Left"):
            check(f"{role} becomes solo after {stage}", stages.get(stage, {}).get("party_count") == 0
                  and stages.get(stage, {}).get("party", {}).get("members") == [])
        check(f"{role} observes leadership transfer", stages.get("Transferred", {}).get("party", {}).get("leader") == m)
    check("both clients agree on all three distinct group ids", ids[0] == ids[1] and len(ids[0]) == 3)
    check("member receives and clears four invitations exactly once", member.get("events", {}).get("PARTY_INVITE_REQUEST") == 4
          and member.get("events", {}).get("RAPPELZ_PARTY_INVITE_CLOSED") == 4
          and leader.get("events", {}).get("PARTY_INVITE_REQUEST", 0) == 0
          and (ms.get("Invited", {}).get("party", {}).get("invite") or {}).get("inviter") == l)
    check("ordinary member cannot disband the group", "PermissionDenied" in ms
          and ms["PermissionDenied"].get("party_count") == 1 and any("Seul le chef" in x for x in member.get("messages", [])))
    def roster_member(step, handle):
        return next((u for u in step.get("party", {}).get("members", []) if u.get("handle") == handle), {})
    ld, md = ls.get("DamagedIsolated", {}), ms.get("DamagedIsolated", {})
    damage = roster_member(ld, l)
    hp = damage.get("hp", 0)
    check("real damaged roster agrees across layers without visible entity", 0 < hp < 100
          and damage == roster_member(md, l) and damage.get("layer") == 1 and damage.get("mp") == 50
          and ld.get("party_count") == md.get("party_count") == 1
          and ld.get("peer_visible") is False and md.get("peer_visible") is False
          and ld.get("player", {}).get("hp") == hp and ld.get("player", {}).get("layer") == 1)
    hits = [h for h in lt.get("hits", []) if h.get("target") == l]
    check("damage originates in matching real monster attacks", hits and lt.get("hits") == mt.get("hits")
          and all(h.get("attacker") == leader.get("monster") and h.get("damage", 0) > 0 for h in hits)
          and 100 - sum(h["damage"] for h in hits) == hp)
    check("both clients received matching damaged HPMP", any(p.get("handle") == l and p.get("hp") == hp for p in lt.get("hpmp", []))
          and any(p.get("handle") == l and p.get("hp") == hp for p in mt.get("hpmp", [])))
    check("member Lua observes real party health changes", member.get("events", {}).get("UNIT_HEALTH", 0) >= 3
          and any("EVENT UNIT_HEALTH party1 " in x and f"partyHP={hp}" in x for x in member.get("messages", [])))
    check("both regain healed layer-zero roster and visibility", all(
        roster_member(s.get("Restored", {}), l).get("hp") == 100
        and roster_member(s.get("Restored", {}), l).get("mp") == 50
        and roster_member(s.get("Restored", {}), l).get("layer") == 0 and s.get("Restored", {}).get("peer_visible") is True for s in (ls, ms)))
    chats = lt.get("party_chats", [])
    check("group chats agree and excluded sender cannot broadcast", len(chats) == 2 and chats == mt.get("party_chats")
          and {c.get("message") for c in chats} == {"LuaPartyLeader", "LuaPartyMember"}
          and any("Vous n'êtes pas dans un groupe" in x for x in leader.get("messages", [])))
    check("chat events are delivered once per real message", leader.get("events", {}).get("CHAT_MSG_PARTY") == member.get("events", {}).get("CHAT_MSG_PARTY") == 2)
    check("leader logout clears remaining member roster and entity", ms.get("PeerLogout", {}).get("party_count") == 0
          and ms.get("PeerLogout", {}).get("peer_visible") is False and l in mt.get("leaves", []))
    return {"success": all(c["passed"] for c in checks), "checks": checks}


if __name__ == "__main__":
    directory = Path(sys.argv[1]).resolve(strict=True)
    result = assess(*[json.loads((directory / (role + ".json")).read_text(encoding="utf-8")) for role in ("leader", "member")])
    (directory / "comparison.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
    print(json.dumps(result, indent=2))
    sys.exit(0 if result["success"] else 1)
