"""The daily pin job: draft pull requests for new releases of our own mods.

Christopher, 2026-10-28: a GitHub Actions job looks for new releases of our mods once a
day, and on demand. It prepares each new release in every installer that ships the mod and
opens draft pull requests for his test; it never merges. The pin rules are in
`docs/own-mod-pins.md`. The workflow, `.github/workflows/own-mod-pin-job.yml`, lives on
`main`, which is the shared source and release line for both installers.

`plan` only reads. For every mod in `manifest/own-mods.toml` it compares each installer's
pins with the mod's newest release. Every installer that ships the mod and is behind gets
the release in the same run, or none does. These stop a release, with the reason:

- a hold for that release;
- a newest release numbered below the pin, which is a new line of the mod;
- an installer that pins the release in only some of its recipes;
- a pull request the job already opened for that release, open or closed;
- an open pull request that already changes the mod's pin, unless it is the job's own pull
  request for an older release, which the new one supersedes.

`run` plans again, then prepares each release that goes ahead with
`tools/prepare_own_mod_pin.py`, in one fresh worktree for both installers. It pushes
the branch `own-mod-pin/<installers>/<mod>-<release>`, opens a draft pull request against
`main`, and starts its checks, because pull requests that the workflow
token opens start none by themselves. A release that needs an agent gets an issue and no
pull request. The job never merges, closes or approves anything.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass, field
from pathlib import Path
from typing import Callable

if __package__ in {None, ""}:
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from tools import prepare_own_mod_pin as prep  # noqa: E402
from tools.own_mod_pins import (  # noqa: E402
    GITHUB_API,
    REGISTRY,
    GitHub,
    GitHubError,
    Installer,
    Newest,
    OwnMod,
    Pin,
    Registry,
    RegistryError,
    SourceError,
    TreeSource,
    collect_pins,
    installer_source,
    is_newest,
    load_registry,
    newest_release,
    tag_order,
)


BRANCH_PREFIX = "own-mod-pin"
BOT_NAME = "github-actions[bot]"
BOT_EMAIL = "41898282+github-actions[bot]@users.noreply.github.com"
# The checks a pull request of the job gets, where its branch has the workflow.
CHECK_WORKFLOWS = ("ci.yml", "own-mod-pins.yml")


# --- GitHub ---------------------------------------------------------------------------


class JobGitHub(GitHub):
    """The check's read calls, plus what the job writes: pull requests, issues and
    workflow runs. Writes are never retried, so that a slow answer cannot open a pull
    request twice."""

    def __init__(self, repository: str, api: str = GITHUB_API, token: str | None = None) -> None:
        super().__init__(api, token)
        self.repository = repository

    def _pages(self, path: str, limit: int = 30) -> list[dict]:
        separator = "&" if "?" in path else "?"
        entries: list[dict] = []
        for page in range(1, limit + 1):
            data = self._get(f"{path}{separator}per_page=100&page={page}")
            if not isinstance(data, list):
                raise GitHubError(f"GitHub API {path}: unexpected answer")
            entries.extend(data)
            if len(data) < 100:
                break
        return entries

    def _send(self, method: str, path: str, payload: dict) -> object:
        request = urllib.request.Request(f"{self.api}{path}", method=method,
                                         data=json.dumps(payload).encode("utf-8"))
        request.add_header("Accept", "application/vnd.github+json")
        request.add_header("X-GitHub-Api-Version", "2022-11-28")
        request.add_header("User-Agent", "chriz-bg-collection-own-mod-pin-job")
        request.add_header("Content-Type", "application/json")
        if self.token:
            request.add_header("Authorization", f"Bearer {self.token}")
        try:
            with urllib.request.urlopen(request, timeout=60) as response:
                body = response.read()
        except urllib.error.HTTPError as error:
            detail = error.read().decode("utf-8", "replace")
            try:
                detail = json.loads(detail).get("message", detail)
            except (ValueError, AttributeError):
                pass
            raise GitHubError(f"GitHub API {method} {path}: HTTP {error.code}: {detail}") from None
        except (urllib.error.URLError, TimeoutError, OSError) as error:
            reason = getattr(error, "reason", error)
            raise GitHubError(f"GitHub API {method} {path}: {reason}") from None
        return json.loads(body) if body else None

    def open_pulls(self, base: str) -> list[dict]:
        return self._pages(f"/repos/{self.repository}/pulls?state=open&base="
                           f"{urllib.parse.quote(base, safe='')}")

    def pull_files(self, number: int) -> list[dict]:
        return self._pages(f"/repos/{self.repository}/pulls/{number}/files")

    def pulls_from(self, branch: str) -> list[dict]:
        head = f"{self.repository.split('/')[0]}:{branch}"
        return self._pages(f"/repos/{self.repository}/pulls?state=all&head="
                           f"{urllib.parse.quote(head, safe='')}")

    def create_pull(self, title: str, head: str, base: str, body: str) -> dict:
        return self._send("POST", f"/repos/{self.repository}/pulls",
                          {"title": title, "head": head, "base": base, "body": body,
                           "draft": True})

    def edit_pull(self, number: int, body: str) -> None:
        self._send("PATCH", f"/repos/{self.repository}/pulls/{number}", {"body": body})

    def issue_titled(self, title: str) -> dict | None:
        query = urllib.parse.quote(f'repo:{self.repository} is:issue in:title "{title}"')
        data = self._get(f"/search/issues?q={query}&per_page=100")
        items = data.get("items", []) if isinstance(data, dict) else []
        return next((item for item in items if item.get("title") == title), None)

    def create_issue(self, title: str, body: str) -> dict:
        return self._send("POST", f"/repos/{self.repository}/issues",
                          {"title": title, "body": body})

    def dispatch(self, workflow: str, ref: str) -> None:
        self._send("POST", f"/repos/{self.repository}/actions/workflows/{workflow}/dispatches",
                   {"ref": ref})


# --- Planning -------------------------------------------------------------------------


@dataclass(frozen=True)
class Target:
    """Both installers on the shared release line, read from `source`."""

    branch: str
    installers: tuple[Installer, ...]
    source: TreeSource


@dataclass
class Part:
    """One branch's share of a release: the installers it pins there, and their pins."""

    target: Target
    installers: list[Installer]
    pins: list[Pin]

    def files(self) -> set[str]:
        """The files a pull request changes when it changes these pins."""
        return ({f"{pin.recipe}/mods/{pin.mod_id}.toml" for pin in self.pins}
                | {f"{pin.recipe}/artifacts/{pin.artifact_id}.toml" for pin in self.pins})


