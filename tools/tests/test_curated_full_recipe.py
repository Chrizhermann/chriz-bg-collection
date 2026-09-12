from __future__ import annotations

import hashlib
import json
import tempfile
import tomllib
import unittest
from pathlib import Path

from tools.curated_full_recipe import _effective_features, _preserve_native_run_orders, build_recipe
from tools.public_component_credits import build_public_component_credits


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

    def test_acton_balthis_is_unchecked_optional_without_collateral_changes(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output = Path(temp_dir) / "recipe"
            build_recipe(self.root, output, "d6d46647b24b1a4baa501bca8c1d23048da3e83f")
            collection = tomllib.loads((output / "collection.toml").read_text(encoding="utf-8"))
            preset = tomllib.loads((output / "presets/chris-recommended.toml").read_text())["selections"]
            features = {feature["id"]: feature for feature in collection["features"]}
            feature_id = "feature:ub:component-25"
            feature = features[feature_id]
            self.assertEqual(feature["decision"], "optional")
            self.assertEqual(feature["readiness"], "ready")
            self.assertEqual(feature["components"], [{"run_id": "ub-bg2", "component": 25}])
            defaults = _effective_features(collection, preset)
            self.assertFalse(defaults[feature_id])
            enabled = _effective_features(collection, {**preset, feature_id: "on"})
            self.assertEqual({key for key in defaults if defaults[key] != enabled[key]}, {feature_id})
            self.assertTrue(defaults["feature:cdtweaks:component-2380"])
            self.assertTrue(defaults["feature:ub:component-21"])

    def test_racial_kit_unlock_is_default_optional_and_follows_kit_additions(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output = Path(temp_dir) / "recipe"
            build_recipe(self.root, output, "d6d46647b24b1a4baa501bca8c1d23048da3e83f")
            collection = tomllib.loads((output / "collection.toml").read_text(encoding="utf-8"))
            preset = tomllib.loads((output / "presets/chris-recommended.toml").read_text())["selections"]
            features = {feature["id"]: feature for feature in collection["features"]}
            feature_id = "feature:cdtweaks:component-2380"
            self.assertTrue(feature_id in features, f"Missing default: {feature_id}")
            feature = features[feature_id]
            self.assertEqual(feature["decision"], "default")
            self.assertEqual(feature["readiness"], "ready")
            self.assertEqual(feature["components"], [{"run_id": "cdtweaks-bg2", "component": 2380}])
            self.assertIn("gnome", feature["description"].lower())
            defaults = _effective_features(collection, preset)
            self.assertTrue(defaults[feature_id])
            disabled = _effective_features(collection, {**preset, feature_id: "off"})
            self.assertEqual({key for key in defaults if defaults[key] != disabled[key]}, {feature_id})
            for other in ("mod:artisanskitpack", "mod:spell-rev", "mod:bardicwonders"):
                self.assertTrue(_effective_features(collection, {**preset, other: "off"})[feature_id])
            runs = collection["runs"]
            positions = {run["run_id"]: index for index, run in enumerate(runs)}
            # Later Bardic 1012 supplies Gallant with all-race availability already.
            for run_id in ("artisanskitpack-main-bg2", "artisanskitpack-npc-bg2",
                           "bardicwonders-bg2", "bardicwonders-garrick-bg2"):
                self.assertLess(positions[run_id], positions["cdtweaks-bg2"])
            self.assertEqual(sum(run["components"].count(2380) for run in runs if run["mod_id"] == "cdtweaks"), 1)
            for component in (2550, 2551, 2552):
                self.assertNotIn(f"feature:cdtweaks:component-{component}", features)

    def test_screened_optional_batch_has_real_exclusivity_and_keeps_defaults(self) -> None:
        additions = {
            "stratagems": [3015, 4020, 4050, 4051, 4052, 4093, 4145, 4150,
                           4160, 4161, 4162, 4163, 4164, 4170, 4171, 4172,
                           4173, 4174, 4216, 4217, 4230],
            "cdtweaks": [70, 90, 140, 150, 160, 171, 220, 240, 241, 1030,
                         1035, 1036, 1100, 1101, 1140, 1141, 2151, 2190,
                         2191, 2220, 2310, 2311, 3030, 3031, 3060, 3070,
                         3071, 3072, 3073, 3131, 3132, 3150, 3151, 3191,
                         3194, 3195, 3196, 3197, 3198, 3200, 3205, 3230,
                         3320, 4140],
        }
        groups = {
            "stratagems": [(4050, 4051, 4052, 4093), tuple(range(4160, 4165)),
                           tuple(range(4170, 4175)), (4216, 4217, 4218)],
            "cdtweaks": [(170, 171), (240, 241), (1035, 1036), (1100, 1101),
                         (1140, 1141, 1142), (2151, 2152), (2190, 2191, 2192),
                         (2310, 2311, 2312), (3030, 3031), tuple(range(3070, 3074)),
                         (3130, 3131, 3132), (200, 3150, 3151), tuple(range(3194, 3199))],
        }
        with tempfile.TemporaryDirectory() as temp_dir:
            output = Path(temp_dir) / "recipe"
            build_recipe(self.root, output, "d6d46647b24b1a4baa501bca8c1d23048da3e83f")
            collection = tomllib.loads((output / "collection.toml").read_text(encoding="utf-8"))
            preset = tomllib.loads((output / "presets/chris-recommended.toml").read_text())["selections"]
            features = {feature["id"]: feature for feature in collection["features"]}
            effective = _effective_features(collection, preset)
            runs = {run["run_id"]: run for run in collection["runs"]}
            declared = {
                mod: {entry["id"] for entry in tomllib.loads((output / f"mods/{mod}.toml").read_text(encoding="utf-8"))["components"]}
                for mod in additions
            }
            for mod, components in additions.items():
                for component in components:
                    feature_id = f"feature:{mod}:component-{component}"
                    self.assertTrue(feature_id in features, f"Missing offered option: {feature_id}")
                    self.assertEqual(features[feature_id]["decision"], "optional")
                    self.assertFalse(effective[feature_id], feature_id)
                    for entry in features[feature_id]["components"]:
                        self.assertIn(entry["component"], runs[entry["run_id"]]["components"], feature_id)
                        self.assertIn(entry["component"], declared[mod], feature_id)
            for mod, choices in groups.items():
                for components in choices:
                    ids = {f"feature:{mod}:component-{component}" for component in components}
                    self.assertEqual(len({features[key]["choice_group"] for key in ids}), 1)
                    self.assertEqual(len({(features[key]["category"], features[key].get("parent"),
                                           features[key]["group_label"]) for key in ids}), 1)
                    for key in ids:
                        edges = {edge["feature_id"] for edge in features[key].get("conflicts", [])}
                        self.assertTrue(ids - {key} <= edges, f"Incomplete exclusivity: {key}")
                    # Every alternative can be selected after turning its siblings off.
                    for chosen in ids:
                        selection = {**preset, **{key: "off" for key in ids}, chosen: "on"}
                        switched = _effective_features(collection, selection)
                        self.assertEqual({key for key in ids if switched[key]}, {chosen})
            graphics = "feature:cdtweaks:component-70"
            iwd_graphics = "feature:iwdification:component-10"
            self.assertIn(iwd_graphics, {edge["feature_id"] for edge in features[graphics]["conflicts"]})
            self.assertFalse(_effective_features(collection, {**preset, graphics: "on"})[graphics])
            self.assertTrue(_effective_features(collection, {**preset, graphics: "on", iwd_graphics: "off"})[graphics])
            runs = {run["run_id"]: run for run in collection["runs"]}
            self.assertEqual(runs["cdtweaks-spell-save-penalties-bg2"]["components"], [2310, 2311, 2312])
            self.assertTrue({2310, 2311, 2312}.isdisjoint(runs["cdtweaks-bg2"]["components"]))
            for component in (2310, 2311, 2312):
                self.assertEqual(features[f"feature:cdtweaks:component-{component}"]["components"],
                                 [{"run_id": "cdtweaks-spell-save-penalties-bg2", "component": component}])
            for mod, excluded in {"stratagems": [3017, 3551, 3552, 4030, 4100],
                                  "cdtweaks": [72, 2100, 2150, 260, 2680, 3280, 3420]}.items():
                for component in excluded:
                    self.assertNotIn(f"feature:{mod}:component-{component}", set(features))

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
                "27f91688b3ce5132fc8ad76614e4d2d7a7ad76d2a58775c27b4ed0bb89e5cb59",
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
            self.assertEqual(potions["choice_group"], "choice:cdtweaks:gems-and-potions-require-identification")
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
            self.assertEqual(bardic["artifact_id"], "bardicwonders-v2.9c-balance.4")
            bardic_artifact = tomllib.loads(
                (output / "artifacts/bardicwonders-v2.9c-balance.4.toml").read_text(
                    encoding="utf-8"
                )
            )
            self.assertFalse(
                (output / "artifacts/bardicwonders-v2.9c-balance.2.toml").exists()
            )
            self.assertFalse(
                (output / "artifacts/bardicwonders-v2.9c-balance.3.toml").exists()
            )
            self.assertEqual(bardic_artifact["version"], "2.9c-balance.4")
            self.assertEqual(bardic_artifact["source"]["kind"], "github-release")
            self.assertEqual(bardic_artifact["source"]["reference"], "v2.9c-balance.4")
            self.assertEqual(
                bardic_artifact["source"]["url"],
                "https://github.com/Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch/"
                "releases/download/v2.9c-balance.4/Bardic-Wonders-v2.9c-balance.4.zip",
            )
            self.assertEqual(
                bardic_artifact["source"]["expected_filename"],
                "Bardic-Wonders-v2.9c-balance.4.zip",
            )
            self.assertEqual(bardic_artifact["source"]["expected_length"], 5177696)
            self.assertEqual(
                bardic_artifact["source"]["sha256"],
                "ca7bb2b70ad50b5b6c0fa59a051e53b90c42d3cc9f98fd187a5c1ed40fc1efa7",
            )
            self.assertEqual(bardic_artifact["archive"]["root_rule"], "direct")
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
            credits = build_public_component_credits(
                output, self.root / "app/package.json"
            )
            bardic_credit = next(
                mod for mod in credits["mods"] if mod["id"] == "bardicwonders"
            )
            self.assertEqual(bardic_credit["version"], "2.9c-balance.4")
            self.assertEqual(
                bardic_credit["homepage"],
                "https://github.com/Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch",
            )
            darkbloom = features["feature:bardicwonders:component-1006"]
            self.assertEqual(darkbloom["decision"], "default")
            self.assertEqual(
                [conflict["feature_id"] for conflict in darkbloom["conflicts"]],
                ["feature:spell-rev:mandatory-components"],
            )
            self.assertFalse(baseline_effective["feature:bardicwonders:component-1006"])
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
            self.assertEqual(runs["cdtweaks-spell-save-penalties-bg2"]["components"], [2310, 2311, 2312])
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
            self.assertEqual(json.loads((output / "release.json").read_text())["version"], "0.1.0-alpha.14")
            for version in ("0.1.0-alpha.1", "0.1.0-alpha.12", "0.1.0-alpha.13"):
                generated_ledger = output / f"releases/v{version}/ledger.toml"
                authored_ledger = self.root / f"manifest/releases/v{version}/ledger.toml"
                self.assertEqual(
                    generated_ledger.read_bytes(),
                    authored_ledger.read_bytes(),
                    f"historical ledger {version} changed during generation",
                )
            draft_ledger = tomllib.loads(
                (output / "releases/v0.1.0-alpha.14/ledger.toml").read_text()
            )
            self.assertEqual(draft_ledger["version"], "0.1.0-alpha.14")
            self.assertEqual(draft_ledger["minimum_app_version"], "0.1.0-alpha.15")
            self.assertEqual(draft_ledger["supersedes"], "0.1.0-alpha.13")
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

    def test_scs_immersion_options_are_opt_in_and_honor_native_sr_gate(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output = Path(temp_dir) / "recipe"
            build_recipe(self.root, output, "d6d46647b24b1a4baa501bca8c1d23048da3e83f")
            collection = tomllib.loads((output / "collection.toml").read_text(encoding="utf-8"))
            baseline = tomllib.loads(
                (output / "presets/chris-recommended.toml").read_text(encoding="utf-8")
            )["selections"]
            features = {feature["id"]: feature for feature in collection["features"]}
            ids = [f"feature:stratagems:component-{component}" for component in (4130, 4135, 4140)]
            effective = _effective_features(collection, baseline)
            for feature_id in ids:
                self.assertTrue(feature_id in features, f"Missing offered option: {feature_id}")
                self.assertEqual(features[feature_id]["decision"], "optional")
                self.assertEqual(features[feature_id]["readiness"], "ready")
                self.assertFalse(effective[feature_id])
                self.assertEqual(features[feature_id]["parent"], "mod:stratagems")
            requested = {**baseline, **{feature_id: "on" for feature_id in ids}}
            with_sr = _effective_features(collection, requested)
            self.assertFalse(with_sr[ids[0]])
            self.assertTrue(all(with_sr[feature_id] for feature_id in ids[1:]))
            without_sr = _effective_features(collection, {**requested, "mod:spell-rev": "off"})
            self.assertTrue(all(without_sr[feature_id] for feature_id in ids))
            self.assertIn("Spell Revisions", features[ids[0]]["description"])
            self.assertIn("experimental", features[ids[0]]["description"].lower())
            self.assertIn("SoD", features[ids[1]]["description"])
            self.assertFalse(features[ids[1]].get("requires"))
            self.assertFalse(features[ids[2]].get("requires"))
            scs_run = next(run for run in collection["runs"] if run["run_id"] == "stratagems-bg2")
            nearby = [c for c in scs_run["components"] if c in {4115, 4130, 4135, 4140, 4210}]
            self.assertEqual(nearby, [4115, 4130, 4135, 4140, 4210])
            declarations = {entry["id"] for entry in tomllib.loads((output / "mods/stratagems.toml").read_text())["components"]}
            for feature_id in ids:
                self.assertIn("Not recommended:", features[feature_id]["title"])
                self.assertIn("not independently reproduced", features[feature_id]["description"])
                for entry in features[feature_id]["components"]:
                    self.assertIn(entry["component"], declarations)

    def test_red_wizard_defaults_with_sr_but_remains_optional(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output = Path(temp_dir) / "recipe"
            build_recipe(self.root, output, "d6d46647b24b1a4baa501bca8c1d23048da3e83f")
            collection = tomllib.loads((output / "collection.toml").read_text(encoding="utf-8"))
            baseline = tomllib.loads(
                (output / "presets/chris-recommended.toml").read_text(encoding="utf-8")
            )["selections"]
            red_wizard_id = "feature:artisanskitpack-npc:component-5102"
            features = {feature["id"]: feature for feature in collection["features"]}
            self.assertEqual(features[red_wizard_id]["decision"], "default")
            self.assertEqual(features[red_wizard_id]["readiness"], "ready")
            effective = _effective_features(collection, baseline)
            self.assertTrue(effective["feature:spell-rev:mandatory-components"])
            self.assertTrue(effective[red_wizard_id])
            self.assertTrue(_effective_features(
                collection, {**baseline, "mod:spell-rev": "off"}
            )[red_wizard_id])
            for disabled in (red_wizard_id, "mod:artisanskitpack-npc"):
                with self.subTest(disabled=disabled):
                    self.assertFalse(_effective_features(
                        collection, {**baseline, disabled: "off"}
                    )[red_wizard_id])
            # Only Edwin's unsupported conflict is removed; other gates are separate.
            self.assertFalse(effective["feature:artisanskitpack-npc:component-10004"])
            run_ids = [run["run_id"] for run in collection["runs"]]
            self.assertLess(run_ids.index("artisanskitpack-npc-bg2"),
                            run_ids.index("spell-rev-npc-spellbooks-bg2"))
            limitations = tomllib.loads(
                (output / "releases/v0.1.0-alpha.1/known-limitations.toml").read_text(encoding="utf-8")
            )
            self.assertNotIn(red_wizard_id, {item["feature_id"] for item in limitations["omissions"]})

    def test_kivan_quest_guard_is_automatic_conditional_and_installed_once(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output = Path(temp_dir) / "recipe"
            build_recipe(self.root, output, "d6d46647b24b1a4baa501bca8c1d23048da3e83f")
            collection = tomllib.loads((output / "collection.toml").read_text(encoding="utf-8"))
            baseline = tomllib.loads(
                (output / "presets/chris-recommended.toml").read_text(encoding="utf-8")
            )["selections"]
            guard_id = "feature:chriz-bg-modpack:component-130"
            guard = next(feature for feature in collection["features"] if feature["id"] == guard_id)
            self.assertEqual(guard["decision"], "mandatory")
            self.assertEqual(guard["readiness"], "ready")
            self.assertEqual(guard["requires"], [
                "feature:bg1npc:component-10", "feature:stratagems:mandatory-components"
            ])
            self.assertTrue(_effective_features(collection, baseline)[guard_id])
            for prerequisite in ("mod:bg1npc", "feature:bg1npc:component-10", "mod:stratagems"):
                with self.subTest(prerequisite=prerequisite):
                    effective = _effective_features(collection, {**baseline, prerequisite: "off"})
                    self.assertFalse(effective[guard_id])
            # Kivan's class choice is unrelated to his quest dialogue guard.
            self.assertTrue(_effective_features(
                collection, {**baseline, "feature:chriz-bg-modpack:component-198": "off"}
            )[guard_id])
            runs = collection["runs"]
            guard_runs = [run for run in runs if run["mod_id"] == "chriz-bg-modpack" and 130 in run["components"]]
            self.assertEqual(len(guard_runs), 1)
            self.assertEqual(guard_runs[0]["components"].count(130), 1)
            self.assertEqual(guard_runs[0]["phase"], "post-eet-end")
            run_ids = [run["run_id"] for run in runs]
            for predecessor in ("bg1npc-bg1", "stratagems-bg2", "eet-end-bg2"):
                self.assertLess(run_ids.index(predecessor), run_ids.index(guard_runs[0]["run_id"]))
            for mod_path in (output / "mods").glob("*.toml"):
                mod = tomllib.loads(mod_path.read_text(encoding="utf-8"))
                self.assertNotIn("kivan_quest_fix", mod["tp2"].casefold())

    def test_reconciliation_covers_every_default_and_mandatory_row(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output = Path(temp_dir) / "recipe"
            build_recipe(self.root, output, "d6d46647b24b1a4baa501bca8c1d23048da3e83f")
            report = (output / "RECONCILIATION.md").read_text(encoding="utf-8")
            self.assertIn("Engine-equivalent resolved-selection summary", report)
            self.assertTrue("445 default/mandatory rows" in report)
            self.assertTrue("`CDTWEAKS:2380` | default" in report)
            self.assertIn("`KLATU:2150` | default", report)
            self.assertIn("`IWDIFICATION:120` | default", report)
            self.assertIn("`BARDICWONDERS:1006` | default", report)
            self.assertIn("`CHRIZ-BG-MODPACK:400` | mandatory", report)
            self.assertIn("`CHRIZ-BG-MODPACK:610` | default", report)
            self.assertIn("`CHRIZ-SOD-REMIX:290` | mandatory", report)
            self.assertIn("`CHRIZ-SOD-REMIX:910` | mandatory", report)


if __name__ == "__main__":
    unittest.main()
