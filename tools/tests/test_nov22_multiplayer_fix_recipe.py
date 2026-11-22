"""November 22 intake: Chriz Multiplayer Fix v0.1.0 in both Chriz Easy installers.

Component 0 stops the crash right after leaving Beamdog's multiplayer lobby, component 1 adds a
Load screen button that copies saves between single player and multiplayer; both through EEex.
Chriz Easy IWD numbers its collection 0.1.18 and offers both to existing installations as a
hotpatch. These are static source/package and recipe assertions only, not a combined collection
install, a multiplayer session or campaign gameplay acceptance.
"""

import json
from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import _effective_features, build_recipe


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = (
    "manifest/evidence/"
    "release-intake-20261122-chriz-multiplayer-fix-chriz-easy-bg-chriz-easy-iwd.toml"
)
MOD = "chriz-multiplayer-fix"
PIN = "chriz-multiplayer-fix-0.1.0"
TAG = "v0.1.0"
FILENAME = "chriz-multiplayer-fix-v0.1.0.zip"
URL = f"https://github.com/Chrizhermann/chriz-multiplayer-fix/releases/download/{TAG}/{FILENAME}"
SHA256 = "d948dd3276b781d776229e35a08f8edcf3504dfde3a5b8b1c60024d8434bc73b"
TP2 = "chriz_multiplayer_fix/setup-chriz_multiplayer_fix.tp2"
TP2_SHA256 = "9a739babb86919f12585e009beb42ff0960a2b718d0560630e08e59b716fb09c"
LENGTH = 1_280_778
TAG_COMMIT = "025fb264335348edbd1d2ba65b5bb843da1ceea3"
COMPONENTS = [0, 1]
COMPONENT_NAMES = {
    0: "Multiplayer lobby crash fix (EEex)",
    1: "Copy saves between single player and multiplayer on the Load screen (EEex)",
}
FEATURE_0 = "feature:chriz-multiplayer-fix:component-0"
FEATURE_1 = "feature:chriz-multiplayer-fix:component-1"
MOD_FEATURE = "mod:chriz-multiplayer-fix"
DESCRIPTIONS = {
    MOD_FEATURE: "Fixes the crash after leaving the multiplayer lobby and adds a Load screen button"
    " that copies saves between single player and multiplayer.",
    FEATURE_0: "Stops the random crash right after leaving Beamdog's multiplayer lobby, for example"
    " on Multiplayer > Local network game > Load.",
    FEATURE_1: "Adds a button to the Load screen that copies a save into the other mode's save"
    " list, so a party created in multiplayer can continue in single player and back. The"
    " original save stays.",
}
BG_RUN = "chriz-multiplayer-fix-bg2"
IWD_RUN = "chriz-multiplayer-fix-main"
# The take-in commit that the static-test records in the three acceptance files name.
TAKE_IN_COMMIT = "@TAKE_IN_COMMIT@"
HOTPATCH = "chriz-multiplayer-fix-0.1.0"


def load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def features(collection: dict) -> dict[str, dict]:
    return {feature["id"]: feature for feature in collection["features"]}


