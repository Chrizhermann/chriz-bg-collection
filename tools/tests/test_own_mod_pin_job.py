"""Synthetic checks of tools/own_mod_pin_job.py, the daily pin job.

Each test builds a small origin repository with two installer branches and clones it. A
fake stands in for GitHub, and a fake preparer edits the recipes the way
tools/prepare_own_mod_pin.py would, so these tests need no network and no engine build.
The fixture mods are named Test*, so that no real pin ever finds this file.
"""

from __future__ import annotations

import io
import json
import subprocess
import tempfile
import tomllib
import unittest
import urllib.error
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from unittest import mock

from tools import own_mod_pin_job as job
from tools.own_mod_pins import GitHubError, load_registry
from tools.prepare_own_mod_pin import NeedsAgent


ACCOUNT = "Chrizhermann"
TESTBOT = f"{ACCOUNT}/testbot"
TESTREMOTE = f"{ACCOUNT}/testremote"
TESTENCOUNTERS = f"{ACCOUNT}/testencounters"
NEXT = "next-release"
IWD_BRANCH = "iwd-app"
COMMITS = {"v1.1.0": "1" * 40, "v1.1.5": "2" * 40, "v1.2.0": "3" * 40, "v0.9.0": "4" * 40}
REMOTE_COMMIT = "5" * 40
BG_BRANCH = "own-mod-pin/bg/testbot-v1.2.0"
IWD_PIN_BRANCH = "own-mod-pin/iwd/testbot-v1.2.0"


def release_artifact(name: str, tag: str) -> str:
    version = tag.removeprefix("v")
    return f'''id = "{name}-{version}"
version = "{version}"

[source]
kind = "github-release"
url = "https://github.com/{ACCOUNT}/{name}/releases/download/{tag}/{name}-{tag}.zip"
reference = "{tag}"
expected_length = 100
sha256 = "{"a" * 64}"
'''


def commit_artifact() -> str:
    return f'''id = "testremote-0.2.0"
version = "0.2.0"

[source]
kind = "github-commit-zip"
url = "https://codeload.github.com/{TESTREMOTE}/zip/{REMOTE_COMMIT}"
reference = "{REMOTE_COMMIT}"
expected_length = 100
sha256 = "{"b" * 64}"
'''


class FakeGitHub:
    """What the job asks GitHub, answered from dictionaries; writes are recorded."""

    def __init__(self) -> None:
        self.latest = {TESTBOT: "v1.2.0"}
        self.tag_lists = {TESTBOT: list(COMMITS.items()),
                          TESTREMOTE: [("v0.2.0", REMOTE_COMMIT)]}
        self.open: dict[str, list[dict]] = {}
        self.files: dict[int, list[dict]] = {}
        self.heads: dict[str, list[dict]] = {}
        self.issues: dict[str, dict] = {}
        self.created: list[dict] = []
        self.edited: dict[int, str] = {}
        self.dispatched: list[tuple[str, str]] = []
        self.created_issues: list[tuple[str, str]] = []
        self.broken: set[str] = set()

    def latest_release(self, repository: str) -> str:
        if repository in self.broken or repository not in self.latest:
            raise GitHubError(f"no Latest release for {repository}")
        return self.latest[repository]

    def tags(self, repository: str) -> list[tuple[str, str]]:
        return self.tag_lists.get(repository, [])

    def open_pulls(self, base: str) -> list[dict]:
        return self.open.get(base, [])

    def pull_files(self, number: int) -> list[dict]:
        return self.files.get(number, [])

    def pulls_from(self, branch: str) -> list[dict]:
        return self.heads.get(branch, [])

    def create_pull(self, title: str, head: str, base: str, body: str) -> dict:
        number = 100 + len(self.created)
        self.created.append({"number": number, "title": title, "head": head, "base": base,
                             "body": body})
        return {"number": number, "html_url": f"https://example.invalid/pull/{number}"}

    def edit_pull(self, number: int, body: str) -> None:
        self.edited[number] = body

    def issue_titled(self, title: str) -> dict | None:
        return self.issues.get(title)

    def create_issue(self, title: str, body: str) -> dict:
        self.created_issues.append((title, body))
        return {"number": 200 + len(self.created_issues)}

    def dispatch(self, workflow: str, ref: str) -> None:
        self.dispatched.append((workflow, ref))


