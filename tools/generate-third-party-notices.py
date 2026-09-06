#!/usr/bin/env python3
"""Generate bundled notices from checksum-verified, already cached locked packages.

Python 3.11+, Cargo and previously populated Cargo/npm caches are required. This
script never downloads, installs, compiles, or extracts package files to disk.
Supplemental upstream texts are checked in with pinned provenance under LICENSES/.
"""

from __future__ import annotations

import argparse
import base64
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys
import tarfile
import tomllib
from urllib.parse import quote


TARGET = "x86_64-pc-windows-msvc"
BEGIN = "<!-- BEGIN GENERATED DEPENDENCY NOTICES -->"
END = "<!-- END GENERATED DEPENDENCY NOTICES -->"
NOTICE_NAME = re.compile(
    r"^(?:licen[cs]e|copying|copyright|notice|authors?)(?:$|[-._])|^acknow",
    re.IGNORECASE,
)
# Vite injects its module-preload helper into the built frontend. Include its
# upstream notice even though npm correctly labels the compiler a dev dependency.
FRONTEND_BUILD_HELPERS = ("node_modules/vite",)


class NoticeError(Exception):
    """A required input is absent, inconsistent, or not reviewed."""


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def text_bytes(data: bytes, label: str) -> str:
    try:
        text = data.decode("utf-8-sig").replace("\r\n", "\n")
    except UnicodeDecodeError as error:
        raise NoticeError(f"Non-UTF-8 notice requires review: {label}") from error
    if not text.strip() or "\x00" in text:
        raise NoticeError(f"Empty or non-text notice: {label}")
    # Preserve all wording while normalizing presentation in the generated Markdown.
    # The original archive/supplement bytes remain separately hash-verified.
    return "\n".join(line.rstrip() for line in text.splitlines()).rstrip("\n") + "\n"


def archive_members(data: bytes, prefix: str) -> dict[str, bytes]:
    """Read ordinary package members in memory; never follow archive links."""
    members: dict[str, bytes] = {}
    with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as archive:
        for member in archive.getmembers():
            path = PurePosixPath(member.name)
            if path.is_absolute() or ".." in path.parts or not path.parts:
                raise NoticeError("Unsafe package member path")
            if path.parts[0] != prefix:
                raise NoticeError(f"Unexpected package root: {path.parts[0]}")
            if not member.isfile():
                continue
            relative = PurePosixPath(*path.parts[1:]).as_posix()
            if relative in members:
                raise NoticeError(f"Duplicate package member: {relative}")
            # Reading only metadata and licensing inputs avoids retaining native
            # libraries or the complete dependency source tree in memory.
            if relative in ("Cargo.toml", ".cargo_vcs_info.json", "package.json") or is_notice(relative):
                if member.size > 2_000_000:
                    raise NoticeError(f"Unexpectedly large notice: {relative}")
                stream = archive.extractfile(member)
                if stream is None:
                    raise NoticeError(f"Cannot read package member: {relative}")
                members[relative] = stream.read()
    return members


def is_notice(path: str) -> bool:
    parts = PurePosixPath(path).parts
    return bool(NOTICE_NAME.search(parts[-1])) or any(
        part.lower() in ("licenses", "licences") for part in parts[:-1]
    )


def cargo_packages(root: Path, cargo: str) -> list[tuple[str, str]]:
    command = [
        cargo, "tree", "--locked", "--offline", "--target", TARGET,
        "-p", "chriz-bg-app", "--edges", "normal,build", "--prefix", "none",
        "--format", "{p}|{l}",
    ]
    result = subprocess.run(command, cwd=root, capture_output=True, text=True, encoding="utf-8")
    if result.returncode:
        raise NoticeError(f"Offline cargo tree failed:\n{result.stderr.strip()}")
    packages = set()
    for line in result.stdout.splitlines():
        if "|" not in line:
            raise NoticeError(f"Unexpected cargo tree output: {line}")
        match = re.match(r"^(\S+) v(\S+)(?:\s|$)", line.split("|", 1)[0])
        if not match:
            raise NoticeError(f"Cannot identify cargo dependency: {line}")
        name, version = match.groups()
        if name not in ("chriz-bg-app", "chriz-bg-engine"):
            packages.add((name, version))
    if not packages:
        raise NoticeError("Cargo returned no dependency packages")
    return sorted(packages)


def cargo_archive(cargo_home: Path, name: str, version: str, checksum: str) -> bytes:
    for candidate in sorted((cargo_home / "registry/cache").glob(f"*/{name}-{version}.crate")):
        data = candidate.read_bytes()
        if sha256(data) == checksum:
            return data
    raise NoticeError(f"Missing checksum-matching Cargo archive: {name} {version}")


