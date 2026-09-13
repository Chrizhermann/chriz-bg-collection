# Existing-install updates: quick triage

Status: **source review and proposed policy only**, 2026-09-14. No game changes,
native tests or patch-engine implementation in this pass. Companion to the
[next-release plan](2026-09-13-next-release-scope-and-acceptance.md).
Do not make a general hotpatch engine a prerequisite for the upcoming release.

## Remaining tests: small but meaningful

- Artisan/Bardic: new characters, console XP, then **normal level-up/HLA selection**
  and actual ability use. Giving abilities directly bypasses grant testing. Use a
  target/ally where needed and one save/reload. New-character success does not prove
  migration of existing characters. Abettor's start/end party invisibility remains
  a specifically documented unverified effect, not proof its entire kit is broken.
- SoD: retain Christopher's substantial accepted playtesting. Khalid 115 needs its
  focused fort route. Owner notes distinguish final 257/266 from earlier bridge
  combat acceptance; a short final sanity check, not another campaign, is enough.
- Dragons: actual combat is pending. These belong to the optional challenge
  offering, not automatic difficulty changes for existing users. 110 gates lethal
  melee on high difficulty; 111 independently changes buffet timing at all supported
  difficulties. Test those separately in disposable saves.
- Modpack/SR: reused code reduces implementation risk, not transition/order risk.
  Retain one real Xan/Yeslick transition, Safana arrival/gear-retention, Imoen
  XP/rejoin persistence and a few Lightning casts. No full playthrough repeat.
- Installer: keep the final packaged-install and updater/pause smoke. No new
  exhaustive review, but implemented code alone does not establish native acceptance.

## Patch candidates and boundaries

“Candidate” means worth preparing/testing, **not already proven for automatic public
delivery**. Patch with the game closed and fully restart afterward.

| Change | Existing-game assessment |
|---|---|
| Artisan kit descriptions | Good first candidate: separate, uncommitted `AKCB_KIT_DESCRIPTIONS` checks installed kit components and links campaign description tables to that install's KITLIST HELP strings. No new text, abilities or saved actors. |
| Armor thieving / racial kit availability | Focused components are candidates after relevant item/kit writers. Preserve unrelated restrictions. Racial availability affects future character creation, not an existing character's kit. Avoid reinstalling an already-installed middle-stack component. |
| Fighter modals / Hivemaster spell effects | Plausible resource patches for existing owners after old effects end and a fresh activation. No dedicated retrofit identified. Patch effective installed resources; copying original SPLs could overwrite later SCS changes. |
| Assassin / Magekiller grants | No complete saved-character migration established. Assassin compatibility helpers do not prove all old effects/grants are repaired. Magekiller deliberately retains old Imprison Arcana; a new HLA choice does not retroactively refund/grant a spent choice. Exclude from the first automatic patch batch pending a dedicated migration. |
| Bardic Wonders | Dedicated Abettor tail exists for 1004 + finite-song controller 2004; it does NOT deliver all other bard changes. Other resource fixes need narrow adapters; CLAB grants and saved passives are separate. Let old songs expire before a fresh activation. |
| SR Lightning | Snapshot `29538896` has `SRCB_CLASSIC_LIGHTNING`: tail 0 corresponds to core 80, tail 1 to core 81. Resolves local IDs, preserves foreign effects/SCS hooks and handles local descriptions. Requires SR; rejects CDTweaks 2530 and unsupported shapes. Never stack core and tail variants. Restart; refresh projectile protections/re-equip reflection gear for 81. Preference change: opt-in, not an automatic bug fix. |
| SR protection refresh | Existing `SRCB_PROT_REFRESH` is a narrow supported-stack tail candidate. Future casts change; active saved buffs do not. Already-fixed versions should be recognized, not patched again. |
| Modpack Safana 189 | Append after Safana and the final arrival-script writer, before first SoA arrival. Deliberately does not strip gear from an already-arrived Safana. Do not automatically add the companion or unfinished conversion. |
| Modpack Imoen 620 | Existing games before Spellhold recruitment, or Imoen already in the party but still in the maze. No effect beyond the maze. Requires EEex and supported final script. Uninstall does not reverse saved XP. |
| Modpack continuity 199 | **New BG1 campaign, proper pre-EET_end order.** Changes actor identities/references and cannot rename actors embedded in saves. Reused EET mechanisms do not make this an existing-save patch. |
| SoD Khalid 115 | Purpose-built append component requiring 110. Before first Bridgefort briefing, ideally before SoD starts; original-route fallback remains. Not a migration for contradictory scenes already played. Native acceptance pending. |
| Other SoD changes | Individual assessment: 257 requires matching current 256 and pre-spawn state; 266 has a narrow script correction. Area/loot/actor changes can miss cached instances. Repair components 176/235/291 target specific older states, not a blanket repair set. |
| Dragons 110/111 | Runtime scripts/additive resources make later installation plausible after supported SCS; 110 also needs EEex. Owner explicitly withholds saved-encounter/combat acceptance. Never inject optional challenge difficulty simply because SCS is installed. |
| BuffBot / Radar | BuffBot 1.8.4 is a promising bounded upgrade after changed-runtime/generated-file and settings checks. Do not copy a whole override. Radar already has an independent update path; neither establishes generic mod-upgrade support. |