class JobFixture:
    """An origin with the next-release branch (Chriz Easy BG's recipes, the registry) and an
    installer branch (Chriz Easy IWD's recipe), and a clone of it as the job's root."""

    def __init__(self, base: Path) -> None:
        self.base = base
        self.origin = base / "origin.git"
        self.seed = base / "seed"
        self.root = base / "root"
        self.work = base / "work"
        self.work.mkdir()
        self.github = FakeGitHub()
        self.git(base, "init", "--quiet", "--bare", str(self.origin))
        self.git(base, "init", "--quiet", str(self.seed))

    def git(self, cwd: Path, *args: str) -> str:
        result = subprocess.run(
            ["git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
             "-c", "core.autocrlf=false", *args],
            cwd=cwd, capture_output=True, text=True, check=True,
        )
        return result.stdout.strip()

    def write(self, relative: str, text: str) -> None:
        path = self.seed / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8", newline="\n")

    def recipe(self, recipe: str, pins: dict[str, str]) -> None:
        for mod_id, text in pins.items():
            artifact_id = tomllib.loads(text)["id"]
            self.write(f"{recipe}/mods/{mod_id}.toml",
                       f'id = "{mod_id}"\nartifact_id = "{artifact_id}"\n')
            self.write(f"{recipe}/artifacts/{artifact_id}.toml", text)
        self.write(f"{recipe}/collection.toml", "schema = 2\n")

    def registry(self, holds: str = "", bg_recipes: str = '"manifest"') -> str:
        return f'''schema = 1
account = "{ACCOUNT}"

[[installers]]
id = "bg"
name = "BG"
recipes = [{bg_recipes}]

[[installers]]
id = "iwd"
name = "IWD"
recipes = ["recipes/iwd"]
branch = "{IWD_BRANCH}"

[[mods]]
id = "testbot"
name = "Testbot"
repository = "{TESTBOT}"

[[mods]]
id = "testremote"
name = "Testremote"
repository = "{TESTREMOTE}"
newest = "newest-tag"

[[mods]]
id = "testencounters"
name = "Testencounters"
repository = "{TESTENCOUNTERS}"
{holds}'''

    def build(self, *, bg: str = "v1.1.0", iwd: str = "v1.1.5", holds: str = "",
              extra: str | None = None) -> None:
        """Commit both branches to the origin and clone it. `extra` adds a second recipe to
        Chriz Easy BG that pins Testbot at that release."""
        self.git(self.seed, "checkout", "--quiet", "-b", NEXT)
        self.recipe("manifest", {"testbot": release_artifact("testbot", bg),
                                 "testremote": commit_artifact()})
        bg_recipes = '"manifest"'
        if extra:
            self.recipe("recipes/extra", {"testbot": release_artifact("testbot", extra)})
            bg_recipes = '"manifest", "recipes/extra"'
        self.write("manifest/own-mods.toml", self.registry(holds, bg_recipes))
        self.write(".github/workflows/ci.yml", "name: CI\n")
        self.write(".github/workflows/own-mod-pins.yml", "name: Own mod pins\n")
        self.git(self.seed, "add", "--all")
        self.git(self.seed, "commit", "--quiet", "-m", "next-release")
        self.git(self.seed, "checkout", "--quiet", "--orphan", IWD_BRANCH)
        self.git(self.seed, "rm", "-r", "--quiet", "--cached", ".")
        for path in ("manifest", "recipes", ".github"):
            subprocess.run(["git", "clean", "-fdq", path], cwd=self.seed, check=False)
        self.recipe("recipes/iwd", {"testbot": release_artifact("testbot", iwd)})
        self.write(".github/workflows/ci.yml", "name: CI\n")
        self.git(self.seed, "add", "--all")
        self.git(self.seed, "commit", "--quiet", "-m", "iwd")
        self.git(self.seed, "push", "--quiet", str(self.origin), NEXT, IWD_BRANCH)
        self.git(self.base, "clone", "--quiet", "--branch", NEXT, str(self.origin),
                 str(self.root))
        self.git(self.root, "fetch", "--quiet", "origin",
                 f"+refs/heads/{IWD_BRANCH}:refs/remotes/origin/{IWD_BRANCH}")

    def plan(self, only: str | None = None) -> list[job.Item]:
        registry = load_registry(self.root / "manifest/own-mods.toml")
        return job.plan(self.root, registry, self.github, NEXT, "origin", only)

    def run(self, preparer, *, dry_run: bool = False, only: str | None = None):
        return job.run(self.root, self.github, self.plan(only), preparer, remote="origin",
                       work=self.work, dry_run=dry_run, run_url="https://example.invalid/run")

    def origin_branches(self) -> list[str]:
        listing = self.git(self.origin, "for-each-ref", "--format=%(refname:short)",
                           "refs/heads")
        return sorted(listing.split())


