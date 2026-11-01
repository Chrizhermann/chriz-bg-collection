#!/usr/bin/env python3
"""Nightly check that every pinned mod download link still serves the reviewed file.

Reads the artifact pins (URL, length, SHA-256) that the installers' recipes use and asks each
link, as cheaply as it can, whether it still serves that file. No AI, no third-party packages.
See docs/mod-link-check.md.

    check_mod_links.py check --installer NAME DIR RECIPE [RECIPE ...] [--installer ...]
    check_mod_links.py issue --report REPORT.json --repo OWNER/NAME

`check` classifies each link as same, changed, gone, moved or unreachable and writes a report.
`issue` opens, updates or closes the one "Mod link check" issue from that report.
"""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import http.client
import json
import os
import re
import subprocess
import sys
import time
import tomllib
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Callable

USER_AGENT = "chriz-bg-collection-link-check/1 (+https://github.com/Chrizhermann/chriz-bg-collection)"
TIMEOUT_SECONDS = 20
RETRY_WAIT_SECONDS = 5
REQUEST_DELAY_SECONDS = 0.3
MAX_REDIRECTS = 8
DEFAULT_DOWNLOAD_CAP_MB = 512
MAX_API_BODY = 8 * 1024 * 1024
ISSUE_TITLE = "Mod link check"
GITHUB_API = "https://api.github.com"
API_HEADERS = {
    "Accept": "application/vnd.github+json",
    "X-GitHub-Api-Version": "2022-11-28",
}
# Answers that mean "this server does not do HEAD (for us)", not "the file is gone".
HEAD_REFUSED = frozenset({400, 403, 405, 501})
ARTIFACT_KEYS = frozenset({"artifact_id", "weidu_artifact_id"})
# The app refuses this recipe; its old pins are history, not downloads a player makes.
HISTORICAL_RECIPES = frozenset({"creator-full-current"})
RELEASE_URL = re.compile(r"^https://github\.com/([^/]+)/([^/]+)/releases/download/([^/]+)/([^/?#]+)$")

SAME, CHANGED, GONE, MOVED, UNREACHABLE = "same", "changed", "gone", "moved", "unreachable"
PENDING = "pending"  # unreachable for the first night: remembered, not reported yet
UNVERIFIED = "unverified"  # the metadata cannot confirm the pin and the file is over the limit
REPORTED = (GONE, CHANGED, MOVED, UNREACHABLE)


class LinkCheckError(Exception):
    """The check could not run (bad input or an empty checkout), as opposed to a bad link."""


class TransportError(Exception):
    """A request got no usable answer: timeout, connection or TLS failure, too many redirects."""


@dataclass
class Response:
    status: int
    url: str  # final URL, after redirects
    headers: dict[str, str]  # lower-case names
    body: bytes = b""


@dataclass(frozen=True)
class Installer:
    name: str
    root: Path
    recipes: tuple[str, ...]


@dataclass
class Pin:
    url: str
    sha256: str
    length: int
    kind: str
    filename: str
    redirect_hosts: tuple[str, ...]
    mods: set[str] = field(default_factory=set)
    recipes: dict[str, set[str]] = field(default_factory=dict)  # installer -> recipe labels


@dataclass
class Result:
    pin: Pin
    state: str
    detail: str
    since: str = ""
    streak: int = 0


# --------------------------------------------------------------------------------------------
# Reading the pins


def _collect_ids(node: Any, found: set[str]) -> None:
    if isinstance(node, dict):
        for key, value in node.items():
            if key in ARTIFACT_KEYS and isinstance(value, str):
                found.add(value)
            else:
                _collect_ids(value, found)
    elif isinstance(node, list):
        for item in node:
            _collect_ids(item, found)


def _load_toml(path: Path) -> dict[str, Any]:
    try:
        return tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise LinkCheckError(f"cannot read {path.name}: {error}") from error


