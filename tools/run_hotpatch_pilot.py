"""Supervised Windows-only RC pilot. Not exposed by the public installer.

Only this explicitly authorized RC, this reviewed adapter and this complete
pre-patch backup are accepted. No games are launched and no saves are edited.
Run with `python -m tools.run_hotpatch_pilot inspect|apply|rollback|recover|audit`.
"""
from __future__ import annotations

import argparse
import contextlib
import ctypes
import json
import os
import shutil
import stat
import subprocess
import uuid
from datetime import datetime, timezone
from pathlib import Path

from tools.hotpatch_pilot import (ADAPTER, PINS, TABLES, WEIDU_SHA256, Unsupported,
                                 direct_file, file_hash, digest, inspect, parse_log, verify_tables)

GAME = Path('C:/BG-EET-RC-20260903/game')
STATE = GAME.parent / '.chriz/patches/kit-links-pilot-v1'
BACKUP = Path('C:/Users/chris/Games/CEBG-Backups/RC-20260903-before-kit-links-20260920/game')
OWNER = Path('C:/src/private/The-Artisan-s-Kitpack-Chriz-Balance-Patch/live-patch') / ADAPTER
TOOL = GAME.parent / '.chriz/extracted/chriz-bg-rebalance-v0.3.0/chriz-bg-rebalance-v0.3.0/weidu.exe'
ORIGINAL_LOG = '37a828a8a628abe9acbc26cd6062f005e1d7317e772373e895179665067435d4'
PATCH_ID = 'artisan-campaign-description-links-1.0'


def json_new(path: Path, value: object) -> None:
    """Durable file content, atomic publication; caller holds the single-writer lock."""
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        raise Unsupported(f'Will not replace evidence: {path}')
    temporary = path.with_name(path.name + '.' + uuid.uuid4().hex + '.tmp')
    with temporary.open('xb') as stream:
        stream.write(json.dumps(value, indent=2, sort_keys=True).encode('utf-8'))
        stream.flush()
        os.fsync(stream.fileno())
    temporary.rename(path)


def read_events(state: Path) -> list[dict]:
    events, previous = [], None
    for number, path in enumerate(sorted((state / 'events').glob('*.json')), 1):
        event = json.loads(path.read_text(encoding='utf-8'))
        if path.name != f'{number:06}.json' or event['previous'] != previous or event['sequence'] != number:
            raise Unsupported('Patch event chain mismatch')
        events.append(event)
        previous = file_hash(path)
    return events


def append_event(state: Path, kind: str, data: dict) -> dict:
    events = read_events(state)
    previous = file_hash(state / f'events/{len(events):06}.json') if events else None
    event = dict(sequence=len(events) + 1, previous=previous, kind=kind, data=data,
                 time=datetime.now(timezone.utc).isoformat())
    json_new(state / f'events/{len(events) + 1:06}.json', event)
    return event


def metadata(root: Path) -> dict:
    """One-off pilot audit, not a startup scan. Never follow links."""
    result = {}
    for folder, directories, files in os.walk(root, followlinks=False):
        for name in directories + files:
            path = Path(folder) / name
            info = path.lstat()
            if stat.S_ISLNK(info.st_mode) or getattr(info, 'st_file_attributes', 0) & 0x400:
                raise Unsupported(f'Unexpected linked game/backup path: {path}')
            result[path.relative_to(root).as_posix().lower()] = (
                None if stat.S_ISDIR(info.st_mode) else [info.st_size, info.st_mtime_ns])
    return result


def changed_paths(before: dict, after: dict) -> set[str]:
    return {name for name in before.keys() | after.keys()
            if name not in before or name not in after or before[name] != after[name]}


def allowed(name: str, files: set[str], new_roots: tuple[str, ...]) -> bool:
    return name in files or any(name == root or name.startswith(root + '/') for root in new_roots)