@dataclass
class Item:
    """One mod's newest release, and whether the job pins it."""

    mod: OwnMod
    release: str
    parts: list[Part] = field(default_factory=list)
    stop: str | None = None
    supersedes: list[int] = field(default_factory=list)

    def branch(self, part: Part) -> str:
        installers = "+".join(installer.id for installer in part.installers)
        return f"{BRANCH_PREFIX}/{installers}/{self.mod.id}-{self.release}"

    def installer_names(self) -> str:
        return " and ".join(installer.name for part in self.parts
                            for installer in part.installers)


def targets(root: Path, registry: Registry, next_release: str) -> list[Target]:
    """Prepare every installer's pin together on the shared release line."""
    for installer in registry.installers:
        installer_source(root, installer)
    return [Target(next_release, registry.installers, TreeSource(root))]


def pinned_tag(pin: Pin, tags: list[tuple[str, str]]) -> str | None:
    if pin.kind == "github-commit-zip":
        return next((name for name, commit in tags if commit == pin.release.lower()), None)
    return pin.release


class Planner:
    def __init__(self, registry: Registry, github: JobGitHub) -> None:
        self.registry = registry
        self.github = github
        self._open: dict[str, list[tuple[int, str, set[str]]]] = {}

    def open_pulls(self, branch: str) -> list[tuple[int, str, set[str]]]:
        """Each open pull request against a branch: number, head branch, changed files."""
        if branch not in self._open:
            entries = []
            for pull in self.github.open_pulls(branch):
                files: set[str] = set()
                for entry in self.github.pull_files(pull["number"]):
                    files.add(entry["filename"])
                    if entry.get("previous_filename"):
                        files.add(entry["previous_filename"])
                entries.append((pull["number"], pull["head"]["ref"], files))
            self._open[branch] = entries
        return self._open[branch]

    def stop_reason(self, item: Item, newest: Newest) -> str | None:
        own_mod, release = item.mod, item.release
        hold = self.registry.hold_for(own_mod.id)
        if hold is not None and hold.release == release:
            return f"held since {hold.decided}: {hold.reason}"
        for part in item.parts:
            for installer in part.installers:
                pins = [pin for pin in part.pins if pin.installer == installer.id]
                if any(is_newest(pin, newest) for pin in pins):
                    return (f"{installer.name} pins {release} in only some of its recipes; "
                            f"the own-mods check reports it")
        newest_order = tag_order(release)
        if newest_order is not None:
            tags = self.github.tags(own_mod.repository)
            for pin in (pin for part in item.parts for pin in part.pins):
                old = pinned_tag(pin, tags)
                old_order = tag_order(old) if old else None
                if old_order is not None and old_order > newest_order:
                    return (f"its newest release {release} is numbered below the pin {old}, "
                            f"so it starts a new line of the mod; that is Christopher's choice")
        for part in item.parts:
            for pull in self.github.pulls_from(item.branch(part)):
                state = "merged" if pull.get("merged_at") else pull.get("state", "?")
                return f"the job already prepared it as #{pull['number']} ({state})"
        own_older = re.compile(rf"^{re.escape(BRANCH_PREFIX)}/[^/]+/{re.escape(own_mod.id)}-.+$")
        for part in item.parts:
            wanted = part.files()
            for number, head, files in self.open_pulls(part.target.branch):
                if not files & wanted:
                    continue
                if own_older.match(head):
                    item.supersedes.append(number)
                    continue
                return (f"#{number} already changes `{sorted(files & wanted)[0]}`; the job "
                        f"waits until it is merged or closed")
        return None


