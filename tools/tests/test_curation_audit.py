from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.curation_audit import (
    CatalogError,
    Decision,
    decision_totals,
    load_catalogs,
    parse_catalog_text,
)


HEADER = """\
| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
"""


class CatalogParserTests(unittest.TestCase):
    def test_reports_each_component_cell_with_normalized_semantics(self) -> None:
        rows = parse_catalog_text(
            "EXAMPLE",
            HEADER
            + """\
| 0 | Hidden row | Rules |  | ✓ | |
| 10 | Visible off | Rules | Speed |  | optional |
| 11 | Visible on | Rules | Speed | ✓ | default |
| 20 | Parent-owned child | Rules |  | ✓ | mandatory |
""",
            Path("EXAMPLE.md"),
        )

        self.assertEqual([row.catalog for row in rows], ["EXAMPLE"] * 4)
        self.assertEqual([row.component_id for row in rows], [0, 10, 11, 20])
        self.assertEqual([row.subgroup for row in rows], ["", "Speed", "Speed", ""])
        self.assertEqual([row.installed for row in rows], [True, False, True, True])
        self.assertEqual(
            [row.decision for row in rows],
            [
                Decision.EXCLUDED,
                Decision.OPTIONAL,
                Decision.DEFAULT,
                Decision.MANDATORY,
            ],
        )
        self.assertEqual([row.line_number for row in rows], [3, 4, 5, 6])

    def test_accepts_the_dev_installed_marker_header(self) -> None:
        rows = parse_catalog_text(
            "DEV",
            HEADER.replace("✓", "Dev")
            + "| 100 | Development preset | | | ✓ | default |\n",
            Path("DEV.md"),
        )
        self.assertTrue(rows[0].installed)

    def test_rejects_an_unknown_decision(self) -> None:
        with self.assertRaisesRegex(CatalogError, "unknown Decision.*recommended"):
            parse_catalog_text(
                "BAD",
                HEADER + "| 1 | Bad | | | | recommended |\n",
                Path("BAD.md"),
            )

    def test_rejects_duplicate_component_rows_with_both_locations(self) -> None:
        with self.assertRaisesRegex(CatalogError, r"duplicate component 1.*line 3.*line 4"):
            parse_catalog_text(
                "DUP",
                HEADER
                + "| 1 | First | | | | optional |\n"
                + "| 1 | Second | | | | default |\n",
                Path("DUP.md"),
            )

    def test_rejects_malformed_numeric_component_ids(self) -> None:
        with self.assertRaisesRegex(CatalogError, "malformed numeric component id.*1a"):
            parse_catalog_text(
                "BADID",
                HEADER + "| 1a | Bad | | | | optional |\n",
                Path("BADID.md"),
            )

    def test_rejects_raw_notes_inserted_between_component_rows(self) -> None:
        with self.assertRaisesRegex(CatalogError, r":4: raw text inside component table"):
            parse_catalog_text(
                "NOTE",
                HEADER
                + "| 1 | First | | | | optional |\n"
                + "This note belongs below the table.\n"
                + "| 2 | Second | | | | default |\n",
                Path("NOTE.md"),
            )

    def test_rejects_unknown_installed_markers(self) -> None:
        with self.assertRaisesRegex(CatalogError, "unknown installed marker.*yes"):
            parse_catalog_text(
                "MARKER",
                HEADER + "| 1 | Bad | | | yes | optional |\n",
                Path("MARKER.md"),
            )

    def test_load_catalogs_rejects_a_markdown_file_with_two_component_tables(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "DUP_TABLE.md").write_text(
                HEADER
                + "| 1 | First | | | | optional |\n\n"
                + HEADER
                + "| 2 | Second | | | | default |\n",
                encoding="utf-8",
            )
            with self.assertRaisesRegex(CatalogError, "multiple component tables"):
                load_catalogs(root)

    def test_reviewed_catalog_totals_are_frozen(self) -> None:
        root = Path(__file__).resolve().parents[2] / "docs" / "curation" / "components"
        rows = load_catalogs(root)

        self.assertEqual(len(rows), 1_173)
        self.assertEqual(
            decision_totals(rows),
            {
                Decision.EXCLUDED: 592,
                Decision.OPTIONAL: 158,
                Decision.DEFAULT: 290,
                Decision.MANDATORY: 133,
            },
        )


if __name__ == "__main__":
    unittest.main()
