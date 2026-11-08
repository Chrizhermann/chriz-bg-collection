"""Offline tests for tools/check_mod_links.py.

The fake transport answers with responses recorded from the live links (headers trimmed), so
nothing here touches the network; one test class talks to a server on the loopback interface.
"""

from __future__ import annotations

import contextlib
import hashlib
import io
import json
import tempfile
import threading
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

from tools import check_mod_links as links
from tools.check_mod_links import (
    CHANGED,
    GONE,
    MOVED,
    PENDING,
    SAME,
    UNREACHABLE,
    UNVERIFIED,
    Checker,
    Installer,
    LinkCheckError,
    Pin,
    Response,
    TransportError,
)

SHA = "e116ab16e98f055e398e23b3f1f417f61e37c9d2bf04aefced6c9a3f629ed31d"
OTHER_SHA = "0" * 63 + "1"
RELEASE_URL = "https://github.com/Chrizhermann/bg-eeex-buffbot/releases/download/v1.16.1/buffbot-v1.16.1.zip"
RELEASE_API = "https://api.github.com/repos/Chrizhermann/bg-eeex-buffbot/releases/tags/v1.16.1"
TAG_URL = "https://codeload.github.com/Bubb13/Bubbs-Spell-Menu-Extended/zip/refs/tags/v5.2"
COMMIT_URL = "https://github.com/Chrizhermann/Kitpack/archive/ac718614991e34b4f720807bec5edc96266c6c5e.zip"
COMMIT_FINAL = "https://codeload.github.com/Chrizhermann/Kitpack/zip/ac718614991e34b4f720807bec5edc96266c6c5e"
WEASEL_URL = "https://downloads.weaselmods.net/download/bethany/?wpdmdl=751"
ETAG = '"8b1b8e85479aebb49f753c24be0ffa3f9e29dcfdbac5602998fe12e85af89c7b"'
TODAY = "2026-11-08"


def response(status: int = 200, url: str = "", body: bytes = b"", **headers: str) -> Response:
    return Response(status, url, {name.replace("_", "-"): value for name, value in headers.items()}, body)


def release_json(size: int = 5797598, digest: str | None = f"sha256:{SHA}", name: str = "buffbot-v1.16.1.zip") -> bytes:
    asset = {"name": name, "size": size, "state": "uploaded", "digest": digest}
    return json.dumps({"tag_name": "v1.16.1", "assets": [asset]}).encode()


def make_pin(url: str = RELEASE_URL, kind: str = "github-release", length: int = 5797598, sha: str = SHA,
             filename: str = "buffbot-v1.16.1.zip", hosts: tuple[str, ...] = ("release-assets.githubusercontent.com",)) -> Pin:
    pin = Pin(url, sha, length, kind, filename, hosts)
    pin.mods.add("BuffBot 1.16.1")
    pin.recipes["Chriz Easy BG"] = {"manifest"}
    return pin


class FakeTransport:
    """Scripted answers by (method, url). A list answers one entry per call; the last repeats."""

    def __init__(self, routes: dict, hashes: dict | None = None) -> None:
        self.routes = routes
        self.hashes = hashes or {}
        self.calls: list[tuple[str, str]] = []
        self.downloaded: list[str] = []

    def request(self, method, url, headers=None, *, body=False):
        self.calls.append((method, url))
        answer = self.routes[(method, url)]
        if isinstance(answer, list):
            answer = answer.pop(0) if len(answer) > 1 else answer[0]
        if isinstance(answer, Exception):
            raise answer
        return answer

    def hash_body(self, url, limit):
        self.downloaded.append(url)
        answer = self.hashes[url]
        if isinstance(answer, Exception):
            raise answer
        return answer


def checker(transport, state=None, cap_mb=512):
    return Checker(transport, state or {}, TODAY, sleep=lambda seconds: None, download_cap=cap_mb * 1024 * 1024)


