from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re
import shutil
import tempfile
import tomllib
import unittest
import zipfile

from tools.prepare_combined_playtest_recipe import REQUIRED_SOURCE_IDS, SOURCE_MODS, _literal_components, build_recipe


class CombinedPlaytestRecipeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.root = Path(__file__).resolve().parents[2]

    def _sources(self, root: Path, omit: dict[str, set[int]] | None = None) -> Path:
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
                for tp2, mod_id in zip(tp2_paths, SOURCE_MODS[source_id], strict=True):
                    mod = tomllib.loads((self.root / "recipes/curated-full-current/mods" / f"{mod_id}.toml").read_text(encoding="utf-8"))
                    ids = [component["id"] for component in mod["components"] if component["id"] not in (omit or {}).get(source_id, set())]
                    declarations = "".join(f"BEGIN ~component {component}~ DESIGNATED {component}\n" for component in ids)
                    package.writestr(tp2, f"BACKUP ~backup~\nAUTHOR ~test~\n{declarations}")
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
            # Today's recipe must work without any obsolete development archives.
            build_recipe(self.root, None, output)

            self.assertEqual(before, self._tree_hashes(base))
            collection = tomllib.loads((output / "collection.toml").read_text(encoding="utf-8"))
            base_collection = tomllib.loads((base / "collection.toml").read_text(encoding="utf-8"))
            preset = tomllib.loads((output / "presets/chris-recommended.toml").read_text(encoding="utf-8"))["selections"]
            runs = collection["runs"]
            # The public base absorbed this content in the September 16 intake.
            # Neither old archives nor duplicate runs may replace the public selection.
            base_runs = [run["run_id"] for run in base_collection["runs"]]
            run_ids = [run["run_id"] for run in runs]
            self.assertEqual(len(run_ids), len(set(run_ids)))
            self.assertEqual(run_ids[: len(base_runs)], base_runs)
            feature_ids = [feature["id"] for feature in collection["features"]]
            self.assertEqual(len(feature_ids), len(set(feature_ids)))
            for required in (
                "srcb-rr-compat-bg2",
                "chriz-bg-modpack-pre-continuity-bg2",
                "chriz-bg-modpack-continuity-bg2",
                "chriz-bg-modpack-late-companions-bg2",
                "safana-bg2",
                "spell-rev-lightning-bg2",
            ):
                self.assertIn(required, run_ids)
            by_run = {run["run_id"]: run for run in runs}
            order = {run["run_id"]: index for index, run in enumerate(runs)}

            self.assertIn(80, by_run["spell-rev-lightning-bg2"]["components"])
            self.assertEqual(preset.get("feature:spell-rev:component-80"), "on")
            self.assertNotEqual(preset.get("feature:spell-rev:component-81"), "on")
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

            # Curated features are carried over untouched; the tool only adds what is missing.
            generated_features = {feature["id"]: feature for feature in collection["features"]}
            for original in base_collection["features"]:
                self.assertEqual(original, generated_features[original["id"]], original["id"])

            for path in (base / "mods").glob("*.toml"):
                self.assertEqual(path.read_bytes(), (output / "mods" / path.name).read_bytes())
            self.assertEqual(json.loads((output / "combined-playtest-sources.json").read_text())["sources"], [])

    def test_old_snapshots_do_not_replace_delivered_public_components(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp = Path(temp_dir)
            sources = self._sources(temp, {"chriz-sod-remix": {115, 257, 266}, "chriz-bg-modpack": {188}})
            output = temp / "recipe"
            build_recipe(self.root, sources, output)
            base = self.root / "recipes/curated-full-current"
            for name in ("chriz-sod-remix", "chriz-bg-modpack"):
                self.assertEqual((base / "mods" / f"{name}.toml").read_bytes(), (output / "mods" / f"{name}.toml").read_bytes())

    def test_explicit_local_override_rejects_missing_real_declarations(self) -> None:
        for source_id, missing in (("chriz-sod-remix", {115, 257, 266}), ("chriz-bg-modpack", {188})):
            with self.subTest(source=source_id), tempfile.TemporaryDirectory() as temp_dir:
                temp = Path(temp_dir)
                sources = self._sources(temp, {source_id: missing})
                with self.assertRaisesRegex(ValueError, "cannot prove component coverage") as raised:
                    build_recipe(self.root, sources, temp / "recipe", local_sources={source_id})
                for component in missing:
                    self.assertIn(str(component), str(raised.exception))
                self.assertFalse((temp / "recipe").exists())

    def test_explicit_compatible_local_overrides_are_checked_and_pinned(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp = Path(temp_dir)
            output = temp / "recipe"
            build_recipe(self.root, self._sources(temp), output, local_sources=set(REQUIRED_SOURCE_IDS))
            for mod_ids in SOURCE_MODS.values():
                for mod_id in mod_ids:
                    mod = tomllib.loads((output / "mods" / f"{mod_id}.toml").read_text(encoding="utf-8"))
                    self.assertTrue(mod["artifact_id"].startswith("local-playtest-"))

    def test_missing_safana_and_compat_mods_reach_their_fallbacks(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp = Path(temp_dir)
            old_root = temp / "old-base"
            old_base = old_root / "recipes/curated-full-current"
            shutil.copytree(self.root / "recipes/curated-full-current", old_base)
            for name in ("safana", "srcb-rr-compat"):
                (old_base / "mods" / f"{name}.toml").unlink()
            output = temp / "recipe"
            build_recipe(old_root, self._sources(temp), output)
            for name in ("safana", "srcb-rr-compat"):
                mod = tomllib.loads((output / "mods" / f"{name}.toml").read_text(encoding="utf-8"))
                self.assertTrue(mod["artifact_id"].startswith("local-playtest-"))
                self.assertEqual(mod["components"][0]["id"], 0)
            self.assertFalse((old_base / "mods/safana.toml").exists())

    def test_missing_catalogue_component_requires_a_compatible_local_source(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp = Path(temp_dir)
            old_root = temp / "old-base"
            old_base = old_root / "recipes/curated-full-current"
            shutil.copytree(self.root / "recipes/curated-full-current", old_base)
            mod_path = old_base / "mods/chriz-sod-remix.toml"
            original = mod_path.read_text(encoding="utf-8")
            reduced, count = re.subn(r"(?ms)^\[\[components\]\]\nid = 265\n.*?(?=^\[\[components\]\]|\Z)", "", original)
            self.assertEqual(count, 1)
            mod_path.write_text(reduced, encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "frozen sources are required for chriz-sod-remix"):
                build_recipe(old_root, None, temp / "missing-source")
            self.assertFalse((temp / "missing-source").exists())
            output = temp / "recipe"
            build_recipe(old_root, self._sources(temp), output)
            mod = tomllib.loads((output / "mods/chriz-sod-remix.toml").read_text(encoding="utf-8"))
            self.assertTrue(mod["artifact_id"].startswith("local-playtest-chriz-sod-remix-"))
            self.assertIn(265, [component["id"] for component in mod["components"]])

    def test_component_coverage_ignores_comment_and_string_examples(self) -> None:
        text = '''// BEGIN ~fake~ DESIGNATED 115
/* nested /* comment */ BEGIN ~fake~ DESIGNATED 266 */
BEGIN ~real~ DESIGNATED 100
PRINT ~BEGIN bogus DESIGNATED 188~
BEGIN @10 DESIGNATED 110
'''
        self.assertEqual(_literal_components(text), {100, 110})
        self.assertEqual(_literal_components('BEGIN ~core~\nPRINT ~ok~'), {0})
        self.assertEqual(_literal_components('BEGIN ~dynamic~ DESIGNATED %component%'), set())

    def test_reports_missing_frozen_source_instead_of_dropping_it(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp = Path(temp_dir)
            sources_path = self._sources(temp)
            payload = json.loads(sources_path.read_text(encoding="utf-8"))
            payload["sources"] = [source for source in payload["sources"] if source["id"] != "srcb-rr-compat"]
            sources_path.write_text(json.dumps(payload), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "missing required frozen sources.*srcb-rr-compat"):
                build_recipe(self.root, sources_path, temp / "recipe", local_sources={"srcb-rr-compat"})

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
