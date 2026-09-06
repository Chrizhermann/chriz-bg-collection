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
the complete mod stack as a workaround. Implementation/acceptance is pending.
