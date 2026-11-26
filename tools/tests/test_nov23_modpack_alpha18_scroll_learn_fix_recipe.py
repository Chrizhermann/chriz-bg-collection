"""November 23 intake: Chriz BG Modpack v0.2.0-alpha.18 (component 470) in BG and IWD.

Modpack PRs #24 and #25 add default component 470 (cbm_scroll_learn_fix) after 460 in TP2
declaration order and publish v0.2.0-alpha.18 (which also includes PR #23 fast traps 690, left
unoffered and off in both installers until approved).
This is static pin, evidence, selection, and order coverage; standalone op147 behavior was
verified live in quiet BG2:EE and IWD:EE 2.7.3.0 labs with EEex v1.3.1, not a full collection
WeiDU install or BG:EE/SoD live launch.
"""

from __future__ import annotations

import json
from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import _effective_features, build_recipe


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261123-chriz-bg-modpack-chriz-easy-bg-chriz-easy-iwd.toml"
PREVIOUS_EVIDENCE = "manifest/evidence/release-intake-20261122-chriz-bg-modpack-chriz-easy-bg.toml"
MOD = "chriz-bg-modpack"
PIN = "chriz-bg-modpack-0.2.0-alpha.18"
PREVIOUS = "chriz-bg-modpack-0.2.0-alpha.17"
SUCCESSOR = "chriz-bg-modpack-0.2.0-alpha.21"
TAG = "v0.2.0-alpha.18"
SOURCE_COMMIT = "533913c1d4874d34aceb800c54305c779e3c0748"
PIN_COMMIT = "eddb2b9650373e4d9d9cebf140db8c5adbb03414"
FILENAME = "chriz-bg-modpack-v0.2.0-alpha.18.zip"
URL = f"https://github.com/Chrizhermann/chriz-bg-modpack/releases/download/{TAG}/{FILENAME}"
LENGTH = 9_554_843
SHA256 = "ea7fd706d2e7eded3af29853874603e4fc7963fa18193d0c5df1757dbc3bcb6a"
TP2_SHA256 = "400dc518c3db8fde508e1c20acb615a74676d8f8f8d2a17469ceddc0bd53a18a"
COMPONENT = 470
UNOFFERED = 690
BG_RUN = "chriz-bg-modpack-bg2"
IWD_RUN = "chriz-bg-modpack-main"
IWD_PROFILE = "recipes/iwd-current"
FEATURE = f"feature:{MOD}:component-{COMPONENT}"
LABEL = "Spell Learning: restrict +/-15% specialist modifier to specialist mages (BG:EE, SoD, BG2:EE, EET, IWD:EE; Windows EEex)"
DIGEST_0117 = "6aceb183900fb7b3640dbedc8f4dd84e931c7cfe1549aa0c31cf115726d397c1"
DIGEST_0118 = "bb1b5b7fabc87f7a6ae5c4b96355eab7fae53698cd39a029b0b45852fa13de0d"
DIGEST_0119 = "3eadb7a097a3476a876fc2ea667fb27e6544b0228683348b9c10695048724891"


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


