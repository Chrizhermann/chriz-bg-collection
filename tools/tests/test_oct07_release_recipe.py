"""Approved October 6 mechanics must actually reach new installations."""
from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import build_recipe, _effective_features


class OctoberReleaseRecipeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.root = Path(__file__).resolve().parents[2]
        cls.temp = tempfile.TemporaryDirectory()
        cls.output = Path(cls.temp.name) / "recipe"
        # Synthetic evidence marker used only in this temporary test output.
        build_recipe(cls.root, cls.output, "0" * 40)
        cls.collection = tomllib.loads((cls.output / "collection.toml").read_text())
        cls.features = {f["id"]: f for f in cls.collection["features"]}
        cls.preset = tomllib.loads((cls.output / "presets/chris-recommended.toml").read_text())["selections"]

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def effective(self, **changes):
        return _effective_features(self.collection, {**self.preset, **changes})

    def test_approved_rebalance_defaults_are_present_and_not_95_cap(self):
        defaults = self.effective()
        for component in (130, 200, 210, 211, 220, 310, 320):
            with self.subTest(component=component):
                self.assertTrue(defaults[f"feature:chriz-bg-rebalance:component-{component}"])
        self.assertNotIn("feature:chriz-bg-rebalance:component-311", self.features)

    def test_hardiness_has_independent_extra_resistance_axis(self):
        a, b, extra = (f"feature:chriz-bg-rebalance:component-{n}" for n in (200, 201, 202))
        self.assertEqual(self.features[a]["choice_group"], self.features[b]["choice_group"])
        self.assertNotIn("choice_group", self.features[extra])
        self.assertEqual(set(self.features[extra]["requires_any"]), {a, b})
        self.assertIn(b, [c["feature_id"] for c in self.features[a]["conflicts"]])
        self.assertIn(a, [c["feature_id"] for c in self.features[b]["conflicts"]])
        self.assertFalse(self.effective()[b])
        self.assertFalse(self.effective()[extra])
        for physical in (a, b):
            selected = self.effective(**{a: "off", b: "off", physical: "on", extra: "on"})
            self.assertTrue(selected[physical])
            self.assertTrue(selected[extra])
        self.assertFalse(self.effective(**{a: "off", b: "off", extra: "on"})[extra])

    def test_disabled_prerequisites_stay_disabled(self):
        for disabled in ("mod:spell-rev", "feature:spell-rev:component-65", "mod:chriz-bg-rebalance"):
            result = self.effective(**{disabled: "off", "feature:chriz-bg-rebalance:component-202": "on"})
            for component in (200, 201, 202):
                self.assertFalse(result[f"feature:chriz-bg-rebalance:component-{component}"])
        no_eeex = self.effective(**{"mod:eeex": "off"})
        for component in (210, 310):
            self.assertFalse(no_eeex[f"feature:chriz-bg-rebalance:component-{component}"])
        self.assertTrue(no_eeex["feature:chriz-bg-rebalance:component-211"])
        no_sr = self.effective(**{"mod:spell-rev": "off"})
        for component in (210, 211, 220):
            self.assertFalse(no_sr[f"feature:chriz-bg-rebalance:component-{component}"])

    def test_companion_fixes_are_ordered_and_baeloth_stays_optional(self):
        positions = {r["run_id"]: i for i, r in enumerate(self.collection["runs"])}
        baeloth = "feature:chriz-bg-modpack:component-235"
        edwin = "feature:chriz-bg-modpack:component-236"
        self.assertFalse(self.effective()[baeloth])
        self.assertTrue(self.effective(**{baeloth: "on"})[baeloth])
        self.assertTrue(self.effective()[edwin])
        self.assertFalse(self.effective(**{"feature:artisanskitpack-npc:component-5102": "off"})[edwin])
        self.assertLess(positions["artisanskitpack-npc-bg2"], positions["chriz-bg-modpack-edwin-bg2"])
        self.assertLess(positions["chriz-bg-modpack-edwin-bg2"], positions["spell-rev-npc-spellbooks-bg2"])
        self.assertGreater(positions["chriz-bg-modpack-baeloth-bg2"], positions["spell-rev-npc-spellbooks-bg2"])

    def test_new_components_have_release_coverage_without_promising_hotpatches(self):
        ledger = tomllib.loads((self.output / "releases/v0.1.0-alpha.19/ledger.toml").read_text())
        self.assertEqual(ledger["supersedes"], "0.1.0-alpha.18")
        self.assertEqual(ledger["minimum_app_version"], "0.1.0-alpha.19")
        covered = {address for change in ledger["changes"] for address in change["covers"]}
        for mod, components in {
            "chriz-bg-rebalance": (130, 200, 201, 202, 210, 211, 220, 310, 320),
            "chriz-bg-modpack": (235, 236),
        }.items():
            for component in components:
                self.assertIn(f"feature:feature:{mod}:component-{component}", covered)
        self.assertEqual({change["save_applicability"] for change in ledger["changes"]},
                         {"next-playthrough"})

    def test_published_intake_matches_current_pins_and_native_order(self):
        evidence = tomllib.loads((self.root / "manifest/evidence/release-intake-20261007.toml").read_text())
        self.assertEqual(evidence["verified_on"], "2026-10-07")
        self.assertEqual({record["mod_id"] for record in evidence["artifacts"]},
                         {"chriz-bg-rebalance", "chriz-bg-modpack", "buffbot"})
        # The October 13 published intake supersedes these pins (test_oct13_intake_recipe),
        # and the October 23 intake (test_oct23_buffbot_eeexremote_recipe) the BuffBot pin;
        # their pinned files stay exactly as verified here.
        superseded = {"chriz-bg-rebalance", "chriz-bg-modpack", "buffbot"}
        for profile in (self.root / "manifest", self.output):
            collection = tomllib.loads((profile / "collection.toml").read_text())
            for record in evidence["artifacts"]:
                with self.subTest(profile=str(profile), mod=record["mod_id"]):
                    self.assertTrue(record["fresh_download"])
                    self.assertTrue(record["published_release_verified"])
                    self.assertTrue(record["native_component_order_verified"])
                    artifact = tomllib.loads((profile / f"artifacts/{record['artifact_id']}.toml").read_text())
                    for key, measured in (("sha256", "actual_sha256"), ("expected_length", "actual_length"),
                                          ("url", "immutable_source_url"), ("reference", "source_reference")):
                        self.assertEqual(artifact["source"][key], record[measured])
                    mod = tomllib.loads((profile / f"mods/{record['mod_id']}.toml").read_text())
                    if record["mod_id"] in superseded:
                        self.assertNotEqual(mod["artifact_id"], record["artifact_id"])
                        continue
                    self.assertEqual(mod["artifact_id"], record["artifact_id"])
                    for run in collection["runs"]:
                        if run["mod_id"] == record["mod_id"]:
                            self.assertEqual(run["components"], [c for c in record["native_components"]
                                                                 if c in run["components"]])


if __name__ == "__main__":
    unittest.main()
