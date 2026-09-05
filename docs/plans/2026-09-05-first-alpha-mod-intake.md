# First-alpha intake from the owning mod repositories

**2026-09-06 approvals supersede the pending-selection notes below:**
[current intake and website handoffs](2026-09-06-release-intake-and-website-roadmap.md).
610,220-223 and releasedSoD290 are approved for recipe intake; Hexxat default
is explicitly Shadowdancer221. Component620/dragons remain WIP. Full skip
has a new owner-readiness request. Implementation targets recipealpha10 and is
not yet an accepted build. Older rows remain the preceding snapshot.

Checked 2026-09-05, approximately 23:20 KST; recipe and modpack status updated
2026-09-06 KST. Christopher permits work completed
today to enter the first public alpha; newer unfinished components need not hold
up the installer. This is an intake queue plus a record of what entered the next
versioned recipe; release availability by itself still does not select a component.

## Preserve the current acceptance run

**Latest correction (2026-09-06):** the failure snapshot below is historical.
R5 has completed all 430 selected components through the documented supervised
repair, with modpack alpha.5 and retained Bardic balance.2. Christopher now
reports initial gameplay without crashes or issues. The source recipe alpha.9
instead pins Bardic balance.3; neither recipe nor current r5 has automatically
selected new components simply because their package was released.

