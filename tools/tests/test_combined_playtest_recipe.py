from __future__ import annotations

import hashlib
import json
from pathlib import Path
import tempfile
import tomllib
import unittest
import zipfile

from tools.prepare_combined_playtest_recipe import REQUIRED_SOURCE_IDS, build_recipe


class CombinedPlaytestRecipeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.root = Path(__file__).resolve().parents[2]

    def _sources(self, root: Path) -> Path:
        layouts = {
            "chriz-bg-modpack": (["chriz-bg-modpack", "setup-chriz-bg-modpack.tp2"], ["setup-chriz-bg-modpack.tp2"]),
            "chriz-sod-remix": (["chriz-sod-remix"], ["chriz-sod-remix/setup-chriz-sod-remix.tp2"]),
            "artisans-kitpack": (["ArtisansKitpack", "ArtisansKitpack_npc", "ArtisansKitpack_tweak"], ["ArtisansKitpack/ArtisansKitpack.TP2", "ArtisansKitpack_npc/ArtisansKitpack_npc.TP2", "ArtisansKitpack_tweak/ArtisansKitpack_tweak.TP2"]),
            "spell-rev-lightning": (["spell_rev"], ["spell_rev/setup-spell_rev.tp2"]),
            "chriz-bg-rebalance": (["chriz-bg-rebalance", "setup-chriz-bg-rebalance.tp2"], ["setup-chriz-bg-rebalance.tp2"]),
            "srcb-rr-compat": (["SRCB_RR_COMPAT"], ["SRCB_RR_COMPAT/setup-SRCB_RR_COMPAT.tp2"]),
            "safana-in-amn": (["Safana"], ["Safana/Safana.tp2"]),
        }
        sources = []
        for index, source_id in enumerate(REQUIRED_SOURCE_IDS):
            publish_roots, tp2_paths = layouts[source_id]
            archive = root / f"{source_id}-20260908.zip"
            with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as package:
                for tp2 in tp2_paths:
                    package.writestr(tp2, f"BACKUP ~backup~\nAUTHOR ~test~\nBEGIN ~{source_id}~\n")
            data = archive.read_bytes()
            with zipfile.ZipFile(archive) as package:
                infos = package.infolist()
                entry_count = len(infos)
                max_entry = max(info.file_size for info in infos)
                total = sum(info.file_size for info in infos)
                max_depth = max(len(Path(info.filename).parts) for info in infos)
            sources.append(
                {
                    "id": source_id,
                    "archive": archive.name,
                    "sha256": hashlib.sha256(data).hexdigest(),
                    "size_bytes": len(data),
                    "entry_count": entry_count,
                    "max_entry_bytes": max_entry,
                    "max_depth": max_depth,
                    "uncompressed_bytes": total,
                    "root_layout": publish_roots,
                    "root_rule": "direct",
                    "publish_roots": publish_roots,
                    "tp2_paths": tp2_paths,
                    "max_compression_ratio": 100,
                    "commit": f"{index + 1:040x}",
                    "dirty_overlay": [],
                    "source_path": f"C:/test/{source_id}",
                    "provenance": {
                        "homepage": f"https://example.invalid/{source_id}",
                        "license": "test-only",
                        "provenance_url": f"https://example.invalid/{source_id}/commit/{index + 1:040x}",
                    },
                }
            )
        manifest = root / "sources.json"
        manifest.write_text(json.dumps({"schema_version": 1, "sources": sources}), encoding="utf-8")
        return manifest

    @staticmethod
    def _tree_hashes(root: Path) -> dict[str, str]:
        return {
            path.relative_to(root).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in root.rglob("*")
            if path.is_file()
        }

    def test_builds_local_recipe_without_mutating_public_base(self) -> None:
        base = self.root / "recipes/curated-full-current"
        before = self._tree_hashes(base)
        with tempfile.TemporaryDirectory() as temp_dir:
            temp = Path(temp_dir)
            output = temp / "recipe"
            build_recipe(self.root, self._sources(temp), output)

            self.assertEqual(before, self._tree_hashes(base))
            collection = tomllib.loads((output / "collection.toml").read_text(encoding="utf-8"))
            base_collection = tomllib.loads((base / "collection.toml").read_text(encoding="utf-8"))
            preset = tomllib.loads((output / "presets/chris-recommended.toml").read_text(encoding="utf-8"))["selections"]
            runs = collection["runs"]
            self.assertEqual(len(runs), len(base_collection["runs"]) + 6)
            self.assertEqual(len(collection["features"]), len(base_collection["features"]) + 10)
            by_run = {run["run_id"]: run for run in runs}
            order = {run["run_id"]: index for index, run in enumerate(runs)}

            self.assertEqual(by_run["spell-rev-lightning-bg2"]["components"], [80])
            self.assertNotIn(81, by_run["spell-rev-lightning-bg2"]["components"])
            self.assertNotIn(2530, by_run["cdtweaks-bg2"]["components"])
            self.assertLess(order["srcb-rr-compat-bg2"], order["stratagems-bg2"])
            self.assertGreater(order["spell-rev-lightning-bg2"], order["spell-rev-npc-spellbooks-bg2"])
            self.assertLess(order["spell-rev-lightning-bg2"], order["klatu-armor-thieving-bg2"])
            self.assertLess(order["spell-rev-lightning-bg2"], order["buffbot-bg2"])
            self.assertEqual(by_run["chriz-bg-modpack-continuity-bg2"]["components"], [199])
            self.assertEqual(by_run["chriz-bg-modpack-late-companions-bg2"]["components"], [189, 620])
            self.assertLess(order["chriz-bg-modpack-pre-continuity-bg2"], order["chriz-bg-modpack-continuity-bg2"])
            self.assertEqual(order["chriz-bg-modpack-continuity-bg2"] + 1, order["eet-end-bg2"])
            self.assertGreater(order["chriz-bg-modpack-late-companions-bg2"], order["eet-end-bg2"])
            self.assertGreater(order["chriz-bg-rebalance-bg2"], order["stratagems-bg2"])
            self.assertIn(110, by_run["chriz-bg-rebalance-bg2"]["components"])
            self.assertIn(111, by_run["chriz-bg-rebalance-bg2"]["components"])
            sod = by_run["chriz-sod-remix-bg2"]["components"]
            for component in (135, 256, 265):
                self.assertIn(component, sod)
            for component in (176, 235):
                self.assertNotIn(component, sod)
            self.assertLess(sod.index(180), sod.index(175))
            self.assertLess(sod.index(260), sod.index(265))

            selected_additions = {
                "feature:spell-rev:component-80",
                "feature:srcb-rr-compat:component-0",
                "feature:chriz-bg-modpack:component-189",
                "feature:chriz-bg-modpack:component-199",
                "feature:chriz-bg-modpack:component-620",
                "feature:chriz-bg-rebalance:component-110",
                "feature:chriz-bg-rebalance:component-111",
                "mod:safana",
                "feature:safana:mandatory-components",
            }
            self.assertTrue(all(preset.get(feature) == "on" for feature in selected_additions))
            self.assertFalse(any("bristlelick" in key or "aura" in key for key, value in preset.items() if value == "on"))
            self.assertEqual(
                {key for key, value in preset.items() if value == "on" and "safana" in key.lower()},
                {"mod:safana", "feature:safana:mandatory-components"},
            )
            self.assertIn("LOCAL", (output / "LOCAL-ONLY.md").read_text(encoding="utf-8"))
            self.assertFalse((output / "release.json").exists())

            generated_features = {feature["id"]: feature for feature in collection["features"]}
            for original in base_collection["features"]:
                generated = json.loads(json.dumps(generated_features[original["id"]]))
                if original["id"].startswith("feature:chriz-bg-modpack:component-"):
                    for component in generated.get("components", []):
                        if component["run_id"] == "chriz-bg-modpack-pre-continuity-bg2":
                            component["run_id"] = "chriz-bg-modpack-bg2"
                if original["id"] == "feature:chriz-sod-remix:mandatory-components":
                    generated["components"] = [
                        component for component in generated["components"] if component["component"] not in {135, 256, 265}
                    ]
                self.assertEqual(original, generated, original["id"])

            changed_mods = {
                "artisanskitpack.toml", "artisanskitpack-npc.toml", "artisanskitpack-tweak.toml",
                "spell-rev.toml", "chriz-sod-remix.toml", "chriz-bg-modpack.toml",
                "chriz-bg-rebalance.toml", "srcb-rr-compat.toml",
            }
            for path in (base / "mods").glob("*.toml"):
                if path.name not in changed_mods:
                    self.assertEqual(path.read_bytes(), (output / "mods" / path.name).read_bytes())

    def test_reports_missing_frozen_source_instead_of_dropping_it(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp = Path(temp_dir)
            sources_path = self._sources(temp)
            payload = json.loads(sources_path.read_text(encoding="utf-8"))
            payload["sources"] = [source for source in payload["sources"] if source["id"] != "srcb-rr-compat"]
            sources_path.write_text(json.dumps(payload), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "missing required frozen sources.*srcb-rr-compat"):
                build_recipe(self.root, sources_path, temp / "recipe")

    def test_refuses_to_overwrite_any_existing_destination(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp = Path(temp_dir)
            output = temp / "existing"
            output.mkdir()
            sentinel = output / "keep.txt"
            sentinel.write_text("do not delete", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "refusing to overwrite or delete"):
                build_recipe(self.root, self._sources(temp), output)
            self.assertEqual(sentinel.read_text(encoding="utf-8"), "do not delete")


if __name__ == "__main__":
    unittest.main()
