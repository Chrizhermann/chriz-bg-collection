"""Mechanical audit helpers for the authored component-catalog tables.

This module validates table structure and reports authored cells.  It deliberately does
not infer dependencies, readiness, parents, or player-facing grouping from component
names or prose.
"""

from __future__ import annotations

from collections import Counter
from dataclasses import dataclass
from enum import Enum
from pathlib import Path
import argparse
import re
import sys
import tomllib
from typing import Iterable


class CatalogError(ValueError):
    """A component catalog is structurally ambiguous or malformed."""


class AuditError(ValueError):
    """The authored curation map is incomplete, inconsistent, or malformed."""


class Decision(str, Enum):
    """The four normalized curation decisions authored by Christopher."""

    EXCLUDED = "excluded"
    OPTIONAL = "optional"
    DEFAULT = "default"
    MANDATORY = "mandatory"


@dataclass(frozen=True)
class CatalogRow:
    """One mechanically parsed component row from a catalog."""

    catalog: str
    component_id: int
    component: str
    group: str
    subgroup: str
    installed: bool
    decision: Decision
    source_path: Path
    line_number: int


@dataclass(frozen=True, order=True)
class RowKey:
    """Stable identity of one catalog component row."""

    catalog: str
    component_id: int

    def __str__(self) -> str:
        return f"{self.catalog}:{self.component_id}"


@dataclass(frozen=True)
class CatalogMapping:
    """Explicitly excluded component ids for one catalog."""

    catalog: str
    excluded: tuple[int, ...]


@dataclass(frozen=True)
class MappingTarget:
    """An authored feature expansion or approved alpha omission."""

    target_id: str
    kind: str
    rows: tuple[RowKey, ...]
    parent: str | None
    collection_root: bool
    reason: str | None


@dataclass(frozen=True)
class ChoiceGroup:
    """One player-facing at-most-one choice group."""

    group_id: str
    catalog: str
    subgroup: str
    options: tuple[RowKey, ...]
    default: str


@dataclass(frozen=True)
class FixedGroup:
    """A source subgroup intentionally represented as a fixed/internal expansion."""

    group_id: str
    catalog: str
    subgroup: str
    options: tuple[RowKey, ...]
    reason: str


@dataclass(frozen=True)
class CurationMap:
    """Parsed, authored row-to-outcome map."""

    expected_rows: int
    expected_choice_groups: int
    expected_optional_none_groups: int
    expected_fixed_groups: int
    parents: frozenset[str]
    catalogs: tuple[CatalogMapping, ...]
    targets: tuple[MappingTarget, ...]
    choice_groups: tuple[ChoiceGroup, ...]
    fixed_groups: tuple[FixedGroup, ...]


@dataclass(frozen=True)
class AuditResult:
    """Successful coverage totals suitable for deterministic reporting."""

    mapped_rows: int
    excluded_rows: int
    feature_rows: int
    omission_rows: int
    choice_groups: int
    optional_none_groups: int
    fixed_groups: int


_HEADER_PREFIX = ("#", "Component", "Group", "Subgroup")
_INSTALLED_HEADERS = {"✓", "Dev"}
_SEPARATOR_RE = re.compile(r":?-{3,}:?")
_COMPONENT_ID_RE = re.compile(r"(?:0|[1-9][0-9]*)")
_ROW_KEY_RE = re.compile(r"([^:]+):(0|[1-9][0-9]*)")


def _cells(line: str) -> list[str] | None:
    stripped = line.strip()
    if not (stripped.startswith("|") and stripped.endswith("|")):
        return None
    return [cell.strip() for cell in stripped[1:-1].split("|")]


def _is_component_header(cells: list[str] | None) -> bool:
    return bool(
        cells
        and len(cells) == 6
        and tuple(cells[:4]) == _HEADER_PREFIX
        and cells[4] in _INSTALLED_HEADERS
        and cells[5] == "Decision"
    )


def _location(path: Path, line_number: int) -> str:
    return f"{path}:{line_number}"


