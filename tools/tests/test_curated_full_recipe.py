from __future__ import annotations

import tempfile
import json
import tomllib
import unittest
from pathlib import Path

from tools.curated_full_recipe import build_recipe


class CuratedFullRecipeTests(unittest.TestCase):
    def setUp(self) -> None:
        self.root = Path(__file__).resolve().parents[2]

    def test_build_is_curation_derived_and_preserves_required_regressions(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output = Path(temp_dir) / "recipe"
            build_recipe(self.root, output, "d6d46647b24b1a4baa501bca8c1d23048da3e83f")
            collection = tomllib.loads((output / "collection.toml").read_text(encoding="utf-8"))
            preset = tomllib.loads((output / "presets/chris-recommended.toml").read_text(encoding="utf-8"))
            mods = {path.stem for path in (output / "mods").glob("*.toml")}
            runs = {run["run_id"]: run for run in collection["runs"]}
            features = {feature["id"]: feature for feature in collection["features"]}

            self.assertEqual(
                {"bristlelick", "wings", "ajantisbg2", "bg2ee-eet-fixpack", "eet-tweaks"} & mods,
                set(),
            )
            self.assertNotIn("creator-full", preset["selections"])
            # WeiDU follows the pinned TP2 declaration order, not catalog/ID order.
            bg1npc_components = runs["bg1npc-bg1"]["components"]
            self.assertLess(bg1npc_components.index(240), bg1npc_components.index(160))
            self.assertLess(bg1npc_components.index(241), bg1npc_components.index(160))
            self.assertLess(bg1npc_components.index(160), bg1npc_components.index(200))
            self.assertEqual(runs["chriz-sod-remix-bg2"]["components"], [100, 110, 120, 130, 140, 150, 145, 160, 170, 180, 175, 185, 190, 195, 210, 197, 187, 200, 215, 220, 225, 245, 230, 240, 250, 255, 260, 270, 280, 900])
            self.assertLess(runs["chriz-sod-remix-bg2"]["components"].index(210), runs["chriz-sod-remix-bg2"]["components"].index(197))
            self.assertEqual(runs["bardicwonders-garrick-bg2"]["components"], [1008])
            bardic = tomllib.loads((output / "mods/bardicwonders.toml").read_text(encoding="utf-8"))
            private_artifact = tomllib.loads(
                (output / "artifacts/creator-full-private-extras-20260902.toml").read_text(encoding="utf-8")
            )
            self.assertEqual(private_artifact["archive"]["publish_roots"], ["evandra"])
            self.assertEqual(private_artifact["archive"]["tp2_paths"], ["evandra/setup-evandra.tp2"])
            prompt = next(component for component in bardic["components"] if component["id"] == 1008)["prompts"][0]
            self.assertEqual(prompt["answer"]["value"], {"kind": "integer", "value": 2})
            self.assertEqual(features["feature:chriz-bg-modpack:component-400"]["requires"], ["feature:branwen:component-0", "mod:spell-rev"])
            self.assertEqual(features["feature:chriz-bg-modpack:component-430"]["requires"], ["feature:cdtweaks:component-2170"])
            self.assertEqual(features["feature:bardicwonders:component-1006"]["readiness"], "blocked")
            self.assertEqual(features["feature:iwdification:mandatory-components"]["components"], [{"run_id": "iwdification-bg2", "component": 30}, {"run_id": "iwdification-bg2", "component": 40}])
            self.assertNotIn(120, runs["iwdification-bg2"]["components"])
            self.assertIn(2720, runs["cdtweaks-bg2"]["components"])
            self.assertIn(3121, runs["cdtweaks-bg2"]["components"])
            self.assertEqual(runs["cdtweaks-spell-save-penalties-bg2"]["components"], [2312])
            order = list(runs)
            self.assertLess(order.index("artisanskitpack-main-bg2"), order.index("bardicwonders-garrick-bg2"))
            self.assertLess(order.index("bardicwonders-garrick-bg2"), order.index("artisanskitpack-npc-bg2"))
            self.assertLess(order.index("evandra-core-bg2"), order.index("xan-bg2"))
            self.assertLess(order.index("xan-bg2"), order.index("evandra-crossmod-bg2"))
            self.assertLess(order.index("evandra-crossmod-bg2"), order.index("crossmodbg2-bg2"))
            self.assertLess(order.index("chriz-bg-modpack-bg2"), order.index("cdtweaks-spell-save-penalties-bg2"))
            self.assertLess(order.index("cdtweaks-spell-save-penalties-bg2"), order.index("spell-rev-npc-spellbooks-bg2"))
            self.assertEqual(preset["selections"]["feature:evandra:component-1"], "on")
            self.assertEqual(json.loads((output / "release.json").read_text())["version"], "0.1.0-alpha.5")
            self.assertFalse((output / "reference").exists())

    def test_reconciliation_covers_every_default_and_mandatory_row(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output = Path(temp_dir) / "recipe"
            build_recipe(self.root, output, "d6d46647b24b1a4baa501bca8c1d23048da3e83f")
            report = (output / "RECONCILIATION.md").read_text(encoding="utf-8")
            self.assertIn("Engine-equivalent resolved-selection summary", report)
            self.assertIn("440 default/mandatory rows", report)
            self.assertIn("`IWDIFICATION:120` | default", report)
            self.assertIn("`BARDICWONDERS:1006` | default", report)
            self.assertIn("`CHRIZ-BG-MODPACK:400` | mandatory", report)


if __name__ == "__main__":
    unittest.main()
