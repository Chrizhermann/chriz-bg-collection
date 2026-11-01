"""October 29 intake: BG Rebalance v0.8.7 replaces v0.8.6 in both recipe profiles.

v0.8.7 renames 360 from "Time Stop and Haste effect clocks" to True Time Stop – fair Haste &
Slow and offers it as a standalone download as well, CBR_TRUE_TIMESTOP, which the collection
does not use. 360 refuses to install while that package is installed and keeps what it does
in the game. Against v0.8.6 the TP2 adds no component and keeps the declaration order, so no
decision changes (Christopher took v0.8.7 into alpha.22 on 2026-10-28).

Static pin, evidence, component, selection and order checks for both recipe profiles only.
Nothing here is a game installation or gameplay acceptance. BG Rebalance v0.8.8 (October 29,
test_oct29_rebalance_088_pin_recipe) supersedes this pin and adds 410 and 411; the v0.8.7
record and artifacts stay as verified here.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import NATIVE_RUN_ORDERS, _effective_features, build_recipe
from tools.curation_audit import Decision, load_catalogs


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261029-rebalance.toml"
PREVIOUS_EVIDENCE = "manifest/evidence/release-intake-20261027.toml"
PROFILES = ["manifest", "recipes/curated-full-current"]
MOD = "chriz-bg-rebalance"
PIN = "chriz-bg-rebalance-0.8.7"
PREVIOUS = "chriz-bg-rebalance-0.8.6"
TAG = "v0.8.7"
FILENAME = "chriz-bg-rebalance-v0.8.7-windows.zip"
SHA256 = "0f0edbf5f37fc5528f6cf3e5fbeffda126dcd0c18814c68f9da81a002793b072"
LENGTH = 1_818_389
TAG_COMMIT = "2c0aae47c8960e51c006061eaa68ed66b5b96f6a"
RUN = "chriz-bg-rebalance-bg2"
EEEX_RUN = "eeex-bg2"
EEEX_CORE = "feature:eeex:mandatory-components"
CLOCKS = "feature:chriz-bg-rebalance:component-360"
# CEBG shows the new name with an en dash; WeiDU's string uses a plain hyphen.
NAME = "True Time Stop – fair Haste & Slow"
WEIDU_NAME = "True Time Stop - fair Haste & Slow"
OLD_NAME = "Time Stop and Haste effect clocks"
# Chris's Recommended selection of the Rebalance run, unchanged by v0.8.7.
RECOMMENDED = {100, 101, 111, 120, 121, 200, 210, 211, 400, 401, 404, 405, 407, 408, 302, 420, 421,
               310, 130, 220, 320, 331, 332, 340, 360}
# The v0.8.8 intake (test_oct29_rebalance_088_pin_recipe) replaces this pin and declares
# Druid weapons 410 and Alaghor of Clangeddin weapons 411, both defaults, which are checked
# against its own record.
SUCCESSOR = "chriz-bg-rebalance-0.8.8"
LATER = {410, 411}


def _load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def _load_profile(directory: Path) -> tuple[dict, dict]:
    collection = _load(directory / "collection.toml")
    preset = _load(directory / "presets/chris-recommended.toml")["selections"]
    return collection, preset


class Oct29RebalanceV087PinTests(unittest.TestCase):
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
        self.assertIn("CBR_TRUE_TIMESTOP", self.evidence["scope"])
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
        # The TP2 changed, but it declares the same components in the same order.
        self.assertEqual(self.previous["artifact_id"], PREVIOUS)
        self.assertNotEqual(record["tp2_sha256"], self.previous["tp2_sha256"])
        self.assertEqual(record["native_components"], self.previous["native_components"])
        self.assertEqual(record["native_components"][-2:], [340, 360])

    def test_both_profiles_keep_v087_and_v086_and_leave_the_standalone_packages_out(self):
        for profile_name, directory in self.directories:
            with self.subTest(profile=str(directory)):
                self.assertIn(profile_name, self.record["recipe_profiles"])
                mod = _load(directory / f"mods/{MOD}.toml")
                self.assertEqual(mod["artifact_id"], SUCCESSOR)
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
                    # The v0.8.6 record still describes the kept v0.8.6 artifact.
                    self.assertEqual(previous["source"][key], self.previous[measured], key)
                # Same layout, limits and redirect host as v0.8.6, which stays for
                # installations made from the earlier recipe.
                self.assertEqual(artifact["archive"], previous["archive"])
                self.assertEqual(artifact["source"]["redirect_hosts"], previous["source"]["redirect_hosts"])
                self.assertNotEqual(artifact["source"]["sha256"], previous["source"]["sha256"])
                self.assertEqual(artifact["archive"]["tp2_paths"], [mod["tp2"]])
                # The release's standalone packages (CBR_SR_ANTIMAGIC, CBR_SR_HARDINESS and
                # the new CBR_TRUE_TIMESTOP) are not part of the recipe.
                names = {path.name.casefold() for path in (directory / "artifacts").glob("*.toml")}
                self.assertFalse(any(name.startswith(("cbr_", "cbr-")) for name in names), names)
        # The checked-in curated artifacts are the generator's copies of the authored pins.
        for artifact_id in (PIN, PREVIOUS):
            name = f"artifacts/{artifact_id}.toml"
            self.assertEqual((ROOT / "recipes/curated-full-current" / name).read_bytes(),
                             (ROOT / "manifest" / name).read_bytes())

    def test_360_has_the_new_name_and_keeps_its_place_and_decision(self):
        for _, directory in self.directories:
            with self.subTest(profile=str(directory)):
                mod = _load(directory / f"mods/{MOD}.toml")
                names = {component["id"]: component["name"] for component in mod["components"]}
                self.assertEqual(names[360], NAME)
                self.assertNotIn(OLD_NAME, names.values())
                features = {feature["id"]: feature for feature in _load(directory / "collection.toml")["features"]}
                self.assertEqual(features[CLOCKS]["title"], NAME)
                self.assertFalse(any(OLD_NAME in feature["title"] for feature in features.values()))
        for _, _, collection, preset in self._each_profile():
            feature = {feature["id"]: feature for feature in collection["features"]}[CLOCKS]
            self.assertEqual((feature["category"], feature["decision"], feature["readiness"]),
                             ("spells", "default", "ready"))
            self.assertEqual(feature["parent"], "mod:chriz-bg-rebalance")
            self.assertEqual(feature["requires"], [EEEX_CORE])
            self.assertEqual(feature.get("conflicts", []), [])
            self.assertEqual(feature["components"], [{"run_id": RUN, "component": 360}])
            for phrase in ("Time Stop", "everything but the caster stops", "Haste and Slow",
                           "before a playthrough", "not supported", "not covered yet"):
                self.assertIn(phrase, feature["description"])
            self.assertEqual(preset.get(CLOCKS), "on")
            baseline = self._selected(collection, preset)
            self.assertEqual({component for run_id, component in baseline if run_id == RUN} - LATER, RECOMMENDED)
            self.assertEqual(baseline - self._selected(collection, preset, **{CLOCKS: "off"}), {(RUN, 360)})
            self.assertNotIn((RUN, 360), self._selected(collection, preset, **{"mod:eeex": "off"}))

    def test_components_and_the_run_are_unchanged_in_native_order(self):
        native = self.record["native_components"]
        rows = {row.component_id: row for row in load_catalogs(ROOT / "docs/curation/components")
                if row.catalog == "CHRIZ-BG-REBALANCE"}
        self.assertEqual(set(rows) - LATER, set(native))
        self.assertEqual(rows[360].component, NAME)
        self.assertEqual(rows[360].decision, Decision.DEFAULT)
        order = [c for c in NATIVE_RUN_ORDERS[RUN] if c not in LATER]
        self.assertEqual(order, [c for c in native if c in order])
        for _, directory in self.directories:
            with self.subTest(profile=str(directory)):
                mod = _load(directory / f"mods/{MOD}.toml")
                self.assertTrue({component["id"] for component in mod["components"]} - LATER <= set(native))
                self.assertEqual((mod["tp2"], mod["language"], mod["weidu_artifact_id"], mod["invocation_mode"]),
                                 ("setup-chriz-bg-rebalance.tp2", 0, "weidu-249-amd64", "setup-name"))
                collection = _load(directory / "collection.toml")
                runs = [run for run in collection["runs"] if run["mod_id"] == MOD]
                self.assertEqual([run["run_id"] for run in runs], [RUN])
                (run,) = runs
                components = [c for c in run["components"] if c not in LATER]
                self.assertEqual(components, [c for c in native if c in components])
                self.assertEqual(run["components"][-2:], [340, 360])
                self.assertEqual(run["phase"], "post-eet-end")
                order = [entry["run_id"] for entry in collection["runs"]]
                self.assertLess(order.index(EEEX_RUN), order.index(RUN))

    def test_catalog_and_changelog_name_the_pin_the_rename_and_the_new_check(self):
        text = (ROOT / "docs/curation/components/CHRIZ-BG-REBALANCE.md").read_text(encoding="utf-8")
        self.assertIn(f"## {TAG}: the new name and the standalone package", text)
        self.assertIn(f"`{TAG_COMMIT[:7]}`", text)
        self.assertNotIn(f"| 360 | {OLD_NAME} |", text)
        for phrase in (NAME, f"`{WEIDU_NAME}`", "`@9366`", "`CBR_TRUE_TIMESTOP-v0.8.7-windows.zip`",
                       "`lib/time_stop_haste_clocks.tpa`", "`0x6a18d73d`", EVIDENCE,
                       "Christopher, 2026-10-28"):
            self.assertIn(phrase, text)
        changelog = (ROOT / "docs/plans/2026-10-14-next-release-changelogs.md").read_text(encoding="utf-8")
        # v0.8.8 replaced v0.8.7 in the version list and the release-note entry.
        for phrase in (f"**{NAME}** — **Default.**", "BG Rebalance v0.8.8 (was v0.8.2)", "Rebalance to v0.8.8",
                       "`CBR_TRUE_TIMESTOP`", "(Rebalance v0.8.6; named in v0.8.7)"):
            self.assertIn(phrase, changelog)


if __name__ == "__main__":
    unittest.main()
