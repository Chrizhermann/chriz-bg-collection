"""November 27 intake: Chriz BG Modpack v0.2.0-alpha.19 (component 695) in BG, pin bump in IWD.

Modpack PR #26 adds default BG1 early-party XP boost 695 (cbm_bg1_early_party_xp) after 680 in
TP2 declaration order and publishes v0.2.0-alpha.19. Chriz Easy BG enables component 695 by
default with EEex + EET, while Chriz Easy IWD updates the pinned Modpack package to v0.2.0-alpha.19
(collection 0.1.20) keeping component 470 as its sole Modpack component and omitting 690 and 695.
"""

from __future__ import annotations

import json
from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import _effective_features, build_recipe


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261127-chriz-bg-modpack-chriz-easy-bg-chriz-easy-iwd.toml"
PREVIOUS_EVIDENCE = "manifest/evidence/release-intake-20261123-chriz-bg-modpack-chriz-easy-bg-chriz-easy-iwd.toml"
MOD = "chriz-bg-modpack"
PIN = "chriz-bg-modpack-0.2.0-alpha.19"
PREVIOUS = "chriz-bg-modpack-0.2.0-alpha.18"
SUCCESSOR = "chriz-bg-modpack-0.2.0-alpha.22"
TAG = "v0.2.0-alpha.19"
SOURCE_COMMIT = "6e6b3643682d50a63abcfe99a958afeb0ae3a108"
PIN_COMMIT = "864102896d76b54c5cdb6e4ca7ce68610dd0a77b"
FILENAME = "chriz-bg-modpack-v0.2.0-alpha.19.zip"
URL = f"https://github.com/Chrizhermann/chriz-bg-modpack/releases/download/{TAG}/{FILENAME}"
LENGTH = 9_564_616
SHA256 = "c2f1fc147f7f426ae6d1a36d2348e959f6313e009d96593afca02106cb1154d5"
TP2_SHA256 = "0d017622b51fad4e3e8eb931a9200d97d190a5a8568c9c5e44167a333569ba25"
COMPONENT = 695
UNOFFERED = 690
BG_RUN = "chriz-bg-modpack-bg2"
IWD_RUN = "chriz-bg-modpack-main"
IWD_PROFILE = "recipes/iwd-current"
FEATURE = f"feature:{MOD}:component-{COMPONENT}"
LABEL = "BG1 early-party XP boost: +70% party XP pool per extra companion up to 16,000 XP (BG:EE and EET; Windows EEex)"
DIGEST_0119 = "3eadb7a097a3476a876fc2ea667fb27e6544b0228683348b9c10695048724891"
DIGEST_0120 = "169e05d1a2607ebc69e7016732b42c8d18024dd16f814e5b67796e9249488526"


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


