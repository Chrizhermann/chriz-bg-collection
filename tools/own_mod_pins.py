"""Check that every installer pins the same, newest release of our own mods.

Our own mods are the repositories listed in `manifest/own-mods.toml`: every mod released
from Christopher's GitHub account, forks included (Christopher, 2026-10-28). The rule
agents follow on every pin is in `docs/own-mod-pins.md`. In short:

- Every installer that ships one of our mods pins the same release of it. A split between
  installers needs a recorded hold that names each installer's pin.
- That release is the mod's newest one, unless a recorded hold keeps the newest release
  out (for example because it waits for Christopher's test).
- The registry lists every one of our mods that a recipe pins, and every recipe the app
  bundles belongs to an installer in the registry.

Installer recipes are read from this working tree, or from a git ref while an installer
lives on another branch. The newest releases come from the GitHub API; `--offline` checks
only what needs no network. Being behind the newest release is an error in a release
check (`--release`, or a change of a recipe or app version against `--base`) and for every
mod whose pin changed against `--base`; otherwise it is a note, so that a release of one of
our mods does not turn unrelated pull requests red. The check never downloads an archive
and never edits a file.
"""

from __future__ import annotations

import argparse
import datetime as dt
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


REGISTRY = "manifest/own-mods.toml"
GITHUB_API = "https://api.github.com"
NEWEST_SOURCES = ("latest-release", "newest-tag")
GITHUB_KINDS = ("github-release", "github-tag-archive", "github-commit-zip")
APP_CONFIG_DIR = "app/src-tauri"
APP_PACKAGE = "app/package.json"
INSTALLER_ID = re.compile(r"^[a-z0-9][a-z0-9-]*$")
REPOSITORY = re.compile(r"^[A-Za-z0-9-]+/[A-Za-z0-9._-]+$")
NUMERIC_TAG = re.compile(r"^v?(\d+(?:\.\d+)*)$")


class RegistryError(ValueError):
    """The registry file is missing, malformed, or contradicts itself."""


class SourceError(RuntimeError):
    """A recipe could not be read from the working tree or a git ref."""


class GitHubError(RuntimeError):
    """The GitHub API could not answer a question about a repository."""


# --- Registry -------------------------------------------------------------------------


@dataclass(frozen=True)
class Installer:
    id: str
    name: str
    recipes: tuple[str, ...]
    branch: str | None = None
    # "keep": a new pin leaves the superseded artifact TOML as history; "remove": it goes.
    superseded_artifacts: str = "keep"


@dataclass(frozen=True)
class OwnMod:
    id: str
    name: str
    repository: str
    newest: str = "latest-release"


@dataclass(frozen=True)
class Hold:
    mod: str
    release: str
    reason: str
    decided: str
    installers: tuple[tuple[str, str], ...] | None = None

    def installer_pins(self) -> dict[str, str] | None:
        return None if self.installers is None else dict(self.installers)


@dataclass(frozen=True)
class Registry:
    account: str
    installers: tuple[Installer, ...]
    mods: tuple[OwnMod, ...]
    holds: tuple[Hold, ...]

    def mod_for_repository(self, repository: str) -> OwnMod | None:
        for own_mod in self.mods:
            if own_mod.repository.lower() == repository.lower():
                return own_mod
        return None

    def hold_for(self, mod_id: str) -> Hold | None:
        return next((hold for hold in self.holds if hold.mod == mod_id), None)


def _text(table: dict, key: str, where: str, *, required: bool = True) -> str | None:
    value = table.get(key)
    if value is None and not required:
        return None
    if not isinstance(value, str) or not value.strip():
        raise RegistryError(f"{where}: `{key}` must be a non-empty string")
    return value.strip()


