"""October 26 intake, the alpha.22 pin batch: BG Rebalance v0.8.5 with Chris Revisions 340,
the Spell Revisions fork v4.21-chriz.7 with its SR/RR compatibility package, BuffBot v1.11.0,
the Artisan's Kitpack fork v4.81a-dev.85928f9-chriz.3 and EEex Attack Info 9.

Static pin, evidence, component, selection and order checks for both recipe profiles only.
Nothing here is a game installation or gameplay acceptance.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import NATIVE_RUN_ORDERS, _effective_features, build_recipe
from tools.curation_audit import Decision, RowKey, load_catalogs


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261026.toml"
EEEX_EVIDENCE = "manifest/evidence/release-intake-20261018.toml"
PROFILES = ["manifest", "recipes/curated-full-current"]
# chriz.3 replaced chriz.2 in this batch before it merged; chriz.2 was never in a recipe.
KITPACK = "artisans-kitpack-4.81a-dev.85928f9-chriz.3"
KITPACK_PREVIOUS = "artisans-kitpack-4.81a-dev.85928f9-chriz.1"
KITPACK_TAG = "v4.81a-dev.85928f9-chriz.3"
# Per mod: the new pin, the superseded pin (kept as history), its tag, the archive name and
# the catalog that lists the mod's components.
PINS = {
    "chriz-bg-rebalance": ("chriz-bg-rebalance-0.8.5", "chriz-bg-rebalance-0.8.3", "v0.8.5",
                           "chriz-bg-rebalance-v0.8.5-windows.zip", "CHRIZ-BG-REBALANCE"),
    "spell-rev": ("spell-revisions-4.21-chriz.7", "spell-revisions-4.21-chriz.5", "v4.21-chriz.7",
                  "spell_rev-v4-21-chriz-7.zip", "SPELL_REV"),
    "srcb-rr-compat": ("srcb-rr-compat-4.21-chriz.7", "srcb-rr-compat-4.21-chriz.5", "v4.21-chriz.7",
                       "SRCB_RR_COMPAT-v4-21-chriz-7.zip", "SRCB_RR_COMPAT"),
    "buffbot": ("buffbot-1.11.0", "buffbot-1.10.1", "v1.11.0", "buffbot-v1.11.0.zip", "BUFFBOT"),
    "artisanskitpack": (KITPACK, KITPACK_PREVIOUS, KITPACK_TAG,
                        f"ArtisansKitpack-Chriz-Balance-Patch-{KITPACK_TAG}.zip", "ARTISANSKITPACK"),
    "artisanskitpack-npc": (KITPACK, KITPACK_PREVIOUS, KITPACK_TAG,
                            f"ArtisansKitpack-Chriz-Balance-Patch-{KITPACK_TAG}.zip", "ARTISANSKITPACK_NPC"),
    "artisanskitpack-tweak": (KITPACK, KITPACK_PREVIOUS, KITPACK_TAG,
                              f"ArtisansKitpack-Chriz-Balance-Patch-{KITPACK_TAG}.zip", "ARTISANSKITPACK_TWEAK"),
}
REBALANCE_RUN = "chriz-bg-rebalance-bg2"
EEEX_RUN = "eeex-bg2"
EEEX_CORE = "feature:eeex:mandatory-components"
REVISIONS = "feature:chriz-bg-rebalance:component-340"
ATTACK_INFO = "feature:eeex:component-9"
ARTISAN_MAIN_RUN = "artisanskitpack-main-bg2"


def _load_profile(directory: Path) -> tuple[dict, dict]:
    collection = tomllib.loads((directory / "collection.toml").read_text(encoding="utf-8"))
    preset = tomllib.loads((directory / "presets/chris-recommended.toml").read_text(encoding="utf-8"))["selections"]
    return collection, preset


class Oct26Alpha22PinBatchTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        cls.output = Path(cls.temp.name) / "recipe"
        # Synthetic evidence marker used only in this temporary test output.
        build_recipe(ROOT, cls.output, "0" * 40)
        # Each recipe directory with the evidence profile it must match.
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

    def test_evidence_records_each_fresh_published_download(self):
        self.assertEqual(self.evidence["verified_on"], "2026-10-26")
        self.assertEqual(len(self.evidence["artifacts"]), len(PINS))
        self.assertEqual(set(self.records), set(PINS))
        for mod_id, (artifact_id, _, reference, filename, _) in PINS.items():
            record = self.records[mod_id]
            with self.subTest(mod=mod_id):
                self.assertEqual(record["artifact_id"], artifact_id)
                self.assertEqual(record["recipe_profiles"], PROFILES)
                self.assertEqual(record["source_reference"], reference)
                self.assertTrue(record["immutable_source_url"].startswith("https://github.com/Chrizhermann/"))
                self.assertTrue(record["immutable_source_url"].endswith(f"/releases/download/{reference}/{filename}"))
                for flag in ("fresh_download", "fresh_extraction", "published_release_verified",
                             "native_component_order_verified"):
                    self.assertIs(record[flag], True, flag)
                for digest in ("actual_sha256", "tp2_sha256"):
                    self.assertRegex(record[digest], r"^[0-9a-f]{64}$")
                self.assertRegex(record["source_commit"], r"^[0-9a-f]{40}$")
                self.assertEqual(len(record["native_components"]), len(set(record["native_components"])))
        # One release carries Spell Revisions and the SR/RR package; one archive carries
        # the three Kitpack installers, each with its own TP2.
        self.assertEqual(self.records["spell-rev"]["source_commit"], self.records["srcb-rr-compat"]["source_commit"])
        kitpack = [self.records[mod_id] for mod_id in ("artisanskitpack", "artisanskitpack-npc", "artisanskitpack-tweak")]
        for key in ("immutable_source_url", "actual_length", "actual_sha256", "source_commit"):
            self.assertEqual(len({record[key] for record in kitpack}), 1, key)
        self.assertEqual(len({record["tp2_sha256"] for record in kitpack}), 3)
        # Only the Kitpack main record narrows its order check to the curated profile.
        for mod_id, record in self.records.items():
            with self.subTest(mod=mod_id):
                expected = ["recipes/curated-full-current"] if mod_id == "artisanskitpack" else PROFILES
                self.assertEqual(record.get("native_order_recipe_profiles", record["recipe_profiles"]), expected)

    def test_both_profiles_pin_the_verified_archives_and_keep_the_previous_pins(self):
        for profile_name, directory in self.directories:
            for mod_id, (artifact_id, previous_id, reference, filename, _) in PINS.items():
                record = self.records[mod_id]
                with self.subTest(profile=str(directory), mod=mod_id):
                    self.assertIn(profile_name, record["recipe_profiles"])
                    mod = tomllib.loads((directory / f"mods/{mod_id}.toml").read_text(encoding="utf-8"))
                    self.assertEqual(mod["artifact_id"], artifact_id)
                    artifact = tomllib.loads((directory / f"artifacts/{artifact_id}.toml").read_text(encoding="utf-8"))
                    self.assertEqual(artifact["acquisition"], "fetch-only")
                    self.assertEqual(artifact["source"]["kind"], "github-release")
                    self.assertEqual(artifact["source"]["expected_filename"], filename)
                    self.assertEqual(artifact["version"], reference.removeprefix("v"))
                    for key, measured in (("sha256", "actual_sha256"), ("expected_length", "actual_length"),
                                          ("url", "immutable_source_url"), ("reference", "source_reference")):
                        self.assertEqual(artifact["source"][key], record[measured])
                    # Same layout, limits and redirect host as the previous pin, which stays
                    # available for installations made from earlier recipes.
                    previous = tomllib.loads((directory / f"artifacts/{previous_id}.toml").read_text(encoding="utf-8"))
                    self.assertEqual(artifact["archive"], previous["archive"])
                    self.assertEqual(artifact["source"]["redirect_hosts"], previous["source"]["redirect_hosts"])
                    self.assertEqual(artifact["source"]["kind"], previous["source"]["kind"])
                    self.assertNotEqual(artifact["source"]["sha256"], previous["source"]["sha256"])
                    self.assertTrue(set(artifact["archive"]["tp2_paths"]) >= {mod["tp2"]})
        # The checked-in curated artifacts are the generator's copies of the authored pins.
        for artifact_id in {pin[0] for pin in PINS.values()}:
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
        # The curated generator restores the Kitpack main run's native order; the authored
        # fallback keeps the order it has had since chriz.1 (see the evidence record).
        native_main = self.records["artisanskitpack"]["native_components"]
        curated_main = NATIVE_RUN_ORDERS[ARTISAN_MAIN_RUN]
        self.assertEqual(curated_main, [c for c in native_main if c in curated_main])
        fallback_runs = {run["run_id"]: run for run in self.profiles["fallback"][1]["runs"]}
        self.assertEqual(sorted(fallback_runs[ARTISAN_MAIN_RUN]["components"]), sorted(curated_main))
        self.assertEqual(NATIVE_RUN_ORDERS[REBALANCE_RUN],
                         [c for c in self.records["chriz-bg-rebalance"]["native_components"]
                          if c in NATIVE_RUN_ORDERS[REBALANCE_RUN]])

    def test_catalogs_list_every_declared_component_with_the_recorded_decisions(self):
        rows = {}
        for row in load_catalogs(ROOT / "docs/curation/components"):
            rows.setdefault(row.catalog, {})[row.component_id] = row
        for mod_id, (*_, catalog) in PINS.items():
            with self.subTest(catalog=catalog):
                self.assertEqual(set(rows[catalog]), set(self.records[mod_id]["native_components"]))
        # The only new component in the batch, and the only changed decision.
        self.assertEqual(rows["CHRIZ-BG-REBALANCE"][340].decision, Decision.DEFAULT)
        self.assertEqual(rows["EEEX"][9].decision, Decision.OPTIONAL)
        self.assertEqual(rows["BUFFBOT"][1].decision, Decision.MANDATORY)
        self.assertEqual(rows["BUFFBOT"][0].decision, Decision.MANDATORY)
        self.assertEqual(rows["SRCB_RR_COMPAT"][0].decision, Decision.MANDATORY)
        keys = {RowKey(catalog, component) for catalog, components in rows.items() for component in components}
        self.assertIn(RowKey("CHRIZ-BG-REBALANCE", 340), keys)
        self.assertIn(RowKey("EEEX", 9), keys)

    def test_chris_revisions_340_is_a_default_at_the_end_of_the_rebalance_run(self):
        for name, _, collection, preset in self._each_profile():
            features = {feature["id"]: feature for feature in collection["features"]}
            feature = features[REVISIONS]
            self.assertEqual((feature["category"], feature["decision"], feature["readiness"]),
                             ("spells", "default", "ready"))
            self.assertEqual(feature["parent"], "mod:chriz-bg-rebalance")
            self.assertEqual(feature.get("requires", []), [])
            self.assertEqual(feature.get("conflicts", []), [])
            self.assertNotIn("choice_group", feature)
            self.assertEqual(feature["components"], [{"run_id": REBALANCE_RUN, "component": 340}])
            for spell in ("Aid", "Luck", "Bless", "Chaotic Commands", "Prayer", "Recitation"):
                self.assertIn(spell, feature["title"])
            self.assertEqual(preset.get(REVISIONS), "on")
            runs = {run["run_id"]: run for run in collection["runs"]}
            self.assertEqual(runs[REBALANCE_RUN]["components"][-4:], [330, 331, 332, 340])
            self.assertEqual(runs[REBALANCE_RUN]["phase"], "post-eet-end")
            baseline = self._selected(collection, preset)
            self.assertIn((REBALANCE_RUN, 340), baseline)
            self.assertEqual(baseline - self._selected(collection, preset, **{REVISIONS: "off"}),
                             {(REBALANCE_RUN, 340)})
            self.assertNotIn((REBALANCE_RUN, 340),
                             self._selected(collection, preset, **{"mod:chriz-bg-rebalance": "off"}))
            # It needs no EEex.
            self.assertIn((REBALANCE_RUN, 340), self._selected(collection, preset, **{"mod:eeex": "off"}))
            # It changes the final spells of SCS, Spell Revisions, Bardic Wonders and
            # IWDification, and runs before SR's NPC spellbook pass and the scroll policy.
            order = [run["run_id"] for run in collection["runs"]]
            rebalance = order.index(REBALANCE_RUN)
            earlier = ["spell-rev-core-bg2", "stratagems-bg2", "eet-end-bg2"]
            if name == "curated":
                earlier += ["bardicwonders-bg2", "iwdification-bg2"]
            for run_id in earlier:
                self.assertLess(order.index(run_id), rebalance, run_id)
            for run_id in ("spell-rev-npc-spellbooks-bg2", "chriz-bg-modpack-scrolls-bg2"):
                self.assertLess(rebalance, order.index(run_id), run_id)
            group = next(g for g in collection["feature_groups"] if g["id"] == "challenge-mode")
            self.assertNotIn(REVISIONS, group["members"])

    def test_eeex_attack_info_9_is_offered_unchecked_in_native_order(self):
        (eeex_record,) = tomllib.loads((ROOT / EEEX_EVIDENCE).read_text(encoding="utf-8"))["artifacts"]
        native = eeex_record["native_components"]
        self.assertEqual(native, [0, 1, 9, 2, 3, 4, 5, 6, 7, 8])
        for name, directory, collection, preset in self._each_profile():
            mod = tomllib.loads((directory / "mods/eeex.toml").read_text(encoding="utf-8"))
            self.assertEqual(mod["artifact_id"], eeex_record["artifact_id"])
            self.assertIn(9, {component["id"] for component in mod["components"]})
            features = {feature["id"]: feature for feature in collection["features"]}
            feature = features[ATTACK_INFO]
            self.assertEqual((feature["category"], feature["decision"], feature["readiness"]),
                             ("engine", "optional", "ready"))
            self.assertEqual(feature["parent"], "mod:eeex")
            self.assertEqual(feature["requires"], [EEEX_CORE])
            self.assertEqual(feature["components"], [{"run_id": EEEX_RUN, "component": 9}])
            self.assertIn("restart", feature["description"])
            self.assertNotEqual(preset.get(ATTACK_INFO), "on")
            runs = {run["run_id"]: run for run in collection["runs"]}
            self.assertEqual(runs[EEEX_RUN]["components"], native)
            baseline = self._selected(collection, preset)
            self.assertNotIn((EEEX_RUN, 9), baseline)
            self.assertEqual({component for run, component in baseline if run == EEEX_RUN}, set(range(9)))
            ticked = self._selected(collection, preset, **{ATTACK_INFO: "on"})
            self.assertEqual(ticked - baseline, {(EEEX_RUN, 9)})
            self.assertEqual(baseline - ticked, set())
            self.assertFalse(any(run == EEEX_RUN for run, _ in self._selected(
                collection, preset, **{ATTACK_INFO: "on", "mod:eeex": "off"})))
            group = next(g for g in collection["feature_groups"] if g["id"] == "challenge-mode")
            self.assertNotIn(ATTACK_INFO, group["members"])

    def test_superseded_records_still_match_their_kept_artifacts(self):
        superseded = {
            "manifest/evidence/release-intake-20261014.toml": "chriz-bg-rebalance",
            "manifest/evidence/release-intake-20261023-buffbot-eeexremote.toml": "buffbot",
        }
        for path, mod_id in superseded.items():
            evidence = tomllib.loads((ROOT / path).read_text(encoding="utf-8"))
            (record,) = [record for record in evidence["artifacts"] if record["mod_id"] == mod_id]
            self.assertEqual(record["artifact_id"], PINS[mod_id][1])
            for _, directory in self.directories:
                with self.subTest(evidence=path, profile=str(directory)):
                    artifact = tomllib.loads(
                        (directory / f"artifacts/{record['artifact_id']}.toml").read_text(encoding="utf-8"))
                    for key, measured in (("sha256", "actual_sha256"), ("expected_length", "actual_length"),
                                          ("url", "immutable_source_url"), ("reference", "source_reference")):
                        self.assertEqual(artifact["source"][key], record[measured])


if __name__ == "__main__":
    unittest.main()
