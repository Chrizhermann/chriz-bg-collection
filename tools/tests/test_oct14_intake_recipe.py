"""October 14 intake: Modpack v0.2.0-alpha.12, Rebalance v0.8.3, SCS fine weapons, Tweaks 7649ced,
Bardic Wonders v2.9c-balance.8. Later intakes supersede the Modpack and Rebalance pins.

Static recipe, selection, dependency and order checks for both recipe profiles only.
The pins follow the October 14 publications and the unreleased Tweaks Anthology master
snapshot; nothing here is a game installation or gameplay acceptance.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import CURATED_GROUP_MEMBERS, NATIVE_RUN_ORDERS, _effective_features, build_recipe
from tools.curation_audit import Decision, RowKey, load_catalogs


ROOT = Path(__file__).resolve().parents[2]
PRE_RUN = "chriz-bg-modpack-pre-continuity-bg2"
REBALANCE_RUN = "chriz-bg-rebalance-bg2"
SCS_RUN = "stratagems-bg2"
TWEAKS_RUN = "cdtweaks-bg2"
MODPACK = "feature:chriz-bg-modpack:component-{}"
REBALANCE = "feature:chriz-bg-rebalance:component-{}"
SCS = "feature:stratagems:component-{}"
TWEAKS = "feature:cdtweaks:component-{}"
TWEAKS_SNAPSHOT = "cdtweaks-7649ced6cd25865874d787ec1a9abbc67b068729"

COMPANION_STATS = {
    230: ("Rasaad: Dexterity 17 and Constitution 16", "Rasaad's base Dexterity to 17 and Constitution to 16"),
    231: ("Dorn: Constitution 16", "Dorn's base Constitution to 16"),
    232: ("Eldoth: Strength 17, Constitution 16, Wisdom 9 and Intelligence 14",
          "Eldoth's base Strength to 17, Constitution to 16, Wisdom to 9 and Intelligence to 14"),
    233: ("Garrick: Dexterity 17 and Intelligence 15", "Garrick's base Dexterity to 17 and Intelligence to 15"),
    234: ("Quayle: Constitution 15", "Quayle's base Constitution to 15"),
}


def _load_profile(directory: Path) -> tuple[dict, dict]:
    collection = tomllib.loads((directory / "collection.toml").read_text(encoding="utf-8"))
    preset = tomllib.loads((directory / "presets/chris-recommended.toml").read_text(encoding="utf-8"))["selections"]
    return collection, preset


class Oct14IntakeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        cls.output = Path(cls.temp.name) / "recipe"
        # Synthetic evidence marker used only in this temporary test output.
        build_recipe(ROOT, cls.output, "0" * 40)
        cls.profiles = {
            "curated": (cls.output, *_load_profile(cls.output)),
            "fallback": (ROOT / "manifest", *_load_profile(ROOT / "manifest")),
        }
        cls.evidence = tomllib.loads(
            (ROOT / "manifest/evidence/release-intake-20261014.toml").read_text(encoding="utf-8"))
        cls.native = {record["mod_id"]: record["native_components"] for record in cls.evidence["artifacts"]}

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

    def _native(self, mod_id, components):
        return [component for component in self.native[mod_id] if component in components]

    def test_companion_stats_are_independent_defaults_before_continuity(self):
        for _, _, collection, preset in self._each_profile():
            runs = {run["run_id"]: run for run in collection["runs"]}
            order = [run["run_id"] for run in collection["runs"]]
            features = {feature["id"]: feature for feature in collection["features"]}
            components = runs[PRE_RUN]["components"]
            self.assertEqual(components[-5:], [230, 231, 232, 233, 234])
            self.assertEqual(components, self._native("chriz-bg-modpack", components))
            self.assertLess(order.index(PRE_RUN), order.index("chriz-bg-modpack-continuity-bg2"))
            self.assertLess(order.index(PRE_RUN), order.index("eet-end-bg2"))
            baseline = self._selected(collection, preset)
            for component, (title, change) in COMPANION_STATS.items():
                feature = features[MODPACK.format(component)]
                self.assertEqual(feature["title"], title)
                self.assertIn(f"Set {change}", feature["description"])
                self.assertIn("current hit points stay as they are", feature["description"])
                self.assertEqual((feature["category"], feature["decision"], feature["readiness"]),
                                 ("npcs", "default", "ready"))
                self.assertEqual(feature["parent"], "mod:chriz-bg-modpack")
                self.assertEqual(feature.get("requires", []), [])
                self.assertEqual(feature["components"], [{"run_id": PRE_RUN, "component": component}])
                self.assertEqual(preset.get(MODPACK.format(component)), "on")
                self.assertIn((PRE_RUN, component), baseline)
                # Each stat change is separately deselectable.
                self.assertEqual(baseline - self._selected(collection, preset, **{MODPACK.format(component): "off"}),
                                 {(PRE_RUN, component)})
        self.assertEqual(NATIVE_RUN_ORDERS[PRE_RUN][-5:], [230, 231, 232, 233, 234])

    def test_potion_uniqueness_is_a_choice_with_genius_stacking_by_default(self):
        strict, genius, heroism = REBALANCE.format(330), REBALANCE.format(331), REBALANCE.format(332)
        for _, _, collection, preset in self._each_profile():
            runs = {run["run_id"]: run for run in collection["runs"]}
            features = {feature["id"]: feature for feature in collection["features"]}
            components = runs[REBALANCE_RUN]["components"]
            # v0.8.5 (test_oct26_alpha22_pins_recipe) declares Chris Revisions 340 after
            # them and v0.8.6 (test_oct27_alpha22_pins_recipe) the Time Stop and Haste
            # effect clocks 360 after 340, each checked against its own evidence; the v0.8.3
            # rows keep their order.
            self.assertEqual(components[-5:], [330, 331, 332, 340, 360])
            v083_rows = [component for component in components if component not in (340, 360)]
            self.assertEqual(v083_rows, self._native("chriz-bg-rebalance", v083_rows))
            self.assertEqual((features[strict]["decision"], features[genius]["decision"], features[heroism]["decision"]),
                             ("optional", "default", "default"))
            self.assertEqual(features[strict]["choice_group"], features[genius]["choice_group"])
            self.assertEqual([c["feature_id"] for c in features[strict]["conflicts"]], [genius])
            self.assertEqual([c["feature_id"] for c in features[genius]["conflicts"]], [strict])
            self.assertNotIn("choice_group", features[heroism])
            self.assertIn("Potions of Genius still stack", features[genius]["description"])
            self.assertIn("Potions of Genius follow the same rule", features[strict]["description"])
            baseline = self._selected(collection, preset)
            self.assertTrue({(REBALANCE_RUN, 331), (REBALANCE_RUN, 332)} <= baseline)
            self.assertNotIn((REBALANCE_RUN, 330), baseline)
            switched = self._selected(collection, preset, **{strict: "on", genius: "off"})
            self.assertEqual(switched - baseline, {(REBALANCE_RUN, 330)})
            self.assertEqual(baseline - switched, {(REBALANCE_RUN, 331)})
            self.assertEqual(baseline - self._selected(collection, preset, **{heroism: "off"}), {(REBALANCE_RUN, 332)})

    def test_scs_fine_weapons_are_default_and_fine_ammunition_is_offered_unchecked(self):
        weapons, ammunition = SCS.format(3021), SCS.format(3010)
        for _, _, collection, preset in self._each_profile():
            runs = {run["run_id"]: run for run in collection["runs"]}
            features = {feature["id"]: feature for feature in collection["features"]}
            components = runs[SCS_RUN]["components"]
            self.assertEqual(components[:7], [2000, 2500, 2510, 3010, 3015, 3021, 3040])
            self.assertEqual(components, self._native("stratagems", components))
            # 3020 is BG2-only and 3022 is the alternative that may shatter; neither is offered.
            self.assertFalse({3020, 3022} & set(components))
            self.assertEqual((features[weapons]["decision"], features[ammunition]["decision"]), ("default", "optional"))
            self.assertIn("never break", features[weapons]["description"])
            baseline = self._selected(collection, preset)
            self.assertIn((SCS_RUN, 3021), baseline)
            self.assertNotIn((SCS_RUN, 3010), baseline)
            self.assertEqual(self._selected(collection, preset, **{ammunition: "on"}) - baseline, {(SCS_RUN, 3010)})
            self.assertEqual(baseline - self._selected(collection, preset, **{weapons: "off"}), {(SCS_RUN, 3021)})
            group = next(g for g in collection["feature_groups"] if g["id"] == "challenge-mode")
            # Fine ammunition is not part of Challenge Mode (Christopher, 2026-10-14).
            self.assertNotIn(ammunition, group["members"])

    def test_tweaks_snapshot_defaults_and_challenge_member_exist_only_in_the_curated_recipe(self):
        _, curated, preset = self.profiles["curated"]
        runs = {run["run_id"]: run for run in curated["runs"]}
        features = {feature["id"]: feature for feature in curated["features"]}
        components = runs[TWEAKS_RUN]["components"]
        self.assertEqual(components, self._native("cdtweaks", components))
        baseline = self._selected(curated, preset)
        for component in (3220, 3500, 3510):
            self.assertEqual(features[TWEAKS.format(component)]["decision"], "default")
            self.assertIn((TWEAKS_RUN, component), baseline)
        # 3500's TP2 requires EEex; WeiDU would skip it without EEex and fail the run.
        self.assertEqual(features[TWEAKS.format(3500)]["requires"], ["feature:eeex:mandatory-components"])
        without_eeex = self._selected(curated, preset, **{"mod:eeex": "off"})
        self.assertNotIn((TWEAKS_RUN, 3500), without_eeex)
        self.assertTrue({(TWEAKS_RUN, 3220), (TWEAKS_RUN, 3510)} <= without_eeex)
        for component in (2760, 3220, 3510):
            self.assertEqual(features[TWEAKS.format(component)].get("requires", []), [])
        fences = TWEAKS.format(2760)
        self.assertEqual(features[fences]["decision"], "optional")
        self.assertNotIn((TWEAKS_RUN, 2760), baseline)
        self.assertEqual(features[TWEAKS.format(2210)]["title"],
                         "True grandmastery [Baldurdash]: High Mastery and Grand Mastery give +3 to hit and +5 damage")
        self.assertIn("Ajantis", features[TWEAKS.format(1257)]["title"])
        # Not offered: 1258 is skipped by WeiDU after 1257, which now moves Ajantis too;
        # the others per Christopher; 260 is withdrawn and 3470 deprecated upstream.
        for component in (1258, 2212, 2213, 2670, 2750, 2770, 2780, 3520, 3530, 3531, 4240, 6360, 260, 3470):
            self.assertNotIn(TWEAKS.format(component), features)
            self.assertNotIn(component, components)
        group = next(g for g in curated["feature_groups"] if g["id"] == "challenge-mode")
        self.assertEqual(group["members"][-1], fences)
        self.assertEqual(CURATED_GROUP_MEMBERS, {"challenge-mode": [fences]})
        self.assertEqual(self._selected(curated, preset, **{fences: "on"}) - baseline, {(TWEAKS_RUN, 2760)})
        _, fallback, _ = self.profiles["fallback"]
        self.assertFalse(any(run["mod_id"] == "cdtweaks" for run in fallback["runs"]))
        mod = tomllib.loads((self.output / "mods/cdtweaks.toml").read_text(encoding="utf-8"))
        self.assertEqual(mod["artifact_id"], TWEAKS_SNAPSHOT)
        artifact = tomllib.loads((self.output / f"artifacts/{TWEAKS_SNAPSHOT}.toml").read_text(encoding="utf-8"))
        self.assertEqual((artifact["source"]["kind"], artifact["archive"]["root_rule"]),
                         ("github-commit-zip", "single-wrapper"))
        # v18 stays available as history for installations made from earlier recipes.
        self.assertTrue((self.output / "artifacts/cdtweaks-18.toml").is_file())

    def test_catalog_rows_record_the_intake_decisions(self):
        rows = {RowKey(row.catalog, row.component_id): row for row in load_catalogs(ROOT / "docs/curation/components")}
        expected = {
            **{RowKey("CHRIZ-BG-MODPACK", component): Decision.DEFAULT for component in COMPANION_STATS},
            RowKey("CHRIZ-BG-REBALANCE", 330): Decision.OPTIONAL,
            RowKey("CHRIZ-BG-REBALANCE", 331): Decision.DEFAULT,
            RowKey("CHRIZ-BG-REBALANCE", 332): Decision.DEFAULT,
            RowKey("STRATAGEMS", 3010): Decision.OPTIONAL,
            RowKey("STRATAGEMS", 3021): Decision.DEFAULT,
            RowKey("CDTWEAKS", 2210): Decision.DEFAULT,
            RowKey("CDTWEAKS", 2760): Decision.OPTIONAL,
            RowKey("CDTWEAKS", 3220): Decision.DEFAULT,
            RowKey("CDTWEAKS", 3500): Decision.DEFAULT,
            RowKey("CDTWEAKS", 3510): Decision.DEFAULT,
            **{RowKey("CDTWEAKS", component): Decision.EXCLUDED
               for component in (1258, 2212, 2213, 2670, 2750, 2770, 2780, 3520, 3530, 3531, 4240, 6360)},
        }
        for key, decision in expected.items():
            self.assertEqual(rows[key].decision, decision, key)
        self.assertNotIn(RowKey("CDTWEAKS", 260), rows)
        self.assertNotIn(RowKey("CDTWEAKS", 3470), rows)
        cdtweaks = [row.component_id for row in load_catalogs(ROOT / "docs/curation/components") if row.catalog == "CDTWEAKS"]
        # The catalog lists every non-deprecated snapshot component in TP2 order.
        self.assertEqual(cdtweaks, self._native("cdtweaks", cdtweaks))
        self.assertEqual(len(cdtweaks), 441)

    def test_intake_evidence_matches_current_pins_and_native_order(self):
        self.assertEqual(self.evidence["verified_on"], "2026-10-14")
        self.assertEqual({record["mod_id"] for record in self.evidence["artifacts"]},
                         {"chriz-bg-modpack", "chriz-bg-rebalance", "stratagems", "cdtweaks", "bardicwonders"})
        directories = {
            "manifest": [ROOT / "manifest"],
            "recipes/curated-full-current": [ROOT / "recipes/curated-full-current", self.output],
        }
        # The October 20 intake (test_oct20_intake_recipe) supersedes the Modpack pin and
        # the October 26 intake (test_oct26_alpha22_pins_recipe) the Rebalance pin; their
        # pinned files stay exactly as verified here.
        superseded = {"chriz-bg-modpack", "chriz-bg-rebalance"}
        for record in self.evidence["artifacts"]:
            for flag in ("fresh_download", "fresh_extraction", "native_component_order_verified"):
                self.assertIs(record[flag], True, (record["mod_id"], flag))
            # Only the Tweaks commit snapshot has no published release behind it.
            self.assertIs(record["published_release_verified"], record["mod_id"] != "cdtweaks")
            for name, profiles in directories.items():
                for profile in profiles:
                    with self.subTest(profile=str(profile), mod=record["mod_id"]):
                        mod_path = profile / f"mods/{record['mod_id']}.toml"
                        if record["mod_id"] in superseded and name in record["recipe_profiles"]:
                            artifact = tomllib.loads(
                                (profile / f"artifacts/{record['artifact_id']}.toml").read_text(encoding="utf-8"))
                            for key, measured in (("sha256", "actual_sha256"), ("expected_length", "actual_length"),
                                                  ("url", "immutable_source_url"), ("reference", "source_reference")):
                                self.assertEqual(artifact["source"][key], record[measured])
                            mod = tomllib.loads(mod_path.read_text(encoding="utf-8"))
                            self.assertNotEqual(mod["artifact_id"], record["artifact_id"])
                            continue
                        if name not in record["recipe_profiles"]:
                            pinned = mod_path.is_file() and tomllib.loads(
                                mod_path.read_text(encoding="utf-8"))["artifact_id"] == record["artifact_id"]
                            self.assertFalse(pinned)
                            continue
                        mod = tomllib.loads(mod_path.read_text(encoding="utf-8"))
                        self.assertEqual(mod["artifact_id"], record["artifact_id"])
                        artifact = tomllib.loads(
                            (profile / f"artifacts/{record['artifact_id']}.toml").read_text(encoding="utf-8"))
                        for key, measured in (("sha256", "actual_sha256"), ("expected_length", "actual_length"),
                                              ("url", "immutable_source_url"), ("reference", "source_reference")):
                            self.assertEqual(artifact["source"][key], record[measured])
                        collection = tomllib.loads((profile / "collection.toml").read_text(encoding="utf-8"))
                        runs = [run for run in collection["runs"] if run["mod_id"] == record["mod_id"]]
                        self.assertTrue(runs)
                        for run in runs:
                            self.assertEqual(run["components"], [c for c in record["native_components"]
                                                                 if c in run["components"]], run["run_id"])

    def test_challenge_mode_adds_only_its_members(self):
        for name, _, collection, preset in self._each_profile():
            group = next(g for g in collection["feature_groups"] if g["id"] == "challenge-mode")
            baseline = self._selected(collection, preset)
            challenge = self._selected(collection, preset, **{member: "on" for member in group["members"]})
            added = {
                (ref["run_id"], ref["component"])
                for feature in collection["features"] if feature["id"] in group["members"]
                for ref in feature["components"]
            }
            self.assertEqual(challenge - baseline, added)
            self.assertEqual(baseline - challenge, set())
            self.assertEqual((TWEAKS_RUN, 2760) in added, name == "curated")


if __name__ == "__main__":
    unittest.main()
