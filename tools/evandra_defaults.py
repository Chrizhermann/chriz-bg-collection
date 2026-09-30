"""Author and gate Evandra defaults without inventing a released Modpack pin.

The Modpack catalogue must come from a verified artifact before integration. These
checks prove recipe delivery and selection semantics, not archive or live acceptance.
"""

from __future__ import annotations

import argparse
import re
from pathlib import Path
import sys
import tomllib

if __package__ in (None, ""):
    # The packaging gate invokes this file directly, not with ``python -m``.
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))


EVANDRA_CORE = "feature:evandra:mandatory-components"
MODPACK_RUN = "chriz-bg-modpack-pre-continuity-bg2"
DEFAULT_COMPONENTS = (224, 225)
DEFAULT_FEATURE_IDS = tuple(f"feature:chriz-bg-modpack:component-{n}" for n in DEFAULT_COMPONENTS)
_LABELS = (
    ("Make Evandra a Sorcerer", "Use Evandra's curated Sorcerer build and starting spellbook."),
    ("Use Evandra's custom portrait", "Use Christopher's custom Evandra portrait."),
)


def _catalogue_supports_defaults(modpack: dict) -> bool:
    # These published archives predate this migration. Editing their catalogue
    # cannot make their immutable payloads contain the new components.
    artifact_id = modpack.get("artifact_id", "")
    if not artifact_id or re.fullmatch(r"chriz-bg-modpack-0\.2\.0-alpha\.[1-7]", artifact_id):
        return False
    ids = [component["id"] for component in modpack.get("components", [])]
    return modpack.get("id") == "chriz-bg-modpack" and all(ids.count(n) == 1 for n in DEFAULT_COMPONENTS)


def _expected_feature(component: int, title: str, description: str) -> dict:
    return {
        "id": f"feature:chriz-bg-modpack:component-{component}",
        "title": title,
        "description": description,
        "category": "npcs",
        "decision": "default",
        "readiness": "ready",
        "parent": "mod:chriz-bg-modpack",
        "requires": [EVANDRA_CORE],
        "components": [{"run_id": MODPACK_RUN, "component": component}],
    }


def integrate_evandra_defaults(collection_text: str, modpack: dict) -> str:
    """Add both defaults only when the pinned catalogue supports both components.

    This never changes artifact metadata or adds catalogue declarations. Missing
    content leaves the public snapshot intact; the publication check still fails.
    """
    if not _catalogue_supports_defaults(modpack):
        return collection_text
    collection = tomllib.loads(collection_text)
    features = {feature["id"]: feature for feature in collection["features"]}
    if not {"mod:evandra", EVANDRA_CORE, "mod:chriz-bg-modpack"} <= features.keys():
        raise ValueError("Evandra defaults require the existing Evandra core and Modpack choices")
    runs = [run for run in collection["runs"] if run["run_id"] == MODPACK_RUN]
    if len(runs) != 1 or runs[0]["mod_id"] != "chriz-bg-modpack" or runs[0]["phase"] != "main":
        raise ValueError("Evandra defaults require the main-phase pre-continuity Modpack run")
    components = runs[0]["components"]
    if components.count(223) != 1:
        raise ValueError("Evandra defaults need a native-order audit: component 223 is not unique")
    for run in collection["runs"]:
        if run["mod_id"] == "chriz-bg-modpack" and run["run_id"] != MODPACK_RUN:
            if set(DEFAULT_COMPONENTS) & set(run["components"]):
                raise ValueError("Evandra defaults already appear in another Modpack run")
    if any(components.count(n) > 1 for n in DEFAULT_COMPONENTS):
        raise ValueError("Evandra defaults repeat in the Modpack run")
    ordered = [n for n in components if n not in DEFAULT_COMPONENTS]
    index = ordered.index(223) + 1
    ordered[index:index] = DEFAULT_COMPONENTS
    pattern = rf'(?m)(^run_id = "{re.escape(MODPACK_RUN)}"\r?\nmod_id = "chriz-bg-modpack"\r?\nphase = "main"\r?\n)components = \[[^\]\r\n]*\]'
    collection_text, count = re.subn(
        pattern, lambda match: match[1] + f"components = {ordered}", collection_text,
    )
    if count != 1:
        raise ValueError("Evandra defaults cannot identify the authored Modpack run")
    for component, (title, description) in zip(DEFAULT_COMPONENTS, _LABELS):
        expected = _expected_feature(component, title, description)
        existing = features.get(expected["id"])
        if existing is not None:
            if any(existing.get(key) != expected[key] for key in ("decision", "readiness", "parent", "requires", "components")):
                raise ValueError(f"Evandra default has a conflicting definition: {expected['id']}")
            continue
        collection_text = collection_text.rstrip() + "\n\n" + "\n".join([
            "[[features]]",
            *(f'{key} = "{expected[key]}"' for key in ("id", "title", "description", "category", "decision", "readiness", "parent")),
            f'requires = ["{EVANDRA_CORE}"]',
            f'components = [{{ run_id = "{MODPACK_RUN}", component = {component} }}]',
        ]) + "\n"
    return collection_text


