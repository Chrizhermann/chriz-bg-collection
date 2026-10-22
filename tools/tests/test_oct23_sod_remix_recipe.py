"""October 23 intake: SoD Remix v0.6.15 and the Imoen in Shadows of Amn choice.

Static recipe, selection, dependency and order checks for both recipe profiles only.
The pin follows the October 23 publication; nothing here is a game installation or
gameplay acceptance.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import _effective_features, build_recipe
from tools.curation_audit import Decision, RowKey, load_catalogs, load_curation_map


ROOT = Path(__file__).resolve().parents[2]
MOD_ID = "chriz-sod-remix"
ARTIFACT_ID = "chriz-sod-remix-0.6.15"
PREVIOUS_ARTIFACT_ID = "chriz-sod-remix-0.6.13"
SOD_RUN = "chriz-sod-remix-bg2"
SOD = "feature:chriz-sod-remix:component-{}"
BUNDLE = "feature:chriz-sod-remix:mandatory-components"
EEEX_CORE = "feature:eeex:mandatory-components"
CHOICE_GROUP = "sod-remix-imoen-soa"
SUBGROUP = "SoD remix: Imoen in Shadows of Amn (EET; her SoD gear is handed over like the party's)"
# Christopher, 2026-10-16: three choices, the vanilla BG2 thief-to-mage dual by default.
DEFAULT_IMOEN, *ALTERNATIVE_IMOEN = IMOEN = (292, 293, 294)
# v0.6.15 TP2 declaration order (49 declarations).
V0615_NATIVE = [
    100, 110, 115, 120, 130, 135, 140, 150, 145, 160, 170, 180, 175, 176, 177, 185, 190, 195,
    210, 197, 198, 187, 200, 215, 220, 225, 245, 230, 235, 240, 250, 255, 256, 257, 258, 260,
    265, 266, 270, 280, 285, 290, 291, 900, 901, 910, 292, 293, 294,
]
MANDATORY_BUNDLE = [
    100, 110, 115, 120, 130, 135, 140, 150, 145, 160, 170, 180, 175, 177, 185, 190, 195, 210,
    197, 198, 187, 200, 215, 220, 225, 245, 230, 240, 250, 255, 256, 260, 265, 266, 270, 280,
    285, 290, 900, 910,
]
# The run keeps its place: after EET_end (curated: after the Bardic dialogue patch), before HGO.
RUN_NEIGHBOURS = {
    "fallback": ("eet-end-bg2", "hiddengameplayoptions-bg2"),
    "curated": ("bardicwonders-dialogue-patch-bg2", "hiddengameplayoptions-bg2"),
}


def _load_profile(directory: Path) -> tuple[dict, dict]:
    collection = tomllib.loads((directory / "collection.toml").read_text(encoding="utf-8"))
    preset = tomllib.loads((directory / "presets/chris-recommended.toml").read_text(encoding="utf-8"))["selections"]
    return collection, preset


class Oct23SodRemixIntakeTests(unittest.TestCase):
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
            (ROOT / "manifest/evidence/release-intake-20261023.toml").read_text(encoding="utf-8"))
        [cls.record] = cls.evidence["artifacts"]

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

    @staticmethod
    def _imoen(selected):
        return {component for run_id, component in selected if run_id == SOD_RUN and component in IMOEN}

    def test_imoen_choice_is_one_group_with_the_vanilla_dual_as_default(self):
        for _, _, collection, preset in self._each_profile():
            features = {feature["id"]: feature for feature in collection["features"]}
            for component in IMOEN:
                feature = features[SOD.format(component)]
                others = {SOD.format(other) for other in IMOEN if other != component}
                self.assertEqual(feature["choice_group"], CHOICE_GROUP)
                self.assertEqual(feature["group_label"], SUBGROUP)
                self.assertEqual(feature["parent"], "mod:chriz-sod-remix")
                self.assertEqual(feature["readiness"], "ready")
                self.assertEqual(feature["components"], [{"run_id": SOD_RUN, "component": component}])
                self.assertEqual({conflict["feature_id"] for conflict in feature["conflicts"]}, others)
                # 160 and 290 come from the mandatory bundle, EET_end from the mod itself.
                self.assertEqual(feature["requires"][0], BUNDLE)
                self.assertIn("handed over like the party's", feature["description"])
            self.assertEqual(features[SOD.format(292)]["decision"], "default")
            self.assertEqual(preset.get(SOD.format(292)), "on")
            for component in ALTERNATIVE_IMOEN:
                self.assertEqual(features[SOD.format(component)]["decision"], "optional")
                self.assertNotEqual(preset.get(SOD.format(component)), "on")
            self.assertEqual(features[SOD.format(292)]["requires"], [BUNDLE])
            self.assertEqual(features[SOD.format(293)]["requires"], [BUNDLE])
            # Only the dual-class at her SoD thief level needs EEex (its TP2 checks M___EEex.lua).
            self.assertEqual(features[SOD.format(294)]["requires"], [BUNDLE, EEEX_CORE])
            self.assertIn("thief 7 / mage 8", features[SOD.format(292)]["description"])
            self.assertIn("Requires EEex", features[SOD.format(294)]["description"])

    def test_exactly_one_imoen_choice_installs(self):
        for _, _, collection, preset in self._each_profile():
            off = {SOD.format(292): "off"}
            self.assertEqual(self._imoen(self._selected(collection, preset)), {292})
            self.assertEqual(self._imoen(self._selected(collection, preset, **off, **{SOD.format(293): "on"})), {293})
            self.assertEqual(self._imoen(self._selected(collection, preset, **off, **{SOD.format(294): "on"})), {294})
            # Two choices at once install neither; the installer offers them as alternatives.
            self.assertEqual(self._imoen(self._selected(collection, preset, **{SOD.format(293): "on"})), set())
            self.assertEqual(self._imoen(self._selected(collection, preset, **off)), set())
            # 294 needs EEex; the default and 293 do not.
            self.assertEqual(self._imoen(self._selected(
                collection, preset, **off, **{SOD.format(294): "on", "mod:eeex": "off"})), set())
            self.assertEqual(self._imoen(self._selected(collection, preset, **{"mod:eeex": "off"})), {292})
            self.assertEqual(self._imoen(self._selected(collection, preset, **{"mod:chriz-sod-remix": "off"})), set())

    def test_run_keeps_its_place_and_native_order(self):
        for name, _, collection, preset in self._each_profile():
            runs = {run["run_id"]: run for run in collection["runs"]}
            order = [run["run_id"] for run in collection["runs"]]
            components = runs[SOD_RUN]["components"]
            self.assertEqual(components, [*MANDATORY_BUNDLE[:31], 257, *MANDATORY_BUNDLE[31:], *IMOEN])
            self.assertEqual(components, [c for c in V0615_NATIVE if c in components])
            self.assertFalse({176, 235, 258, 291, 901} & set(components))
            self.assertEqual(runs[SOD_RUN]["phase"], "post-eet-end")
            before, after = RUN_NEIGHBOURS[name]
            index = order.index(SOD_RUN)
            self.assertEqual((order[index - 1], order[index + 1]), (before, after))
            features = {feature["id"]: feature for feature in collection["features"]}
            self.assertEqual([ref["component"] for ref in features[BUNDLE]["components"]], MANDATORY_BUNDLE)
            selected = self._selected(collection, preset)
            self.assertEqual(sorted(c for run_id, c in selected if run_id == SOD_RUN), sorted([*MANDATORY_BUNDLE, 292]))

    def test_bridge_finale_and_summon_fog_tweak_install_together(self):
        # v0.6.15 fixes chriz-sod-rebalance#31; the static plan records no blocker between them.
        _, collection, preset = self.profiles["curated"]
        selected = self._selected(collection, preset)
        self.assertIn((SOD_RUN, 256), selected)
        self.assertIn(("cdtweaks-bg2", 3500), selected)
        names = {SOD.format(256), BUNDLE, "feature:cdtweaks:component-3500"}
        for feature in collection["features"]:
            for conflict in feature.get("conflicts", []):
                self.assertFalse({feature["id"], conflict["feature_id"]} <= names, feature["id"])
        for directory in (self.output, ROOT / "manifest"):
            limitations = (directory / "releases/v0.1.0-alpha.1/known-limitations.toml").read_text(encoding="utf-8")
            self.assertNotIn("3500", limitations)
            self.assertNotIn("CDTFOW", limitations.upper())

    def test_catalog_rows_and_curation_map_record_the_decision(self):
        rows = {RowKey(row.catalog, row.component_id): row for row in load_catalogs(ROOT / "docs/curation/components")}
        expected = {292: Decision.DEFAULT, 293: Decision.OPTIONAL, 294: Decision.OPTIONAL,
                    256: Decision.MANDATORY, 257: Decision.OPTIONAL, 291: Decision.EXCLUDED, 901: Decision.EXCLUDED}
        for component, decision in expected.items():
            self.assertEqual(rows[RowKey("CHRIZ-SOD-REMIX", component)].decision, decision, component)
        for component in IMOEN:
            self.assertEqual(rows[RowKey("CHRIZ-SOD-REMIX", component)].subgroup, SUBGROUP)
        mapping = load_curation_map(ROOT / "manifest/curation-map.toml")
        [group] = [group for group in mapping.choice_groups if group.group_id == CHOICE_GROUP]
        self.assertEqual(group.options, tuple(RowKey("CHRIZ-SOD-REMIX", component) for component in IMOEN))
        self.assertEqual(group.default, "CHRIZ-SOD-REMIX:292")
        targets = {target.target_id: target for target in mapping.targets}
        for component in IMOEN:
            self.assertEqual(targets[SOD.format(component)].rows, (RowKey("CHRIZ-SOD-REMIX", component),))
        catalog = (ROOT / "docs/curation/components/CHRIZ-SOD-REMIX.md").read_text(encoding="utf-8")
        self.assertIn("Pinned public release **v0.6.15**", catalog)
        self.assertNotIn("Pinned public release **v0.6.11**", catalog)

    def test_intake_evidence_matches_current_pins_and_native_order(self):
        record = self.record
        self.assertEqual(self.evidence["verified_on"], "2026-10-23")
        self.assertEqual(record["mod_id"], MOD_ID)
        self.assertEqual(record["artifact_id"], ARTIFACT_ID)
        self.assertEqual(record["native_components"], V0615_NATIVE)
        for flag in ("fresh_download", "fresh_extraction", "published_release_verified",
                     "native_component_order_verified"):
            self.assertIs(record[flag], True, flag)
        self.assertEqual(record["recipe_profiles"], ["manifest", "recipes/curated-full-current"])
        for profile in (ROOT / "manifest", ROOT / "recipes/curated-full-current", self.output):
            with self.subTest(profile=str(profile)):
                mod = tomllib.loads((profile / f"mods/{MOD_ID}.toml").read_text(encoding="utf-8"))
                self.assertEqual(mod["artifact_id"], ARTIFACT_ID)
                self.assertTrue(set(IMOEN) <= {component["id"] for component in mod["components"]})
                artifact = tomllib.loads((profile / f"artifacts/{ARTIFACT_ID}.toml").read_text(encoding="utf-8"))
                for key, measured in (("sha256", "actual_sha256"), ("expected_length", "actual_length"),
                                      ("url", "immutable_source_url"), ("reference", "source_reference")):
                    self.assertEqual(artifact["source"][key], record[measured])
                self.assertEqual(artifact["source"]["expected_filename"], "chriz-sod-remix-v0.6.15.zip")
                # Same layout and limits as v0.6.13, which stays available as history.
                previous = tomllib.loads(
                    (profile / f"artifacts/{PREVIOUS_ARTIFACT_ID}.toml").read_text(encoding="utf-8"))
                self.assertEqual(artifact["archive"], previous["archive"])
                self.assertEqual(artifact["source"]["redirect_hosts"], previous["source"]["redirect_hosts"])
                collection = tomllib.loads((profile / "collection.toml").read_text(encoding="utf-8"))
                for run in (run for run in collection["runs"] if run["mod_id"] == MOD_ID):
                    self.assertEqual(run["components"], [c for c in V0615_NATIVE if c in run["components"]])
        # The checked-in curated artifact is the generator's copy of the authored pin.
        self.assertEqual((ROOT / f"recipes/curated-full-current/artifacts/{ARTIFACT_ID}.toml").read_bytes(),
                         (ROOT / f"manifest/artifacts/{ARTIFACT_ID}.toml").read_bytes())


if __name__ == "__main__":
    unittest.main()
