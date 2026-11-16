"""November 16 intake: Chriz Overhead Text v0.1.0 in both Chriz Easy installers.

Keeps spoken overhead lines visible while Tab / Reveal Details or overhead HP labels are active,
stacking the spoken box cleanly above the nameplate or HP bar via EEex runtime hooks. These are
static source/package and recipe assertions only, not a combined collection install or campaign
gameplay acceptance.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import _effective_features, build_recipe


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = (
    "manifest/evidence/"
    "release-intake-20261116-chriz-overhead-text-chriz-easy-bg-chriz-easy-iwd.toml"
)
MOD = "chriz-overhead-text"
PIN = "chriz-overhead-text-0.1.0"
TAG = "v0.1.0"
FILENAME = "chriz-overhead-text-v0.1.0.zip"
SHA256 = "a7023258fbc6f5621663ed40fd7927877f45a074d896002b36993c88b132ef9a"
TP2_SHA256 = "c0412909760ce209e6e3023c9711411eacad67c6bb96a69cf4fe1b622b1cdbb2"
LENGTH = 1_277_567
TAG_COMMIT = "2bbd76b803a76e18ab10621b4d0fc32050b5309a"
COMPONENTS = [0]


def load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


class Nov16ChrizOverheadTextIntakeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
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
    def tearDownClass(cls) -> None:
        cls.temp.cleanup()

    def test_fresh_release_evidence_is_complete(self) -> None:
        self.assertEqual(self.evidence["verified_on"], "2026-11-16")
        self.assertIn("Chriz Overhead Text v0.1.0", self.evidence["scope"])
        self.assertIn("not a combined full-collection WeiDU install", self.evidence["scope"])
        for record in self.records.values():
            self.assertEqual(record["mod_id"], MOD)
            self.assertEqual(record["artifact_id"], PIN)
            self.assertEqual(record["source_reference"], TAG)
            self.assertEqual(record["source_commit"], TAG_COMMIT)
            self.assertEqual(record["actual_length"], LENGTH)
            self.assertEqual(record["actual_sha256"], SHA256)
            self.assertEqual(record["tp2_sha256"], TP2_SHA256)
            self.assertEqual(record["native_components"], COMPONENTS)
            for flag in (
                "fresh_download",
                "fresh_extraction",
                "published_release_verified",
                "native_component_order_verified",
            ):
                self.assertIs(record[flag], True, flag)

    def test_all_shipped_recipes_freeze_the_new_artifact_and_default_run(self) -> None:
        profiles = [
            ("manifest", ROOT / "manifest", "chriz-overhead-text-bg2", "post-eet-end"),
            (
                "recipes/curated-full-current",
                ROOT / "recipes/curated-full-current",
                "chriz-overhead-text-bg2",
                "post-eet-end",
            ),
            (
                "recipes/curated-full-current",
                self.generated,
                "chriz-overhead-text-bg2",
                "post-eet-end",
            ),
            ("recipes/iwd-current", ROOT / "recipes/iwd-current", "chriz-overhead-text-main", "main"),
        ]
        for profile, directory, expected_run_id, expected_phase in profiles:
            with self.subTest(profile=directory):
                record = self.records[profile]
                mod = load(directory / f"mods/{MOD}.toml")
                artifact = load(directory / f"artifacts/{PIN}.toml")
                self.assertEqual(mod["artifact_id"], PIN)
                self.assertEqual(mod["tp2"], "chriz_overhead_text/setup-chriz_overhead_text.tp2")
                self.assertEqual(artifact["version"], TAG.removeprefix("v"))
                self.assertEqual(artifact["source"]["reference"], TAG)
                self.assertEqual(artifact["source"]["expected_filename"], FILENAME)
                self.assertEqual(artifact["archive"]["publish_roots"], ["chriz_overhead_text"])
                for key, evidence_key in (
                    ("url", "immutable_source_url"),
                    ("expected_length", "actual_length"),
                    ("sha256", "actual_sha256"),
                ):
                    self.assertEqual(artifact["source"][key], record[evidence_key], key)
                collection = load(directory / "collection.toml")
                runs = collection["runs"]
                run_ids = [run["run_id"] for run in runs]
                (run,) = [run for run in runs if run["mod_id"] == MOD]
                self.assertEqual(run["run_id"], expected_run_id)
                self.assertEqual(run["phase"], expected_phase)
                self.assertEqual(run["components"], COMPONENTS)
                if expected_phase == "post-eet-end":
                    self.assertLess(
                        run_ids.index("chriz-bg-rebalance-bg2"),
                        run_ids.index(expected_run_id),
                    )
                    self.assertEqual(
                        run_ids.index(expected_run_id) + 1,
                        run_ids.index("eeexremote-bg2"),
                    )
                    preset = load(directory / "presets/chris-recommended.toml")["selections"]
                    effective = _effective_features(collection, preset)
                    self.assertTrue(effective["feature:chriz-overhead-text:component-0"])
                else:
                    self.assertEqual(
                        run_ids.index("buffbot-main") + 1,
                        run_ids.index(expected_run_id),
                    )
                    self.assertLess(
                        run_ids.index(expected_run_id),
                        run_ids.index("eeexremote-main"),
                    )

    def test_own_mods_catalog_changelog_and_pin_list_name_the_pin(self) -> None:
        own_mods = load(ROOT / "manifest/own-mods.toml")
        entry = next(mod for mod in own_mods["mods"] if mod["id"] == MOD)
        self.assertEqual(entry["repository"], "Chrizhermann/chriz-overhead-text")
        catalog = (ROOT / "docs/curation/components/CHRIZ_OVERHEAD_TEXT.md").read_text(
            encoding="utf-8"
        )
        self.assertIn(f"Listed at **{TAG}**", catalog)
        self.assertIn(f"`{TAG_COMMIT[:7]}`", catalog)
        changelog = (ROOT / "docs/plans/2026-10-14-next-release-changelogs.md").read_text(
            encoding="utf-8"
        )
        self.assertIn("Chriz Overhead Text v0.1.0 (new)", changelog)
        self.assertIn("**Chriz Overhead Text v0.1.0**", changelog)
        pin_list = (ROOT / "docs/pin-list-2.7.md").read_text(encoding="utf-8")
        self.assertEqual(
            pin_list.count(f"| Chriz Overhead Text | 0.1.0 | `{MOD}` | `{SHA256[:12]}…` |"),
            2,
        )
        for acceptance_path, subject_id in (
            ("manifest/releases/v0.1.0-alpha.1/acceptance.toml", "chriz-overhead-text-bg2"),
            (
                "recipes/curated-full-current/releases/v0.1.0-alpha.1/acceptance.toml",
                "chriz-overhead-text-bg2",
            ),
            (
                "recipes/iwd-current/releases/v0.1.0-alpha.1/acceptance.toml",
                "chriz-overhead-text-main",
            ),
        ):
            acceptance = load(ROOT / acceptance_path)
            record = next(
                row for row in acceptance["evidence"] if row["subject_id"] == subject_id
            )
            self.assertEqual(record["commit"], "626e0bce8cdf7678df32354b2309daf5e7dc657f")
        iwd_ledger = load(ROOT / "recipes/iwd-current/releases/v0.1.13/ledger.toml")
        self.assertEqual(iwd_ledger["supersedes"], "0.1.10")
        self.assertIn(MOD, {change["id"] for change in iwd_ledger["changes"]})
        iwd_catalog = load(ROOT / "hotpatches/iwd/catalog.toml")
        base = next(row for row in iwd_catalog["bases"] if row["recipe_version"] == "0.1.13")
        self.assertEqual(base["id"], "iwd-0.1.13")
        self.assertEqual(
            base["content_sha256"],
            "3540064fb278f30e240eedec4d0a2e43aebc6f16e3ba0fa5ee11fd4630c521b8",
        )


if __name__ == "__main__":
    unittest.main()
