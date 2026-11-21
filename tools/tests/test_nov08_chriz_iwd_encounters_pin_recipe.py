"""November 8 intake: Chriz IWD Encounters v0.2.2 replaces v0.2.1.

The IWD-only pin keeps the selected component list and writes every decided encounter despite
edited game data. This is static source/package coverage only, not a WeiDU installation or
gameplay acceptance.
The November 21 intake (test_nov21_chriz_iwd_encounters_030_recipe) supersedes this pin with
v0.3.0; the v0.2.2 evidence record and v0.1.9 ledger stay as verified here.
"""

from pathlib import Path
import json
import tomllib
import unittest


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261108-chriz-iwd-encounters-chriz-easy-iwd.toml"
MOD = "chriz-iwd-encounters"
PIN = "chriz-iwd-encounters-0.2.2"
SUCCESSOR = "chriz-iwd-encounters-0.3.0"
TAG = "v0.2.2"
FILENAME = "chriz-iwd-encounters-v0.2.2.zip"
SHA256 = "c9a653aeb61bee0f3b76e230e793b4f835bb2e5ce13de48d5f1dce630a94442c"
LENGTH = 381_722
TAG_COMMIT = "bc8bde6a3150667d5527db089201b11f24ccbff3"


def load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


class Nov08ChrizIwdEncountersV022PinTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.evidence = load(ROOT / EVIDENCE)
        (cls.record,) = cls.evidence["artifacts"]
        cls.recipe = ROOT / "recipes/iwd-current"

    def test_fresh_release_evidence_is_complete(self):
        self.assertEqual(self.evidence["verified_on"], "2026-11-08")
        self.assertIn("replaces chriz-iwd-encounters-0.2.1", self.evidence["scope"])
        self.assertIn("not a WeiDU install or gameplay acceptance", self.evidence["scope"])
        self.assertEqual(self.record["artifact_id"], PIN)
        self.assertEqual(self.record["recipe_profiles"], ["recipes/iwd-current"])
        self.assertEqual(self.record["source_reference"], TAG)
        self.assertEqual(self.record["source_commit"], TAG_COMMIT)
        self.assertEqual(self.record["actual_length"], LENGTH)
        self.assertEqual(self.record["actual_sha256"], SHA256)
        for flag in ("fresh_download", "fresh_extraction", "published_release_verified",
                     "native_component_order_verified"):
            self.assertIs(self.record[flag], True, flag)

    def test_iwd_recipe_supersedes_0_2_2_with_0_3_0_and_keeps_v0_1_9_ledger(self):
        mod = load(self.recipe / f"mods/{MOD}.toml")
        self.assertEqual(mod["artifact_id"], SUCCESSOR)
        self.assertFalse((self.recipe / "artifacts/chriz-iwd-encounters-0.2.1.toml").exists())
        self.assertFalse((self.recipe / f"artifacts/{PIN}.toml").exists())
        (run,) = [run for run in load(self.recipe / "collection.toml")["runs"] if run["mod_id"] == MOD]
        self.assertEqual(run["components"][:2], self.record["native_components"][:2])
        self.assertEqual(run["components"][2], 120)
        self.assertEqual(run["components"][3:-2], self.record["native_components"][2:])
        self.assertEqual(run["components"][-2:], [420, 500])
        # 0.1.9 brought the 0.2.2 pin; later releases keep its ledger entry and record their own.
        version = json.loads((self.recipe / "release.json").read_text(encoding="utf-8"))["version"]
        self.assertTrue((self.recipe / f"releases/v{version}/ledger.toml").is_file(), version)
        ledger = load(self.recipe / "releases/v0.1.9/ledger.toml")
        self.assertIn(PIN, {change["id"] for change in ledger["changes"]})

    def test_iwd_catalog_and_historical_release_text_stay_recorded(self):
        inventory = (ROOT / "docs/curation/iwdee/INVENTORY.md").read_text(encoding="utf-8")
        self.assertIn("Chriz IWD Encounters", inventory)
        self.assertIn(SUCCESSOR, inventory)
        self.assertIn("occupied script slots", inventory)
        notes = (ROOT / "docs/release-notes-iwd-0.1.9.md").read_text(encoding="utf-8")
        self.assertIn("Chriz IWD Encounters 0.2.2", notes)
        self.assertIn("Idol", notes)


if __name__ == "__main__":
    unittest.main()