def plan(
    root: Path,
    registry: Registry,
    github: JobGitHub,
    next_release: str,
    remote: str = "origin",
    only: str | None = None,
) -> list[Item]:
    groups = targets(root, registry, next_release)
    errors: list[str] = []
    pins: dict[tuple[str, str], list[Pin]] = {}
    for target in groups:
        for installer in target.installers:
            pins[(target.branch, installer.id)] = collect_pins(registry, installer,
                                                               target.source, errors).pins
    if errors:
        raise SourceError("; ".join(errors))

    planner = Planner(registry, github)
    items: list[Item] = []
    for own_mod in registry.mods:
        if only is not None and own_mod.id != only:
            continue
        shipped = [(target, installer, [pin for pin in pins[(target.branch, installer.id)]
                                        if pin.repository.lower() == own_mod.repository.lower()])
                   for target in groups for installer in target.installers]
        shipped = [entry for entry in shipped if entry[2]]
        if not shipped:
            continue  # No installer ships it; adding it to one is Christopher's choice.
        try:
            newest = newest_release(github, own_mod, True)
        except GitHubError as error:
            items.append(Item(own_mod, "?", stop=f"its newest release is unknown: {error}"))
            continue
        behind = [(target, installer, mod_pins) for target, installer, mod_pins in shipped
                  if not all(is_newest(pin, newest) for pin in mod_pins)]
        if not behind:
            continue
        item = Item(own_mod, newest.tag)
        for target in groups:
            entries = [(installer, mod_pins) for owner, installer, mod_pins in behind
                       if owner is target]
            if entries:
                item.parts.append(Part(target, [installer for installer, _ in entries],
                                       [pin for _, mod_pins in entries for pin in mod_pins]))
        item.stop = planner.stop_reason(item, newest)
        items.append(item)
    return items


