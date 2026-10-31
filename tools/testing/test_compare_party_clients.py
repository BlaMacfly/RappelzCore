import copy
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from compare_party_clients import assess


def reports():
    def unit(handle, hp=100, layer=0):
        return dict(handle=handle, name=f"Fixture{handle}", level=1, layer=layer, hp=hp, max_hp=100, mp=50, max_mp=50)
    def party(id=0, leader=1, hp=100, layer=0):
        return {"id": id, "leader": leader if id else 0, "members": [unit(1, hp, layer), unit(2)] if id else [], "invite": None}
    groups = [party(10), party(11), party(12)]
    damaged = party(10, hp=82, layer=1)
    transferred = party(10, leader=2)
    invite = {**party(), "invite": {"inviter": 1, "name": "Fixture1", "seconds": 60}}
    def step(stage, p, own, visible=True):
        others = [copy.deepcopy(u) for u in p["members"] if u["handle"] != own]
        player = next((copy.deepcopy(u) for u in p["members"] if u["handle"] == own), unit(own))
        return {"stage": stage, "party": copy.deepcopy(p), "party_count": len(others), "party_units": others,
                "player": player, "peer_visible": visible, "display": f"Player\nParty: {len(others)}  Member"}
    result = []
    for own, role in [(1, "leader"), (2, "member")]:
        steps = [step("Solo", party(), own), step("Declined", party(), own), step("Joined", groups[0], own),
                 step("DamagedIsolated", damaged, own, False), step("Restored", groups[0], own),
                 step("Transferred", transferred, own), step("Kicked", party(), own),
                 step("Rejoined", groups[1], own), step("Left", party(), own), step("FinalGroup", groups[2], own)]
        wire = [copy.deepcopy(s["party"]) for s in steps]
        messages = ["EVENT UNIT_HEALTH party1 partyHP=82", "Vous n'êtes pas dans un groupe."]
        events = {"PLAYER_LOGIN": 1, "PLAYER_LOGOUT": 1, "RAPPELZ_PARTY_STATE_READY": 1,
                  "PARTY_MEMBERS_CHANGED": 6 if own == 1 else 7, "RAPPELZ_LAYER_CHANGED": 2 if own == 1 else 0,
                  "CHAT_MSG_PARTY": 2, "UNIT_HEALTH": 6}
        if own == 2:
            steps += [step("Invited", invite, own), step("PermissionDenied", groups[0], own), step("PeerLogout", party(), own, False)]
            wire += [copy.deepcopy(invite), party()]
            events.update(PARTY_INVITE_REQUEST=4, RAPPELZ_PARTY_INVITE_CLOSED=4)
            messages += ["Seul le chef du groupe peut effectuer cette action."]
        trace = {"logins": [{"handle": own, "layer": 0}], "party_states": wire, "errors": [],
                 "experience": [{"handle": own, "xp": 0, "jp": 0}],
                 "hits": [{"attacker": 9, "target": 1, "damage": 18}], "hpmp": [{"handle": 1, "hp": 82}],
                 "party_chats": [{"sender": "Fixture1", "message": "LuaPartyLeader"}, {"sender": "Fixture2", "message": "LuaPartyMember"}], "leaves": [1]}
        result.append(dict(scenario="party", success=True, role=role, own=own, other=3-own, monster=9,
                           steps=steps, trace=trace, events=events, messages=messages))
    return result


class PartyComparisonTest(unittest.TestCase):
    def test_independent_consented_group_lifecycle_and_real_damage_sample(self):
        result = assess(*reports())
        self.assertTrue(result["success"], result)

    def test_success_flags_alone_are_not_evidence(self):
        self.assertFalse(assess({"success": True}, {"success": True})["success"])

    def test_damaged_roster_lua_permissions_events_chat_and_lifecycle_mutations_fail(self):
        for case in ("damage", "lua", "permission", "events", "chat", "logout", "identity"):
            with self.subTest(case=case):
                leader, member = reports()
                if case == "damage":
                    next(s for s in member["steps"] if s["stage"] == "DamagedIsolated")["party"]["members"][0]["hp"] = 100
                elif case == "lua": member["steps"][0]["display"] = "Party: ?"
                elif case == "permission": member["messages"] = []
                elif case == "events": member["events"]["PARTY_INVITE_REQUEST"] = 5
                elif case == "chat": member["trace"]["party_chats"].append({"message": "LuaPartyOutside"})
                elif case == "logout": member["trace"]["leaves"] = []
                else: member["other"] = 2
                self.assertFalse(assess(leader, member)["success"])

    def test_cli_reads_utf8_server_notices_and_writes_comparison(self):
        with tempfile.TemporaryDirectory() as directory:
            for role, report in zip(("leader", "member"), reports()):
                Path(directory, role + ".json").write_text(json.dumps(report, ensure_ascii=False), encoding="utf-8")
            result = subprocess.run([sys.executable, str(Path(__file__).with_name("compare_party_clients.py")), directory], capture_output=True)
            self.assertEqual(result.returncode, 0, result.stdout.decode("utf-8", errors="replace"))
            self.assertTrue(json.loads(Path(directory, "comparison.json").read_text(encoding="utf-8"))["success"])


if __name__ == "__main__":
    unittest.main()
