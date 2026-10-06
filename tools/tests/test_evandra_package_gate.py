"""The normal package paths must enforce approved defaults before output."""

import json
from contextlib import redirect_stderr, redirect_stdout
from io import StringIO
from pathlib import Path
import shutil
import tempfile
import tomllib
import unittest

from tools.evandra_defaults import main as check_defaults

ROOT = Path(__file__).resolve().parents[2]


class EvandraPackageGateTests(unittest.TestCase):
    def test_ready_bundled_recipe_passes_the_publication_command(self):
        recipe = ROOT / "recipes/curated-full-current"
        output, errors = StringIO(), StringIO()
        with redirect_stdout(output), redirect_stderr(errors):
            result = check_defaults(["--recipe", str(recipe)])
        self.assertEqual(result, 0, errors.getvalue())
        self.assertEqual(errors.getvalue(), "")
        self.assertIn("Evandra recipe default contract passed", output.getvalue())
        self.assertIn("archive and gameplay acceptance are separate", output.getvalue())

    def test_publication_command_rejects_synthetic_alpha7_even_with_new_catalogue(self):
        with tempfile.TemporaryDirectory() as temporary:
            recipe = Path(temporary)
            for relative in ("collection.toml", "mods/chriz-bg-modpack.toml", "presets/chris-recommended.toml"):
                target = recipe / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT / "recipes/curated-full-current" / relative, target)
            catalogue = recipe / "mods/chriz-bg-modpack.toml"
            original = catalogue.read_text(encoding="utf-8")
            current_pin = tomllib.loads(original)["artifact_id"]
            self.assertNotEqual(current_pin, "chriz-bg-modpack-0.2.0-alpha.7")
            catalogue.write_text(original.replace(
                f'artifact_id = "{current_pin}"',
                'artifact_id = "chriz-bg-modpack-0.2.0-alpha.7"',
            ), encoding="utf-8")
            self.assertEqual(tomllib.loads(catalogue.read_text())["artifact_id"],
                             "chriz-bg-modpack-0.2.0-alpha.7")
            output, errors = StringIO(), StringIO()
            with redirect_stdout(output), redirect_stderr(errors):
                result = check_defaults(["--recipe", str(recipe)])
            self.assertEqual(result, 1)
            self.assertEqual(output.getvalue(), "")
            self.assertIn("not ready for public packaging", errors.getvalue())
            self.assertIn("224 (Sorcerer) and 225 (custom portrait)", errors.getvalue())

    def test_normal_tauri_package_checks_the_bundled_curated_recipe_first(self):
        config = json.loads((ROOT / "app/src-tauri/tauri.conf.json").read_text())
        package = json.loads((ROOT / "app/package.json").read_text())
        self.assertEqual(
            "npm run check:recipe-defaults && npm run build",
            config["build"]["beforeBuildCommand"],
        )
        self.assertEqual(
            "cd .. && python -m tools.evandra_defaults --recipe recipes/curated-full-current",
            package["scripts"]["check:recipe-defaults"],
        )
        self.assertIn("../../recipes/curated-full-current/", config["bundle"]["resources"])

    def test_signed_recipe_package_checks_its_input_before_signing(self):
        script = (ROOT / "tools/package-recipe.ps1").read_text()
        guard = script.index("& python -m tools.evandra_defaults --recipe $resolvedRecipe")
        key_read = script.index("$resolvedKey =")
        payload = script.index("-- package-recipe")
        signing = script.index("& $minisign.Source -S")
        self.assertLess(guard, key_read)
        self.assertLess(key_read, payload)
        self.assertLess(payload, signing)
        self.assertIn("if ($LASTEXITCODE -ne 0)", script[guard:key_read])


if __name__ == "__main__":
    unittest.main()