class GitHubReleaseTests(unittest.TestCase):
    def check(self, answer, **pin_options):
        transport = FakeTransport({("GET", RELEASE_API): answer})
        run = checker(transport)
        return run.check(make_pin(**pin_options)), transport, run

    def test_same_by_published_digest(self):
        result, transport, _ = self.check(response(200, RELEASE_API, release_json()))
        self.assertEqual(result.state, SAME)
        self.assertEqual(transport.downloaded, [])
        self.assertEqual([call[0] for call in transport.calls], ["GET"])

    def test_changed_digest(self):
        result, transport, _ = self.check(response(200, RELEASE_API, release_json(digest=f"sha256:{OTHER_SHA}")))
        self.assertEqual(result.state, CHANGED)
        self.assertIn("sha256 differs", result.detail)
        self.assertIn(OTHER_SHA[:12], result.detail)
        self.assertEqual(transport.downloaded, [], "a published digest needs no download")

    def test_changed_size_with_digest_reports_both_without_downloading(self):
        result, transport, _ = self.check(response(200, RELEASE_API, release_json(size=9, digest=f"sha256:{OTHER_SHA}")))
        self.assertEqual(result.state, CHANGED)
        self.assertIn("size differs: serves 9 bytes, pinned 5,797,598", result.detail)
        self.assertEqual(transport.downloaded, [])

    def test_no_digest_and_same_size_is_same_without_download(self):
        result, transport, _ = self.check(response(200, RELEASE_API, release_json(digest=None)))
        self.assertEqual(result.state, SAME)
        self.assertEqual(transport.downloaded, [])

    def test_no_digest_and_changed_size_downloads_once_to_learn_the_hash(self):
        transport = FakeTransport({("GET", RELEASE_API): response(200, RELEASE_API, release_json(size=100, digest=None))},
                                  {RELEASE_URL: (100, OTHER_SHA)})
        first = checker(transport)
        result = first.check(make_pin())
        self.assertEqual(result.state, CHANGED)
        self.assertIn("size differs", result.detail)
        self.assertIn(OTHER_SHA[:12], result.detail)
        self.assertEqual(transport.downloaded, [RELEASE_URL])
        # Next night: the remembered download answers; the file is not fetched again.
        second = checker(transport, first.new_state)
        again = second.check(make_pin())
        self.assertEqual(again.state, CHANGED)
        self.assertIn(OTHER_SHA[:12], again.detail)
        self.assertEqual(transport.downloaded, [RELEASE_URL])
        self.assertEqual(again.since, TODAY)

    def test_missing_release_and_missing_asset_are_gone(self):
        result, _, _ = self.check(response(404, RELEASE_API))
        self.assertEqual(result.state, GONE)
        result, _, _ = self.check(response(200, RELEASE_API, release_json(name="other.zip")))
        self.assertEqual(result.state, GONE)
        self.assertIn("no longer in release", result.detail)

    def test_rate_limited_api_falls_back_to_the_download_link(self):
        transport = FakeTransport({
            ("GET", RELEASE_API): response(403, RELEASE_API),
            ("HEAD", RELEASE_URL): response(200, "https://release-assets.githubusercontent.com/x", content_length="5797598"),
        })
        result = checker(transport).check(make_pin())
        self.assertEqual(result.state, SAME)
        self.assertEqual(result.detail, "size matches; no digest to compare")

    def test_github_release_asset_host_is_not_a_move(self):
        transport = FakeTransport({
            ("GET", RELEASE_API): response(429, RELEASE_API),
            ("HEAD", RELEASE_URL): response(200, "https://objects.githubusercontent.com/y", content_length="5797598"),
        })
        self.assertEqual(checker(transport).check(make_pin(hosts=())).state, SAME)


