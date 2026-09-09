# One combined playtest and the next CEBG release

Status: local combined installation completed and accepted, 2026-09-09.
No stream patch, public upload or updater-feed change has started.

**Hold resolved (2026-09-08):** the bridge and filler changes now share SoD commit
`3b34eaee19dcb9043f3b72c77bf5b92adbed05ca`. Modpack integration is complete at
`85cbc42551ca256b167cfdb1ffc4bd3c22df85e4`; Christopher asked us to continue.
Assemble one local experimental recipe with frozen sources, then start the isolated
test. This does not publish experimental mods or change the public recipe.

Assembly is complete: **446 components / 50 runs**, retaining every one of the
435-component baseline selections and adding eleven intended experimental
components. Seven hash-pinned local source archives use ordinary verified manual
intake; existing official downloads are reused. Run location:
`C:\Users\chris\CEBG-Tests\Combined-20260908`. Source lock, frozen development
recipe and worker logs are under ignored `target/combined-playtest-20260908/`.
The receipt uses a local recipe digest, not a public release identity. Installation
is complete: **446/446 (27 BG1 + 419 BG2)**, including BuffBot; BG Radar Overlay
2.5.2.0 is installed too. Native launch and gameplay acceptance remain pending.

SoD 256 initially failed its donor-effect preflight. A supervised recovery used
source fix `29e123a79b9f03334ab88ce93e28c300b287a8e0`, preserved all 391 completed
components, appended only corrected 256, then finished the ten untouched frozen
runs. No full reinstall or sibling replay occurred. The separate recovery receipt
passes a fresh CLI report; original failure evidence and frozen recipe remain
immutable. See the top of `docs/handover.md` for exact evidence and launch paths.
Keep this successful installation for the checkpoints below; do not restart it.

## Decision

Use **one fresh SR-enabled EET installation with several disposable test saves**
for the maximum compatible implemented set. Do not require a full reinstall per
feature or a public mod release merely to test a frozen development snapshot.
Record each source commit plus any allowlisted patch and its checksum first.

A stream clone is suitable for testing individual existing-game patches, not a
faithful all-feature installation. The stream is older than the current recipe,
has no modpack install entry, and is already EET-finalized. Changing source folders
does not apply patches; copying finished override/TLK/KEY files can lose later-mod
changes and does not recreate install-time processing or saved actors.

The previous Alpha13 full-SR test folder is gone; no reusable early full checkpoint
was found in the inspected test roots. The retained r5 copy is an older alpha.8
recipe with supervised recovery, not an unmodified pre-EET checkpoint. Reuse cached
verified downloads and, only if validated separately, the unchanged prepared BG1
source. Do not promise existing CLI support for importing an arbitrary checkpoint.

## Candidate inputs and assembly work

| Work | Exact source / assembly requirement | One combined SR-on install? |
|---|---|---|
| Armor QoL | Released Klatu 1.7.4, component 2150; already in draft recipe | Yes; late, before BuffBot |
| Bardic balance | Released `v2.9c-balance.4` | Yes for supported selections; Darkbloom 1006 remains off with SR |
| Artisan Assassin / Magekiller / Hivemaster | `136af5422cfa3de14d707c7189088b1ea55d050e`; relevant components 7004 / 7002 / 5002 | Yes; install at the normal kit stage, test new characters and level grants |
| Artisan kit descriptions | Dirty worktree `5f1b`; production `ArtisansKitpack/lib/kit_strref.tpa` diff applies cleanly to the above commit | Yes after snapshotting that explicit diff. The separate repair-only tail patch is for existing stacks, not also needed in the fresh build |
| SR Lightning Bolt | Snapshot `29538896e4d9f2836833f5d925b90f7fe181c69c`, based on SR `.3` | Choose 80 **or** 81, never both. Install after spell modifiers; CDTweaks 2530 must stay off (currently unselected) |
| SR/RR compatibility | Separate released `.4` `SRCB_RR_COMPAT:0` package | Experimental test candidate after SR and RR 11/12; public curation deferral is not lifted. Separate package avoids merging the conflicting SR `.4` and Lightning source trees |
| SoD filler-removal work | Integrated with bridge at `3b34eaee19dcb9043f3b72c77bf5b92adbed05ca`, unreleased despite TP2 label 0.6.8 | Add 135 and 265; use corrected 175/230. Preserve TP2 order including 175 after 170/180 and 265 after 260. Do not add repair-only 176/235 to a fresh install |
| New SoD bridge encounter | Same integrated SoD commit, component 256 | Ready for installation; installer/resource checks passed, native fight pending. Use a pre-first-BD2000-entry save |
| Modpack continuity and Safana arrival | Integrated `85cbc42551ca256b167cfdb1ffc4bd3c22df85e4`; continuity 199, Safana **189** | Current released companions preserved. Continuity runs before EET_end, after companion/kit changes; Safana remains late |
| Safana in Amn prerequisite | Official `RoxanneSHS/SafanaBG2` tag `v05`, component 0 | Add only to this experiment, before modpack 189. The recorded core-mod gate was its arrival inventory cleanup, which 189 now supplies. Do not enable the unfinished Bard/Abettor conversion |
| Imoen Spellhold XP | Same integrated modpack commit, component 620 | Ready for installation; after EET_end and every IMOEN2.BCS replacer |
| Dragons | Dirty `a947` rebalance checkout based on `d31fda2`, components 110/111 | Yes after exact snapshot; requires SCS Smarter Dragons 6540; 110 also requires EEex |

