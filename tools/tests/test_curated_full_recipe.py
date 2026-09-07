from __future__ import annotations

import hashlib
import json
import tempfile
import tomllib
import unittest
from pathlib import Path

from tools.curated_full_recipe import _effective_features, _preserve_native_run_orders, build_recipe


class CuratedFullRecipeTests(unittest.TestCase):
    def setUp(self) -> None:
        self.root = Path(__file__).resolve().parents[2]

    def test_evandra_uses_official_windows_archive_without_private_aggregate(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output = Path(temp_dir) / "recipe"
            build_recipe(self.root, output, "d6d46647b24b1a4baa501bca8c1d23048da3e83f")
            mod = tomllib.loads((output / "mods/evandra.toml").read_text(encoding="utf-8"))
            self.assertEqual(mod["artifact_id"], "evandra-2.2-windows")
            self.assertFalse((output / "artifacts/creator-full-private-extras-20260902.toml").exists())
            artifact = tomllib.loads((output / "artifacts/evandra-2.2-windows.toml").read_text(encoding="utf-8"))
            self.assertEqual(artifact["source"]["expected_filename"], "evandra-v2.2.exe")
            self.assertEqual(artifact["source"]["expected_length"], 13430253)
            self.assertEqual(artifact["source"]["sha256"], "21724b6d4679d6df6dbcf95a0a4dbe6ee41d5bfdb3ae13907ec89f5d00014861")
            self.assertEqual(artifact["archive"]["kind"], "self-extracting-rar")
            self.assertEqual(artifact["archive"]["publish_roots"], ["evandra"])

    def test_skipping_evandra_omits_core_and_crossmod_but_keeps_other_defaults(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output = Path(temp_dir) / "recipe"
            build_recipe(self.root, output, "d6d46647b24b1a4baa501bca8c1d23048da3e83f")
            collection = tomllib.loads((output / "collection.toml").read_text(encoding="utf-8"))
            baseline = tomllib.loads((output / "presets/chris-recommended.toml").read_text(encoding="utf-8"))["selections"]
            defaults = _effective_features(collection, baseline)
            self.assertTrue(defaults["feature:evandra:mandatory-components"])
            self.assertTrue(defaults["feature:evandra:component-1"])
            skipped = _effective_features(collection, {**baseline, "mod:evandra": "off"})
            changes = {key for key in defaults if defaults[key] != skipped[key]}
            self.assertEqual(changes, {"mod:evandra", "feature:evandra:mandatory-components", "feature:evandra:component-1"})
            self.assertFalse(skipped["feature:evandra:mandatory-components"])
            self.assertFalse(skipped["feature:evandra:component-1"])

    def test_new_offered_component_requires_native_order_audit(self) -> None:
        collection = (self.root / "manifest/collection.toml").read_text(encoding="utf-8")
        collection = collection.replace("components = [1, 2, 20000", "components = [999999, 1, 2, 20000", 1)
        with self.assertRaisesRegex(ValueError, "requires a component-order audit"):
            _preserve_native_run_orders(collection)

    def test_build_is_curation_derived_and_preserves_required_regressions(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output = Path(temp_dir) / "recipe"
            (output / "artifacts").mkdir(parents=True)
            for stale in [
                "chriz-bg-modpack-0.2.0-alpha.1",
                "bardicwonders-v2.9c-balance.2",
                "chriz-sod-remix-0.6.5",
                "chriz-sod-remix-0.6.6",
                "creator-full-private-extras-20260902",
            ]:
                (output / "artifacts" / f"{stale}.toml").write_text(f'id = "{stale}"\n', encoding="utf-8")
            build_recipe(self.root, output, "d6d46647b24b1a4baa501bca8c1d23048da3e83f")
            self.assertFalse((output / "artifacts/chriz-bg-modpack-0.2.0-alpha.1.toml").exists())
            modpack = tomllib.loads((output / "mods/chriz-bg-modpack.toml").read_text(encoding="utf-8"))
            self.assertEqual(modpack["artifact_id"], "chriz-bg-modpack-0.2.0-alpha.5")
            collection = tomllib.loads((output / "collection.toml").read_text(encoding="utf-8"))
            preset = tomllib.loads((output / "presets/chris-recommended.toml").read_text(encoding="utf-8"))
            mods = {path.stem for path in (output / "mods").glob("*.toml")}
            runs = {run["run_id"]: run for run in collection["runs"]}
            features = {feature["id"]: feature for feature in collection["features"]}

            semantic_recipe = {
                "runs": [
                    (
                        run["run_id"],
                        run["mod_id"],
                        run["phase"],
                        run["components"],
                        run.get("args", []),
                    )
                    for run in collection["runs"]
                ],
                "features": [
                    (
                        feature["id"],
                        feature["decision"],
                        feature["readiness"],
                        None
                        if feature["id"]
                        in {
                            "feature:bg1npc:component-110",
                            "feature:bg1npc:component-112",
                            "feature:bg1npc:component-113",
                            "feature:bg1npc:component-114",
                            "feature:bg1npc:component-131",
                            "feature:bg1npc:component-241",
                        }
                        else feature.get("parent"),
                        feature.get("requires", []),
                        []
                        if feature["id"]
                        in {
                            "feature:bg1npc:component-111",
                            "feature:bg1npc:component-130",
                            "feature:bg1npc:component-240",
                        }
                        else feature.get("conflicts", []),
                        feature.get("components", []),
                    )
                    for feature in collection["features"]
                ],
                "preset": preset["selections"],
            }
            semantic_digest = hashlib.sha256(
                json.dumps(semantic_recipe, sort_keys=True).encode()
            ).hexdigest()
            self.assertEqual(
                semantic_digest,
                "4320bd5a5a766e1acc1667d8979e1c33816baa0f0e110a1dabdc4a53d5cc8415",
            )

            legacy_bg1npc_groups = {
                (110, 111, 112, 113, 114): 111,
                (240, 241): 240,
                (130, 131): 130,
            }
            baseline = preset["selections"]
            baseline_effective = _effective_features(collection, baseline)
            for components, default_component in legacy_bg1npc_groups.items():
                ids = [f"feature:bg1npc:component-{component}" for component in components]
                self.assertTrue(all(features[feature_id]["parent"] == "mod:bg1npc" for feature_id in ids))
                self.assertTrue(all(features[feature_id]["category"] == "bg1-npcs" for feature_id in ids))
                for feature_id in ids:
                    others = set(ids) - {feature_id}
                    self.assertEqual(
                        {conflict["feature_id"] for conflict in features[feature_id]["conflicts"]},
                        others,
                    )
                default_id = f"feature:bg1npc:component-{default_component}"
                self.assertTrue(baseline_effective[default_id])
                alternate_id = next(feature_id for feature_id in ids if feature_id != default_id)
                switched = {**baseline, default_id: "off", alternate_id: "on"}
                switched_effective = _effective_features(collection, switched)
                changed = {
                    feature_id
                    for feature_id in baseline_effective
                    if baseline_effective[feature_id] != switched_effective[feature_id]
                }
                self.assertEqual(changed, {default_id, alternate_id})
                self.assertFalse(switched_effective[default_id])
                self.assertTrue(switched_effective[alternate_id])
                baseline_components = {
                    (component["run_id"], component["component"])
                    for feature_id, selected in baseline_effective.items()
                    if selected
                    for component in features[feature_id].get("components", [])
                }
                switched_components = {
                    (component["run_id"], component["component"])
                    for feature_id, selected in switched_effective.items()
                    if selected
                    for component in features[feature_id].get("components", [])
                }
                alternate_component = int(alternate_id.rsplit("-", 1)[1])
                self.assertEqual(
                    baseline_components ^ switched_components,
                    {
                        ("bg1npc-bg1", default_component),
                        ("bg1npc-bg1", alternate_component),
                    },
                )

            self.assertTrue(all(feature.get("source_label") for feature in features.values()))
            self.assertTrue(all(feature.get("group_label") for feature in features.values()))
            potions = features["feature:cdtweaks:component-1142"]
            self.assertEqual(potions["title"], "Potions require identification")
            self.assertEqual(
                potions["description"],
                "Potions need identification; gems are unchanged.",
            )
            self.assertEqual(potions["source_label"], "The Tweaks Anthology")
            self.assertEqual(potions["group_label"], "Gems and Potions Require Identification")
            self.assertNotIn("choice_group", potions)
            strongholds = features["feature:cdtweaks:component-1160"]
            self.assertEqual(strongholds["title"], "No restrictions")
            self.assertEqual(
                strongholds["description"],
                "Allows multiple strongholds without class restrictions.",
            )
            self.assertEqual(
                strongholds["choice_group"],
                "choice:cdtweaks:multiple-strongholds-sabre-baldurdash-weimer",
            )
            potion_stacks = [
                features[f"feature:cdtweaks:component-{component}"]
                for component in range(3100, 3104)
            ]
            self.assertEqual(
                {feature["choice_group"] for feature in potion_stacks},
                {"choice:cdtweaks:increase-potion-stacking"},
            )
            self.assertTrue(
                all(feature["group_label"] == "Increase Potion Stacking" for feature in potion_stacks)
            )
            self.assertEqual(
                potion_stacks[2]["description"],
                "Allows up to 80 potions per inventory slot.",
            )
            self.assertNotIn("choice_group", features["feature:cdtweaks:component-2090"])

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
            artisan_components = runs["artisanskitpack-main-bg2"]["components"]
            self.assertLess(artisan_components.index(10001), artisan_components.index(10002))
            self.assertLess(artisan_components.index(1100), artisan_components.index(1003))
            randomiser_components = runs["randomiser-bg2"]["components"]
            self.assertLess(randomiser_components.index(1100), randomiser_components.index(9000))
            randomiser = tomllib.loads((output / "mods/randomiser.toml").read_text(encoding="utf-8"))
            compatibility_prompt = next(component for component in randomiser["components"] if component["id"] == 1100)["prompts"][0]
            self.assertEqual(compatibility_prompt["when_any_features"], ["feature:xan:mandatory-components", "feature:rr:component-12"])
            self.assertEqual(compatibility_prompt["answer"]["value"], {"kind": "choice", "value": "y"})
            self.assertIn("leave these items where they are", compatibility_prompt["expected_output"])
            sod_mod = tomllib.loads((output / "mods/chriz-sod-remix.toml").read_text(encoding="utf-8"))
            self.assertEqual(sod_mod["artifact_id"], "chriz-sod-remix-0.6.8")
            # Setup-name WeiDU resolves the nested copy when both identical TP2s
            # are shipped. The frozen expected log identity must use that path.
            self.assertEqual(sod_mod["tp2"], "chriz-sod-remix/setup-chriz-sod-remix.tp2")
            sod_artifact = tomllib.loads(
                (output / "artifacts/chriz-sod-remix-0.6.8.toml").read_text(encoding="utf-8")
            )
            self.assertFalse((output / "artifacts/chriz-sod-remix-0.6.4.toml").exists())
            self.assertFalse((output / "artifacts/chriz-sod-remix-0.6.5.toml").exists())
            self.assertFalse((output / "artifacts/chriz-sod-remix-0.6.6.toml").exists())
            self.assertFalse((output / "artifacts/chriz-sod-remix-0.6.7.toml").exists())
            self.assertEqual(sod_artifact["version"], "0.6.8")
            self.assertEqual(sod_artifact["source"]["reference"], "v0.6.8")
            self.assertEqual(
                sod_artifact["source"]["expected_filename"],
                "chriz-sod-remix-v0.6.8.zip",
            )
            self.assertEqual(sod_artifact["source"]["expected_length"], 1513381)
            self.assertEqual(
                sod_artifact["source"]["sha256"],
                "29eb10537ebf759608da93b8764acfc678cd301bcef24e14cc860db01b33efbb",
            )
            self.assertEqual(runs["chriz-sod-remix-bg2"]["components"], [100, 110, 120, 130, 140, 150, 145, 160, 170, 180, 175, 185, 190, 195, 210, 197, 187, 200, 215, 220, 225, 245, 230, 240, 250, 255, 260, 270, 280, 290, 900, 910])
            self.assertNotIn(291, runs["chriz-sod-remix-bg2"]["components"])
            self.assertLess(runs["chriz-sod-remix-bg2"]["components"].index(210), runs["chriz-sod-remix-bg2"]["components"].index(197))
            self.assertEqual(runs["bardicwonders-garrick-bg2"]["components"], [1008])
            bardic = tomllib.loads((output / "mods/bardicwonders.toml").read_text(encoding="utf-8"))
            self.assertEqual(bardic["artifact_id"], "bardicwonders-v2.9c-balance.3")
            bardic_artifact = tomllib.loads(
                (output / "artifacts/bardicwonders-v2.9c-balance.3.toml").read_text(
                    encoding="utf-8"
                )
            )
            self.assertFalse(
                (output / "artifacts/bardicwonders-v2.9c-balance.2.toml").exists()
            )
            self.assertEqual(bardic_artifact["version"], "2.9c-balance.3")
            self.assertEqual(bardic_artifact["source"]["reference"], "v2.9c-balance.3")
            self.assertEqual(
                bardic_artifact["source"]["url"],
                "https://codeload.github.com/Chrizhermann/"
                "Bardic-Wonders-Chriz-Balance-Patch/zip/refs/tags/v2.9c-balance.3",
            )
            self.assertEqual(
                bardic_artifact["source"]["expected_filename"],
                "Bardic-Wonders-Chriz-Balance-Patch-2.9c-balance.3.zip",
            )
            self.assertEqual(bardic_artifact["source"]["expected_length"], 5282818)
            self.assertEqual(
                bardic_artifact["source"]["sha256"],
                "3cee2244562e048c1f1b466520ed0f34da4078f6e78fe42ebf8a562c8d063cb0",
            )
            self.assertEqual(bardic_artifact["archive"]["root_rule"], "single-wrapper")
            self.assertEqual(bardic_artifact["archive"]["publish_roots"], ["BardicWonders"])
            self.assertEqual(
                bardic_artifact["archive"]["tp2_paths"],
                ["BardicWonders/Setup-BardicWonders.tp2"],
            )
            # The release fixes already-selected Abettor content; it must not
            # alter the curated defaults or their pinned TP2 declaration order.
            self.assertEqual(
                runs["bardicwonders-bg2"]["components"],
                [
                    1001, 1002, 1003, 1004, 1005, 1006, 1007, 1009, 1010,
                    1011, 2002, 2007, 2008, 2003, 2004, 2005, 2006,
                ],
            )
            bardic_main = runs["bardicwonders-bg2"]["components"]
            self.assertLess(bardic_main.index(1004), bardic_main.index(2007))
            self.assertLess(bardic_main.index(2007), bardic_main.index(2004))
            evandra_artifact = tomllib.loads(
                (output / "artifacts/evandra-2.2-windows.toml").read_text(encoding="utf-8")
            )
            self.assertFalse((output / "artifacts/creator-full-private-extras-20260902.toml").exists())
            self.assertEqual(evandra_artifact["archive"]["publish_roots"], ["evandra"])
            self.assertEqual(evandra_artifact["archive"]["tp2_paths"], ["evandra/setup-evandra.tp2"])
            prompt = next(component for component in bardic["components"] if component["id"] == 1008)["prompts"][0]
            self.assertEqual(prompt["answer"]["value"], {"kind": "integer", "value": 2})
            self.assertEqual(features["feature:chriz-bg-modpack:component-400"]["requires"], ["feature:branwen:component-0", "mod:spell-rev"])
            self.assertEqual(features["feature:chriz-bg-modpack:component-430"]["requires"], ["feature:cdtweaks:component-2170"])
            self.assertEqual(
                features["feature:chriz-bg-modpack:component-610"]["requires"],
                ["feature:eeex:mandatory-components", "mod:eet-end"],
            )
            self.assertEqual(features["feature:chriz-bg-modpack:component-220"]["decision"], "default")
            hexxat = [features[f"feature:chriz-bg-modpack:component-{component}"] for component in (221, 222, 223)]
            self.assertEqual([feature["decision"] for feature in hexxat], ["default", "optional", "optional"])
            self.assertEqual(
                [[conflict["feature_id"] for conflict in feature["conflicts"]] for feature in hexxat],
                [
                    ["feature:chriz-bg-modpack:component-222", "feature:chriz-bg-modpack:component-223", "feature:artisanskitpack-npc:component-7104"],
                    ["feature:chriz-bg-modpack:component-221", "feature:chriz-bg-modpack:component-223", "feature:artisanskitpack-npc:component-7104"],
                    ["feature:chriz-bg-modpack:component-221", "feature:chriz-bg-modpack:component-222", "feature:artisanskitpack-npc:component-7104"],
                ],
            )
            artisan_hexxat = features["feature:artisanskitpack-npc:component-7104"]
            self.assertEqual(artisan_hexxat["decision"], "optional")
            self.assertEqual(
                [conflict["feature_id"] for conflict in artisan_hexxat["conflicts"]],
                [f"feature:chriz-bg-modpack:component-{component}" for component in (221, 222, 223)],
            )
            conflict_reasons = {
                conflict["feature_id"]: conflict["reason"]
                for feature in [*hexxat, artisan_hexxat]
                for conflict in feature["conflicts"]
            }
            self.assertIn("Shadowdancer", conflict_reasons["feature:chriz-bg-modpack:component-221"])
            self.assertIn("Fighter/Thief", conflict_reasons["feature:chriz-bg-modpack:component-222"])
            self.assertIn("Assassin", conflict_reasons["feature:chriz-bg-modpack:component-223"])
            self.assertIn("Invisible Blade", conflict_reasons["feature:artisanskitpack-npc:component-7104"])
            self.assertNotIn("feature:artisanskitpack-npc:component-7104", preset["selections"])
            for component in (220, 221, 610):
                self.assertEqual(
                    preset["selections"][f"feature:chriz-bg-modpack:component-{component}"],
                    "on",
                )
            for component in (222, 223):
                self.assertNotIn(
                    f"feature:chriz-bg-modpack:component-{component}",
                    preset["selections"],
                )
            self.assertEqual(features["mod:bardicwonders"]["decision"], "default")
            bardic_features = [
                feature
                for feature in collection["features"]
                if feature["id"].startswith("feature:bardicwonders:")
            ]
            self.assertEqual(len(bardic_features), 20)
            self.assertTrue(
                all(feature["parent"] == "mod:bardicwonders" for feature in bardic_features)
            )
            darkbloom = features["feature:bardicwonders:component-1006"]
            self.assertEqual(darkbloom["readiness"], "ready")
            self.assertEqual(
                [conflict["feature_id"] for conflict in darkbloom["conflicts"]],
                ["feature:spell-rev:mandatory-components"],
            )
            self.assertEqual(features["feature:iwdification:mandatory-components"]["components"], [{"run_id": "iwdification-bg2", "component": 30}, {"run_id": "iwdification-bg2", "component": 40}])
            self.assertNotIn(120, runs["iwdification-bg2"]["components"])
            self.assertIn(2720, runs["cdtweaks-bg2"]["components"])
            self.assertIn(3121, runs["cdtweaks-bg2"]["components"])
            self.assertEqual(runs["cdtweaks-spell-save-penalties-bg2"]["components"], [2312])
            order = list(runs)
            self.assertLess(order.index("stratagems-bg2"), order.index("randomiser-bg2"))
            self.assertLess(order.index("randomiser-bg2"), order.index("eet-end-bg2"))
            self.assertLess(order.index("artisanskitpack-main-bg2"), order.index("bardicwonders-garrick-bg2"))
            self.assertLess(order.index("bardicwonders-garrick-bg2"), order.index("artisanskitpack-npc-bg2"))
            self.assertLess(order.index("evandra-core-bg2"), order.index("xan-bg2"))
            self.assertLess(order.index("xan-bg2"), order.index("evandra-crossmod-bg2"))
            self.assertLess(order.index("evandra-crossmod-bg2"), order.index("crossmodbg2-bg2"))
            self.assertLess(order.index("chriz-bg-modpack-bg2"), order.index("cdtweaks-spell-save-penalties-bg2"))
            self.assertLess(order.index("cdtweaks-spell-save-penalties-bg2"), order.index("spell-rev-npc-spellbooks-bg2"))
            self.assertEqual(preset["selections"]["feature:evandra:component-1"], "on")
            bg_mod = tomllib.loads((output / "mods/chriz-bg-rebalance.toml").read_text(encoding="utf-8"))
            self.assertEqual(bg_mod["artifact_id"], "chriz-bg-rebalance-0.3.2")
            bg_artifact = tomllib.loads(
                (output / "artifacts/chriz-bg-rebalance-0.3.2.toml").read_text(encoding="utf-8")
            )
            self.assertFalse((output / "artifacts/chriz-bg-rebalance-0.3.1.toml").exists())
            self.assertEqual(bg_artifact["version"], "0.3.2")
            self.assertEqual(bg_artifact["source"]["reference"], "v0.3.2")
            self.assertEqual(bg_artifact["source"]["expected_length"], 1369825)
            self.assertEqual(
                bg_artifact["source"]["sha256"],
                "25480a8e597d316d3cf1799f641971f3b6edb113eea24da7f45a8dd70b0a9ef4",
            )
            self.assertEqual(json.loads((output / "release.json").read_text())["version"], "0.1.0-alpha.13")
            ledger = tomllib.loads((output / "releases/v0.1.0-alpha.13/ledger.toml").read_text())
            self.assertEqual(ledger["version"], "0.1.0-alpha.13")
            self.assertEqual(ledger["minimum_app_version"], "0.1.0-alpha.15")
            self.assertFalse((output / "reference").exists())

    def test_common_customization_routes_preserve_dependency_collateral(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output = Path(temp_dir) / "recipe"
            build_recipe(self.root, output, "d6d46647b24b1a4baa501bca8c1d23048da3e83f")
            collection = tomllib.loads((output / "collection.toml").read_text(encoding="utf-8"))
            baseline = tomllib.loads(
                (output / "presets/chris-recommended.toml").read_text(encoding="utf-8")
            )["selections"]

            artisan_off = baseline.copy()
            for feature_id in (
                "mod:artisanskitpack",
                "mod:artisanskitpack-npc",
                "mod:artisanskitpack-tweak",
            ):
                artisan_off[feature_id] = "off"
            effective = _effective_features(collection, artisan_off)
            self.assertFalse(
                any(
                    selected
                    for feature_id, selected in effective.items()
                    if feature_id.startswith("feature:artisanskitpack")
                )
            )
            self.assertFalse(effective["feature:chriz-bg-modpack:component-140"])
            self.assertFalse(effective["feature:chriz-bg-modpack:component-170"])

            bardic_off = baseline.copy()
            bardic_off["mod:bardicwonders"] = "off"
            effective = _effective_features(collection, bardic_off)
            self.assertFalse(
                any(
                    selected
                    for feature_id, selected in effective.items()
                    if feature_id.startswith("feature:bardicwonders:")
                )
            )
            self.assertFalse(effective["feature:artisanskitpack-npc:component-99001"])

            spell_revisions_off = baseline.copy()
            spell_revisions_off["mod:spell-rev"] = "off"
            effective = _effective_features(collection, spell_revisions_off)
            self.assertFalse(
                any(
                    selected
                    for feature_id, selected in effective.items()
                    if feature_id.startswith("feature:spell-rev:")
                )
            )
            self.assertFalse(effective["feature:chriz-bg-modpack:component-400"])
            self.assertTrue(effective["feature:bardicwonders:component-1006"])
            self.assertTrue(effective["feature:artisanskitpack:component-8101"])

            original_classes = baseline.copy()
            original_classes["feature:yeslicknpc:component-0"] = "on"
            original_classes["feature:yeslicknpc:component-1"] = "off"
            artisan_npc_components = (
                1101,
                2001,
                3101,
                3102,
                5101,
                5102,
                7101,
                7102,
                7104,
                21001,
                9101,
                10004,
                20002,
                99001,
                200010,
            )
            for component in artisan_npc_components:
                original_classes[f"feature:artisanskitpack-npc:component-{component}"] = "off"
            for component in (110, 190, 192, 193, 194, 195, 196, 198, 220, 221, 222, 223):
                original_classes[f"feature:chriz-bg-modpack:component-{component}"] = "off"
            for component in (1, 2, 3, 4):
                original_classes[f"feature:xan:component-{component}"] = "off"
            effective = _effective_features(collection, original_classes)
            self.assertTrue(effective["feature:yeslicknpc:component-0"])
            self.assertFalse(effective["feature:yeslicknpc:component-1"])
            self.assertFalse(effective["feature:chriz-bg-modpack:component-140"])
            self.assertFalse(effective["feature:chriz-bg-modpack:component-170"])
            self.assertFalse(effective["feature:chriz-bg-modpack:component-220"])
            self.assertFalse(effective["feature:chriz-bg-modpack:component-221"])
            self.assertTrue(effective["feature:chriz-bg-modpack:component-197"])
            for component in (130, 430, 440, 450):
                self.assertTrue(effective[f"feature:chriz-bg-modpack:component-{component}"])

    def test_reconciliation_covers_every_default_and_mandatory_row(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output = Path(temp_dir) / "recipe"
            build_recipe(self.root, output, "d6d46647b24b1a4baa501bca8c1d23048da3e83f")
            report = (output / "RECONCILIATION.md").read_text(encoding="utf-8")
            self.assertIn("Engine-equivalent resolved-selection summary", report)
            self.assertIn("444 default/mandatory rows", report)
            self.assertIn("`IWDIFICATION:120` | default", report)
            self.assertIn("`BARDICWONDERS:1006` | default", report)
            self.assertIn("`CHRIZ-BG-MODPACK:400` | mandatory", report)
            self.assertIn("`CHRIZ-BG-MODPACK:610` | default", report)
            self.assertIn("`CHRIZ-SOD-REMIX:290` | mandatory", report)
            self.assertIn("`CHRIZ-SOD-REMIX:910` | mandatory", report)


if __name__ == "__main__":
    unittest.main()
