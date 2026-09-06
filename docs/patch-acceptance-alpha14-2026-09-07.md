# App alpha.14 — recover before-spawn stops and reduce launcher friction

Release authorized by Christopher on 2026-09-07 while requesting overnight
continuation of his Alpha13 test. Collection remains alpha.12; no mod/default/pin
or signing-key changes. Public-source/license publication is coordinated separately
with the existing audit task.

## Verification and triage

- Engine recovery: CLI 29, orchestrator 28 and diagnostics 12 tests passed. Both
  pre-spawn boundaries and the strict alpha.13 legacy shape resume only unstarted
  work. Eleven evidence-corruption scenarios reject recovery. Diagnostics exports
  now retain the hash-bound guard evidence.
- Process probe: four native Windows tests plus all 19 preflight tests passed,
  including exited-but-queryable handles, exit code 259 and conservative fallback.
- Existing EET compatibility: seven tests passed; opt-in upstream/real-WeiDU smoke
  was not repeated for a behavior-equivalent ASCII case-comparison lint correction.
- Engine all-targets Clippy passed with warnings denied. No ignored lint or bypass.
- Native app: 10 library tests (including seven real Windows shortcut tests), 39
  command contracts and three normal package contracts passed. Signed-setup
  verification follows packaging; it is not claimed by the normal package tests.
- Frontend: TypeScript, all 139 tests and production Vite build passed. Existing
  error/focus/state contracts cover the small UI additions; no new visual redesign
  or native GUI click-through acceptance is claimed.
- Python curation/public inventory/package tooling: all 35 tests passed.
- Cleanup/security review: no debug prints, secrets or new dependencies. The
  deliberate failure injection is debug-build-only, scoped to synthetic tests.
  Process guards still protect live processes, and pre-spawn recovery cannot rely
  merely on missing logs. Stored setup preferences never contain review tokens or
  authorize installation; native source/destination validation remains required.
- Broader impact: original sources, stream games, saves, shared downloads and
  frozen recipe selection stay untouched. Recovery extends the verifier trait and
  diagnostics file allowlist; native contracts are rebuilt against those changes.
- Project knowledge and documentation: incident, release notes, live handover and
  deferred cleanup roadmap are updated. No generated memory/global skill changes.
- Communication: keep the public-source audit and website owner informed using the
  existing tasks; do not send Discord/forum/email announcements on Christopher's
  behalf. Publication/updated feed must be verified before saying the patch is live.

## Real installation continuation

The old failed app closed normally. The freshly built release engine resumed
`C:\Users\chris\Games\CEBG-Tests\Alpha13-20260907` after its native registry ID,
root and frozen recipe digest matched the ledger. At 06:37 KST, ledger 155 began
EET attempt 2 and EET imported resources. No completed mod run was repeated and
no cleanup/redownload/new installation was performed. This is the same engine used
by native Resume, invoked in the background; it is not a GUI-click-through claim.

Full-stack completion, gameplay and the alpha.14 package publication remain pending.