def restore_files(game: Path, backup: Path, before: dict, files: set[str],
                  new_roots: tuple[str, ...]) -> None:
    """Restore only the declared set; never claim to undo an undeclared write."""
    current = metadata(game)
    unexpected = {p for p in changed_paths(before, current) if not allowed(p, files, new_roots)}
    if unexpected:
        raise Unsupported(f'Undeclared changes; automatic restore blocked: {sorted(unexpected)[:10]}')
    for name in files:
        if name in before:
            if (game / name).exists():
                direct_file(game / name)
            else:
                # Deleted/truncated declared outputs are recoverable too; parent
                # paths still must be ordinary directories inside the target.
                for parent in (game / name).parents:
                    info = parent.lstat()
                    if stat.S_ISLNK(info.st_mode) or getattr(info, 'st_file_attributes', 0) & 0x400:
                        raise Unsupported(f'Linked restore parent: {parent}')
            shutil.copy2(direct_file(backup / name), game / name)
    created = set(current) - set(before)
    # Each exact path was observed under game and allowlisted above. No recursive delete.
    for name in sorted(created, key=lambda p: (p.count('/'), p), reverse=True):
        path = game / name
        if path.is_dir():
            path.rmdir()
        else:
            direct_file(path).unlink()
    if metadata(game) != before:
        raise Unsupported('Restoration did not reproduce the pre-patch metadata')


@contextlib.contextmanager
def pilot_lock():
    import msvcrt
    for part in (STATE, *STATE.parents):
        try:
            info = part.lstat()
        except FileNotFoundError:
            continue
        if not stat.S_ISDIR(info.st_mode) or getattr(info, 'st_file_attributes', 0) & 0x400:
            raise Unsupported(f'Unsafe pilot state directory: {part}')
    STATE.mkdir(parents=True, exist_ok=True)
    # Reject links in the state hierarchy as well as game paths.
    lock_path = STATE / 'pilot.lock'
    if lock_path.exists() or lock_path.is_symlink():
        direct_file(lock_path)
    with lock_path.open('a+b') as stream:
        if not lock_path.stat().st_size:
            stream.write(b'0'); stream.flush()
        direct_file(lock_path)
        stream.seek(0)
        msvcrt.locking(stream.fileno(), msvcrt.LK_NBLCK, 1)
        try:
            yield
        finally:
            stream.seek(0)
            msvcrt.locking(stream.fileno(), msvcrt.LK_UNLCK, 1)


def closed_game(paths: list[Path]) -> None:
    # Include explicitly addressed external WeiDU processes, not just EXEs in game.
    command = "$p = @(Get-CimInstance Win32_Process | Where-Object { ($_.ExecutablePath -and $_.ExecutablePath.StartsWith('C:\\BG-EET-RC-20260903\\game\\', [StringComparison]::OrdinalIgnoreCase)) -or ($_.Name -match '^(weidu|setup-.+)\\.exe$' -and $_.CommandLine -like '*C:\\BG-EET-RC-20260903\\game*') }); if ($p.Count) { $p | Select-Object ProcessId,Name | ConvertTo-Json -Compress; exit 1 }"
    result = subprocess.run(['powershell.exe', '-NoProfile', '-NonInteractive', '-Command', command],
                            capture_output=True, text=True, timeout=30,
                            creationflags=subprocess.CREATE_NO_WINDOW)
    if result.returncode:
        raise Unsupported(f'Target process active or process query failed: {result.stdout} {result.stderr}')
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    kernel.CreateFileW.argtypes = [ctypes.c_wchar_p, ctypes.c_uint32, ctypes.c_uint32,
                                  ctypes.c_void_p, ctypes.c_uint32, ctypes.c_uint32, ctypes.c_void_p]
    kernel.CreateFileW.restype = ctypes.c_void_p
    kernel.CloseHandle.argtypes = [ctypes.c_void_p]
    for path in paths:
        direct_file(path)
        handle = kernel.CreateFileW(str(path), 0x80000000, 0, None, 3, 0, None)
        if handle == ctypes.c_void_p(-1).value:
            raise Unsupported(f'File is open/unavailable: {path} (Windows {ctypes.get_last_error()})')
        kernel.CloseHandle(handle)


def verify_hashes(root: Path, hashes: dict) -> None:
    for name, expected in hashes.items():
        if file_hash(direct_file(root / name)) != expected:
            raise Unsupported(f'File no longer matches the recorded snapshot: {name}')


