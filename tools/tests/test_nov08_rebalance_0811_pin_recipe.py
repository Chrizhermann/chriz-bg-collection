"""November 8 intake: BG Rebalance v0.8.11 replaces v0.8.10 in both BG recipes.

The annotated tag is Rebalance PR #27's merge commit. Every component now writes its decided
result over edited game values, another mod's version, or missing owned pieces, and bard
progression 420 copies its tables through their exact file names for case-sensitive WeiDU.
The release keeps its 36 component declarations, their order and the Recommended selection.

Static pin, evidence, component, selection, order and release-text checks for both recipe
profiles only. Nothing here is a game installation or gameplay acceptance.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import NATIVE_RUN_ORDERS, _effective_features, build_recipe
from tools.curation_audit import Decision, load_catalogs


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261108-chriz-bg-rebalance-chriz-easy-bg.toml"
PROFILES = ["manifest", "recipes/curated-full-current"]
MOD = "chriz-bg-rebalance"
PIN = "chriz-bg-rebalance-0.8.11"
PREVIOUS = "chriz-bg-rebalance-0.8.10"
TAG = "v0.8.11"
COMMIT = "5cd90679f06e6d5be4792dad66a85723563f9e24"
URL = ("https://github.com/Chrizhermann/chriz-bg-rebalance/releases/download/v0.8.11/"
       "chriz-bg-rebalance-v0.8.11-windows.zip")
LENGTH = 1_813_831
SHA256 = "c9e9adf57a06b95bac055b4aee983b92690e3dacb2173b6c584cf4f75f059b34"
RUN = "chriz-bg-rebalance-bg2"
STATIC_COMMIT = "5de5d4b1293ce689da51e069fa691c5bc7e09dd8"
RECOMMENDED = {
    100, 101, 111, 120, 121, 200, 210, 211, 400, 401, 404, 405, 407, 408, 302, 410, 411,
    420, 421, 310, 130, 220, 320, 331, 332, 340, 360,
}
RUN_COMPONENTS = [
    100, 101, 110, 111, 120, 121, 200, 201, 202, 210, 211, 301, 400, 401, 404, 405, 407, 408,
    302, 410, 411, 420, 421, 310, 130, 220, 320, 330, 331, 332, 340, 360,
]


def _load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def _selected(collection: dict, selection: dict) -> set[tuple[str, int]]:
    state = _effective_features(collection, selection)
    return {
        (ref["run_id"], ref["component"])
        for feature in collection["features"]
        if state[feature["id"]]
        for ref in feature.get("components", [])
    }


class Nov08RebalanceV0811PinTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        cls.output = Path(cls.temp.name) / "recipe"
        build_recipe(ROOT, cls.output, "0" * 40)
        cls.directories = [
            ("manifest", ROOT / "manifest"),
            ("recipes/curated-full-current", ROOT / "recipes/curated-full-current"),
            ("recipes/curated-full-current", cls.output),
        ]
        cls.evidence = _load(ROOT / EVIDENCE)
        (cls.record,) = cls.evidence["artifacts"]

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def test_evidence_freezes_the_fresh_annotated_release(self):
        self.assertEqual(self.evidence["verified_on"], "2026-11-08")
        self.assertEqual(self.evidence["evidence_kind"],
                         "published-release-download-and-native-order-verification")
        self.assertEqual(self.evidence["generator"], "tools/prepare_own_mod_pin.py")
        for phrase in (
            "BG Rebalance v0.8.11", "annotated tag on merge commit 5cd9067",
            "Rebalance PR #27", "same 36 components in the same order",
            "overwrite their decided spell, item and table results", "exact file name",
            "not a WeiDU install or gameplay acceptance",
        ):
            self.assertIn(phrase, self.evidence["scope"])
        record = self.record
        self.assertEqual(record["mod_id"], MOD)
        self.assertEqual(record["artifact_id"], PIN)
        self.assertEqual(record["recipe_profiles"], PROFILES)
        self.assertEqual(record["source_reference"], TAG)
        self.assertEqual(record["source_commit"], COMMIT)
        self.assertEqual(record["immutable_source_url"], URL)
        self.assertEqual(record["actual_length"], LENGTH)
        self.assertEqual(record["actual_sha256"], SHA256)
        for flag in ("fresh_download", "fresh_extraction", "published_release_verified",
                     "native_component_order_verified"):
            self.assertIs(record[flag], True, flag)

    def test_both_profiles_pin_v0811_and_keep_v0810(self):
        for profile_name, directory in self.directories:
            with self.subTest(profile=str(directory)):
                self.assertIn(profile_name, self.record["recipe_profiles"])
                mod = _load(directory / f"mods/{MOD}.toml")
                artifact = _load(directory / f"artifacts/{PIN}.toml")
                previous = _load(directory / f"artifacts/{PREVIOUS}.toml")
                self.assertEqual(mod["artifact_id"], PIN)
                self.assertEqual(artifact["id"], PIN)
                self.assertEqual(artifact["version"], TAG.removeprefix("v"))
                self.assertEqual(artifact["acquisition"], "fetch-only")
                self.assertEqual(artifact["source"]["kind"], "github-release")
                self.assertEqual(artifact["source"]["url"], URL)
                self.assertEqual(artifact["source"]["reference"], TAG)
                self.assertEqual(artifact["source"]["expected_filename"], URL.rsplit("/", 1)[1])
                self.assertEqual(artifact["source"]["expected_length"], LENGTH)
                self.assertEqual(artifact["source"]["sha256"], SHA256)
                self.assertEqual(artifact["provenance"]["reviewed_on"], "2026-11-08")
                self.assertEqual(artifact["archive"], previous["archive"])
                self.assertEqual(artifact["source"]["redirect_hosts"], previous["source"]["redirect_hosts"])
                self.assertNotEqual(artifact["source"]["sha256"], previous["source"]["sha256"])
                self.assertEqual(artifact["archive"]["tp2_paths"], [mod["tp2"]])
        for artifact_id in (PIN, PREVIOUS):
            name = f"artifacts/{artifact_id}.toml"
            self.assertEqual((ROOT / "manifest" / name).read_bytes(),
                             (ROOT / "recipes/curated-full-current" / name).read_bytes())

    def test_components_order_and_recommended_selection_stay_the_same(self):
        rows = {row.component_id: row for row in load_catalogs(ROOT / "docs/curation/components")
                if row.catalog == "CHRIZ-BG-REBALANCE"}
        self.assertEqual(set(rows), set(self.record["native_components"]))
        self.assertEqual(rows[420].decision, Decision.DEFAULT)
        self.assertEqual(NATIVE_RUN_ORDERS[RUN], RUN_COMPONENTS)
        for _, directory in self.directories:
            collection = _load(directory / "collection.toml")
            preset = _load(directory / "presets/chris-recommended.toml")["selections"]
            runs = [run for run in collection["runs"] if run["mod_id"] == MOD]
            self.assertEqual([run["run_id"] for run in runs], [RUN])
            self.assertEqual(runs[0]["components"], RUN_COMPONENTS)
            selected = _selected(collection, preset)
            self.assertEqual({component for run_id, component in selected if run_id == RUN}, RECOMMENDED)

    def test_static_acceptance_records_name_the_pin_commit(self):
        for relative in (
            "manifest/releases/v0.1.0-alpha.1/acceptance.toml",
            "recipes/curated-full-current/releases/v0.1.0-alpha.1/acceptance.toml",
        ):
            evidence = _load(ROOT / relative)["evidence"]
            record = next(item for item in evidence if item["subject_id"] == RUN)
            self.assertEqual(record["commit"], STATIC_COMMIT)
            self.assertEqual(record["test_artifact"], "engine/tests/production_recipe_chriz.rs")
            self.assertEqual(record["date"], "2026-11-08")
            self.assertEqual(record["scope"],
                             "Static recipe source, component, and order assertions only; not live game acceptance.")

    def test_catalog_changelog_pin_list_and_source_record_name_v0811(self):
        catalog = (ROOT / "docs/curation/components/CHRIZ-BG-REBALANCE.md").read_text(encoding="utf-8")
        for phrase in ("The recipe pins **v0.8.11**", "Rebalance PR #27", "exact file names", EVIDENCE):
            self.assertIn(phrase, catalog)
        changelog = (ROOT / "docs/plans/2026-10-14-next-release-changelogs.md").read_text(encoding="utf-8")
        draft = changelog[changelog.index("## Collection 0.1.2 (draft)"):changelog.index("## Installer changelog")]
        for phrase in ("**BG Rebalance writes every selected balance change as decided**",
                       "BG Rebalance v0.8.11 (was v0.8.9)", "**Rebalance v0.8.11**",
                       "`artifact:chriz-bg-rebalance-0.8.11`"):
            self.assertIn(phrase, draft)
        pin_list = (ROOT / "docs/pin-list-2.7.md").read_text(encoding="utf-8")
        self.assertEqual(pin_list.count(
            f"| CHRIZ-BG-REBALANCE | {TAG.removeprefix('v')} | `{MOD}` | `{SHA256[:12]}…` |"), 2)
        self.assertNotIn("| CHRIZ-BG-REBALANCE | 0.8.10 |", pin_list)
        source = next(line for line in (ROOT / "manifest/mod-sources.tsv").read_text(encoding="utf-8")
                      .splitlines() if line.startswith("CHRIZ-BG-REBALANCE\t"))
        for phrase in ("v0.8.11", COMMIT, str(LENGTH), SHA256, "exact file name"):
            self.assertIn(phrase, source)


if __name__ == "__main__":
    unittest.main()
