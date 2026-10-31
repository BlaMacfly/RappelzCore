import copy
import unittest

from compare_terrain_clients import compare, ROLES


def fixture():
    reports = {}
    for renderer in ("reference", "optimized"):
        pair = {}
        for role in ROLES:
            y, peer_y = (7227, 7247) if role == "owner" else (7247, 7227)
            own, peer = (10, 11) if role == "owner" else (11, 10)
            n = 49 if renderer == "reference" else 15
            cache = {"bake_px":1024,"bytes_per_chunk":4194304,"ready":True,
                     "reference":renderer == "reference","resident":n,"wanted":n,
                     "texture_payload_bytes":n*4194304,"peak_texture_payload_bytes":n*4194304,"evicted":12}
            steps = [{"phase":p,"own":own,"peer":peer,"position":[10540 if p == 4 else 8440,y],
                      "peer_position":[10540 if p == 4 else 8440,peer_y],"level":1,"hp":100,"mp":50,
                      "xp_jp":[0,0],"layer":0,"money":0,"cache":copy.deepcopy(cache)} for p in range(6)]
            trace = {k:[] for k in ("errors","hits","levels","warps","cast_events","cast_results","learn_results")}
            trace.update(logins=[{"handle":own}], moves=[{"handle":h,"layer":0,"points":[[x,ty]]}
                         for h,ty in ((own,y),(peer,peer_y)) for x in (10540,8440)])
            pair[role] = {"success":True,"scenario":"terrain","renderer":renderer,"role":role,"steps":steps,
                          "trace":trace,"events":{k:1 for k in ("PLAYER_LOGIN","PLAYER_ENTERING_WORLD","PLAYER_LOGOUT")}}
        reports[renderer] = pair
    pictures = {role:[{"phase":p,"size":[1280,720],"reference_sha256":"a"*64,"optimized_sha256":"a"*64}
                      for p in range(6)] for role in ROLES}
    gpu = {r:[{"renderer":r,"role":role,"phase":p,"process_id":10,"Dedicated Usage":100,
               "Shared Usage":20,"Total Committed":1000 if r == "reference" else 500}
              for role in ROLES for p in range(6) for _ in range(3)] for r in reports}
    return reports["reference"], reports["optimized"], pictures, gpu


class TerrainComparisonTests(unittest.TestCase):
    def test_complete_synchronized_pair_and_identical_images_pass(self):
        result = compare(*fixture())
        self.assertTrue(result["success"])
        self.assertGreater(len(result["checks"]), 30)

    def test_blur_changed_pixels_or_stale_textures_are_detected(self):
        for mutation in ("resolution", "pixels", "stale"):
            with self.subTest(mutation=mutation):
                a,b,images,gpu = fixture()
                if mutation == "resolution": b["owner"]["steps"][2]["cache"]["bake_px"] = 512
                elif mutation == "pixels": images["witness"][4]["optimized_sha256"] = "b"*64
                else: b["owner"]["steps"][5]["cache"]["wanted"] -= 1
                self.assertFalse(compare(a,b,images,gpu)["success"])

    def test_desynchronization_missing_server_route_or_gameplay_change_fails(self):
        for mutation in ("peer", "identity", "move", "reward", "logout", "error"):
            with self.subTest(mutation=mutation):
                a,b,images,gpu = fixture()
                r = b["witness"]
                if mutation == "peer": r["steps"][4]["peer_position"][0] = 8440
                elif mutation == "identity": r["steps"][4]["peer"] = 99
                elif mutation == "move": r["trace"]["moves"].pop()
                elif mutation == "reward": r["steps"][5]["xp_jp"] = [5,0]
                elif mutation == "logout": r["events"]["PLAYER_LOGOUT"] = 0
                else: r["trace"]["errors"] = ["invalid packet"]
                self.assertFalse(compare(a,b,images,gpu)["success"])

    def test_unmeasured_or_increased_gpu_memory_fails(self):
        for mutation in ("missing", "increase"):
            with self.subTest(mutation=mutation):
                a,b,images,gpu = fixture()
                if mutation == "missing": gpu["optimized"].pop()
                else:
                    for row in gpu["optimized"]: row["Total Committed"] = 2000
                self.assertFalse(compare(a,b,images,gpu)["success"])


if __name__ == "__main__": unittest.main()