def _relative_recipe_path(value: object, where: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise RegistryError(f"{where}: recipe paths must be non-empty strings")
    path = value.strip()
    parts = path.split("/")
    if "\\" in path or path.startswith("/") or ":" in path or any(
        part in ("", ".", "..") for part in parts
    ):
        raise RegistryError(f"{where}: `{path}` must be a plain relative path with `/`")
    return path


def parse_registry(data: dict) -> Registry:
    """Validate a parsed registry. Every rule here guards a later comparison."""
    if data.get("schema") != 1:
        raise RegistryError("own-mods registry: `schema` must be 1")
    account = _text(data, "account", "own-mods registry")
    known_keys = {"schema", "account", "installers", "mods", "holds"}
    unknown = sorted(set(data) - known_keys)
    if unknown:
        raise RegistryError(f"own-mods registry: unknown keys {unknown}")

    installers: list[Installer] = []
    recipe_owner: dict[str, str] = {}
    for index, table in enumerate(data.get("installers", [])):
        where = f"installers[{index}]"
        if not isinstance(table, dict):
            raise RegistryError(f"{where} must be a table")
        unknown = sorted(set(table) - {"id", "name", "recipes", "branch",
                                       "superseded_artifacts"})
        if unknown:
            raise RegistryError(f"{where}: unknown keys {unknown}")
        superseded = table.get("superseded_artifacts", "keep")
        if superseded not in ("keep", "remove"):
            raise RegistryError(f"{where}: `superseded_artifacts` must be \"keep\" or \"remove\"")
        installer_id = _text(table, "id", where)
        if not INSTALLER_ID.match(installer_id):
            raise RegistryError(f"{where}: id `{installer_id}` must be lowercase words joined by -")
        if any(installer.id == installer_id for installer in installers):
            raise RegistryError(f"{where}: installer id `{installer_id}` is listed twice")
        recipes = table.get("recipes")
        if not isinstance(recipes, list) or not recipes:
            raise RegistryError(f"{where}: `recipes` must list at least one recipe")
        paths = tuple(_relative_recipe_path(value, where) for value in recipes)
        for path in paths:
            if path in recipe_owner:
                raise RegistryError(
                    f"{where}: recipe `{path}` already belongs to `{recipe_owner[path]}`"
                )
            recipe_owner[path] = installer_id
        branch = _text(table, "branch", where, required=False)
        installers.append(Installer(installer_id, _text(table, "name", where), paths, branch,
                                    superseded))
    if not installers:
        raise RegistryError("own-mods registry: list at least one installer")

    mods: list[OwnMod] = []
    for index, table in enumerate(data.get("mods", [])):
        where = f"mods[{index}]"
        if not isinstance(table, dict):
            raise RegistryError(f"{where} must be a table")
        unknown = sorted(set(table) - {"id", "name", "repository", "newest"})
        if unknown:
            raise RegistryError(f"{where}: unknown keys {unknown}")
        mod_id = _text(table, "id", where)
        if not INSTALLER_ID.match(mod_id):
            raise RegistryError(f"{where}: id `{mod_id}` must be lowercase words joined by -")
        repository = _text(table, "repository", where)
        if not REPOSITORY.match(repository):
            raise RegistryError(f"{where}: repository `{repository}` must look like owner/name")
        if repository.split("/")[0].lower() != account.lower():
            raise RegistryError(f"{where}: repository `{repository}` is not on {account}")
        newest = table.get("newest", "latest-release")
        if newest not in NEWEST_SOURCES:
            raise RegistryError(f"{where}: `newest` must be one of {list(NEWEST_SOURCES)}")
        for other in mods:
            if other.id == mod_id:
                raise RegistryError(f"{where}: mod id `{mod_id}` is listed twice")
            if other.repository.lower() == repository.lower():
                raise RegistryError(f"{where}: repository `{repository}` is listed twice")
        mods.append(OwnMod(mod_id, _text(table, "name", where), repository, newest))

    installer_ids = {installer.id for installer in installers}
    mod_ids = {own_mod.id for own_mod in mods}
    holds: list[Hold] = []
    for index, table in enumerate(data.get("holds", [])):
        where = f"holds[{index}]"
        if not isinstance(table, dict):
            raise RegistryError(f"{where} must be a table")
        unknown = sorted(set(table) - {"mod", "release", "reason", "decided", "installers"})
        if unknown:
            raise RegistryError(f"{where}: unknown keys {unknown}")
        mod_id = _text(table, "mod", where)
        if mod_id not in mod_ids:
            raise RegistryError(f"{where}: `{mod_id}` is not one of the listed mods")
        if any(hold.mod == mod_id for hold in holds):
            raise RegistryError(f"{where}: `{mod_id}` already has a hold; keep one per mod")
        decided = _text(table, "decided", where)
        try:
            dt.date.fromisoformat(decided)
        except ValueError:
            raise RegistryError(f"{where}: `decided` must be a date like 2026-10-28") from None
        pins = table.get("installers")
        installer_pins: tuple[tuple[str, str], ...] | None = None
        if pins is not None:
            if not isinstance(pins, dict) or len(pins) < 2:
                raise RegistryError(
                    f"{where}: `installers` records a split, so it names at least two installers"
                )
            for installer_id, release in pins.items():
                if installer_id not in installer_ids:
                    raise RegistryError(f"{where}: `{installer_id}` is not a listed installer")
                if not isinstance(release, str) or not release.strip():
                    raise RegistryError(f"{where}: the pin of `{installer_id}` must be a string")
            installer_pins = tuple(sorted((key, value.strip()) for key, value in pins.items()))
        holds.append(Hold(
            mod_id,
            _text(table, "release", where),
            _text(table, "reason", where),
            decided,
            installer_pins,
        ))
    return Registry(account, tuple(installers), tuple(mods), tuple(holds))


def load_registry(path: Path) -> Registry:
    try:
        data = tomllib.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        raise RegistryError(f"{path} does not exist") from None
    except tomllib.TOMLDecodeError as error:
        raise RegistryError(f"{path}: {error}") from None
    return parse_registry(data)


# --- Reading recipes ------------------------------------------------------------------


class TreeSource:
    """Recipe files in a working tree."""

    def __init__(self, root: Path) -> None:
        self.root = root
        self.label = "this working tree"

    def exists(self, path: str) -> bool:
        return (self.root / path).is_file()

    def toml_files(self, directory: str) -> list[str]:
        folder = self.root / directory
        if not folder.is_dir():
            raise SourceError(f"{directory} does not exist in {self.label}")
        return sorted(
            f"{directory}/{entry.name}"
            for entry in folder.iterdir()
            if entry.is_file() and entry.suffix.lower() == ".toml"
        )

    def read_many(self, paths: list[str]) -> dict[str, str]:
        return {path: (self.root / path).read_text(encoding="utf-8") for path in paths}


class GitRefSource:
    """Recipe files at a git ref, read without touching the working tree."""

    def __init__(self, repository: Path, ref: str) -> None:
        self.repository = repository
        self.ref = ref
        commit = self._git("rev-parse", "--verify", "--quiet", f"{ref}^{{commit}}", check=False)
        if not commit:
            raise SourceError(f"git ref `{ref}` does not exist here; fetch it first")
        self.commit = commit
        self.label = f"{ref} at {commit[:9]}"

    def _git(self, *args: str, check: bool = True) -> str:
        result = subprocess.run(
            ["git", *args],
            cwd=self.repository,
            capture_output=True,
            check=False,
        )
        if check and result.returncode != 0:
            message = result.stderr.decode("utf-8", "replace").strip()
            raise SourceError(f"git {' '.join(args)} failed: {message}")
        return result.stdout.decode("utf-8", "replace").strip() if result.returncode == 0 else ""

    def exists(self, path: str) -> bool:
        result = subprocess.run(
            ["git", "cat-file", "-e", f"{self.commit}:{path}"],
            cwd=self.repository,
            capture_output=True,
            check=False,
        )
        return result.returncode == 0

    def toml_files(self, directory: str) -> list[str]:
        listing = subprocess.run(
            ["git", "ls-tree", "--name-only", "-z", f"{self.commit}:{directory}"],
            cwd=self.repository,
            capture_output=True,
            check=False,
        )
        if listing.returncode != 0:
            raise SourceError(f"{directory} does not exist in {self.label}")
        names = [name for name in listing.stdout.decode("utf-8").split("\0") if name]
        return sorted(f"{directory}/{name}" for name in names if name.lower().endswith(".toml"))

    def read_many(self, paths: list[str]) -> dict[str, str]:
        if not paths:
            return {}
        request = "".join(f"{self.commit}:{path}\n" for path in paths).encode("utf-8")
        result = subprocess.run(
            ["git", "cat-file", "--batch"],
            cwd=self.repository,
            input=request,
            capture_output=True,
            check=False,
        )
        if result.returncode != 0:
            raise SourceError(f"git cat-file failed for {self.label}")
        output = result.stdout
        contents: dict[str, str] = {}
        offset = 0
        for path in paths:
            end = output.index(b"\n", offset)
            header = output[offset:end].decode("utf-8", "replace").split()
            offset = end + 1
            if len(header) != 3 or header[1] != "blob":
                raise SourceError(f"{path} is not a file in {self.label}")
            size = int(header[2])
            contents[path] = output[offset:offset + size].decode("utf-8")
            offset += size + 1
        return contents


# --- Pins -----------------------------------------------------------------------------


@dataclass(frozen=True)
class Pin:
    """One recipe's use of one artifact that comes from one of our repositories."""

    installer: str
    recipe: str
    mod_id: str
    artifact_id: str
    repository: str
    kind: str
    release: str
    url: str
    sha256: str
    length: int | None


@dataclass
class InstallerPins:
    installer: Installer
    source_label: str
    pins: list[Pin] = field(default_factory=list)


def github_origin(url: str) -> tuple[str, list[str]] | None:
    """Return `owner/name` and the remaining path segments of a GitHub download URL."""
    parsed = urllib.parse.urlsplit(url)
    host = (parsed.hostname or "").lower()
    if host not in ("github.com", "codeload.github.com"):
        return None
    segments = [urllib.parse.unquote(part) for part in parsed.path.split("/") if part]
    if len(segments) < 2:
        return None
    return f"{segments[0]}/{segments[1]}", segments[2:]


def release_of(kind: str, rest: list[str], reference: str) -> str | None:
    """The tag or commit an artifact pins; the engine validates that the URL carries it."""
    if kind == "github-release":
        if len(rest) >= 3 and rest[0].lower() == "releases" and rest[1].lower() == "download":
            return rest[2]
        return None
    if kind in ("github-tag-archive", "github-commit-zip"):
        return reference or None
    return None


def collect_pins(
    registry: Registry,
    installer: Installer,
    source: TreeSource | GitRefSource,
    errors: list[str],
) -> InstallerPins:
    """Every artifact of our repositories that an installer's recipes install."""
    result = InstallerPins(installer, source.label)
    for recipe in installer.recipes:
        mod_paths = source.toml_files(f"{recipe}/mods")
        artifact_paths = source.toml_files(f"{recipe}/artifacts")
        files = source.read_many(mod_paths + artifact_paths)
        artifacts: dict[str, tuple[str, dict]] = {}
        for path in artifact_paths:
            data = tomllib.loads(files[path])
            artifact_id = data.get("id")
            if isinstance(artifact_id, str):
                artifacts[artifact_id] = (path, data)
        for path in mod_paths:
            data = tomllib.loads(files[path])
            mod_id = data.get("id", Path(path).stem)
            artifact_id = data.get("artifact_id")
            if not isinstance(artifact_id, str):
                continue
            if artifact_id not in artifacts:
                errors.append(
                    f"{installer.name}: {path} installs `{artifact_id}`, which "
                    f"{recipe}/artifacts does not define ({source.label})"
                )
                continue
            artifact_path, artifact = artifacts[artifact_id]
            artifact_source = artifact.get("source") or {}
            url = artifact_source.get("url", "")
            origin = github_origin(url) if isinstance(url, str) else None
            if origin is None:
                continue
            repository, rest = origin
            if repository.split("/")[0].lower() != registry.account.lower():
                continue
            own_mod = registry.mod_for_repository(repository)
            if own_mod is None:
                errors.append(
                    f"{installer.name}: `{mod_id}` in {recipe} installs `{artifact_id}` from "
                    f"{repository}, which {REGISTRY} does not list. Add the repository there."
                )
                continue
            kind = artifact_source.get("kind", "")
            release = release_of(kind, rest, artifact_source.get("reference", ""))
            if release is None:
                errors.append(
                    f"{installer.name}: {artifact_path} is a `{kind}` download of {repository}; "
                    f"the check knows only {list(GITHUB_KINDS)} with a tag or commit in the URL"
                )
                continue
            result.pins.append(Pin(
                installer.id,
                recipe,
                mod_id,
                artifact_id,
                own_mod.repository,
                kind,
                release,
                url,
                str(artifact_source.get("sha256", "")).lower(),
                artifact_source.get("expected_length"),
            ))
    return result


def pin_identity(pins: list[Pin]) -> dict[str, set[tuple[str, str, str, str, str]]]:
    """Per repository, what a set of pins installs; equal sets mean an untouched pin."""
    identity: dict[str, set[tuple[str, str, str, str, str]]] = {}
    for pin in pins:
        identity.setdefault(pin.repository.lower(), set()).add(
            (pin.recipe, pin.mod_id, pin.artifact_id, pin.url, pin.sha256)
        )
    return identity


# --- GitHub ---------------------------------------------------------------------------


class GitHub:
    """The few read-only GitHub API calls the check needs."""

    def __init__(self, api: str = GITHUB_API, token: str | None = None, attempts: int = 3) -> None:
        self.api = api.rstrip("/")
        self.token = token
        self.attempts = attempts
        self._tags: dict[str, list[tuple[str, str]]] = {}

    def _get(self, path: str) -> object:
        request = urllib.request.Request(f"{self.api}{path}")
        request.add_header("Accept", "application/vnd.github+json")
        request.add_header("X-GitHub-Api-Version", "2022-11-28")
        request.add_header("User-Agent", "chriz-bg-collection-own-mod-pins")
        if self.token:
            request.add_header("Authorization", f"Bearer {self.token}")
        last_error = "no answer"
        for attempt in range(self.attempts):
            try:
                with urllib.request.urlopen(request, timeout=30) as response:
                    return json.loads(response.read().decode("utf-8"))
            except urllib.error.HTTPError as error:
                if error.code == 404:
                    raise GitHubError(f"GitHub has no {path}") from None
                last_error = f"HTTP {error.code}"
                if error.code not in (403, 429, 500, 502, 503, 504):
                    break
            except (urllib.error.URLError, TimeoutError, OSError) as error:
                last_error = str(getattr(error, "reason", error))
            if attempt + 1 < self.attempts:
                time.sleep(2 * (attempt + 1))
        raise GitHubError(f"GitHub API {path}: {last_error}")

    def latest_release(self, repository: str) -> str:
        data = self._get(f"/repos/{repository}/releases/latest")
        if not isinstance(data, dict) or data.get("draft") or data.get("prerelease"):
            raise GitHubError(f"{repository}: the Latest release is not a published release")
        tag = data.get("tag_name")
        if not isinstance(tag, str) or not tag:
            raise GitHubError(f"{repository}: the Latest release has no tag")
        return tag

    def release_by_tag(self, repository: str, tag: str) -> dict:
        data = self._get(f"/repos/{repository}/releases/tags/{urllib.parse.quote(tag)}")
        if not isinstance(data, dict):
            raise GitHubError(f"{repository}: unexpected release {tag}")
        return data

    def tags(self, repository: str) -> list[tuple[str, str]]:
        if repository not in self._tags:
            tags: list[tuple[str, str]] = []
            for page in range(1, 11):
                data = self._get(f"/repos/{repository}/tags?per_page=100&page={page}")
                if not isinstance(data, list):
                    raise GitHubError(f"{repository}: unexpected tag listing")
                for entry in data:
                    name = entry.get("name")
                    commit = (entry.get("commit") or {}).get("sha")
                    if isinstance(name, str) and isinstance(commit, str):
                        tags.append((name, commit.lower()))
                if len(data) < 100:
                    break
            self._tags[repository] = tags
        return self._tags[repository]


def tag_order(name: str) -> tuple[int, ...] | None:
    match = NUMERIC_TAG.match(name)
    return tuple(int(part) for part in match.group(1).split(".")) if match else None


@dataclass(frozen=True)
class Newest:
    tag: str
    commit: str | None = None


def newest_release(github: GitHub, own_mod: OwnMod, need_commit: bool) -> Newest:
    if own_mod.newest == "newest-tag":
        numbered = [(tag_order(name), name, commit)
                    for name, commit in github.tags(own_mod.repository)]
        numbered = [entry for entry in numbered if entry[0] is not None]
        if not numbered:
            raise GitHubError(f"{own_mod.repository} has no version tag like v1.2.3")
        _, name, commit = max(numbered)
        return Newest(name, commit)
    tag = github.latest_release(own_mod.repository)
    commit = None
    if need_commit:
        commit = dict(github.tags(own_mod.repository)).get(tag)
        if commit is None:
            raise GitHubError(f"{own_mod.repository}: the Latest release's tag {tag} has no commit")
    return Newest(tag, commit)


def is_newest(pin: Pin, newest: Newest) -> bool:
    if pin.kind == "github-commit-zip":
        return newest.commit is not None and pin.release.lower() == newest.commit
    return pin.release == newest.tag


def display(pin_release: str, kind: str, tags: list[tuple[str, str]] | None) -> str:
    if kind == "github-commit-zip":
        for name, commit in tags or []:
            if commit == pin_release.lower():
                return f"{name} ({pin_release[:9]})"
        return pin_release[:9]
    return pin_release


# --- The check ------------------------------------------------------------------------


@dataclass
class ModStatus:
    own_mod: OwnMod
    pins: dict[str, str]
    newest: str | None = None
    state: str = ""


@dataclass
class Report:
    errors: list[str] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)
    notes: list[str] = field(default_factory=list)
    statuses: list[ModStatus] = field(default_factory=list)
    sources: list[str] = field(default_factory=list)
    mode: str = ""


