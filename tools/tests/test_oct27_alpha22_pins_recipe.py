"""October 27 intake, the second alpha.22 pin batch: the Artisan's Kitpack fork
v4.81a-dev.85928f9-chriz.4 for the main, NPC and tweak installers.

chriz.4 grants monks quarterstaves after Monk Revisions and restricts plain shamans and
kits that a component adds late (Sacred Fist, Enlightened Fist, Ninja) from the start,
with the EEex-based weapon usability. It reaches new installations only (Christopher,
2026-10-26 and 2026-10-27); there is no tail patch. Its TP2s add no component and keep the
declaration order, so no decision changes.

Static pin, evidence, component, selection and order checks for both recipe profiles only.
Nothing here is a game installation or gameplay acceptance.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import NATIVE_RUN_ORDERS, _effective_features, build_recipe
from tools.curation_audit import Decision, load_catalogs


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261027.toml"
PREVIOUS_EVIDENCE = "manifest/evidence/release-intake-20261026.toml"
PROFILES = ["manifest", "recipes/curated-full-current"]
KITPACK = "artisans-kitpack-4.81a-dev.85928f9-chriz.4"
KITPACK_PREVIOUS = "artisans-kitpack-4.81a-dev.85928f9-chriz.3"
KITPACK_TAG = "v4.81a-dev.85928f9-chriz.4"
FILENAME = f"ArtisansKitpack-Chriz-Balance-Patch-{KITPACK_TAG}.zip"
SHA256 = "8d08fc2be48c45b1d17a79abc010b085868696fb00f045c75d057f9c72a73336"
LENGTH = 89_682_621
TAG_COMMIT = "afa3316cdcf0d9dd8c0678c5e578b2621a3373ee"
# Mod id, its TP2 and the catalog that lists its components.
INSTALLERS = {
    "artisanskitpack": ("ArtisansKitpack/ArtisansKitpack.TP2", "ARTISANSKITPACK"),
    "artisanskitpack-npc": ("ArtisansKitpack_npc/ArtisansKitpack_npc.TP2", "ARTISANSKITPACK_NPC"),
    "artisanskitpack-tweak": ("ArtisansKitpack_tweak/ArtisansKitpack_tweak.TP2", "ARTISANSKITPACK_TWEAK"),
}
EEEX_RUN = "eeex-bg2"
EEEX_CORE = "feature:eeex:mandatory-components"
MAIN_RUN = "artisanskitpack-main-bg2"
NPC_RUN = "artisanskitpack-npc-bg2"
KITPACK_RUNS = (MAIN_RUN, NPC_RUN, "artisanskitpack-tweak-early-bg2", "artisanskitpack-tweak-late-bg2",
                "artisanskitpack-npc-late-bg2")
# The components whose characters chriz.4's fixes concern, with their recorded decisions:
# Monk Revisions, Sacred Fist and Enlightened Fist, Ninja, Warhorn Shaman (a shaman kit) and
# the NPC kits for Minsc and Edwin, which now end with the ability table pass.
CONCERNED = {
    ("ARTISANSKITPACK", 10001): (MAIN_RUN, Decision.DEFAULT),
    ("ARTISANSKITPACK", 10003): (MAIN_RUN, Decision.DEFAULT),
    ("ARTISANSKITPACK", 10004): (MAIN_RUN, Decision.DEFAULT),
    ("ARTISANSKITPACK", 9001): (MAIN_RUN, Decision.DEFAULT),
    ("ARTISANSKITPACK_NPC", 2001): (NPC_RUN, Decision.DEFAULT),
    ("ARTISANSKITPACK_NPC", 5102): (NPC_RUN, Decision.DEFAULT),
}
# Named in chriz.4's notes and still not offered: the Favored Soul, the weapon usability
# update tweak and the dual-class kit name tweak.
NOT_OFFERED = {("artisanskitpack", 30001), ("artisanskitpack-tweak", 1010000), ("artisanskitpack-tweak", 1020000)}


def _load_profile(directory: Path) -> tuple[dict, dict]:
    collection = tomllib.loads((directory / "collection.toml").read_text(encoding="utf-8"))
    preset = tomllib.loads((directory / "presets/chris-recommended.toml").read_text(encoding="utf-8"))["selections"]
    return collection, preset


class Oct27Alpha22KitpackPinTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        cls.output = Path(cls.temp.name) / "recipe"
        # Synthetic evidence marker used only in this temporary test output.
        build_recipe(ROOT, cls.output, "0" * 40)
        cls.directories = [
            ("manifest", ROOT / "manifest"),
            ("recipes/curated-full-current", ROOT / "recipes/curated-full-current"),
            ("recipes/curated-full-current", cls.output),
        ]
        cls.profiles = {
            "curated": (cls.output, *_load_profile(cls.output)),
            "fallback": (ROOT / "manifest", *_load_profile(ROOT / "manifest")),
        }
        cls.evidence = tomllib.loads((ROOT / EVIDENCE).read_text(encoding="utf-8"))
        cls.records = {record["mod_id"]: record for record in cls.evidence["artifacts"]}
        previous = tomllib.loads((ROOT / PREVIOUS_EVIDENCE).read_text(encoding="utf-8"))
        cls.previous = {record["mod_id"]: record for record in previous["artifacts"] if record["mod_id"] in INSTALLERS}

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def _each_profile(self):
        for name, (directory, collection, preset) in self.profiles.items():
            with self.subTest(profile=name):
                yield name, directory, collection, preset

    @staticmethod
    def _selected(collection, preset, **changes):
        state = _effective_features(collection, {**preset, **changes})
        return {
            (ref["run_id"], ref["component"])
            for feature in collection["features"]
            if state[feature["id"]]
            for ref in feature.get("components", [])
        }

    def test_evidence_records_the_fresh_published_download_for_all_three_installers(self):
        self.assertEqual(self.evidence["verified_on"], "2026-10-27")
        self.assertEqual(set(self.records), set(INSTALLERS))
        self.assertEqual(len(self.evidence["artifacts"]), len(INSTALLERS))
        url = (f"https://github.com/Chrizhermann/The-Artisan-s-Kitpack-Chriz-Balance-Patch/releases/download/"
               f"{KITPACK_TAG}/{FILENAME}")
        for mod_id, record in self.records.items():
            with self.subTest(mod=mod_id):
                self.assertEqual(record["artifact_id"], KITPACK)
                self.assertEqual(record["recipe_profiles"], PROFILES)
                self.assertEqual(record["source_reference"], KITPACK_TAG)
                self.assertEqual(record["source_commit"], TAG_COMMIT)
                self.assertEqual(record["immutable_source_url"], url)
                self.assertEqual(record["actual_length"], LENGTH)
                self.assertEqual(record["actual_sha256"], SHA256)
                self.assertRegex(record["tp2_sha256"], r"^[0-9a-f]{64}$")
                for flag in ("fresh_download", "fresh_extraction", "published_release_verified",
                             "native_component_order_verified"):
                    self.assertIs(record[flag], True, flag)
                # Every TP2 changed, but none adds, removes or reorders a component.
                previous = self.previous[mod_id]
                self.assertEqual(previous["artifact_id"], KITPACK_PREVIOUS)
                self.assertNotEqual(record["tp2_sha256"], previous["tp2_sha256"])
                self.assertEqual(record["native_components"], previous["native_components"])
                self.assertEqual(len(record["native_components"]), len(set(record["native_components"])))
                # Only the main record narrows its order check to the curated profile, as before.
                self.assertEqual(record.get("native_order_recipe_profiles", record["recipe_profiles"]),
                                 previous.get("native_order_recipe_profiles", previous["recipe_profiles"]))
        self.assertEqual(self.records["artisanskitpack"]["native_order_recipe_profiles"],
                         ["recipes/curated-full-current"])
        self.assertEqual(len({record["tp2_sha256"] for record in self.records.values()}), 3)

    def test_both_profiles_pin_chriz4_and_keep_chriz3(self):
        for profile_name, directory in self.directories:
            artifact = tomllib.loads((directory / f"artifacts/{KITPACK}.toml").read_text(encoding="utf-8"))
            previous = tomllib.loads((directory / f"artifacts/{KITPACK_PREVIOUS}.toml").read_text(encoding="utf-8"))
            with self.subTest(profile=str(directory)):
                self.assertEqual(artifact["id"], KITPACK)
                self.assertEqual(artifact["version"], KITPACK_TAG.removeprefix("v"))
                self.assertEqual(artifact["acquisition"], "fetch-only")
                self.assertEqual(artifact["source"]["kind"], "github-release")
                self.assertEqual(artifact["source"]["expected_filename"], FILENAME)
                self.assertTrue(artifact["provenance"]["url"].endswith(f"/releases/tag/{KITPACK_TAG}"))
                # Same layout, limits and redirect host as chriz.3, which stays for
                # installations made from the earlier recipe.
                self.assertEqual(artifact["archive"], previous["archive"])
                self.assertEqual(artifact["source"]["redirect_hosts"], previous["source"]["redirect_hosts"])
                self.assertNotEqual(artifact["source"]["sha256"], previous["source"]["sha256"])
                for mod_id, (tp2, _) in INSTALLERS.items():
                    record = self.records[mod_id]
                    self.assertIn(profile_name, record["recipe_profiles"])
                    for key, measured in (("sha256", "actual_sha256"), ("expected_length", "actual_length"),
                                          ("url", "immutable_source_url"), ("reference", "source_reference")):
                        self.assertEqual(artifact["source"][key], record[measured], key)
                    mod = tomllib.loads((directory / f"mods/{mod_id}.toml").read_text(encoding="utf-8"))
                    self.assertEqual(mod["artifact_id"], KITPACK, mod_id)
                    self.assertEqual(mod["tp2"], tp2)
                    self.assertIn(tp2, artifact["archive"]["tp2_paths"])
                # The chriz.3 records still describe the kept chriz.3 artifact.
                for record in self.previous.values():
                    for key, measured in (("sha256", "actual_sha256"), ("expected_length", "actual_length"),
                                          ("url", "immutable_source_url"), ("reference", "source_reference")):
                        self.assertEqual(previous["source"][key], record[measured], key)
        for artifact_id in (KITPACK, KITPACK_PREVIOUS):
            name = f"artifacts/{artifact_id}.toml"
            self.assertEqual((ROOT / "recipes/curated-full-current" / name).read_bytes(),
                             (ROOT / "manifest" / name).read_bytes())

    def test_runs_offer_only_declared_components_in_native_order(self):
        for profile_name, directory in self.directories:
            collection = tomllib.loads((directory / "collection.toml").read_text(encoding="utf-8"))
            for mod_id, record in self.records.items():
                native = record["native_components"]
                mod = tomllib.loads((directory / f"mods/{mod_id}.toml").read_text(encoding="utf-8"))
                self.assertTrue({component["id"] for component in mod["components"]} <= set(native), mod_id)
                runs = [run for run in collection["runs"] if run["mod_id"] == mod_id]
                self.assertTrue(runs, mod_id)
                for run in runs:
                    with self.subTest(profile=str(directory), run=run["run_id"]):
                        self.assertTrue(set(run["components"]) <= set(native))
                        if profile_name in record.get("native_order_recipe_profiles", record["recipe_profiles"]):
                            self.assertEqual(run["components"], [c for c in native if c in run["components"]])
                        for not_offered in NOT_OFFERED:
                            if not_offered[0] == mod_id:
                                self.assertNotIn(not_offered[1], run["components"])
        # The curated generator restores the main run's native order; the authored fallback
        # keeps the order it has had since chriz.1 (see the evidence record).
        native_main = self.records["artisanskitpack"]["native_components"]
        curated_main = NATIVE_RUN_ORDERS[MAIN_RUN]
        self.assertEqual(curated_main, [c for c in native_main if c in curated_main])
        fallback_runs = {run["run_id"]: run for run in self.profiles["fallback"][1]["runs"]}
        self.assertEqual(sorted(fallback_runs[MAIN_RUN]["components"]), sorted(curated_main))

    def test_catalogs_keep_every_declared_component_and_decision(self):
        rows = {}
        for row in load_catalogs(ROOT / "docs/curation/components"):
            rows.setdefault(row.catalog, {})[row.component_id] = row
        for mod_id, (_, catalog) in INSTALLERS.items():
            with self.subTest(catalog=catalog):
                self.assertEqual(set(rows[catalog]), set(self.records[mod_id]["native_components"]))
        for (catalog, component), (_, decision) in CONCERNED.items():
            self.assertEqual(rows[catalog][component].decision, decision, (catalog, component))
        # The catalogs name the new pin.
        for _, catalog in INSTALLERS.values():
            text = (ROOT / f"docs/curation/components/{catalog}.md").read_text(encoding="utf-8")
            self.assertIn(f"**{KITPACK_TAG}**", text, catalog)

    def test_the_fixed_characters_stay_in_recommended_after_eeex(self):
        for name, _, collection, preset in self._each_profile():
            order = [run["run_id"] for run in collection["runs"]]
            features = {feature["id"]: feature for feature in collection["features"]}
            baseline = self._selected(collection, preset)
            # The fixes act only with the EEex-based weapon usability, which Kitpack sets up
            # when EEex is already installed: EEex installs before every Kitpack run.
            self.assertIn((EEEX_RUN, 0), baseline)
            for run_id in KITPACK_RUNS:
                self.assertLess(order.index(EEEX_RUN), order.index(run_id), run_id)
            for (catalog, component), (run_id, _) in CONCERNED.items():
                with self.subTest(catalog=catalog, component=component):
                    mod_id = {catalog_name: mod for mod, (_, catalog_name) in INSTALLERS.items()}[catalog]
                    feature = features[f"feature:{mod_id}:component-{component}"]
                    self.assertEqual(feature["decision"], "default")
                    self.assertEqual(feature["components"], [{"run_id": run_id, "component": component}])
                    self.assertIn((run_id, component), baseline)
            # Sacred Fist, Enlightened Fist and Ninja still require EEex.
            for component in (10003, 10004):
                self.assertIn(EEEX_CORE, features[f"feature:artisanskitpack:component-{component}"]["requires"])
            # Monk Revisions comes before the monk kits that need it in the main run.
            main = {run["run_id"]: run for run in collection["runs"]}[MAIN_RUN]["components"]
            self.assertLess(main.index(10001), main.index(10003))
            self.assertLess(main.index(10001), main.index(10004))
            for mod_id, component in NOT_OFFERED:
                self.assertNotIn(f"feature:{mod_id}:component-{component}", features)


if __name__ == "__main__":
    unittest.main()