def parse_catalog_text(catalog: str, text: str, source_path: Path) -> list[CatalogRow]:
    """Parse the single six-column component table in ``text``, if present."""

    lines = text.splitlines()
    header_indexes = [
        index for index, line in enumerate(lines) if _is_component_header(_cells(line))
    ]
    if not header_indexes:
        return []
    if len(header_indexes) != 1:
        locations = ", ".join(str(index + 1) for index in header_indexes)
        raise CatalogError(f"{source_path}: multiple component tables at lines {locations}")

    header_index = header_indexes[0]
    separator_index = header_index + 1
    if separator_index >= len(lines):
        raise CatalogError(
            f"{_location(source_path, header_index + 1)}: component table has no separator"
        )
    separator = _cells(lines[separator_index])
    if not separator or len(separator) != 6 or not all(
        _SEPARATOR_RE.fullmatch(cell) for cell in separator
    ):
        raise CatalogError(
            f"{_location(source_path, separator_index + 1)}: malformed component table separator"
        )

    rows: list[CatalogRow] = []
    first_line_by_component: dict[int, int] = {}
    index = separator_index + 1
    while index < len(lines):
        cells = _cells(lines[index])
        if cells is None:
            break
        if len(cells) != 6:
            raise CatalogError(
                f"{_location(source_path, index + 1)}: component row has "
                f"{len(cells)} cells; expected 6"
            )

        component_cell, component, group, subgroup, installed_cell, decision_cell = cells
        if not _COMPONENT_ID_RE.fullmatch(component_cell):
            raise CatalogError(
                f"{_location(source_path, index + 1)}: malformed numeric component id "
                f"{component_cell!r}"
            )
        component_id = int(component_cell)
        if component_id in first_line_by_component:
            first_line = first_line_by_component[component_id]
            raise CatalogError(
                f"{source_path}: duplicate component {component_id} at line {first_line} "
                f"and line {index + 1}"
            )
        first_line_by_component[component_id] = index + 1

        if installed_cell not in {"", "✓"}:
            raise CatalogError(
                f"{_location(source_path, index + 1)}: unknown installed marker "
                f"{installed_cell!r}"
            )
        normalized_decision = decision_cell or Decision.EXCLUDED.value
        try:
            decision = Decision(normalized_decision)
        except ValueError as error:
            raise CatalogError(
                f"{_location(source_path, index + 1)}: unknown Decision "
                f"{decision_cell!r}"
            ) from error

        rows.append(
            CatalogRow(
                catalog=catalog,
                component_id=component_id,
                component=component,
                group=group,
                subgroup=subgroup,
                installed=installed_cell == "✓",
                decision=decision,
                source_path=source_path,
                line_number=index + 1,
            )
        )
        index += 1

    if not rows:
        raise CatalogError(
            f"{_location(source_path, separator_index + 1)}: component table has no rows"
        )

    first_raw_line = index + 1
    for later_index in range(index, len(lines)):
        later_cells = _cells(lines[later_index])
        if later_cells and len(later_cells) == 6 and _COMPONENT_ID_RE.fullmatch(later_cells[0]):
            raise CatalogError(
                f"{_location(source_path, first_raw_line)}: raw text inside component table "
                f"before component row at line {later_index + 1}"
            )

    return rows


def load_catalogs(root: Path) -> list[CatalogRow]:
    """Load every component table below ``root`` in stable filename order."""

    rows: list[CatalogRow] = []
    for path in sorted(root.glob("*.md"), key=lambda candidate: candidate.name.casefold()):
        rows.extend(
            parse_catalog_text(
                path.stem,
                path.read_text(encoding="utf-8"),
                path,
            )
        )
    return rows


def decision_totals(rows: Iterable[CatalogRow]) -> dict[Decision, int]:
    """Count every decision, retaining zeroes for all four legal values."""

    counts = Counter(row.decision for row in rows)
    return {decision: counts[decision] for decision in Decision}


