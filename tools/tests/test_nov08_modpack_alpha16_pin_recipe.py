"""November 8 intake: Chriz BG Modpack v0.2.0-alpha.16 replaces alpha.15.

The tagged workflow published Modpack PR #19's public-main squash commit.  It keeps every
component, run and Recommended choice, but each component writes its decided result over edited
data, another mod's version or a missing owned piece; its XP table is copied as xplevel.2da.
This is static source/package coverage, not a WeiDU installation or gameplay acceptance.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import build_recipe


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261108-chriz-bg-modpack-chriz-easy-bg.toml"
MOD = "chriz-bg-modpack"
PIN = "chriz-bg-modpack-0.2.0-alpha.16"
PREVIOUS = "chriz-bg-modpack-0.2.0-alpha.15"
SUCCESSOR = "chriz-bg-modpack-0.2.0-alpha.19"
TAG = "v0.2.0-alpha.16"
TAG_COMMIT = "ee82f6bcc82074c52b3216453e785cfb1a9ac182"
LENGTH = 9_529_061
SHA256 = "668937ba7ea468834aad1fd1ecc214be72a9b1b0564109a48fc0827dab09e570"
STATIC_COMMIT = "e0afccd7658c307dca82ad24b1891387b1f16b4d"
STATIC_RUNS = {
    "chriz-bg-modpack-bg2",
    "chriz-bg-modpack-edwin-bg2",
    "chriz-bg-modpack-baeloth-bg2",
    "chriz-bg-modpack-scrolls-bg2",
    "chriz-bg-modpack-pre-continuity-bg2",
    "chriz-bg-modpack-continuity-bg2",
    "chriz-bg-modpack-late-companions-bg2",
}


def load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


class Nov08ModpackAlpha16PinTests(unittest.TestCase):
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

    def test_evidence_freezes_the_tagged_public_main_release(self):
        self.assertEqual(self.evidence["verified_on"], "2026-11-08")
        self.assertEqual(self.evidence["generator"], "tools/prepare_own_mod_pin.py")
        for phrase in (
            "Modpack PR #19", "tagged release workflow", "replaces chriz-bg-modpack-0.2.0-alpha.15",
            "every component writes its decided result", "xplevel.2da",
            "not a WeiDU install or gameplay acceptance",
        ):
            self.assertIn(phrase, self.evidence["scope"])
        self.assertEqual(self.record["mod_id"], MOD)
        self.assertEqual(self.record["artifact_id"], PIN)
        self.assertEqual(self.record["recipe_profiles"], ["manifest", "recipes/curated-full-current"])
        self.assertEqual(self.record["source_reference"], TAG)
        self.assertEqual(self.record["source_commit"], TAG_COMMIT)
        self.assertEqual(self.record["actual_length"], LENGTH)
        self.assertEqual(self.record["actual_sha256"], SHA256)
        for flag in ("fresh_download", "fresh_extraction", "published_release_verified",
                     "native_component_order_verified"):
            self.assertIs(self.record[flag], True, flag)

    def test_both_profiles_keep_alpha16_as_history(self):
        for directory in (ROOT / "manifest", ROOT / "recipes/curated-full-current", self.generated):
            with self.subTest(profile=directory):
                mod = load(directory / f"mods/{MOD}.toml")
                artifact = load(directory / f"artifacts/{PIN}.toml")
                previous = load(directory / f"artifacts/{PREVIOUS}.toml")
                self.assertEqual(mod["artifact_id"], SUCCESSOR)
                self.assertEqual(artifact["version"], TAG.removeprefix("v"))
                self.assertEqual(artifact["source"]["reference"], TAG)
                self.assertEqual(artifact["source"]["expected_length"], LENGTH)
                self.assertEqual(artifact["source"]["sha256"], SHA256)
                self.assertEqual(artifact["archive"], previous["archive"])
                self.assertEqual(artifact["source"]["redirect_hosts"], previous["source"]["redirect_hosts"])
                collection = load(directory / "collection.toml")
                native = set(self.record["native_components"])
                # Alpha.17 adds component 680 in TP2 order; alpha.18 adds component 470 after 460;
                # alpha.19 adds component 695 after 680; earlier declarations keep their order.
                for run in (run for run in collection["runs"] if run["mod_id"] == MOD):
                    self.assertTrue(set(run["components"]) - {680, 470, 695} <= native, run["run_id"])

    def test_history_documents_name_alpha16_and_pin_list_moves_on(self):
        catalog = (ROOT / "docs/curation/components/CHRIZ-BG-MODPACK.md").read_text(encoding="utf-8")
        for phrase in ("**v0.2.0-alpha.16**", "Modpack PR #19", "xplevel.2da", EVIDENCE):
            self.assertIn(phrase, catalog)
        changelog = (ROOT / "docs/plans/2026-10-14-next-release-changelogs.md").read_text(encoding="utf-8")
        draft = changelog[changelog.index("## Collection 0.1.2 (draft)"):changelog.index("## Installer changelog")]
        for phrase in ("**Chriz BG Modpack writes every selected result as decided**",
                       "Chriz BG Modpack v0.2.0-alpha.16", "`artifact:chriz-bg-modpack-0.2.0-alpha.16`"):
            self.assertIn(phrase, draft)
        pin_list = (ROOT / "docs/pin-list-2.7.md").read_text(encoding="utf-8")
        self.assertNotIn(f"| chriz-bg-modpack | {TAG.removeprefix('v')} |", pin_list)


if __name__ == "__main__":
    unittest.main()
