"""November 18 intake of BG Rebalance Round Clock in both installers."""

import json
import tomllib
import unittest
from pathlib import Path

from tools.curated_full_recipe import _effective_features


ROOT = Path(__file__).resolve().parents[2]
MOD = "chriz-bg-rebalance"
PIN = "chriz-bg-rebalance-0.8.12"
PARENT = f"mod:{MOD}"
FEATURE = f"feature:{MOD}:component-361"
FILENAME = "chriz-bg-rebalance-v0.8.12-windows.zip"
URL = f"https://github.com/Chrizhermann/{MOD}/releases/download/v0.8.12/{FILENAME}"
SOURCE_COMMIT = "2cb1b14dc7c465a66be8da272112b69ac28574bd"
PIN_COMMIT = "05b1bd527dd73e3228b714cf4e8f02c661f19622"
SHA256 = "f64314ab1462cef661dc4191e37e9d8911e95bd8df13ad44c3a09744b23847a8"
TP2_SHA256 = "5eec12295567dc3855cfe4a7a76aadfd155b64f6a752f8a2cbb763f537e057cc"
MENU_NAME = ("Complete personal rounds and stealth cooldowns across time jumps "
             "(EEex, BG:EE / SoD / BG2:EE / EET / IWD:EE 2.7.3.0)")
IWD = "recipes/iwd-current"
PROFILES = ["manifest", "recipes/curated-full-current", IWD]


def load(path):
    return tomllib.loads((ROOT / path).read_text(encoding="utf-8"))


