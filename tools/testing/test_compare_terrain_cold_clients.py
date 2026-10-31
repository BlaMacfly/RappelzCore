import copy
import unittest
from compare_terrain_cold_clients import compare,CPU_LIMIT,ROLES
from test_compare_terrain_clients import fixture as settled_fixture


def fixture():
    a,b,pictures,memory=settled_fixture();reports={"reference":a,"optimized":b}
    for mode,pair in reports.items():
        enabled=mode=="optimized"
        for role,r in pair.items():
            r.update(scenario="terrain-cold",renderer="optimized",cold=True,motion=True,
                     pixel_cache_enabled=True,async_bake=enabled)
            for s in r["steps"]:
                s["cache"].update(reference=False,wanted=15,resident=15,texture_payload_bytes=15*4194304,
                    cpu_pixel_limit=CPU_LIMIT,cpu_pixel_bytes=CPU_LIMIT,async_enabled=enabled,worker_error=None,
                    job_active=False,submitted=10 if enabled else 0,completed=10 if enabled else 0,cancelled=0,
                    last_restores=0,last_completed_uploads=0,last_submits=0,last_bakes=0,prepare_us=100,
                    worker_us=50000 if enabled else 0)
            samples=[]
            for phase in range(6):
                for frame in range(2):
                    cache=copy.deepcopy(r["steps"][phase]["cache"])
                    if phase in (1,4):
                        if enabled:
                            cache.update(prepare_us=1000,job_active=frame==0,completed=9 if frame==0 else 10,
                                         last_submits=1 if frame==0 else 0,last_completed_uploads=0 if frame==0 else 1)
                        else: cache.update(last_bakes=1,prepare_us=50000)
                    samples.append({"phase":phase,"frame":frame,"cache":cache,"position":[8440 if frame==0 else 10540,7227]})
            r["motion_frames"]=samples
        for row in memory[mode]:row.update({"Private Bytes":200,"Working Set":100})
    return reports,pictures,memory


class TerrainColdComparisonTests(unittest.TestCase):
    def test_background_work_exact_images_and_lower_foreground_cost_pass(self):
        r=compare(*fixture());self.assertTrue(r["success"]);self.assertEqual(len(r["checks"]),50)

    def test_mixed_startup_modes_or_foreground_fallback_is_detected(self):
        for kind in ("foreground","mixed_control","wrong_mode"):
            with self.subTest(kind=kind):
                reports,images,memory=fixture()
                if kind=="foreground":reports["optimized"]["owner"]["motion_frames"][0]["cache"]["last_bakes"]=1
                elif kind=="mixed_control":reports["reference"]["owner"]["steps"][0]["cache"].update(submitted=1,completed=1)
                else:reports["optimized"]["owner"]["steps"][0]["cache"]["async_enabled"]=False
                self.assertFalse(compare(reports,images,memory)["success"])

    def test_worker_failure_queue_accounting_and_resource_limits_are_detected(self):
        for kind in ("failure","queue","cpu","upload"):
            with self.subTest(kind=kind):
                reports,images,memory=fixture();c=reports["optimized"]["witness"]["steps"][0]["cache"]
                if kind=="failure":c["worker_error"]="controlled error"
                elif kind=="queue":c["submitted"]+=2
                elif kind=="cpu":c["cpu_pixel_bytes"]=CPU_LIMIT+1
                else:c["last_restores"]=5
                self.assertFalse(compare(reports,images,memory)["success"])

    def test_blur_missing_cold_entry_desynchronization_and_non_improvement_fail(self):
        for kind in ("pixels","cold","identity","cost","memory"):
            with self.subTest(kind=kind):
                reports,images,memory=fixture()
                if kind=="pixels":images["owner"][4]["optimized_sha256"]="b"*64
                elif kind=="identity":reports["optimized"]["owner"]["steps"][4]["peer"]=99
                elif kind=="memory":memory["optimized"].pop()
                else:
                    for s in reports["optimized"]["owner"]["motion_frames"]:
                        if kind=="cost":s["cache"]["prepare_us"]=100000
                        elif s["phase"]==4:s["cache"].update(last_submits=0,last_completed_uploads=0,job_active=False,completed=10)
                self.assertFalse(compare(reports,images,memory)["success"])


if __name__=="__main__":unittest.main()