def fake_preparer(needs_agent_in: set[str] = frozenset()):
    """Switch a mod's pins to the release the way the real preparation would."""
    calls: list[tuple[str, list[str]]] = []

    def preparer(tree: Path, own_mod, release: str, installers: list[str], work: Path) -> str:
        calls.append((release, installers))
        if needs_agent_in & set(installers):
            raise NeedsAgent("cannot tell which asset of v1.2.0 replaces testbot-v1.1.5.zip")
        new_id = f"{own_mod.id}-{release.removeprefix('v')}"
        for mods in tree.glob(f"**/mods/{own_mod.id}.toml"):
            mods.write_text(f'id = "{own_mod.id}"\nartifact_id = "{new_id}"\n',
                            encoding="utf-8")
            (mods.parent.parent / f"artifacts/{new_id}.toml").write_text(
                release_artifact(own_mod.id, release), encoding="utf-8")
        return f"## {own_mod.name} {release}\n\nPrepared for {', '.join(installers)}.\n"

    preparer.calls = calls
    return preparer


class PlanTests(unittest.TestCase):
    def fixture(self, **build) -> JobFixture:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        fixture = JobFixture(Path(directory.name))
        fixture.build(**build)
        return fixture

    def test_every_installer_that_is_behind_gets_the_release_on_its_branch(self) -> None:
        fixture = self.fixture()
        items = fixture.plan()
        self.assertEqual([(item.mod.id, item.release, item.stop) for item in items],
                         [("testbot", "v1.2.0", None)])
        item = items[0]
        self.assertEqual([(part.target.branch, [installer.id for installer in part.installers])
                          for part in item.parts], [(NEXT, ["bg"]), (IWD_BRANCH, ["iwd"])])
        self.assertEqual([item.branch(part) for part in item.parts], [BG_BRANCH, IWD_PIN_BRANCH])

    def test_mods_on_their_newest_release_or_in_no_installer_need_nothing(self) -> None:
        fixture = self.fixture(bg="v1.2.0", iwd="v1.2.0")
        self.assertEqual(fixture.plan(), [])
        # Only the behind installer goes ahead; the other already has the release.
        fixture = self.fixture(bg="v1.2.0")
        item, = fixture.plan()
        self.assertEqual([part.target.branch for part in item.parts], [IWD_BRANCH])

    def test_a_release_stops_with_its_reason(self) -> None:
        hold = '''
[[holds]]
mod = "testbot"
release = "{release}"
reason = "Christopher tests it first."
decided = "2026-11-01"
installers = {{ bg = "v1.1.0", iwd = "v1.1.5" }}
'''
        cases = [
            ("held since 2026-11-01: Christopher tests it first.",
             {"holds": hold.format(release="v1.2.0")}, lambda github: None),
            ("BG pins v1.2.0 in only some of its recipes", {"extra": "v1.2.0"},
             lambda github: None),
            ("its newest release v0.9.0 is numbered below the pin v1.1.0", {},
             lambda github: github.latest.update({TESTBOT: "v0.9.0"})),
            ("its newest release is unknown", {}, lambda github: github.broken.add(TESTBOT)),
            ("the job already prepared it as #7 (closed)", {},
             lambda github: github.heads.update({IWD_PIN_BRANCH: [{"number": 7,
                                                                   "state": "closed"}]})),
            ("the job already prepared it as #8 (merged)", {},
             lambda github: github.heads.update({BG_BRANCH: [
                 {"number": 8, "state": "closed", "merged_at": "2026-11-01T10:00:00Z"}]})),
            ("#9 already changes `recipes/iwd/mods/testbot.toml`; the job waits", {},
             lambda github: (github.open.update({IWD_BRANCH: [
                 {"number": 9, "head": {"ref": "claude/pin-testbot-iwd"}}]}),
                 github.files.update({9: [{"filename": "recipes/iwd/mods/testbot.toml"}]}))),
            ("#10 already changes `manifest/artifacts/testbot-1.1.0.toml`", {},
             lambda github: (github.open.update({NEXT: [
                 {"number": 10, "head": {"ref": "claude/other"}}]}),
                 github.files.update({10: [{"filename": "manifest/artifacts/testbot-1.2.0.toml",
                                            "previous_filename":
                                                "manifest/artifacts/testbot-1.1.0.toml"}]}))),
        ]
        for reason, build, change in cases:
            with self.subTest(reason=reason):
                fixture = self.fixture(**build)
                change(fixture.github)
                item, = fixture.plan("testbot")
                self.assertIsNotNone(item.stop)
                self.assertIn(reason, item.stop)

    def test_what_does_not_stop_a_release(self) -> None:
        # A hold for an older release, an open pull request on other files, and the job's
        # own pull request for an older release, which the new one supersedes.
        fixture = self.fixture(holds='''
[[holds]]
mod = "testbot"
release = "v1.1.5"
reason = "Older."
decided = "2026-10-01"
installers = { bg = "v1.1.0", iwd = "v1.1.5" }
''')
        github = fixture.github
        github.open[NEXT] = [{"number": 11, "head": {"ref": "claude/docs"}},
                             {"number": 12, "head": {"ref": "own-mod-pin/bg/testbot-v1.1.5"}}]
        github.files[11] = [{"filename": "docs/handover.md"}]
        github.files[12] = [{"filename": "manifest/mods/testbot.toml"}]
        item, = fixture.plan("testbot")
        self.assertIsNone(item.stop)
        self.assertEqual(item.supersedes, [12])


