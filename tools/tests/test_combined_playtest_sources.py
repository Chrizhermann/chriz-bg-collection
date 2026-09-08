from __future__ import annotations

import hashlib
import importlib.util
import json
import tempfile
import unittest
import zipfile
from pathlib import Path


MODULE_PATH = Path(__file__).parents[1] / "prepare_combined_playtest_sources.py"
SPEC = importlib.util.spec_from_file_location("combined_sources", MODULE_PATH)
assert SPEC and SPEC.loader
combined_sources = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(combined_sources)


class CombinedPlaytestSourceTests(unittest.TestCase):
    def test_zip_is_deterministic_sorted_and_measured(self) -> None:
        payloads = {
            "mod/z.txt": b"z" * 50,
            "setup-mod.tp2": b"BEGIN test\n",
            "mod/a.bin": bytes(range(32)),
        }
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            first = root / "first.zip"
            second = root / "second.zip"
            first_metrics = combined_sources.write_deterministic_zip(first, payloads)
            second_metrics = combined_sources.write_deterministic_zip(second, payloads)

            self.assertEqual(first.read_bytes(), second.read_bytes())
            self.assertEqual(hashlib.sha256(first.read_bytes()).hexdigest(), first_metrics["sha256"])
            self.assertEqual(first_metrics, second_metrics)
            self.assertEqual(first_metrics["entry_count"], 3)
            self.assertEqual(first_metrics["max_entry_bytes"], 50)
            self.assertEqual(first_metrics["max_depth"], 2)
            self.assertEqual(first_metrics["uncompressed_bytes"], 93)
            self.assertGreaterEqual(first_metrics["max_compression_ratio"], 2)
            with zipfile.ZipFile(first) as archive:
                self.assertEqual(archive.namelist(), sorted(payloads, key=str.casefold))
                self.assertTrue(all(info.date_time == (2026, 9, 8, 0, 0, 0) for info in archive.infolist()))
                self.assertTrue(all(info.external_attr >> 16 == 0o100644 for info in archive.infolist()))

    def test_zip_rejects_unsafe_and_case_colliding_paths(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            destination = Path(raw) / "bad.zip"
            for payloads in (
                {"../escape": b"bad"},
                {"Mod/file": b"one", "mod/FILE": b"two"},
                {"/absolute": b"bad"},
            ):
                with self.subTest(payloads=payloads):
                    with self.assertRaises(ValueError):
                        combined_sources.write_deterministic_zip(destination, payloads)

    def test_overlay_hash_is_pinned(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            path = Path(raw) / "overlay.tpa"
            path.write_bytes(b"reviewed")
            self.assertEqual(
                combined_sources.read_pinned_file(path, hashlib.sha256(b"reviewed").hexdigest()),
                b"reviewed",
            )
            with self.assertRaisesRegex(ValueError, "overlay hash mismatch"):
                combined_sources.read_pinned_file(path, "0" * 64)

    def test_safana_runtime_strips_only_the_official_wrapper(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            path = Path(raw) / "safana.zip"
            with zipfile.ZipFile(path, "w") as archive:
                archive.writestr("SafanaBG2-05/Safana/Safana.tp2", b"BEGIN test")
                archive.writestr("SafanaBG2-05/Safana/lib/runtime.tpa", b"runtime")
                archive.writestr("SafanaBG2-05/readme.md", b"not runtime")
            original = combined_sources.SAFANA_ARCHIVE_SHA256
            try:
                combined_sources.SAFANA_ARCHIVE_SHA256 = hashlib.sha256(path.read_bytes()).hexdigest()
                self.assertEqual(
                    combined_sources.safana_payloads(path),
                    {
                        "Safana/Safana.tp2": b"BEGIN test",
                        "Safana/lib/runtime.tpa": b"runtime",
                    },
                )
            finally:
                combined_sources.SAFANA_ARCHIVE_SHA256 = original

    def test_sources_document_uses_stable_schema(self) -> None:
        source = combined_sources.source_record(
            source_id="example",
            archive="example.zip",
            commit="a" * 40,
            source_path="C:/source",
            publish_roots=["example"],
            tp2_paths=["example/setup-example.tp2"],
            root_layout=["example/"],
            dirty_overlay=[],
            provenance={
                "homepage": "https://example.invalid/project",
                "license": "Local test snapshot; do not redistribute",
                "provenance_url": "https://example.invalid/project/commit/" + "a" * 40,
            },
            metrics={
                "sha256": "b" * 64,
                "size_bytes": 12,
                "entry_count": 1,
                "max_entry_bytes": 3,
                "max_depth": 2,
                "uncompressed_bytes": 3,
                "max_compression_ratio": 2,
            },
        )
        document = json.loads(combined_sources.render_sources_json([source]))
        self.assertEqual(document["schema_version"], 1)
        self.assertEqual(document["sources"][0]["root_rule"], "direct")
        self.assertEqual(document["sources"][0]["provenance"]["homepage"], "https://example.invalid/project")


if __name__ == "__main__":
    unittest.main()