def load_pins(installers: list[Installer]) -> tuple[list[Pin], list[dict[str, str]], list[str]]:
    """Every download that a recipe's mods use, one pin per (URL, SHA-256, length).

    Artifact files that no mod uses (superseded pins kept as history) are not downloads anyone
    makes and are left out. Manual downloads cannot be fetched by link and are listed as skipped.
    """
    pins: dict[tuple[str, str, int], Pin] = {}
    skipped: list[dict[str, str]] = []
    notes: list[str] = []
    listed = {Path(recipe).name for installer in installers for recipe in installer.recipes}
    for installer in installers:
        # A branch also carries recipes that another installer ships; only a folder that no
        # installer lists could be a recipe this check forgot.
        for extra in sorted((installer.root / "recipes").glob("*/artifacts")):
            if extra.parent.name not in listed and extra.parent.name not in HISTORICAL_RECIPES:
                notes.append(f"{installer.name}: recipes/{extra.parent.name} has artifacts but is not checked")
        for recipe in installer.recipes:
            directory = installer.root / recipe
            ids: set[str] = set()
            for path in sorted((directory / "mods").glob("*.toml")):
                _collect_ids(_load_toml(path), ids)
            if not ids:
                raise LinkCheckError(f"{installer.name}: {directory} names no artifacts; is the checkout complete?")
            for artifact_id in sorted(ids):
                path = directory / "artifacts" / f"{artifact_id}.toml"
                if not path.is_file():
                    notes.append(f"{installer.name}/{recipe}: artifact {artifact_id} has no file")
                    continue
                artifact = _load_toml(path)
                source = artifact.get("source", {})
                label = f"{artifact.get('name', artifact_id)} {artifact.get('version', '')}".strip()
                if artifact.get("acquisition") != "fetch-only" or source.get("kind") == "manual":
                    skipped.append({"mod": label, "url": str(source.get("url", "")), "reason": "manual download"})
                    continue
                try:
                    key = (source["url"], source["sha256"].lower(), int(source["expected_length"]))
                except (KeyError, TypeError, ValueError) as error:
                    raise LinkCheckError(f"{path.name}: incomplete [source] pin: {error}") from error
                pin = pins.setdefault(
                    key,
                    Pin(
                        url=key[0],
                        sha256=key[1],
                        length=key[2],
                        kind=str(source.get("kind", "")),
                        filename=str(source.get("expected_filename", "")),
                        redirect_hosts=tuple(source.get("redirect_hosts", [])),
                    ),
                )
                pin.mods.add(label)
                pin.recipes.setdefault(installer.name, set()).add(Path(recipe).name)
    if not pins:
        raise LinkCheckError("no pinned downloads found")
    return list(pins.values()), skipped, notes


# --------------------------------------------------------------------------------------------
# Talking to servers


class _NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args: Any, **kwargs: Any) -> None:
        return None


def _describe(error: BaseException) -> str:
    return str(getattr(error, "reason", None) or error) or type(error).__name__


