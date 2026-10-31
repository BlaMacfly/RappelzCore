"""Read-only comparison of the two native -validate-layer reports."""
import json
from pathlib import Path
import sys


def assess(operator, witness):
    checks = []

    def check(name, value):
        checks.append({"name": name, "passed": bool(value)})

    o, w = operator.get("trace", {}), witness.get("trace", {})
    old, new, peer = operator.get("old_operator"), operator.get("operator"), witness.get("own")
    os = {s.get("stage"): s for s in operator.get("steps", [])}
    ws = {s.get("stage"): s for s in witness.get("steps", [])}
    check("both native layer runs succeeded", operator.get("success") is True and witness.get("success") is True
          and operator.get("scenario") == witness.get("scenario") == "layer")
    check("distinct clients agree on old and restored operator identity", old and new and peer and len({old, new, peer}) == 3
          and operator.get("own") == new and witness.get("old_operator") == old and witness.get("operator") == new)
    for role, report, trace, stages, position, logins in [
            ("operator", operator, o, os, [8440, 7227], 2), ("witness", witness, w, ws, [8420, 7240], 1)]:
        login_packets = trace.get("logins", [])
        handles = {p.get("handle") for p in login_packets}
        check(f"{role} has expected real logins", len(login_packets) == logins
              and [p.get("layer") for p in login_packets] == ([0, 1] if role == "operator" else [0]))
        live = [s for s in report.get("steps", []) if s.get("player")]
        check(f"{role} state adapter retains authoritative layer and stats", live and all(
            s["player"].get("handle") in handles and s["player"].get("layer") == s.get("layer")
            and s["player"].get("level") == 1 and s["player"].get("hp") == 100 and s["player"].get("mp") == 50
            and s.get("xp_jp") == [0, 0] and s.get("money") == 0 and s.get("bag_revision") == 0 and s.get("position") == position for s in live))
        xp = trace.get("experience", [])
        check(f"{role} receives only unchanged own XP", xp and all(p.get("handle") in handles
              and p.get("xp") == 0 and p.get("jp") == 0 for p in xp))
        check(f"{role} has no combat, movement, level or decoding errors", trace.get("hits") == []
              and trace.get("moves") == [] and trace.get("levels") == [] and trace.get("errors") == [])
        events = report.get("events", {})
        check(f"{role} lifecycle and layer event counts agree", events.get("PLAYER_LOGIN") == logins
              and events.get("PLAYER_LOGOUT") == logins and events.get("RAPPELZ_LAYER_CHANGED", 0) == (2 if role == "operator" else 0)
              and events.get("PLAYER_DEAD", 0) == events.get("PLAYER_ALIVE", 0) == 0)
    check("ordinary player layer command is refused", "PermissionDenied" in ws and ws["PermissionDenied"].get("layer") == 0
          and any("niveau requis" in m for m in witness.get("messages", [])) and w.get("warps") == [])
    check("out-of-range layer value is refused", "InvalidLayerDenied" in os and os["InvalidLayerDenied"].get("layer") == 0
          and any("Usage : .layer" in m for m in operator.get("messages", [])))
    warps = o.get("warps", [])
    check("operator changes 0 to 1 then back to 0 at fixed position", len(warps) == 2 and [p.get("layer") for p in warps] == [1, 0]
          and all(p.get("x") == 8440 and p.get("y") == 7227 and p.get("z") == 0 for p in warps))
    away, isolated, reentered = [os.get(s, {}) for s in ("Away", "Isolated", "Reentered")]
    check("old layer entities target and NPC UI disappear", away.get("layer") == 1 and away.get("entities") == 0
          and away.get("target") is None and away.get("dialog_open") is False)
    check("hidden NPC contact and market do not reopen UI", isolated.get("layer") == 1 and isolated.get("entities") == 0
          and isolated.get("dialog_open") is False and isolated.get("target") is None)
    check("layer one persists through real logout and relogin", "Disconnected" in os
          and reentered.get("layer") == 1 and reentered.get("entities") == 0 and reentered.get("target") is None)
    for stage in ("Away", "Isolated", "Reentered"):
        check(f"Lua displays actual layer one at {stage}", "\nLayer: 1\n" in os.get(stage, {}).get("display", ""))
    wa = ws.get("Isolated", {})
    check("witness stays in base layer with other content and no operator", wa.get("layer") == 0 and wa.get("entities", 0) > 0
          and wa.get("target") is None and "\nLayer: 0\n" in wa.get("display", ""))
    dialogs = o.get("dialogs", [])
    check("NPC dialogues are allowed before and after layer restoration", os.get("InitialDialog", {}).get("dialog_open") is True
          and os.get("Restored", {}).get("dialog_open") is True and len(dialogs) == 2
          and all(p.get("npc") == operator.get("npc") and p.get("kind") == "dialog" for p in dialogs) and w.get("dialogs") == [])
    check("hidden monster attack is explicitly denied", o.get("denied_attacks") == [{"attacker": old, "target": operator.get("monster"), "reason": 1}])
    check("hidden targeting is refused and restored targeting succeeds", o.get("targets") == [operator.get("npc"), 0, peer]
          and w.get("targets") == [old, 0, new])
    oe = [e for e in o.get("enters", []) if e.get("kind") == "player"]
    we = [e for e in w.get("enters", []) if e.get("kind") == "player"]
    check("cross-layer QUERY produces no extra player ENTER", [e.get("handle") for e in oe] == [peer, peer, peer]
          and [e.get("handle") for e in we] == [old, new, new] and all(e.get("layer") == 0 for e in oe + we))
    check("old and restored sessions both leave the witness", w.get("leaves", []).count(old) == 1
          and w.get("leaves", []).count(new) == 1 and o.get("leaves", []).count(peer) == 1)
    for role, stages, handle in [("operator", os, peer), ("witness", ws, new)]:
        restored = stages.get("Restored", {})
        target = restored.get("target") or {}
        check(f"{role} regains a real layer-zero target and Lua state", restored.get("layer") == 0 and restored.get("entities", 0) > 0
              and target.get("handle") == handle and target.get("layer") == 0 and target.get("hp") == 100 and target.get("mp") == 50
              and "\nLayer: 0\n" in restored.get("display", "") and "Target layer: 0" in restored.get("display", ""))
    query = ws.get("Queried", {}).get("target") or {}
    check("fresh restored query retains correct vitals and layer", query.get("handle") == new and query.get("layer") == 0
          and query.get("hp") == 100 and query.get("mp") == 50)
    check("final operator logout clears witness target", "PeerLogout" in ws and ws["PeerLogout"].get("target") is None
          and witness.get("events", {}).get("PLAYER_TARGET_CHANGED") == 4)
    changes = [m for m in operator.get("messages", []) if "EVENT RAPPELZ_LAYER_CHANGED " in m]
    check("Lua layer change arguments follow real transitions exactly once", len(changes) == 2
          and "RAPPELZ_LAYER_CHANGED 1 " in changes[0] and "RAPPELZ_LAYER_CHANGED 0 " in changes[1])
    return {"success": all(c["passed"] for c in checks), "checks": checks}


if __name__ == "__main__":
    directory = Path(sys.argv[1]).resolve(strict=True)
    result = assess(*[json.loads((directory / (role + ".json")).read_text()) for role in ("operator", "witness")])
    (directory / "comparison.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
    print(json.dumps(result, indent=2))
    sys.exit(0 if result["success"] else 1)
