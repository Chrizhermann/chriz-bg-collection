import unittest

from tools.hotpatch_pilot import Unsupported, plan_tables, parse_log, parse_table, verify_tables


KITLIST = b'2DA V1.0\n*\n ROWNAME LOWER MIXED HELP\n0 ASSASIN 1 2 40\n1 FERALAN 3 4 41\n'
CLASSES = b'2DA V1.0\n-1\n CLASSID KITID LOWER DESCSTR MIXED\nASSASSIN 4 1 2 10 3\nARCHER 12 2 3 11 4\nOTHER 1 0 1 99 2\n'
LOG = b'// install\n~ARTISANSKITPACK/ARTISANSKITPACK.TP2~ #0 #7004 // Assassin: chriz-v1.3.0\n'
TARGETS = [('ASSASIN', 7004), ('FERALAN', 2010)]


class HotpatchPilotTests(unittest.TestCase):
    def plan(self, **overrides):
        args = dict(kitlist=KITLIST, tables={'BGCLATXT.2DA': CLASSES},
                    log=LOG, entry_count=100, targets=TARGETS)
        args.update(overrides)
        return plan_tables(**args)

    def test_only_installed_component_and_owned_cell_change(self):
        result = self.plan()
        self.assertEqual(result['status'], 'applicable')
        self.assertEqual(result['changes'], [{'file': 'BGCLATXT.2DA', 'row': 'ASSASSIN', 'before': '10', 'after': '40'}])
        self.assertEqual(result['expected']['BGCLATXT.2DA'][4][4], '11')

    def test_already_fixed_is_noop(self):
        self.assertEqual(self.plan(tables={'BGCLATXT.2DA': CLASSES.replace(b'2 10 3', b'2 40 3')})['status'], 'already_fixed')

    def test_missing_component_does_not_add_new_content(self):
        with self.assertRaises(Unsupported):
            self.plan(log=b'')

    def test_unknown_or_missing_version_is_unsupported(self):
        for version in (b'other', b'', b'chriz-v1.5.0'):
            with self.subTest(version=version), self.assertRaises(Unsupported):
                self.plan(log=LOG.replace(b'chriz-v1.3.0', version))

    def test_alias_and_second_selected_component(self):
        result = self.plan(log=LOG + b'~ArtisansKitpack\\ArtisansKitpack.TP2~ #0 #2010 // Archer: chriz-v1.3.0\n')
        self.assertEqual(len(result['changes']), 2)

    def test_reference_bounds_and_missing_symbol(self):
        for bad in (b'-1', b'100', b'oops'):
            with self.subTest(bad=bad), self.assertRaises(Unsupported):
                self.plan(kitlist=KITLIST.replace(b'2 40', b'2 ' + bad))
        with self.assertRaises(Unsupported):
            self.plan(kitlist=KITLIST.replace(b'ASSASIN', b'NOT_ASSASIN'))

    def test_duplicate_symbols_or_rows_rejected(self):
        with self.assertRaises(Unsupported):
            self.plan(kitlist=KITLIST + b'2 ASSASIN 1 2 40\n')
        with self.assertRaises(Unsupported):
            self.plan(tables={'BGCLATXT.2DA': CLASSES + b'ASSASSIN 4 1 2 10 3\n'})

    def test_shifted_columns_and_ragged_rows_rejected(self):
        for bad in (CLASSES.replace(b'DESCSTR MIXED', b'MIXED DESCSTR'), CLASSES + b'SHORT 1 2\n'):
            with self.assertRaises(Unsupported):
                self.plan(tables={'BGCLATXT.2DA': bad})

    def test_comment_rows_and_case_are_supported(self):
        result = self.plan(tables={'BGCLATXT.2DA': CLASSES.lower() + b'// example only\n'})
        self.assertEqual(result['changes'][0]['after'], '40')

    def test_postcondition_allows_whitespace_not_unrelated_changes(self):
        result = self.plan()
        after = CLASSES.replace(b'2 10 3', b'2    40   3')
        verify_tables(result, {'BGCLATXT.2DA': after})
        with self.assertRaises(Unsupported):
            verify_tables(result, {'BGCLATXT.2DA': after.replace(b'99', b'98')})

    def test_parse_log_preserves_order_and_rejects_malformed_rows(self):
        self.assertEqual(parse_log(LOG)[0][:3], ('artisanskitpack/artisanskitpack.tp2', 0, 7004))
        with self.assertRaises(Unsupported):
            parse_log(b'~MOD.TP2~ #0 broken')

    def test_bad_table_signature_rejected(self):
        with self.assertRaises(Unsupported):
            parse_table(b'not a table')


if __name__ == '__main__':
    unittest.main()