class UrllibTransport:
    """One request at a time, redirects followed by hand so that every hop is known."""

    def __init__(self, token: str | None = None, timeout: float = TIMEOUT_SECONDS) -> None:
        self.token = token
        self.timeout = timeout
        self._opener = urllib.request.build_opener(_NoRedirect)

    def _open(self, method: str, url: str, headers: dict[str, str], keep_open: bool = False, body: bool = False):
        base = {"User-Agent": USER_AGENT, "Accept-Encoding": "identity", **headers}
        for _ in range(MAX_REDIRECTS + 1):
            send = dict(base)
            if self.token and urllib.parse.urlsplit(url).hostname == "api.github.com":
                send["Authorization"] = f"Bearer {self.token}"
            request = urllib.request.Request(url, method=method, headers=send)
            try:
                reply = self._opener.open(request, timeout=self.timeout)
            except urllib.error.HTTPError as error:
                reply = error
            except (urllib.error.URLError, OSError, http.client.HTTPException, ValueError) as error:
                raise TransportError(_describe(error)) from error
            received = {name.lower(): value for name, value in reply.headers.items()}
            location = received.get("location")
            if reply.status in (301, 302, 303, 307, 308) and location:
                reply.close()
                url = urllib.parse.urljoin(url, location)
                if urllib.parse.urlsplit(url).scheme not in ("http", "https"):
                    raise TransportError(f"redirect to {url}")
                continue
            content = b""
            try:
                if body and reply.status == 200:
                    content = reply.read(MAX_API_BODY)
            except (OSError, http.client.HTTPException) as error:
                reply.close()
                raise TransportError(_describe(error)) from error
            response = Response(reply.status, url, received, content)
            if keep_open and reply.status == 200:
                return response, reply
            reply.close()
            return response, None
        raise TransportError("too many redirects")

    def request(self, method: str, url: str, headers: dict[str, str] | None = None, *, body: bool = False) -> Response:
        return self._open(method, url, headers or {}, body=body)[0]

    def hash_body(self, url: str, limit: int) -> tuple[int, str] | None:
        """Stream the file through SHA-256 without keeping it; None when it is over the limit."""
        response, stream = self._open("GET", url, {}, keep_open=True)
        if stream is None:
            raise TransportError(f"HTTP {response.status}")
        digest = hashlib.sha256()
        total = 0
        try:
            while chunk := stream.read(1 << 20):
                total += len(chunk)
                if total > limit:
                    return None
                digest.update(chunk)
        except (OSError, http.client.HTTPException) as error:
            raise TransportError(_describe(error)) from error
        finally:
            stream.close()
        return total, digest.hexdigest()


# --------------------------------------------------------------------------------------------
# Checking one link


def _family(host: str) -> str:
    """GitHub serves one download from several of its own hosts; those are not a move."""
    host = host.lower()
    if host in ("github.com", "githubusercontent.com") or host.endswith((".github.com", ".githubusercontent.com")):
        return "github"
    return host


def _served_length(response: Response) -> int | None:
    if response.status == 206:
        total = response.headers.get("content-range", "").rpartition("/")[2]
        return int(total) if total.isdigit() and int(total) > 0 else None
    length = response.headers.get("content-length", "")
    return int(length) if length.isdigit() and int(length) > 0 else None


def _served_filename(response: Response) -> str:
    match = re.search(r"filename\*?=(?:UTF-8'')?\"?([^\";]+)", response.headers.get("content-disposition", ""), re.I)
    return urllib.parse.unquote(match.group(1)).strip() if match else ""


def _short(digest: str) -> str:
    return digest[:12]


