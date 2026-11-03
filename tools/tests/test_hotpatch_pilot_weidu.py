"""Opt-in real-WeiDU integration evidence for the Artisan description pilot.

The test has no default tool path: set the two explicit environment variables to
run it.  It constructs a disposable BG2EE-shaped KEY/BIF/TLK game and never
points WeiDU at an installed game or user profile.
"""
from __future__ import annotations

import hashlib
import os
import shutil
import struct
import subprocess
import tempfile
import unittest
from pathlib import Path

from tools.hotpatch_pilot import (
    SUPPORTED_VERSION,
    TABLES,
    WEIDU_SHA256,
    adapter_targets,
    file_hash,
    parse_log,
    plan_tables,
    verify_tables,
)


ADAPTER_ENV = "CEBG_PILOT_ADAPTER"
WEIDU_ENV = "CEBG_PILOT_WEIDU"
BASELINE_ROW = "~BASE/SETUP-BASE.TP2~ #0 #0 // fixture baseline: 1.0"
COMPONENT_ROWS = (
    "~ARTISANSKITPACK/ARTISANSKITPACK.TP2~ #0 #7004 // Assassin: chriz-v1.3.0",
    "~ARTISANSKITPACK/ARTISANSKITPACK.TP2~ #0 #2010 // Archer: chriz-v1.3.0",
    "~ARTISANSKITPACK/ARTISANSKITPACK.TP2~ #0 #2012 // Beast Master: chriz-v1.3.0",
)
TARGETS = (("ASSASIN", 7004), ("FERALAN", 2010), ("BEASTMASTER", 2012))


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def snapshot(root: Path) -> dict[str, str]:
    return {
        path.relative_to(root).as_posix().lower(): sha256(path)
        for path in root.rglob("*")
        if path.is_file()
    }


def changed(before: dict[str, str], after: dict[str, str]) -> list[str]:
    return sorted(path for path in set(before) | set(after) if before.get(path) != after.get(path))


def tlk(entries: int = 96) -> bytes:
    """A TLK V1 with nonempty entries used by the fixture's HELP references."""
    header_len, entry_len = 18, 26
    text = b"".join(f"description {index}".encode("ascii") for index in range(entries))
    out = bytearray(b"TLK V1  ")
    out += struct.pack("<HII", 0, entries, header_len + entry_len * entries)
    offset = 0
    for index in range(entries):
        value = f"description {index}".encode("ascii")
        out += b"\0" * 18 + struct.pack("<II", offset, len(value))
        offset += len(value)
    return bytes(out) + text


def fake_key_bif(game: Path) -> None:
    """Minimal KEY/BIFF with OH6000.ARE, sufficient for GAME_IS bg2ee."""
    data = game / "data"
    data.mkdir()
    area = b"AREAV1.0" + b"\0" * 0x11C
    bif = bytearray(b"BIFFV1  ") + struct.pack("<III", 1, 0, 0x14)
    bif += struct.pack("<IIIHH", 0, 0x24, len(area), 1010, 0) + area
    (data / "fake.bif").write_bytes(bif)
    name = b"data/fake.bif\0"
    resource_table = 0x18 + 12 + len(name)
    key = bytearray(b"KEY V1  ") + struct.pack("<IIII", 1, 1, 0x18, resource_table)
    key += struct.pack("<IIHH", len(bif), 0x24, len(name), 1) + name
    key += b"OH6000\0\0" + struct.pack("<HI", 1010, 0)
    (game / "chitin.key").write_bytes(key)


def table(rows: list[tuple[str, str]], *, suffix: str) -> bytes:
    result = ["2DA V1.0", "-1", " CLASSID KITID LOWER DESCSTR MIXED"]
    result.extend(f"{name} 1 1 1 {description} {suffix}" for name, description in rows)
    return ("\r\n".join(result) + "\r\n").encode("ascii")


def kitlist() -> bytes:
    return (
        "2DA V1.0\r\n*\r\n ROWNAME LOWER MIXED HELP ABILITIES\r\n"
        "0 ASSASIN 1 1 40 CLABTH01\r\n"
        "1 FERALAN 1 1 41 CLABRN01\r\n"
        "2 BEASTMASTER 1 1 42 CLABRN02\r\n"
    ).encode("ascii")


class HotpatchPilotWeiDUTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        adapter = os.environ.get(ADAPTER_ENV)
        weidu = os.environ.get(WEIDU_ENV)
        if not adapter or not weidu:
            raise unittest.SkipTest(f"set {ADAPTER_ENV} and {WEIDU_ENV} for real-WeiDU pilot evidence")
        cls.adapter = Path(adapter).resolve()
        cls.weidu = Path(weidu).resolve()
        if file_hash(cls.weidu) != WEIDU_SHA256:
            raise AssertionError("CEBG_PILOT_WEIDU does not match the pinned x64 WeiDU hash")
        cls.targets = adapter_targets(cls.adapter)
        cls.assert_targets = TARGETS

    def _run(self, game: Path, debug: Path, *operation: str) -> subprocess.CompletedProcess[bytes]:
        command = [
            str(self.weidu),
            "AKCB_KIT_DESCRIPTIONS/setup-AKCB_KIT_DESCRIPTIONS.tp2",
            *operation,
            "--language", "0", "--use-lang", "en_US", "--no-exit-pause",
            "--noautoupdate", "--log", str(debug),
        ]
        kwargs: dict[str, object] = dict(cwd=game, stdin=subprocess.DEVNULL,
                                         stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                         check=False, timeout=180)
        if os.name == "nt":
            kwargs["creationflags"] = subprocess.CREATE_NO_WINDOW
        return subprocess.run(command, **kwargs)

    def _build(self, root: Path) -> Path:
        game = root / "fake-bg2ee"
        game.mkdir()
        fake_key_bif(game)
        (game / "override").mkdir()
        (game / "lang" / "en_US").mkdir(parents=True)
        (game / "engine.lua").write_text("engine_name = 'AKCB synthetic fixture'\n", encoding="ascii")
        data = tlk()
        (game / "dialog.tlk").write_bytes(data)
        (game / "lang" / "en_US" / "dialog.tlk").write_bytes(data)
        (game / "weidu.conf").write_text("lang_dir = en_us\n", encoding="ascii")
        (game / "WeiDU.log").write_text("\n".join((BASELINE_ROW, *COMPONENT_ROWS)) + "\n", encoding="utf-8")
        (game / "BASE").mkdir()
        (game / "BASE" / "SETUP-BASE.TP2").write_text(
            "BACKUP ~BASE/backup~\nVERSION ~1.0~\nBEGIN ~fixture baseline~ DESIGNATED 0\n",
            encoding="ascii",
        )
        (game / "ArtisansKitpack").mkdir()
        (game / "ArtisansKitpack" / "ArtisansKitpack.TP2").write_text(
            "BACKUP ~ArtisansKitpack/backup~\nVERSION ~chriz-v1.3.0~\n"
            "BEGIN ~Assassin~ DESIGNATED 7004\n"
            "BEGIN ~Archer~ DESIGNATED 2010\n"
            "BEGIN ~Beast Master~ DESIGNATED 2012\n",
            encoding="ascii",
        )
        # WeiDU re-reads historic log annotations through setup executables.
        # These are inert copied test tools, never executed by this test.
        shutil.copy2(self.weidu, game / "Setup-BASE.exe")
        shutil.copy2(self.weidu, game / "Setup-ArtisansKitpack.exe")
        (game / "override" / "KITLIST.2DA").write_bytes(kitlist())
        differing = [("ASSASSIN", "10"), ("ARCHER", "11"), ("BEAST_MASTER", "12"), ("OTHER", "90")]
        fixed = [("ASSASSIN", "40"), ("ARCHER", "41"), ("BEAST_MASTER", "42"), ("OTHER", "90")]
        (game / "override" / "BGCLATXT.2DA").write_bytes(table(differing, suffix="7"))
        (game / "override" / "CLASTEXT.2DA").write_bytes(table(fixed, suffix="8"))
        (game / "override" / "SODCLTXT.2DA").write_bytes(table(differing, suffix="9"))
        shutil.copytree(self.adapter, game / "AKCB_KIT_DESCRIPTIONS")
        return game

    def test_apply_is_scoped_idempotent_by_eligibility_and_uninstalls_byte_exactly(self) -> None:
        with tempfile.TemporaryDirectory(prefix="cebg-akcb-weidu-") as temporary:
            game = self._build(Path(temporary))
            override = game / "override"
            original_tables = {name: (override / name).read_bytes() for name in TABLES}
            original_tlks = {path: (game / path).read_bytes() for path in ("dialog.tlk", "lang/en_US/dialog.tlk")}
            original_log = (game / "WeiDU.log").read_bytes()
            plan = plan_tables(kitlist=(override / "KITLIST.2DA").read_bytes(),
                               tables=original_tables, log=original_log,
                               entry_count=96, targets=self.targets)
            self.assertEqual(plan["status"], "applicable")
            self.assertEqual(set(plan["selected"]), {"ASSASIN", "FERALAN", "BEASTMASTER"})
            self.assertEqual({(item["file"], item["row"]) for item in plan["changes"]}, {
                ("BGCLATXT.2DA", "ASSASSIN"), ("BGCLATXT.2DA", "ARCHER"), ("BGCLATXT.2DA", "BEAST_MASTER"),
                ("SODCLTXT.2DA", "ASSASSIN"), ("SODCLTXT.2DA", "ARCHER"), ("SODCLTXT.2DA", "BEAST_MASTER"),
            })
            before = snapshot(game)
            applied = self._run(game, Path(temporary) / "apply.debug", "--force-install-list", "0")
            self.assertEqual(applied.returncode, 0, applied.stdout.decode("utf-8", "replace"))
            actual_tables = {name: (override / name).read_bytes() for name in TABLES}
            verify_tables(plan, actual_tables)
            self.assertEqual({path: (game / path).read_bytes() for path in original_tlks}, original_tlks)
            original_ids = [row[:3] for row in parse_log(original_log)]
            applied_ids = [row[:3] for row in parse_log((game / 'WeiDU.log').read_bytes())]
            self.assertEqual(applied_ids[:-1], original_ids)
            self.assertEqual(applied_ids[-1], ('akcb_kit_descriptions/setup-akcb_kit_descriptions.tp2', 0, 0))
            # The disposable stub does not ship Artisan translations, so WeiDU
            # renders its historical comments as ???. Use the immutable pre-run
            # provenance snapshot for this eligibility-only check.
            post_plan = plan_tables(kitlist=(override / "KITLIST.2DA").read_bytes(),
                                    tables=actual_tables, log=original_log,
                                    entry_count=96, targets=self.targets)
            self.assertEqual(post_plan["status"], "already_fixed")
            self.assertEqual(post_plan["changes"], [])  # do not install the adapter twice
            active = [line for line in (game / "WeiDU.log").read_text(encoding="utf-8").splitlines() if not line.startswith("//")]
            self.assertRegex(active[-2], r"^~ARTISANSKITPACK/ARTISANSKITPACK\.TP2~ #0 #2012 //")
            self.assertIn("AKCB_KIT_DESCRIPTIONS", active[-1])
            apply_changes = changed(before, snapshot(game))
            self.assertEqual((game / "weidu.conf").read_text(encoding="ascii"), "lang_dir = en_us\n")
            self.assertEqual(apply_changes, [
                "akcb_kit_descriptions/backup/0/args.0",
                "akcb_kit_descriptions/backup/0/args.0.text",
                "akcb_kit_descriptions/backup/0/bgclatxt.2da",
                "akcb_kit_descriptions/backup/0/mappings.0",
                "akcb_kit_descriptions/backup/0/move.0",
                "akcb_kit_descriptions/backup/0/other.0",
                "akcb_kit_descriptions/backup/0/readln.0",
                "akcb_kit_descriptions/backup/0/readln.0.text",
                "akcb_kit_descriptions/backup/0/sodcltxt.2da",
                "akcb_kit_descriptions/backup/0/tlkpath.0",
                "akcb_kit_descriptions/backup/0/uninstall.0",
                "override/bgclatxt.2da",
                "override/sodcltxt.2da",
                "weidu.conf",
                "weidu.log",
            ])
            self.assertTrue((Path(temporary) / "apply.debug").is_file())
            self.assertIn("weidu.log", apply_changes)
            self.assertIn("override/bgclatxt.2da", apply_changes)
            self.assertIn("override/sodcltxt.2da", apply_changes)
            self.assertNotIn("override/clastext.2da", apply_changes)
            self.assertNotIn("dialog.tlk", apply_changes)
            self.assertNotIn("lang/en_us/dialog.tlk", apply_changes)
            removed = self._run(game, Path(temporary) / "uninstall.debug", "--force-uninstall-list", "0")
            self.assertEqual(removed.returncode, 0, removed.stdout.decode("utf-8", "replace"))
            self.assertEqual({name: (override / name).read_bytes() for name in TABLES}, original_tables)
            self.assertEqual({path: (game / path).read_bytes() for path in original_tlks}, original_tlks)
            self.assertEqual([row[:3] for row in parse_log((game / 'WeiDU.log').read_bytes())], original_ids)
            remaining = [line for line in (game / "WeiDU.log").read_text(encoding="utf-8").splitlines() if not line.startswith("//")]
            self.assertRegex(remaining[-1], r"^~ARTISANSKITPACK/ARTISANSKITPACK\.TP2~ #0 #2012 //")


if __name__ == "__main__":
    unittest.main()
