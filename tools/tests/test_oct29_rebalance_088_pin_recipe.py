"""October 29 intake: BG Rebalance v0.8.8 replaces v0.8.7 in both recipe profiles.

v0.8.8 adds Druid weapons 410 and Alaghor of Clangeddin weapons 411, which its TP2 declares
between 302 and 420; no other component changes. Christopher took both into alpha.22 on
2026-10-29 and made both defaults the same day. 411 needs EEex, Yeslick NPC's
Fighter-Alaghor and Artisan's Kitpack's EEex weapon usability, which within the collection
Eldritch Knight 20000 or Arcane Trickster sorcerer equipment 8204 creates. Both run in the
post-EET-end Rebalance run, after every Artisan's Kitpack run (the late tweak run with
1010000 included), Yeslick NPC and EEex.

Static pin, evidence, component, selection and order checks for both recipe profiles only.
Nothing here is a game installation or gameplay acceptance.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import NATIVE_RUN_ORDERS, _effective_features, build_recipe
from tools.curation_audit import Decision, load_catalogs


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261029-rebalance-088.toml"
PREVIOUS_EVIDENCE = "manifest/evidence/release-intake-20261029-rebalance.toml"
PROFILES = ["manifest", "recipes/curated-full-current"]
MOD = "chriz-bg-rebalance"
PIN = "chriz-bg-rebalance-0.8.8"
PREVIOUS = "chriz-bg-rebalance-0.8.7"
TAG = "v0.8.8"
FILENAME = "chriz-bg-rebalance-v0.8.8-windows.zip"
SHA256 = "dec0544b3cc0f2197b5791e89cf0eb73d2cdb563d3ae35c254d0da719d89a2e3"
LENGTH = 1_839_187
TAG_COMMIT = "92479d1baa83e8b0780b7dfc3009dc8820478cc6"
RUN = "chriz-bg-rebalance-bg2"
EEEX_RUN = "eeex-bg2"
YESLICK_RUN = "yeslicknpc-bg2"
EEEX_CORE = "feature:eeex:mandatory-components"
YESLICK_ALAGHOR = "feature:yeslicknpc:component-1"
ELDRITCH_KNIGHT = "feature:artisanskitpack:component-20000"
TRICKSTER_EQUIPMENT = "feature:artisanskitpack-tweak:component-8204"
DRUID = "feature:chriz-bg-rebalance:component-410"
ALAGHOR = "feature:chriz-bg-rebalance:component-411"
NEW = {410: DRUID, 411: ALAGHOR}
# Chris's Recommended selection of the Rebalance run: v0.8.7's plus 410 and 411.
RECOMMENDED = {100, 101, 111, 120, 121, 200, 210, 211, 400, 401, 404, 405, 407, 408, 302, 410, 411,
               420, 421, 310, 130, 220, 320, 331, 332, 340, 360}


def _load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def _load_profile(directory: Path) -> tuple[dict, dict]:
    collection = _load(directory / "collection.toml")
    preset = _load(directory / "presets/chris-recommended.toml")["selections"]
    return collection, preset


class Oct29RebalanceV088PinTests(unittest.TestCase):
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
        cls.evidence = _load(ROOT / EVIDENCE)
        (cls.record,) = cls.evidence["artifacts"]
        (cls.previous,) = [record for record in _load(ROOT / PREVIOUS_EVIDENCE)["artifacts"]
                           if record["mod_id"] == MOD]

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

    def test_evidence_records_the_fresh_published_download(self):
        self.assertEqual(self.evidence["verified_on"], "2026-10-29")
        self.assertEqual(self.evidence["evidence_kind"], "published-release-download-and-native-order-verification")
        for phrase in ("v0.8.8", "92479d1", "CBR_TRUE_TIMESTOP", "not pinned"):
            self.assertIn(phrase, self.evidence["scope"])
        record = self.record
        self.assertEqual(record["mod_id"], MOD)
        self.assertEqual(record["artifact_id"], PIN)
        self.assertEqual(record["recipe_profiles"], PROFILES)
        self.assertNotIn("native_order_recipe_profiles", record)
        self.assertNotIn("verified_on", record)
        self.assertEqual(record["source_reference"], TAG)
        self.assertEqual(record["source_commit"], TAG_COMMIT)
        self.assertEqual(record["immutable_source_url"],
                         f"https://github.com/Chrizhermann/chriz-bg-rebalance/releases/download/{TAG}/{FILENAME}")
        self.assertEqual(record["actual_length"], LENGTH)
        self.assertEqual(record["actual_sha256"], SHA256)
        self.assertRegex(record["tp2_sha256"], r"^[0-9a-f]{64}$")
        for flag in ("fresh_download", "fresh_extraction", "published_release_verified",
                     "native_component_order_verified"):
            self.assertIs(record[flag], True, flag)
        # The TP2 declares 410 and 411 right after 302 and changes nothing else in the order.
        self.assertEqual(self.previous["artifact_id"], PREVIOUS)
        self.assertNotEqual(record["tp2_sha256"], self.previous["tp2_sha256"])
        previous = self.previous["native_components"]
        after = previous.index(302) + 1
        self.assertEqual(record["native_components"], previous[:after] + [410, 411] + previous[after:])

    def test_both_profiles_pin_v088_keep_v087_and_leave_the_standalone_packages_out(self):
        for profile_name, directory in self.directories:
            with self.subTest(profile=str(directory)):
                self.assertIn(profile_name, self.record["recipe_profiles"])
                mod = _load(directory / f"mods/{MOD}.toml")
                self.assertEqual(mod["artifact_id"], PIN)
                artifact = _load(directory / f"artifacts/{PIN}.toml")
                previous = _load(directory / f"artifacts/{PREVIOUS}.toml")
                self.assertEqual(artifact["id"], PIN)
                self.assertEqual(artifact["name"], previous["name"])
                self.assertEqual(artifact["version"], TAG.removeprefix("v"))
                self.assertEqual(artifact["acquisition"], "fetch-only")
                self.assertEqual(artifact["source"]["kind"], "github-release")
                self.assertEqual(artifact["source"]["expected_filename"], FILENAME)
                self.assertEqual(artifact["provenance"]["url"],
                                 f"https://github.com/Chrizhermann/chriz-bg-rebalance/releases/tag/{TAG}")
                for key, measured in (("sha256", "actual_sha256"), ("expected_length", "actual_length"),
                                      ("url", "immutable_source_url"), ("reference", "source_reference")):
                    self.assertEqual(artifact["source"][key], self.record[measured], key)
                    # The v0.8.7 record still describes the kept v0.8.7 artifact.
                    self.assertEqual(previous["source"][key], self.previous[measured], key)
                # Same layout, limits and redirect host as v0.8.7, which stays for
                # installations made from the earlier recipe.
                self.assertEqual(artifact["archive"], previous["archive"])
                self.assertEqual(artifact["source"]["redirect_hosts"], previous["source"]["redirect_hosts"])
                self.assertNotEqual(artifact["source"]["sha256"], previous["source"]["sha256"])
                self.assertEqual(artifact["archive"]["tp2_paths"], [mod["tp2"]])
                # The release's standalone packages are not part of the recipe.
                names = {path.name.casefold() for path in (directory / "artifacts").glob("*.toml")}
                self.assertFalse(any(name.startswith(("cbr_", "cbr-")) for name in names), names)
                components = {component["id"]: component["name"] for component in mod["components"]}
                self.assertIn("axes and short bows", components[410])
                self.assertIn("Alaghor of Clangeddin", components[411])
        # The checked-in curated artifacts are the generator's copies of the authored pins.
        for artifact_id in (PIN, PREVIOUS):
            name = f"artifacts/{artifact_id}.toml"
            self.assertEqual((ROOT / "recipes/curated-full-current" / name).read_bytes(),
                             (ROOT / "manifest" / name).read_bytes())

    def test_410_and_411_are_defaults_with_the_tp2_requirements(self):
        for _, _, collection, preset in self._each_profile():
            features = {feature["id"]: feature for feature in collection["features"]}
            for component, feature_id in NEW.items():
                feature = features[feature_id]
                self.assertEqual((feature["category"], feature["decision"], feature["readiness"]),
                                 ("kits", "default", "ready"))
                self.assertEqual(feature["parent"], "mod:chriz-bg-rebalance")
                self.assertEqual(feature.get("conflicts", []), [])
                self.assertNotIn("choice_group", feature)
                self.assertEqual(feature["components"], [{"run_id": RUN, "component": component}])
                self.assertEqual(preset.get(feature_id), "on")
            self.assertEqual(features[DRUID]["title"], "Druid weapons")
            self.assertEqual(features[ALAGHOR]["title"], "Alaghor of Clangeddin weapons")
            for phrase in ("every druid kit", "axes and short bows", "long bows", "next level-up"):
                self.assertIn(phrase, features[DRUID]["description"])
            for phrase in ("Yeslick", "cleric weapons plus axes", "thief weapons", "EEex", "Artisan's Kitpack"):
                self.assertIn(phrase, features[ALAGHOR]["description"])
            # 410's TP2 needs only BG2:EE or EET with WEAPPROF, KITLIST and the druid classes.
            self.assertEqual(features[DRUID].get("requires", []), [])
            self.assertEqual(features[DRUID].get("requires_any", []), [])
            # 411's TP2 also needs EEex, the LK_ALAGHOR kit and C0PR#CL, which Eldritch
            # Knight 20000 (with EEex) or Arcane Trickster sorcerer equipment 8204 creates.
            self.assertEqual(features[ALAGHOR]["requires"], [EEEX_CORE, YESLICK_ALAGHOR])
            self.assertEqual(features[ALAGHOR]["requires_any"], [ELDRITCH_KNIGHT, TRICKSTER_EQUIPMENT])

            baseline = self._selected(collection, preset)
            self.assertEqual({component for run_id, component in baseline if run_id == RUN}, RECOMMENDED)
            for component, feature_id in NEW.items():
                self.assertEqual(baseline - self._selected(collection, preset, **{feature_id: "off"}),
                                 {(RUN, component)})
            without_eeex = self._selected(collection, preset, **{"mod:eeex": "off"})
            self.assertIn((RUN, 410), without_eeex)
            self.assertNotIn((RUN, 411), without_eeex)
            self.assertNotIn((RUN, 411), self._selected(collection, preset, **{YESLICK_ALAGHOR: "off"}))
            for provider in (ELDRITCH_KNIGHT, TRICKSTER_EQUIPMENT):
                self.assertIn((RUN, 411), self._selected(collection, preset, **{provider: "off"}))
            neither = self._selected(collection, preset, **{ELDRITCH_KNIGHT: "off", TRICKSTER_EQUIPMENT: "off"})
            self.assertNotIn((RUN, 411), neither)
            self.assertIn((RUN, 410), neither)
            self.assertFalse(any(run_id == RUN for run_id, _ in
                                 self._selected(collection, preset, **{"mod:chriz-bg-rebalance": "off"})))
            group = next(g for g in collection["feature_groups"] if g["id"] == "challenge-mode")
            self.assertFalse(set(NEW.values()) & set(group["members"]))

    def test_the_rebalance_run_follows_the_tp2_after_the_kitpack_yeslick_and_eeex(self):
        native = self.record["native_components"]
        self.assertEqual(NATIVE_RUN_ORDERS[RUN], [c for c in native if c in NATIVE_RUN_ORDERS[RUN]])
        self.assertIn([302, 410, 411, 420], [NATIVE_RUN_ORDERS[RUN][i:i + 4] for i in range(len(NATIVE_RUN_ORDERS[RUN]))])
        for _, directory in self.directories:
            with self.subTest(profile=str(directory)):
                mod = _load(directory / f"mods/{MOD}.toml")
                self.assertTrue({component["id"] for component in mod["components"]} <= set(native))
                collection = _load(directory / "collection.toml")
                runs = [run for run in collection["runs"] if run["mod_id"] == MOD]
                self.assertEqual([run["run_id"] for run in runs], [RUN])
                (run,) = runs
                self.assertEqual(run["components"], [c for c in native if c in run["components"]])
                position = run["components"].index(302)
                self.assertEqual(run["components"][position:position + 4], [302, 410, 411, 420])
                self.assertEqual(run["phase"], "post-eet-end")
                order = [entry["run_id"] for entry in collection["runs"]]
                # 1010000 rebuilds C0PR#CL's class grants in the late tweak run; 410 adds the
                # druid grants afterwards, and no Kitpack run follows the Rebalance run.
                kitpack_runs = [entry["run_id"] for entry in collection["runs"]
                                if entry["mod_id"].startswith("artisanskitpack")]
                self.assertIn("artisanskitpack-tweak-late-bg2", kitpack_runs)
                late = next(entry for entry in collection["runs"] if entry["run_id"] == "artisanskitpack-tweak-late-bg2")
                self.assertEqual(late["components"][-1], 1010000)
                for kitpack_run in kitpack_runs:
                    self.assertLess(order.index(kitpack_run), order.index(RUN), kitpack_run)
                self.assertLess(order.index(YESLICK_RUN), order.index(RUN))
                self.assertLess(order.index(EEEX_RUN), order.index(RUN))

    def test_catalog_and_changelog_name_the_pin_the_decision_and_the_order(self):
        rows = {row.component_id: row for row in load_catalogs(ROOT / "docs/curation/components")
                if row.catalog == "CHRIZ-BG-REBALANCE"}
        self.assertEqual(set(rows), set(self.record["native_components"]))
        for component in NEW:
            self.assertEqual(rows[component].decision, Decision.DEFAULT)
        text = (ROOT / "docs/curation/components/CHRIZ-BG-REBALANCE.md").read_text(encoding="utf-8")
        self.assertIn(f"**{TAG}**", text)
        self.assertIn(f"`{TAG_COMMIT[:7]}`", text)
        for phrase in ("36 curated entries, 27 recommended selections",
                       "Christopher's decision, 2026-10-29", "`LK_ALAGHOR`", "`C0PR#CL.SPL`",
                       "`..., 408, 302, 410, 411, 420, 421, ...`", "`1010000` rebuilds", EVIDENCE):
            self.assertIn(phrase, text)
        changelog = (ROOT / "docs/plans/2026-10-14-next-release-changelogs.md").read_text(encoding="utf-8")
        for phrase in ("**Druid weapons** — **Default.**", "**Alaghor of Clangeddin weapons** — **Default.**",
                       "BG Rebalance v0.8.8 (was v0.8.2)", "Rebalance to v0.8.8",
                       "**Rebalance `410` and `411`**"):
            self.assertIn(phrase, changelog)


if __name__ == "__main__":
    unittest.main()
