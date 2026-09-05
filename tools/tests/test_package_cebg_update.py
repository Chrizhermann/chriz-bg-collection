import base64
import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class PackageCebgUpdateTests(unittest.TestCase):
    def setUp(self) -> None:
        self.root = Path(__file__).resolve().parents[2]
        self.script = self.root / "tools/package-cebg-update.ps1"

    def _run(self, published_name: str | None = None) -> tuple[dict, str]:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            setup = root / "Chriz Easy BG_0.1.0-alpha.10_x64-setup.exe"
            signature = root / f"{setup.name}.sig"
            output = root / "feed"
            setup.write_bytes(b"fake signed setup")
            signature.write_text(base64.b64encode(b"fake signature").decode(), encoding="ascii")
            command = [
                "pwsh", "-NoProfile", "-File", str(self.script),
                "-Version", "0.1.0-alpha.10", "-RecipeVersion", "0.1.0-alpha.11",
                "-SetupPath", str(setup), "-SignaturePath", str(signature),
                "-OutputDirectory", str(output),
            ]
            if published_name is not None:
                command.extend(["-PublishedSetupFilename", published_name])
            completed = subprocess.run(command, capture_output=True, text=True)
            if completed.returncode != 0:
                self.fail(f"packaging script failed:\n{completed.stderr}\n{completed.stdout}")
            return (
                json.loads((output / "latest.json").read_text(encoding="utf-8")),
                (output / "SHA256SUMS").read_text(encoding="utf-8"),
            )

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
        self.assertTrue(feed["platforms"]["windows-x86_64"]["url"].endswith(f"/{published}"))
        self.assertIn(f"  {published}\n", sums)


if __name__ == "__main__":
    unittest.main()
