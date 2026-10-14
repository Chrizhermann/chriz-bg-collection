"""Check recipe batches against a pinned ZIP's literal TP2 declaration order.

This read-only authoring check does not execute WeiDU or inspect a game folder.
It deliberately supports direct-layout and single-wrapper ZIPs (such as GitHub commit
archives) with literal DESIGNATED components; other layouts or generated component
identities need their own source audit.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import tomllib
import zipfile
from pathlib import Path


def mask_tp2_source(text: str) -> str:
    """Mask comments/string contents while retaining component-name placeholders."""
    masked: list[str] = []
    index = 0
    while index < len(text):
        if text.startswith("//", index):
            end = text.find("\n", index + 2)
            index = len(text) if end < 0 else end
            masked.append(" ")
        elif text.startswith("/*", index):
            index += 2
            depth = 1
            while index < len(text) and depth:
                if text.startswith("/*", index):
                    depth += 1
                    index += 2
                elif text.startswith("*/", index):
                    depth -= 1
                    index += 2
                else:
                    index += 1
            if depth:
                raise ValueError("frozen TP2 has an unterminated comment")
            masked.append(" ")
        elif text[index] in '~"%':
            delimiter = "~~~~~" if text.startswith("~~~~~", index) else text[index]
            end = text.find(delimiter, index + len(delimiter))
            if end < 0:
                raise ValueError("frozen TP2 has an unterminated string")
            index = end + len(delimiter)
            masked.append(" __TP2_STRING__ ")
        else:
            masked.append(text[index])
            index += 1
    return "".join(masked)


def literal_component_order(text: str) -> list[int]:
    """Recognize explicit literal component IDs only; never infer numeric order."""
    code = mask_tp2_source(text)
    declarations = list(re.finditer(
        r"\bBEGIN\s+(?:__TP2_STRING__|@\d+)\b", code, re.IGNORECASE,
    ))
    if not declarations:
        raise ValueError("TP2 has no supported component declarations")
    components: list[int] = []
    for index, declaration in enumerate(declarations):
        end = declarations[index + 1].start() if index + 1 < len(declarations) else len(code)
        body = code[declaration.end():end]
        designated = re.findall(
            r"\bDESIGNATED\s+(\d+)\b(?!\s*[+\-*/%=<>&|^])", body, re.IGNORECASE,
        )
        tokens = re.findall(r"\bDESIGNATED\b", body, re.IGNORECASE)
        if len(designated) != 1 or len(tokens) != 1:
            raise ValueError(f"TP2 component {index + 1} needs a literal DESIGNATED order audit")
        components.append(int(designated[0]))
    if len(components) != len(set(components)):
        raise ValueError("TP2 repeats a DESIGNATED component identity")
    return components


def validate_run_order(run: dict, native_order: list[int]) -> None:
    requested = run["components"]
    if len(requested) != len(set(requested)):
        raise ValueError(f"{run['run_id']} repeats a component identity")
    missing = set(requested) - set(native_order)
    if missing:
        raise ValueError(f"{run['run_id']} components absent from pinned TP2: {sorted(missing)}")
    expected = [component for component in native_order if component in requested]
    if requested != expected:
        raise ValueError(
            f"{run['run_id']} has incorrect component order: requested {requested}; "
            f"pinned TP2 installs {expected}"
        )


def payload_prefix(files: list[str], root_rule: str, tp2: str) -> str:
    """Mirror the engine's unique payload-root rule for normalized ZIP file paths.

    The direct layout has the TP2 at the archive root. A wrapper candidate is a
    top-level directory that contains every file and the TP2. Exactly one
    candidate may match, and it must be the layout the artifact pin declares.
    """
    direct = tp2 in files
    wrappers = sorted({path.split("/", 1)[0] for path in files if "/" in path})
    wrapped = [
        wrapper for wrapper in wrappers
        if f"{wrapper}/{tp2}" in files and all(path.startswith(f"{wrapper}/") for path in files)
    ]
    candidates = ([""] if direct else []) + [f"{wrapper}/" for wrapper in wrapped]
    if len(candidates) != 1:
        raise ValueError("pinned archive must have exactly one TP2 payload root")
    if (root_rule == "direct") != (candidates[0] == ""):
        raise ValueError(f"pinned archive layout does not match root_rule {root_rule}")
    return candidates[0]


def check_archive_component_order(recipe: Path, mod_id: str, archive_path: Path) -> dict:
    mod = tomllib.loads((recipe / "mods" / f"{mod_id}.toml").read_text(encoding="utf-8"))
    artifact = tomllib.loads(
        (recipe / "artifacts" / f"{mod['artifact_id']}.toml").read_text(encoding="utf-8")
    )
    source = artifact["source"]
    layout = artifact["archive"]
    if layout["kind"] != "zip" or layout["root_rule"] not in {"direct", "single-wrapper"}:
        raise ValueError("native order check needs a reviewed direct-layout or single-wrapper ZIP")
    if archive_path.stat().st_size != source["expected_length"]:
        raise ValueError("archive length differs from the recipe pin")
    with archive_path.open("rb") as archive_file:
        digest = hashlib.file_digest(archive_file, "sha256").hexdigest()
    if digest != source["sha256"]:
        raise ValueError("archive SHA-256 differs from the recipe pin")
    normalize = lambda path: path.replace("\\", "/").casefold()
    with zipfile.ZipFile(archive_path) as archive:
        files = [entry for entry in archive.infolist() if not entry.is_dir()]
        prefix = payload_prefix(
            [normalize(entry.filename) for entry in files], layout["root_rule"], normalize(mod["tp2"]),
        )
        matches = [entry for entry in files
                   if normalize(entry.filename) == prefix + normalize(mod["tp2"])]
        if len(matches) != 1:
            raise ValueError("pinned archive must contain exactly one descriptor TP2")
        entry = matches[0]
        if entry.file_size > layout["limits"]["max_entry_uncompressed_bytes"]:
            raise ValueError("pinned TP2 exceeds the recipe archive entry limit")
        tp2 = archive.read(entry)
    order = literal_component_order(tp2.decode("latin-1"))
    collection = tomllib.loads((recipe / "collection.toml").read_text(encoding="utf-8"))
    runs = [run for run in collection["runs"] if run["mod_id"] == mod_id]
    if not runs:
        raise ValueError(f"recipe contains no runs for {mod_id}")
    for run in runs:
        validate_run_order(run, order)
    return {
        "recipe": str(recipe), "mod_id": mod_id, "artifact_id": mod["artifact_id"],
        "archive_sha256": digest, "tp2_sha256": hashlib.sha256(tp2).hexdigest(),
        "native_components": order, "checked_runs": [run["run_id"] for run in runs],
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--recipe-root", type=Path, action="append", required=True)
    parser.add_argument("--mod-id", required=True)
    parser.add_argument("--archive", type=Path, required=True)
    args = parser.parse_args()
    try:
        results = [check_archive_component_order(root, args.mod_id, args.archive)
                   for root in args.recipe_root]
    except (ValueError, OSError, zipfile.BadZipFile) as error:
        parser.exit(1, f"Component order check failed: {error}\n")
    print(json.dumps(results, indent=2))


if __name__ == "__main__":
    main()
