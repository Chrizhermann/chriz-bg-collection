"""November 17 intake: Chriz IWD Party Dialog v0.1.0 in Chriz Easy IWD.

Evaluates world and quest dialogue class, kit, and race checks across all living party members in
the current area and hands dialogue speaker attribution and tokens to the qualifying character via
EEex runtime hooks. These are static source/package and recipe assertions only, not a combined
collection install or campaign gameplay acceptance.
"""

from pathlib import Path
import tomllib
import unittest


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261117-chriz-iwd-party-dialog-chriz-easy-iwd.toml"
MOD = "chriz-iwd-party-dialog"
PIN = "chriz-iwd-party-dialog-0.1.0"
TAG = "v0.1.0"
FILENAME = "chriz-iwd-party-dialog-v0.1.0.zip"
SHA256 = "d801101ca396b88b39995185bf4d784777a8f24a23bdb7e4746cda08a3bbe8e2"
TP2_SHA256 = "2811b57764ceb429621916b07e506577f6200e2d0da23ad9b8c0847605266cbf"
LENGTH = 1_282_596
TAG_COMMIT = "33fa3c17ccda5aaf989dc66f9dd3c489dedb94be"
COMPONENTS = [0]


def load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


class Nov17ChrizIwdPartyDialogIntakeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.evidence = load(ROOT / EVIDENCE)
        (cls.record,) = cls.evidence["artifacts"]
        cls.recipe = ROOT / "recipes/iwd-current"

    def test_fresh_release_evidence_is_complete(self) -> None:
        self.assertEqual(self.evidence["verified_on"], "2026-11-17")
        self.assertIn("Chriz IWD Party Dialog v0.1.0", self.evidence["scope"])
        self.assertIn("not a combined full-collection WeiDU install", self.evidence["scope"])
        self.assertEqual(self.record["mod_id"], MOD)
        self.assertEqual(self.record["artifact_id"], PIN)
        self.assertEqual(self.record["recipe_profiles"], ["recipes/iwd-current"])
        self.assertEqual(self.record["source_reference"], TAG)
        self.assertEqual(self.record["source_commit"], TAG_COMMIT)
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

    def test_iwd_recipe_freezes_the_new_artifact_and_default_run(self) -> None:
        mod = load(self.recipe / f"mods/{MOD}.toml")
        artifact = load(self.recipe / f"artifacts/{PIN}.toml")
        self.assertEqual(mod["artifact_id"], PIN)
        self.assertEqual(mod["tp2"], "chriz-iwd-party-dialog/setup-chriz-iwd-party-dialog.tp2")
        self.assertEqual(artifact["version"], TAG.removeprefix("v"))
        self.assertEqual(artifact["source"]["reference"], TAG)
        self.assertEqual(artifact["source"]["expected_filename"], FILENAME)
        self.assertEqual(artifact["archive"]["publish_roots"], ["chriz-iwd-party-dialog"])
        for key, evidence_key in (
            ("url", "immutable_source_url"),
            ("expected_length", "actual_length"),
            ("sha256", "actual_sha256"),
        ):
            self.assertEqual(artifact["source"][key], self.record[evidence_key], key)

        collection = load(self.recipe / "collection.toml")
        runs = collection["runs"]
        run_ids = [run["run_id"] for run in runs]
        (run,) = [run for run in runs if run["mod_id"] == MOD]
        self.assertEqual(run["run_id"], "chriz-iwd-party-dialog-main")
        self.assertEqual(run["phase"], "main")
        self.assertEqual(run["components"], COMPONENTS)
        # November 22: Chriz Multiplayer Fix runs between Chriz Overhead Text and this mod.
        self.assertEqual(
            run_ids.index("chriz-multiplayer-fix-main") + 1,
            run_ids.index("chriz-iwd-party-dialog-main"),
        )
        self.assertEqual(
            run_ids.index("chriz-iwd-party-dialog-main") + 1,
            run_ids.index("chriz-bg-rebalance-main"),
        )

        # BG recipes do not install this IWD-only mod.
        self.assertFalse((ROOT / "manifest/mods/chriz-iwd-party-dialog.toml").exists())
        self.assertFalse(
            (ROOT / "recipes/curated-full-current/mods/chriz-iwd-party-dialog.toml").exists()
        )

    def test_own_mods_curation_docs_and_hotpatch_runtime_files_name_the_pin(self) -> None:
        own_mods = load(ROOT / "manifest/own-mods.toml")
        entry = next(mod for mod in own_mods["mods"] if mod["id"] == MOD)
        self.assertEqual(entry["repository"], "Chrizhermann/chriz-iwd-party-dialog")

        worksheet = (ROOT / "docs/curation/iwdee/WORKSHEET.md").read_text(encoding="utf-8")
        self.assertIn("Chriz IWD Party Dialog 0.1.0", worksheet)
        inventory = (ROOT / "docs/curation/iwdee/INVENTORY.md").read_text(encoding="utf-8")
        self.assertIn(PIN, inventory)

        iwd_catalog = load(ROOT / "hotpatches/iwd/catalog.toml")
        self.assertIn("chriz_iwd_party_dialog.log", iwd_catalog["runtime_files"])

        acceptance = load(self.recipe / "releases/v0.1.0-alpha.1/acceptance.toml")
        record = next(
            row for row in acceptance["evidence"] if row["subject_id"] == "chriz-iwd-party-dialog-main"
        )
        self.assertEqual(record["commit"], "583bf937288674b95bc25815c54674a33a97c42b")

        iwd_ledger = load(self.recipe / "releases/v0.1.14/ledger.toml")
        self.assertEqual(iwd_ledger["supersedes"], "0.1.13")
        self.assertIn(MOD, {change["id"] for change in iwd_ledger["changes"]})
        base = next(row for row in iwd_catalog["bases"] if row["recipe_version"] == "0.1.14")
        self.assertEqual(base["id"], "iwd-0.1.14")
        self.assertEqual(
            base["content_sha256"],
            "d0bb83c08061f3b55306a00df14dd3a6950f151b9a83d5f508951c9cc56ab523",
        )


if __name__ == "__main__":
    unittest.main()
