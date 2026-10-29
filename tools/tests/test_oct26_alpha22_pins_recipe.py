"""October 26 intake, the alpha.22 pin batch: BG Rebalance v0.8.5 with Chris Revisions 340,
the Spell Revisions fork v4.21-chriz.7 with its SR/RR compatibility package, BuffBot v1.11.0,
the Artisan's Kitpack fork v4.81a-dev.85928f9-chriz.3, EEex Attack Info 9, and Chriz BG
Modpack v0.2.0-alpha.14 with alpha.13's Dorn ambush fix 135, party AI fix 460 and
chapter XP caps 652.

Static pin, evidence, component, selection and order checks for both recipe profiles only.
Nothing here is a game installation or gameplay acceptance. The Artisan's Kitpack fork
chriz.4 (October 27, test_oct27_alpha22_pins_recipe) supersedes the chriz.3 pin and keeps
its components and order, and BG Rebalance v0.8.6 (October 28, same test) supersedes the
v0.8.5 pin, keeps its components and order and adds the Time Stop and Haste effect clocks
360 after 340. BuffBot v1.12.1 (October 28, test_oct28_buffbot_pin_recipe) supersedes the
v1.11.0 pin and keeps its components and order. The chriz.3, v0.8.5 and v1.11.0 records and
artifacts stay as verified here. The Kitpack fork chriz.5 (October 29,
test_oct29_kitpack_chriz5_pin_recipe) then supersedes chriz.4 and adds tweak 7205, which is
checked against its own record.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import NATIVE_RUN_ORDERS, _effective_features, build_recipe
from tools.curation_audit import Decision, RowKey, load_catalogs


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261026.toml"
EEEX_EVIDENCE = "manifest/evidence/release-intake-20261018.toml"
PROFILES = ["manifest", "recipes/curated-full-current"]
# chriz.3 replaced chriz.2 in this batch before it merged; chriz.2 was never in a recipe.
KITPACK = "artisans-kitpack-4.81a-dev.85928f9-chriz.3"
KITPACK_PREVIOUS = "artisans-kitpack-4.81a-dev.85928f9-chriz.1"
KITPACK_TAG = "v4.81a-dev.85928f9-chriz.3"
# The October 27 intake replaces the chriz.3 pin of all three Kitpack installers.
KITPACK_SUCCESSOR = "artisans-kitpack-4.81a-dev.85928f9-chriz.4"
SUCCESSORS = {mod_id: KITPACK_SUCCESSOR
              for mod_id in ("artisanskitpack", "artisanskitpack-npc", "artisanskitpack-tweak")}
# The October 27 batch's second pin replaces Rebalance v0.8.5 with v0.8.6.
SUCCESSORS["chriz-bg-rebalance"] = "chriz-bg-rebalance-0.8.6"
# The October 28 intake (test_oct28_buffbot_pin_recipe) replaces BuffBot v1.11.0 with v1.12.1.
SUCCESSORS["buffbot"] = "buffbot-1.12.1"
# Components declared by a successor pin, checked against its own record; none is in a
# record of this intake.
LATER_COMPONENTS = {"chriz-bg-rebalance": {360}}
LATER_ROWS = {"CHRIZ-BG-REBALANCE": {360}}
# The October 29 intake (test_oct29_rebalance_pin_recipe) then replaces Rebalance v0.8.6
# with v0.8.7, which keeps every component and the order.
SUCCESSORS["chriz-bg-rebalance"] = "chriz-bg-rebalance-0.8.7"
# The October 29 v0.8.8 intake (test_oct29_rebalance_088_pin_recipe) replaces v0.8.7 and
# declares Druid weapons 410 and Alaghor of Clangeddin weapons 411 between 302 and 420.
SUCCESSORS["chriz-bg-rebalance"] = "chriz-bg-rebalance-0.8.8"
LATER_COMPONENTS["chriz-bg-rebalance"] = {360, 410, 411}
LATER_ROWS["CHRIZ-BG-REBALANCE"] = {360, 410, 411}
# The October 29 Kitpack intake (test_oct29_kitpack_chriz5_pin_recipe) replaces chriz.4 with
# chriz.5 for all three installers; its tweak TP2 declares 7205 between 7203 and 8204.
SUCCESSORS.update({mod_id: "artisans-kitpack-4.81a-dev.85928f9-chriz.5"
                   for mod_id in ("artisanskitpack", "artisanskitpack-npc", "artisanskitpack-tweak")})
LATER_COMPONENTS["artisanskitpack-tweak"] = {7205}
LATER_ROWS["ARTISANSKITPACK_TWEAK"] = {7205}
# Per mod: the new pin, the superseded pin (kept as history), its tag, the archive name and
# the catalog that lists the mod's components.
PINS = {
    "chriz-bg-rebalance": ("chriz-bg-rebalance-0.8.5", "chriz-bg-rebalance-0.8.3", "v0.8.5",
                           "chriz-bg-rebalance-v0.8.5-windows.zip", "CHRIZ-BG-REBALANCE"),
    "spell-rev": ("spell-revisions-4.21-chriz.7", "spell-revisions-4.21-chriz.5", "v4.21-chriz.7",
                  "spell_rev-v4-21-chriz-7.zip", "SPELL_REV"),
    "srcb-rr-compat": ("srcb-rr-compat-4.21-chriz.7", "srcb-rr-compat-4.21-chriz.5", "v4.21-chriz.7",
                       "SRCB_RR_COMPAT-v4-21-chriz-7.zip", "SRCB_RR_COMPAT"),
    "buffbot": ("buffbot-1.11.0", "buffbot-1.10.1", "v1.11.0", "buffbot-v1.11.0.zip", "BUFFBOT"),
    "artisanskitpack": (KITPACK, KITPACK_PREVIOUS, KITPACK_TAG,
                        f"ArtisansKitpack-Chriz-Balance-Patch-{KITPACK_TAG}.zip", "ARTISANSKITPACK"),
    "artisanskitpack-npc": (KITPACK, KITPACK_PREVIOUS, KITPACK_TAG,
                            f"ArtisansKitpack-Chriz-Balance-Patch-{KITPACK_TAG}.zip", "ARTISANSKITPACK_NPC"),
    "artisanskitpack-tweak": (KITPACK, KITPACK_PREVIOUS, KITPACK_TAG,
                              f"ArtisansKitpack-Chriz-Balance-Patch-{KITPACK_TAG}.zip", "ARTISANSKITPACK_TWEAK"),
    "chriz-bg-modpack": ("chriz-bg-modpack-0.2.0-alpha.14", "chriz-bg-modpack-0.2.0-alpha.13", "v0.2.0-alpha.14",
                         "chriz-bg-modpack-v0.2.0-alpha.14.zip", "CHRIZ-BG-MODPACK"),
}
# Catalog rows of retired Modpack numbers that no TP2 declares any more; none is offered.
RETIRED_ROWS = {"CHRIZ-BG-MODPACK": {100, 120, 150, 180, 200, 210, 300, 310, 420, 500, 510, 511, 512, 513, 514, 600}}
REBALANCE_RUN = "chriz-bg-rebalance-bg2"
EEEX_RUN = "eeex-bg2"
EEEX_CORE = "feature:eeex:mandatory-components"
REVISIONS = "feature:chriz-bg-rebalance:component-340"
ATTACK_INFO = "feature:eeex:component-9"
ARTISAN_MAIN_RUN = "artisanskitpack-main-bg2"
MODPACK = "feature:chriz-bg-modpack:component-{}"
MODPACK_MAIN_RUN = "chriz-bg-modpack-bg2"
CAMPAIGN_RUN = "chriz-bg-modpack-campaign-xp-bg2"
LATE_RUN = "chriz-bg-modpack-late-companions-bg2"
EET_CORE = "feature:eet:mandatory-components"
DORN_AMBUSH = MODPACK.format(135)
DEFENSIVE_STANCE = MODPACK.format(460)
CHAPTER_CAPS = MODPACK.format(652)
CAMPAIGN_CAPS = MODPACK.format(650)
SOA_CAP = MODPACK.format(651)


def _load_profile(directory: Path) -> tuple[dict, dict]:
    collection = tomllib.loads((directory / "collection.toml").read_text(encoding="utf-8"))
    preset = tomllib.loads((directory / "presets/chris-recommended.toml").read_text(encoding="utf-8"))["selections"]
    return collection, preset


class Oct26Alpha22PinBatchTests(unittest.TestCase):
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
        cls.evidence = tomllib.loads((ROOT / EVIDENCE).read_text(encoding="utf-8"))
        cls.records = {record["mod_id"]: record for record in cls.evidence["artifacts"]}

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

    def test_evidence_records_each_fresh_published_download(self):
        self.assertEqual(self.evidence["verified_on"], "2026-10-26")
        self.assertEqual(len(self.evidence["artifacts"]), len(PINS))
        self.assertEqual(set(self.records), set(PINS))
        for mod_id, (artifact_id, _, reference, filename, _) in PINS.items():
            record = self.records[mod_id]
            with self.subTest(mod=mod_id):
                self.assertEqual(record["artifact_id"], artifact_id)
                self.assertEqual(record["recipe_profiles"], PROFILES)
                self.assertEqual(record["source_reference"], reference)
                self.assertTrue(record["immutable_source_url"].startswith("https://github.com/Chrizhermann/"))
                self.assertTrue(record["immutable_source_url"].endswith(f"/releases/download/{reference}/{filename}"))
                for flag in ("fresh_download", "fresh_extraction", "published_release_verified",
                             "native_component_order_verified"):
                    self.assertIs(record[flag], True, flag)
                for digest in ("actual_sha256", "tp2_sha256"):
                    self.assertRegex(record[digest], r"^[0-9a-f]{64}$")
                self.assertRegex(record["source_commit"], r"^[0-9a-f]{40}$")
                self.assertEqual(len(record["native_components"]), len(set(record["native_components"])))
        # One release carries Spell Revisions and the SR/RR package; one archive carries
        # the three Kitpack installers, each with its own TP2.
        self.assertEqual(self.records["spell-rev"]["source_commit"], self.records["srcb-rr-compat"]["source_commit"])
        kitpack = [self.records[mod_id] for mod_id in ("artisanskitpack", "artisanskitpack-npc", "artisanskitpack-tweak")]
        for key in ("immutable_source_url", "actual_length", "actual_sha256", "source_commit"):
            self.assertEqual(len({record[key] for record in kitpack}), 1, key)
        self.assertEqual(len({record["tp2_sha256"] for record in kitpack}), 3)
        # Only the Kitpack main record narrows its order check to the curated profile.
        for mod_id, record in self.records.items():
            with self.subTest(mod=mod_id):
                expected = ["recipes/curated-full-current"] if mod_id == "artisanskitpack" else PROFILES
                self.assertEqual(record.get("native_order_recipe_profiles", record["recipe_profiles"]), expected)

    def test_both_profiles_pin_the_verified_archives_and_keep_the_previous_pins(self):
        for profile_name, directory in self.directories:
            for mod_id, (artifact_id, previous_id, reference, filename, _) in PINS.items():
                record = self.records[mod_id]
                with self.subTest(profile=str(directory), mod=mod_id):
                    self.assertIn(profile_name, record["recipe_profiles"])
                    mod = tomllib.loads((directory / f"mods/{mod_id}.toml").read_text(encoding="utf-8"))
                    # A superseded pin stays available as history, verified as recorded here.
                    self.assertEqual(mod["artifact_id"], SUCCESSORS.get(mod_id, artifact_id))
                    artifact = tomllib.loads((directory / f"artifacts/{artifact_id}.toml").read_text(encoding="utf-8"))
                    self.assertEqual(artifact["acquisition"], "fetch-only")
                    self.assertEqual(artifact["source"]["kind"], "github-release")
                    self.assertEqual(artifact["source"]["expected_filename"], filename)
                    self.assertEqual(artifact["version"], reference.removeprefix("v"))
                    for key, measured in (("sha256", "actual_sha256"), ("expected_length", "actual_length"),
                                          ("url", "immutable_source_url"), ("reference", "source_reference")):
                        self.assertEqual(artifact["source"][key], record[measured])
                    # Same layout, limits and redirect host as the previous pin, which stays
                    # available for installations made from earlier recipes.
                    previous = tomllib.loads((directory / f"artifacts/{previous_id}.toml").read_text(encoding="utf-8"))
                    self.assertEqual(artifact["archive"], previous["archive"])
                    self.assertEqual(artifact["source"]["redirect_hosts"], previous["source"]["redirect_hosts"])
                    self.assertEqual(artifact["source"]["kind"], previous["source"]["kind"])
                    self.assertNotEqual(artifact["source"]["sha256"], previous["source"]["sha256"])
                    self.assertTrue(set(artifact["archive"]["tp2_paths"]) >= {mod["tp2"]})
        # The checked-in curated artifacts are the generator's copies of the authored pins.
        for artifact_id in {pin[0] for pin in PINS.values()}:
            name = f"artifacts/{artifact_id}.toml"
            self.assertEqual((ROOT / "recipes/curated-full-current" / name).read_bytes(),
                             (ROOT / "manifest" / name).read_bytes())

    def test_runs_offer_only_declared_components_in_native_order(self):
        for profile_name, directory in self.directories:
            collection = tomllib.loads((directory / "collection.toml").read_text(encoding="utf-8"))
            for mod_id, record in self.records.items():
                native = record["native_components"]
                later = LATER_COMPONENTS.get(mod_id, set())
                mod = tomllib.loads((directory / f"mods/{mod_id}.toml").read_text(encoding="utf-8"))
                self.assertTrue({component["id"] for component in mod["components"]} - later <= set(native), mod_id)
                runs = [run for run in collection["runs"] if run["mod_id"] == mod_id]
                self.assertTrue(runs, mod_id)
                for run in runs:
                    with self.subTest(profile=str(directory), run=run["run_id"]):
                        components = [c for c in run["components"] if c not in later]
                        self.assertTrue(set(components) <= set(native))
                        if profile_name in record.get("native_order_recipe_profiles", record["recipe_profiles"]):
                            self.assertEqual(components, [c for c in native if c in components])
        # The curated generator restores the Kitpack main run's native order; the authored
        # fallback keeps the order it has had since chriz.1 (see the evidence record).
        native_main = self.records["artisanskitpack"]["native_components"]
        curated_main = NATIVE_RUN_ORDERS[ARTISAN_MAIN_RUN]
        self.assertEqual(curated_main, [c for c in native_main if c in curated_main])
        fallback_runs = {run["run_id"]: run for run in self.profiles["fallback"][1]["runs"]}
        self.assertEqual(sorted(fallback_runs[ARTISAN_MAIN_RUN]["components"]), sorted(curated_main))
        rebalance_order = [c for c in NATIVE_RUN_ORDERS[REBALANCE_RUN] if c not in LATER_COMPONENTS["chriz-bg-rebalance"]]
        self.assertEqual(rebalance_order,
                         [c for c in self.records["chriz-bg-rebalance"]["native_components"]
                          if c in rebalance_order])

    def test_catalogs_list_every_declared_component_with_the_recorded_decisions(self):
        rows = {}
        for row in load_catalogs(ROOT / "docs/curation/components"):
            rows.setdefault(row.catalog, {})[row.component_id] = row
        for mod_id, (*_, catalog) in PINS.items():
            with self.subTest(catalog=catalog):
                retired = RETIRED_ROWS.get(catalog, set())
                later = LATER_ROWS.get(catalog, set())
                self.assertEqual(set(rows[catalog]) - retired - later, set(self.records[mod_id]["native_components"]))
                for component in retired:
                    self.assertEqual(rows[catalog][component].decision, Decision.EXCLUDED, component)
        # The only new component in the batch (340) and the newly offered ones: EEex 9 and
        # Modpack alpha.13's 135, 460 and 652 (Christopher, 2026-10-25).
        self.assertEqual(rows["CHRIZ-BG-REBALANCE"][340].decision, Decision.DEFAULT)
        self.assertEqual(rows["EEEX"][9].decision, Decision.OPTIONAL)
        self.assertEqual(rows["CHRIZ-BG-MODPACK"][135].decision, Decision.DEFAULT)
        self.assertEqual(rows["CHRIZ-BG-MODPACK"][460].decision, Decision.DEFAULT)
        self.assertEqual(rows["CHRIZ-BG-MODPACK"][652].decision, Decision.OPTIONAL)
        # Unchanged by alpha.14.
        self.assertEqual(rows["CHRIZ-BG-MODPACK"][430].decision, Decision.DEFAULT)
        self.assertEqual(rows["CHRIZ-BG-MODPACK"][650].decision, Decision.OPTIONAL)
        self.assertEqual(rows["CHRIZ-BG-MODPACK"][651].decision, Decision.OPTIONAL)
        self.assertEqual(rows["BUFFBOT"][1].decision, Decision.MANDATORY)
        self.assertEqual(rows["BUFFBOT"][0].decision, Decision.MANDATORY)
        self.assertEqual(rows["SRCB_RR_COMPAT"][0].decision, Decision.MANDATORY)
        keys = {RowKey(catalog, component) for catalog, components in rows.items() for component in components}
        self.assertIn(RowKey("CHRIZ-BG-REBALANCE", 340), keys)
        self.assertIn(RowKey("EEEX", 9), keys)

    def test_chris_revisions_340_is_a_default_at_the_end_of_the_rebalance_run(self):
        for name, _, collection, preset in self._each_profile():
            features = {feature["id"]: feature for feature in collection["features"]}
            feature = features[REVISIONS]
            self.assertEqual((feature["category"], feature["decision"], feature["readiness"]),
                             ("spells", "default", "ready"))
            self.assertEqual(feature["parent"], "mod:chriz-bg-rebalance")
            self.assertEqual(feature.get("requires", []), [])
            self.assertEqual(feature.get("conflicts", []), [])
            self.assertNotIn("choice_group", feature)
            self.assertEqual(feature["components"], [{"run_id": REBALANCE_RUN, "component": 340}])
            for spell in ("Aid", "Luck", "Bless", "Chaotic Commands", "Prayer", "Recitation"):
                self.assertIn(spell, feature["title"])
            self.assertEqual(preset.get(REVISIONS), "on")
            runs = {run["run_id"]: run for run in collection["runs"]}
            # v0.8.6 declares 360 after 340 (test_oct27_alpha22_pins_recipe).
            self.assertEqual(runs[REBALANCE_RUN]["components"][-5:], [330, 331, 332, 340, 360])
            self.assertEqual(runs[REBALANCE_RUN]["phase"], "post-eet-end")
            baseline = self._selected(collection, preset)
            self.assertIn((REBALANCE_RUN, 340), baseline)
            self.assertEqual(baseline - self._selected(collection, preset, **{REVISIONS: "off"}),
                             {(REBALANCE_RUN, 340)})
            self.assertNotIn((REBALANCE_RUN, 340),
                             self._selected(collection, preset, **{"mod:chriz-bg-rebalance": "off"}))
            # It needs no EEex.
            self.assertIn((REBALANCE_RUN, 340), self._selected(collection, preset, **{"mod:eeex": "off"}))
            # It changes the final spells of SCS, Spell Revisions, Bardic Wonders and
            # IWDification, and runs before SR's NPC spellbook pass and the scroll policy.
            order = [run["run_id"] for run in collection["runs"]]
            rebalance = order.index(REBALANCE_RUN)
            earlier = ["spell-rev-core-bg2", "stratagems-bg2", "eet-end-bg2"]
            if name == "curated":
                earlier += ["bardicwonders-bg2", "iwdification-bg2"]
            for run_id in earlier:
                self.assertLess(order.index(run_id), rebalance, run_id)
            for run_id in ("spell-rev-npc-spellbooks-bg2", "chriz-bg-modpack-scrolls-bg2"):
                self.assertLess(rebalance, order.index(run_id), run_id)
            group = next(g for g in collection["feature_groups"] if g["id"] == "challenge-mode")
            self.assertNotIn(REVISIONS, group["members"])

    def test_eeex_attack_info_9_is_offered_unchecked_in_native_order(self):
        (eeex_record,) = tomllib.loads((ROOT / EEEX_EVIDENCE).read_text(encoding="utf-8"))["artifacts"]
        native = eeex_record["native_components"]
        self.assertEqual(native, [0, 1, 9, 2, 3, 4, 5, 6, 7, 8])
        for name, directory, collection, preset in self._each_profile():
            mod = tomllib.loads((directory / "mods/eeex.toml").read_text(encoding="utf-8"))
            self.assertEqual(mod["artifact_id"], eeex_record["artifact_id"])
            self.assertIn(9, {component["id"] for component in mod["components"]})
            features = {feature["id"]: feature for feature in collection["features"]}
            feature = features[ATTACK_INFO]
            self.assertEqual((feature["category"], feature["decision"], feature["readiness"]),
                             ("engine", "optional", "ready"))
            self.assertEqual(feature["parent"], "mod:eeex")
            self.assertEqual(feature["requires"], [EEEX_CORE])
            self.assertEqual(feature["components"], [{"run_id": EEEX_RUN, "component": 9}])
            self.assertIn("restart", feature["description"])
            self.assertNotEqual(preset.get(ATTACK_INFO), "on")
            runs = {run["run_id"]: run for run in collection["runs"]}
            self.assertEqual(runs[EEEX_RUN]["components"], native)
            baseline = self._selected(collection, preset)
            self.assertNotIn((EEEX_RUN, 9), baseline)
            self.assertEqual({component for run, component in baseline if run == EEEX_RUN}, set(range(9)))
            ticked = self._selected(collection, preset, **{ATTACK_INFO: "on"})
            self.assertEqual(ticked - baseline, {(EEEX_RUN, 9)})
            self.assertEqual(baseline - ticked, set())
            self.assertFalse(any(run == EEEX_RUN for run, _ in self._selected(
                collection, preset, **{ATTACK_INFO: "on", "mod:eeex": "off"})))
            group = next(g for g in collection["feature_groups"] if g["id"] == "challenge-mode")
            self.assertNotIn(ATTACK_INFO, group["members"])

    def test_modpack_alpha14_offers_135_and_460_by_default(self):
        expected = {
            DORN_AMBUSH: ("Stop the crash after Dorn's BG1 ambush", 135, [EET_CORE]),
            DEFENSIVE_STANCE: ("Party AI: Defensive Stance only with \"Use special abilities\" on", 460, []),
        }
        for name, directory, collection, preset in self._each_profile():
            features = {feature["id"]: feature for feature in collection["features"]}
            runs = {run["run_id"]: run for run in collection["runs"]}
            order = [run["run_id"] for run in collection["runs"]]
            # Alpha.13's TP2 declares 135 after 130 and 460 after 450.
            self.assertEqual(runs[MODPACK_MAIN_RUN]["components"],
                             [130, 135, 400, 410, 440, 450, 460, 610, 611, 630, 660])
            self.assertEqual(runs[MODPACK_MAIN_RUN]["phase"], "post-eet-end")
            # 135 installs after the mods that change Dorn's ambush scripts, such as EET,
            # and 460 after EET and the party AI writers; SCS changes neither.
            for run_id in ("eet-end-bg2", "stratagems-bg2", "chriz-sod-remix-bg2", "chriz-bg-rebalance-bg2"):
                self.assertLess(order.index(run_id), order.index(MODPACK_MAIN_RUN), run_id)
            baseline = self._selected(collection, preset)
            for feature_id, (title, component, requires) in expected.items():
                with self.subTest(feature=feature_id):
                    feature = features[feature_id]
                    self.assertEqual(feature["title"], title)
                    self.assertEqual((feature["category"], feature["decision"], feature["readiness"]),
                                     ("fixes", "default", "ready"))
                    self.assertEqual(feature["parent"], "mod:chriz-bg-modpack")
                    # 135's TP2 needs CUTDN1B.BCS, which a BG2-based install gets from EET's
                    # BG1 import; 460's TP2 needs only the game's own PARTYAI.2DA.
                    self.assertEqual(feature.get("requires", []), requires)
                    self.assertEqual(feature.get("conflicts", []), [])
                    self.assertNotIn("choice_group", feature)
                    self.assertEqual(feature["components"], [{"run_id": MODPACK_MAIN_RUN, "component": component}])
                    self.assertEqual(preset.get(feature_id), "on")
                    self.assertIn((MODPACK_MAIN_RUN, component), baseline)
                    self.assertEqual(baseline - self._selected(collection, preset, **{feature_id: "off"}),
                                     {(MODPACK_MAIN_RUN, component)})
                    # Neither needs EEex, Spell Revisions or SCS.
                    for mod_id in ("mod:eeex", "mod:spell-rev", "mod:stratagems"):
                        self.assertIn((MODPACK_MAIN_RUN, component),
                                      self._selected(collection, preset, **{mod_id: "off"}), mod_id)
                    group = next(g for g in collection["feature_groups"] if g["id"] == "challenge-mode")
                    self.assertNotIn(feature_id, group["members"])
            self.assertIn("Senjak", features[DORN_AMBUSH]["description"])
            self.assertIn("Use special abilities", features[DEFENSIVE_STANCE]["description"])

    def test_modpack_chapter_caps_652_is_offered_unchecked_instead_of_the_soa_cap(self):
        for name, directory, collection, preset in self._each_profile():
            features = {feature["id"]: feature for feature in collection["features"]}
            runs = {run["run_id"]: run for run in collection["runs"]}
            order = [run["run_id"] for run in collection["runs"]]
            self.assertEqual(runs[CAMPAIGN_RUN]["components"], [650, 651, 652])
            self.assertEqual(order.index(CAMPAIGN_RUN), order.index(LATE_RUN) + 1)
            feature = features[CHAPTER_CAPS]
            self.assertEqual((feature["category"], feature["decision"], feature["readiness"]),
                             ("rules", "optional", "experimental"))
            self.assertEqual(feature["parent"], "mod:chriz-bg-modpack")
            self.assertEqual(feature["requires"], [CAMPAIGN_CAPS])
            self.assertEqual(feature["components"], [{"run_id": CAMPAIGN_RUN, "component": 652}])
            for phrase in ("16,000", "161,000", "500,000", "3,500,000", "8,900,000", "Baeloth", "Chapter 5"):
                self.assertIn(phrase, feature["description"])
            # The TP2 refuses 651 with 652 and 652 with 651; both features say so.
            self.assertEqual([conflict["feature_id"] for conflict in feature["conflicts"]], [SOA_CAP])
            self.assertEqual([conflict["feature_id"] for conflict in features[SOA_CAP]["conflicts"]], [CHAPTER_CAPS])
            self.assertNotIn("choice_group", feature)
            self.assertNotEqual(preset.get(CHAPTER_CAPS), "on")
            self.assertNotEqual(preset.get(CAMPAIGN_CAPS), "on")
            baseline = self._selected(collection, preset)
            self.assertFalse(any(run == CAMPAIGN_RUN for run, _ in baseline))
            # Without 650 it stays out; with 650 it joins after it.
            self.assertNotIn((CAMPAIGN_RUN, 652), self._selected(collection, preset, **{CHAPTER_CAPS: "on"}))
            chapters = self._selected(collection, preset, **{CAMPAIGN_CAPS: "on", CHAPTER_CAPS: "on"})
            self.assertEqual(chapters - baseline, {(CAMPAIGN_RUN, 650), (CAMPAIGN_RUN, 652)})
            soa = self._selected(collection, preset, **{CAMPAIGN_CAPS: "on", SOA_CAP: "on"})
            self.assertEqual(soa - baseline, {(CAMPAIGN_RUN, 650), (CAMPAIGN_RUN, 651)})
            group = next(g for g in collection["feature_groups"] if g["id"] == "challenge-mode")
            self.assertNotIn(CHAPTER_CAPS, group["members"])
        self.assertEqual(NATIVE_RUN_ORDERS[CAMPAIGN_RUN], [650, 651, 652])
        self.assertEqual(NATIVE_RUN_ORDERS[MODPACK_MAIN_RUN],
                         [130, 135, 400, 410, 440, 450, 460, 610, 611, 630, 660])

    def test_superseded_records_still_match_their_kept_artifacts(self):
        superseded = {
            "manifest/evidence/release-intake-20261014.toml": "chriz-bg-rebalance",
            "manifest/evidence/release-intake-20261023-buffbot-eeexremote.toml": "buffbot",
            "manifest/evidence/release-intake-20261020.toml": "chriz-bg-modpack",
        }
        for path, mod_id in superseded.items():
            evidence = tomllib.loads((ROOT / path).read_text(encoding="utf-8"))
            (record,) = [record for record in evidence["artifacts"] if record["mod_id"] == mod_id]
            self.assertEqual(record["artifact_id"], PINS[mod_id][1])
            for _, directory in self.directories:
                with self.subTest(evidence=path, profile=str(directory)):
                    artifact = tomllib.loads(
                        (directory / f"artifacts/{record['artifact_id']}.toml").read_text(encoding="utf-8"))
                    for key, measured in (("sha256", "actual_sha256"), ("expected_length", "actual_length"),
                                          ("url", "immutable_source_url"), ("reference", "source_reference")):
                        self.assertEqual(artifact["source"][key], record[measured])


if __name__ == "__main__":
    unittest.main()
