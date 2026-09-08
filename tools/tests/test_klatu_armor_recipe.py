from __future__ import annotations

import tempfile
import tomllib
import unittest
from pathlib import Path

from tools.curated_full_recipe import _effective_features, build_recipe
from tools.public_component_credits import build_public_component_credits


FEATURE_ID = "feature:klatu:component-2150"
RUN_ID = "klatu-armor-thieving-bg2"


class KlatuArmorRecipeTests(unittest.TestCase):
    def setUp(self) -> None:
        self.root = Path(__file__).resolve().parents[2]

    def _build(self, output: Path) -> tuple[dict, dict]:
        build_recipe(
            self.root,
            output,
            "d6d46647b24b1a4baa501bca8c1d23048da3e83f",
        )
        collection = tomllib.loads(
            (output / "collection.toml").read_text(encoding="utf-8")
        )
        preset = tomllib.loads(
            (output / "presets/chris-recommended.toml").read_text(encoding="utf-8")
        )["selections"]
        return collection, preset

    @staticmethod
    def _selected_components(
        collection: dict, selections: dict[str, str]
    ) -> set[tuple[str, int]]:
        effective = _effective_features(collection, selections)
        return {
            (component["run_id"], component["component"])
            for feature in collection["features"]
            if effective[feature["id"]]
            for component in feature.get("components", [])
        }

    def test_authors_one_default_checked_optional_component_with_clear_scope(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output = Path(temp_dir) / "recipe"
            collection, preset = self._build(output)
            feature = next(
                feature
                for feature in collection["features"]
                if feature["id"] == FEATURE_ID
            )

            self.assertEqual(feature["decision"], "default")
            self.assertEqual(feature["readiness"], "ready")
            self.assertEqual(feature["title"], "Use thief skills in armor")
            self.assertIn("No added skill penalties", feature["description"])
            self.assertEqual(
                feature["components"], [{"run_id": RUN_ID, "component": 2150}]
            )
            self.assertEqual(preset[FEATURE_ID], "on")
            self.assertEqual(
                [
                    feature["id"]
                    for feature in collection["features"]
                    if any(
                        component["run_id"] == RUN_ID
                        for component in feature.get("components", [])
                    )
                ],
                [FEATURE_ID],
            )

    def test_turning_the_option_off_drops_only_klatu_and_keeps_order(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output = Path(temp_dir) / "recipe"
            collection, preset = self._build(output)
            selected = self._selected_components(collection, preset)
            disabled = dict(preset)
            disabled[FEATURE_ID] = "off"
            without_klatu = self._selected_components(collection, disabled)

            self.assertEqual(
                selected - without_klatu,
                {(RUN_ID, 2150)},
            )
            self.assertFalse(without_klatu - selected)

            run_ids = [run["run_id"] for run in collection["runs"]]
            klatu = run_ids.index(RUN_ID)
            self.assertEqual(collection["runs"][klatu]["phase"], "post-eet-end")
            self.assertLess(run_ids.index("artisanskitpack-tweak-late-bg2"), klatu)
            self.assertLess(run_ids.index("artisanskitpack-npc-late-bg2"), klatu)
            self.assertLess(run_ids.index("eet-end-bg2"), klatu)
            self.assertEqual(run_ids[klatu + 1], "buffbot-bg2")
            self.assertEqual(run_ids[-1], "buffbot-bg2")

    def test_freezes_the_released_source_and_exports_coherent_public_credits(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output = Path(temp_dir) / "recipe"
            self._build(output)
            artifact = tomllib.loads(
                (output / "artifacts/klatu-tweaks-1.7.4.toml").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(artifact["version"], "1.7.4")
            self.assertEqual(artifact["source"]["kind"], "github-release")
            self.assertEqual(artifact["source"]["reference"], "Version-1.7.4")
            self.assertEqual(artifact["source"]["expected_length"], 2_461_213)
            self.assertEqual(
                artifact["source"]["sha256"],
                "7393116098ee549d9b53fdd0600d56a779ccc7feb0046667162fd10118856daa",
            )
            self.assertEqual(artifact["archive"]["publish_roots"], ["klatu"])
            self.assertEqual(
                artifact["archive"]["tp2_paths"], ["klatu/setup-klatu.tp2"]
            )

            credits = build_public_component_credits(
                output, self.root / "app/package.json"
            )
            klatu = next(mod for mod in credits["mods"] if mod["id"] == "klatu")
            self.assertEqual(klatu["version"], "1.7.4")
            self.assertEqual(
                klatu["homepage"],
                "https://github.com/The-Gate-Project/klatu-tweaks-and-fixes",
            )
            self.assertEqual(
                klatu["runs"],
                [{"id": RUN_ID, "components": [{"id": 2150, "name": "Allow Thievery in Armor"}]}],
            )


if __name__ == "__main__":
    unittest.main()
