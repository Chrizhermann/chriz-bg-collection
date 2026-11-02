"""Read-only eligibility/postconditions for the supervised Artisan description pilot.

This is deliberately not a generic mod updater. Mod code stays in its owner repo;
the pilot accepts two exact reviewed adapter files and one known installed version.
No save file is read or edited by this module.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import stat
import struct
from pathlib import Path


ADAPTER = 'AKCB_KIT_DESCRIPTIONS'
PINS = {
    'setup-AKCB_KIT_DESCRIPTIONS.tp2': '02545032703c71d6b687461bb6dd7164f970274d610184dd9e176a033bf213aa',
    'lib/kit_strref.tpa': '7588106d681f53b920c8bdecb55d8d2c7e864b2cd972c463f7a00aac61e55400',
}
TABLES = ('BGCLATXT.2DA', 'CLASTEXT.2DA', 'SODCLTXT.2DA')
ALIASES = {'ASSASIN': 'ASSASSIN', 'FERALAN': 'ARCHER',
           'BEASTMASTER': 'BEAST_MASTER', 'BEAST_FRIEND': 'AVENGER'}
WEIDU_SHA256 = 'ad70f5897a6d0ba4b0d226f845a9b14cf345f56cc9697ca8d05cac9fe4932c1a'
SUPPORTED_VERSION = 'chriz-v1.3.0'


class Unsupported(ValueError):
    """Stop before mutation, or report an unverified postcondition."""


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_hash(path: Path) -> str:
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def direct_file(path: Path) -> Path:
    """Pilot game/tool paths must have no reparse ancestors or hardlinks."""
    for parent in (path, *path.parents):
        info = parent.lstat()
        if stat.S_ISLNK(info.st_mode) or getattr(info, 'st_file_attributes', 0) & 0x400:
            raise Unsupported(f'Linked/reparse path: {parent}')
    info = path.stat()
    if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1:
        raise Unsupported(f'Not an independent regular file: {path}')
    return path


def parse_log(data: bytes) -> list[tuple[str, int, int, str]]:
    rows = []
    for line in data.decode('utf-8-sig').splitlines():
        if not line.strip() or line.lstrip().startswith('//'):
            continue
        match = re.fullmatch(r'\s*~([^~]+)~\s+#(\d+)\s+#(\d+)(?:\s*//\s*(.*))?\s*', line)
        if not match:
            raise Unsupported('Malformed WeiDU.log entry')
        path, language, component, comment = match.groups()
        rows.append((path.replace('\\', '/').lower(), int(language), int(component), comment or ''))
    return rows


def parse_table(data: bytes) -> list[list[str]]:
    lines = [line.split('//', 1)[0].split() for line in data.decode('ascii').splitlines()]
    lines = [line for line in lines if line]
    if len(lines) < 4 or [x.upper() for x in lines[0]] != ['2DA', 'V1.0'] or len(lines[1]) != 1:
        raise Unsupported('Invalid 2DA header')
    width = len(lines[2]) + 1
    if width < 5 or any(len(row) != width for row in lines[3:]):
        raise Unsupported('Unsupported/ragged 2DA layout')
    return lines


def plan_tables(*, kitlist: bytes, tables: dict[str, bytes], log: bytes,
                entry_count: int, targets: list[tuple[str, int]]) -> dict:
    kits = parse_table(kitlist)
    if kits[2][0].upper() != 'ROWNAME' or kits[2][3].upper() != 'HELP':
        raise Unsupported('Unsupported KITLIST column layout')
    installed = {}
    for path, language, component, comment in parse_log(log):
        if path == 'artisanskitpack/artisanskitpack.tp2':
            if component in installed:
                raise Unsupported('Duplicate Artisan component')
            installed[component] = (language, comment.rsplit(': ', 1)[-1])
    selected = {}
    for symbol, component in targets:
        if component not in installed:
            continue
        if installed[component] != (0, SUPPORTED_VERSION):
            raise Unsupported(f'Component {component}: unsupported language/version')
        matches = [row for row in kits[3:] if row[1].upper() == symbol]
        if len(matches) != 1:
            raise Unsupported(f'Missing/ambiguous KITLIST symbol: {symbol}')
        value = matches[0][4]
        if not value.isdecimal() or not 0 <= int(value) < entry_count:
            raise Unsupported(f'Invalid HELP reference: {symbol}')
        selected[symbol] = value
    if not selected:
        raise Unsupported('No installed supported Artisan components')
    changes, expected = [], {}
    for filename, data in tables.items():
        rows = parse_table(data)
        if rows[2][3].upper() != 'DESCSTR':
            raise Unsupported(f'Unsupported description columns: {filename}')
        names = [row[0].upper() for row in rows[3:]]
        if len(set(names)) != len(names):
            raise Unsupported(f'Duplicate class row: {filename}')
        for row in rows[3:]:
            for symbol, value in selected.items():
                if row[0].upper() in (symbol, ALIASES.get(symbol, symbol)) and row[4] != value:
                    changes.append(dict(file=filename, row=row[0], before=row[4], after=value))
                    row[4] = value
        expected[filename] = rows
    return dict(status='applicable' if changes else 'already_fixed', changes=changes,
                selected=selected, expected=expected)


def verify_tables(plan: dict, tables: dict[str, bytes]) -> None:
    if set(tables) != set(plan['expected']):
        raise Unsupported('Postcondition table set changed')
    for name, data in tables.items():
        if parse_table(data) != plan['expected'][name]:
            raise Unsupported(f'Unexpected table edit: {name}')


def adapter_targets(adapter: Path) -> list[tuple[str, int]]:
    for relative, expected in PINS.items():
        if file_hash(direct_file(adapter / relative)) != expected:
            raise Unsupported(f'Adapter identity mismatch: {relative}')
    text = (adapter / 'setup-AKCB_KIT_DESCRIPTIONS.tp2').read_text(encoding='utf-8')
    targets = [(name, int(number)) for name, number in re.findall(r'~([A-Z_0-9]+)~,\s*~(\d+)~\s*=>\s*1', text)]
    if len(targets) != 25:
        raise Unsupported('Unexpected adapter target mapping')
    return targets


def inspect(game: Path, adapter: Path) -> dict:
    paths = ['WeiDU.log', 'weidu.conf', 'engine.lua', 'Baldur.exe', 'chitin.key',
             'override/KITLIST.2DA', *(f'override/{name}' for name in TABLES)]
    tlks = [path.relative_to(game).as_posix() for path in game.rglob('*.tlk')]
    if 'lang/en_US/dialog.tlk'.lower() not in [p.lower() for p in tlks]:
        raise Unsupported('Expected English TLK absent')
    # No archive materialization, no touching alternate-language TLKs.
    paths.extend(tlks)
    for relative in paths:
        direct_file(game / relative)
    if (game / 'weidu.conf').read_text().strip().lower() != 'lang_dir = en_us':
        raise Unsupported('Pilot requires the existing en_US configuration')
    tlk = game / 'lang/en_US/dialog.tlk'
    with tlk.open('rb') as stream:
        header = stream.read(18)
    if len(header) != 18 or header[:8] != b'TLK V1  ':
        raise Unsupported('Invalid TLK header')
    entry_count, text_start = struct.unpack_from('<II', header, 10)
    if not 18 + entry_count * 26 <= text_start <= tlk.stat().st_size:
        raise Unsupported('Invalid TLK table bounds')
    plan = plan_tables(kitlist=(game / 'override/KITLIST.2DA').read_bytes(),
                       tables={name: (game / 'override' / name).read_bytes() for name in TABLES},
                       log=(game / 'WeiDU.log').read_bytes(), entry_count=entry_count,
                       targets=adapter_targets(adapter))
    # Resolve each selected string's actual text bounds, not just its numeric index.
    with tlk.open('rb') as stream:
        for value in plan['selected'].values():
            stream.seek(18 + int(value) * 26)
            entry = stream.read(26)
            offset, length = struct.unpack_from('<II', entry, 18)
            if not length or text_start + offset + length > tlk.stat().st_size:
                raise Unsupported(f'Invalid/empty HELP text: {value}')
    plan.update(game=str(game), hashes={p: file_hash(game / p) for p in paths},
                tlks=tlks, entry_count=entry_count,
                component_rows=[list(row) for row in parse_log((game / 'WeiDU.log').read_bytes())])
    return plan


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--game', required=True, type=Path)
    parser.add_argument('--adapter', required=True, type=Path)
    args = parser.parse_args()
    plan = inspect(args.game, args.adapter)
    print(json.dumps({k: v for k, v in plan.items() if k not in ('expected', 'component_rows')}, indent=2))


if __name__ == '__main__':
    main()
