"""October 13 next-release intake: Modpack v0.2.0-alpha.11 and SoD Remix v0.6.13 rows.

Static recipe, selection, dependency and order checks for both recipe profiles only.
The artifact pins follow the October 13 publications (alpha.11 / v0.6.13); nothing
here is a game installation or gameplay acceptance.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import NATIVE_RUN_ORDERS, _effective_features, build_recipe
from tools.curation_audit import Decision, RowKey, load_catalogs


ROOT = Path(__file__).resolve().parents[2]
MODPACK = "feature:chriz-bg-modpack:component-{}"
SCROLLS_RUN = "chriz-bg-modpack-scrolls-bg2"
CAMPAIGN_RUN = "chriz-bg-modpack-campaign-xp-bg2"
LATE_RUN = "chriz-bg-modpack-late-companions-bg2"
SOD_RUN = "chriz-sod-remix-bg2"
SOD_CORE = "feature:chriz-sod-remix:mandatory-components"
EEEX = "feature:eeex:mandatory-components"

# Native DESIGNATED order of the candidates' TP2 files, restricted to offered rows.
MODPACK_ALPHA11_ORDER = [
    110, 130, 140, 170, 188, 189, 190, 192, 193, 194, 195, 196, 197, 198, 199,
    220, 221, 222, 223, 224, 225, 235, 236, 237, 238, 400, 410, 430, 440, 450,
    610, 611, 620, 630, 640, 660, 650, 651,
]
SOD_V0613_RUN = [
    100, 110, 115, 120, 130, 135, 140, 150, 145, 160, 170, 180, 175, 177, 185, 190,
    195, 210, 197, 198, 187, 200, 215, 220, 225, 245, 230, 240, 250, 255, 256, 257,
    260, 265, 266, 270, 280, 285, 290, 900, 910,
]


def _load_profile(directory: Path) -> tuple[dict, dict]:
    collection = tomllib.loads((directory / "collection.toml").read_text(encoding="utf-8"))
    preset = tomllib.loads((directory / "presets/chris-recommended.toml").read_text(encoding="utf-8"))["selections"]
    return collection, preset


class Oct13IntakeTests(unittest.TestCase):
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

    def test_modpack_runs_follow_the_alpha11_tp2_and_the_scroll_policy_runs_last(self):
        for _, _, collection, _ in self._each_profile():
            runs = {run["run_id"]: run for run in collection["runs"]}
            order = [run["run_id"] for run in collection["runs"]]
            self.assertEqual(runs["chriz-bg-modpack-bg2"]["components"], [130, 400, 410, 440, 450, 610, 611, 630, 660])
            self.assertEqual(runs[LATE_RUN]["components"], [189, 237, 238, 620])
            self.assertEqual(runs[CAMPAIGN_RUN]["components"], [650, 651])
            self.assertEqual(runs[SCROLLS_RUN]["components"], [430])
            for run_id in ("chriz-bg-modpack-bg2", LATE_RUN, CAMPAIGN_RUN, SCROLLS_RUN):
                self.assertEqual(runs[run_id]["mod_id"], "chriz-bg-modpack")
                self.assertEqual(runs[run_id]["phase"], "post-eet-end")
                components = runs[run_id]["components"]
                self.assertEqual(components, [c for c in MODPACK_ALPHA11_ORDER if c in components])
            self.assertEqual(order.index(CAMPAIGN_RUN), order.index(LATE_RUN) + 1)
            # 430 hashes every scroll and the spells they cast at install time.
            self.assertEqual(order[-1], SCROLLS_RUN)
            self.assertEqual(order[-2], "buffbot-bg2")
            for writer in ("cdtweaks-spell-save-penalties-bg2", "spell-rev-lightning-bg2", "chriz-bg-modpack-baeloth-bg2"):
                if writer in order:
                    self.assertLess(order.index(writer), order.index(SCROLLS_RUN))
            occurrences = [
                (run["run_id"], component)
                for run in collection["runs"] if run["mod_id"] == "chriz-bg-modpack"
                for component in run["components"]
            ]
            self.assertEqual(len(occurrences), len({c for _, c in occurrences}))
        for run_id, native in NATIVE_RUN_ORDERS.items():
            if run_id.startswith("chriz-bg-modpack-") and run_id not in {"chriz-bg-modpack-pre-continuity-bg2"}:
                self.assertEqual(native, [c for c in MODPACK_ALPHA11_ORDER if c in native], run_id)

    def test_reader_level_scrolls_are_a_deselectable_eeex_default(self):
        for _, _, collection, preset in self._each_profile():
            feature = next(f for f in collection["features"] if f["id"] == MODPACK.format(430))
            self.assertEqual(feature["title"], "Scrolls cast at the reader's own caster level")
            self.assertEqual(
                feature["description"],
                "Arcane and divine scrolls use the reader's caster level of that type; other readers, "
                "such as Use Any Item users, use half their highest class level, minimum 1. Requires EEex.",
            )
            self.assertEqual((feature["category"], feature["decision"], feature["readiness"]), ("rules", "default", "ready"))
            self.assertEqual(feature["parent"], "mod:chriz-bg-modpack")
            self.assertEqual(feature["requires"], [EEEX])
            self.assertNotIn("unavailable_reason", feature)
            self.assertEqual(feature["components"], [{"run_id": SCROLLS_RUN, "component": 430}])
            baseline = self._selected(collection, preset)
            self.assertIn((SCROLLS_RUN, 430), baseline)
            self.assertEqual(baseline - self._selected(collection, preset, **{MODPACK.format(430): "off"}), {(SCROLLS_RUN, 430)})
            self.assertNotIn((SCROLLS_RUN, 430), self._selected(collection, preset, **{"mod:eeex": "off"}))

    def test_fade_recruitment_xp_is_default_with_fade(self):
        for _, _, collection, preset in self._each_profile():
            feature = next(f for f in collection["features"] if f["id"] == MODPACK.format(237))
            self.assertEqual(feature["title"], "Raise Fade's XP when she first joins in Shadows of Amn")
            self.assertEqual((feature["category"], feature["decision"], feature["readiness"]), ("npcs", "default", "ready"))
            self.assertEqual(feature["requires"], ["feature:fade:mandatory-components"])
            self.assertIn((LATE_RUN, 237), self._selected(collection, preset))
            self.assertNotIn((LATE_RUN, 237), self._selected(collection, preset, **{"mod:fade": "off"}))

    def test_evandra_recruitment_xp_is_default_only_where_evandra_exists(self):
        _, curated, curated_preset = self.profiles["curated"]
        feature = next(f for f in curated["features"] if f["id"] == MODPACK.format(238))
        self.assertEqual(feature["title"], "Raise Evandra's XP when she first joins in Shadows of Amn")
        self.assertEqual((feature["category"], feature["decision"], feature["readiness"]), ("npcs", "default", "ready"))
        self.assertEqual(feature["requires"], ["feature:evandra:mandatory-components"])
        self.assertNotIn("unavailable_reason", feature)
        self.assertEqual(curated_preset[MODPACK.format(238)], "on")
        self.assertIn((LATE_RUN, 238), self._selected(curated, curated_preset))
        self.assertNotIn((LATE_RUN, 238), self._selected(curated, curated_preset, **{"mod:evandra": "off"}))
        # The narrower fallback has no Evandra core: optional and blocked, so it
        # needs no omission record in the fallback release gate.
        _, fallback, fallback_preset = self.profiles["fallback"]
        feature = next(f for f in fallback["features"] if f["id"] == MODPACK.format(238))
        self.assertEqual((feature["decision"], feature["readiness"]), ("optional", "blocked"))
        self.assertEqual(feature["requires"], ["mod:evandra"])
        self.assertTrue(feature["unavailable_reason"])
        self.assertNotIn((LATE_RUN, 238), self._selected(fallback, fallback_preset))

    def test_campaign_xp_caps_are_optional_experimental_and_unchecked(self):
        cap, soa = MODPACK.format(650), MODPACK.format(651)
        for _, _, collection, preset in self._each_profile():
            features = {f["id"]: f for f in collection["features"]}
            self.assertEqual(features[cap]["title"], "Campaign XP caps with banked XP (experimental)")
            self.assertEqual(features[soa]["title"], "Also cap Shadows of Amn at 2,950,000 XP (experimental)")
            for feature_id in (cap, soa):
                self.assertEqual((features[feature_id]["category"], features[feature_id]["decision"], features[feature_id]["readiness"]), ("rules", "optional", "experimental"))
                self.assertNotEqual(preset.get(feature_id, "off"), "on")
            self.assertEqual(features[cap]["requires"], [EEEX])
            self.assertEqual(features[soa]["requires"], [cap])
            baseline = self._selected(collection, preset)
            self.assertFalse({(CAMPAIGN_RUN, 650), (CAMPAIGN_RUN, 651)} & baseline)
            self.assertEqual(self._selected(collection, preset, **{cap: "on"}) - baseline, {(CAMPAIGN_RUN, 650)})
            self.assertEqual(self._selected(collection, preset, **{soa: "on"}), baseline)
            self.assertEqual(self._selected(collection, preset, **{cap: "on", soa: "on"}) - baseline, {(CAMPAIGN_RUN, 650), (CAMPAIGN_RUN, 651)})
            self.assertEqual(self._selected(collection, preset, **{cap: "on", soa: "on", "mod:eeex": "off"}) & {(CAMPAIGN_RUN, 650), (CAMPAIGN_RUN, 651)}, set())

    def test_degrodel_choice_is_a_default_quest_change(self):
        for _, _, collection, preset in self._each_profile():
            feature = next(f for f in collection["features"] if f["id"] == MODPACK.format(660))
            self.assertEqual(feature["title"], "Degrodel: refuse to hand over the Helm of Balduran and fight him")
            self.assertEqual((feature["category"], feature["decision"], feature["readiness"]), ("quests", "default", "ready"))
            self.assertEqual(feature["parent"], "mod:chriz-bg-modpack")
            self.assertEqual(feature.get("requires", []), [])
            baseline = self._selected(collection, preset)
            self.assertIn(("chriz-bg-modpack-bg2", 660), baseline)
            self.assertEqual(baseline - self._selected(collection, preset, **{MODPACK.format(660): "off"}), {("chriz-bg-modpack-bg2", 660)})

    def test_sod_remix_new_start_rows_are_mandatory_in_tp2_order(self):
        for _, _, collection, preset in self._each_profile():
            runs = {run["run_id"]: run for run in collection["runs"]}
            features = {f["id"]: f for f in collection["features"]}
            self.assertEqual(runs[SOD_RUN]["components"], SOD_V0613_RUN)
            self.assertEqual([ref["component"] for ref in features[SOD_CORE]["components"]], [c for c in SOD_V0613_RUN if c != 257])
            self.assertFalse({176, 291, 901} & set(runs[SOD_RUN]["components"]))
            selected = self._selected(collection, preset)
            for component in (177, 198, 285):
                self.assertIn((SOD_RUN, component), selected)
            self.assertFalse(any(run == SOD_RUN for run, _ in self._selected(collection, preset, **{"mod:chriz-sod-remix": "off"})))
            bridge = features["feature:chriz-sod-remix:component-257"]
            self.assertEqual(
                bridge["description"],
                "Add the implemented harder sequencer behaviour to the revised SoD bridge encounter on Insane. "
                "Optional and unchecked; the normal revised bridge is already included.",
            )
            self.assertEqual(bridge["decision"], "optional")
            group = next(g for g in collection["feature_groups"] if g["id"] == "challenge-mode")
            self.assertEqual(group["members"], [
                "feature:chriz-bg-rebalance:component-301",
                "feature:chriz-bg-modpack:component-235",
                "feature:chriz-bg-rebalance:component-110",
                "feature:chriz-sod-remix:component-257",
            ])

    def test_release_gate_records_cover_only_what_the_default_selection_installs(self):
        for name, directory, collection, preset in self._each_profile():
            release = directory / "releases/v0.1.0-alpha.1"
            limitations = tomllib.loads((release / "known-limitations.toml").read_text(encoding="utf-8"))
            approved = limitations["approved_tail_runs"]
            self.assertIn(SCROLLS_RUN, approved)
            self.assertEqual(len(approved), len(set(approved)))
            # Optional campaign caps are not selected by default, so the gate needs nothing for them.
            self.assertNotIn(CAMPAIGN_RUN, {run for run, _ in self._selected(collection, preset)})
            omitted = {omission["feature_id"] for omission in limitations.get("omissions", [])}
            self.assertNotIn(MODPACK.format(430), omitted)
            self.assertNotIn(MODPACK.format(238), omitted)

    def test_catalog_rows_record_the_intake_decisions(self):
        rows = {RowKey(row.catalog, row.component_id): row for row in load_catalogs(ROOT / "docs/curation/components")}
        expected = {
            RowKey("CHRIZ-BG-MODPACK", 237): Decision.DEFAULT,
            RowKey("CHRIZ-BG-MODPACK", 238): Decision.DEFAULT,
            RowKey("CHRIZ-BG-MODPACK", 430): Decision.DEFAULT,
            RowKey("CHRIZ-BG-MODPACK", 650): Decision.OPTIONAL,
            RowKey("CHRIZ-BG-MODPACK", 651): Decision.OPTIONAL,
            RowKey("CHRIZ-BG-MODPACK", 660): Decision.DEFAULT,
            RowKey("CHRIZ-SOD-REMIX", 177): Decision.MANDATORY,
            RowKey("CHRIZ-SOD-REMIX", 198): Decision.MANDATORY,
            RowKey("CHRIZ-SOD-REMIX", 285): Decision.MANDATORY,
        }
        for key, decision in expected.items():
            self.assertEqual(rows[key].decision, decision, key)
        mod = tomllib.loads((self.output / "mods/chriz-bg-modpack.toml").read_text(encoding="utf-8"))
        names = {component["id"]: component["name"] for component in mod["components"]}
        self.assertEqual(names[430], "Scrolls: use the reader's active caster level, or half their active class level")
        self.assertTrue({237, 238, 650, 651, 660} <= names.keys())
        # The owners published both releases on October 13; the pins follow them.
        self.assertEqual(mod["artifact_id"], "chriz-bg-modpack-0.2.0-alpha.11")
        sod = tomllib.loads((self.output / "mods/chriz-sod-remix.toml").read_text(encoding="utf-8"))
        self.assertEqual(sod["artifact_id"], "chriz-sod-remix-0.6.13")
        self.assertTrue({177, 198, 285} <= {component["id"] for component in sod["components"]})

    def test_alpha20_ledger_covers_the_intake_for_new_installations(self):
        authored = ROOT / "manifest/releases/v0.1.0-alpha.20/ledger.toml"
        generated = self.output / "releases/v0.1.0-alpha.20/ledger.toml"
        self.assertEqual(generated.read_bytes(), authored.read_bytes())
        ledger = tomllib.loads(authored.read_text(encoding="utf-8"))
        self.assertEqual(ledger["version"], "0.1.0-alpha.20")
        self.assertEqual(ledger["supersedes"], "0.1.0-alpha.19")
        # The recipe adds [[feature_groups]] (Challenge Mode), which alpha.19 apps cannot read.
        self.assertEqual(ledger["minimum_app_version"], "0.1.0-alpha.20")
        collection = self.profiles["curated"][1]
        self.assertTrue(collection["feature_groups"])
        covered = {address for change in ledger["changes"] for address in change["covers"]}
        for feature in (
            "chriz-bg-modpack:component-237", "chriz-bg-modpack:component-238",
            "chriz-bg-modpack:component-430", "chriz-bg-modpack:component-650",
            "chriz-bg-modpack:component-651", "chriz-bg-modpack:component-660",
            "chriz-bg-rebalance:bard-spell-ceilings", "chriz-bg-rebalance:component-301",
            "bardicwonders:component-3010", "eet:component-100",
            "chriz-sod-remix:mandatory-components",
        ):
            self.assertIn(f"feature:feature:{feature}", covered)
        for run in (SCROLLS_RUN, CAMPAIGN_RUN, LATE_RUN, SOD_RUN, "bardicwonders-progression-bg2"):
            self.assertIn(f"run:{run}", covered)
        evidence = tomllib.loads((ROOT / "manifest/evidence/release-intake-20261013.toml").read_text(encoding="utf-8"))
        for record in evidence["artifacts"]:
            self.assertIn(f"artifact:{record['artifact_id']}", covered)
        # Fresh-install recipe changes only; existing games and saves are not patched.
        self.assertEqual({change["save_applicability"] for change in ledger["changes"]}, {"next-playthrough"})
        banking = next(change for change in ledger["changes"] if change["id"] == "campaign-xp-banking-experimental")
        self.assertEqual(banking["urgency"], "optional")

    def test_published_intake_matches_current_pins_and_native_order(self):
        evidence = tomllib.loads((ROOT / "manifest/evidence/release-intake-20261013.toml").read_text(encoding="utf-8"))
        self.assertEqual(evidence["verified_on"], "2026-10-13")
        self.assertEqual({record["mod_id"] for record in evidence["artifacts"]},
                         {"chriz-bg-modpack", "chriz-sod-remix", "chriz-bg-rebalance", "bardicwonders"})
        # The checked-in curated recipe and a fresh generation must both carry the pins.
        directories = {
            "manifest": [ROOT / "manifest"],
            "recipes/curated-full-current": [ROOT / "recipes/curated-full-current", self.output],
        }
        for record in evidence["artifacts"]:
            for flag in ("fresh_download", "fresh_extraction", "published_release_verified",
                         "native_component_order_verified"):
                self.assertIs(record[flag], True, (record["mod_id"], flag))
            self.assertTrue(record["recipe_profiles"])
            for name, profiles in directories.items():
                for profile in profiles:
                    with self.subTest(profile=str(profile), mod=record["mod_id"]):
                        mod_path = profile / f"mods/{record['mod_id']}.toml"
                        if name not in record["recipe_profiles"]:
                            # A curated-only fork never leaks into the fallback recipe.
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


if __name__ == "__main__":
    unittest.main()
