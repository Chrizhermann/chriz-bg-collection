"""November 4 intake: BuffBot v1.16.0 replaces v1.12.6 in both Chriz Easy BG recipe profiles.

v1.16.0 contains v1.12.7 to v1.15.0: the party table where buffs are chosen, its cast order,
Project Images and Simulacra planned before they exist, summons as recipients, five text sizes
and the main component installing without the LuaJIT loader. Against v1.12.6 the TP2 keeps
both components, their names and their order, so no decision changes (Christopher,
2026-11-04: newer BuffBot releases go into both installers for 0.1.2). Chriz Easy IWD gets the
same release in PR #107.

Static pin, evidence, component, selection, order and release-text checks for both recipe
profiles only. Nothing here is a game installation or gameplay acceptance.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import _effective_features, build_recipe
from tools.curation_audit import Decision, load_catalogs


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261104-buffbot-chriz-easy-bg.toml"
PREVIOUS_EVIDENCE = "manifest/evidence/release-intake-20261101-buffbot-chriz-easy-bg.toml"
PROFILES = ["manifest", "recipes/curated-full-current"]
MOD = "buffbot"
PIN = "buffbot-1.16.0"
PREVIOUS = "buffbot-1.12.6"
# Older pins that stay as history next to the superseded one.
HISTORY = ["buffbot-1.10.1", "buffbot-1.11.0", "buffbot-1.12.1"]
TAG = "v1.16.0"
FILENAME = "buffbot-v1.16.0.zip"
SHA256 = "ce38d81102f717acf709cc8a7827586f8ab6ff5ff450aaca42f0b5d513da0082"
LENGTH = 5_797_041
TP2_SHA256 = "6701fe2da8ca3321b91b01ecb28533590a7b4847101efcf4318db9ff4dff29ad"
TAG_COMMIT = "512a1fa625b26a864dbb394336110595b08b81a6"
RUN = "buffbot-bg2"
SCROLL_RUN = "chriz-bg-modpack-scrolls-bg2"
RUNTIME = "feature:buffbot:mandatory-components"
# The TP2 declares the LuaJIT helper 1 before the main component 0; both names are unchanged.
COMPONENTS = [
    (1, "BuffBot: EEex LuaJIT Support (auto-detected)"),
    (0, "BuffBot: In-Game Buff Automation"),
]


def _load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def _load_profile(directory: Path) -> tuple[dict, dict]:
    collection = _load(directory / "collection.toml")
    preset = _load(directory / "presets/chris-recommended.toml")["selections"]
    return collection, preset


class Nov04BuffBotV1160PinTests(unittest.TestCase):
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
        self.assertEqual(self.evidence["verified_on"], "2026-11-04")
        self.assertEqual(self.evidence["evidence_kind"], "published-release-download-and-native-order-verification")
        self.assertEqual(self.evidence["generator"], "tools/prepare_own_mod_pin.py")
        for phrase in ("replaces buffbot-1.12.6", "fresh cache", "a normal GitHub release",
                       "superseded artifact TOMLs in manifest, recipes/curated-full-current",
                       "tag v1.16.0 is on commit 512a1fa", "byte-identical to the tag commit's tree",
                       "FAIL @112", "PRINT @114", "BfBotSum.lua",
                       "same two components with the same names in the same order",
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
        # The TP2 changed, but it declares the same two components in the same order.
        self.assertEqual(self.previous["artifact_id"], PREVIOUS)
        self.assertNotEqual(record["tp2_sha256"], self.previous["tp2_sha256"])
        self.assertEqual(record["native_components"], self.previous["native_components"])
        self.assertEqual(record["native_components"], [component for component, _ in COMPONENTS])

    def test_both_profiles_pin_v1160_and_keep_the_earlier_pins(self):
        for profile_name, directory in self.directories:
            with self.subTest(profile=str(directory)):
                self.assertIn(profile_name, self.record["recipe_profiles"])
                mod = _load(directory / f"mods/{MOD}.toml")
                self.assertEqual(mod["artifact_id"], PIN)
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
                self.assertEqual(artifact["provenance"]["reviewed_on"], "2026-11-04")
                self.assertEqual(artifact["provenance"]["license"], previous["provenance"]["license"])
                for key, measured in (("sha256", "actual_sha256"), ("expected_length", "actual_length"),
                                      ("url", "immutable_source_url"), ("reference", "source_reference")):
                    self.assertEqual(artifact["source"][key], self.record[measured], key)
                    # The v1.12.6 record still describes the kept v1.12.6 artifact.
                    self.assertEqual(previous["source"][key], self.previous[measured], key)
                # Same layout, limits and redirect host as v1.12.6, which stays for
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

    def test_the_main_component_is_what_the_runtime_exists_for(self):
        # Since v1.12.7 the main component installs without the LuaJIT helper, so a failed
        # helper can leave alone and takes nothing along.
        for _, _, collection, _ in self._each_profile():
            runtime = next(feature for feature in collection["features"] if feature["id"] == RUNTIME)
            self.assertEqual(runtime["core_components"], [{"run_id": RUN, "component": 0}])
            needs = [row for row in collection.get("component_needs", [])
                     if row.get("component", {}).get("run_id") == RUN]
            self.assertEqual(needs, [])

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

    def test_catalog_changelog_and_pin_list_name_the_pin(self):
        catalog = (ROOT / "docs/curation/components/BUFFBOT.md").read_text(encoding="utf-8")
        folded = " ".join(catalog.split())
        for phrase in (f"Target release: **{TAG}**", f"`{TAG_COMMIT[:7]}`", EVIDENCE,
                       "Christopher, 2026-11-04", "`FAIL @112`", "`PRINT @114`", "`BfBotSum.lua`",
                       "schema 10", "`cln`", "`430`", "PR #107",
                       *(f"bg-eeex-buffbot#{issue}" for issue in (105, 106, 107, 112, 115, 116, 117)),
                       *(f"- v{version} (released" for version in
                         ("1.12.7", "1.12.8", "1.12.9", "1.13.0", "1.14.0", "1.15.0", "1.16.0"))):
            self.assertIn(phrase, folded)
        changelog = (ROOT / "docs/plans/2026-10-14-next-release-changelogs.md").read_text(encoding="utf-8")
        draft = changelog[changelog.index("## Collection 0.1.2 (draft)"):changelog.index("## Installer changelog")]
        for phrase in ("- BuffBot v1.16.0 (was v1.12.6)", "**BuffBot v1.16.0**", "`artifact:buffbot-1.16.0`",
                       *(f"(BuffBot v{version}" for version in
                         ("1.12.7", "1.12.8", "1.12.9", "1.13.0", "1.14.0", "1.15.0", "1.16.0"))):
            self.assertIn(phrase, draft)
        # The 0.1.0 sections keep the version 0.1.0 shipped.
        self.assertIn("BuffBot v1.12.6 (was v1.10.0)", changelog[changelog.index("## Collection changelog"):])
        pin_list = (ROOT / "docs/pin-list-2.7.md").read_text(encoding="utf-8")
        self.assertEqual(pin_list.count(f"| BuffBot | {TAG.removeprefix('v')} | `{MOD}` | `{SHA256[:12]}…` |"), 2)
        self.assertNotIn("| BuffBot | 1.12.6 |", pin_list)


if __name__ == "__main__":
    unittest.main()
