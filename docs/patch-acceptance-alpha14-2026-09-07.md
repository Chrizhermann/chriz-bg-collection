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

EET subsequently completed (ledger 156, components 0/100 recorded), and the worker
continued into the main mods. Full-stack completion and gameplay remain pending.

## Signed package and publication

The signed Windows x64 NSIS package built successfully from source commit
`5610783590ad49b24101d5d3a6a85b018abadb7e`. It includes the audited dependency
notices supplied by the parallel public-source task. Regenerating notices for the
alpha.14 version-only lockfile changes altered only their two header digests;
the independent generator check and all 42 Python tests passed.

- Setup size: **5,447,242 bytes**.
- SHA256: `c36058d92165ff4e2e3d993c5eaedee95e05c1de95f388668f61b4d7571c1a3b`.
- Actual setup signature passed the freshly compiled package contract. The existing
  compiled headless real-Tauri harness downloaded/verified the exact setup and
  rejected tampering, offering alpha.14 over alpha.13. It uses the unchanged updater
  dependency/public key; no fresh mock-harness rebuild or native GUI apply is claimed.
- Windows Defender custom scan, remediation disabled, reported no threats in that
  setup. This is neither an Authenticode signature nor a security guarantee.
- The public versioned release contains setup/signature/feed/checksums, component
  inventory, LICENSE and notices. Anonymous download matched exact size/hash;
  its signature matched the public feed and passed against the bundled public key.
- The mutable `alpha/latest.json` channel was updated only after versioned download
  verification. One immediate request hit a stale GitHub CDN redirect (`BlobNotFound`);
  a fresh query and then the normal configured URL with no-cache headers returned
  alpha.14/recipe alpha.12 with the correct versioned setup and signature. No old
  versioned release was overwritten.
- A verified public copy is at
  `C:\Users\chris\Downloads\Chriz Easy BG_0.1.0-alpha.14_x64-setup.exe`.
  Attempted unattended app-setup execution was rejected by the tool environment
  before execution; it was not retried through another route. Host app remains
  alpha.13, closed, while the separately rebuilt patched engine continues the game
  installation. This is not evidence of a Windows installer permission defect.

The website owner received the versioned URL/hash/notes and normal scoped deployment
handoff and verified deployment: twitch-setup-chriz PR 9, merge
`a47e11d6a170ad09a6b95cf28cddc5ea64bc3025`, 76/76 tests locally and hosted, deployment
dry-run, and live `/collection`/JSON identity and security-header checks. The
public-source audit owner received the exact build commit and release
identity. No Discord/forum/email post or source-visibility change was performed by
this task. Overnight follow-up is scheduled on the existing heartbeat, every 20
minutes, quiet on ordinary progress; it must pause after final acceptance/failure
handoff. Successful game copies are retained for Christopher, not cleaned up.