class Nov26ModpackAlpha19Bg1EarlyPartyXpTests(unittest.TestCase):
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

    def test_evidence_freezes_v0_2_0_alpha_19_and_native_order_for_all_three_profiles(self) -> None:
        self.assertEqual(self.evidence["verified_on"], "2026-11-27")
        self.assertEqual(self.evidence["generator"], "tools/prepare_own_mod_pin.py + chriz-bg-author + check_native_component_order.py")
        for phrase in (
            "Modpack PR #26",
            "replaces chriz-bg-modpack-0.2.0-alpha.18",
            "Component 695",
            "omitted from Chriz Easy IWD",
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
        native.insert(native.index(UNOFFERED) + 1, COMPONENT)
        self.assertEqual(self.record["native_components"], native)

    def test_bg_enables_695_by_default_and_iwd_omits_695_and_690(self) -> None:
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
                self.assertNotIn(UNOFFERED, names)

                native = set(self.record["native_components"])
                # Alpha.20/alpha.21 add 696, 697, and 698 after 695 in BG; alpha.22 adds 699 and 700;
                # earlier declarations keep their order.
                for r in (r for r in collection["runs"] if r["mod_id"] == MOD):
                    self.assertTrue(set(r["components"]) - {696, 697, 698, 699, 700} <= native, r["run_id"])
                    self.assertNotIn(UNOFFERED, r["components"], r["run_id"])

                if is_iwd:
                    self.assertNotIn(FEATURE, features)
                    self.assertNotIn(COMPONENT, names)
                    (run,) = [run for run in collection["runs"] if run["mod_id"] == MOD]
                    self.assertEqual(run["run_id"], IWD_RUN)
                    self.assertEqual(run["phase"], "main")
                    self.assertEqual(run["components"], [470, 699, 700])
                    self.assertNotIn(COMPONENT, run["components"])
                else:
                    feature = features[FEATURE]
                    self.assertEqual(names[COMPONENT], LABEL)
                    (run,) = [run for run in collection["runs"] if run["run_id"] == BG_RUN]
                    self.assertEqual(run["phase"], "post-eet-end")
                    idx_680 = run["components"].index(680)
                    self.assertEqual(run["components"][idx_680 + 1:idx_680 + 2], [COMPONENT])
                    self.assertEqual(list(names)[list(names).index(680) + 1:list(names).index(680) + 2], [COMPONENT])
                    self.assertEqual(feature["category"], "rules")
                    self.assertEqual(feature["readiness"], "ready")
                    self.assertEqual(feature["parent"], f"mod:{MOD}")
                    self.assertEqual(feature["decision"], "default")
                    self.assertEqual(
                        feature["requires"],
                        ["feature:eeex:mandatory-components", "feature:eet:mandatory-components"],
                    )
                    self.assertEqual(feature["components"], [{"run_id": BG_RUN, "component": COMPONENT}])
                    self.assertEqual(preset[FEATURE], "on")
                    baseline = _selected(collection, preset)
                    self.assertIn((BG_RUN, COMPONENT), baseline)
                    for disabled in (FEATURE, f"mod:{MOD}", "mod:eeex"):
                        self.assertNotIn((BG_RUN, COMPONENT), _selected(collection, {**preset, disabled: "off"}), disabled)

    def test_iwd_0_1_20_release_ledger_hotpatch_and_docs(self) -> None:
        release = json.loads((ROOT / IWD_PROFILE / "release.json").read_text(encoding="utf-8"))
        self.assertIn(release["version"], ("0.1.22", "0.1.23"))
        ledger = _load(ROOT / f"{IWD_PROFILE}/releases/v0.1.20/ledger.toml")
        self.assertEqual((ledger["recipe_id"], ledger["version"], ledger["supersedes"]), ("chriz-easy-iwd", "0.1.20", "0.1.19"))
        (change,) = ledger["changes"]
        self.assertEqual((change["id"], change["save_applicability"], change["urgency"]), (MOD, "new-game-only", "informational"))
        self.assertEqual(
            set(change["covers"]),
            {f"artifact:{PIN}", f"feature:mod:{MOD}", f"feature:feature:{MOD}:component-470", f"run:{IWD_RUN}"},
        )

        bases = {base["id"]: base for base in _load(ROOT / "hotpatches/iwd/catalog.toml")["bases"]}
        self.assertEqual(bases["iwd-0.1.19"]["content_sha256"], DIGEST_0119)
        self.assertEqual(bases["iwd-0.1.20"]["recipe_version"], "0.1.20")
        self.assertEqual(bases["iwd-0.1.20"]["label"], "Chriz Easy IWD 0.1.17")
        self.assertEqual(bases["iwd-0.1.20"]["content_sha256"], DIGEST_0120)

        curation = _load(ROOT / "manifest/curation-map.toml")
        targets = {target["id"]: target for target in curation["targets"]}
        self.assertEqual(targets[FEATURE]["kind"], "feature")
        self.assertEqual(targets[FEATURE]["rows"], [f"CHRIZ-BG-MODPACK:{COMPONENT}"])
        self.assertEqual(targets[FEATURE]["parent"], f"mod:{MOD}")

        catalog = (ROOT / "docs/curation/components/CHRIZ-BG-MODPACK.md").read_text(encoding="utf-8")
        for phrase in (
            "**v0.2.0-alpha.19**",
            "6e6b364",
            "Modpack PR #26",
            EVIDENCE,
            f"| {COMPONENT} | {LABEL} | chriz-bg-modpack: Gameplay tweaks | | | default |",
            "73 current or historical entries; the collection exposes 56.",
        ):
            self.assertIn(phrase, catalog)

        source = next(
            line for line in (ROOT / "manifest/mod-sources.tsv").read_text(encoding="utf-8").splitlines()
            if line.startswith("CHRIZ-BG-MODPACK\t")
        )
        self.assertIn("695", source)
        pin_list = (ROOT / "docs/pin-list-2.7.md").read_text(encoding="utf-8")
        self.assertNotIn(f"| chriz-bg-modpack | {TAG.removeprefix('v')} |", pin_list)

        for rel, fragment in (
            ("docs/curation/iwdee/INVENTORY.md", "v0.2.0-alpha.19"),
            ("docs/curation/iwdee/WORKSHEET.md", "v0.2.0-alpha.19"),
            ("docs/curation/iwdee/FOLLOW_UPS.md", "695, 696, 697, and 698 omitted from IWD"),
        ):
            self.assertIn(fragment, (ROOT / rel).read_text(encoding="utf-8"), rel)

        changelog = (ROOT / "docs/plans/2026-10-14-next-release-changelogs.md").read_text(encoding="utf-8")
        current = changelog.split("\n## Next Chriz Easy BG release (draft)\n", 1)[1].split("\n## Collection 0.1.2 (draft)\n", 1)[0]
        self.assertIn("**BG1 early-party XP boost** (Chriz BG Modpack component 695", current)
        self.assertIn("v0.2.0-alpha.19", current)
        self.assertIn(EVIDENCE, current)
        self.assertIn(f"artifact:{PIN}", current)
        self.assertIn(f"feature:{FEATURE}", current)

    def test_static_acceptance_records_remain_valid(self) -> None:
        bg_runs = {
            "chriz-bg-modpack-bg2",
            "chriz-bg-modpack-pre-continuity-bg2",
            "chriz-bg-modpack-continuity-bg2",
            "chriz-bg-modpack-late-companions-bg2",
            "chriz-bg-modpack-edwin-bg2",
            "chriz-bg-modpack-baeloth-bg2",
            "chriz-bg-modpack-scrolls-bg2",
        }
        for rel_path, expected_runs in (
            ("manifest/releases/v0.1.0-alpha.1/acceptance.toml", bg_runs),
            ("recipes/curated-full-current/releases/v0.1.0-alpha.1/acceptance.toml", bg_runs),
            (f"{IWD_PROFILE}/releases/v0.1.0-alpha.1/acceptance.toml", {"chriz-bg-modpack-main"}),
        ):
            data = tomllib.loads((ROOT / rel_path).read_text(encoding="utf-8"))
            by_id = {
                entry.get("subject_id"): entry
                for entry in data.get("evidence", [])
                if entry.get("subject_kind") == "run"
            }
            for run_id in expected_runs:
                entry = by_id[run_id]
                self.assertEqual((entry.get("kind"), entry.get("status")), ("static-test", "accepted"), (rel_path, run_id))


if __name__ == "__main__":
    unittest.main()
