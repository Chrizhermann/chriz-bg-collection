"""November 27 intake: Chriz BG Modpack v0.2.0-alpha.21 (696, 697, 698) in BG, pin bump in IWD.

Modpack PR #29 adds optional Challenge Mode component 696 (cbm_no_combat_quickloot, disable
Quick Loot during combat, requiring EEex) after 695 in TP2 declaration order on top of PR #28 /
v0.2.0-alpha.20 (which added defaults 697 Watcher's Keep instant teleporters and 698 clear lingering
room effects on leaving an area or hazard room) and publishes v0.2.0-alpha.21. Chriz Easy BG
exposes 696 as optional (off in Chris's Recommended, part of the Challenge Mode group) and enables
697 and 698 by default after 695, while Chriz Easy IWD updates the pinned Modpack package to
v0.2.0-alpha.21 (collection 0.1.21) keeping component 470 as its sole Modpack component and
omitting 690, 695, 696, 697, and 698.
"""

from __future__ import annotations

import json
from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import _effective_features, build_recipe


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261127-chriz-bg-modpack-v0.2.0-alpha.21-chriz-easy-bg-chriz-easy-iwd.toml"
ALPHA20_EVIDENCE = "manifest/evidence/release-intake-20261127-chriz-bg-modpack-v0.2.0-alpha.20-chriz-easy-bg-chriz-easy-iwd.toml"
PREVIOUS_EVIDENCE = "manifest/evidence/release-intake-20261127-chriz-bg-modpack-chriz-easy-bg-chriz-easy-iwd.toml"
MOD = "chriz-bg-modpack"
PIN = "chriz-bg-modpack-0.2.0-alpha.21"
ALPHA20 = "chriz-bg-modpack-0.2.0-alpha.20"
PREVIOUS = "chriz-bg-modpack-0.2.0-alpha.19"
SUCCESSOR = "chriz-bg-modpack-0.2.0-alpha.22"
TAG = "v0.2.0-alpha.21"
SOURCE_COMMIT = "4e143894a8db7aef0ff4827dae096ba56aab2299"
PIN_COMMIT = "5b178fb6ea390d52799bac5e6d00af7eeb6a8d4a"
FILENAME = "chriz-bg-modpack-v0.2.0-alpha.21.zip"
URL = f"https://github.com/Chrizhermann/chriz-bg-modpack/releases/download/{TAG}/{FILENAME}"
LENGTH = 9_597_844
SHA256 = "717b17071e26e426d7d74f3a0c8b1d6ac1aa49d55932512b92bb21bb418c190f"
TP2_SHA256 = "97347d1e47b7b91462f63ee2314f88e0ba01479d47012217dfcc355aee8a79e6"

QUICKLOOT = 696
TELEPORTERS = 697
AREA_CLEANUP = 698
UNOFFERED = 690
GROUP = "challenge-mode"
BG_RUN = "chriz-bg-modpack-bg2"
IWD_RUN = "chriz-bg-modpack-main"
IWD_PROFILE = "recipes/iwd-current"
QUICKLOOT_FEATURE = f"feature:{MOD}:component-{QUICKLOOT}"
TELEPORTERS_FEATURE = f"feature:{MOD}:component-{TELEPORTERS}"
AREA_CLEANUP_FEATURE = f"feature:{MOD}:component-{AREA_CLEANUP}"
LABELS = {
    QUICKLOOT: "Challenge Mode: disable Quick Loot during combat (BG:EE, SoD, BG2:EE, EET, IWD:EE; EEex)",
    TELEPORTERS: "Watcher's Keep: instant teleporters",
    AREA_CLEANUP: "Clear lingering room effects when leaving an area or hazard room (BG:EE, SoD, BG2:EE, EET; Windows EEex)",
}
DIGEST_0120 = "169e05d1a2607ebc69e7016732b42c8d18024dd16f814e5b67796e9249488526"
DIGEST_0121 = "adba2f20b157ebb8d97c0103518c7b78c5efe23e053983a32607851b9280cd9a"


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


