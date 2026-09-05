"""Build the executable full recipe from Christopher's authored curation model.

This authoring path deliberately starts from the reviewed public-alpha feature model and
adds only curation-map targets backed by frozen sources.  It never reads WeiDU.log or the
legacy creator-full replay tables.
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import sys
import tomllib
from collections import Counter
from pathlib import Path

if __package__ in {None, ""}:
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from tools.curation_audit import Decision, RowKey, load_catalogs, load_curation_map


RECIPE_VERSION = "0.1.0-alpha.8"
RECIPE_LABEL = "CEBG curated full setup"
ADDED_CATALOGS = {"BARDICWONDERS", "BG1NPC", "BRANWEN", "CDTWEAKS", "EVANDRA", "IWDIFICATION"}

# bg1npc v32 bg1npc.tp2: DESIGNATED declaration order (not numeric ID order).
# --force-install-list selects components but WeiDU still visits the TP2 in this order.
BG1NPC_NATIVE_ORDER = [
    0, 10,
    20, 21, 22, 23, 24, 30, 31, 32, 33, 34, 40, 41, 42, 43, 44,
    50, 51, 52, 53, 54, 60, 61, 62, 63, 64, 70, 71, 72, 73, 74,
    80, 90, 100, 110, 111, 112, 113, 114, 120, 130, 131,
    240, 241, 150, 155, 160, 200,
]

# Exact pinned Artisan chriz-v1.3.1 and Randomiser v8.1.1 TP2 order, restricted
# to offered choices. Re-audit a newly offered ID instead of guessing its position.
NATIVE_RUN_ORDERS = {
    "artisanskitpack-main-bg2": [
        1, 2, 20000, 20001, 8001, 8101, 8002, 8004,
        10001, 10002, 10003, 10004, 1100, 1003, 1006, 1004, 1005,
        1007, 1000, 1001, 1008, 1009, 2000, 2010, 2011, 2012, 2002,
        3000, 3010, 3003, 3011, 3004, 3001, 3002, 3005, 5100, 5110,
        5001, 5002, 7004, 7006, 7001, 7002, 7003, 7005, 9001,
    ],
    "randomiser-bg2": [500, 510, 530, 540, 560, 570, 1100, 9000, 10200, 10210, 10300],
}


def _preserve_native_run_orders(collection: str) -> str:
    for run_id, native_order in NATIVE_RUN_ORDERS.items():
        pattern = rf'(?m)(^run_id = "{re.escape(run_id)}"\nmod_id = "[^"\n]+"\nphase = "[^"\n]+"\n)components = (\[[^\]\n]*\])'
        matches = list(re.finditer(pattern, collection))
        if len(matches) != 1:
            raise ValueError(f"native component order requires one run: {run_id}")
        match = matches[0]
        components = tomllib.loads(f"components = {match.group(2)}")["components"]
        unknown = set(components) - set(native_order)
        if unknown or len(components) != len(set(components)):
            raise ValueError(f"{run_id} requires a component-order audit: {components}")
        ordered = [component for component in native_order if component in components]
        collection = collection[:match.start()] + match.group(1) + f"components = {ordered}" + collection[match.end():]
    return collection

CATALOG_RUNS = {
    "BG1NPC": "bg1npc-bg1",
    "BRANWEN": "branwen-bg2",
    "IWDIFICATION": "iwdification-bg2",
}

CATEGORY_BY_GROUP = {
    "Kits": "kits",
    "Tweaks & Additions": "tweaks",
    "Patches": "compatibility",
    "Cosmetic Changes": "cosmetics",
    "Content Changes": "content",
    "Rule Changes": "rules",
    "Convenience Tweaks/Cheats": "convenience",
    "NPC Tweaks": "npcs",
    "Miscellaneous Changes": "tweaks",
    "Class Updates": "rules",
    "Additional Spells": "spells",
}


ARTIFACTS = {
    "bardicwonders-v2.9c-balance.2.toml": """id = "bardicwonders-v2.9c-balance.2"
name = "Bardic Wonders — Christopher's balance fork"
version = "2.9c-balance.2"
acquisition = "fetch-only"

[source]
kind = "github-tag-archive"
url = "https://codeload.github.com/Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch/zip/refs/tags/v2.9c-balance.2"
reference = "v2.9c-balance.2"
expected_filename = "Bardic-Wonders-Chriz-Balance-Patch-2.9c-balance.2.zip"
expected_length = 5234586
sha256 = "bfa16cde633d9722ecc9ddb84693e0ff07dfb5fb2ebf1ec6a6b9d0e3a29cb4fb"
redirect_hosts = []

