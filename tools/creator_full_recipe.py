"""Generate the private creator-full executable recipe from read-only WeiDU logs.

The generated recipe contains only source metadata.  Third-party and local payloads that
are not already pinned by the public recipe are placed in one manual, user-supplied ZIP
outside the repository.
"""

from __future__ import annotations

import argparse
import hashlib
import re
import shutil
import sys
import tomllib
import zipfile
from dataclasses import dataclass
from pathlib import Path


LOG_LINE = re.compile(
    r"^~(?P<tp2>.+)~\s+#(?P<language>\d+)\s+#(?P<component>\d+)"
    r"(?:\s+//\s*(?P<name>.*))?$"
)
PSEUDO_INSTALLERS = {"__EXTRACT.TP2", "__IDS.TP2"}
PRIVATE_ARTIFACT_ID = "creator-full-private-extras-20260902"
PRIVATE_ARCHIVE_NAME = f"{PRIVATE_ARTIFACT_ID}.zip"
BUFFBOT_TP2 = "BUFFBOT/SETUP-BUFFBOT.TP2"
MODPACK_TP2 = "SETUP-CHRIZ-BG-MODPACK.TP2"
ARTISAN_NPC_TP2 = "ARTISANSKITPACK_NPC/ARTISANSKITPACK_NPC.TP2"
SIRENE_TP2 = "SIRENE_BG2/SIRENE_BG2.TP2"

# These live-install tail patches are superseded by maintained source packages in the
# creator replay. They remain visible in source-bg2.tsv and PARITY.md, but must not be
# staged or invoked a second time.
OBSOLETE_TAIL_INSTALLERS = {
    "AKCB_SHAPESHIFTER/SETUP-AKCB_SHAPESHIFTER.TP2",
    "SR_SUBSPELL_FIX/SR_SUBSPELL_FIX.TP2",
}
MAINTAINED_REPLACEMENT_TAIL_INSTALLERS = {
    "FADE_FT_FIX/SETUP-FADE_FT_FIX.TP2",
    "FADE_FT_PATCH/SETUP-FADE_FT_PATCH.TP2",
    "MAZZY_PROF_FIX/MAZZY_PROF_FIX.TP2",
    "VICONIA_MULTICLASS/VICONIA_MULTICLASS.TP2",
    "XAN_EK_FIX/XAN_EK_FIX.TP2",
    "KIVAN_QUEST_FIX/SETUP-KIVAN_QUEST_FIX.TP2",
    "YESLICK_KELDORN_DISPEL_FIX.TP2",
    "CBM_UAI_SCROLL/CBM_UAI_SCROLL.TP2",
    "SKIE_SKILL_FIX/SKIE_SKILL_FIX.TP2",
}
UNSAFE_TAIL_INSTALLERS = {
    # This old monolith assigns kits by numeric kit index and overlaps maintained,
    # semantic assignments. Its Safana and Aura branches have no current replacement.
    "NPC_KIT_CHANGES/NPC_KIT_CHANGES.TP2",
}
OMITTED_TAIL_INSTALLERS = (
    OBSOLETE_TAIL_INSTALLERS
    | MAINTAINED_REPLACEMENT_TAIL_INSTALLERS
    | UNSAFE_TAIL_INSTALLERS
)
PRIVATE_COMPANION_ROOTS = {
    # The root-level TP2 includes this sibling source tree.
    "SETUP-ABETTORHLAREBALANCE.TP2": ["abettor-hla-rebalance"],
}

# The current modpack artifact deliberately replaced the old v0.1 component surface.
# Replay the approved ready mandatory/default components plus component 430, which the
# reference selected and the full recipe supplies its Tweaks Anthology prerequisite for.
CURRENT_MODPACK_COMPONENTS = [
    (110, "Make Fade a Fighter/Thief"),
    (130, "Protect Kivan's sea-elf quest dialogue"),
    (140, "Correct Mazzy's Divine Champion proficiencies"),
    (170, "Complete Xan's Eldritch Knight conversion"),
    (190, "Make Sarah an Archer"),
    (192, "Make Viconia a Cleric/Thief"),
    (193, "Make Shar-Teel a Wizard Slayer"),
    (194, "Make Kagain a Dwarven Defender"),
    (195, "Make Skie a Swashbuckler"),
    (196, "Make Faldorn an Avenger"),
    (197, "Teach Dynaheir Haste"),
    (198, "Make Kivan an Archer"),
    (410, "Correct Yeslick and Keldorn's dispels"),
    (430, "Give Use Any Item scrolls a fair caster level"),
    (440, "Repair Ascension's upgraded Slayer forms"),
    (450, "Repair SCS shapechange forms"),
]

