"""November 18 save and rest fixes and their source/package verification."""

from pathlib import Path
import tomllib
import unittest

from tools.curated_full_recipe import _effective_features


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE_DIR = ROOT / "manifest/evidence"
CURATED = "recipes/curated-full-current"
ARTISAN = "artisans-kitpack-4.81a-dev.85928f9-chriz.7"
PREVIOUS_ARTISAN = "artisans-kitpack-4.81a-dev.85928f9-chriz.6"
BARDIC = "bardicwonders-v2.9c-balance.9"
PREVIOUS_BARDIC = "bardicwonders-v2.9c-balance.8"

INSTALLERS = {
    "artisanskitpack": (
        ARTISAN, "ArtisansKitpack/ArtisansKitpack.TP2",
        "221c0e04458cf3a088272bc60cb11a343efa41cc3cb9c86af4fdb221f92d17b4",
    ),
    "artisanskitpack-npc": (
        ARTISAN, "ArtisansKitpack_npc/ArtisansKitpack_npc.TP2",
        "7253c89a39b8a3c9ca1a79d821d89d35b7ee1d4be2a64a630a14feab4235bcf4",
    ),
    "artisanskitpack-tweak": (
        ARTISAN, "ArtisansKitpack_tweak/ArtisansKitpack_tweak.TP2",
        "7bccfa5b90a14db87f452fff784ddd46eb1e7a08cfb68347881b277caeb05771",
    ),
    "bardicwonders": (
        BARDIC, "BardicWonders/Setup-BardicWonders.tp2",
        "ae60a1f219a744a1ec917f93a12e128d4c1c0710299e1ddf02bb51695d95ea02",
    ),
}
RELEASES = {
    ARTISAN: {
        "repository": "Chrizhermann/The-Artisan-s-Kitpack-Chriz-Balance-Patch",
        "tag": "v4.81a-dev.85928f9-chriz.7",
        "commit": "b8065326f40307267e85290e0ae91188dbcc201d",
        "pull_request": 19,
        "filename": "ArtisansKitpack-Chriz-Balance-Patch-v4.81a-dev.85928f9-chriz.7.zip",
        "length": 89_864_746,
        "sha256": "6148fa21b58e181db1673f143e10d45fa2cecd2513ae9efa72e7969d066ab709",
        "profiles": ["manifest", CURATED],
        "shape": (7, 3028, 6_142_193, 118_400_047),
        "tests": 154,
        "syntax_checks": 3,
        "synthetic_games": ["BG2:EE", "EET"],
    },
    BARDIC: {
        "repository": "Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch",
        "tag": "v2.9c-balance.9",
        "commit": "c0c2e4113d016c935162011667cffe0c0756b325",
        "pull_request": 14,
        "filename": "Bardic-Wonders-v2.9c-balance.9.zip",
        "length": 5_220_111,
        "sha256": "e34af0d979e43e95885452d2b8133c009014558c40fd0ea9bfdcd958dbbb5708",
        "profiles": [CURATED],
        "shape": (5, 856, 3_311_752, 33_115_406),
        "tests": 211,
        "syntax_checks": 54,
        "synthetic_games": ["BG2:EE", "EET", "IWD:EE"],
    },
}
ARTISAN_RUNS = {
    "artisanskitpack-main-bg2": [
        1, 2, 20000, 20001, 8001, 8101, 8002, 8004, 10001, 10002, 10003, 10004,
        1100, 1003, 1006, 1004, 1005, 1007, 1000, 1001, 1008, 1009,
        2000, 2010, 2011, 2012, 2002, 3000, 3010, 3003, 3011, 3004, 3001, 3002,
        3005, 5100, 5110, 5001, 5002, 7004, 7006, 7001, 7002, 7003, 7005, 9001,
    ],
    "artisanskitpack-npc-bg2": [
        1101, 2001, 3101, 3102, 5101, 5102, 7101, 7102, 7104, 21001, 9101,
        10004, 20002, 99001,
    ],
    "artisanskitpack-tweak-early-bg2": [20101, 1209, 7203],
    "artisanskitpack-tweak-late-bg2": [51010, 12012, 3202, 3302, 7205, 8204, 1010000],
    "artisanskitpack-npc-late-bg2": [200010],
}
BARDIC_RUNS = {
    "bardicwonders-bg2": [
        1001, 1002, 1003, 1004, 1005, 1006, 1007, 1009, 1010, 1011,
        2002, 2007, 2008, 2003, 2004, 2005, 2006,
    ],
    "bardicwonders-garrick-bg2": [1008],
    "bardicwonders-dialogue-patch-bg2": [1012, 3001],
    "bardicwonders-progression-bg2": [3010],
}


