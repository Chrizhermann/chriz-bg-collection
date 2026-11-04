"""Static and synthetic checks of tools/own_mod_pins.py and manifest/own-mods.toml.

The synthetic recipes carry only the fields the check reads. GitHub answers come from a
fake, so these tests need no network; the CI workflow runs the real check online.
"""

from __future__ import annotations

import io
import json
import subprocess
import tempfile
import tomllib
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest import mock

from tools import own_mod_pins
from tools.own_mod_pins import (
    GitHubError,
    RegistryError,
    check,
    load_registry,
    parse_registry,
)


ROOT = Path(__file__).resolve().parents[2]
ACCOUNT = "Chrizhermann"
SHA_A = "a" * 64
SHA_B = "b" * 64
COMMIT_1 = "1" * 40
COMMIT_2 = "2" * 40


def release_url(repository: str, tag: str, asset: str | None = None) -> str:
    name = asset or f"{repository.split('/')[1]}-{tag}.zip"
    return f"https://github.com/{repository}/releases/download/{tag}/{name}"


def artifact(
    artifact_id: str,
    url: str,
    *,
    kind: str = "github-release",
    reference: str = "",
    sha256: str = SHA_A,
    length: int = 100,
) -> str:
    return "\n".join([
        f'id = "{artifact_id}"',
        f'name = "{artifact_id}"',
        'version = "1"',
        'acquisition = "fetch-only"',
        "",
        "[source]",
        f'kind = "{kind}"',
        f'url = "{url}"',
        f'reference = "{reference}"',
        f"expected_length = {length}",
        f'sha256 = "{sha256}"',
        "",
    ])


class FakeGitHub:
    def __init__(
        self,
        latest: dict[str, str] | None = None,
        tags: dict[str, list[tuple[str, str]]] | None = None,
        broken: set[str] | None = None,
    ) -> None:
        self.latest = latest or {}
        self.tag_lists = tags or {}
        self.broken = broken or set()

    def latest_release(self, repository: str) -> str:
        if repository in self.broken or repository not in self.latest:
            raise GitHubError(f"no answer for {repository}")
        return self.latest[repository]

    def tags(self, repository: str) -> list[tuple[str, str]]:
        if repository in self.broken:
            raise GitHubError(f"no answer for {repository}")
        return self.tag_lists.get(repository, [])


class Fixture:
    """A repository root with installer recipes, app configs and a registry."""

    def __init__(self, root: Path) -> None:
        self.root = root

    def write(self, relative: str, text: str) -> None:
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8", newline="\n")

    def recipe(self, recipe: str, pins: dict[str, str], *, version: str = "1.0.0") -> None:
        """`pins` maps a recipe mod id to artifact TOML text (its id is read from the text)."""
        self.write(f"{recipe}/collection.toml", "schema = 2\n")
        self.write(f"{recipe}/release.json", json.dumps({"version": version}))
        for mod_id, text in pins.items():
            artifact_id = tomllib.loads(text)["id"]
            self.write(f"{recipe}/mods/{mod_id}.toml",
                       f'id = "{mod_id}"\nartifact_id = "{artifact_id}"\n')
            self.write(f"{recipe}/artifacts/{artifact_id}.toml", text)

    def bundle(self, config: str, recipes: list[str]) -> None:
        resources = {}
        for recipe in recipes:
            if recipe == "manifest":
                resources["../../manifest/mods/"] = "manifest/mods/"
            else:
                resources[f"../../{recipe}/"] = f"{recipe}/"
        self.write(f"app/src-tauri/{config}", json.dumps({"bundle": {"resources": resources}}))

    def registry(self, extra: str = "", *, iwd_branch: str | None = None) -> None:
        branch = f'branch = "{iwd_branch}"\n' if iwd_branch else ""
        self.write("manifest/own-mods.toml", f"""schema = 1
account = "{ACCOUNT}"

[[installers]]
id = "bg"
name = "BG"
recipes = ["manifest", "recipes/curated"]

[[installers]]
id = "iwd"
name = "IWD"
recipes = ["recipes/iwd"]
{branch}
[[mods]]
id = "buffbot"
name = "BuffBot"
repository = "{ACCOUNT}/bg-eeex-buffbot"

[[mods]]
id = "modpack"
name = "Modpack"
repository = "{ACCOUNT}/chriz-bg-modpack"

[[mods]]
id = "remote"
name = "Remote"
repository = "{ACCOUNT}/eeex-remote-console"
newest = "newest-tag"
{extra}""")

    def git(self, *args: str) -> str:
        result = subprocess.run(
            ["git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
             *args],
            cwd=self.root,
            capture_output=True,
            text=True,
            check=True,
        )
        return result.stdout.strip()