# Preserve the source-selected assignments and add the approved semantic replacements
# for the unsafe NPC_KIT_CHANGES branches. Safana-to-Abettor and Aura-to-Bard remain
# explicit gaps rather than reviving the numeric-kit monolith.
CURRENT_ARTISAN_NPC_COMPONENTS = [
    (1101, "Give Khalid the Vanguard Kit"),
    (2001, "Rashemi Berserker Ranger Kit for Minsc"),
    (3101, "Give Ajantis the Divine Champion Kit"),
    (3102, "Make Mazzy a Divine Champion"),
    (5102, "Red Wizard Mage Kit for Edwin"),
    (7102, "Make Imoen a Trickster"),
    (7104, "Give Hexxat the Invisible Blade Kit"),
    (21001, "Give Jan the Arcane Trickster Kit"),
    (20002, "Make Xan an Eldritch Knight"),
]
CURRENT_SIRENE_COMPONENTS = [
    (0, "Sirene NPC for BG2:EE"),
    (2, "Alternate portrait: Sirick, Light Armor"),
    (5, "True Paladin"),
]


@dataclass(frozen=True)
class Entry:
    tp2: str
    language: int
    component: int
    name: str


@dataclass(frozen=True)
class OrderedRun:
    run_id: str
    mod_id: str
    tp2: str
    language: int
    phase: str
    components: list[int]


def normalize_tp2(value: str) -> str:
    return value.replace("\\", "/").strip("/").upper()


def parse_weidu_log(path: Path) -> list[Entry]:
    entries: list[Entry] = []
    for line in path.read_text(encoding="utf-8-sig", errors="replace").splitlines():
        match = LOG_LINE.match(line.strip())
        if match is None:
            continue
        entries.append(
            Entry(
                match.group("tp2").replace("\\", "/"),
                int(match.group("language")),
                int(match.group("component")),
                match.group("name") or "Unlabelled component",
            )
        )
    return entries


def slug_for_tp2(tp2: str) -> str:
    stem = Path(tp2.replace("\\", "/")).stem.lower()
    stem = re.sub(r"^setup-", "", stem)
    slug = re.sub(r"[^a-z0-9]+", "-", stem).strip("-")
    return slug or "installer"


def private_publish_roots(tp2_paths: list[str]) -> list[str]:
    roots: set[str] = set()
    for tp2 in tp2_paths:
        normalized = tp2.replace("\\", "/").strip("/")
        parts = normalized.split("/")
        roots.add(parts[0] if len(parts) > 1 else normalized)
        roots.update(PRIVATE_COMPANION_ROOTS.get(normalize_tp2(normalized), []))
    return sorted(roots, key=str.casefold)


def _group_entries(entries: list[Entry]) -> list[list[Entry]]:
    groups: list[list[Entry]] = []
    for entry in entries:
        if groups and normalize_tp2(groups[-1][0].tp2) == normalize_tp2(entry.tp2):
            groups[-1].append(entry)
        else:
            groups.append([entry])
    return groups


