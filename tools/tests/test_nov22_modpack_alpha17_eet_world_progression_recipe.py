"""November 22 intake: Chriz BG Modpack v0.2.0-alpha.17 (component 680).

Modpack PRs #20 and #21 add default component 680 (cbm_eet_world_progression) after
670 and before 650 in TP2 declaration order.
This is static pin, evidence, selection, and order coverage, not a WeiDU install
or live campaign gameplay acceptance.
"""

from __future__ import annotations

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import _effective_features, build_recipe


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261122-chriz-bg-modpack-chriz-easy-bg.toml"
PREVIOUS_EVIDENCE = "manifest/evidence/release-intake-20261108-chriz-bg-modpack-chriz-easy-bg.toml"
MOD = "chriz-bg-modpack"
PIN = "chriz-bg-modpack-0.2.0-alpha.17"
PREVIOUS = "chriz-bg-modpack-0.2.0-alpha.16"
TAG = "v0.2.0-alpha.17"
SOURCE_COMMIT = "5d93338aff1b85a2bc1ebc602084e255a8c11cea"
PIN_COMMIT = "41e1ac82d45e93402707aaafc25337784fd11477"
FILENAME = "chriz-bg-modpack-v0.2.0-alpha.17.zip"
URL = f"https://github.com/Chrizhermann/chriz-bg-modpack/releases/download/{TAG}/{FILENAME}"
LENGTH = 9_542_254
SHA256 = "c65fe907a5d646fc5cf1564077c7f5f2db5a8a6c77f852a013479d77e3b9108a"
TP2_SHA256 = "a9ddca687c604bd5e463c099dd683ee32bf9042382e087bd7f6b82ca284a88ae"
COMPONENT = 680
RUN = "chriz-bg-modpack-bg2"
FEATURE = f"feature:{MOD}:component-{COMPONENT}"
LABEL = "EET: story-consistent campaign boundaries and SoD backtracking fixes"
STATIC_RUNS = {
    "chriz-bg-modpack-bg2",
    "chriz-bg-modpack-edwin-bg2",
    "chriz-bg-modpack-baeloth-bg2",
    "chriz-bg-modpack-scrolls-bg2",
    "chriz-bg-modpack-pre-continuity-bg2",
    "chriz-bg-modpack-continuity-bg2",
    "chriz-bg-modpack-late-companions-bg2",
}


def _load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def _selected(collection: dict, selection: dict) -> set[tuple[str, int]]:
    state = _effective_features(collection, selection)
    return {
        (ref["run_id"], ref["component"])
        for feature in collection["features"]
        if state[feature["id"]]
        for ref in feature.get("components", [])
    }


