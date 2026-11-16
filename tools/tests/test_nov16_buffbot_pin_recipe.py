"""November 16 intake: BuffBot v1.16.2 in both Chriz Easy installers.

The release updates the Italian localization catalog (`buffbot/lang/italian/setup.tra`, 72 entries
updated across the 252-entry catalog) with no runtime Lua logic, `.menu`, component declaration,
order, selected plan or save-schema changes. Static source/package assertions only, not a WeiDU
installation or gameplay acceptance.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import build_recipe


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261116-buffbot-chriz-easy-bg-chriz-easy-iwd.toml"
MOD = "buffbot"
PIN = "buffbot-1.16.2"
TAG = "v1.16.2"
FILENAME = "buffbot-v1.16.2.zip"
SHA256 = "ab2d95f589111d49f0b1e62cce47c2203e1cede91e7cd2316ffe76ce6ba6924a"
LENGTH = 5_798_498
TAG_COMMIT = "268af1268b07375ac1261d4a16c815e639b0e155"
PIN_COMMIT = "dd8247c02a07d5b6f5afccf33770b042c7c9baf2"
COMPONENTS = [1, 0]


def load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


class Nov16BuffBotV1162PinTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        cls.generated = Path(cls.temp.name) / "recipe"
        build_recipe(ROOT, cls.generated, "0" * 40)
        cls.evidence = load(ROOT / EVIDENCE)
        cls.records = {
            profile: record
            for record in cls.evidence["artifacts"]
            for profile in record["recipe_profiles"]
        }

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def test_fresh_release_evidence_is_complete(self):
        self.assertEqual(self.evidence["verified_on"], "2026-11-16")
        self.assertIn("replaces buffbot-1.16.1", self.evidence["scope"])
        self.assertIn("not a WeiDU install or gameplay acceptance", self.evidence["scope"])
        for record in self.records.values():
            self.assertEqual(record["artifact_id"], PIN)
            self.assertEqual(record["source_reference"], TAG)
            self.assertEqual(record["source_commit"], TAG_COMMIT)
            self.assertEqual(record["actual_length"], LENGTH)
            self.assertEqual(record["actual_sha256"], SHA256)
            self.assertEqual(record["native_components"], COMPONENTS)
            for flag in ("fresh_download", "fresh_extraction", "published_release_verified",
                         "native_component_order_verified"):
                self.assertIs(record[flag], True, flag)

    def test_all_shipped_recipes_freeze_the_new_artifact_and_run(self):
        profiles = [
            ("manifest", ROOT / "manifest"),
            ("recipes/curated-full-current", ROOT / "recipes/curated-full-current"),
            ("recipes/curated-full-current", self.generated),
            ("recipes/iwd-current", ROOT / "recipes/iwd-current"),
        ]
        for profile, directory in profiles:
            with self.subTest(profile=directory):
                record = self.records[profile]
                mod = load(directory / f"mods/{MOD}.toml")
                artifact = load(directory / f"artifacts/{PIN}.toml")
                self.assertEqual(mod["artifact_id"], PIN)
                self.assertEqual(artifact["version"], TAG.removeprefix("v"))
                self.assertEqual(artifact["source"]["reference"], TAG)
                self.assertEqual(artifact["source"]["expected_filename"], FILENAME)
                for key, evidence_key in (("url", "immutable_source_url"),
                                          ("expected_length", "actual_length"),
                                          ("sha256", "actual_sha256")):
                    self.assertEqual(artifact["source"][key], record[evidence_key], key)
                (run,) = [run for run in load(directory / "collection.toml")["runs"]
                          if run["mod_id"] == MOD]
                self.assertEqual(run["components"], COMPONENTS)

    def test_catalog_release_text_and_iwd_static_record_name_the_pin(self):
        catalog = (ROOT / "docs/curation/components/BUFFBOT.md").read_text(encoding="utf-8")
        self.assertIn(f"Target release: **{TAG}**", catalog)
        self.assertIn(f"`{TAG_COMMIT[:7]}`", catalog)
        self.assertIn(EVIDENCE, catalog)
        changelog = (ROOT / "docs/plans/2026-10-14-next-release-changelogs.md").read_text(encoding="utf-8")
        self.assertIn("**BuffBot v1.16.2**", changelog)
        self.assertIn("`artifact:buffbot-1.16.2`", changelog)
        acceptance = load(ROOT / "recipes/iwd-current/releases/v0.1.0-alpha.1/acceptance.toml")
        record = next(row for row in acceptance["evidence"] if row["subject_id"] == "buffbot-main")
        self.assertEqual(record["commit"], PIN_COMMIT)
        pin_list = (ROOT / "docs/pin-list-2.7.md").read_text(encoding="utf-8")
        self.assertEqual(pin_list.count(f"| BuffBot | 1.16.2 | `{MOD}` | `{SHA256[:12]}…` |"), 2)


if __name__ == "__main__":
    unittest.main()
