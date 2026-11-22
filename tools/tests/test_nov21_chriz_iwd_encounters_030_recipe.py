"""November 21 intake and release: Chriz IWD Encounters v0.3.0 in Chriz Easy IWD 0.1.15 (0.1.16)."""

import json
import tomllib
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261121-chriz-iwd-encounters-chriz-easy-iwd.toml"
MOD = "chriz-iwd-encounters"
PIN = "chriz-iwd-encounters-0.3.0"
SUCCESSOR = "chriz-iwd-encounters-0.3.2"
TAG = "v0.3.0"
FILENAME = "chriz-iwd-encounters-v0.3.0.zip"
SHA256 = "856a69b7e22d9a891ec26f7b37a076f659431b9ca698d956f616566ac2a27d90"
TP2_SHA256 = "da6b48e81c1a4d6db6bab89395269692eda36aa0f064be74117eb59c2e9d9812"
LENGTH = 678_296
PIN_COMMIT = "ec3710e9e4fbba7bdfcd9dda36ba132fb4f8927d"
DIGEST_0116 = "ba389b0325f180c53d14c65d2c4638e9a129fdee84ca3627e7e53ca2212b09d4"
COMPONENTS = [100, 110, 120, 200, 300, 310, 320, 330, 340, 350, 360, 370, 380, 390, 400, 410, 420, 500]


def load(rel: str) -> dict:
    return tomllib.loads((ROOT / rel).read_text(encoding="utf-8"))


class Nov21ChrizIwdEncounters030RecipeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.evidence = load(EVIDENCE)
        (cls.record,) = cls.evidence["artifacts"]
        cls.recipe = "recipes/iwd-current"

    def test_fresh_release_evidence_matches_v0_3_0(self) -> None:
        self.assertEqual(self.evidence["verified_on"], "2026-11-21")
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

    def test_iwd_recipe_supersedes_0_3_0_while_preserving_components_and_0_1_16_history(self) -> None:
        mod = load(f"{self.recipe}/mods/{MOD}.toml")
        self.assertEqual(mod["artifact_id"], SUCCESSOR)
        self.assertFalse((ROOT / f"{self.recipe}/artifacts/{PIN}.toml").exists())

        collection = load(f"{self.recipe}/collection.toml")
        (run,) = [r for r in collection["runs"] if r["mod_id"] == MOD]
        self.assertEqual(run["components"], COMPONENTS)
        by_id = {feat["id"]: feat for feat in collection["features"]}
        for cid in (120, 420):
            feat = by_id[f"feature:{MOD}:component-{cid}"]
            self.assertEqual(feat["decision"], "default")
            self.assertEqual(feat["parent"], f"mod:{MOD}")
        opt = by_id[f"feature:{MOD}:component-500"]
        self.assertEqual(opt["decision"], "optional")
        self.assertEqual(opt["parent"], f"mod:{MOD}")

    def test_release_0_1_16_ledger_acceptance_hotpatch_and_notes(self) -> None:
        tauri_iwd = json.loads((ROOT / "app/src-tauri/tauri.iwd.conf.json").read_text(encoding="utf-8"))
        self.assertGreaterEqual(tuple(int(x) for x in tauri_iwd["version"].split(".")), (0, 1, 15))
        release = json.loads((ROOT / f"{self.recipe}/release.json").read_text(encoding="utf-8"))
        self.assertTrue((ROOT / f"{self.recipe}/releases/v{release['version']}/ledger.toml").is_file(),
                        release["version"])

        ledger = load(f"{self.recipe}/releases/v0.1.16/ledger.toml")
        self.assertEqual((ledger["recipe_id"], ledger["version"], ledger["supersedes"]),
                         ("chriz-easy-iwd", "0.1.16", "0.1.15"))
        (change,) = ledger["changes"]
        self.assertEqual(change["id"], PIN)
        self.assertIn(f"artifact:{PIN}", change["covers"])
        self.assertIn("feature:feature:chriz-iwd-encounters:component-120", change["covers"])
        self.assertIn("feature:feature:chriz-iwd-encounters:component-420", change["covers"])
        self.assertIn("feature:feature:chriz-iwd-encounters:component-500", change["covers"])
        self.assertIn("run:chriz-iwd-encounters-main", change["covers"])

        acceptance = load(f"{self.recipe}/releases/v0.1.0-alpha.1/acceptance.toml")["evidence"]
        record = next(row for row in acceptance if row["subject_id"] == "chriz-iwd-encounters-main")
        self.assertEqual(record["status"], "accepted")

        bases = load("hotpatches/iwd/catalog.toml")["bases"]
        (base,) = [b for b in bases if b["recipe_version"] == "0.1.16"]
        self.assertEqual(base["id"], "iwd-0.1.16")
        self.assertEqual(base["content_sha256"], DIGEST_0116)

        inventory = (ROOT / "docs/curation/iwdee/INVENTORY.md").read_text(encoding="utf-8")
        self.assertIn(SUCCESSOR, inventory)
        notes = (ROOT / "docs/release-notes-iwd-0.1.15.md").read_text(encoding="utf-8")
        self.assertIn("App **0.1.15** bundles collection **0.1.16**", notes)
        self.assertIn("**145 default", notes)
        self.assertIn("Chriz.Easy.IWD_0.1.15_x64-setup.exe", notes)


if __name__ == "__main__":
    unittest.main()
