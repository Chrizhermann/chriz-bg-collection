"""November 22 intake: Chriz IWD Encounters v0.3.2 in Chriz Easy IWD collection 0.1.18.

v0.3.2 keeps the components of v0.3.1 and changes 300 (Poquelin's finale dispel restored),
420 (mixed elite squads, Kaylessa) and 500 (Heart of Winter and Trials of the Luremaster).
Static source/package and recipe assertions only, not campaign gameplay acceptance.
"""

import tomllib
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261122-chriz-iwd-encounters-v0.3.2-chriz-easy-iwd.toml"
MOD = "chriz-iwd-encounters"
PIN = "chriz-iwd-encounters-0.3.2"
PRIOR_PIN = "chriz-iwd-encounters-0.3.1"
TAG = "v0.3.2"
FILENAME = "chriz-iwd-encounters-v0.3.2.zip"
URL = f"https://github.com/Chrizhermann/chriz-iwd-encounters/releases/download/{TAG}/{FILENAME}"
SHA256 = "077040e15da1bc7fc0e468e9258c97d920aeea3b817f4de9792ffc5e0198023d"
TP2_SHA256 = "55b63d66bcb1f84d084c0e34c948575173c515a698f5ca5ae46c0b1f3565731f"
LENGTH = 706_913
TAG_COMMIT = "20dad715555bdb0bff9735554f520a3d15c3eeaf"
PIN_COMMIT = "10f1bc33f6565b9631206694e5037ff7cb6cf68b"
COMPONENTS = [100, 110, 120, 200, 300, 310, 320, 330, 340, 350, 360, 370, 380, 390, 400, 410, 420, 500]
RECIPE = "recipes/iwd-current"


def load(rel: str) -> dict:
    return tomllib.loads((ROOT / rel).read_text(encoding="utf-8"))


class Nov22ChrizIwdEncounters032RecipeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.evidence = load(EVIDENCE)
        (cls.record,) = cls.evidence["artifacts"]

    def test_fresh_release_evidence_matches_v0_3_2(self) -> None:
        self.assertEqual(self.evidence["verified_on"], "2026-11-22")
        self.assertEqual(
            self.evidence["evidence_kind"],
            "published-release-download-and-native-order-verification",
        )
        self.assertIn("Chriz IWD Encounters v0.3.2", self.evidence["scope"])
        self.assertEqual(self.record["mod_id"], MOD)
        self.assertEqual(self.record["artifact_id"], PIN)
        self.assertEqual(self.record["recipe_profiles"], [RECIPE])
        self.assertEqual(self.record["source_reference"], TAG)
        self.assertEqual(self.record["source_commit"], TAG_COMMIT)
        self.assertEqual(self.record["immutable_source_url"], URL)
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

    def test_iwd_recipe_pins_v0_3_2_with_the_same_selection(self) -> None:
        mod = load(f"{RECIPE}/mods/{MOD}.toml")
        self.assertEqual(mod["artifact_id"], PIN)
        artifact = load(f"{RECIPE}/artifacts/{PIN}.toml")
        self.assertEqual(artifact["version"], "0.3.2")
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
        decisions = {
            number: features[f"feature:chriz-iwd-encounters:component-{number}"]["decision"]
            for number in COMPONENTS
        }
        self.assertEqual(decisions[500], "optional")
        self.assertTrue(all(decisions[n] != "optional" for n in COMPONENTS if n != 500), decisions)
        # The player texts follow v0.3.2.
        finale = features["feature:chriz-iwd-encounters:component-300"]["description"]
        self.assertIn("Poquelin's transformation dispels the party as in the base game", finale)
        self.assertNotIn("no longer dispels", finale)
        campaign = features["feature:chriz-iwd-encounters:component-420"]
        self.assertIn("six mixed elite squads", campaign["description"])
        self.assertIn("Kaylessa", campaign["description"])
        self.assertNotIn("Drasus", campaign["title"] + campaign["description"])
        expansion = features["feature:chriz-iwd-encounters:component-500"]["description"]
        self.assertIn("Polar Bear Spirit", expansion)

    def test_release_0_1_18_ledger_and_static_evidence(self) -> None:
        ledger = load(f"{RECIPE}/releases/v0.1.18/ledger.toml")
        self.assertEqual((ledger["version"], ledger["supersedes"]), ("0.1.18", "0.1.17"))
        change = next(change for change in ledger["changes"] if change["id"] == PIN)
        self.assertEqual(change["save_applicability"], "new-game-only")
        self.assertEqual(change["urgency"], "recommended")
        self.assertIn("Kaylessa", change["summary"])
        self.assertIn("Poquelin", change["summary"])
        self.assertEqual(
            change["covers"],
            [
                f"artifact:{PRIOR_PIN}",
                f"artifact:{PIN}",
                "feature:feature:chriz-iwd-encounters:component-300",
                "feature:feature:chriz-iwd-encounters:component-420",
                "feature:feature:chriz-iwd-encounters:component-500",
                "run:chriz-iwd-encounters-main",
            ],
        )

        acceptance = load(f"{RECIPE}/releases/v0.1.0-alpha.1/acceptance.toml")["evidence"]
        records = [row for row in acceptance if row["subject_id"] == "chriz-iwd-encounters-main"]
        self.assertEqual(len(records), 1)
        (record,) = records
        self.assertEqual(
            (record["kind"], record["status"], record["date"], record["commit"]),
            ("static-test", "accepted", "2026-11-22", PIN_COMMIT),
        )
        self.assertIn("v0.3.2", record["scope"])

        # No hotpatch brings Encounters v0.3.2 to existing installations: it changes new games only.
        catalog = load("hotpatches/iwd/catalog.toml")
        self.assertFalse(any(hotpatch.get("artifact") == PIN for hotpatch in catalog["hotpatches"]))

        inventory = (ROOT / "docs/curation/iwdee/INVENTORY.md").read_text(encoding="utf-8")
        self.assertIn(f"{PIN} (100, 110, 120, 200, 300–420; optional 500)", inventory)
        changelog = (ROOT / "docs/plans/2026-10-14-next-release-changelogs.md").read_text(
            encoding="utf-8"
        )
        self.assertIn("(Chriz IWD Encounters v0.3.2,\n  collection 0.1.18)", changelog.replace("\r\n", "\n"))
        self.assertIn("Chriz IWD Encounters v0.3.2 (was v0.3.1)", changelog)


if __name__ == "__main__":
    unittest.main()
