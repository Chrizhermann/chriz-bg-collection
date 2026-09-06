import base64
import hashlib
import importlib.util
import io
from pathlib import Path
import tarfile
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "generate-third-party-notices.py"
SPEC = importlib.util.spec_from_file_location("generate_notices", SCRIPT)
notices = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(notices)


def make_archive(files: dict[str, bytes]) -> bytes:
    stream = io.BytesIO()
    with tarfile.open(fileobj=stream, mode="w:gz") as archive:
        for name, data in files.items():
            info = tarfile.TarInfo(name)
            info.size = len(data)
            archive.addfile(info, io.BytesIO(data))
    return stream.getvalue()


class ThirdPartyNoticeTests(unittest.TestCase):
    def test_includes_nested_licenses_and_unrar_acknowledgments_without_payloads(self):
        data = make_archive({
            "example-1.0.0/Cargo.toml": b'[package]\nname="example"\nversion="1.0.0"\n',
            "example-1.0.0/LICENSE": b"root license\n",
            "example-1.0.0/src/polyfill/LICENSE-MIT": b"nested license\n",
            "example-1.0.0/vendor/unrar/acknow.txt": b"Intel copyright notice\n",
            "example-1.0.0/vendor/unrar/payload.cpp": b"not a licensing input",
        })
        actual = notices.archive_members(data, "example-1.0.0")
        self.assertEqual(set(actual), {
            "Cargo.toml", "LICENSE", "src/polyfill/LICENSE-MIT", "vendor/unrar/acknow.txt",
        })

    def test_rejects_archive_paths_outside_the_package(self):
        data = make_archive({"example-1.0.0/../LICENSE": b"text"})
        with self.assertRaisesRegex(notices.NoticeError, "Unsafe package"):
            notices.archive_members(data, "example-1.0.0")

    def test_cargo_cache_must_match_the_lockfile_checksum(self):
        with tempfile.TemporaryDirectory() as temporary:
            cache = Path(temporary) / "registry/cache/test-registry"
            cache.mkdir(parents=True)
            data = make_archive({"example-1.0.0/LICENSE": b"license"})
            (cache / "example-1.0.0.crate").write_bytes(data)
            self.assertEqual(notices.cargo_archive(Path(temporary), "example", "1.0.0", notices.sha256(data)), data)
            with self.assertRaisesRegex(notices.NoticeError, "checksum-matching"):
                notices.cargo_archive(Path(temporary), "example", "1.0.0", "0" * 64)

    def test_npm_cache_content_is_checked_against_lockfile_integrity(self):
        data = b"cached npm archive"
        digest = hashlib.sha512(data).digest()
        integrity = "sha512-" + base64.b64encode(digest).decode()
        digest_hex = digest.hex()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            target = root / "_cacache/content-v2/sha512" / digest_hex[:2] / digest_hex[2:4] / digest_hex[4:]
            target.parent.mkdir(parents=True)
            target.write_bytes(data)
            self.assertEqual(notices.npm_archive(root, integrity, "example"), data)
            target.write_bytes(b"tampered")
            with self.assertRaisesRegex(notices.NoticeError, "integrity mismatch"):
                notices.npm_archive(root, integrity, "example")

    def test_missing_license_text_fails_instead_of_emitting_a_blank_notice(self):
        with self.assertRaisesRegex(notices.NoticeError, "No license text"):
            notices.collect_notices(Path("."), "cargo:example@1.0.0", {}, "checksum", {})

    def test_supplement_is_bound_to_both_package_and_license_bytes(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "LICENSES").mkdir()
            data = b"Copyright original author\r\nLicense text\r\n"
            (root / "LICENSES/license.txt").write_bytes(data)
            supplement = {"cargo:example@1.0.0": {
                "package_checksum": "reviewed-package",
                "notices": [{"file": "license.txt", "sha256": notices.sha256(data), "source": "https://example.org/license"}],
            }}
            actual = notices.collect_notices(root, "cargo:example@1.0.0", {}, "reviewed-package", supplement)
            self.assertEqual(actual[0][1], "Copyright original author\nLicense text\n")
            with self.assertRaisesRegex(notices.NoticeError, "different package bytes"):
                notices.collect_notices(root, "cargo:example@1.0.0", {}, "new-package", supplement)
            (root / "LICENSES/license.txt").write_bytes(data.replace(b"original", b"changed"))
            with self.assertRaisesRegex(notices.NoticeError, "checksum mismatch"):
                notices.collect_notices(root, "cargo:example@1.0.0", {}, "reviewed-package", supplement)

    def test_regeneration_preserves_reviewed_manual_sections_and_is_idempotent(self):
        manual = "# Notices\n\n## UnRAR\nOriginal required text.\n\n## EET\nUpstream attribution.\n"
        generated = notices.BEGIN + "\nNew generated notices\n" + notices.END + "\n"
        first = notices.replace_generated(manual, generated)
        self.assertTrue(first.startswith(manual))
        self.assertEqual(notices.replace_generated(first, generated), first)
        self.assertIn("Original required text.", first)
        self.assertIn("Upstream attribution.", first)


if __name__ == "__main__":
    unittest.main()
