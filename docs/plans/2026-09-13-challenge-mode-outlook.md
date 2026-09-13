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

## Complete rule inventory

| Source | Rule / requested change | Preparation and boundary for later work |
|---|---|---|
| New request | Challenge-oriented SoD changes | Source inventory on September 13 identifies implemented optional SoD 257: two Insane bridge sequencers, requiring regular bridge 256. The final split has offline installer checks but awaits native acceptance. Reuse it through the next-release intake; this is not implementation of the full rules below. |
| New request | Stronger dragons | Reuse BG Rebalance's existing dragon work, not a second implementation. Check its then-current release and SCS/EEex prerequisites. |
| 1 | No exploits | Keep as player guidance unless a specific exploit has a small, safe fix. No blanket detection project. |
| 2 + new request | No using wands; original rule allows selling them. Requested enforcement: remove all wands and replace them with scrolls | Inventory the actual collection's sources and agree replacement spells/counts later. The new replacement approach changes the original selling allowance/economy; preserve this distinction instead of silently equating the two. Protect quest dependencies and scripted users. |
| 3 + new request | No traps; Skull Trap only after combat starts. Untriggered Skull Traps explode automatically after one round | Treat party trap-setting separately from world traps/enemy encounters. Include kit/HLA trap sources in the audit. Keep both Skull Trap requirements: the timer alone does not enforce combat-only casting. Verify the actual SR/non-SR spell variants later. |
| 4 + new request | No Arrows of Dispelling against enemies; remove almost all from availability, at least BG1 | Identify BG1 sources first; later-game removals and any retained exceptions need an explicit list. “Almost all” is not approval for deleting every copy globally. |
| 5 | At most one Wish per combat | Candidate enforcement; settle combat boundaries and cast/clone accounting before implementing. Do not silently reinterpret as one per character. |
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
