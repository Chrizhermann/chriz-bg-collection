"""Synthetic checks of tools/prepare_own_mod_pin.py.

The recipes, release archives and GitHub answers are made up, and small fakes stand in for
the engine's author CLI, the native order check and the curated recipe generator, so these
tests need no network and no engine build. The fixture mods are named Test*, so that no
real pin ever finds this file among the tests that freeze it.
"""

from __future__ import annotations

import hashlib
import io
import json
import sys
import tempfile
import tomllib
import unittest
import zipfile
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from unittest import mock

from tools import prepare_own_mod_pin
from tools.own_mod_pins import GitHubError, load_registry
from tools.prepare_own_mod_pin import (
    CURATED,
    CURATED_ACCEPTANCE,
    NeedsAgent,
    Outcome,
    prepare,
    previous_native_profiles,
    set_toml_value,
    summary,
)


ACCOUNT = "Chrizhermann"
TESTBOT = f"{ACCOUNT}/testbot"
TESTREMOTE = f"{ACCOUNT}/testremote"
TESTVOICES = f"{ACCOUNT}/testvoices"
TESTENCOUNTERS = f"{ACCOUNT}/testencounters"
IWD = "recipes/iwd-current"
IWD_ACCEPTANCE = f"{IWD}/releases/v0.1.0-alpha.1/acceptance.toml"
GENERATOR_COMMIT = "e" * 40
REMOTE_OLD = "1" * 40
REMOTE_NEW = "2" * 40
BOT_COMMITS = {"v1.1.0": "3" * 40, "v1.1.5": "4" * 40, "v1.2.0": "5" * 40}
TODAY = "2026-11-02"
DATE = TODAY.replace("-", "")
COMPONENTS = [(0, "Testbot"), (1, "Testbot extras")]


FAKE_AUTHOR = r'''
"""Fake `chriz-bg-author artifact inspect|verify`; argv[1] is its JSON configuration."""
import hashlib
import json
import re
import sys
import tomllib
import zipfile
from pathlib import Path

config = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
action, target, *options = sys.argv[3:]
cache = Path(options[options.index("--cache-root") + 1])
expected = [options[index + 1] for index, value in enumerate(options) if value == "--expected-tp2"]


def download(url):
    if url not in config["downloads"]:
        sys.exit(f"no download at {url}")
    return config["downloads"][url]


if action == "inspect":
    entry = download(target)
    data = Path(entry["zip"]).read_bytes()
    cache.mkdir(parents=True, exist_ok=True)
    quarantined = cache / "download.zip"
    quarantined.write_bytes(data)
    with zipfile.ZipFile(quarantined) as archive:
        infos = archive.infolist()
        tp2s = {info.filename: archive.read(info).decode("utf-8") for info in infos
                if info.filename.lower().endswith(".tp2")}
    files = [info.filename for info in infos if not info.is_dir()]
    print(json.dumps({
        "final_url": entry["final_url"],
        "length": len(data),
        "sha256": hashlib.sha256(data).hexdigest().upper(),
        "quarantined_path": str(quarantined),
        "archive": {
            "entry_count": len(infos),
            "entries": [{"path": info.filename, "is_directory": info.is_dir(),
                         "length": info.file_size, "compressed_length": info.compress_size}
                        for info in infos],
            "expected_tp2_matches": [
                {"expected": tp2, "matches": [name for name in files
                                              if name.lower() == tp2.lower()
                                              or name.lower().endswith("/" + tp2.lower())]}
                for tp2 in expected
            ],
        },
        "component_menu_evidence": [
            {"tp2_path": path, "component": int(number), "title": title}
            for path, text in tp2s.items()
            for title, number in re.findall(r"BEGIN ~([^~]*)~\s+DESIGNATED (\d+)", text)
        ],
        "version_evidence": [{"tp2_path": path, "value": value} for path, text in tp2s.items()
                             for value in re.findall(r"VERSION ~([^~]*)~", text)],
    }))
elif action == "verify":
    artifact = tomllib.loads(Path(target).read_text(encoding="utf-8"))
    source = artifact["source"]
    data = Path(download(source["url"])["zip"]).read_bytes()
    print(json.dumps({
        "verified": source["expected_length"] == len(data)
        and source["sha256"] == hashlib.sha256(data).hexdigest()
        and artifact["id"] not in config["refuse"],
        "archive": {"publish_roots": artifact["archive"]["publish_roots"]},
    }))
else:
    sys.exit(f"unknown action {action}")
'''

FAKE_ORDER_CHECK = r'''
"""Fake tools/check_native_component_order.py; order.json next to it lists refused recipes."""
import argparse
import hashlib
import json
import re
import sys
import tomllib
import zipfile
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--recipe-root", required=True)
parser.add_argument("--mod-id", required=True)
parser.add_argument("--archive", required=True)
args = parser.parse_args()
config = json.loads(Path(__file__).with_name("order.json").read_text(encoding="utf-8"))
recipe = Path(args.recipe_root)
if any(recipe.as_posix().endswith(refused) for refused in config["refuse"]):
    sys.exit(f"{args.mod_id}: the run order does not fit the archive")
collection = tomllib.loads((recipe / "collection.toml").read_text(encoding="utf-8"))
runs = [run["run_id"] for run in collection["runs"] if run["mod_id"] == args.mod_id]
with zipfile.ZipFile(args.archive) as archive:
    tp2 = archive.read(next(name for name in archive.namelist() if name.endswith(".tp2")))
print(json.dumps([{
    "tp2_sha256": hashlib.sha256(tp2).hexdigest(),
    "native_components": [int(number) for number in re.findall(rb"DESIGNATED (\d+)", tp2)],
    "checked_runs": runs,
}]))
'''