# --- Running --------------------------------------------------------------------------


# Prepares one part in a tree: (tree, mod, release, installer ids, work) -> summary markdown.
Preparer = Callable[[Path, OwnMod, str, list[str], Path], str]


def make_preparer(registry: Registry, github: GitHub, author: list[str], tools_root: Path,
                  today: str) -> Preparer:
    def preparer(tree: Path, own_mod: OwnMod, release: str, installers: list[str],
                 work: Path) -> str:
        outcome = prep.prepare(tree, registry, own_mod, github, author, tag=release,
                               installers=installers, today=today, work=work,
                               tools_root=tools_root)
        return prep.summary(outcome)
    return preparer


def git(cwd: Path, *args: str) -> str:
    result = subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True,
                            encoding="utf-8", errors="replace", check=False)
    if result.returncode != 0:
        raise RuntimeError(f"git {' '.join(args[:2])} failed: "
                           f"{(result.stderr or result.stdout).strip()}")
    return result.stdout.strip()


def remove_worktree(root: Path, tree: Path) -> None:
    subprocess.run(["git", "worktree", "remove", "--force", str(tree)], cwd=root,
                   capture_output=True, check=False)
    shutil.rmtree(tree, ignore_errors=True)
    subprocess.run(["git", "worktree", "prune"], cwd=root, capture_output=True, check=False)


def commit_and_push(tree: Path, branch: str, subject: str, body: str, remote: str) -> None:
    git(tree, "add", "--all")
    if subprocess.run(["git", "diff", "--cached", "--quiet"], cwd=tree,
                      capture_output=True, check=False).returncode == 0:
        raise RuntimeError(f"{branch}: the preparation changed no file")
    git(tree, "-c", f"user.name={BOT_NAME}", "-c", f"user.email={BOT_EMAIL}", "commit",
        "--quiet", "-m", subject, "-m", body)
    # The branch is the job's own; one without a pull request is left from a stopped run.
    git(tree, "push", "--force", "--quiet", remote, f"HEAD:refs/heads/{branch}")


def pull_title(item: Item, part: Part) -> str:
    names = " and ".join(installer.name for installer in part.installers)
    return f"Pin {item.mod.name} {item.release} in {names}"


def pull_body(item: Item, part: Part, prepared: str, run_url: str) -> str:
    names = " and ".join(installer.name for installer in part.installers)
    lines = [
        f"Prepared by the [daily pin job]({run_url}) with `tools/prepare_own_mod_pin.py`: "
        f"{item.mod.name} {item.release} for {names}, on `{part.target.branch}`.",
        "",
        "> Draft for Christopher's test; it never merges by itself. An agent first completes "
        "\"Left for the agent\" on this branch (`docs/own-mod-pins.md`).",
        "",
    ]
    if item.supersedes:
        lines += [f"Supersedes {', '.join(f'#{number}' for number in item.supersedes)}, the "
                  f"job's pull request for an older release; close it.", ""]
    return "\n".join(lines) + prepared


@dataclass
class Result:
    item: Item
    action: str
    detail: str


def needs_agent(github: JobGitHub, item: Item, reason: str, dry_run: bool,
                run_url: str) -> Result:
    title = f"Pin {item.mod.name} {item.release} by hand"
    if dry_run:
        return Result(item, "needs an agent", reason)
    existing = github.issue_titled(title)
    if existing is not None:
        return Result(item, "needs an agent", f"{reason} (issue #{existing['number']})")
    body = "\n".join([
        f"The [daily pin job]({run_url}) could not prepare {item.mod.name} {item.release} for "
        f"{item.installer_names()}:",
        "",
        f"> {reason}",
        "",
        "An agent pins it by hand, in every installer in the same step "
        "(`docs/own-mod-pins.md`); `tools/prepare_own_mod_pin.py` may still do part of it. "
        "The job opens no pull request for this release.",
    ])
    issue = github.create_issue(title, body)
    return Result(item, "needs an agent", f"{reason} (issue #{issue['number']})")


