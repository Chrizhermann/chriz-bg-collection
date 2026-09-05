"""Generate the public selected-component/source-credit list for the website."""

from __future__ import annotations

import argparse
import json
import sys
import tomllib
from pathlib import Path

if __package__ in {None, ""}:
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from tools.curated_full_recipe import _effective_features


def _load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def build_public_component_credits(recipe: Path, app_package: Path) -> dict:
    collection = _load(recipe / "collection.toml")
    preset = _load(recipe / "presets/chris-recommended.toml")["selections"]
    effective = _effective_features(collection, preset)

    selected_by_run: dict[str, set[int]] = {}
    for feature in collection["features"]:
        if not effective[feature["id"]]:
            continue
        for component in feature.get("components", []):
            selected_by_run.setdefault(component["run_id"], set()).add(component["component"])

    mods = {_load(path)["id"]: _load(path) for path in (recipe / "mods").glob("*.toml")}
    artifacts = {
        _load(path)["id"]: _load(path) for path in (recipe / "artifacts").glob("*.toml")
    }
    public_mods: list[dict] = []
    public_by_mod: dict[str, dict] = {}
    component_count = 0

    for run in collection["runs"]:
        selected = selected_by_run.get(run["run_id"])
        if not selected:
            continue
        mod = mods[run["mod_id"]]
        artifact = artifacts[mod["artifact_id"]]
        names = {component["id"]: component["name"] for component in mod["components"]}
        components = [
            {"id": component_id, "name": names[component_id]}
            for component_id in run["components"]
            if component_id in selected
        ]
        if len(components) != len(selected):
            missing = sorted(selected - {component["id"] for component in components})
            raise ValueError(f"selected components absent from run {run['run_id']}: {missing}")

        entry = public_by_mod.get(mod["id"])
        if entry is None:
            homepage = artifact.get("provenance", {}).get("homepage")
            if not homepage or not homepage.startswith(("https://", "http://")):
                raise ValueError(f"mod {mod['id']} lacks a public homepage")
            entry = {
                "id": mod["id"],
                "name": mod["name"],
                "version": artifact["version"],
                "homepage": homepage,
                "runs": [],
            }
            public_by_mod[mod["id"]] = entry
            public_mods.append(entry)
        entry["runs"].append({"id": run["run_id"], "components": components})
        component_count += len(components)

    app_version = json.loads(app_package.read_text(encoding="utf-8"))["version"]
    recipe_version = json.loads((recipe / "release.json").read_text(encoding="utf-8"))["version"]
    return {
        "schemaVersion": 1,
        "applicationVersion": app_version,
        "recipeVersion": recipe_version,
        "presetId": "chris-recommended",
        "creditScope": "Official project/source links only; author credits are published separately.",
        "componentCount": component_count,
        "mods": public_mods,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--recipe", type=Path, required=True)
    parser.add_argument("--app-package", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = build_public_component_credits(args.recipe, args.app_package)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(result, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