FAKE_GENERATOR = r'''
"""Fake tools/curated_full_recipe.py: copies manifest's mods and their artifacts."""
import argparse
import shutil
import sys
import tomllib
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--root", required=True)
parser.add_argument("--output", required=True)
parser.add_argument("--commit", required=True)
args = parser.parse_args()
root, output = Path(args.root), Path(args.output)
if (root / "generator-fails").exists():
    sys.exit("the generator stopped")
for mod in sorted((root / "manifest/mods").glob("*.toml")):
    artifact_id = tomllib.loads(mod.read_text(encoding="utf-8"))["artifact_id"]
    shutil.copyfile(mod, output / "mods" / mod.name)
    shutil.copyfile(root / "manifest/artifacts" / f"{artifact_id}.toml",
                    output / "artifacts" / f"{artifact_id}.toml")
(output / "GENERATED.txt").write_text(args.commit, encoding="utf-8")
'''


def tree_state(root: Path) -> dict[str, bytes]:
    return {path.relative_to(root).as_posix(): path.read_bytes()
            for path in sorted(root.rglob("*")) if path.is_file()}


def tp2_text(version: str, components: list[tuple[int, str]]) -> str:
    lines = ["BACKUP ~weidu_external/backup/testbot~", "AUTHOR ~Fixture~",
             f"VERSION ~{version}~", ""]
    for number, title in components:
        lines += [f"BEGIN ~{title}~", f"DESIGNATED {number}", ""]
    return "\n".join(lines)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def release_artifact(repository: str, tag: str, asset: str, archive: Path, *,
                     version: str | None = None) -> str:
    name = repository.split("/")[1]
    version = version or tag.removeprefix("v")
    return f'''id = "{name}-{version}"
name = "{name}"
version = "{version}"
acquisition = "fetch-only"

[source]
kind = "github-release"
url = "https://github.com/{repository}/releases/download/{tag}/{asset}"
reference = "{tag}"
expected_filename = "{asset}"
expected_length = {archive.stat().st_size}
sha256 = "{sha256(archive)}"
redirect_hosts = ["release-assets.githubusercontent.com"]

[archive]
kind = "zip"
root_rule = "direct"
publish_roots = ["{name}"]
tp2_paths = ["{name}/setup-{name}.tp2"]

[archive.limits]
max_depth = 8
max_entries = 256

[provenance]
homepage = "https://github.com/{repository}"
license = "Fetch-only release"
url = "https://github.com/{repository}/releases/tag/{tag}"
reviewed_on = "2026-10-01"
'''


def commit_artifact(version: str, commit: str, archive: Path) -> str:
    return f'''id = "testremote-{version}"
name = "Testremote"
version = "{version}"
acquisition = "fetch-only"

[source]
kind = "github-commit-zip"
url = "https://codeload.github.com/{TESTREMOTE}/zip/{commit}"
reference = "{commit}"
expected_filename = "testremote-{commit}.zip"
expected_length = {archive.stat().st_size}
sha256 = "{sha256(archive)}"
redirect_hosts = []

[archive]
kind = "zip"
root_rule = "single-wrapper"
publish_roots = ["testremote", "setup-testremote.tp2"]
tp2_paths = ["setup-testremote.tp2"]

[provenance]
homepage = "https://github.com/{TESTREMOTE}"
license = "Fetch-only source commit"
url = "https://github.com/{TESTREMOTE}/commit/{commit}"
reviewed_on = "2026-10-01"
'''


class FakeGitHub:
    def __init__(self) -> None:
        self.latest: dict[str, str] = {}
        self.tag_lists: dict[str, list[tuple[str, str]]] = {}
        self.releases: dict[tuple[str, str], dict] = {}

    def latest_release(self, repository: str) -> str:
        if repository not in self.latest:
            raise GitHubError(f"{repository} has no Latest release")
        return self.latest[repository]

    def tags(self, repository: str) -> list[tuple[str, str]]:
        return self.tag_lists.get(repository, [])

    def release_by_tag(self, repository: str, tag: str) -> dict:
        if (repository, tag) not in self.releases:
            raise GitHubError(f"{repository} has no release {tag}")
        return self.releases[(repository, tag)]