def load(path: Path) -> dict:
    with path.open("rb") as source:
        return tomllib.load(source)


class Nov18SaveRestFixPinsTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.evidence = load(EVIDENCE_DIR / "release-intake-20261118-artisan-chriz7-bardic-balance9.toml")
        cls.records = {record["mod_id"]: record for record in cls.evidence["artifacts"]}

    def test_fresh_evidence_and_active_pins_match_the_published_releases(self):
        self.assertEqual(self.evidence["schema"], 1)
        self.assertEqual(self.evidence["evidence_kind"], "published-release-download-and-native-order-verification")
        self.assertEqual(self.evidence["verified_on"], "2026-11-18")
        self.assertEqual(len(self.evidence["artifacts"]), len(INSTALLERS))
        self.assertEqual(set(self.records), set(INSTALLERS))
        for mod_id, (artifact_id, tp2_path, tp2_sha256) in INSTALLERS.items():
            with self.subTest(mod=mod_id):
                release = RELEASES[artifact_id]
                record = self.records[mod_id]
                source_url = (
                    f"https://github.com/{release['repository']}/releases/download/"
                    f"{release['tag']}/{release['filename']}"
                )
                self.assertEqual(record["artifact_id"], artifact_id)
                self.assertEqual(record["recipe_profiles"], release["profiles"])
                self.assertEqual(record["source_reference"], release["tag"])
                self.assertEqual(record["source_commit"], release["commit"])
                self.assertEqual(record["immutable_source_url"], source_url)
                self.assertEqual(record["actual_length"], release["length"])
                self.assertEqual(record["actual_sha256"], release["sha256"])
                self.assertEqual(record["tp2_sha256"], tp2_sha256)
                for flag in ("fresh_download", "fresh_extraction", "published_release_verified",
                             "native_component_order_verified"):
                    self.assertIs(record[flag], True, flag)
                self.assertEqual(record.get("native_order_recipe_profiles", release["profiles"]),
                                 release["profiles"])
                for profile in ("manifest", CURATED, "recipes/iwd-current"):
                    descriptor = ROOT / profile / f"mods/{mod_id}.toml"
                    if profile not in release["profiles"]:
                        self.assertFalse(descriptor.exists(), descriptor)
                        self.assertFalse((ROOT / profile / f"artifacts/{artifact_id}.toml").exists())
                        continue
                    mod = load(descriptor)
                    artifact = load(ROOT / profile / f"artifacts/{artifact_id}.toml")
                    self.assertEqual((mod["artifact_id"], mod["tp2"]), (artifact_id, tp2_path))
                    self.assertEqual(artifact["id"], artifact_id)
                    self.assertEqual(artifact["version"], release["tag"].removeprefix("v"))
                    self.assertEqual(artifact["acquisition"], "fetch-only")
                    source = artifact["source"]
                    self.assertEqual(source["kind"], "github-release")
                    self.assertEqual(source["url"], source_url)
                    self.assertEqual(source["reference"], release["tag"])
                    self.assertEqual(source["expected_filename"], release["filename"])
                    self.assertEqual(source["expected_length"], release["length"])
                    self.assertEqual(source["sha256"], release["sha256"])

    def test_archive_shapes_publish_roots_and_descriptor_history(self):
        observations = {
            record["artifact_id"]: record
            for record in load(EVIDENCE_DIR / "artifact-verification.toml")["artifacts"]
        }
        historical = ROOT / f"manifest/reference/historical-artifacts/{PREVIOUS_BARDIC}.toml"
        self.assertFalse((ROOT / CURATED / f"artifacts/{PREVIOUS_BARDIC}.toml").exists())
        previous = {
            ARTISAN: load(ROOT / f"manifest/artifacts/{PREVIOUS_ARTISAN}.toml"),
            BARDIC: load(historical),
        }
        self.assertEqual(
            previous[ARTISAN], load(ROOT / CURATED / f"artifacts/{PREVIOUS_ARTISAN}.toml"),
        )
        self.assertEqual(previous[BARDIC]["source"]["sha256"],
                         "fa99ef91f39dfa4ae1fc15d234022e2623c359e991372f24afa116f5053feda1")
        for artifact_id, release in RELEASES.items():
            with self.subTest(artifact=artifact_id):
                observed = observations[artifact_id]
                self.assertEqual(observed["actual_length"], release["length"])
                self.assertEqual(observed["actual_sha256"], release["sha256"])
                self.assertEqual(observed["observed_final_host"], "release-assets.githubusercontent.com")
                self.assertEqual(observed["wrapper_kind"], "direct")
                self.assertIs(observed["fresh_download"], True)
                self.assertIs(observed["fresh_extraction"], True)
                self.assertEqual(observed["observed_shape"], dict(zip(
                    ("max_depth", "entry_count", "max_entry_uncompressed_bytes",
                     "total_uncompressed_bytes", "max_compression_ratio"),
                    (*release["shape"], 432),
                )))
                paths = [tp2 for pin, tp2, _ in INSTALLERS.values() if pin == artifact_id]
                roots = [path.split("/", 1)[0] for path in paths]
                self.assertEqual(observed["declared_tp2_paths"], paths)
                self.assertEqual(observed["materialized_publish_roots"], roots)
                for profile in release["profiles"]:
                    archive = load(ROOT / profile / f"artifacts/{artifact_id}.toml")["archive"]
                    self.assertEqual(archive, previous[artifact_id]["archive"])
                    self.assertEqual(archive["publish_roots"], roots)
                    self.assertEqual(archive["tp2_paths"], paths)
                    self.assertEqual(observed["verified_limits"], archive["limits"])

    def test_component_lists_native_order_and_recommended_selections_stay(self):
        previous_records = {}
        for filename in ("release-intake-20261102-kitpack-chriz6-rebalance-089.toml",
                         "release-intake-20261014.toml"):
            previous_records.update({
                record["mod_id"]: record for record in load(EVIDENCE_DIR / filename)["artifacts"]
                if record["mod_id"] in INSTALLERS
            })
        for mod_id, record in self.records.items():
            self.assertEqual(record["native_components"], previous_records[mod_id]["native_components"], mod_id)

        for profile in ("manifest", CURATED, "recipes/iwd-current"):
            with self.subTest(profile=profile):
                collection = load(ROOT / profile / "collection.toml")
                runs = {run["run_id"]: run for run in collection["runs"] if run["mod_id"] in INSTALLERS}
                if profile == "recipes/iwd-current":
                    self.assertEqual(runs, {})
                    continue
                expected = ARTISAN_RUNS if profile == "manifest" else ARTISAN_RUNS | BARDIC_RUNS
                self.assertEqual({name: run["components"] for name, run in runs.items()}, expected)
                for name, run in runs.items():
                    native = self.records[run["mod_id"]]["native_components"]
                    self.assertEqual(run["components"], [component for component in native
                                                         if component in run["components"]], name)

                preset = load(ROOT / profile / "presets/chris-recommended.toml")["selections"]
                enabled = _effective_features(collection, preset)
                selected = {
                    (ref["run_id"], ref["component"])
                    for feature in collection["features"] if enabled[feature["id"]]
                    for ref in feature.get("components", ()) if ref["run_id"] in runs
                }
                unselected = {
                    ("artisanskitpack-main-bg2", 8101),
                    ("artisanskitpack-tweak-late-bg2", 51010),
                    ("artisanskitpack-npc-late-bg2", 200010),
                    *(("artisanskitpack-npc-bg2", component) for component in (5101, 7101, 7104, 10004)),
                }
                if profile == "manifest":
                    unselected.add(("artisanskitpack-npc-bg2", 99001))
                else:
                    unselected.add(("bardicwonders-bg2", 1006))
                self.assertEqual(selected, {
                    (run_id, component) for run_id, components in expected.items() for component in components
                } - unselected)

    def test_owner_verification_records_automated_coverage(self):
        records = {record["release"]: record for record in self.evidence["owner_releases"]}
        self.assertEqual(set(records), {release["tag"] for release in RELEASES.values()})
        for release in RELEASES.values():
            with self.subTest(release=release["tag"]):
                record = records[release["tag"]]
                homepage = f"https://github.com/{release['repository']}"
                self.assertEqual(record["repository"], release["repository"])
                self.assertEqual(record["source_commit"], release["commit"])
                self.assertEqual(record["release_url"], f"{homepage}/releases/tag/{release['tag']}")
                self.assertEqual(record["pull_request_url"], f"{homepage}/pull/{release['pull_request']}")
                self.assertEqual(record["offline_tests"], release["tests"])
                self.assertEqual(record["weidu_syntax_checks"], release["syntax_checks"])
                self.assertEqual(record["synthetic_games"], release["synthetic_games"])
                self.assertIs(record["live_gameplay_verified"], False)
        self.assertEqual(records[RELEASES[BARDIC]["tag"]]["native_fixture_checks"], 15)


if __name__ == "__main__":
    unittest.main()
