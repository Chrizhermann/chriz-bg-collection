"""October 27 intake, the second alpha.22 pin batch: the Artisan's Kitpack fork
v4.81a-dev.85928f9-chriz.4 for the main, NPC and tweak installers, and BG Rebalance v0.8.6
with the Time Stop and Haste effect clocks 360 (downloaded and verified on 2026-10-28).

chriz.4 grants monks quarterstaves after Monk Revisions and restricts plain shamans and
kits that a component adds late (Sacred Fist, Enlightened Fist, Ninja) from the start,
with the EEex-based weapon usability. It reaches new installations only (Christopher,
2026-10-26 and 2026-10-27); there is no tail patch. Its TP2s add no component and keep the
declaration order, so no decision changes.

Static pin, evidence, component, selection and order checks for both recipe profiles only.
Nothing here is a game installation or gameplay acceptance. BG Rebalance v0.8.7 (October 29,
test_oct29_rebalance_pin_recipe) supersedes the v0.8.6 pin, renames 360 to True Time Stop –
fair Haste & Slow and keeps its components and order; the v0.8.6 record and artifacts stay
as verified here.
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
# Named in chriz.4's notes and still not offered: the Favored Soul and the dual-class kit
# name tweak. The weapon usability update tweak 1010000 is a default since the October 29
# intake (Christopher, 2026-10-26; test_oct29_kitpack_weapon_sweep_recipe.py).
NOT_OFFERED = {("artisanskitpack", 30001), ("artisanskitpack-tweak", 1020000)}


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
        cls.all_records = {record["mod_id"]: record for record in cls.evidence["artifacts"]}
        # The Kitpack records; the Rebalance record of the same batch has its own tests below.
        cls.records = {mod_id: record for mod_id, record in cls.all_records.items() if mod_id in INSTALLERS}
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
        self.assertEqual(set(self.all_records), set(INSTALLERS) | {"chriz-bg-rebalance"})
        self.assertEqual(len(self.evidence["artifacts"]), len(INSTALLERS) + 1)
        for record in self.records.values():
            self.assertNotIn("verified_on", record)
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


REBALANCE = "chriz-bg-rebalance"
REBALANCE_PIN = "chriz-bg-rebalance-0.8.6"
REBALANCE_PREVIOUS = "chriz-bg-rebalance-0.8.5"
REBALANCE_TAG = "v0.8.6"
REBALANCE_FILENAME = "chriz-bg-rebalance-v0.8.6-windows.zip"
REBALANCE_SHA256 = "24e3117e9a32485fa848f37c213fc4c4c24bb0166a04c229a1b3fe7a28ede3ba"
REBALANCE_LENGTH = 1_817_306
REBALANCE_COMMIT = "43b8811abe04ad71575c89d5589866919801bdff"
# The October 29 intake replaces v0.8.6 with v0.8.7, which renames 360.
REBALANCE_SUCCESSOR = "chriz-bg-rebalance-0.8.7"
CLOCKS_TITLE = "True Time Stop – fair Haste & Slow"
REBALANCE_RUN = "chriz-bg-rebalance-bg2"
CLOCKS = "feature:chriz-bg-rebalance:component-360"
# CEBG's only accepted BG2:EE executable, whose PE timestamp 360's installer requires.
ACCEPTED_BALDUR_EXE = "b51093a49140b2b8a7c046b4652bb8e535be24ebbc12b1d735e0b94217a14d57"


class Oct28RebalanceV086PinTests(unittest.TestCase):
    """BG Rebalance v0.8.6: the release adds only the Time Stop and Haste effect clocks 360,
    a default after 340 (Christopher, 2026-10-27)."""

    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        cls.output = Path(cls.temp.name) / "recipe"
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
        evidence = tomllib.loads((ROOT / EVIDENCE).read_text(encoding="utf-8"))
        (cls.record,) = [record for record in evidence["artifacts"] if record["mod_id"] == REBALANCE]
        previous = tomllib.loads((ROOT / PREVIOUS_EVIDENCE).read_text(encoding="utf-8"))
        (cls.previous,) = [record for record in previous["artifacts"] if record["mod_id"] == REBALANCE]

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def _each_profile(self):
        for name, (directory, collection, preset) in self.profiles.items():
            with self.subTest(profile=name):
                yield name, directory, collection, preset

    _selected = staticmethod(Oct27Alpha22KitpackPinTests._selected)

    def test_evidence_records_the_fresh_download_of_october_28(self):
        record = self.record
        self.assertEqual(record["verified_on"], "2026-10-28")
        self.assertEqual(record["artifact_id"], REBALANCE_PIN)
        self.assertEqual(record["recipe_profiles"], PROFILES)
        self.assertNotIn("native_order_recipe_profiles", record)
        self.assertEqual(record["source_reference"], REBALANCE_TAG)
        self.assertEqual(record["source_commit"], REBALANCE_COMMIT)
        self.assertEqual(record["immutable_source_url"],
                         f"https://github.com/Chrizhermann/chriz-bg-rebalance/releases/download/"
                         f"{REBALANCE_TAG}/{REBALANCE_FILENAME}")
        self.assertEqual(record["actual_length"], REBALANCE_LENGTH)
        self.assertEqual(record["actual_sha256"], REBALANCE_SHA256)
        self.assertRegex(record["tp2_sha256"], r"^[0-9a-f]{64}$")
        for flag in ("fresh_download", "fresh_extraction", "published_release_verified",
                     "native_component_order_verified"):
            self.assertIs(record[flag], True, flag)
        # v0.8.6 declares 360 after 340; everything before it is v0.8.5's order.
        self.assertEqual(self.previous["artifact_id"], REBALANCE_PREVIOUS)
        self.assertNotEqual(record["tp2_sha256"], self.previous["tp2_sha256"])
        self.assertEqual(record["native_components"], self.previous["native_components"] + [360])

    def test_both_profiles_keep_v086_and_v085_and_leave_the_tail_packages_out(self):
        for profile_name, directory in self.directories:
            with self.subTest(profile=str(directory)):
                mod = tomllib.loads((directory / f"mods/{REBALANCE}.toml").read_text(encoding="utf-8"))
                self.assertEqual(mod["artifact_id"], REBALANCE_SUCCESSOR)
                self.assertIn(360, {component["id"] for component in mod["components"]})
                artifact = tomllib.loads((directory / f"artifacts/{REBALANCE_PIN}.toml").read_text(encoding="utf-8"))
                previous = tomllib.loads((directory / f"artifacts/{REBALANCE_PREVIOUS}.toml").read_text(encoding="utf-8"))
                self.assertEqual(artifact["version"], REBALANCE_TAG.removeprefix("v"))
                self.assertEqual(artifact["acquisition"], "fetch-only")
                self.assertEqual(artifact["source"]["kind"], "github-release")
                self.assertEqual(artifact["source"]["expected_filename"], REBALANCE_FILENAME)
                for key, measured in (("sha256", "actual_sha256"), ("expected_length", "actual_length"),
                                      ("url", "immutable_source_url"), ("reference", "source_reference")):
                    self.assertEqual(artifact["source"][key], self.record[measured], key)
                    self.assertEqual(previous["source"][key], self.previous[measured], key)
                self.assertEqual(artifact["archive"], previous["archive"])
                self.assertEqual(artifact["source"]["redirect_hosts"], previous["source"]["redirect_hosts"])
                # The release's standalone tails for existing games are not part of the recipe.
                names = {path.name.casefold() for path in (directory / "artifacts").glob("*.toml")}
                self.assertFalse(any(name.startswith("cbr_sr_") or name.startswith("cbr-sr-") for name in names))
                self.assertIn(profile_name, self.record["recipe_profiles"])
        for artifact_id in (REBALANCE_PIN, REBALANCE_PREVIOUS):
            name = f"artifacts/{artifact_id}.toml"
            self.assertEqual((ROOT / "recipes/curated-full-current" / name).read_bytes(),
                             (ROOT / "manifest" / name).read_bytes())

    def test_clocks_360_is_a_default_after_340_and_needs_eeex_first(self):
        native = self.record["native_components"]
        self.assertEqual([c for c in NATIVE_RUN_ORDERS[REBALANCE_RUN] if c in native], NATIVE_RUN_ORDERS[REBALANCE_RUN])
        self.assertEqual(NATIVE_RUN_ORDERS[REBALANCE_RUN][-2:], [340, 360])
        for name, _, collection, preset in self._each_profile():
            features = {feature["id"]: feature for feature in collection["features"]}
            feature = features[CLOCKS]
            self.assertEqual((feature["category"], feature["decision"], feature["readiness"]),
                             ("spells", "default", "ready"))
            self.assertEqual(feature["parent"], "mod:chriz-bg-rebalance")
            # The TP2 refuses 360 unless EEex is already installed.
            self.assertEqual(feature["requires"], [EEEX_CORE])
            self.assertEqual(feature.get("conflicts", []), [])
            self.assertNotIn("choice_group", feature)
            self.assertEqual(feature["components"], [{"run_id": REBALANCE_RUN, "component": 360}])
            # Released as "Time Stop and Haste effect clocks"; v0.8.7 renamed it.
            self.assertEqual(feature["title"], CLOCKS_TITLE)
            for phrase in ("Time Stop", "everything but the caster stops", "Haste and Slow",
                           "poison", "regeneration", "before a playthrough", "not supported",
                           "not covered yet"):
                self.assertIn(phrase, feature["description"])
            self.assertEqual(preset.get(CLOCKS), "on")
            runs = {run["run_id"]: run for run in collection["runs"]}
            self.assertEqual(runs[REBALANCE_RUN]["components"][-2:], [340, 360])
            self.assertEqual(runs[REBALANCE_RUN]["components"],
                             [c for c in native if c in runs[REBALANCE_RUN]["components"]])
            self.assertEqual(runs[REBALANCE_RUN]["phase"], "post-eet-end")
            order = [run["run_id"] for run in collection["runs"]]
            self.assertLess(order.index(EEEX_RUN), order.index(REBALANCE_RUN))
            baseline = self._selected(collection, preset)
            self.assertIn((REBALANCE_RUN, 360), baseline)
            self.assertIn((EEEX_RUN, 0), baseline)
            self.assertEqual(baseline - self._selected(collection, preset, **{CLOCKS: "off"}),
                             {(REBALANCE_RUN, 360)})
            self.assertNotIn((REBALANCE_RUN, 360), self._selected(collection, preset, **{"mod:eeex": "off"}))
            self.assertNotIn((REBALANCE_RUN, 360),
                             self._selected(collection, preset, **{"mod:chriz-bg-rebalance": "off"}))
            group = next(g for g in collection["feature_groups"] if g["id"] == "challenge-mode")
            self.assertNotIn(CLOCKS, group["members"])

    def test_catalog_records_the_decision_and_the_executable_check(self):
        rows = {row.component_id: row for row in load_catalogs(ROOT / "docs/curation/components")
                if row.catalog == "CHRIZ-BG-REBALANCE"}
        self.assertEqual(set(rows), set(self.record["native_components"]))
        self.assertEqual(rows[360].decision, Decision.DEFAULT)
        self.assertEqual(rows[340].decision, Decision.DEFAULT)
        text = (ROOT / "docs/curation/components/CHRIZ-BG-REBALANCE.md").read_text(encoding="utf-8")
        self.assertIn(f"{REBALANCE_TAG} (2026-10-27, tag commit `{REBALANCE_COMMIT[:7]}`) added `360`", text)
        for phrase in ("Christopher's decision, 2026-10-27", "`data/PATCH27.BIF`", "`0x6a18d73d`", "`0x8664`",
                       "`b51093a4…`", "adding or removing it during one is\n  not supported"):
            self.assertIn(phrase, text)
        # The executable the catalog names is CEBG's only accepted BG2:EE 2.7.3.0 build.
        for directory in (ROOT / "manifest", ROOT / "recipes/curated-full-current"):
            build = tomllib.loads((directory / "game-builds/steam-bg2ee-2.7.3-en-us.toml").read_text(encoding="utf-8"))
            self.assertEqual(build["product_version"], "2.7.3.0")
            self.assertEqual(build["executable"], "Baldur.exe")
            executables = {fingerprint["sha256"] for variant in build["allowed_clean_variants"]
                           for fingerprint in variant["fingerprints"] if fingerprint["path"] == "Baldur.exe"}
            self.assertEqual(executables, {ACCEPTED_BALDUR_EXE})


if __name__ == "__main__":
    unittest.main()