def check_evandra_defaults(collection: dict, modpack: dict, selections: dict) -> list[str]:
    """Return publication blockers for a recipe offering usable Evandra content."""
    features = {feature["id"]: feature for feature in collection["features"]}
    evandra = features.get("mod:evandra")
    if evandra is None or evandra.get("readiness") == "blocked":
        return []
    findings = []
    if not _catalogue_supports_defaults(modpack):
        findings.append("Evandra requires a verified Modpack artifact/catalogue containing 224 (Sorcerer) and 225 (custom portrait); the current pin does not supply both.")
    for component, (title, description) in zip(DEFAULT_COMPONENTS, _LABELS):
        expected = _expected_feature(component, title, description)
        feature = features.get(expected["id"])
        if feature is None:
            findings.append(f"Evandra default {expected['id']} is missing.")
            continue
        for key in ("decision", "readiness", "parent", "requires", "components"):
            if feature.get(key) != expected[key]:
                findings.append(f"Evandra default {feature['id']} has incorrect {key}; expected {expected[key]!r}.")
        if feature.get("requires_any") or feature.get("conflicts"):
            findings.append(f"Evandra default {feature['id']} must not depend on crossmod or the other Evandra option.")
        if selections.get(feature["id"]) not in ("on", "true", True):
            findings.append(f"Evandra default {feature['id']} must be checked in the authored recommended preset.")
    runs = collection["runs"]
    target_indices = [index for index, run in enumerate(runs) if run["run_id"] == MODPACK_RUN]
    if len(target_indices) != 1:
        findings.append("Evandra defaults require one pre-continuity Modpack run.")
    else:
        target_index = target_indices[0]
        target = runs[target_index]
        components = target["components"]
        if target["mod_id"] != "chriz-bg-modpack" or target["phase"] != "main":
            findings.append("Evandra defaults must run in the main-phase Modpack companion conversion slot.")
        start = components.index(223) if 223 in components else -1
        if start < 0 or components[start:start + 3] != [223, 224, 225]:
            findings.append("Evandra native component order must place 224 and 225 immediately after 223.")
        core_refs = features.get(EVANDRA_CORE, {}).get("components", [])
        core_indices = [index for index, run in enumerate(runs) if run["mod_id"] == "evandra"
                        and {"run_id": run["run_id"], "component": 0} in core_refs
                        and 0 in run["components"]]
        if len(core_indices) != 1 or core_indices[0] >= target_index:
            findings.append("Evandra defaults must follow the actual Evandra core component 0 run.")
    for component in DEFAULT_COMPONENTS:
        occurrences = sum(run["components"].count(component) for run in runs if run["mod_id"] == "chriz-bg-modpack")
        if occurrences != 1:
            findings.append(f"Evandra Modpack component {component} must occur in exactly one run (found {occurrences}).")
    if findings:
        return findings

    # Reuse the generator's evaluator; do not maintain another selection model.
    from tools.curated_full_recipe import _effective_features

    baseline = {**selections, "mod:evandra": "on", "mod:chriz-bg-modpack": "on"}
    enabled = _effective_features(collection, baseline)
    if not all(enabled.get(key, False) for key in DEFAULT_FEATURE_IDS):
        findings.append("Evandra defaults are not selected when Evandra and Modpack are selected.")
    skipped = _effective_features(collection, {**baseline, "mod:evandra": "off"})
    if any(skipped.get(key, False) for key in DEFAULT_FEATURE_IDS):
        findings.append("Skipping Evandra must also omit its Sorcerer and portrait components.")
    for disabled in ("feature:evandra:component-1", *DEFAULT_FEATURE_IDS):
        state = _effective_features(collection, {**baseline, disabled: "off"})
        if state.get(disabled, False) or any(state.get(key, False) != enabled.get(key, False) for key in DEFAULT_FEATURE_IDS if key != disabled):
            findings.append(f"Evandra choice {disabled} must be independently switchable.")
    return findings


def evandra_default_findings(recipe_root: Path) -> list[str]:
    """Read a recipe and return its Evandra publication blockers."""
    def read(relative: str) -> dict:
        return tomllib.loads((recipe_root / relative).read_text(encoding="utf-8"))

    collection = read("collection.toml")
    # Packaging also supports profiles that do not offer Evandra (and need not
    # contain a Modpack catalogue or our named recommended preset).
    evandra = next((feature for feature in collection.get("features", [])
                    if feature["id"] == "mod:evandra"), None)
    if evandra is None or evandra.get("readiness") == "blocked":
        return []
    modpack = read("mods/chriz-bg-modpack.toml")
    selections = read("presets/chris-recommended.toml")["selections"]
    return check_evandra_defaults(collection, modpack, selections)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Check required Evandra defaults before public packaging")
    parser.add_argument("--recipe", type=Path, required=True)
    args = parser.parse_args(argv)
    try:
        findings = evandra_default_findings(args.recipe.resolve())
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"Evandra recipe readiness could not be checked: {error}", file=sys.stderr)
        return 1
    if findings:
        print("Evandra recipe is not ready for public packaging:", file=sys.stderr)
        for finding in findings:
            print(f"- {finding}", file=sys.stderr)
        return 1
    print("Evandra recipe default contract passed; archive and gameplay acceptance are separate.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
