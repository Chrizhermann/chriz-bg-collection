"""November 22 intake and release: Chriz IWD Encounters v0.3.1 in Chriz Easy IWD 0.1.16 (0.1.17)."""

import json
import tomllib
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261122-chriz-iwd-encounters-chriz-easy-iwd.toml"
MOD = "chriz-iwd-encounters"
PIN = "chriz-iwd-encounters-0.3.1"
PRIOR_PIN = "chriz-iwd-encounters-0.3.0"
TAG = "v0.3.1"
SHA256 = "63b69f39c8f06f7ce1dac151e418f1e17ca84d095cbd11bb6f35045c0a008eef"
TP2_SHA256 = "3e2b1799b8f235087eb69a0e868c2a688ec46b9b03b24db9681c47eafeb91317"
LENGTH = 680_143
PIN_COMMIT = "21ba55dbe0091c97df77886140b7f1765a0ce37a"
DIGEST_0117 = "6aceb183900fb7b3640dbedc8f4dd84e931c7cfe1549aa0c31cf115726d397c1"
COMPONENTS = [100, 110, 120, 200, 300, 310, 320, 330, 340, 350, 360, 370, 380, 390, 400, 410, 420, 500]


def load(rel: str) -> dict:
    return tomllib.loads((ROOT / rel).read_text(encoding="utf-8"))


class Nov22ChrizIwdEncounters031RecipeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.evidence = load(EVIDENCE)
        (cls.record,) = cls.evidence["artifacts"]
        cls.recipe = "recipes/iwd-current"

    def test_fresh_release_evidence_matches_v0_3_1(self) -> None:
        self.assertEqual(self.evidence["verified_on"], "2026-11-22")
        self.assertEqual(self.record["mod_id"], MOD)
        self.assertEqual(self.record["artifact_id"], PIN)
        self.assertEqual(self.record["recipe_profiles"], [self.recipe])
        self.assertEqual(self.record["source_reference"], TAG)
        self.assertEqual(self.record["actual_length"], LENGTH)
        self.assertEqual(self.record["actual_sha256"], SHA256)
        self.assertEqual(self.record["tp2_sha256"], TP2_SHA256)
        self.assertEqual(self.record["native_components"], COMPONENTS)
        for flag in (
            "fresh_download",
            "fresh_extraction",
            "published_release_verified",
            "native_component_order_verified",
        ):
            self.assertIs(self.record[flag], True, flag)

    def test_iwd_recipe_pins_v0_3_1(self) -> None:
        mod = load(f"{self.recipe}/mods/{MOD}.toml")
        self.assertEqual(mod["artifact_id"], PIN)
        self.assertTrue((ROOT / f"{self.recipe}/artifacts/{PIN}.toml").is_file())
        self.assertFalse((ROOT / f"{self.recipe}/artifacts/{PRIOR_PIN}.toml").exists())

        collection = load(f"{self.recipe}/collection.toml")
        (run,) = [r for r in collection["runs"] if r["mod_id"] == MOD]
        self.assertEqual(run["components"], COMPONENTS)

    def test_release_0_1_17_ledger_acceptance_hotpatch_and_notes(self) -> None:
        tauri_iwd = json.loads((ROOT / "app/src-tauri/tauri.iwd.conf.json").read_text(encoding="utf-8"))
        self.assertGreaterEqual(tuple(int(x) for x in tauri_iwd["version"].split(".")), (0, 1, 16))
        release = json.loads((ROOT / f"{self.recipe}/release.json").read_text(encoding="utf-8"))
        self.assertGreaterEqual(tuple(int(x) for x in release["version"].split(".")), (0, 1, 17))
        self.assertTrue((ROOT / f"{self.recipe}/releases/v{release['version']}/ledger.toml").is_file(),
                        release["version"])

        ledger = load(f"{self.recipe}/releases/v0.1.17/ledger.toml")
        self.assertEqual((ledger["recipe_id"], ledger["version"], ledger["supersedes"]),
                         ("chriz-easy-iwd", "0.1.17", "0.1.16"))
        (change,) = ledger["changes"]
        self.assertEqual(change["id"], PIN)
        self.assertIn(f"artifact:{PRIOR_PIN}", change["covers"])
        self.assertIn(f"artifact:{PIN}", change["covers"])
        self.assertIn("feature:feature:chriz-iwd-encounters:component-300", change["covers"])
        self.assertIn("run:chriz-iwd-encounters-main", change["covers"])

        acceptance = load(f"{self.recipe}/releases/v0.1.0-alpha.1/acceptance.toml")["evidence"]
        record = next(row for row in acceptance if row["subject_id"] == "chriz-iwd-encounters-main")
        self.assertEqual(record["status"], "accepted")
        self.assertEqual(record["date"], "2026-11-22")
        self.assertEqual(record["commit"], PIN_COMMIT)

        bases = load("hotpatches/iwd/catalog.toml")["bases"]
        (base,) = [b for b in bases if b["recipe_version"] == "0.1.17"]
        self.assertEqual(base["id"], "iwd-0.1.17")
        self.assertEqual(base["label"], "Chriz Easy IWD 0.1.16")
        self.assertEqual(base["content_sha256"], DIGEST_0117)

        inventory = (ROOT / "docs/curation/iwdee/INVENTORY.md").read_text(encoding="utf-8")
        self.assertIn(PIN, inventory)
        notes = (ROOT / "docs/release-notes-iwd-0.1.16.md").read_text(encoding="utf-8")
        self.assertIn("App **0.1.16** bundles collection **0.1.17**", notes)