def bundled_recipe_roots(root: Path) -> dict[str, list[str]]:
    """Recipe roots each app configuration bundles, by configuration file."""
    bundled: dict[str, list[str]] = {}
    folder = root / APP_CONFIG_DIR
    if not folder.is_dir():
        return bundled
    for config in sorted(folder.glob("tauri*.conf.json")):
        data = json.loads(config.read_text(encoding="utf-8"))
        resources = (data.get("bundle") or {}).get("resources") or {}
        if isinstance(resources, list):
            entries = [(entry, entry) for entry in resources if isinstance(entry, str)]
        else:
            entries = [(key, value) for key, value in resources.items() if value is not None]
        roots: set[str] = set()
        for key, _ in entries:
            path = key.replace("\\", "/")
            while path.startswith("../"):
                path = path[3:]
            parts = [part for part in path.split("/") if part]
            if parts[:1] == ["manifest"]:
                roots.add("manifest")
            elif len(parts) >= 2 and parts[0] == "recipes":
                roots.add(f"recipes/{parts[1]}")
        bundled[f"{APP_CONFIG_DIR}/{config.name}"] = sorted(roots)
    return bundled


def version_of(source: TreeSource | GitRefSource, path: str) -> str | None:
    if not source.exists(path):
        return None
    try:
        data = json.loads(source.read_many([path])[path])
    except (ValueError, SourceError):
        return None
    version = data.get("version") if isinstance(data, dict) else None
    return version if isinstance(version, str) else None