class ServedLinkTests(unittest.TestCase):
    def test_weasel_mods_same(self):
        transport = FakeTransport({("HEAD", WEASEL_URL): response(
            200, WEASEL_URL, content_type="application/zip", content_length="3759411",
            content_disposition='attachment;filename="Bethany-v1.03.zip"')})
        pin = make_pin(WEASEL_URL, "weaselmods-download", 3759411, SHA, "Bethany-v1.03.zip", ())
        self.assertEqual(checker(transport).check(pin).state, SAME)

    def test_weasel_mods_serves_a_new_file_name(self):
        transport = FakeTransport(
            {("HEAD", WEASEL_URL): response(200, WEASEL_URL, content_length="3759411",
                                            content_disposition='attachment;filename="Bethany-v1.04.zip"')},
            {WEASEL_URL: (3759411, OTHER_SHA)})
        pin = make_pin(WEASEL_URL, "weaselmods-download", 3759411, SHA, "Bethany-v1.03.zip", ())
        result = checker(transport).check(pin)
        self.assertEqual(result.state, CHANGED)
        self.assertIn("serves file name Bethany-v1.04.zip", result.detail)

    def test_file_name_is_not_compared_for_github_archives(self):
        transport = FakeTransport({("HEAD", COMMIT_URL): response(
            200, COMMIT_FINAL, content_length="1234", content_disposition="attachment; filename=Kitpack-ac71861.zip")})
        pin = make_pin(COMMIT_URL, "github-commit-zip", 1234, SHA, "ac718614991e34b4f720807bec5edc96266c6c5e.zip", ("codeload.github.com",))
        self.assertEqual(checker(transport).check(pin).state, SAME)

    def test_changed_size_is_changed(self):
        transport = FakeTransport({("HEAD", WEASEL_URL): response(200, WEASEL_URL, content_length="9999")},
                                  {WEASEL_URL: (9999, OTHER_SHA)})
        pin = make_pin(WEASEL_URL, "weaselmods-download", 3759411, SHA, "Bethany-v1.03.zip", ())
        result = checker(transport).check(pin)
        self.assertEqual(result.state, CHANGED)
        self.assertIn("size differs: serves 9,999 bytes, pinned 3,759,411", result.detail)

    def test_404_and_410_are_gone(self):
        for status in (404, 410):
            transport = FakeTransport({("HEAD", WEASEL_URL): response(status, WEASEL_URL)})
            pin = make_pin(WEASEL_URL, "weaselmods-download", 1, SHA, "x.zip", ())
            result = checker(transport).check(pin)
            self.assertEqual((result.state, result.detail), (GONE, f"HTTP {status}"))

    def test_redirect_to_another_host_is_moved(self):
        transport = FakeTransport({("HEAD", WEASEL_URL): response(200, "https://cdn.example.org/Bethany.zip", content_length="3759411")})
        pin = make_pin(WEASEL_URL, "weaselmods-download", 3759411, SHA, "Bethany-v1.03.zip", ())
        result = checker(transport).check(pin)
        self.assertEqual(result.state, MOVED)
        self.assertIn("cdn.example.org", result.detail)
        self.assertEqual(transport.downloaded, [])

    def test_a_reviewed_redirect_host_is_not_a_move(self):
        transport = FakeTransport({("HEAD", WEASEL_URL): response(200, "https://cdn.example.org/Bethany.zip", content_length="3759411")})
        pin = make_pin(WEASEL_URL, "weaselmods-download", 3759411, SHA, "Bethany-v1.03.zip", ("cdn.example.org",))
        self.assertEqual(checker(transport).check(pin).state, SAME)

    def test_head_refused_uses_a_one_byte_range_request(self):
        transport = FakeTransport({
            ("HEAD", WEASEL_URL): response(405, WEASEL_URL),
            ("GET", WEASEL_URL): response(206, WEASEL_URL, content_length="1", content_range="bytes 0-0/3759411"),
        })
        pin = make_pin(WEASEL_URL, "weaselmods-download", 3759411, SHA, "x.zip", ())
        self.assertEqual(checker(transport).check(pin).state, SAME)

    def test_archive_without_length_is_downloaded_once_then_trusted_by_etag(self):
        pin = make_pin(TAG_URL, "github-tag-archive", 1432756, SHA, "x.zip", ())
        route = {("HEAD", TAG_URL): response(200, TAG_URL, etag=ETAG, content_type="application/zip")}
        transport = FakeTransport(route, {TAG_URL: (1432756, SHA)})
        first = checker(transport)
        self.assertEqual(first.check(pin).state, SAME)
        self.assertEqual(transport.downloaded, [TAG_URL])
        second = checker(transport, first.new_state)
        self.assertEqual(second.check(pin).state, SAME)
        self.assertEqual(transport.downloaded, [TAG_URL], "same ETag: no second download")
        # The tag moved: a new ETag makes the next run hash the file again.
        route[("HEAD", TAG_URL)] = response(200, TAG_URL, etag='"new"')
        transport.hashes[TAG_URL] = (1432000, OTHER_SHA)
        third = checker(transport, second.new_state)
        result = third.check(pin)
        self.assertEqual(result.state, CHANGED)
        self.assertEqual(transport.downloaded, [TAG_URL, TAG_URL])

    def test_file_over_the_download_limit_is_unverified_not_changed(self):
        pin = make_pin(TAG_URL, "github-tag-archive", 5 * 1024 * 1024, SHA, "x.zip", ())
        transport = FakeTransport({("HEAD", TAG_URL): response(200, TAG_URL, etag=ETAG)})
        result = checker(transport, cap_mb=1).check(pin)
        self.assertEqual(result.state, UNVERIFIED)
        self.assertEqual(transport.downloaded, [])