BUFFBOT = f"{ACCOUNT}/bg-eeex-buffbot"
MODPACK = f"{ACCOUNT}/chriz-bg-modpack"
REMOTE = f"{ACCOUNT}/eeex-remote-console"


def buffbot(tag: str, sha256: str = SHA_A) -> str:
    return artifact(f"buffbot-{tag}", release_url(BUFFBOT, tag), reference=tag, sha256=sha256)


def modpack(tag: str) -> str:
    return artifact(f"modpack-{tag}", release_url(MODPACK, tag), reference=tag)


def remote(commit: str) -> str:
    return artifact(
        f"remote-{commit[:7]}",
        f"https://codeload.github.com/{REMOTE}/zip/{commit}",
        kind="github-commit-zip",
        reference=commit,
    )


def third_party(tag: str) -> str:
    return artifact(f"eeex-{tag}", release_url("Bubb13/EEex", tag), reference=tag)


class OwnModPinsTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.fixture = Fixture(Path(self.directory.name))

    def standard(self, *, bg_buffbot: str = "v1.2.0", iwd_buffbot: str = "v1.2.0",
                 extra: str = "") -> None:
        fixture = self.fixture
        fixture.recipe("manifest", {"buffbot": buffbot(bg_buffbot), "modpack": modpack("v0.3"),
                                    "eeex": third_party("v1.3.1")})
        fixture.recipe("recipes/curated", {"buffbot": buffbot(bg_buffbot),
                                           "modpack": modpack("v0.3"),
                                           "remote": remote(COMMIT_2)})
        fixture.recipe("recipes/iwd", {"buffbot": buffbot(iwd_buffbot),
                                       "remote": remote(COMMIT_2)})
        fixture.bundle("tauri.conf.json", ["manifest", "recipes/curated"])
        fixture.bundle("tauri.iwd.conf.json", ["recipes/iwd"])
        fixture.registry(extra)

    def github(self, buffbot_latest: str = "v1.2.0") -> FakeGitHub:
        return FakeGitHub(
            latest={BUFFBOT: buffbot_latest, MODPACK: "v0.3"},
            tags={REMOTE: [("v0.1.0", COMMIT_1), ("v0.2.0", COMMIT_2), ("nightly", "f" * 40)]},
        )

    def run_check(self, github: FakeGitHub | None = None, **options):
        registry = load_registry(self.fixture.root / "manifest/own-mods.toml")
        return check(self.fixture.root, registry, github, **options)

    def test_consistent_newest_pins_pass(self) -> None:
        self.standard()
        report = self.run_check(self.github())
        self.assertEqual(report.errors, [])
        self.assertEqual(report.warnings, [])
        states = {status.own_mod.id: status.state for status in report.statuses}
        self.assertEqual(states, {"buffbot": "newest", "modpack": "newest", "remote": "newest"})
        remote_status = next(s for s in report.statuses if s.own_mod.id == "remote")
        self.assertEqual(remote_status.pins, {"bg": "v0.2.0 (222222222)",
                                              "iwd": "v0.2.0 (222222222)"})

    def test_installers_on_different_releases_fail_without_a_hold(self) -> None:
        self.standard(iwd_buffbot="v1.1.0")
        report = self.run_check(None)
        self.assertEqual(len(report.errors), 1)
        self.assertIn("BuffBot: the installers pin different releases", report.errors[0])
        self.assertIn("BG v1.2.0, IWD v1.1.0", report.errors[0])

    def test_a_hold_that_records_the_split_passes_and_shows_its_reason(self) -> None:
        self.standard(bg_buffbot="v1.1.0", iwd_buffbot="v1.2.0", extra="""
[[holds]]
mod = "buffbot"
release = "v1.3.0"
reason = "Waits for Christopher's test."
decided = "2026-10-28"
installers = { bg = "v1.1.0", iwd = "v1.2.0" }
""")
        report = self.run_check(self.github("v1.3.0"), release=True)
        self.assertEqual(report.errors, [])
        self.assertEqual(report.notes,
                         ["BuffBot v1.3.0 is held (decided 2026-10-28): Waits for Christopher's "
                          "test."])

    def test_a_hold_with_other_pins_than_the_installers_fails(self) -> None:
        self.standard(bg_buffbot="v1.3.0", iwd_buffbot="v1.2.0", extra="""
[[holds]]
mod = "buffbot"
release = "v1.3.0"
reason = "Waits for Christopher's test."
decided = "2026-10-28"
installers = { bg = "v1.1.0", iwd = "v1.2.0" }
""")
        report = self.run_check(self.github("v1.3.0"))
        self.assertEqual(len(report.errors), 1)
        self.assertIn("its hold names BG v1.1.0, IWD v1.2.0, but the installers pin "
                      "BG v1.3.0, IWD v1.2.0", report.errors[0])

    def test_a_hold_fails_once_every_installer_pins_its_release(self) -> None:
        self.standard(bg_buffbot="v1.3.0", iwd_buffbot="v1.3.0", extra="""
[[holds]]
mod = "buffbot"
release = "v1.3.0"
reason = "Waits for Christopher's test."
decided = "2026-10-28"
installers = { bg = "v1.1.0", iwd = "v1.2.0" }
""")
        for github in (None, self.github("v1.3.0")):
            report = self.run_check(github)
            self.assertEqual(report.errors, [
                "BuffBot: every installer pins v1.3.0 now; remove its hold from "
                "manifest/own-mods.toml"
            ])

    def test_a_hold_for_an_older_release_fails_when_every_installer_is_newest(self) -> None:
        self.standard(extra="""
[[holds]]
mod = "buffbot"
release = "v1.1.0"
reason = "Old."
decided = "2026-10-01"
""")
        report = self.run_check(self.github())
        self.assertEqual(len(report.errors), 1)
        self.assertIn("every installer has the newest release v1.2.0; remove its hold",
                      report.errors[0])

    def test_a_hold_for_a_mod_that_no_installer_pins_fails(self) -> None:
        self.standard(extra="""
[[mods]]
id = "encounters"
name = "Encounters"
repository = "Chrizhermann/chriz-iwd-encounters"

[[holds]]
mod = "encounters"
release = "v0.1.0"
reason = "Not yet."
decided = "2026-10-28"
""")
        report = self.run_check(None)
        self.assertEqual(len(report.errors), 1)
        self.assertIn("Encounters: no installer pins it", report.errors[0])

    def test_a_repository_of_the_account_that_the_registry_does_not_list_fails(self) -> None:
        self.standard()
        unlisted = artifact("cir-v1.0.2",
                            release_url(f"{ACCOUNT}/chriz-item-randomiser", "v1.0.2"),
                            reference="v1.0.2")
        self.fixture.write("recipes/curated/mods/randomiser.toml",
                           'id = "randomiser"\nartifact_id = "cir-v1.0.2"\n')
        self.fixture.write("recipes/curated/artifacts/cir-v1.0.2.toml", unlisted)
        report = self.run_check(None)
        self.assertEqual(len(report.errors), 1)
        self.assertIn("`randomiser` in recipes/curated installs `cir-v1.0.2` from "
                      "Chrizhermann/chriz-item-randomiser, which manifest/own-mods.toml does not "
                      "list", report.errors[0])

    def test_unused_artifact_files_and_third_party_mods_are_ignored(self) -> None:
        self.standard()
        self.fixture.write("manifest/artifacts/buffbot-v0.9.toml", buffbot("v0.9"))
        report = self.run_check(self.github())
        self.assertEqual(report.errors, [])
        self.assertEqual(report.warnings, [])

    def test_the_same_url_with_another_digest_fails(self) -> None:
        self.standard()
        self.fixture.write("recipes/curated/artifacts/buffbot-v1.2.0.toml",
                           buffbot("v1.2.0", sha256=SHA_B))
        report = self.run_check(None)
        self.assertEqual(len(report.errors), 1)
        self.assertIn("is pinned with different SHA-256 or length values", report.errors[0])

    def test_one_installer_with_two_releases_of_a_mod_fails(self) -> None:
        self.standard()
        self.fixture.write("recipes/curated/mods/buffbot.toml",
                           'id = "buffbot"\nartifact_id = "buffbot-v1.1.0"\n')
        self.fixture.write("recipes/curated/artifacts/buffbot-v1.1.0.toml", buffbot("v1.1.0"))
        report = self.run_check(None)
        self.assertIn("BuffBot: BG mixes releases ['v1.1.0', 'v1.2.0']; one installer pins one "
                      "release of a mod", report.errors)

    def test_a_missing_artifact_definition_fails(self) -> None:
        self.standard()
        (self.fixture.root / "recipes/iwd/artifacts/buffbot-v1.2.0.toml").unlink()
        report = self.run_check(None)
        self.assertEqual(len(report.errors), 1)
        self.assertIn("installs `buffbot-v1.2.0`, which recipes/iwd/artifacts does not define",
                      report.errors[0])

    def test_behind_is_a_warning_in_a_status_check_and_an_error_in_a_release_check(self) -> None:
        self.standard()
        status = self.run_check(self.github("v1.3.0"))
        self.assertEqual(status.errors, [])
        self.assertEqual(status.warnings, [
            "BuffBot: BG and IWD are not on its newest release v1.3.0. Pin v1.3.0 in every "
            "installer in the same step, or record a hold with its reason in "
            "manifest/own-mods.toml."
        ])
        release = self.run_check(self.github("v1.3.0"), release=True)
        self.assertEqual(release.errors, status.warnings)
        self.assertEqual(release.warnings, [])

    def test_a_hold_for_an_older_release_does_not_cover_a_newer_one(self) -> None:
        self.standard(extra="""
[[holds]]
mod = "buffbot"
release = "v1.3.0"
reason = "Waits for Christopher's test."
decided = "2026-10-28"
""")
        report = self.run_check(self.github("v1.4.0"), release=True)
        self.assertEqual(len(report.errors), 1)
        self.assertIn("not on its newest release v1.4.0. Its hold covers only v1.3.0.",
                      report.errors[0])

    def test_newest_tag_mode_compares_the_pinned_commit(self) -> None:
        self.standard()
        github = self.github()
        github.tag_lists[REMOTE].append(("v0.10.0", "3" * 40))
        report = self.run_check(github, release=True)
        self.assertEqual(len(report.errors), 1)
        self.assertIn("Remote: BG and IWD are not on its newest release v0.10.0",
                      report.errors[0])

    def test_an_unreachable_github_is_a_warning_unless_the_check_is_strict(self) -> None:
        self.standard()
        github = self.github()
        github.broken.add(MODPACK)
        lenient = self.run_check(github)
        self.assertEqual(lenient.errors, [])
        self.assertEqual(len(lenient.warnings), 1)
        self.assertIn("Modpack: could not read its newest release", lenient.warnings[0])
        strict = self.run_check(github, release=True)
        self.assertEqual(strict.errors, lenient.warnings)

    def test_every_bundled_recipe_belongs_to_an_installer(self) -> None:
        self.standard()
        self.fixture.bundle("tauri.conf.json", ["manifest", "recipes/curated", "recipes/new"])
        report = self.run_check(None)
        self.assertEqual(report.errors, [
            "app/src-tauri/tauri.conf.json bundles recipes/new, but no installer in "
            "manifest/own-mods.toml lists it"
        ])

    def test_every_listed_recipe_in_the_tree_is_bundled(self) -> None:
        self.standard()
        self.fixture.bundle("tauri.conf.json", ["manifest"])
        report = self.run_check(None)
        self.assertEqual(report.errors, [
            "BG: manifest/own-mods.toml lists recipes/curated, but no app configuration in "
            "app/src-tauri bundles it"
        ])

    def test_a_null_resource_does_not_count_as_bundled(self) -> None:
        self.standard()
        config = {"bundle": {"resources": {"../../manifest/mods/": None,
                                           "../../recipes/iwd/": "recipes/iwd/"}}}
        self.fixture.write("app/src-tauri/tauri.iwd.conf.json", json.dumps(config))
        self.assertEqual(own_mod_pins.bundled_recipe_roots(self.fixture.root)
                         ["app/src-tauri/tauri.iwd.conf.json"], ["recipes/iwd"])

    def test_an_installer_on_another_branch_is_read_from_that_branch(self) -> None:
        fixture = self.fixture
        self.standard(iwd_buffbot="v1.1.0")
        fixture.git("init", "-q", "-b", "trunk")
        fixture.git("add", "-A")
        fixture.git("commit", "-q", "-m", "iwd on v1.1.0")
        fixture.git("update-ref", "refs/remotes/origin/iwd-branch", "HEAD")
        # This tree has no IWD recipe; the registry sends the check to the branch.
        for path in sorted((fixture.root / "recipes/iwd").rglob("*"), reverse=True):
            path.unlink() if path.is_file() else path.rmdir()
        (fixture.root / "recipes/iwd").rmdir()
        (fixture.root / "app/src-tauri/tauri.iwd.conf.json").unlink()
        fixture.registry(iwd_branch="iwd-branch")
        report = self.run_check(None)
        self.assertEqual(len(report.errors), 1)
        self.assertIn("BG v1.2.0, IWD v1.1.0", report.errors[0])
        self.assertTrue(any(source.startswith("IWD: recipes/iwd from origin/iwd-branch at ")
                            for source in report.sources))

        fixture.registry(iwd_branch="missing-branch")
        report = self.run_check(None)
        self.assertIn("git ref `origin/missing-branch` does not exist here; fetch it first",
                      report.errors)
        self.assertNotIn("iwd", {key for status in report.statuses for key in status.pins})

    def test_an_installer_without_its_recipe_and_without_a_branch_fails(self) -> None:
        self.standard()
        (self.fixture.root / "recipes/iwd/collection.toml").unlink()
        report = self.run_check(None)
        self.assertIn("IWD: recipes/iwd is not in this working tree", report.errors)

    def committed_base(self) -> None:
        self.standard(extra="")
        self.fixture.git("init", "-q", "-b", "trunk")
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-q", "-m", "base")

    def test_behind_is_an_error_only_for_a_mod_whose_pin_this_change_touches(self) -> None:
        self.committed_base()
        # The change pins BuffBot v1.3.0 everywhere while v1.4.0 is out; Modpack is untouched
        # and also behind.
        fixture = self.fixture
        for recipe in ("manifest", "recipes/curated", "recipes/iwd"):
            (fixture.root / f"{recipe}/artifacts/buffbot-v1.2.0.toml").unlink()
            fixture.write(f"{recipe}/mods/buffbot.toml",
                          'id = "buffbot"\nartifact_id = "buffbot-v1.3.0"\n')
            fixture.write(f"{recipe}/artifacts/buffbot-v1.3.0.toml", buffbot("v1.3.0"))
        github = self.github("v1.4.0")
        github.latest[MODPACK] = "v0.4"
        report = self.run_check(github, base="HEAD")
        self.assertIn("newest release required for BuffBot", report.mode)
        self.assertEqual(len(report.errors), 1)
        self.assertIn("BuffBot: BG and IWD are not on its newest release v1.4.0",
                      report.errors[0])
        self.assertEqual(len(report.warnings), 1)
        self.assertIn("Modpack: BG is not on its newest release v0.4", report.warnings[0])

    def test_a_release_version_change_makes_it_a_release_check(self) -> None:
        self.committed_base()
        self.fixture.write("recipes/curated/release.json", json.dumps({"version": "1.1.0"}))
        github = self.github()
        github.latest[MODPACK] = "v0.4"
        report = self.run_check(github, base="HEAD")
        self.assertIn("release check", report.mode)
        self.assertEqual(len(report.errors), 1)
        self.assertIn("Modpack: BG is not on its newest release v0.4", report.errors[0])

    def test_an_unchanged_tree_against_its_base_is_a_status_check(self) -> None:
        self.committed_base()
        github = self.github()
        github.latest[MODPACK] = "v0.4"
        report = self.run_check(github, base="HEAD")
        self.assertIn("newest release required for no mod", report.mode)
        self.assertEqual(report.errors, [])
        self.assertEqual(len(report.warnings), 1)

    def test_the_command_line_reports_and_exits_with_the_result(self) -> None:
        self.standard(iwd_buffbot="v1.1.0")
        output = io.StringIO()
        with redirect_stdout(output), mock.patch.dict(
            "os.environ", {"GITHUB_ACTIONS": "true", "GITHUB_STEP_SUMMARY": ""}, clear=False
        ):
            code = own_mod_pins.main(["check", "--root", str(self.fixture.root), "--offline"])
        self.assertEqual(code, 1)
        text = output.getvalue()
        self.assertIn("Errors (1):", text)
        self.assertIn("::error file=manifest/own-mods.toml,title=Own mod pins::BuffBot: the "
                      "installers pin different releases", text)
        self.standard()
        output = io.StringIO()
        with redirect_stdout(output):
            code = own_mod_pins.main(["check", "--root", str(self.fixture.root), "--offline"])
        self.assertEqual(code, 0)
        self.assertIn("OK: no installer is out of step.", output.getvalue())