class RunTests(unittest.TestCase):
    def fixture(self, **build) -> JobFixture:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        fixture = JobFixture(Path(directory.name))
        fixture.build(**build)
        return fixture

    def test_it_pushes_a_branch_per_installer_and_opens_linked_draft_pull_requests(self) -> None:
        fixture = self.fixture()
        preparer = fake_preparer()
        result, = fixture.run(preparer)
        self.assertEqual((result.action, result.detail), ("pull requests", "opened #100, #101"))
        self.assertEqual(preparer.calls, [("v1.2.0", ["bg"]), ("v1.2.0", ["iwd"])])

        self.assertEqual(fixture.origin_branches(), sorted([NEXT, IWD_BRANCH, BG_BRANCH,
                                                            IWD_PIN_BRANCH]))
        for branch, base, changed in (
            (BG_BRANCH, NEXT, ["manifest/artifacts/testbot-1.2.0.toml",
                               "manifest/mods/testbot.toml"]),
            (IWD_PIN_BRANCH, IWD_BRANCH, ["recipes/iwd/artifacts/testbot-1.2.0.toml",
                                          "recipes/iwd/mods/testbot.toml"]),
        ):
            with self.subTest(branch=branch):
                self.assertEqual(fixture.git(fixture.origin, "rev-parse", f"{branch}^"),
                                 fixture.git(fixture.origin, "rev-parse", base))
                self.assertEqual(fixture.git(fixture.origin, "log", "-1", "--format=%an <%ae>",
                                             branch), f"{job.BOT_NAME} <{job.BOT_EMAIL}>")
                self.assertEqual(fixture.git(fixture.origin, "diff", "--name-only",
                                             f"{base}..{branch}").split(), changed)

        created = fixture.github.created
        self.assertEqual([(pull["title"], pull["head"], pull["base"]) for pull in created], [
            ("Pin Testbot v1.2.0 in BG", BG_BRANCH, NEXT),
            ("Pin Testbot v1.2.0 in IWD", IWD_PIN_BRANCH, IWD_BRANCH),
        ])
        self.assertIn("Prepared by the [daily pin job](https://example.invalid/run)",
                      created[0]["body"])
        self.assertIn("never merges by itself", created[0]["body"])
        self.assertIn("## Testbot v1.2.0\n\nPrepared for bg.", created[0]["body"])
        # The workflow token's pull requests start no checks, so the job starts them, where
        # the branch has the workflow.
        self.assertEqual(fixture.github.dispatched, [("ci.yml", BG_BRANCH),
                                                     ("own-mod-pins.yml", BG_BRANCH),
                                                     ("ci.yml", IWD_PIN_BRANCH)])
        for number in (100, 101):
            body = fixture.github.edited[number]
            self.assertIn("### Every installer in the same step", body)
            self.assertIn(f"- BG: #100, into `{NEXT}`", body)
            self.assertIn(f"- IWD: #101, into `{IWD_BRANCH}`", body)
            self.assertIn("merge the one into the installer's own branch first", body)
        # Its worktrees are gone again.
        self.assertEqual(len(fixture.git(fixture.root, "worktree", "list").splitlines()), 1)

    def test_a_superseded_pull_request_is_named_in_the_new_one(self) -> None:
        fixture = self.fixture()
        fixture.github.open[IWD_BRANCH] = [{"number": 12,
                                            "head": {"ref": "own-mod-pin/iwd/testbot-v1.1.9"}}]
        fixture.github.files[12] = [{"filename": "recipes/iwd/mods/testbot.toml"}]
        fixture.run(fake_preparer())
        self.assertIn("Supersedes #12, the job's pull request for an older release; close it.",
                      fixture.github.created[1]["body"])

    def test_a_release_that_needs_an_agent_gets_one_issue_and_no_branch(self) -> None:
        fixture = self.fixture()
        result, = fixture.run(fake_preparer({"iwd"}))
        self.assertEqual(result.action, "needs an agent")
        self.assertIn("(issue #201)", result.detail)
        self.assertEqual(fixture.origin_branches(), sorted([NEXT, IWD_BRANCH]))
        self.assertEqual(fixture.github.created, [])
        title, body = fixture.github.created_issues[0]
        self.assertEqual(title, "Pin Testbot v1.2.0 by hand")
        self.assertIn("for BG and IWD:", body)
        self.assertIn("> cannot tell which asset of v1.2.0 replaces testbot-v1.1.5.zip", body)
        # The next day, the open issue is enough.
        fixture.github.issues[title] = {"number": 201}
        result, = fixture.run(fake_preparer({"iwd"}))
        self.assertIn("(issue #201)", result.detail)
        self.assertEqual(len(fixture.github.created_issues), 1)

    def test_a_dry_run_pushes_opens_and_starts_nothing(self) -> None:
        fixture = self.fixture()
        result, = fixture.run(fake_preparer(), dry_run=True)
        self.assertEqual(result.action, "dry run")
        self.assertIn(f"`{BG_BRANCH}` into `{NEXT}`", result.detail)
        self.assertEqual(fixture.origin_branches(), sorted([NEXT, IWD_BRANCH]))
        self.assertEqual((fixture.github.created, fixture.github.dispatched,
                          fixture.github.created_issues), ([], [], []))

    def test_a_left_over_branch_without_pull_request_is_prepared_again(self) -> None:
        fixture = self.fixture()
        # A stopped run left a branch whose commit the new preparation does not build on.
        (fixture.root / "left-over.txt").write_text("left over\n", encoding="utf-8")
        fixture.git(fixture.root, "add", "left-over.txt")
        fixture.git(fixture.root, "commit", "--quiet", "-m", "left over")
        fixture.git(fixture.root, "push", "--quiet", "origin", f"HEAD:refs/heads/{BG_BRANCH}")
        fixture.git(fixture.root, "reset", "--quiet", "--hard", f"origin/{NEXT}")
        fixture.run(fake_preparer())
        self.assertEqual(fixture.git(fixture.origin, "rev-parse", f"{BG_BRANCH}^"),
                         fixture.git(fixture.origin, "rev-parse", NEXT))
        self.assertEqual(fixture.github.created[0]["head"], BG_BRANCH)

    def test_a_failure_is_reported_and_the_other_mods_go_on(self) -> None:
        fixture = self.fixture()
        fixture.github.tag_lists[TESTREMOTE].append(("v0.3.0", "6" * 40))

        def preparer(tree, own_mod, release, installers, work):
            if own_mod.id == "testbot":
                raise RuntimeError("the curated recipe generation failed")
            return fake_preparer()(tree, own_mod, release, installers, work)

        results = fixture.run(preparer)
        self.assertEqual([(result.item.mod.id, result.action) for result in results],
                         [("testbot", "failed"), ("testremote", "pull requests")])
        self.assertEqual(results[0].detail, "the curated recipe generation failed")