[archive]
kind = "zip"
root_rule = "single-wrapper"
publish_roots = ["BardicWonders"]
tp2_paths = ["BardicWonders/Setup-BardicWonders.tp2"]

[archive.limits]
max_depth = 8
max_entries = 1024
max_entry_uncompressed_bytes = 4194304
max_total_uncompressed_bytes = 67108864
max_compression_ratio = 512

[provenance]
homepage = "https://github.com/Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch"
license = "Fetch-only Christopher Hermann fork; the collection does not redistribute it"
url = "https://github.com/Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch/releases/tag/v2.9c-balance.2"
reviewed_on = "2026-09-05"
""",
    "branwen-8.toml": """id = "branwen-8"
name = "Branwen for BGII"
version = "8"
acquisition = "fetch-only"

[source]
kind = "github-release"
url = "https://github.com/Pocket-Plane-Group/Branwen_for_BGII/releases/download/v8/branwen-v8pre.zip"
reference = "v8"
expected_filename = "branwen-v8pre.zip"
expected_length = 13926210
sha256 = "f82c93e67f1910f0754364c17d40e7f4ee2c9609a39cd081efbfc69c9faaea17"
redirect_hosts = ["release-assets.githubusercontent.com"]

[archive]
kind = "zip"
root_rule = "direct"
publish_roots = ["Branwen"]
tp2_paths = ["Branwen/branwen.tp2"]

[archive.limits]
max_depth = 8
max_entries = 512
max_entry_uncompressed_bytes = 8388608
max_total_uncompressed_bytes = 33554432
max_compression_ratio = 100

[provenance]
homepage = "https://github.com/Pocket-Plane-Group/Branwen_for_BGII"
license = "Fetch-only third-party release; the collection does not redistribute it"
url = "https://github.com/Pocket-Plane-Group/Branwen_for_BGII/releases/tag/v8"
reviewed_on = "2026-09-05"
""",
    "cdtweaks-18.toml": """id = "cdtweaks-18"
name = "The Tweaks Anthology"
version = "18"
acquisition = "fetch-only"

[source]
kind = "github-release"
url = "https://github.com/Gibberlings3/Tweaks-Anthology/releases/download/v18/the-tweaks-anthology-v18.iemod"
reference = "v18"
expected_filename = "the-tweaks-anthology-v18.iemod"
expected_length = 21817521
sha256 = "776212ec781ddf14071a9f74df075824fe8430a13302d2bea7d11fd91d179b17"
redirect_hosts = ["release-assets.githubusercontent.com"]

[archive]
kind = "iemod"
root_rule = "direct"
publish_roots = ["cdtweaks"]
tp2_paths = ["cdtweaks/setup-cdtweaks.tp2"]

[archive.limits]
max_depth = 8
max_entries = 4096
max_entry_uncompressed_bytes = 4194304
max_total_uncompressed_bytes = 67108864
max_compression_ratio = 256

[provenance]
homepage = "https://github.com/Gibberlings3/Tweaks-Anthology"
license = "Fetch-only third-party release; the collection does not redistribute it"
url = "https://github.com/Gibberlings3/Tweaks-Anthology/releases/tag/v18"
reviewed_on = "2026-09-05"
""",
    "iwdification-11.toml": """id = "iwdification-11"
name = "IWDification"
version = "11"
acquisition = "fetch-only"

[source]
kind = "github-release"
url = "https://github.com/Gibberlings3/iwdification/releases/download/v11/iwdification-v11.iemod"
reference = "v11"
expected_filename = "iwdification-v11.iemod"
expected_length = 44166170
sha256 = "4af9d463d3b4afe1e296fe056b6c93e9b6b95e5bcb5541eca81dcbb8ba15db3f"
redirect_hosts = ["release-assets.githubusercontent.com"]

[archive]
kind = "iemod"
root_rule = "direct"
publish_roots = ["iwdification"]
tp2_paths = ["iwdification/setup-iwdification.tp2"]

[archive.limits]
max_depth = 8
max_entries = 4096
max_entry_uncompressed_bytes = 4194304
max_total_uncompressed_bytes = 134217728
max_compression_ratio = 512

