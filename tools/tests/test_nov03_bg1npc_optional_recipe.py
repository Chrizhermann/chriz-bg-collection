"""November 3: BG1 NPC Project leaves the defaults (Christopher, 2026-11-03).

"mod:bg1npc" becomes optional and unchecked in both recipe profiles. Its components keep
their decisions inside the mod, so ticking it under Customize selects the same reviewed set
as before (the pattern of the optional quest mods). Chriz BG Modpack's Kivan sea-elf quest
guard 130 stays mandatory and follows BG1NPC component 10; its omission is recorded in the
known limitations. Static recipe checks only; nothing here is a game installation.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import BG1NPC_NATIVE_ORDER, _effective_features, build_recipe


ROOT = Path(__file__).resolve().parents[2]
MOD = "mod:bg1npc"
# The reviewed set that ticking the mod brings back; 160 (portraits) stays blocked.
DEFAULT_COMPONENTS = (0, 10, 90, 111, 120, 130, 240, 200)
GUARD = "feature:chriz-bg-modpack:component-130"
LEDGERS = ["manifest/releases/v0.1.1/ledger.toml",
           "recipes/curated-full-current/releases/v0.1.1/ledger.toml"]


def _load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def _selected(collection: dict, selection: dict) -> set[tuple[str, int]]:
    effective = _effective_features(collection, selection)
    return {(ref["run_id"], ref["component"]) for feature in collection["features"]
            if effective[feature["id"]] for ref in feature.get("components", [])}


class Nov03Bg1npcOptionalTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        cls.output = Path(cls.temp.name) / "recipe"
        # Synthetic evidence marker used only in this temporary test output.
        build_recipe(ROOT, cls.output, "0" * 40)
        cls.directories = [ROOT / "manifest", ROOT / "recipes/curated-full-current", cls.output]

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def test_the_mod_is_optional_and_off_and_keeps_its_reviewed_set(self):
        for directory in self.directories:
            collection = _load(directory / "collection.toml")
            preset = _load(directory / "presets/chris-recommended.toml")["selections"]
            features = {feature["id"]: feature for feature in collection["features"]}
            with self.subTest(profile=str(directory)):
                self.assertEqual(features[MOD]["decision"], "optional")
                self.assertEqual(features[MOD]["readiness"], "ready")
                self.assertEqual(preset[MOD], "off")
                for component in DEFAULT_COMPONENTS:
                    feature_id = f"feature:bg1npc:component-{component}"
                    self.assertEqual(features[feature_id]["decision"], "default", feature_id)
                    self.assertEqual(features[feature_id]["parent"], MOD, feature_id)
                    self.assertEqual(preset[feature_id], "on", feature_id)
                portraits = features["feature:bg1npc:component-160"]
                self.assertEqual(portraits["readiness"], "blocked")
                # The run and its TP2 order stay; it is only left out of the default plan.
                (run,) = [run for run in collection["runs"] if run["run_id"] == "bg1npc-bg1"]
                self.assertEqual(run["components"],
                                 [component for component in BG1NPC_NATIVE_ORDER if component in run["components"]])

    def test_ticking_the_mod_brings_back_exactly_its_defaults_and_the_kivan_guard(self):
        for directory in self.directories:
            collection = _load(directory / "collection.toml")
            preset = _load(directory / "presets/chris-recommended.toml")["selections"]
            with self.subTest(profile=str(directory)):
                baseline = _selected(collection, preset)
                self.assertFalse(any(run == "bg1npc-bg1" for run, _ in baseline))
                ticked = _selected(collection, {**preset, MOD: "on"})
                guard_runs = {ref["run_id"] for feature in collection["features"]
                              if feature["id"] == GUARD for ref in feature["components"]}
                self.assertEqual(
                    ticked - baseline,
                    {("bg1npc-bg1", component) for component in DEFAULT_COMPONENTS}
                    | {(run_id, 130) for run_id in guard_runs},
                )
                self.assertEqual(baseline - ticked, set())

    def test_the_kivan_guard_stays_mandatory_and_its_omission_is_approved(self):
        for directory in self.directories:
            collection = _load(directory / "collection.toml")
            (guard,) = [feature for feature in collection["features"] if feature["id"] == GUARD]
            limitations = _load(directory / "releases/v0.1.0-alpha.1/known-limitations.toml")
            omissions = [item for item in limitations["omissions"] if item["feature_id"] == GUARD]
            with self.subTest(profile=str(directory)):
                self.assertEqual(guard["decision"], "mandatory")
                self.assertIn("feature:bg1npc:component-10", guard["requires"])
                self.assertEqual(len(omissions), 1)
                self.assertEqual(omissions[0]["approved_by"], "Christopher")
                self.assertEqual(omissions[0]["approved_on"], "2026-11-03")
                self.assertIn("only when BG1 NPC Project", omissions[0]["user_facing_limitation"])

    def test_the_011_ledger_tells_players(self):
        for path in LEDGERS:
            with self.subTest(ledger=path):
                ledger = _load(ROOT / path)
                self.assertEqual(ledger["version"], "0.1.1")
                self.assertEqual(ledger["supersedes"], "0.1.0")
                self.assertEqual(ledger["minimum_app_version"], "0.1.0")
                (change,) = [item for item in ledger["changes"] if item["id"] == "bg1npc-optional"]
                self.assertEqual(change["covers"], ["feature:mod:bg1npc"])
                self.assertEqual(change["save_applicability"], "next-playthrough")
                for phrase in ("no longer installed by default", "tick it under Customize",
                               "Games you already installed keep it."):
                    self.assertIn(phrase, change["summary"])
        self.assertEqual((ROOT / LEDGERS[0]).read_bytes().replace(b"\r\n", b"\n"),
                         (ROOT / LEDGERS[1]).read_bytes().replace(b"\r\n", b"\n"))


if __name__ == "__main__":
    unittest.main()
