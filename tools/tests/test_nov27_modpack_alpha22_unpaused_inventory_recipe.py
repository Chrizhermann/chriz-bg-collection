"""November 27 intake: Chriz BG Modpack v0.2.0-alpha.22 (699, 700) in BG and IWD.

Modpack PR #30 adds mutually exclusive optional EEex inventory-unpause components 699
(cbm_unpaused_inventory_always) and 700 (cbm_unpaused_inventory_combat) after 698 in TP2
declaration order on top of v0.2.0-alpha.21 (696, 697, 698). Both Chriz Easy BG and Chriz Easy
IWD expose 699 and 700 as optional mutually exclusive choices requiring EEex and off by default in
Chris's Recommended; Chriz Easy IWD updates the pinned Modpack package to v0.2.0-alpha.22
(collection 0.1.23) alongside mandatory component 470, while keeping 695, 696, 697, and 698 BG-only
and keeping 690 unoffered and off in all profiles.
"""

from __future__ import annotations

import json
from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import _effective_features, build_recipe


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261127-chriz-bg-modpack-v0.2.0-alpha.22-chriz-easy-bg-chriz-easy-iwd.toml"
PREVIOUS_EVIDENCE = "manifest/evidence/release-intake-20261127-chriz-bg-modpack-v0.2.0-alpha.21-chriz-easy-bg-chriz-easy-iwd.toml"
MOD = "chriz-bg-modpack"
PIN = "chriz-bg-modpack-0.2.0-alpha.22"
PREVIOUS = "chriz-bg-modpack-0.2.0-alpha.21"
TAG = "v0.2.0-alpha.22"
SOURCE_COMMIT = "283e59e3647ac1b422eae922b2efda62960aa072"
PIN_COMMIT = ""
FILENAME = "chriz-bg-modpack-v0.2.0-alpha.22.zip"
URL = f"https://github.com/Chrizhermann/chriz-bg-modpack/releases/download/{TAG}/{FILENAME}"
LENGTH = 9_607_048
SHA256 = "53044637cbd3259b3eabf8032b53137560aa2ae821074c8ecf8a64a42474a317"
TP2_SHA256 = "dfc689118152bd06df90bcb16df5196ba675a37c6617cec9ad7b2a557682ceeb"

ALWAYS = 699
COMBAT = 700
UNOFFERED = 690
BG_RUN = "chriz-bg-modpack-bg2"
IWD_RUN = "chriz-bg-modpack-main"
IWD_PROFILE = "recipes/iwd-current"
ALWAYS_FEATURE = f"feature:{MOD}:component-{ALWAYS}"
COMBAT_FEATURE = f"feature:{MOD}:component-{COMBAT}"
CHOICE_GROUP = "choice:chriz-bg-modpack:unpaused-inventory-mode"
LABELS = {
    ALWAYS: "Inventory: always keep game unpaused while Inventory is open (BG:EE, SoD, BG2:EE, EET, IWD:EE; Windows EEex)",
    COMBAT: "Inventory: keep game unpaused in Inventory during combat only (BG:EE, SoD, BG2:EE, EET, IWD:EE; Windows EEex)",
}
DIGEST_0122 = "31c158ea070afe5cd516d42593b9f133f58842ae4e4ecd56b033e4069fbb8e1e"
DIGEST_0123 = "ff2e35f3f41b9b24569c1f31b47e185c4f6df70e0cee9a5846c921425ecabba0"


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


