from __future__ import annotations

import tempfile
import tomllib
import unittest
from pathlib import Path

from tools.curation_audit import (
    AuditError,
    CatalogError,
    Decision,
    RowKey,
    audit_curation_map,
    coverage_report,
    decision_totals,
    load_curation_map,
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

        # October 13 intake adds Modpack 237/238/650/651/660 and SoD 177/198/285.
        self.assertEqual(len(rows), 1_235)
        self.assertEqual(
            decision_totals(rows),
            {
                Decision.EXCLUDED: 515,
                # September 22 approval moves HGO 38 from optional to default.
                # October 12 moves Emotion 301 back to optional.
                # October 13 makes EET's desktop shortcut (EET 100) optional.
                # October 13 intake: campaign XP caps Modpack 650/651 are optional.
                Decision.OPTIONAL: 242,
                # Approved Rebalance 111/302 defaults; Artisan 51010 is optional.
                # October 13: bard ceilings Rebalance 420/421 and Bardic 3010 are defaults;
                # EET 100 (desktop shortcut) leaves the defaults.
                # October 13 intake: Modpack 237/238/660 are defaults and scrolls 430
                # moves from mandatory to default.
                Decision.DEFAULT: 324,
                # October 13 intake: SoD 177/198/285 join the mandatory bundle (430 left it).
                Decision.MANDATORY: 154,
            },
        )


class CurationMapTests(unittest.TestCase):
    def _write_map(self, root: Path, text: str) -> Path:
        path = root / "curation-map.toml"
        path.write_text(text, encoding="utf-8")
        return path

    def test_audits_explicit_feature_omission_and_excluded_outcomes(self) -> None:
        rows = parse_catalog_text(
            "EXAMPLE",
            HEADER
            + "| 0 | Hidden | | | | |\n"
            + "| 1 | Root child | | | ✓ | mandatory |\n"
            + "| 2 | Choice A | | Mode | | optional |\n"
            + "| 3 | Choice B | | Mode | ✓ | default |\n"
            + "| 4 | Deferred | | | | default |\n",
            Path("EXAMPLE.md"),
        )
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            map_path = self._write_map(
                root,
                """\
schema = 1
expected_rows = 5
expected_choice_groups = 1
expected_optional_none_groups = 0
parents = ["mod:EXAMPLE"]

[[catalogs]]
id = "EXAMPLE"
excluded = [0]

[[targets]]
id = "feature:example-core"
kind = "feature"
rows = ["EXAMPLE:1"]
parent = "mod:EXAMPLE"

[[targets]]
id = "feature:example-a"
kind = "feature"
rows = ["EXAMPLE:2"]

[[targets]]
id = "feature:example-b"
kind = "feature"
rows = ["EXAMPLE:3"]

[[targets]]
id = "omission:example-deferred"
kind = "omission"
rows = ["EXAMPLE:4"]
reason = "The required maintained implementation has not been released."

[[choice_groups]]
id = "choice:example-mode"
catalog = "EXAMPLE"
subgroup = "Mode"
options = ["EXAMPLE:2", "EXAMPLE:3"]
default = "EXAMPLE:3"
""",
            )

            curation_map = load_curation_map(map_path)
            result = audit_curation_map(rows, curation_map)

        self.assertEqual(result.mapped_rows, 5)
        self.assertEqual(result.feature_rows, 3)
        self.assertEqual(result.omission_rows, 1)
        self.assertEqual(result.excluded_rows, 1)
        self.assertEqual(result.choice_groups, 1)

    def test_rejects_a_blank_row_mapped_to_a_selectable_feature(self) -> None:
        rows = parse_catalog_text(
            "EXAMPLE",
            HEADER + "| 0 | Hidden | | | | |\n",
            Path("EXAMPLE.md"),
        )
        with tempfile.TemporaryDirectory() as temp_dir:
            map_path = self._write_map(
                Path(temp_dir),
                """\
schema = 1
expected_rows = 1
expected_choice_groups = 0
expected_optional_none_groups = 0
parents = []

[[catalogs]]
id = "EXAMPLE"
excluded = []

[[targets]]
id = "feature:hidden"
kind = "feature"
rows = ["EXAMPLE:0"]
""",
            )
            with self.assertRaisesRegex(AuditError, "blank Decision.*selectable"):
                audit_curation_map(rows, load_curation_map(map_path))

    def test_rejects_mandatory_row_without_parent_or_collection_root(self) -> None:
        rows = parse_catalog_text(
            "EXAMPLE",
            HEADER + "| 1 | Child | | | | mandatory |\n",
            Path("EXAMPLE.md"),
        )
        with tempfile.TemporaryDirectory() as temp_dir:
            map_path = self._write_map(
                Path(temp_dir),
                """\
schema = 1
expected_rows = 1
expected_choice_groups = 0
expected_optional_none_groups = 0
parents = []

[[catalogs]]
id = "EXAMPLE"
excluded = []

[[targets]]
id = "feature:orphan"
kind = "feature"
rows = ["EXAMPLE:1"]
""",
            )
            with self.assertRaisesRegex(AuditError, "mandatory.*parent or collection_root"):
                audit_curation_map(rows, load_curation_map(map_path))

    def test_rejects_duplicate_target_definitions(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            map_path = self._write_map(
                Path(temp_dir),
                """\
schema = 1
expected_rows = 0
expected_choice_groups = 0
expected_optional_none_groups = 0
parents = []
targets = [
  { id = "feature:same", kind = "feature", rows = ["EXAMPLE:1"] },
  { id = "feature:same", kind = "feature", rows = ["EXAMPLE:2"] },
]
""",
            )
            with self.assertRaisesRegex(AuditError, "duplicate target.*feature:same"):
                load_curation_map(map_path)

    def test_rejects_an_omission_without_an_authored_reason(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            map_path = self._write_map(
                Path(temp_dir),
                """\
schema = 1
expected_rows = 0
expected_choice_groups = 0
expected_optional_none_groups = 0
parents = []
targets = [
  { id = "omission:no-reason", kind = "omission", rows = ["EXAMPLE:1"] },
]
""",
            )
            with self.assertRaisesRegex(AuditError, "omission.*reason"):
                load_curation_map(map_path)

    def test_rejects_a_choice_group_with_multiple_authored_defaults(self) -> None:
        rows = parse_catalog_text(
            "EXAMPLE",
            HEADER
            + "| 1 | First default | | Mode | | default |\n"
            + "| 2 | Second default | | Mode | | default |\n",
            Path("EXAMPLE.md"),
        )
        with tempfile.TemporaryDirectory() as temp_dir:
            map_path = self._write_map(
                Path(temp_dir),
                """\
schema = 1
expected_rows = 2
expected_choice_groups = 1
expected_optional_none_groups = 0
parents = []

[[catalogs]]
id = "EXAMPLE"
excluded = []

[[targets]]
id = "feature:first"
kind = "feature"
rows = ["EXAMPLE:1"]

[[targets]]
id = "feature:second"
kind = "feature"
rows = ["EXAMPLE:2"]

[[choice_groups]]
id = "choice:ambiguous"
catalog = "EXAMPLE"
subgroup = "Mode"
options = ["EXAMPLE:1", "EXAMPLE:2"]
default = "none"
""",
            )
            with self.assertRaisesRegex(AuditError, "exactly one authored default"):
                audit_curation_map(rows, load_curation_map(map_path))

    def test_current_map_covers_every_reviewed_catalog_row(self) -> None:
        root = Path(__file__).resolve().parents[2]
        rows = load_catalogs(root / "docs" / "curation" / "components")
        curation_map = load_curation_map(root / "manifest" / "curation-map.toml")

        result = audit_curation_map(rows, curation_map)

        self.assertEqual(result.mapped_rows, 1_235)
        self.assertEqual(result.excluded_rows, 515)
        self.assertEqual(result.feature_rows + result.omission_rows, 720)
        self.assertEqual(result.choice_groups, 74)
        self.assertEqual(result.optional_none_groups, 23)
        self.assertEqual(result.fixed_groups, 10)
        self.assertIn("rows=1235", coverage_report(rows, result))

    def test_september16_intake_rows_match_the_approved_recipe_features(self) -> None:
        root = Path(__file__).resolve().parents[2]
        rows = {
            RowKey(row.catalog, row.component_id): row
            for row in load_catalogs(root / "docs/curation/components")
        }
        curation_map = load_curation_map(root / "manifest/curation-map.toml")
        target_by_row = {row: target for target in curation_map.targets for row in target.rows}
        collection = tomllib.loads((root / "manifest/collection.toml").read_text())
        features = {feature["id"]: feature for feature in collection["features"]}
        approved = {
            "CHRIZ-BG-MODPACK": ("chriz-bg-modpack", {188: "mandatory", 189: "mandatory", 199: "default", 620: "default"}),
            "CHRIZ-SOD-REMIX": ("chriz-sod-remix", {115: "mandatory", 135: "mandatory", 256: "mandatory", 257: "optional", 265: "mandatory", 266: "mandatory"}),
            "CHRIZ-BG-REBALANCE": ("chriz-bg-rebalance", {110: "optional", 111: "default"}),
            "SPELL_REV": ("spell-rev", {80: "default", 81: "optional"}),
            "SRCB_RR_COMPAT": ("srcb-rr-compat", {0: "mandatory"}),
        }
        for catalog, (mod_id, components) in approved.items():
            for component, decision in components.items():
                with self.subTest(catalog=catalog, component=component):
                    key = RowKey(catalog, component)
                    self.assertTrue(key in rows, f"missing approved catalog row {key}")
                    self.assertEqual(rows[key].decision.value, decision)
                    self.assertTrue(key in target_by_row, f"missing approved mapping {key}")
                    target = target_by_row[key]
                    feature = features[target.target_id]
                    self.assertEqual(target.kind, "feature")
                    self.assertEqual(target.parent, f"mod:{mod_id}")
                    self.assertEqual(feature["decision"], decision)
                    self.assertTrue(any(ref["component"] == component for ref in feature["components"]))

        lightning = next((group for group in curation_map.choice_groups
                          if group.group_id == "spell-rev-lightning-bolt"), None)
        self.assertIsNotNone(lightning, "missing approved Lightning Bolt choice group")
        self.assertEqual(lightning.options, (RowKey("SPELL_REV", 80), RowKey("SPELL_REV", 81)))
        self.assertEqual(lightning.default, "SPELL_REV:80")

    def test_current_map_freezes_reviewed_omissions_and_atomic_bundles(self) -> None:
        root = Path(__file__).resolve().parents[2]
        curation_map = load_curation_map(root / "manifest" / "curation-map.toml")
        targets = {target.target_id: target for target in curation_map.targets}

        omission_rows = {
            row
            for target in curation_map.targets
            if target.kind == "omission"
            for row in target.rows
        }
        self.assertEqual(
            omission_rows,
            {
                RowKey("ASCENSION", 40),
                RowKey("AURA_BG1_2_EET", 2),
                RowKey("AURA_BG1_2_EET", 3),
                RowKey("AURA_BG1_2_EET", 7),
                RowKey("AURA_BG1_2_EET", 14),
                RowKey("C0WARLOCK", 0),
                RowKey("DLCMERGER", 2),
                RowKey("DLCMERGER", 3),
                RowKey("HIDDENGAMEPLAYOPTIONS", 40),
                RowKey("IWDIFICATION", 120),
                RowKey("UB", 19),
            },
        )
        self.assertEqual(
            targets["feature:safana:mandatory-components"].rows,
            (RowKey("SAFANA", 0),),
        )
        self.assertEqual(
            targets["feature:artisanskitpack:component-5110"].rows,
            (RowKey("ARTISANSKITPACK", 5110),),
        )
        self.assertEqual(
            targets["feature:artisanskitpack-npc:component-20002"].rows,
            (RowKey("ARTISANSKITPACK_NPC", 20002),),
        )
        self.assertEqual(
            targets["feature:chriz-bg-rebalance:tempus-bundle"].rows,
            (
                RowKey("CHRIZ-BG-REBALANCE", 400),
                RowKey("CHRIZ-BG-REBALANCE", 401),
                RowKey("CHRIZ-BG-REBALANCE", 404),
                RowKey("CHRIZ-BG-REBALANCE", 405),
                RowKey("CHRIZ-BG-REBALANCE", 407),
                RowKey("CHRIZ-BG-REBALANCE", 408),
            ),
        )
        self.assertEqual(
            targets["feature:buffbot:mandatory-components"].rows,
            (RowKey("BUFFBOT", 1), RowKey("BUFFBOT", 0)),
        )
        self.assertEqual(
            targets["feature:eeex:mandatory-components"].rows,
            tuple(RowKey("EEEX", component_id) for component_id in range(9)),
        )
        self.assertEqual(
            targets["feature:chriz-sod-remix:mandatory-components"].rows,
            tuple(
                RowKey("CHRIZ-SOD-REMIX", component_id)
                for component_id in (
                    100,
                    110,
                    115,
                    120,
                    130,
                    135,
                    140,
                    150,
                    145,
                    160,
                    170,
                    180,
                    175,
                    177,
                    185,
                    190,
                    195,
                    210,
                    197,
                    198,
                    187,
                    200,
                    215,
                    220,
                    225,
                    245,
                    230,
                    240,
                    250,
                    255,
                    256,
                    260,
                    265,
                    266,
                    270,
                    280,
                    285,
                    290,
                    900,
                    910,
                )
            ),
        )
        self.assertEqual(
            targets["feature:chriz-bg-modpack:component-610"].rows,
            (RowKey("CHRIZ-BG-MODPACK", 610),),
        )
        hexxat = next(
            group
            for group in curation_map.choice_groups
            if group.group_id == "choice:chriz-bg-modpack:hexxat-class"
        )
        self.assertEqual(
            hexxat.options,
            tuple(RowKey("CHRIZ-BG-MODPACK", component) for component in (221, 222, 223)),
        )
        self.assertEqual(hexxat.default, "CHRIZ-BG-MODPACK:221")


if __name__ == "__main__":
    unittest.main()