class Nov23ModpackAlpha18ScrollLearnFixTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.temp = tempfile.TemporaryDirectory()
        cls.generated = Path(cls.temp.name) / "recipe"
        build_recipe(ROOT, cls.generated, "0" * 40)
        cls.evidence = _load(ROOT / EVIDENCE)
        (cls.record,) = cls.evidence["artifacts"]
        (cls.previous_record,) = _load(ROOT / PREVIOUS_EVIDENCE)["artifacts"]
        cls.bg_directories = [
            ROOT / "manifest",
            ROOT / "recipes/curated-full-current",
            cls.generated,
        ]

    @classmethod
    def tearDownClass(cls) -> None:
        cls.temp.cleanup()

    def test_evidence_freezes_v0_2_0_alpha_18_and_native_order_for_all_three_profiles(self) -> None:
        self.assertEqual(self.evidence["verified_on"], "2026-11-23")
        self.assertEqual(self.evidence["generator"], "tools/prepare_own_mod_pin.py + chriz-bg-author + check_native_component_order.py")
        for phrase in (
            "Modpack PRs #24 and #25",
            "replaces chriz-bg-modpack-0.2.0-alpha.17",
            "Component 470",
            "component 690 (Fast Traps) stays unoffered and off in all profiles",
            "not a combined collection WeiDU installation or full campaign gameplay",
        ):
            self.assertIn(phrase, self.evidence["scope"])
        self.assertEqual(self.record["mod_id"], MOD)
        self.assertEqual(self.record["artifact_id"], PIN)
        self.assertEqual(self.record["recipe_profiles"], ["manifest", "recipes/curated-full-current", IWD_PROFILE])
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
        native.insert(native.index(460) + 1, COMPONENT)
        native.insert(native.index(680) + 1, UNOFFERED)
        self.assertEqual(self.record["native_components"], native)

    def test_bg_and_iwd_profiles_pin_alpha18_enable_470_and_omit_690(self) -> None:
        iwd_root = ROOT / IWD_PROFILE
        for directory in [*self.bg_directories, iwd_root]:
            is_iwd = directory == iwd_root
            with self.subTest(profile=str(directory)):
                mod = _load(directory / f"mods/{MOD}.toml")
                if is_iwd:
                    self.assertFalse((directory / f"artifacts/{PIN}.toml").exists())
                    self.assertTrue((directory / f"artifacts/{SUCCESSOR}.toml").exists())
                    artifact = _load(ROOT / f"manifest/artifacts/{PIN}.toml")
                else:
                    artifact = _load(directory / f"artifacts/{PIN}.toml")
                    previous = _load(directory / f"artifacts/{PREVIOUS}.toml")
                    self.assertEqual(artifact["archive"], previous["archive"])
                    self.assertEqual(artifact["source"]["redirect_hosts"], previous["source"]["redirect_hosts"])
                self.assertEqual(mod["artifact_id"], SUCCESSOR)
                self.assertEqual(artifact["version"], TAG.removeprefix("v"))
                self.assertEqual(artifact["source"]["reference"], TAG)
                self.assertEqual(artifact["source"]["url"], URL)
                self.assertEqual(artifact["source"]["expected_filename"], FILENAME)
                self.assertEqual(artifact["source"]["expected_length"], LENGTH)
                self.assertEqual(artifact["source"]["sha256"], SHA256)

                collection = _load(directory / "collection.toml")
                preset = _load(directory / "presets/chris-recommended.toml").get("selections", {})
                features = {feature["id"]: feature for feature in collection["features"]}
                self.assertNotIn(f"feature:{MOD}:component-{UNOFFERED}", features)
                names = {item["id"]: item["name"] for item in mod["components"]}
                self.assertEqual(names[COMPONENT], LABEL)
                self.assertNotIn(UNOFFERED, names)

                feature = features[FEATURE]
                self.assertEqual(feature["category"], "rules")
                self.assertEqual(feature["readiness"], "ready")
                self.assertEqual(feature["parent"], f"mod:{MOD}")
                native = set(self.record["native_components"])
                # Alpha.19 adds component 695 after 680; alpha.20/alpha.21 add 696, 697, 698 in BG; earlier declarations keep their order.
                for r in (r for r in collection["runs"] if r["mod_id"] == MOD):
                    self.assertTrue(set(r["components"]) - {695, 696, 697, 698} <= native, r["run_id"])
                    self.assertNotIn(UNOFFERED, r["components"], r["run_id"])

                if is_iwd:
                    (run,) = [run for run in collection["runs"] if run["mod_id"] == MOD]
                    self.assertEqual(run["run_id"], IWD_RUN)
                    self.assertEqual(run["phase"], "main")
                    self.assertEqual(run["components"], [COMPONENT])
                    order = [r["run_id"] for r in collection["runs"]]
                    index = order.index(IWD_RUN)
                    self.assertEqual(order[index - 1:index + 2], ["chriz-bg-rebalance-main", IWD_RUN, "eeexremote-main"])
                    self.assertEqual(features[f"mod:{MOD}"]["decision"], "default")
                    self.assertEqual(feature["decision"], "mandatory")
                    self.assertEqual(feature["requires"], ["feature:eeex:component-1"])
                    self.assertEqual(feature["components"], [{"run_id": IWD_RUN, "component": COMPONENT}])
                    choices = {fid: "on" for fid, row in features.items() if row["decision"] == "default"}
                    choices.update(preset)
                    baseline = _selected(collection, choices)
                    self.assertIn((IWD_RUN, COMPONENT), baseline)
                    for disabled in (f"mod:{MOD}", "mod:eeex"):
                        self.assertNotIn((IWD_RUN, COMPONENT), _selected(collection, {**choices, disabled: "off"}), disabled)
                else:
                    (run,) = [run for run in collection["runs"] if run["run_id"] == BG_RUN]
                    self.assertEqual(run["phase"], "post-eet-end")
                    idx_460 = run["components"].index(460)
                    self.assertEqual(run["components"][idx_460 + 1:idx_460 + 3], [COMPONENT, 610])
                    self.assertEqual(list(names)[list(names).index(460) + 1:list(names).index(460) + 3], [COMPONENT, 610])
                    self.assertEqual(feature["decision"], "default")
                    self.assertEqual(feature["requires"], ["feature:eeex:mandatory-components"])
                    self.assertEqual(feature["components"], [{"run_id": BG_RUN, "component": COMPONENT}])
                    self.assertEqual(preset[FEATURE], "on")
                    baseline = _selected(collection, preset)
                    self.assertIn((BG_RUN, COMPONENT), baseline)
                    for disabled in (FEATURE, f"mod:{MOD}", "mod:eeex"):
                        self.assertNotIn((BG_RUN, COMPONENT), _selected(collection, {**preset, disabled: "off"}), disabled)

    def test_iwd_0_1_19_release_ledger_hotpatch_and_docs(self) -> None:
        release = json.loads((ROOT / IWD_PROFILE / "release.json").read_text(encoding="utf-8"))
        self.assertEqual(release["version"], "0.1.21")
        ledger = _load(ROOT / f"{IWD_PROFILE}/releases/v0.1.19/ledger.toml")
        self.assertEqual((ledger["recipe_id"], ledger["version"], ledger["supersedes"]), ("chriz-easy-iwd", "0.1.19", "0.1.18"))
        (change,) = ledger["changes"]
        self.assertEqual((change["id"], change["save_applicability"], change["urgency"]), (MOD, "new-game-only", "informational"))
        self.assertEqual(set(change["covers"]), {f"artifact:{PIN}", f"feature:mod:{MOD}", f"feature:{FEATURE}", f"run:{IWD_RUN}"})

        bases = {base["id"]: base for base in _load(ROOT / "hotpatches/iwd/catalog.toml")["bases"]}
        self.assertEqual(bases["iwd-0.1.17"]["content_sha256"], DIGEST_0117)
        self.assertEqual(bases["iwd-0.1.18"]["recipe_version"], "0.1.18")
        self.assertEqual(bases["iwd-0.1.18"]["content_sha256"], DIGEST_0118)
        self.assertEqual(bases["iwd-0.1.19"]["recipe_version"], "0.1.19")
        self.assertEqual(bases["iwd-0.1.19"]["label"], "Chriz Easy IWD 0.1.17")
        self.assertEqual(bases["iwd-0.1.19"]["content_sha256"], DIGEST_0119)

        curation = _load(ROOT / "manifest/curation-map.toml")
        targets = {target["id"]: target for target in curation["targets"]}
        self.assertEqual(targets[FEATURE]["kind"], "feature")
        self.assertEqual(targets[FEATURE]["rows"], [f"CHRIZ-BG-MODPACK:{COMPONENT}"])
        self.assertEqual(targets[FEATURE]["parent"], f"mod:{MOD}")

        catalog = (ROOT / "docs/curation/components/CHRIZ-BG-MODPACK.md").read_text(encoding="utf-8")
        for phrase in (
            "**v0.2.0-alpha.18**",
            "533913c",
            "Modpack PRs #24 and #25",
            EVIDENCE,
            f"| {COMPONENT} | {LABEL} | chriz-bg-modpack: Spell and compatibility fixes | | | default |",
            "71 current or historical entries; the collection exposes 54.",
        ):
            self.assertIn(phrase, catalog)

        source = next(
            line for line in (ROOT / "manifest/mod-sources.tsv").read_text(encoding="utf-8").splitlines()
            if line.startswith("CHRIZ-BG-MODPACK\t")
        )
        self.assertIn("470", source)
        pin_list = (ROOT / "docs/pin-list-2.7.md").read_text(encoding="utf-8")
        self.assertNotIn(f"| chriz-bg-modpack | {TAG.removeprefix('v')} |", pin_list)

        for rel, fragment in (
            ("docs/curation/iwdee/INVENTORY.md", "Chriz BG Modpack v0.2.0-alpha.18"),
            ("docs/curation/iwdee/WORKSHEET.md", "690 Fast Traps stays unoffered"),
            ("docs/curation/iwdee/FOLLOW_UPS.md", "Chriz BG Modpack v0.2.0-alpha.18 Spell Learning RAW fix (470"),
        ):
            self.assertIn(fragment, (ROOT / rel).read_text(encoding="utf-8"), rel)

        changelog = (ROOT / "docs/plans/2026-10-14-next-release-changelogs.md").read_text(encoding="utf-8")
        current = changelog.split("\n## Next Chriz Easy BG release (draft)\n", 1)[1].split("\n## Collection 0.1.2 (draft)\n", 1)[0]
        self.assertIn("**Spell scroll learning matches Rules as Written** (Chriz BG Modpack component 470", current)
        self.assertIn("v0.2.0-alpha.18", current)
        self.assertIn(EVIDENCE, current)
        self.assertIn(f"feature:{FEATURE}", current)

    def test_static_acceptance_records_remain_valid(self) -> None:
        bg_runs = (
            "chriz-bg-modpack-bg2",
            "chriz-bg-modpack-edwin-bg2",
            "chriz-bg-modpack-baeloth-bg2",
            "chriz-bg-modpack-scrolls-bg2",
            "chriz-bg-modpack-pre-continuity-bg2",
            "chriz-bg-modpack-continuity-bg2",
            "chriz-bg-modpack-late-companions-bg2",
        )
        for profile, expected_runs in (
            ("manifest", bg_runs),
            ("recipes/curated-full-current", bg_runs),
            (IWD_PROFILE, (IWD_RUN,)),
        ):
            evidence = _load(ROOT / f"{profile}/releases/v0.1.0-alpha.1/acceptance.toml")["evidence"]
            by_run = {
                row["subject_id"]: row
                for row in evidence
                if row["subject_kind"] == "run"
            }
            for run_id in expected_runs:
                with self.subTest(profile=profile, run_id=run_id):
                    record = by_run[run_id]
                    self.assertEqual((record["kind"], record["status"]), ("static-test", "accepted"))


if __name__ == "__main__":
    unittest.main()
