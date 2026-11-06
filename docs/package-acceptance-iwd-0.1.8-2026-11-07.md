# Chriz Easy IWD 0.1.8 package acceptance

Status: source checks, full installation reconciliation, the bounded Arszyk runtime
checks, final package verification, updater-signature acceptance and the Defender
scan below have passed. Version 0.1.8, its update channel and the deployed website are
public and verified. Release work is complete within the coverage documented below.
Christopher authorized completing the existing work, releasing if clean, then updating
and deploying the website on 2026-11-07.

## Source and scope

- Candidate: PR [116](https://github.com/Chrizhermann/chriz-bg-collection/pull/116),
  head `079f7df874c820a042e25b731662bc6f1263f8f7`, against `claude/iwdee-app` at
  `c7f0166cfcee1117ebd78f4b99071a56a993e630`.
- PR 116 is merged at `889c430ec6eb0812de0252cddbf6fa8080a10900`. The final
  release setup was built from this merge.
- App and recipe: 0.1.8. The previous public app is 0.1.7, whose recipe is 0.1.6.
- Changes: Chriz IWD Encounters 0.2.1, banter-guard component 110 and its separate
  existing-game hotpatch; Arszyk's Wonderful Decay BETA1.4 for new installations.
- Production CLI `plan` confirms 29 mods and 139 components in `chris-recommended`.
- Independent review of all 17 changed files found no consequential source defect.
  A joined checklist line in `docs/curation/iwdee/FOLLOW_UPS.md` is corrected separately.

## Automated source checks

[Windows CI run 42250154382](https://github.com/Chrizhermann/chriz-bg-collection/actions/runs/42250154382)
passed every step. It tested GitHub's merge commit
`58bb219fb3c1dc009b3c5a700c5dd81f1d423e95` (the candidate merged into the base above).

- Rust workspace: 1,265 passed, 20 existing opt-in ignored tests.
- IWD app flavor: 141 passed, 3 existing opt-in ignored tests.
- Frontend: 504 passed in 42 files; typecheck and production builds passed.
- Python tools: 201 tests run, 1 existing skip, no failures.
- Rust formatting, Clippy, locked third-party notices, unsigned BG and IWD packaging,
  and both update-install cleanup checks passed. The IWD cleanup check also validates
  the installed recipe on the disposable GitHub runner.

This is automated evidence, not gameplay acceptance. Do not run the CI-only NSIS
cleanup test locally: it changes the current user's app registration.

The final own-mod release gate passed against current Chriz Easy BG and the merged
IWD branch at `889c430`: no installer is out of step. Chriz IWD Encounters is pinned
to 0.2.1. BuffBot 1.16.0 and Campaign Item Randomiser 1.0.3 remain under the recorded
holds.

`npm ci` reports existing development-tool advisories in Vite, Vitest/mocker and
source-map-js. Production-only audit reports zero vulnerabilities. The affected
development server, mock WebSocket and untrusted indexed-source-map paths are not
part of the shipped app or this release build's inputs. Follow-up
[117](https://github.com/Chrizhermann/chriz-bg-collection/issues/117) records the assessment
and the required tooling update; this is not a claim of a clean full dependency audit.

## Existing-install hotpatch

The already completed 2026-11-07 lab sequence exercised a copy of a 0.1.3 installation:
fight update, banter-guard update, then guard Undo. The guard was offered after the
fights, its installation passed all 40 pair checks, and Undo restored `baldur.bcs`
byte for byte and offered the guard again. The fight hotpatch retains its original
0.2.0 package. This is install/undo evidence, separate from gameplay.

## Full installation

The complete installation finished successfully on a fresh Steam IWD:EE 2.7.3.0
lab using WeiDU 249. It installed the 139 recommended components plus the optional
EEex Remote Console used by the lab: 140 components across 31 planned runs.
All 99 installer steps succeeded; no planned component was omitted.

An independent reconciliation parsed the entire command result, not just its final
status. All 27 checks passed:

- The frozen recipe and resolved selection match candidate `079f7df`; a fresh plan
  from that candidate matches the receipt. All 95 recipe files match the candidate
  after line-ending normalization.
- All 31 acquired artifacts match the pinned URLs, sizes and hashes. The source
  executable independently reports version 2.7.3.0.
- All 140 ordered WeiDU entries match the plan and receipt. Each planned run has
  one attempt with matching invocation, process result and retained output hashes.
- The current and immutable attempt receipts agree. All 195 ledger records have a
  valid sequence and hash chain.

The planner's two notices concern defaults whose parent mods are disabled; they
are not missing planned components. This evidence establishes installation
completeness and integrity, not a warning-free install or full gameplay coverage.

The unchanged SCS 35.21 pin produced an installed-with-warnings result. Its missing
timer constant removes a noncombat patrol pause; this is a real, bounded upstream
defect tracked in [118](https://github.com/Chrizhermann/chriz-bg-collection/issues/118),
not a regression introduced by this candidate. Yeti classification uses an explicit
fallback, and ranged-without-melee warnings do not omit creatures or components.
The spell-edit and script-order warnings were retained and assessed; the later
caster-AI repair component installed, without implying that every affected encounter
was played. Already-present resources and already-active LuaJIT messages are
intentional no-op notices.

## Arszyk static and runtime checks

Static inspection compared the pinned upstream archive, installed resources and
WeiDU backups. All 16 creature, 16 item and 22 spell resources checked have valid
table bounds; creature inventory indices and spell memorization indices are valid.
No direct resource target was missing for the checked spell-casting, summoning and
external-effect opcodes. This is not an exhaustive recursive dependency or animation
frame audit.

SCS preserves Arszyk's scripts, animation, inventory and creature effects. It changes
one classification byte on Arszyk; on the ice actor it changes classification,
THAC0 and five saving throws. The two rings preserve their original effects and gain
six spell-state effects each. Three references already present in upstream creature
files remain unresolved: a class AI script, a summoned-eye dialogue and a memorized
innate on a summoned creature. No failure from these references was demonstrated;
their wider behavior remains outside this acceptance scope.

Two initial diagnostic attempts were investigated rather than counted as passes:
the first crash dump identifies a queued creature-creation action running after the
test player character died and lost its area, not an Arszyk actor failure. The second
attempt timed out without producing a crash dump; a dialogue interrupting the lab's
polling is consistent with the evidence, but the timeout alone proves neither a
crash nor successful completion.

The subsequent controlled runs protected the test player from incidental death and
allowed the harness to observe dialogue. They passed the following checks using the
installed mod behavior:

- Started the game and loaded the lab save, then freed Arszyk with fire through the
  original ice actor and completed the full initial dialogue using the first choices.
  Arszyk joined, the recruitment state changed, and the party contained two members.
- Moved the party outside, saved with a distinct verification marker, stopped the
  owned game process and loaded that save in a fresh game process. Both party members
  and the saved marker persisted. Daytime was independently confirmed and Arszyk's
  daylight visual effect was present.
- Triggered the departure dialogue and chose to leave Arszyk in place. Recruitment
  state cleared, the party returned to one member, and Arszyk remained alive at the
  same location after leaving and returning to the area.

Neither controlled run produced a crash dump or triggered the audio guard; measured
audio peak was zero. The temporary dialogue-observation UI change was restored byte
for byte after both runs. The harness terminated only its owned processes; its
termination exit codes are not evidence of a graceful in-game exit. No game-content
fix was needed for these checks.

Coverage is limited to startup, fire liberation, that recruitment dialogue path,
an outdoor daylight observation, save/reload and dismissal with an area round trip.
It does not establish the complete quest, all dialogue branches, combat, summoned
creature abilities, daylight damage or re-recruitment behavior. No claim of all mod
combinations is made.

Private command results, receipts, ledgers, comparisons, crash diagnostics and
runtime observations are retained outside the repository. No game files or private
diagnostic data belong in this repository.

## Final package

The final artifact built from `889c430` is
`Chriz.Easy.IWD_0.1.8_x64-setup.exe`, exactly **6,395,645 bytes**, with SHA-256:

```text
10a1532c284eb3fc1ad10e6199c5eab8596633e864c423e75c3c97d4693fa3c0
```

All 16 independent package-content checks passed after extracting that exact setup:

- All 95 bundled recipe files and five hotpatch files have the exact recursive file
  set and byte-for-byte contents of the merged source. These source resources were
  unchanged from the merge when checked.
- `LICENSE` and `THIRD_PARTY_NOTICES.md` match the source exactly. The extraction
  contains exactly 109 files: those resources, the app and six NSIS support files.
  There are no extra BG resources, third-party mod archives, game-resource payloads
  or reparse entries.
- The packaged app matches the final build apart from the three expected bytes
  identifying the Tauri bundle as NSIS (`UNK` to `NSS`). Setup, packaged app and
  source app all report Chriz Easy IWD 0.1.8.

The detached Tauri updater signature was produced and verified against the public
key compiled into the IWD configuration. The real updater integration test downloaded
and verified the exact setup from a local test feed, checked the downloaded bytes
against the artifact, and rejected a one-byte modification with a Minisign error.
Both the explicit signature test and updater-download acceptance test passed. This
verifies the updater signature and download path; it is not an Authenticode or
public-hosting check and does not execute the installer.

Microsoft Defender scanned that exact setup successfully (exit 0) and reported no
threats.

## Publication

- [Versioned release](https://github.com/Chrizhermann/chriz-easy-iwd/releases/tag/v0.1.8)
  published as the latest release on 2026-11-06 at 20:21:38 UTC (November 7 KST).
  All six assets were downloaded anonymously and match their local originals exactly;
  the published checksums, feed URL, version and signature agree with the verified setup.
- The fixed `updates/latest.json` channel now serves that exact 0.1.8 feed. The first
  request immediately after replacement still returned cached 0.1.7; a subsequent
  ordinary request to the configured URL returned the exact new bytes. The prior feed
  and channel notes are retained outside the repository for rollback.
- Distribution README updated at `43f860fb598c42fe9905cad39f0f474b66af5718` and read back
  successfully, including 29 mods, 139 components, 18 companions and the beta/new-install
  scope of Arszyk.
- Website [PR 66](https://github.com/Chrizhermann/twitch-setup-chriz/pull/66) merged at
  `af4dae1157e71f9c4a6708d2393b506a290f4fa5`. All 158 website tests and the production
  build passed; GitHub and Cloudflare checks passed for the merge. The published site
  uses the versioned setup link, exact checksum and 6.1 MB display size.
- The live IWD page and home card were checked at desktop 1440 px and mobile 375 px:
  correct 0.1.8 release and 18-companion content, no horizontal overflow, script or
  console errors. Public content JSON and the 1200 x 630 social card match the reviewed
  files. The actual website download returned the exact setup size and SHA-256 above.
  Live checks completed on 2026-11-07 at 05:25 KST.

## Cleanup

The final release assets and diagnostic evidence are retained outside the checkout.
The temporary signing-key copy was removed. Only this session's release-worktree build
output was cleaned with `cargo clean` (5.9 GiB); other agents' worktrees, installed apps
and live games were left intact.
