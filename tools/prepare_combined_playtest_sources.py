#!/usr/bin/env python3
"""Build six bounded, local-only source archives for the 2026-09-08 playtest.

The archives are assembled from exact Git objects wherever possible.  The two dirty
inputs are deliberately narrow: one hash-pinned Artisan file, and the six reviewed
dragon runtime changes rebased onto the released BG Rebalance v0.3.2 tree.
"""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import math
import os
import subprocess
import tempfile
import zipfile
from pathlib import Path, PurePosixPath
from typing import Iterable


FIXED_ZIP_TIME = (2026, 9, 8, 0, 0, 0)
MODPACK_COMMIT = "85cbc42551ca256b167cfdb1ffc4bd3c22df85e4"
SOD_COMMIT = "3b34eaee19dcb9043f3b72c77bf5b92adbed05ca"
ARTISAN_COMMIT = "136af5422cfa3de14d707c7189088b1ea55d050e"
ARTISAN_OVERLAY_COMMIT = "ac718614991e34b4f720807bec5edc96266c6c5e"
ARTISAN_OVERLAY = "ArtisansKitpack/lib/kit_strref.tpa"
ARTISAN_OVERLAY_SHA256 = "a33ca3c9e6a2bb4ee2555e1ce86e8c6471678997f9f621b5e18a0da9893118d4"
LIGHTNING_COMMIT = "29538896e4d9f2836833f5d925b90f7fe181c69c"
RR_COMPAT_COMMIT = "0f23c21077547a9ae03ec25114f805242166bd1e"
SAFANA_COMMIT = "81d2fa099eafad6b6d1485a743db78b5001e82ad"
SAFANA_ARCHIVE_SHA256 = "29672c878cfb8dbec35d106e4baa4559b1eded55b035e80fdbf831f08f69cad6"
DRAGON_DIRTY_BASE = "d31fda2c6723610c8ac1c6b712306446a9dbfd5b"
DRAGON_RELEASE_COMMIT = "632af10873d54af649e9982724b5c351bd34075e"