class Nov27ModpackAlpha21CombatQuickLootTests(unittest.TestCase):
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

    def test_evidence_freezes_v0_2_0_alpha_21_and_native_order_for_all_three_profiles(self) -> None:
        self.assertEqual(self.evidence["verified_on"], "2026-11-27")
        self.assertEqual(
            self.evidence["generator"],
            "tools/prepare_own_mod_pin.py + chriz-bg-author + check_native_component_order.py",
        )
        for phrase in (
            "Modpack PR #29",
            "v0.2.0-alpha.20 at 099aaf4",
            "replaces chriz-bg-modpack-0.2.0-alpha.19",
            "Component 696",
            "components 697",
            "and 698",
            "component 690 stays unoffered and off in all profiles",
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
        idx_695 = native.index(695)
        native[idx_695 + 1 : idx_695 + 1] = [QUICKLOOT, TELEPORTERS, AREA_CLEANUP]
        self.assertEqual(self.record["native_components"], native)

        (alpha20_record,) = _load(ROOT / ALPHA20_EVIDENCE)["artifacts"]
        self.assertEqual(alpha20_record["artifact_id"], ALPHA20)
        self.assertEqual(alpha20_record["source_commit"], "099aaf4761ece2134a797010f5f3c679823b46cd")

    def test_bg_wires_696_into_challenge_mode_697_698_as_defaults_and_iwd_omits_them(self) -> None:
        iwd_root = ROOT / IWD_PROFILE
        for directory in [*self.bg_directories, iwd_root]:
            is_iwd = directory == iwd_root
            with self.subTest(profile=str(directory)):
                mod = _load(directory / f"mods/{MOD}.toml")
                if is_iwd:
                    self.assertFalse((directory / f"artifacts/{PIN}.toml").exists())
                    self.assertTrue((directory / f"artifacts/{SUCCESSOR}.toml").exists())
                    artifact = _load(ROOT / f"manifest/artifacts/{PIN}.toml")
                    self.assertFalse((directory / f"artifacts/{ALPHA20}.toml").exists())
                    self.assertFalse((directory / f"artifacts/{PREVIOUS}.toml").exists())
                else:
                    artifact = _load(directory / f"artifacts/{PIN}.toml")
                    previous = _load(directory / f"artifacts/{PREVIOUS}.toml")
                    self.assertTrue((directory / f"artifacts/{ALPHA20}.toml").exists())
                    self.assertEqual(artifact["archive"], previous["archive"])
                    self.assertEqual(
                        artifact["source"]["redirect_hosts"],
                        previous["source"]["redirect_hosts"],
                    )
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
                # Alpha.22 adds optional mutually exclusive 699 and 700; earlier declarations keep their order.
                for r in (r for r in collection["runs"] if r["mod_id"] == MOD):
                    self.assertTrue(set(r["components"]) - {699, 700} <= native, r["run_id"])
                    self.assertNotIn(UNOFFERED, r["components"], r["run_id"])

                if is_iwd:
                    for cid in (695, QUICKLOOT, TELEPORTERS, AREA_CLEANUP):
                        self.assertNotIn(f"feature:{MOD}:component-{cid}", features)
                        self.assertNotIn(cid, names)
                    (run,) = [run for run in collection["runs"] if run["mod_id"] == MOD]
                    self.assertEqual(run["run_id"], IWD_RUN)
                    self.assertEqual(run["phase"], "main")
                    self.assertEqual(run["components"], [470, 699, 700])
                else:
                    for cid, label in LABELS.items():
                        self.assertEqual(names[cid], label)
                    (run,) = [run for run in collection["runs"] if run["run_id"] == BG_RUN]
                    self.assertEqual(run["phase"], "post-eet-end")
                    idx_695 = run["components"].index(695)
                    self.assertEqual(
                        run["components"][idx_695 + 1 : idx_695 + 4],
                        [QUICKLOOT, TELEPORTERS, AREA_CLEANUP],
                    )
                    self.assertEqual(
                        list(names)[list(names).index(695) + 1 : list(names).index(695) + 4],
                        [QUICKLOOT, TELEPORTERS, AREA_CLEANUP],
                    )

                    (group,) = [g for g in collection["feature_groups"] if g["id"] == GROUP]
                    self.assertIn(QUICKLOOT_FEATURE, group["members"])
                    self.assertEqual(
                        group["members"][6:8],
                        ["feature:chriz-bg-modpack:component-680", QUICKLOOT_FEATURE],
                    )
                    self.assertNotIn(GROUP, preset)

                    ql = features[QUICKLOOT_FEATURE]
                    self.assertEqual((ql["category"], ql["decision"], ql["readiness"]), ("rules", "optional", "ready"))
                    self.assertEqual(ql["parent"], f"mod:{MOD}")
                    self.assertEqual(ql["requires"], ["feature:eeex:mandatory-components"])
                    self.assertEqual(ql["components"], [{"run_id": BG_RUN, "component": QUICKLOOT}])
                    self.assertNotIn(QUICKLOOT_FEATURE, preset)

                    tp = features[TELEPORTERS_FEATURE]
                    self.assertEqual((tp["category"], tp["decision"], tp["readiness"]), ("convenience", "default", "ready"))
                    self.assertEqual(tp["parent"], f"mod:{MOD}")
                    self.assertEqual(tp.get("requires", []), [])
                    self.assertEqual(tp["components"], [{"run_id": BG_RUN, "component": TELEPORTERS}])
                    self.assertEqual(preset[TELEPORTERS_FEATURE], "on")

                    ac = features[AREA_CLEANUP_FEATURE]
                    self.assertEqual((ac["category"], ac["decision"], ac["readiness"]), ("fixes", "default", "ready"))
                    self.assertEqual(ac["parent"], f"mod:{MOD}")
                    self.assertEqual(ac["requires"], ["feature:eeex:mandatory-components"])
                    self.assertEqual(ac["components"], [{"run_id": BG_RUN, "component": AREA_CLEANUP}])
                    self.assertEqual(preset[AREA_CLEANUP_FEATURE], "on")

                    baseline = _selected(collection, preset)
                    self.assertNotIn((BG_RUN, QUICKLOOT), baseline)
                    self.assertIn((BG_RUN, TELEPORTERS), baseline)
                    self.assertIn((BG_RUN, AREA_CLEANUP), baseline)

                    challenge = _selected(
                        collection,
                        {**preset, **{member: "on" for member in group["members"]}},
                    )
                    self.assertIn((BG_RUN, QUICKLOOT), challenge)
                    self.assertNotIn(
                        (BG_RUN, QUICKLOOT),
                        _selected(collection, {**preset, QUICKLOOT_FEATURE: "on", "mod:eeex": "off"}),
                    )
                    self.assertNotIn(
                        (BG_RUN, QUICKLOOT),
                        _selected(collection, {**preset, QUICKLOOT_FEATURE: "on", f"mod:{MOD}": "off"}),
                    )
                    self.assertIn(
                        (BG_RUN, TELEPORTERS),
                        _selected(collection, {**preset, "mod:eeex": "off"}),
                    )
                    self.assertNotIn(
                        (BG_RUN, AREA_CLEANUP),
                        _selected(collection, {**preset, "mod:eeex": "off"}),
                    )
                    for disabled in (f"mod:{MOD}", "mod:eeex"):
                        self.assertNotIn(
                            (BG_RUN, AREA_CLEANUP),
                            _selected(collection, {**preset, disabled: "off"}),
                            disabled,
                        )

    def test_iwd_0_1_21_release_ledger_hotpatch_and_docs(self) -> None:
        release = json.loads((ROOT / IWD_PROFILE / "release.json").read_text(encoding="utf-8"))
        self.assertIn(release["version"], ("0.1.21", "0.1.22", "0.1.23"))
        ledger = _load(ROOT / f"{IWD_PROFILE}/releases/v0.1.21/ledger.toml")
        self.assertEqual(
            (ledger["recipe_id"], ledger["version"], ledger["supersedes"]),
            ("chriz-easy-iwd", "0.1.21", "0.1.20"),
        )
        (change,) = ledger["changes"]
        self.assertEqual(
            (change["id"], change["save_applicability"], change["urgency"]),
            (MOD, "new-game-only", "informational"),
        )
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
        for cid, fid in (
            (QUICKLOOT, QUICKLOOT_FEATURE),
            (TELEPORTERS, TELEPORTERS_FEATURE),
            (AREA_CLEANUP, AREA_CLEANUP_FEATURE),
        ):
            self.assertEqual(targets[fid]["kind"], "feature")
            self.assertEqual(targets[fid]["rows"], [f"CHRIZ-BG-MODPACK:{cid}"])
            self.assertEqual(targets[fid]["parent"], f"mod:{MOD}")

        catalog = (ROOT / "docs/curation/components/CHRIZ-BG-MODPACK.md").read_text(encoding="utf-8")
        for phrase in (
            "**v0.2.0-alpha.21**",
            "4e14389",
            "Modpack PR #29",
            EVIDENCE,
            f"| {QUICKLOOT} | {LABELS[QUICKLOOT]} | chriz-bg-modpack: Gameplay tweaks | | | optional |",
            f"| {TELEPORTERS} | {LABELS[TELEPORTERS]} | chriz-bg-modpack: Gameplay tweaks | | | default |",
            f"| {AREA_CLEANUP} | {LABELS[AREA_CLEANUP]} | chriz-bg-modpack: Gameplay tweaks | | | default |",
            "73 current or historical entries; the collection exposes 56.",
        ):
            self.assertIn(phrase, catalog)

        source = next(
            line for line in (ROOT / "manifest/mod-sources.tsv").read_text(encoding="utf-8").splitlines()
            if line.startswith("CHRIZ-BG-MODPACK\t")
        )
        self.assertIn("696", source)
        pin_list = (ROOT / "docs/pin-list-2.7.md").read_text(encoding="utf-8")
        self.assertNotIn(f"| chriz-bg-modpack | {TAG.removeprefix('v')} |", pin_list)

        for rel, fragment in (
            ("docs/curation/iwdee/INVENTORY.md", "v0.2.0-alpha.21"),
            ("docs/curation/iwdee/WORKSHEET.md", "v0.2.0-alpha.21"),
            ("docs/curation/iwdee/FOLLOW_UPS.md", "695, 696, 697, and 698 omitted from IWD"),
        ):
            self.assertIn(fragment, (ROOT / rel).read_text(encoding="utf-8"), rel)

        changelog = (ROOT / "docs/plans/2026-10-14-next-release-changelogs.md").read_text(encoding="utf-8")
        current = changelog.split("\n## Next Chriz Easy BG release (draft)\n", 1)[1].split("\n## Collection 0.1.2 (draft)\n", 1)[0]
        self.assertIn("**No Quick Loot during combat** (Chriz BG Modpack component 696", current)
        self.assertIn("**Watcher's Keep instant teleporters and instant room-hazard cleanup**", current)
        self.assertIn("v0.2.0-alpha.21", current)
        self.assertIn(EVIDENCE, current)
        self.assertIn(f"feature:{QUICKLOOT_FEATURE}", current)
        self.assertIn(f"feature:{TELEPORTERS_FEATURE}", current)
        self.assertIn(f"feature:{AREA_CLEANUP_FEATURE}", current)

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
            (f"{IWD_PROFILE}/releases/v0.1.0-alpha.1/acceptance.toml", {IWD_RUN}),
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
