"""Released Evandra recipe delivery plus synthetic unsupported-pin rejection."""

from copy import deepcopy
from pathlib import Path
import re
import shutil
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import _effective_features, build_recipe
from tools.evandra_defaults import (
    DEFAULT_FEATURE_IDS,
    EVANDRA_CORE,
    MODPACK_RUN,
    check_evandra_defaults,
    evandra_default_findings,
    integrate_evandra_defaults,
)


class EvandraDefaultTests(unittest.TestCase):
    root = Path(__file__).resolve().parents[2]
    recipe = root / "recipes/curated-full-current"

    def setUp(self):
        self.text = (self.recipe / "collection.toml").read_text(encoding="utf-8")
        self.modpack = tomllib.loads(
            (self.recipe / "mods/chriz-bg-modpack.toml").read_text(encoding="utf-8")
        )
        self.preset = tomllib.loads(
            (self.recipe / "presets/chris-recommended.toml").read_text(encoding="utf-8")
        )["selections"]

    def released_shape(self):
        modpack = deepcopy(self.modpack)
        text = integrate_evandra_defaults(self.text, modpack)
        return text, modpack, deepcopy(self.preset)

    @staticmethod
    def without_defaults(text):
        """Synthetic pre-migration collection, without altering production files."""
        for feature_id in DEFAULT_FEATURE_IDS:
            text = re.sub(
                rf'(?ms)^\[\[features\]\]\r?\nid = "{re.escape(feature_id)}"\r?\n.*?(?=^\[\[|\Z)',
                "", text,
            )
        pattern = rf'(?m)(^run_id = "{re.escape(MODPACK_RUN)}"\r?\nmod_id = "chriz-bg-modpack"\r?\nphase = "main"\r?\n)components = \[([^\]\r\n]*)\]'
        def remove_components(match):
            components = [int(value.strip()) for value in match[2].split(",") if value.strip()]
            return match[1] + f"components = {[n for n in components if n not in (224, 225)]}"
        text, count = re.subn(pattern, remove_components, text)
        if count != 1:
            raise AssertionError("Synthetic fixture could not identify the Modpack run")
        return text

    def test_released_alpha9_catalogue_preserves_evandra_default_delivery(self):
        self.assertEqual(self.modpack["artifact_id"], "chriz-bg-modpack-0.2.0-alpha.10")
        ids = [component["id"] for component in self.modpack["components"]]
        for component in (224, 225, 611, 630, 640):
            self.assertEqual(ids.count(component), 1)
        self.assertEqual(
            (self.recipe / "mods/chriz-bg-modpack.toml").read_bytes(),
            (self.root / "manifest/mods/chriz-bg-modpack.toml").read_bytes(),
        )
        self.assertEqual(integrate_evandra_defaults(self.text, self.modpack), self.text)
        self.assertEqual(evandra_default_findings(self.recipe), [])

    def test_authored_fallback_has_default_definitions_without_claiming_enabled_evandra(self):
        authored = tomllib.loads((self.root / "manifest/collection.toml").read_text(encoding="utf-8"))
        authored_preset = tomllib.loads((self.root / "manifest/presets/chris-recommended.toml").read_text(encoding="utf-8"))["selections"]
        features = {feature["id"]: feature for feature in authored["features"]}
        generated = {feature["id"]: feature for feature in tomllib.loads(self.text)["features"]}
        # The narrow authored fallback intentionally leaves acquisition unavailable;
        # a green gate there skips Evandra and is not production delivery evidence.
        self.assertEqual(features["mod:evandra"]["readiness"], "blocked")
        self.assertNotIn(EVANDRA_CORE, features)
        self.assertEqual(generated["mod:evandra"]["readiness"], "ready")
        self.assertIn(EVANDRA_CORE, generated)
        for feature_id in DEFAULT_FEATURE_IDS:
            for field in ("decision", "parent", "components"):
                self.assertEqual(features[feature_id][field], generated[feature_id][field])
            self.assertEqual(features[feature_id]["readiness"], "blocked")
            self.assertEqual(features[feature_id]["requires"], ["mod:evandra"])
            self.assertIn("generated full recipe", features[feature_id]["unavailable_reason"])
            self.assertEqual(generated[feature_id]["readiness"], "ready")
            self.assertEqual(generated[feature_id]["requires"], [EVANDRA_CORE])
            self.assertNotIn("unavailable_reason", generated[feature_id])
            self.assertEqual(authored_preset[feature_id], "on")
        fallback_state = _effective_features(authored, authored_preset)
        self.assertTrue(all(not fallback_state[key] for key in DEFAULT_FEATURE_IDS))

    def test_profiles_not_offering_evandra_need_no_modpack_or_named_preset(self):
        with tempfile.TemporaryDirectory() as temporary:
            recipe = Path(temporary)
            for text in ('features = []\n', '[collection]\nname = "Minimal"\n',
                         '[[features]]\nid = "mod:evandra"\nreadiness = "blocked"\n'):
                with self.subTest(text=text):
                    (recipe / "collection.toml").write_text(text, encoding="utf-8")
                    self.assertEqual(evandra_default_findings(recipe), [])

    def test_synthetic_alpha7_is_rejected_even_with_relabelled_catalogue(self):
        text = self.without_defaults(self.text)
        for relabelled in (False, True):
            with self.subTest(relabelled=relabelled):
                _, modpack, preset = self.released_shape()
                modpack["artifact_id"] = "chriz-bg-modpack-0.2.0-alpha.7"
                if not relabelled:
                    modpack["components"] = [c for c in modpack["components"] if c["id"] not in (224, 225)]
                self.assertEqual(integrate_evandra_defaults(text, modpack), text)
                findings = check_evandra_defaults(tomllib.loads(self.text), modpack, preset)
                self.assertTrue(any("224" in item and "225" in item for item in findings))

    def test_partial_catalogue_never_enables_either_default(self):
        for missing in (224, 225):
            _, modpack, _ = self.released_shape()
            modpack["components"] = [c for c in modpack["components"] if c["id"] != missing]
            text = self.without_defaults(self.text)
            self.assertEqual(integrate_evandra_defaults(text, modpack), text)

    def test_released_shape_has_independent_defaults_after_the_actual_core(self):
        text, modpack, preset = self.released_shape()
        collection = tomllib.loads(text)
        self.assertEqual(check_evandra_defaults(collection, modpack, preset), [])
        defaults = _effective_features(collection, preset)
        self.assertTrue(all(defaults[key] for key in DEFAULT_FEATURE_IDS))
        features = {f["id"]: f for f in collection["features"]}
        for key in DEFAULT_FEATURE_IDS:
            self.assertEqual(features[key]["requires"], [EVANDRA_CORE])
        run = next(r for r in collection["runs"] if r["run_id"] == MODPACK_RUN)
        index = run["components"].index(223)
        self.assertEqual(run["components"][index:index + 3], [223, 224, 225])
        self.assertNotIn(191, run["components"])
        self.assertEqual(integrate_evandra_defaults(text, modpack), text)

    def test_released_catalogue_can_author_missing_default_definitions(self):
        text = integrate_evandra_defaults(self.without_defaults(self.text), self.modpack)
        self.assertEqual(check_evandra_defaults(tomllib.loads(text), self.modpack, self.preset), [])
        self.assertEqual(integrate_evandra_defaults(text, self.modpack), text)

    def test_skipping_evandra_cascades_and_core_availability_is_required(self):
        text, _, preset = self.released_shape()
        collection = tomllib.loads(text)
        defaults = _effective_features(collection, preset)
        skipped = _effective_features(collection, {**preset, "mod:evandra": "off"})
        self.assertEqual({key for key in defaults if defaults[key] != skipped[key]}, {
            "mod:evandra", EVANDRA_CORE, "feature:evandra:component-1", *DEFAULT_FEATURE_IDS,
        })
        next(f for f in collection["features"] if f["id"] == EVANDRA_CORE)["readiness"] = "blocked"
        state = _effective_features(collection, preset)
        self.assertTrue(all(not state[key] for key in DEFAULT_FEATURE_IDS))

    def test_crossmod_and_each_default_can_be_disabled_independently(self):
        text, _, preset = self.released_shape()
        collection = tomllib.loads(text)
        defaults = _effective_features(collection, preset)
        for disabled in ("feature:evandra:component-1", *DEFAULT_FEATURE_IDS):
            state = _effective_features(collection, {**preset, disabled: "off"})
            self.assertEqual({key for key in defaults if defaults[key] != state[key]}, {disabled})
        state = _effective_features(collection, {**preset, "mod:chriz-bg-modpack": "off"})
        self.assertTrue(state[EVANDRA_CORE])
        self.assertTrue(all(not state[key] for key in DEFAULT_FEATURE_IDS))

    def test_missing_or_rewired_delivery_cannot_pass_readiness(self):
        for fault in ("feature", "catalogue", "run", "default", "core", "order", "duplicate"):
            with self.subTest(fault=fault):
                text, modpack, preset = self.released_shape()
                collection = tomllib.loads(text)
                feature = next(f for f in collection["features"] if f["id"] == DEFAULT_FEATURE_IDS[0])
                run = next(r for r in collection["runs"] if r["run_id"] == MODPACK_RUN)
                if fault == "feature":
                    collection["features"].remove(feature)
                elif fault == "catalogue":
                    modpack["components"] = [c for c in modpack["components"] if c["id"] != 224]
                elif fault == "run":
                    run["components"].remove(224)
                elif fault == "default":
                    preset[DEFAULT_FEATURE_IDS[0]] = "off"
                elif fault == "core":
                    feature["requires"] = ["mod:evandra"]
                elif fault == "order":
                    start = run["components"].index(224)
                    run["components"][start:start + 2] = [225, 224]
                else:
                    run["components"].append(224)
                self.assertTrue(check_evandra_defaults(collection, modpack, preset))

    def test_generator_preserves_released_catalogue_and_produces_green_defaults(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "recipe"
            build_recipe(self.root, output, "d6d46647b24b1a4baa501bca8c1d23048da3e83f")
            self.assertEqual(
                (output / "mods/chriz-bg-modpack.toml").read_bytes(),
                (self.root / "manifest/mods/chriz-bg-modpack.toml").read_bytes(),
            )
            generated = tomllib.loads((output / "collection.toml").read_text(encoding="utf-8"))
            self.assertTrue(set(DEFAULT_FEATURE_IDS) <= {f["id"] for f in generated["features"]})
            self.assertEqual(evandra_default_findings(output), [])

    def test_generator_restores_recommended_selections_from_released_catalogue(self):
        with tempfile.TemporaryDirectory() as temporary:
            fixture = Path(temporary) / "source"
            for relative in ("manifest", "docs/curation"):
                shutil.copytree(self.root / relative, fixture / relative)
            preset = fixture / "manifest/presets/chris-recommended.toml"
            text = preset.read_text(encoding="utf-8")
            for feature_id in DEFAULT_FEATURE_IDS:
                text = re.sub(rf'(?m)^"{re.escape(feature_id)}" = "on"\r?\n', "", text)
            preset.write_text(text, encoding="utf-8")
            output = Path(temporary) / "recipe"
            build_recipe(fixture, output, "d6d46647b24b1a4baa501bca8c1d23048da3e83f")
            self.assertEqual(evandra_default_findings(output), [])
            # The synthetic preset omission exercises recipe authoring, not archive or live acceptance.
            # The generator must preserve the real frozen catalogue and artifact bytes.
            self.assertEqual(
                (output / "artifacts/chriz-bg-modpack-0.2.0-alpha.10.toml").read_bytes(),
                (self.root / "manifest/artifacts/chriz-bg-modpack-0.2.0-alpha.10.toml").read_bytes(),
            )


if __name__ == "__main__":
    unittest.main()
