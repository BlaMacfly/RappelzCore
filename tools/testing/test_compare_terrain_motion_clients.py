import copy
import unittest

from compare_terrain_motion_clients import compare, CPU_LIMIT, ROLES
from test_compare_terrain_clients import fixture as settled_fixture


def fixture():
    reference, optimized, pictures, memory = settled_fixture()
    reports = {"reference":reference,"optimized":optimized}
    for mode, pair in reports.items():
        enabled = mode == "optimized"
        for role, r in pair.items():
            r.update(scenario="terrain-motion",renderer="optimized",motion=True,pixel_cache_enabled=enabled)
            for step in r["steps"]:
                step["cache"].update(reference=False,wanted=15,resident=15,texture_payload_bytes=15*4194304,
                    peak_texture_payload_bytes=15*4194304,missing=0,prepare_us=100,last_bakes=0,last_restores=0,
                    cpu_pixel_limit=CPU_LIMIT if enabled else 0,cpu_pixel_bytes=CPU_LIMIT if enabled else 0)
            cold = copy.deepcopy(r["steps"][0]["cache"])
            cold.update(resident=14 if enabled else 5,missing=1 if enabled else 10,
                texture_payload_bytes=(14 if enabled else 5)*4194304,prepare_us=100 if enabled else 1000,
                last_bakes=0 if enabled else 1,last_restores=3 if enabled else 0)
            r["motion_frames"]=[{"phase":2,"frame":i,"view":(i//8)%2,"cache":copy.deepcopy(cold),
                                 "position":[8440,7227]} for i in range(96)]
            r["motion_frames"] += [{"phase":4,"frame":i,"view":(i//8)%2,"cache":copy.deepcopy(cold),
                                    "position":[8440+2100*i/95,7227]} for i in range(96)]
            r["motion_frames"].append({"phase":5,"frame":0,"view":0,"cache":copy.deepcopy(cold),
                                        "position":[8440,7227]})
        for sample in memory[mode]:
            sample.update({"Private Bytes":200,"Working Set":100})
    return reports,pictures,memory


class TerrainMotionComparisonTests(unittest.TestCase):
    def test_same_images_and_gpu_budget_with_faster_pixel_reuse_pass(self):
        r=compare(*fixture())
        self.assertTrue(r["success"])
        self.assertGreater(len(r["checks"]),40)

    def test_cpu_growth_gpu_growth_or_blur_is_detected(self):
        for mutation in ("cpu","gpu","pixels","uploads"):
            with self.subTest(mutation=mutation):
                reports,images,memory=fixture()
                r=reports["optimized"]["owner"]
                if mutation=="cpu": r["motion_frames"][0]["cache"]["cpu_pixel_bytes"]=CPU_LIMIT+1
                elif mutation=="gpu": r["steps"][5]["cache"]["resident"]=16
                elif mutation=="uploads": r["motion_frames"][0]["cache"]["last_restores"]=5
                else: images["owner"][0]["optimized_sha256"]="b"*64
                self.assertFalse(compare(reports,images,memory)["success"])

    def test_missing_motion_desynchronization_or_reblending_fails(self):
        for mutation in ("frames","identity","cold","stationary"):
            with self.subTest(mutation=mutation):
                reports,images,memory=fixture()
                r=reports["optimized"]["witness"]
                if mutation=="frames": r["motion_frames"].pop(0)
                elif mutation=="identity": r["steps"][2]["peer"]=99
                elif mutation=="cold": r["motion_frames"][0]["cache"]["last_bakes"]=1
                else:
                    for s in r["motion_frames"]: s["position"]=[8440,7227]
                self.assertFalse(compare(reports,images,memory)["success"])

    def test_no_measured_improvement_or_missing_memory_samples_fails(self):
        for mutation in ("cost","missing","counters"):
            with self.subTest(mutation=mutation):
                reports,images,memory=fixture()
                if mutation=="counters": memory["optimized"].pop()
                else:
                    for s in reports["optimized"]["owner"]["motion_frames"]:
                        s["cache"]["prepare_us" if mutation=="cost" else "missing"]=10000
                self.assertFalse(compare(reports,images,memory)["success"])


if __name__=="__main__": unittest.main()