class Checker:
    def __init__(
        self,
        transport: Any,
        state: dict[str, dict[str, Any]],
        today: str,
        *,
        sleep: Callable[[float], None] = time.sleep,
        download_cap: int = DEFAULT_DOWNLOAD_CAP_MB * 1024 * 1024,
    ) -> None:
        self.transport = transport
        self.state = state  # what the previous run learned: url -> entry (read only)
        self.today = today
        self.sleep = sleep
        self.download_cap = download_cap
        self.new_state: dict[str, dict[str, Any]] = {}
        self.downloads = 0
        self._asked: dict[tuple, Response] = {}
        self._hashed: dict[str, tuple[int, str] | None] = {}
        self._learned: dict[str, dict[str, Any]] = {}

    # -- requests, with one retry -----------------------------------------------------------

    def _retrying(self, call: Callable[[], Any], transient: Callable[[Any], bool] = lambda value: False) -> Any:
        for attempt in (0, 1):
            if attempt:
                self.sleep(RETRY_WAIT_SECONDS)
            try:
                value = call()
            except TransportError:
                if attempt:
                    raise
                continue
            if attempt or not transient(value):
                return value
        raise AssertionError("unreachable")

    def _ask(self, method: str, url: str, headers: dict[str, str] | None = None, body: bool = False) -> Response:
        key = (method, url, tuple(sorted((headers or {}).items())), body)
        if key not in self._asked:
            self._asked[key] = self._retrying(
                lambda: self.transport.request(method, url, headers, body=body),
                lambda response: response.status >= 500 or response.status == 429,
            )
        return self._asked[key]

    # -- deciding ---------------------------------------------------------------------------

    def _hash(self, pin: Pin, length: int | None, etag: str | None) -> str | None:
        """SHA-256 of what the link serves now: from the last download if nothing changed since,
        else by downloading. None when the file is over the limit."""
        known = self.state.get(pin.url, {})
        same_file = (etag or length is not None) and known.get("etag") == etag and known.get("length") == length
        if known.get("sha256") and same_file:
            self._learned[pin.url] = {key: known.get(key) for key in ("etag", "length", "sha256")}
            return known["sha256"]
        if (length if length is not None else pin.length) > self.download_cap:
            return None
        if pin.url not in self._hashed:
            self.downloads += 1
            self._hashed[pin.url] = self._retrying(lambda: self.transport.hash_body(pin.url, self.download_cap))
        outcome = self._hashed[pin.url]
        if outcome is None:
            return None
        self._learned[pin.url] = {"etag": etag, "length": length, "sha256": outcome[1]}
        return outcome[1]

    def _decide(self, pin: Pin, *, length: int | None, sha256: str | None, etag: str | None, problems: list[str]) -> Result:
        if sha256 is None and length == pin.length and not problems:
            return Result(pin, SAME, "size matches; no digest to compare")
        if sha256 is None:
            sha256 = self._hash(pin, length, etag)
        if sha256 == pin.sha256:
            return Result(pin, SAME, "sha256 matches")
        found = list(problems)
        if length is not None and length != pin.length:
            found.append(f"size differs: serves {length:,} bytes, pinned {pin.length:,}")
        if sha256:
            found.append(f"sha256 differs: serves {_short(sha256)}, pinned {_short(pin.sha256)}")
        if found:
            return Result(pin, CHANGED, "; ".join(found))
        return Result(pin, UNVERIFIED, "no size to compare and the file is over the download limit")

    def _github_release(self, pin: Pin) -> Result | None:
        """GitHub release assets: the REST API publishes each asset's size and digest."""
        match = RELEASE_URL.match(pin.url)
        if not match:
            return None
        owner, repo, tag, asset = (urllib.parse.unquote(part) for part in match.groups())
        api = f"{GITHUB_API}/repos/{owner}/{repo}/releases/tags/{urllib.parse.quote(tag, safe='')}"
        response = self._ask("GET", api, API_HEADERS, body=True)
        if response.status == 404:
            return Result(pin, GONE, f"release {tag} not found")
        if response.status in (401, 403, 429):
            return None  # rate limited or not allowed: ask the download link itself instead
        if response.status != 200:
            return Result(pin, UNREACHABLE, f"GitHub API HTTP {response.status}")
        try:
            assets = json.loads(response.body).get("assets") or []
        except (ValueError, AttributeError):
            return None
        found = next((item for item in assets if item.get("name") == asset), None)
        if found is None:
            return Result(pin, GONE, f"{asset} is no longer in release {tag}")
        digest = found.get("digest")
        sha256 = digest[7:].lower() if isinstance(digest, str) and digest.startswith("sha256:") else None
        size = found.get("size")
        result = self._decide(pin, length=size if isinstance(size, int) else None, sha256=sha256, etag=None, problems=[])
        if result.state == SAME and sha256:
            result.detail = "published digest matches"
        return result

    def _served(self, pin: Pin) -> Result:
        """Any other link: size, final host and, where the host names it, the file name."""
        head = self._ask("HEAD", pin.url)
        if head.status in HEAD_REFUSED:
            head = self._ask("GET", pin.url, {"Range": "bytes=0-0"})
        if head.status in (404, 410):
            return Result(pin, GONE, f"HTTP {head.status}")
        if head.status not in (200, 206):
            return Result(pin, UNREACHABLE, f"HTTP {head.status}")
        length = _served_length(head)
        host = urllib.parse.urlsplit(head.url).hostname or ""
        pinned_host = urllib.parse.urlsplit(pin.url).hostname or ""
        if _family(host) != _family(pinned_host) and host not in pin.redirect_hosts:
            size = "" if length is None else f"; serves {length:,} bytes, pinned {pin.length:,}"
            return Result(pin, MOVED, f"now served from {host}, pinned {pinned_host}{size}")
        problems = []
        name = _served_filename(head)
        if pin.kind == "weaselmods-download" and name and name != pin.filename:
            problems.append(f"serves file name {name}, pinned {pin.filename}")
        return self._decide(pin, length=length, sha256=None, etag=head.headers.get("etag"), problems=problems)

    # -- one pin, and what to remember about it ---------------------------------------------

    def check(self, pin: Pin) -> Result:
        try:
            result = self._github_release(pin) if pin.kind == "github-release" else None
            if result is None:
                result = self._served(pin)
        except TransportError as error:
            result = Result(pin, UNREACHABLE, str(error))
        return self._remember(result)

    def _remember(self, result: Result) -> Result:
        url = result.pin.url
        previous = self.state.get(url, {})
        same_as_before = previous.get("state") == result.state
        result.since = previous.get("since", self.today) if same_as_before else self.today
        entry: dict[str, Any] = {key: previous[key] for key in ("etag", "length", "sha256") if key in previous}
        entry.update(self._learned.get(url, {}))
        if result.state == UNREACHABLE:
            result.streak = previous.get("streak", 0) + 1 if same_as_before else 1
            entry["streak"] = result.streak
            if result.streak < 2:  # one bad night can be the server's; say it from the second on
                result.state = PENDING
                result.detail += " (first night, not reported yet)"
        if result.state in (CHANGED, GONE, MOVED, UNREACHABLE, PENDING):
            entry.update(state=UNREACHABLE if result.state == PENDING else result.state, since=result.since)
        if entry:
            self.new_state[url] = entry
        return result


