"""Release-record regressions; these checks do not install mods or touch games."""

from pathlib import Path
import json
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import _effective_features, build_recipe


class ReleaseMetadataTests(unittest.TestCase):
    root = Path(__file__).resolve().parents[2]

    def test_rebalance_candidate_does_not_claim_published_acquisition(self):
        evidence = tomllib.loads(
            (self.root / "manifest/evidence/release-candidate-20260930-rebalance.toml").read_text()
        )
        self.assertEqual(evidence["evidence_kind"], "release-candidate-local-verification")
        record, = evidence["artifacts"]
        self.assertEqual(record["artifact_id"], "chriz-bg-rebalance-0.7.1")
        self.assertFalse(record["fresh_download"])
        self.assertFalse(record["fresh_extraction"])
        self.assertFalse(record["published_release_verified"])
        for relative in ("manifest", "recipes/curated-full-current"):
            with self.subTest(profile=relative):
                profile = self.root / relative
                # Historical candidate evidence describes its immutable source,
                # not the newer current default pin.
                artifact = tomllib.loads((profile / f"artifacts/{record['artifact_id']}.toml").read_text())
                for source_key, evidence_key in (
                    ("reference", "source_reference"), ("url", "planned_source_url"),
                    ("expected_length", "actual_length"), ("sha256", "actual_sha256"),
                ):
                    self.assertEqual(artifact["source"][source_key], record[evidence_key])

    def test_published_rebalance_intake_matches_pins_and_native_order_without_claiming_extraction(self):
        evidence = tomllib.loads(
            (self.root / "manifest/evidence/release-intake-20260930.toml").read_text()
        )
        self.assertEqual(evidence["evidence_kind"], "published-release-download-and-native-order-verification")
        record, = evidence["artifacts"]
        self.assertEqual(record["source_commit"], "1578fac00cd93ba88341ffdfcb0346c08480cd21")
        self.assertTrue(record["fresh_download"])
        self.assertFalse(record["fresh_extraction"])
        self.assertTrue(record["published_release_verified"])
        self.assertTrue(record["normal_latest"])
        self.assertTrue(record["native_component_order_verified"])
        for relative in ("manifest", "recipes/curated-full-current"):
            with self.subTest(profile=relative):
                profile = self.root / relative
                artifact = tomllib.loads((profile / f"artifacts/{record['artifact_id']}.toml").read_text())
                for source_key, evidence_key in (
                    ("reference", "source_reference"), ("url", "immutable_source_url"),
                    ("expected_length", "actual_length"), ("sha256", "actual_sha256"),
                ):
                    self.assertEqual(artifact["source"][source_key], record[evidence_key])
                self.assertEqual(record["native_components"][-3:], [302, 420, 421])

    def test_both_profiles_cover_every_selected_run_and_tail(self):
        for relative in ("manifest", "recipes/curated-full-current"):
            with self.subTest(profile=relative):
                profile = self.root / relative
                collection = tomllib.loads((profile / "collection.toml").read_text())
                preset = tomllib.loads((profile / "presets/chris-recommended.toml").read_text())
                state = _effective_features(collection, preset["selections"])
                selected = {ref["run_id"] for feature in collection["features"]
                            if state[feature["id"]] for ref in feature.get("components", [])}
                records = profile / "releases/v0.1.0-alpha.1"
                evidence = tomllib.loads((records / "acceptance.toml").read_text())["evidence"]
                accepted = {item["subject_id"] for item in evidence
                            if item["subject_kind"] == "run" and item["kind"] == "static-test"
                            and item["status"] == "accepted"}
                self.assertFalse(selected - accepted)
                limits = tomllib.loads((records / "known-limitations.toml").read_text())
                tail = {run["run_id"] for run in collection["runs"]
                        if run["run_id"] in selected and run["phase"] == "post-eet-end"}
                self.assertFalse(tail - set(limits["approved_tail_runs"]))
                self.assertNotIn("feature:artisanskitpack-npc:component-5102",
                                 {item["feature_id"] for item in limits.get("omissions", [])})

    def test_current_release_has_unique_change_ids_across_history(self):
        with tempfile.TemporaryDirectory() as temp:
            output = Path(temp) / "recipe"
            build_recipe(self.root, output, "d6d46647b24b1a4baa501bca8c1d23048da3e83f")
            self.assertEqual(json.loads((output / "release.json").read_text())["version"],
                             "0.1.0-alpha.22")
            seen = set()
            previous = "0.1.0-alpha.13"
            # Alpha.21 was an app-only release with the alpha.20 recipe, so alpha.22
            # supersedes alpha.20 directly.
            self.assertFalse((output / "releases/v0.1.0-alpha.21").exists())
            for number in (14, 15, 16, 17, 18, 19, 20, 22):
                version = f"0.1.0-alpha.{number}"
                ledger = tomllib.loads((output / f"releases/v{version}/ledger.toml").read_text())
                self.assertEqual(ledger["supersedes"], previous)
                ids = {item["id"] for item in ledger["changes"]}
                self.assertFalse(ids & seen, f"repeated changes in {version}: {ids & seen}")
                seen.update(ids)
                for item in ledger["changes"]:
                    self.assertFalse(any(address.startswith("mod:") for address in item["covers"]),
                                     f"nonexistent diff address in {version}/{item['id']}")
                previous = version
            self.assertFalse((output / "artifacts/chriz-bg-modpack-0.2.0-alpha.5.toml").exists())
            self.assertFalse((output / "artifacts/bardicwonders-v2.9c-balance.4.toml").exists())
            self.assertFalse((output / "artifacts/chriz-bg-rebalance-0.4.0.toml").exists())

    def test_emotion_default_has_a_separate_next_playthrough_release(self):
        ledger = tomllib.loads(
            (self.root / "manifest/releases/v0.1.0-alpha.18/ledger.toml").read_text()
        )
        self.assertEqual(ledger["version"], "0.1.0-alpha.18")
        self.assertEqual(ledger["supersedes"], "0.1.0-alpha.17")
        self.assertEqual(ledger["minimum_app_version"], "0.1.0-alpha.19")
        self.assertEqual(len(ledger["changes"]), 12)
        change = next(item for item in ledger["changes"]
                      if item["id"] == "emotion-courage-hope-exclusive-default")
        self.assertEqual(change["id"], "emotion-courage-hope-exclusive-default")
        self.assertEqual(change["save_applicability"], "next-playthrough")
        self.assertEqual(change["urgency"], "recommended")
        self.assertCountEqual(change["covers"], [
            "artifact:chriz-bg-rebalance-0.4.0",
            "artifact:chriz-bg-rebalance-0.6.0",
            "run:chriz-bg-rebalance-bg2",
            "feature:feature:chriz-bg-rebalance:component-301",
        ])


if __name__ == "__main__":
    unittest.main()
