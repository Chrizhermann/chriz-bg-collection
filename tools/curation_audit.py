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
import re
from typing import Iterable


class CatalogError(ValueError):
    """A component catalog is structurally ambiguous or malformed."""


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


_HEADER_PREFIX = ("#", "Component", "Group", "Subgroup")
_INSTALLED_HEADERS = {"✓", "Dev"}
_SEPARATOR_RE = re.compile(r":?-{3,}:?")
_COMPONENT_ID_RE = re.compile(r"(?:0|[1-9][0-9]*)")


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
                f"{_location(source_path, index + 1)}: component row has {len(cells)} cells; expected 6"
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