class UnreachableTests(unittest.TestCase):
    def pin(self):
        return make_pin(WEASEL_URL, "weaselmods-download", 3759411, SHA, "x.zip", ())

    def test_one_transient_failure_is_retried(self):
        transport = FakeTransport({("HEAD", WEASEL_URL): [TransportError("timed out"), response(200, WEASEL_URL, content_length="3759411")]})
        waits = []
        run = Checker(transport, {}, TODAY, sleep=waits.append)
        self.assertEqual(run.check(self.pin()).state, SAME)
        self.assertEqual(waits, [links.RETRY_WAIT_SECONDS])
        self.assertEqual(len(transport.calls), 2)

    def test_5xx_after_one_retry_is_unreachable_and_reported_from_the_second_night(self):
        transport = FakeTransport({("HEAD", WEASEL_URL): response(503, WEASEL_URL)})
        first = checker(transport)
        night_one = first.check(self.pin())
        self.assertEqual(night_one.state, PENDING)
        self.assertEqual(len(transport.calls), 2, "asked once more after the 503")
        self.assertEqual(first.new_state[WEASEL_URL]["streak"], 1)
        second = checker(transport, first.new_state)
        night_two = second.check(self.pin())
        self.assertEqual(night_two.state, UNREACHABLE)
        self.assertEqual(night_two.since, TODAY)
        self.assertIn("HTTP 503", night_two.detail)

    def test_a_timeout_that_stays_is_unreachable(self):
        transport = FakeTransport({("HEAD", WEASEL_URL): TransportError("timed out")})
        first = checker(transport)
        self.assertEqual(first.check(self.pin()).state, PENDING)
        self.assertEqual(checker(transport, first.new_state).check(self.pin()).state, UNREACHABLE)

    def test_a_good_night_in_between_resets_the_streak(self):
        bad = FakeTransport({("HEAD", WEASEL_URL): response(503, WEASEL_URL)})
        first = checker(bad)
        first.check(self.pin())
        good = FakeTransport({("HEAD", WEASEL_URL): response(200, WEASEL_URL, content_length="3759411")})
        middle = checker(good, first.new_state)
        self.assertEqual(middle.check(self.pin()).state, SAME)
        self.assertNotIn(WEASEL_URL, middle.new_state)
        self.assertEqual(checker(bad, middle.new_state).check(self.pin()).state, PENDING)

    def test_a_forbidden_answer_is_unreachable_never_gone(self):
        transport = FakeTransport({("HEAD", WEASEL_URL): response(403, WEASEL_URL), ("GET", WEASEL_URL): response(403, WEASEL_URL)})
        self.assertEqual(checker(transport).check(self.pin()).state, PENDING)


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def artifact_toml(name: str, version: str, url: str, length: int, sha: str = SHA, kind: str = "github-release",
                  acquisition: str = "fetch-only") -> str:
    return (f'id = "{name}-{version}"\nname = "{name}"\nversion = "{version}"\nacquisition = "{acquisition}"\n\n[source]\n'
            f'kind = "{kind}"\nurl = "{url}"\nreference = "{version}"\nexpected_filename = "{name}.zip"\n'
            f'expected_length = {length}\nsha256 = "{sha}"\nredirect_hosts = []\n')


