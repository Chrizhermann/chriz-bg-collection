import json
import tomllib
import unittest
from pathlib import Path

from tools.curated_full_recipe import _effective_features
from tools.public_component_credits import build_public_component_credits


class PublicComponentCreditsTests(unittest.TestCase):
    def setUp(self) -> None:
        self.root = Path(__file__).resolve().parents[2]
        self.recipe = self.root / "recipes/curated-full-current"
        self.result = build_public_component_credits(
            self.recipe, self.root / "app/package.json"
        )

    def test_matches_the_effective_recommended_selection(self) -> None:
        collection = tomllib.loads((self.recipe / "collection.toml").read_text(encoding="utf-8"))
        preset = tomllib.loads(
            (self.recipe / "presets/chris-recommended.toml").read_text(encoding="utf-8")
        )["selections"]
        effective = _effective_features(collection, preset)
        expected: dict[str, set[int]] = {}
        for feature in collection["features"]:
            if effective[feature["id"]]:
                for component in feature.get("components", []):
                    expected.setdefault(component["run_id"], set()).add(component["component"])

        actual = {
            run["id"]: [component["id"] for component in run["components"]]
            for mod in self.result["mods"]
            for run in mod["runs"]
        }
        runs = {run["run_id"]: run for run in collection["runs"]}
        self.assertEqual(set(actual), set(expected))
        for run_id, selected in expected.items():
            self.assertEqual(actual[run_id], [value for value in runs[run_id]["components"] if value in selected])
            self.assertEqual(len(actual[run_id]), len(set(actual[run_id])))
        self.assertEqual(self.result["componentCount"], sum(map(len, actual.values())))
        self.assertEqual(self.result["componentCount"], 434)

    def test_contains_only_public_credit_fields_and_distinct_versions(self) -> None:
        self.assertEqual(self.result["applicationVersion"], "0.1.0-alpha.12")
        self.assertEqual(self.result["recipeVersion"], "0.1.0-alpha.12")
        encoded = json.dumps(self.result).lower()
        for forbidden in ("artifact_id", "source_reference", "expected_filename", ".zip", ".iemod", "c:\\\\", "creator-full"):
            self.assertNotIn(forbidden, encoded)
        for mod in self.result["mods"]:
            self.assertTrue(mod["homepage"].startswith(("https://", "http://")))
            self.assertEqual(set(mod), {"id", "name", "version", "homepage", "runs"})


if __name__ == "__main__":
    unittest.main()
