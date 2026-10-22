"""October 22 intake: ChrizFader's 18 player voicepacks (release v1.1.0).

Static recipe, selection, order, pin and evidence checks for both recipe profiles only.
The owner's fixture installs are recorded in the intake evidence; nothing here is a game
installation or gameplay acceptance.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import _effective_features, build_recipe
from tools.curation_audit import Decision, RowKey, load_catalogs, load_curation_map


ROOT = Path(__file__).resolve().parents[2]
RELEASE = "https://github.com/Chrizhermann/chriz-bg-voicepacks/releases"
RELEASE_COMMIT = "5cdb10639d9e5d729c0ce881165623995ac127ff"
# Release-manifest order: (id, TP2 folder, version, length, SHA-256, title, TP2 @0 name).
PACKS = (
    ("neera", "cd_neera", "1.0.1", 2_361_220, "f2adbfd775f86f9cfbcae37ca1b4aaa1ca9e57876b8952603a56b632aba50fbc",
     "Neera", "Neera Soundset"),
    ("safana-bg1", "cd_safana", "1.0.1", 2_947_151, "e5b3e7fa69461c333530d61979bf713dfddc693614e95f6cea854e45ae8a541d",
     "Safana (BG1)", "Safana Soundset"),
    ("safana-sod", "cd_safsod", "1.0.1", 2_749_003, "edd486e85c9521905dd8cba2a5f3fddb846f3ae33b3f36e0308bab89c0432602",
     "Safana (SoD)", "Safana SoD Soundset"),
    ("jaheira-bg1", "cd_jahebg1", "1.0.1", 2_710_434, "d8a3bd683fc1c044eceec662414ff8279ea4992053b6c931486097436ad03806",
     "Jaheira (BG1)", "Jaheira BG1 Soundset"),
    ("jaheira-bg2", "cd_jahebg2", "1.0.1", 2_119_330, "5efe01db15944ad675eaf115ba0ca6bbcf76656ffa6c0408096e4c82be5ccf07",
     "Jaheira (BG2)", "Jaheira BG2 Soundset"),
    ("yeslick", "cd_yeslick", "1.0.1", 2_710_823, "c3c1213cc4ec89002b750788c58cd533d5c765d839c9c6b7c463c9db9ec3f22b",
     "Yeslick", "Yeslick Soundset"),
    ("edwin-bg1", "cd_edwnbg1", "1.0.1", 2_904_129, "6c02995b38edc5a021a54afbf544ae3e3ec3fa8f7cf5750aad22eb853a5e5411",
     "Edwin (BG1)", "Edwin BG1 Soundset"),
    ("edwin-bg2", "cd_edwnbg2", "1.0.1", 2_454_719, "d32e6f5577b95ae73b2113cb2ec83de2952a8c83383b4863d7df5fdcbc6aebc9",
     "Edwin (BG2)", "Edwin BG2 Soundset"),
    ("aerie", "cd_aerie", "1.0.1", 2_341_205, "04ce446df621ad6def59f5ca922c44a4d9a147ca86a236afdb22df1390ba6775",
     "Aerie", "Aerie Soundset"),
    ("imoen-bg1", "cd_imobg1", "1.0.1", 1_796_373, "cda87d6b56a15437025c536e2ea50194330565adf8a5e55f1cbdbbcaf8de79bb",
     "Imoen (BG1)", "Imoen (BG1) Soundset"),
    ("imoen-bg2", "cd_imobg2", "1.0.1", 2_046_524, "043d6216b4cf347a8a28f0d464af4286f01624587e55c14e324d0d21f628be60",
     "Imoen (BG2)", "Imoen (BG2) Soundset"),
    ("sarevok", "cd_sarevok", "1.0.1", 2_115_482, "e105f9b296d06677d9d0ba67ffdc3583c90ac13a97ea4dd7d45117f721af7937",
     "Sarevok", "Sarevok Soundset"),
    ("amelyssan", "cd_amely", "1.0.1", 2_447_749, "f67c4e4b87ccafc9272539d212c0211d53e45f8cd76d1a06aabd86239f4b6809",
     "Amelyssan", "Amelyssan Soundset"),
    ("gromnir-il-khan", "cd_gromn", "1.0.1", 1_972_660, "7aaad0379a6cbba87c5b2aa772ad3b53b777ae1d1821194567cfa8288811c9f5",
     "Gromnir Il-Khan", "Gromnir Il-Khan Soundset"),
    ("ajantis", "cd_ajanti", "2.0.0", 4_057_717, "7d2150ba920205e68258b9fc74a35594c1368d8b09e4e66ca9f1569a1baed77b",
     "Ajantis", "Ajantis: original BG1 companion soundset"),
    ("viconia-bg1", "cd_vicbg1", "1.0.0", 4_120_894, "bd56af0ef69e84986061ee53fad36611291fb2038c2dbfd059542eb6d6a0d5f5",
     "Viconia (BG1)", "Viconia (BG1): original BG1 companion soundset"),
    ("viconia-bg2", "cd_vicbg2", "1.0.0", 5_058_181, "dde9cfca2692de0b75e1bd9a6bc8dabba85db79608b87da18f49ae685d31420a",
     "Viconia (BG2)", "Viconia (BG2): original BG2/ToB companion soundset"),
    ("garithos", "cd_garith", "1.1.0", 5_117_808, "24abfd30cd8c9e5bedc77b2f8c9ab8b1444cc5c4189f5857d2b70def805525c5",
     "Garithos", "Garithos - player voicepack"),
)
RUNS = [f"voicepack-{pack[0]}-bg2" for pack in PACKS]
FEATURES = [f"feature:voicepack-{pack[0]}:component-0" for pack in PACKS]
# Runs that write after EET_end; the sound-only voicepacks follow them and precede BuffBot.
POST_EET_WRITERS = (
    "chriz-sod-remix-bg2", "hiddengameplayoptions-bg2", "chriz-bg-rebalance-bg2", "chriz-bg-modpack-bg2",
    "safana-bg2", "chriz-bg-modpack-late-companions-bg2", "spell-rev-npc-spellbooks-bg2",
    "spell-rev-lightning-bg2", "chriz-bg-modpack-baeloth-bg2",
)


def _load_profile(directory: Path) -> tuple[dict, dict]:
    collection = tomllib.loads((directory / "collection.toml").read_text(encoding="utf-8"))
    preset = tomllib.loads((directory / "presets/chris-recommended.toml").read_text(encoding="utf-8"))["selections"]
    return collection, preset


class Oct22VoicepacksTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        cls.output = Path(cls.temp.name) / "recipe"
        # Synthetic evidence marker used only in this temporary test output.
        build_recipe(ROOT, cls.output, "0" * 40)
        cls.profiles = {
            "curated": (cls.output, *_load_profile(cls.output)),
            "checked-in curated": (ROOT / "recipes/curated-full-current",
                                   *_load_profile(ROOT / "recipes/curated-full-current")),
            "fallback": (ROOT / "manifest", *_load_profile(ROOT / "manifest")),
        }
        cls.evidence = tomllib.loads(
            (ROOT / "manifest/evidence/release-intake-20261022.toml").read_text(encoding="utf-8"))

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

    def test_artifacts_pin_the_published_release_and_publish_only_the_mod_folder(self):
        for _, directory, _, _ in self._each_profile():
            for pack_id, folder, version, length, sha256, title, _ in PACKS:
                artifact_id = f"voicepack-{pack_id}-{version}"
                artifact = tomllib.loads((directory / f"artifacts/{artifact_id}.toml").read_text(encoding="utf-8"))
                filename = f"{pack_id}-player-voicepack-v{version}.zip"
                self.assertEqual(artifact["id"], artifact_id)
                self.assertEqual(artifact["version"], version)
                self.assertEqual(artifact["acquisition"], "fetch-only")
                self.assertEqual(artifact["source"], {
                    "kind": "github-release",
                    "url": f"{RELEASE}/download/v1.1.0/{filename}",
                    "reference": "v1.1.0",
                    "expected_filename": filename,
                    "expected_length": length,
                    "sha256": sha256,
                    "redirect_hosts": ["release-assets.githubusercontent.com"],
                })
                # The bundled setup EXE (and Garithos's Extras/README) are never published.
                self.assertEqual(artifact["archive"]["kind"], "zip")
                self.assertEqual(artifact["archive"]["root_rule"], "direct")
                self.assertEqual(artifact["archive"]["publish_roots"], [folder])
                self.assertEqual(artifact["archive"]["tp2_paths"], [f"{folder}/{folder}.tp2"])
                # Measured on October 22: at most 62 entries, depth 3, 1,390,080 bytes for
                # the largest entry (the unpublished EXE), 5,905,009 bytes in total.
                limits = artifact["archive"]["limits"]
                self.assertGreaterEqual(limits["max_entries"], 62)
                self.assertGreaterEqual(limits["max_depth"], 3)
                self.assertGreaterEqual(limits["max_entry_uncompressed_bytes"], 1_390_080)
                self.assertGreaterEqual(limits["max_total_uncompressed_bytes"], 5_905_009)
                self.assertEqual(artifact["provenance"]["homepage"], "https://github.com/Chrizhermann/chriz-bg-voicepacks")
                self.assertEqual(artifact["provenance"]["url"], f"{RELEASE}/tag/v1.1.0")
                self.assertEqual(artifact["provenance"]["reviewed_on"], "2026-10-22")
                self.assertIn("does not redistribute", artifact["provenance"]["license"])
                self.assertEqual(artifact["name"], f"{title} player voicepack")
        # The checked-in curated artifacts are the generator's copies of the authored pins.
        for pack_id, _, version, *_ in PACKS:
            name = f"artifacts/voicepack-{pack_id}-{version}.toml"
            self.assertEqual((ROOT / "recipes/curated-full-current" / name).read_bytes(),
                             (ROOT / "manifest" / name).read_bytes())

    def test_mods_install_component_0_with_the_pinned_weidu(self):
        for _, directory, _, _ in self._each_profile():
            for pack_id, folder, version, _, _, title, component_name in PACKS:
                mod = tomllib.loads((directory / f"mods/voicepack-{pack_id}.toml").read_text(encoding="utf-8"))
                self.assertEqual(mod, {
                    "id": f"voicepack-{pack_id}",
                    "artifact_id": f"voicepack-{pack_id}-{version}",
                    "name": f"{title} player voicepack",
                    "tp2": f"{folder}/{folder}.tp2",
                    "language": 0,
                    "weidu_artifact_id": "weidu-249-amd64",
                    "invocation_mode": "setup-name",
                    "components": [{"id": 0, "name": component_name}],
                })

    def test_voicepack_runs_follow_eet_end_and_precede_buffbot_and_scrolls_stays_last(self):
        for name, _, collection, _ in self._each_profile():
            runs = {run["run_id"]: run for run in collection["runs"]}
            order = [run["run_id"] for run in collection["runs"]]
            start = order.index(RUNS[0])
            # Contiguous, in release-manifest order.
            self.assertEqual(order[start:start + len(RUNS)], RUNS)
            for run_id, pack in zip(RUNS, PACKS):
                run = runs[run_id]
                self.assertEqual(run["mod_id"], f"voicepack-{pack[0]}")
                self.assertEqual(run["phase"], "post-eet-end")
                self.assertEqual(run["components"], [0])
                self.assertEqual(run["args"], [])
            self.assertLess(order.index("eet-end-bg2"), start)
            for writer in POST_EET_WRITERS:
                self.assertLess(order.index(writer), start, writer)
            end = start + len(RUNS)
            # Klatu (curated profile only) keeps its place right before BuffBot.
            following = ["klatu-armor-thieving-bg2"] if "klatu-armor-thieving-bg2" in order else []
            self.assertEqual(order[end:], [*following, "buffbot-bg2", "chriz-bg-modpack-scrolls-bg2"], name)
            self.assertEqual(order[-1], "chriz-bg-modpack-scrolls-bg2")

    def test_voicepacks_are_independent_ready_defaults_in_the_voicepacks_group(self):
        for _, _, collection, preset in self._each_profile():
            features = {feature["id"]: feature for feature in collection["features"]}
            baseline = self._selected(collection, preset)
            for feature_id, run_id, pack in zip(FEATURES, RUNS, PACKS):
                feature = features[feature_id]
                self.assertEqual(feature["title"], pack[5])
                self.assertEqual((feature["category"], feature["decision"], feature["readiness"]),
                                 ("audio", "default", "ready"))
                self.assertEqual(feature["group_label"], "Voicepacks")
                self.assertEqual(feature["source_label"], "ChrizFader's voicepacks")
                for key in ("parent", "requires", "requires_any", "conflicts", "choice_group"):
                    self.assertNotIn(key, feature)
                self.assertEqual(feature["components"], [{"run_id": run_id, "component": 0}])
                self.assertEqual(preset.get(feature_id), "on")
                self.assertIn((run_id, 0), baseline)
                # Switching one voicepack off removes exactly its own run component.
                self.assertEqual(baseline - self._selected(collection, preset, **{feature_id: "off"}),
                                 {(run_id, 0)})
            self.assertNotIn("portraits", features[FEATURES[-1]]["description"])
            self.assertIn("IAIWE", features[FEATURES[-1]]["description"])
            # No Spell Revisions or EEex dependency.
            for overrides in ({"mod:spell-rev": "off"}, {"mod:eeex": "off"}):
                selected = self._selected(collection, preset, **overrides)
                self.assertTrue({(run_id, 0) for run_id in RUNS} <= selected, overrides)
            # Not a main toggle like Challenge Mode.
            for group in collection.get("feature_groups", []):
                self.assertFalse(set(FEATURES) & set(group["members"]), group["id"])

    def test_release_records_approve_every_voicepack_tail_run(self):
        for _, directory, _, _ in self._each_profile():
            limits = tomllib.loads(
                (directory / "releases/v0.1.0-alpha.1/known-limitations.toml").read_text(encoding="utf-8"))
            approved = limits["approved_tail_runs"]
            self.assertEqual(len(approved), len(set(approved)))
            self.assertTrue(set(RUNS) <= set(approved))
            self.assertLess(max(approved.index(run_id) for run_id in RUNS), approved.index("buffbot-bg2"))

    def test_catalog_rows_record_the_default_decision(self):
        rows = {RowKey(row.catalog, row.component_id): row for row in load_catalogs(ROOT / "docs/curation/components")}
        mapping = load_curation_map(ROOT / "manifest/curation-map.toml")
        catalogs = {catalog.catalog: catalog for catalog in mapping.catalogs}
        targets = {target.target_id: target for target in mapping.targets}
        for feature_id, pack in zip(FEATURES, PACKS):
            key = RowKey(pack[1].upper(), 0)
            row = rows[key]
            self.assertEqual(row.decision, Decision.DEFAULT)
            self.assertEqual(row.component, pack[6])
            self.assertEqual(row.group, "Voicepacks")
            self.assertEqual(row.subgroup, "")
            self.assertEqual(catalogs[key.catalog].excluded, ())
            self.assertEqual(targets[feature_id].rows, (key,))
            self.assertEqual(targets[feature_id].kind, "feature")
        self.assertEqual(len([key for key in rows if key.catalog.startswith("CD_")]), len(PACKS))

    def test_intake_evidence_matches_current_pins(self):
        self.assertEqual(self.evidence["verified_on"], "2026-10-22")
        records = self.evidence["artifacts"]
        self.assertEqual([record["mod_id"] for record in records], [f"voicepack-{pack[0]}" for pack in PACKS])
        for record, (pack_id, folder, version, length, sha256, _, _) in zip(records, PACKS):
            with self.subTest(pack=pack_id):
                self.assertEqual(record["artifact_id"], f"voicepack-{pack_id}-{version}")
                self.assertEqual(record["recipe_profiles"], ["manifest", "recipes/curated-full-current"])
                self.assertEqual(record["source_reference"], "v1.1.0")
                self.assertEqual(record["source_commit"], RELEASE_COMMIT)
                self.assertEqual((record["actual_length"], record["actual_sha256"]), (length, sha256))
                self.assertRegex(record["tp2_sha256"], r"^[0-9a-f]{64}$")
                self.assertEqual(record["published_roots"], [folder])
                self.assertEqual(record["native_components"], [0])
                for flag in ("fresh_download", "fresh_extraction", "published_release_verified",
                             "native_component_order_verified"):
                    self.assertIs(record[flag], True, flag)
                for profile in (ROOT / "manifest", ROOT / "recipes/curated-full-current", self.output):
                    artifact = tomllib.loads(
                        (profile / f"artifacts/{record['artifact_id']}.toml").read_text(encoding="utf-8"))
                    for key, measured in (("sha256", "actual_sha256"), ("expected_length", "actual_length"),
                                          ("url", "immutable_source_url"), ("reference", "source_reference")):
                        self.assertEqual(artifact["source"][key], record[measured])
        self.assertEqual(len({record["tp2_sha256"] for record in records}), len(PACKS))


if __name__ == "__main__":
    unittest.main()