[provenance]
homepage = "https://github.com/Gibberlings3/iwdification"
license = "Fetch-only third-party release; the collection does not redistribute it"
url = "https://github.com/Gibberlings3/iwdification/releases/tag/v11"
reviewed_on = "2026-09-05"
""",
}


MOD_SPECS = {
    "bardicwonders": ("bardicwonders-v2.9c-balance.2", "Bardic Wonders", "BardicWonders/Setup-BardicWonders.tp2", "BARDICWONDERS"),
    "branwen": ("branwen-8", "Branwen for BGII", "Branwen/branwen.tp2", "BRANWEN"),
    "cdtweaks": ("cdtweaks-18", "The Tweaks Anthology", "cdtweaks/setup-cdtweaks.tp2", "CDTWEAKS"),
    "evandra": ("creator-full-private-extras-20260902", "Evandra NPC", "evandra/setup-evandra.tp2", "EVANDRA"),
    "iwdification": ("iwdification-11", "IWDification", "iwdification/setup-iwdification.tp2", "IWDIFICATION"),
}


RUN_BLOCKS = {
    "before-xan": """[[runs]]
run_id = "evandra-core-bg2"
mod_id = "evandra"
phase = "main"
components = [0]
args = []

""",
    "before-ascension": """[[runs]]
run_id = "branwen-bg2"
mod_id = "branwen"
phase = "main"
components = [0]
args = []

""",
    "before-artisan": """[[runs]]
run_id = "bardicwonders-bg2"
mod_id = "bardicwonders"
phase = "main"
components = [1001, 1002, 1003, 1004, 1005, 1006, 1007, 1009, 1010, 1011, 2002, 2007, 2008, 2003, 2004, 2005, 2006]
args = []

""",
    "before-artisan-npc": """[[runs]]
run_id = "bardicwonders-garrick-bg2"
mod_id = "bardicwonders"
phase = "main"
components = [1008]
args = []

""",
    "before-crossmod": """[[runs]]
run_id = "evandra-crossmod-bg2"
mod_id = "evandra"
phase = "main"
components = [1]
args = []

""",
    "before-randomiser": """[[runs]]
run_id = "iwdification-bg2"
mod_id = "iwdification"
phase = "main"
components = [10, 60, 90, 130, 140, 190, 30, 40]
args = []

[[runs]]
run_id = "cdtweaks-bg2"
mod_id = "cdtweaks"
phase = "main"
components = []
args = []

""",
    "late-spell-scan": """[[runs]]
run_id = "cdtweaks-spell-save-penalties-bg2"
mod_id = "cdtweaks"
phase = "post-eet-end"
components = [2312]
args = []

""",
    "after-eet-end": """[[runs]]
run_id = "bardicwonders-dialogue-patch-bg2"
mod_id = "bardicwonders"
phase = "post-eet-end"
components = [1012, 3001]
args = []