# --------------------------------------------------------------------------------------------
# The report and the issue


def _recipes_text(pin: Pin) -> str:
    return "; ".join(f"{name} ({', '.join(sorted(labels))})" for name, labels in sorted(pin.recipes.items()))


def build_report(results: list[Result], skipped: list[dict[str, str]], notes: list[str], today: str, downloads: int) -> dict[str, Any]:
    counts = {state: 0 for state in (SAME, CHANGED, GONE, MOVED, UNREACHABLE, PENDING, UNVERIFIED)}
    for result in results:
        counts[result.state] += 1
    findings = [
        {
            "mod": "; ".join(sorted(result.pin.mods)),
            "recipes": _recipes_text(result.pin),
            "state": result.state,
            "detail": result.detail,
            "url": result.pin.url,
            "since": result.since,
        }
        for result in results
        if result.state in REPORTED
    ]
    findings.sort(key=lambda item: (REPORTED.index(item["state"]), item["mod"]))
    waiting = [
        {"mod": "; ".join(sorted(r.pin.mods)), "state": r.state, "detail": r.detail, "url": r.pin.url}
        for r in results
        if r.state in (PENDING, UNVERIFIED)
    ]
    return {
        "date": today,
        "checked": len(results),
        "downloads": downloads,
        "counts": counts,
        "skipped": len(skipped),
        "findings": findings,
        "waiting": waiting,
        "notes": notes,
    }


def _cell(text: str) -> str:
    return text.replace("|", "\\|").replace("\n", " ")


def _keys(findings: list[dict[str, str]]) -> list[str]:
    return sorted(f"{item['state']}: {item['url']}" for item in findings)


def counts_line(report: dict[str, Any]) -> str:
    counts = report["counts"]
    parts = [f"{counts[state]} {state}" for state in (SAME, CHANGED, GONE, MOVED, UNREACHABLE) if counts[state] or state == SAME]
    for state, text in ((PENDING, "unreachable for one night so far"), (UNVERIFIED, "not verifiable")):
        if counts[state]:
            parts.append(f"{counts[state]} {text}")
    return f"Checked {report['checked']} pinned download links on {report['date']} (UTC): " + ", ".join(parts) + "."