def npm_archive(npm_cache: Path, integrity: str, name: str) -> bytes:
    if not integrity.startswith("sha512-") or " " in integrity:
        raise NoticeError(f"Expected one SHA-512 integrity value: {name}")
    try:
        digest = base64.b64decode(integrity[7:], validate=True)
    except ValueError as error:
        raise NoticeError(f"Malformed npm integrity value: {name}") from error
    if len(digest) != 64:
        raise NoticeError(f"Invalid npm SHA-512 length: {name}")
    hexdigest = digest.hex()
    candidate = npm_cache / "_cacache/content-v2/sha512" / hexdigest[:2] / hexdigest[2:4] / hexdigest[4:]
    if not candidate.is_file():
        raise NoticeError(f"Missing cached npm archive: {name}; populate the npm cache separately")
    data = candidate.read_bytes()
    if hashlib.sha512(data).digest() != digest:
        raise NoticeError(f"npm archive integrity mismatch: {name}")
    return data


def collect_notices(
    root: Path, package_id: str, members: dict[str, bytes],
    checksum: str, supplement: dict,
) -> list[tuple[str, str, str | None]]:
    notices = [
        (path, text_bytes(data, f"{package_id}/{path}"), None)
        for path, data in sorted(members.items()) if is_notice(path)
    ]
    if package_id in supplement:
        entry = supplement[package_id]
        if entry["package_checksum"] != checksum:
            raise NoticeError(f"Supplement was reviewed for different package bytes: {package_id}")
        for item in entry["notices"]:
            relative = PurePosixPath(item["file"])
            if relative.is_absolute() or ".." in relative.parts:
                raise NoticeError(f"Unsafe supplemental notice path: {package_id}")
            data = (root / "LICENSES" / str(relative)).read_bytes()
            if sha256(data) != item["sha256"]:
                raise NoticeError(f"Supplemental notice checksum mismatch: {item['file']}")
            notices.append((f"supplement: {item['file']}", text_bytes(data, item["file"]), item["source"]))
    if not notices:
        raise NoticeError(f"No license text for {package_id}; review and pin an upstream supplement")
    return notices