class PinFixture:
    """A working tree with both installers, the releases on a fake GitHub, and the fakes.

    Standard state: Chriz Easy BG pins Testbot v1.1.0 and Testremote v0.1.0 (a commit zip),
    Chriz Easy IWD pins Testbot v1.1.5; Testbot v1.2.0 and Testremote v0.2.0 are newest."""

    def __init__(self, base: Path) -> None:
        self.root = base / "tree"
        self.tools = base / "fakes"
        self.archives = base / "archives"
        self.work = base / "work"
        for folder in (self.root, self.tools / "tools", self.archives, self.work):
            folder.mkdir(parents=True)
        self.github = FakeGitHub()
        self.downloads: dict[str, dict[str, str]] = {}
        self.refuse_verify: list[str] = []
        self.refuse_order: list[str] = []
        (self.tools / "fake_author.py").write_text(FAKE_AUTHOR, encoding="utf-8")
        (self.tools / "tools/check_native_component_order.py").write_text(FAKE_ORDER_CHECK,
                                                                         encoding="utf-8")
        self.author = [sys.executable, "-I", str(self.tools / "fake_author.py"),
                       str(self.tools / "author.json")]
        self.write("tools/curated_full_recipe.py", FAKE_GENERATOR)

    def write(self, relative: str, text: str) -> None:
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8", newline="\n")

    def read(self, relative: str) -> dict:
        return tomllib.loads((self.root / relative).read_text(encoding="utf-8"))

    def archive(self, files: dict[str, str]) -> Path:
        path = self.archives / f"{len(list(self.archives.iterdir()))}.zip"
        with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as archive:
            for member, text in files.items():
                archive.writestr(member, text)
        return path

    def bot_archive(self, tag: str, components: list[tuple[int, str]] = COMPONENTS) -> Path:
        return self.archive({"testbot/setup-testbot.tp2": tp2_text(tag, components),
                             "testbot/testbot.lua": f"-- Testbot {tag}\n"})

    def serve(self, url: str, archive: Path, host: str) -> None:
        self.downloads[url] = {"zip": str(archive), "final_url": f"https://{host}/{archive.name}"}

    def release(self, repository: str, tag: str, assets: dict[str, Path], *,
                latest: bool = False) -> None:
        self.github.tag_lists.setdefault(repository, []).append((tag, BOT_COMMITS[tag]))
        entries = []
        for name, archive in assets.items():
            url = f"https://github.com/{repository}/releases/download/{tag}/{name}"
            entries.append({"name": name, "size": archive.stat().st_size,
                            "digest": f"sha256:{sha256(archive)}", "browser_download_url": url})
            self.serve(url, archive, "release-assets.githubusercontent.com")
        self.github.releases[(repository, tag)] = {
            "tag_name": tag, "draft": False, "prerelease": False,
            "published_at": "2026-11-01T09:30:00Z", "assets": entries,
        }
        if latest:
            self.github.latest[repository] = tag

    def recipe(self, recipe: str, pins: dict[str, str], run_suffix: str) -> None:
        runs = []
        for mod_id, text in pins.items():
            artifact_id = tomllib.loads(text)["id"]
            self.write(f"{recipe}/mods/{mod_id}.toml",
                       f'id = "{mod_id}"\nartifact_id = "{artifact_id}"\n')
            self.write(f"{recipe}/artifacts/{artifact_id}.toml", text)
            runs.append(f'[[runs]]\nrun_id = "{mod_id}-{run_suffix}"\nmod_id = "{mod_id}"\n')
        self.write(f"{recipe}/collection.toml", "schema = 2\n\n" + "\n".join(runs))

    def standard(self, asset: str = "testbot-{tag}.zip", *, hold: bool = False,
                 iwd: bool = True) -> None:
        def name(tag: str) -> str:
            version = tag.removeprefix("v")
            return asset.format(tag=tag, dashed_tag=tag.replace(".", "-"), version=version,
                                dashed_version=version.replace(".", "-"))

        archives = {tag: self.bot_archive(tag) for tag in BOT_COMMITS}
        for tag, archive in archives.items():
            self.release(TESTBOT, tag, {name(tag): archive}, latest=tag == "v1.2.0")
        remote = {}
        for tag, commit in (("v0.1.0", REMOTE_OLD), ("v0.2.0", REMOTE_NEW)):
            remote[commit] = self.archive({
                f"testremote-{commit}/setup-testremote.tp2": tp2_text(tag, [(0, "Testremote")]),
                f"testremote-{commit}/testremote/init.lua": f"-- {tag}\n",
            })
            self.serve(f"https://codeload.github.com/{TESTREMOTE}/zip/{commit}", remote[commit],
                       "codeload.github.com")
        self.github.tag_lists[TESTREMOTE] = [("v0.1.0", REMOTE_OLD), ("v0.2.0", REMOTE_NEW),
                                             ("nightly", "f" * 40)]

        bg_pins = {
            "testbot": release_artifact(TESTBOT, "v1.1.0", name("v1.1.0"), archives["v1.1.0"]),
            "testremote": commit_artifact("0.1.0", REMOTE_OLD, remote[REMOTE_OLD]),
        }
        self.recipe("manifest", bg_pins, "bg2")
        self.recipe(CURATED, bg_pins, "bg2")
        self.write(CURATED_ACCEPTANCE, f'''[[evidence]]
subject_id = "testbot-bg2"
test_artifact = "tools/tests/test_curated_full_recipe.py"
commit = "{GENERATOR_COMMIT}"
''')
        if iwd:
            self.recipe(IWD, {"testbot": release_artifact(TESTBOT, "v1.1.5", name("v1.1.5"),
                                                          archives["v1.1.5"])}, "main")
            self.write(IWD_ACCEPTANCE, f'''[[evidence]]
subject_id = "testbot-main"
test_artifact = "engine/tests/production_recipe_iwd.rs"
commit = "{GENERATOR_COMMIT}"
''')
        self.registry(hold=hold)

    def registry(self, *, hold: bool = False) -> None:
        mods = "".join(f'''
[[mods]]
id = "{repository.split('/')[1]}"
name = "{repository.split('/')[1].title()}"
repository = "{repository}"
''' for repository in (TESTBOT, TESTVOICES, TESTENCOUNTERS))
        text = f'''schema = 1
account = "{ACCOUNT}"

[[installers]]
id = "bg"
name = "BG"
recipes = ["manifest", "{CURATED}"]

[[installers]]
id = "iwd"
name = "IWD"
recipes = ["{IWD}"]
superseded_artifacts = "remove"
{mods}
[[mods]]
id = "testremote"
name = "Testremote"
repository = "{TESTREMOTE}"
newest = "newest-tag"
'''
        if hold:
            text += '''
[[holds]]
mod = "testbot"
release = "v1.2.0"
reason = "Christopher tests it first."
decided = "2026-11-01"
installers = { bg = "v1.1.0", iwd = "v1.1.5" }
'''
        self.write("manifest/own-mods.toml", text)

    def prepare(self, mod: str = "testbot", *, tag: str | None = None,
                installers: list[str] | None = None) -> Outcome:
        (self.tools / "author.json").write_text(
            json.dumps({"downloads": self.downloads, "refuse": self.refuse_verify}),
            encoding="utf-8")
        (self.tools / "tools/order.json").write_text(json.dumps({"refuse": self.refuse_order}),
                                                     encoding="utf-8")
        registry = load_registry(self.root / "manifest/own-mods.toml")
        own_mod = next(entry for entry in registry.mods if entry.id == mod)
        return prepare(self.root, registry, own_mod, self.github, self.author, tag=tag,
                       installers=installers, today=TODAY, work=self.work,
                       tools_root=self.tools)


