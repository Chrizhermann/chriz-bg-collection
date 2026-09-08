#!/usr/bin/env python3
"""Build the frozen, local-only 2026-09-08 combined playtest recipe.

This intentionally starts from the public curated recipe and writes only to a
separate destination. Development snapshots use the ordinary manual archive
contract so the installer cache still verifies their exact length and SHA-256.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import shutil
import sys
import tempfile
import tomllib
import zipfile


REQUIRED_SOURCE_IDS = (
    "chriz-bg-modpack",
    "chriz-sod-remix",
    "artisans-kitpack",
    "spell-rev-lightning",
    "chriz-bg-rebalance",
    "srcb-rr-compat",
    "safana-in-amn",
)

SOURCE_MODS = {
    "chriz-bg-modpack": ("chriz-bg-modpack",),
    "chriz-sod-remix": ("chriz-sod-remix",),
    "artisans-kitpack": ("artisanskitpack", "artisanskitpack-npc", "artisanskitpack-tweak"),
    "spell-rev-lightning": ("spell-rev",),
    "chriz-bg-rebalance": ("chriz-bg-rebalance",),
}

PRE_CONTINUITY_COMPONENTS = (110, 140, 170, 190, 192, 193, 194, 195, 196, 197, 198, 220, 221, 222, 223)


def _read_json(path: Path) -> object:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"cannot read frozen source manifest {path}: {error}") from error


def _safe_archive_path(root: Path, value: object, source_id: str) -> Path:
    if not isinstance(value, str) or not value or Path(value).is_absolute():
        raise ValueError(f"source {source_id!r} archive must be a non-empty relative path")
    path = (root / value).resolve()
    try:
        path.relative_to(root.resolve())
    except ValueError as error:
        raise ValueError(f"source {source_id!r} archive escapes the frozen source directory") from error
    return path


def _verify_zip(source: dict[str, object], archive: Path) -> None:
    source_id = str(source["id"])
    if not archive.is_file():
        raise ValueError(f"required frozen source {source_id!r} is unavailable: {archive}")
    data = archive.read_bytes()
    expected_length = source.get("size_bytes")
    expected_hash = source.get("sha256")
    if expected_length != len(data):
        raise ValueError(f"frozen source {source_id!r} length mismatch: expected {expected_length}, got {len(data)}")
    actual_hash = hashlib.sha256(data).hexdigest()
    if not isinstance(expected_hash, str) or actual_hash != expected_hash.lower():
        raise ValueError(f"frozen source {source_id!r} SHA-256 mismatch: expected {expected_hash}, got {actual_hash}")

    try:
        with zipfile.ZipFile(archive) as package:
            infos = package.infolist()
    except (OSError, zipfile.BadZipFile) as error:
        raise ValueError(f"frozen source {source_id!r} is not a readable ZIP: {error}") from error
    if not infos:
        raise ValueError(f"frozen source {source_id!r} ZIP is empty")
    names: set[str] = set()
    total = 0
    largest = 0
    depth = 0
    for info in infos:
        normalized = info.filename.replace("\\", "/").rstrip("/")
        path = PurePosixPath(normalized)
        if not normalized or path.is_absolute() or ".." in path.parts:
            raise ValueError(f"frozen source {source_id!r} contains unsafe ZIP path {info.filename!r}")
        folded = normalized.casefold()
        if folded in names:
            raise ValueError(f"frozen source {source_id!r} contains a duplicate case-insensitive ZIP path {info.filename!r}")
        names.add(folded)
        depth = max(depth, len(path.parts))
        if not info.is_dir():
            total += info.file_size
            largest = max(largest, info.file_size)
    observed = {
        "entry_count": len(infos),
        "max_entry_bytes": largest,
        "max_depth": depth,
        "uncompressed_bytes": total,
    }
    for key, value in observed.items():
        if source.get(key) != value:
            raise ValueError(f"frozen source {source_id!r} {key} mismatch: expected {source.get(key)}, got {value}")

    publish_roots = source.get("publish_roots")
    tp2_paths = source.get("tp2_paths")
    if source.get("root_rule") != "direct" or not isinstance(publish_roots, list) or not publish_roots:
        raise ValueError(f"frozen source {source_id!r} must declare direct non-empty publish_roots")
    if not isinstance(tp2_paths, list) or not tp2_paths:
        raise ValueError(f"frozen source {source_id!r} must declare non-empty tp2_paths")
    for tp2 in tp2_paths:
        if not isinstance(tp2, str) or tp2.casefold() not in names:
            raise ValueError(f"frozen source {source_id!r} is missing declared TP2 {tp2!r}")


def load_sources(path: Path) -> dict[str, dict[str, object]]:
    payload = _read_json(path)
    if not isinstance(payload, dict) or payload.get("schema_version") != 1 or not isinstance(payload.get("sources"), list):
        raise ValueError("frozen source manifest must have schema_version 1 and a sources list")
    sources: dict[str, dict[str, object]] = {}
    for item in payload["sources"]:
        if not isinstance(item, dict) or not isinstance(item.get("id"), str):
            raise ValueError("every frozen source entry must have a string id")
        source_id = item["id"]
        if source_id in sources:
            raise ValueError(f"duplicate frozen source id {source_id!r}")
        sources[source_id] = item
    missing = [source_id for source_id in REQUIRED_SOURCE_IDS if source_id not in sources]
    if missing:
        raise ValueError(f"missing required frozen sources: {', '.join(missing)}")
    extras = sorted(set(sources) - set(REQUIRED_SOURCE_IDS))
    if extras:
        raise ValueError(f"unexpected frozen sources (recipe mapping is not authored): {', '.join(extras)}")
    for source_id in REQUIRED_SOURCE_IDS:
        source = sources[source_id]
        required = {
            "archive", "sha256", "size_bytes", "entry_count", "max_entry_bytes", "max_depth",
            "uncompressed_bytes", "root_layout", "root_rule", "publish_roots", "tp2_paths",
            "max_compression_ratio", "commit", "dirty_overlay", "source_path", "provenance",
        }
        absent = sorted(required - set(source))
        if absent:
            raise ValueError(f"frozen source {source_id!r} lacks metadata: {', '.join(absent)}")
        provenance = source["provenance"]
        if not isinstance(provenance, dict) or not all(isinstance(provenance.get(key), str) and provenance[key] for key in ("homepage", "license", "provenance_url")):
            raise ValueError(f"frozen source {source_id!r} has incomplete provenance")
        commit = source["commit"]
        if not isinstance(commit, str) or not re.fullmatch(r"[0-9a-fA-F]{40}", commit):
            raise ValueError(f"frozen source {source_id!r} commit must be 40 hexadecimal characters")
        archive = _safe_archive_path(path.parent, source["archive"], source_id)
        _verify_zip(source, archive)
        source["_archive_path"] = archive
    return sources


def _quote(value: str) -> str:
    return json.dumps(value, ensure_ascii=False)


def _artifact_id(source_id: str, source: dict[str, object]) -> str:
    return f"local-playtest-{source_id}-{str(source['sha256'])[:12]}"


def _artifact_text(source_id: str, source: dict[str, object]) -> str:
    artifact_id = _artifact_id(source_id, source)
    archive = Path(str(source["archive"]))
    provenance = source["provenance"]
    assert isinstance(provenance, dict)
    overlay = source["dirty_overlay"]
    reference = str(source["commit"])
    if overlay:
        reference += f"+local-overlay-{str(source['sha256'])[:12]}"
    roots = ", ".join(_quote(str(value)) for value in source["publish_roots"])
    tp2s = ", ".join(_quote(str(value)) for value in source["tp2_paths"])
    return f'''id = {_quote(artifact_id)}
name = {_quote('LOCAL PLAYTEST snapshot: ' + source_id)}
version = {_quote(reference)}
acquisition = "manual-user-supplied"

[source]
kind = "manual"
url = {_quote(str(provenance['provenance_url']))}
reference = {_quote(reference)}
expected_filename = {_quote(archive.name)}
expected_length = {source['size_bytes']}
sha256 = {_quote(str(source['sha256']))}

[archive]
kind = "zip"
root_rule = "direct"
publish_roots = [{roots}]
tp2_paths = [{tp2s}]

[archive.limits]
max_depth = {source['max_depth']}
max_entries = {source['entry_count']}
max_entry_uncompressed_bytes = {max(1, int(source['max_entry_bytes']))}
max_total_uncompressed_bytes = {max(1, int(source['uncompressed_bytes']))}
max_compression_ratio = {source['max_compression_ratio']}

[provenance]
homepage = {_quote(str(provenance['homepage']))}
license = {_quote(str(provenance['license']))}
url = {_quote(str(provenance['provenance_url']))}
reviewed_on = "2026-09-08"
'''


def _replace_scalar(text: str, key: str, value: str) -> str:
    pattern = rf'(?m)^{re.escape(key)} = "[^"]*"$'
    replaced, count = re.subn(pattern, f'{key} = {_quote(value)}', text, count=1)
    if count != 1:
        raise ValueError(f"expected exactly one {key!r} field")
    return replaced


def _append_components(mod_text: str, additions: list[tuple[int, str]]) -> str:
    parsed = tomllib.loads(mod_text)
    ids = {component["id"] for component in parsed["components"]}
    blocks = []
    for component, name in additions:
        if component in ids:
            raise ValueError(f"component {component} already exists in mod {parsed['id']}")
        blocks.append(f'[[components]]\nid = {component}\nname = {_quote(name)}\n')
    return mod_text.rstrip() + "\n\n" + "\n".join(blocks)


def _run_block(run_id: str, mod_id: str, phase: str, components: tuple[int, ...] | list[int]) -> str:
    values = ", ".join(str(value) for value in components)
    return f'''[[runs]]
run_id = {_quote(run_id)}
mod_id = {_quote(mod_id)}
phase = {_quote(phase)}
components = [{values}]
args = []

'''


def _replace_run_components(collection: str, run_id: str, components: tuple[int, ...] | list[int]) -> str:
    pattern = re.compile(rf'(?ms)^\[\[runs\]\]\nrun_id = "{re.escape(run_id)}"\n.*?(?=^\[\[runs\]\]|^\[\[features\]\])')
    match = pattern.search(collection)
    if match is None:
        raise ValueError(f"run {run_id!r} is missing")
    block = match.group(0)
    values = ", ".join(str(value) for value in components)
    changed, count = re.subn(r'(?m)^components = \[[^\]]*\]$', f"components = [{values}]", block, count=1)
    if count != 1:
        raise ValueError(f"run {run_id!r} components field is ambiguous")
    return collection[: match.start()] + changed + collection[match.end() :]


def _insert_before_run(collection: str, marker_run_id: str, blocks: str) -> str:
    marker = f'[[runs]]\nrun_id = "{marker_run_id}"'
    if collection.count(marker) != 1:
        raise ValueError(f"run insertion marker {marker_run_id!r} is ambiguous")
    return collection.replace(marker, blocks + marker, 1)


def _feature(title: str, feature_id: str, source_label: str, parent: str, run_id: str, component: int, requires: tuple[str, ...] = ()) -> str:
    lines = [
        "[[features]]", f"id = {_quote(feature_id)}", f"title = {_quote(title)}",
        f"description = {_quote('LOCAL EXPERIMENT ONLY: ' + title + '. Frozen for the combined 2026-09-08 playtest; no public curation or live acceptance is implied.')}",
        'category = "collection"', f"source_label = {_quote(source_label)}", 'group_label = "Local combined playtest"',
        'decision = "default"', 'readiness = "ready"', f"parent = {_quote(parent)}",
    ]
    if requires:
        lines.append("requires = [" + ", ".join(_quote(value) for value in requires) + "]")
    lines.append(f'components = [{{ run_id = {_quote(run_id)}, component = {component} }}]')
    return "\n".join(lines) + "\n"


def _parent_feature() -> str:
    return '''[[features]]
id = "mod:srcb-rr-compat"
title = "SR/RR compatibility (local experiment)"
description = "LOCAL EXPERIMENT ONLY: install the separately frozen SR/RR compatibility tail. This does not change the public curation decision."
category = "collection"
source_label = "SR/RR compatibility"
group_label = "Local combined playtest"
decision = "default"
readiness = "ready"
'''


def _add_sod_components_to_mandatory_feature(collection: str, additions: tuple[int, ...]) -> str:
    start = collection.index('id = "feature:chriz-sod-remix:mandatory-components"')
    end = collection.find("\n[[features]]", start)
    if end < 0:
        end = len(collection)
    block = collection[start:end]
    marker = "components = [\n"
    if block.count(marker) != 1:
        raise ValueError("SoD mandatory feature component list is ambiguous")
    closing = block.index("\n]", block.index(marker))
    refs = "".join(f'  {{ run_id = "chriz-sod-remix-bg2", component = {component} }},\n' for component in additions)
    block = block[:closing] + "\n" + refs.rstrip("\n") + block[closing:]
    return collection[:start] + block + collection[end:]


def _write_mod_overrides(destination: Path, sources: dict[str, dict[str, object]]) -> None:
    for source_id, mod_ids in SOURCE_MODS.items():
        artifact_id = _artifact_id(source_id, sources[source_id])
        for mod_id in mod_ids:
            path = destination / "mods" / f"{mod_id}.toml"
            text = _replace_scalar(path.read_text(encoding="utf-8"), "artifact_id", artifact_id)
            path.write_text(text, encoding="utf-8", newline="\n")

    additions = {
        "spell-rev": [(80, "Lightning Bolt overhaul (primary local playtest variant)")],
        "chriz-sod-remix": [(135, "Assassin ambush without dead magic"), (256, "Bridge elemental finale"), (265, "Mizhena amulet, filler cuts, and XP")],
        "chriz-bg-modpack": [(189, "Safana: one-time SoA arrival inventory cleanup"), (199, "EET companion continuity"), (620, "Imoen Spellhold party-average XP")],
        "chriz-bg-rebalance": [(110, "Dragon melee delivery and death-protection rebalance"), (111, "Dragon wing-buffet spacing rebalance")],
    }
    for mod_id, components in additions.items():
        path = destination / "mods" / f"{mod_id}.toml"
        path.write_text(_append_components(path.read_text(encoding="utf-8"), components), encoding="utf-8", newline="\n")

    compat = sources["srcb-rr-compat"]
    tp2_paths = compat["tp2_paths"]
    assert isinstance(tp2_paths, list)
    text = f'''id = "srcb-rr-compat"
artifact_id = {_quote(_artifact_id('srcb-rr-compat', compat))}
name = "SRCB SR/RR compatibility (LOCAL EXPERIMENT)"
tp2 = {_quote(str(tp2_paths[0]))}
language = 0
weidu_artifact_id = "weidu-249-amd64"
invocation_mode = "setup-name"

[[components]]
id = 0
name = "Spell Revisions / Rogue Rebalancing compatibility"
'''
    (destination / "mods/srcb-rr-compat.toml").write_text(text, encoding="utf-8", newline="\n")

    safana = sources["safana-in-amn"]
    safana_tp2s = safana["tp2_paths"]
    assert isinstance(safana_tp2s, list)
    safana_text = f'''id = "safana"
artifact_id = {_quote(_artifact_id('safana-in-amn', safana))}
name = "Safana in Amn (LOCAL COMBINED PLAYTEST)"
tp2 = {_quote(str(safana_tp2s[0]))}
language = 0
weidu_artifact_id = "weidu-249-amd64"
invocation_mode = "setup-name"

[[components]]
id = 0
name = "Safana in Amn: v0.5"
'''
    (destination / "mods/safana.toml").write_text(safana_text, encoding="utf-8", newline="\n")


def _write_collection(destination: Path) -> None:
    path = destination / "collection.toml"
    collection = path.read_text(encoding="utf-8").replace("\r\n", "\n")
    collection = _replace_run_components(collection, "chriz-sod-remix-bg2", [100, 110, 120, 130, 135, 140, 150, 145, 160, 170, 180, 175, 185, 190, 195, 210, 197, 187, 200, 215, 220, 225, 245, 230, 240, 250, 255, 256, 260, 265, 270, 280, 290, 900, 910])
    collection = _replace_run_components(collection, "chriz-bg-rebalance-bg2", [100, 101, 110, 111, 120, 121, 400, 401, 404, 405, 407, 408])
    collection = _replace_run_components(collection, "chriz-bg-modpack-bg2", [130, 400, 410, 430, 440, 450, 610])

    collection = _insert_before_run(collection, "stratagems-bg2", _run_block("srcb-rr-compat-bg2", "srcb-rr-compat", "main", [0]))
    pre_final = _run_block("chriz-bg-modpack-pre-continuity-bg2", "chriz-bg-modpack", "main", PRE_CONTINUITY_COMPONENTS) + _run_block("chriz-bg-modpack-continuity-bg2", "chriz-bg-modpack", "main", [199])
    collection = _insert_before_run(collection, "eet-end-bg2", pre_final)
    collection = _insert_before_run(collection, "spell-rev-npc-spellbooks-bg2", _run_block("chriz-bg-modpack-late-companions-bg2", "chriz-bg-modpack", "post-eet-end", [189, 620]))
    collection = _insert_before_run(collection, "chriz-bg-modpack-late-companions-bg2", _run_block("safana-bg2", "safana", "post-eet-end", [0]))
    collection = _insert_before_run(collection, "klatu-armor-thieving-bg2", _run_block("spell-rev-lightning-bg2", "spell-rev", "post-eet-end", [80]))

    for component in PRE_CONTINUITY_COMPONENTS:
        collection = collection.replace(
            f'components = [{{ run_id = "chriz-bg-modpack-bg2", component = {component} }}]',
            f'components = [{{ run_id = "chriz-bg-modpack-pre-continuity-bg2", component = {component} }}]',
        )
    collection = _add_sod_components_to_mandatory_feature(collection, (135, 256, 265))
    features = [
        _feature("Lightning Bolt overhaul, primary component 80", "feature:spell-rev:component-80", "Spell Revisions", "mod:spell-rev", "spell-rev-lightning-bg2", 80),
        _parent_feature(),
        _feature("SR/RR compatibility tail", "feature:srcb-rr-compat:component-0", "SR/RR compatibility", "mod:srcb-rr-compat", "srcb-rr-compat-bg2", 0, ("feature:spell-rev:mandatory-components", "feature:rr:component-11", "feature:rr:component-12")),
        '''[[features]]
id = "mod:safana"
title = "Safana in Amn core (local combined playtest)"
description = "LOCAL EXPERIMENT ONLY: add Safana in Amn v0.5 so the newly implemented component 189 cleanup has its real prerequisite. Bard and Abettor preset variants remain excluded."
category = "npcs"
source_label = "Safana in Amn"
group_label = "Local combined playtest"
decision = "default"
readiness = "ready"
''',
        _feature("Safana in Amn core", "feature:safana:mandatory-components", "Safana in Amn", "mod:safana", "safana-bg2", 0),
        _feature("Safana arrival inventory cleanup", "feature:chriz-bg-modpack:component-189", "chriz-bg-modpack", "mod:chriz-bg-modpack", "chriz-bg-modpack-late-companions-bg2", 189, ("feature:safana:mandatory-components",)),
        _feature("EET companion continuity", "feature:chriz-bg-modpack:component-199", "chriz-bg-modpack", "mod:chriz-bg-modpack", "chriz-bg-modpack-continuity-bg2", 199, ("mod:xan", "feature:yeslicknpc:component-1")),
        _feature("Imoen Spellhold party-average XP", "feature:chriz-bg-modpack:component-620", "chriz-bg-modpack", "mod:chriz-bg-modpack", "chriz-bg-modpack-late-companions-bg2", 620, ("mod:eeex",)),
        _feature("Dragon melee and death-protection rebalance", "feature:chriz-bg-rebalance:component-110", "BG Rebalance", "mod:chriz-bg-rebalance", "chriz-bg-rebalance-bg2", 110, ("mod:eeex", "feature:stratagems:mandatory-components")),
        _feature("Dragon wing-buffet spacing rebalance", "feature:chriz-bg-rebalance:component-111", "BG Rebalance", "mod:chriz-bg-rebalance", "chriz-bg-rebalance-bg2", 111, ("feature:stratagems:mandatory-components",)),
    ]
    collection = collection.rstrip() + "\n\n" + "\n".join(features)
    tomllib.loads(collection)
    path.write_text(collection, encoding="utf-8", newline="\n")


def _write_preset(destination: Path) -> None:
    path = destination / "presets/chris-recommended.toml"
    text = path.read_text(encoding="utf-8").rstrip()
    selections = [
        "feature:spell-rev:component-80", "mod:srcb-rr-compat", "feature:srcb-rr-compat:component-0",
        "mod:safana", "feature:safana:mandatory-components",
        "feature:chriz-bg-modpack:component-189", "feature:chriz-bg-modpack:component-199",
        "feature:chriz-bg-modpack:component-620", "feature:chriz-bg-rebalance:component-110",
        "feature:chriz-bg-rebalance:component-111",
    ]
    for selection in selections:
        text += f'\n{_quote(selection)} = "on"'
    path.write_text(text + "\n", encoding="utf-8", newline="\n")


def build_recipe(root: Path, sources_json: Path, destination: Path) -> None:
    root = root.resolve()
    base = (root / "recipes/curated-full-current").resolve()
    destination = destination.resolve()
    if destination == base or base in destination.parents or destination in base.parents:
        raise ValueError("destination must be separate from the public curated recipe")
    if destination.exists():
        raise ValueError(f"destination already exists; refusing to overwrite or delete it: {destination}")
    sources = load_sources(sources_json.resolve())
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="combined-recipe-", dir=destination.parent) as temp_dir:
        staged = Path(temp_dir) / "recipe"
        shutil.copytree(base, staged)
        for source_id, source in sources.items():
            artifact_id = _artifact_id(source_id, source)
            (staged / "artifacts" / f"{artifact_id}.toml").write_text(_artifact_text(source_id, source), encoding="utf-8", newline="\n")
        _write_mod_overrides(staged, sources)
        _write_collection(staged)
        _write_preset(staged)
        lock = {"schema_version": 1, "local_only": True, "sources": [{key: value for key, value in source.items() if key != "_archive_path"} for source in sources.values()]}
        (staged / "combined-playtest-sources.json").write_text(json.dumps(lock, indent=2) + "\n", encoding="utf-8", newline="\n")
        (staged / "release.json").unlink(missing_ok=True)
        (staged / "LOCAL-ONLY.md").write_text("# LOCAL EXPERIMENT ONLY\n\nThis frozen recipe is for one combined fresh SR-on playtest. It is not a public recipe, release, or live-acceptance record. Supply the exact ZIPs named in `combined-playtest-sources.json` through the normal verified manual-cache flow.\n", encoding="utf-8", newline="\n")
        shutil.move(str(staged), destination)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Prepare the local-only combined playtest recipe")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--sources", type=Path, required=True, help="Frozen sources.json from the source packager")
    parser.add_argument("--output", type=Path, default=Path("target/combined-playtest-20260908/recipe"))
    args = parser.parse_args(argv)
    try:
        build_recipe(args.root, args.sources, args.output)
    except (OSError, ValueError) as error:
        print(f"combined playtest recipe generation failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