def load_preparation() -> tuple[dict, dict]:
    events = read_events(STATE)
    if not events:
        raise Unsupported('No prepared pilot')
    baseline = events[0]
    if baseline['kind'] != 'prepared':
        raise Unsupported('Missing imported baseline')
    for name in ('plan.json', 'before.json'):
        if file_hash(direct_file(STATE / name)) != baseline['data'][name]:
            raise Unsupported(f'Preparation evidence changed: {name}')
    return (json.loads((STATE / 'plan.json').read_text()),
            json.loads((STATE / 'before.json').read_text()))


def prepare(plan: dict) -> None:
    if read_events(STATE):
        raise Unsupported('Pilot already prepared; use audit/recover/rollback, not another application')
    if plan['hashes']['WeiDU.log'] != ORIGINAL_LOG:
        raise Unsupported('RC differs from the inspected baseline; re-assess before application')
    if (GAME / ADAPTER).exists():
        raise Unsupported('Adapter folder already exists')
    before = metadata(GAME)
    if before != metadata(BACKUP):
        raise Unsupported('Full backup does not match current game metadata')
    verify_hashes(BACKUP, plan['hashes'])
    if file_hash(direct_file(TOOL)) != WEIDU_SHA256:
        raise Unsupported('WeiDU identity mismatch')
    if shutil.disk_usage(STATE).free < 1024 ** 3:
        raise Unsupported('Insufficient space for pilot state')
    json_new(STATE / 'plan.json', plan)
    json_new(STATE / 'before.json', before)
    append_event(STATE, 'prepared', {
        'patch_id': PATCH_ID, 'adapter_file_hashes': PINS,
        'weidu_sha256': WEIDU_SHA256, 'backup': str(BACKUP),
        'baseline_kind': 'imported-observed-legacy-RC-not-a-CEBG-recipe',
        'plan.json': file_hash(STATE / 'plan.json'),
        'before.json': file_hash(STATE / 'before.json'),
    })


def apply() -> dict:
    plan = inspect(GAME, OWNER)
    if plan['status'] == 'already_fixed':
        return {'status': 'already_fixed', 'weidu_started': False}
    paths = [GAME / p for p in plan['hashes']]
    closed_game(paths)
    prepare(plan)
    # Recheck immediately before the first game write.
    verify_hashes(GAME, plan['hashes'])
    inspect(GAME, OWNER)
    for relative in PINS:
        target = GAME / ADAPTER / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(OWNER / relative, target)
        if file_hash(target) != PINS[relative]:
            raise Unsupported('Staged adapter hash mismatch')
    args = [str(TOOL), f'{ADAPTER}/setup-{ADAPTER}.tp2', '--game', str(GAME),
            '--language', '0', '--use-lang', 'en_US', '--force-install-list', '0',
            '--no-exit-pause', '--skip-at-view', '--safe-exit', '--noautoupdate',
            '--log', str(STATE / 'attempt.debug')]
    json_new(STATE / 'invocation.json', {'argv': args, 'cwd': str(GAME)})
    try:
        with (STATE / 'stdout.log').open('xb') as stdout, (STATE / 'stderr.log').open('xb') as stderr:
            result = subprocess.run(args, cwd=GAME, stdin=subprocess.DEVNULL, stdout=stdout,
                                    stderr=stderr, timeout=180, creationflags=subprocess.CREATE_NO_WINDOW)
        if result.returncode != 0:
            raise Unsupported(f'WeiDU exited {result.returncode}; inspect attempt diagnostics')
        verify_application(plan)
        after = metadata(GAME)
        changed = changed_paths(json.loads((STATE / 'before.json').read_text()), after)
        hashes = {name: file_hash(GAME / name) for name in changed if after.get(name) is not None}
        json_new(STATE / 'after.json', after)
        append_event(STATE, 'applied', {'patch_id': PATCH_ID, 'after.json': file_hash(STATE / 'after.json'),
                                     'post_hashes': hashes, 'changed_paths': sorted(changed),
                                     'weidu_before': plan['hashes']['WeiDU.log'],
                                     'weidu_after': file_hash(GAME / 'WeiDU.log')})
        return {'status': 'applied', 'changes': plan['changes'], 'changed_paths': sorted(changed)}
    except Exception as error:
        append_event(STATE, 'attempt_failed', {'error': str(error)})
        try:
            recover()
        except Exception as restoration:
            append_event(STATE, 'blocked_unverified', {'error': str(restoration)})
        raise