class RoundClockIntakeTests(unittest.TestCase):
    def test_one_fresh_archive_and_native_order_record_covers_all_profiles(self):
        evidence = load(f"manifest/evidence/release-intake-20261118-{MOD}-chriz-easy-bg-chriz-easy-iwd.toml")
        self.assertEqual(evidence["verified_on"], "2026-11-18")
        (record,) = evidence["artifacts"]
        self.assertEqual(record["recipe_profiles"], PROFILES)
        self.assertEqual((record["mod_id"], record["artifact_id"]), (MOD, PIN))
        self.assertEqual((record["source_reference"], record["source_commit"]), ("v0.8.12", SOURCE_COMMIT))
        self.assertEqual((record["immutable_source_url"], record["actual_length"], record["actual_sha256"]),
                         (URL, 1_825_506, SHA256))
        self.assertEqual(record["tp2_sha256"], TP2_SHA256)
        earlier = load(f"manifest/evidence/release-intake-20261108-{MOD}-chriz-easy-bg.toml")
        (previous,) = earlier["artifacts"]
        self.assertEqual(record["native_components"], [*previous["native_components"], 361])
        for flag in ("fresh_download", "fresh_extraction", "published_release_verified", "native_component_order_verified"):
            self.assertIs(record[flag], True, flag)

    def test_profiles_use_the_same_reviewed_archive_and_numbered_menu_entry(self):
        previous = load(f"manifest/artifacts/{MOD}-0.8.11.toml")
        for profile in PROFILES:
            with self.subTest(profile=profile):
                artifact = load(f"{profile}/artifacts/{PIN}.toml")
                mod = load(f"{profile}/mods/{MOD}.toml")
                self.assertEqual(artifact, load(f"manifest/artifacts/{PIN}.toml"))
                self.assertEqual((artifact["id"], mod["artifact_id"], artifact["version"]), (PIN, PIN, "0.8.12"))
                self.assertEqual(artifact["acquisition"], "fetch-only")
                self.assertEqual(artifact["archive"], previous["archive"])
                self.assertEqual(artifact["archive"]["tp2_paths"], [mod["tp2"]])
                self.assertEqual(artifact["provenance"]["reviewed_on"], "2026-11-18")
                source = artifact["source"]
                self.assertEqual((source["kind"], source["reference"], source["expected_filename"]),
                                 ("github-release", "v0.8.12", FILENAME))
                self.assertEqual((source["url"], source["expected_length"], source["sha256"]), (URL, 1_825_506, SHA256))
                entries = {c["id"]: c["name"] for c in mod["components"]}
                self.assertEqual(entries[361], MENU_NAME)
                if profile == IWD:
                    self.assertEqual(set(entries), {361})
                else:
                    self.assertTrue((ROOT / profile / f"artifacts/{MOD}-0.8.11.toml").is_file())

    def test_recommended_selects_round_clock_with_eeex_in_all_profiles(self):
        for profile in PROFILES:
            with self.subTest(profile=profile):
                collection = load(f"{profile}/collection.toml")
                features = {row["id"]: row for row in collection["features"]}
                parent, feature = features[PARENT], features[FEATURE]
                self.assertEqual((parent["decision"], parent["readiness"]), ("default", "ready"))
                self.assertEqual((feature["category"], feature["parent"], feature["readiness"]),
                                 ("spells", PARENT, "ready"))
                (run,) = [row for row in collection["runs"] if row["mod_id"] == MOD]
                self.assertEqual(feature["components"], [{"run_id": run["run_id"], "component": 361}])
                preset = load(f"{profile}/presets/chris-recommended.toml").get("selections", {})
                if profile == IWD:
                    self.assertEqual(run["run_id"], f"{MOD}-main")
                    self.assertEqual(run["components"], [361])
                    order = [r["run_id"] for r in collection["runs"]]
                    index = order.index(run["run_id"])
                    self.assertEqual(order[index - 1:index + 2],
                                     ["chriz-iwd-party-dialog-main", run["run_id"], "chriz-bg-modpack-main"])
                    self.assertEqual(feature["decision"], "mandatory")
                    self.assertEqual(feature["requires"], ["feature:eeex:component-1"])
                    self.assertEqual(feature["source_label"], "BG Rebalance")
                else:
                    self.assertEqual(run["run_id"], f"{MOD}-bg2")
                    self.assertEqual(run["components"][-2:], [360, 361])
                    self.assertEqual(feature["decision"], "default")
                    self.assertEqual(feature["requires"], ["feature:eeex:mandatory-components"])
                    self.assertEqual(preset[FEATURE], "on")
                # IWD's Recommended preset inherits the recipe defaults.
                choices = {key: "on" for key, row in features.items() if row["decision"] == "default"}
                choices.update(preset)
                self.assertTrue(_effective_features(collection, choices)[FEATURE])
                for disabled in (PARENT, "mod:eeex"):
                    self.assertFalse(_effective_features(collection, {**choices, disabled: "off"})[FEATURE], disabled)
                if profile != IWD:
                    without_time_stop = {**choices, f"feature:{MOD}:component-360": "off"}
                    self.assertTrue(_effective_features(collection, without_time_stop)[FEATURE])

    def test_source_register_and_bg_pin_list_name_the_current_release(self):
        source = next(row for row in (ROOT / "manifest/mod-sources.tsv").read_text(encoding="utf-8").splitlines()
                      if row.startswith("CHRIZ-BG-REBALANCE\t")).split("\t")
        self.assertEqual((source[3], source[4], source[7]), ("v0.8.12", "2026-11-18", URL))
        for fact in (SOURCE_COMMIT, "1825506", SHA256):
            self.assertIn(fact, source[-1])
        pins = (ROOT / "docs/pin-list-2.7.md").read_text(encoding="utf-8")
        self.assertEqual(pins.count(f"| CHRIZ-BG-REBALANCE | 0.8.12 | `{MOD}` | `{SHA256[:12]}…` |"), 2)

    def test_iwd_0_1_15_ledger_and_hotpatch_base(self):
        release = json.loads((ROOT / IWD / "release.json").read_text(encoding="utf-8"))
        self.assertTrue((ROOT / IWD / f"releases/v{release['version']}/ledger.toml").is_file(),
                        release["version"])
        ledger = load(f"{IWD}/releases/v0.1.15/ledger.toml")
        self.assertEqual((ledger["recipe_id"], ledger["version"], ledger["supersedes"]),
                         ("chriz-easy-iwd", "0.1.15", "0.1.14"))
        (change,) = ledger["changes"]
        self.assertEqual((change["id"], change["save_applicability"], change["urgency"]),
                         (MOD, "new-game-only", "informational"))
        self.assertEqual(set(change["covers"]), {f"artifact:{PIN}", f"feature:{PARENT}",
                                                f"feature:{FEATURE}", f"run:{MOD}-main"})
        bases = load("hotpatches/iwd/catalog.toml")["bases"]
        (base,) = [b for b in bases if b["recipe_version"] == "0.1.15"]
        self.assertEqual(base["id"], "iwd-0.1.15")
        self.assertEqual(base["content_sha256"], "518171a581f42c3c82e732ae2d67759ae49668258d8d803f24fc9368245e83fe")

    def test_all_three_static_run_records_name_the_round_clock_pin_commit(self):
        for profile, run, test in (
            ("manifest", f"{MOD}-bg2", "engine/tests/production_recipe_chriz.rs"),
            ("recipes/curated-full-current", f"{MOD}-bg2", "engine/tests/production_recipe_chriz.rs"),
            (IWD, f"{MOD}-main", "engine/tests/production_recipe_iwd.rs"),
        ):
            with self.subTest(profile=profile):
                evidence = load(f"{profile}/releases/v0.1.0-alpha.1/acceptance.toml")["evidence"]
                (record,) = [e for e in evidence if e["subject_kind"] == "run" and e["subject_id"] == run]
                self.assertEqual((record["kind"], record["status"]), ("static-test", "accepted"))
                self.assertEqual((record["commit"], record["date"], record["test_artifact"]),
                                 (PIN_COMMIT, "2026-11-18", test))

    def test_round_clock_is_in_the_next_bg_changelog(self):
        changelog = (ROOT / "docs/plans/2026-10-14-next-release-changelogs.md").read_text(encoding="utf-8")
        current, historical = changelog.split("\n## Next Chriz Easy BG release (draft)\n", 1)[1].split(
            "\n## Collection 0.1.2 (draft)\n", 1)
        behavior, rest = current.split("### Changed behavior\n", 1)[1].split("### Mod versions\n", 1)
        versions, notes = rest.split("### Notes for the release integrator\n", 1)
        self.assertIn("**Round Clock** (BG Rebalance component 361, Default, requires EEex)", behavior)
        self.assertIn("- BG Rebalance v0.8.12 (was v0.8.11)", versions)
        self.assertIn(f"manifest/evidence/release-intake-20261118-{MOD}-chriz-easy-bg-chriz-easy-iwd.toml", notes)
        self.assertIn(f"artifact:{PIN}", notes)
        self.assertIn(f"feature:{FEATURE}", notes)
        old_draft = historical.split("\n## Installer changelog\n", 1)[0]
        self.assertNotIn("BG Rebalance v0.8.12", old_draft)
        self.assertNotIn("Round Clock", old_draft)


if __name__ == "__main__":
    unittest.main()
