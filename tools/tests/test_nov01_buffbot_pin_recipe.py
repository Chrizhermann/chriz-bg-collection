"""November 1 intake: BuffBot v1.12.6 replaces v1.12.1 in both recipe profiles.

v1.12.6 contains v1.12.2 to v1.12.5: the Summons view can switch back to the party again and
lists only creatures the party conjured, the colour schemes carry the right game's names,
pressed buttons look pressed, and the fast-forward guard reads the game's own menu list.
Against v1.12.1 the TP2 changes only its version and two comments, so no decision changes
(Christopher, 2026-11-01: v1.12.6 goes into alpha.22 and into Chriz Easy IWD).

Static pin, evidence, component, selection, order and release-text checks for both recipe
profiles only. Nothing here is a game installation or gameplay acceptance. The November 4
intake (test_nov04_buffbot_pin_recipe) supersedes this pin with v1.16.0, which keeps both
components and their order; the v1.12.6 record and artifacts stay as verified here, and 0.1.0
and 0.1.1 ship v1.12.6.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import _effective_features, build_recipe
from tools.curation_audit import Decision, load_catalogs


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261101-buffbot-chriz-easy-bg.toml"
PREVIOUS_EVIDENCE = "manifest/evidence/release-intake-20261028-buffbot.toml"
PROFILES = ["manifest", "recipes/curated-full-current"]
MOD = "buffbot"
PIN = "buffbot-1.12.6"
PREVIOUS = "buffbot-1.12.1"
# The November 4 intake (test_nov04_buffbot_pin_recipe) moves both profiles to v1.16.0.
SUCCESSOR = "buffbot-1.16.0"
# Older pins that stay as history next to the superseded one.
HISTORY = ["buffbot-1.10.1", "buffbot-1.11.0"]
TAG = "v1.12.6"
FILENAME = "buffbot-v1.12.6.zip"
SHA256 = "f99781368d64a419a567f48767728a600b8e81e203946217228ee9d61bc8b534"
LENGTH = 5_713_218
TP2_SHA256 = "45d0613c141f2f62b8562b63dc738e0058990a9f43c4853382eb72f4a2125daa"
TAG_COMMIT = "171e616d2c9f1126aad0883d94acaeadf49f8efe"
RUN = "buffbot-bg2"
SCROLL_RUN = "chriz-bg-modpack-scrolls-bg2"
RUNTIME = "feature:buffbot:mandatory-components"
# The TP2 declares the LuaJIT helper 1 before the main component 0; both names are unchanged.
COMPONENTS = [
    (1, "BuffBot: EEex LuaJIT Support (auto-detected)"),
    (0, "BuffBot: In-Game Buff Automation"),
]
# The release prepared as alpha.22 became 0.1.0 (Christopher, 2026-11-01).
LEDGERS = ["manifest/releases/v0.1.0/ledger.toml",
           "recipes/curated-full-current/releases/v0.1.0/ledger.toml"]


def _load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def _load_profile(directory: Path) -> tuple[dict, dict]:
    collection = _load(directory / "collection.toml")
    preset = _load(directory / "presets/chris-recommended.toml")["selections"]
    return collection, preset


class Nov01BuffBotV1126PinTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        cls.output = Path(cls.temp.name) / "recipe"
        # Synthetic evidence marker used only in this temporary test output.
        build_recipe(ROOT, cls.output, "0" * 40)
        # Each recipe directory with the evidence profile it must match.
        cls.directories = [
            ("manifest", ROOT / "manifest"),
            ("recipes/curated-full-current", ROOT / "recipes/curated-full-current"),
            ("recipes/curated-full-current", cls.output),
        ]
        cls.profiles = {
            "curated": (cls.output, *_load_profile(cls.output)),
            "fallback": (ROOT / "manifest", *_load_profile(ROOT / "manifest")),
        }
        cls.evidence = _load(ROOT / EVIDENCE)
        (cls.record,) = cls.evidence["artifacts"]
        (cls.previous,) = _load(ROOT / PREVIOUS_EVIDENCE)["artifacts"]

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

    def test_evidence_records_the_fresh_published_download(self):
        self.assertEqual(self.evidence["verified_on"], "2026-11-01")
        self.assertEqual(self.evidence["evidence_kind"], "published-release-download-and-native-order-verification")
        self.assertEqual(self.evidence["generator"], "tools/prepare_own_mod_pin.py")
        for phrase in ("replaces buffbot-1.12.1", "fresh cache", "a normal GitHub release",
                       "superseded artifact TOMLs in manifest, recipes/curated-full-current",
                       "not a WeiDU install or gameplay acceptance"):
            self.assertIn(phrase, self.evidence["scope"])
        record = self.record
        self.assertEqual(record["mod_id"], MOD)
        self.assertEqual(record["artifact_id"], PIN)
        self.assertEqual(record["recipe_profiles"], PROFILES)
        self.assertNotIn("native_order_recipe_profiles", record)
        self.assertEqual(record["source_reference"], TAG)
        self.assertEqual(record["source_commit"], TAG_COMMIT)
        self.assertEqual(record["immutable_source_url"],
                         f"https://github.com/Chrizhermann/bg-eeex-buffbot/releases/download/{TAG}/{FILENAME}")
        self.assertEqual(record["actual_length"], LENGTH)
        self.assertEqual(record["actual_sha256"], SHA256)
        self.assertEqual(record["tp2_sha256"], TP2_SHA256)
        for flag in ("fresh_download", "fresh_extraction", "published_release_verified",
                     "native_component_order_verified"):
            self.assertIs(record[flag], True, flag)
        # The TP2 changed (version and comments), but it declares the same two components in
        # the same order.
        self.assertEqual(self.previous["artifact_id"], PREVIOUS)
        self.assertNotEqual(record["tp2_sha256"], self.previous["tp2_sha256"])
        self.assertEqual(record["native_components"], self.previous["native_components"])
        self.assertEqual(record["native_components"], [component for component, _ in COMPONENTS])

    def test_both_profiles_keep_v1126_and_the_earlier_pins_as_verified(self):
        for profile_name, directory in self.directories:
            with self.subTest(profile=str(directory)):
                self.assertIn(profile_name, self.record["recipe_profiles"])
                mod = _load(directory / f"mods/{MOD}.toml")
                # A superseded pin stays available as history, verified as recorded here.
                self.assertEqual(mod["artifact_id"], SUCCESSOR)
                artifact = _load(directory / f"artifacts/{PIN}.toml")
                previous = _load(directory / f"artifacts/{PREVIOUS}.toml")
                self.assertEqual(artifact["id"], PIN)
                self.assertEqual(artifact["name"], previous["name"])
                self.assertEqual(artifact["version"], TAG.removeprefix("v"))
                self.assertEqual(artifact["acquisition"], "fetch-only")
                self.assertEqual(artifact["source"]["kind"], "github-release")
                self.assertEqual(artifact["source"]["expected_filename"], FILENAME)
                self.assertEqual(artifact["provenance"]["url"],
                                 f"https://github.com/Chrizhermann/bg-eeex-buffbot/releases/tag/{TAG}")
                self.assertEqual(artifact["provenance"]["reviewed_on"], "2026-11-01")
                self.assertEqual(artifact["provenance"]["license"], previous["provenance"]["license"])
                for key, measured in (("sha256", "actual_sha256"), ("expected_length", "actual_length"),
                                      ("url", "immutable_source_url"), ("reference", "source_reference")):
                    self.assertEqual(artifact["source"][key], self.record[measured], key)
                    # The v1.12.1 record still describes the kept v1.12.1 artifact.
                    self.assertEqual(previous["source"][key], self.previous[measured], key)
                # Same layout, limits and redirect host as v1.12.1, which stays for
                # installations made from the earlier recipe, like the older pins.
                self.assertEqual(artifact["archive"], previous["archive"])
                self.assertEqual(artifact["source"]["redirect_hosts"], previous["source"]["redirect_hosts"])
                self.assertNotEqual(artifact["source"]["sha256"], previous["source"]["sha256"])
                self.assertEqual(artifact["archive"]["publish_roots"], [MOD])
                self.assertEqual(artifact["archive"]["tp2_paths"], [mod["tp2"]])
                for artifact_id in HISTORY:
                    self.assertTrue((directory / f"artifacts/{artifact_id}.toml").is_file(), artifact_id)
        # The checked-in curated artifacts are the generator's copies of the authored pins.
        for artifact_id in (PIN, PREVIOUS, *HISTORY):
            name = f"artifacts/{artifact_id}.toml"
            self.assertEqual((ROOT / "recipes/curated-full-current" / name).read_bytes(),
                             (ROOT / "manifest" / name).read_bytes())

    def test_components_and_the_run_are_unchanged_in_native_order(self):
        rows = {row.component_id: row.decision for row in load_catalogs(ROOT / "docs/curation/components")
                if row.catalog == "BUFFBOT"}
        self.assertEqual(rows, {1: Decision.MANDATORY, 0: Decision.MANDATORY})
        for _, directory in self.directories:
            with self.subTest(profile=str(directory)):
                mod = _load(directory / f"mods/{MOD}.toml")
                self.assertEqual([(component["id"], component["name"]) for component in mod["components"]],
                                 COMPONENTS)
                self.assertEqual((mod["tp2"], mod["language"], mod["weidu_artifact_id"], mod["invocation_mode"]),
                                 ("buffbot/setup-buffbot.tp2", 0, "weidu-249-amd64", "setup-name"))
                collection = _load(directory / "collection.toml")
                runs = [run for run in collection["runs"] if run["mod_id"] == MOD]
                self.assertEqual([run["run_id"] for run in runs], [RUN])
                self.assertEqual(runs[0]["components"], self.record["native_components"])
                self.assertEqual(runs[0]["phase"], "post-eet-end")
                self.assertEqual(runs[0].get("args", []), [])

    def test_buffbot_stays_after_the_voicepacks_and_directly_before_the_scroll_policy(self):
        for _, _, collection, preset in self._each_profile():
            order = [run["run_id"] for run in collection["runs"]]
            buffbot = order.index(RUN)
            voicepacks = [index for index, run_id in enumerate(order) if run_id.startswith("voicepack-")]
            self.assertTrue(voicepacks)
            self.assertLess(max(voicepacks), buffbot)
            # Modpack's scroll policy 430 reads the final scrolls and spells, so its run alone follows.
            self.assertEqual(order[buffbot + 1:], [SCROLL_RUN])
            features = {feature["id"]: feature for feature in collection["features"]}
            self.assertEqual(features["mod:buffbot"]["decision"], "default")
            self.assertEqual(features["mod:buffbot"]["requires"], ["mod:eeex"])
            runtime = features[RUNTIME]
            self.assertEqual((runtime["decision"], runtime["parent"]), ("mandatory", "mod:buffbot"))
            self.assertEqual(runtime["components"], [{"run_id": RUN, "component": component}
                                                     for component, _ in COMPONENTS])
            self.assertEqual(preset.get("mod:buffbot"), "on")
            baseline = self._selected(collection, preset)
            self.assertEqual({component for run_id, component in baseline if run_id == RUN}, {1, 0})
            # Chris's Recommended installs BuffBot together with the scroll policy.
            self.assertIn((SCROLL_RUN, 430), baseline)
            self.assertEqual(baseline - self._selected(collection, preset, **{"mod:buffbot": "off"}),
                             {(RUN, 1), (RUN, 0)})
            # BuffBot needs EEex.
            self.assertFalse(any(run_id == RUN for run_id, _ in
                                 self._selected(collection, preset, **{"mod:eeex": "off"})))

    def test_catalog_and_changelog_keep_the_pin_as_history(self):
        catalog = (ROOT / "docs/curation/components/BUFFBOT.md").read_text(encoding="utf-8")
        for phrase in (f"{TAG} (released 2026-11-01, tag on commit `{TAG_COMMIT[:7]}`", EVIDENCE,
                       "Christopher, 2026-11-01", "PR #51", "`430`",
                       *(f"bg-eeex-buffbot#{issue}" for issue in (101, 102, 103, 104, 105))):
            self.assertIn(phrase, catalog)
        changelog = (ROOT / "docs/plans/2026-10-14-next-release-changelogs.md").read_text(encoding="utf-8")
        for phrase in ("BuffBot v1.12.6 (was v1.10.0)", "BuffBot to v1.12.6", "(BuffBot v1.12.2)",
                       "(BuffBot v1.12.3)", "(BuffBot v1.12.4)", "(BuffBot v1.12.5)", "(BuffBot v1.12.6)",
                       "**BuffBot v1.12.6**"):
            self.assertIn(phrase, changelog)
        self.assertNotIn("BuffBot v1.12.1 (was", changelog)
        self.assertNotIn("BuffBot to v1.12.1", changelog)

    def test_release_ledgers_name_the_release_and_cover_its_artifact(self):
        # The superseded pins stay covered because their artifact TOMLs stay in the recipe.
        for path in LEDGERS:
            with self.subTest(ledger=path):
                changes = {change["id"]: change for change in _load(ROOT / path)["changes"]}
                self.assertEqual(changes["buffbot-1-12-6"]["title"], "BuffBot 1.12.6")
                self.assertIn("BuffBot v1.12.6 (was v1.10.0)", changes["buffbot-1-12-6"]["summary"])
                self.assertEqual(changes["buffbot-1-12-6"]["covers"],
                                 [f"artifact:{artifact}" for artifact in (*HISTORY, PREVIOUS, PIN)])
                self.assertIn("v1.11.0 to v1.12.6", changes["buffbot-fixes-to-1-12-6"]["summary"])
                self.assertEqual(changes["buffbot-fixes-to-1-12-6"]["covers"],
                                 [f"artifact:{artifact}" for artifact in (HISTORY[1], PREVIOUS, PIN)])
                self.assertFalse({"buffbot-1-12-1", "buffbot-fixes-to-1-12-1"} & set(changes))


if __name__ == "__main__":
    unittest.main()