""",
}


def _q(value: str) -> str:
    return json.dumps(value, ensure_ascii=False)


def _target_rows(curation_map) -> dict[RowKey, object]:
    return {row: target for target in curation_map.targets for row in target.rows}


def _run_for(row: RowKey) -> str:
    if row.catalog == "BARDICWONDERS":
        if row.component_id == 1008:
            return "bardicwonders-garrick-bg2"
        if row.component_id in {1012, 3001}:
            return "bardicwonders-dialogue-patch-bg2"
        return "bardicwonders-bg2"
    if row.catalog == "CDTWEAKS":
        return "cdtweaks-spell-save-penalties-bg2" if row.component_id == 2312 else "cdtweaks-bg2"
    if row.catalog == "EVANDRA":
        return "evandra-core-bg2" if row.component_id == 0 else "evandra-crossmod-bg2"
    return CATALOG_RUNS[row.catalog]


def _replace_feature(collection: str, feature_id: str, replacements: dict[str, str | None], additions: list[str] = []) -> str:
    pattern = re.compile(rf'(?ms)^\[\[features\]\]\nid = {re.escape(_q(feature_id))}\n.*?(?=^\[\[features\]\]|\Z)')
    match = pattern.search(collection)
    if match is None:
        raise ValueError(f"missing base feature {feature_id}")
    lines = match.group(0).rstrip().splitlines()
    for key, value in replacements.items():
        index = next((i for i, line in enumerate(lines) if line.startswith(f"{key} = ")), None)
        if value is None:
            if index is not None:
                lines.pop(index)
        elif index is not None:
            lines[index] = f"{key} = {value}"
        else:
            lines.append(f"{key} = {value}")
    for line in additions:
        key = line.split(" = ", 1)[0]
        if not any(existing.startswith(f"{key} = ") for existing in lines):
            lines.append(line)
    return collection[: match.start()] + "\n".join(lines) + "\n\n" + collection[match.end() :]


def _feature_blocks(root: Path, collection: dict) -> str:
    rows = load_catalogs(root / "docs/curation/components")
    row_by_key = {RowKey(row.catalog, row.component_id): row for row in rows}
    curation_map = load_curation_map(root / "manifest/curation-map.toml")
    existing = {feature["id"] for feature in collection["features"]}
    target_by_row = _target_rows(curation_map)
    feature_for_row = {
        row: target.target_id
        for row, target in target_by_row.items()
        if target.kind == "feature"
    }
    conflicts: dict[str, list[tuple[str, str]]] = {}
    for group in curation_map.choice_groups:
        ids = []
        for option in group.options:
            feature_id = feature_for_row.get(option)
            if feature_id is not None:
                ids.append(feature_id)
        for feature_id in ids:
            for other in ids:
                if other != feature_id:
                    conflicts.setdefault(feature_id, []).append(
                        (other, f"Choose at most one option for {group.subgroup}.")
                    )

    blocks = [
        "[[features]]\n"
        'id = "mod:iwdification"\n'
        'title = "IWDification"\n'
        'description = "Install the curated IWDification spell packs and selected visual, item, and rule updates."\n'
        'category = "collection"\n'
        'decision = "default"\n'
        'readiness = "ready"\n'
    ]
    for target in curation_map.targets:
        if target.kind != "feature" or target.target_id in existing:
            continue
        if not target.rows or target.rows[0].catalog not in ADDED_CATALOGS:
            raise ValueError(f"unhandled curation target {target.target_id}")
        target_rows = [row_by_key[row] for row in target.rows]
        first = target_rows[0]
        decisions = {row.decision for row in target_rows}
        if len(decisions) != 1:
            raise ValueError(f"mixed decisions for {target.target_id}")
        category = CATEGORY_BY_GROUP.get(first.group, "npcs" if first.catalog in {"BG1NPC", "BRANWEN", "EVANDRA"} else "tweaks")
        requires: list[str] = []
        if target.target_id == "feature:bardicwonders:component-1012":
            requires = ["feature:eeex:mandatory-components"]
        elif target.target_id == "feature:bardicwonders:component-3001":
            requires = ["feature:bardicwonders:component-1012"]
        components = [
            f'  {{ run_id = {_q(_run_for(RowKey(row.catalog, row.component_id)))}, component = {row.component_id} }},'
            for row in target_rows
        ]
        lines = [
            "[[features]]",
            f"id = {_q(target.target_id)}",
            f"title = {_q(first.component)}",
            f"description = {_q(first.component if len(target_rows) == 1 else 'Install the curated mandatory IWD arcane and divine spell packs.')}",
            f"category = {_q(category)}",
            f"decision = {_q(next(iter(decisions)).value)}",
            'readiness = "blocked"'
            if target.target_id == "feature:bardicwonders:component-1006"
            else 'readiness = "ready"',
        ]
        if target.target_id == "feature:bardicwonders:component-1006":
            lines.append(
                'unavailable_reason = "Darkbloom is unavailable with the selected Spell Revisions setup."'
            )
        if target.parent:
            lines.append(f"parent = {_q(target.parent)}")
        if requires:
            lines.append("requires = [" + ", ".join(_q(value) for value in requires) + "]")
        feature_conflicts = conflicts.get(target.target_id, [])
        if feature_conflicts:
            lines.append(
                "conflicts = ["
                + ", ".join(
                    f"{{ feature_id = {_q(feature_id)}, reason = {_q(reason)} }}"
                    for feature_id, reason in feature_conflicts
                )
                + "]"
            )
        if len(components) == 1:
            lines.append(f"components = [{components[0].strip().rstrip(',')}]")
        else:
            lines.extend(["components = [", *components, "]"])
        blocks.append("\n".join(lines))
    return "\n\n".join(blocks).rstrip() + "\n"


def _catalog_component_ids(root: Path, catalog: str) -> list[int]:
    curation_map = load_curation_map(root / "manifest/curation-map.toml")
    return [
        row.component_id
        for target in curation_map.targets
        if target.kind == "feature"
        for row in target.rows
        if row.catalog == catalog
    ]


def _write_mods(root: Path, destination: Path) -> None:
    rows = load_catalogs(root / "docs/curation/components")
    by_catalog = {}
    for row in rows:
        by_catalog.setdefault(row.catalog, {})[row.component_id] = row
    curation_map = load_curation_map(root / "manifest/curation-map.toml")
    target_rows = {
        row
        for target in curation_map.targets
        if target.kind == "feature"
        for row in target.rows
    }

    bg1_path = destination / "mods/bg1npc.toml"
    bg1 = tomllib.loads(bg1_path.read_text(encoding="utf-8"))
    declared = {component["id"] for component in bg1["components"]}
    additions = []
    for row in rows:
        key = RowKey(row.catalog, row.component_id)
        if row.catalog == "BG1NPC" and key in target_rows and row.component_id not in declared:
            additions.extend(["", "[[components]]", f"id = {row.component_id}", f"name = {_q(row.component)}"])
    bg1_path.write_text(bg1_path.read_text(encoding="utf-8").rstrip() + "\n" + "\n".join(additions) + "\n", encoding="utf-8", newline="\n")

    for mod_id, (artifact_id, name, tp2, catalog) in MOD_SPECS.items():
        component_rows = [
            row for row in rows if row.catalog == catalog and RowKey(catalog, row.component_id) in target_rows
        ]
        lines = [
            f"id = {_q(mod_id)}",
            f"artifact_id = {_q(artifact_id)}",
            f"name = {_q(name)}",
            f"tp2 = {_q(tp2)}",
            "language = 0",
            'weidu_artifact_id = "weidu-249-amd64"',
            'invocation_mode = "setup-name"',
        ]
        for row in component_rows:
            lines.extend(["", "[[components]]", f"id = {row.component_id}", f"name = {_q(row.component)}"])
            if catalog == "BARDICWONDERS" and row.component_id == 1008:
                lines.append('prompts = [{ expected_output = "Please select 1 or 2 and press Enter.", answer = { kind = "literal", value = { kind = "integer", value = 2 } } }]')
        (destination / "mods" / f"{mod_id}.toml").write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")


def _update_release_records(destination: Path, commit: str) -> None:
    limitations_path = destination / "releases/v0.1.0-alpha.1/known-limitations.toml"
    limitations = limitations_path.read_text(encoding="utf-8")
    for feature_id in ["mod:evandra", "feature:artisanskitpack-npc:component-99001", "feature:chriz-bg-modpack:component-430"]:
        pattern = re.compile(rf'(?ms)^\[\[omissions\]\]\nfeature_id = {re.escape(_q(feature_id))}\n.*?(?=^\[\[omissions\]\]|\Z)')
        limitations, count = pattern.subn("", limitations)
        if count != 1:
            raise ValueError(f"expected one stale limitation for {feature_id}, found {count}")
    limitations = limitations.replace(
        '  "chriz-sod-remix-bg2",',
        '  "bardicwonders-dialogue-patch-bg2",\n  "chriz-sod-remix-bg2",',
    )
    limitations = limitations.replace(
        '  "spell-rev-npc-spellbooks-bg2",',
        '  "cdtweaks-spell-save-penalties-bg2",\n  "spell-rev-npc-spellbooks-bg2",',
    )
    limitations = limitations.replace(
        'reason = "The Branwen Spiritual Hammer repair is not enabled in the frozen alpha recipe."\n'
        'user_facing_limitation = "Branwen\'s Spell Revisions Spiritual Hammer repair is unavailable in this alpha."',
        'reason = "The maintained repair is selected conditionally and stays inactive unless the optional Branwen component is selected."\n'
        'user_facing_limitation = "Branwen\'s Spell Revisions Spiritual Hammer repair is selected only when Branwen is selected."',
    )
    limitations += """
