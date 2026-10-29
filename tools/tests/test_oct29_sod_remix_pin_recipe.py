"""October 29 intake: SoD Remix v0.6.16 replaces v0.6.15 in both recipe profiles.

v0.6.16 lets 290 accept every victory remark in the vanilla shape, whoever added it (Sirene NPC
for BG:EE adds one), and lets 265 and six more components read timer names from the game's own
GTIMES.IDS. Against v0.6.15 the TP2 adds no component and keeps the declaration order, so no
decision changes (Christopher approved the release and this pin on 2026-10-29).

Static pin, evidence, component, selection and order checks for both recipe profiles only.
Nothing here is a game installation or gameplay acceptance.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import _effective_features, build_recipe
from tools.curation_audit import Decision, RowKey, load_catalogs


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261029-sodremix.toml"
PREVIOUS_EVIDENCE = "manifest/evidence/release-intake-20261023.toml"
PROFILES = ["manifest", "recipes/curated-full-current"]
MOD = "chriz-sod-remix"
PIN = "chriz-sod-remix-0.6.16"
PREVIOUS = "chriz-sod-remix-0.6.15"
TAG = "v0.6.16"
FILENAME = "chriz-sod-remix-v0.6.16.zip"
SHA256 = "7da50d206e4f0d50dd94d5e60fd985290a245d2dc3bed32068e1eaad0219c2e6"
LENGTH = 3_055_655
TP2_SHA256 = "4bf744945b40a880ddef4169156d20a9f7c9b130e2e54f5a52909fe884f8c98f"
TAG_COMMIT = "f0d6c5a1d659a5e9c96abd9d5d26c97f383b0637"
RUN = "chriz-sod-remix-bg2"
BUNDLE = "feature:chriz-sod-remix:mandatory-components"
SOD = "feature:chriz-sod-remix:component-{}"
# The 40-row mandatory bundle, unchanged by v0.6.16.
MANDATORY_BUNDLE = [
    100, 110, 115, 120, 130, 135, 140, 150, 145, 160, 170, 180, 175, 177, 185, 190, 195, 210,
    197, 198, 187, 200, 215, 220, 225, 245, 230, 240, 250, 255, 256, 260, 265, 266, 270, 280,
    285, 290, 900, 910,
]
# The components whose installers v0.6.16 changes; all stay mandatory bundle rows.
CHANGED = [135, 140, 200, 245, 256, 265, 266, 290]


def _load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def _load_profile(directory: Path) -> tuple[dict, dict]:
    collection = _load(directory / "collection.toml")
    preset = _load(directory / "presets/chris-recommended.toml")["selections"]
    return collection, preset


class Oct29SodRemixV0616PinTests(unittest.TestCase):
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
        (cls.previous,) = [record for record in _load(ROOT / PREVIOUS_EVIDENCE)["artifacts"]
                           if record["mod_id"] == MOD]

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
        self.assertEqual(self.evidence["verified_on"], "2026-10-29")
        self.assertEqual(self.evidence["evidence_kind"], "published-release-download-and-native-order-verification")
        for phrase in ("replaces v0.6.15", "fresh cache", "cache hit", "294 entries", "CRLF"):
            self.assertIn(phrase, self.evidence["scope"])
        record = self.record
        self.assertEqual(record["mod_id"], MOD)
        self.assertEqual(record["artifact_id"], PIN)
        self.assertEqual(record["recipe_profiles"], PROFILES)
        self.assertNotIn("verified_on", record)
        self.assertEqual(record["source_reference"], TAG)
        self.assertEqual(record["source_commit"], TAG_COMMIT)
        self.assertEqual(record["immutable_source_url"],
                         f"https://github.com/Chrizhermann/chriz-sod-rebalance/releases/download/{TAG}/{FILENAME}")
        self.assertEqual(record["actual_length"], LENGTH)
        self.assertEqual(record["actual_sha256"], SHA256)
        self.assertEqual(record["tp2_sha256"], TP2_SHA256)
        for flag in ("fresh_download", "fresh_extraction", "published_release_verified",
                     "native_component_order_verified"):
            self.assertIs(record[flag], True, flag)
        # The TP2 changed, but it declares the same components in the same order.
        self.assertEqual(self.previous["artifact_id"], PREVIOUS)
        self.assertNotEqual(record["tp2_sha256"], self.previous["tp2_sha256"])
        self.assertEqual(record["native_components"], self.previous["native_components"])
        self.assertEqual(len(record["native_components"]), 49)

    def test_both_profiles_pin_v0616_and_keep_v0615(self):
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
                                 f"https://github.com/Chrizhermann/chriz-sod-rebalance/releases/tag/{TAG}")
                self.assertEqual(artifact["provenance"]["reviewed_on"], "2026-10-29")
                for key, measured in (("sha256", "actual_sha256"), ("expected_length", "actual_length"),
                                      ("url", "immutable_source_url"), ("reference", "source_reference")):
                    self.assertEqual(artifact["source"][key], self.record[measured], key)
                    # The v0.6.15 record still describes the kept v0.6.15 artifact.
                    self.assertEqual(previous["source"][key], self.previous[measured], key)
                # Same layout, limits and redirect host as v0.6.15, which stays for
                # installations made from earlier recipes.
                self.assertEqual(artifact["archive"], previous["archive"])
                self.assertEqual(artifact["source"]["redirect_hosts"], previous["source"]["redirect_hosts"])
                self.assertNotEqual(artifact["source"]["sha256"], previous["source"]["sha256"])
                self.assertIn(mod["tp2"], artifact["archive"]["tp2_paths"])
        # The checked-in curated artifacts are the generator's copies of the authored pins.
        for artifact_id in (PIN, PREVIOUS):
            name = f"artifacts/{artifact_id}.toml"
            self.assertEqual((ROOT / "recipes/curated-full-current" / name).read_bytes(),
                             (ROOT / "manifest" / name).read_bytes())

    def test_components_run_and_recommended_selection_are_unchanged(self):
        native = self.record["native_components"]
        rows = {RowKey(row.catalog, row.component_id): row for row in load_catalogs(ROOT / "docs/curation/components")}
        for component in CHANGED:
            self.assertEqual(rows[RowKey("CHRIZ-SOD-REMIX", component)].decision, Decision.MANDATORY, component)
        self.assertEqual(rows[RowKey("CHRIZ-SOD-REMIX", 292)].decision, Decision.DEFAULT)
        for _, directory in self.directories:
            with self.subTest(profile=str(directory)):
                mod = _load(directory / f"mods/{MOD}.toml")
                self.assertTrue({component["id"] for component in mod["components"]} <= set(native))
                self.assertEqual((mod["tp2"], mod["language"], mod["weidu_artifact_id"], mod["invocation_mode"]),
                                 ("chriz-sod-remix/setup-chriz-sod-remix.tp2", 0, "weidu-249-amd64", "setup-name"))
                collection = _load(directory / "collection.toml")
                runs = [run for run in collection["runs"] if run["mod_id"] == MOD]
                self.assertEqual([run["run_id"] for run in runs], [RUN])
                (run,) = runs
                self.assertEqual(run["components"], [c for c in native if c in run["components"]])
                self.assertEqual(run["components"], [*MANDATORY_BUNDLE[:31], 257, *MANDATORY_BUNDLE[31:], 292, 293, 294])
                self.assertEqual(run["phase"], "post-eet-end")
        for _, _, collection, preset in self._each_profile():
            features = {feature["id"]: feature for feature in collection["features"]}
            self.assertEqual([ref["component"] for ref in features[BUNDLE]["components"]], MANDATORY_BUNDLE)
            selected = self._selected(collection, preset)
            self.assertEqual(sorted(c for run_id, c in selected if run_id == RUN), sorted([*MANDATORY_BUNDLE, 292]))
            self.assertEqual({c for run_id, c in self._selected(collection, preset, **{"mod:chriz-sod-remix": "off"})
                              if run_id == RUN}, set())

    def test_catalog_changelog_ledger_and_pin_list_name_the_pin(self):
        catalog = (ROOT / "docs/curation/components/CHRIZ-SOD-REMIX.md").read_text(encoding="utf-8")
        for phrase in (f"Pinned public release **{TAG}**", f"`{TAG_COMMIT[:7]}`", EVIDENCE,
                       "chriz-sod-rebalance#34", "chriz-sod-rebalance#35", "chriz-sod-rebalance#36",
                       "chriz-sod-rebalance#37", "Sirene NPC for BG:EE"):
            self.assertIn(phrase, catalog)
        changelog = (ROOT / "docs/plans/2026-10-14-next-release-changelogs.md").read_text(encoding="utf-8")
        for phrase in ("SoD Remix v0.6.16 (was v0.6.13)", "SoD Remix to v0.6.16", "(SoD Remix v0.6.16)",
                       "**SoD Remix's victory ending** (Mandatory)"):
            self.assertIn(phrase, changelog)
        self.assertNotIn("SoD Remix v0.6.15 (was", changelog)
        ledger = (ROOT / "manifest/mod-sources.tsv").read_text(encoding="utf-8").splitlines()
        (row,) = [line.split("\t") for line in ledger if line.startswith("CHRIZ-SOD-REMIX\t")]
        self.assertEqual(row[3:5], [TAG, "2026-10-29"])
        self.assertEqual(row[7], self.record["immutable_source_url"])
        self.assertIn(f"manifest/artifacts/{PIN}.toml", row[9])
        self.assertIn(SHA256.upper(), row[9])
        pin_list = (ROOT / "docs/pin-list-2.7.md").read_text(encoding="utf-8")
        self.assertEqual(pin_list.count(f"| CHRIZ-SOD-REMIX | {TAG.removeprefix('v')} | `{MOD}` | `{SHA256[:12]}…` |"), 2)
        self.assertNotIn("| CHRIZ-SOD-REMIX | 0.6.15 |", pin_list)


if __name__ == "__main__":
    unittest.main()