def render_issue(report: dict[str, Any]) -> str:
    lines = [counts_line(report), ""]
    lines += ["| Mod | Recipe | What changed | Link |", "|---|---|---|---|"]
    for item in report["findings"]:
        what = f"{item['state']}: {item['detail']} (since {item['since']})"
        lines.append(f"| {_cell(item['mod'])} | {_cell(item['recipes'])} | {_cell(what)} | {item['url']} |")
    lines += [
        "",
        "A link that serves a different file makes the installer ask the player (#69, #70). Look at what it serves now: "
        "if it is fine, pin the new file in the recipe; if not, find another source. "
        "This issue closes itself when every link serves the reviewed file again. "
        "How the check works: `docs/mod-link-check.md`.",
        "",
        f"<!-- mod-link-check:{json.dumps({'keys': _keys(report['findings'])})} -->",
    ]
    return "\n".join(lines) + "\n"


def render_summary(report: dict[str, Any]) -> str:
    lines = [f"### {ISSUE_TITLE}", "", counts_line(report), f"Downloads this run: {report['downloads']}.", ""]
    lines += [f"- {item['state']} - {item['mod']}: {item['detail']} - {item['url']}" for item in report["findings"] + report["waiting"]]
    lines += [f"- note: {note}" for note in report["notes"]]
    return "\n".join(lines) + "\n"


GhRunner = Callable[..., str]


def default_gh(args: list[str], input: str | None = None) -> str:
    done = subprocess.run(["gh", *args], input=input, capture_output=True, text=True, encoding="utf-8", check=False)
    if done.returncode:
        raise LinkCheckError(f"gh {' '.join(args[:2])} failed: {done.stderr.strip()}")
    return done.stdout


def sync_issue(report: dict[str, Any], repo: str, gh: GhRunner = default_gh, dry_run: bool = False) -> list[str]:
    """Open, update or close the one issue. Returns what it did, for the log."""
    listed = json.loads(
        gh(["issue", "list", "--repo", repo, "--state", "open", "--search", f"{ISSUE_TITLE} in:title",
            "--json", "number,title,body", "--limit", "30"])
        or "[]"
    )
    existing = next((item for item in listed if item["title"] == ISSUE_TITLE), None)
    findings = report["findings"]
    done: list[str] = []

    def run(args: list[str], text: str | None = None) -> None:
        done.append(" ".join(args[:3]))
        if not dry_run:
            gh(args, text)

    if not findings:
        if existing:
            run(["issue", "comment", str(existing["number"]), "--repo", repo, "--body-file", "-"],
                f"Every pinned link serves the reviewed file again. {counts_line(report)} Closing.")
            run(["issue", "close", str(existing["number"]), "--repo", repo])
        return done
    body = render_issue(report)
    if not existing:
        run(["issue", "create", "--repo", repo, "--title", ISSUE_TITLE, "--body-file", "-"], body)
        return done
    marker = re.search(r"<!-- mod-link-check:(\{.*?\}) -->", existing.get("body") or "")
    before = set(json.loads(marker.group(1)).get("keys", [])) if marker else set()
    now = set(_keys(findings))
    run(["issue", "edit", str(existing["number"]), "--repo", repo, "--body-file", "-"], body)
    if now != before:
        text = [f"The findings changed in the run of {report['date']}."]
        text += [f"- new: {key}" for key in sorted(now - before)]
        text += [f"- no longer reported: {key}" for key in sorted(before - now)]
        run(["issue", "comment", str(existing["number"]), "--repo", repo, "--body-file", "-"], "\n".join(text))
    return done


# --------------------------------------------------------------------------------------------
# Command line


def load_state(path: Path | None) -> dict[str, dict[str, Any]]:
    if path and path.is_file():
        try:
            loaded = json.loads(path.read_text(encoding="utf-8"))
            return loaded.get("links", {}) if isinstance(loaded, dict) else {}
        except ValueError:
            print(f"note: {path} is not valid JSON; starting without memory", file=sys.stderr)
    return {}