class Nov27ModpackAlpha22UnpausedInventoryTests(unittest.TestCase):
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

    def test_evidence_freezes_v0_2_0_alpha_22_and_native_order_for_all_three_profiles(self) -> None:
        self.assertEqual(self.evidence["verified_on"], "2026-11-27")
        self.assertEqual(
            self.evidence["generator"],
            "tools/prepare_own_mod_pin.py + chriz-bg-author + check_native_component_order.py",
        )
        for phrase in (
            "Modpack PR #30",
            "replaces chriz-bg-modpack-0.2.0-alpha.21",
            "components 699",
            "and 700",
            "component 690 unoffered and off in all profiles",
            "not a combined collection WeiDU installation or full campaign gameplay",
        ):
            self.assertIn(phrase, self.evidence["scope"])
        self.assertEqual(self.record["mod_id"], MOD)
        self.assertEqual(self.record["artifact_id"], PIN)
        self.assertEqual(
            self.record["recipe_profiles"],
            ["manifest", "recipes/curated-full-current", IWD_PROFILE],
        )
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
        idx_698 = native.index(698)
        native[idx_698 + 1 : idx_698 + 1] = [ALWAYS, COMBAT]
        self.assertEqual(self.record["native_components"], native)

    def test_bg_and_iwd_expose_699_and_700_as_optional_mutually_exclusive_eeex_choices(self) -> None:
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
                    self.assertFalse((directory / f"artifacts/{PREVIOUS}.toml").exists())
                else:
                    previous = _load(directory / f"artifacts/{PREVIOUS}.toml")
                    self.assertEqual(artifact["archive"], previous["archive"])
                    self.assertEqual(
                        artifact["source"]["redirect_hosts"],
                        previous["source"]["redirect_hosts"],
                    )

                collection = _load(directory / "collection.toml")
                preset = _load(directory / "presets/chris-recommended.toml").get("selections", {})
                features = {feature["id"]: feature for feature in collection["features"]}
                self.assertNotIn(f"feature:{MOD}:component-{UNOFFERED}", features)
                names = {item["id"]: item["name"] for item in mod["components"]}
                self.assertNotIn(UNOFFERED, names)
                for cid, label in LABELS.items():
                    self.assertEqual(names[cid], label)

                native = set(self.record["native_components"])
                for r in (r for r in collection["runs"] if r["mod_id"] == MOD):
                    self.assertTrue(set(r["components"]) <= native, r["run_id"])
                    self.assertNotIn(UNOFFERED, r["components"], r["run_id"])

                run_id = IWD_RUN if is_iwd else BG_RUN
                (run,) = [run for run in collection["runs"] if run["run_id"] == run_id]
                if is_iwd:
                    for cid in (695, 696, 697, 698):
                        self.assertNotIn(f"feature:{MOD}:component-{cid}", features)
                        self.assertNotIn(cid, names)
                    self.assertEqual(run["phase"], "main")
                    self.assertEqual(run["components"], [470, ALWAYS, COMBAT])
                    expected_req = ["feature:eeex:component-1"]
                else:
                    self.assertEqual(run["phase"], "post-eet-end")
                    idx_698 = run["components"].index(698)
                    self.assertEqual(run["components"][idx_698 + 1 :], [ALWAYS, COMBAT])
                    self.assertEqual(
                        list(names)[list(names).index(698) + 1 : list(names).index(698) + 3],
                        [ALWAYS, COMBAT],
                    )
                    expected_req = ["feature:eeex:mandatory-components"]
                    (group,) = [g for g in collection["feature_groups"] if g["id"] == "challenge-mode"]
                    self.assertNotIn(ALWAYS_FEATURE, group["members"])
                    self.assertNotIn(COMBAT_FEATURE, group["members"])

                for fid, other_fid in ((ALWAYS_FEATURE, COMBAT_FEATURE), (COMBAT_FEATURE, ALWAYS_FEATURE)):
                    feat = features[fid]
                    self.assertEqual(
                        (feat["category"], feat["decision"], feat["readiness"]),
                        ("convenience", "optional", "ready"),
                    )
                    self.assertEqual(feat["parent"], f"mod:{MOD}")
                    self.assertEqual(feat["choice_group"], CHOICE_GROUP)
                    self.assertEqual(feat["requires"], expected_req)
                    self.assertEqual([c["feature_id"] for c in feat["conflicts"]], [other_fid])
                    self.assertNotIn(fid, preset)

                base_sel = (
                    {fid: "on" for fid, row in features.items() if row["decision"] == "default"} | preset
                    if is_iwd
                    else preset
                )
                baseline = _selected(collection, base_sel)
                self.assertNotIn((run_id, ALWAYS), baseline)
                self.assertNotIn((run_id, COMBAT), baseline)

                with_always = _selected(collection, {**base_sel, ALWAYS_FEATURE: "on"})
                self.assertEqual(with_always - baseline, {(run_id, ALWAYS)})

                with_combat = _selected(collection, {**base_sel, COMBAT_FEATURE: "on"})
                self.assertEqual(with_combat - baseline, {(run_id, COMBAT)})

                no_eeex = _selected(collection, {**base_sel, ALWAYS_FEATURE: "on", "mod:eeex": "off"})
                self.assertNotIn((run_id, ALWAYS), no_eeex)
                self.assertNotIn((run_id, COMBAT), no_eeex)

    def test_iwd_0_1_23_release_ledger_hotpatch_and_docs(self) -> None:
        release = json.loads((ROOT / IWD_PROFILE / "release.json").read_text(encoding="utf-8"))
        self.assertEqual(release["version"], "0.1.23")
        ledger = _load(ROOT / f"{IWD_PROFILE}/releases/v0.1.23/ledger.toml")
        self.assertEqual(
            (ledger["recipe_id"], ledger["version"], ledger["supersedes"]),
            ("chriz-easy-iwd", "0.1.23", "0.1.22"),
        )
        (change,) = ledger["changes"]
        self.assertEqual(
            (change["id"], change["save_applicability"], change["urgency"]),
            (PIN, "new-game-only", "informational"),
        )
        self.assertEqual(
            set(change["covers"]),
            {
                f"artifact:{PREVIOUS}",
                f"artifact:{PIN}",
                f"feature:mod:{MOD}",
                f"feature:{ALWAYS_FEATURE}",
                f"feature:{COMBAT_FEATURE}",
                f"run:{IWD_RUN}",
            },
        )

        bases = {base["id"]: base for base in _load(ROOT / "hotpatches/iwd/catalog.toml")["bases"]}
        self.assertEqual(bases["iwd-0.1.22"]["content_sha256"], DIGEST_0122)
        self.assertEqual(bases["iwd-0.1.23"]["recipe_version"], "0.1.23")
        self.assertEqual(bases["iwd-0.1.23"]["label"], "Chriz Easy IWD 0.1.17")
        self.assertEqual(bases["iwd-0.1.23"]["content_sha256"], DIGEST_0123)

        curation = _load(ROOT / "manifest/curation-map.toml")
        targets = {target["id"]: target for target in curation["targets"]}
        for cid, fid in ((ALWAYS, ALWAYS_FEATURE), (COMBAT, COMBAT_FEATURE)):
            self.assertEqual(targets[fid]["kind"], "feature")
            self.assertEqual(targets[fid]["rows"], [f"CHRIZ-BG-MODPACK:{cid}"])
            self.assertEqual(targets[fid]["parent"], f"mod:{MOD}")

        catalog = (ROOT / "docs/curation/components/CHRIZ-BG-MODPACK.md").read_text(encoding="utf-8")
        for phrase in (
            "**v0.2.0-alpha.22**",
            "283e59e",
            "Modpack PR #30",
            EVIDENCE,
            f"| {ALWAYS} | {LABELS[ALWAYS]} | chriz-bg-modpack: Gameplay tweaks | | | optional |",
            f"| {COMBAT} | {LABELS[COMBAT]} | chriz-bg-modpack: Gameplay tweaks | | | optional |",
            "73 current or historical entries; the collection exposes 56.",
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
            ("docs/curation/iwdee/INVENTORY.md", "v0.2.0-alpha.22"),
            ("docs/curation/iwdee/WORKSHEET.md", "v0.2.0-alpha.22"),
            ("docs/curation/iwdee/FOLLOW_UPS.md", "v0.2.0-alpha.22"),
        ):
            self.assertIn(fragment, (ROOT / rel).read_text(encoding="utf-8"), rel)

    def test_static_acceptance_records_cite_alpha22_pin_commit(self) -> None:
        if not PIN_COMMIT:
            return
        bg_runs = {
            "chriz-bg-modpack-bg2",
            "chriz-bg-modpack-pre-continuity-bg2",
            "chriz-bg-modpack-continuity-bg2",
            "chriz-bg-modpack-late-companions-bg2",
            "chriz-bg-modpack-edwin-bg2",
            "chriz-bg-modpack-baeloth-bg2",
            "chriz-bg-modpack-scrolls-bg2",
        }
        expected_artifact = "tools/tests/test_nov27_modpack_alpha22_unpaused_inventory_recipe.py"
        for rel_path in (
            "manifest/releases/v0.1.0-alpha.1/acceptance.toml",
            "recipes/curated-full-current/releases/v0.1.0-alpha.1/acceptance.toml",
            f"{IWD_PROFILE}/releases/v0.1.0-alpha.1/acceptance.toml",
        ):
            data = tomllib.loads((ROOT / rel_path).read_text(encoding="utf-8"))
            by_id = {
                entry.get("subject_id"): entry
                for entry in data.get("evidence", [])
                if entry.get("subject_kind") == "run"
            }
            targets = {IWD_RUN} if rel_path.startswith(IWD_PROFILE) else bg_runs
            for run_id in targets:
                entry = by_id[run_id]
                self.assertEqual(entry.get("commit"), PIN_COMMIT)
                self.assertEqual(entry.get("test_artifact"), expected_artifact)
                self.assertEqual(entry.get("date"), "2026-11-27")
                self.assertEqual(entry.get("status"), "accepted")
                self.assertIn("699", entry.get("scope", ""))
                self.assertIn("700", entry.get("scope", ""))
                self.assertIn(EVIDENCE.rsplit("/", 1)[-1], entry.get("scope", ""))


if __name__ == "__main__":
    unittest.main()