def apply_replay_policy(bg2_entries: list[Entry]) -> tuple[list[Entry], list[Entry]]:
    """Apply reviewed maintained-source substitutions before allocating artifacts."""
    omissions: list[Entry] = []
    result: list[Entry] = []
    modpack_added = False
    artisan_npc_added = False
    sirene_added = False

    for entry in bg2_entries:
        normalized = normalize_tp2(entry.tp2)
        if normalized in PSEUDO_INSTALLERS or normalized in OMITTED_TAIL_INSTALLERS:
            omissions.append(entry)
            continue
        if normalized == BUFFBOT_TP2:
            # BuffBot is re-added once, at the absolute end, below.
            continue
        if normalized == MODPACK_TP2:
            if entry.component == 600:
                # v0.2.0-alpha.1 intentionally has no component 600 and has no mapped
                # equivalent.  Keep the gap explicit instead of asking WeiDU for a
                # removed component or installing an older copy of this mod twice.
                omissions.append(entry)
            if not modpack_added:
                result.extend(
                    Entry(entry.tp2, entry.language, component, name)
                    for component, name in CURRENT_MODPACK_COMPONENTS
                )
                modpack_added = True
            continue
        if normalized == ARTISAN_NPC_TP2:
            if not artisan_npc_added:
                result.extend(
                    Entry(entry.tp2, entry.language, component, name)
                    for component, name in CURRENT_ARTISAN_NPC_COMPONENTS
                )
                artisan_npc_added = True
            continue
        if normalized == SIRENE_TP2:
            if entry.component == 6:
                omissions.append(entry)
            if not sirene_added:
                result.extend(
                    Entry(entry.tp2, entry.language, component, name)
                    for component, name in CURRENT_SIRENE_COMPONENTS
                )
                sirene_added = True
            continue
        result.append(entry)

    # Preserve the reference EEex 1.2 components and add its launch-required LuaJIT
    # component 8 when the source log has only components 0 through 7.
    for index, entry in enumerate(result):
        if normalize_tp2(entry.tp2) == "EEEX/EEEX.TP2":
            eeex_end = index + 1
            while (
                eeex_end < len(result)
                and normalize_tp2(result[eeex_end].tp2) == "EEEX/EEEX.TP2"
            ):
                eeex_end += 1
            if not any(item.component == 8 for item in result[index:eeex_end]):
                result.insert(
                    eeex_end,
                    Entry(entry.tp2, entry.language, 8, "Experimental - Use LuaJIT"),
                )
            break

    result.extend(
        [
            Entry(BUFFBOT_TP2, 0, 1, "EEex LuaJIT activation safeguard"),
            Entry(BUFFBOT_TP2, 0, 0, "BuffBot: In-Game Buff Automation"),
        ]
    )
    return result, omissions


def build_ordered_runs(
    bg1_entries: list[Entry],
    bg2_entries: list[Entry],
    mod_ids: dict[str, str],
) -> tuple[list[OrderedRun], list[Entry]]:
    """Build split, ordered runs after applying the reviewed replay policy."""
    bg2_entries, omissions = apply_replay_policy(bg2_entries)

    runs: list[OrderedRun] = []
    run_counts: dict[str, int] = {}

    def append(groups: list[list[Entry]], fixed_phase: str | None = None) -> None:
        seen_eet = False
        seen_eet_end = False
        for group in groups:
            normalized = normalize_tp2(group[0].tp2)
            if fixed_phase is not None:
                phase = fixed_phase
            elif normalized == "EET/EET.TP2":
                phase = "eet-initialization"
                seen_eet = True
            elif normalized == "EET_END/EET_END.TP2":
                phase = "eet-finalization"
                seen_eet_end = True
            elif seen_eet_end:
                phase = "post-eet-end"
            elif seen_eet:
                phase = "main"
            else:
                phase = "bg2-preparation"

            mod_id = mod_ids[normalized]
            target = "bg1" if phase == "bg1-preparation" else "bg2"
            base = f"{mod_id}-{target}"
            run_counts[base] = run_counts.get(base, 0) + 1
            suffix = "" if run_counts[base] == 1 else f"-{run_counts[base]}"
            runs.append(
                OrderedRun(
                    run_id=f"{base}{suffix}",
                    mod_id=mod_id,
                    tp2=group[0].tp2,
                    language=group[0].language,
                    phase=phase,
                    components=[entry.component for entry in group],
                )
            )

    append(_group_entries(bg1_entries), "bg1-preparation")
    append(_group_entries(bg2_entries))
    return runs, omissions


def _toml_string(value: str) -> str:
    escaped = value.replace("\\", "\\\\").replace('"', '\\"')
    return f'"{escaped}"'


def _load_base_mods(base_recipe: Path) -> tuple[dict[str, dict], dict[str, Path]]:
    by_tp2: dict[str, dict] = {}
    sources: dict[str, Path] = {}
    for path in sorted((base_recipe / "mods").glob("*.toml")):
        with path.open("rb") as handle:
            data = tomllib.load(handle)
        by_tp2[normalize_tp2(data["tp2"])] = data
        sources[data["id"]] = path
    return by_tp2, sources


def _known_mod_for(tp2: str, base_mods: dict[str, dict]) -> dict | None:
    normalized = normalize_tp2(tp2)
    if normalized in base_mods:
        return base_mods[normalized]
    basename = normalized.rsplit("/", 1)[-1]
    matches = [
        data
        for candidate, data in base_mods.items()
        if candidate.rsplit("/", 1)[-1] == basename
    ]
    return matches[0] if len(matches) == 1 else None


