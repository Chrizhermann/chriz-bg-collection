# One combined playtest and the next CEBG release

Status: feasibility/source preparation, 2026-09-08. No new game installation,
stream patch, public upload or updater-feed change has started.

**User-approved hold (2026-09-08):** wait for the SoD bridge work (issue 14)
to have an implemented test source before starting the combined installation.
Include it with the implemented filler-removal changes in one frozen SoD test
input. A public release or prior live acceptance is not required to test that
snapshot. Once ready, refresh the exact commits and component requirements, then
assemble the combined build once. Preserve the prepared alpha.16 source and the
other assembly tasks below; do not start an incomplete combined install meanwhile.

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
| SoD filler-removal work | PR21 source `6c155d83139c2bed4f518e811c4be4632f3c76e3`, unreleased despite TP2 label 0.6.8 | Add 135 and 265; use corrected 175/230. Preserve TP2 order including 175 after 170/180 and 265 after 260. Do not add repair-only 176/235 to a fresh install |
| New SoD bridge encounter | Issue 14 design, no implementation/component yet | **Not available to install**. Keep its future test checkpoint recorded |
| Modpack continuity and Safana arrival | Snapshot `fbacb809113ff2cf8548566d66ab64d98b9e5e5b` | Needs integration with current modpack first; see ID collision below. Continuity must run before EET_end, after companion/kit changes |
| Imoen Spellhold XP | Dirty modpack checkout based on `ebdb7424d4155335c2029146c5f29c8693c93bc0`, component 620 | Yes after integrating source; after EET_end and every IMOEN2.BCS replacer |
| Dragons | Dirty `a947` rebalance checkout based on `d31fda2`, components 110/111 | Yes after exact snapshot; requires SCS Smarter Dragons 6540; 110 also requires EEex |

### Concrete modpack integration fix

The continuity snapshot defines `cbm_companion_continuity` as **190**. Released
modpack alpha.5 and the public CEBG recipe already define **190 as Sarah's Archer
conversion**. Preserve the published Sarah ID; give continuity a verified unused
ID and update its labels, translations, tests and planned recipe mapping. `199`
is free in alpha.5 and is the proposed continuity ID; `191` is free for Safana.
Confirm those reservations against the owner's integrated tree. Do not
reuse the public Sarah selection or silently change what it installs.

That old snapshot also lacks released components 192–198. Integrate only the new
continuity/Safana implementation with current modpack plus 620 in a separate worktree,
preserving all current companion components and unrelated dirty files. Do not
package a whole dirty checkout or unrelated untracked documents. No existing
source snapshot yet contains that complete reconciled modpack.

## Order and test saves

Keep the current curated install order except for required explicit additions:
SCS remains in the main phase **before** EET_end; keep the existing approved SoD
post-EET_end placement rather than moving the whole stack. Split modpack delivery
as needed so companion conversions precede continuity and continuity precedes
EET_end, while Safana 191 and its late spell/XP repairs remain late. Do not add
continuity before EET_end while accidentally leaving required companion conversions
after it: the exact small pre-finalization run split needs authoring before the
test can start. Preserve SR's late component
60 scan. Add dragons after SCS 6540; add Imoen 620 after the final IMOEN2.BCS writer.
Resolve Lightning 80/81 tail order explicitly; leave BuffBot last.

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
4. **SoD:** saves before the ambush, Liia payment, and first visits to affected
   areas; check removal, compensation and retained quest/loot behavior. Use the
   original transition rather than only teleporting to a late area. Bridge test
   waits for code, not another full installation by default.
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
