"""September 16 release selections: bounded planning checks, not game acceptance."""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import _effective_features, build_recipe


class ExpandedReleaseRecipeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.root = Path(__file__).resolve().parents[2]
        cls.temp = tempfile.TemporaryDirectory()
        cls.output = Path(cls.temp.name) / "recipe"
        build_recipe(cls.root, cls.output, "d6d46647b24b1a4baa501bca8c1d23048da3e83f")
        cls.collection = tomllib.loads((cls.output / "collection.toml").read_text())
        cls.features = {item["id"]: item for item in cls.collection["features"]}
        cls.runs = {item["run_id"]: item for item in cls.collection["runs"]}
        cls.positions = {item["run_id"]: i for i, item in enumerate(cls.collection["runs"])}
        cls.preset = tomllib.loads((cls.output / "presets/chris-recommended.toml").read_text())["selections"]

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def selected(self, overrides=None):
        state = _effective_features(self.collection, {**self.preset, **(overrides or {})})
        return {(ref["run_id"], ref["component"]) for feature in self.collection["features"]
                if state[feature["id"]] for ref in feature.get("components", [])}

    def test_selected_runs_have_static_evidence_and_tail_approvals(self):
        release = self.output / "releases/v0.1.0-alpha.1"
        evidence = tomllib.loads((release / "acceptance.toml").read_text())["evidence"]
        accepted = {item["subject_id"] for item in evidence
                    if item["subject_kind"] == "run" and item["kind"] == "static-test"
                    and item["status"] == "accepted"}
        selected_runs = {run for run, _ in self.selected()}
        self.assertFalse(selected_runs - accepted)
        approved = set(tomllib.loads((release / "known-limitations.toml").read_text())["approved_tail_runs"])
        selected_tail = {run for run in selected_runs if self.runs[run]["phase"] == "post-eet-end"}
        self.assertFalse(selected_tail - approved)

    def test_classic_lightning_is_default_with_real_exclusive_nonbounce_alternative(self):
        classic, alternative = "feature:spell-rev:component-80", "feature:spell-rev:component-81"
        self.assertIn(("spell-rev-lightning-bg2", 80), self.selected())
        self.assertNotIn(("spell-rev-lightning-bg2", 81), self.selected())
        self.assertEqual(self.features[classic]["choice_group"], self.features[alternative]["choice_group"])
        for chosen, other in ((classic, alternative), (alternative, classic)):
            self.assertIn(other, {rule["feature_id"] for rule in self.features[chosen]["conflicts"]})
        switched = self.selected({classic: "off", alternative: "on"})
        self.assertIn(("spell-rev-lightning-bg2", 81), switched)
        self.assertNotIn(("spell-rev-lightning-bg2", 80), switched)
        self.assertLess(self.positions["spell-rev-npc-spellbooks-bg2"], self.positions["spell-rev-lightning-bg2"])
        self.assertLess(self.positions["spell-rev-lightning-bg2"], self.positions["buffbot-bg2"])

    def test_sr_off_omits_lightning_and_rr_semantic_compatibility(self):
        selected = self.selected({"mod:spell-rev": "off"})
        self.assertFalse(any(run in {"spell-rev-lightning-bg2", "srcb-rr-compat-bg2"} for run, _ in selected))
        self.assertIn(("srcb-rr-compat-bg2", 0), self.selected())
        for earlier in ("spell-rev-core-bg2", "rr-bg2"):
            self.assertLess(self.positions[earlier], self.positions["srcb-rr-compat-bg2"])
        self.assertLess(self.positions["srcb-rr-compat-bg2"], self.positions["stratagems-bg2"])

    def test_either_rr_encounter_selects_compatibility_but_neither_does_not(self):
        for first, second, expected in ((True, True, True), (True, False, True),
                                        (False, True, True), (False, False, False)):
            selected = self.selected({"feature:rr:component-11": "on" if first else "off",
                                      "feature:rr:component-12": "on" if second else "off"})
            self.assertEqual(("srcb-rr-compat-bg2", 0) in selected, expected)

    def test_recipe_requires_app_with_alternative_dependency_support(self):
        ledger = tomllib.loads((self.output / "releases/v0.1.0-alpha.14/ledger.toml").read_text())
        self.assertEqual(ledger["minimum_app_version"], "0.1.0-alpha.16")

    def test_companions_are_prepared_before_continuity_and_eet_end(self):
        pre = "chriz-bg-modpack-pre-continuity-bg2"
        continuity = "chriz-bg-modpack-continuity-bg2"
        self.assertIn((pre, 188), self.selected())
        self.assertIn((continuity, 199), self.selected())
        self.assertEqual(self.positions[continuity] + 1, self.positions["eet-end-bg2"])
        self.assertLess(self.positions[pre], self.positions[continuity])
        self.assertEqual(self.runs[pre]["components"], [110, 140, 170, 188, 190, 192, 193, 194, 195, 196, 197, 198, 220, 221, 222, 223])
        for component in self.runs[pre]["components"]:
            self.assertNotIn(component, self.runs["chriz-bg-modpack-bg2"]["components"])

    def test_vanilla_yeslick_omits_conversion_but_retains_continuity(self):
        selected = self.selected({"feature:yeslicknpc:component-0": "on", "feature:yeslicknpc:component-1": "off"})
        self.assertNotIn(("chriz-bg-modpack-pre-continuity-bg2", 188), selected)
        self.assertIn(("yeslicknpc-bg2", 0), selected)
        self.assertIn(("chriz-bg-modpack-continuity-bg2", 199), selected)

    def test_safana_and_arrival_cleanup_are_default_with_clean_optout(self):
        selected = self.selected()
        self.assertIn(("safana-bg2", 0), selected)
        self.assertIn(("chriz-bg-modpack-late-companions-bg2", 189), selected)
        self.assertIn(("chriz-bg-modpack-late-companions-bg2", 620), selected)
        self.assertLess(self.positions["safana-bg2"], self.positions["chriz-bg-modpack-late-companions-bg2"])
        self.assertLess(self.positions["chriz-sod-remix-bg2"], self.positions["chriz-bg-modpack-late-companions-bg2"])
        self.assertLess(self.positions["chriz-bg-modpack-late-companions-bg2"], self.positions["spell-rev-npc-spellbooks-bg2"])
        opted_out = self.selected({"mod:safana": "off"})
        self.assertEqual(selected - opted_out, {("safana-bg2", 0), ("chriz-bg-modpack-late-companions-bg2", 189)})
        for unimplemented in (100, 150):
            self.assertNotIn(f"feature:chriz-bg-modpack:component-{unimplemented}", self.features)

    def test_sod_new_fresh_components_default_but_challenge_and_repairs_are_not(self):
        selected = self.selected()
        for component in (115, 135, 256, 265, 266):
            self.assertIn(("chriz-sod-remix-bg2", component), selected)
        self.assertNotIn(("chriz-sod-remix-bg2", 257), selected)
        self.assertIn(("chriz-sod-remix-bg2", 257), self.selected({"feature:chriz-sod-remix:component-257": "on"}))
        self.assertFalse(any(run == "chriz-sod-remix-bg2" for run, _ in self.selected({"mod:chriz-sod-remix": "off"})))
        self.assertFalse({176, 235, 291, 901} & set(self.runs["chriz-sod-remix-bg2"]["components"]))
        self.assertLess(self.runs["chriz-sod-remix-bg2"]["components"].index(256), self.runs["chriz-sod-remix-bg2"]["components"].index(257))

    def test_dragons_are_opt_in_and_follow_scs(self):
        for component in (110, 111):
            identity = ("chriz-bg-rebalance-bg2", component)
            self.assertNotIn(identity, self.selected())
            self.assertIn(identity, self.selected({f"feature:chriz-bg-rebalance:component-{component}": "on"}))
            self.assertIn("feature:stratagems:mandatory-components", self.features[f"feature:chriz-bg-rebalance:component-{component}"]["requires"])
        self.assertLess(self.positions["stratagems-bg2"], self.positions["chriz-bg-rebalance-bg2"])
        self.assertEqual(self.collection["runs"][-1]["run_id"], "buffbot-bg2")


if __name__ == "__main__":
    unittest.main()
