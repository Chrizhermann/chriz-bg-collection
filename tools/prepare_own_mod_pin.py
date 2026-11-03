"""Prepare the pin of a new release of one of our own mods in every installer recipe.

This does the mechanical part of the pin rules in `docs/own-mod-pins.md`, so that an agent
or the daily pin job only reviews and completes it:

1. Find the release (default: the mod's newest) and the asset that replaces each pinned
   artifact. A pin whose shape cannot be carried over by name stops here for an agent.
2. Download it fresh with `chriz-bg-author artifact inspect`, and check its length and
   SHA-256 against the digest GitHub publishes for the asset.
3. Write the new artifact TOML from the old one: only the release fields change, the
   archive contract stays. Switch every mod that used the old artifact. The old TOML stays
   as history, or goes in an installer whose registry entry says
   `superseded_artifacts = "remove"`.
4. Chriz Easy BG: regenerate `recipes/curated-full-current` from `manifest/`.
5. Verify each recipe's new artifact with `chriz-bg-author artifact verify` (fresh caches)
   and its components with `tools/check_native_component_order.py`.
6. Compare the components and VERSION of the old and new TP2s, and write the evidence
   record `manifest/evidence/release-intake-<date>-<mod>-<installers>.toml`.

It edits only the installers whose recipes are in the working tree (`--installer` narrows
that). It never commits, pushes or merges, never changes a component choice, and leaves the
tests that freeze a pin, the plan digests, the catalog and the changelog to the agent; the
summary lists them.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import re
import subprocess
import sys
import tempfile
import tomllib
import urllib.parse
from dataclasses import dataclass, field
from pathlib import Path

if __package__ in {None, ""}:
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from tools.own_mod_pins import (  # noqa: E402
    GITHUB_API,
    REGISTRY,
    GitHub,
    GitHubError,
    Installer,
    OwnMod,
    Pin,
    Registry,
    RegistryError,
    SourceError,
    TreeSource,
    collect_pins,
    load_registry,
    newest_release,
)


CURATED = "recipes/curated-full-current"
CURATED_ACCEPTANCE = f"{CURATED}/releases/v0.1.0-alpha.1/acceptance.toml"
ALLOWED_HOSTS = {"github.com", "codeload.github.com"}


class NeedsAgent(RuntimeError):
    """The release cannot be pinned mechanically; an agent has to decide."""


@dataclass(frozen=True)
class Asset:
    name: str
    size: int
    sha256: str | None
    url: str


@dataclass(frozen=True)
class Release:
    tag: str
    commit: str
    published_at: str | None
    assets: dict[str, Asset]
    published_release: bool


@dataclass
class Plan:
    """One pinned artifact and the artifact that replaces it."""

    old_id: str
    old_text: str
    old_data: dict
    pins: list[Pin]
    new_id: str = ""
    new_text: str = ""
    new_url: str = ""
    expected_filename: str = ""
    asset: Asset | None = None
    inspection: dict = field(default_factory=dict)
    old_inspection: dict = field(default_factory=dict)
    archive_stats: dict = field(default_factory=dict)
    verification: dict[str, dict] = field(default_factory=dict)
    native: dict[str, dict] = field(default_factory=dict)
    native_failures: dict[str, str] = field(default_factory=dict)
    removed_in: list[str] = field(default_factory=list)

    @property
    def recipes(self) -> list[str]:
        return sorted({pin.recipe for pin in self.pins})

    @property
    def mod_ids(self) -> list[str]:
        return sorted({pin.mod_id for pin in self.pins})


# --- GitHub ---------------------------------------------------------------------------


def fetch_release(github: GitHub, own_mod: OwnMod, tag: str) -> Release:
    commit = dict(github.tags(own_mod.repository)).get(tag)
    if commit is None:
        raise NeedsAgent(f"{own_mod.repository} has no tag {tag}")
    if own_mod.newest == "newest-tag":
        return Release(tag, commit, None, {}, False)
    data = github.release_by_tag(own_mod.repository, tag)
    if data.get("draft") or data.get("prerelease"):
        raise NeedsAgent(f"{own_mod.repository} {tag} is a draft or prerelease")
    assets: dict[str, Asset] = {}
    for entry in data.get("assets") or []:
        digest = entry.get("digest")
        sha256 = digest.split(":", 1)[1].lower() if isinstance(digest, str) and digest.startswith(
            "sha256:") else None
        assets[entry["name"]] = Asset(entry["name"], int(entry["size"]), sha256,
                                      entry["browser_download_url"])
    return Release(tag, commit, data.get("published_at"), assets, True)


# --- Planning the new artifact ----------------------------------------------------------


def dashed(value: str) -> str:
    return value.replace(".", "-")


def substitute(text: str, pairs: list[tuple[str, str]]) -> str | None:
    """Replace the first form of the old release that the text contains."""
    for old, new in pairs:
        if old and old in text:
            return text.replace(old, new)
    return None


def set_toml_value(text: str, section: str | None, key: str, value: str | int) -> str:
    """Set one `key = value` line of a section; the line must exist exactly once."""
    rendered = json.dumps(value) if isinstance(value, str) else str(value)
    lines = text.split("\n")
    current: str | None = None
    hits = 0
    for index, line in enumerate(lines):
        header = re.match(r"^\s*\[([^\[\]]+)\]\s*$", line)
        if header:
            current = header.group(1).strip()
            continue
        if current == section and re.match(rf"^{re.escape(key)}\s*=", line):
            lines[index] = f"{key} = {rendered}"
            hits += 1
    if hits != 1:
        where = f"[{section}]" if section else "the top level"
        raise NeedsAgent(f"`{key}` appears {hits} times in {where} of the old artifact TOML")
    return "\n".join(lines)


def plan_artifact(
    plan: Plan,
    own_mod: OwnMod,
    release: Release,
    today: str,
    tag_of_commit: dict[str, str],
) -> None:
    source = plan.old_data.get("source") or {}
    kind = source.get("kind")
    old_release = plan.pins[0].release
    old_version = plan.old_data.get("version", "")
    repository_name = own_mod.repository.split("/")[1]
    if kind == "github-commit-zip":
        old_tag = tag_of_commit.get(old_release.lower())
        if old_tag is None:
            raise NeedsAgent(f"{plan.old_id}: its commit {old_release[:9]} carries no tag")
    else:
        old_tag = old_release
    new_tag = release.tag
    if old_version == old_tag:
        new_version = new_tag
    elif old_version == old_tag.removeprefix("v"):
        new_version = new_tag.removeprefix("v")
    else:
        raise NeedsAgent(
            f"{plan.old_id}: its version {old_version} is not its release tag {old_tag}, so the "
            f"new version cannot be read from {new_tag}"
        )
    if old_version not in plan.old_id:
        raise NeedsAgent(f"{plan.old_id}: the artifact id does not contain its version")
    plan.new_id = plan.old_id.replace(old_version, new_version)
    pairs = [(old_tag, new_tag), (dashed(old_tag), dashed(new_tag)),
             (old_version, new_version), (dashed(old_version), dashed(new_version))]

    if kind == "github-release":
        old_asset = urllib.parse.unquote(source["url"].rsplit("/", 1)[1])
        new_asset = substitute(old_asset, pairs)
        if new_asset is None or new_asset not in release.assets:
            names = ", ".join(sorted(release.assets)) or "none"
            raise NeedsAgent(
                f"{plan.old_id}: cannot tell which asset of {new_tag} replaces {old_asset} "
                f"(assets: {names})"
            )
        plan.asset = release.assets[new_asset]
        plan.new_url = (f"https://github.com/{own_mod.repository}/releases/download/"
                        f"{urllib.parse.quote(new_tag)}/{urllib.parse.quote(new_asset)}")
        plan.expected_filename = new_asset
        reference = new_tag
        provenance = f"https://github.com/{own_mod.repository}/releases/tag/{new_tag}"
    elif kind == "github-commit-zip":
        plan.new_url = f"https://codeload.github.com/{own_mod.repository}/zip/{release.commit}"
        plan.expected_filename = f"{repository_name}-{release.commit}.zip"
        reference = release.commit
        provenance = f"https://github.com/{own_mod.repository}/commit/{release.commit}"
    elif kind == "github-tag-archive":
        url = substitute(source["url"], [(old_tag, new_tag)])
        filename = substitute(source.get("expected_filename") or "", pairs)
        if url is None or filename is None:
            raise NeedsAgent(f"{plan.old_id}: its tag archive URL does not carry {old_tag}")
        plan.new_url, plan.expected_filename = url, filename
        reference = new_tag
        provenance = f"https://github.com/{own_mod.repository}/releases/tag/{new_tag}"
    else:
        raise NeedsAgent(f"{plan.old_id}: `{kind}` sources need an agent")

    text = plan.old_text.replace("\r\n", "\n")
    text = set_toml_value(text, None, "id", plan.new_id)
    text = set_toml_value(text, None, "version", new_version)
    text = set_toml_value(text, "source", "url", plan.new_url)
    text = set_toml_value(text, "source", "reference", reference)
    text = set_toml_value(text, "source", "expected_filename", plan.expected_filename)
    # Length and SHA-256 are written after the fresh download.
    text = set_toml_value(text, "provenance", "url", provenance)
    text = set_toml_value(text, "provenance", "reviewed_on", today)
    plan.new_text = text


def finish_artifact_text(plan: Plan, length: int, sha256: str) -> None:
    text = set_toml_value(plan.new_text, "source", "expected_length", length)
    plan.new_text = set_toml_value(text, "source", "sha256", sha256)


# --- Engine and order checks ----------------------------------------------------------


def run_json(command: list[str], cwd: Path) -> dict:
    result = subprocess.run(command, cwd=cwd, capture_output=True, text=True,
                            encoding="utf-8", errors="replace", check=False)
    if result.returncode != 0:
        raise RuntimeError(f"{Path(command[0]).name} {' '.join(command[1:3])} failed: "
                           f"{(result.stderr or result.stdout).strip()}")
    return json.loads(result.stdout)


def archive_stats(inspection: dict) -> dict:
    entries = [entry for entry in inspection["archive"]["entries"] if not entry["is_directory"]]
    total = sum(entry["length"] for entry in entries)
    largest = max((entry["length"] for entry in entries), default=0)
    depth = max((len(entry["path"].strip("/").split("/")) for entry in entries), default=0)
    ratios = [entry["length"] / entry["compressed_length"] for entry in entries
              if entry["compressed_length"]]
    return {
        "entries": inspection["archive"]["entry_count"],
        "uncompressed_bytes": total,
        "largest_entry_bytes": largest,
        "depth": depth,
        "max_compression_ratio": round(max(ratios, default=0.0), 1),
    }


def components(inspection: dict) -> list[tuple[str, int | None, str]]:
    return [(entry["tp2_path"], entry["component"], entry["title"])
            for entry in inspection.get("component_menu_evidence", [])]


def versions(inspection: dict) -> list[tuple[str, str]]:
    return [(entry["tp2_path"], entry["value"])
            for entry in inspection.get("version_evidence", [])]


def component_changes(plan: Plan) -> list[str]:
    # A single-wrapper archive, such as a commit zip, names its top folder after the
    # release; compare its TP2 paths without that folder.
    wrapped = (plan.old_data.get("archive") or {}).get("root_rule") == "single-wrapper"

    def unwrapped(path: str) -> str:
        return path.split("/", 1)[1] if wrapped and "/" in path else path

    old = {(unwrapped(path), number): title for path, number, title in
           components(plan.old_inspection)}
    new = {(unwrapped(path), number): title for path, number, title in
           components(plan.inspection)}
    changes: list[str] = []
    for key in sorted(set(old) | set(new), key=lambda item: (item[0], item[1] or -1)):
        path, number = key
        label = f"{path} #{number}" if number is not None else f"{path} (no literal number)"
        if key not in new:
            changes.append(f"removed {label}: {old[key]}")
        elif key not in old:
            changes.append(f"added {label}: {new[key]}")
        elif old[key] != new[key]:
            changes.append(f"renamed {label}: {old[key]} -> {new[key]}")
    return changes


def previous_native_profiles(root: Path, mod_id: str, artifact_id: str) -> set[str] | None:
    """The recipes whose run order any record of the previous pin verified; None without a
    record. Records per installer add up."""
    found: set[str] | None = None
    for path in sorted((root / "manifest/evidence").glob("*.toml")):
        data = tomllib.loads(path.read_text(encoding="utf-8"))
        for record in data.get("artifacts", []):
            if record.get("artifact_id") != artifact_id or record.get("mod_id") != mod_id:
                continue
            found = found or set()
            if record.get("native_component_order_verified"):
                found |= set(record.get("native_order_recipe_profiles",
                                        record.get("recipe_profiles")) or [])
    return found


# --- Recipes --------------------------------------------------------------------------


def switch_mods(root: Path, recipe: str, old_id: str, new_id: str) -> list[str]:
    switched: list[str] = []
    for path in sorted((root / recipe / "mods").glob("*.toml")):
        text = path.read_text(encoding="utf-8")
        line = f'artifact_id = "{old_id}"'
        if re.search(rf"^{re.escape(line)}\s*$", text, re.MULTILINE):
            path.write_text(re.sub(rf"^{re.escape(line)}(\s*)$", f'artifact_id = "{new_id}"\\1',
                                   text, flags=re.MULTILINE), encoding="utf-8", newline="\n")
            switched.append(f"{recipe}/mods/{path.name}")
    return switched


def curated_commit(root: Path) -> str:
    """The evidence commit the generator writes for its own static test; regenerating with
    it reproduces the checked-in recipe apart from the pin."""
    acceptance = tomllib.loads((root / CURATED_ACCEPTANCE).read_text(encoding="utf-8"))
    commits = {entry.get("commit") for entry in acceptance.get("evidence", [])
               if entry.get("test_artifact") == "tools/tests/test_curated_full_recipe.py"}
    if len(commits) != 1:
        raise NeedsAgent(f"{CURATED_ACCEPTANCE} records {len(commits)} commits for the curated "
                         f"recipe's static test, not one")
    return commits.pop()


def regenerate_curated(root: Path, commit: str) -> None:
    result = subprocess.run(
        [sys.executable, "-I", str(root / "tools/curated_full_recipe.py"), "--root", str(root),
         "--output", str(root / CURATED), "--commit", commit],
        cwd=root, capture_output=True, text=True, check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(f"curated recipe generation failed: {result.stderr.strip()}")


class Snapshot:
    """The files under some folders, so that a preparation that stops can undo its edits."""

    def __init__(self, root: Path, folders: list[str]) -> None:
        self.folders = [root / folder for folder in folders]
        self.files = {path: path.read_bytes() for path in self._files()}

    def _files(self) -> list[Path]:
        return [path for folder in self.folders if folder.is_dir()
                for path in folder.rglob("*") if path.is_file()]

    def restore(self) -> None:
        for path in self._files():
            if path not in self.files:
                path.unlink()
        for path, content in self.files.items():
            if not path.is_file() or path.read_bytes() != content:
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(content)


def generated_recipes(installer: Installer, root: Path) -> dict[str, str]:
    """Recipes another recipe generates, mapped to their source recipe."""
    if "manifest" in installer.recipes and CURATED in installer.recipes and (
        root / "tools/curated_full_recipe.py"
    ).is_file():
        return {CURATED: "manifest"}
    return {}


# --- The preparation ------------------------------------------------------------------


@dataclass
class Outcome:
    mod: OwnMod
    release: Release
    old_releases: list[str]
    plans: list[Plan]
    changed_files: list[str]
    evidence: str
    notes: list[str]
    followups: list[str] = field(default_factory=list)


TEST_FOLDERS = ("engine/tests", "app/src-tauri/tests", "tools/tests")


def followups(root: Path, registry: Registry, own_mod: OwnMod, plans: list[Plan]) -> list[str]:
    """What an agent still does, named by the files in this tree that need it."""
    old_ids = sorted({plan.old_id for plan in plans})
    old_tags = sorted({pin.release for plan in plans for pin in plan.pins})
    edited = {recipe for plan in plans for recipe in plan.recipes}

    def containing(paths: list[Path], needles: list[str]) -> list[str]:
        found = []
        for path in paths:
            text = path.read_text(encoding="utf-8", errors="replace")
            if any(needle in text for needle in needles):
                found.append(path.relative_to(root).as_posix())
        return sorted(found)

    items = ["Read the release notes and the TP2 changes above."]
    # The component catalogs, the changelog draft, the plan digests and the pin list are
    # Chriz Easy BG's.
    bg = bool(edited & {"manifest", CURATED})
    # FOLLOW_UPS.md there is the live to-do list, not a catalog.
    catalogs = containing([path for path in sorted((root / "docs/curation/components")
                                                   .glob("*.md"))
                           if path.name != "FOLLOW_UPS.md"], old_tags)
    if bg and catalogs:
        items.append("Update the component catalog: " + ", ".join(f"`{c}`" for c in catalogs)
                     + ".")
    drafts = sorted((root / "docs/plans").glob("*next-release-changelogs*.md"))
    if bg and drafts:
        draft = drafts[-1].relative_to(root).as_posix()
        items.append(f"Add the changelog draft entries in `{draft}`.")
    tests = containing([path for folder in TEST_FOLDERS
                        for path in sorted((root / folder).glob("*"))
                        if path.suffix in (".rs", ".py")], old_ids)
    if tests:
        items.append("Update the tests that freeze the old pin: "
                     + ", ".join(f"`{test}`" for test in tests) + ".")
    # The newest intake test, by the date of the evidence record it reads.
    intake_tests: dict[str, str] = {}
    for path in sorted((root / "tools/tests").glob("test_*.py")):
        dates = re.findall(r"manifest/evidence/release-intake-(\d{8})",
                           path.read_text(encoding="utf-8", errors="replace"))
        if dates:
            intake_tests[path.relative_to(root).as_posix()] = max(dates)
    if bg and intake_tests:
        example = max(intake_tests, key=lambda name: (intake_tests[name], name))
        items.append("Add an intake test that freezes the new pin and its evidence record, "
                     f"like `{example}`.")
    if bg and (root / "engine/tests/feature_groups.rs").is_file():
        items.append("Re-pin the Recommended plan digests in `engine/tests/feature_groups.rs`.")
    mod_ids = {pin.mod_id for plan in plans for pin in plan.pins}
    for recipe in sorted(edited):
        collection = tomllib.loads((root / recipe / "collection.toml").read_text(encoding="utf-8"))
        runs = {run["run_id"] for run in collection.get("runs", []) if run.get("mod_id") in mod_ids}
        records = containing(sorted((root / recipe / "releases").glob("*/acceptance.toml")),
                             [f'subject_id = "{run}"' for run in sorted(runs)])
        if records:
            items.append(f"Check the static-test records of {', '.join(sorted(runs))} in "
                         + ", ".join(f"`{record}`" for record in records)
                         + f" by this installer's convention (`docs/own-mod-pins.md`).")
    # The pin list names each archive by its download URL.
    old_urls = [(plan.old_data.get("source") or {}).get("url", "") for plan in plans]
    pin_list = root / "docs/pin-list-2.7.md"
    if bg and pin_list.is_file() and any(old and old in pin_list.read_text(encoding="utf-8")
                                         for old in old_ids + old_urls):
        items.append("Regenerate `docs/pin-list-2.7.md`: `CEBG_UPDATE_PIN_LIST=1 cargo test "
                     "--locked -p chriz-bg-engine --test pin_list`.")
    if registry.hold_for(own_mod.id) is not None and (root / REGISTRY).is_file():
        items.append(f"Remove or update {own_mod.name}'s hold in `{REGISTRY}`.")
    items += [
        "Run the full Rust and Python suites and `tools/own_mod_pins.py check`.",
        "Waits for Christopher's test; it never merges by itself.",
    ]
    return items


def author_command(root: Path, explicit: Path | None) -> list[str]:
    if explicit is not None:
        if not explicit.is_file():
            raise SystemExit(f"error: {explicit} does not exist")
        return [str(explicit)]
    for profile in ("release", "debug"):
        for name in ("chriz-bg-author.exe", "chriz-bg-author"):
            candidate = root / "target" / profile / name
            if candidate.is_file():
                return [str(candidate)]
    raise SystemExit("error: build it first: cargo build --locked -p chriz-bg-engine "
                     "--bin chriz-bg-author (or pass --author)")


def prepare(
    root: Path,
    registry: Registry,
    own_mod: OwnMod,
    github: GitHub,
    author: list[str],
    *,
    tag: str | None,
    installers: list[str] | None,
    today: str,
    work: Path,
    tools_root: Path,
) -> Outcome:
    tree = TreeSource(root)
    chosen = [installer for installer in registry.installers
              if (installers is None or installer.id in installers)
              and all(tree.exists(f"{recipe}/collection.toml") for recipe in installer.recipes)]
    if not chosen:
        raise NeedsAgent("no chosen installer has its recipes in this working tree")
    errors: list[str] = []
    pins: list[Pin] = []
    for installer in chosen:
        pins.extend(pin for pin in collect_pins(registry, installer, tree, errors).pins
                    if pin.repository.lower() == own_mod.repository.lower())
    if errors:
        raise NeedsAgent("; ".join(errors))
    if not pins:
        raise NeedsAgent(f"no recipe of {', '.join(i.name for i in chosen)} here pins "
                         f"{own_mod.name}; adding a mod to an installer is Christopher's choice")

    target = tag or newest_release(github, own_mod, True).tag
    release = fetch_release(github, own_mod, target)
    old_releases = sorted({pin.release for pin in pins})
    current = [pin for pin in pins if pin.release in (release.tag, release.commit)]
    if current:
        raise NeedsAgent(f"{own_mod.name} {release.tag} is already pinned in "
                         f"{', '.join(sorted({pin.recipe for pin in current}))}")

    # One plan per old artifact; a generated recipe follows its source recipe.
    generated: dict[str, str] = {}
    for installer in chosen:
        generated.update(generated_recipes(installer, root))
    plans: dict[str, Plan] = {}
    for pin in pins:
        if pin.artifact_id not in plans:
            source_recipe = generated.get(pin.recipe, pin.recipe)
            path = root / source_recipe / "artifacts" / f"{pin.artifact_id}.toml"
            if not path.is_file():
                path = root / pin.recipe / "artifacts" / f"{pin.artifact_id}.toml"
            text = path.read_text(encoding="utf-8")
            plans[pin.artifact_id] = Plan(pin.artifact_id, text, tomllib.loads(text), [])
        plans[pin.artifact_id].pins.append(pin)

    notes: list[str] = []
    tag_of_commit = {commit: name for name, commit in github.tags(own_mod.repository)}
    for plan in plans.values():
        plan_artifact(plan, own_mod, release, today, tag_of_commit)
        for recipe in plan.recipes:
            if (root / recipe / "artifacts" / f"{plan.new_id}.toml").exists():
                raise NeedsAgent(f"{recipe}/artifacts/{plan.new_id}.toml already exists")
        tp2s = (plan.old_data.get("archive") or {}).get("tp2_paths") or []
        expected = [arg for tp2 in tp2s for arg in ("--expected-tp2", tp2)]
        plan.inspection = run_json([*author, "artifact", "inspect", plan.new_url, "--cache-root",
                                    str(work / f"inspect-{plan.new_id}"), *expected], root)
        plan.old_inspection = run_json([*author, "artifact", "inspect",
                                        plan.old_data["source"]["url"], "--cache-root",
                                        str(work / f"inspect-{plan.old_id}"), *expected], root)
        host = (urllib.parse.urlsplit(plan.inspection["final_url"]).hostname or "").lower()
        allowed = ALLOWED_HOSTS | {value.lower() for value in
                                   plan.old_data["source"].get("redirect_hosts", [])}
        if host not in allowed:
            raise NeedsAgent(f"{plan.new_id}: the download ends on {host}, which the old pin "
                             f"does not allow; review redirect_hosts")
        for match in plan.inspection["archive"]["expected_tp2_matches"]:
            if len(match["matches"]) != 1:
                raise NeedsAgent(f"{plan.new_id}: the TP2 {match['expected']} matches "
                                 f"{len(match['matches'])} files")
        if plan.asset is not None:
            if plan.inspection["length"] != plan.asset.size:
                raise NeedsAgent(f"{plan.new_id}: downloaded {plan.inspection['length']} bytes, "
                                 f"GitHub lists {plan.asset.size}")
            if plan.asset.sha256 is None:
                notes.append(f"GitHub publishes no SHA-256 digest for {plan.asset.name}.")
            elif plan.inspection["sha256"].lower() != plan.asset.sha256:
                raise NeedsAgent(f"{plan.new_id}: the download's SHA-256 differs from the "
                                 f"digest GitHub publishes")
        plan.archive_stats = archive_stats(plan.inspection)
        finish_artifact_text(plan, plan.inspection["length"], plan.inspection["sha256"].lower())

    regenerate = any(recipe in generated for plan in plans.values() for recipe in plan.recipes)
    commit = curated_commit(root) if regenerate else ""
    # Named per installer, so that the branches of two installers never write the same file.
    installer_ids = "-".join(sorted({pin.installer for pin in pins}))
    evidence = (f"manifest/evidence/release-intake-{today.replace('-', '')}-{own_mod.id}-"
                f"{installer_ids}.toml")
    if (root / evidence).exists():
        raise NeedsAgent(f"{evidence} already exists")

    # Everything up to here only read. From here on, a stop undoes the edits.
    edited = sorted({recipe for plan in plans.values() for recipe in plan.recipes}
                    | {generated.get(recipe, recipe) for plan in plans.values()
                       for recipe in plan.recipes})
    snapshot = Snapshot(root, [*edited, "manifest/evidence"])
    removing = {recipe for installer in chosen if installer.superseded_artifacts == "remove"
                for recipe in installer.recipes}
    try:
        changed = write_and_verify(root, plans, generated, commit, author, work, tools_root,
                                   removing)
        write_evidence(root, evidence, own_mod, release, plans, today)
    except BaseException:
        snapshot.restore()
        raise
    changed.append(evidence)
    outcome = Outcome(own_mod, release, old_releases, list(plans.values()), changed, evidence,
                      notes)
    outcome.followups = followups(root, registry, own_mod, outcome.plans)
    return outcome


def still_used(root: Path, recipe: str, artifact_id: str) -> bool:
    for path in (root / recipe / "mods").glob("*.toml"):
        data = tomllib.loads(path.read_text(encoding="utf-8"))
        if artifact_id in (data.get("artifact_id"), data.get("weidu_artifact_id")):
            return True
    return False


def write_and_verify(
    root: Path,
    plans: dict[str, Plan],
    generated: dict[str, str],
    commit: str,
    author: list[str],
    work: Path,
    tools_root: Path,
    removing: set[str] = frozenset(),
) -> list[str]:
    """Write and switch the source recipes, regenerate the generated ones, and verify.

    In a recipe of `removing`, the superseded artifact TOML goes once no mod uses it."""
    changed: list[str] = []
    for plan in plans.values():
        for recipe in sorted({generated.get(recipe, recipe) for recipe in plan.recipes}):
            target_path = root / recipe / "artifacts" / f"{plan.new_id}.toml"
            target_path.write_text(plan.new_text, encoding="utf-8", newline="\n")
            changed.append(f"{recipe}/artifacts/{plan.new_id}.toml")
            changed.extend(switch_mods(root, recipe, plan.old_id, plan.new_id))
            old_path = root / recipe / "artifacts" / f"{plan.old_id}.toml"
            if recipe in removing and old_path.is_file() and not still_used(root, recipe,
                                                                            plan.old_id):
                old_path.unlink()
                plan.removed_in.append(recipe)
                changed.append(f"{recipe}/artifacts/{plan.old_id}.toml (removed)")
    if commit:
        regenerate_curated(root, commit)
        changed.append(f"{CURATED}/ (regenerated)")

    for plan in plans.values():
        for recipe in plan.recipes:
            path = root / recipe / "artifacts" / f"{plan.new_id}.toml"
            if not path.is_file():
                raise RuntimeError(f"{recipe} has no {plan.new_id}.toml after the switch")
            report = run_json([*author, "artifact", "verify", str(path), "--cache-root",
                               str(work / f"verify-{recipe.replace('/', '-')}-{plan.new_id}")],
                              root)
            if not report.get("verified"):
                raise NeedsAgent(f"{recipe}: {plan.new_id} did not verify")
            plan.verification[recipe] = report
        for mod_id in plan.mod_ids:
            previous = previous_native_profiles(root, mod_id, plan.old_id)
            for recipe in sorted({pin.recipe for pin in plan.pins if pin.mod_id == mod_id}):
                result = subprocess.run(
                    [sys.executable, "-I",
                     str(tools_root / "tools/check_native_component_order.py"),
                     "--recipe-root", str(root / recipe), "--mod-id", mod_id,
                     "--archive", plan.inspection["quarantined_path"]],
                    cwd=root, capture_output=True, text=True, check=False,
                )
                if result.returncode == 0:
                    plan.native[f"{recipe}:{mod_id}"] = json.loads(result.stdout)[0]
                    continue
                message = (result.stderr or result.stdout).strip()
                plan.native_failures[f"{recipe}:{mod_id}"] = message
                if previous is None or recipe in previous:
                    raise NeedsAgent(f"{mod_id} in {recipe}: {message}")
    return changed


def write_evidence(root: Path, relative: str, own_mod: OwnMod, release: Release,
                   plans: dict[str, Plan], today: str) -> None:
    path = root / relative
    if release.published_release and release.published_at:
        moment = dt.datetime.fromisoformat(release.published_at.replace("Z", "+00:00"))
        published = f"published {moment:%Y-%m-%d %H:%M} UTC, a normal GitHub release"
    else:
        published = "a version tag without a GitHub release"
    recipes = sorted({recipe for plan in plans.values() for recipe in plan.recipes})
    old = ", ".join(sorted({plan.old_id for plan in plans.values()}))
    removed = sorted({recipe for plan in plans.values() for recipe in plan.removed_in})
    kept = [recipe for recipe in recipes if recipe not in removed]
    history = "The superseded evidence records stay as history"
    if kept:
        history += f", and so do the superseded artifact TOMLs in {', '.join(kept)}"
    if removed:
        history += (f"; in {', '.join(removed)} the new artifact TOML replaces the "
                    f"superseded one")
    lines = [
        "schema = 1",
        'evidence_kind = "published-release-download-and-native-order-verification"',
        f'verified_on = "{today}"',
        'generator = "tools/prepare_own_mod_pin.py"',
        ('command = "chriz-bg-author artifact inspect <release asset URL> --cache-root <fresh '
         'cache> --expected-tp2 <TP2 path>; chriz-bg-author artifact verify <artifact.toml> '
         '--cache-root <fresh cache>; python -I tools/check_native_component_order.py '
         '--recipe-root <recipe> --mod-id <mod> --archive <fresh downloaded ZIP>"'),
        "scope = " + json.dumps(
            f"{own_mod.name} {release.tag} ({published}, tag commit {release.commit[:7]}) "
            f"replaces {old} in {', '.join(recipes)}. Each archive was downloaded anonymously "
            f"into a fresh cache for the inspection and for each recipe's artifact TOML, and "
            f"passed the production acquisition and extraction contracts. {history}. This is "
            f"automated source/package verification, not a WeiDU install or gameplay "
            f"acceptance."
        ),
    ]
    for plan in plans.values():
        stats = plan.archive_stats
        for mod_id in plan.mod_ids:
            mod_recipes = sorted({pin.recipe for pin in plan.pins if pin.mod_id == mod_id})
            passed = [recipe for recipe in mod_recipes if f"{recipe}:{mod_id}" in plan.native]
            first = plan.native.get(f"{passed[0]}:{mod_id}") if passed else None
            lines += [
                "",
                f"# {stats['entries']} entries, {stats['uncompressed_bytes']:,} bytes "
                f"uncompressed, largest entry {stats['largest_entry_bytes']:,} bytes, depth "
                f"{stats['depth']}, compression ratio at most {stats['max_compression_ratio']}.",
                "[[artifacts]]",
                f'mod_id = "{mod_id}"',
                f'artifact_id = "{plan.new_id}"',
                f"recipe_profiles = {json.dumps(mod_recipes)}",
            ]
            if passed != mod_recipes:
                lines.append(f"native_order_recipe_profiles = {json.dumps(passed)}")
            lines += [
                f'source_reference = "{release.tag}"',
                f'source_commit = "{release.commit}"',
                f'immutable_source_url = "{plan.new_url}"',
                f"actual_length = {plan.inspection['length']}",
                f'actual_sha256 = "{plan.inspection["sha256"].lower()}"',
            ]
            if first is not None:
                lines.append(f'tp2_sha256 = "{first["tp2_sha256"]}"')
                lines.append(f"native_components = {json.dumps(first['native_components'])}")
            lines += [
                "fresh_download = true",
                "fresh_extraction = true",
                f"published_release_verified = {str(release.published_release).lower()}",
                f"native_component_order_verified = {str(bool(passed)).lower()}",
            ]
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")


# --- Summary --------------------------------------------------------------------------


def summary(outcome: Outcome) -> str:
    own_mod, release = outcome.mod, outcome.release
    lines = [
        f"## {own_mod.name} {release.tag}",
        "",
        f"Prepared by `tools/prepare_own_mod_pin.py`: {own_mod.name} "
        f"{', '.join(outcome.old_releases)} -> {release.tag} (tag commit "
        f"`{release.commit[:7]}`).",
        "",
        "| Artifact | Recipes | Bytes | SHA-256 | Archive |",
        "|---|---|---|---|---|",
    ]
    for plan in outcome.plans:
        stats = plan.archive_stats
        lines.append(
            f"| `{plan.old_id}` -> `{plan.new_id}` | {', '.join(plan.recipes)} | "
            f"{plan.inspection['length']:,} | `{plan.inspection['sha256'].lower()}` | "
            f"{stats['entries']} entries, {stats['uncompressed_bytes']:,} bytes unpacked |"
        )
    lines += ["", "### Checks", ""]
    for plan in outcome.plans:
        digest = ("matches the digest GitHub publishes" if plan.asset and plan.asset.sha256
                  else "GitHub publishes no digest for it")
        lines.append(f"- `{plan.new_id}`: fresh download, length and SHA-256 {digest}; final "
                     f"host `{urllib.parse.urlsplit(plan.inspection['final_url']).hostname}`.")
        for recipe, report in sorted(plan.verification.items()):
            lines.append(f"- `artifact verify` in {recipe}: verified, publish roots "
                         f"{', '.join(report['archive']['publish_roots'])}.")
        for key, result in sorted(plan.native.items()):
            lines.append(f"- Native component order, {key.replace(':', ', ')}: "
                         f"{result['native_components']} fits {', '.join(result['checked_runs'])}.")
        for key, message in sorted(plan.native_failures.items()):
            lines.append(f"- Native component order, {key.replace(':', ', ')}: does not fit, "
                         f"as with the previous pin, whose record leaves this recipe out "
                         f"({message}).")
        changes = component_changes(plan)
        old_versions = sorted({value for _, value in versions(plan.old_inspection)})
        new_versions = sorted({value for _, value in versions(plan.inspection)})
        lines.append(f"- TP2 VERSION: {', '.join(old_versions) or 'none'} -> "
                     f"{', '.join(new_versions) or 'none'}.")
        if changes:
            lines.append("- **Components changed; Christopher decides any new component:**")
            lines.extend(f"  - {change}" for change in changes)
        else:
            lines.append("- Components: the same numbers and names as the previous pin.")
    lines += [f"- {note}" for note in outcome.notes]
    lines += ["", "### Files", ""]
    lines += [f"- `{path}`" for path in outcome.changed_files]
    lines += ["", "### Left for the agent", ""]
    lines += [f"- {item}" for item in outcome.followups]
    return "\n".join(lines) + "\n"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("mod", help="the mod's id in manifest/own-mods.toml")
    parser.add_argument("--release", help="the release tag to pin (default: the newest)")
    parser.add_argument("--root", type=Path, default=Path.cwd(),
                        help="the working tree to change (default: the current directory)")
    parser.add_argument("--registry", type=Path,
                        help=f"registry file (default: <root>/{REGISTRY})")
    parser.add_argument("--installer", action="append",
                        help="only this installer's recipes (repeatable)")
    parser.add_argument("--author", type=Path, help="the chriz-bg-author executable")
    parser.add_argument("--today", default=dt.date.today().isoformat(),
                        help="date for reviewed_on and the evidence record")
    parser.add_argument("--summary", type=Path, help="also write the summary to this file")
    parser.add_argument("--api", default=os.environ.get("CEBG_GITHUB_API", GITHUB_API),
                        help=argparse.SUPPRESS)
    args = parser.parse_args(argv)

    root = args.root.resolve()
    tools_root = Path(__file__).resolve().parents[1]
    try:
        dt.date.fromisoformat(args.today)
        registry = load_registry(args.registry or root / REGISTRY)
    except (RegistryError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2
    own_mod = next((entry for entry in registry.mods if entry.id == args.mod), None)
    if own_mod is None:
        print(f"error: {args.mod} is not a mod in the registry", file=sys.stderr)
        return 2
    author = author_command(tools_root, args.author)
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    with tempfile.TemporaryDirectory(prefix="own-mod-pin-") as work:
        try:
            outcome = prepare(root, registry, own_mod, GitHub(args.api, token), author,
                              tag=args.release, installers=args.installer, today=args.today,
                              work=Path(work), tools_root=tools_root)
        except NeedsAgent as error:
            print(f"needs an agent: {error}", file=sys.stderr)
            return 3
        except (GitHubError, SourceError, RuntimeError, OSError) as error:
            print(f"error: {error}", file=sys.stderr)
            return 1
    text = summary(outcome)
    sys.stdout.write(text)
    if args.summary:
        args.summary.write_text(text, encoding="utf-8", newline="\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
