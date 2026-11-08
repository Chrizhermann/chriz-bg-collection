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

    def test_both_profiles_pin_alpha16_and_keep_alpha15(self):
        for directory in (ROOT / "manifest", ROOT / "recipes/curated-full-current", self.generated):
            with self.subTest(profile=directory):
                mod = load(directory / f"mods/{MOD}.toml")
                artifact = load(directory / f"artifacts/{PIN}.toml")
                previous = load(directory / f"artifacts/{PREVIOUS}.toml")
                self.assertEqual(mod["artifact_id"], PIN)
                self.assertEqual(artifact["version"], TAG.removeprefix("v"))
                self.assertEqual(artifact["source"]["reference"], TAG)
                self.assertEqual(artifact["source"]["expected_length"], LENGTH)
                self.assertEqual(artifact["source"]["sha256"], SHA256)
                self.assertEqual(artifact["archive"], previous["archive"])
                self.assertEqual(artifact["source"]["redirect_hosts"], previous["source"]["redirect_hosts"])
                collection = load(directory / "collection.toml")
                native = set(self.record["native_components"])
                for run in (run for run in collection["runs"] if run["mod_id"] == MOD):
                    self.assertTrue(set(run["components"]) <= native, run["run_id"])

    def test_static_records_and_player_documents_name_alpha16(self):
        for relative in (
            "manifest/releases/v0.1.0-alpha.1/acceptance.toml",
            "recipes/curated-full-current/releases/v0.1.0-alpha.1/acceptance.toml",
        ):
            records = {item["subject_id"]: item for item in load(ROOT / relative)["evidence"]}
            for run in STATIC_RUNS:
                with self.subTest(record=relative, run=run):
                    record = records[run]
                    self.assertEqual(record["commit"], STATIC_COMMIT)
                    self.assertEqual(record["test_artifact"],
                                     "tools/tests/test_nov08_modpack_alpha16_pin_recipe.py")
                    self.assertEqual(record["date"], "2026-11-08")
                    self.assertEqual(
                        record["scope"],
                        "Static alpha.16 pin, source, component and order assertions only; "
                        "fresh source/package verification is in "
                        "release-intake-20261108-chriz-bg-modpack-chriz-easy-bg.toml; "
                        "not a WeiDU installation or gameplay acceptance.",
                    )
        catalog = (ROOT / "docs/curation/components/CHRIZ-BG-MODPACK.md").read_text(encoding="utf-8")
        for phrase in ("**v0.2.0-alpha.16**", "Modpack PR #19", "xplevel.2da", EVIDENCE):
            self.assertIn(phrase, catalog)
        changelog = (ROOT / "docs/plans/2026-10-14-next-release-changelogs.md").read_text(encoding="utf-8")
        draft = changelog[changelog.index("## Collection 0.1.2 (draft)"):changelog.index("## Installer changelog")]
        for phrase in ("**Chriz BG Modpack writes every selected result as decided**",
                       "Chriz BG Modpack v0.2.0-alpha.16", "`artifact:chriz-bg-modpack-0.2.0-alpha.16`"):
            self.assertIn(phrase, draft)
        source = next(line for line in (ROOT / "manifest/mod-sources.tsv").read_text(encoding="utf-8").splitlines()
                      if line.startswith("CHRIZ-BG-MODPACK\t"))
        for phrase in (TAG, TAG_COMMIT, str(LENGTH), SHA256, "xplevel.2da"):
            self.assertIn(phrase, source)
        pin_list = (ROOT / "docs/pin-list-2.7.md").read_text(encoding="utf-8")
        self.assertEqual(pin_list.count(f"| chriz-bg-modpack | {TAG.removeprefix('v')} | `{MOD}` | `{SHA256[:12]}…` |"), 2)


if __name__ == "__main__":
    unittest.main()