class FakeResponse(io.BytesIO):
    def __enter__(self):
        return self

    def __exit__(self, *exc) -> None:
        self.close()


class ApiTests(unittest.TestCase):
    """The requests JobGitHub sends; the run tests replace it with a fake."""

    def requests(self, call, answers: list[bytes]) -> list:
        sent = []

        def urlopen(request, timeout):
            sent.append(request)
            return FakeResponse(answers[len(sent) - 1])

        with mock.patch("urllib.request.urlopen", side_effect=urlopen):
            call(job.JobGitHub("Owner/repo", "https://api.example", "token"))
        return sent

    def test_writes_open_drafts_and_start_workflows(self) -> None:
        create, = self.requests(lambda github: github.create_pull(
            "Pin X v2 in BG", "own-mod-pin/bg/x-v2", "claude/next", "Body"), [b'{"number": 5}'])
        self.assertEqual((create.get_method(), create.full_url),
                         ("POST", "https://api.example/repos/Owner/repo/pulls"))
        self.assertEqual(json.loads(create.data), {"title": "Pin X v2 in BG",
                                                   "head": "own-mod-pin/bg/x-v2",
                                                   "base": "claude/next", "body": "Body",
                                                   "draft": True})
        self.assertEqual(create.get_header("Authorization"), "Bearer token")
        dispatch, = self.requests(lambda github: github.dispatch("ci.yml", "own-mod-pin/bg/x"),
                                  [b""])
        self.assertEqual((dispatch.get_method(), dispatch.full_url, json.loads(dispatch.data)),
                         ("POST", "https://api.example/repos/Owner/repo/actions/workflows/"
                                  "ci.yml/dispatches", {"ref": "own-mod-pin/bg/x"}))
        edit, = self.requests(lambda github: github.edit_pull(5, "New"), [b"{}"])
        self.assertEqual((edit.get_method(), json.loads(edit.data)), ("PATCH", {"body": "New"}))

    def test_reads_quote_branches_and_match_issue_titles_exactly(self) -> None:
        pulls, = self.requests(lambda github: github.open_pulls("claude/next"), [b"[]"])
        self.assertIn("state=open&base=claude%2Fnext&per_page=100&page=1", pulls.full_url)
        heads, = self.requests(lambda github: github.pulls_from("own-mod-pin/bg/x-v2"), [b"[]"])
        self.assertIn("state=all&head=Owner%3Aown-mod-pin%2Fbg%2Fx-v2", heads.full_url)
        found = []
        search, = self.requests(lambda github: found.append(
            github.issue_titled("Pin X v2 by hand")),
            [json.dumps({"items": [{"title": "Pin X v2 by hand, again", "number": 1},
                                   {"title": "Pin X v2 by hand", "number": 2}]}).encode()])
        self.assertIn("repo%3AOwner/repo%20is%3Aissue%20in%3Atitle%20%22Pin%20X%20v2%20by%20"
                      "hand%22", search.full_url)
        self.assertEqual(found, [{"title": "Pin X v2 by hand", "number": 2}])

    def test_a_refused_write_names_the_reason(self) -> None:
        def urlopen(request, timeout):
            raise urllib.error.HTTPError(
                request.full_url, 403, "Forbidden", {},
                io.BytesIO(b'{"message": "GitHub Actions is not permitted to create or '
                           b'approve pull requests."}'))

        github = job.JobGitHub("Owner/repo", "https://api.example", "token")
        with mock.patch("urllib.request.urlopen", side_effect=urlopen), \
                self.assertRaisesRegex(GitHubError, "HTTP 403: GitHub Actions is not permitted"):
            github.create_pull("T", "h", "b", "B")


