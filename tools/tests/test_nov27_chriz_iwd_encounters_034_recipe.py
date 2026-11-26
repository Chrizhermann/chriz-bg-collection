"""November 27 intake and release: Chriz IWD Encounters v0.3.4 in Chriz Easy IWD 0.1.17 (0.1.22).

Pins Chriz IWD Encounters v0.3.4 (Tag commit 6161252, PR #9), preserving Talonite &
ELVEN_SOLDIER quest death variables on AR4002/AR5403 squad actors and aligning all
thinned-area quest kill counters (Dragon's Eye Level 2 Talonite >5/<6, Severed Hand
ELVEN_SOLDIER >11, Dorn's Deep 8001_Salamanders >8 and 8008_Salamanders >4), plus
fixing CIE4xx/CIE5xx mastery stat stacking, marking bespoke elite gear and AI buff
potions undroppable, stretching Kresselack AR3503 wave timers, removing all
Bombardier Beetles, and adding six apex encounters. Existing 0.1.15-0.1.16 installs
receive the standalone quest-counter repair as hotpatch component 425.
"""

import json
import tomllib
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261127-chriz-iwd-encounters-chriz-easy-iwd.toml"
MOD = "chriz-iwd-encounters"
PIN = "chriz-iwd-encounters-0.3.4"
PRIOR_PIN = "chriz-iwd-encounters-0.3.2"
TAG = "v0.3.4"
FILENAME = "chriz-iwd-encounters-v0.3.4.zip"
URL = f"https://github.com/Chrizhermann/chriz-iwd-encounters/releases/download/{TAG}/{FILENAME}"
SHA256 = "8f6b5abf7f8a5af6ce22d99cabd0c4e5c188c09872d676f8542cf8456b5f83c6"
TP2_SHA256 = "85500dede00bbea7536e52158fac6ab71ecb8990d967ca25b52b3a35e89e72b3"
LENGTH = 756_790
TAG_COMMIT = "6161252d7263bb4dc94b825421133e7c8e7fccc9"
PIN_COMMIT = "0000000000000000000000000000000000000000"
DIGEST_0122 = "31c158ea070afe5cd516d42593b9f133f58842ae4e4ecd56b033e4069fbb8e1e"
COMPONENTS = [100, 110, 120, 200, 300, 310, 320, 330, 340, 350, 360, 370, 380, 390, 400, 410, 420, 500]
HOTPATCH_ID = "chriz-iwd-encounters-0.3.4-quest-counters"
HOTPATCH_COMPONENTS = [100, 110, 120, 200, 300, 310, 320, 330, 340, 350, 360, 370, 380, 390, 400, 410, 420, 425, 500]
RECIPE = "recipes/iwd-current"


def load(rel: str) -> dict:
    return tomllib.loads((ROOT / rel).read_text(encoding="utf-8"))


