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

## Follow-up roadmap: My installs clarity and unique names — September 16

**Planned, not implemented in alpha.17.** Christopher's screenshots show excessive
spacing, repeated oversized headings/status text, and indistinguishable entries
named `Chriz Easy BG — incomplete installation`. Record this as a focused follow-up;
do not delay his current full-install test for another design/release cycle.

- Make My installs a compact, coherent view: one page heading, a clear installation
  selector/list, a concise status, and consistently grouped actions. Put diagnostics
  and technical detail behind secondary disclosures; avoid repeated warning prose
  and unnecessary scrolling at a normal desktop window size.
- Keep the installation's chosen name separate from status. Preserve it throughout
  preparation, failure, restart of the app and completion; failed copies must not
  all fall back to the same generic title.
- New installation names must be unique among registered copies, including failed,
  incomplete and unavailable entries. Propose `Chriz Easy BG`, then
  `Chriz Easy BG (2)`, `(3)`, etc. Use case-insensitive, trimmed comparison; validate
  custom names inline and at creation in the backend, including competing requests.
- Existing duplicate names must remain distinguishable with folder/date or a short
  stable identifier until explicitly renamed. Renaming is display metadata only:
  never move game folders or rename save profiles, and never replace stable IDs
  with names as the identity used for launch/removal/recovery.
- Show enough location/version context to choose the right copy. After deletion or
  forgetting, name the entry removed and clearly identify whichever copy remains;
  a generic success notice above another generic failure card is ambiguous.
- Retain exact-path destructive confirmation and the current cleanup protections.

When implemented, use small UI/backend fixtures covering duplicate custom names,
automatic suffixes, failed-name persistence, legacy duplicates and removal of one
of two similar copies, plus one desktop/small-window visual check. No full mod
installation is needed to validate these presentation/naming changes.

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
