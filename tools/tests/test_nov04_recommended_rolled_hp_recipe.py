"""November 4: Recommended uses rolled HP (Christopher, 2026-11-04, for Chriz Easy BG 0.1.2).

Tweaks Anthology 3000 (Higher HP on Level Up -> Maximum) and 3010 (Maximum HP Creatures, for
all creatures in game) leave the defaults and stay offered unchecked, the way the fight-AI
labs run the game. 2999 (Max HP at Level One) stays on; 3001, 3002, 3008, 3011 and 3012 stay
off. The Kitpack's and Bardic Wonders' components with the same numbers are other features.
Static recipe checks only; nothing here is a game installation.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import _effective_features, build_recipe


ROOT = Path(__file__).resolve().parents[2]
RUN = "cdtweaks-bg2"
OFF = (3000, 3010)
STILL_OFF = (3001, 3002, 3008, 3011, 3012)
GROUPS = {
    "choice:cdtweaks:higher-hp-on-level-up": (3000, 3001, 3002),
    "choice:cdtweaks:maximum-hp-creatures-the-bigg": (3010, 3011, 3012),
}


def _tweak(component: int) -> str:
    return f"feature:cdtweaks:component-{component}"


def _load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def _selected(collection: dict, selection: dict) -> set[tuple[str, int]]:
    effective = _effective_features(collection, selection)
    return {(ref["run_id"], ref["component"]) for feature in collection["features"]
            if effective[feature["id"]] for ref in feature.get("components", [])}


class Nov04RecommendedRolledHpTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        cls.output = Path(cls.temp.name) / "recipe"
        # Synthetic evidence marker used only in this temporary test output.
        build_recipe(ROOT, cls.output, "0" * 40)
        cls.directories = [ROOT / "recipes/curated-full-current", cls.output]

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def test_both_are_unchecked_options_and_the_rest_stays(self):
        for directory in self.directories:
            collection = _load(directory / "collection.toml")
            preset = _load(directory / "presets/chris-recommended.toml")["selections"]
            features = {feature["id"]: feature for feature in collection["features"]}
            selected = _selected(collection, preset)
            with self.subTest(profile=str(directory)):
                for component in OFF + STILL_OFF:
                    self.assertEqual(features[_tweak(component)]["decision"], "optional", component)
                    self.assertEqual(features[_tweak(component)]["readiness"], "ready", component)
                    self.assertNotEqual(preset.get(_tweak(component), "off"), "on", component)
                    self.assertNotIn((RUN, component), selected)
                self.assertEqual(features[_tweak(2999)]["decision"], "default")
                self.assertEqual(preset[_tweak(2999)], "on")
                self.assertIn((RUN, 2999), selected)

    def test_each_option_can_still_be_ticked_alone(self):
        for directory in self.directories:
            collection = _load(directory / "collection.toml")
            preset = _load(directory / "presets/chris-recommended.toml")["selections"]
            baseline = _selected(collection, preset)
            features = {feature["id"]: feature for feature in collection["features"]}
            for group, components in GROUPS.items():
                for component in components:
                    with self.subTest(profile=str(directory), component=component):
                        self.assertEqual(features[_tweak(component)]["choice_group"], group)
                        ticked = _selected(collection, {**preset, _tweak(component): "on"})
                        self.assertEqual(ticked - baseline, {(RUN, component)})
                        self.assertEqual(baseline - ticked, set())
                        self.assertEqual(
                            {item["feature_id"] for item in features[_tweak(component)]["conflicts"]},
                            {_tweak(other) for other in components if other != component},
                        )

    def test_the_run_keeps_offering_the_components_in_their_order(self):
        for directory in self.directories:
            collection = _load(directory / "collection.toml")
            (run,) = [run for run in collection["runs"] if run["run_id"] == RUN]
            with self.subTest(profile=str(directory)):
                hp = [component for component in run["components"]
                      if component in (2999, 3000, 3001, 3002, 3008, 3010, 3011, 3012)]
                self.assertEqual(hp, [2999, 3000, 3001, 3002, 3008, 3010, 3011, 3012])

    def test_the_same_numbers_of_other_mods_stay_on(self):
        for directory in self.directories:
            preset = _load(directory / "presets/chris-recommended.toml")["selections"]
            with self.subTest(profile=str(directory)):
                for feature_id in ("feature:artisanskitpack:component-3000",
                                   "feature:artisanskitpack:component-3010",
                                   "feature:bardicwonders:component-3010"):
                    self.assertEqual(preset[feature_id], "on", feature_id)


if __name__ == "__main__":
    unittest.main()