SR/RR's earlier public-selection deferral remains. Fresh placement is after SR/RR
and **before SCS**. A separate tail does not prove safety for every already-generated
SCS script or saved enemy spellbook; no automatic retrofit approved here.

## Why component detection is necessary but insufficient

WeiDU.log records components/order, not every current resource or campaign state.
Use receipt provenance plus the actual relevant effective files (override or BIF):
known hashes for exact replacements, or tested structural conditions for patches
that preserve unrelated content. No complete game scan on every startup is needed.
Later mods, customization and manual edits can change the patch's inputs.

Party actors are embedded in saves; visited areas/stores can also be cached.
Replacing an override CRE/ARE/STO therefore does not generally update its saved
instance. Future casts can use changed spells while old effects remain applied.
File extension alone proves neither safety nor effectiveness.

**TLK changes are not categorically forbidden.** Resolve/append text using the
target installation's WeiDU and repoint only intended consumers. Preserve existing
string numbers, actual language and female TLK where applicable. Never copy another
install's dialog.tlk or compiled files with its string numbers; do not rewrite a
shared string without checking its other users.

Concrete owner evidence: isolated Khalid 115 installation changed 37 audited
resources, preserved all 354,856 existing TLK entries and added 55. This demonstrates
installation-local text delivery, not native quest acceptance or universal safety.

## Small automatic-patch contract, when implemented

1. Publish an authenticated, explicit patch list with supported component versions,
   prerequisites/conflicts, resource checks and save applicability. New optional
   gameplay requires opt-in; never re-enable a user's exclusions.
2. Distinguish eligible, already fixed, not selected and unsupported/unknown.
   Unknown means no write and an explanation. A mod list cannot prove a quest has
   not happened: prefer audited runtime guards that safely do nothing outside the
   supported state. Otherwise keep delivery conditional/manual until a selected-save
   check exists. One latest save does not prove every save the user may load is safe.
3. Close game/writers, lock the install and preflight the dependency-linked batch.
   Back up affected resources, TLKs and WeiDU/patch bookkeeping; preserve a pre-update
   save checkpoint. Use purpose-built tail patches or validated transactions, not
   middle-stack reinstall cascades. Failed preflight leaves the install unchanged.
4. Verify changes and append a patch receipt linked to the immutable original.
   Record applied/skipped/failed entries and checksums. Interrupted mutation needs
   verified recovery before launch; do not leave half a dependent patch accepted.
5. Display **“Collection alpha.13 + fixes A/B; C requires a new game”**, not the
   entire newest recipe version. Unsupported selected fixes remain visible;
   irrelevant absent mods can be omitted. Explain restart/recast/event limits.
6. Rollback restores the matching installation backup, not XP or abilities saved
   afterward. Returning to the old state may also require the pre-update save.
   Do not promise unrestricted uninstall-after-play.

CEBG already has applicability labels in `engine/src/updates.rs`, receipts, and
separate app/Radar updates. `app/src/screens/updates.ts` still offers **Create updated
installation** for collection changes. Labels are not a patch executor/save scanner.

Recommendation: prepare a small named batch, starting with accepted table/resource
fixes. Prove install, unsupported-input rejection, rollback and an existing-save
restart smoke on an isolated copy. Keep saved-character migration and broad upgrades
outside the first batch; do not hold up the new-install release for this capability.

## Evidence pointers

- [IESDP TLK format](https://gibberlings3.github.io/iesdp/file_formats/ie_formats/tlk_v1.htm),
  [saved party data](https://gibberlings3.github.io/iesdp/file_formats/ie_formats/gam_v2.0.htm),
  [WeiDU documentation](https://weidu.org/WeiDU/README-WeiDU.html).
- Artisan `136af542`, modal `4602fd1`, worktree `5f1b`'s
  `live-patch/AKCB_KIT_DESCRIPTIONS/setup-AKCB_KIT_DESCRIPTIONS.tp2`.
- Bardic `37d3e5a`: `live-patch/abettor-hla/README.md` and installer.
- SR `29538896`: `live-patch/SRCB_CLASSIC_LIGHTNING/README.md` and installer.
- Modpack `85cbc425`: `docs/companion-continuity.md`, `docs/imoen-spellhold-xp.md`.
- SoD `docs/plans/2026-09-10-khalid-continuity.md`, `docs/releases/v0.6.9.md`;
  BG Rebalance dragon worktree `a947` README 110/111.