def _expect_table(value: object, context: str) -> dict[str, object]:
    if not isinstance(value, dict):
        raise AuditError(f"{context} must be a TOML table")
    return value


def _expect_list(value: object, context: str) -> list[object]:
    if not isinstance(value, list):
        raise AuditError(f"{context} must be a TOML array")
    return value


def _expect_str(value: object, context: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise AuditError(f"{context} must be a non-empty string")
    return value


def _expect_int(value: object, context: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise AuditError(f"{context} must be a non-negative integer")
    return value


def _row_key(value: object, context: str) -> RowKey:
    raw = _expect_str(value, context)
    match = _ROW_KEY_RE.fullmatch(raw)
    if not match:
        raise AuditError(f"{context} has malformed row reference {raw!r}")
    return RowKey(match.group(1), int(match.group(2)))


def _row_keys(value: object, context: str) -> tuple[RowKey, ...]:
    raw_rows = _expect_list(value, context)
    rows = tuple(_row_key(raw, f"{context}[{index}]") for index, raw in enumerate(raw_rows))
    if len(rows) != len(set(rows)):
        raise AuditError(f"{context} contains duplicate row references")
    return rows


def _only_keys(table: dict[str, object], allowed: set[str], context: str) -> None:
    unknown = sorted(set(table) - allowed)
    if unknown:
        raise AuditError(f"{context} has unknown keys: {', '.join(unknown)}")


def load_curation_map(path: Path) -> CurationMap:
    """Load and structurally validate the authored TOML curation map."""

    try:
        raw = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise AuditError(f"could not load {path}: {error}") from error

    _only_keys(
        raw,
        {
            "schema",
            "expected_rows",
            "expected_choice_groups",
            "expected_optional_none_groups",
            "expected_fixed_groups",
            "parents",
            "catalogs",
            "targets",
            "choice_groups",
            "fixed_groups",
        },
        str(path),
    )
    if raw.get("schema") != 1:
        raise AuditError(f"{path}: schema must be 1")

    expected_rows = _expect_int(raw.get("expected_rows"), "expected_rows")
    expected_choice_groups = _expect_int(
        raw.get("expected_choice_groups"), "expected_choice_groups"
    )
    expected_optional_none_groups = _expect_int(
        raw.get("expected_optional_none_groups"), "expected_optional_none_groups"
    )
    expected_fixed_groups = _expect_int(
        raw.get("expected_fixed_groups", 0), "expected_fixed_groups"
    )

    parent_values = _expect_list(raw.get("parents", []), "parents")
    parent_list = [
        _expect_str(value, f"parents[{index}]")
        for index, value in enumerate(parent_values)
    ]
    if len(parent_list) != len(set(parent_list)):
        raise AuditError("parents contains duplicate feature ids")
    parents = frozenset(parent_list)

    catalogs: list[CatalogMapping] = []
    catalog_ids: set[str] = set()
    for index, value in enumerate(_expect_list(raw.get("catalogs", []), "catalogs")):
        context = f"catalogs[{index}]"
        table = _expect_table(value, context)
        _only_keys(table, {"id", "excluded"}, context)
        catalog = _expect_str(table.get("id"), f"{context}.id")
        if catalog in catalog_ids:
            raise AuditError(f"duplicate catalog mapping {catalog!r}")
        catalog_ids.add(catalog)
        raw_excluded = _expect_list(table.get("excluded", []), f"{context}.excluded")
        excluded = tuple(
            _expect_int(component, f"{context}.excluded[{item_index}]")
            for item_index, component in enumerate(raw_excluded)
        )
        if len(excluded) != len(set(excluded)):
            raise AuditError(f"{context}.excluded contains duplicate component ids")
        catalogs.append(CatalogMapping(catalog, excluded))

    targets: list[MappingTarget] = []
    target_ids: set[str] = set()
    for index, value in enumerate(_expect_list(raw.get("targets", []), "targets")):
        context = f"targets[{index}]"
        table = _expect_table(value, context)
        _only_keys(
            table,
            {"id", "kind", "rows", "parent", "collection_root", "reason"},
            context,
        )
        target_id = _expect_str(table.get("id"), f"{context}.id")
        if target_id in target_ids:
            raise AuditError(f"duplicate target definition {target_id!r}")
        target_ids.add(target_id)
        kind = _expect_str(table.get("kind"), f"{context}.kind")
        if kind not in {"feature", "omission"}:
            raise AuditError(f"{context}.kind must be 'feature' or 'omission'")
        rows = _row_keys(table.get("rows", []), f"{context}.rows")
        if not rows:
            raise AuditError(f"{context}.rows must name at least one catalog row")
        parent_value = table.get("parent")
        parent = None if parent_value is None else _expect_str(parent_value, f"{context}.parent")
        collection_root_value = table.get("collection_root", False)
        if not isinstance(collection_root_value, bool):
            raise AuditError(f"{context}.collection_root must be a boolean")
        reason_value = table.get("reason")
        reason = None if reason_value is None else _expect_str(reason_value, f"{context}.reason")
        if kind == "omission" and reason is None:
            raise AuditError(f"{context}: omission {target_id!r} requires an authored reason")
        if parent is not None and parent not in parents:
            raise AuditError(f"{context}.parent references undeclared parent {parent!r}")
        targets.append(
            MappingTarget(
                target_id,
                kind,
                rows,
                parent,
                collection_root_value,
                reason,
            )
        )

    choice_groups: list[ChoiceGroup] = []
    choice_ids: set[str] = set()
    choice_identities: set[tuple[str, str]] = set()
    for index, value in enumerate(
        _expect_list(raw.get("choice_groups", []), "choice_groups")
    ):
        context = f"choice_groups[{index}]"
        table = _expect_table(value, context)
        _only_keys(table, {"id", "catalog", "subgroup", "options", "default"}, context)
        group_id = _expect_str(table.get("id"), f"{context}.id")
        if group_id in choice_ids:
            raise AuditError(f"duplicate choice-group definition {group_id!r}")
        choice_ids.add(group_id)
        catalog = _expect_str(table.get("catalog"), f"{context}.catalog")
        subgroup = _expect_str(table.get("subgroup"), f"{context}.subgroup")
        identity = (catalog, subgroup)
        if identity in choice_identities:
            raise AuditError(f"duplicate choice subgroup {catalog!r} / {subgroup!r}")
        choice_identities.add(identity)
        options = _row_keys(table.get("options", []), f"{context}.options")
        if not options:
            raise AuditError(f"{context}.options must not be empty")
        if any(option.catalog != catalog for option in options):
            raise AuditError(f"{context}.options must all belong to catalog {catalog!r}")
        default = _expect_str(table.get("default"), f"{context}.default")
        if default != "none" and default not in {str(option) for option in options}:
            raise AuditError(f"{context}.default must be 'none' or one of its options")
        choice_groups.append(ChoiceGroup(group_id, catalog, subgroup, options, default))

    fixed_groups: list[FixedGroup] = []
    fixed_ids: set[str] = set()
    fixed_identities: set[tuple[str, str]] = set()
    for index, value in enumerate(_expect_list(raw.get("fixed_groups", []), "fixed_groups")):
        context = f"fixed_groups[{index}]"
        table = _expect_table(value, context)
        _only_keys(table, {"id", "catalog", "subgroup", "options", "reason"}, context)
        group_id = _expect_str(table.get("id"), f"{context}.id")
        if group_id in fixed_ids:
            raise AuditError(f"duplicate fixed-group definition {group_id!r}")
        fixed_ids.add(group_id)
        catalog = _expect_str(table.get("catalog"), f"{context}.catalog")
        subgroup = _expect_str(table.get("subgroup"), f"{context}.subgroup")
        identity = (catalog, subgroup)
        if identity in fixed_identities or identity in choice_identities:
            raise AuditError(f"duplicate represented subgroup {catalog!r} / {subgroup!r}")
        fixed_identities.add(identity)
        options = _row_keys(table.get("options", []), f"{context}.options")
        if not options:
            raise AuditError(f"{context}.options must not be empty")
        if any(option.catalog != catalog for option in options):
            raise AuditError(f"{context}.options must all belong to catalog {catalog!r}")
        reason = _expect_str(table.get("reason"), f"{context}.reason")
        fixed_groups.append(FixedGroup(group_id, catalog, subgroup, options, reason))

    return CurationMap(
        expected_rows,
        expected_choice_groups,
        expected_optional_none_groups,
        expected_fixed_groups,
        parents,
        tuple(catalogs),
        tuple(targets),
        tuple(choice_groups),
        tuple(fixed_groups),
    )


def audit_curation_map(rows: Iterable[CatalogRow], curation_map: CurationMap) -> AuditResult:
    """Require exactly one explicit map outcome for every catalog row."""

    row_list = list(rows)
    by_key = {RowKey(row.catalog, row.component_id): row for row in row_list}
    if len(by_key) != len(row_list):
        raise AuditError("catalog input contains duplicate row identities")
    if len(row_list) != curation_map.expected_rows:
        raise AuditError(
            f"expected {curation_map.expected_rows} catalog rows, found {len(row_list)}"
        )

    catalog_names = {row.catalog for row in row_list}
    mapped_catalogs = {catalog.catalog for catalog in curation_map.catalogs}
    if mapped_catalogs != catalog_names:
        missing = sorted(catalog_names - mapped_catalogs)
        extra = sorted(mapped_catalogs - catalog_names)
        raise AuditError(f"catalog map mismatch; missing={missing}, extra={extra}")

    outcome_by_row: dict[RowKey, tuple[str, MappingTarget | None]] = {}
    for catalog in curation_map.catalogs:
        for component_id in catalog.excluded:
            key = RowKey(catalog.catalog, component_id)
            if key in outcome_by_row:
                raise AuditError(f"row {key} has more than one map outcome")
            outcome_by_row[key] = ("excluded", None)

    for target in curation_map.targets:
        for key in target.rows:
            if key in outcome_by_row:
                raise AuditError(f"row {key} has more than one map outcome")
            outcome_by_row[key] = (target.kind, target)

    unknown = sorted(set(outcome_by_row) - set(by_key))
    missing = sorted(set(by_key) - set(outcome_by_row))
    if unknown or missing:
        raise AuditError(
            "row coverage mismatch; "
            f"missing={[str(key) for key in missing]}, "
            f"unknown={[str(key) for key in unknown]}"
        )

    feature_rows = 0
    omission_rows = 0
    excluded_rows = 0
    for key, row in by_key.items():
        outcome, target = outcome_by_row[key]
        if row.decision == Decision.EXCLUDED:
            if outcome != "excluded":
                raise AuditError(
                    f"row {key} has blank Decision but maps to selectable {outcome} target"
                )
            excluded_rows += 1
            continue
        if outcome == "excluded":
            raise AuditError(f"row {key} has Decision {row.decision.value!r} but maps to excluded")
        if target is None:
            raise AuditError(f"row {key} has no declared mapping target")
        if row.decision == Decision.MANDATORY and not (
            target.parent is not None or target.collection_root
        ):
            raise AuditError(
                f"mandatory row {key} requires a declared parent or collection_root target"
            )
        if outcome == "feature":
            feature_rows += 1
        elif outcome == "omission":
            omission_rows += 1
        else:  # pragma: no cover - guarded by load_curation_map
            raise AuditError(f"row {key} has unknown outcome {outcome!r}")

    rows_by_group: dict[tuple[str, str], set[RowKey]] = {}
    for key, row in by_key.items():
        if row.subgroup and row.decision != Decision.EXCLUDED:
            rows_by_group.setdefault((row.catalog, row.subgroup), set()).add(key)

    represented: set[tuple[str, str]] = set()
    optional_none_groups = 0
    for group in curation_map.choice_groups:
        identity = (group.catalog, group.subgroup)
        expected_options = rows_by_group.get(identity)
        if expected_options is None:
            raise AuditError(f"choice group {group.group_id!r} does not match a curated subgroup")
        if set(group.options) != expected_options:
            raise AuditError(
                f"choice group {group.group_id!r} options do not match its nonblank catalog rows"
            )
        represented.add(identity)
        option_rows = [by_key[option] for option in group.options]
        if all(row.decision == Decision.OPTIONAL for row in option_rows):
            optional_none_groups += 1
            if group.default != "none":
                raise AuditError(
                    f"optional-only choice group {group.group_id!r} must declare default 'none'"
                )
        else:
            defaults = [row for row in option_rows if row.decision == Decision.DEFAULT]
            expected_default = (
                str(RowKey(defaults[0].catalog, defaults[0].component_id))
                if len(defaults) == 1
                else None
            )
            if expected_default is not None and group.default != expected_default:
                raise AuditError(
                    f"choice group {group.group_id!r} must use authored default "
                    f"{expected_default!r}"
                )

    for group in curation_map.fixed_groups:
        identity = (group.catalog, group.subgroup)
        expected_options = rows_by_group.get(identity)
        if expected_options is None:
            raise AuditError(f"fixed group {group.group_id!r} does not match a curated subgroup")
        if set(group.options) != expected_options:
            raise AuditError(
                f"fixed group {group.group_id!r} options do not match its nonblank catalog rows"
            )
        represented.add(identity)

    if represented != set(rows_by_group):
        missing_groups = sorted(set(rows_by_group) - represented)
        raise AuditError(f"unrepresented curated subgroups: {missing_groups}")
    if len(curation_map.choice_groups) != curation_map.expected_choice_groups:
        raise AuditError(
            f"expected {curation_map.expected_choice_groups} choice groups, "
            f"found {len(curation_map.choice_groups)}"
        )
    if optional_none_groups != curation_map.expected_optional_none_groups:
        raise AuditError(
            f"expected {curation_map.expected_optional_none_groups} optional-only none groups, "
            f"found {optional_none_groups}"
        )
    if len(curation_map.fixed_groups) != curation_map.expected_fixed_groups:
        raise AuditError(
            f"expected {curation_map.expected_fixed_groups} fixed groups, "
            f"found {len(curation_map.fixed_groups)}"
        )

    return AuditResult(
        mapped_rows=len(outcome_by_row),
        excluded_rows=excluded_rows,
        feature_rows=feature_rows,
        omission_rows=omission_rows,
        choice_groups=len(curation_map.choice_groups),
        optional_none_groups=optional_none_groups,
        fixed_groups=len(curation_map.fixed_groups),
    )


def coverage_report(rows: Iterable[CatalogRow], result: AuditResult) -> str:
    """Render one stable, compact coverage line."""

    totals = decision_totals(rows)
    return (
        f"rows={result.mapped_rows} "
        f"excluded={totals[Decision.EXCLUDED]} "
        f"optional={totals[Decision.OPTIONAL]} "
        f"default={totals[Decision.DEFAULT]} "
        f"mandatory={totals[Decision.MANDATORY]} "
        f"feature_rows={result.feature_rows} "
        f"omission_rows={result.omission_rows} "
        f"choice_groups={result.choice_groups} "
        f"optional_none_groups={result.optional_none_groups} "
        f"fixed_groups={result.fixed_groups}"
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Audit component curation coverage")
    parser.add_argument("catalog_directory", type=Path)
    parser.add_argument("curation_map", type=Path)
    args = parser.parse_args(argv)
    try:
        rows = load_catalogs(args.catalog_directory)
        result = audit_curation_map(rows, load_curation_map(args.curation_map))
    except (CatalogError, AuditError) as error:
        print(f"curation audit failed: {error}", file=sys.stderr)
        return 1
    print(coverage_report(rows, result))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
