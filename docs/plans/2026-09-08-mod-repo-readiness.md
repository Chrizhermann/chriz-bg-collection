# Mod repository readiness and existing-game update candidates

Read-only snapshot: 2026-09-08. Compared the current CEBG recipe's pins with
GitHub releases/default branches, local unfinished work and the owning tasks.
This is a quick coordination pass, not another code audit or blanket gameplay
acceptance. Do not change curation simply because an upstream version exists.

Follow-up: the [combined-playtest plan](2026-09-08-combined-playtest-and-alpha16.md)
records exact assembly constraints. Crucially, snapshot continuity **190 conflicts
with released Sarah 190** and must be ported/renumbered (proposed 199); never replace
the current modpack with that old snapshot. SoD PR21 filler-removal code at
`6c155d8` is also available for experimental testing; bridge issue 14 remains design-only.
These are source-preparation requirements, not a demand for public releases before testing.

## Released versus unfinished

| Repository | Current CEBG pin | New work / next action |
|---|---|---|
| Artisan's Kitpack balance fork | `chriz-v1.3.1`, still Latest | Remote master `136af542` includes Hivemaster's 25% spell failure, Magekiller's Imprison Arcana removal/Witchbane Strike, and Assassin changes. **Not released**: owning tasks explicitly said no release yet; Assassin and Witchbane need live testing. Kit-description repair is separate uncommitted work in **Fix missing kit descriptions**. |
| Bardic Wonders balance fork | `v2.9c-balance.3` | **`v2.9c-balance.4` released**, commit `37d3e5a`: Skald/Dancer/Jester and shared HLA corrections plus the reviewed kit adjustments. Next straightforward recipe-pin update; preserve Darkbloom/SR exclusion. 116 automated tests and 42 syntax checks reported; full live combat acceptance remains open. |
| chriz-bg-modpack | `v0.2.0-alpha.5`, still Latest | Companion continuity 190/191 is preserved in snapshot `fbacb809113ff2cf8548566d66ab64d98b9e5e5b`, not a release. Imoen Spellhold XP 620 is implemented in the dirty main checkout but unreleased. Both need consolidation by their owner and live acceptance, not silent ingestion of that checkout. |
| chriz-bg-rebalance | `v0.3.2`, still Latest | Dragon 110/111 remains uncommitted in **Dragon stuff - Review open rebalance todos**, installed only into a copied test game. User combat test pending. |
| SoD Remix / chriz-sod-rebalance | `v0.6.8`, still Latest | New wizard-defense direction (issue 14) is agreed but not implemented/playtested. Current recipe already has the latest released skip/remix fixes. |
| Spell Revisions balance fork | `v4.21-chriz.3` | **`v4.21-chriz.4` released**. Its new SR/RR compatibility fix is separate `SRCB_RR_COMPAT:0`; changing the main SR pin alone does not enable it. Earlier collection deferral of SR/RR compatibility remains a decision to revisit, not implicit approval here. Lightning Bolt options are implemented in snapshot `29538896e4d9f2836833f5d925b90f7fe181c69c`, unreleased/live casting pending. Death Ward work is documentation only. |
| Item Randomiser fork | `v8.1.1`, still Latest | No new implementation gap found. Recent save audit was healthy; do not repeat the earlier repair or rerandomize the current game. |
| BuffBot | `v1.8.3-alpha`, still Latest | Recent tasks are feature feasibility/discussion, not a new release. No pin change needed. |
| EEex Remote Console | immutable `661927e`, tag `v0.2.0` | Current remote HEAD, no newer GitHub release. No pin change needed. |
| chriz-bg-script-engine | not included | Local main `eb15211`, research/implementation roadmap only; no accessible release. Not ready to add. |
| Aura balance work | not included | Remote/local main `5f8ca3`; local usability edits remain dirty, no release. Keep out of this readiness claim. |

