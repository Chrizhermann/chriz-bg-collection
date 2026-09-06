# Alpha.13 EET pre-spawn stop — 2026-09-07

Christopher supplied a screenshot from his alpha.13 test. Read-only inspection of
`C:\Users\chris\Games\CEBG-Tests\Alpha13-20260907` established:

- Ledger 145–152 records successful BG1 EE Fixpack, BG1 UB, BG1 NPC Project and BG2
  EE Fixpack; DLC Merger also completed. All five invoked mod runs exited 0.
- Ledger 153 starts `install:eet-initialize-bg2`; 154 records failure at
  `1788729378730` ms UTC epoch, reporting an executable beneath the managed target:
  `game/EET/bin/win32/x86_64/weidu.exe`.
- The EET attempt directory
  `.chriz/attempts/attempt-7da66eb7040f78cca7c1/steps/0076-b0de61d0bf938f3c/attempt-0001`
  contains only `before.log` (319 bytes): no invocation or process evidence. The
  terminal receipt lists zero EET attempts. EET did not launch through this attempt.
- That snapshot and the current game `WeiDU.log` share SHA256
  `fd074370ab77cabd23dab77e55a06f77c7868def76f85a4908d61c2131f958df`.
  Current BG2 log contains only EE Fixpack components 0 and 2.
- The reported EET process was absent at inspection. A separate rebalance agent's
  `--nogame` WeiDU ran in its own worktree, outside this target. That process cannot
  explain this exact target-path match. Original process parent/command line was
  not captured: concurrent target access or a lingering/exited helper is not proven.

## Source gaps

1. The Windows process probe accepts a successfully resolved image path without
   checking termination; it only checks exit state after image lookup fails.
   Resolving an image path is not proof a process is still running.
2. The install sequence records `before.log`, checks before invocation construction,
   records the invocation, then checks again before process spawn. Recovery currently
   assumes invocation and process-output evidence exists even if the first check
   failed. It can incorrectly seal this never-started step as requiring a new copy.

Christopher was asked to hold off on Retry and preserve the folder. No process was
killed, no installation retried, and no file in the target was changed by this task.
Fixes must preserve live-process/TLK guards and ambiguous launched-attempt protection.
Missing output alone is not proof that WeiDU never ran. Validate with small fixtures,
then allow the same managed copy to continue through the installer; do not restart
the complete mod stack as a workaround.

## Fix and real continuation

The Windows probe now checks a process handle without waiting, including the actual
exit-code-259 case; conservative inspection remains when synchronization rights are
unavailable. Four native process regressions and all 19 existing preflight tests pass.

Pre-spawn guard failures now retain a hash-bound evidence record. Recovery checks
that record, terminal ledger identity, exact allowed files and unchanged WeiDU.log.
The alpha.13 before-log-only case has a narrowly restricted compatibility path.
Launched/incomplete and altered evidence remain fail-closed. CLI 29, orchestrator 28
and diagnostics 12 tests pass, including both guard boundaries and corruption cases.
Engine all-targets Clippy passes after fixing an equivalent case-comparison lint.

Christopher subsequently authorized overnight continuation, with cleanup/restart
only if safe recovery is impossible, and a patch release as needed. At 06:37 KST,
2026-09-07, the freshly compiled release CLI resumed the same managed copy using
the same engine entry point as the native app. Native registry and ledger identity
matched `install-5f63e5d7939d3f509139`; frozen recipe SHA256 and plan were unchanged.
Ledger 155 starts EET attempt 2, and actual EET resource-import output followed.
The five completed mod runs were not repeated. This is successful continuation,
not yet proof of a completed full installation or gameplay acceptance.

The old failed alpha.13 app was closed normally before continuation. Background
worker PID at launch: 40540. Monitoring output is under
`target/cebg-overnight/alpha14-resume-20260907/`; the authoritative evidence remains
the managed copy's ledger and receipts. Do not start a second worker while it runs.
