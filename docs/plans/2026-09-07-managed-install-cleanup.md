# Managed installation cleanup — deferred roadmap

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
