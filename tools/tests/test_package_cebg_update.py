import base64
import json
import subprocess
import tempfile
import unittest
from pathlib import Path


def quote(value: str) -> str:
    """A PowerShell single-quoted string literal."""
    return "'" + value.replace("'", "''") + "'"


class PackageCebgUpdateTests(unittest.TestCase):
    def setUp(self) -> None:
        self.root = Path(__file__).resolve().parents[2]
        self.script = self.root / "tools/package-cebg-update.ps1"

    def _invoke(
        self,
        setup_name: str = "Chriz Easy BG_0.1.0-alpha.10_x64-setup.exe",
        version: str = "0.1.0-alpha.10",
        extra: tuple[str, ...] = (),
        assets: dict[str, bytes] | None = None,
    ) -> tuple[subprocess.CompletedProcess, dict | None, str | None]:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            setup = root / setup_name
            signature = root / f"{setup.name}.sig"
            output = root / "feed"
            setup.write_bytes(b"fake signed setup")
            signature.write_text(base64.b64encode(b"fake signature").decode(), encoding="ascii")
            arguments = [
                "-Version", version, "-RecipeVersion", "0.1.0-alpha.11",
                "-SetupPath", str(setup), "-SignaturePath", str(signature),
                "-OutputDirectory", str(output), *extra,
            ]
            command = [
                "&", quote(str(self.script)),
                *(argument if argument.startswith("-") else quote(argument) for argument in arguments),
            ]
            if assets:
                for name, content in assets.items():
                    (root / name).write_bytes(content)
                # An array parameter, as a release builder passes it from PowerShell.
                command.extend(["-AdditionalAssets", ", ".join(quote(str(root / name)) for name in assets)])
            completed = subprocess.run(
                ["pwsh", "-NoProfile", "-NonInteractive", "-Command", " ".join(command)],
                capture_output=True, text=True,
            )
            if completed.returncode != 0:
                return completed, None, None
            return (
                completed,
                json.loads((output / "latest.json").read_text(encoding="utf-8")),
                (output / "SHA256SUMS").read_text(encoding="utf-8"),
            )

    def _run(self, published_name: str | None = None, **kwargs) -> tuple[dict, str]:
        extra = kwargs.pop("extra", ())
        if published_name is not None:
            extra = (*extra, "-PublishedSetupFilename", published_name)
        completed, feed, sums = self._invoke(extra=extra, **kwargs)
        if feed is None or sums is None:
            self.fail(f"packaging script failed:\n{completed.stderr}\n{completed.stdout}")
        return feed, sums

    def test_explicit_published_filename_drives_feed_and_checksum_names(self) -> None:
        published = "Chriz.Easy.BG_0.1.0-alpha.10_x64-setup.exe"
        feed, sums = self._run(published)
        self.assertTrue(feed["platforms"]["windows-x86_64"]["url"].endswith(f"/{published}"))
        self.assertIn(f"  {published}\n", sums)
        self.assertIn(f"  {published}.sig\n", sums)
        self.assertNotIn("Chriz Easy BG", sums)

    def test_default_published_filename_normalizes_spaces_to_dots(self) -> None:
        feed, sums = self._run()
        published = "Chriz.Easy.BG_0.1.0-alpha.10_x64-setup.exe"
        self.assertEqual(
            feed["platforms"]["windows-x86_64"]["url"],
            f"https://github.com/Chrizhermann/chriz-easy-bg/releases/download/v0.1.0-alpha.10/{published}",
        )
        self.assertEqual(feed["notes"], "Chriz Easy BG 0.1.0-alpha.10.")
        self.assertIn(f"  {published}\n", sums)

    def test_iwd_feed_points_to_the_iwd_releases_and_lists_every_asset(self) -> None:
        feed, sums = self._run(
            setup_name="Chriz Easy IWD_0.1.7_x64-setup.exe", version="0.1.7",
            extra=("-Flavor", "iwd"),
            assets={"LICENSE": b"license text", "THIRD_PARTY_NOTICES.md": b"notices"},
        )
        published = "Chriz.Easy.IWD_0.1.7_x64-setup.exe"
        self.assertEqual(feed["version"], "0.1.7")
        self.assertEqual(feed["notes"], "Chriz Easy IWD 0.1.7.")
        self.assertEqual(
            feed["platforms"]["windows-x86_64"]["url"],
            f"https://github.com/Chrizhermann/chriz-easy-iwd/releases/download/v0.1.7/{published}",
        )
        names = [line.split("  ", 1)[1] for line in sums.splitlines()]
        self.assertEqual(
            names,
            [published, f"{published}.sig", "latest.json", "LICENSE", "THIRD_PARTY_NOTICES.md"],
        )

    def test_a_setup_of_another_app_or_version_is_refused(self) -> None:
        for setup_name, version, flavor in [
            ("Chriz Easy BG_0.1.7_x64-setup.exe", "0.1.7", "iwd"),
            ("Chriz Easy IWD_0.1.7_x64-setup.exe", "0.1.7", "bg"),
            ("Chriz Easy IWD_0.1.6_x64-setup.exe", "0.1.7", "iwd"),
        ]:
            with self.subTest(setup=setup_name, flavor=flavor):
                completed, feed, _ = self._invoke(
                    setup_name=setup_name, version=version, extra=("-Flavor", flavor),
                )
                self.assertNotEqual(completed.returncode, 0)
                self.assertIsNone(feed)
                self.assertIn("SetupPath must name the", completed.stderr + completed.stdout)

    def test_published_names_must_not_collide(self) -> None:
        completed, feed, _ = self._invoke(assets={"latest.json": b"{}"})
        self.assertNotEqual(completed.returncode, 0)
        self.assertIsNone(feed)
        self.assertIn("Every published release file needs its own name.", completed.stderr + completed.stdout)


if __name__ == "__main__":
    unittest.main()