def run_check(args: argparse.Namespace, transport: Any | None = None, sleep: Callable[[float], None] = time.sleep) -> int:
    installers = []
    for spec in args.installer:
        if len(spec) < 3:
            raise LinkCheckError("--installer needs NAME DIR RECIPE [RECIPE ...]")
        installers.append(Installer(spec[0], Path(spec[1]), tuple(spec[2:])))
    pins, skipped, notes = load_pins(installers)
    if args.only:
        pins = [pin for pin in pins if any(text in pin.url or any(text in mod for mod in pin.mods) for text in args.only)]
    today = args.today or dt.datetime.now(dt.timezone.utc).date().isoformat()
    token = os.environ.get("GITHUB_TOKEN") or os.environ.get("GH_TOKEN")
    checker = Checker(
        transport or UrllibTransport(token),
        load_state(Path(args.state) if args.state else None),
        today,
        sleep=sleep,
        download_cap=int(args.max_download_mb * 1024 * 1024),
    )
    results = []
    for number, pin in enumerate(pins):
        if number and args.delay:
            sleep(args.delay)
        result = checker.check(pin)
        results.append(result)
        if result.state != SAME:
            print(f"{result.state:11} {'; '.join(sorted(pin.mods))}: {result.detail}\n{'':11} {pin.url}")
    report = build_report(results, skipped, notes, today, checker.downloads)
    print(counts_line(report), f"Downloads: {report['downloads']}. Skipped (manual): {report['skipped']}.")
    for note in notes:
        print(f"note: {note}")
    if args.report:
        Path(args.report).write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    if args.summary:
        with open(args.summary, "a", encoding="utf-8") as handle:
            handle.write(render_summary(report))
    if args.state and not args.only:
        # Only links that are still pinned stay in the memory.
        live = {pin.url for pin in pins}
        links = {url: entry for url, entry in checker.new_state.items() if url in live}
        Path(args.state).write_text(json.dumps({"version": 1, "updated": today, "links": links}, indent=1) + "\n", encoding="utf-8")
    return 0


def run_issue(args: argparse.Namespace, gh: GhRunner = default_gh) -> int:
    repo = args.repo or os.environ.get("GITHUB_REPOSITORY")
    if not repo:
        raise LinkCheckError("--repo or GITHUB_REPOSITORY is required")
    report = json.loads(Path(args.report).read_text(encoding="utf-8"))
    for action in sync_issue(report, repo, gh, args.dry_run) or ["nothing to do"]:
        print(("would run: " if args.dry_run else "ran: ") + action)
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    commands = parser.add_subparsers(dest="command", required=True)
    check = commands.add_parser("check", help="check every pinned link and write a report")
    check.add_argument("--installer", nargs="+", action="append", required=True, metavar=("NAME", "DIR_THEN_RECIPES"),
                       help="an installer: its name, the folder holding its branch's files, then the recipe folders "
                            "it ships (manifest, recipes/...)")
    check.add_argument("--state", help="JSON file with what earlier runs learned; read first, rewritten at the end")
    check.add_argument("--report", help="write the report as JSON")
    check.add_argument("--summary", help="append a Markdown summary (GITHUB_STEP_SUMMARY)")
    check.add_argument("--max-download-mb", type=float, default=DEFAULT_DOWNLOAD_CAP_MB,
                       help="largest file this run may download to hash it (default %(default)s)")
    check.add_argument("--delay", type=float, default=REQUEST_DELAY_SECONDS, help="seconds between links")
    check.add_argument("--only", action="append", help="check only links whose URL or mod name contains this text")
    check.add_argument("--today", help=argparse.SUPPRESS)
    issue = commands.add_parser("issue", help="open, update or close the one issue from a report")
    issue.add_argument("--report", required=True)
    issue.add_argument("--repo", help="OWNER/NAME (default: GITHUB_REPOSITORY)")
    issue.add_argument("--dry-run", action="store_true")
    return parser


def main(argv: list[str] | None = None) -> int:
    for stream in (sys.stdout, sys.stderr):
        if hasattr(stream, "reconfigure"):
            stream.reconfigure(encoding="utf-8", errors="replace")
    args = build_parser().parse_args(argv)
    try:
        return run_check(args) if args.command == "check" else run_issue(args)
    except (LinkCheckError, OSError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
