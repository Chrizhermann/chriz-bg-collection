"""October 20 intake: Modpack v0.2.0-alpha.13 and EET join XP 239.

Static recipe, selection, dependency and order checks for both recipe profiles only.
The pin follows the October 20 publication; nothing here is a game installation or
gameplay acceptance.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import NATIVE_RUN_ORDERS, _effective_features, build_recipe
from tools.curation_audit import Decision, RowKey, load_catalogs, load_curation_map


ROOT = Path(__file__).resolve().parents[2]
MOD_ID = "chriz-bg-modpack"
ARTIFACT_ID = "chriz-bg-modpack-0.2.0-alpha.13"
PREVIOUS_ARTIFACT_ID = "chriz-bg-modpack-0.2.0-alpha.12"
LATE_RUN = "chriz-bg-modpack-late-companions-bg2"
CAMPAIGN_RUN = "chriz-bg-modpack-campaign-xp-bg2"
MODPACK = "feature:chriz-bg-modpack:component-{}"
JOIN_XP = MODPACK.format(239)
EET_CORE = "feature:eet:mandatory-components"
# Released in alpha.13; the collection offer waits for Christopher's decision.
UNDECIDED = (135, 460, 652)
ALPHA13_NATIVE = [
    110, 130, 135, 140, 160, 170, 188, 189, 190, 192, 193, 194, 195, 196, 197, 198, 199, 220, 221,
    222, 223, 224, 225, 230, 231, 232, 233, 234, 235, 236, 237, 238, 239, 400, 410, 430, 440, 450,
    460, 610, 611, 620, 630, 640, 660, 650, 651, 652,
]
# Runs of the NPC mods and NPC customization whose scripts 239 must see, when present.
NPC_SCRIPT_PROVIDERS = (
    "bg1npc-bg1", "fade-bg2", "paina-bg2", "sarahtob-bg2", "ub-bg2", "evandra-core-bg2", "xan-bg2",
    "yeslicknpc-bg2", "sirene-bg2", "branwen-bg2", "iepbanters-bg2", "evandra-crossmod-bg2",
    "crossmodbg2-bg2", "stratagems-bg2", "artisanskitpack-npc-bg2", "artisanskitpack-npc-late-bg2",
    "chriz-bg-modpack-continuity-bg2", "eet-end-bg2", "chriz-sod-remix-bg2", "safana-bg2",
)


def _load_profile(directory: Path) -> tuple[dict, dict]:
    collection = tomllib.loads((directory / "collection.toml").read_text(encoding="utf-8"))
    preset = tomllib.loads((directory / "presets/chris-recommended.toml").read_text(encoding="utf-8"))["selections"]
    return collection, preset


class Oct20IntakeTests(unittest.TestCase):
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
            (ROOT / "manifest/evidence/release-intake-20261020.toml").read_text(encoding="utf-8"))
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

    def test_eet_join_xp_is_a_default_after_npc_mods_and_before_campaign_xp(self):
        for _, _, collection, preset in self._each_profile():
            runs = {run["run_id"]: run for run in collection["runs"]}
            order = [run["run_id"] for run in collection["runs"]]
            features = {feature["id"]: feature for feature in collection["features"]}
            components = runs[LATE_RUN]["components"]
            self.assertEqual(components, [189, 237, 238, 239, 620])
            self.assertEqual(components, [c for c in ALPHA13_NATIVE if c in components])
            self.assertEqual(runs[LATE_RUN]["phase"], "post-eet-end")
            # 239 reads the joinable NPCs' final scripts: every NPC, NPC customization and
            # EET transition writer in the profile runs first.
            providers = [run_id for run_id in NPC_SCRIPT_PROVIDERS if run_id in order]
            self.assertTrue({"eet-end-bg2", "stratagems-bg2", "safana-bg2", "fade-bg2"} <= set(providers))
            for run_id in providers:
                self.assertLess(order.index(run_id), order.index(LATE_RUN), run_id)
            # 650 must follow the companion XP options it caps.
            self.assertEqual(order.index(CAMPAIGN_RUN), order.index(LATE_RUN) + 1)

            feature = features[JOIN_XP]
            self.assertEqual(feature["title"], "Give every joinable NPC EET's join-XP catch-up")
            for phrase in ("from 125,000 up to 1,250,000", "in ToB to 2,500,000", "Never lowers XP"):
                self.assertIn(phrase, feature["description"])
            self.assertEqual((feature["category"], feature["decision"], feature["readiness"]),
                             ("npcs", "default", "ready"))
            self.assertEqual(feature["parent"], "mod:chriz-bg-modpack")
            # The TP2 requires EET only (GAME_IS eet); no spell, kit, SR or EEex dependency.
            self.assertEqual(feature["requires"], [EET_CORE])
            self.assertNotIn("conflicts", feature)
            self.assertEqual(feature["components"], [{"run_id": LATE_RUN, "component": 239}])
            self.assertEqual(preset.get(JOIN_XP), "on")

            baseline = self._selected(collection, preset)
            self.assertIn((LATE_RUN, 239), baseline)
            self.assertEqual(baseline - self._selected(collection, preset, **{JOIN_XP: "off"}), {(LATE_RUN, 239)})
            # It is independent of Fade's own catch-up and of Spell Revisions.
            self.assertIn((LATE_RUN, 239), self._selected(collection, preset, **{"mod:fade": "off"}))
            self.assertIn((LATE_RUN, 239), self._selected(collection, preset, **{"mod:spell-rev": "off"}))
        self.assertEqual(NATIVE_RUN_ORDERS[LATE_RUN], [189, 237, 238, 239, 620])

    def test_campaign_xp_stays_optional_and_scrolls_stay_default(self):
        for _, _, collection, preset in self._each_profile():
            features = {feature["id"]: feature for feature in collection["features"]}
            baseline = self._selected(collection, preset)
            for component in (650, 651):
                feature = features[MODPACK.format(component)]
                self.assertEqual((feature["decision"], feature["readiness"]), ("optional", "experimental"))
                self.assertNotEqual(preset.get(feature["id"]), "on")
                self.assertNotIn((CAMPAIGN_RUN, component), baseline)
            self.assertEqual(features[MODPACK.format(430)]["decision"], "default")
            self.assertIn(("chriz-bg-modpack-scrolls-bg2", 430), baseline)
            # With 650 opted in, it follows 239 in the next run.
            banking = self._selected(collection, preset, **{MODPACK.format(650): "on"})
            self.assertEqual(banking - baseline, {(CAMPAIGN_RUN, 650)})

    def test_undecided_alpha13_components_are_not_offered(self):
        for _, directory, collection, _ in self._each_profile():
            features = {feature["id"] for feature in collection["features"]}
            installed = {
                component for run in collection["runs"] if run["mod_id"] == MOD_ID
                for component in run["components"]
            }
            mod = tomllib.loads((directory / f"mods/{MOD_ID}.toml").read_text(encoding="utf-8"))
            declared = {component["id"] for component in mod["components"]}
            self.assertIn(239, declared)
            for component in UNDECIDED:
                self.assertNotIn(MODPACK.format(component), features)
                self.assertNotIn(component, installed)
                self.assertNotIn(component, declared)

    def test_catalog_rows_record_the_intake_decisions(self):
        rows = {RowKey(row.catalog, row.component_id): row for row in load_catalogs(ROOT / "docs/curation/components")}
        expected = {
            239: Decision.DEFAULT,
            # Unchanged by this intake.
            237: Decision.DEFAULT, 238: Decision.DEFAULT, 430: Decision.DEFAULT,
            650: Decision.OPTIONAL, 651: Decision.OPTIONAL,
            # Open Decisions parse as not installed, like EEex 9.
            **{component: Decision.EXCLUDED for component in UNDECIDED},
        }
        for component, decision in expected.items():
            self.assertEqual(rows[RowKey("CHRIZ-BG-MODPACK", component)].decision, decision, component)
        mapping = load_curation_map(ROOT / "manifest/curation-map.toml")
        [modpack] = [catalog for catalog in mapping.catalogs if catalog.catalog == "CHRIZ-BG-MODPACK"]
        self.assertTrue(set(UNDECIDED) <= set(modpack.excluded))
        self.assertNotIn(239, modpack.excluded)
        targets = {target.target_id: target for target in mapping.targets}
        self.assertEqual(targets[JOIN_XP].rows, (RowKey("CHRIZ-BG-MODPACK", 239),))

    def test_intake_evidence_matches_current_pins_and_native_order(self):
        record = self.record
        self.assertEqual(self.evidence["verified_on"], "2026-10-20")
        self.assertEqual(record["mod_id"], MOD_ID)
        self.assertEqual(record["artifact_id"], ARTIFACT_ID)
        self.assertEqual(record["source_commit"], "72fc4e6d31887d9598e0a8dc4faf5b76aeb988a3")
        self.assertEqual(record["native_components"], ALPHA13_NATIVE)
        for flag in ("fresh_download", "fresh_extraction", "published_release_verified",
                     "native_component_order_verified"):
            self.assertIs(record[flag], True, flag)
        self.assertEqual(record["recipe_profiles"], ["manifest", "recipes/curated-full-current"])
        for profile in (ROOT / "manifest", ROOT / "recipes/curated-full-current", self.output):
            with self.subTest(profile=str(profile)):
                mod = tomllib.loads((profile / f"mods/{MOD_ID}.toml").read_text(encoding="utf-8"))
                self.assertEqual(mod["artifact_id"], ARTIFACT_ID)
                artifact = tomllib.loads((profile / f"artifacts/{ARTIFACT_ID}.toml").read_text(encoding="utf-8"))
                for key, measured in (("sha256", "actual_sha256"), ("expected_length", "actual_length"),
                                      ("url", "immutable_source_url"), ("reference", "source_reference")):
                    self.assertEqual(artifact["source"][key], record[measured])
                self.assertEqual(artifact["source"]["expected_filename"], "chriz-bg-modpack-v0.2.0-alpha.13.zip")
                # Same layout and limits as alpha.12, which stays available as history.
                previous = tomllib.loads(
                    (profile / f"artifacts/{PREVIOUS_ARTIFACT_ID}.toml").read_text(encoding="utf-8"))
                self.assertEqual(artifact["archive"], previous["archive"])
                self.assertEqual(artifact["source"]["redirect_hosts"], previous["source"]["redirect_hosts"])
                collection = tomllib.loads((profile / "collection.toml").read_text(encoding="utf-8"))
                runs = [run for run in collection["runs"] if run["mod_id"] == MOD_ID]
                self.assertEqual(len(runs), 9)
                for run in runs:
                    self.assertEqual(run["components"], [c for c in ALPHA13_NATIVE if c in run["components"]],
                                     run["run_id"])
        # The checked-in curated artifact is the generator's copy of the authored pin.
        self.assertEqual((ROOT / f"recipes/curated-full-current/artifacts/{ARTIFACT_ID}.toml").read_bytes(),
                         (ROOT / f"manifest/artifacts/{ARTIFACT_ID}.toml").read_bytes())


if __name__ == "__main__":
    unittest.main()