### Modpack integration resolved

The owning task delivered a clean integration based on public alpha.5: Sarah **190**
and all released companion components are preserved; continuity uses **199**, and
Imoen **620** is integrated from an explicit dirty-source allowlist. Safana uses
**189**, not 191: the earlier plan missed the private Sarah portrait reservation.
Do not reintroduce either obsolete mapping from the old snapshot.

Owner handoff: `modpack-combined-playtest/chriz-bg-modpack/docs/testing/2026-09-08-combined-playtest-source.md`.
375 tests passed; two optional installed-source checks skipped. Native transitions
remain pending. Pin the exact commit, not its retained alpha.5 version label.

The base curated recipe deliberately has no Safana run. Component 189 cannot be
tested without it: add the official Safana core and its verified source to this
local recipe together with the cleanup. This resolves that experimental dependency,
not the separate unfinished Safana class/spell preset or public curation decision.

## Order and test saves

Keep the current curated install order except for required explicit additions:
SCS remains in the main phase **before** EET_end; keep the existing approved SoD
post-EET_end placement rather than moving the whole stack. Split modpack delivery
as needed so companion conversions precede continuity and continuity precedes
EET_end, while Safana 189 and its late spell/XP repairs remain late. The generated
recipe now has that pre-finalization split; it does not leave required companion
conversions after continuity. Preserve SR's late component 60 scan. Dragons run
after SCS 6540; Imoen 620 after the final IMOEN2.BCS writer. Lightning **80** follows
SR60 and the other spell modifiers, then Klatu 2150 and BuffBot last. SR/RR
compatibility runs after both core mods and before SCS; SR60 only scans joinable
NPCs, not the five hostile RR actors repaired by that compatibility component.

One installed game can host these independent disposable save checkpoints:

1. **Kits/bards:** new characters and level-up checks for Assassin grants, Cloak,
   Death Attack and Expose Weakness; Magekiller/Witchbane; Hivemaster retaliation;
   changed bard songs/HLAs and kit descriptions. Imported actors can retain old
   permanent effects, so are not a substitute for grant tests.
2. **Spells:** a fresh mage for Lightning geometry, saves, Mirror Image, wands,
   traps and reflection; fresh RR actors for spellbooks and casting guards.
3. **Continuity:** a new BG1 campaign, real recruitments, recognizable permanent
   progression, then BG1-to-SoD-to-SoA transitions and save/reload. Safana before
   her first SoA arrival; ensure later earned BG2 gear is not repeatedly cleared.
   Companion regression: Kivan's sea-elf encounter dialogue must complete before
   SCS combat starts. Use a pre-encounter save; modpack 130 is already installed
   and its two guards match the verified standalone fix. Do not add a second fix
   or treat it as a repair for dead actors in a previously failed encounter.
4. **SoD:** saves before the ambush, Liia payment, and first visits to affected
   areas; check removal, compensation and retained quest/loot behavior. Use the
   original transition rather than only teleporting to a late area. For the bridge,
   use a save before first entering BD2000; a visited-area pre-fight save is insufficient.
5. **Spellhold:** before first Imoen recruitment in AR1512/13/14, known party XP;
   verify average/cap and no repeat award after rejoin or reload.
6. **Dragons:** fresh encounter/actor at the intended SCS difficulty; verify melee
   delivery, death protections, wing-buffet spacing and save/reload.

Alternate Lightning settings can be tested sequentially at their own tail after
restoring the local checkpoint; no need to repeat BG1/EET/SCS for that alone.
Darkbloom without SR is a different configuration and cannot be called accepted
by the primary SR-on test. This plan is not exhaustive gameplay coverage.

Before any test launch, use a unique managed game name and separate Documents save
profile. A plain stream-folder copy retains its original `engine_name` and could
otherwise share saves. Use real copies, not writable hardlinks/junctions to the
stream. Keep test saves separate; do not copy them back into the stream game.

## Public release preparation, independent of experiments

Prepare **app alpha.16 / collection alpha.14** with startup loading feedback,
Klatu 2150 and released Bardic `.4`. Keep current other pins/curation unchanged.
This is not an approval to publish unreleased Artisan, modpack, dragon or SoD
branches, or to change the SR/RR decision. Release notes explicitly state that
existing installs and saves are not modified.

App version markers, notices and draft notes are prepared. Frontend TypeScript,
146 tests and production build pass; 3 native package-contract tests pass. The
signed-setup check is intentionally not claimed without a new packaged setup.
Recipe verification is recorded in the handover after the Bardic pin lands.
Packaging, signature/update-download acceptance, publication and live combined
testing remain separate outstanding steps.