def build_tree(root: Path) -> list[Installer]:
    """Two recipes that share one download, an unused superseded pin, a manual download."""
    for recipe in ("manifest", "recipes/curated-full-current"):
        base = root / "bg" / recipe
        write(base / "mods" / "buffbot.toml", 'id = "buffbot"\nartifact_id = "buffbot-1.16.1"\nweidu_artifact_id = "weidu-249"\n')
        write(base / "mods" / "evandra.toml", 'id = "evandra"\nartifact_id = "evandra-2.2"\n')
        write(base / "artifacts" / "buffbot-1.16.1.toml", artifact_toml("BuffBot", "1.16.1", RELEASE_URL, 5797598))
        write(base / "artifacts" / "buffbot-1.12.1.toml", artifact_toml("BuffBot", "1.12.1", RELEASE_URL.replace("1.16.1", "1.12.1"), 1, OTHER_SHA))
        write(base / "artifacts" / "weidu-249.toml", artifact_toml("WeiDU", "249", "https://github.com/weidu/weidu/releases/download/v249.00/WeiDU-Windows-249-amd64.zip", 1000))
        write(base / "artifacts" / "evandra-2.2.toml", artifact_toml("Evandra", "2.2", "https://www.gibberlings3.net/files/file/1008-evandra/?do=download", 2, kind="manual", acquisition="manual-user-supplied"))
    write(root / "bg" / "recipes" / "creator-full-current" / "artifacts" / "old.toml", "")
    write(root / "bg" / "recipes" / "new-recipe" / "artifacts" / "x.toml", "")
    base = root / "iwd" / "recipes" / "iwd-current"
    write(base / "mods" / "buffbot.toml", 'artifact_id = "buffbot-1.16.1"\nweidu_artifact_id = "weidu-249"\n')
    write(base / "artifacts" / "buffbot-1.16.1.toml", artifact_toml("BuffBot", "1.16.1", RELEASE_URL, 5797598))
    write(base / "artifacts" / "weidu-249.toml", artifact_toml("WeiDU", "249", "https://github.com/weidu/weidu/releases/download/v249.00/WeiDU-Windows-249-amd64.zip", 1000))
    return [
        Installer("Chriz Easy BG", root / "bg", ("manifest", "recipes/curated-full-current")),
        Installer("Chriz Easy IWD", root / "iwd", ("recipes/iwd-current",)),
    ]


