# Claude handover for the next BG update

Prepared 12 October 2026; Degrodel follow-up added 13 October. Christopher wants the already-decided work finished,
with one useful combined release/test batch. He does not want another broad
research project or a fresh installation for every component.

## Start here

Use this CEBG worktree:

`C:\Users\chris\.codex\worktrees\installer-v0-real-alpha\chriz-bg-collection`

Branch: `codex/installer-v0-real-alpha`. The `4d39` checkout used by the earlier
chat is an old detached snapshot, not the integration branch.

Read applicable `AGENTS.md` / `CLAUDE.md`, `docs/handover.md`, then
[the full BG work overview](2026-10-12-bg-work-overview.md). That overview is the
cross-repository inventory, including older backlog and source/owner locations.
Use this handover to decide where to begin; do not repeat its whole audit.

First refresh the work reservations below and local status. The overview was
checked against source, owner chats and GitHub on October 12, but owners can
advance after that snapshot. Do not reset, stash away or absorb other people's
dirty work without inspecting it.

## Already started by Codex

Only tasks marked in progress are still reserved. Do not duplicate them or edit
their owned files while they are running. Results are linked in the overview's
October 12 dispatch section. Consume completed changes rather than rebuild them.

| Task | Current handover status | Owner and working area | Finish condition |
|---|---|---|---|
| Emotion default | Implemented locally; reservation released | Native worker `/root/emotion_default`, active CEBG worktree | Emotion301 optional/off, Baeloth235 still off, other defaults unchanged. [Delivery note](2026-10-12-emotion-default-delivery.md) records passing focused checks and both recipe validations. Ready for next-release integration, not published. |
| Blade and Battleluck | Local candidate ready; reservation released | Existing Bardic chat `01a07fe8-2fd5-79d3-bb42-4fef15d27fb8`, `C:\Users\chris\.codex\worktrees\bard-spell-blade-followup\Bardic-Wonders-Chriz-Balance-Patch` | Normal installable `v2.9c-balance.7-rc.1` package prepared, installer/archive checks pass. Brief player check remains. No Heroism or old song/spin/parry backlog. |
| SoD relocated-item identification | Implemented locally; reservation released | Native worker `/root/sod_loot_identification`, isolated `codex/relocated-loot-identification-oct12` branch under `C:\src\private\chriz-sod-rebalance\.worktrees\relocated-loot-identification-oct12` | Components170/900 no longer force identification, preserving original identification and existing inventory data. Focused WeiDU checks pass. No new treasure placement. |

All three workers above completed before this handover was delivered; their edit
reservations are released. Codex is handing the overview and continuation back
to Christopher/Claude. Keep one current owner/status per task. If another owner's status is uncertain,
inspect its delivery note/diff or ask the coordinator; elapsed time alone is not
evidence that its files are free.

Bardic delivery: `C:\Users\chris\.codex\artifacts\bardic-blade-battleluck-20261012\DELIVERY.md`.
The adjacent `Bardic-Wonders-v2.9c-balance.7-rc.1.zip` is the unpublished candidate;
SHA256 `d3425e82452d8862372e716f7ee68ba2d9fa24e4c4c7bb43969955d35d206879`.
Its five changed/added runtime files match the source candidate; untouched release
files are preserved. CEBG still pins public balance.6. Do not treat this archive
as already published or as a repair for an existing saved character.

SoD delivery: `C:\src\private\chriz-sod-rebalance\.worktrees\relocated-loot-identification-oct12\docs\relocated-loot-identification-oct12.md`.
The narrow source fix remains uncommitted and unreleased. It covers relocated
Korlasz/camp-chest rewards, not every other loot path or existing saved inventory.

## Best work for Claude to continue

### 1. Finish the existing Modpack candidate for integration

Use:

`C:\Users\chris\.codex\worktrees\campaign-xp-banking\chriz-bg-modpack`

Branch `codex/campaign-scroll-integration`, audit baseline `1eadb92`.
This already combines recruitment 237/238, replacement reader-level scroll 430
and XP banking 650/651. Do not replace it with the stale primary/private-archive
checkout or restart the scroll hook from October 9 notes.

Read `docs/testing/2026-10-09-campaign-scroll-candidate.md`,
`docs/scroll-casting.md`, `docs/recruitment-xp.md`, and
`docs/campaign-xp-banking.md` there. The existing owner is
**Review Dynaheir and Baeloth spells**, chat
`01a0b92c-da6d-7701-8d51-b0df1a17fe08`; check it is not editing the same files.

