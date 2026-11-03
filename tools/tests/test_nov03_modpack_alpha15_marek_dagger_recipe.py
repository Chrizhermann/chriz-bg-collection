"""November 3 intake: Chriz BG Modpack v0.2.0-alpha.15 for CEBG 0.1.1 (Christopher, 2026-11-03).

Alpha.15 replaces alpha.14 and adds only component 670: both of Marek's creature files carry
one Boomerang Dagger +2 (DAGG11) in the backpack, BG1's only source of it since BG1 NPC
Project left the defaults. 670 changes nothing when the Kivan quest's Shipwreck's Coast
script already creates the dagger, so it needs no conflict with BG1 NPC Project. It is a
default in Recommended and runs in the main post-EET_end Modpack run after 660.

Static pin, evidence, component, selection, order and release-text checks for both recipe
profiles only. Nothing here is a game installation or gameplay acceptance.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import _effective_features, build_recipe


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261103-modpack-alpha15.toml"
PREVIOUS_EVIDENCE = "manifest/evidence/release-intake-20261026.toml"
PROFILES = ["manifest", "recipes/curated-full-current"]
MOD = "chriz-bg-modpack"
PIN = "chriz-bg-modpack-0.2.0-alpha.15"
PREVIOUS = "chriz-bg-modpack-0.2.0-alpha.14"
TAG = "v0.2.0-alpha.15"
COMMIT = "0920504b954ff8d7e8a3640f58acbe4b33242b85"
URL = ("https://github.com/Chrizhermann/chriz-bg-modpack/releases/download/v0.2.0-alpha.15/"
       "chriz-bg-modpack-v0.2.0-alpha.15.zip")
LENGTH = 10_555_318
SHA256 = "a6031332c10c0bd5ee72b63ff1e1a870b1ef1b5e056f59e3542949e0c4ba7d8d"
FEATURE = "feature:chriz-bg-modpack:component-670"
LABEL = "Marek: carry the Boomerang Dagger +2 in BG1 (drops it when he dies)"
RUN = "chriz-bg-modpack-bg2"
LEDGERS = ["manifest/releases/v0.1.1/ledger.toml",
           "recipes/curated-full-current/releases/v0.1.1/ledger.toml"]


def _load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def _selected(collection: dict, selection: dict) -> set[tuple[str, int]]:
    state = _effective_features(collection, selection)
    return {(ref["run_id"], ref["component"]) for feature in collection["features"]
            if state[feature["id"]] for ref in feature.get("components", [])}


class Nov03ModpackAlpha15Tests(unittest.TestCase):
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
        (cls.record,) = cls.evidence["artifacts"]
        (cls.previous,) = [record for record in _load(ROOT / PREVIOUS_EVIDENCE)["artifacts"]
                           if record["mod_id"] == MOD]

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def test_evidence_records_the_fresh_published_download(self):
        self.assertEqual(self.evidence["verified_on"], "2026-11-03")
        self.assertEqual(self.evidence["evidence_kind"], "published-release-download-and-native-order-verification")
        for phrase in ("Modpack PR #18", "fresh cache", "cache hit", "byte for byte",
                       "exactly one commit ahead", "not yet tested in a game",
                       "not a combined WeiDU install or gameplay acceptance"):
            self.assertIn(phrase, self.evidence["scope"])
        record = self.record
        self.assertEqual(record["mod_id"], MOD)
        self.assertEqual(record["artifact_id"], PIN)
        self.assertEqual(record["recipe_profiles"], PROFILES)
        self.assertEqual(record["source_reference"], TAG)
        self.assertEqual(record["source_commit"], COMMIT)
        self.assertEqual(record["immutable_source_url"], URL)
        self.assertEqual(record["actual_length"], LENGTH)
        self.assertEqual(record["actual_sha256"], SHA256)
        for flag in ("fresh_download", "fresh_extraction", "published_release_verified",
                     "native_component_order_verified"):
            self.assertIs(record[flag], True, flag)
        # Alpha.15 adds only 670, declared between 660 and 650; every other component keeps
        # its place.
        self.assertEqual(self.previous["artifact_id"], PREVIOUS)
        self.assertNotEqual(record["tp2_sha256"], self.previous["tp2_sha256"])
        native = list(self.previous["native_components"])
        native.insert(native.index(660) + 1, 670)
        self.assertEqual(record["native_components"], native)

    def test_both_profiles_pin_alpha15_and_keep_alpha14(self):
        for profile_name, directory in self.directories:
            artifact = _load(directory / f"artifacts/{PIN}.toml")
            previous = _load(directory / f"artifacts/{PREVIOUS}.toml")
            with self.subTest(profile=str(directory)):
                self.assertEqual(artifact["id"], PIN)
                self.assertEqual(artifact["name"], previous["name"])
                self.assertEqual(artifact["version"], TAG.removeprefix("v"))
                self.assertEqual(artifact["acquisition"], "fetch-only")
                self.assertEqual(artifact["source"]["kind"], "github-release")
                self.assertEqual(artifact["source"]["url"], URL)
                self.assertEqual(artifact["source"]["reference"], TAG)
                self.assertEqual(artifact["source"]["expected_filename"], URL.rsplit("/", 1)[1])
                self.assertEqual(artifact["source"]["expected_length"], LENGTH)
                self.assertEqual(artifact["source"]["sha256"], SHA256)
                self.assertEqual(artifact["provenance"]["reviewed_on"], "2026-11-03")
                # Same layout, limits and redirect host as alpha.14, which stays for
                # installations made from the earlier recipe.
                self.assertEqual(artifact["archive"], previous["archive"])
                self.assertEqual(artifact["source"]["redirect_hosts"], previous["source"]["redirect_hosts"])
                self.assertIn(profile_name, self.record["recipe_profiles"])
                mod = _load(directory / f"mods/{MOD}.toml")
                self.assertEqual(mod["artifact_id"], PIN)
                self.assertIn(mod["tp2"], artifact["archive"]["tp2_paths"])
        # The checked-in curated artifacts are the generator's copies of the authored pins.
        for artifact_id in (PIN, PREVIOUS):
            name = f"artifacts/{artifact_id}.toml"
            self.assertEqual((ROOT / "recipes/curated-full-current" / name).read_bytes(),
                             (ROOT / "manifest" / name).read_bytes())

    def test_670_is_a_recommended_default_after_660_and_after_scs(self):
        for _, directory in self.directories:
            collection = _load(directory / "collection.toml")
            preset = _load(directory / "presets/chris-recommended.toml")["selections"]
            features = {feature["id"]: feature for feature in collection["features"]}
            runs = [run["run_id"] for run in collection["runs"]]
            (run,) = [run for run in collection["runs"] if run["run_id"] == RUN]
            mod = _load(directory / f"mods/{MOD}.toml")
            with self.subTest(profile=str(directory)):
                feature = features[FEATURE]
                self.assertEqual(feature["decision"], "default")
                self.assertEqual(feature["readiness"], "ready")
                self.assertEqual(feature["parent"], f"mod:{MOD}")
                self.assertEqual(feature["category"], "items")
                self.assertEqual(feature["requires"], ["feature:eet:mandatory-components"])
                self.assertEqual(feature["components"], [{"run_id": RUN, "component": 670}])
                # 670 skips itself when the Kivan quest gives the dagger: no conflict rule.
                self.assertNotIn("conflicts", feature)
                self.assertIn("Boomerang Dagger +2", feature["title"])
                self.assertEqual(preset[FEATURE], "on")
                self.assertEqual(run["components"][-2:], [660, 670])
                self.assertEqual(run["phase"], "post-eet-end")
                # After SCS, which changes Marek's creature files.
                self.assertLess(runs.index("stratagems-bg2"), runs.index(RUN))
                names = {component["id"]: component["name"] for component in mod["components"]}
                self.assertEqual(names[670], LABEL)
                self.assertEqual(list(names).index(670), list(names).index(660) + 1)
                # Selected in Recommended, and still selected with BG1 NPC Project ticked.
                baseline = _selected(collection, preset)
                self.assertIn((RUN, 670), baseline)
                self.assertIn((RUN, 670), _selected(collection, {**preset, "mod:bg1npc": "on"}))
                self.assertNotIn((RUN, 670), _selected(collection, {**preset, FEATURE: "off"}))

    def test_the_011_ledger_tells_players(self):
        for path in LEDGERS:
            with self.subTest(ledger=path):
                ledger = _load(ROOT / path)
                self.assertEqual(ledger["version"], "0.1.1")
                changes = {change["id"]: change for change in ledger["changes"]}
                self.assertEqual(list(changes), ["bg1npc-optional", "modpack-marek-boomerang-dagger"])
                change = changes["modpack-marek-boomerang-dagger"]
                self.assertEqual(change["title"], "Marek carries the Boomerang Dagger +2")
                self.assertEqual(change["covers"], [f"artifact:{PIN}", f"feature:{FEATURE}", f"run:{RUN}"])
                self.assertEqual(change["save_applicability"], "next-playthrough")
                self.assertEqual(change["urgency"], "recommended")
                for phrase in ("Marek", "Boomerang Dagger +2", "drops it when he dies",
                               "Blushing Mermaid", "Kivan quest", "Marek carries none",
                               "(Chriz BG Modpack alpha.15)"):
                    self.assertIn(phrase, change["summary"])
        self.assertEqual((ROOT / LEDGERS[0]).read_bytes().replace(b"\r\n", b"\n"),
                         (ROOT / LEDGERS[1]).read_bytes().replace(b"\r\n", b"\n"))

    def test_catalog_pin_list_and_follow_ups_name_the_pin(self):
        catalog = (ROOT / "docs/curation/components/CHRIZ-BG-MODPACK.md").read_text(encoding="utf-8")
        self.assertIn(f"| 670 | {LABEL} | chriz-bg-modpack: Gameplay tweaks | | | default |", catalog)
        self.assertIn("65 current or historical entries; the collection exposes 48.", catalog)
        folded = " ".join(catalog.split())
        for phrase in ("**v0.2.0-alpha.15** (published 2026-11-03)",
                       "Marek's Boomerang Dagger `670` (alpha.15) is **default (Christopher, 2026-11-03)**",
                       "`670` has to move before it"):
            self.assertIn(phrase, folded)
        pin_list = (ROOT / "docs/pin-list-2.7.md").read_text(encoding="utf-8")
        self.assertEqual(pin_list.count(f"| {MOD} | {TAG.removeprefix('v')} | `{MOD}` | `{SHA256[:12]}…` |"), 2)
        self.assertNotIn(f"| {MOD} | 0.2.0-alpha.14 |", pin_list)
        follow_ups = " ".join((ROOT / "docs/curation/components/FOLLOW_UPS.md").read_text(encoding="utf-8").split())
        for phrase in (EVIDENCE, "The carrier is Marek (Christopher, 2026-11-03)",
                       "Still open: the Randomiser's BG1 pool."):
            self.assertIn(phrase, follow_ups)


if __name__ == "__main__":
    unittest.main()
