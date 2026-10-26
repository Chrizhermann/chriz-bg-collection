"""October 23 intake: BuffBot v1.10.1 and EEex Remote Console v0.2.1.

Static pin, evidence, component, selection and order checks for both recipe profiles only.
Both pins follow their October releases; nothing here is a game installation or gameplay
acceptance. The October 26 intake supersedes the BuffBot pin.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import _effective_features, build_recipe
from tools.curation_audit import Decision, load_catalogs


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261023-buffbot-eeexremote.toml"
REMOTE_COMMIT = "3d98990766abe9ed1024c79ff43d609101f37836"
# Per mod: the new pin, the superseded pin (kept as history), the archive name, the native
# TP2 declaration order and whether a GitHub release (not only a tag) publishes the source.
PINS = {
    "buffbot": {
        "artifact": "buffbot-1.10.1",
        "previous": "buffbot-1.10.0",
        "kind": "github-release",
        "reference": "v1.10.1",
        "filename": "buffbot-v1.10.1.zip",
        "native": [1, 0],
        "published_release": True,
    },
    "eeexremote": {
        "artifact": "eeexremote-0.2.1",
        "previous": "eeexremote-0.2.0",
        "kind": "github-commit-zip",
        "reference": REMOTE_COMMIT,
        "filename": f"eeex-remote-console-{REMOTE_COMMIT}.zip",
        "native": [0],
        # The repository publishes tags only; v0.2.0 had no GitHub release either.
        "published_release": False,
    },
}
# The October 26 intake (test_oct26_alpha22_pins_recipe) moves BuffBot to v1.11.0; the
# v1.10.1 artifact stays exactly as verified here.
SUPERSEDED = {"buffbot"}
# Neither release adds or changes a component, so the catalogs keep these rows and decisions.
CATALOG_ROWS = {
    "BUFFBOT": {1: Decision.MANDATORY, 0: Decision.MANDATORY},
    "EEEXREMOTE": {0: Decision.OPTIONAL},
}
BUFFBOT_RUN = "buffbot-bg2"
REMOTE_RUN = "eeexremote-bg2"
SCROLL_RUN = "chriz-bg-modpack-scrolls-bg2"
REMOTE_FEATURE = "feature:eeexremote:component-0"
EEEX_CORE = "feature:eeex:mandatory-components"


def _load_profile(directory: Path) -> tuple[dict, dict]:
    collection = tomllib.loads((directory / "collection.toml").read_text(encoding="utf-8"))
    preset = tomllib.loads((directory / "presets/chris-recommended.toml").read_text(encoding="utf-8"))["selections"]
    return collection, preset


class Oct23BuffBotRemoteConsoleIntakeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        cls.output = Path(cls.temp.name) / "recipe"
        # Synthetic evidence marker used only in this temporary test output.
        build_recipe(ROOT, cls.output, "0" * 40)
        cls.profiles = {
            "curated": (cls.output, *_load_profile(cls.output)),
            "fallback": (ROOT / "manifest", *_load_profile(ROOT / "manifest")),
        }
        cls.evidence = tomllib.loads((ROOT / EVIDENCE).read_text(encoding="utf-8"))
        cls.records = {record["mod_id"]: record for record in cls.evidence["artifacts"]}

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

    @staticmethod
    def _run(selected, run_id):
        return {component for selected_run, component in selected if selected_run == run_id}

    def test_intake_evidence_matches_current_pins(self):
        self.assertEqual(self.evidence["verified_on"], "2026-10-23")
        self.assertEqual(len(self.evidence["artifacts"]), len(PINS))
        self.assertEqual(set(self.records), set(PINS))
        for mod_id, pin in PINS.items():
            record = self.records[mod_id]
            with self.subTest(mod=mod_id):
                self.assertEqual(record["artifact_id"], pin["artifact"])
                self.assertEqual(record["recipe_profiles"], ["manifest", "recipes/curated-full-current"])
                self.assertEqual(record["source_reference"], pin["reference"])
                self.assertEqual(record["native_components"], pin["native"])
                for flag in ("fresh_download", "fresh_extraction", "native_component_order_verified"):
                    self.assertIs(record[flag], True, flag)
                self.assertIs(record["published_release_verified"], pin["published_release"])
                for digest in ("actual_sha256", "tp2_sha256"):
                    self.assertRegex(record[digest], r"^[0-9a-f]{64}$")
                self.assertRegex(record["source_commit"], r"^[0-9a-f]{40}$")
        # The Remote Console pin is the commit archive of the v0.2.1 tag commit.
        self.assertEqual(self.records["eeexremote"]["source_commit"], REMOTE_COMMIT)
        self.assertIn(REMOTE_COMMIT, self.records["eeexremote"]["immutable_source_url"])

    def test_both_profiles_pin_the_verified_archives(self):
        for profile in (ROOT / "manifest", ROOT / "recipes/curated-full-current", self.output):
            for mod_id, pin in PINS.items():
                record = self.records[mod_id]
                with self.subTest(profile=str(profile), mod=mod_id):
                    mod = tomllib.loads((profile / f"mods/{mod_id}.toml").read_text(encoding="utf-8"))
                    if mod_id in SUPERSEDED:
                        self.assertNotIn(mod["artifact_id"], (pin["artifact"], pin["previous"]))
                    else:
                        self.assertEqual(mod["artifact_id"], pin["artifact"])
                    artifact = tomllib.loads(
                        (profile / f"artifacts/{pin['artifact']}.toml").read_text(encoding="utf-8"))
                    for key, measured in (("sha256", "actual_sha256"), ("expected_length", "actual_length"),
                                          ("url", "immutable_source_url"), ("reference", "source_reference")):
                        self.assertEqual(artifact["source"][key], record[measured])
                    self.assertEqual(artifact["source"]["kind"], pin["kind"])
                    self.assertEqual(artifact["source"]["expected_filename"], pin["filename"])
                    # Same source kind, layout and limits as the previous pin, which stays as history.
                    previous = tomllib.loads(
                        (profile / f"artifacts/{pin['previous']}.toml").read_text(encoding="utf-8"))
                    self.assertEqual(artifact["source"]["kind"], previous["source"]["kind"])
                    self.assertEqual(artifact["archive"], previous["archive"])
                    self.assertEqual(artifact["source"]["redirect_hosts"], previous["source"]["redirect_hosts"])
                    self.assertNotEqual(artifact["source"]["sha256"], previous["source"]["sha256"])
        # The checked-in curated artifacts are the generator's copies of the authored pins.
        for pin in PINS.values():
            name = f"artifacts/{pin['artifact']}.toml"
            self.assertEqual((ROOT / "recipes/curated-full-current" / name).read_bytes(),
                             (ROOT / "manifest" / name).read_bytes())

    def test_components_are_unchanged_and_runs_follow_native_order(self):
        rows = {}
        for row in load_catalogs(ROOT / "docs/curation/components"):
            if row.catalog in CATALOG_ROWS:
                rows.setdefault(row.catalog, {})[row.component_id] = row.decision
        self.assertEqual(rows, CATALOG_ROWS)
        for profile in (ROOT / "manifest", ROOT / "recipes/curated-full-current", self.output):
            collection = tomllib.loads((profile / "collection.toml").read_text(encoding="utf-8"))
            for mod_id, pin in PINS.items():
                with self.subTest(profile=str(profile), mod=mod_id):
                    mod = tomllib.loads((profile / f"mods/{mod_id}.toml").read_text(encoding="utf-8"))
                    self.assertEqual([component["id"] for component in mod["components"]], pin["native"])
                    mod_runs = [run for run in collection["runs"] if run["mod_id"] == mod_id]
                    self.assertEqual(len(mod_runs), 1)
                    self.assertEqual(mod_runs[0]["components"], pin["native"])
                    self.assertEqual(mod_runs[0]["phase"], "post-eet-end")

    def test_buffbot_stays_after_the_voicepacks_and_before_the_scroll_policy(self):
        for _, _, collection, preset in self._each_profile():
            order = [run["run_id"] for run in collection["runs"]]
            buffbot = order.index(BUFFBOT_RUN)
            voicepacks = [index for index, run_id in enumerate(order) if run_id.startswith("voicepack-")]
            self.assertTrue(voicepacks)
            self.assertLess(max(voicepacks), buffbot)
            # Modpack's scroll policy 430 reads the final scrolls and spells, so its run alone follows.
            self.assertEqual(order[buffbot + 1:], [SCROLL_RUN])
            features = {feature["id"]: feature for feature in collection["features"]}
            self.assertEqual(features["mod:buffbot"]["decision"], "default")
            self.assertEqual(preset.get("mod:buffbot"), "on")
            selected = self._selected(collection, preset)
            self.assertEqual(self._run(selected, BUFFBOT_RUN), {1, 0})
            self.assertEqual(self._run(self._selected(collection, preset, **{"mod:buffbot": "off"}), BUFFBOT_RUN),
                             set())

    def test_remote_console_stays_an_optional_tool_before_the_modpack(self):
        for _, _, collection, preset in self._each_profile():
            order = [run["run_id"] for run in collection["runs"]]
            remote = order.index(REMOTE_RUN)
            self.assertLess(order.index("eet-end-bg2"), remote)
            self.assertEqual(order[remote + 1], "chriz-bg-modpack-bg2")
            features = {feature["id"]: feature for feature in collection["features"]}
            feature = features[REMOTE_FEATURE]
            self.assertEqual(feature["decision"], "optional")
            self.assertEqual(feature["readiness"], "experimental")
            self.assertEqual(feature["requires"], [EEEX_CORE])
            self.assertEqual(feature["components"], [{"run_id": REMOTE_RUN, "component": 0}])
            self.assertNotEqual(preset.get(REMOTE_FEATURE), "on")
            self.assertEqual(self._run(self._selected(collection, preset), REMOTE_RUN), set())
            self.assertEqual(self._run(self._selected(collection, preset, **{REMOTE_FEATURE: "on"}), REMOTE_RUN),
                             {0})
            self.assertEqual(self._run(self._selected(
                collection, preset, **{REMOTE_FEATURE: "on", "mod:eeex": "off"}), REMOTE_RUN), set())


if __name__ == "__main__":
    unittest.main()
