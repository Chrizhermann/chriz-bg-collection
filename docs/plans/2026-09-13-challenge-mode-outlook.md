# Challenge Mode — future scope and rule inventory

**Status:** Requested roadmap item, 2026-09-13. Record and prepare only; no
implementation, component selection, release commitment or current-game changes.
This must not delay the next installer patch. Scope comes from Christopher's
message and accompanying 16-rule screenshot, not a new balance design.

## Direction locked in

Provide Challenge Mode components that enforce Christopher's run rules where
implementation is straightforward and safe. Combine the relevant SoD changes,
stronger dragons and anti-cheese restrictions. Keep individual gameplay work in
its owning mod repo; CEBG only groups, selects, explains and pins those components.
Future default/opt-in policy and the exact bundle remain to be agreed; do not
silently impose new restrictions on the current recommended installation.

Allow partial delivery: describe exactly which rules are enforced, which are
partly enforced and which remain player rules. Do not promise an exploit-proof
mode or build elaborate enforcement for low-value edge cases.

September 29 clarification: place **Challenge Mode** among CEBG's main prominent
mod checkboxes, with multiple individually explained component choices beneath
it. Do not hide it among advanced settings or make it an indivisible bundle.
This is future UI scope, not a change to the current install or release candidate.

October 7 approved next-release policy: **Emotion: Courage and Hope do not stack**
(existing BG Rebalance component 301) belongs in Challenge Mode, not the normal
recommended defaults. Make it optional and unchecked in the next release, retain
its individual Customize choice, and select it when the Challenge Mode group is
introduced. Do not delay the default change until the whole mode exists. Reuse the
existing component without changing its mechanics. Published alpha.19 and the
current stream installation retain their existing selection. This entry records
the decision only; implementation is queued in the
[next-release list](2026-09-20-next-work-index.md#next-release--october-7-approved-addition).

October 4 clarification, reaffirmed October 7: include **Baeloth: standard
sorcerer spells known** alongside Emotion as an
individually selectable part of planned Challenge Mode. Companion gameplay code
stays in `chriz-bg-modpack`. This supersedes the older owner's collection-default
proposal; do not silently add it to the normal recommended setup. Christopher
recalls applying the earlier personal change through EE Keeper, which is not an
installed component or evidence of a released implementation.

October 7 released status supersedes the earlier design-only snapshot:
**Modpack 235** now ships as an optional, unchecked choice in CEBG alpha.19
(Modpack alpha.10). Select this existing component with Challenge Mode, retain
its individual choice and SR/non-SR spell selection, and keep it off in normal
defaults. The remaining task is grouping, not gameplay implementation. The
original owner design is `docs/companion-spellbooks.md`; do not duplicate it.
This follow-up records next-release work only, with no changes to the published
release, saves or current game.

## Complete rule inventory

| Source | Rule / requested change | Preparation and boundary for later work |
|---|---|---|
| New request | Challenge-oriented SoD changes | Source inventory on September 13 identifies implemented optional SoD 257: two Insane bridge sequencers, requiring regular bridge 256. The final split has offline installer checks but awaits native acceptance. Reuse it through the next-release intake; this is not implementation of the full rules below. |
| New request | Stronger dragons | Reuse BG Rebalance's existing dragon work, not a second implementation. Check its then-current release and SCS/EEex prerequisites. |
| October 7 addition | Emotion: Courage and Hope do not stack | Reuse BG Rebalance 301. Next release: off in normal defaults, individually selectable; selected within Challenge Mode when available. Published alpha.19 and the current stream game stay unchanged. |
| October 4 addition, reaffirmed October 7 | Baeloth has a legal sorcerer starting spellbook | Reuse released Modpack 235, already optional in alpha.19. Group alongside Emotion and select with Challenge Mode; keep individually selectable and off in normal defaults. Preserve SR/non-SR spell choices and unrelated traits. No current-game changes. |
| 1 | No exploits | Keep as player guidance unless a specific exploit has a small, safe fix. No blanket detection project. |
| 2 + new request | No using wands; original rule allows selling them. Requested enforcement: remove all wands and replace them with scrolls | Inventory the actual collection's sources and agree replacement spells/counts later. The new replacement approach changes the original selling allowance/economy; preserve this distinction instead of silently equating the two. Protect quest dependencies and scripted users. |
| 3 + new request | No traps; Skull Trap only after combat starts. Untriggered Skull Traps explode automatically after one round | Treat party trap-setting separately from world traps/enemy encounters. Include kit/HLA trap sources in the audit. Keep both Skull Trap requirements: the timer alone does not enforce combat-only casting. Verify the actual SR/non-SR spell variants later. |
| 4 + new request | No Arrows of Dispelling against enemies; remove almost all from availability, at least BG1 | Identify BG1 sources first; later-game removals and any retained exceptions need an explicit list. “Almost all” is not approval for deleting every copy globally. |
| 5 | At most one Wish per combat | Candidate enforcement; settle combat boundaries and cast/clone accounting before implementing. Do not silently reinterpret as one per character. |
| September 29 addition | Wish's Improved Haste uses the Spell Revisions version when SR is installed | Separate Challenge Mode component choice. Replace the vanilla Improved Haste effect granted by Wish with SR Improved Haste; no change requested for non-SR installs or other Wish results. Coordinate with the SR owner and check the effective Wish effect/provider when implementing. This is independent of the once-per-combat Wish rule; record only now, no source research, implementation or current-release addition. |
| 6 | Each character may memorize at most one Time Stop | This is a prepared-copy limit, not automatically a once-per-fight casting limit. Check kit and spell-system interactions before choosing enforcement. |
| 7 | No explosive arrows | Candidate availability/use restriction; identify all relevant variants without removing unrelated ammunition. |
| 8 | Stealing limited to five items per shopkeeper | Lower-priority enforcement unless simple. Decide item-versus-stack and persistent shopkeeper accounting later. |
| 9 | No farming | Player rule until particular repeatable rewards/spawns are identified; do not globally remove respawns or rest encounters. |
| 10 | No Balthazar | Retain the rule verbatim. Clarify the prohibited benefit/participation before altering Ascension or story progression; do not infer removal of the NPC. |
| 11 | No Protection from Magic or Protection from Undead scrolls or potions | Target those consumables, not all protection spells or unrelated effects. Check quest/scripted uses and mod-added equivalents. |
| 12 | No Ravager transformation | Target the player transformation once its actual provider is identified; not the unrelated encounter by the same name. |
| 13 | No off-screen spellcasting before combat starts | Lower-priority enforcement unless safe and simple; preserve legitimate targeting and scripted casting. |
| 14 | Current prebuff scripts trigger much faster than regular SCS; no additional restrictions needed | Context and existing behavior, not a request for more prebuff restrictions. Reuse the owning repo's accepted ambient/readiness work. |
| 15 | Do not remove party members during combat to escape it | Candidate enforcement distinct from area-exit blocking; preserve forced story/companion transitions. |
| 16 | Most rules do not apply to SoD filler quests | Mandatory scope exception to resolve per rule/encounter before enforcement. “Most” is not a blanket exemption for all SoD. |
| New request | Prevent leaving an area while selected fights are active | Explicit encounter allowlist only. Some fights require inter-area movement; keep those transitions available. Never globally lock every exit whenever any hostile exists. |

## Small preparation batches, when this work is picked up

1. Ask the existing SoD and BG Rebalance owners for current component/release
   mappings and acceptance evidence. Historical starting points are the
   [combined test plan](2026-09-08-combined-playtest-and-alpha16.md): SoD bridge/filler
   work and BG Rebalance dragon 110/111. The
   [September 13 release inventory](2026-09-13-next-release-scope-and-acceptance.md)
   identifies newer SoD 257 and the current candidate sources. Historical snapshots
   are not proof of native acceptance for later revisions.
2. Start with bounded content restrictions/replacements and existing encounter
   improvements. Suggested owner for shared gameplay-rule enforcement is
   `chriz-bg-rebalance`; route content replacements to `chriz-bg-modpack` if that
   matches its maintained scope. SoD stays in SoD Remix. Spell-specific changes
   must coordinate with the Spell Revisions owner; do not put mod code in CEBG.
3. Before area locks, agree the fight/exit allowlist and exceptions. Require reliable
   restoration after victory, interrupted combat, save/reload and scripted travel;
   account for split parties and do not strand the player. Preserve original exit
   state rather than indiscriminately unlocking everything afterward.
4. At component intake, check Randomiser and loot-source ordering, added items,
   SR on/off, kit abilities, SoD skip/filler exceptions and existing dragon/SCS
   changes. Item removal across a campaign cannot automatically honor a local
   filler-quest exemption; choose that policy explicitly before implementation.
5. Later CEBG work: a clearly explained Challenge Mode group with real dependencies
   and exclusions, versioned selections and honest coverage labels. Existing-save
   applicability needs separate assessment; no automatic hotpatch promise.

No component numbers are reserved and no owners are dispatched by this note.
The website may eventually describe this as **Planned**, through the existing
[website coordination](2026-09-06-release-intake-and-website-roadmap.md), not as a
shipped feature or a promised release date. Detailed balance decisions can wait.