The SoD owner published **v0.6.6** at **2026-09-05 17:28:49 UTC** (verified via
the GitHub release API on 2026-09-06). It includes the accepted, tested EET
victory-ending **290** and repair-only **291**, retaining v0.6.5 fixes. This is
not the optional full-SoD skip. Current recipe/r5 remain v0.6.5 with 290 absent;
the old '290 unfinished' deferral is no longer an accurate owner-status claim.
Intake still needs the new artifact pin and explicit collection selection;
fresh installations must not select repair-only 291. Do not patch playing r5.
Source: [SoD v0.6.6 release](https://github.com/Chrizhermann/chriz-sod-rebalance/releases/tag/v0.6.6).

R5 remains the frozen alpha.8 recipe: 43 runs / 430 components. It is terminal
after pinned alpha.1 modpack 170/192 failed (both bugs also persisted in alpha.4). They are now
fixed in published alpha.5, but that does not alter or make r5 resumable. See the
[owning-repo handoff](../handoffs/2026-09-06-modpack-xan-viconia.md).
Do not change its pins, selection, ledger
or receipt, restart it for each new mod release, or deploy anything into the
stream game. Additional accepted work belongs in the next versioned candidate.
R5 cannot provide acceptance evidence for versions/components it did not install.

Recipe **`0.1.0-alpha.9`** pins modpack **`v0.2.0-alpha.5`** and Bardic Wonders
**`v2.9c-balance.3`**. Both immutable packages passed independent acquisition,
hash and archive-layout verification. The resolved plan remains exactly **43 runs /
430 components**: selected components, order, prompts, and arguments are unchanged
from alpha.8. Alpha.9 therefore retains components 170 and 192, updates the package
that supplies their fixes, and updates the already-selected Bardic components. It
does not select utility XP 610 or expose/select companion choices 220-223 yet.

## Ready and in-progress additions

| Work | Verified state | First-alpha disposition |
|---|---|---|
| Modpack progressive utility XP, **610** | Present in public **v0.2.0-alpha.5**. Component implemented; owner records successful native startup/load/save and one scribing award, plus Christopher's successful test. Rounding was subsequently changed and covered automatically. | Package is pinned, but component 610 is **not selected** in alpha.9. Selection policy is awaiting Christopher: mandatory versus default-checked optional versus omitted for now. It requires EEex; when selected, place it after EET finalization and other utility-XP tweaks. |
| Bardic Wonders Abettor fixes | Public **v2.9c-balance.3**, pinned in alpha.9. Affects already-selected **1004/2004/2007**. | Pin update complete with selections unchanged. Do not add the separate existing-playthrough repair component to a fresh installation. Preserve Darkbloom 1006's Spell Revisions exclusion. Ordinary party invisibility at Symphony start/end remains an owner's known unverified behavior, not a newly imposed release blocker. |
| Yoshimo / Hexxat choices | Present in public modpack **v0.2.0-alpha.5**; originally released in alpha.4. Final IDs **220–223**, no existing IDs changed. | Not wired or selected in alpha.9. Future intake may expose Yoshimo Swashbuckler and mutually exclusive Hexxat Shadowdancer / Fighter-Thief / Assassin alternatives, without inventing a checked default. Live recruitment remains untested as agreed. |
| Xan 170 / Viconia 192 integration fixes | Public modpack **v0.2.0-alpha.5**, pinned in alpha.9. Focused install, idempotent repeat and exact-uninstall checks passed on captured r5 resources; the package was independently downloaded and verified. | Blockers cleared at component/fixture scope. Existing selections remain unchanged. Full curated integration still requires a fresh run; do not resume r5 or describe the focused checks as a successful full installation. |
| Imoen Spellhold XP, **620** | Implemented locally, requires EEex, 186 tests and installer checks reported. Average of the other party members, capped at 3 million mage XP; user's recruitment test has not been reported complete. | Pending owning-task acceptance/release, not part of the published alpha.3 payload. Keep separate from utility XP 610. |
| New BG Rebalance dragon work | **Review open rebalance todos** is still discussing/researching encounter design. Public release remains the already-pinned **v0.3.1**. | WIP. Recheck the eventual release delta; do not treat discussed dragon rules as implemented components. Existing accepted ambient/urgent 120/121 stay included. |
| Optional SoD skip | Native ground-pile probe failed, including restart/reload persistence; collection method needs replacement before banking/transition integration. Owner commit **19e221b**, 24 automated tests do not override the native failure. Released **v0.6.5** is unchanged. | Do not ingest the prototype. Keep the released SoD selection; skip can join when actually ready, otherwise an early follow-up alpha. Separate from deferred 290. See the existing [handoff](../handoffs/2026-09-05-optional-sod-skip.md). |
| SR/RR AI compatibility | Separate owning-repo branch; public SR remains the already-pinned **v4.21-chriz.3**. | Previously deferred; no newly verified release to ingest. Do not reopen the design or hold the installer for it. |
| Script engine | **Begin script engine Task 1** reports infrastructure Task 2 completed, later tasks pending/in progress. | Not yet a released collection mod. No automatic lab setup or recipe entry. |

The public modpack alpha.1 -> alpha.3 TP2 comparison added **only 610**: no
existing component IDs were removed or renumbered. Alpha.2 was not published
as a release because a Windows temporary-path build check failed; alpha.3
included the packaging/test-harness correction. Alpha.4 introduced 220–223;
alpha.5 repairs 170/192 without changing their approved gameplay policies.
Public 190 is Sarah, 192 is Viconia and 193 is Shar-Teel.

Utility-XP evidence has limits: the native scribing check used the previous
whole-point rounding on BG2EE 2.6.6 / EEex 0.11; the final configurable rounding
is automatically tested. Native lock/trap awards, the complete live class
matrix and long sessions were not exhaustively tested. Do not claim those
passed, but do not demand another exhaustive review before alpha intake.

## Public pins checked, already current

- BG Rebalance **v0.3.1**; SoD Remix **v0.6.5**.
- BuffBot **v1.8.3-alpha**; Spell Revisions **v4.21-chriz.3**.
- Artisan **chriz-v1.3.1**, tag resolves to pinned `ac718614...`; includes the
  released Shield Bash work. Its original owning-task live check was pending;
  do not infer current-stack gameplay acceptance from packaging tests.
- Randomiser **v8.1.1**.
- EEex Remote Console: no public release; public HEAD remains pinned
  `661927e...` / version 0.2.0. Unpublished work is not a new public pin.

## Bounded integration pass before packaging

1. Recheck the rows still finishing once when assembling the candidate, or when
   Christopher reports an owning task complete. No high-frequency all-repo poll.
2. Use published package bytes/final component IDs, not dirty shared checkouts.
   Verify acquisition/hash/layout and affected component dependencies/exclusivity.
3. Show the small selection/version delta against recorded curation. Existing
   defaults remain authoritative; release availability alone is not a new default.
4. Run affected recipe/resolver tests and a proportionate install/launch check
   of changed content. Reuse unaffected download/UI/full-stack evidence; do not
   restart the 430-component run for each arriving commit. Any final receipt must
   truthfully identify what that installation actually contains.
5. Leave unfinished work in this queue for a follow-up alpha. Continue the
   [installer release path](2026-09-05-alpha-release-short-path.md), especially
   safe pause/close and remaining native/update/distribution acceptance.

Sources: [modpack alpha.5 release](https://github.com/Chrizhermann/chriz-bg-modpack/releases/tag/v0.2.0-alpha.5),
[modpack alpha.4 release](https://github.com/Chrizhermann/chriz-bg-modpack/releases/tag/v0.2.0-alpha.4),
[public modpack tag comparison](https://github.com/Chrizhermann/chriz-bg-modpack/compare/v0.2.0-alpha.1...v0.2.0-alpha.3),
[Bardic balance.3 release](https://github.com/Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch/releases/tag/v2.9c-balance.3),
the owning task summaries, and the modpack's current `docs/utility-xp.md` /
`docs/imoen-spellhold-xp.md`. Release state is a snapshot, not a promise that
other repositories will remain unchanged.
