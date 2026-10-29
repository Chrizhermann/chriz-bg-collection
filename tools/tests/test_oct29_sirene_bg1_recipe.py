"""October 29 intake: Sirene NPC for BG:EE (BG1 and Siege of Dragonspear) as a default.

Christopher, 2026-10-29: the mod by the author of Sirene NPC for BG2 becomes a default in
alpha.22 after an EET lab test, pinned at unreleased master commit 1c86000 because release
v3.1 supports BG:EE only. The picks mirror the BG2 mod: main component 0, the Sirick portrait
3 and True Paladin 6 by default, Cavalier, Inquisitor and Undead Hunter 7-9 optional, the
other portraits 1, 2, 4 and 5 not offered. Both Sirene mods install after Artisan's Kitpack's
main run, BG1 first, so that the Kitpack's Martyr kit stays the only one and the BG2 mod
continues Sirene from SoD into SoA on EET.

Static pin, evidence, component, selection and order checks for both recipe profiles only.
Nothing here is a game installation or gameplay acceptance.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import _effective_features, build_recipe
from tools.curation_audit import Decision, RowKey, load_catalogs, load_curation_map


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261029-sirene-bg1.toml"
PROFILES = ["manifest", "recipes/curated-full-current"]
MOD = "sirene-bg1"
RUN = "sirene-bg1-bg2"
COMMIT = "1c8600029a9ac31395a61b5d4d03f98337dcf83e"
PIN = f"sirene-bg1-{COMMIT}"
URL = f"https://codeload.github.com/TheArtisanBG/Sirene-BG1/zip/{COMMIT}"
SHA256 = "f46b035395c4310103e9be2d1623a113e6f3aa66f520624c30c7f355518f4eb9"
LENGTH = 3_734_521
TP2 = "Sirene/Setup-Sirene.tp2"
OFFERED = [0, 3, 6, 7, 8, 9]
DEFAULTS = {0, 3, 6}
CLASSES = [6, 7, 8, 9]
NAMES = {
    0: "Sirene NPC for Baldur's Gate: Enhanced Edition",
    3: "Alternate portrait: Sirick, Light Armor",
    6: "True Paladin",
    7: "Cavalier",
    8: "Inquisitor",
    9: "Undead Hunter",
}


def _load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def _load_profile(directory: Path) -> tuple[dict, dict]:
    collection = _load(directory / "collection.toml")
    preset = _load(directory / "presets/chris-recommended.toml")["selections"]
    return collection, preset


def _feature(component: int) -> str:
    return f"feature:{MOD}:component-{component}"


class Oct29SireneBg1RecipeTests(unittest.TestCase):
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
        cls.profiles = {
            "curated": (cls.output, *_load_profile(cls.output)),
            "fallback": (ROOT / "manifest", *_load_profile(ROOT / "manifest")),
        }
        cls.evidence = _load(ROOT / EVIDENCE)
        (cls.record,) = cls.evidence["artifacts"]

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def _each_profile(self):
        for name, (directory, collection, preset) in self.profiles.items():
            with self.subTest(profile=name):
                yield name, directory, collection, preset

    @staticmethod
    def _selected(collection, preset, **changes):
        state = _effective_features(collection, {**preset, **changes})
        return {
            (ref["run_id"], ref["component"])
            for feature in collection["features"]
            if state[feature["id"]]
            for ref in feature.get("components", [])
        }

    def test_evidence_records_the_fresh_commit_archive_download_and_source_audit(self):
        self.assertEqual(self.evidence["verified_on"], "2026-10-29")
        self.assertEqual(self.evidence["evidence_kind"], "published-commit-archive-download-and-source-audit")
        record = self.record
        self.assertEqual(record["mod_id"], MOD)
        self.assertEqual(record["artifact_id"], PIN)
        self.assertEqual(record["recipe_profiles"], PROFILES)
        self.assertEqual(record["source_reference"], COMMIT)
        self.assertEqual(record["source_commit"], COMMIT)
        self.assertEqual(record["immutable_source_url"], URL)
        self.assertEqual(record["actual_length"], LENGTH)
        self.assertEqual(record["actual_sha256"], SHA256)
        self.assertRegex(record["tp2_sha256"], r"^[0-9a-f]{64}$")
        self.assertEqual(record["published_roots"], ["Sirene"])
        # WeiDU numbers the ten components 0-9 in declaration order (no DESIGNATED).
        self.assertEqual(record["native_components"], list(range(10)))
        for flag in ("fresh_download", "fresh_extraction", "native_component_order_verified"):
            self.assertIs(record[flag], True, flag)
        # An unreleased master commit: there is no published release to verify.
        self.assertIs(record["published_release_verified"], False)

    def test_both_profiles_pin_the_eet_ready_commit_archive(self):
        for profile_name, directory in self.directories:
            with self.subTest(profile=str(directory)):
                self.assertIn(profile_name, self.record["recipe_profiles"])
                mod = _load(directory / f"mods/{MOD}.toml")
                self.assertEqual(mod["artifact_id"], PIN)
                self.assertEqual((mod["tp2"], mod["language"], mod["weidu_artifact_id"], mod["invocation_mode"]),
                                 (TP2, 0, "weidu-249-amd64", "setup-name"))
                self.assertEqual({component["id"]: component["name"] for component in mod["components"]}, NAMES)
                self.assertEqual([component["id"] for component in mod["components"]], OFFERED)
                artifact = _load(directory / f"artifacts/{PIN}.toml")
                self.assertEqual((artifact["id"], artifact["version"], artifact["acquisition"]),
                                 (PIN, COMMIT, "fetch-only"))
                self.assertEqual(artifact["source"]["kind"], "github-commit-zip")
                self.assertEqual(artifact["source"]["expected_filename"], f"Sirene-BG1-{COMMIT}.zip")
                self.assertEqual(artifact["source"]["redirect_hosts"], [])
                for key, measured in (("sha256", "actual_sha256"), ("expected_length", "actual_length"),
                                      ("url", "immutable_source_url"), ("reference", "source_reference")):
                    self.assertEqual(artifact["source"][key], self.record[measured], key)
                self.assertEqual(artifact["archive"]["root_rule"], "single-wrapper")
                self.assertEqual(artifact["archive"]["publish_roots"], self.record["published_roots"])
                self.assertEqual(artifact["archive"]["tp2_paths"], [mod["tp2"]])
                self.assertEqual(artifact["provenance"]["url"],
                                 f"https://github.com/TheArtisanBG/Sirene-BG1/commit/{COMMIT}")
        # The checked-in curated copies are the generator's copies of the authored files.
        for name in (f"artifacts/{PIN}.toml", f"mods/{MOD}.toml"):
            self.assertEqual((ROOT / "recipes/curated-full-current" / name).read_bytes(),
                             (ROOT / "manifest" / name).read_bytes())
        ledger = (ROOT / "manifest/mod-sources.tsv").read_text(encoding="utf-8").splitlines()
        (row,) = [line.split("\t") for line in ledger if line.startswith("SIRENE\t")]
        self.assertEqual((row[1], row[3], row[5], row[7]),
                         ("third-party", f"commit {COMMIT}", "github-commit-zip", URL))

    def test_runs_after_kitpack_main_and_before_sirene_bg2_in_every_profile(self):
        for _, directory in self.directories:
            with self.subTest(profile=str(directory)):
                runs = _load(directory / "collection.toml")["runs"]
                order = [run["run_id"] for run in runs]
                (run,) = [run for run in runs if run["mod_id"] == MOD]
                self.assertEqual(run["run_id"], RUN)
                self.assertEqual(run["phase"], "main")
                self.assertEqual(run["components"], OFFERED)
                self.assertEqual(run["args"], [])
                position = order.index
                # Kitpack Martyr 3005 is installed first, so both Sirene mods skip their own
                # copy of the kit; the BG1 mod's copy would hide the kit from the player.
                self.assertLess(position("artisanskitpack-main-bg2"), position(RUN))
                # On EET the BG2 mod continues Sirene only when c0sirene.cre already exists.
                self.assertLess(position(RUN), position("sirene-bg2"))
                self.assertLess(position("sirene-bg2"), position("artisanskitpack-npc-bg2"))
                for later in ("iepbanters-bg2", "crossmodbg2-bg2", "eet-end-bg2",
                              "spell-rev-npc-spellbooks-bg2"):
                    self.assertLess(position(RUN), position(later), later)
                    self.assertLess(position("sirene-bg2"), position(later), later)

    def test_recommended_selects_the_main_component_portrait_and_true_paladin(self):
        for _, _, collection, preset in self._each_profile():
            features = {feature["id"]: feature for feature in collection["features"]}
            offered = sorted(ref["component"] for feature in features.values()
                             for ref in feature.get("components", []) if ref["run_id"] == RUN)
            self.assertEqual(offered, OFFERED)
            for component in OFFERED:
                feature = features[_feature(component)]
                self.assertEqual(feature["components"], [{"run_id": RUN, "component": component}])
                self.assertEqual(feature["readiness"], "ready")
                self.assertEqual(feature["decision"], "default" if component in DEFAULTS else "optional")
                self.assertEqual(preset.get(_feature(component), "off"),
                                 "on" if component in DEFAULTS else "off")
                if component:
                    self.assertEqual(feature["parent"], _feature(0))
                if component in CLASSES:
                    self.assertEqual({conflict["feature_id"] for conflict in feature["conflicts"]},
                                     {_feature(other) for other in CLASSES if other != component})
            main = features[_feature(0)]
            self.assertEqual(main["category"], "npcs")
            self.assertIn("Siege of Dragonspear", main["title"])
            self.assertIn("BG2", main["description"])
            self.assertEqual(features[_feature(3)]["category"], "visuals")

            baseline = self._selected(collection, preset)
            self.assertEqual({component for run_id, component in baseline if run_id == RUN}, DEFAULTS)
            self.assertIn(("sirene-bg2", 0), baseline)
            # Kitpack Martyr 3005 is part of Chris's Recommended.
            self.assertIn(("artisanskitpack-main-bg2", 3005), baseline)
            for alternate in (7, 8, 9):
                swapped = self._selected(collection, preset,
                                         **{_feature(6): "off", _feature(alternate): "on"})
                self.assertEqual(baseline - swapped, {(RUN, 6)}, alternate)
                self.assertEqual(swapped - baseline, {(RUN, alternate)}, alternate)
            without = self._selected(collection, preset, **{_feature(0): "off"})
            self.assertFalse({ref for ref in without if ref[0] == RUN})
            self.assertEqual(baseline - without, {(RUN, component) for component in DEFAULTS})

    def test_catalog_map_and_changelog_record_the_decision(self):
        rows = {row.component_id: row for row in load_catalogs(ROOT / "docs/curation/components")
                if row.catalog == "SIRENE_BG1"}
        self.assertEqual(set(rows), set(range(10)))
        for component, row in rows.items():
            expected = (Decision.DEFAULT if component in DEFAULTS
                        else Decision.OPTIONAL if component in OFFERED else Decision.EXCLUDED)
            self.assertEqual(row.decision, expected, component)
        self.assertEqual(rows[3].component, "Alternate 1 (Sirick, Light Armor)")
        self.assertEqual({rows[component].subgroup for component in CLASSES},
                         {"Choose an alternate class for Sirene?"})

        curation_map = load_curation_map(ROOT / "manifest/curation-map.toml")
        (catalog,) = [entry for entry in curation_map.catalogs if entry.catalog == "SIRENE_BG1"]
        self.assertEqual(catalog.excluded, (1, 2, 4, 5))
        (choice,) = [group for group in curation_map.choice_groups if group.catalog == "SIRENE_BG1"]
        self.assertEqual(choice.options, tuple(RowKey("SIRENE_BG1", component) for component in CLASSES))
        self.assertEqual(choice.default, "SIRENE_BG1:6")
        (fixed,) = [group for group in curation_map.fixed_groups if group.catalog == "SIRENE_BG1"]
        self.assertEqual(fixed.options, (RowKey("SIRENE_BG1", 3),))

        text = (ROOT / "docs/curation/components/SIRENE_BG1.md").read_text(encoding="utf-8")
        for phrase in (f"`{COMMIT}`", "v3.1", "`3005`", "`visible = 0`", "`AR0903_e.baf`",
                       "`bc0siren.dlg`", "Christopher, 2026-10-29"):
            self.assertIn(phrase, text)
        changelog = (ROOT / "docs/plans/2026-10-14-next-release-changelogs.md").read_text(encoding="utf-8")
        for phrase in ("**Sirene in BG1 and Siege of Dragonspear** — **Default.**",
                       "**The Martyr paladin kit** (Default, unchanged choice) now comes from Artisan's Kitpack",
                       "Sirene NPC for BG:EE: development version `1c86000` (new)"):
            self.assertIn(phrase, changelog)


if __name__ == "__main__":
    unittest.main()