class Nov22ChrizMultiplayerFixIntakeTests(unittest.TestCase):
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

    def test_release_evidence_names_the_asset_and_its_native_order(self) -> None:
        self.assertEqual(self.evidence["verified_on"], "2026-11-22")
        self.assertIn("Chriz Multiplayer Fix v0.1.0", self.evidence["scope"])
        self.assertIn("not a combined full-collection WeiDU install", self.evidence["scope"])
        self.assertEqual(
            set(self.records),
            {"manifest", "recipes/curated-full-current", "recipes/iwd-current"},
        )
        for record in self.records.values():
            self.assertEqual(record["mod_id"], MOD)
            self.assertEqual(record["artifact_id"], PIN)
            self.assertEqual(record["source_reference"], TAG)
            self.assertEqual(record["source_commit"], TAG_COMMIT)
            self.assertEqual(record["immutable_source_url"], URL)
            self.assertEqual(record["actual_length"], LENGTH)
            self.assertEqual(record["actual_sha256"], SHA256)
            self.assertEqual(record["tp2_sha256"], TP2_SHA256)
            self.assertEqual(record["native_components"], COMPONENTS)
            self.assertIs(record["native_component_order_verified"], True)
            self.assertIs(record["fresh_extraction"], True)
            # The scope says what the two download flags stand for; both stay honest booleans.
            for flag in ("fresh_download", "published_release_verified"):
                self.assertIsInstance(record[flag], bool, flag)
            if not record["published_release_verified"]:
                self.assertIn("must be run once the release is published", self.evidence["scope"])

    def test_every_recipe_and_the_hotpatch_freeze_the_same_artifact(self) -> None:
        directories = [
            ("manifest", ROOT / "manifest"),
            ("recipes/curated-full-current", ROOT / "recipes/curated-full-current"),
            ("recipes/curated-full-current", self.generated),
            ("recipes/iwd-current", ROOT / "recipes/iwd-current"),
            ("recipes/iwd-current", ROOT / "hotpatches/iwd"),
        ]
        for profile, directory in directories:
            with self.subTest(directory=directory):
                record = self.records[profile]
                mod = load(directory / f"mods/{MOD}.toml")
                artifact = load(directory / f"artifacts/{PIN}.toml")
                self.assertEqual(mod["id"], MOD)
                self.assertEqual(mod["artifact_id"], PIN)
                self.assertEqual(mod["tp2"], TP2)
                self.assertEqual(mod["weidu_artifact_id"], "weidu-249-amd64")
                self.assertEqual(mod["invocation_mode"], "setup-name")
                self.assertEqual(
                    {component["id"]: component["name"] for component in mod["components"]},
                    COMPONENT_NAMES,
                )
                self.assertEqual(artifact["id"], PIN)
                self.assertEqual(artifact["version"], TAG.removeprefix("v"))
                self.assertEqual(artifact["acquisition"], "fetch-only")
                self.assertEqual(artifact["source"]["kind"], "github-release")
                self.assertEqual(artifact["source"]["reference"], TAG)
                self.assertEqual(artifact["source"]["expected_filename"], FILENAME)
                self.assertEqual(artifact["archive"]["root_rule"], "direct")
                self.assertEqual(artifact["archive"]["publish_roots"], ["chriz_multiplayer_fix"])
                self.assertEqual(artifact["archive"]["tp2_paths"], [TP2])
                for key, evidence_key in (
                    ("url", "immutable_source_url"),
                    ("expected_length", "actual_length"),
                    ("sha256", "actual_sha256"),
                ):
                    self.assertEqual(artifact["source"][key], record[evidence_key], key)

    def test_bg_recipes_install_both_components_by_default_after_overhead_text(self) -> None:
        for directory in (
            ROOT / "manifest",
            ROOT / "recipes/curated-full-current",
            self.generated,
        ):
            with self.subTest(directory=directory):
                collection = load(directory / "collection.toml")
                run_ids = [run["run_id"] for run in collection["runs"]]
                (run,) = [run for run in collection["runs"] if run["mod_id"] == MOD]
                self.assertEqual(run["run_id"], BG_RUN)
                self.assertEqual(run["phase"], "post-eet-end")
                self.assertEqual(run["components"], COMPONENTS)
                self.assertLess(run_ids.index("eeex-bg2"), run_ids.index(BG_RUN))
                self.assertEqual(run_ids.index("chriz-overhead-text-bg2") + 1, run_ids.index(BG_RUN))
                self.assertEqual(run_ids.index(BG_RUN) + 1, run_ids.index("eeexremote-bg2"))

                by_id = features(collection)
                self.assertNotIn(MOD_FEATURE, by_id)
                for feature_id, component, category, group in (
                    (FEATURE_0, 0, "fixes", "Fixes"),
                    (FEATURE_1, 1, "interface", "Interface"),
                ):
                    feature = by_id[feature_id]
                    self.assertEqual(feature["title"], COMPONENT_NAMES[component])
                    self.assertEqual(feature["description"], DESCRIPTIONS[feature_id])
                    self.assertEqual(feature["category"], category)
                    self.assertEqual(feature["group_label"], group)
                    self.assertEqual(feature["source_label"], "Chriz Multiplayer Fix")
                    self.assertEqual(feature["decision"], "default")
                    self.assertEqual(feature["readiness"], "ready")
                    self.assertEqual(feature["requires"], ["feature:eeex:mandatory-components"])
                    self.assertEqual(
                        feature["components"], [{"run_id": BG_RUN, "component": component}]
                    )

                preset = load(directory / "presets/chris-recommended.toml")["selections"]
                self.assertEqual(preset[FEATURE_0], "on")
                self.assertEqual(preset[FEATURE_1], "on")
                effective = _effective_features(collection, preset)
                self.assertTrue(effective[FEATURE_0])
                self.assertTrue(effective[FEATURE_1])

        known = load(ROOT / "manifest/releases/v0.1.0-alpha.1/known-limitations.toml")
        tail = known["approved_tail_runs"]
        self.assertEqual(tail.index("chriz-overhead-text-bg2") + 1, tail.index(BG_RUN))

        curation = load(ROOT / "manifest/curation-map.toml")
        self.assertIn({"id": "CHRIZ_MULTIPLAYER_FIX", "excluded": []}, curation["catalogs"])
        targets = {target["id"]: target for target in curation["targets"]}
        self.assertEqual(targets[FEATURE_0]["rows"], ["CHRIZ_MULTIPLAYER_FIX:0"])
        self.assertEqual(targets[FEATURE_1]["rows"], ["CHRIZ_MULTIPLAYER_FIX:1"])

    def test_iwd_recipe_installs_the_fix_always_and_the_copy_button_by_default(self) -> None:
        collection = load(ROOT / "recipes/iwd-current/collection.toml")
        run_ids = [run["run_id"] for run in collection["runs"]]
        (run,) = [run for run in collection["runs"] if run["mod_id"] == MOD]
        self.assertEqual(run["run_id"], IWD_RUN)
        self.assertEqual(run["phase"], "main")
        self.assertEqual(run["components"], COMPONENTS)
        self.assertLess(run_ids.index("eeex-main"), run_ids.index(IWD_RUN))
        self.assertEqual(run_ids.index("chriz-overhead-text-main") + 1, run_ids.index(IWD_RUN))
        self.assertEqual(run_ids.index(IWD_RUN) + 1, run_ids.index("chriz-iwd-party-dialog-main"))

        by_id = features(collection)
        parent = by_id[MOD_FEATURE]
        self.assertEqual(parent["title"], "Chriz Multiplayer Fix")
        self.assertEqual(parent["description"], DESCRIPTIONS[MOD_FEATURE])
        self.assertEqual(parent["category"], "fixes")
        self.assertEqual(parent["decision"], "default")
        self.assertEqual(parent["readiness"], "ready")
        self.assertNotIn("parent", parent)
        for feature_id, component, category, decision in (
            (FEATURE_0, 0, "fixes", "mandatory"),
            (FEATURE_1, 1, "interface", "default"),
        ):
            feature = by_id[feature_id]
            self.assertEqual(feature["title"], COMPONENT_NAMES[component])
            self.assertEqual(feature["description"], DESCRIPTIONS[feature_id])
            self.assertEqual(feature["category"], category)
            self.assertEqual(feature["decision"], decision)
            self.assertEqual(feature["readiness"], "ready")
            self.assertEqual(feature["parent"], MOD_FEATURE)
            self.assertEqual(feature["requires"], ["feature:eeex:component-1"])
            self.assertEqual(feature["components"], [{"run_id": IWD_RUN, "component": component}])
        # Chris's Recommended for IWD takes every feature at its recipe default.
        preset = load(ROOT / "recipes/iwd-current/presets/chris-recommended.toml")
        self.assertNotIn(MOD_FEATURE, preset.get("selections", {}))

    def test_own_mods_catalog_sources_changelog_and_pin_list_name_the_mod(self) -> None:
        own_mods = load(ROOT / "manifest/own-mods.toml")
        entry = next(mod for mod in own_mods["mods"] if mod["id"] == MOD)
        self.assertEqual(entry["repository"], "Chrizhermann/chriz-multiplayer-fix")
        catalog = (ROOT / "docs/curation/components/CHRIZ_MULTIPLAYER_FIX.md").read_text(
            encoding="utf-8"
        )
        self.assertIn(f"Listed at **{TAG}**", catalog)
        self.assertIn(f"`{TAG_COMMIT[:7]}`", catalog)
        self.assertIn("| 0 | Multiplayer lobby crash fix (EEex) | Fixes |  |  | default |", catalog)
        self.assertIn(
            "| 1 | Copy saves between single player and multiplayer on the Load screen (EEex)"
            " | Interface |  |  | default |",
            catalog,
        )
        index = (ROOT / "docs/curation/components/README.md").read_text(encoding="utf-8")
        self.assertIn(
            "| CHRIZ_MULTIPLAYER_FIX | 2 | 0 | [CHRIZ_MULTIPLAYER_FIX.md](CHRIZ_MULTIPLAYER_FIX.md) |",
            index,
        )
        sources = (ROOT / "manifest/mod-sources.tsv").read_text(encoding="utf-8").splitlines()
        (row,) = [line.split("\t") for line in sources if line.startswith("CHRIZ_MULTIPLAYER_FIX\t")]
        self.assertEqual(row[3], TAG)
        self.assertEqual(row[7], URL)
        changelog = (ROOT / "docs/plans/2026-10-14-next-release-changelogs.md").read_text(
            encoding="utf-8"
        )
        self.assertIn("Chriz Multiplayer Fix v0.1.0 (new)", changelog)
        self.assertIn("**Chriz Multiplayer Fix v0.1.0**", changelog)
        self.assertIn("(Chriz Multiplayer Fix v0.1.0, collection 0.1.18)", changelog)
        pin_list = (ROOT / "docs/pin-list-2.7.md").read_text(encoding="utf-8")
        self.assertEqual(
            pin_list.count(f"| Chriz Multiplayer Fix | 0.1.0 | `{MOD}` | `{SHA256[:12]}…` |"),
            2,
        )
        inventory = (ROOT / "docs/curation/iwdee/INVENTORY.md").read_text(encoding="utf-8")
        self.assertIn(f"{PIN} (0, 1)", inventory)

    def test_static_test_records_name_the_take_in_commit(self) -> None:
        for acceptance_path, subject_id, test_artifact in (
            (
                "manifest/releases/v0.1.0-alpha.1/acceptance.toml",
                BG_RUN,
                "tools/tests/test_nov22_multiplayer_fix_recipe.py",
            ),
            (
                "recipes/curated-full-current/releases/v0.1.0-alpha.1/acceptance.toml",
                BG_RUN,
                "tools/tests/test_nov22_multiplayer_fix_recipe.py",
            ),
            (
                "recipes/iwd-current/releases/v0.1.0-alpha.1/acceptance.toml",
                IWD_RUN,
                "engine/tests/production_recipe_iwd.rs",
            ),
        ):
            with self.subTest(acceptance=acceptance_path):
                acceptance = load(ROOT / acceptance_path)
                (record,) = [
                    row for row in acceptance["evidence"] if row["subject_id"] == subject_id
                ]
                self.assertEqual(record["subject_kind"], "run")
                self.assertEqual(record["kind"], "static-test")
                self.assertEqual(record["status"], "accepted")
                self.assertEqual(record["date"], "2026-11-22")
                self.assertEqual(record["commit"], TAKE_IN_COMMIT)
                self.assertEqual(record["test_artifact"], test_artifact)

    def test_iwd_collection_0_1_18_ledger_and_base(self) -> None:
        release = json.loads(
            (ROOT / "recipes/iwd-current/release.json").read_text(encoding="utf-8")
        )
        self.assertGreaterEqual(
            tuple(int(part) for part in release["version"].split(".")), (0, 1, 18)
        )
        ledger = load(ROOT / "recipes/iwd-current/releases/v0.1.18/ledger.toml")
        self.assertEqual(
            (ledger["recipe_id"], ledger["version"], ledger["supersedes"]),
            ("chriz-easy-iwd", "0.1.18", "0.1.17"),
        )
        (change,) = ledger["changes"]
        self.assertEqual(change["id"], MOD)
        self.assertEqual(
            change["title"],
            "No more crash after the multiplayer lobby; saves copy between single player and"
            " multiplayer",
        )
        self.assertEqual(change["save_applicability"], "current-save")
        self.assertEqual(change["urgency"], "recommended")
        self.assertEqual(
            change["covers"],
            [
                f"artifact:{PIN}",
                f"feature:{MOD_FEATURE}",
                f"feature:{FEATURE_0}",
                f"feature:{FEATURE_1}",
                f"run:{IWD_RUN}",
            ],
        )
        catalog = load(ROOT / "hotpatches/iwd/catalog.toml")
        (base,) = [row for row in catalog["bases"] if row["recipe_version"] == "0.1.18"]
        self.assertEqual(base["id"], "iwd-0.1.18")
        # The engine test the_bundled_iwd_catalog_fits_the_current_release checks the digest.
        self.assertRegex(base["content_sha256"], r"^[0-9a-f]{64}$")
        self.assertNotEqual(base["content_sha256"], "0" * 64)

    def test_iwd_hotpatch_brings_both_components_to_every_earlier_base(self) -> None:
        catalog = load(ROOT / "hotpatches/iwd/catalog.toml")
        (hotpatch,) = [row for row in catalog["hotpatches"] if row["id"] == HOTPATCH]
        self.assertEqual(hotpatch["title"], "No more crash after the multiplayer lobby")
        self.assertEqual(hotpatch["artifact"], PIN)
        self.assertEqual(hotpatch["mod"], MOD)
        self.assertEqual(hotpatch["components"], COMPONENTS)
        self.assertEqual(hotpatch["requires_components"], [{"mod": "eeex", "any_of": [1]}])
        earlier = [base["id"] for base in catalog["bases"] if base["id"] != "iwd-0.1.18"]
        self.assertEqual(hotpatch["bases"], earlier)
        self.assertEqual(len(hotpatch["changes"]), 2)
        for change in hotpatch["changes"]:
            self.assertEqual(change["save_applicability"], "current-save")
        # It is the last hotpatch, so every earlier one for the same bases comes first.
        self.assertEqual(catalog["hotpatches"][-1]["id"], HOTPATCH)
        # BG offers no hotpatch.
        self.assertEqual(load(ROOT / "hotpatches/bg/catalog.toml").get("hotpatches", []), [])


if __name__ == "__main__":
    unittest.main()