def installer_source(
    root: Path,
    installer: Installer,
    remote: str,
) -> TreeSource | GitRefSource:
    tree = TreeSource(root)
    if all(tree.exists(f"{recipe}/collection.toml") for recipe in installer.recipes):
        return tree
    if installer.branch is None:
        missing = [recipe for recipe in installer.recipes
                   if not tree.exists(f"{recipe}/collection.toml")]
        raise SourceError(f"{installer.name}: {', '.join(missing)} is not in this working tree")
    return GitRefSource(root, f"{remote}/{installer.branch}")


def fetch_branches(root: Path, registry: Registry, remote: str, depth: int | None) -> None:
    """Fetch the branches installers live on; `depth` only for a shallow CI checkout."""
    tree = TreeSource(root)
    for installer in registry.installers:
        if installer.branch and not all(
            tree.exists(f"{recipe}/collection.toml") for recipe in installer.recipes
        ):
            refspec = f"+refs/heads/{installer.branch}:refs/remotes/{remote}/{installer.branch}"
            shallow = [f"--depth={depth}"] if depth else []
            result = subprocess.run(
                ["git", "fetch", "--no-tags", "--quiet", *shallow, remote, refspec],
                cwd=root,
                capture_output=True,
                check=False,
            )
            if result.returncode != 0:
                message = result.stderr.decode("utf-8", "replace").strip()
                raise SourceError(f"could not fetch {installer.branch} from {remote}: {message}")


