import os
import shutil
import tempfile
import unittest
from pathlib import Path

from tools.hotpatch_pilot import Unsupported
from tools.run_hotpatch_pilot import append_event, read_events, metadata, restore_files, changed_paths


class TransactionTests(unittest.TestCase):
    def test_chained_events_detect_tampering(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            append_event(root, 'prepared', {'patch': 'one'})
            append_event(root, 'applied', {'patch': 'one'})
            self.assertEqual([e['kind'] for e in read_events(root)], ['prepared', 'applied'])
            first = root / 'events/000001.json'
            first.write_text(first.read_text().replace('one', 'two'))
            with self.assertRaises(Unsupported):
                read_events(root)

    def test_interrupted_write_can_restore_exact_bytes_and_remove_new_files(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            game, backup = root / 'game', root / 'backup'
            game.mkdir(); backup.mkdir()
            (game / 'old').write_bytes(b'original')
            shutil.copy2(game / 'old', backup / 'old')
            before = metadata(game)
            (game / 'old').write_bytes(b'partial')
            (game / 'new').mkdir()
            (game / 'new/file').write_bytes(b'incomplete')
            restore_files(game, backup, before, {'old'}, ('new',))
            self.assertEqual((game / 'old').read_bytes(), b'original')
            self.assertEqual(metadata(game), before)

    def test_undeclared_change_is_not_called_restored(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            game, backup = root / 'game', root / 'backup'
            game.mkdir(); backup.mkdir()
            before = metadata(game)
            (game / 'unexpected').write_bytes(b'leave evidence')
            with self.assertRaises(Unsupported):
                restore_files(game, backup, before, set(), ('new',))
            self.assertTrue((game / 'unexpected').exists())

    def test_restore_recreates_a_deleted_declared_file(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            game, backup = root / 'game', root / 'backup'
            game.mkdir(); backup.mkdir()
            (game / 'old').write_bytes(b'original')
            shutil.copy2(game / 'old', backup / 'old')
            before = metadata(game)
            (game / 'old').unlink()
            restore_files(game, backup, before, {'old'}, ())
            self.assertEqual(metadata(game), before)

    def test_hardlinked_write_target_is_rejected(self):
        from tools.hotpatch_pilot import direct_file
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / 'one').write_bytes(b'original')
            os.link(root / 'one', root / 'two')
            with self.assertRaises(Unsupported):
                direct_file(root / 'two')

    def test_changed_paths_include_removed_created_and_modified(self):
        self.assertEqual(changed_paths({'a': [1, 1], 'b': [2, 2]}, {'a': [1, 2], 'c': [2, 2]}), {'a', 'b', 'c'})

    def test_gap_in_events_is_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            append_event(root, 'prepared', {})
            (root / 'events/000001.json').rename(root / 'events/000002.json')
            with self.assertRaises(Unsupported):
                read_events(root)
