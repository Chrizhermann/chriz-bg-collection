"""Deterministic player-facing context for curated recipe features.

The component catalogs and curation map remain the authority.  This module only
projects their source/group vocabulary into recipe metadata and disambiguates
short option labels; it does not infer compatibility or selection policy.
"""

from __future__ import annotations

from dataclasses import dataclass
import json
import re

from tools.curation_audit import CurationMap, CatalogRow, RowKey


CATEGORY_LABELS = {
    "bg1-content": "BG1 content",
    "bg1-fixes": "BG1 fixes",
    "bg1-npcs": "BG1 NPCs",
}

# The catalog option is intentionally terse because its prompt supplies the
# missing noun.  Use the same authored meaning in a standalone feature title.
TITLE_OVERRIDES = {
    RowKey("CDTWEAKS", 1142): "Potions require identification",
}

DESCRIPTION_OVERRIDES = {
    RowKey("CDTWEAKS", 1142): "Potions need identification; gems are unchanged.",
    RowKey("CDTWEAKS", 1160): "Allows multiple strongholds without class restrictions.",
    RowKey("CDTWEAKS", 1161): "Allows multiple strongholds while retaining class restrictions.",
}

for _component, _kind, _size in (
    (3080, "ammo", None),
    (3081, "ammo", 40),
    (3082, "ammo", 80),
    (3083, "ammo", 120),
    (3090, "jewelry, gems, and miscellaneous items", None),
    (3091, "jewelry, gems, and miscellaneous items", 40),
    (3092, "jewelry, gems, and miscellaneous items", 80),
    (3093, "jewelry, gems, and miscellaneous items", 120),
    (3100, "potions", None),
    (3101, "potions", 40),
    (3102, "potions", 80),
    (3103, "potions", 120),
    (3110, "scrolls", None),
    (3111, "scrolls", 40),
    (3112, "scrolls", 80),
    (3113, "scrolls", 120),
):
    DESCRIPTION_OVERRIDES[RowKey("CDTWEAKS", _component)] = (
        f"Allows unlimited stacking of {_kind} in each inventory slot."
        if _size is None
        else f"Allows up to {_size} {_kind} per inventory slot."
    )

SOURCE_LABELS = {
    "BG1NPC": "The BG1 NPC Project",
    "BGGO": "Baldur's Gate Graphical Overhaul",
    "BRANWEN": "Branwen for BGII",
    "BUBB_SPELL_MENU_EXTENDED": "Bubb's Spell Menu Extended",
    "CDTWEAKS": "The Tweaks Anthology",
    "CROSSMODBG2": "Crossmod Banter Pack",
    "EEEXREMOTE": "EEex Remote Console",
    "HQ_SOUNDCLIPS_BG2EE": "HQ SoundClips for BG2EE",
    "IEPBANTERS": "IEP Extended Banters",
    "IWDIFICATION": "IWDification",
    "RR": "Rogue Rebalancing",
    "SIRENE_BG2": "Sirene for BG2",
    "UB": "Unfinished Business",
    "YESLICKNPC": "Yeslick NPC for BGII",
}


@dataclass(frozen=True)
class FeaturePresentation:
    source_label: str
    group_label: str
    choice_group: str | None = None
    title: str | None = None
    description: str | None = None


def _clean(value: str) -> str:
    return re.sub(r"\s+", " ", value.replace("�", " — ")).strip()


def _category_label(category: str) -> str:
    return CATEGORY_LABELS.get(category, category.replace("-", " ").title())


def build_feature_presentations(
    collection: dict,
    rows: list[CatalogRow],
    curation_map: CurationMap,
) -> dict[str, FeaturePresentation]:
    """Return authored presentation context for every recipe feature."""

    row_by_key = {RowKey(row.catalog, row.component_id): row for row in rows}
    target_by_id = {target.target_id: target for target in curation_map.targets}
    features = {feature["id"]: feature for feature in collection["features"]}
    parent_titles = {feature_id: _clean(feature["title"]) for feature_id, feature in features.items()}

    # Only genuine alternatives share a choice group.  Several authored groups
    # contain one retained option solely to record an optional-none/default
    # policy; those are not rendered as mutually exclusive alternatives.
    choice_for_feature: dict[str, str] = {}
    target_for_row = {
        row: target.target_id
        for target in curation_map.targets
        if target.kind == "feature"
        for row in target.rows
    }
    for group in curation_map.choice_groups:
        option_ids = list(dict.fromkeys(target_for_row[row] for row in group.options if row in target_for_row))
        if len(option_ids) > 1:
            for feature_id in option_ids:
                choice_for_feature[feature_id] = group.group_id

    result: dict[str, FeaturePresentation] = {}
    for feature_id, feature in features.items():
        target = target_by_id.get(feature_id)
        target_rows = [row_by_key[row] for row in target.rows] if target and target.kind == "feature" else []
        parent = feature.get("parent")
        source_label = parent_titles.get(parent, _clean(feature["title"]))
        group_label = _category_label(feature["category"])
        title = _clean(feature["title"])
        description = _clean(feature["description"])

        if target_rows:
            source_label = parent_titles.get(parent, SOURCE_LABELS.get(target_rows[0].catalog, source_label))
            groups = list(dict.fromkeys(_clean(row.group) for row in target_rows if row.group.strip()))
            subgroups = list(dict.fromkeys(_clean(row.subgroup) for row in target_rows if row.subgroup.strip()))
            if subgroups:
                group_label = " / ".join(subgroups)
            elif groups:
                group_label = " / ".join(groups)

            if len(target_rows) == 1 and title == description:
                row = target_rows[0]
                contextual_title = TITLE_OVERRIDES.get(RowKey(row.catalog, row.component_id))
                if contextual_title:
                    title = contextual_title
                description = DESCRIPTION_OVERRIDES.get(
                    RowKey(row.catalog, row.component_id), description
                )

        result[feature_id] = FeaturePresentation(
            source_label=source_label,
            group_label=group_label,
            choice_group=choice_for_feature.get(feature_id),
            title=title if title != feature["title"] else None,
            description=description if description != feature["description"] else None,
        )
    return result


def apply_feature_presentations(collection_text: str, presentations: dict[str, FeaturePresentation]) -> str:
    """Add presentation fields without reserializing or reordering the recipe."""

    quoted = lambda value: json.dumps(value, ensure_ascii=False)
    pattern = re.compile(r"(?ms)^\[\[features\]\]\n.*?(?=^\[\[features\]\]|^\[\[runs\]\]|\Z)")

    def update(match: re.Match[str]) -> str:
        block = match.group(0).rstrip()
        parsed_id = re.search(r'^id = "([^"]+)"$', block, re.MULTILINE)
        if parsed_id is None or parsed_id.group(1) not in presentations:
            return match.group(0)
        presentation = presentations[parsed_id.group(1)]
        replacements = {"title": presentation.title, "description": presentation.description}
        for key, value in replacements.items():
            if value is not None:
                block = re.sub(rf"(?m)^{key} = .*?$", f"{key} = {quoted(value)}", block, count=1)
        fields = [
            f"source_label = {quoted(presentation.source_label)}",
            f"group_label = {quoted(presentation.group_label)}",
        ]
        if presentation.choice_group:
            fields.append(f"choice_group = {quoted(presentation.choice_group)}")
        lines = block.splitlines()
        insert_at = next((i for i, line in enumerate(lines) if line.startswith("decision = ")), len(lines))
        lines[insert_at:insert_at] = fields
        return "\n".join(lines) + "\n\n"

    return pattern.sub(update, collection_text).rstrip() + "\n"
