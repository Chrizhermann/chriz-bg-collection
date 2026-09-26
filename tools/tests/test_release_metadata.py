"""Release-record regressions; these checks do not install mods or touch games."""

from pathlib import Path
import json
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import _effective_features, build_recipe


class ReleaseMetadataTests(unittest.TestCase):
    root = Path(__file__).resolve().parents[2]

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
                             "0.1.0-alpha.17")
            seen = set()
            previous = "0.1.0-alpha.13"
            for number in range(14, 18):
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


if __name__ == "__main__":
    unittest.main()