[[omissions]]
feature_id = "feature:bardicwonders:component-1006"
approved_by = "Christopher"
approved_on = "2026-09-05"
reason = "Darkbloom remains deferred because it is unavailable with the selected Spell Revisions setup."
user_facing_limitation = "The Darkbloom kit is unavailable while Spell Revisions is selected."
"""
    limitations_path.write_text(limitations, encoding="utf-8", newline="\n")

    acceptance_path = destination / "releases/v0.1.0-alpha.1/acceptance.toml"
    acceptance = acceptance_path.read_text(encoding="utf-8").rstrip()
    for run_id in [
        "evandra-core-bg2",
        "evandra-crossmod-bg2",
        "bardicwonders-bg2",
        "bardicwonders-garrick-bg2",
        "iwdification-bg2",
        "cdtweaks-bg2",
        "cdtweaks-spell-save-penalties-bg2",
        "bardicwonders-dialogue-patch-bg2",
    ]:
        acceptance += f"""

[[evidence]]
subject_kind = "run"
subject_id = {_q(run_id)}
kind = "static-test"
repository = "Chrizhermann/chriz-bg-collection"
commit = {_q(commit)}
test_artifact = "tools/tests/test_curated_full_recipe.py"
date = "2026-09-05"
status = "accepted"
scope = "Focused static curation, source, component, prompt, and order assertions only; not live game acceptance."
"""
    acceptance_path.write_text(acceptance + "\n", encoding="utf-8", newline="\n")


def _effective_features(collection: dict, preset: dict) -> dict[str, bool]:
    features = collection["features"]
    desired = {
        feature["id"]: preset.get(feature["id"], "off") in {"on", "true"}
        for feature in features
    }
    selected = desired.copy()
    for _ in range(len(features) + 1):
        previous = selected.copy()
        for feature in features:
            requested = True if feature["decision"] == "mandatory" else desired[feature["id"]]
            parent = feature.get("parent")
            requirements = feature.get("requires", [])
            selected[feature["id"]] = (
                requested
                and feature["readiness"] != "blocked"
                and (parent is None or previous.get(parent, False))
                and all(previous.get(required, False) for required in requirements)
            )
        if selected == previous:
            break
    base = selected.copy()
    effective = base.copy()
    for _ in range(len(features) + 1):
        previous = effective.copy()
        for feature in features:
            value = base[feature["id"]]
            if value:
                parent = feature.get("parent")
                value = parent is None or previous.get(parent, False)
                value = value and all(previous.get(required, False) for required in feature.get("requires", []))
                value = value and all(not base.get(conflict["feature_id"], False) for conflict in feature.get("conflicts", []))
            effective[feature["id"]] = value
        if effective == previous:
            break
    return effective


def _write_reconciliation(root: Path, destination: Path) -> None:
    rows = load_catalogs(root / "docs/curation/components")
    curation_map = load_curation_map(root / "manifest/curation-map.toml")
    target_by_row = _target_rows(curation_map)
    collection = tomllib.loads((destination / "collection.toml").read_text(encoding="utf-8"))
    preset = tomllib.loads((destination / "presets/chris-recommended.toml").read_text(encoding="utf-8"))["selections"]
    effective = _effective_features(collection, preset)
    features = {feature["id"]: feature for feature in collection["features"]}
    runs = {run["run_id"]: run for run in collection["runs"]}
    mods = {path.stem: tomllib.loads(path.read_text(encoding="utf-8")) for path in (destination / "mods").glob("*.toml")}
    artifacts = {path.stem: tomllib.loads(path.read_text(encoding="utf-8")) for path in (destination / "artifacts").glob("*.toml")}

    lines = [
        "# Curated full recipe reconciliation",
        "",
        "Generated from `docs/curation/components/` plus `manifest/curation-map.toml`; no WeiDU log is used as an inclusion source.",
        "The source column records the pinned installer identity, while the status is the engine-equivalent resolved-selection outcome for the recommended preset.",
        "",
        "| Curated row | Decision | Semantic target | Status | Current source/component identity | Reason |",
        "|---|---|---|---|---|---|",
    ]
    counts = Counter()
    for row in rows:
        if row.decision not in {Decision.DEFAULT, Decision.MANDATORY}:
            continue
        key = RowKey(row.catalog, row.component_id)
        target = target_by_row[key]
        reason = ""
        identity = "No executable source selected"
        if target.kind == "omission":
            status = "deferred"
            reason = target.reason or "Authored omission"
        else:
            feature = features[target.target_id]
            if effective[target.target_id]:
                status = "selected for installation"
            elif feature["readiness"] == "blocked":
                status = "deferred"
                reason = feature.get("unavailable_reason", "Blocked by authored readiness")
            else:
                status = "conditional inactive"
                parent = feature.get("parent")
                missing = [required for required in feature.get("requires", []) if not effective.get(required, False)]
                conflicts = [conflict["feature_id"] for conflict in feature.get("conflicts", []) if effective.get(conflict["feature_id"], False)]
                if parent and not effective.get(parent, False):
                    reason = f"Parent {parent} is inactive."
                elif missing:
                    reason = "Requires " + ", ".join(missing) + "."
                elif conflicts:
                    reason = "Conflicts with " + ", ".join(conflicts) + "."
                else:
                    reason = "Inactive under the recommended selection."
            for component in feature.get("components", []):
                if component["component"] != row.component_id and len(target.rows) > 1:
                    continue
                mod = mods[runs[component["run_id"]]["mod_id"]]
                artifact = artifacts[mod["artifact_id"]]
                identity = f"{mod['id']} → {artifact['id']} {artifact['version']} ({artifact['source']['reference']}), component {component['component']}"
                break
        counts[status] += 1
        clean_reason = reason.replace("|", "\\|")
        clean_identity = identity.replace("|", "\\|")
        lines.append(f"| `{key}` | {row.decision.value} | `{target.target_id}` | {status} | {clean_identity} | {clean_reason} |")
    lines[5:5] = [
        f"Engine-equivalent resolved-selection summary: {sum(counts.values())} default/mandatory rows — " + ", ".join(f"{name}: {count}" for name, count in sorted(counts.items())) + ".",
        "",
    ]
    (destination / "RECONCILIATION.md").write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")


def build_recipe(root: Path, destination: Path, commit: str) -> None:
    root = root.resolve()
    destination = destination.resolve()
    destination.mkdir(parents=True, exist_ok=True)
    for directory in ["artifacts", "game-builds", "mods", "releases"]:
        shutil.copytree(root / "manifest" / directory, destination / directory, dirs_exist_ok=True)
    (destination / "artifacts/chriz-sod-remix-0.6.4.toml").unlink(missing_ok=True)
    shutil.copytree(root / "manifest/presets", destination / "presets", dirs_exist_ok=True)
    shutil.copy2(
        root / "recipes/creator-full-current/artifacts/creator-full-private-extras-20260902.toml",
        destination / "artifacts/creator-full-private-extras-20260902.toml",
    )
    private_artifact_path = destination / "artifacts/creator-full-private-extras-20260902.toml"
    private_artifact = private_artifact_path.read_text(encoding="utf-8")
    private_artifact = re.sub(
        r'publish_roots = \[[^\n]*\]',
        'publish_roots = ["evandra"]',
        private_artifact,
        count=1,
    )
    private_artifact = re.sub(
        r'tp2_paths = \[[^\n]*\]',
        'tp2_paths = ["evandra/setup-evandra.tp2"]',
        private_artifact,
        count=1,
    )
    private_artifact = private_artifact.replace(
        'license = "Manual local archive containing 49 reference installers; never distribute or bundle"',
        'license = "Manual local archive used only to acquire the independently curated Evandra v2.2 installer; never distribute or bundle"',
    )
    private_artifact_path.write_text(private_artifact, encoding="utf-8", newline="\n")
    for filename, text in ARTIFACTS.items():
        (destination / "artifacts" / filename).write_text(text, encoding="utf-8", newline="\n")

    base_collection = (root / "manifest/collection.toml").read_text(encoding="utf-8").replace("\r\n", "\n")
    base_collection = _preserve_native_run_orders(base_collection)
    base = tomllib.loads(base_collection)
    rows = load_catalogs(root / "docs/curation/components")
    curation_map = load_curation_map(root / "manifest/curation-map.toml")
    target_rows = {row for target in curation_map.targets if target.kind == "feature" for row in target.rows}
    bg1_components = [row.component_id for row in rows if row.catalog == "BG1NPC" and RowKey(row.catalog, row.component_id) in target_rows]
    unknown_bg1_components = set(bg1_components) - set(BG1NPC_NATIVE_ORDER)
    if unknown_bg1_components:
        raise ValueError(f"BG1NPC components lack verified native order: {sorted(unknown_bg1_components)}")
    bg1_components = [component for component in BG1NPC_NATIVE_ORDER if component in bg1_components]
    base_collection = re.sub(
        r'(?ms)(run_id = "bg1npc-bg1"\nmod_id = "bg1npc"\nphase = "bg1-preparation"\n)components = \[[^\]]*\]',
        rf'\1components = [{", ".join(map(str, bg1_components))}]',
        base_collection,
        count=1,
    )
    cdt_components = [row.component_id for row in rows if row.catalog == "CDTWEAKS" and RowKey(row.catalog, row.component_id) in target_rows and row.component_id != 2312]
    run_blocks = dict(RUN_BLOCKS)
    run_blocks["before-randomiser"] = run_blocks["before-randomiser"].replace("components = []", f"components = [{', '.join(map(str, cdt_components))}]")
    insertions = [
        ('[[runs]]\nrun_id = "xan-bg2"', run_blocks["before-xan"]),
        ('[[runs]]\nrun_id = "ascension-bg2"', run_blocks["before-ascension"]),
        ('[[runs]]\nrun_id = "artisanskitpack-main-bg2"', run_blocks["before-artisan"]),
        ('[[runs]]\nrun_id = "artisanskitpack-npc-bg2"', run_blocks["before-artisan-npc"]),
        ('[[runs]]\nrun_id = "crossmodbg2-bg2"', run_blocks["before-crossmod"]),
        ('[[runs]]\nrun_id = "randomiser-bg2"', run_blocks["before-randomiser"]),
        ('[[runs]]\nrun_id = "chriz-sod-remix-bg2"', run_blocks["after-eet-end"]),
        ('[[runs]]\nrun_id = "spell-rev-npc-spellbooks-bg2"', run_blocks["late-spell-scan"]),
    ]
    for marker, block in insertions:
        if base_collection.count(marker) != 1:
            raise ValueError(f"collection marker is not unique: {marker}")
        base_collection = base_collection.replace(marker, block + marker, 1)

    # RANDOMISER.md explicitly keeps the fresh-stack run after SCS. Leave the
    # earlier Tweaks/IWDification insertions in place; move only Randomiser's run.
    randomiser_runs = list(re.finditer(
        r'(?ms)^\[\[runs\]\]\nrun_id = "randomiser-bg2"\n.*?(?=^\[\[runs\]\])',
        base_collection,
    ))
    late_kit_marker = '[[runs]]\nrun_id = "artisanskitpack-tweak-late-bg2"'
    if len(randomiser_runs) != 1 or base_collection.count(late_kit_marker) != 1:
        raise ValueError("Randomiser's reviewed post-SCS placement is ambiguous")
    randomiser_run = randomiser_runs[0].group(0)
    base_collection = base_collection.replace(randomiser_run, "", 1)
    base_collection = base_collection.replace(late_kit_marker, randomiser_run + late_kit_marker, 1)

    base_collection = _replace_feature(
        base_collection,
        "mod:evandra",
        {
            "description": _q("Install page-gated Evandra v2.2 from the exact user-supplied archive; the collection never redistributes it."),
            "readiness": '"ready"',
            "unavailable_reason": None,
        },
    )
    base_collection = _replace_feature(
        base_collection,
        "feature:hiddengameplayoptions:component-38",
        {"readiness": '"ready"', "unavailable_reason": None},
        ['requires = ["feature:cdtweaks:component-3354"]'],
    )
    base_collection = _replace_feature(
        base_collection,
        "feature:artisanskitpack-npc:component-99001",
        {"readiness": '"ready"', "unavailable_reason": None},
        ['requires = ["feature:bardicwonders:component-1008"]'],
    )
    base_collection = _replace_feature(
        base_collection,
        "feature:chriz-bg-modpack:component-400",
        {"readiness": '"ready"', "unavailable_reason": None},
        ['requires = ["feature:branwen:component-0", "mod:spell-rev"]'],
    )
    base_collection = _replace_feature(
        base_collection,
        "feature:chriz-bg-modpack:component-430",
        {"readiness": '"ready"', "unavailable_reason": None},
        ['requires = ["feature:cdtweaks:component-2170"]'],
    )
    base_collection = base_collection.rstrip() + "\n\n" + _feature_blocks(root, base)
    (destination / "collection.toml").write_text(base_collection, encoding="utf-8", newline="\n")

    _write_mods(root, destination)
    preset_path = destination / "presets/chris-recommended.toml"
    preset = preset_path.read_text(encoding="utf-8").rstrip()
    collection = tomllib.loads(base_collection)
    existing_selections = tomllib.loads(preset_path.read_text(encoding="utf-8"))["selections"]
    for feature in collection["features"]:
        if feature["decision"] == "default" and feature["id"] not in existing_selections:
            preset += f"\n{_q(feature['id'])} = \"on\""
    preset_path.write_text(preset + "\n", encoding="utf-8", newline="\n")
    (destination / "release.json").write_text(
        json.dumps({"version": RECIPE_VERSION, "label": RECIPE_LABEL}, indent=2) + "\n",
        encoding="utf-8",
        newline="\n",
    )
    _update_release_records(destination, commit)
    _write_reconciliation(root, destination)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Build the curation-derived full recipe")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--commit", required=True, help="40-hex commit recorded for focused static evidence")
    args = parser.parse_args(argv)
    try:
        if not re.fullmatch(r"[0-9a-fA-F]{40}", args.commit):
            raise ValueError("--commit must be a 40-hex Git commit")
        build_recipe(args.root, args.output, args.commit.lower())
    except (OSError, ValueError) as error:
        print(f"curated full recipe generation failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