class CommandLineTests(unittest.TestCase):
    def test_plan_writes_whether_there_is_work_and_run_reports_it(self) -> None:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        fixture = JobFixture(Path(directory.name))
        fixture.build()

        def main(*arguments: str) -> tuple[int, str]:
            stdout = io.StringIO()
            with mock.patch.object(job, "JobGitHub", return_value=fixture.github), \
                    mock.patch.object(job.prep, "author_command", return_value=[]), \
                    mock.patch.object(job, "make_preparer", return_value=fake_preparer()), \
                    redirect_stdout(stdout), redirect_stderr(io.StringIO()):
                code = job.main([*arguments, "--next-release-branch", NEXT, "--root",
                                 str(fixture.root), "--repository", f"{ACCOUNT}/collection"])
            return code, stdout.getvalue()

        output = fixture.work / "github-output"
        summary = fixture.work / "summary.md"
        code, text = main("plan", "--github-output", str(output), "--summary", str(summary))
        self.assertEqual(code, 0)
        self.assertEqual(output.read_text(encoding="utf-8"), "work=true\n")
        self.assertIn(f"| Testbot | v1.2.0 | goes ahead: `{BG_BRANCH}` into `{NEXT}`", text)
        self.assertEqual(summary.read_text(encoding="utf-8"), text)

        code, text = main("run", "--dry-run")
        self.assertEqual(code, 0)
        self.assertIn("| Testbot | v1.2.0 | dry run: would open", text)

        fixture.github.latest[TESTBOT] = "v1.1.0"
        fixture.github.heads.clear()
        code, _ = main("plan", "--github-output", str(output), "--mod", "testbot")
        self.assertEqual(output.read_text(encoding="utf-8"), "work=true\nwork=false\n")
        self.assertEqual(main("plan", "--mod", "nosuchmod")[0], 2)


if __name__ == "__main__":
    unittest.main()
