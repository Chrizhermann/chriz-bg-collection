from __future__ import annotations

import hashlib
import tempfile
import unittest
import zipfile
from pathlib import Path

from tools.check_native_component_order import (
    check_archive_component_order, literal_component_order, validate_run_order,
)


class NativeComponentOrderTests(unittest.TestCase):
    def test_native_declarations_not_numeric_ids_determine_order(self) -> None:
        # Shape regression from Rebalance v0.7.0: Flail is declared AFTER Tempus.
        source = """BEGIN @301 DESIGNATED 301
BEGIN @400 DESIGNATED 400
BEGIN @408 DESIGNATED 408
BEGIN @302 DESIGNATED 302
"""
        order = literal_component_order(source)
        self.assertEqual(order, [301, 400, 408, 302])
        validate_run_order({"run_id": "rebalance", "components": [301, 408, 302]}, order)
        with self.assertRaisesRegex(ValueError, "pinned TP2 installs"):
            validate_run_order({"run_id": "rebalance", "components": [301, 302, 408]}, order)

    def test_comments_and_strings_cannot_invent_components(self) -> None:
        source = '''// BEGIN ~fake~ DESIGNATED 111
/* nested /* comment */ BEGIN ~fake~ DESIGNATED 112 */
BEGIN ~real~ DESIGNATED 100
PRINT "BEGIN bogus DESIGNATED 188"
BEGIN @110 DESIGNATED 110
'''
        self.assertEqual(literal_component_order(source), [100, 110])
        self.assertEqual(literal_component_order(
            'BEGIN ~one~ DESIGNATED 1\nPRINT ~~~~~BEGIN "fake" DESIGNATED 7~~~~~'
        ), [1])

    def test_unknown_dynamic_duplicate_or_unterminated_declarations_fail(self) -> None:
        for source in (
            'BEGIN ~implicit~',
            'BEGIN ~dynamic~ DESIGNATED %component%',
            'BEGIN ~computed~ DESIGNATED 100 + 2',
            'BEGIN ~one~ DESIGNATED 1\nBEGIN ~two~ DESIGNATED 1',
            'BEGIN ~one~ DESIGNATED 1 DESIGNATED 2',
            'BEGIN ~unterminated',
            '/* unterminated',
            'PRINT ~no components~',
        ):
            with self.subTest(source=source), self.assertRaises(ValueError):
                literal_component_order(source)
        for components in ([301, 301], [999]):
            with self.subTest(components=components), self.assertRaises(ValueError):
                validate_run_order({"run_id": "rebalance", "components": components}, [301])

    def _fixture(self, root: Path, components: list[int]) -> tuple[Path, Path]:
        recipe = root / "recipe"
        (recipe / "mods").mkdir(parents=True)
        (recipe / "artifacts").mkdir()
        archive_path = root / "fixture.zip"
        with zipfile.ZipFile(archive_path, "w") as archive:
            archive.writestr("setup-example.tp2", (
                "BEGIN @301 DESIGNATED 301\n"
                "BEGIN @408 DESIGNATED 408\n"
                "BEGIN @302 DESIGNATED 302\n"
            ))
        data = archive_path.read_bytes()
        (recipe / "mods/example.toml").write_text(
            'artifact_id = "example-v1"\ntp2 = "setup-example.tp2"\n', encoding="utf-8",
        )
        (recipe / "artifacts/example-v1.toml").write_text(
            f'''[source]
expected_length = {len(data)}
sha256 = "{hashlib.sha256(data).hexdigest()}"
[archive]
kind = "zip"
root_rule = "direct"
[archive.limits]
max_entry_uncompressed_bytes = 4096
''', encoding="utf-8",
        )
        (recipe / "collection.toml").write_text(
            f'''[[runs]]
run_id = "example-bg2"
mod_id = "example"
components = {components}
''', encoding="utf-8",
        )
        return recipe, archive_path

    def test_checker_reads_verified_archive_instead_of_an_expected_array(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            recipe, archive = self._fixture(Path(temp), [301, 408, 302])
            result = check_archive_component_order(recipe, "example", archive)
            self.assertEqual(result["native_components"], [301, 408, 302])
            self.assertEqual(result["checked_runs"], ["example-bg2"])
            self.assertEqual(len(result["tp2_sha256"]), 64)

    def test_valid_pin_does_not_hide_recipe_order_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            recipe, archive = self._fixture(Path(temp), [301, 302, 408])
            with self.assertRaisesRegex(ValueError, "incorrect component order"):
                check_archive_component_order(recipe, "example", archive)

    def test_unverified_archive_cannot_supply_order_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            recipe, archive = self._fixture(Path(temp), [301, 408, 302])
            data = archive.read_bytes()
            archive.write_bytes(data[:-1] + bytes([data[-1] ^ 1]))
            with self.assertRaisesRegex(ValueError, "SHA-256"):
                check_archive_component_order(recipe, "example", archive)
            archive.write_bytes(data + b"extra")
            with self.assertRaisesRegex(ValueError, "length"):
                check_archive_component_order(recipe, "example", archive)


if __name__ == "__main__":
    unittest.main()