class Nov27ChrizIwdEncounters034RecipeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.evidence = load(EVIDENCE)
        (cls.record,) = cls.evidence["artifacts"]

    def test_fresh_release_evidence_matches_v0_3_4(self) -> None:
        self.assertEqual(self.evidence["verified_on"], "2026-11-27")
        self.assertEqual(
            self.evidence["evidence_kind"],
            "published-release-download-and-native-order-verification",
        )
        self.assertIn("Chriz IWD Encounters v0.3.4", self.evidence["scope"])
        self.assertEqual(self.record["mod_id"], MOD)
        self.assertEqual(self.record["artifact_id"], PIN)
        self.assertEqual(self.record["recipe_profiles"], [RECIPE])
        self.assertEqual(self.record["source_reference"], TAG)
        self.assertEqual(self.record["source_commit"], TAG_COMMIT)
        self.assertEqual(self.record["immutable_source_url"], URL)
        self.assertEqual(self.record["actual_length"], LENGTH)
        self.assertEqual(self.record["actual_sha256"], SHA256)
        self.assertEqual(self.record["tp2_sha256"], TP2_SHA256)
        self.assertEqual(self.record["native_components"], HOTPATCH_COMPONENTS)
        for flag in (
            "fresh_download",
            "fresh_extraction",
            "published_release_verified",
            "native_component_order_verified",
        ):
            self.assertIs(self.record[flag], True, flag)

    def test_iwd_recipe_pins_v0_3_4_and_omits_hotpatch_only_425(self) -> None:
        mod = load(f"{RECIPE}/mods/{MOD}.toml")
        self.assertEqual(mod["artifact_id"], PIN)
        self.assertEqual([item["id"] for item in mod["components"]], COMPONENTS)
        self.assertNotIn(425, [item["id"] for item in mod["components"]])
        artifact = load(f"{RECIPE}/artifacts/{PIN}.toml")
        self.assertEqual(artifact["version"], "0.3.4")
        self.assertEqual(artifact["source"]["url"], URL)
        self.assertEqual(artifact["source"]["reference"], TAG)
        self.assertEqual(artifact["source"]["expected_filename"], FILENAME)
        self.assertEqual(artifact["source"]["expected_length"], LENGTH)
        self.assertEqual(artifact["source"]["sha256"], SHA256)
        self.assertFalse((ROOT / f"{RECIPE}/artifacts/{PRIOR_PIN}.toml").exists())

        collection = load(f"{RECIPE}/collection.toml")
        (run,) = [r for r in collection["runs"] if r["mod_id"] == MOD]
        self.assertEqual(run["components"], COMPONENTS)
        features = {feature["id"]: feature for feature in collection["features"]}
        self.assertNotIn("feature:chriz-iwd-encounters:component-425", features)

    def test_release_0_1_22_ledger_hotpatch_425_and_docs(self) -> None:
        tauri_iwd = json.loads((ROOT / "app/src-tauri/tauri.iwd.conf.json").read_text(encoding="utf-8"))
        self.assertEqual(tauri_iwd["version"], "0.1.17")
        release = json.loads((ROOT / f"{RECIPE}/release.json").read_text(encoding="utf-8"))
        self.assertEqual(release["version"], "0.1.22")
        ledger = load(f"{RECIPE}/releases/v0.1.22/ledger.toml")
        self.assertEqual((ledger["recipe_id"], ledger["version"], ledger["supersedes"]), ("chriz-easy-iwd", "0.1.22", "0.1.21"))
        change = next(change for change in ledger["changes"] if change["id"] == PIN)
        self.assertEqual(change["save_applicability"], "new-game-only")
        self.assertEqual(change["urgency"], "recommended")
        self.assertIn("Talonite", change["summary"])
        self.assertIn("48s", change["summary"])
        self.assertIn("component 425", change["summary"])
        self.assertEqual(
            change["covers"],
            [
                f"artifact:{PRIOR_PIN}",
                f"artifact:{PIN}",
                "feature:feature:chriz-iwd-encounters:component-420",
                "feature:feature:chriz-iwd-encounters:component-500",
                "run:chriz-iwd-encounters-main",
            ],
        )

        hotpatch_artifact = load(f"hotpatches/iwd/artifacts/{PIN}.toml")
        self.assertEqual(hotpatch_artifact["version"], "0.3.4")
        self.assertEqual(hotpatch_artifact["source"]["expected_length"], LENGTH)
        self.assertEqual(hotpatch_artifact["source"]["sha256"], SHA256)
        hotpatch_mod = load(f"hotpatches/iwd/mods/{PIN}.toml")
        self.assertEqual(hotpatch_mod["artifact_id"], PIN)
        self.assertEqual([item["id"] for item in hotpatch_mod["components"]], HOTPATCH_COMPONENTS)

        catalog = load("hotpatches/iwd/catalog.toml")
        bases = {base["id"]: base for base in catalog["bases"]}
        self.assertEqual(bases["iwd-0.1.22"]["recipe_version"], "0.1.22")
        self.assertEqual(bases["iwd-0.1.22"]["label"], "Chriz Easy IWD 0.1.17")
        self.assertEqual(bases["iwd-0.1.22"]["content_sha256"], DIGEST_0122)

        hotpatch = next(row for row in catalog["hotpatches"] if row["id"] == HOTPATCH_ID)
        self.assertEqual(hotpatch["artifact"], PIN)
        self.assertEqual(hotpatch["mod"], PIN)
        self.assertEqual(hotpatch["components"], [425])
        self.assertEqual(hotpatch["requires_components"], [{"mod": MOD, "any_of": [420]}])
        self.assertEqual(
            hotpatch["bases"],
            ["iwd-0.1.16", "iwd-0.1.17", "iwd-0.1.18", "iwd-0.1.19", "iwd-0.1.20", "iwd-0.1.21"],
        )
        self.assertEqual(len(hotpatch["changes"]), 3)
        for item in hotpatch["changes"]:
            self.assertEqual(item["save_applicability"], "current-save")

        acceptance = load(f"{RECIPE}/releases/v0.1.0-alpha.1/acceptance.toml")["evidence"]
        (record,) = [row for row in acceptance if row["subject_id"] == "chriz-iwd-encounters-main"]
        self.assertEqual(
            (record["kind"], record["status"], record["date"], record["commit"]),
            ("static-test", "accepted", "2026-11-27", PIN_COMMIT),
        )
        self.assertIn("v0.3.4", record["scope"])

        inventory = (ROOT / "docs/curation/iwdee/INVENTORY.md").read_text(encoding="utf-8")
        self.assertIn(f"{PIN} (100, 110, 120, 200, 300–420; optional 500; hotpatch-only 425)", inventory)
        changelog = (ROOT / "docs/plans/2026-10-14-next-release-changelogs.md").read_text(encoding="utf-8")
        self.assertIn("Chriz IWD Encounters v0.3.4 (was v0.3.2", changelog)
        self.assertTrue((ROOT / "docs/release-notes-iwd-0.1.17.md").is_file())


if __name__ == "__main__":
    unittest.main()
