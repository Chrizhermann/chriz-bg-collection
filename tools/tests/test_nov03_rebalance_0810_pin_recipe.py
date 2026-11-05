"""November 3 intake: BG Rebalance v0.8.10 for CEBG 0.1.2 (Christopher, 2026-11-04).

v0.8.10 replaces v0.8.9 in both recipe profiles. Flail of the Ages balance 302 now reads
SCS's Haste and Slow layout, which SCS writes when Spell Revisions is not installed, so a
new installation without Spell Revisions no longer stops at BG Rebalance (0.1.1's known
issue). The +5 flail also lets Potion of Haste and Haste rings work. The TP2 changes VERSION
and 302's Haste/Slow detection; it declares the same 36 components in the same order, so no
component, selection or order changes.

The daily pin job prepared the pin (tools/prepare_own_mod_pin.py); this test freezes it and
its evidence record. Static pin, evidence, component, selection, order and release-text
checks for both recipe profiles only. Nothing here is a game installation or gameplay
acceptance.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import NATIVE_RUN_ORDERS, _effective_features, build_recipe
from tools.curation_audit import Decision, load_catalogs


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261103-chriz-bg-rebalance-chriz-easy-bg.toml"
PREVIOUS_EVIDENCE = "manifest/evidence/release-intake-20261102-kitpack-chriz6-rebalance-089.toml"
PROFILES = ["manifest", "recipes/curated-full-current"]
MOD = "chriz-bg-rebalance"
PIN = "chriz-bg-rebalance-0.8.10"
PREVIOUS = "chriz-bg-rebalance-0.8.9"
TAG = "v0.8.10"
COMMIT = "cd0ffa2dfa9b42e432e1ef9f59751eaa19cf75e1"
URL = ("https://github.com/Chrizhermann/chriz-bg-rebalance/releases/download/v0.8.10/"
       "chriz-bg-rebalance-v0.8.10-windows.zip")
LENGTH = 1_848_783
SHA256 = "40be063d26d531f3355022f7ab822e0b2018f00d99ecfd0d493913ffc4f68826"
TP2_SHA256 = "30c39b3e23e35b5cfa67d2fcbbf8a6623bdbfb24c3859dab2865645d25f5c5b9"
RUN = "chriz-bg-rebalance-bg2"
FLAIL = "feature:chriz-bg-rebalance:component-302"
# Chris's Recommended selection of the Rebalance run, unchanged since v0.8.8.
RECOMMENDED = {100, 101, 111, 120, 121, 200, 210, 211, 400, 401, 404, 405, 407, 408, 302, 410, 411,
               420, 421, 310, 130, 220, 320, 331, 332, 340, 360}
CHANGELOG = "docs/plans/2026-10-14-next-release-changelogs.md"


def _load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def _selected(collection: dict, selection: dict) -> set[tuple[str, int]]:
    state = _effective_features(collection, selection)
    return {(ref["run_id"], ref["component"]) for feature in collection["features"]
            if state[feature["id"]] for ref in feature.get("components", [])}


class Nov03RebalanceV0810PinTests(unittest.TestCase):
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
        cls.evidence = _load(ROOT / EVIDENCE)
        (cls.record,) = cls.evidence["artifacts"]
        (cls.previous,) = [record for record in _load(ROOT / PREVIOUS_EVIDENCE)["artifacts"]
                           if record["mod_id"] == MOD]

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def test_evidence_records_the_fresh_published_download(self):
        self.assertEqual(self.evidence["verified_on"], "2026-11-03")
        self.assertEqual(self.evidence["evidence_kind"], "published-release-download-and-native-order-verification")
        self.assertEqual(self.evidence["generator"], "tools/prepare_own_mod_pin.py")
        for phrase in ("BG Rebalance v0.8.10", "tag commit cd0ffa2", "fresh cache",
                       "Rebalance PR #26", "apart from its CRLF line endings",
                       "same 36 components in the same order",
                       "not a WeiDU install or gameplay acceptance"):
            self.assertIn(phrase, self.evidence["scope"])
        record = self.record
        self.assertEqual(record["mod_id"], MOD)
        self.assertEqual(record["artifact_id"], PIN)
        self.assertEqual(record["recipe_profiles"], PROFILES)
        self.assertNotIn("native_order_recipe_profiles", record)
        self.assertEqual(record["source_reference"], TAG)
        self.assertEqual(record["source_commit"], COMMIT)
        self.assertEqual(record["immutable_source_url"], URL)
        self.assertEqual(record["actual_length"], LENGTH)
        self.assertEqual(record["actual_sha256"], SHA256)
        self.assertEqual(record["tp2_sha256"], TP2_SHA256)
        for flag in ("fresh_download", "fresh_extraction", "published_release_verified",
                     "native_component_order_verified"):
            self.assertIs(record[flag], True, flag)
        # The TP2 changed (VERSION and 302's Haste/Slow detection), its components and their
        # order did not.
        self.assertEqual(self.previous["artifact_id"], PREVIOUS)
        self.assertNotEqual(record["tp2_sha256"], self.previous["tp2_sha256"])
        self.assertEqual(record["native_components"], self.previous["native_components"])

    def test_both_profiles_pin_v0810_and_keep_v089(self):
        for profile_name, directory in self.directories:
            artifact = _load(directory / f"artifacts/{PIN}.toml")
            previous = _load(directory / f"artifacts/{PREVIOUS}.toml")
            with self.subTest(profile=str(directory)):
                self.assertEqual(artifact["id"], PIN)
                self.assertEqual(artifact["name"], previous["name"])
                self.assertEqual(artifact["version"], TAG.removeprefix("v"))
                self.assertEqual(artifact["acquisition"], "fetch-only")
                self.assertEqual(artifact["source"]["kind"], "github-release")
                self.assertEqual(artifact["source"]["url"], URL)
                self.assertEqual(artifact["source"]["reference"], TAG)
                self.assertEqual(artifact["source"]["expected_filename"], URL.rsplit("/", 1)[1])
                self.assertEqual(artifact["source"]["expected_length"], LENGTH)
                self.assertEqual(artifact["source"]["sha256"], SHA256)
                self.assertEqual(artifact["provenance"]["url"],
                                 f"https://github.com/Chrizhermann/chriz-bg-rebalance/releases/tag/{TAG}")
                self.assertEqual(artifact["provenance"]["reviewed_on"], "2026-11-03")
                # Same layout, limits, redirect host and provenance as v0.8.9, which stays
                # for installations made from the earlier recipe.
                self.assertEqual(artifact["archive"], previous["archive"])
                self.assertEqual(artifact["source"]["redirect_hosts"], previous["source"]["redirect_hosts"])
                self.assertEqual(artifact["provenance"]["license"], previous["provenance"]["license"])
                self.assertIn(profile_name, self.record["recipe_profiles"])
                mod = _load(directory / f"mods/{MOD}.toml")
                self.assertEqual(mod["artifact_id"], PIN)
                self.assertEqual(artifact["archive"]["tp2_paths"], [mod["tp2"]])
                # The release's standalone packages are not part of the recipe.
                names = {path.name.casefold() for path in (directory / "artifacts").glob("*.toml")}
                self.assertFalse(any(name.startswith(("cbr_", "cbr-")) for name in names), names)
        # The checked-in curated artifacts are the generator's copies of the authored pins.
        for artifact_id in (PIN, PREVIOUS):
            name = f"artifacts/{artifact_id}.toml"
            self.assertEqual((ROOT / "recipes/curated-full-current" / name).read_bytes(),
                             (ROOT / "manifest" / name).read_bytes())

    def test_components_order_and_recommended_stay_and_302_needs_no_spell_revisions(self):
        native = self.record["native_components"]
        self.assertEqual(NATIVE_RUN_ORDERS[RUN], [c for c in native if c in NATIVE_RUN_ORDERS[RUN]])
        for _, directory in self.directories:
            collection = _load(directory / "collection.toml")
            preset = _load(directory / "presets/chris-recommended.toml")["selections"]
            features = {feature["id"]: feature for feature in collection["features"]}
            mod = _load(directory / f"mods/{MOD}.toml")
            with self.subTest(profile=str(directory)):
                self.assertTrue({component["id"] for component in mod["components"]} <= set(native))
                runs = [run for run in collection["runs"] if run["mod_id"] == MOD]
                self.assertEqual([run["run_id"] for run in runs], [RUN])
                self.assertEqual(runs[0]["components"], [c for c in native if c in runs[0]["components"]])
                baseline = _selected(collection, preset)
                self.assertEqual({component for run_id, component in baseline if run_id == RUN}, RECOMMENDED)
                # 302 stays a default with no requirement; without Spell Revisions it is still
                # selected, which v0.8.10 now installs.
                flail = features[FLAIL]
                self.assertEqual((flail["decision"], flail["readiness"]), ("default", "ready"))
                self.assertEqual(flail["components"], [{"run_id": RUN, "component": 302}])
                self.assertEqual(flail.get("requires", []), [])
                self.assertEqual(flail.get("requires_any", []), [])
                self.assertEqual(preset[FLAIL], "on")
                without_sr = _selected(collection, {**preset, "mod:spell-rev": "off"})
                self.assertFalse(any(run_id.startswith("spell-rev") for run_id, _ in without_sr))
                self.assertIn((RUN, 302), without_sr)

    def test_catalog_pin_list_and_changelog_draft_name_the_pin(self):
        rows = {row.component_id: row for row in load_catalogs(ROOT / "docs/curation/components")
                if row.catalog == "CHRIZ-BG-REBALANCE"}
        self.assertEqual(set(rows), set(self.record["native_components"]))
        self.assertEqual(rows[302].decision, Decision.DEFAULT)
        catalog = " ".join((ROOT / "docs/curation/components/CHRIZ-BG-REBALANCE.md")
                           .read_text(encoding="utf-8").split())
        for phrase in ("The recipe pins **v0.8.10** (published 2026-11-03 18:42 UTC, tag commit `cd0ffa2`",
                       "for CEBG 0.1.2", "v0.8.9 (2026-11-01 21:45 UTC, tag commit `bb3f11b`; pinned",
                       "Potion of Haste", EVIDENCE):
            self.assertIn(phrase, catalog)
        pin_list = (ROOT / "docs/pin-list-2.7.md").read_text(encoding="utf-8")
        self.assertEqual(pin_list.count(f"| CHRIZ-BG-REBALANCE | {TAG.removeprefix('v')} | `{MOD}` | `{SHA256[:12]}…` |"), 2)
        self.assertNotIn("| CHRIZ-BG-REBALANCE | 0.8.9 |", pin_list)
        changelog = " ".join((ROOT / CHANGELOG).read_text(encoding="utf-8").split())
        for phrase in ("## Collection 0.1.2 (draft)",
                       "**A new installation without Spell Revisions no longer stops at BG Rebalance.**",
                       "known issue of 0.1.1", "Potion of Haste", "(Rebalance v0.8.10)",
                       "BG Rebalance v0.8.10 (was v0.8.9)", "**Rebalance v0.8.10** (pinned 2026-11-04",
                       # 0.1.0's own entries keep the version it shipped.
                       "BG Rebalance v0.8.9 (was v0.8.2)", "Rebalance to v0.8.9"):
            self.assertIn(phrase, changelog)


if __name__ == "__main__":
    unittest.main()
