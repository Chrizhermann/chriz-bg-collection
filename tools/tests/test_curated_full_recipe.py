from __future__ import annotations

import hashlib
import json
import tempfile
import tomllib
import unittest
from pathlib import Path

from tools.curated_full_recipe import (
    _effective_features, _preserve_native_run_orders, _validate_feature_groups, _validate_game_interface_order,
    build_recipe,
)
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
            self.assertEqual(changes, {"mod:evandra", "feature:evandra:mandatory-components", "feature:evandra:component-1", "feature:chriz-bg-modpack:component-224", "feature:chriz-bg-modpack:component-225", "feature:chriz-bg-modpack:component-238"})
            self.assertFalse(skipped["feature:evandra:mandatory-components"])
            self.assertFalse(skipped["feature:evandra:component-1"])

    def test_new_offered_component_requires_native_order_audit(self) -> None:
        collection = (self.root / "manifest/collection.toml").read_text(encoding="utf-8")
        collection = collection.replace("components = [1, 2, 20000", "components = [999999, 1, 2, 20000", 1)
        with self.assertRaisesRegex(ValueError, "requires a component-order audit"):
            _preserve_native_run_orders(collection)

    def test_rebalance_order_preserves_native_flail_tail_and_rejects_unaudited_ids(self) -> None:
        collection = (self.root / "manifest/collection.toml").read_text(encoding="utf-8")
        run_id = "chriz-bg-rebalance-bg2"
        runs = tomllib.loads(collection)["runs"]
        run = next(run for run in runs if run["run_id"] == run_id)
        self.assertEqual(run["components"][-8:], [408, 302, 420, 421, 310, 130, 220, 320])
        # Regression: sorting numerical IDs recreated the September 30 failure.
        sorted_source = collection.replace(
            f"components = {run['components']}",
            f"components = {sorted(run['components'])}", 1,
        )
        corrected = tomllib.loads(_preserve_native_run_orders(sorted_source))
        actual = next(run for run in corrected["runs"] if run["run_id"] == run_id)
        self.assertEqual(actual["components"], run["components"])
        unknown = collection.replace(
            f"components = {run['components']}",
            f"components = {run['components'] + [999999]}", 1,
        )
        with self.assertRaisesRegex(ValueError, "chriz-bg-rebalance-bg2 requires a component-order audit"):
            _preserve_native_run_orders(unknown)

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

    def test_stronghold_families_exclude_only_native_incompatible_choices(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output = Path(temp_dir) / "recipe"
            build_recipe(self.root, output, "d6d46647b24b1a4baa501bca8c1d23048da3e83f")
            self.assertEqual(
                (output / "collection.toml").read_text(encoding="utf-8"),
                (self.root / "recipes/curated-full-current/collection.toml").read_text(encoding="utf-8"),
                "Bundled collection must include the generated selection constraints",
            )
            collection = tomllib.loads((output / "collection.toml").read_text(encoding="utf-8"))
            preset = tomllib.loads((output / "presets/chris-recommended.toml").read_text())["selections"]
            features = {feature["id"]: feature for feature in collection["features"]}
            multiple = {f"feature:cdtweaks:component-{n}" for n in (1160, 1161)}
            individual = {f"feature:cdtweaks:component-{n}" for n in range(1340, 1348)}
            for key in multiple | individual:
                expected = (multiple | individual) - {key} if key in multiple else multiple
                self.assertEqual({rule["feature_id"] for rule in features[key]["conflicts"]}, expected)
                self.assertEqual(features[key]["readiness"], "ready")
            baseline = _effective_features(collection, preset)
            self.assertEqual({key for key in multiple | individual if baseline[key]},
                             {"feature:cdtweaks:component-1161"})
            all_individual = _effective_features(collection, {
                **preset, **{key: "off" for key in multiple}, **{key: "on" for key in individual},
            })
            self.assertEqual({key for key in multiple | individual if all_individual[key]}, individual)

    def test_custom_choices_require_actual_installed_cores(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output = Path(temp_dir) / "recipe"
            build_recipe(self.root, output, "d6d46647b24b1a4baa501bca8c1d23048da3e83f")
            collection = tomllib.loads((output / "collection.toml").read_text(encoding="utf-8"))
            preset = tomllib.loads((output / "presets/chris-recommended.toml").read_text())["selections"]
            features = {feature["id"]: feature for feature in collection["features"]}
            core = "feature:bg1npc:component-0"
            children = {key for key in features if key.startswith("feature:bg1npc:") and key != core}
            for key in children:
                self.assertIn(core, features[key].get("requires", []), key)
            selection = {**preset, **{key: "on" for key in children}, core: "off"}
            effective = _effective_features(collection, selection)
            self.assertTrue(effective["mod:bg1npc"])
            self.assertFalse(any(effective[key] for key in children))
            self.assertEqual(features["mod:stratagems"]["requires"], ["feature:ascension:component-0"])
            for key in ("mod:ascension", "feature:ascension:component-0"):
                effective = _effective_features(collection, {**preset, key: "off"})
                self.assertFalse(effective["mod:stratagems"], key)
                self.assertFalse(effective["feature:stratagems:mandatory-components"], key)

    def test_build_is_curation_derived_and_preserves_required_regressions(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output = Path(temp_dir) / "recipe"
            (output / "artifacts").mkdir(parents=True)
            for stale in [
                "chriz-bg-modpack-0.2.0-alpha.1",
                "chriz-bg-modpack-0.2.0-alpha.6",
                "bardicwonders-v2.9c-balance.2",
                "chriz-sod-remix-0.6.5",
                "chriz-sod-remix-0.6.6",
                "chriz-bg-rebalance-0.4.0",
                "creator-full-private-extras-20260902",
            ]:
                (output / "artifacts" / f"{stale}.toml").write_text(f'id = "{stale}"\n', encoding="utf-8")
            build_recipe(self.root, output, "d6d46647b24b1a4baa501bca8c1d23048da3e83f")
            self.assertFalse((output / "artifacts/chriz-bg-modpack-0.2.0-alpha.1.toml").exists())
            self.assertFalse((output / "artifacts/chriz-bg-modpack-0.2.0-alpha.6.toml").exists())
            modpack = tomllib.loads((output / "mods/chriz-bg-modpack.toml").read_text(encoding="utf-8"))
            self.assertEqual(modpack["artifact_id"], "chriz-bg-modpack-0.2.0-alpha.10")
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
                "feature_groups": collection.get("feature_groups", []),
            }
            semantic_digest = hashlib.sha256(
                json.dumps(semantic_recipe, sort_keys=True).encode()
            ).hexdigest()
            self.assertEqual(
                semantic_digest,
                # September 26 adds Rebalance 301. September 28 adds stronghold
                # exclusions and BG1NPC/SCS core gates; defaults/order unchanged.
                # September 29 adds UI providers/helper, shifted dialogue, default
                # Flail302, WingBuffet111 and Evandra224/225, and swaps utility XP610 for611.
                # September 30 corrects Rebalance302 to its native TP2 position after408.
                # October 7 adds approved Hardiness/anti-magic/items/resistance
                # defaults and two separately ordered companion components.
                # October 12 implements the agreed next-release Emotion301 opt-in;
                # all other semantic selections, dependencies and order stay intact.
                # October 13 adds default bard spell-level ceilings
                # (Rebalance 420/421 plus Bardic Wonders 3010 in its own late run).
                # October 13 (later) adds the approved Challenge Mode feature group,
                # now part of this digest; runs, features and the preset are unchanged.
                # October 13 makes EET's desktop shortcut (EET 100) an unchecked option.
                # October 13 intake (Modpack alpha.11 / SoD Remix v0.6.13 candidates):
                # default reader-level scrolls 430 in its own final run, default
                # Fade/Evandra recruitment XP 237/238 and Degrodel 660, optional
                # experimental campaign XP caps 650/651 in their own run, mandatory
                # SoD 177/198/285, and SoD 257's reworded description.
                "0e297b32a5f9aa71fd15581b9c8409bb547e7a5cfea8542657a71d1ac7eeab06",
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
            self.assertEqual(sod_mod["artifact_id"], "chriz-sod-remix-0.6.12")
            # Setup-name WeiDU resolves the nested copy when both identical TP2s
            # are shipped. The frozen expected log identity must use that path.
            self.assertEqual(sod_mod["tp2"], "chriz-sod-remix/setup-chriz-sod-remix.tp2")
            sod_artifact = tomllib.loads(
                (output / "artifacts/chriz-sod-remix-0.6.12.toml").read_text(encoding="utf-8")
            )
            self.assertFalse((output / "artifacts/chriz-sod-remix-0.6.4.toml").exists())
            self.assertFalse((output / "artifacts/chriz-sod-remix-0.6.5.toml").exists())
            self.assertFalse((output / "artifacts/chriz-sod-remix-0.6.6.toml").exists())
            self.assertFalse((output / "artifacts/chriz-sod-remix-0.6.7.toml").exists())
            self.assertFalse((output / "artifacts/chriz-sod-remix-0.6.11.toml").exists())
            self.assertEqual(sod_artifact["version"], "0.6.12")
            self.assertEqual(sod_artifact["source"]["reference"], "v0.6.12")
            self.assertEqual(
                sod_artifact["source"]["expected_filename"],
                "chriz-sod-remix-v0.6.12.zip",
            )
            self.assertEqual(sod_artifact["source"]["expected_length"], 2959604)
            self.assertEqual(
                sod_artifact["source"]["sha256"],
                "8c1a18debed45530d629efa19aa6c6d3b34317751d89683104b0c0306e67cd97",
            )
            # October 13 adds mandatory v0.6.13 rows 177, 198 and 285 in native TP2 order.
            self.assertEqual(runs["chriz-sod-remix-bg2"]["components"], [100, 110, 115, 120, 130, 135, 140, 150, 145, 160, 170, 180, 175, 177, 185, 190, 195, 210, 197, 198, 187, 200, 215, 220, 225, 245, 230, 240, 250, 255, 256, 257, 260, 265, 266, 270, 280, 285, 290, 900, 910])
            self.assertNotIn(291, runs["chriz-sod-remix-bg2"]["components"])
            self.assertNotIn(258, runs["chriz-sod-remix-bg2"]["components"])
            self.assertLess(runs["chriz-sod-remix-bg2"]["components"].index(210), runs["chriz-sod-remix-bg2"]["components"].index(197))
            buffbot = tomllib.loads((output / "mods/buffbot.toml").read_text(encoding="utf-8"))
            self.assertEqual(buffbot["artifact_id"], "buffbot-1.10.0")
            buffbot_artifact = tomllib.loads(
                (output / "artifacts/buffbot-1.10.0.toml").read_text(encoding="utf-8")
            )
            self.assertEqual(buffbot_artifact["version"], "1.10.0")
            self.assertEqual(buffbot_artifact["source"]["reference"], "v1.10.0")
            self.assertEqual(buffbot_artifact["source"]["expected_filename"], "buffbot-v1.10.0.zip")
            self.assertEqual(buffbot_artifact["source"]["expected_length"], 4268920)
            self.assertEqual(buffbot_artifact["source"]["sha256"], "cea6628cc6f00eeb3d296fe50f481f5c13ad4ea3530bc67716493d0a5b016f77")
            self.assertEqual(runs["buffbot-bg2"]["components"], [1, 0])
            self.assertEqual(runs["bardicwonders-garrick-bg2"]["components"], [1008])
            bardic = tomllib.loads((output / "mods/bardicwonders.toml").read_text(encoding="utf-8"))
            self.assertEqual(bardic["artifact_id"], "bardicwonders-v2.9c-balance.6")
            bardic_artifact = tomllib.loads(
                (output / "artifacts/bardicwonders-v2.9c-balance.6.toml").read_text(
                    encoding="utf-8"
                )
            )
            self.assertFalse(
                (output / "artifacts/bardicwonders-v2.9c-balance.2.toml").exists()
            )
            self.assertFalse(
                (output / "artifacts/bardicwonders-v2.9c-balance.3.toml").exists()
            )
            self.assertFalse((output / "artifacts/bardicwonders-v2.9c-balance.5.toml").exists())
            self.assertEqual(bardic_artifact["version"], "2.9c-balance.6")
            self.assertEqual(bardic_artifact["source"]["kind"], "github-release")
            self.assertEqual(bardic_artifact["source"]["reference"], "v2.9c-balance.6")
            self.assertEqual(
                bardic_artifact["source"]["url"],
                "https://github.com/Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch/"
                "releases/download/v2.9c-balance.6/Bardic-Wonders-v2.9c-balance.6.zip",
            )
            self.assertEqual(
                bardic_artifact["source"]["expected_filename"],
                "Bardic-Wonders-v2.9c-balance.6.zip",
            )
            self.assertEqual(bardic_artifact["source"]["expected_length"], 5191057)
            self.assertEqual(
                bardic_artifact["source"]["sha256"],
                "48b222eb7dd827c7541dea33129bd094ee81e28c33f33bdcb3bef81de197a59a",
            )
            self.assertEqual(bardic_artifact["archive"]["root_rule"], "direct")
            self.assertEqual(bardic_artifact["archive"]["publish_roots"], ["BardicWonders"])
            self.assertEqual(
                bardic_artifact["archive"]["tp2_paths"],
                ["BardicWonders/Setup-BardicWonders.tp2"],
            )
            # The default 3010 progression adapter runs only in its own late run;
            # existing choices and their native TP2 order are unchanged.
            self.assertIn(3010, [component["id"] for component in bardic["components"]])
            self.assertEqual(runs["bardicwonders-progression-bg2"]["components"], [3010])
            self.assertTrue(baseline_effective["feature:bardicwonders:component-3010"])
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
            self.assertEqual(bardic_credit["version"], "2.9c-balance.6")
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
            # Alpha.11's reader-level scroll policy needs EEex, not Tweaks Anthology 2170.
            self.assertEqual(features["feature:chriz-bg-modpack:component-430"]["requires"], ["feature:eeex:mandatory-components"])
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
            for component in (220, 221, 611):
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
            self.assertEqual(len(bardic_features), 21)
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
            self.assertEqual(bg_mod["artifact_id"], "chriz-bg-rebalance-0.8.1")
            bg_artifact = tomllib.loads(
                (output / "artifacts/chriz-bg-rebalance-0.8.1.toml").read_text(encoding="utf-8")
            )
            self.assertFalse((output / "artifacts/chriz-bg-rebalance-0.3.1.toml").exists())
            self.assertFalse((output / "artifacts/chriz-bg-rebalance-0.4.0.toml").exists())
            self.assertEqual(bg_artifact["version"], "0.8.1")
            self.assertEqual(bg_artifact["source"]["reference"], "v0.8.1")
            self.assertEqual(bg_artifact["source"]["expected_length"], 1753775)
            self.assertEqual(
                bg_artifact["source"]["sha256"],
                "5d6cc9dfec8779e1e4534da686dc2f3849bcb795b24981c0930d95918efccf0c",
            )
            self.assertEqual(json.loads((output / "release.json").read_text())["version"], "0.1.0-alpha.19")
            for version in ("0.1.0-alpha.1", "0.1.0-alpha.12", "0.1.0-alpha.13", "0.1.0-alpha.14", "0.1.0-alpha.15", "0.1.0-alpha.16", "0.1.0-alpha.17", "0.1.0-alpha.18"):
                generated_ledger = output / f"releases/v{version}/ledger.toml"
                authored_ledger = self.root / f"manifest/releases/v{version}/ledger.toml"
                self.assertEqual(
                    generated_ledger.read_bytes(),
                    authored_ledger.read_bytes(),
                    f"historical ledger {version} changed during generation",
                )
            draft_ledger = tomllib.loads(
                (output / "releases/v0.1.0-alpha.16/ledger.toml").read_text()
            )
            self.assertEqual(draft_ledger["version"], "0.1.0-alpha.16")
            self.assertEqual(draft_ledger["minimum_app_version"], "0.1.0-alpha.18")
            self.assertEqual(draft_ledger["supersedes"], "0.1.0-alpha.15")
            self.assertEqual({change["id"] for change in draft_ledger["changes"]}, {"modpack-compatible-presets"})
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

    def test_interval_save_options_default_with_hgo_and_can_be_disabled(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output = Path(temp_dir) / "recipe"
            build_recipe(self.root, output, "d6d46647b24b1a4baa501bca8c1d23048da3e83f")
            collection = tomllib.loads((output / "collection.toml").read_text(encoding="utf-8"))
            baseline = tomllib.loads(
                (output / "presets/chris-recommended.toml").read_text(encoding="utf-8")
            )["selections"]
            features = {feature["id"]: feature for feature in collection["features"]}
            runs = {run["run_id"]: run for run in collection["runs"]}
            hgo_id = "feature:hiddengameplayoptions:component-38"
            interval_ids = [
                f"feature:cdtweaks:component-{component}"
                for component in range(3350, 3358)
            ]

            self.assertEqual(features[hgo_id]["decision"], "default")
            self.assertEqual(features[hgo_id]["readiness"], "ready")
            self.assertEqual(features[hgo_id].get("requires", []), [])
            self.assertEqual(features[hgo_id]["requires_any"], interval_ids)
            self.assertEqual(baseline["feature:cdtweaks:component-3354"], "on")
            self.assertEqual(baseline[hgo_id], "on")
            self.assertTrue(_effective_features(collection, baseline)[hgo_id])
            self.assertIn(3354, runs["cdtweaks-bg2"]["components"])
            self.assertIn(38, runs["hiddengameplayoptions-bg2"]["components"])

            alternate = {
                **baseline,
                "feature:cdtweaks:component-3354": "off",
                "feature:cdtweaks:component-3355": "on",
            }
            self.assertTrue(_effective_features(collection, alternate)[hgo_id])

            no_interval_saves = baseline.copy()
            no_interval_saves.update({feature_id: "off" for feature_id in interval_ids})
            effective = _effective_features(collection, no_interval_saves)
            self.assertFalse(effective[hgo_id])
            self.assertFalse(any(effective[feature_id] for feature_id in interval_ids))

            explicit_no_interval_saves = {**no_interval_saves, hgo_id: "off"}
            self.assertFalse(_effective_features(collection, explicit_no_interval_saves)[hgo_id])

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

    def test_release_intake_preserves_the_freshly_verified_artifact_identities(self) -> None:
        evidence = tomllib.loads((self.root / "manifest/evidence/release-intake-20260929.toml").read_text())
        records = {record["artifact_id"]: record for record in evidence["artifacts"]}
        required = {
            "infinity-ui-1.28", "dragonspear-ui-22.01.3", "leui-4.9.1",
            "leui-bg1ee-4.9.1", "leui-sod-4.9.1",
            "eet-official-74e91d72bca5d073fa11c1d088b90d7ff0c7105d",
            "artisans-kitpack-4.81a-dev.85928f9-chriz.1", "chriz-bg-rebalance-0.7.0",
            "chriz-bg-modpack-0.2.0-alpha.8",
        }
        self.assertTrue(required <= records.keys())
        with tempfile.TemporaryDirectory() as temp_dir:
            output = Path(temp_dir) / "recipe"
            build_recipe(self.root, output, "d6d46647b24b1a4baa501bca8c1d23048da3e83f")
            for directory in (self.root / "manifest", output):
                for id_, record in records.items():
                    with self.subTest(recipe=str(directory), artifact=id_):
                        artifact = tomllib.loads((directory / f"artifacts/{id_}.toml").read_text())
                        self.assertEqual(artifact["acquisition"], "fetch-only")
                        self.assertTrue(record["fresh_download"])
                        self.assertTrue(record["fresh_extraction"])
                        for source_key, evidence_key in (
                            ("reference", "source_reference"), ("url", "immutable_source_url"),
                            ("expected_length", "actual_length"), ("sha256", "actual_sha256"),
                        ):
                            self.assertEqual(artifact["source"][source_key], record[evidence_key])

    def test_game_interface_choices_keep_defaults_and_component_specific_conflicts(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output = Path(temp_dir) / "recipe"
            build_recipe(self.root, output, "d6d46647b24b1a4baa501bca8c1d23048da3e83f")
            for directory in (self.root / "manifest", output):
                with self.subTest(recipe=str(directory)):
                    collection = tomllib.loads((directory / "collection.toml").read_text(encoding="utf-8"))
                    preset = tomllib.loads((directory / "presets/chris-recommended.toml").read_text())["selections"]
                    features = {feature["id"]: feature for feature in collection["features"]}
                    providers = {id_: feature for id_, feature in features.items()
                                 if feature.get("choice_group") == "game-interface"}
                    self.assertEqual(set(providers), {
                        f"feature:game-interface:{name}" for name in
                        ("infinity", "eet-sod", "dragonspear", "leui-bg2", "leui-bg1", "leui-sod")
                    })
                    defaults = _effective_features(collection, preset)
                    self.assertTrue(all(not defaults[id_] for id_ in providers))
                    for id_, provider in providers.items():
                        self.assertEqual(provider["decision"], "optional")
                        self.assertEqual({c["feature_id"] for c in provider["conflicts"]}, set(providers) - {id_})
                        self.assertIn("feature:eet:mandatory-components", provider["requires"])
                    infinity = _effective_features(collection, {
                        **preset, "feature:game-interface:infinity": "on",
                    })
                    expected_omissions = {
                        "feature:stratagems:component-4115",
                        *[f"feature:hiddengameplayoptions:component-{c}" for c in (29, 30, 300, 301)],
                    }
                    self.assertEqual({id_ for id_ in defaults if defaults[id_] and not infinity[id_]},
                                     expected_omissions)
                    self.assertTrue(infinity["feature:eeex:mandatory-components"])
                    self.assertTrue(infinity["feature:buffbot:mandatory-components"])
                    restored = _effective_features(collection, {
                        **preset, "feature:game-interface:infinity": "off",
                    })
                    self.assertEqual(defaults, restored)
                    for component in (28, 29, 30, 200):
                        conflicts = features[f"feature:hiddengameplayoptions:component-{component}"].get("conflicts", [])
                        self.assertNotIn("feature:game-interface:dragonspear",
                                         {conflict["feature_id"] for conflict in conflicts})
                    for component in (300, 301, 302, 304):
                        self.assertIn("feature:game-interface:dragonspear", {
                            c["feature_id"] for c in
                            features[f"feature:hiddengameplayoptions:component-{component}"]["conflicts"]
                        })

    def test_flail_default_is_independent_of_eeex_and_emotion_optin(self) -> None:
        collection = tomllib.loads((self.root / "manifest/collection.toml").read_text(encoding="utf-8"))
        preset = tomllib.loads((self.root / "manifest/presets/chris-recommended.toml").read_text())["selections"]
        id_ = "feature:chriz-bg-rebalance:component-302"
        features = {feature["id"]: feature for feature in collection["features"]}
        self.assertEqual(features[id_]["decision"], "default")
        self.assertEqual(features[id_].get("requires", []), [])
        defaults = _effective_features(collection, preset)
        self.assertTrue(defaults[id_])
        self.assertFalse(defaults["feature:chriz-bg-rebalance:component-301"])
        self.assertTrue(_effective_features(collection, {
            **preset, "feature:chriz-bg-rebalance:component-301": "on",
        })[id_])
        self.assertTrue(_effective_features(collection, {**preset, "mod:eeex": "off"})[id_])
        self.assertFalse(_effective_features(collection, {**preset, id_: "off"})[id_])
        self.assertFalse(_effective_features(collection, {**preset, "mod:chriz-bg-rebalance": "off"})[id_])
        self.assertTrue(all(f"feature:chriz-bg-rebalance:component-{id_}" not in features for id_ in (420, 421)))
        run = next(run for run in collection["runs"] if run["run_id"] == "chriz-bg-rebalance-bg2")
        self.assertEqual(run["phase"], "post-eet-end")
        self.assertEqual(run["components"][-8:], [408, 302, 420, 421, 310, 130, 220, 320])

    def test_shifted_dialogue_is_optional_and_requires_either_kit_without_eeex(self) -> None:
        collection = tomllib.loads((self.root / "manifest/collection.toml").read_text(encoding="utf-8"))
        preset = tomllib.loads((self.root / "manifest/presets/chris-recommended.toml").read_text())["selections"]
        id_ = "feature:artisanskitpack-tweak:component-51010"
        feature = next(feature for feature in collection["features"] if feature["id"] == id_)
        self.assertEqual(feature["decision"], "optional")
        self.assertEqual(feature["requires_any"], [
            "feature:artisanskitpack:component-5100", "feature:artisanskitpack:component-5002",
        ])
        self.assertNotIn("feature:eeex:mandatory-components", feature.get("requires", []))
        defaults = _effective_features(collection, preset)
        self.assertFalse(defaults[id_])
        self.assertTrue(_effective_features(collection, {**preset, id_: "on"})[id_])
        for shapeshifter, hivemaster, expected in (("on", "off", True), ("off", "on", True), ("off", "off", False)):
            self.assertEqual(_effective_features(collection, {
                **preset, id_: "on", "mod:eeex": "off",
                "feature:artisanskitpack:component-5100": shapeshifter,
                "feature:artisanskitpack:component-5002": hivemaster,
            })[id_], expected)
        self.assertFalse(_effective_features(collection, {
            **preset, id_: "on", "mod:artisanskitpack-tweak": "off",
        })[id_])

    def test_classic_interfaces_require_released_helper_independent_of_optional_parents(self) -> None:
        collection = tomllib.loads((self.root / "manifest/collection.toml").read_text(encoding="utf-8"))
        preset = tomllib.loads((self.root / "manifest/presets/chris-recommended.toml").read_text())["selections"]
        features = {feature["id"]: feature for feature in collection["features"]}
        helper = "feature:chriz-bg-modpack:component-640"
        self.assertEqual(features[helper]["decision"], "mandatory")
        self.assertNotIn("parent", features[helper])
        self.assertFalse(_effective_features(collection, preset)[helper])
        for name in ("dragonspear", "leui-bg2", "leui-bg1", "leui-sod"):
            id_ = f"feature:game-interface:{name}"
            self.assertEqual(features[id_]["readiness"], "experimental")
            selected = _effective_features(collection, {
                **preset, id_: "on", helper: "off", "mod:eeex": "off",
                "mod:hidden-gameplay-options": "off", "mod:chriz-bg-modpack": "off",
            })
            self.assertTrue(selected[id_])
            self.assertTrue(selected[helper])
        for name in ("infinity", "eet-sod"):
            self.assertFalse(_effective_features(collection, {**preset, f"feature:game-interface:{name}": "on"})[helper])

    def test_game_interface_generator_rejects_a_ui_replacement_after_bubb(self) -> None:
        collection = (self.root / "manifest/collection.toml").read_text(encoding="utf-8")
        _validate_game_interface_order(collection)
        # Swapping the names preserves valid TOML but violates the consumer boundary.
        moved = collection.replace('run_id = "game-interface-infinity-bg2"', 'run_id = "swap"', 1)
        moved = moved.replace('run_id = "bubb-spell-menu-bg2"', 'run_id = "game-interface-infinity-bg2"', 1)
        moved = moved.replace('run_id = "swap"', 'run_id = "bubb-spell-menu-bg2"', 1)
        with self.assertRaisesRegex(ValueError, "Review game interface order"):
            _validate_game_interface_order(moved)

    def test_reconciliation_covers_every_default_and_mandatory_row(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output = Path(temp_dir) / "recipe"
            build_recipe(self.root, output, "d6d46647b24b1a4baa501bca8c1d23048da3e83f")
            report = (output / "RECONCILIATION.md").read_text(encoding="utf-8")
            self.assertIn("Engine-equivalent resolved-selection summary", report)
            self.assertTrue("`CDTWEAKS:2380` | default" in report)
            self.assertIn("`HIDDENGAMEPLAYOPTIONS:38` | default", report)
            self.assertIn("`KLATU:2150` | default", report)
            self.assertIn("`IWDIFICATION:120` | default", report)
            self.assertIn("`BARDICWONDERS:1006` | default", report)
            self.assertIn("`CHRIZ-BG-MODPACK:400` | mandatory", report)
            self.assertIn("`CHRIZ-BG-MODPACK:611` | default", report)
            self.assertIn("`CHRIZ-SOD-REMIX:290` | mandatory", report)
            self.assertIn("`CHRIZ-SOD-REMIX:910` | mandatory", report)


    def test_challenge_mode_group_is_carried_unchanged_and_never_preselected(self) -> None:
        base = tomllib.loads((self.root / "manifest/collection.toml").read_text(encoding="utf-8"))
        with tempfile.TemporaryDirectory() as temp_dir:
            output = Path(temp_dir) / "curated"
            build_recipe(self.root, output, "d6d46647b24b1a4baa501bca8c1d23048da3e83f")
            collection = tomllib.loads((output / "collection.toml").read_text(encoding="utf-8"))
            preset = tomllib.loads((output / "presets/chris-recommended.toml").read_text(encoding="utf-8"))["selections"]
        self.assertEqual(collection["feature_groups"], base["feature_groups"])
        [group] = collection["feature_groups"]
        self.assertEqual(group["id"], "challenge-mode")
        self.assertEqual(group["members"], [
            "feature:chriz-bg-rebalance:component-301",
            "feature:chriz-bg-modpack:component-235",
            "feature:chriz-bg-rebalance:component-110",
            "feature:chriz-sod-remix:component-257",
        ])
        features = {feature["id"]: feature for feature in collection["features"]}
        for member in group["members"]:
            self.assertEqual(features[member]["decision"], "optional", member)
            self.assertNotIn("choice_group", features[member], member)
            self.assertIn(preset.get(member, "off"), {"off"}, member)
        self.assertNotIn("challenge-mode", preset)

    def test_feature_group_guard_rejects_dropped_or_altered_groups(self) -> None:
        base = {"feature_groups": [{"id": "challenge-mode", "title": "Challenge Mode", "description": "Harder.", "members": ["a"]}]}
        _validate_feature_groups(base, {"features": [{"id": "a"}], **base})
        with self.assertRaisesRegex(ValueError, "carried unchanged"):
            _validate_feature_groups(base, {"features": [{"id": "a"}]})
        with self.assertRaisesRegex(ValueError, "absent from the curated recipe"):
            _validate_feature_groups(base, {"features": [], **base})


if __name__ == "__main__":
    unittest.main()