Owning task titles above identify unfinished work; source checkout roots may be
older than the active worktrees. Do not reset, clean or publish dirty mod repos.
Snapshot hashes preserve the location of work that is not on a release branch.

## Existing-game patch candidates

Here, "patch" means **close the game, apply a backed-up targeted patch, restart**,
not writing while the engine is running. These are candidates, not permission to
modify the stream installation or promises of an automatic CEBG hotpatch feature.

| Candidate | Why practical / boundary |
|---|---|
| Klatu 2150 armor QoL | Focused WeiDU install/uninstall checks passed. Append only this new component; do not reinstall a mid-stack mod. Re-equip affected armor after restarting so saved equipped effects can refresh. |
| Artisan kit-description repair | Existing standalone tail patch links campaign 2DA rows to **existing installed** KITLIST descriptions; creates no new TLK strings or kit abilities. Strong candidate after owner consolidation; it does not import newer descriptions/balance changes. |
| Hivemaster Insect Shroud | Small runtime SPL change is a plausible surgical patch. Match the installed effects and preserve later mod edits; let old effects expire. Updated description needs deliberate TLK-aware handling, not a copied TLK. Unreleased work is not automatically deployable. |
| Selected Bardic balance changes | Runtime spells/song payloads may be patchable individually, but the full `.4` release also changes level grants, generated effects and descriptions. The existing Abettor tail patch does **not** deliver this whole update. Ask the owner for a scoped updater, not an archive overlay. |
| SR/RR compatibility `.4` | An official separate tail ZIP already exists and needs no SR reinstall. Apply after SR + relevant RR 11/12 and before affected creatures spawn/save. Already saved creatures need separate assessment. User previously deferred this collection feature. |
| Imoen XP 620 | Designed for install before Spellhold recruitment (also works while she is in the maze). Needs release and live test first. Uninstall cannot undo XP already saved. |
| Assassin / NPC continuity / dragons / broader SoD updates | Not a blanket hotpatch: changed grants/permanent effects or saved CRE/ARE state may require migration, and continuity 190/191 is a pre-EET_end new-install path. Use the owning repo's acceptance work. |

The older stream copy's WeiDU log is not the current CEBG recipe: Artisan 1.3.0,
Bardic `.3`, SoD Remix 0.6.3, BG Rebalance 0.3.0, SR `.2`, Randomiser 8.1.1 and
BuffBot 1.8.3-alpha. It has **no chriz-bg-modpack WeiDU entry**; earlier individual
save repairs are separate and do not install every modpack NPC/template change.
This is a log-based observation, not an audit of every manual resource edit.

## Recommended next sequence

1. Armor option is integrated and checked in draft collection alpha.14; keep it
   separate from unrelated pin changes. Publication and existing-game application
   have not happened.
2. Take Bardic `.4` into the next recipe update; decide separately whether to lift
   the earlier SR/RR deferral now that a released, tested tail component exists.
3. Have the mod owners consolidate kit-description and modpack work, and complete
   the remaining Assassin/Magekiller/dragon/Imoen live checks. Do not release
   Artisan merely because those commits are on master.
4. Start existing-game patch work with armor + kit-description repair, then scoped
   runtime-spell changes. Each patch needs exact target/version checks, a backup,
   preserved install history and a small smoke test. Do not replace `dialog.tlk`,
   generated tables or the whole override directory from a different installation.

Sources: [Bardic `.4` release](https://github.com/Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch/releases/tag/v2.9c-balance.4),
[Artisan merged work](https://github.com/Chrizhermann/The-Artisan-s-Kitpack-Chriz-Balance-Patch/commit/136af5422cfa3de14d707c7189088b1ea55d050e),
[SR `.4` release](https://github.com/Chrizhermann/chriz-spell-revisions-patch/releases/tag/v4.21-chriz.4).
Modpack's `docs/imoen-spellhold-xp.md`, Bardic's `docs/bard-balance-audit.md`,
and the owning task endings retain the detailed test scope.