class LoadPinsTests(unittest.TestCase):
    def test_reads_used_artifacts_once_per_download_and_names_every_recipe(self):
        with tempfile.TemporaryDirectory() as folder:
            pins, skipped, notes = links.load_pins(build_tree(Path(folder)))
        self.assertEqual(sorted(pin.url for pin in pins), sorted([RELEASE_URL, "https://github.com/weidu/weidu/releases/download/v249.00/WeiDU-Windows-249-amd64.zip"]))
        buffbot = next(pin for pin in pins if pin.url == RELEASE_URL)
        self.assertEqual(buffbot.mods, {"BuffBot 1.16.1"})
        self.assertEqual(buffbot.recipes, {"Chriz Easy BG": {"manifest", "curated-full-current"}, "Chriz Easy IWD": {"iwd-current"}})
        self.assertEqual([item["mod"] for item in skipped], ["Evandra 2.2", "Evandra 2.2"])
        self.assertEqual(notes, ["Chriz Easy BG: recipes/new-recipe has artifacts but is not checked"])

    def test_an_empty_checkout_is_an_error_not_a_clean_bill(self):
        with tempfile.TemporaryDirectory() as folder:
            (Path(folder) / "recipes" / "iwd-current" / "mods").mkdir(parents=True)
            with self.assertRaises(LinkCheckError):
                links.load_pins([Installer("Chriz Easy IWD", Path(folder), ("recipes/iwd-current",))])

    def test_a_pin_without_a_digest_is_an_error(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            write(root / "r" / "mods" / "a.toml", 'artifact_id = "a-1"\n')
            write(root / "r" / "artifacts" / "a-1.toml", 'name = "A"\nacquisition = "fetch-only"\n[source]\nkind = "github-release"\nurl = "https://x/y"\n')
            with self.assertRaises(LinkCheckError):
                links.load_pins([Installer("I", root, ("r",))])


class RunTests(unittest.TestCase):
    """The whole command over a temporary tree, with the fake transport."""

    def run_night(self, transport, root, state, report, today):
        with contextlib.redirect_stdout(io.StringIO()):
            return self._run_night(transport, root, state, report, today)

    def _run_night(self, transport, root, state, report, today):
        args = links.build_parser().parse_args([
            "check", "--installer", "Chriz Easy BG", str(root / "bg"), "manifest", "recipes/curated-full-current",
            "--installer", "Chriz Easy IWD", str(root / "iwd"), "recipes/iwd-current",
            "--state", str(state), "--report", str(report), "--today", today, "--delay", "0"])
        return links.run_check(args, transport, sleep=lambda seconds: None)

    def test_two_nights(self):
        weidu_url = "https://github.com/weidu/weidu/releases/download/v249.00/WeiDU-Windows-249-amd64.zip"
        weidu_api = "https://api.github.com/repos/weidu/weidu/releases/tags/v249.00"
        weidu_digest = f"sha256:{SHA}"
        weidu_asset = json.dumps({"assets": [{"name": "WeiDU-Windows-249-amd64.zip", "size": 1000, "digest": weidu_digest}]}).encode()
        routes = {
            ("GET", RELEASE_API): response(200, RELEASE_API, release_json()),
            ("GET", weidu_api): response(200, weidu_api, weidu_asset),
        }
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            build_tree(root)
            state, report = root / "state.json", root / "report.json"
            self.assertEqual(self.run_night(FakeTransport(routes), root, state, report, "2026-11-02"), 0)
            first = json.loads(report.read_text(encoding="utf-8"))
            self.assertEqual(first["checked"], 2)
            self.assertEqual(first["counts"]["same"], 2)
            self.assertEqual(first["findings"], [])
            self.assertEqual(first["skipped"], 2)
            self.assertEqual(json.loads(state.read_text(encoding="utf-8"))["links"], {}, "nothing to remember when all is same")
            # Night two: WeiDU's release is gone and BuffBot's digest changed.
            routes[("GET", weidu_api)] = response(404, weidu_api)
            routes[("GET", RELEASE_API)] = response(200, RELEASE_API, release_json(digest=f"sha256:{OTHER_SHA}"))
            self.run_night(FakeTransport(routes), root, state, report, "2026-11-03")
            second = json.loads(report.read_text(encoding="utf-8"))
            self.assertEqual([item["state"] for item in second["findings"]], [GONE, CHANGED])
            gone = second["findings"][0]
            self.assertEqual(gone["mod"], "WeiDU 249")
            self.assertEqual(gone["url"], weidu_url)
            self.assertIn("Chriz Easy IWD (iwd-current)", gone["recipes"])
            self.assertEqual(gone["since"], "2026-11-03")
            # Night three: the same findings keep their first date.
            self.run_night(FakeTransport(routes), root, state, report, "2026-11-04")
            third = json.loads(report.read_text(encoding="utf-8"))
            self.assertEqual({item["since"] for item in third["findings"]}, {"2026-11-03"})

    def test_only_limits_the_run_and_leaves_the_memory_alone(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            build_tree(root)
            state = root / "state.json"
            state.write_text('{"version": 1, "links": {"kept": {"state": "gone"}}}', encoding="utf-8")
            args = links.build_parser().parse_args([
                "check", "--installer", "Chriz Easy BG", str(root / "bg"), "manifest", "--state", str(state), "--only", "BuffBot", "--delay", "0"])
            transport = FakeTransport({("GET", RELEASE_API): response(200, RELEASE_API, release_json())})
            with contextlib.redirect_stdout(io.StringIO()):
                links.run_check(args, transport, sleep=lambda seconds: None)
            self.assertEqual(len(transport.calls), 1)
            self.assertIn("kept", json.loads(state.read_text(encoding="utf-8"))["links"])


class IssueTests(unittest.TestCase):
    def report(self, *findings):
        counts = {state: 0 for state in (SAME, CHANGED, GONE, MOVED, UNREACHABLE, PENDING, UNVERIFIED)}
        counts[SAME] = 5
        for item in findings:
            counts[item["state"]] += 1
        return {"date": TODAY, "checked": 5 + len(findings), "downloads": 0, "counts": counts, "skipped": 0,
                "findings": list(findings), "waiting": [], "notes": []}

    def finding(self, state="changed", url="https://example.org/a.zip", mod="Mod A | 1.0"):
        return {"mod": mod, "recipes": "Chriz Easy BG (manifest)", "state": state, "detail": "size differs", "url": url, "since": TODAY}

    def gh(self, listed):
        calls = []

        def run(args, input=None):
            calls.append((args, input))
            return json.dumps(listed) if args[:2] == ["issue", "list"] else ""

        return run, calls

    def test_findings_open_one_issue_with_a_table(self):
        run, calls = self.gh([])
        links.sync_issue(self.report(self.finding()), "o/r", run)
        self.assertEqual([args[1] for args, _ in calls], ["list", "create"])
        create, body = calls[1]
        self.assertIn("Mod link check", create)
        self.assertIn("| Mod | Recipe | What changed | Link |", body)
        self.assertIn("Mod A \\| 1.0", body, "pipes are escaped")
        self.assertIn("https://example.org/a.zip", body)

    def test_an_existing_issue_is_edited_and_new_findings_are_commented(self):
        old = links.render_issue(self.report(self.finding()))
        run, calls = self.gh([{"number": 7, "title": "Mod link check", "body": old}, {"number": 8, "title": "Mod link check notes", "body": ""}])
        links.sync_issue(self.report(self.finding(), self.finding("gone", "https://example.org/b.zip", "Mod B")), "o/r", run)
        self.assertEqual([args[1] for args, _ in calls], ["list", "edit", "comment"])
        self.assertEqual(calls[1][0][2], "7")
        self.assertIn("new: gone: https://example.org/b.zip", calls[2][1])

    def test_an_unchanged_set_edits_quietly(self):
        report = self.report(self.finding())
        run, calls = self.gh([{"number": 7, "title": "Mod link check", "body": links.render_issue(report)}])
        links.sync_issue(report, "o/r", run)
        self.assertEqual([args[1] for args, _ in calls], ["list", "edit"])

    def test_all_same_closes_the_open_issue_with_a_comment(self):
        run, calls = self.gh([{"number": 7, "title": "Mod link check", "body": "x"}])
        links.sync_issue(self.report(), "o/r", run)
        self.assertEqual([args[1] for args, _ in calls], ["list", "comment", "close"])
        self.assertIn("serves the reviewed file again", calls[1][1])

    def test_all_same_without_an_issue_does_nothing(self):
        run, calls = self.gh([])
        self.assertEqual(links.sync_issue(self.report(), "o/r", run), [])
        self.assertEqual(len(calls), 1)

    def test_a_missing_report_is_an_error_exit_not_a_clean_bill(self):
        with contextlib.redirect_stderr(io.StringIO()) as errors:
            code = links.main(["issue", "--report", "no-such-report.json", "--repo", "o/r", "--dry-run"])
        self.assertEqual(code, 2)
        self.assertIn("error:", errors.getvalue())

    def test_dry_run_only_lists(self):
        run, calls = self.gh([])
        links.sync_issue(self.report(self.finding()), "o/r", run, dry_run=True)
        self.assertEqual([args[1] for args, _ in calls], ["list"])


class _Handler(BaseHTTPRequestHandler):
    payload = b"x" * 3000

    def log_message(self, *args):
        pass

    def _send(self, send_body: bool) -> None:
        path = self.path
        if path == "/moved":
            self.send_response(302)
            self.send_header("Location", "/file")
            self.end_headers()
        elif path == "/file":
            self.send_response(200)
            self.send_header("Content-Length", str(len(self.payload)))
            self.end_headers()
            if send_body:
                self.wfile.write(self.payload)
        elif path == "/head-refused":
            if self.command == "HEAD":
                self.send_response(405)
                self.end_headers()
            else:
                self.send_response(206)
                self.send_header("Content-Range", f"bytes 0-0/{len(self.payload)}")
                self.send_header("Content-Length", "1")
                self.end_headers()
                self.wfile.write(b"x")
        elif path == "/broken":
            self.send_response(503)
            self.end_headers()
        else:
            self.send_response(404)
            self.end_headers()

    def do_HEAD(self):
        self._send(False)

    def do_GET(self):
        self._send(True)


class UrllibTransportTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.server = ThreadingHTTPServer(("127.0.0.1", 0), _Handler)
        cls.base = f"http://127.0.0.1:{cls.server.server_address[1]}"
        threading.Thread(target=cls.server.serve_forever, daemon=True).start()

    @classmethod
    def tearDownClass(cls):
        cls.server.shutdown()
        cls.server.server_close()

    def setUp(self):
        self.transport = links.UrllibTransport(timeout=5)

    def test_head_follows_a_redirect_and_reports_the_final_url_and_length(self):
        answer = self.transport.request("HEAD", f"{self.base}/moved")
        self.assertEqual((answer.status, answer.url, answer.headers["content-length"]), (200, f"{self.base}/file", "3000"))

    def test_error_statuses_are_answers_not_exceptions(self):
        self.assertEqual(self.transport.request("HEAD", f"{self.base}/nothing").status, 404)
        self.assertEqual(self.transport.request("HEAD", f"{self.base}/broken").status, 503)

    def test_range_request_gives_the_total_length(self):
        answer = self.transport.request("GET", f"{self.base}/head-refused", {"Range": "bytes=0-0"})
        self.assertEqual(links._served_length(answer), 3000)

    def test_hash_body_streams_the_file_and_honours_the_limit(self):
        expected = hashlib.sha256(_Handler.payload).hexdigest()
        self.assertEqual(self.transport.hash_body(f"{self.base}/moved", 10_000), (3000, expected))
        self.assertIsNone(self.transport.hash_body(f"{self.base}/file", 1000))
        with self.assertRaises(links.TransportError):
            self.transport.hash_body(f"{self.base}/nothing", 10_000)

    def test_a_refused_connection_is_a_transport_error(self):
        with self.assertRaises(links.TransportError):
            self.transport.request("HEAD", "http://127.0.0.1:1/")

    def test_checker_over_the_real_transport_sees_the_unchanged_file(self):
        digest = hashlib.sha256(_Handler.payload).hexdigest()
        pin = make_pin(f"{self.base}/moved", "github-tag-archive", 3000, digest, "x.zip", ())
        run = Checker(self.transport, {}, TODAY, sleep=lambda seconds: None)
        self.assertEqual(run.check(pin).state, SAME)
        self.assertEqual(run.downloads, 0)
        longer = make_pin(f"{self.base}/file", "github-tag-archive", 2999, OTHER_SHA, "x.zip", ())
        self.assertEqual(Checker(self.transport, {}, TODAY, sleep=lambda seconds: None).check(longer).state, CHANGED)


if __name__ == "__main__":
    unittest.main()