class RegistryTests(unittest.TestCase):
    BASE = {
        "schema": 1,
        "account": ACCOUNT,
        "installers": [{"id": "bg", "name": "BG", "recipes": ["manifest"]},
                       {"id": "iwd", "name": "IWD", "recipes": ["recipes/iwd"]}],
        "mods": [{"id": "buffbot", "name": "BuffBot", "repository": BUFFBOT}],
    }

    def registry(self, **changes):
        data = json.loads(json.dumps(self.BASE))
        data.update(changes)
        return parse_registry(data)

    def assertRejected(self, message: str, **changes) -> None:
        with self.assertRaises(RegistryError) as raised:
            self.registry(**changes)
        self.assertIn(message, str(raised.exception))

    def test_the_shipped_registry_lists_the_installers_and_our_mods(self) -> None:
        registry = load_registry(ROOT / "manifest/own-mods.toml")
        self.assertEqual(registry.account, "Chrizhermann")
        self.assertEqual(
            {installer.id: installer.recipes for installer in registry.installers},
            {"chriz-easy-bg": ("manifest", "recipes/curated-full-current"),
             "chriz-easy-iwd": ("recipes/iwd-current",)},
        )
        # Chriz Easy IWD's static test pins every artifact file; Chriz Easy BG keeps history.
        self.assertEqual(
            {installer.id: installer.superseded_artifacts for installer in registry.installers},
            {"chriz-easy-bg": "keep", "chriz-easy-iwd": "remove"},
        )
        repositories = {own_mod.repository for own_mod in registry.mods}
        # Christopher, 2026-10-28: every mod released from his account, forks included.
        self.assertTrue({
            "Chrizhermann/bg-eeex-buffbot",
            "Chrizhermann/chriz-bg-modpack",
            "Chrizhermann/chriz-bg-rebalance",
            "Chrizhermann/chriz-sod-rebalance",
            "Chrizhermann/eeex-remote-console",
            "Chrizhermann/chriz-bg-voicepacks",
            "Chrizhermann/chriz-iwd-encounters",
            "Chrizhermann/The-Artisan-s-Kitpack-Chriz-Balance-Patch",
            "Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch",
            "Chrizhermann/chriz-spell-revisions-patch",
            "Chrizhermann/chriz-item-randomiser",
        } <= repositories)
        for hold in registry.holds:
            self.assertTrue(hold.reason)

    def test_the_shipped_registry_matches_this_tree_without_network(self) -> None:
        """The in-tree installers; the CI workflow also reads the branches and GitHub."""
        registry = load_registry(ROOT / "manifest/own-mods.toml")
        in_tree = own_mod_pins.Registry(
            registry.account,
            tuple(installer for installer in registry.installers if installer.branch is None),
            registry.mods,
            (),
        )
        report = check(ROOT, in_tree, None)
        self.assertEqual(report.errors, [])
        pinned = {status.own_mod.id for status in report.statuses if status.pins}
        self.assertTrue({"buffbot", "chriz-bg-modpack", "chriz-bg-rebalance"} <= pinned)

    def test_invalid_registries_are_rejected(self) -> None:
        self.assertRejected("`schema` must be 1", schema=2)
        self.assertRejected("unknown keys ['extra']", extra=True)
        self.assertRejected("is not on Chrizhermann",
                            mods=[{"id": "eeex", "name": "EEex", "repository": "Bubb13/EEex"}])
        self.assertRejected("is listed twice", mods=[
            {"id": "a", "name": "A", "repository": BUFFBOT},
            {"id": "b", "name": "B", "repository": BUFFBOT.lower()},
        ])
        self.assertRejected("`newest` must be one of", mods=[
            {"id": "a", "name": "A", "repository": BUFFBOT, "newest": "main"},
        ])
        self.assertRejected("must be a plain relative path", installers=[
            {"id": "bg", "name": "BG", "recipes": ["../manifest"]},
        ])
        self.assertRejected("already belongs to `bg`", installers=[
            {"id": "bg", "name": "BG", "recipes": ["manifest"]},
            {"id": "iwd", "name": "IWD", "recipes": ["manifest"]},
        ])
        self.assertEqual(self.registry().installers[0].superseded_artifacts, "keep")
        self.assertRejected('`superseded_artifacts` must be "keep" or "remove"', installers=[
            {"id": "bg", "name": "BG", "recipes": ["manifest"], "superseded_artifacts": "delete"},
        ])
        hold = {"mod": "buffbot", "release": "v1", "reason": "Test.", "decided": "2026-10-28"}
        self.assertRejected("is not one of the listed mods", holds=[{**hold, "mod": "other"}])
        self.assertRejected("must be a date", holds=[{**hold, "decided": "yesterday"}])
        self.assertRejected("`reason` must be a non-empty string", holds=[{**hold, "reason": " "}])
        self.assertRejected("already has a hold", holds=[hold, hold])
        self.assertRejected("names at least two installers",
                            holds=[{**hold, "installers": {"bg": "v1"}}])
        self.assertRejected("`tv` is not a listed installer",
                            holds=[{**hold, "installers": {"bg": "v1", "tv": "v2"}}])


if __name__ == "__main__":
    unittest.main()