def render(root: Path, cargo_home: Path, npm_cache: Path, cargo: str) -> str:
    cargo_lock = tomllib.loads((root / "Cargo.lock").read_text(encoding="utf-8"))
    locked = {(p["name"], p["version"]): p for p in cargo_lock["package"]}
    npm_lock = json.loads((root / "app/package-lock.json").read_text(encoding="utf-8"))
    supplemental = json.loads((root / "LICENSES/supplemental.json").read_text(encoding="utf-8"))
    if supplemental.get("schema_version") != 1:
        raise NoticeError("Unsupported supplemental notice manifest schema")
    supplement = supplemental["packages"]
    used_supplements = set()
    records = []
    rust_packages = cargo_packages(root, cargo)
    for name, version in rust_packages:
        package = locked[name, version]
        if package.get("source") != "registry+https://github.com/rust-lang/crates.io-index":
            raise NoticeError(f"Unreviewed non-crates.io dependency: {name} {version}")
        checksum = package["checksum"]
        members = archive_members(cargo_archive(cargo_home, name, version, checksum), f"{name}-{version}")
        metadata = tomllib.loads(members["Cargo.toml"].decode("utf-8"))["package"]
        if metadata["name"] != name or metadata["version"] != version:
            raise NoticeError(f"Cargo archive identity mismatch: {name} {version}")
        package_id = f"cargo:{name}@{version}"
        notices = collect_notices(root, package_id, members, checksum, supplement)
        if package_id in supplement:
            used_supplements.add(package_id)
        license_expression = metadata.get("license") or f"See {metadata.get('license-file', 'upstream notices')}"
        version_url = quote(version, safe="")
        records.append((
            package_id, license_expression,
            f"https://crates.io/api/v1/crates/{name}/{version_url}/download",
            f"https://docs.rs/crate/{name}/{version_url}/source/", checksum, notices,
        ))
    npm_count = helper_count = 0
    for install_path, package in sorted(npm_lock["packages"].items()):
        if not install_path or (package.get("dev", False) and install_path not in FRONTEND_BUILD_HELPERS):
            continue
        name = install_path.rsplit("node_modules/", 1)[1]
        version = package["version"]
        data = npm_archive(npm_cache, package["integrity"], f"{name} {version}")
        members = archive_members(data, "package")
        metadata = json.loads(members["package.json"].decode("utf-8"))
        if metadata["name"] != name or metadata["version"] != version:
            raise NoticeError(f"npm archive identity mismatch: {name} {version}")
        package_id = f"npm:{name}@{version}"
        notices = collect_notices(root, package_id, members, sha256(data), supplement)
        if package_id in supplement:
            used_supplements.add(package_id)
        if package.get("dev", False):
            helper_count += 1
        else:
            npm_count += 1
        records.append((
            package_id, metadata.get("license", package.get("license", "See upstream notices")),
            package["resolved"], f"https://www.npmjs.com/package/{name}/v/{version}", sha256(data), notices,
        ))
    unused = set(supplement) - used_supplements
    if unused:
        raise NoticeError(f"Stale supplemental package reviews: {', '.join(sorted(unused))}")

    texts: dict[str, str] = {}
    output = [
        BEGIN, "", "## Locked dependency notices", "",
        "Generated by `python tools/generate-third-party-notices.py`. Do not edit this section by hand.",
        "The package archives were verified against Cargo.lock checksums and npm lockfile integrity.",
        f"Scope: **{len(rust_packages)} Rust normal/build packages** for `{TARGET}`,",
        f"**{npm_count} frontend runtime package(s)** and **{helper_count} frontend build-helper package(s)**.",
        "Build dependencies are included conservatively; this inventory is not a claim that every package is linked.",
        "Vite is included because it emits a module-preload helper into the frontend.",
        "Upstream licenses govern the respective components; CEBG's MIT license does not replace them.",
        "For MPL components, the versioned source/archive links below identify the corresponding unmodified source.",
        "Supplemental files fill upstream package omissions and document embedded components; their exact sources",
        "and checksums are recorded in `LICENSES/supplemental.json`. Original copyright/NOTICE texts follow.", "",
    ]
    for lock_path in ("Cargo.lock", "app/package-lock.json"):
        normalized = (root / lock_path).read_text(encoding="utf-8").encode("utf-8")
        output.append(f"- `{lock_path}` SHA-256 (LF-normalized): `{sha256(normalized)}`")
    output.extend(["", "### Package inventory", ""])
    for package_id, license_expression, archive_url, source_url, checksum, notices in sorted(records):
        output.extend([
            f"#### {package_id}", "", f"Declared license: `{license_expression}`.",
            f"[Exact source archive]({archive_url}) · [Versioned source]({source_url}).",
            f"Archive SHA-256: `{checksum}`.", "",
        ])
        for path, content, source in notices:
            digest = sha256(content.encode("utf-8"))
            texts[digest] = content
            origin = f" ([supplemental source]({source}))" if source else ""
            output.append(f"- `{path}`: [full text](#license-text-{digest}).{origin}")
        output.append("")
    output.extend(["### Full license and notice texts", ""])
    for digest, content in sorted(texts.items()):
        fence = "`" * max(3, 1 + max((len(m.group()) for m in re.finditer(r"`+", content)), default=0))
        output.extend([f"#### License text {digest}", "", fence, content.rstrip("\n"), fence, ""])
    output.extend([END, ""])
    return "\n".join(output)


def replace_generated(current: str, generated: str) -> str:
    if BEGIN not in current and END not in current:
        return current.rstrip("\n") + "\n\n" + generated
    if current.count(BEGIN) != 1 or current.count(END) != 1:
        raise NoticeError("Missing or duplicate generated notice markers")
    before, rest = current.split(BEGIN, 1)
    old, after = rest.split(END, 1)
    if BEGIN in old or after.strip():
        raise NoticeError("Generated notices must be the final section")
    return before + generated


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="verify without writing")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--cargo-home", type=Path, default=Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo")))
    default_npm = Path(os.environ["LOCALAPPDATA"]) / "npm-cache" if os.name == "nt" else Path.home() / ".npm"
    parser.add_argument("--npm-cache", type=Path, default=Path(os.environ.get("npm_config_cache", default_npm)))
    parser.add_argument("--cargo", default="cargo")
    args = parser.parse_args()
    try:
        target = args.root / "THIRD_PARTY_NOTICES.md"
        current = target.read_text(encoding="utf-8")
        updated = replace_generated(current, render(args.root, args.cargo_home, args.npm_cache, args.cargo))
        if args.check:
            if current != updated:
                raise NoticeError("THIRD_PARTY_NOTICES.md is stale; regenerate and review the changes")
            print("Third-party notices match the locked packages and verified cached license texts.")
        else:
            target.write_bytes(updated.encode("utf-8"))
            print(f"Updated THIRD_PARTY_NOTICES.md ({len(updated.encode('utf-8')):,} bytes).")
        return 0
    except (NoticeError, OSError, ValueError, KeyError, tarfile.TarError) as error:
        print(f"Notice generation failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