def _allocate_mod_ids(tp2s: list[str], base_mods: dict[str, dict]) -> tuple[dict[str, str], set[str]]:
    result: dict[str, str] = {}
    private: set[str] = set()
    used = {data["id"] for data in base_mods.values()}
    for tp2 in tp2s:
        normalized = normalize_tp2(tp2)
        known = _known_mod_for(tp2, base_mods)
        if known is not None:
            result[normalized] = known["id"]
            continue
        candidate = slug_for_tp2(tp2)
        base = candidate
        number = 2
        while candidate in used:
            candidate = f"{base}-{number}"
            number += 1
        used.add(candidate)
        result[normalized] = candidate
        private.add(normalized)
    return result, private


def _actual_relative_path(root: Path, relative: str) -> str:
    current = root
    actual: list[str] = []
    for part in relative.replace("\\", "/").split("/"):
        match = next(
            (child for child in current.iterdir() if child.name.casefold() == part.casefold()),
            None,
        )
        if match is None:
            raise FileNotFoundError(root / relative)
        actual.append(match.name)
        current = match
    return "/".join(actual)


def _write_private_bundle(source_root: Path, tp2s: list[str], destination: Path) -> tuple[int, str, list[str], list[str]]:
    actual_tp2s = [_actual_relative_path(source_root, tp2) for tp2 in tp2s]
    roots = private_publish_roots(actual_tp2s)
    destination.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(
        destination, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=6
    ) as archive:
        for root_name in roots:
            source = source_root / root_name
            candidates = [source] if source.is_file() else sorted(source.rglob("*"))
            for path in candidates:
                if not path.is_file() or path.is_symlink():
                    continue
                relative = path.relative_to(source_root)
                folded = {part.casefold() for part in relative.parts}
                if ".git" in folded or any(
                    part == "backup" or part.startswith("backup-") or part == "backups"
                    for part in folded
                ):
                    continue
                info = zipfile.ZipInfo(relative.as_posix(), (2026, 9, 2, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                info.external_attr = 0o100644 << 16
                with path.open("rb") as source_file, archive.open(info, "w") as target_file:
                    shutil.copyfileobj(source_file, target_file, length=1024 * 1024)
    digest_hash = hashlib.sha256()
    with destination.open("rb") as archive_file:
        for chunk in iter(lambda: archive_file.read(1024 * 1024), b""):
            digest_hash.update(chunk)
    digest = digest_hash.hexdigest()
    return destination.stat().st_size, digest, roots, actual_tp2s


def _write_artifact_manifest(path: Path, length: int, digest: str, roots: list[str], tp2s: list[str], source_entry_count: int) -> None:
    # Limits are deliberately generous but finite; the generator records the exact digest/length.
    text = [
        f'id = {_toml_string(PRIVATE_ARTIFACT_ID)}',
        'name = "Creator full private/manual extras"',
        'version = "2026-09-02-reference"',
        'acquisition = "manual-user-supplied"',
        "",
        "[source]",
        'kind = "manual"',
        'url = "https://github.com/Chrizhermann/chriz-bg-collection/blob/main/docs/creator-full-install.md"',
        'reference = "local-reference-2026-09-02"',
        f'expected_filename = {_toml_string(PRIVATE_ARCHIVE_NAME)}',
        f"expected_length = {length}",
        f'sha256 = "{digest}"',
        "redirect_hosts = []",
        "",
        "[archive]",
        'kind = "zip"',
        'root_rule = "direct"',
        "publish_roots = [" + ", ".join(_toml_string(value) for value in roots) + "]",
        "tp2_paths = [" + ", ".join(_toml_string(value) for value in tp2s) + "]",
        "",
        "[archive.limits]",
        "max_depth = 24",
        "max_entries = 100000",
        "max_entry_uncompressed_bytes = 1073741824",
        "max_total_uncompressed_bytes = 21474836480",
        "max_compression_ratio = 1000",
        "",
        "[provenance]",
        'homepage = "https://github.com/Chrizhermann/chriz-bg-collection"',
        f'license = "Manual local archive containing {source_entry_count} reference installers; never distribute or bundle"',
        'url = "https://github.com/Chrizhermann/chriz-bg-collection/blob/main/docs/creator-full-install.md"',
        'reviewed_on = "2026-09-05"',
        "",
    ]
    path.write_text("\n".join(text), encoding="utf-8", newline="\n")


def _write_mod_manifest(path: Path, mod_id: str, artifact_id: str, tp2: str, language: int, components: list[Entry], base_text: str | None) -> None:
    if base_text is not None:
        base_text = re.sub(r"(?m)^prompts\s*=.*\n?", "", base_text)
        declared = {
            int(value)
            for value in re.findall(r"(?m)^id\s*=\s*(\d+)\s*$", base_text)
        }
        additions = []
        for entry in components:
            if entry.component not in declared:
                additions.extend(
                    [
                        "",
                        "[[components]]",
                        f"id = {entry.component}",
                        f"name = {_toml_string(entry.name)}",
                    ]
                )
                declared.add(entry.component)
        path.write_text((base_text.rstrip() + "\n" + "\n".join(additions)).rstrip() + "\n", encoding="utf-8", newline="\n")
        return

    lines = [
        f"id = {_toml_string(mod_id)}",
        f"artifact_id = {_toml_string(artifact_id)}",
        f"name = {_toml_string(components[0].name.split(': v', 1)[0])}",
        f"tp2 = {_toml_string(tp2)}",
        f"language = {language}",
        'weidu_artifact_id = "weidu-249-amd64"',
        'invocation_mode = "setup-name"',
    ]
    seen: set[int] = set()
    for entry in components:
        if entry.component in seen:
            continue
        seen.add(entry.component)
        lines.extend(
            [
                "",
                "[[components]]",
                f"id = {entry.component}",
                f"name = {_toml_string(entry.name)}",
            ]
        )
        if normalize_tp2(tp2) == "BARDICWONDERS/SETUP-BARDICWONDERS.TP2" and entry.component == 1008:
            lines.append(
                'prompts = [{ expected_output = "Please select 1 or 2 and press Enter.", answer = { kind = "literal", value = { kind = "integer", value = 1 } } }]'
            )
    path.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")


def _write_collection(path: Path, runs: list[OrderedRun]) -> None:
    lines = ['schema = 2', 'game_build = "2.7.3.0"']
    refs: list[tuple[str, int]] = []
    for run in runs:
        lines.extend(
            [
                "",
                "[[runs]]",
                f"run_id = {_toml_string(run.run_id)}",
                f"mod_id = {_toml_string(run.mod_id)}",
                f"phase = {_toml_string(run.phase)}",
                "components = [" + ", ".join(str(value) for value in run.components) + "]",
            ]
        )
        if run.mod_id == "eet":
            lines.extend(
                [
                    "args = [",
                    '  { kind = "literal", value = "--args-list" },',
                    '  { kind = "literal", value = "p" },',
                    '  { kind = "staged-root", value = "bg1" },',
                    "]",
                ]
            )
        else:
            lines.append("args = []")
        refs.extend((run.run_id, component) for component in run.components)

    lines.extend(
        [
            "",
            "[[features]]",
            'id = "feature:creator-full:all-components"',
            'title = "Christopher\'s full current collection"',
            'description = "Maintained-source replay of Christopher\'s full current stack; old modpack component 600 and the retired Safana-to-Abettor/Aura-to-Bard monolith branches remain explicit unmapped differences."',
            'category = "creator-full"',
            'decision = "mandatory"',
            'readiness = "experimental"',
            "components = [",
        ]
    )
    lines.extend(
        f"  {{ run_id = {_toml_string(run_id)}, component = {component} }},"
        for run_id, component in refs
    )
    lines.extend(["]", ""])
    path.write_text("\n".join(lines), encoding="utf-8", newline="\n")


def _write_reference(path: Path, entries: list[Entry]) -> None:
    rows = ["position\ttp2\tlanguage\tcomponent\tcomponent_name"]
    rows.extend(
        f"{index}\t{entry.tp2}\t{entry.language}\t{entry.component}\t{entry.name}"
        for index, entry in enumerate(entries, 1)
    )
    path.write_text("\n".join(rows) + "\n", encoding="utf-8", newline="\n")


def generate(args: argparse.Namespace) -> None:
    base_recipe = args.base_recipe.resolve()
    output = args.output.resolve()
    if output.exists():
        raise FileExistsError(f"refusing to replace existing recipe directory: {output}")
    output.mkdir(parents=True)
    for name in ("artifacts", "mods", "presets", "game-builds", "reference"):
        (output / name).mkdir()

    bg1 = parse_weidu_log(args.bg1_log)
    bg2 = parse_weidu_log(args.bg2_log)
    effective_bg2, omissions = apply_replay_policy(bg2)
    base_mods, base_mod_sources = _load_base_mods(base_recipe)
    all_tp2s = list(dict.fromkeys(entry.tp2 for entry in bg1 + effective_bg2))
    mod_ids, private_tp2s = _allocate_mod_ids(all_tp2s, base_mods)
    # effective_bg2 already includes all substitutions; build raw grouping without
    # applying the policy twice.
    runs: list[OrderedRun] = []
    run_counts: dict[str, int] = {}

    def append_effective(groups: list[list[Entry]], fixed_phase: str | None = None) -> None:
        seen_eet = False
        seen_eet_end = False
        for group in groups:
            normalized = normalize_tp2(group[0].tp2)
            if fixed_phase is not None:
                phase = fixed_phase
            elif normalized == "EET/EET.TP2":
                phase = "eet-initialization"
                seen_eet = True
            elif normalized == "EET_END/EET_END.TP2":
                phase = "eet-finalization"
                seen_eet_end = True
            elif seen_eet_end:
                phase = "post-eet-end"
            elif seen_eet:
                phase = "main"
            else:
                phase = "bg2-preparation"
            mod_id = mod_ids[normalized]
            target = "bg1" if phase == "bg1-preparation" else "bg2"
            base = f"{mod_id}-{target}"
            run_counts[base] = run_counts.get(base, 0) + 1
            suffix = "" if run_counts[base] == 1 else f"-{run_counts[base]}"
            runs.append(
                OrderedRun(
                    run_id=f"{base}{suffix}",
                    mod_id=mod_id,
                    tp2=group[0].tp2,
                    language=group[0].language,
                    phase=phase,
                    components=[entry.component for entry in group],
                )
            )

    append_effective(_group_entries(bg1), "bg1-preparation")
    append_effective(_group_entries(effective_bg2))

    entries_by_tp2: dict[str, list[Entry]] = {}
    for entry in bg1 + effective_bg2:
        entries_by_tp2.setdefault(normalize_tp2(entry.tp2), []).append(entry)
    buffbot_base = _known_mod_for(BUFFBOT_TP2, base_mods)
    if buffbot_base is None:
        raise RuntimeError("base recipe does not define BuffBot")
    private_paths = sorted(private_tp2s, key=str.casefold)
    archive_path = args.cache_root.resolve() / "manual" / PRIVATE_ARCHIVE_NAME
    length, digest, roots, actual_private_tp2s = _write_private_bundle(
        args.source_root.resolve(), private_paths, archive_path
    )
    actual_by_normalized = {
        normalize_tp2(original): actual
        for original, actual in zip(private_paths, actual_private_tp2s, strict=True)
    }
    _write_artifact_manifest(
        output / "artifacts" / f"{PRIVATE_ARTIFACT_ID}.toml",
        length,
        digest,
        roots,
        actual_private_tp2s,
        len(private_paths),
    )

    used_artifact_ids = {PRIVATE_ARTIFACT_ID, "weidu-249-amd64"}
    for normalized, entries in entries_by_tp2.items():
        mod_id = mod_ids[normalized]
        known = _known_mod_for(entries[0].tp2, base_mods)
        if known is not None:
            source = base_mod_sources[known["id"]]
            base_text = source.read_text(encoding="utf-8")
            artifact_id = known["artifact_id"]
            tp2 = known["tp2"]
            language = known["language"]
            used_artifact_ids.add(artifact_id)
        else:
            base_text = None
            artifact_id = PRIVATE_ARTIFACT_ID
            tp2 = actual_by_normalized[normalized]
            language = entries[0].language
        _write_mod_manifest(
            output / "mods" / f"{mod_id}.toml",
            mod_id,
            artifact_id,
            tp2,
            language,
            entries,
            base_text,
        )

    for artifact_id in sorted(used_artifact_ids):
        if artifact_id == PRIVATE_ARTIFACT_ID:
            continue
        shutil.copy2(
            base_recipe / "artifacts" / f"{artifact_id}.toml",
            output / "artifacts" / f"{artifact_id}.toml",
        )
    for profile in (base_recipe / "game-builds").glob("*.toml"):
        shutil.copy2(profile, output / "game-builds" / profile.name)

    _write_collection(output / "collection.toml", runs)
    (output / "presets" / "creator-full.toml").write_text(
        'id = "creator-full"\nname = "Full collection"\n',
        encoding="utf-8",
        newline="\n",
    )
    (output / "release.json").write_text(
        '{"version":"0.1.0-alpha.2","label":"Full creator setup"}\n',
        encoding="utf-8",
        newline="\n",
    )
    _write_reference(output / "reference" / "source-bg1.tsv", bg1)
    _write_reference(output / "reference" / "source-bg2.tsv", bg2)
    parity = (
        "# Creator-full parity\n\n"
        f"- BG1 source entries: {len(bg1)}.\n"
        f"- BG2 source entries: {len(bg2)}.\n"
        f"- Generated executable runs: {len(runs)}.\n"
        f"- Manual/private installer payloads: {len(private_paths)} in `{PRIVATE_ARCHIVE_NAME}`.\n"
        "- Intentional source-log deltas: omit the unavailable Project Infinity pseudo-installers "
        "`__EXTRACT.TP2` and `__IDS.TP2`; replace the reference modpack selection with the "
        "approved current components 110, 130, 140, 170, 190, 192-198, 410, 430, 440, and "
        "450; omit removed modpack component 600 because v0.2.0-alpha.1 has no equivalent; "
        "replace the raw Fade, Mazzy, Viconia, Xan, Kivan, Yeslick/Keldorn, UAI-scroll, "
        "and Skie tails with maintained modpack components; replace the unsafe numeric-kit "
        "NPC monolith with semantic Artisan/modpack assignments and Sirene's approved True "
        "Paladin choice; omit the obsolete Shapeshifter and SR standalone tail installers; add EEex "
        "1.2 component 8 (LuaJIT); append BuffBot components 1 then 0 as the absolute final run.\n"
        "- Direct maintained mappings: both Fade tails -> modpack 110; Kivan -> 130; Mazzy -> "
        "140; Xan -> 170; Viconia -> 192; Skie -> 195; Yeslick/Keldorn -> 410; and the "
        "UAI-scroll tail -> 430. Modpack 195 includes both Skie's semantic Swashbuckler "
        "assignment and the stealth-to-lock-picking transfer, so `SKIE_SKILL_FIX` is not replayed.\n"
        "- The unsafe `NPC_KIT_CHANGES` monolith is not replayed beside semantic components. "
        "Khalid, Shar-Teel, Kagain, Sarah, Skie, Imoen, and Mazzy map to current Artisan/modpack "
        "components; Sirene deliberately uses the approved True Paladin component 5 instead of "
        "the retired Martyr rewrite. Its Safana-to-Abettor and Aura-to-Bard assignments remain "
        "unmapped outcome differences.\n"
        "- Replacement evidence: Artisan chriz-v1.3.1 already sets every greater-werewolf "
        "template to `personal_space=3`; Spell Revisions v4.21-chriz.3 migrates hidden elemental "
        "subspells itself. The standalone Abettor semantic rebalance is retained with its sibling "
        "source root. The old component-600 Cat and Mouse behavior and the monolith's "
        "Safana-to-Abettor and Aura-to-Bard assignments have no approved current mappings and "
        "remain explicit outcome differences.\n"
        "- All overlapping public-alpha artifacts use the newer pinned artifact manifests.\n"
        "- This recipe is private/manual authoring data. The repository and any public recipe "
        "package contain no third-party mod payloads.\n"
    )
    (output / "PARITY.md").write_text(parity, encoding="utf-8", newline="\n")
    print(f"recipe={output}")
    print(f"manual_archive={archive_path}")
    print(f"manual_archive_length={length}")
    print(f"manual_archive_sha256={digest}")
    print(f"source_entries={len(bg1) + len(bg2)} omissions={len(omissions)} runs={len(runs)}")


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=__doc__)
    subcommands = result.add_subparsers(dest="command", required=True)
    generate_parser = subcommands.add_parser("generate")
    generate_parser.add_argument("--base-recipe", type=Path, required=True)
    generate_parser.add_argument("--bg1-log", type=Path, required=True)
    generate_parser.add_argument("--bg2-log", type=Path, required=True)
    generate_parser.add_argument("--source-root", type=Path, required=True)
    generate_parser.add_argument("--output", type=Path, required=True)
    generate_parser.add_argument("--cache-root", type=Path, required=True)
    generate_parser.set_defaults(func=generate)
    return result


def main(argv: list[str] | None = None) -> int:
    args = parser().parse_args(argv)
    args.func(args)
    return 0


if __name__ == "__main__":
    sys.exit(main())