def run(
    root: Path,
    github: JobGitHub,
    items: list[Item],
    preparer: Preparer,
    *,
    remote: str,
    work: Path,
    dry_run: bool,
    run_url: str,
) -> list[Result]:
    results: list[Result] = []
    for item in items:
        if item.stop:
            results.append(Result(item, "stopped", item.stop))
            continue
        trees: list[Path] = []
        try:
            prepared: list[tuple[Part, Path, str]] = []
            for index, part in enumerate(item.parts):
                tree = work / f"{item.mod.id}-{index}"
                git(root, "worktree", "add", "--detach", "--force", str(tree),
                    f"{remote}/{part.target.branch}")
                trees.append(tree)
                text = preparer(tree, item.mod, item.release,
                                [installer.id for installer in part.installers],
                                work / f"prepare-{item.mod.id}-{index}")
                prepared.append((part, tree, text))
            if dry_run:
                results.append(Result(item, "dry run", "would open " + ", ".join(
                    f"`{item.branch(part)}` into `{part.target.branch}`"
                    for part, _, _ in prepared)))
                continue
            pulls: list[tuple[Part, int, str]] = []
            notes: list[str] = []
            for part, tree, text in prepared:
                branch = item.branch(part)
                subject = pull_title(item, part)
                commit_and_push(tree, branch, subject,
                                f"Prepared by tools/prepare_own_mod_pin.py in the daily pin "
                                f"job: {run_url}", remote)
                body = pull_body(item, part, text, run_url)
                pull = github.create_pull(subject, branch, part.target.branch, body)
                pulls.append((part, pull["number"], body))
                for workflow in CHECK_WORKFLOWS:
                    if (tree / ".github/workflows" / workflow).is_file():
                        try:
                            github.dispatch(workflow, branch)
                        except GitHubError as error:
                            notes.append(f"#{pull['number']}: {workflow} did not start: {error}")
            detail = "opened " + ", ".join(f"#{number}" for _, number, _ in pulls)
            results.append(Result(item, "pull requests", "; ".join([detail, *notes])))
        except prep.NeedsAgent as error:
            results.append(needs_agent(github, item, str(error), dry_run, run_url))
        except (GitHubError, SourceError, RuntimeError, OSError) as error:
            results.append(Result(item, "failed", str(error)))
        finally:
            for tree in trees:
                remove_worktree(root, tree)
    return results


# --- Reports --------------------------------------------------------------------------


def table(rows: list[tuple[str, str, str]]) -> str:
    lines = ["| Mod | Newest | Result |", "|---|---|---|"]
    lines += [f"| {mod} | {release} | {result.replace('|', '/')} |"
              for mod, release, result in rows]
    return "\n".join(lines)


def render_plan(items: list[Item]) -> str:
    rows = []
    for item in items:
        if item.stop:
            rows.append((item.mod.name, item.release, f"stopped: {item.stop}"))
        else:
            branches = ", ".join(f"`{item.branch(part)}` into `{part.target.branch}`"
                                 for part in item.parts)
            rows.append((item.mod.name, item.release, f"goes ahead: {branches}"))
    if not rows:
        return "## Own mod pin job: plan\n\nEvery installer is on the newest release of our mods.\n"
    return "## Own mod pin job: plan\n\n" + table(rows) + "\n"


def render_results(results: list[Result]) -> str:
    if not results:
        return "## Own mod pin job\n\nEvery installer is on the newest release of our mods.\n"
    rows = [(result.item.mod.name, result.item.release, f"{result.action}: {result.detail}")
            for result in results]
    return "## Own mod pin job\n\n" + table(rows) + "\n"