def check(
    root: Path,
    registry: Registry,
    github: GitHub | None,
    *,
    base: str | None = None,
    release: bool = False,
    remote: str = "origin",
) -> Report:
    report = Report()

    # Every recipe the app bundles belongs to an installer, and every in-tree installer
    # recipe is bundled; otherwise the registry would compare the wrong recipes.
    listed = {recipe for installer in registry.installers for recipe in installer.recipes}
    bundled = bundled_recipe_roots(root)
    for config, roots in bundled.items():
        for recipe in roots:
            if recipe not in listed:
                report.errors.append(
                    f"{config} bundles {recipe}, but no installer in {REGISTRY} lists it"
                )
    all_bundled = {recipe for roots in bundled.values() for recipe in roots}
    tree = TreeSource(root)

    collected: list[InstallerPins] = []
    for installer in registry.installers:
        try:
            source = installer_source(root, installer, remote)
        except SourceError as error:
            report.errors.append(str(error))
            continue
        if isinstance(source, TreeSource):
            for recipe in installer.recipes:
                if bundled and recipe not in all_bundled:
                    report.errors.append(
                        f"{installer.name}: {REGISTRY} lists {recipe}, but no app configuration "
                        f"in {APP_CONFIG_DIR} bundles it"
                    )
        try:
            collected.append(collect_pins(registry, installer, source, report.errors))
        except (SourceError, tomllib.TOMLDecodeError, UnicodeDecodeError) as error:
            report.errors.append(f"{installer.name}: {error}")
            continue
        recipes = ", ".join(installer.recipes)
        report.sources.append(f"{installer.name}: {recipes} from {source.label}")

    # Which of our mods this change pins anew, and whether it prepares a release.
    touched: set[str] = set()
    if base is not None:
        try:
            base_source = GitRefSource(root, base)
        except SourceError as error:
            report.errors.append(f"base: {error}")
            base_source = None
        if base_source is not None:
            for installer_pins in collected:
                installer = installer_pins.installer
                in_tree = (tree.exists(f"{recipe}/collection.toml") for recipe in installer.recipes)
                if not all(in_tree):
                    continue
                base_installer = Installer(
                    installer.id,
                    installer.name,
                    tuple(recipe for recipe in installer.recipes
                          if base_source.exists(f"{recipe}/collection.toml")),
                )
                before: list[Pin] = []
                if base_installer.recipes:
                    try:
                        before = collect_pins(registry, base_installer, base_source, []).pins
                    except (SourceError, tomllib.TOMLDecodeError, UnicodeDecodeError) as error:
                        report.errors.append(f"base {installer.name}: {error}")
                after_identity = pin_identity(installer_pins.pins)
                before_identity = pin_identity(before)
                for repository in set(after_identity) | set(before_identity):
                    if after_identity.get(repository) != before_identity.get(repository):
                        own_mod = registry.mod_for_repository(repository)
                        if own_mod is not None:
                            touched.add(own_mod.id)
                for recipe in installer.recipes:
                    path = f"{recipe}/release.json"
                    if version_of(tree, path) != version_of(base_source, path):
                        release = True
            if version_of(tree, APP_PACKAGE) != version_of(base_source, APP_PACKAGE):
                release = True

    if release:
        report.mode = "release check: every mod must be on its newest release or held"
    elif base is not None:
        names = [own_mod.name for own_mod in registry.mods if own_mod.id in touched]
        report.mode = (
            f"change against {base}: newest release required for "
            f"{', '.join(names) if names else 'no mod (no pin changed)'}"
        )
    else:
        report.mode = "status: being behind the newest release is a note"

    by_repository: dict[str, list[Pin]] = {}
    for installer_pins in collected:
        for pin in installer_pins.pins:
            by_repository.setdefault(pin.repository.lower(), []).append(pin)

    # Same URL, same file: a mismatch would install different bytes for one release.
    by_url: dict[str, set[tuple[str, int | None]]] = {}
    for pins in by_repository.values():
        for pin in pins:
            by_url.setdefault(pin.url, set()).add((pin.sha256, pin.length))
    for url, digests in sorted(by_url.items()):
        if len(digests) > 1:
            report.errors.append(f"{url} is pinned with different SHA-256 or length values")

    installer_names = {installer.id: installer.name for installer in registry.installers}

    def listing(pins: dict[str, str]) -> str:
        return ", ".join(f"{installer_names[key]} {value}" for key, value in sorted(pins.items()))

    for own_mod in registry.mods:
        pins = by_repository.get(own_mod.repository.lower(), [])
        hold = registry.hold_for(own_mod.id)
        status = ModStatus(own_mod, {})
        report.statuses.append(status)
        if not pins:
            status.state = "not in any installer"
            if hold is not None:
                report.errors.append(
                    f"{own_mod.name}: no installer pins it, so its hold for {hold.release} is "
                    f"not needed; remove the hold from {REGISTRY}"
                )
            continue

        releases: dict[str, set[str]] = {}
        kinds: dict[str, str] = {}
        for pin in pins:
            releases.setdefault(pin.installer, set()).add(pin.release)
            kinds[pin.installer] = pin.kind
        for installer_id, values in sorted(releases.items()):
            if len(values) > 1:
                report.errors.append(
                    f"{own_mod.name}: {installer_names[installer_id]} mixes releases "
                    f"{sorted(values)}; one installer pins one release of a mod"
                )
        pinned = {installer_id: min(values) for installer_id, values in releases.items()}
        status.pins = {key: display(value, kinds[key], None) for key, value in pinned.items()}

        # A split between installers is only acceptable when a hold records it exactly.
        hold_pins = hold.installer_pins() if hold else None
        hold_done = hold is not None and all(value == hold.release for value in pinned.values())
        if hold_done:
            report.errors.append(
                f"{own_mod.name}: every installer pins {hold.release} now; remove its hold "
                f"from {REGISTRY}"
            )
        elif hold_pins is not None and hold_pins != pinned:
            report.errors.append(
                f"{own_mod.name}: its hold names {listing(hold_pins)}, but the installers pin "
                f"{listing(pinned)}; update or remove the hold in {REGISTRY}"
            )
        elif hold_pins is None and len(set(pinned.values())) > 1:
            report.errors.append(
                f"{own_mod.name}: the installers pin different releases ({listing(pinned)}). "
                f"Pin the same release in every installer, or record why in a hold in {REGISTRY}."
            )

        if github is None:
            status.state = "held; newest not checked" if hold else "newest not checked"
            continue
        strict = release or own_mod.id in touched
        commit_pins = "github-commit-zip" in kinds.values()
        try:
            newest = newest_release(github, own_mod, commit_pins)
            tags = github.tags(own_mod.repository) if commit_pins else None
        except GitHubError as error:
            message = f"{own_mod.name}: could not read its newest release: {error}"
            (report.errors if strict else report.warnings).append(message)
            status.state = "newest unknown"
            continue
        status.newest = newest.tag
        status.pins = {key: display(value, kinds[key], tags) for key, value in pinned.items()}
        behind = sorted({installer_names[pin.installer] for pin in pins
                         if not is_newest(pin, newest)})
        if not behind:
            status.state = "newest"
            if hold is not None and not hold_done:
                report.errors.append(
                    f"{own_mod.name}: every installer has the newest release {newest.tag}; "
                    f"remove its hold from {REGISTRY}"
                )
            continue
        if hold is not None and hold.release == newest.tag:
            status.state = "held (see notes)"
            report.notes.append(
                f"{own_mod.name} {newest.tag} is held (decided {hold.decided}): {hold.reason}"
            )
            continue
        covered = f" Its hold covers only {hold.release}." if hold is not None else ""
        verb = "is" if len(behind) == 1 else "are"
        message = (
            f"{own_mod.name}: {' and '.join(behind)} {verb} not on its newest release "
            f"{newest.tag}.{covered} Pin {newest.tag} in every installer in the same step, or "
            f"record a hold with its reason in {REGISTRY}."
        )
        status.state = f"behind {newest.tag}"
        (report.errors if strict else report.warnings).append(message)
    return report