Concrete delivery: prepare a coherent normal installable candidate and accurate
release notes; resolve any real source/packaging defect found in this bounded
pass. Update the old blanket “not installed/no existing-save use” notes: banking 650
was privately installed for Garithos on October 12, with existing XP/levels retained.
That is not yet a successful in-game test or public release. Do not embed private
game paths, receipts, saves or captures in a public distribution.

Keep recruitment/scroll delivery separable from unresolved public banking behavior.
Banking is optional, including its optional SoA ceiling. Ask Christopher about a
small in-game bank display and the remaining no-bank dual-class rule; do not
invent either. Do not hold unrelated completed features for those decisions.

### 2. Prepare the finished Flail correction for the next intake

Source:

`C:\src\private\chriz-bg-rebalance\.worktrees\foa-compatibility-20261008`

Audit baseline `0a14531`; implementation `acbbe86`;
[Rebalance PR7](https://github.com/Chrizhermann/chriz-bg-rebalance/pull/7).

This is compatibility coverage for the already-shipped Flail balance, not another
balance design. Review the existing change/result, fix only concrete findings,
and prepare its release/intake notes. Do not automatically rerun a large lab or
rewrite the reviewed mechanism. PR merge and publication remain separate actions.

### 3. Consolidate the two finished SoD features locally

Recruitment source:
`C:\Users\chris\.codex\worktrees\early-sod-companions\chriz-sod-rebalance`

Finale source:
`C:\Users\chris\.codex\worktrees\avernus-finale\chriz-sod-rebalance`

Both start from public `v0.6.12` / `b2af832` and contain uncommitted feature work.
Read their `docs/design/wave1/11-early-bg1-companions.md` and
`11-avernus-interim-finale.md` respectively, including later approved details.
The older playthrough-followups document contains superseded, open design notes;
do not use those to undo the later finished design.

Existing chats: **Plan BG1 companion recruitment**
`01a10596-3537-7e91-9d1e-478d749ffd38` and **Rework Belifid’s end fight**
`01a0e4cd-cca5-76e0-bb08-9ebbbe82ad30`.

Work in a separate integration checkout. Preserve both original dirty worktrees.
Resolve shared installer/release changes with one integrator. Incorporate the
reserved item-identification fix only after it is delivered. Do not redesign
companions, reward amounts, boss mechanics or loot placement. Deliver a combined
candidate, exact included components, known limitations and a short player check.

### 4. Bring the selected finished work into CEBG

The Emotion worker has now released its file reservation. Wire the already-released
bard policy in its real order: Rebalance 420 → 421 → Bardic 3010. Document normal
versus optional selection explicitly; do not infer a new default from a private
game installation. Preserve the existing provider/component dependencies.

Prepare intake for delivered owner candidates without pretending an unpublished
archive already has a verified public URL/hash. Actual new mod pins wait for
their publication. CEBG bundles the recipe, not third-party mods.
The local Emotion change still carries the old version markers deliberately;
version the next candidate properly and never republish it over alpha.19.

Challenge Mode is more than Emotion/Baeloth. Consult
`docs/plans/2026-09-13-challenge-mode-outlook.md`, especially the complete rule
inventory. Propose a small initial group and its on/off/mixed behavior; Christopher
still needs to choose membership beyond the two settled components. Do not move
Flail/arrows/physical resistance and other existing defaults off by accident.

Prepare separate installer and collection changelogs, marking default/optional.
The website must describe what actually ships, not every planned component.

## Decisions that are already settled

- Emotion no-stacking is optional/off normally. Baeloth's legal-spells nerf is
  also optional/off and belongs in the planned Challenge group. Other current
  defaults remain as they are.
- Scrolls use the reader's relevant active caster level. Legitimate non-casters
  use half their highest active class level, rounded down, minimum 1; dormant
  dual classes do not supply it. This is implemented in the candidate.
- BG1 banking ceiling 161,000, SoD 500,000; optional SoA 2,950,000. Existing XP and
  levels are not taken away. Garithos uses 650 only; his non-dual playtest may
  proceed before resolving general dual-class handling.
- Bard normal spell ceilings: Blade/Skald/Dancer 6; most others, including Jester, 7;
  Kapellmeister/Darkbloom 8. These mechanics are already released.
- The newer settled Blade follow-up uses two weapon pips in its listed groups
  and three in Two-Weapon Style. Battleluck is five rounds, caster/allies within 30 feet,
  normal/Gallant versions refresh each other. See the owner document, not older
  conversational suggestions of three weapon pips.
- Heroism/Greater Heroism/Gallant copies/Potion of Heroism: first active buff
  stays, no refreshing/upgrading while active. Song of Heroism, Aid and Potion
  of Power are outside that family.
- Two mutually exclusive potion choices: unique temporary potion effects, or
  the same with Genius exempt. Different potion types may overlap. Repeated
  Heroism/Power keep their first effects; other safe repeats refresh normally.
  Healing, cures and permanent/quest uses stay normal.
- Pathfinding's out-of-combat-only setting is installed privately; public
  inclusion awaits Christopher's opinion. Normal allied blocking in combat and
  no enemy pass-through are required.

## Still needs Christopher, not an agent guess

- Luck/Bless/Aid durations and whether/how Aid changes; druid Chaotic Commands.
  The research proposals of 60s/60s/120s and druid level 5 are not approvals.
- Entropy Shield's exact removability and enemy party Dispelling Screen allocation.
  Spellstrike already counters Entropy in source; do not implement that twice.
- XP-bank display placement and the no-bank between-level dual arithmetic.
- Wider Challenge group membership, potion default variant, Wish enemy-clone scope.
- Fighter-wide proficiency grant, Mystic Fire changes, chatter reduction list,
  and the long-term direction of configurable Spell Revisions.

## New backlog note — Degrodel and the Helm of Balduran

Christopher's October 13 note: "Degrodel just running away and stealing your
helmet when you bring him the baldurans helmet."

Keep this in the follow-up list. Check the dialogue, item transfer and escape
behavior to establish whether this is intended quest behavior or a mod interaction;
it has not been investigated or confirmed as a bug. Explain what happens and
propose a small correction if appropriate. Ask Christopher if the desired change
is unclear rather than inventing a quest rewrite. This note does not authorize
editing his current game/save or make this a blocker for the prepared batch.

Also recorded in the [main backlog](2026-09-20-next-work-index.md#october-13-additions)
and the overview's Modpack "Preserved later work" section.

## Larger unfinished work is not in the quick batch

The potion work is still a six-type prototype, not a completed all-potion component.
Heroism additionally needs normal spell/recipient handling. If continuing either,
own the shared provider once and coordinate Bardic only after its current reserved
work finishes. Do not add another partial pilot and report the whole feature done.

Preserve the Time Stop/Haste history separately. Do not assign potion work back
to its mixed-history chat as if it had no other purpose. The user explicitly
requested a recovery plan before rearranging those histories.

The configurable-SR branch (`codex/configurable-sr-catalog`, audit `3d3b083`) is
substantial preserved work, not a released SR replacement. Randomizer Mode1 has
a new persistent playtest but incomplete ordinary delivery/parity; its existing
owner continues it. Script engine has an active owner and a working bounded
prototype, not a complete SCS replacement. Do not repurpose either owner.

BuffBot 1.10.0 item powers are already released. Pending gameplay checks are not
proof that the feature still needs building. Artisan Archer/Assassin work is
released too. The full overview preserves the remaining issues for these and
adjacent Aura/Radar/dashboard/voice-pack projects.

## Working and completion boundaries

- Finish local source, focused checks and candidate packaging. This handover
  does not authorize a new public release, PR merge, website deployment, game
  launch, live install/save edit, broad cleanup or deleting old worktrees.
- Read the BG compatibility policy before implementation. Fix owned behavior,
  preserve unrelated mod data and normal layout variations. Do not require a
  byte-identical captured installation. Keep real format/capability checks and
  CEBG integrity validation intact.
- Do useful bounded automated checks for changed behavior. One combined install
  and a short player checklist are the target; no new full copies/labs by default.
  Separate source/automated evidence from actual game acceptance.
- Honor each repository's guidance. Verify `gh auth status` before each GitHub
  operation; personal BG repositories use `Chrizhermann`. Never commit private
  logs, game resources, saves, credentials or transcripts.
- Update the overview with real owner, status, exact source and next step. Keep
  shipped, locally implemented, privately installed and player-tested distinct.
  Report meaningful gameplay progress and genuine decisions, not just test counts.

The deliverable is a small set of complete candidates ready for the next combined
package, with a short question list for remaining decisions—not a claim that every
item in the larger backlog is done.