def origin_repository(root: Path) -> str:
    url = git(root, "remote", "get-url", "origin")
    match = re.search(r"github\.com[:/]([^/]+/[^/]+?)(?:\.git)?/?$", url)
    if not match:
        raise RegistryError(f"cannot tell the GitHub repository from the origin URL {url}")
    return match.group(1)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("command", choices=("plan", "run"))
    parser.add_argument("--next-release-branch", required=True,
                        help="the branch whose recipes and tools the job uses")
    parser.add_argument("--root", type=Path, default=Path.cwd(),
                        help="a checkout of the next-release branch (default: here)")
    parser.add_argument("--remote", default="origin")
    parser.add_argument("--repository", default=os.environ.get("GITHUB_REPOSITORY"),
                        help="owner/name (default: GITHUB_REPOSITORY, else the origin URL)")
    parser.add_argument("--mod", help="only this mod (its id in the registry)")
    parser.add_argument("--author", type=Path, help="the chriz-bg-author executable")
    parser.add_argument("--dry-run", action="store_true",
                        help="prepare and report, but push, open and start nothing")
    parser.add_argument("--today", default=dt.datetime.now(dt.timezone.utc).date().isoformat())
    parser.add_argument("--fetch-depth", type=int,
                        help="fetch the installer branches this shallow (CI)")
    parser.add_argument("--summary", type=Path, help="append the report here (job summary)")
    parser.add_argument("--github-output", type=Path,
                        help="plan: write work=true or work=false here")
    parser.add_argument("--api", default=os.environ.get("CEBG_GITHUB_API", GITHUB_API),
                        help=argparse.SUPPRESS)
    args = parser.parse_args(argv)

    root = args.root.resolve()
    try:
        dt.date.fromisoformat(args.today)
        registry = load_registry(root / REGISTRY)
        if args.mod is not None and all(own_mod.id != args.mod for own_mod in registry.mods):
            raise RegistryError(f"{args.mod} is not a mod in {REGISTRY}")
        repository = args.repository or origin_repository(root)
    except (RegistryError, ValueError, RuntimeError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    github = JobGitHub(repository, args.api, token)
    try:
        git(root, "fetch", "--no-tags", "--quiet",
            *([f"--depth={args.fetch_depth}"] if args.fetch_depth else []), args.remote,
            f"+refs/heads/{args.next_release_branch}:refs/remotes/{args.remote}/"
            f"{args.next_release_branch}")
        items = plan(root, registry, github, args.next_release_branch, args.remote, args.mod)
    except (SourceError, GitHubError, RuntimeError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1

    if args.command == "plan":
        text = render_plan(items)
        if args.github_output:
            work = "true" if any(item.stop is None for item in items) else "false"
            with args.github_output.open("a", encoding="utf-8") as output:
                output.write(f"work={work}\n")
        code = 0
    else:
        server = os.environ.get("GITHUB_SERVER_URL", "https://github.com")
        run_id = os.environ.get("GITHUB_RUN_ID")
        run_url = (f"{server}/{repository}/actions/runs/{run_id}" if run_id
                   else f"{server}/{repository}/actions")
        author = (prep.author_command(root, args.author)
                  if any(item.stop is None for item in items) else [])
        with tempfile.TemporaryDirectory(prefix="own-mod-pin-job-") as work:
            results = run(root, github, items,
                          make_preparer(registry, github, author, root, args.today),
                          remote=args.remote, work=Path(work), dry_run=args.dry_run,
                          run_url=run_url)
        text = render_results(results)
        code = 1 if any(result.action == "failed" for result in results) else 0
    sys.stdout.write(text)
    if args.summary:
        with args.summary.open("a", encoding="utf-8") as summary_file:
            summary_file.write(text)
    return code


if __name__ == "__main__":
    raise SystemExit(main())