class Nov22ModpackAlpha17EETWorldProgressionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.temp = tempfile.TemporaryDirectory()
        cls.generated = Path(cls.temp.name) / "recipe"
        build_recipe(ROOT, cls.generated, "0" * 40)
        cls.evidence = _load(ROOT / EVIDENCE)
        (cls.record,) = cls.evidence["artifacts"]
        (cls.previous_record,) = _load(ROOT / PREVIOUS_EVIDENCE)["artifacts"]
        cls.directories = [
            ROOT / "manifest",
            ROOT / "recipes/curated-full-current",
            cls.generated,
        ]

    @classmethod
    def tearDownClass(cls) -> None:
        cls.temp.cleanup()

    def test_evidence_freezes_v0_2_0_alpha_17_and_native_order_with_680(self) -> None:
        self.assertEqual(self.evidence["verified_on"], "2026-11-22")
        self.assertEqual(self.evidence["generator"], "tools/prepare_own_mod_pin.py")
        for phrase in (
            "Modpack PRs #20 and #21",
            "replaces chriz-bg-modpack-0.2.0-alpha.16",
            "component 680",
            "not a WeiDU install or gameplay acceptance",
        ):
            self.assertIn(phrase, self.evidence["scope"])
        self.assertEqual(self.record["mod_id"], MOD)
        self.assertEqual(self.record["artifact_id"], PIN)
        self.assertEqual(self.record["recipe_profiles"], ["manifest", "recipes/curated-full-current"])
        self.assertEqual(self.record["source_reference"], TAG)
        self.assertEqual(self.record["source_commit"], SOURCE_COMMIT)
        self.assertEqual(self.record["immutable_source_url"], URL)
        self.assertEqual(self.record["actual_length"], LENGTH)
        self.assertEqual(self.record["actual_sha256"], SHA256)
        self.assertEqual(self.record["tp2_sha256"], TP2_SHA256)
        for flag in (
            "fresh_download",
            "fresh_extraction",
            "published_release_verified",
            "native_component_order_verified",
        ):
            self.assertIs(self.record[flag], True, flag)
        native = list(self.previous_record["native_components"])
        native.insert(native.index(670) + 1, COMPONENT)
        self.assertEqual(self.record["native_components"], native)

    def test_profiles_pin_alpha17_keep_alpha16_and_enable_680_by_default(self) -> None:
        for directory in self.directories:
            with self.subTest(profile=str(directory)):
                mod = _load(directory / f"mods/{MOD}.toml")
                artifact = _load(directory / f"artifacts/{PIN}.toml")
                previous = _load(directory / f"artifacts/{PREVIOUS}.toml")
                self.assertEqual(mod["artifact_id"], PIN)
                self.assertEqual(artifact["version"], TAG.removeprefix("v"))
                self.assertEqual(artifact["source"]["reference"], TAG)
                self.assertEqual(artifact["source"]["url"], URL)
                self.assertEqual(artifact["source"]["expected_filename"], FILENAME)
                self.assertEqual(artifact["source"]["expected_length"], LENGTH)
                self.assertEqual(artifact["source"]["sha256"], SHA256)
                self.assertEqual(artifact["archive"], previous["archive"])
                self.assertEqual(artifact["source"]["redirect_hosts"], previous["source"]["redirect_hosts"])

                collection = _load(directory / "collection.toml")
                preset = _load(directory / "presets/chris-recommended.toml")["selections"]
                features = {feature["id"]: feature for feature in collection["features"]}
                run_ids = [run["run_id"] for run in collection["runs"]]
                (run,) = [run for run in collection["runs"] if run["run_id"] == RUN]
                native = set(self.record["native_components"])
                for r in (r for r in collection["runs"] if r["mod_id"] == MOD):
                    self.assertTrue(set(r["components"]) <= native, r["run_id"])

                feature = features[FEATURE]
                self.assertEqual(feature["decision"], "default")
                self.assertEqual(feature["readiness"], "ready")
                self.assertEqual(feature["category"], "rules")
                self.assertEqual(feature["parent"], f"mod:{MOD}")
                self.assertEqual(feature["requires"], ["feature:eet:mandatory-components", "mod:eet-end"])
                self.assertEqual(feature["components"], [{"run_id": RUN, "component": COMPONENT}])
                self.assertEqual(preset[FEATURE], "on")
                self.assertEqual(run["phase"], "post-eet-end")
                self.assertEqual(run["components"][-2:], [670, COMPONENT])
                for earlier in ("eet-end-bg2", "stratagems-bg2", "chriz-sod-remix-bg2"):
                    self.assertLess(run_ids.index(earlier), run_ids.index(RUN), earlier)
                names = {item["id"]: item["name"] for item in mod["components"]}
                self.assertEqual(names[COMPONENT], LABEL)
                self.assertEqual(list(names).index(COMPONENT), list(names).index(670) + 1)

                baseline = _selected(collection, preset)
                self.assertIn((RUN, COMPONENT), baseline)
                self.assertNotIn((RUN, COMPONENT), _selected(collection, {**preset, FEATURE: "off"}))
                self.assertNotIn((RUN, COMPONENT), _selected(collection, {**preset, f"mod:{MOD}": "off"}))
                for optional_mod in ("mod:eeex", "mod:spell-rev", "mod:stratagems"):
                    self.assertIn((RUN, COMPONENT), _selected(collection, {**preset, optional_mod: "off"}), optional_mod)

    def test_static_records_and_docs_name_alpha17(self) -> None:
        for relative in (
            "manifest/releases/v0.1.0-alpha.1/acceptance.toml",
            "recipes/curated-full-current/releases/v0.1.0-alpha.1/acceptance.toml",
        ):
            records = {item["subject_id"]: item for item in _load(ROOT / relative)["evidence"]}
            for run in STATIC_RUNS:
                with self.subTest(record=relative, run=run):
                    record = records[run]
                    self.assertEqual(record["kind"], "static-test")
                    self.assertEqual(record["status"], "accepted")
                    self.assertEqual(record["commit"], PIN_COMMIT)
                    self.assertEqual(
                        record["test_artifact"],
                        "tools/tests/test_nov22_modpack_alpha17_eet_world_progression_recipe.py",
                    )
                    self.assertEqual(record["date"], "2026-11-22")
                    self.assertEqual(
                        record["scope"],
                        "Static alpha.17 pin, source, component (including EET world progression 680) "
                        "and order assertions only; fresh source/package verification is in "
                        "release-intake-20261122-chriz-bg-modpack-chriz-easy-bg.toml; "
                        "not a WeiDU installation or gameplay acceptance.",
                    )

        curation = _load(ROOT / "manifest/curation-map.toml")
        targets = {target["id"]: target for target in curation["targets"]}
        self.assertEqual(targets[FEATURE]["kind"], "feature")
        self.assertEqual(targets[FEATURE]["rows"], [f"CHRIZ-BG-MODPACK:{COMPONENT}"])
        self.assertEqual(targets[FEATURE]["parent"], f"mod:{MOD}")

        catalog = (ROOT / "docs/curation/components/CHRIZ-BG-MODPACK.md").read_text(encoding="utf-8")
        for phrase in (
            "**v0.2.0-alpha.17**",
            "5d93338",
            "Modpack PRs #20 and #21",
            EVIDENCE,
            f"| {COMPONENT} | {LABEL} | chriz-bg-modpack: Gameplay tweaks | | | default |",
            "66 current or historical entries; the collection exposes 49.",
        ):
            self.assertIn(phrase, catalog)

        changelog = (ROOT / "docs/plans/2026-10-14-next-release-changelogs.md").read_text(encoding="utf-8")
        current, historical = changelog.split("\n## Next Chriz Easy BG release (draft)\n", 1)[1].split(
            "\n## Collection 0.1.2 (draft)\n", 1
        )
        behavior, rest = current.split("### Changed behavior\n", 1)[1].split("### Mod versions\n", 1)
        versions, notes = rest.split("### Notes for the release integrator\n", 1)
        self.assertIn("**EET world progression and SoD backtracking fixes** (Chriz BG Modpack component 680", behavior)
        self.assertIn("- Chriz BG Modpack v0.2.0-alpha.17 (was v0.2.0-alpha.16)", versions)
        self.assertIn(EVIDENCE, notes)
        self.assertIn(f"artifact:{PIN}", notes)
        self.assertIn(f"feature:{FEATURE}", notes)
        old_draft = historical.split("\n## Installer changelog\n", 1)[0]
        self.assertNotIn("v0.2.0-alpha.17", old_draft)

        source = next(
            line for line in (ROOT / "manifest/mod-sources.tsv").read_text(encoding="utf-8").splitlines()
            if line.startswith("CHRIZ-BG-MODPACK\t")
        )
        for phrase in (TAG, SOURCE_COMMIT, str(LENGTH), SHA256):
            self.assertIn(phrase, source)
        pin_list = (ROOT / "docs/pin-list-2.7.md").read_text(encoding="utf-8")
        self.assertEqual(
            pin_list.count(f"| chriz-bg-modpack | {TAG.removeprefix('v')} | `{MOD}` | `{SHA256[:12]}…` |"),
            2,
        )


if __name__ == "__main__":
    unittest.main()
