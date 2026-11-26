"""November 27 intake: Chriz BG Modpack v0.2.0-alpha.20 (components 697 and 698) in BG, pin bump in IWD.

Modpack PR #28 adds default Watcher's Keep: instant teleporters 697 (cbm_watchers_keep_teleporters)
and Clear lingering room effects when leaving an area or hazard room 698
(cbm_area_effects_clear_on_leave) after 695 in TP2 declaration order and publishes v0.2.0-alpha.20.
Chriz Easy BG enables components 697 and 698 by default (698 with EEex), while Chriz Easy IWD
updates the pinned Modpack package to v0.2.0-alpha.20 (collection 0.1.21) keeping component 470
as its sole Modpack component and omitting 690, 695, 697, and 698.
"""

from __future__ import annotations

import json
from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import _effective_features, build_recipe


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261127-chriz-bg-modpack-v0.2.0-alpha.20-chriz-easy-bg-chriz-easy-iwd.toml"
PREVIOUS_EVIDENCE = "manifest/evidence/release-intake-20261127-chriz-bg-modpack-chriz-easy-bg-chriz-easy-iwd.toml"
MOD = "chriz-bg-modpack"
PIN = "chriz-bg-modpack-0.2.0-alpha.20"
PREVIOUS = "chriz-bg-modpack-0.2.0-alpha.19"
TAG = "v0.2.0-alpha.20"
SOURCE_COMMIT = "099aaf4761ece2134a797010f5f3c679823b46cd"
FILENAME = "chriz-bg-modpack-v0.2.0-alpha.20.zip"
URL = f"https://github.com/Chrizhermann/chriz-bg-modpack/releases/download/{TAG}/{FILENAME}"
LENGTH = 9_590_173
SHA256 = "6108ad40ca751226457386b6774e61c41722b0cd998725e762c4502bf461dc89"
TP2_SHA256 = "1c644351745fa30c54c29b373bb2b5de0e6e4831b006dc8c4a4b49d339f7b3f9"
COMPONENTS = (697, 698)
UNOFFERED = 690
BG_RUN = "chriz-bg-modpack-bg2"
IWD_RUN = "chriz-bg-modpack-main"
IWD_PROFILE = "recipes/iwd-current"
LABELS = {
    697: "Watcher's Keep: instant teleporters",
    698: "Clear lingering room effects when leaving an area or hazard room (BG:EE, SoD, BG2:EE, EET; Windows EEex)",
}
DIGEST_0120 = "169e05d1a2607ebc69e7016732b42c8d18024dd16f814e5b67796e9249488526"
DIGEST_0121 = "42c9ae4a4b0596b14b6fc17ac59b968f441d49bc1cf808d910b40b14fa39d758"


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


