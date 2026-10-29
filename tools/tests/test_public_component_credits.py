import json
import tomllib
import unittest
from pathlib import Path

from tools.curated_full_recipe import _effective_features
from tools.public_component_credits import build_public_component_credits


class PublicComponentCreditsTests(unittest.TestCase):
    def setUp(self) -> None:
        self.root = Path(__file__).resolve().parents[2]
        self.recipe = self.root / "recipes/curated-full-current"
        self.result = build_public_component_credits(
            self.recipe, self.root / "app/package.json"
        )

    def test_matches_the_effective_recommended_selection(self) -> None:
        collection = tomllib.loads((self.recipe / "collection.toml").read_text(encoding="utf-8"))
        preset = tomllib.loads(
            (self.recipe / "presets/chris-recommended.toml").read_text(encoding="utf-8")
        )["selections"]
        effective = _effective_features(collection, preset)
        expected: dict[str, set[int]] = {}
        for feature in collection["features"]:
            if effective[feature["id"]]:
                for component in feature.get("components", []):
                    expected.setdefault(component["run_id"], set()).add(component["component"])

        actual = {
            run["id"]: [component["id"] for component in run["components"]]
            for mod in self.result["mods"]
            for run in mod["runs"]
        }
        runs = {run["run_id"]: run for run in collection["runs"]}
        self.assertEqual(set(actual), set(expected))
        for run_id, selected in expected.items():
            self.assertEqual(actual[run_id], [value for value in runs[run_id]["components"] if value in selected])
            self.assertEqual(len(actual[run_id]), len(set(actual[run_id])))
        self.assertEqual(self.result["componentCount"], sum(map(len, actual.values())))
        # Alpha.18 adds Rebalance 301/302 to alpha.17's 449 components; interval-save
        # controls and the existing 15-minute rotating saves remain selected.
        controls = "feature:hiddengameplayoptions:component-38"
        self.assertTrue(effective[controls])
        controls_feature = next(feature for feature in collection["features"] if feature["id"] == controls)
        self.assertEqual(controls_feature["components"], [{"run_id": "hiddengameplayoptions-bg2", "component": 38}])
        self.assertEqual(actual["hiddengameplayoptions-bg2"].count(38), 1)
        self.assertIn(3354, actual["cdtweaks-bg2"])
        self.assertNotIn(301, actual["chriz-bg-rebalance-bg2"])
        # October 13 default bard ceilings add Rebalance 420/421 and Bardic 3010's late run.
        self.assertTrue({420, 421} <= set(actual["chriz-bg-rebalance-bg2"]))
        self.assertEqual(actual["bardicwonders-progression-bg2"], [3010])
        # October 13: EET's desktop shortcut (EET 100) is no longer selected by default.
        self.assertNotIn(100, actual["eet-initialize-bg2"])
        # The October 13 intake adds Modpack 237/238/660 and SoD 177/198/285 and
        # moves scroll policy 430 into its own final run.
        self.assertEqual(actual["chriz-bg-modpack-scrolls-bg2"], [430])
        # The October 14 intake adds default Modpack 230-234, Rebalance 331/332,
        # SCS 3021 and Tweaks 3220/3500/3510; October 16 makes all UB rows optional.
        # The October 20 intake (Modpack alpha.13) adds default EET join XP 239.
        # The October 22 intake adds 18 default player voicepacks, one run each.
        # The October 23 intake (SoD Remix v0.6.15) adds default Imoen choice 292.
        # The October 23 quest-mod intake makes BG1UB optional and unselected (its 16
        # former defaults and its run leave); the new quest mods are unchecked.
        # The October 26 intake adds default Rebalance 340; EEex 9 stays unchecked.
        # Its Modpack alpha.14 pin adds default 135 and 460; chapter caps 652 stay unchecked.
        self.assertIn(340, actual["chriz-bg-rebalance-bg2"])
        self.assertNotIn(9, actual["eeex-bg2"])
        self.assertTrue({135, 460} <= set(actual["chriz-bg-modpack-bg2"]))
        self.assertNotIn("chriz-bg-modpack-campaign-xp-bg2", actual)
        # October 27: Tweaks 2292 (druids level on the cleric XP table) leaves the defaults.
        self.assertNotIn(2292, actual["cdtweaks-bg2"])
        # October 28: the Rebalance v0.8.6 pin adds default 360.
        self.assertIn(360, actual["chriz-bg-rebalance-bg2"])
        # October 29: the Kitpack's weapon-usability sweep 1010000 ends the late tweak run,
        # and Sirene NPC for BG:EE adds its own run with default 0, 3 and 6.
        self.assertEqual(actual["artisanskitpack-tweak-late-bg2"][-1], 1010000)
        self.assertEqual(actual["sirene-bg1-bg2"], [0, 3, 6])
        self.assertEqual(self.result["componentCount"], 473)
        self.assertEqual(len(actual), 70)

    def test_contains_only_public_credit_fields_and_distinct_versions(self) -> None:
        # The app and bundled collection are versioned independently; each field must come from its own source.
        app_version = json.loads((self.root / "app/package.json").read_text(encoding="utf-8"))["version"]
        recipe_version = json.loads((self.recipe / "release.json").read_text(encoding="utf-8"))["version"]
        self.assertEqual(self.result["applicationVersion"], app_version)
        self.assertEqual(self.result["recipeVersion"], recipe_version)
        self.assertEqual(recipe_version, "0.1.0-alpha.20")
        encoded = json.dumps(self.result).lower()
        for forbidden in ("artifact_id", "source_reference", "expected_filename", ".zip", ".iemod", "c:\\\\", "creator-full"):
            self.assertNotIn(forbidden, encoded)
        for mod in self.result["mods"]:
            self.assertTrue(mod["homepage"].startswith(("https://", "http://")))
            self.assertEqual(set(mod), {"id", "name", "version", "homepage", "runs"})


if __name__ == "__main__":
    unittest.main()