# --- Output ---------------------------------------------------------------------------


def render(report: Report, registry: Registry) -> str:
    installers = list(registry.installers)
    header = ["Mod", *(installer.name for installer in installers), "Newest", "State"]
    rows = [header]
    for status in report.statuses:
        rows.append([
            status.own_mod.name,
            *(status.pins.get(installer.id, "-") for installer in installers),
            status.newest or "?",
            status.state,
        ])
    widths = [max(len(row[column]) for row in rows) for column in range(len(header) - 1)]
    lines = [f"Our own mods ({REGISTRY}); {report.mode}", ""]
    lines.extend(f"Read {source}" for source in report.sources)
    lines.append("")
    for row in rows:
        cells = [cell.ljust(widths[index]) for index, cell in enumerate(row[:-1])]
        lines.append("  ".join([*cells, row[-1]]).rstrip())
    for title, messages in (("Errors", report.errors), ("Warnings", report.warnings),
                            ("Notes", report.notes)):
        if messages:
            lines.extend(["", f"{title} ({len(messages)}):"])
            lines.extend(f"- {message}" for message in messages)
    if not report.errors:
        lines.extend(["", "OK: no installer is out of step."])
    return "\n".join(lines) + "\n"


def render_markdown(report: Report, registry: Registry) -> str:
    installers = list(registry.installers)
    lines = [f"### Our own mods: {report.mode}", ""]
    lines.append("| Mod | " + " | ".join(installer.name for installer in installers)
                 + " | Newest | State |")
    lines.append("|---" * (len(installers) + 3) + "|")
    for status in report.statuses:
        cells = [status.pins.get(installer.id, "–") for installer in installers]
        state = status.state.replace("|", "/")
        lines.append(f"| {status.own_mod.name} | " + " | ".join(cells)
                     + f" | {status.newest or '?'} | {state} |")
    for title, messages in (("Errors", report.errors), ("Warnings", report.warnings),
                            ("Notes", report.notes)):
        if messages:
            lines.extend(["", f"**{title}**", ""])
            lines.extend(f"- {message}" for message in messages)
    lines.extend(["", *(f"Read {source}." for source in report.sources)])
    return "\n".join(lines) + "\n"