def verify_application(plan: dict) -> None:
    verify_tables(plan, {name: (GAME / 'override' / name).read_bytes() for name in TABLES})
    unchanged = {name: value for name, value in plan['hashes'].items()
                 if name != 'WeiDU.log' and name.lower() not in {f'override/{t.lower()}' for t in TABLES}}
    verify_hashes(GAME, unchanged)
    rows = [list(row) for row in parse_log((GAME / 'WeiDU.log').read_bytes())]
    if rows[:-1] != plan['component_rows'] or rows[-1][:3] != [f'{ADAPTER.lower()}/setup-{ADAPTER.lower()}.tp2', 0, 0]:
        raise Unsupported('WeiDU component sequence changed outside the exact single-row suffix')
    before = json.loads((STATE / 'before.json').read_text())
    writable = {f'override/{name.lower()}' for name in TABLES} | {'weidu.log', 'weidu.conf'}
    unexpected = {p for p in changed_paths(before, metadata(GAME)) if not allowed(p, writable, (ADAPTER.lower(),))}
    if unexpected:
        raise Unsupported(f'Undeclared game changes: {sorted(unexpected)[:20]}')


def recover(*, rollback: bool = False) -> dict:
    plan, before = load_preparation()
    events = read_events(STATE)
    if events[-1]['kind'] in ('rolled_back', 'restored'):
        verify_hashes(GAME, plan['hashes'])
        return {'status': events[-1]['kind']}
    if rollback:
        last = events[-1]
        if last['kind'] != 'applied':
            raise Unsupported('Only the latest successfully applied patch can be rolled back')
        if file_hash(STATE / 'after.json') != last['data']['after.json']:
            raise Unsupported('Post-patch evidence changed')
        if metadata(GAME) != json.loads((STATE / 'after.json').read_text()):
            raise Unsupported('Game changed after patch; rollback requires assessment')
        verify_hashes(GAME, last['data']['post_hashes'])
    elif any(event['kind'] == 'applied' for event in events):
        raise Unsupported('Use guarded rollback for an applied patch, not failure recovery')
    closed_game([GAME / name for name in plan['hashes'] if (GAME / name).exists()])
    verify_hashes(BACKUP, plan['hashes'])
    restore_files(GAME, BACKUP, before,
                  {f'override/{name.lower()}' for name in TABLES} | {'weidu.log', 'weidu.conf'}, (ADAPTER.lower(),))
    verify_hashes(GAME, plan['hashes'])
    status = 'rolled_back' if rollback else 'restored'
    append_event(STATE, status, {'patch_id': PATCH_ID, 'restored_hashes': plan['hashes']})
    return {'status': status, 'game_matches_pre_patch': True, 'saves_edited': False}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=['inspect', 'apply', 'audit', 'rollback', 'recover'])
    args = parser.parse_args()
    if os.name != 'nt' or str(GAME.resolve()).lower() != str(GAME).lower():
        raise Unsupported('Only the explicitly named plain Windows RC is authorized')
    direct_file(GAME / 'WeiDU.log')
    if args.mode == 'inspect':
        plan = inspect(GAME, OWNER)
        result = {k: plan[k] for k in ('status', 'changes')}
    else:
        with pilot_lock():
            if args.mode == 'apply':
                # Never turn an interrupted attempt into an apparent no-op success.
                events = read_events(STATE)
                if events and events[-1]['kind'] != 'applied':
                    raise Unsupported('Prior pilot state requires audit/recovery')
                result = apply()
            elif args.mode == 'audit':
                events = read_events(STATE)
                plan, before = load_preparation()
                if events[-1]['kind'] == 'applied':
                    verify_application(plan)
                elif events[-1]['kind'] in ('restored', 'rolled_back'):
                    verify_hashes(GAME, plan['hashes'])
                    if metadata(GAME) != before:
                        raise Unsupported('Restored game has since changed')
                else:
                    raise Unsupported('Pending/failed transaction; do not launch this RC')
                result = {'status': events[-1]['kind'], 'verified': True}
            else:
                result = recover(rollback=args.mode == 'rollback')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
