# Managed installation cleanup

## Alpha.17 candidate — September 16

Implemented for completed and incomplete registered copies, including sealed
failures. A preview shows the exact folder before a separate permanent-delete
confirmation. A missing folder offers entry-only forgetting. Sources, shared
downloads and save profiles outside the managed folder are preserved. Files and
saves manually placed inside the deleted folder are explicitly included in the
warning. Existing desktop shortcuts are left in place.

The backend resolves registry IDs itself, revalidates identity under the target
lock, blocks active workers/processes and protected/overlapping/reparse paths,
and retains the registry until deletion succeeds. A durable external removal
journal binds the original directory identity so interrupted deletion is retryable
even if the last metadata cleanup was interrupted. Tokens are single-use; a
failed attempt requires a fresh preview and another explicit confirmation.

Ten disposable engine tests and a native confirmation test pass; UI confirmation,
cancellation, retry and missing-folder flows pass. No real game copy was deleted.
Christopher explicitly defers live deletion acceptance. Disk-space estimates,
automatic diagnostic export and owned-shortcut cleanup remain later refinements;
failed copies are never removed automatically.

## Original roadmap (historical)

Christopher requested this on 2026-09-07 while authorizing recovery of the specific
`CEBG-Tests/Alpha13-20260907` test. Prefer resuming that existing copy; cleanup and
restart are fallback actions, not an excuse to repeat completed work.

Current state: CEBG preserves its managed-install registry and failure diagnostics,
but does not expose a supported delete/cleanup action. This feature is useful, not
a requirement for the alpha.14 recovery patch.

Later, add **Remove installation** in My installs with the exact folder, status,
approximate space recovered, and explicit confirmation. Default to preserving a
diagnostics export for failed copies and clearly explain treatment of saves and
owned shortcuts. Source games, other managed copies and shared downloads must not
be deleted. Cache cleanup should be a separate action.

Reuse validated managed-root identity, containment/reparse checks and active-process
protection. Never recursively delete a drive, home, source-game or parent workspace
directory. Keep the registry entry until deletion completes; if files are locked,
report the remaining path and offer retry. Handle already-missing folders separately
as **Forget this entry**, without treating a stale registry pointer as deletion
authority. Do not delete anything automatically because a build failed.