DRAGON_MODIFIED = (
    "setup-chriz-bg-rebalance.tp2",
    "chriz-bg-rebalance/languages/english/setup.tra",
)
DRAGON_NEW_HASHES = {
    "chriz-bg-rebalance/lib/dragon_vorpal.tpa": "bf2eeca1664988b7981ddc86340acead362d384f948905463d7b1613a91fa8c1",
    "chriz-bg-rebalance/lib/dragon_vorpal_effects.tpa": "bfc6204b5a4d0bcefd5fc4a74cbff62784bc721d823a9cd598cc5639a56c48df",
    "chriz-bg-rebalance/lib/dragon_wing_buffet.tpa": "32e3e028da4cf51ed47782fc66b1d5c2274da454fdea9b1fc1ead305caf00829",
    "chriz-bg-rebalance/lua/M_CBRDVG.lua": "1673a2f824a9204b9a55bba6e90eac9ee5261043ffc59a639a0d88722edbb41d",
}
DRAGON_DIRTY_HASHES = {
    "setup-chriz-bg-rebalance.tp2": "2e182e3d85fc76507185f8cb63aacac0e7680d9c916dae960803fc5e264f8e30",
    "chriz-bg-rebalance/languages/english/setup.tra": "8bee5beade298bd5f4bb3d1ca73d2390c7cc6c7baf1ea7915a10f8ed2da8c3e6",
    **DRAGON_NEW_HASHES,
}


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run_git(repo: Path, *args: str, input_bytes: bytes | None = None) -> bytes:
    result = subprocess.run(
        ["git", "-C", os.fspath(repo), *args],
        input=input_bytes,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode:
        raise ValueError(
            f"git {' '.join(args)} failed in {repo}: "
            f"{result.stderr.decode('utf-8', errors='replace').strip()}"
        )
    return result.stdout


def require_commit(repo: Path, expected: str, *, clean: bool) -> None:
    actual = run_git(repo, "rev-parse", "HEAD").decode("ascii").strip()
    if actual != expected:
        raise ValueError(f"unexpected HEAD for {repo}: expected {expected}, got {actual}")
    if clean:
        status = run_git(repo, "status", "--porcelain", "--untracked-files=all")
        if status:
            raise ValueError(f"source checkout must be clean: {repo}")


def git_file(repo: Path, commit: str, relative: str) -> bytes:
    return run_git(repo, "show", f"{commit}:{relative}")


def tracked_files(repo: Path, commit: str, prefixes: Iterable[str]) -> list[str]:
    records = run_git(repo, "ls-tree", "-r", "-z", commit).split(b"\0")
    normalized = tuple(prefix.rstrip("/") + "/" for prefix in prefixes)
    result: list[str] = []
    for record in records:
        if not record:
            continue
        metadata, raw_path = record.split(b"\t", 1)
        mode = metadata.split(b" ", 1)[0]
        path = raw_path.decode("utf-8")
        if any(path.startswith(prefix) for prefix in normalized):
            if mode == b"120000":
                raise ValueError(f"runtime allowlist contains a symlink: {path}")
            if mode != b"100644" and mode != b"100755":
                raise ValueError(f"runtime allowlist contains unsupported mode {mode!r}: {path}")
            result.append(path)
    if not result:
        raise ValueError(f"runtime allowlist matched no tracked files at {commit}")
    return sorted(result, key=str.casefold)


def git_payloads(repo: Path, commit: str, paths: Iterable[str]) -> dict[str, bytes]:
    ordered = sorted(set(paths), key=str.casefold)
    queries = b"".join(f"{commit}:{path}\n".encode("utf-8") for path in ordered)
    result = subprocess.run(
        ["git", "-C", os.fspath(repo), "cat-file", "--batch"],
        input=queries,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode:
        raise ValueError(
            f"git cat-file --batch failed in {repo}: "
            + result.stderr.decode("utf-8", errors="replace").strip()
        )
    stream = io.BytesIO(result.stdout)
    payloads: dict[str, bytes] = {}
    for path in ordered:
        header = stream.readline().decode("ascii", errors="replace").strip().split()
        if len(header) != 3 or header[1] != "blob":
            raise ValueError(f"missing or non-file Git object at {commit}:{path}: {header}")
        size = int(header[2])
        payloads[path] = stream.read(size)
        if stream.read(1) != b"\n":
            raise ValueError(f"malformed git cat-file response for {commit}:{path}")
    return payloads


def read_pinned_file(path: Path, expected_sha256: str) -> bytes:
    data = path.read_bytes()
    actual = sha256(data)
    if actual != expected_sha256:
        raise ValueError(
            f"overlay hash mismatch for {path}: expected {expected_sha256}, got {actual}"
        )
    return data


def validate_archive_paths(paths: Iterable[str]) -> None:
    lowered: set[str] = set()
    for raw in paths:
        path = PurePosixPath(raw)
        if not raw or "\\" in raw or path.is_absolute() or ".." in path.parts:
            raise ValueError(f"unsafe archive path: {raw!r}")
        folded = raw.casefold()
        if folded in lowered:
            raise ValueError(f"case-colliding archive path: {raw!r}")
        lowered.add(folded)


def write_deterministic_zip(destination: Path, payloads: dict[str, bytes]) -> dict[str, int | str]:
    validate_archive_paths(payloads)
    destination.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(destination, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for name in sorted(payloads, key=str.casefold):
            info = zipfile.ZipInfo(name, date_time=FIXED_ZIP_TIME)
            info.create_system = 3
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, payloads[name], compresslevel=9)

    with zipfile.ZipFile(destination) as archive:
        infos = archive.infolist()
        observed_ratio = max(
            (info.file_size / max(1, info.compress_size) for info in infos), default=1.0
        )
    data = destination.read_bytes()
    return {
        "sha256": sha256(data),
        "size_bytes": len(data),
        "entry_count": len(payloads),
        "max_entry_bytes": max(map(len, payloads.values()), default=0),
        "max_depth": max((len(PurePosixPath(path).parts) for path in payloads), default=0),
        "uncompressed_bytes": sum(map(len, payloads.values())),
        # This is an extraction limit, not a claim that compression achieved it.
        "max_compression_ratio": max(2, math.ceil(observed_ratio * 1.1)),
    }


def source_record(
    *,
    source_id: str,
    archive: str,
    commit: str,
    source_path: str,
    publish_roots: list[str],
    tp2_paths: list[str],
    root_layout: list[str],
    dirty_overlay: list[dict[str, str]],
    provenance: dict[str, str],
    metrics: dict[str, int | str],
) -> dict[str, object]:
    return {
        "id": source_id,
        "archive": archive,
        "sha256": metrics["sha256"],
        "size_bytes": metrics["size_bytes"],
        "entry_count": metrics["entry_count"],
        "max_entry_bytes": metrics["max_entry_bytes"],
        "max_depth": metrics["max_depth"],
        "uncompressed_bytes": metrics["uncompressed_bytes"],
        "max_compression_ratio": metrics["max_compression_ratio"],
        "root_rule": "direct",
        "publish_roots": publish_roots,
        "tp2_paths": tp2_paths,
        "root_layout": root_layout,
        "commit": commit,
        "source_path": source_path,
        "dirty_overlay": dirty_overlay,
        "provenance": provenance,
    }


def render_sources_json(sources: list[dict[str, object]]) -> str:
    return json.dumps(
        {"schema_version": 1, "sources": sources},
        indent=2,
        ensure_ascii=False,
    ) + "\n"


def top_level_layout(payloads: dict[str, bytes]) -> list[str]:
    roots = sorted({PurePosixPath(path).parts[0] for path in payloads}, key=str.casefold)
    return [root + ("/" if any(path.startswith(root + "/") for path in payloads) else "") for root in roots]


def overlay_record(
    path: str,
    packaged: bytes,
    source: bytes,
    *,
    source_commit: str,
    source_path: Path,
) -> dict[str, str]:
    return {
        "path": path,
        "sha256": sha256(packaged),
        "source_sha256": sha256(source),
        "source_commit": source_commit,
        "source_path": os.fspath(source_path),
    }


def safana_payloads(archive_path: Path) -> dict[str, bytes]:
    archive_bytes = read_pinned_file(archive_path, SAFANA_ARCHIVE_SHA256)
    payloads: dict[str, bytes] = {}
    with zipfile.ZipFile(io.BytesIO(archive_bytes)) as archive:
        for info in archive.infolist():
            name = PurePosixPath(info.filename)
            if info.is_dir():
                continue
            if name.is_absolute() or ".." in name.parts:
                raise ValueError(f"unsafe Safana source member: {info.filename!r}")
            if (info.external_attr >> 16) & 0o170000 == 0o120000:
                raise ValueError(f"Safana source member may not be a symlink: {info.filename!r}")
            if len(name.parts) < 3 or name.parts[0] != "SafanaBG2-05" or name.parts[1] != "Safana":
                continue
            relative = PurePosixPath(*name.parts[1:]).as_posix()
            payloads[relative] = archive.read(info)
    validate_archive_paths(payloads)
    if "Safana/Safana.tp2" not in payloads:
        raise ValueError("official Safana v0.5 archive lacks Safana/Safana.tp2")
    return payloads


def build_dragon_payloads(repo: Path) -> tuple[dict[str, bytes], list[dict[str, str]]]:
    require_commit(repo, DRAGON_DIRTY_BASE, clean=False)
    release_paths = tracked_files(repo, DRAGON_RELEASE_COMMIT, ["chriz-bg-rebalance"])
    release_paths.append("setup-chriz-bg-rebalance.tp2")
    payloads = git_payloads(repo, DRAGON_RELEASE_COMMIT, release_paths)

    dirty_sources = {
        path: read_pinned_file(repo / Path(path), expected)
        for path, expected in DRAGON_DIRTY_HASHES.items()
    }
    patch = run_git(repo, "diff", "--binary", "--", *DRAGON_MODIFIED)
    if not patch:
        raise ValueError("dragon tracked overlay unexpectedly has no diff")

    with tempfile.TemporaryDirectory(prefix="cebg-dragons-") as raw:
        staging = Path(raw)
        for path, data in payloads.items():
            destination = staging / Path(path)
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(data)
        result = subprocess.run(
            ["git", "apply", "--whitespace=nowarn", "-"],
            cwd=staging,
            input=patch,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        if result.returncode:
            raise ValueError(
                "dragon runtime hunks do not apply to released v0.3.2 baseline: "
                + result.stderr.decode("utf-8", errors="replace").strip()
            )
        for path in DRAGON_MODIFIED:
            payloads[path] = (staging / Path(path)).read_bytes()

    for path in DRAGON_NEW_HASHES:
        payloads[path] = dirty_sources[path]
    overlays = [
        overlay_record(
            path,
            payloads[path],
            dirty_sources[path],
            source_commit=DRAGON_DIRTY_BASE,
            source_path=repo,
        )
        for path in sorted(DRAGON_DIRTY_HASHES, key=str.casefold)
    ]
    return payloads, overlays


def build_all(args: argparse.Namespace) -> list[dict[str, object]]:
    modpack = args.modpack_root.resolve()
    sod = args.sod_root.resolve()
    artisan = args.artisan_root.resolve()
    artisan_overlay = args.artisan_overlay_root.resolve()
    spell = args.spell_rev_root.resolve()
    dragons = args.dragon_root.resolve()
    safana_archive = args.safana_archive.resolve()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)

    require_commit(modpack, MODPACK_COMMIT, clean=True)
    require_commit(sod, SOD_COMMIT, clean=True)
    require_commit(artisan, ARTISAN_COMMIT, clean=True)
    require_commit(artisan_overlay, ARTISAN_OVERLAY_COMMIT, clean=False)

    modpack_manifest = json.loads(git_file(modpack, MODPACK_COMMIT, "release-manifest.json"))
    modpack_paths = list(modpack_manifest["files"])
    modpack_payloads = git_payloads(modpack, MODPACK_COMMIT, modpack_paths)

    sod_paths = tracked_files(sod, SOD_COMMIT, ["chriz-sod-remix"])
    sod_paths.extend(["README.md", "THIRD_PARTY_NOTICES.md"])
    sod_payloads = git_payloads(sod, SOD_COMMIT, sod_paths)

    artisan_paths = tracked_files(
        artisan,
        ARTISAN_COMMIT,
        ["ArtisansKitpack", "ArtisansKitpack_npc", "ArtisansKitpack_tweak"],
    )
    artisan_payloads = git_payloads(artisan, ARTISAN_COMMIT, artisan_paths)
    artisan_source = read_pinned_file(artisan_overlay / Path(ARTISAN_OVERLAY), ARTISAN_OVERLAY_SHA256)
    artisan_payloads[ARTISAN_OVERLAY] = artisan_source
    artisan_overlays = [
        overlay_record(
            ARTISAN_OVERLAY,
            artisan_source,
            artisan_source,
            source_commit=ARTISAN_OVERLAY_COMMIT,
            source_path=artisan_overlay,
        )
    ]

    lightning_paths = tracked_files(spell, LIGHTNING_COMMIT, ["spell_rev"])
    lightning_payloads = git_payloads(spell, LIGHTNING_COMMIT, lightning_paths)

    rr_paths = [
        "live-patch/SRCB_RR_COMPAT/README.md",
        "live-patch/SRCB_RR_COMPAT/lib/rr_spell_semantics.tpa",
        "live-patch/SRCB_RR_COMPAT/setup-SRCB_RR_COMPAT.tp2",
    ]
    rr_payloads = {
        path.removeprefix("live-patch/"): git_file(spell, RR_COMPAT_COMMIT, path)
        for path in rr_paths
    }

    dragon_payloads, dragon_overlays = build_dragon_payloads(dragons)
    safana_runtime = safana_payloads(safana_archive)

    configs = [
        ("chriz-bg-modpack", MODPACK_COMMIT, modpack, modpack_payloads,
         ["chriz-bg-modpack", "setup-chriz-bg-modpack.tp2"], ["setup-chriz-bg-modpack.tp2"], [],
         "https://github.com/Chrizhermann/chriz-bg-modpack", "Local combined-playtest snapshot; do not publish or redistribute",
         f"https://github.com/Chrizhermann/chriz-bg-modpack/commit/{MODPACK_COMMIT}"),
        ("chriz-sod-remix", SOD_COMMIT, sod, sod_payloads,
         ["chriz-sod-remix"], ["chriz-sod-remix/setup-chriz-sod-remix.tp2"], [],
         "https://github.com/Chrizhermann/chriz-sod-rebalance", "Local combined-playtest snapshot; do not publish or redistribute",
         f"https://github.com/Chrizhermann/chriz-sod-rebalance/commit/{SOD_COMMIT}"),
        ("artisans-kitpack", ARTISAN_COMMIT, artisan, artisan_payloads,
         ["ArtisansKitpack", "ArtisansKitpack_npc", "ArtisansKitpack_tweak"],
         ["ArtisansKitpack/ArtisansKitpack.TP2", "ArtisansKitpack_npc/ArtisansKitpack_npc.TP2", "ArtisansKitpack_tweak/ArtisansKitpack_tweak.TP2"], artisan_overlays,
         "https://github.com/Chrizhermann/The-Artisan-s-Kitpack-Chriz-Balance-Patch", "Local combined-playtest snapshot of maintained fork; do not publish or redistribute",
         f"https://github.com/Chrizhermann/The-Artisan-s-Kitpack-Chriz-Balance-Patch/commit/{ARTISAN_COMMIT}"),
        ("spell-rev-lightning", LIGHTNING_COMMIT, spell, lightning_payloads,
         ["spell_rev"], ["spell_rev/setup-spell_rev.tp2"], [],
         "https://github.com/Chrizhermann/chriz-spell-revisions-patch", "Local combined-playtest snapshot of Spell Revisions fork; do not publish or redistribute",
         f"https://github.com/Chrizhermann/chriz-spell-revisions-patch/commit/{LIGHTNING_COMMIT}"),
        ("chriz-bg-rebalance", DRAGON_RELEASE_COMMIT, dragons, dragon_payloads,
         ["chriz-bg-rebalance", "setup-chriz-bg-rebalance.tp2"], ["setup-chriz-bg-rebalance.tp2"], dragon_overlays,
         "https://github.com/Chrizhermann/chriz-bg-rebalance", "Local v0.3.2-based dragon test snapshot; do not publish or redistribute",
         "https://github.com/Chrizhermann/chriz-bg-rebalance/releases/tag/v0.3.2"),
        ("srcb-rr-compat", RR_COMPAT_COMMIT, spell, rr_payloads,
         ["SRCB_RR_COMPAT"], ["SRCB_RR_COMPAT/setup-SRCB_RR_COMPAT.tp2"], [],
         "https://github.com/Chrizhermann/chriz-spell-revisions-patch", "Released v4.21-chriz.4 compatibility subpackage; local test copy only",
         "https://github.com/Chrizhermann/chriz-spell-revisions-patch/releases/tag/v4.21-chriz.4"),
        ("safana-in-amn", SAFANA_COMMIT, safana_archive, safana_runtime,
         ["Safana"], ["Safana/Safana.tp2"], [],
         "https://github.com/RoxanneSHS/SafanaBG2", "Upstream Safana in Amn v0.5 runtime; local test copy only",
         "https://github.com/RoxanneSHS/SafanaBG2/tree/v05"),
    ]

    records: list[dict[str, object]] = []
    for source_id, commit, source_path, payloads, roots, tp2s, overlays, homepage, license_text, url in configs:
        archive_name = f"{source_id}-20260908.zip"
        metrics = write_deterministic_zip(output / archive_name, payloads)
        records.append(source_record(
            source_id=source_id,
            archive=archive_name,
            commit=commit,
            source_path=os.fspath(source_path),
            publish_roots=roots,
            tp2_paths=tp2s,
            root_layout=top_level_layout(payloads),
            dirty_overlay=overlays,
            provenance={"homepage": homepage, "license": license_text, "provenance_url": url},
            metrics=metrics,
        ))
    (output / "sources.json").write_text(render_sources_json(records), encoding="utf-8", newline="\n")
    return records


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--modpack-root", type=Path, default=Path(r"C:\Users\chris\.codex\worktrees\modpack-combined-playtest\chriz-bg-modpack"))
    parser.add_argument("--sod-root", type=Path, default=Path(r"C:\src\private\chriz-sod-rebalance\.worktrees\issue-14-bridge-finale"))
    parser.add_argument("--artisan-root", type=Path, default=Path(r"C:\Users\chris\.codex\worktrees\9666\The-Artisan-s-Kitpack-Chriz-Balance-Patch"))
    parser.add_argument("--artisan-overlay-root", type=Path, default=Path(r"C:\Users\chris\.codex\worktrees\5f1b\The-Artisan-s-Kitpack-Chriz-Balance-Patch"))
    parser.add_argument("--spell-rev-root", type=Path, default=Path(r"C:\src\private\chriz-spell-revisions-patch"))
    parser.add_argument("--dragon-root", type=Path, default=Path(r"C:\Users\chris\.codex\worktrees\a947\chriz-bg-rebalance"))
    parser.add_argument("--safana-archive", type=Path, default=Path("target/combined-playtest-20260908/inputs/SafanaBG2-v05.zip"))
    parser.add_argument("--output", type=Path, default=Path("target/combined-playtest-20260908/sources"))
    return parser.parse_args()


if __name__ == "__main__":
    built = build_all(parse_args())
    for record in built:
        print(f"{record['id']}: {record['archive']} {record['sha256']}")