class PreparePinTests(unittest.TestCase):
    def fixture(self) -> PinFixture:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        return PinFixture(Path(directory.name))

    def assertNeedsAgent(self, fixture: PinFixture, message: str, **options) -> None:
        """The preparation stops for an agent and leaves the tree exactly as it was."""
        before = tree_state(fixture.root)
        with self.assertRaises(NeedsAgent) as raised:
            fixture.prepare(**options)
        self.assertIn(message, str(raised.exception))
        self.assertEqual(tree_state(fixture.root), before)

    def test_the_newest_release_goes_into_every_recipe_of_both_installers_at_once(self) -> None:
        fixture = self.fixture()
        fixture.standard(hold=True)
        old_bg = fixture.read("manifest/artifacts/testbot-1.1.0.toml")
        old_remote = fixture.read("manifest/mods/testremote.toml")
        new_archive = Path(fixture.downloads[
            f"https://github.com/{TESTBOT}/releases/download/v1.2.0/testbot-v1.2.0.zip"]["zip"])

        outcome = fixture.prepare()

        self.assertEqual(outcome.release.tag, "v1.2.0")
        self.assertEqual(outcome.old_releases, ["v1.1.0", "v1.1.5"])
        new = fixture.read("manifest/artifacts/testbot-1.2.0.toml")
        url = f"https://github.com/{TESTBOT}/releases/download/v1.2.0/testbot-v1.2.0.zip"
        self.assertEqual(new["id"], "testbot-1.2.0")
        self.assertEqual(new["version"], "1.2.0")
        self.assertEqual(new["source"], {
            **old_bg["source"],
            "url": url,
            "reference": "v1.2.0",
            "expected_filename": "testbot-v1.2.0.zip",
            "expected_length": new_archive.stat().st_size,
            "sha256": sha256(new_archive),
        })
        # The archive contract and everything else stays as the old pin had it.
        self.assertEqual(new["archive"], old_bg["archive"])
        self.assertEqual(new["provenance"], {
            **old_bg["provenance"],
            "url": f"https://github.com/{TESTBOT}/releases/tag/v1.2.0",
            "reviewed_on": TODAY,
        })
        self.assertEqual({key: new[key] for key in ("name", "acquisition")},
                         {key: old_bg[key] for key in ("name", "acquisition")})

        # Chriz Easy BG: manifest switched, the curated recipe regenerated with the commit
        # of its static test, the superseded artifact kept as history.
        for recipe in ("manifest", CURATED):
            self.assertEqual(fixture.read(f"{recipe}/mods/testbot.toml")["artifact_id"],
                             "testbot-1.2.0")
            self.assertEqual(fixture.read(f"{recipe}/artifacts/testbot-1.2.0.toml"), new)
            self.assertTrue((fixture.root / recipe / "artifacts/testbot-1.1.0.toml").is_file())
        self.assertEqual((fixture.root / CURATED / "GENERATED.txt").read_text(encoding="utf-8"),
                         GENERATOR_COMMIT)
        self.assertEqual(fixture.read("manifest/mods/testremote.toml"), old_remote)
        # Chriz Easy IWD: the new artifact replaces the superseded one.
        self.assertEqual(fixture.read(f"{IWD}/mods/testbot.toml")["artifact_id"],
                         "testbot-1.2.0")
        self.assertEqual(fixture.read(f"{IWD}/artifacts/testbot-1.2.0.toml")["source"]["url"],
                         url)
        self.assertFalse((fixture.root / IWD / "artifacts/testbot-1.1.5.toml").exists())

        evidence = f"manifest/evidence/release-intake-{DATE}-testbot-bg-iwd.toml"
        self.assertEqual(outcome.evidence, evidence)
        record = fixture.read(evidence)
        self.assertEqual(record["verified_on"], TODAY)
        self.assertEqual(record["generator"], "tools/prepare_own_mod_pin.py")
        self.assertIn("Testbot v1.2.0 (published 2026-11-01 09:30 UTC", record["scope"])
        self.assertIn("so do the superseded artifact TOMLs in manifest, "
                      f"{CURATED}; in {IWD} the new artifact TOML replaces", record["scope"])
        tp2 = tp2_text("v1.2.0", COMPONENTS).encode("utf-8")
        shared = {
            "mod_id": "testbot",
            "artifact_id": "testbot-1.2.0",
            "source_reference": "v1.2.0",
            "source_commit": BOT_COMMITS["v1.2.0"],
            "immutable_source_url": url,
            "actual_length": new_archive.stat().st_size,
            "actual_sha256": sha256(new_archive),
            "tp2_sha256": hashlib.sha256(tp2).hexdigest(),
            "native_components": [0, 1],
            "fresh_download": True,
            "fresh_extraction": True,
            "published_release_verified": True,
            "native_component_order_verified": True,
        }
        self.assertCountEqual(record["artifacts"], [
            {**shared, "recipe_profiles": ["manifest", CURATED]},
            {**shared, "recipe_profiles": [IWD]},
        ])

        text = summary(outcome)
        self.assertIn("`testbot-1.1.0` -> `testbot-1.2.0` | manifest, "
                      f"{CURATED} |", text)
        self.assertIn(f"`testbot-1.1.5` -> `testbot-1.2.0` | {IWD} |", text)
        self.assertIn("- TP2 VERSION: v1.1.0 -> v1.2.0.", text)
        self.assertIn("- TP2 VERSION: v1.1.5 -> v1.2.0.", text)
        self.assertIn("- Components: the same numbers and names as the previous pin.", text)
        self.assertIn("length and SHA-256 matches the digest GitHub publishes", text)
        self.assertIn("- Native component order, recipes/iwd-current, testbot: [0, 1] fits "
                      "testbot-main.", text)
        self.assertIn("- Remove or update Testbot's hold in `manifest/own-mods.toml`.", text)
        self.assertIn(f"- Check the static-test records of testbot-bg2 in `{CURATED_ACCEPTANCE}`",
                      text)
        self.assertIn(f"- Check the static-test records of testbot-main in `{IWD_ACCEPTANCE}`",
                      text)
        self.assertIn("never merges by itself", text)

    def test_the_changed_files_are_listed_in_order(self) -> None:
        fixture = self.fixture()
        fixture.standard()
        outcome = fixture.prepare()
        evidence = f"manifest/evidence/release-intake-{DATE}-testbot-bg-iwd.toml"
        self.assertCountEqual(outcome.changed_files, [
            "manifest/artifacts/testbot-1.2.0.toml",
            "manifest/mods/testbot.toml",
            f"{CURATED}/ (regenerated)",
            f"{IWD}/artifacts/testbot-1.2.0.toml",
            f"{IWD}/mods/testbot.toml",
            f"{IWD}/artifacts/testbot-1.1.5.toml (removed)",
            evidence,
        ])
        self.assertEqual(outcome.changed_files[-1], evidence)
        self.assertLess(outcome.changed_files.index("manifest/mods/testbot.toml"),
                        outcome.changed_files.index(f"{CURATED}/ (regenerated)"))

    def test_only_the_chosen_installer_changes(self) -> None:
        fixture = self.fixture()
        fixture.standard()
        before = tree_state(fixture.root)
        outcome = fixture.prepare(installers=["iwd"])
        after = tree_state(fixture.root)
        changed = sorted(path for path in set(before) | set(after)
                         if before.get(path) != after.get(path))
        evidence = f"manifest/evidence/release-intake-{DATE}-testbot-iwd.toml"
        self.assertEqual(changed, sorted([
            f"{IWD}/artifacts/testbot-1.1.5.toml",
            f"{IWD}/artifacts/testbot-1.2.0.toml",
            f"{IWD}/mods/testbot.toml",
            evidence,
        ]))
        self.assertEqual(outcome.evidence, evidence)
        self.assertIn("in recipes/iwd-current the new artifact TOML replaces the superseded one",
                      fixture.read(evidence)["scope"])

    def test_the_follow_ups_name_the_files_of_the_installers_it_changed(self) -> None:
        def with_documents() -> PinFixture:
            fixture = self.fixture()
            fixture.standard()
            old_url = f"https://github.com/{TESTBOT}/releases/download/v1.1.0/testbot-v1.1.0.zip"
            fixture.write("docs/curation/components/TESTBOT.md", "Pinned: v1.1.0, IWD v1.1.5.\n")
            fixture.write("docs/curation/components/FOLLOW_UPS.md", "Testbot v1.1.0 is pinned.\n")
            fixture.write("docs/plans/2026-10-01-next-release-changelogs.md", "# Drafts\n")
            fixture.write("docs/plans/2026-10-14-next-release-changelogs.md", "# Drafts\n")
            fixture.write("docs/pin-list-2.7.md", f"| Testbot | 1.1.0 | [asset]({old_url}) |\n")
            fixture.write("engine/tests/feature_groups.rs", "// plan digests\n")
            fixture.write("engine/tests/production_recipe_tail.rs", '"testbot-1.1.0"\n')
            fixture.write("engine/tests/production_recipe_iwd.rs", '"testbot-1.1.5"\n')
            intake = "manifest/evidence/release-intake-"
            fixture.write("tools/tests/test_early_intake.py", f'E = "{intake}20261001.toml"\n')
            fixture.write("tools/tests/test_late_intake.py", f'E = "{intake}20261020-x.toml"\n')
            fixture.write("tools/tests/test_zz_metadata.py", f'E = "{intake}20260901.toml"\n')
            return fixture

        bg = with_documents().prepare(installers=["bg"]).followups
        self.assertEqual(bg, [
            "Read the release notes and the TP2 changes above.",
            "Update the component catalog: `docs/curation/components/TESTBOT.md`.",
            "Add the changelog draft entries in "
            "`docs/plans/2026-10-14-next-release-changelogs.md`.",
            "Update the tests that freeze the old pin: `engine/tests/production_recipe_tail.rs`.",
            "Add an intake test that freezes the new pin and its evidence record, like "
            "`tools/tests/test_late_intake.py`.",
            "Re-pin the Recommended plan digests in `engine/tests/feature_groups.rs`.",
            f"Check the static-test records of testbot-bg2 in `{CURATED_ACCEPTANCE}` by this "
            "installer's convention (`docs/own-mod-pins.md`).",
            "Regenerate `docs/pin-list-2.7.md`: `CEBG_UPDATE_PIN_LIST=1 cargo test --locked -p "
            "chriz-bg-engine --test pin_list`.",
            "Run the full Rust and Python suites and `tools/own_mod_pins.py check`.",
            "Waits for Christopher's test; it never merges by itself.",
        ])
        # The catalogs, the changelog draft, the plan digests and the pin list are Chriz Easy
        # BG's; a pin of Chriz Easy IWD alone leaves them alone.
        iwd = with_documents().prepare(installers=["iwd"]).followups
        self.assertEqual(iwd, [
            "Read the release notes and the TP2 changes above.",
            "Update the tests that freeze the old pin: `engine/tests/production_recipe_iwd.rs`.",
            f"Check the static-test records of testbot-main in `{IWD_ACCEPTANCE}` by this "
            "installer's convention (`docs/own-mod-pins.md`).",
            "Run the full Rust and Python suites and `tools/own_mod_pins.py check`.",
            "Waits for Christopher's test; it never merges by itself.",
        ])

    def test_the_asset_name_follows_the_form_of_the_old_one(self) -> None:
        for form, expected in (
            ("testbot-{tag}.zip", "testbot-v1.2.0.zip"),
            ("Testbot-{dashed_tag}.zip", "Testbot-v1-2-0.zip"),
            ("testbot-{version}.iemod", "testbot-1.2.0.iemod"),
            ("testbot_{dashed_version}.zip", "testbot_1-2-0.zip"),
        ):
            with self.subTest(form=form):
                fixture = self.fixture()
                fixture.standard(form, iwd=False)
                fixture.prepare()
                source = fixture.read("manifest/artifacts/testbot-1.2.0.toml")["source"]
                self.assertEqual(source["expected_filename"], expected)
                self.assertEqual(source["url"], f"https://github.com/{TESTBOT}/releases/"
                                                f"download/v1.2.0/{expected}")

    def test_a_commit_zip_moves_to_the_commit_of_the_newest_tag(self) -> None:
        fixture = self.fixture()
        fixture.standard()
        outcome = fixture.prepare("testremote")
        self.assertEqual(outcome.release.tag, "v0.2.0")
        new = fixture.read("manifest/artifacts/testremote-0.2.0.toml")
        self.assertEqual(new["source"]["url"],
                         f"https://codeload.github.com/{TESTREMOTE}/zip/{REMOTE_NEW}")
        self.assertEqual(new["source"]["reference"], REMOTE_NEW)
        self.assertEqual(new["source"]["expected_filename"], f"testremote-{REMOTE_NEW}.zip")
        self.assertEqual(new["provenance"]["url"],
                         f"https://github.com/{TESTREMOTE}/commit/{REMOTE_NEW}")
        for recipe in ("manifest", CURATED):
            self.assertEqual(fixture.read(f"{recipe}/mods/testremote.toml")["artifact_id"],
                             "testremote-0.2.0")
        record = fixture.read(f"manifest/evidence/release-intake-{DATE}-testremote-bg.toml")
        self.assertIn("a version tag without a GitHub release", record["scope"])
        self.assertFalse(record["artifacts"][0]["published_release_verified"])
        text = summary(outcome)
        self.assertIn("GitHub publishes no digest for it", text)
        # The wrapper folder carries the commit; the components compare without it.
        self.assertIn("- Components: the same numbers and names as the previous pin.", text)

    def test_changed_components_are_listed_for_christopher(self) -> None:
        fixture = self.fixture()
        fixture.standard(iwd=False)
        changed = fixture.bot_archive("v1.2.0", [(0, "Testbot"), (1, "Testbot extras plus"),
                                                 (2, "Testbot party mode")])
        fixture.github.releases.pop((TESTBOT, "v1.2.0"))
        fixture.github.tag_lists[TESTBOT].remove(("v1.2.0", BOT_COMMITS["v1.2.0"]))
        fixture.release(TESTBOT, "v1.2.0", {"testbot-v1.2.0.zip": changed}, latest=True)
        text = summary(fixture.prepare())
        self.assertIn("- **Components changed; Christopher decides any new component:**", text)
        self.assertIn("  - renamed testbot/setup-testbot.tp2 #1: Testbot extras -> Testbot "
                      "extras plus", text)
        self.assertIn("  - added testbot/setup-testbot.tp2 #2: Testbot party mode", text)

    def test_nothing_changes_when_a_release_needs_an_agent(self) -> None:
        def broken(change) -> PinFixture:
            fixture = self.fixture()
            fixture.standard()
            change(fixture)
            return fixture

        def asset(fixture: PinFixture) -> dict:
            return fixture.github.releases[(TESTBOT, "v1.2.0")]["assets"][0]

        def voicepack_style(fixture: PinFixture) -> None:
            # Each asset carries its own version, which the tag does not tell.
            old = fixture.bot_archive("v1.1.0")
            fixture.recipe(IWD, {"testbot": release_artifact(
                TESTBOT, "v1.1.0", "testbot-aerie-2.0.zip", old, version="2.0")}, "main")

        def two_tp2s(fixture: PinFixture) -> None:
            url = asset(fixture)["browser_download_url"]
            archive = fixture.archive({"testbot/setup-testbot.tp2": "BEGIN ~A~\nDESIGNATED 0\n",
                                       "old/testbot/setup-testbot.tp2": "BEGIN ~A~\n"})
            fixture.serve(url, archive, "release-assets.githubusercontent.com")
            asset(fixture).update(size=archive.stat().st_size, digest=f"sha256:{sha256(archive)}")

        cases = [
            ("is not its release tag", voicepack_style, {}),
            ("cannot tell which asset of v1.2.0 replaces",
             lambda fixture: asset(fixture).update(name="testbot-full.zip"), {}),
            ("differs from the digest GitHub publishes",
             lambda fixture: asset(fixture).update(digest="sha256:" + "0" * 64), {}),
            ("GitHub lists 7",
             lambda fixture: asset(fixture).update(size=7), {}),
            ("draft or prerelease",
             lambda fixture: fixture.github.releases[(TESTBOT, "v1.2.0")].update(prerelease=True),
             {}),
            ("has no tag v9.0.0", lambda fixture: None, {"tag": "v9.0.0"}),
            ("Testbot v1.1.0 is already pinned in manifest", lambda fixture: None,
             {"tag": "v1.1.0", "installers": ["bg"]}),
            ("adding a mod to an installer is Christopher's choice", lambda fixture: None,
             {"mod": "testencounters"}),
            ("which the old pin does not allow",
             lambda fixture: fixture.serve(asset(fixture)["browser_download_url"],
                                           fixture.bot_archive("v1.2.0"), "example.com"), {}),
            ("matches 2 files", two_tp2s, {}),
            ("testbot-1.2.0.toml already exists",
             lambda fixture: fixture.write(f"{IWD}/artifacts/testbot-1.2.0.toml", "id = 'x'\n"),
             {}),
            ("already exists",
             lambda fixture: fixture.write(
                 f"manifest/evidence/release-intake-{DATE}-testbot-bg-iwd.toml", "schema = 1\n"),
             {}),
            ("no chosen installer has its recipes", lambda fixture: None,
             {"installers": ["tv"]}),
        ]
        for message, change, options in cases:
            with self.subTest(message=message):
                self.assertNeedsAgent(broken(change), message, **options)

    def test_a_stop_after_the_first_edit_restores_the_tree(self) -> None:
        cases = [
            (NeedsAgent, "did not verify",
             lambda fixture: fixture.refuse_verify.append("testbot-1.2.0")),
            (RuntimeError, "curated recipe generation failed",
             lambda fixture: fixture.write("generator-fails", "")),
            (NeedsAgent, "testbot in recipes/iwd-current",
             lambda fixture: fixture.refuse_order.append(IWD)),
        ]
        for error, message, change in cases:
            with self.subTest(message=message):
                fixture = self.fixture()
                fixture.standard()
                change(fixture)
                before = tree_state(fixture.root)
                with self.assertRaises(error) as raised:
                    fixture.prepare()
                self.assertIn(message, str(raised.exception))
                self.assertEqual(tree_state(fixture.root), before)

    def test_an_order_misfit_that_the_previous_record_left_out_is_carried_over(self) -> None:
        def previous(order_profiles: list[str]) -> str:
            return f'''schema = 1

[[artifacts]]
mod_id = "testbot"
artifact_id = "testbot-1.1.0"
recipe_profiles = ["manifest", "{CURATED}"]
native_order_recipe_profiles = {json.dumps(order_profiles)}
native_component_order_verified = true
'''

        fixture = self.fixture()
        fixture.standard(iwd=False)
        fixture.write("manifest/evidence/release-intake-earlier.toml", previous(["manifest"]))
        fixture.refuse_order.append(CURATED)
        outcome = fixture.prepare()
        record = fixture.read(f"manifest/evidence/release-intake-{DATE}-testbot-bg.toml")
        self.assertEqual(record["artifacts"][0]["recipe_profiles"], ["manifest", CURATED])
        self.assertEqual(record["artifacts"][0]["native_order_recipe_profiles"], ["manifest"])
        self.assertIn(f"- Native component order, {CURATED}, testbot: does not fit, as with the "
                      "previous pin", summary(outcome))

        # A recipe the previous record verified must still fit.
        fixture = self.fixture()
        fixture.standard(iwd=False)
        fixture.write("manifest/evidence/release-intake-earlier.toml",
                      previous(["manifest", CURATED]))
        fixture.refuse_order.append(CURATED)
        before = tree_state(fixture.root)
        with self.assertRaises(NeedsAgent):
            fixture.prepare()
        self.assertEqual(tree_state(fixture.root), before)

    def test_the_previous_order_records_add_up_per_installer(self) -> None:
        fixture = self.fixture()
        self.assertIsNone(previous_native_profiles(fixture.root, "testbot", "testbot-1.1.0"))
        fixture.write("manifest/evidence/a.toml", '''[[artifacts]]
mod_id = "testbot"
artifact_id = "testbot-1.1.0"
recipe_profiles = ["manifest", "recipes/curated-full-current"]
native_order_recipe_profiles = ["manifest"]
native_component_order_verified = true

[[artifacts]]
mod_id = "other"
artifact_id = "testbot-1.1.0"
recipe_profiles = ["recipes/other"]
native_component_order_verified = true
''')
        self.assertEqual(previous_native_profiles(fixture.root, "testbot", "testbot-1.1.0"),
                         {"manifest"})
        fixture.write("manifest/evidence/b.toml", '''[[artifacts]]
mod_id = "testbot"
artifact_id = "testbot-1.1.0"
recipe_profiles = ["recipes/iwd-current"]
native_component_order_verified = true
''')
        self.assertEqual(previous_native_profiles(fixture.root, "testbot", "testbot-1.1.0"),
                         {"manifest", "recipes/iwd-current"})
        fixture = self.fixture()
        fixture.write("manifest/evidence/c.toml", '''[[artifacts]]
mod_id = "testbot"
artifact_id = "testbot-1.1.0"
recipe_profiles = ["manifest"]
native_component_order_verified = false
''')
        self.assertEqual(previous_native_profiles(fixture.root, "testbot", "testbot-1.1.0"),
                         set())

    def test_the_command_line_writes_the_summary_and_exits_by_outcome(self) -> None:
        fixture = self.fixture()
        fixture.standard()
        (fixture.tools / "author.json").write_text(
            json.dumps({"downloads": fixture.downloads, "refuse": []}), encoding="utf-8")
        (fixture.tools / "tools/order.json").write_text(json.dumps({"refuse": []}),
                                                        encoding="utf-8")
        real_prepare = prepare_own_mod_pin.prepare

        def run(*arguments: str) -> tuple[int, str, str]:
            stdout, stderr = io.StringIO(), io.StringIO()
            with mock.patch.object(prepare_own_mod_pin, "GitHub", return_value=fixture.github), \
                    mock.patch.object(prepare_own_mod_pin, "author_command",
                                      return_value=fixture.author), \
                    mock.patch.object(prepare_own_mod_pin, "prepare",
                                      side_effect=lambda *a, **k: real_prepare(
                                          *a, **{**k, "tools_root": fixture.tools})), \
                    redirect_stdout(stdout), redirect_stderr(stderr):
                code = prepare_own_mod_pin.main([*arguments, "--root", str(fixture.root)])
            return code, stdout.getvalue(), stderr.getvalue()

        summary_file = fixture.work / "summary.md"
        code, out, _ = run("testbot", "--installer", "bg", "--today", TODAY,
                           "--summary", str(summary_file))
        self.assertEqual(code, 0)
        self.assertTrue(out.startswith("## Testbot v1.2.0\n"))
        self.assertEqual(summary_file.read_text(encoding="utf-8"), out)

        code, _, err = run("testbot", "--installer", "bg", "--today", TODAY)
        self.assertEqual(code, 3)
        self.assertIn("needs an agent: Testbot v1.2.0 is already pinned", err)
        self.assertEqual(run("nosuchmod")[0], 2)
        self.assertEqual(run("testbot", "--today", "yesterday")[0], 2)


class SetTomlValueTests(unittest.TestCase):
    TEXT = '''id = "a"
version = "1"

[source]
url = "https://example.com/a"
expected_length = 1

[provenance]
url = "https://example.com/page"
'''

    def test_it_sets_one_key_of_one_section(self) -> None:
        text = set_toml_value(self.TEXT, "source", "url", "https://example.com/b")
        text = set_toml_value(text, "source", "expected_length", 42)
        text = set_toml_value(text, None, "version", "2")
        self.assertEqual(tomllib.loads(text), {
            "id": "a",
            "version": "2",
            "source": {"url": "https://example.com/b", "expected_length": 42},
            "provenance": {"url": "https://example.com/page"},
        })

    def test_a_missing_or_repeated_key_needs_an_agent(self) -> None:
        with self.assertRaisesRegex(NeedsAgent, r"`sha256` appears 0 times in \[source\]"):
            set_toml_value(self.TEXT, "source", "sha256", "x")
        with self.assertRaisesRegex(NeedsAgent, "`url` appears 2 times in the top level"):
            set_toml_value('url = "a"\nurl = "b"\n', None, "url", "c")


if __name__ == "__main__":
    unittest.main()
