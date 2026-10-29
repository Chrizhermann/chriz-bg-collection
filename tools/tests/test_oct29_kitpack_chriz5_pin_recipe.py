"""October 29 intake: the Artisan's Kitpack fork v4.81a-dev.85928f9-chriz.5 replaces chriz.4
for the main, NPC and tweak installers in both recipe profiles.

chriz.5 lets the Invisible Blade and Bardic Wonders' Dancer, who borrow the Shapeshifter's
item usability bit to keep out of armor, use the katanas and the rogue and bard items that
bit closed to them; every other kit with the bit, the Shapeshifter and the Elementalist
included, stays blocked through an exact kit check. The Elementalist 5001 and the Invisible
Blade 7005 run the pass after adding their kit, and the new tweak 7205, declared between 7203
and 8204, runs it for items and kits that later mods add. "Update EEex-based Weapon Usability
Changes" 1010000 now ends with the same pass. Christopher took chriz.5 and 7205 into alpha.22
on 2026-10-29, 7205 as a default. The release changes new installations only.

Static pin, evidence, component, selection and order checks for both recipe profiles only.
Nothing here is a game installation or gameplay acceptance.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import NATIVE_RUN_ORDERS, _effective_features, build_recipe
from tools.curation_audit import Decision, RowKey, load_catalogs, load_curation_map


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261029-kitpack-chriz5.toml"
PREVIOUS_EVIDENCE = "manifest/evidence/release-intake-20261027.toml"
PROFILES = ["manifest", "recipes/curated-full-current"]
PIN = "artisans-kitpack-4.81a-dev.85928f9-chriz.5"
PREVIOUS = "artisans-kitpack-4.81a-dev.85928f9-chriz.4"
TAG = "v4.81a-dev.85928f9-chriz.5"
FILENAME = f"ArtisansKitpack-Chriz-Balance-Patch-{TAG}.zip"
SHA256 = "b5527feef6d641a61f90c8faddaa98d1b81cff43a0141cd4748d9c099a5412a6"
LENGTH = 89_698_447
TAG_COMMIT = "10a4a2987c8a2898534022d1ce138ac5fd395ca4"
# Mod id, its TP2 and the catalog that lists its components.
INSTALLERS = {
    "artisanskitpack": ("ArtisansKitpack/ArtisansKitpack.TP2", "ARTISANSKITPACK"),
    "artisanskitpack-npc": ("ArtisansKitpack_npc/ArtisansKitpack_npc.TP2", "ARTISANSKITPACK_NPC"),
    "artisanskitpack-tweak": ("ArtisansKitpack_tweak/ArtisansKitpack_tweak.TP2", "ARTISANSKITPACK_TWEAK"),
}
LATE_RUN = "artisanskitpack-tweak-late-bg2"
LATE_COMPONENTS = [51010, 12012, 3202, 3302, 7205, 8204, 1010000]
KATANA_FIX = "feature:artisanskitpack-tweak:component-7205"
SWEEP = "feature:artisanskitpack-tweak:component-1010000"
# The kits whose items the pass changes, and the main components that run it.
INVISIBLE_BLADE = "feature:artisanskitpack:component-7005"
ELEMENTALIST = "feature:artisanskitpack:component-5001"
DANCER = "feature:bardicwonders:component-1005"
# Still not offered: the Favored Soul and the dual-class kit name tweak.
NOT_OFFERED = {("artisanskitpack", 30001), ("artisanskitpack-tweak", 1020000)}


def _load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def _load_profile(directory: Path) -> tuple[dict, dict]:
    collection = _load(directory / "collection.toml")
    preset = _load(directory / "presets/chris-recommended.toml")["selections"]
    return collection, preset


class Oct29KitpackChriz5PinTests(unittest.TestCase):
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
        cls.evidence = _load(ROOT / EVIDENCE)
        cls.records = {record["mod_id"]: record for record in cls.evidence["artifacts"]}
        cls.previous = {record["mod_id"]: record for record in _load(ROOT / PREVIOUS_EVIDENCE)["artifacts"]
                        if record["mod_id"] in INSTALLERS}

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
        self.assertEqual(self.evidence["verified_on"], "2026-10-29")
        self.assertEqual(self.evidence["evidence_kind"], "published-release-download-and-native-order-verification")
        for phrase in (TAG, "10a4a29", "fork PR #16", "byte for byte", "release-intake-20261027.toml"):
            self.assertIn(phrase, self.evidence["scope"])
        self.assertEqual(set(self.records), set(INSTALLERS))
        self.assertEqual(len(self.evidence["artifacts"]), len(INSTALLERS))
        url = (f"https://github.com/Chrizhermann/The-Artisan-s-Kitpack-Chriz-Balance-Patch/releases/download/"
               f"{TAG}/{FILENAME}")
        for mod_id, record in self.records.items():
            with self.subTest(mod=mod_id):
                self.assertNotIn("verified_on", record)
                self.assertEqual(record["artifact_id"], PIN)
                self.assertEqual(record["recipe_profiles"], PROFILES)
                self.assertEqual(record["source_reference"], TAG)
                self.assertEqual(record["source_commit"], TAG_COMMIT)
                self.assertEqual(record["immutable_source_url"], url)
                self.assertEqual(record["actual_length"], LENGTH)
                self.assertEqual(record["actual_sha256"], SHA256)
                self.assertRegex(record["tp2_sha256"], r"^[0-9a-f]{64}$")
                for flag in ("fresh_download", "fresh_extraction", "published_release_verified",
                             "native_component_order_verified"):
                    self.assertIs(record[flag], True, flag)
                previous = self.previous[mod_id]
                self.assertEqual(previous["artifact_id"], PREVIOUS)
                self.assertNotEqual(record["tp2_sha256"], previous["tp2_sha256"])
                self.assertEqual(len(record["native_components"]), len(set(record["native_components"])))
                # Only the main record narrows its order check to the curated profile, as before.
                self.assertEqual(record.get("native_order_recipe_profiles", record["recipe_profiles"]),
                                 previous.get("native_order_recipe_profiles", previous["recipe_profiles"]))
        self.assertEqual(self.records["artisanskitpack"]["native_order_recipe_profiles"],
                         ["recipes/curated-full-current"])
        self.assertEqual(len({record["tp2_sha256"] for record in self.records.values()}), 3)
        # The main and NPC TP2s keep chriz.4's components and order; the tweak TP2 adds 7205
        # right after 7203.
        for mod_id in ("artisanskitpack", "artisanskitpack-npc"):
            self.assertEqual(self.records[mod_id]["native_components"], self.previous[mod_id]["native_components"])
        previous = self.previous["artisanskitpack-tweak"]["native_components"]
        after = previous.index(7203) + 1
        self.assertEqual(self.records["artisanskitpack-tweak"]["native_components"],
                         previous[:after] + [7205] + previous[after:])

    def test_both_profiles_pin_chriz5_and_keep_chriz4(self):
        for profile_name, directory in self.directories:
            artifact = _load(directory / f"artifacts/{PIN}.toml")
            previous = _load(directory / f"artifacts/{PREVIOUS}.toml")
            with self.subTest(profile=str(directory)):
                self.assertEqual(artifact["id"], PIN)
                self.assertEqual(artifact["name"], previous["name"])
                self.assertEqual(artifact["version"], TAG.removeprefix("v"))
                self.assertEqual(artifact["acquisition"], "fetch-only")
                self.assertEqual(artifact["source"]["kind"], "github-release")
                self.assertEqual(artifact["source"]["expected_filename"], FILENAME)
                self.assertEqual(artifact["provenance"]["url"],
                                 f"https://github.com/Chrizhermann/The-Artisan-s-Kitpack-Chriz-Balance-Patch/"
                                 f"releases/tag/{TAG}")
                # Same layout, limits and redirect host as chriz.4, which stays for
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
                        # The chriz.4 records still describe the kept chriz.4 artifact.
                        self.assertEqual(previous["source"][key], self.previous[mod_id][measured], key)
                    mod = _load(directory / f"mods/{mod_id}.toml")
                    self.assertEqual(mod["artifact_id"], PIN, mod_id)
                    self.assertEqual(mod["tp2"], tp2)
                    self.assertIn(tp2, artifact["archive"]["tp2_paths"])
                tweak = {component["id"]: component["name"] for component in
                         _load(directory / "mods/artisanskitpack-tweak.toml")["components"]}
                self.assertEqual(tweak[7205], "Invisible Blade: Apply the Katana Fix to Items and Kits Added Later")
        # The checked-in curated artifacts are the generator's copies of the authored pins.
        for artifact_id in (PIN, PREVIOUS):
            name = f"artifacts/{artifact_id}.toml"
            self.assertEqual((ROOT / "recipes/curated-full-current" / name).read_bytes(),
                             (ROOT / "manifest" / name).read_bytes())

    def test_runs_offer_only_declared_components_in_native_order(self):
        tweak_native = self.records["artisanskitpack-tweak"]["native_components"]
        self.assertEqual(NATIVE_RUN_ORDERS[LATE_RUN], LATE_COMPONENTS)
        self.assertEqual(LATE_COMPONENTS, [c for c in tweak_native if c in LATE_COMPONENTS])
        for profile_name, directory in self.directories:
            collection = _load(directory / "collection.toml")
            for mod_id, record in self.records.items():
                native = record["native_components"]
                mod = _load(directory / f"mods/{mod_id}.toml")
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
            (late,) = [run for run in collection["runs"] if run["run_id"] == LATE_RUN]
            self.assertEqual(late["components"], LATE_COMPONENTS, str(directory))
        # The curated generator restores the main run's native order; the authored fallback
        # keeps the order it has had since chriz.1 (see the evidence record).
        native_main = self.records["artisanskitpack"]["native_components"]
        curated_main = NATIVE_RUN_ORDERS["artisanskitpack-main-bg2"]
        self.assertEqual(curated_main, [c for c in native_main if c in curated_main])

    def test_7205_is_a_default_without_requirements_before_8204_and_1010000(self):
        for name, _, collection, preset in self._each_profile():
            features = {feature["id"]: feature for feature in collection["features"]}
            feature = features[KATANA_FIX]
            self.assertEqual((feature["category"], feature["decision"], feature["readiness"]),
                             ("kits", "default", "ready"))
            self.assertEqual(feature["parent"], "mod:artisanskitpack-tweak")
            self.assertEqual(feature["title"], "Katana fix for items and kits from later mods")
            for phrase in ("Invisible Blade", "Bardic Wonders' Dancer", "katanas", "after the Kitpack",
                           "Every other kit stays blocked"):
                self.assertIn(phrase, feature["description"])
            # Its TP2 has no REQUIRE or predicate: the pass needs neither EEex nor a kit.
            self.assertEqual(feature.get("requires", []), [])
            self.assertEqual(feature.get("requires_any", []), [])
            self.assertEqual(feature.get("conflicts", []), [])
            self.assertNotIn("choice_group", feature)
            self.assertEqual(feature["components"], [{"run_id": LATE_RUN, "component": 7205}])
            self.assertEqual(preset.get(KATANA_FIX), "on")
            # The kits and components the pass concerns stay defaults.
            for kit in (INVISIBLE_BLADE, ELEMENTALIST):
                self.assertEqual(features[kit]["decision"], "default", kit)
            if name == "curated":
                self.assertEqual(features[DANCER]["decision"], "default")

            baseline = self._selected(collection, preset)
            late = [c for c in LATE_COMPONENTS if (LATE_RUN, c) in baseline]
            self.assertEqual(late, [12012, 3202, 3302, 7205, 8204, 1010000])
            self.assertEqual(baseline - self._selected(collection, preset, **{KATANA_FIX: "off"}),
                             {(LATE_RUN, 7205)})
            # Without EEex, 1010000 cannot install, and 7205 alone frees later mods' items.
            without_eeex = self._selected(collection, preset, **{"mod:eeex": "off"})
            self.assertIn((LATE_RUN, 7205), without_eeex)
            self.assertNotIn((LATE_RUN, 1010000), without_eeex)
            self.assertFalse(any(run_id == LATE_RUN for run_id, _ in
                                 self._selected(collection, preset, **{"mod:artisanskitpack-tweak": "off"})))
            group = next(g for g in collection["feature_groups"] if g["id"] == "challenge-mode")
            self.assertNotIn(KATANA_FIX, group["members"])

            # 7205 follows the runs that add katanas and kits: the Kitpack's main and NPC
            # runs and SCS in both profiles, Bardic Wonders (before the main run) and Tweaks
            # Anthology in the curated one.
            order = [run["run_id"] for run in collection["runs"]]
            for earlier in ("artisanskitpack-main-bg2", "artisanskitpack-npc-bg2",
                            "artisanskitpack-tweak-early-bg2", "stratagems-bg2"):
                self.assertLess(order.index(earlier), order.index(LATE_RUN), earlier)
            if name == "curated":
                for earlier in ("bardicwonders-bg2", "cdtweaks-bg2"):
                    self.assertLess(order.index(earlier), order.index(LATE_RUN), earlier)
                self.assertLess(order.index("bardicwonders-bg2"), order.index("artisanskitpack-main-bg2"))
            self.assertLess(order.index(LATE_RUN), order.index("eet-end-bg2"))
            self.assertEqual(features[SWEEP]["decision"], "default")

    def test_catalogs_map_changelog_and_todo_record_the_pin_and_the_decision(self):
        rows = {}
        for row in load_catalogs(ROOT / "docs/curation/components"):
            rows.setdefault(row.catalog, {})[row.component_id] = row
        for mod_id, (_, catalog) in INSTALLERS.items():
            with self.subTest(catalog=catalog):
                self.assertEqual(set(rows[catalog]), set(self.records[mod_id]["native_components"]))
                text = (ROOT / f"docs/curation/components/{catalog}.md").read_text(encoding="utf-8")
                self.assertIn(f"**{TAG}**", text)
                self.assertIn("`10a4a29`", text)
        tweak_rows = rows["ARTISANSKITPACK_TWEAK"]
        self.assertEqual(tweak_rows[7205].decision, Decision.DEFAULT)
        self.assertEqual(tweak_rows[1010000].decision, Decision.DEFAULT)
        self.assertEqual(tweak_rows[1020000].decision, Decision.EXCLUDED)
        self.assertEqual(rows["ARTISANSKITPACK"][7005].decision, Decision.DEFAULT)
        self.assertEqual(rows["ARTISANSKITPACK"][5001].decision, Decision.DEFAULT)
        curation_map = load_curation_map(ROOT / "manifest/curation-map.toml")
        (target,) = [target for target in curation_map.targets if target.target_id == KATANA_FIX]
        self.assertEqual(target.rows, (RowKey("ARTISANSKITPACK_TWEAK", 7205),))

        tweak = (ROOT / "docs/curation/components/ARTISANSKITPACK_TWEAK.md").read_text(encoding="utf-8")
        for phrase in ("`lib/shapeshifter_kit_bit.tpa`", "Christopher made it a default on 2026-10-29",
                       "Its TP2 requires nothing", "changes nothing (the fork's tests)", "SCS's Drow Katana",
                       "19 entries, 7 reference-installed"):
            self.assertIn(phrase, tweak)
        main = (ROOT / "docs/curation/components/ARTISANSKITPACK.md").read_text(encoding="utf-8")
        for phrase in ("- chriz.5: the Invisible Blade (`7005`) and Bardic Wonders' Dancer", "chriz.5's release notes"):
            self.assertIn(phrase, main)
        changelog = (ROOT / "docs/plans/2026-10-14-next-release-changelogs.md").read_text(encoding="utf-8")
        for phrase in ("**The Invisible Blade and Bardic Wonders' Dancer can use katanas** (Default)",
                       "(Artisan's Kitpack fork chriz.5)", f"The Artisan's Kitpack fork {TAG} (was chriz.1)",
                       f"{TAG} and The Tweaks Anthology", "**Artisan's Kitpack fork chriz.5**",
                       "shaman, Invisible Blade and Dancer weapon behavior until you install again"):
            self.assertIn(phrase, changelog)
        self.assertNotIn("The Artisan's Kitpack fork v4.81a-dev.85928f9-chriz.4 (was", changelog)
        todo = (ROOT / "docs/curation/components/FOLLOW_UPS.md").read_text(encoding="utf-8")
        self.assertIn(EVIDENCE, todo)
        self.assertNotIn("Its release as chriz.5 waits for Christopher's OK.", todo)


if __name__ == "__main__":
    unittest.main()
