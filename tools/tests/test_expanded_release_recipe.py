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
        approved_list = tomllib.loads((release / "known-limitations.toml").read_text())["approved_tail_runs"]
        approved = set(approved_list)
        selected_tail = {run for run in selected_runs if self.runs[run]["phase"] == "post-eet-end"}
        self.assertFalse(selected_tail - approved)
        # Release validation rejects duplicate identities, which set checks above cannot see.
        identities = [(item["subject_kind"], item["subject_id"], item["kind"]) for item in evidence]
        self.assertEqual(len(identities), len(set(identities)))
        self.assertEqual(len(approved_list), len(approved))

    def test_eet_desktop_shortcut_is_an_unchecked_option(self):
        # EET 100 starts Baldur.exe directly (no EEex) and replaces any Desktop EET.lnk.
        feature_id, run_id = "feature:eet:component-100", "eet-initialize-bg2"
        feature = self.features[feature_id]
        self.assertEqual(feature["decision"], "optional")
        self.assertEqual(feature["readiness"], "ready")
        self.assertEqual(feature["components"], [{"run_id": run_id, "component": 100}])
        self.assertEqual(self.preset[feature_id], "off")
        self.assertEqual(self.runs[run_id]["components"], [0, 100])
        baseline = self.selected()
        self.assertIn((run_id, 0), baseline)
        self.assertNotIn((run_id, 100), baseline)
        opted_in = self.selected({feature_id: "on"})
        self.assertEqual(opted_in - baseline, {(run_id, 100)})
        self.assertEqual(baseline - opted_in, set())

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
        ledger = tomllib.loads((self.output / "releases/v0.1.0-alpha.16/ledger.toml").read_text())
        self.assertEqual(ledger["minimum_app_version"], "0.1.0-alpha.18")

    def test_reviewed_modpack_compatibility_release_preserves_selection(self):
        mod = tomllib.loads((self.output / "mods/chriz-bg-modpack.toml").read_text())
        self.assertEqual(mod["artifact_id"], "chriz-bg-modpack-0.2.0-alpha.14")
        artifact = tomllib.loads((self.output / "artifacts/chriz-bg-modpack-0.2.0-alpha.14.toml").read_text())
        self.assertEqual(artifact["source"]["reference"], "v0.2.0-alpha.14")
        self.assertEqual(artifact["source"]["expected_length"], 10550312)
        self.assertEqual(artifact["source"]["sha256"], "45c767b21342d1c7c23ded6cd52aef29b4054daf9cc949613d8f6620207a356c")
        self.assertFalse((self.output / "artifacts/chriz-bg-modpack-0.2.0-alpha.6.toml").exists())
        # Alpha.18 adds Rebalance 301/302 after alpha.17's interval-save controls;
        # Modpack also restores Evandra defaults and switches 610 to 611.
        selected = self.selected()
        interval_controls = "feature:hiddengameplayoptions:component-38"
        without_controls = self.selected({interval_controls: "off"})
        self.assertEqual(self.features[interval_controls]["decision"], "default")
        self.assertEqual(self.preset[interval_controls], "on")
        self.assertEqual(selected - without_controls, {("hiddengameplayoptions-bg2", 38)})
        self.assertEqual(without_controls - selected, set())
        # October 7 adds seven Rebalance defaults and Edwin236's dedicated run;
        # the next-release policy removes only Emotion301 from normal defaults.
        # October 13 default bard ceilings add 420/421 and Bardic 3010's own run;
        # EET's desktop shortcut (EET 100) becomes an unchecked option.
        # The October 13 intake adds default Modpack 237/238/660 and mandatory SoD
        # 177/198/285, and moves 430 into its own final run.
        # The October 14 intake adds default Modpack 230-234, Rebalance 331/332,
        # SCS 3021 and Tweaks 3220/3500/3510.
        # October 16 makes all 18 previously default UB components optional.
        # The October 20 intake (Modpack alpha.13) adds default EET join XP 239.
        # The October 22 intake adds 18 default player voicepacks, one run each.
        # The October 23 intake (SoD Remix v0.6.15) adds default Imoen choice 292 to the SoD run.
        # The October 23 quest-mod intake makes BG1UB optional and unselected, which removes
        # its 16 former defaults and the bg1ub-bg1 run; the new quest mods are unchecked.
        # The October 26 intake adds default Chris Revisions 340 to the Rebalance run;
        # EEex Attack Info 9 is offered unchecked. Its Modpack alpha.14 pin adds default
        # 135 and 460 to the main post-EET_end Modpack run; chapter caps 652 stay unchecked.
        # October 27: Tweaks 2292 (druids level on the cleric XP table) is no longer a default.
        self.assertEqual(len(without_controls), 467)
        self.assertEqual(len(selected), 468)
        self.assertEqual(len({run for run, _ in selected}), 69)
        self.assertIn(("cdtweaks-bg2", 3354), selected)
        controls_ledger = tomllib.loads((self.output / "releases/v0.1.0-alpha.17/ledger.toml").read_text())
        controls_change = next(change for change in controls_ledger["changes"]
                               if change["id"] == "interval-save-controls-default")
        self.assertEqual(controls_change["covers"], [f"feature:{interval_controls}"])
        for component in (140, 170):
            self.assertIn(("chriz-bg-modpack-pre-continuity-bg2", component), self.selected())
        self.assertIn(("chriz-bg-modpack-bg2", 410), self.selected())
        ledger = tomllib.loads((self.output / "releases/v0.1.0-alpha.16/ledger.toml").read_text())
        fix = next(change for change in ledger["changes"] if change["id"] == "modpack-compatible-presets")
        self.assertEqual(fix["save_applicability"], "new-game-only")
        self.assertIn("artifact:chriz-bg-modpack-0.2.0-alpha.7", fix["covers"])

    def test_companions_are_prepared_before_continuity_and_eet_end(self):
        pre = "chriz-bg-modpack-pre-continuity-bg2"
        continuity = "chriz-bg-modpack-continuity-bg2"
        self.assertIn((pre, 188), self.selected())
        self.assertIn((continuity, 199), self.selected())
        self.assertEqual(self.positions[continuity] + 1, self.positions["eet-end-bg2"])
        self.assertLess(self.positions[pre], self.positions[continuity])
        # Modpack alpha.12 companion stats 230-234 follow the class/kit conversions.
        self.assertEqual(self.runs[pre]["components"], [110, 140, 170, 188, 190, 192, 193, 194, 195, 196, 197, 198, 220, 221, 222, 223, 224, 225, 230, 231, 232, 233, 234])
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

    def test_wing_buffet_spacing_is_default_and_deselectable_while_lethal_melee_is_optional(self):
        run_id = "chriz-bg-rebalance-bg2"
        wing_buffet = "feature:chriz-bg-rebalance:component-111"
        lethal_melee = "feature:chriz-bg-rebalance:component-110"
        selected = self.selected()
        self.assertEqual(self.features[wing_buffet]["decision"], "default")
        self.assertEqual(self.features[wing_buffet]["readiness"], "ready")
        self.assertEqual(self.preset[wing_buffet], "on")
        self.assertIn((run_id, 111), selected)
        opted_out = self.selected({wing_buffet: "off"})
        self.assertEqual(selected - opted_out, {(run_id, 111)})
        self.assertEqual(opted_out - selected, set())
        self.assertEqual(self.features[lethal_melee]["decision"], "optional")
        self.assertEqual(self.preset.get(lethal_melee, "off"), "off")
        self.assertNotIn((run_id, 110), selected)
        self.assertIn((run_id, 110), self.selected({lethal_melee: "on"}))
        self.assertEqual(self.features[wing_buffet]["requires"], ["feature:stratagems:mandatory-components"])
        self.assertIn("feature:stratagems:mandatory-components", self.features[lethal_melee]["requires"])
        self.assertNotIn((run_id, 111), self.selected({"mod:stratagems": "off"}))
        self.assertLess(self.positions["stratagems-bg2"], self.positions["chriz-bg-rebalance-bg2"])
        # The scroll policy 430 hashes final scrolls and spells, so it follows BuffBot.
        self.assertEqual(self.collection["runs"][-2]["run_id"], "buffbot-bg2")
        self.assertEqual(self.collection["runs"][-1]["run_id"], "chriz-bg-modpack-scrolls-bg2")

    def test_emotion_exclusivity_is_optional_with_an_independent_optin(self):
        feature_id = "feature:chriz-bg-rebalance:component-301"
        run_id = "chriz-bg-rebalance-bg2"
        feature = self.features[feature_id]
        self.assertEqual(feature["decision"], "optional")
        self.assertEqual(feature["readiness"], "ready")
        self.assertEqual(feature["parent"], "mod:chriz-bg-rebalance")
        self.assertEqual(feature.get("requires", []), [])
        self.assertEqual(feature.get("requires_any", []), [])
        self.assertEqual(feature["components"], [{"run_id": run_id, "component": 301}])
        self.assertEqual(self.preset[feature_id], "off")

        baseline = self.selected()
        self.assertNotIn((run_id, 301), baseline)
        opted_in = self.selected({feature_id: "on"})
        self.assertEqual(opted_in - baseline, {(run_id, 301)})
        self.assertEqual(baseline - opted_in, set())
        # October 27: Tweaks 2292 is no longer selected by default.
        self.assertEqual(len(baseline), 468)
        self.assertEqual(self.selected({feature_id: "off"}), baseline)
        baeloth = "feature:chriz-bg-modpack:component-235"
        self.assertEqual(self.features[baeloth]["decision"], "optional")
        self.assertEqual(self.preset.get(baeloth, "off"), "off")
        self.assertNotIn(("chriz-bg-modpack-baeloth-bg2", 235), baseline)
        for component in (111, 130, 200, 210, 211, 220, 302, 310, 320):
            self.assertIn((run_id, component), baseline)
        parent_off = self.selected({"mod:chriz-bg-rebalance": "off"})
        self.assertFalse(any(run == run_id for run, _ in parent_off))
        # Keep 301 selectable for either provider or standalone spell handling;
        # this checks recipe eligibility, not the installed spell behavior.
        for provider in ("mod:spell-rev", "mod:iwdification", "mod:stratagems"):
            with self.subTest(provider=provider):
                self.assertIn(provider, self.features)
                self.assertIn((run_id, 301), self.selected({provider: "off", feature_id: "on"}))

    def test_emotion_run_keeps_source_order_including_default_bard_ceilings(self):
        run_id = "chriz-bg-rebalance-bg2"
        self.assertEqual(self.runs[run_id]["components"],
                         [100, 101, 110, 111, 120, 121, 200, 201, 202, 210, 211, 301, 400, 401, 404, 405, 407, 408, 302, 420, 421, 310, 130, 220, 320, 330, 331, 332, 340])
        for component in (420, 421):
            self.assertNotIn(f"feature:chriz-bg-rebalance:component-{component}", self.features)
            self.assertIn((run_id, component), self.selected())
        for provider in ("iwdification-bg2", "stratagems-bg2", "spell-rev-core-bg2"):
            self.assertLess(self.positions[provider], self.positions[run_id])
        for later in ("cdtweaks-spell-save-penalties-bg2", "chriz-bg-modpack-bg2"):
            self.assertLess(self.positions[run_id], self.positions[later])
        selected = self.selected()
        for identity in (("iwdification-bg2", 30), ("iwdification-bg2", 40),
                         ("stratagems-bg2", 5900), ("spell-rev-core-bg2", 0),
                         ("cdtweaks-spell-save-penalties-bg2", 2312),
                         ("chriz-bg-modpack-scrolls-bg2", 430)):
            self.assertIn(identity, selected)

    def test_bard_spell_ceilings_are_default_and_gate_the_bardic_adapter(self):
        base = "feature:chriz-bg-rebalance:bard-spell-ceilings"
        adapter = "feature:bardicwonders:component-3010"
        rebalance_run = "chriz-bg-rebalance-bg2"
        bardic_run = "bardicwonders-progression-bg2"
        # Default (Christopher's decision, 2026-10-13): both checked and deselectable.
        for feature_id in (base, adapter):
            self.assertEqual(self.features[feature_id]["decision"], "default")
            self.assertEqual(self.features[feature_id]["readiness"], "ready")
            self.assertEqual(self.features[feature_id]["category"], "kits")
            self.assertEqual(self.preset[feature_id], "on")
        self.assertEqual(self.features[base]["parent"], "mod:chriz-bg-rebalance")
        self.assertEqual(self.features[base]["requires"], ["feature:eeex:mandatory-components"])
        self.assertEqual(self.features[base]["components"], [
            {"run_id": rebalance_run, "component": 420},
            {"run_id": rebalance_run, "component": 421},
        ])
        self.assertEqual(self.features[adapter]["parent"], "mod:bardicwonders")
        self.assertEqual(self.features[adapter]["requires"],
                         [base, "feature:eeex:mandatory-components"])
        self.assertEqual(self.features[adapter]["components"],
                         [{"run_id": bardic_run, "component": 3010}])

        # 420 precedes 421 in the existing Rebalance run; 3010 has its own late run
        # immediately after it, behind every other Bardic Wonders run.
        components = self.runs[rebalance_run]["components"]
        self.assertLess(components.index(302), components.index(420))
        self.assertLess(components.index(420), components.index(421))
        self.assertLess(components.index(421), components.index(310))
        self.assertEqual(self.runs[bardic_run]["phase"], "post-eet-end")
        self.assertEqual(self.runs[bardic_run]["mod_id"], "bardicwonders")
        self.assertEqual(self.runs[bardic_run]["components"], [3010])
        self.assertEqual(self.positions[bardic_run], self.positions[rebalance_run] + 1)
        for earlier in ("bardicwonders-bg2", "bardicwonders-garrick-bg2",
                        "bardicwonders-dialogue-patch-bg2", "cdtweaks-bg2", "eeex-bg2"):
            self.assertLess(self.positions[earlier], self.positions[bardic_run])

        both = self.selected()
        ceilings = {(rebalance_run, 420), (rebalance_run, 421)}
        adapter_identity = {(bardic_run, 3010)}
        self.assertTrue(ceilings | adapter_identity <= both)
        # Opting out of both removes exactly 420/421/3010 and nothing else.
        neither = self.selected({base: "off", adapter: "off"})
        self.assertEqual(both - neither, ceilings | adapter_identity)
        self.assertEqual(neither - both, set())
        # Opting out of only the adapter keeps exactly 420/421.
        base_only = self.selected({adapter: "off"})
        self.assertEqual(base_only - neither, ceilings)
        self.assertEqual(both - base_only, adapter_identity)
        # A stale adapter selection without the base choice installs nothing extra.
        self.assertEqual(self.selected({base: "off"}), neither)
        # 3010 is never installed without 420/421, EEex or Bardic Wonders.
        for missing in ({base: "off"}, {"mod:chriz-bg-rebalance": "off"}, {"mod:eeex": "off"},
                        {"mod:bardicwonders": "off"}):
            with self.subTest(missing=missing):
                selected = self.selected({base: "on", adapter: "on", **missing})
                self.assertNotIn((bardic_run, 3010), selected)
        # Deselecting Bardic Wonders leaves the base ceilings valid.
        no_bardic = self.selected({base: "on", adapter: "on", "mod:bardicwonders": "off"})
        self.assertTrue(ceilings <= no_bardic)
        self.assertFalse(any(run.startswith("bardicwonders-") for run, _ in no_bardic))
        self.assertFalse(ceilings & self.selected({base: "on", "mod:eeex": "off"}))


if __name__ == "__main__":
    unittest.main()
