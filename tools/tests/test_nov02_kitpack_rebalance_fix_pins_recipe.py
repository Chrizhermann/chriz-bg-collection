"""November 2 intake: two content fixes for CEBG 0.1.0 (Christopher, 2026-11-02).

- The Artisan's Kitpack fork v4.81a-dev.85928f9-chriz.6 replaces chriz.5 for the main, NPC
  and tweak installers: the white and greater white werewolves' attack 1 files had no frames
  in the five base directions, which crashes BG2:EE 2.7.3.0 near a wall. Only those two
  files and the three VERSION lines change.
- BG Rebalance v0.8.9 replaces v0.8.8: Pierce Magic 210 and Spellstrike 211 write their
  descriptions in Spell Revisions' layout and repoint the scrolls, the wand and sibling
  spells. No mechanic and no component of the main TP2 changes; its texts-only tail
  components 212/213 ship in a separate package that the collection does not install.

Static pin, evidence, component, selection, order and release-text checks for both recipe
profiles only. Nothing here is a game installation or gameplay acceptance.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import _effective_features, build_recipe


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261102-kitpack-chriz6-rebalance-089.toml"
PROFILES = ["manifest", "recipes/curated-full-current"]
KITPACK_MODS = ("artisanskitpack", "artisanskitpack-npc", "artisanskitpack-tweak")
# Per pin: the new artifact, the superseded one, the previous record, tag, tag commit,
# archive name, length and SHA-256.
PINS = {
    "kitpack": {
        "mods": KITPACK_MODS,
        "pin": "artisans-kitpack-4.81a-dev.85928f9-chriz.6",
        "previous": "artisans-kitpack-4.81a-dev.85928f9-chriz.5",
        "previous_evidence": "manifest/evidence/release-intake-20261029-kitpack-chriz5.toml",
        "tag": "v4.81a-dev.85928f9-chriz.6",
        "commit": "d70d213a5dd55be27f7aab6e65e0eefdcddae687",
        "url": "https://github.com/Chrizhermann/The-Artisan-s-Kitpack-Chriz-Balance-Patch/releases/download/"
               "v4.81a-dev.85928f9-chriz.6/ArtisansKitpack-Chriz-Balance-Patch-v4.81a-dev.85928f9-chriz.6.zip",
        "length": 89_856_136,
        "sha256": "9644f01cdeeebf56912723e53267e91feb4404fb5c8d7305e0b112189095777e",
    },
    "rebalance": {
        "mods": ("chriz-bg-rebalance",),
        "pin": "chriz-bg-rebalance-0.8.9",
        "previous": "chriz-bg-rebalance-0.8.8",
        "previous_evidence": "manifest/evidence/release-intake-20261029-rebalance-088.toml",
        "tag": "v0.8.9",
        "commit": "bb3f11b7a2d631b50254eb556ca477197033fa52",
        "url": "https://github.com/Chrizhermann/chriz-bg-rebalance/releases/download/v0.8.9/"
               "chriz-bg-rebalance-v0.8.9-windows.zip",
        "length": 1_844_195,
        "sha256": "da03fe9e2a36f5af155311197e68d6f82ead270426015363fa161ad56e8ea6f2",
    },
}
REBALANCE_RUN = "chriz-bg-rebalance-bg2"
LEDGERS = ["manifest/releases/v0.1.0/ledger.toml",
           "recipes/curated-full-current/releases/v0.1.0/ledger.toml"]


def _load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


class Nov02FixPinsTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        cls.output = Path(cls.temp.name) / "recipe"
        # Synthetic evidence marker used only in this temporary test output.
        build_recipe(ROOT, cls.output, "0" * 40)
        # Each recipe directory with the evidence profile it must match.
        cls.directories = [
            ("manifest", ROOT / "manifest"),
            ("recipes/curated-full-current", ROOT / "recipes/curated-full-current"),
            ("recipes/curated-full-current", cls.output),
        ]
        cls.evidence = _load(ROOT / EVIDENCE)
        cls.records = {record["mod_id"]: record for record in cls.evidence["artifacts"]}
        cls.previous = {}
        for pin in PINS.values():
            for record in _load(ROOT / pin["previous_evidence"])["artifacts"]:
                if record["mod_id"] in pin["mods"]:
                    cls.previous[record["mod_id"]] = record

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def test_evidence_records_the_fresh_published_downloads(self):
        self.assertEqual(self.evidence["verified_on"], "2026-11-02")
        self.assertEqual(self.evidence["evidence_kind"], "published-release-download-and-native-order-verification")
        for phrase in ("fork PR #17", "Rebalance PR #25", "fresh cache", "cache hit",
                       "byte for byte", "apart from their CRLF line endings", "issue 77",
                       "not a combined WeiDU install or gameplay acceptance"):
            self.assertIn(phrase, self.evidence["scope"])
        self.assertEqual(set(self.records), {mod for pin in PINS.values() for mod in pin["mods"]})
        for pin in PINS.values():
            for mod_id in pin["mods"]:
                with self.subTest(mod=mod_id):
                    record, previous = self.records[mod_id], self.previous[mod_id]
                    self.assertEqual(record["artifact_id"], pin["pin"])
                    self.assertEqual(record["recipe_profiles"], PROFILES)
                    self.assertNotIn("native_order_recipe_profiles", record)
                    self.assertEqual(record["source_reference"], pin["tag"])
                    self.assertEqual(record["source_commit"], pin["commit"])
                    self.assertEqual(record["immutable_source_url"], pin["url"])
                    self.assertEqual(record["actual_length"], pin["length"])
                    self.assertEqual(record["actual_sha256"], pin["sha256"])
                    for flag in ("fresh_download", "fresh_extraction", "published_release_verified",
                                 "native_component_order_verified"):
                        self.assertIs(record[flag], True, flag)
                    # Only the VERSION line of each TP2 changed: new TP2 bytes, same components
                    # in the same order.
                    self.assertEqual(previous["artifact_id"], pin["previous"])
                    self.assertNotEqual(record["tp2_sha256"], previous["tp2_sha256"])
                    self.assertEqual(record["native_components"], previous["native_components"])

    def test_both_profiles_pin_the_fixes_and_keep_the_previous_pins(self):
        for profile_name, directory in self.directories:
            for pin in PINS.values():
                artifact = _load(directory / f"artifacts/{pin['pin']}.toml")
                previous = _load(directory / f"artifacts/{pin['previous']}.toml")
                with self.subTest(profile=str(directory), pin=pin["pin"]):
                    self.assertEqual(artifact["id"], pin["pin"])
                    self.assertEqual(artifact["name"], previous["name"])
                    self.assertEqual(artifact["version"], pin["tag"].removeprefix("v"))
                    self.assertEqual(artifact["acquisition"], "fetch-only")
                    self.assertEqual(artifact["source"]["kind"], "github-release")
                    self.assertEqual(artifact["source"]["url"], pin["url"])
                    self.assertEqual(artifact["source"]["reference"], pin["tag"])
                    self.assertEqual(artifact["source"]["expected_filename"], pin["url"].rsplit("/", 1)[1])
                    self.assertEqual(artifact["source"]["expected_length"], pin["length"])
                    self.assertEqual(artifact["source"]["sha256"], pin["sha256"])
                    self.assertEqual(artifact["provenance"]["reviewed_on"], "2026-11-02")
                    # Same layout, limits and redirect host as the superseded pin, which stays
                    # for installations made from the earlier recipe.
                    self.assertEqual(artifact["archive"], previous["archive"])
                    self.assertEqual(artifact["source"]["redirect_hosts"], previous["source"]["redirect_hosts"])
                    for mod_id in pin["mods"]:
                        self.assertIn(profile_name, self.records[mod_id]["recipe_profiles"])
                        mod = _load(directory / f"mods/{mod_id}.toml")
                        self.assertEqual(mod["artifact_id"], pin["pin"], mod_id)
                        self.assertIn(mod["tp2"], artifact["archive"]["tp2_paths"])
        # The checked-in curated artifacts are the generator's copies of the authored pins.
        for pin in PINS.values():
            for artifact_id in (pin["pin"], pin["previous"]):
                name = f"artifacts/{artifact_id}.toml"
                self.assertEqual((ROOT / "recipes/curated-full-current" / name).read_bytes(),
                                 (ROOT / "manifest" / name).read_bytes())

    def test_components_runs_and_the_recommended_210_and_211_stay(self):
        for _, directory in self.directories:
            collection = _load(directory / "collection.toml")
            preset = _load(directory / "presets/chris-recommended.toml")["selections"]
            state = _effective_features(collection, preset)
            selected = {(ref["run_id"], ref["component"]) for feature in collection["features"]
                        if state[feature["id"]] for ref in feature.get("components", [])}
            with self.subTest(profile=str(directory)):
                for mod_id, record in self.records.items():
                    mod = _load(directory / f"mods/{mod_id}.toml")
                    native = record["native_components"]
                    self.assertTrue({component["id"] for component in mod["components"]} <= set(native), mod_id)
                    for run in (run for run in collection["runs"] if run["mod_id"] == mod_id):
                        self.assertEqual(run["components"], [c for c in native if c in run["components"]],
                                         run["run_id"])
                # 0.1.0 installs Pierce Magic 210 and Spellstrike 211 fresh, as before; the tail's
                # texts-only 212/213 are neither offered nor part of the main TP2.
                rebalance = _load(directory / "mods/chriz-bg-rebalance.toml")
                offered = {component["id"] for component in rebalance["components"]}
                self.assertTrue({210, 211} <= offered)
                self.assertFalse({212, 213} & offered)
                self.assertFalse({212, 213} & set(self.records["chriz-bg-rebalance"]["native_components"]))
                for component in (210, 211):
                    self.assertIn((REBALANCE_RUN, component), selected)

    def test_pin_list_and_changelog_draft_name_the_fixes(self):
        pin_list = (ROOT / "docs/pin-list-2.7.md").read_text(encoding="utf-8")
        for pin in PINS.values():
            self.assertEqual(pin_list.count(f"| {pin['tag'].removeprefix('v')} |"), 2, pin["pin"])
            self.assertEqual(pin_list.count(f"`{pin['sha256'][:12]}…`"), 2, pin["pin"])
        changelog = " ".join((ROOT / "docs/plans/2026-10-14-next-release-changelogs.md")
                             .read_text(encoding="utf-8").split())
        for phrase in ("**The Shapeshifter's white and greater white werewolves no longer crash the game**",
                       "(Artisan's Kitpack fork chriz.6)", "**Pierce Magic and Spellstrike describe what they do**",
                       "(Rebalance v0.8.9)", "BG Rebalance v0.8.9 (was v0.8.2)",
                       "The Artisan's Kitpack fork v4.81a-dev.85928f9-chriz.6 (was chriz.1)",
                       "**Artisan's Kitpack fork chriz.6** (pinned 2026-11-02",
                       "**Rebalance v0.8.9** (pinned 2026-11-02"):
            self.assertIn(phrase, changelog)


if __name__ == "__main__":
    unittest.main()
