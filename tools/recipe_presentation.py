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
    RowKey("CDTWEAKS", 90): "Hide item-granted portrait icons",
    RowKey("CDTWEAKS", 171): "Replace existing unique item icons too",
    RowKey("CDTWEAKS", 240): "Black outlines for white spell icons",
    RowKey("CDTWEAKS", 241): "Gray outlines for white spell icons",
    RowKey("CDTWEAKS", 1035): "Open the first Cloakwood area early",
    RowKey("CDTWEAKS", 1036): "Open Cloakwood early, except the mines",
    RowKey("CDTWEAKS", 1140): "Gems and potions require identification",
    RowKey("CDTWEAKS", 1141): "Gems require identification",
    RowKey("CDTWEAKS", 1142): "Potions require identification",
    RowKey("CDTWEAKS", 2151): "Unrestricted magical protection item stacking",
    RowKey("CDTWEAKS", 2310): "High-level save penalties: arcane spells only",
    RowKey("CDTWEAKS", 2311): "High-level save penalties: divine spells only",
    RowKey("CDTWEAKS", 3030): "Guaranteed scroll learning",
    RowKey("CDTWEAKS", 3031): "Guaranteed scroll learning, no spellbook limits",
    RowKey("CDTWEAKS", 3070): "Store discounts for low reputation",
    RowKey("CDTWEAKS", 3071): "Ignore reputation: normal store prices",
    RowKey("CDTWEAKS", 3072): "Ignore reputation: 80% store prices",
    RowKey("CDTWEAKS", 3073): "Ignore reputation: 60% store prices",
    RowKey("CDTWEAKS", 3131): "Harmless traps and no ordinary locks",
    RowKey("CDTWEAKS", 3132): "Harmless traps, no alarms or ordinary locks",
    RowKey("CDTWEAKS", 3150): "Hide equipment spell-trap and reflection visuals",
    RowKey("CDTWEAKS", 3151): "Hide equipment blur, spell-trap and reflection visuals",
    RowKey("CDTWEAKS", 3191): "Disable nonhostile rest encounters",
    RowKey("CDTWEAKS", 3194): "Disable hostile rest encounters",
    RowKey("CDTWEAKS", 3195): "Half as many hostile rest encounters",
    RowKey("CDTWEAKS", 3196): "50% more hostile rest encounters",
    RowKey("CDTWEAKS", 3197): "Twice as many hostile rest encounters",
    RowKey("CDTWEAKS", 3198): "Four times as many hostile rest encounters",
    RowKey("CDTWEAKS", 3320): "Keep resale prices from falling with shop stock",
    RowKey("KLATU", 2150): "Use thief skills in armor",
}

