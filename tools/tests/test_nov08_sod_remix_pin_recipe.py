"""November 8 intake: SoD Remix v0.6.17 replaces v0.6.16 in Chriz Easy BG.

The release overwrites its decided result on edited game data without changing the component
declarations, curation decisions, run order or Recommended selection. This is static
source/package coverage only, not a WeiDU installation or gameplay acceptance.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import build_recipe


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261108-chriz-sod-remix-chriz-easy-bg.toml"
MOD = "chriz-sod-remix"
PIN = "chriz-sod-remix-0.6.17"
PREVIOUS = "chriz-sod-remix-0.6.16"
TAG = "v0.6.17"
FILENAME = "chriz-sod-remix-v0.6.17.zip"
SHA256 = "d8ce264f4288979bc23b84d501b05eeaf8b2a0462f4a780b9380d062e48c09e3"
LENGTH = 3_087_264
TAG_COMMIT = "42f1929731db8f18bc095fd44959026f9f9ac307"
RUN_COMPONENTS = [
    100, 110, 115, 120, 130, 135, 140, 150, 145, 160, 170, 180, 175, 177, 185, 190, 195,
    210, 197, 198, 187, 200, 215, 220, 225, 245, 230, 240, 250, 255, 256, 257, 260, 265,
    266, 270, 280, 285, 290, 900, 910, 292, 293, 294,
]


def load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


class Nov08SodRemixV0617PinTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        cls.generated = Path(cls.temp.name) / "recipe"
        build_recipe(ROOT, cls.generated, "0" * 40)
        cls.evidence = load(ROOT / EVIDENCE)
        (cls.record,) = cls.evidence["artifacts"]

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def test_fresh_release_evidence_keeps_the_native_declarations(self):
        self.assertEqual(self.evidence["verified_on"], "2026-11-08")
        self.assertIn(f"replaces {PREVIOUS}", self.evidence["scope"])
        self.assertIn("not a WeiDU install or gameplay acceptance", self.evidence["scope"])
        self.assertEqual(self.record["artifact_id"], PIN)
        self.assertEqual(self.record["recipe_profiles"], ["manifest", "recipes/curated-full-current"])
        self.assertEqual(self.record["source_reference"], TAG)
        self.assertEqual(self.record["source_commit"], TAG_COMMIT)
        self.assertEqual(self.record["actual_length"], LENGTH)
        self.assertEqual(self.record["actual_sha256"], SHA256)
        self.assertEqual(len(self.record["native_components"]), 49)
        for flag in ("fresh_download", "fresh_extraction", "published_release_verified",
                     "native_component_order_verified"):
            self.assertIs(self.record[flag], True, flag)

    def test_both_bg_recipes_freeze_the_new_artifact_and_preserve_the_old_one(self):
        for directory in (ROOT / "manifest", ROOT / "recipes/curated-full-current", self.generated):
            with self.subTest(profile=directory):
                mod = load(directory / f"mods/{MOD}.toml")
                artifact = load(directory / f"artifacts/{PIN}.toml")
                self.assertEqual(mod["artifact_id"], PIN)
                self.assertTrue((directory / f"artifacts/{PREVIOUS}.toml").is_file())
                self.assertEqual(artifact["version"], TAG.removeprefix("v"))
                self.assertEqual(artifact["source"]["reference"], TAG)
                self.assertEqual(artifact["source"]["expected_filename"], FILENAME)
                for key, evidence_key in (("url", "immutable_source_url"),
                                          ("expected_length", "actual_length"),
                                          ("sha256", "actual_sha256")):
                    self.assertEqual(artifact["source"][key], self.record[evidence_key], key)
                (run,) = [run for run in load(directory / "collection.toml")["runs"]
                          if run["mod_id"] == MOD]
                self.assertEqual(run["components"], RUN_COMPONENTS)

    def test_catalog_draft_source_ledger_and_pin_list_name_the_pin(self):
        catalog = (ROOT / "docs/curation/components/CHRIZ-SOD-REMIX.md").read_text(encoding="utf-8")
        self.assertIn(f"Pinned public release **{TAG}**", catalog)
        self.assertIn(f"`{TAG_COMMIT[:7]}`", catalog)
        self.assertIn(EVIDENCE, catalog)
        changelog = (ROOT / "docs/plans/2026-10-14-next-release-changelogs.md").read_text(encoding="utf-8")
        self.assertIn("SoD Remix v0.6.17 (was v0.6.16)", changelog)
        self.assertIn("**SoD Remix v0.6.17**", changelog)
        sources = (ROOT / "manifest/mod-sources.tsv").read_text(encoding="utf-8")
        self.assertIn(f"\t{TAG}\t2026-11-08\t", sources)
        self.assertIn(self.record["immutable_source_url"], sources)
        pin_list = (ROOT / "docs/pin-list-2.7.md").read_text(encoding="utf-8")
        self.assertEqual(pin_list.count(f"| CHRIZ-SOD-REMIX | 0.6.17 | `{MOD}` | `{SHA256[:12]}…` |"), 2)


if __name__ == "__main__":
    unittest.main()
