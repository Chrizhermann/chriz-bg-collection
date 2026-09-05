# First-alpha intake from the owning mod repositories

Checked 2026-09-05, approximately 23:20 KST. Christopher permits work completed
today to enter the first public alpha; newer unfinished components need not hold
up the installer. This is an intake queue, **not an already changed recipe**.

## Preserve the current acceptance run

R5 remains the frozen alpha.8 recipe: 43 runs / 430 components. Its worker was
still active in SCS during this check. Do not change its pins, selection, ledger
or receipt, restart it for each new mod release, or deploy anything into the
stream game. Additional accepted work belongs in the next versioned candidate.
R5 cannot provide acceptance evidence for versions/components it did not install.

## Ready and in-progress additions

| Work | Verified state | First-alpha disposition |
|---|---|---|
| Modpack progressive utility XP, **610** | Public **v0.2.0-alpha.3**, newer than pinned alpha.1. Component implemented; owner records successful native startup/load/save and one scribing award, plus Christopher's successful test. Rounding was subsequently changed and covered automatically. | Ready for recipe intake as the requested utility-XP replacement. Requires EEex; place after EET finalization and other utility-XP tweaks. Add its catalog/feature/order declaration and an explicit curation decision; do not guess mandatory versus default/optional. |
| Bardic Wonders Abettor fixes | Public **v2.9c-balance.3**, newer than pinned balance.2. Affects already-selected **1004/2004/2007**. | Queue pin update without changing those selections. Do not add the separate existing-playthrough repair component to a fresh installation. Preserve Darkbloom 1006's Spell Revisions exclusion. Ordinary party invisibility at Symphony start/end remains an owner's known unverified behavior, not a newly imposed release blocker. |
| Yoshimo / Hexxat choices | Owning task **Add Yoshimo and Hexxat kits** is executing the user's merge/tag/package/release request. New unpublished IDs are being moved to **220–223** because 190/192/193 already belong to other NPCs. | Eligible when the actual release asset is available. Verify final IDs before wiring Yoshimo Swashbuckler and Hexxat Shadowdancer / Fighter-Thief / Assassin. Preserve Hexxat exclusivity. Expose alternatives without inventing a new checked default. Owner's focused review/tests are sufficient; the owner explicitly did not make another live test a release blocker. |
| Imoen Spellhold XP, **620** | Implemented locally, requires EEex, 186 tests and installer checks reported. Average of the other party members, capped at 3 million mage XP; user's recruitment test has not been reported complete. | Pending owning-task acceptance/release, not part of the published alpha.3 payload. Keep separate from utility XP 610. |
| New BG Rebalance dragon work | **Review open rebalance todos** is still discussing/researching encounter design. Public release remains the already-pinned **v0.3.1**. | WIP. Recheck the eventual release delta; do not treat discussed dragon rules as implemented components. Existing accepted ambient/urgent 120/121 stay included. |
| Optional SoD skip | Prototype and synthetic checks exist; item-import adapter/native transition acceptance remain unfinished. Released **v0.6.5** is unchanged. | Keep the released SoD selection; skip can join when actually ready, otherwise an early follow-up alpha. Separate from deferred 290. See the existing [handoff](../handoffs/2026-09-05-optional-sod-skip.md). |
| SR/RR AI compatibility | Separate owning-repo branch; public SR remains the already-pinned **v4.21-chriz.3**. | Previously deferred; no newly verified release to ingest. Do not reopen the design or hold the installer for it. |
| Script engine | **Begin script engine Task 1** reports infrastructure Task 2 completed, later tasks pending/in progress. | Not yet a released collection mod. No automatic lab setup or recipe entry. |

The public modpack alpha.1 -> alpha.3 TP2 comparison adds **only 610**: no
existing component IDs were removed or renumbered. Alpha.2 was not published
as a release because a Windows temporary-path build check failed; alpha.3
includes the packaging/test-harness correction. Public 190 is Sarah, 192 is
Viconia and 193 is Shar-Teel, not the unpublished Yoshimo/Hexxat numbering.

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

Sources: [modpack alpha.3 release](https://github.com/Chrizhermann/chriz-bg-modpack/releases/tag/v0.2.0-alpha.3),
[public modpack tag comparison](https://github.com/Chrizhermann/chriz-bg-modpack/compare/v0.2.0-alpha.1...v0.2.0-alpha.3),
[Bardic balance.3 release](https://github.com/Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch/releases/tag/v2.9c-balance.3),
the owning task summaries, and the modpack's current `docs/utility-xp.md` /
`docs/imoen-spellhold-xp.md`. Release state is a snapshot, not a promise that
other repositories will remain unchanged.