DESCRIPTION_OVERRIDES = {
    RowKey("CDTWEAKS", 70): (
        "Uses Icewind Dale's casting visuals. Choose this or IWDification's equivalent, not both."
    ),
    RowKey("CDTWEAKS", 90): (
        "Hides portrait status icons added directly by equipped items; their bonuses remain active."
    ),
    RowKey("CDTWEAKS", 140): "Replaces Boo's squeak with a less grating sound.",
    RowKey("CDTWEAKS", 150): (
        "Removes labels such as '+1' from named unique items without changing their enchantment."
    ),
    RowKey("CDTWEAKS", 160): "Adds a glow to magical shields; their combat bonuses are unchanged.",
    RowKey("CDTWEAKS", 171): (
        "Uses Lava's icon replacements even for items that already have a distinct icon."
    ),
    RowKey("CDTWEAKS", 220): (
        "Makes inventory overlays for unusable items, unidentified items and learnable scrolls easier to distinguish."
    ),
    RowKey("CDTWEAKS", 240): "Adds black outlines to supported white spell icons for easier reading.",
    RowKey("CDTWEAKS", 241): "Adds gray outlines to supported white spell icons for easier reading.",
    RowKey("CDTWEAKS", 1030): (
        "Opens the Small Teeth Pass, North Forest and Forest of Tethir on the BG2 world map before Chapter 6."
    ),
    RowKey("CDTWEAKS", 1035): "Lets you visit the first Cloakwood area before finishing the Bandit Camp.",
    RowKey("CDTWEAKS", 1036): (
        "Lets you visit Cloakwood before finishing the Bandit Camp, but keeps the mines locked until later."
    ),
    RowKey("CDTWEAKS", 1100): "Reveals supported town maps on arrival instead of requiring street-by-street exploration.",
    RowKey("CDTWEAKS", 1101): "Keeps supported town maps unexplored on arrival so you discover them as you walk.",
    RowKey("CDTWEAKS", 1140): "Both gems and potions need identification before their identity is revealed.",
    RowKey("CDTWEAKS", 1141): "Gems need identification; potions are unchanged.",
    RowKey("CDTWEAKS", 1142): "Potions need identification; gems are unchanged.",
    RowKey("CDTWEAKS", 1160): "Allows multiple strongholds without class restrictions.",
    RowKey("CDTWEAKS", 1161): "Allows multiple strongholds while retaining class restrictions.",
    RowKey("CDTWEAKS", 2151): (
        "Allows magical armor, rings, cloaks and other protection items together without the usual stacking restriction."
    ),
    RowKey("CDTWEAKS", 2190): "Only mage and bard shopkeepers offer item identification.",
    RowKey("CDTWEAKS", 2191): "Shopkeepers can identify items only when their lore is high enough.",
    RowKey("CDTWEAKS", 2220): "Magically created weapons no longer add weight to the wielder's inventory.",
    RowKey("CDTWEAKS", 2310): (
        "Higher-level casters impose stronger saving-throw penalties with arcane spells only. "
        "Replaces the recommended arcane-and-divine option."
    ),
    RowKey("CDTWEAKS", 2311): (
        "Higher-level casters impose stronger saving-throw penalties with divine spells only. "
        "Replaces the recommended arcane-and-divine option."
    ),
    RowKey("CDTWEAKS", 3030): (
        "Prevents random failures when learning scrolls. Intelligence-based spellbook limits and kit restrictions remain."
    ),
    RowKey("CDTWEAKS", 3031): (
        "Prevents random scroll-learning failures and lifts intelligence-based spellbook limits. "
        "It does not grant spell slots or bypass opposed-school kit restrictions."
    ),
    RowKey("CDTWEAKS", 3060): "Silences the gather-your-party voice line; you still need the party together to travel.",
    RowKey("CDTWEAKS", 3070): (
        "Low reputation can earn shopping discounts too. Changes prices, not how reputation is gained."
    ),
    RowKey("CDTWEAKS", 3071): "Reputation no longer changes shopping prices; its price multiplier stays at 100%.",
    RowKey("CDTWEAKS", 3072): "Reputation no longer changes shopping prices; its price multiplier stays at 80%.",
    RowKey("CDTWEAKS", 3073): "Reputation no longer changes shopping prices; its price multiplier stays at 60%.",
    RowKey("CDTWEAKS", 3131): (
        "Cheat: removes harmful trap effects, ordinary locks and secret-door concealment. "
        "Alarm summons remain; story-locked doors can still require keys."
    ),
    RowKey("CDTWEAKS", 3132): (
        "Cheat: removes harmful trap effects, alarm summons, ordinary locks and secret-door concealment. "
        "Story-locked doors can still require keys."
    ),
    RowKey("CDTWEAKS", 3150): (
        "Hides spell-trap and reflection visuals from equipped items, without removing the protections themselves."
    ),
    RowKey("CDTWEAKS", 3151): (
        "Hides equipment blur, spell-trap and reflection visuals, without removing their separate combat bonuses."
    ),
    RowKey("CDTWEAKS", 3191): (
        "Disables nonhostile area rest encounters. Hostile ambushes and SCS provision requirements are unchanged."
    ),
    RowKey("CDTWEAKS", 3194): (
        "Disables hostile area rest encounters. SCS provision requirements and scripted story interruptions are unchanged."
    ),
    RowKey("CDTWEAKS", 3195): (
        "Halves the chance of hostile area rest encounters. Does not change SCS provision requirements."
    ),
    RowKey("CDTWEAKS", 3196): (
        "Multiplies hostile area rest-encounter chances by 1.5, up to the tweak's cap. "
        "Does not change SCS provision requirements."
    ),
    RowKey("CDTWEAKS", 3197): (
        "Doubles hostile area rest-encounter chances, up to the tweak's cap. "
        "Does not change SCS provision requirements."
    ),
    RowKey("CDTWEAKS", 3198): (
        "Quadruples hostile area rest-encounter chances, up to the tweak's cap. "
        "Does not change SCS provision requirements."
    ),
    RowKey("CDTWEAKS", 3200): "Gives selected zero-price items a sale value and lets arrow-buying shops buy bolts too.",
    RowKey("CDTWEAKS", 3205): (
        "Lets eligible shops buy more item categories, so you need fewer different merchants to sell your loot."
    ),
    RowKey("CDTWEAKS", 3230): "Lets Taerom make more ankheg armor after completing the first set.",
    RowKey("CDTWEAKS", 3320): (
        "Stores no longer lower their resale offers just because they already stock more copies of an item."
    ),
    RowKey("CDTWEAKS", 4140): (
        "Stops automatic assignment of Beamdog's Advanced AI scripts to party members. "
        "You can still assign scripts yourself; enemy AI is unchanged."
    ),
    RowKey("KLATU", 2150): (
        "No added skill penalties. Allows ordinary thieving and stealth in armor while "
        "preserving equipment permissions, spellcasting restrictions, and other kit abilities."
    ),
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
    "KLATU": "Klatu Tweaks and Fixes",
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