def annotate(report: Report) -> None:
    """GitHub Actions annotations, so the PR shows each finding."""
    def escape(message: str) -> str:
        return message.replace("%", "%25").replace("\r", "%0D").replace("\n", "%0A")

    for message in report.errors:
        print(f"::error file={REGISTRY},title=Own mod pins::{escape(message)}")
    for message in report.warnings:
        print(f"::warning file={REGISTRY},title=Own mod pins::{escape(message)}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    commands = parser.add_subparsers(dest="command", required=True)
    run = commands.add_parser("check", help="compare the pins of our own mods")
    run.add_argument("--root", type=Path, default=Path.cwd(), help="repository root")
    run.add_argument("--offline", action="store_true",
                     help="skip the GitHub lookup of the newest releases")
    run.add_argument("--base", help="git ref this change is compared with")
    run.add_argument("--release", action="store_true",
                     help="require the newest release (or a hold) for every mod")
    run.add_argument("--remote", default="origin",
                     help="remote that holds the branches installers live on")
    run.add_argument("--fetch", action="store_true",
                     help="fetch those branches from the remote first")
    run.add_argument("--fetch-depth", type=int, help="shallow fetch, for CI checkouts only")
    run.add_argument("--api", default=os.environ.get("CEBG_GITHUB_API", GITHUB_API),
                     help=argparse.SUPPRESS)
    args = parser.parse_args(argv)

    root = args.root.resolve()
    try:
        registry = load_registry(root / REGISTRY)
        if args.fetch:
            fetch_branches(root, registry, args.remote, args.fetch_depth)
    except (RegistryError, SourceError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    github = None if args.offline else GitHub(args.api, token)
    report = check(root, registry, github, base=args.base, release=args.release,
                   remote=args.remote)
    sys.stdout.write(render(report, registry))
    if os.environ.get("GITHUB_ACTIONS") == "true":
        annotate(report)
        summary = os.environ.get("GITHUB_STEP_SUMMARY")
        if summary:
            with open(summary, "a", encoding="utf-8") as handle:
                handle.write(render_markdown(report, registry))
    return 1 if report.errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
