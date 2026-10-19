"""October 18 intake: EEex v1.3.1 replaces v1.2.0 in both recipe profiles.

Static pin, evidence, order and selection checks only. The disposable-lab install and
launch smoke tests are recorded outside this repository; nothing here is a game
installation or gameplay acceptance.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import _effective_features, build_recipe
from tools.curation_audit import Decision, RowKey, load_catalogs


ROOT = Path(__file__).resolve().parents[2]
EEEX_RUN = "eeex-bg2"
EEEX_CORE = "feature:eeex:mandatory-components"
PINNED = "eeex-1.3.1"


def _load_profile(directory: Path) -> tuple[dict, dict]:
    collection = tomllib.loads((directory / "collection.toml").read_text(encoding="utf-8"))
    preset = tomllib.loads((directory / "presets/chris-recommended.toml").read_text(encoding="utf-8"))["selections"]
    return collection, preset


class Oct18EeexIntakeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        cls.output = Path(cls.temp.name) / "recipe"
        # Synthetic evidence marker used only in this temporary test output.
        build_recipe(ROOT, cls.output, "0" * 40)
        cls.profiles = {
            "generated": cls.output,
            "curated": ROOT / "recipes/curated-full-current",
            "fallback": ROOT / "manifest",
        }
        evidence = tomllib.loads(
            (ROOT / "manifest/evidence/release-intake-20261018.toml").read_text(encoding="utf-8"))
        (cls.record,) = evidence["artifacts"]

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def test_both_profiles_pin_the_verified_release(self):
        self.assertEqual((self.record["mod_id"], self.record["artifact_id"]), ("eeex", PINNED))
        self.assertTrue(all(self.record[flag] for flag in (
            "fresh_download", "fresh_extraction", "published_release_verified",
            "native_component_order_verified")))
        for name, directory in self.profiles.items():
            with self.subTest(profile=name):
                mod = tomllib.loads((directory / "mods/eeex.toml").read_text(encoding="utf-8"))
                self.assertEqual(mod["artifact_id"], PINNED)
                artifact = tomllib.loads((directory / f"artifacts/{PINNED}.toml").read_text(encoding="utf-8"))
                source = artifact["source"]
                self.assertEqual(source["url"], self.record["immutable_source_url"])
                self.assertEqual(source["reference"], self.record["source_reference"])
                self.assertEqual(source["expected_length"], self.record["actual_length"])
                self.assertEqual(source["sha256"], self.record["actual_sha256"])
                # v1.2.0 stays available for installations made from earlier recipes.
                self.assertTrue((directory / "artifacts/eeex-1.2.0.toml").is_file())

    def test_runs_keep_the_native_order_and_leave_attack_info_out(self):
        native = self.record["native_components"]
        # Attack Info 9 is declared right after the main component.
        self.assertEqual(native, [0, 1, 9, 2, 3, 4, 5, 6, 7, 8])
        for name, directory in self.profiles.items():
            with self.subTest(profile=name):
                collection, preset = _load_profile(directory)
                (run,) = [run for run in collection["runs"] if run["mod_id"] == "eeex"]
                self.assertEqual(run["run_id"], EEEX_RUN)
                self.assertEqual(run["components"], [c for c in native if c in run["components"]])
                self.assertEqual(run["components"], [0, 1, 2, 3, 4, 5, 6, 7, 8])
                features = {feature["id"]: feature for feature in collection["features"]}
                self.assertEqual(
                    {ref["component"] for ref in features[EEEX_CORE]["components"]}, set(range(9)))
                offered = {
                    ref["component"]
                    for feature in collection["features"]
                    for ref in feature.get("components", [])
                    if ref["run_id"] == EEEX_RUN
                }
                self.assertNotIn(9, offered)
                state = _effective_features(collection, preset)
                self.assertTrue(state[EEEX_CORE])

    def test_catalog_lists_attack_info_without_a_decision(self):
        rows = {RowKey(row.catalog, row.component_id): row for row in load_catalogs(ROOT / "docs/curation/components")}
        self.assertEqual(rows[RowKey("EEEX", 9)].decision, Decision.EXCLUDED)
        self.assertFalse(rows[RowKey("EEEX", 9)].installed)
        for component in range(9):
            self.assertEqual(rows[RowKey("EEEX", component)].decision, Decision.MANDATORY)


if __name__ == "__main__":
    unittest.main()