class Nov27ModpackAlpha20WkTeleportsRoomEffectsTests(unittest.TestCase):
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

    def test_evidence_freezes_v0_2_0_alpha_20_and_native_order_for_all_three_profiles(self) -> None:
        self.assertEqual(self.evidence["verified_on"], "2026-11-27")
        self.assertEqual(self.evidence["generator"], "tools/prepare_own_mod_pin.py + chriz-bg-author + check_native_component_order.py")
        for phrase in (
            "Modpack PR #28",
            "replaces chriz-bg-modpack-0.2.0-alpha.19",
            "Component 697",
            "component 698",
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
        idx = native.index(695) + 1
        native[idx:idx] = list(COMPONENTS)
        self.assertEqual(self.record["native_components"], native)

    def test_bg_enables_697_and_698_by_default_and_iwd_omits_them_and_690(self) -> None:
        iwd_root = ROOT / IWD_PROFILE
        for directory in [*self.bg_directories, iwd_root]:
            is_iwd = directory == iwd_root
            with self.subTest(profile=str(directory)):
                mod = _load(directory / f"mods/{MOD}.toml")
                artifact = _load(directory / f"artifacts/{PIN}.toml")
                self.assertEqual(mod["artifact_id"], PIN)
                self.assertEqual(artifact["version"], TAG.removeprefix("v"))
                self.assertEqual(artifact["source"]["reference"], TAG)
                self.assertEqual(artifact["source"]["url"], URL)
                self.assertEqual(artifact["source"]["expected_filename"], FILENAME)
                self.assertEqual(artifact["source"]["expected_length"], LENGTH)
                self.assertEqual(artifact["source"]["sha256"], SHA256)
                if is_iwd:
                    self.assertEqual(artifact, _load(ROOT / f"manifest/artifacts/{PIN}.toml"))
                else:
                    previous = _load(directory / f"artifacts/{PREVIOUS}.toml")
                    self.assertEqual(artifact["archive"], previous["archive"])
                    self.assertEqual(artifact["source"]["redirect_hosts"], previous["source"]["redirect_hosts"])

                collection = _load(directory / "collection.toml")
                preset = _load(directory / "presets/chris-recommended.toml").get("selections", {})
                features = {feature["id"]: feature for feature in collection["features"]}
                self.assertNotIn(f"feature:{MOD}:component-{UNOFFERED}", features)
                names = {item["id"]: item["name"] for item in mod["components"]}
                self.assertNotIn(UNOFFERED, names)

                native = set(self.record["native_components"])
                for r in (r for r in collection["runs"] if r["mod_id"] == MOD):
                    self.assertTrue(set(r["components"]) <= native, r["run_id"])
                    self.assertNotIn(UNOFFERED, r["components"], r["run_id"])

                if is_iwd:
                    for cid in COMPONENTS:
                        self.assertNotIn(f"feature:{MOD}:component-{cid}", features)
                        self.assertNotIn(cid, names)
                    (run,) = [run for run in collection["runs"] if run["mod_id"] == MOD]
                    self.assertEqual(run["run_id"], IWD_RUN)
                    self.assertEqual(run["phase"], "main")
                    self.assertEqual(run["components"], [470])
                else:
                    (run,) = [run for run in collection["runs"] if run["run_id"] == BG_RUN]
                    self.assertEqual(run["phase"], "post-eet-end")
                    idx_695 = run["components"].index(695)
                    self.assertEqual(run["components"][idx_695 + 1:], [697, 698])
                    idx_name_695 = list(names).index(695)
                    self.assertEqual(list(names)[idx_name_695 + 1:idx_name_695 + 3], [697, 698])

                    f697 = features[f"feature:{MOD}:component-697"]
                    self.assertEqual(names[697], LABELS[697])
                    self.assertEqual(f697["category"], "convenience")
                    self.assertEqual(f697["readiness"], "ready")
                    self.assertEqual(f697["parent"], f"mod:{MOD}")
                    self.assertEqual(f697["decision"], "default")
                    self.assertEqual(f697.get("requires", []), [])
                    self.assertEqual(f697["components"], [{"run_id": BG_RUN, "component": 697}])
                    self.assertEqual(preset[f697["id"]], "on")

                    f698 = features[f"feature:{MOD}:component-698"]
                    self.assertEqual(names[698], LABELS[698])
                    self.assertEqual(f698["category"], "fixes")
                    self.assertEqual(f698["readiness"], "ready")
                    self.assertEqual(f698["parent"], f"mod:{MOD}")
                    self.assertEqual(f698["decision"], "default")
                    self.assertEqual(f698["requires"], ["feature:eeex:mandatory-components"])
                    self.assertEqual(f698["components"], [{"run_id": BG_RUN, "component": 698}])
                    self.assertEqual(preset[f698["id"]], "on")

                    baseline = _selected(collection, preset)
                    self.assertIn((BG_RUN, 697), baseline)
                    self.assertIn((BG_RUN, 698), baseline)
                    for disabled in (f697["id"], f"mod:{MOD}"):
                        self.assertNotIn((BG_RUN, 697), _selected(collection, {**preset, disabled: "off"}), disabled)
                    for disabled in (f698["id"], f"mod:{MOD}", "mod:eeex"):
                        self.assertNotIn((BG_RUN, 698), _selected(collection, {**preset, disabled: "off"}), disabled)

    def test_iwd_0_1_21_release_ledger_hotpatch_and_docs(self) -> None:
        release = json.loads((ROOT / IWD_PROFILE / "release.json").read_text(encoding="utf-8"))
        self.assertEqual(release["version"], "0.1.21")
        ledger = _load(ROOT / f"{IWD_PROFILE}/releases/v0.1.21/ledger.toml")
        self.assertEqual((ledger["recipe_id"], ledger["version"], ledger["supersedes"]), ("chriz-easy-iwd", "0.1.21", "0.1.20"))
        (change,) = ledger["changes"]
        self.assertEqual((change["id"], change["save_applicability"], change["urgency"]), (MOD, "new-game-only", "informational"))
        self.assertEqual(
            set(change["covers"]),
            {f"artifact:{PIN}", f"feature:mod:{MOD}", f"feature:feature:{MOD}:component-470", f"run:{IWD_RUN}"},
        )

        bases = {base["id"]: base for base in _load(ROOT / "hotpatches/iwd/catalog.toml")["bases"]}
        self.assertEqual(bases["iwd-0.1.20"]["content_sha256"], DIGEST_0120)
        self.assertEqual(bases["iwd-0.1.21"]["recipe_version"], "0.1.21")
        self.assertEqual(bases["iwd-0.1.21"]["label"], "Chriz Easy IWD 0.1.17")
        self.assertEqual(bases["iwd-0.1.21"]["content_sha256"], DIGEST_0121)

        curation = _load(ROOT / "manifest/curation-map.toml")
        targets = {target["id"]: target for target in curation["targets"]}
        for cid in COMPONENTS:
            feat = f"feature:{MOD}:component-{cid}"
            self.assertEqual(targets[feat]["kind"], "feature")
            self.assertEqual(targets[feat]["rows"], [f"CHRIZ-BG-MODPACK:{cid}"])
            self.assertEqual(targets[feat]["parent"], f"mod:{MOD}")

        catalog = (ROOT / "docs/curation/components/CHRIZ-BG-MODPACK.md").read_text(encoding="utf-8")
        for phrase in (
            "**v0.2.0-alpha.20**",
            "099aaf4",
            "Modpack PR #28",
            EVIDENCE,
            f"| 697 | {LABELS[697]} | chriz-bg-modpack: Gameplay tweaks | | | default |",
            f"| 698 | {LABELS[698]} | chriz-bg-modpack: Compatibility | | | default |",
            "70 current or historical entries; the collection exposes 53.",
        ):
            self.assertIn(phrase, catalog)

        source = next(
            line for line in (ROOT / "manifest/mod-sources.tsv").read_text(encoding="utf-8").splitlines()
            if line.startswith("CHRIZ-BG-MODPACK\t")
        )
        for phrase in (TAG, SOURCE_COMMIT, str(LENGTH), SHA256):
            self.assertIn(phrase, source)
        pin_list = (ROOT / "docs/pin-list-2.7.md").read_text(encoding="utf-8")
        self.assertEqual(pin_list.count(f"| chriz-bg-modpack | {TAG.removeprefix('v')} | `{MOD}` | `{SHA256[:12]}…` |"), 2)

        for rel, fragment in (
            ("docs/curation/iwdee/INVENTORY.md", "v0.2.0-alpha.20"),
            ("docs/curation/iwdee/WORKSHEET.md", "v0.2.0-alpha.20"),
            ("docs/curation/iwdee/FOLLOW_UPS.md", "v0.2.0-alpha.20"),
        ):
            self.assertIn(fragment, (ROOT / rel).read_text(encoding="utf-8"), rel)

        changelog = (ROOT / "docs/plans/2026-10-14-next-release-changelogs.md").read_text(encoding="utf-8")
        current = changelog.split("\n## Next Chriz Easy BG release (draft)\n", 1)[1].split("\n## Collection 0.1.2 (draft)\n", 1)[0]
        self.assertIn("**Watcher's Keep instant teleporters and instant room-hazard cleanup**", current)
        self.assertIn("Chriz BG Modpack v0.2.0-alpha.20 (was v0.2.0-alpha.19)", current)
        self.assertIn(EVIDENCE, current)
        self.assertIn(f"artifact:{PIN}", current)
        self.assertIn(f"feature:feature:{MOD}:component-697", current)
        self.assertIn(f"feature:feature:{MOD}:component-698", current)


if __name__ == "__main__":
    unittest.main()
