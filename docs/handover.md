# chriz-bg-collection — Handover

Live entry point for anyone (user, future agent) picking up work on this repo.

## October 13 — alpha.20 and its website published

**App and collection alpha.20 are now public** as a prerelease and on the alpha
updater channel. Modpack 0.2.0-alpha.11, SoD Remix 0.6.13, BG Rebalance 0.8.2 and
Bardic Wonders 2.9c-balance.7 are pinned from freshly verified public downloads.
New: Challenge Mode, the Lefreut interface fix, resumable re-downloads, bard
spell ceilings, Fade/Evandra catch-up XP, reader-level scrolls, Degrodel's choice
and optional experimental XP banking; Emotion exclusivity and EET's desktop
shortcut are now optional. Full local tests, signed-package checks and Defender
scans passed; this binary was not launched. See
[exact results](package-acceptance-alpha20-2026-10-13.md) and
[complete release notes](release-notes-alpha20.md).

All seven versioned assets and the alpha update URL are verified. The collection
website shows alpha.20 (website PR 28, verified live). Public docs and local tools
now write the owner's home folder as `%USERPROFILE%`. Current coordination:
[work overview](plans/2026-10-12-bg-work-overview.md#october-13-continuation-claude).

## October 12 — straightforward work started and Claude handover

Christopher approved starting the clearly settled tasks. Emotion's normal default,
the existing Blade/Battleluck delivery, and SoD relocated-item identification now
have completed local deliveries: Emotion/SoD source fixes are checked, and Bardic
has a normal installable candidate. Nothing from this batch is published or
applied to the game. Read the
[dispatch and current reservations](plans/2026-10-12-bg-work-overview.md#october-12-implementation-dispatch)
before editing those areas. The
[Claude continuation handover](plans/2026-10-12-claude-continuation-handover.md)
gives the independent next steps and outstanding questions. No new gameplay
decisions, live-game changes, release/publication or cleanup are authorized by
this preparation. The audit-only note immediately below describes the earlier pass.

## October 12 — cross-repository work overview

Start with the [current BG work overview](plans/2026-10-12-bg-work-overview.md)
for next-work planning. It reconciles the nine requested mod repositories,
CEBG, relevant worktrees, current GitHub work and adjacent projects. Older
“pending release” paragraphs below are historical; alpha.19 already shipped.

Important corrections: reader-level scroll430 is fully implemented in the local
combined Modpack candidate at1eadb92, alongside recruitment237/238 and banking;
it is not still only a partial hook. Bard ceilings are released in their owning
mods but unselected in the public recipe. New Blade/Battleluck and SoD recruitment/
finale work are local, unreleased candidates. Full potion uniqueness remains
unfinished. The Randomizer owner now has a persistent Mode1 playtest, separate
from normal public packaging. A substantial configurable-SR branch and older
companion-stat work must be preserved during cleanup.

This was an audit/planning request: only coordination documentation changed.
No games, source implementation, owner assignments or GitHub state changed.
Suggested parallel batches and Christopher's remaining decisions are in the overview.

## October 12 — Garithos XP banking and bard ceilings installed

Christopher approved installing now and testing in his current run; the open
dual-class rule is deferred, not a blocker for this run or cancelled work.
Appended Modpack650 (BG1/SoD banking; no651), Rebalance420/421 and Bardic3010 to
`%USERPROFILE%\Games\Chriz Easy BG\game`. All installs exited0; the 445 existing
component identities/order and previous330,163 TLK strings are preserved.
Latest save `000000990-NEXT IRON THRONE LOSERS` is backed up and hash-unchanged.
Existing XP/levels are retained by adoption; bard capacity updates through the
next normal level-up, not a load-time spellbook rewrite. Eldoth's Jester caps at7.
Sources are local development additions, not a public release/default change.
Game not launched. Startup, XP award/save/reload/transition and bard level-up
checks remain open, alongside deferred dual rules and bank-removal recovery.
Full record, backup and continuing checklist:
`%USERPROFILE%\Games\Chriz Easy BG\repair-backups\oct12-xp-bard-ceilings\README.md`.

## October 12 — BG2 / CEBG cloud saves disabled

At Christopher's request, Steam Cloud for BG2:EE (app 257350) is now disabled.
Verified disabled again in a fresh Steam API session. The current Garithos
profile's `Baldur.lua` also explicitly sets `Cloud Saves Enabled` to `0` so its
legacy setting cannot override Steam's opt-out. No saves deleted or edited,
no game launched, and other Steam games / OneDrive settings are unchanged.
Configuration backup and application record:
`%USERPROFILE%\Games\Chriz Easy BG\repair-backups\oct12-disable-cloud`.

## October 12 — temple donation backlog and pathfinding installed

Added the temple donation helper to the backlog: show the exact cost of the next
reputation point and make that donation convenient, without changing prices.
Pathfinding Redux 0.1.10-preview now supplies the requested out-of-combat-only
mode. Christopher subsequently requested installation and confirmed the game closed.
Installed into `%USERPROFILE%\Games\Chriz Easy BG\game`: WeiDU exit 0, all 444
prior component rows retained in order, one new component appended. Game/EEex/loader
executables, dialog.tlk and the newest original save are hash-unchanged.
Backup: `repair-backups\oct12-pathfinding-ooc` beside the game. Prepared package
and application record: `C:\tmp\cebg-pathfinding-ooc-20261012`.
Component 0, `OutOfCombatOnly=1`, `EnemyPrototype=0`; no public recipe/default change.
The exact current EEex 1.2 / BG2EE 2.7.3 pairing still needs startup/play confirmation.
See the [October 12 scope](plans/2026-10-09-next-patch-batch.md#october-12-additions)
and private preparation README. Game was not launched; startup/playtest is pending.

## October 11 — spell and bard design/research started

Christopher requested design/research and open questions for the new SR/Bardic/
Artisan balance list. Existing relevant owner chats received bounded briefs;
Blade belongs to Bardic, with Artisan checking interactions only. See the
[full scope and owner IDs](plans/2026-10-09-next-patch-batch.md#october-11--spell-and-bard-design-assignments).
No new implementation, game/save changes, labs, publication or installer-default
changes are authorized by this request. Entropy Shield/party Dispelling Screen
AI research is reopened; unrelated Time Stop/potion recovery is not reassigned.

## October 9 — next patch batch planning

Latest scope correction: Christopher wants practical hobby-mod delivery for the
default setup, with a short test and documented limitations—not exhaustive
engine/compatibility projects. Both owners received this correction. See the
[current practical scope](plans/2026-10-09-next-patch-batch.md#practical-hobby-mod-scope--latest-october-9-direction)
before interpreting the older detailed acceptance lists as release prerequisites.
He subsequently clarified that he will test himself later: no new game copies
or agent-run game tests now. Smaller tests do not mean a smaller requested
feature; the six-potion trial is not completion of the potion component.

Owner checkpoint: Fade/Evandra recruitment XP is implemented locally with
automated checks, awaiting integration/native acceptance. XP banking is active;
scroll hook work has a continuation brief, and the potion owner has resumed its
prototype after finding defects in the first draft. See the batch plan's
[delivery checkpoint](plans/2026-10-09-next-patch-batch.md#october-9-owner-checkpoint-and-continuation).
[Pathfinding Redux](plans/2026-10-09-next-patch-batch.md#pathfinding-redux--on-hold-because-of-allied-pass-through)
is on hold: Christopher rejects unavoidable ally pass-through, and the release's
other improvements also stop when that feature is disabled. Nothing installed.

XP-cap banking is now promoted from deferred work: Christopher requested design
and assignment for an October 10 review candidate. The
[implementation brief](plans/2026-10-09-campaign-xp-banking.md) defines optional
BG1/SoD caps, optional SoA cap, per-character saved banking, data-preservation
checks and the required XP-hook proof. Nothing is installed or released by this
assignment; owner delivery and integration remain pending.

Christopher approved planning the broader Challenge Mode grouping, Fade/Evandra
recruitment XP, reader-level scroll casting and potion-effect non-stacking before
owner execution. Existing normal defaults stay, except the already agreed
Emotion/Baeloth policy. See the [current batch plan](plans/2026-10-09-next-patch-batch.md)
for source findings, outstanding choices, ownership and acceptance boundaries.
The settled Modpack XP/scroll work and two Rebalance potion variants have now
been assigned to their existing owner chats for local implementation and automated
checks. No current-game changes, game launches, recipe changes or new release
authority. Challenge Mode grouping is still planned, not yet implemented.
The completed Garithos reminder has now been deleted, not merely paused.

## October 9 — Garithos current-game repairs applied after stream

Christopher approved correcting the three read-only audit findings after his
stream and identified a new latest save. Game and InfinityLoader were closed
throughout. The selected source was `000000984-HUMANS ARE SE BEST`, saved
October9 at03:27:38KST in the `Chriz Easy BG - ec2c39f8d9c3` profile.

- Added private tail `CBR_GARITHOS_SPELLS:0` to
  `%USERPROFILE%\Games\Chriz Easy BG\game`: normal/Wish Hardiness now40%
  physical-only, and Spellstrike now15% arcane/divine failure for2rounds,
  refreshing rather than stacking. Uses unchanged public Rebalance0.8.1
  mechanics. Only four SPLs changed; Spell Shield and Pierce Magic unchanged.
  WeiDU exit0; outputs match tested copies, all442 prior installed rows and
  all330158 prior TLK records/sound metadata retained; two descriptions appended.
- Published a separate **`000000985-HUMANS ARE SE BEST - fixed`** save. Original984
  and all older saves are untouched. Edwin's nine inherited permanent+2 slot
  grants become+1, giving base+2 including specialization (currently6/4/3).
  One duplicate prepared spell per accessible level removed; all distinct
  selections, known spells, base slots, other effects and all50 other saved
  actors preserved. The other save files are identical; old Cloud identity
  metadata was not copied to the new local slot. No cloud operations performed.

Backup: `%USERPROFILE%\Games\Chriz Easy BG\repair-backups\oct09-garithos`.
Private scripts/results: `target/work/oct09-garithos-repair`. Five copied-resource
spell cases plus23 canonical transformation checks and8 independent save checks
passed, including pointer/section validation, byte-preservation and idempotency.
No game launch or native playtest. Edwin dismissal/rejoin/respec or explicit
level1 CLAB replay was not validated; do not additionally apply `C0REDW4` to this
repaired actor. The private spell transaction is tied to reviewed capture hashes;
canonical generic-layout limitations were not silently relaxed. No release,
recipe or public component changes. The requested4:30reminder is paused.

## October 7 — alpha.19 published; website and cleanup complete

**App and collection alpha.19 are now public.** This supersedes older
pending-owner/Radar notes. Modpack alpha.10, Rebalance0.8.1
and BuffBot1.10.0 are pinned from freshly verified public downloads; the approved
new defaults and optional Baeloth choice are included. Full local tests, final
signed-package checks and Defender scans passed. Radar-off and user-approved
Radar-on launch passed on the preceding app candidate; no stream saves were
edited. The final rebuilt binary has separate scan/package evidence, not a new
gameplay claim. See [exact results](package-acceptance-alpha19-2026-10-07.md) and
[complete release notes](release-notes-alpha19.md).

All seven versioned assets and the ordinary alpha update URL are verified.
The website is live with the complete changelog; PR27 and master CI passed,
and its real download matches the verified package. Existing rules are unchanged.
The rebuildable compiler cache is removed (45.27 GiB logical file total), with
all games/saves/backups/evidence retained. Older handover/planning work remains
preserved; only this new top entry is included in the release commit.

## October 7 — authorized Defender reproduction passed, cleanup completed

Christopher authorized autonomous app/game testing and cleanup, superseding the
older user-clicks-only instruction for this test. The exact October 1 Radar Fix
setup/app passed current-definition Defender scans, installed successfully, and
launched **Test19** through the real Play button. Opening installation details
and selecting/copying its path did not reproduce the shutdown. No new Defender
1116/1117 events through 06:04 KST; protections remained enabled. Game, loader and
CEBG exited; the temporary installed app was uninstalled successfully. Original
stream game/saves, candidate packages and backups were not edited or deleted.

Read [the bounded test result](defender-smoke-2026-10-07.md) before claiming full
acceptance: elevated pre-existing Radar could not be closed; new Radar UAC
approve/decline paths were not tested; exclusion inspection was access-denied.
Steam Cloud automatically imported 30 saves into Test19 despite its separate
local profile. No save was loaded or manually saved/deleted; the imported copies
remain intentionally, since cloud deletion semantics were not tested.

The October 1 app alpha.19 / recipe alpha.18 candidate still lacks newer owner
work. Public integration is still required for ready resistance310, Hardiness
200/201/202, anti-magic210/211, clouds320, public bracers wiring and BuffBot1.10.0.
Baeloth235 now exists in Modpack's `codex/baeloth-known-spells` (3d3381f); the
older statement below that implementation is absent is stale. It remains an
optional planned-Challenge choice, not an unconditional default. Edwin/EET
arrows also need owner/public follow-through. Do not publish the old candidate
claiming it includes private stream repairs. No release or channel update today.

## October 6 — Pierce Magic and damageable cloud forms applied privately

Christopher requested the newly approved Pierce Magic balance in his stream
game and the same concise website treatment as the physical-resistance rule.
Owner source: BG Rebalance `codex/sr-antimagic-research`, commit `48938d1`,
unpublished `v0.8.1-dev`, component **210**. It halves current Magic Resistance,
rounding the reduction up and bounding it to 10–40 percentage points (zero
floor), for five rounds. Recasts refresh the original amount; Lower Resistance
can combine with it. Protection removal and Spell Shield interception remain.

Applied through new private tail `CBR_SR_ANTIMAGIC:210` to
`%USERPROFILE%\Games\Chriz Easy BG\game`, with the game/loader closed. WeiDU
exit 0; all 440 previous log rows retained exactly. Only `SPWI608.SPL`,
`SPWI608B.SPL`, and `M_CBRPM.lua` changed in override, matching the copied-game
fixture byte-for-byte. TLK gained one private description at 330155; all 330,155
old entries, including sound metadata, were preserved. Saves untouched; no game
launch. Backup/evidence:
`%USERPROFILE%\Games\Chriz Easy BG\repair-backups\oct06-pierce-magic`.
Independent review and 37 runtime-model/WeiDU checks passed; native gameplay
acceptance remains. Old already-active Pierce Magic effects in saves must expire.
**Spellstrike 211 was not applied** by this request; neither were Hardiness,
Pierce Shield, Lower Resistance or Entropy Shield. Owner 211 is separately
implemented/approved, not evidence of being installed here.

Website owner published and verified the short numbered rule:
“Pierce Magic: halves Magic Resistance for five rounds (10–40 percentage points;
no stacking).” Existing rules and collapsed Small notes retained; 114 tests pass.
No owner mod release or CEBG recipe/preset change occurred. Next-patch integration
must use an eventual immutable owner release, not copy this private adapter into
the collection; see the owner's `docs/handovers/2026-10-06-sr-antimagic-collection.md`.

Christopher chose **75%** for damageable combat clouds. BG Rebalance component
**320**, label `cbr_damageable_combat_clouds`, is now implemented in the
physical-resistance worktree and applied privately as `CBR_CLOUD_FORMS:0`.
SR/SCS genies, known EE/SoD combat clouds, Ascension's combat Bodhi form and SCS
tactical vampire mist are covered. Existing 100% damage-resistance setters become
75%; lower values/open channels, poison immunity, regeneration and duration stay.
Cloud-added MR100 is neutralized; the isolated vampire combat donor has MR0.
These are flat cloud-form values, not a max-with-natural-resistance floor.
Normal-form creatures/items, quest protections and scripted death escapes stay
untouched; no animation-wide sweep or save editing.

27 new real-WeiDU tests + 23 related regressions and independent review passed.
Applied with game/loader closed, exit 0; all 441 preceding WeiDU rows and 330,156
old TLK entries/sound metadata retained. Nine outputs match the captured fixture
exactly; two new SR summon descriptions appended. Backup/evidence:
`%USERPROFILE%\Games\Chriz Easy BG\repair-backups\oct06-cloud-forms`.
Owner research: `research/2026-10-06-damageable-cloud-forms.md`. No combat test,
owner release, recipe/preset change or cloud website publication yet. Next-patch
integration must collect the owner's eventual release (including coordination
with the separate SR anti-magic branch), not copy the private adapter into CEBG.

## October 6 — party physical-resistance component prepared separately

Christopher approved **half effectiveness above 80%**, with a normal maximum
of 90% and a requested optional 95% variant. Owner: **chriz-bg-rebalance**,
branch `codex/physical-resistance-softcap`, checkout
`C:\src\private\chriz-bg-rebalance\.worktrees\bard-progression-release-20260920`.
Prepared source is committed locally as `7777870`; not pushed or released.
Unreleased component **310** implements the 90% variant through EEex:
90 becomes 85, 100 becomes 90; party roster only, independently for the four
physical damage types. No enemies, summons, elemental/magic resistance,
Stoneskin or weapon-immunity changes.

**311 is reserved, not implemented or offered.** Read-only engine inspection
proved physical resistance is truncated at 100 both during opcode handling
and final stat limiting, before the normal EEex callback. Thus merely setting
95 as the Lua ceiling cannot deliver the approved 110 -> 95 behavior.
Christopher was asked whether to defer this alternative or tackle the deeper
engine work; decision pending. Do not silently change the approved curve.
28 focused Lua/WeiDU fixture checks pass. Full owner suite: 423 tests, 422 pass
and one existing optional dragon-capture test skipped, exit 0. Independent code
review found no blocker for 310; native in-game acceptance remains.
See the owner research note `research/2026-10-06-physical-resistance-softcap.md`.

No CEBG pin, preset or release changed. After owner review/release, wire 310
into the normal selection; do not claim it is already in the default installer.
Hardiness 40%/30% is separate parallel work in the owner's `sr-hardiness`
worktree. The latest owner decision is 40% physical only by default, a 30%
alternative, and separately optional 20% elemental/magic-damage resistance.
Do not apply that work as part of the two repairs below.

Later October 6: Christopher explicitly confirmed the game closed and approved
the **90% cap plus Bracers of Blinding Strike SR-haste fix** for the stream game.
The cap is now installed through a new private `CBR_PHYSICAL_CAP:0` tail entry,
using unchanged component-310 mechanics and allocated state 241. WeiDU exit 0;
all 437 earlier log rows and dialog.tlk preserved exactly; no save edits or game
launch. The private adapter matches the public component in a disposable fixture;
29 focused tests pass. Backup and verification:
`%USERPROFILE%\Games\Chriz Easy BG\repair-backups\oct06-resistance-bracers\cap`.
The bracers repair is also now applied as `CBR_SR_BRACERS:0`, after the BuffBot
update below. Its new self-only SR helper grants +1 APR for 20 seconds, preserving
the one daily charge and installed Tempus links. Fourteen existing haste spells
receive only reciprocal exclusion guards; Wish's actual haste effect is unchanged.
All 439 earlier log rows and TLK remain exact, and the only 16 changed resources
match the tested outputs byte-for-byte. Full copied-resource sweep: 6,747 spells;
9 focused WeiDU tests pass, including repeat/rollback. Independent review found
no blocker for this captured installation. Backup/verification:
`%USERPROFILE%\Games\Chriz Easy BG\repair-backups\oct06-resistance-bracers\bracers`.
No game launched or saves edited. Previously active item effects in a save are
not rewritten; test the bracers' next activation. Runtime acceptance remains.
Canonical library/private adapter and research note are in the same BG Rebalance
physical-resistance worktree; public component/recipe wiring is still next-patch
work, not delivered CEBG updater support.
Source, private adapters and evidence notes are committed locally as `9078d11`
on `codex/physical-resistance-softcap`; nothing pushed or released. The combined
focused suite now passes all 38 tests.

SR research was explicitly dispatched to **Review Hardiness changes**
(`01a10da6-02db-7842-94a5-23a4fff5618d`): compare vanilla and installed Pierce
Magic/Shield, Lower Resistance and Spellstrike, plus a separate Entropy Shield
review of existing nerfs (including Tactics Remix). No nerf values approved.
Installed-file evidence identifies **Pierce Magic**, not Pierce Shield, as the
two-round MR-zero spell. Neither is authorized for live changes yet.

Christopher additionally requested the website rule and newest BuffBot in his
current game. Website publication was dispatched to **Build interactive BG run
page** (`01a061fe-a68c-74b1-8272-5bfa9251a731`), using one short numbered rule:
“Party physical resistance: gains above 80% are halved, up to 90%.”
The website owner completed publication and verified it live; all 114 website
tests passed and the existing rules/collapsed notes were preserved.

**BuffBot v1.10.0 is now applied** to the stream game as a private
`CBR_BUFFBOT_110:0` tail entry. The normal Latest release ZIP was verified against
SHA256 `cea6628cc6f00eeb3d296fe50f481f5c13ad4ea3530bc67716493d0a5b016f77`.
Twelve runtime/menu files and the English catalog changed; BfBot5e.lua is new.
Innate/Mp modules and all packaged graphic assets were already byte-identical.
The catalog retains all 177 old keys unchanged and adds four. Existing eight
F12 string references, TLK, loader, original mod sources, presets and saves stay
untouched. All 438 preceding log rows remain exact. The installed payload matches
the disposable-fixture output and released files, exit 0. Backups/verification:
`%USERPROFILE%\Games\Chriz Easy BG\repair-backups\oct06-resistance-bracers\buffbot`.
This is a private runtime update, not a CEBG pin/update-receipt migration. The
original BuffBot log rows truthfully retain 1.8.4; the new tail records 1.10.0.
No in-game test was run. Release owner reports 652 automated tests passed and
explicitly deferred live item-action acceptance; do not turn that into a claim
of runtime verification on the stream game's 2.7.3 executable.

## October 4 — Microsoft recheck and stream rules audit

Both Microsoft cases were opened live in the authenticated browser today:
original `907e4c7d-555b-4016-b5b0-2cee51e79467` and diagnostic follow-up
`af423c35-59b1-4625-920d-b056e5161435`. Both still display **In progress / Pending**.
Both show the same analyst response: no positive detection/telemetry, closure
without further action, and a request for diagnostic CAB data. The follow-up
already contains that CAB. Do not describe this as a fresh clearance or as an
assurance that investigation is actively progressing. The original submitted
executable is still the different Selection Fix build. No new submission,
email reply, scan, quarantine restore, or Defender setting change was made.
Next useful follow-up is clarification that the requested diagnostics are
already in the second case; obtain approval of the exact email before sending.

Rules-page source is prepared on website branch `codex/stream-rules-oct04`,
checkout `%USERPROFILE%\.codex\worktrees\bg-rules-oct04\twitch-setup-chriz`.
Christopher rejected the initial lengthy mod-guide approach during preview.
The three short additions now belong directly in the numbered rules list (17–19):
“Emotion: Courage and Hope do not stack.”, “Flail of the Ages: Slow now allows a
save vs. Breath.”, and “Elemental arrows: Like in BG2.” Christopher explicitly
rejected moving these into a separate notes section and deleting the existing
small notes. Those smaller audit notes are restored underneath in a “Small notes”
disclosure, with shortened copy. Utility XP and the rejected Tempus entry remain
omitted. Do not turn ordinary mod features into numbered run rules. The older
overbroad prebuff claim has also been corrected.
**Baeloth's legal-spell nerf is not installed and is omitted from active rules.**
Follow-up located the owner design on Modpack `codex/companion-spellbooks`,
`docs/companion-spellbooks.md`: SR/non-SR picks are prepared, but the document
explicitly says no component exists; no implementation/tests were found in the
registered Modpack worktrees. Christopher says the prior personal nerf was in
EE Keeper and now wants the installer feature in planned Challenge Mode. This
supersedes that older design's collection-default proposal. See the updated
challenge-mode outlook before resuming the existing owner work.
**XP banking is not implemented; CDTweaks 2090 still removes the XP cap.**
Story utility XP is separate from caps/banking. No game or save files changed.
All 114 website tests and the production build pass after the placement correction.
The refreshed browser shows 19 numbered rules and the restored, working Small
notes disclosure underneath.
Latest October 4 copy review: notes remain collapsed on fresh page load. Artisan
and Bardic are described as broad toned-down balance versions, not cherry-picked
ability examples. Companion conversions and save-migration caveats are removed.
Dragons retain a death-save warning; SR notes cover Improved Haste/celestials and
prominent Malison/Horrid Wilting/Spell Immunity changes; Telekinetic Storm stays
identified separately and the restored classic Lightning Bolt is attributed to
our SR adjustment. Prebuffing is described simply as improved reliability.
The user's scroll-level concern revealed a genuine policy/default mismatch:
CDTweaks 2170 is overridden by Modpack 430's CL15-for-all-readers workaround.
It is not just a wording error. See the October 4 next-work index entry; no game
or recipe edits made. The scroll claim is omitted from website copy.
Baeloth implementation was explicitly requested from the existing Modpack owner
chat (`01a0b92c-da6d-7701-8d51-b0df1a17fe08`); code delivery remains pending.
A full source-grounded current/upcoming SoD Remix summary was requested from
`Rework Belifid’s end fight` (`01a0e4cd-cca5-76e0-bb08-9ebbbe82ad30`) for a future
dedicated subsection. The approved rules wording is “SoD Remix is included.”; don't
present unreleased owner designs as active stream behavior.
Christopher approved publication on October 4. Website PR #21 is merged as
`2be99b820762191c2a3e7d30d59176f0343a8900`; GitHub's site checks and Cloudflare
Workers Builds deployment both succeeded. The public content at
`https://bg.chrizfader.org/rules` now has all 19 rules and the exact approved
“SoD Remix is included.” note. The disclosure remains closed by default.
This publication changes only the website, not the installer release or game.
Microsoft follow-up route check: the case page still has no case-reply form;
the analyst directs replies to their email. Correction after Christopher's
challenge: personal Gmail IS connected through the separate Gmail plugin;
get_profile verified christopher.hermann01@gmail.com. The previous check only
listed the work account in Google Workspace Multi and incorrectly generalized
that result. Gmail searches (including Spam/Trash) for both case IDs, analyst
wording and recent Microsoft senders have not located an analyst email; they
did find the September 30 WDSI account-connection notice. Do not confuse a failed
message lookup with a missing connection. The diagnostic case was rechecked:
still In progress/Pending with the same request for a CAB already attached there.
No mail sent. Obtain the actual reply address before preparing a follow-up, and
get approval of the exact draft before sending.

## October 3 — approved stream-install repairs applied

Christopher confirmed the game was closed and explicitly approved applying the
prepared fixes to `%USERPROFILE%\Games\Chriz Easy BG\game`, including Flail of
the Ages, story-stage utility XP, and Evandra's permanent conversion/portrait.
This is a private current-install repair, **not a new CEBG release or general
updater support**. The original collection receipt/version remains unchanged.

Installed as seven new tail components, without uninstalling existing mods:

- Rebalance v0.7.1's 301/302 mechanics in a private source/backup namespace:
  existing IWDification Emotion Courage/Hope refresh/replace each other; Flail
  Slow has Breath saves -2/-4/-6 by tier, and +5 retains Free Action but allows Haste.
- Modpack alpha.8's 224/225: permanent Evandra Sorcerer starting templates and
  the owner's portrait. Her personal portrait was already byte-identical; no
  saved actor or level/XP was rewritten.
- Red Wizard Edwin: both amulets movable and grant no slots; remove inherited
  source-less +2 CRE grants, add the prior C0REDW4 kit +1 grant (native specialist
  +1 plus kit +1 = +2 total). All 11 templates covered, including EDWIN7_, which
  had missed the selected Artisan kit conversion. No further kit rebalance.
- SCS-style elemental arrows, adapted for EET: AROW04 acid 1d3, AROW09 cold 1d2,
  AROW08/AROWKC fire 1d2 and no +2 attack/physical bonus. Original saves and SCS
  troll helpers retained; other special arrows unchanged. SCS 35.21 component
  3017 itself rejects EET, whose imported resources retain BG1-strength arrows.
- Story XP replaces the progressive runtime: 610's startup becomes an inert
  backed-up stub, and the four unchanged published 611 runtime files load instead.
  610's original log row/config/modules remain; no fake native611 row. BG1-sized
  rewards through SoD, native BG2 rewards from SoA; no retroactive XP edits.

All five WeiDU invocations exited 0. Before live application, the combined
sequence passed on a private copy of this install's resources; the final changed
live resources and TLK match those tested outputs exactly. All **330,147 old TLK
strings and metadata are preserved**, with eight appended strings. All 430 old
WeiDU component identities/order remain, with seven new rows. Original UI.MENU
(including Fade's race joke), XPBONUS, loader configuration and engine.lua are
unchanged. WeiDU's en_US -> en_us config casing normalization was identified and
the original exact weidu.conf restored. No game launch or in-engine acceptance.

Backup and durable verification:
`%USERPROFILE%\Games\Chriz Easy BG\repair-backups\oct03-current-game-fixes`.
Preparation, adapters, fixture reports and logs:
`target/work/oct03-current-game-fixes` (ignored/private; never publish captured
game resources). Cleanup of the resource-only `content-fixture` copy was blocked
by tool policy before deletion; it remains (roughly 6.3 GB). Do not route around
that refusal. Retain the repair backup, original mod backups, adapter sources
and reports independently of any later authorized test-copy cleanup.
Do not re-run the apply command: these components are already installed.

**Still separate:** existing-save Edwin permanent slot grants require inspecting
the specific saved actor; template/CLAB changes do not retrofit that state.
The now-movable amulet is a resource change, but already-equipped cached effects
may need unequip/re-equip. No saves were inspected/edited. XP-cap banking remains
undeveloped/off by default; Fade/Evandra XP catch-up is not solved by the class
conversion. Carry the uncovered Edwin/EET omissions into the owning mod's future
release rather than assuming this private patch updates public defaults.

## October 1 — final candidate packaged; publication still held

The Radar Fix candidate completed automated checks and packaging at 04:09 KST
(September 30, 19:09 UTC), exit 0. This supersedes the earlier statements below
that Radar is source-only or final packaging is pending. App version remains
**0.1.0-alpha.19**, bundled collection **0.1.0-alpha.18**.

Local setup:
`%USERPROFILE%\Downloads\Chriz Easy BG alpha19 - Radar Fix 2026-10-01-setup.exe`.
Its adjacent `.sig` is verified against the app's updater public key. This is
not Windows publisher signing or a Defender clearance. See the exact hashes,
test counts and limits in [candidate verification](package-acceptance-alpha19-radar-2026-10-01.md).
The local update feed is prepared but not published. No setup, app or game was
launched by this job. No new full game installation is required.

Remaining user check: when agreed and after assessing the Defender issue, test
Play with Radar unchecked, then opted in; verify Windows approval, actual overlay
appearance, and a clear nonfatal message when approval is declined. The agent
must not click or launch this check on Christopher's behalf.

Microsoft's latest analyst reply reported no detection/telemetry for the initially
submitted Selection Fix sample and requested the affected machine's support CAB.
This does **not** clear the different Dragon and Order Fix executable identified
in the incident logs. Christopher approved sending the existing diagnostic CAB
and the correction. The follow-up form is prepared with the original submission
ID, corrected affected hash and diagnostic context. Christopher completed the
CAPTCHA and Submit; the diagnostic CAB receipt is verified, submitted at
October 1 14:44:16 as displayed by the portal. Rechecked live on **October 3**:
status **In progress**, final determination **Pending**. The generic analyst reply
reports no positive detection/telemetry and requests a support CAB, despite this
submission already containing that CAB. It says no further action will be taken
without the requested diagnostics; do not interpret the status badge as confirmed
active investigation or clearance. The automated cloud scan labels the CAB
`Trojan:Script/Wacatac.C!ml`; the client scan says **No malware detected**.
These are not a completed analyst verdict on CEBG or evidence identifying the
original quarantine trigger. A clarification that this case already contains the
requested diagnostics is the useful next step; no clarification was sent during
the October 3 read-only check.
The private submission ID and receipt details are recorded under ignored
`target/work/oct01-defender-submission`. No further user submission action is
needed. Do not upload a duplicate or claim the app has been cleared.
No new reproduction, quarantine restore or antivirus setting change was made.
Public downloads, updater channel, website and Discord announcement remain held
until the incident is assessed and Christopher chooses how to proceed.

Cleanup inventory found about 45.3 GiB in `target/debug/incremental`. Its exact,
validated removal was rejected by tool policy **before execution**; zero files
were deleted and no alternate route was attempted. This was not a reported game
file lock. Keep games, saves, backups, release outputs and `target/work` evidence.
The cleanup evidence is local under `target/work/oct01-cache-cleanup`.

## September 30 — quick release preparation during streaming

Christopher requested only quick preparation now; builds, installs and supervised
game/Radar checks wait until after his stream. No automatic evening wake-up or
publication is authorized by that timing discussion. The release notes now contain
short Discord and website draft copy, clearly held for final packaging and the
Defender decision. The smoke checklist records the completed fresh installation
and removes the obsolete September 29 setup instruction; no additional full
installation is required. No announcement, website or download was published.

Tonight: prepare and verify the package containing the Radar fix, agree the short
remaining checks, finalize the release copy and download links, then make the
publication decision based on the unresolved Defender findings below. The pending
Microsoft correction is already approved; do not ask again for the same text scope.

## September 30 — Defender quarantined CEBG; completed game and Radar remain

New release-readiness issue: at 17:17:38–17:18:18 local, Microsoft Defender
detected `Behavior:Win32/DefenseEvasion.A!ml` and quarantined the installed
`%LOCALAPPDATA%\Chriz Easy BG\chriz-bg-app.exe`. The executable and Start-menu
shortcut are absent; the action also names its uninstall registration. Defender
events 1116/1117 and Get-MpThreatDetection confirm successful remediation. Cloud
intelligence events 2010 immediately precede detection, but neither establishes
the exact behavioral trigger. Entries naming Elgato WaveLink are not proof that
WaveLink caused the detection. Antivirus, real-time and behavior protection stay
enabled. No exclusions, restore, reinstall or app/game launch have been performed
by this investigation. The separately authorized Microsoft submission is recorded below.

The first investigation checked the preserved Selection Fix executable and setup
against their verified hashes. **The later support-log collection below identifies
the affected installed app as the earlier Dragon and Order Fix build instead.**
The Selection Fix sample submitted to Microsoft is not byte-identical to that
affected executable; the case needs a correction. Both preserved builds have Windows
Authenticode status `NotSigned`; the existing Tauri updater signature is a
different signature and is not a Windows publisher certificate. Do not describe
this issue as resolved by previous package tests or propose arbitrary binary
changes to evade detection. Next steps: await Microsoft's determination and review
the behavior in a controlled environment with separate live-test authorization.
Evaluate Windows publisher-signing separately; it is no guarantee.
[Microsoft developer guidance](https://learn.microsoft.com/en-us/defender-xdr/developer-faq).

The user reused `%USERPROFILE%\Games\Chriz Easy BG Test 19` for a **new successful
installation**, ID `install-6a5411d310041f2ae2c1`, completion at 08:56 local.
Do not confuse it with the earlier failed attempt. Its game/loader/WeiDU.log
remain present. Radar **2.5.2.0** is also present at
`game\BG Radar Overlay\BG Radar Overlay.exe`; receipt in
`.chriz/addons/bg-radar-overlay.json` records automatic installation at 08:56:35.
There is no Customize checkbox for Radar. CEBG's Play action **attempts** to launch
both game and overlay. Christopher explicitly confirms he used Play and Radar did
not appear; do not attribute his failure to launching InfinityLoader directly.

Follow-up read-only diagnosis: all 15 Radar receipt-owned files match their size
and SHA-256, and receipt/root validation currently supports `UpToDate`. Radar's
embedded XML manifest requires `requireAdministrator` (actual XML nodes, not
commented examples). In the affected packaged build, `SystemBridgeSystem::launch` uses `Command::spawn`, which
does not request Windows elevation consent; `launch_install` silently discards
Radar status and launch errors (`bridge.rs:1361-1367`). This is a concrete launch
design defect and explains rejection when CEBG is unelevated; the exact historic
Windows error was not recorded and no live reproduction was attempted. Fix
direction: Windows-supported consent-aware launching for Radar only, plus clear
nonfatal feedback while preserving successful game launch. Do not elevate the
whole installer or alter Radar's binary/manifest. The old native tests mocked
successful launches without a Radar receipt; the new tests cover this branch.
[Windows UAC launch guidance](https://learn.microsoft.com/en-us/windows/win32/dxtecharts/user-account-control-for-game-developers).

Defender sequence clarified: Play started the game; later, while viewing install
details and copying the folder path, CEBG disappeared and Defender notified him.
Details/path text has no native toggle/copy handler. The Play call chain checks
filesystem records, starts InfinityLoader and attempts Radar; it does not reach
the broad process scanner or the PowerShell patch-mutation checks. There is no
post-Play frontend polling. Therefore do not claim those unrelated scanner paths
or copying text caused this detection. Game/loader launch and possible delayed
Defender attribution remain investigation candidates, not established causes.

Initial read-only checks: Defender's process-start FILETIME places this CEBG
process at 06:43:40 local, over ten hours before the detection, not a newly
started installer. Available event XML exposes no offending API or parent/child
chain; Security process auditing and Sysmon supply no additional evidence in
that window. Defender Support logs were access-denied under the ordinary token;
that initial pass did not elevate or change ACLs. The subsequently authorized
elevated collection is recorded below. The three cloud-entry identifiers
do not match the preserved app's SHA-1; do not present them as its file hash.

The cached EEex 1.2.0 archive matches the recipe's exact length and SHA-256. All
12 EXE/DLL payloads under installed `EEex/loader`, and all nine active loader
binaries (2.7.3.0/LuaJIT), match their respective archive entries. This verifies
provenance, not antivirus safety. EEex's author documents runtime game-memory
patching and antivirus alerts; that makes loader behavior a reasonable lead,
not proof of why Defender quarantined **CEBG** here.
[EEex documentation](https://github.com/Bubb13/EEex/blob/master/README.md#how-eeex-works).

Christopher approved submission of the preserved CEBG executable and a short
report to Microsoft, excluding saves and personal logs. After Christopher completed
sign-in, account consent, the CAPTCHA and final submission, the receipt was verified:
submission for `chriz-bg-app.exe`, September 30
at 17:54:38 as displayed by the portal. Status is **Submitted**, final determination
**Pending**, with no analyst comments yet. Submission is not a clean verdict or
blanket approval of future versions. Treat the antivirus incident as unresolved
for public release even when the independent Radar tests pass. Do not restore
quarantine or weaken protection to continue it.

Radar correction is now implemented in source: remembered per-install **Start
Radar with the game** checkbox, off by default; native Windows consent for Radar
only; visible nonfatal cancellation/failure result after successful game launch.
Automatic Radar installation is unchanged. The frontend's complete 396-test
suite, typecheck and production build pass. Native changes add eight mocked
launch-contract tests and two Windows result/path unit tests. The coordinated
run under `target/work/sep30-radar-consent-tests`, background job
`0346a784-997d-46a5-ab6f-477f63494ff1`, completed with exit 0 at 17:44 local;
status and both logs were verified. All 45 library tests and all 50 native
command-contract tests passed, none failed or ignored. No new package or public
release has been produced.
Do not launch the game/app or a real UAC prompt without renewed live-test
agreement. OS launch success is not proof that Radar attached to the game.

The prepared, sanitized Microsoft report is local-only at
`target/work/sep30-defender-report/submission-report.txt`. The approved sample is
`target/work/sep30-selection-focus-package/extracted/chriz-bg-app.exe`, not the
whole game or any personal log. The uploaded sample SHA-256 was rechecked as
`3e50d7d3c6c155a5e505e48d19df832ada61b69538ca815693d2632e33581243`.
The private receipt and submission ID remain in the local report records;
they are not public troubleshooting links. No automatic follow-up monitor has
been scheduled.

### Authorized local support collection — new evidence, 18:44 local

Microsoft's signed `MpCmdRun -GetFiles -SupportLogLocation` completed with exit 0
after administrator consent. The 28,450,944-byte support CAB is local-only under
ignored `target/work/sep30-defender-local-diagnostics`; status and findings are
saved there. No diagnostic bundle was uploaded; no app/game was launched,
quarantine restored, ACL changed or Defender protection weakened.

MPLog now records InfinityLoader behavior telemetry at 08:17:34 UTC, explicitly
naming the affected CEBG process among its parents. A behavior cloud report follows at
08:17:36, new signatures at 08:17:37, then a resource scan of CEBG together with
`InfinityLoaderCommon.dll` and `InfinityLoaderUtil.dll` at 08:17:37–38. Its sole
detection is CEBG. This strongly narrows investigation to the launch/loader chain,
but does not reveal the precise triggering operation or prove a false positive.
Do not attribute this to UnRAR without evidence.

The same log explicitly records the affected executable's SHA-256 as
`0b0ea4e0217d3c202485a3fe738adb6c470a7e6f1470bd4de7518cb369ae063e`,
matching the **Dragon and Order Fix** package record, not submitted Selection Fix
`3e50d7d3...`. Reading the preserved Dragon/Order executable for a fresh hash was
blocked by Defender (new event 1116 at 18:46:46 local); it was not executed.
The current-read hash is therefore unverified; the match uses Defender's logged
hash and the prior package-verification record. Do not retry or bypass the block.

A sanitized correction is drafted locally as
`target/work/sep30-defender-report/submission-addendum-draft.txt`, **not sent**.
Christopher has now approved adding that correction and timeline to the existing
Microsoft case. A fresh portal check still shows **Submitted / Pending**, no
analyst comments, and no case-edit/contact form. The automatic Cloud and Client
scan results say **No malware detected** for the submitted Selection Fix sample;
this is not a final analyst determination or clearance of the affected build.
The visible "Provide feedback" control is general website feedback and was
closed without submitting. Microsoft's submission guide says the developer
contact form is provided with the submission results. Keep the approved addendum
ready for that route; do not claim it was posted or ask again for the same scope.
Sending the support CAB would require separate explicit approval.
[Microsoft submission guidance](https://learn.microsoft.com/en-us/unified-secops/submission-guide).

## September 30 — Customize selector fix; replacement setup ready

Christopher chose to delete the failed Test19 and start a fresh overnight install.
Do not resume the old recovery work or start/control an installation for him.
While configuring the new install, changing SoD component 257 raised an invalid
CSS selector error. The choice had been saved; the failure happened afterward,
when `#render` tried to focus an ID containing colons via an unescaped selector.
This was a frontend defect, not a failed SoD mod installation.

The app now finds the exact literal ID inside its own root. Recipe IDs, saved
choices and install validation are unchanged. Regression tests verify checkbox
and option-field focus, no error banner, persistence, and isolation from an
outside-root duplicate ID. Focused checks: **24 passed**. Full frontend check:
**382 tests in 33 files passed**, TypeScript and production build passed. A
read-only native-agent review found no defects. No game/setup was launched.

Replacement local package job `a08a616a-fe4c-4d4b-83a4-1899806f118b` **exited 0**.
Results: `target/work/sep30-selection-focus-package/`. Four package tests passed,
including updater-signature verification; all 223 bundled resources matched.
The packaged executable matched the compiled app with only the expected NSIS
marker change. The Downloads setup and signature hashes were rechecked after
completion. Verified setup, superseding Dragon and Order Fix for the fresh test:
`%USERPROFILE%\Downloads\Chriz Easy BG alpha19 - Selection Fix 2026-09-30-setup.exe`.
Size 6,027,850 bytes; SHA-256:
`614e71cfc4416dc0a71e2bb2eb4616b0f96a33ba889796ab8cc1f628f569620f`.
See the [package verification addendum](package-acceptance-alpha19-dragon-order-2026-09-30.md#selection-fix-addendum--latest-local-setup).
Same app alpha.19 / curated collection alpha.18; no public CEBG publication.
Christopher closes CEBG, installs this app setup, then continues his fresh-install
configuration. No executable or game was launched by the agent; manual acceptance
of this replacement is still his next step.

## September 30 — corrected dragon/order setup ready; Test19 recovery still separate

Verified local setup:
`%USERPROFILE%\Downloads\Chriz Easy BG alpha19 - Dragon and Order Fix 2026-09-30-setup.exe`.
Still app alpha.19 / collection alpha.18. SHA-256:
`3d94c594470486030e3792f94c22ef42bc7cc68e9b2894969fb3c653ace8c6f9`.
[Package verification](package-acceptance-alpha19-dragon-order-2026-09-30.md)
records the published Rebalance v0.7.1 pin, corrected 302-last order, improved
failure diagnostics and duplicate-Resume/progress fixes included in this binary.

Build job `794e24e5-6a8d-42dc-8e30-3078d3bcaa26` exited 1 **after successfully
producing the app and NSIS**, because its PowerShell helper could not resolve
`Get-FileHash`. It was not rerun. The hash-bound signing continuation in explicit
PowerShell 7 exited 0: four package tests, signature verification and all 223
bundled resources passed. Previous setup files were preserved. No CEBG/setup,
game install or desktop click was launched; no public CEBG release/feed changed.

**The stopped Test19 was not repaired.** Christopher subsequently chose deletion
and a fresh combined test; the next decision is no longer recovery versus restart.
Do not tell him to Retry/Resume that stopped copy. Earlier incident evidence and
recovery constraints below are historical, not an active recovery assignment.

## September 30 — Test19 stopped at Rebalance; two independent defects corrected

The later Test19 failure supersedes the running-install status below. Attempt
`attempt-efefa812f3e79bcb8406`, step `0121-fab86e09514d1f0a`, attempt 1 exited 2.
Its first 396 WeiDU entries are unchanged. Thirteen Rebalance components were
appended; 110 failed and 302 was installed last, as declared by the actual TP2.
There is no evidence of a second installer or lab test modifying this game.

- **Apex Dragons:** valid SCS GLOBAL difficulty scripts were rejected by our
  INI-only guard. The owner-repo v0.7.1 fix supports either backend, preserving
  the existing SCS body and all approved dragon tuning. All 52 focused dragon
  checks pass, including the actual Test19 inputs in a disposable synthetic
  game and exact uninstall restoration. Full regression passed: 395 tests,
  no skips. **v0.7.1 is published as normal Latest**, PR #5 merged at
  `1578fac00cd93ba88341ffdfcb0346c08480cd21`, annotated tag on that commit.
  Fresh download: 1,734,037 bytes, SHA-256
  `ba53f02e1b5542534412331dc2f33d78c4b3a19e2dd3dee60ef6a9755ef170ab`.
  Christopher authorized the mod release, not a public CEBG release or
  modification of his installed game.
- **Recipe order:** the batch now follows the TP2's declaration order, with 302
  after 408. The curated generator retains it. A new archive-backed order check
  verifies the pinned archive's length/hash and source order before accepting
  the recipe; it never invokes WeiDU. Both profiles are pinned to v0.7.1.
  Historical local-candidate evidence is kept distinct from fresh published
  acquisition evidence; neither claims full installation or gameplay acceptance.
- **Failure explanation:** bounded diagnostics now distinguish missing selected
  components from wrong order when the old log prefix is unchanged. Other-mod,
  duplicate, changed-prefix and malformed cases still reject. This only improves
  the explanation; it does not weaken reconciliation or enable unsafe Resume.
  The earlier duplicate-Resume/frontend progress corrections are also in source.

**Test19 itself is untouched and is not safely resumable yet.** Updating CEBG or
the mod archive alone cannot repair its frozen failed attempt. Do not edit its
recipe, plan, WeiDU log, ledger or failure evidence. A future supervised recovery
must explicitly record the corrected order, rehearse component 301's nonempty
`OTHER.301` ADD_SPELL bookkeeping, restore the 396-row prefix and affected files,
then reinstall the same 14 selections and finish the eight frozen remaining runs.
The old `recover-no-sr-tempus.ps1` worker cannot simply be retargeted to this case.
Expected final count for that exact selection is 27 BG1 + 425 BG2 components.

The 395-case owner-suite job `a1860b1e-802c-4b74-925c-4cab146fbaa8` finished with
exit 0 in `target/work/sep30-rebalance-global-release-tests`. Do not rerun it.
Collection combined verification job `6846363d-d5ff-4337-ab90-4b36a6aace25`
finished with exit 0 in `target/work/sep30-dragon-order-checks`: 137 Python cases
(one existing skip), 850 Rust passes (14 opt-in ignores), 380 frontend tests,
TypeScript, formatting, warnings-denied Clippy and both profile validations
(zero findings). The added post-publication metadata check and archive checker
also passed their separate 11-case Python run after fresh release acquisition.
Do not rerun the completed jobs.

Packaging job `794e24e5-6a8d-42dc-8e30-3078d3bcaa26` and its successful signing
continuation are recorded in the newer entry above. They did not launch the app,
install a game, update a public feed or recover Test19. Read-only final log hash remains
`ac62ca57362662983ad81ec13eb3ebec5fe1536b40824716dd1d7109ab8fe9d0`.

## September 30 — resume display fix in source; leave the live installation running

Christopher installed the corrected local setup and resumed Test 19 himself.
Read-only evidence confirmed Infinity attempt 2 completed (ledger 178), Bubb's
Spell Menu completed (180), and BGGO started (181), with one CEBG process and its
one BGGO child. This proves the Infinity launch recovery, not full installation
completion or gameplay acceptance. No agent started a second game installation.

The reported `build_already_running` banner came from rejecting an additional
resume request; native rejection does not stop or replace the existing worker.
The frontend left Resume enabled while the first request was pending, kept the
rejected request's error banner, and had no phase projection after reopening.
Tool-cache byte counts could also appear as 100% of an unrelated install step.

Source now guards launcher Resume, failed-step Retry, and manual-checkpoint
continuation before asynchronous work. Pending actions are disabled; early
worker events and real rejected-resume errors are preserved. Old event callbacks
and delayed pause/failure snapshots cannot overwrite a newer attempt. Reopened
installs display phases actually reported by their frozen campaign, without
reevaluating against the current recipe. Finished-step progress is cleared, and
artifact download progress is not confused with mod-install progress.

This frontend follow-up is **not in the signed Downloads setup below or
the currently running app**. Include it in the next package; do not close,
replace, pause, or retry the live install to apply a display-only correction.
No game files, frozen recipe, ledger, saves, native campaign guard, or public
release/feed were changed. TypeScript, all **380 frontend tests (32 files)**,
the production frontend build, and scoped whitespace checks pass. This includes
new duplicate-click, delayed-acknowledgement, real-rejection, old-event/snapshot,
manual-pause continuation, phase projection, and native download-ID regressions.
Native/package tests were not rerun for this frontend-only follow-up; the signed
package evidence below remains specific to the earlier delivered binary.

## September 30 — Infinity launch failure: corrected package ready; user retry next

**Use the new verified local setup:**
`%USERPROFILE%\Downloads\Chriz Easy BG alpha19 - UI Resume Fix 2026-09-30-setup.exe`.
It is still app alpha.19 / collection alpha.18. Setup SHA-256:
`b17421335809b1f5b9c26a4475c1194875bbb86e3f6669860fc2e9b85bb19738`.
See [complete package evidence](package-acceptance-alpha19-2026-09-30.md).
Christopher closes CEBG, installs this app update, then reopens Test 19 and retries
the failed step. No game install/resume or desktop click was performed by us.

The same combined **Test 19** resumed beyond the cache failure: EET and EEex
completed. Ledger event 176 stopped `install:game-interface-infinity-bg2` before
Infinity launched. Its attempt contains only `before.log`; there is no invocation
or process evidence and the installed log ends with EEex components 0–8.
All six new optional UI descriptors mistakenly selected `explicit-tp2`, which the
production CLI deliberately forbids. This is our recipe/validation defect, not a
demonstrated Infinity runtime incompatibility. Do not restart the test or edit its
frozen recipe, ledger, game files or saves.

Source now uses supported setup-name mode for all six. Their pinned archives were
hash/length checked and their TP2/setup identities checked for collisions. Public
validation and fresh-install preflight reject unsupported invocation modes before
staging, including unselected options. General authoring fixtures remain permissive.
For the exact previously shipped Infinity 1.28/WeiDU 249 pins, `weidu/ui_compat.rs`
selects setup-name without changing the frozen plan. It verifies the staged TP2
hash, rejects shadowing discovery candidates, and records the correction in new
attempt evidence. Old-failure retry requires the exact terminal guard, attempt 1,
`before.log` alone and an unchanged live WeiDU log. All other missing execution
evidence remains fail-closed. New errors are explained in ordinary language;
technical text is collapsed without losing fresh-copy reasons or Retry access.

The EET cache fix is also implemented: extraction namespace v3 keys immutable
archives by the canonical layout, format, policy and safety limits. Old v2 entries
are left untouched; raw downloads can be reused. No user-cache/game mutation was
performed during this source fix. Focused evidence: 37 archive/materialization/RAR,
13 public validation, 5 UI recipe, 3 native error, 40 frontend error/wizard checks,
TypeScript, exact frozen retry matrix and UI identity/discovery tests pass.
A real pinned WeiDU test also installed a synthetic component using Infinity's
folder/setup naming in an isolated fake game. This is launch-mechanism evidence,
not a full Infinity install or gameplay acceptance.

Complete: combined checks and signed local package. Next Christopher installs the
corrected app and retries **the same Test 19**. No public release/feed update or
desktop clicking is authorized by this repair. Keep the old September 29 candidate
as historical evidence, not the corrected installer. Wish/SR challenge-mode idea
is recorded for later, not implemented in this patch.

Background verification/build job **4e19b950-1727-4886-8d44-69fb261498a3** finished
with exit 101 in `target/work/sep30-ui-launch-recovery-package`, before packaging.
Python (129 cases, one existing skip), Evandra defaults, formatting and all but two
Rust test targets passed. The 13 failures shared outdated disposable production
fixtures copied from the general authoring example's deliberate `explicit-tp2`.
Only fixture preparation in `engine/tests/cli.rs` and native `command_contract.rs`
was corrected; generic authoring files and existing test assertions are unchanged.
Both complete affected suites now pass (30 engine CLI, 42 native command cases).
No production checks were relaxed. Do not rerun the finished job.

Continuation job **04767c1f-361f-4b20-b6fe-dff90cf32fc8** completed the remaining
Clippy/profile/frontend checks (358 frontend tests) and app/NSIS build, then timed
out at the signing password prompt. Its status remains timed_out, not success.
Windows PowerShell had removed the empty-password environment variable.
`target/work/sep30-ui-launch-sign-existing.ps1` then passed the existing key's empty
password explicitly through Node argv, signed the already-built setup and exited
0. Four package tests and all 221 offline resource checks passed; both Downloads
hashes were independently rechecked. No job was rerun and no game install started.
The finished output is:
`%USERPROFILE%\Downloads\Chriz Easy BG alpha19 - UI Resume Fix 2026-09-30-setup.exe`.
The current live WeiDU.log and failed attempt's before.log both hash to
`7890d5bd4df56bcb585ecbac6c8a0d0dfa6a02bde104b41ca670713976d7be0d`.
The extra 34 focused Python cases and the expanded retry matrix passed. Diagnostics
now includes the small UI correction evidence file, never the third-party TP2.

## September 29 — EET extraction-cache collision: test unblocked, source fix pending

The combined Test 19 install stopped at `materialize:eet-official-74e91d72bca5d073fa11c1d088b90d7ff0c7105d:bg2`
(ledger event 94, first attempt), before this step published EET into the game.
The archive hash `9834a53322b7fe9d8923bfde36c0e6bc8bef54d730d32ac29286b80d6f08287e`
was reverified successfully. The existing extracted-cache marker expected only
`EET`/`EET_end`; the installed candidate now also requires `EET_gui` and its TP2.
`engine/src/acquire/archive.rs` keys extracted trees by archive hash alone but
then requires the exact requested roots/TP2 list. This is our cache identity bug,
not a bad download, Evandra problem, UI runtime incompatibility or damaged save.

Immediate recovery: the old extracted-cache copies were moved aside under
`%LOCALAPPDATA%\dev.chrizhermann.bgcollection\extracted\sha256-v2\98\`
as `<hash>.stale-layout-20260929` and `<hash>.stale-layout-20260929-second`.
The first move unexpectedly left another same-marker old directory at the source;
its identity was rechecked and a separate rename preserved that copy too.
The active hash directory was then confirmed absent. No files were deleted;
the verified raw archive, game files, saves, frozen recipe and ledger are unchanged.
Christopher was told to click Retry failed step; no retry or live installer action
was performed by the agent. The subsequent read-only ledger inspection on
September 30 verified EET and EEex completion; see the current entry above.

Before public release, fix extraction identity so differing requested layouts of
the same verified archive can coexist (canonical layout fingerprint, with full
marker/file verification retained). Do not simply weaken the marker comparison
or delete caches broadly. Cover old-to-expanded-to-old layout reuse and concurrent
different-layout publication; the creator profile still uses the old EET layout.
Also replace the user-facing hash/generic-check failure with a named mod and
actionable wording; keep technical detail collapsed. Prefer an automatic cache
refresh that does not need user intervention. Current error classification misses
this materialization failure, and its generic Retry advice cannot repair the old
cache by itself. Source hardening and user-facing copy changes remain pending.

## September 29 — Wing Buffet default; combined install now running

Christopher approved BG Rebalance `111` (18-second Wing Buffet spacing) as a
recommended default. Source recipe/preset/curation now include it; `110` remains
unchecked and the existing Smarter Dragons requirement is unchanged. Rebuild
before public packaging: the verified Downloads candidate below predates this
default-only change and must not be described as containing the new default.
Do not replace or interrupt the running test for this checkbox change.
Both source profiles validate with zero findings; 57 focused Python checks and
22 focused Rust checks pass after updating the exact selection/ledger snapshots.
The other ten curated-clarity cases passed in the preceding run; its stale
Emotion component-list assertion was corrected and the affected case rerun.
Formatting and scoped diff checks pass. New recommended counts are 454 curated /
360 base components; earlier package counts below describe the unchanged EXE.

Read-only inspection of `%USERPROFILE%\Games\Chriz Easy BG Test 19` confirmed its
frozen selections already enable `111`, Infinity UI++, Evandra core, and Modpack
`224`/`225` (Sorcerer/portrait). Evandra acquisition completed using the verified
manual archive cached on September 16; no repeat file prompt was needed.
This confirms selection/acquisition, not completion or in-game behavior.

## September 29 — combined test installer ready; one manual install next

Latest scope decision: **Less companion chatter is deferred to the following
release**, not added to this candidate. Its direction is approved and recorded in
[the chatter plan](plans/2026-09-29-companion-chatter.md). Proceed with the existing
verified installer and minimal combined check; do not rebuild for this idea.

Christopher asked to proceed, not extend planning. The approved owner releases,
UI-provider integration and manual component breakpoints are complete in this
checkout. The refreshed local test installer is built and verified, not installed
or publicly released. See [package evidence](package-acceptance-alpha19-2026-09-29.md).
The Artisan owner published **v4.81a-dev.85928f9-chriz.1** with optional shifted
dialogue (51010); PR #11 merge `0798dc4` and all three TP2 versions match. Twelve
focused checks passed; all 2,705 downloaded files match the commit. Root verified
the public asset size/hash and passed the exact pin to the recipe worker. See
[fork versioning](fork-versioning.md) for provenance. Modpack **v0.2.0-alpha.8**
is also published as normal Latest (commit `baa176b`), with approved 224/225/611/630
and the bounded legacy-UI helper 640. Root verified public asset metadata:
1,575,997 bytes, SHA-256
`8019d60d2d5c9e5b5961b73ddfcc65729d7feb819bc3e99f4134dd3058835989`.
Its owner passed Linux/Windows CI including eight packaged-installer checks.
Unapproved companion-stat/Baeloth choices stay out. Final recipe integration is
complete; helper 640 runs after LeUI/Dragonspear and before Bubb, independent
of HGO and the optional gameplay-Modpack switch.
BG Rebalance **v0.7.0** with 302 is published as normal Latest (PR #4, merge
`5db2f64`). The downloaded archive is 1,733,901 bytes, SHA-256
`3aae59bc634671ad6a6565d5862c4db984584bd90094ecbf640dda338d665b87`.
All 13 Flail checks pass; full suite has 382 cases with 26 existing fixture skips,
exit 0. TP2/TPA parsing and the extracted catalogue pass. Exact pin/default 302
integration is complete. No live gameplay claim.

Manual breakpoints are implemented across engine/native/frontend. Focused checks:
11 checkpoint cases, two CLI cases including a complete two-pause/final-receipt
scenario, 42 native command-contract cases, workspace all-target check and Clippy
with warnings denied pass. Frontend suite passed 352 cases before the last small
guidance refinements; the 17 focused cases/typecheck passed after those refinements.
Combined final checks and package build now pass. Technical pause evidence
stays collapsed; guide explicitly says no automatic full checkpoint backup.
The feature accepts appended WeiDU components, not arbitrary removals/reinstalls
or file-copy mods. Accepted manual additions permanently disable game patches/mod
upgrades for that installation, not app updates. See [guide](manual-mods.md).

Final recipe integration passed 129 Python cases (one existing skip), 50 focused
Rust cases (one existing HTTPS-dependent ignore), both public-alpha validation
profiles and the active Evandra publication gate. Recommended counts: **453
curated components / 50 runs**, **359 base components**. Modpack 611 replaces
610 as default; Evandra 224/225 add two curated defaults. All six alternative UI
choices are selectable/experimental, with 640 automatic only for classic UIs.
The narrower base profile still omits Evandra core; only the full generated
profile enables her dependent fixes. Intermediate Artisan 1.6 recipe descriptors
were removed, not the published legacy release. Default UI/Emotion choices remain.

Combined check job **630d3594-bbb5-4f5e-84ba-7d03240fe9f0** exited 101 before
packaging. Python (129 total, one existing skip) and the Evandra gate passed.
Its first Rust failure was an exact test allowlist omitting the six newly approved
UI modules. Root corrected that list and the corresponding authored-run-order
expectation (six UI runs plus helper 640). Both full test files now pass, 11 cases;
assertions remain exact and no production feature/check was weakened. A read-only
scan found no additional stale integration expectations; the full suite then passed.

Continuation job **a820c812-eac4-479e-bdad-68f1fb19645c** completed with exit 0 in
`target/work/sep29-next-release-package-final`, using
`target/work/sep29-next-release-continue-build.ps1`. Saved status/stdout/stderr were
inspected after completion. Workspace tests, formatting, Clippy, 353 frontend
tests/typecheck/build, signed NSIS build and all four package tests pass. Offline
inspection matched all 221 bundled resources and the packaged executable; the
Downloads setup/signature hashes were independently rechecked. Unchanged Python
and Evandra-gate results above remain valid. Exact results and hashes are in the
[September 29 package record](package-acceptance-alpha19-2026-09-29.md).
Verified installer: `%USERPROFILE%\Downloads\Chriz Easy BG alpha19 - Combined Test 2026-09-29-setup.exe`.
It contains app alpha.19 / curated collection alpha.18. No app/setup/game was
launched; public CEBG release/feed/website are unchanged. Do not rerun either job.

Manual tests must remain minimal: see [the combined smoke check](next-release-smoke-check.md).
One fresh installation and a brief launch/save/reload plus targeted new-feature
check, not a full replay or another install for each component/provider. Existing
accepted work is not reopened without a concrete regression. Do not operate the
desktop, mutate a live game, or start the user's install without authorization.

## September 29 — approved fork refresh, story XP and Flail delivery

The following records the earlier source-integration milestones. Publication and
delivery statements in the section above supersede its then-pending statuses.

Christopher approved including our published **Artisan `chriz-v1.6.0`** and
**Bardic `v2.9c-balance.6`** in the next fresh-install recipe. Both pins are now
integrated in source; this is not an installed-game update or public package release.
Preserve existing kit/default selections and do not enable Bardic 3010 by inference.

Downloaded artifacts passed the engine's acquisition/hash/extraction/TP2 checks;
both profiles validate with zero findings. 27 focused Python and 24 focused Rust
tests passed, and regeneration reproduces all 101 files byte-for-byte. Recommended
counts remain 450 curated / 358 base, with no new selected component IDs. The
Artisan 20101 name now describes its actual legacy-Manyshot cleanup. New ledger
records describe the two package refreshes; the stale expected ledger count was
updated and retested. Existing Evandra publication checks still correctly reject
the old Modpack alpha.7 artifact. No game, app package or public release changed.

The remaining published refreshes are also integrated: **SoD Remix v0.6.12**
and **BuffBot v1.9.0 stable**. Fresh production acquisition/extraction passes for
both. Follow-up checks: 27 Python and 19 Rust tests pass; both profiles validate
with zero findings and all 102 generated files reproduce byte-for-byte. The
semantic selection digest is unchanged. SoD 256 contains the new bridge timer;
258 is not added and 257 stays optional. BuffBot keeps components [1, 0] last.
The existing-game BuffBot 1.8.3-to-1.8.4 update/undo route and its old artifact
are deliberately unchanged. The Evandra package gate remains active.

Version identity: Artisan 1.6.0 is the **legacy independent fork release**, not
upstream's version. Christopher rejected continuing that naming: the next release
must use **upstream version + our revision**, like SR/Bardic. Upstream-only changes
advance the base part; our changes advance the suffix. Keep published tags and
archives immutable, identify untagged upstream snapshots accurately, and migrate
all three installers/release metadata/CEBG pins together when the next owner
release is prepared. See [fork version identity](fork-versioning.md) for the rule.
Current exact migration target: `v4.81a-dev.85928f9-chriz.1`; upstream's latest
named release is v4.81a but the included base is 297 commits later. Artisan owner
chat `01a0c7fa-4c66-79f3-9255-b3fcb0e6a452` is preparing the source/tooling migration;
no tag, archive or current CEBG pin is renamed in place.
Artisan's included upstream commit is `85928f964f6965fd37980bf679d7eeb9af410256`;
the three Artisan installers share one fork artifact. The September 29 owner audits
are complete: Artisan, Bardic, Spell Revisions and Randomiser have no incoming
upstream changes to merge. Updating CEBG's older pins remains separate work.

Utility-XP decision: **campaign/story-based rewards are the intended new default**,
with BG1-sized rewards throughout BG1 and SoD and BG2-sized rewards beginning in
SoA (also in ToB). Keep existing Modpack 610's protagonist-XP smooth curve unchanged
as an optional, mutually exclusive alternative. The suggested party-average
redesign is deferred, not part of this implementation. The Modpack owner chat
`01a0b92c-da6d-7701-8d51-b0df1a17fe08` implemented **611 / cbm_story_utility_xp**
in the `companion-spellbooks` checkout. It uses EET campaign flags rather than
overlapping chapter numbers and rejects unsupported campaigns. Owner reports
376 focused offline checks passing, including transitions/direct starts/reloads,
reciprocal 610/611 exclusions, installation and uninstall. Current 610 calculation
is unchanged. Christopher explicitly approved **EE Fixpack's 155-XP correction**
for level-16+ BG1/SoD locks rather than the original 15; the owner verified the
implemented value and removed provisional wording from its README, changelog and
guide. See that checkout's `docs/story-utility-xp.md`. Reconcile this
branch with the pending Modpack 224/225/630 owner work before release. No new
source component is selectable until its owner release contains it.

**Flail of the Ages is implemented as BG Rebalance 302 / cbr_flail_of_ages_rebalance**,
intended default after publication: +5 Free Action permits Haste/Improved Haste; Slow saves versus
breath at -2/+3, -4/all +4 variants, -6/+5. Preserve proc chance, duration, damage,
unrelated protections and lower tiers. Source is on `codex/flail-of-ages-rebalance`
in `C:\src\private\chriz-bg-rebalance\.worktrees\bard-progression-release-20260920`,
based on released v0.6.0. Thirteen real-WeiDU disposable tests pass, including
captured vanilla/CEBG resources, SR rider saves, local descriptions, idempotence
and exact uninstall; the four full current CEBG descriptions passed separately.
Parsing/diff checks pass. Not committed, published, integrated or live-tested;
do not add 302 to the old v0.6.0 artifact. Use the existing late post-EET_end
Rebalance run, with no EEex dependency. Broader Free Action policy and Dispelling
Screen remain deferred. See the
[Flail design](plans/2026-09-28-flail-free-action-and-dispelling-screen.md).

Next after these owner changes: publish/verify their reviewed artifacts, integrate
the remaining approved pins/components, finish alternative UI provider wiring and
advanced manual breakpoints, then focused combined checks and a new package for
Christopher/community testing. Existing Emotion 301 remains default. No live-game,
save, installed-app, deletion or desktop-control action is authorized by this work.

## September 29 — release follow-up decisions and parallel owner reviews

- **Emotion Courage/Hope is included:** BG Rebalance 301 is already default in
  recipe alpha.18. Its omission from a chat summary was not a scope change. Keep
  it in the next package; the existing no-extra-playtest-gate decision stands.
- **Unidentified-item visuals:** reuse ordinary installed-game item icons and
  pictures by category; no newly drawn/generated art. Modpack 630 already reads
  donors from `unidentified-items/donors.2da`; identified items retain their art.
- **Instant trap detection:** deferred beyond the upcoming release. Do not add it
  as a new last-minute implementation or test gate.
- **Flail of the Ages:** remains the next bounded balance candidate, using the
  already recorded +5 Haste compatibility and tiered Slow saves. No source or
  gameplay implementation was performed in this coordination turn.
- **Utility XP:** Christopher suspects the agreed curve is too generous, not
  necessarily broken. Uncapped XP and a long-running 3–4-member party may reach
  high reward thresholds early. Current source uses protagonist lifetime XP and
  reaches 1,000 party-total scribing XP per spell level at 440,000 XP. Asked the
  Modpack owner to compare the existing mode with EET campaign/chapter-driven
  progression (BG2 rewards unlocked at SoA), with both modes as possible options.
  No new formula, defaults, chapter numbers or live-game changes selected.
- **Artisan Fighter discussion:** inspect the reported all-weapon two-pip grant,
  exact level and single/multi/dual-class behavior, then discuss removal/reduction.
  Do not implement a nerf or change kit defaults before Christopher chooses.

Owner requests dispatched to existing chats: Artisan/upstream and Fighter review
`01a0c7fa-4c66-79f3-9255-b3fcb0e6a452`; utility XP
`01a0b92c-da6d-7701-8d51-b0df1a17fe08`; Randomiser upstream
`01a0bb24-3c8e-77b2-849a-b3d3b0300eb0`; Spell Revisions upstream
`01a0df45-df6a-7cb1-9e6a-0f70435f4419`. Upstream requests are bounded comparisons
and merge-risk recommendations, not permission to merge/release automatically.
Use thread status/results when resuming, not duplicate investigations.

Bardic upstream check completed: live upstream master `4e72d973` equals the
merge-base of published fork master `7db98616` / balance.6; 30 fork-only and zero
upstream-only commits. No upstream merge is needed. Dirty local owner work was
preserved. The other reviews were still running when dispatched; do not infer
their results from Bardic's status.

Fade's proposed humorous race label is a separate user-led fork task, personal
and cosmetic only. No race/stat/global text/save change is authorized here.

## September 29 — UI choices and manual breakpoints in the next release

Christopher explicitly reconfirmed that **manual installation breakpoints belong
in the next release too**, not an unspecified later follow-up. Use the existing
advanced/off-by-default design: multiple completed-component boundaries, explicit
acceptance of manual additions, intact WeiDU history through finalization, and
game hotpatches disabled for manually customized installs. Implementation is still
pending; do not confuse ordinary pause/resume with this feature.

Testing policy: do proportionate automated checks and basic installer-safety
verification, then leave broader UI/mod compatibility and gameplay testing to
Christopher and the community. Do not require exhaustive agent-led full installs
or make Christopher personally test every option before shipping. Fix known
deterministic failures and retain file/history safeguards; untested combinations
can be clearly described as experimental and awaiting community feedback, not
certified or already community-tested.

Christopher now wants Dragonspear UI++ included as an **optional, new-install-only
interface choice**, alongside the previously approved UI providers. This replaces
the earlier wait-for-a-request decision; the recommended interface stays unchanged.
The selector exists, but production provider integration is still pending. Pin
Dragonspear's own package and record its component/order/conflicts rather than
assuming Infinity UI++'s rules apply. See the updated
[UI plan](plans/2026-09-20-ui-choices-and-manual-checkpoints.md) and
[release checklist](plans/2026-09-22-release-readiness.md).

Praetor reports that manually installing it after pausing at EEex made finalization
reject extra WeiDU.log entries. Removing those entries allowed continuation, but
that workaround hides installed history and must not be recommended. Record this
for the manual-checkpoint feature in the same next release: accepted additions must
survive resume/finalization with their log rows intact; unapproved drift remains
blocked. A built-in UI selection should need neither manual insertion nor log edits.
This update records scope/report only: no recipe, package or game changes.

## September 29 — high priority for later: Fade XP catch-up

Christopher reports that Fade does not catch up in experience. Investigate and
fix soon, but this request is **capture only**, not authorization to inspect or
change the running game. Confirm the affected install, expected catch-up behavior
and recruitment/rejoining circumstances before diagnosing the cause. Route any
collection-owned companion fix to `chriz-bg-modpack` after checking the existing
Fade/XP handling. No cause, fix, release target or save-repair procedure is verified.

Additional September 29 report: **Evandra also does not reach the expected
1,250,000 XP**, despite Christopher's protagonist having substantially more XP.
Include her in this high-priority companion catch-up investigation. A subsequent
read-only check found the 1,250,000 XP rule in the installed Evandra source
`scripts/bgee.baf`, but its inclusion in the effective compiled script is not yet
verified. Do not assume Fade and Evandra share a cause. No game or save changes.

The same read-only investigation confirmed two separate migration omissions:

- **Sorcerer conversion:** the historical `EVANDRA_SORCERER` v2 patch and curation
  notes specify it as the intended default, but the public recipe never acquired
  a conversion feature/run. The stream install has Evandra v2.2 core/crossmod and
  Modpack alpha.6, no conversion; effective `rh#eva.cre` and `rh#ev25.cre` remain
  Mages with the Illusionist kit. This is not a failed installed conversion.
- **Custom portrait:** stream `override/rh#eval.bmp` is byte-identical to the
  installed upstream `evandra/images/rh#eval.bmp` (SHA-256
  `1a8a4209a5b0e6c6f59bf4b09cac889eff484b3a44bf4a95125ae7e4d4c5e0b2`). The old
  read-only reference's custom override still exists and differs (SHA-256
  `b4bde98aec195d47db521d3cc4d72b636ae5a0f391cc7469ea5d48688b112aad`). Stream
  `rh#eva.cre` points both portrait slots at `rh#evaL`. The missing asset migration
  was already documented in `curation/components/EVANDRA.md`.

September 29 implementation follow-up: Modpack source now owns **224** (Sorcerer
with a starting spellbook appropriate to her stored level) and **225** (custom
portrait). Christopher confirmed he created the portrait with ChatGPT and
approved including it. The two options are independent and intended defaults
when Evandra's core is selected; skipping Evandra skips both. Source integration
and a packaging check are implemented here, but **the released alpha.7 artifact
does not contain them**. Publish and verify the owner release, pin it and regenerate
the recipe before packaging CEBG; do not append new IDs to the old archive's
catalogue. See [delivery status](handoffs/2026-09-29-evandra-default-restoration.md).
This does not fix the separate XP catch-up report. Christopher is repairing the
current companion himself in EE Keeper; do not edit his running game or saves.

Later September 29, Christopher explicitly authorized the portrait replacement
only. Restored the old custom BMP into the stream game's `override/rh#eval.bmp`
and verified its hash; retained the original at
`%USERPROFILE%\Games\Chriz Easy BG\portrait-backups\evandra-20260929-032318\rh#eval.bmp`.
No save, CRE, TLK, or gameplay state was edited. In-game visual confirmation is
pending reload/restart. This local repair does not implement recipe delivery.
He wants her **starting level-10 spellbook**, then to select the later spells
himself while leveling to 1,250,000 XP. The old SoA conversion starts at level 10 /
300,000 XP with 5/4/3/2/1 known spells and 6/6/6/5/3 casts; leave level 10 when
raising XP in EE Keeper so the game offers the level-up, rather than presetting
level 13 and its spells. He owns the class/spellbook save repair.

## September 28 — balance ideas captured for later

See [Flail / Free Action / Dispelling Screen](plans/2026-09-28-flail-free-action-and-dispelling-screen.md).
Flail balance belongs in BG Rebalance, intended default when delivered: +5 Free
Action should allow Haste; Slow gets breath saves at -2 / -4 / -6 for +3 / all +4
variants / +5. Broader Free Action policy is separate and undecided. Dispelling
Screen first needs a correctness check for reported dispels getting through
while active. Alternatives, not chosen together: a 0.5–1 second grace period,
the earlier longer window, or excluding dispels from sequencers/contingencies
including AI combinations. Entropy Shield balance is a separate low-priority
discussion. Record only: no source investigation, implementation, selection or
release commitment yet; leave all of these for another session.

## September 28 — stronghold selection repair and bounded compatibility audit

See the [selection audit](plans/2026-09-28-selection-compatibility-audit.md).
Source now excludes CDTweaks Multiple Strongholds 1160/1161 from individual
all-class choices 1340–1347, without making the individual choices exclusive to
each other or changing defaults. Generator, curated recipe and unreleased
alpha.18 ledger updated; not packaged/released and no game files touched.
The bounded source scan is complete. It also fixed missing BG1NPC core
requirements on 36 optional choices and made our SCS bundle require Ascension's
actual finale component, not just its parent selection. All defaults remain
unchanged; 46 Python, 31 Rust and 13 Customize tests passed (see audit for scope).
Opus returned no findings: its job failed because the installed Claude client
does not support the requested model. Lead/native source review supplied these
results instead. No retry or client update is pending.
Mystic Fire is parked in the Artisan owner's deferred plan; kit defaults remain
unchanged and Warlock stays deferred.

## September 27 — fresh owner inventory and two small source additions

See the [pre-package inventory](plans/2026-09-27-pre-package-inventory.md) before
building. The source recipe is behind SoD v0.6.12, Artisan chriz-v1.6.0, Bardic
balance.6 and BuffBot v1.9.0. Alternative UI controls exist, but production
provider integration is still absent; Infinity/EET are prepared and LeUI needs
its game-2.7 helper. Simplified proficiency and companion pip options need the
specific compatibility work recorded there, not just inclusion of upstream IDs.

Approved source additions are implemented: advisory folder size in deletion
confirmation (42 frontend/six native focused checks and typecheck passed), and
optional shifted dialogue in the Artisan owner's `codex/shifted-dialogue-qol`
branch (five real-WeiDU synthetic tests passed). Christopher explicitly waived
a separate manual dialogue test gate; do not claim native playtesting. Neither
addition is packaged or released. The inventory also preserves Modpack 630,
separate dirty companion work and active Randomiser work without silently
merging or selecting them. No live-game, deletion or publication action occurred.

## September 27 — batch generic-artwork testing with the next release install

Christopher asked to test the new Modpack **630** generic unidentified-item
artwork component together with the **next release installation**, not start a
separate test now. It is recorded in the
[next release-install checklist](plans/2026-09-22-release-readiness.md#next-release-install-playtest--additions-agreed-september-27).
Use 630 with conflicting Randomiser appearance component **570 off**; ordinary
loot randomization can remain. Source is committed in the Modpack feature branch,
but publication, CEBG pin/exclusion integration and live acceptance are pending.
No game action or recipe/default change was performed for this scheduling request.

## September 26 — recipe alpha.18: approved Emotion default

The prior app alpha.19 / recipe alpha.17 source was completed and committed first
as `d57b70a`. The next recipe, **0.1.0-alpha.18**, pins BG Rebalance **v0.6.0** and
makes component **301** default/ready under `mod:chriz-bg-rebalance`, with no extra
requirements. Recommended selection: **450 components / 50 runs**; the base
profile has 358 components. Bard progression `420`/`421` stays unselected.

Christopher's decision is dated **2026-09-26**: fix any issue in a follow-up,
without gating on more testing. Earlier live checks used standalone spells;
IWDification provider mode has fixture coverage. A quick Hope/Courage replacement
and refresh check on the next install is nonblocking. Source ordering and the
later CDTweaks/Modpack interactions are recorded in the
[recipe note](recipe-alpha18-2026-09-26.md).

The v0.6.0 archive was freshly downloaded, hashed and verified with the installer
authoring tool. The generated recipe, curation totals, source/evidence records,
golden digest and alpha.18 ledger are synchronized. App version stays alpha.19.
This is a source/recipe change for new installations or the next playthrough;
the September 23 package still contains alpha.17. No publication or live action.

Final source verification passed: **806 native tests**, 13 existing opt-in ignored;
formatting and Clippy with warnings denied; **105 Python tests**, one existing
opt-in skip. Native job `4e708c54-f961-425a-9878-fab6c6706bcd` exited 0; status and
logs in `target/work/sep26-alpha18-native` were inspected. Both public-alpha
profile validations returned zero findings. Planning resolves 450/358 components;
the existing 5/6 conditional-omission notices are unchanged, and disabling 301
removes only that component. Fresh generation reproduced all 101 recipe files
with normalized line endings. The semantic golden digest is
`9f5d6131c8c13887a80b6d82b592c440484d84043f38f0a0f5378ee177a29015`.
The app's 327 frontend tests/typecheck/build passed during the preceding source
closeout; this recipe change adds no frontend code.

## September 26 — finish recipe alpha.17 / app alpha.19 source

Christopher requested completion and a commit of the existing installer work
before adding BG Rebalance 301 to the next recipe. The existing comparison,
BuffBot update/Undo, naming, progress, diagnostics, Customize and recipe-record
changes are complete as source. A focused review found and corrected hidden
patch/Radar feedback on the default Updates selection; four regression cases
cover progress, success/error and switching installations.

Current checks: frontend typecheck, 327 tests and production build; Python
102 tests with one existing opt-in real-WeiDU skip; 803 native tests with 13
existing opt-in tests ignored,
formatting and Clippy with warnings denied; both recipe profiles pass
public-alpha validation with zero findings. Native job
`b38bf370-eda2-4cab-9366-b58677d4ea59` exited 0; saved results are in
`target/work/sep26-alpha17-final-native`.

The September 23 package record is historical and predates today's feedback
correction. No package rebuild, publication, installed-app or game action was
performed. Final packaged UI review remains user-driven. Existing accepted
BuffBot Apply/gameplay/Undo is not being repeated. The known concurrent-name
reservation limitation remains a follow-up. No new dependencies or public APIs
were introduced by the finishing correction; diagnostics tests contain deliberate
fake credential strings for redaction coverage, not credentials.

Completed as source in the separate recipe change above: Christopher's September
26 approved BG Rebalance 301 default with v0.6.0, leaving bard progression
420/421 unselected. The provider-mode check remains nonblocking.

## September 24 gameplay follow-ups: preparation only

September 25: Christopher authorized a parallel, source-backed Time Stop/Haste
feasibility plan, not implementation. Proposed owner: BG Rebalance. Check whether
EEex can safely separate periodic cadence and buff expiry from speed/time-stop
behavior; do not promise full tabletop fidelity or current-save compatibility.
See [research scope](plans/2026-09-24-effect-timing-traps-and-shift-dialogue.md#september-25-feasibility-research-requested).
The [source-backed feasibility report](plans/2026-09-25-timestop-haste-feasibility.md)
is now complete: credible route, but usual effect-list listeners run too late.
Next would be a separately authorized isolated proof before native expiry/pulses,
or a focused EEex extension. No implementation or runtime test occurred.

New backlog items: Time Stop/Haste effect timing, instant trap detection, and
optional off-by-default dialogue for shifted Shapeshifters/Hivemasters. See
[scope and preparation](plans/2026-09-24-effect-timing-traps-and-shift-dialogue.md).
These are not additions to the built review package or new release gates. No live
game/desktop control is authorized; Christopher is streaming. Keep the existing
installer acceptance work separate and user-driven.

## Current priorities — September 22, after accepted BuffBot Undo

Christopher lowered priority for (1) broader existing-mod updates, (2) a general
current-run/new-run/new-install classification system and (3) more hotpatches.
Keep the two existing supported routes and their protections; these expansions
are not release gates. See the updated [work index](plans/2026-09-20-next-work-index.md)
and [preserved requirements](plans/2026-09-20-existing-install-upgrades-priority.md).
He subsequently approved the four implementation lanes. Source now includes
compact Updates with real backup progress, persistent completion/Undo/error
feedback, unique display names and rename controls, contextual folder diagnostics,
and the completed Customize/typed-input frontend from `cebg-ui-options`.
Full native verification passed: **797 tests**, 13 existing opt-in tests ignored,
formatting and Clippy with warnings denied. Job
`cc6839dc-ab5b-476b-b2ff-31ff351e25cb` exited 0 at 2026-09-22T15:02:24Z;
saved status/stdout/stderr in `target/work/sep22-feedback-final-native` were read.
The older channel compile failure (`0ecc35d9...`) and generic-name test failure
(`f2299f45...`) are already fixed and superseded. Their delayed notifications do
not call for rerunning them.
A final naming review found that automatic destinations must stay free after
display-only rename. The native defaults helper now independently suffixes
occupied/registered destinations; six extra native tests cover it. The final job
passed all 38 app-library tests (eight naming), 40 command contracts, Clippy,
frontend 323 tests/typecheck/build, and four package tests including the signature.
The first package attempt stopped before packaging: two new test expectations
used mixed Windows path separators. Actual free-destination selection was correct;
the expectations now use separate native path joins. Job `a7af1479...` exit 101
is handled; do not rerun it unchanged.
Final job **`d75b1f80-08e3-4958-9b3d-bc28168d0a12`** exited **0** at
2026-09-22T15:13:36Z. Its saved status/stdout/stderr were read in
`target/work/sep23-feedback-review-package-final`. Normal release build, diagnostic
feature off. The updater signature matches the bundled public key.
Offline extraction matched all **191 resource files** to the current source,
with no extra recipe files. The executable differs from the raw build only by
Tauri's documented three-byte UNK→NSS bundle marker, with every other byte equal.
Verified setup/signature copies are in Downloads under the new, non-overwritten
name `Chriz Easy BG alpha19 - Review 2026-09-23-setup.exe` (adjacent `.sig`).
Setup SHA-256: `9cbae97c0f51505adc66cd36d897d96933982287c7487377c3d67e88b9528e6a`.
See [final package evidence](package-acceptance-alpha19-2026-09-22.md).
All subagent lanes are done; no command is still pending for this batch. The
package is ready for Christopher's finishing UI review, **not installed, launched
or published**. No further automated game/update test is requested.
UI provider **recipe** integration is still separate: the imported controls stay
hidden when the actual recipe has no verified provider group. Infinity UI++ and
EET SoD preparation remains in the owning fork; LeUI remains compatibility-blocked.
The website draft remains with its owning task. Do not describe these as shipped.
No new game actions, real-install deletion or publication is implied.
Frontend typecheck, 323 tests and web build passed. The full Python suite passed
102 tests with one existing opt-in real-WeiDU skip. Focused headless desktop/narrow
checks passed; evidence is in `target/work/sep22-feedback-layout/verification.md`.

### September 22 implementation notes

- Progress is observational only. Backup byte counts are published after verified
  copies, with final 100% after the source inventory recheck. Unknown work uses a
  spinner. Existing mutation checks, backups, recovery and save policy are intact.
- App-update success survives restart only when the requested different version
  is actually running. Mere download or an online up-to-date result is not success.
- Names live in separate AppData metadata keyed to the registered ID/root. Rename
  never rewrites receipts, frozen recipes, game/save folders or profile identity.
  Legacy duplicates remain resumable and are disambiguated visually. New creation
  and rename have cross-process uniqueness checks.
- Known narrow naming limitation: two independent CEBG processes starting the
  same name simultaneously allow only one registration, but the loser can leave
  an unindexed metadata-only destination because freeze precedes registration.
  No game copying has started. A future reservation across freeze/publication is
  needed before promising clean retry for that concurrency case; do not weaken the
  name check or delete an arbitrary destination to work around it.
- Folder codes preserve operation/path and original OS cause. Failed cleanup no
  longer hides the primary problem. This is error clarity, not a proven diagnosis
  of the viewer report or a new OneDrive/Documents fallback.

## No-SR lookup discrepancy explained — 2026-09-22, user-side repair pending

**Current acceptance: Apply, user gameplay and Undo passed.** Christopher applied
the BuffBot 1.8.3-to-1.8.4 update himself in the diagnostic app, accepted gameplay,
then used Undo. The installation is now back on **1.8.3-alpha**. The earlier Apply
disk checks confirmed successful installation of both components and
the expected new artifact. Transaction: `.chriz/mod-upgrades/v1/01790071515151257500`
inside `%USERPROFILE%\Games\ChrizEasyBG-No-SR-Test`.
All 96 current update-file hashes, 96 affected-file backup hashes and 59 unchanged
guard hashes match. Original receipt and dialog.tlk are unchanged; all 400 other
active WeiDU rows match the pre-update log exactly. The prepared-journal hash is
valid. Optional backups also checked: full game has 159,300 files / 3,393 directories
/ 11,898,950,716 bytes, matching every pre-update path/type/size with no extra or
missing entries; all 12 profile files (1,456,735 bytes, five directories) match the
isolated profile byte-for-byte at the Apply audit, before gameplay. Full-game backup contents were not all
rehashed. Actual backup file handles resolve to normal AppData, not Codex LocalCache;
the applied journal and live TLK resolve to the intended No-SR game. Backup audit
helper: `target/work/buffbot-applied-verification-sep22/backup-check.ps1`.
Christopher subsequently reported **"its fine, it works"** after the requested
existing-save/BuffBot check. Record this as user-accepted gameplay for this narrow
upgrade; no further BuffBot gameplay testing is requested. Christopher then used
Undo, completed at 20:12:39 +09:00. Read-only checks confirm all 155 original guarded
game-file hashes and all 96 affected-file backup hashes match, the entire original
WeiDU.log is restored byte-for-byte, and the original receipt and dialog.tlk are
unchanged. The restored marker matches the prepared journal SHA-256
`4a56f5d5c96ce798c6935ac17601c5b611c31ecf5b632c4e2ebb0e89b217026b`.
No game process was running at verification. This accepts this exact Apply → user
gameplay → Undo flow, not other mod upgrades or publication. No agent game launch,
UI click, Apply, Undo or save edit was performed. Do not reapply automatically.
History wording is corrected in source to **Restored BuffBot 1.8.3-alpha** rather
than presenting the original upgrade arrow followed by "restored". It remains
inside closed history; applied, unfinished and unknown states also have explicit
wording. Typecheck and all 18 focused mod-upgrade tests passed. Not packaged yet.
The journal was created at 19:05:15; preparation finished at 19:37:29; the WeiDU
step ran approximately 19:37:32–19:37:35; the applied record was written 19:37:44.
Most waiting time was preparing the backups, supporting the progress-feedback request.

Additional UI requirements captured (not implemented by the compact-selector work):
- Lead with the supported action and applicability: current run / new run / new
  installation. Put comparison prose, dependency explanations and diagnostics in
  closed Details. Eligibility must be evidence-backed; unassessed work must not
  be labelled compatible. Keep consequential warnings beside confirmation.
- Reuse a compact progress display across installation, updates, downloads,
  backups and game discovery. Show the actual current stage (e.g. backing up,
  downloading, applying, verifying), a determinate bar when byte/file/step totals
  are known, and an indeterminate spinner otherwise. Never invent percentages or
  remaining time. Preserve access to optional activity details and clear final
  success/failure. This is queued UI work, not a change to the running update.
- Completion must be conspicuous for every update type: a persistent, dismissible
  **Update complete** banner at the top, installation/add-on name and new version,
  and the appropriate next action (Play / start a new run / open new installation).
  Keep Details and supported Undo accessible, remove obsolete pre-update wording,
  and distinguish success, partial completion and failure. App-update confirmation
  must survive restart and reflect the completed update, not merely "up to date".
  A newly built copy should say Installation ready, not imply the old copy was
  updated. Progress, completion and errors should be accessible announcements;
  do not rely on changed small text, colour alone or a short-lived toast.

**Latest: the user-side record repair succeeded at 16:57:21 +09:00.** Christopher
ran the supplied launcher from File Explorer. Its result file reports that the
single record was copied into normal AppData and verified at the expected SHA-256,
with the Codex copy preserved and no game/save/mod changes. No CEBG process was
running when this result was checked. He subsequently reopened it and supplied
the successful recognition/available-update screenshot recorded above.
The historical "pending" heading is retained for existing links. Do not rerun the
repair or mistake agent-side registry visibility for user-app acceptance.

The diagnostic build's real user-opened process (PID 73584) sees **one completed
record and three campaign pointers**, not the five completed records and 13
campaign pointers visible to agent-launched helpers. Its native lookup, IPC reply
and frontend acknowledgement agree. No-SR's completed-record probe returns
NotFound; its game root exists, but the remaining historical campaign is sealed
FreshCopyRequired. The UI's generic "folder unavailable" explanation is misleading.

**Proven cause: Windows redirects agent-side file reads to Codex's private AppData
copy.** Opening the apparently normal No-SR record and querying its actual file
handle with GetFinalPathNameByHandle resolves to:
`%USERPROFILE%\AppData\Local\Packages\OpenAI.Codex_2p2nqsd0c76g0\LocalCache\Local\Chriz BG Collection\managed-installs\install-c9dc8c3e0e0b0a3476e0.json`.
The normal user-opened CEBG process cannot see that completion record. This is why
helper checks and earlier agent-launched app checks contradicted Christopher's UI.
Do not claim a refresh or reinstall of the game fixes this. Package-identity API,
environment and directory-handle checks alone missed the redirection: check the
actual **file** handle. Do not copy all historical records, delete the package cache,
or change Codex's virtualization settings.

Prepared in Downloads, **subsequently executed successfully as recorded above**, for Christopher to double-click from
File Explorer outside Codex: `CEBG - Restore No-SR completion record.cmd` and its
same-named `.ps1`. The script restores only the 720-byte completed record into normal
AppData, verifies the exact record and existing game receipt hashes, rejects
redirected destinations and conflicting records, and never overwrites a record or
changes game files/saves/mods. Its syntax and read-only path helper were checked;
the user-run copy then succeeded. Result file: `CEBG - No-SR record repair result.txt`.
Record SHA-256: `16b3c35bf2221265eb336be9f22e4d951597af9f57db2522d727e73274f71d5d`.
Receipt SHA-256: `a06b6e0928ee0dc1463247252e4d118c21234e6bdba702ad18a3dd2061b35379`.
Verify the repair through the **user-opened app's trace**, not an agent helper.
Trace evidence: `%TEMP%\CEBG-installation-lookup-73584-1790060629093.jsonl`.

Christopher is using the PC and drives the app. No unsolicited app closing,
launching, clicking, Apply/Undo, or game actions. BuffBot update, gameplay and Undo
acceptance passed as recorded above. Latest UI request: a compact, obvious selected
installation at the top of Updates, less repeated text and smaller selection frame;
all game-specific actions must use that same selection. Independent user work:
draft the website's differences from vanilla, without publishing untested claims.

### Compact Updates selection — source implemented, not packaged

The selected game now appears directly below Updates in a compact bar. Short,
disambiguated labels replace path/ID-heavy choices; exact folder, identity and
status detail remain in a closed disclosure. Installed mods, current-game fixes
and Radar use the same selected ID. Changing games clears old comparison/fix
previews and errors; selection is locked during patch operations or mutations,
while obsolete read-only comparisons remain discardable. Radar no longer quietly
targets the independently selected My installs entry. App/collection actions are
separate below the selected game's controls. New-copy availability on this page
follows the selected game, rather than any other outdated installation.

Existing dark/gold styling retained with tighter, Updates-only spacing. Persistent
installation renaming remains separate roadmap work; these labels do not rename
game folders or rewrite records. Stale entries now say Needs attention in the
top bar rather than asserting a missing folder when other checks may have failed.

Validation: `npm run check` passed (typecheck, **223 tests**, frontend build).
Eight new tests cover shared selection, clearing old Apply controls, exact target
IDs for comparison/fix/Radar, operation locks, stale/empty states and duplicate
labels. One old Radar fixture was inconsistent (managed list `stream`, update list
`fixture-install`); its two mock lists now refer to the same game, without weakening
the expected Radar target. Headless synthetic previews at 1440x1000 and 390x844
had no horizontal overflow, including expanded long paths; controls remain at
least 44px. Evidence: `target/work/compact-updates-sep22/`. Lead inspected both
screenshots. No native app/game actions, NSIS rebuild, publication or real-game
update were performed. The user still has the earlier diagnostic build installed.

## Opt-in running-app lookup diagnostics — 2026-09-22, package verified

Christopher approved instrumenting the running app's lookup ("Then do that").
Source now has the opt-in Cargo feature `installation-lookup-diagnostics`.
Normal builds do not emit lookup traces. Diagnostic builds show
**Chriz Easy BG — Diagnostics** in the window title and automatically write
`%TEMP%\CEBG-installation-lookup-<pid>-<timestamp>.jsonl` (create-new, max 4 MiB).
No logging failure changes lookup acceptance or permits an update.

The trace captures process/executable SHA-256 and the specific LOCALAPPDATA/cwd,
actual registry directory entries, completed/campaign records and which wins,
original availability, separately labelled rechecks with exact rejected gate/IO
reason, and the final native list. Updates additionally records the native IPC
reply and a bounded frontend acknowledgement of its received IDs/names/states.
It does not copy saves/game content, dump other environment variables, upload
anything, reconstruct records, relax checks, or alter game files.

The engine now exposes `inspect_install_availability` and
`inspect_campaign_availability`; existing availability uses the same unchanged
checks, with reasons retained. Thirteen focused registry tests, six trace tests,
and 19 frontend adapter tests passed. A trace test's Windows separator expectation
was corrected; no safety assertion was removed.

Local job `6b13b22c-40c4-4edf-a231-a92af7d809dd` finished with **exit 0**, verified
at completion: 13 registry tests, 40 command contracts, six trace tests, 215
frontend tests, Clippy/formatting and four package/signature tests passed.
Results: `target/work/alpha19-lookup-diagnostics-sep22/`. Do not rerun the job.
The separately labelled setup in Downloads is ready:
`Chriz Easy BG alpha19 - Lookup Diagnostics-setup.exe` (5,872,109 bytes, plus `.sig`).
Setup SHA-256: `6a260c146873f7d9dd9c25edcd60d46eb28590b1be032a90d2fe681a232cd73b`.
Embedded executable SHA-256 (read directly from NSIS):
`316d570650ce4323223cb48951701d985e5da227c5e18ac9d09faf0e7d2844da`.
The normal Downloads candidate, installed app and public feed were not replaced.

Christopher installed and opened this diagnostic version himself. Its process
trace identified the AppData redirection described above; previous unknown-cause
notes below are historical evidence, not the current diagnosis.

## Unavailable No-SR entry recurred after candidate installation — 2026-09-22

Christopher installed the corrected candidate himself. The sole running app
(`chriz-bg-app.exe`, PID 63628 at the time of inspection) started at 14:55:25;
its executable hash matches the executable embedded in the corrected Downloads
setup: `07fbd0b8613861cc7b151af187e9472cf9b64c3d6769f75a30f85b20e09dd036`.
His 14:56 screenshot still shows No-SR as an unavailable **incomplete** campaign.
Do not ask him to reinstall the game or claim the recurrence is fixed.

Read-only checks found the No-SR root, launcher and completed receipt intact;
receipt SHA-256 still matches its immutable registry record
(`a06b6e0928ee0dc1463247252e4d118c21234e6bdba702ad18a3dd2061b35379`).
Both debug/release engine checks see five completed records and 13 campaign
pointers. A helper linked to the current release **app** library, using the
installed resource directory, projects 13 installations and No-SR as
`Ready to play`; its Update Center also recognizes the completed record.
The running app belongs to Christopher and its process environment has the
expected `LOCALAPPDATA`; its working directory is the installed app folder.

The incomplete display name comes from campaign projection, not from a completed
record merely becoming unavailable. Completed IDs normally suppress their old
campaign entries. No alternate frontend fixture/persisted-record source was found.
The discrepancy between the open window and these checks remains unexplained.
Christopher confirms **Check for updates has already been retried and does not
help**. Do not repeat refresh/restart/reinstall instructions as a substitute for
diagnosis. The test originally failed, but its `supervised_recovery` receipt
(`tempus-v032-20260907`) records completion after the tail repair on September 7;
current WeiDU.log reaches the final BuffBot 1.8.3 components 1 and 0. Its historical
failed campaign pointer is not evidence that the repaired game remains incomplete.
Do not launch/close/control his apps while he is working at the machine.

Diagnostics are local under `target/work/inspect_managed_registry*`,
`inspect_packaged_bridge*`, and `inspect-cebg-process-context.ps1`. The latter
reads only the app's working directory and LOCALAPPDATA; never dump its complete
environment. No game, save, receipt, registry record, or public release changed.
Actual BuffBot Apply/Undo and gameplay acceptance remain pending.

## Overnight update check and source-fingerprint fix — 2026-09-22

Christopher explicitly changed the earlier clicking restriction: "Just try it
yourself now, I am going to bed anyway." Automation is scoped to the No-SR test
below, not the stream installation. Windows UI action-time confirmations still
apply to installing software. No actual update/Undo/gameplay test has occurred.

Reopened the existing alpha.19 app. It now shows 13 installation records and
correctly recognizes `ChrizEasyBG-No-SR-Test`; the prior unavailable status cleared
on restart, but its cause is not established. The collection comparison works.
BuffBot preview failed before mutation: both hardcoded source-tree digests were
wrong for the exact hash-verified published ZIPs. Re-downloading was not a remedy.

Corrected both fingerprints in `engine/src/mod_upgrades/buffbot.rs` without
loosening the file inventory or byte checks. Added an explicit real-archive test:
`published_archives_match_reviewed_sources` (requires the two local archive paths;
no network/game execution). It failed before the fix; all **24 update-module tests**,
including that opt-in test, now pass. Changed and extra source files remain rejected.
All 41 installed source files and 31 copied runtime files on No-SR match 1.8.3;
the loader and backup metadata checks also found no drift in read-only inspection.

Read-only preview failures now have distinct wording: no update was applied and
existing recovery warnings still apply. Apply/Restore errors retain their recovery
guidance. **Six native bridge tests** pass. These fixes are not yet in the installed
alpha.19 app or the September 22 first candidate. No game, save or receipt changed.

The corrected engine's complete read-only preview on No-SR now returns
**available** (1.8.3 → 1.8.4), with 11,898,950,716 game-backup bytes,
1,456,735 profile-backup bytes and sufficient free space. This is eligibility
evidence, not a completed update. Replacement local build runs the real-archive
test before packaging; results belong in
`target/work/alpha19-preview-fix-sep22/`. Do not rerun/poll its watcher.

Replacement job `f78462ce-7d67-4d7d-8fb3-20dc1d3aef79` is now **verified complete,
exit 0**: 24 real-package/update tests, six native bridge tests, 211 UI tests,
Clippy/formatting and four package/signature checks passed. Corrected setup and
signature replaced the old candidate in Downloads after hash checks; see
[current hashes](package-acceptance-alpha19-2026-09-22.md#corrected-candidate--current-download).
The installed app is unchanged. Next step is action-time user confirmation to
install this corrected candidate; the computer-use skill requires it even after
general test authorization. No silent-install workaround, Apply or game launch.

## Old app cleanup and user-driven acceptance — 2026-09-22

Historical restriction, superseded by the overnight authorization above:
Christopher will do **all clicking** for the supervised update test. Do not launch
or control apps/games or apply updates on his behalf. The current running app was
verified as alpha.19 at `%LOCALAPPDATA%\Chriz Easy BG\chriz-bg-app.exe`; actual
BuffBot update/Undo and gameplay acceptance remain pending.

At his request, recycled 34 obsolete files (old downloaded/build installers,
signatures, alpha.1 app binaries and its Start Menu shortcut) and the exact old
alpha.8 smoke app directory. Removed its stale Windows uninstall entry after
exporting a `.reg` backup. Current alpha.19 app/setup/signature and all managed
install/campaign record hashes were unchanged; no game/profile/save was touched.
Evidence: `target/work/old-app-cleanup-20260922/` (local, not for publication).

**Naming migration remains open:** `%LOCALAPPDATA%\Chriz BG Collection` still holds
live registries and locks used by both app and engine. Do not delete/rename it or
run an old uninstaller. Move that state to the CEBG-named data location through a
backed-up, tested migration in a new build, with the app closed. Preserve install
identities, recovery histories and compatibility with existing records. Do not
blindly rename the stable updater identifier, game ownership markers or repo.

## Current-source alpha.19 test package ready — 2026-09-22

The fresh signed NSIS build succeeded, with 211 frontend tests and four package
checks including its exact updater signature. It includes the BuffBot update route
and interval-save fix below. See [package path/hashes and acceptance limits](package-acceptance-alpha19-2026-09-22.md).
No setup/app/game was launched and public alpha.18 remains unchanged.

Proposed supervised target: the already registered
`%USERPROFILE%\Games\ChrizEasyBG-No-SR-Test`, whose completed receipt matches the
registry and whose final two WeiDU rows are BuffBot 1.8.3 components 1 then 0.
Remaining eligibility is untested. Christopher approved continuing but will drive
the app himself; guide his checks and inspect results without clicking for him.

## Interval saves repaired; installer regression covered — 2026-09-22

The current game had CDTweaks 3354 (15 minutes, four rotating slots) installed,
but its isolated profile lacked the required Script settings. EET temporarily
changes `engine_name`, so CDTweaks wrote those settings to the shared EET profile;
CEBG restored its private profile name without initializing the settings there.

With no game process running, added only `IntervalSaveEnabled=1` and
`IntervalSaveCombat=0` to
`%USERPROFILE%\OneDrive\Documents\Chriz Easy BG - ec2c39f8d9c3\Baldur.lua`.
Original backup: `%USERPROFILE%\Games\CEBG-Backups\Baldur-before-interval-save-fix-20260922.lua`.
No saves, mod resources, WeiDU.log or dialog.tlk changed. The settings-menu mod was
**not** added to this existing game. No game launch or timed-save test was performed.

Installer finalization now initializes missing interval-save flags in its reserved
profile, only when WeiDU.log confirms CDTweaks 3350–3357. It preserves explicit
preferences (including disabled saves), and final verification checks the settings.
Automated tests cover EET profile switching, all supported presets, exclusions,
existing preferences, repeat finalization, unsafe files and missing settings.

Christopher approved HGO 38's controls as default for future installations, dependent
on any supported interval-save choice rather than only 3354. Curated defaults are
now **449 components / 50 runs**. This is in the unpublished collection alpha.17
and the September 22 app alpha.19 local package; public alpha.18 does not include
it. Interactive acceptance and publication still wait for Christopher.

## Priority correction: real existing-install updates first — 2026-09-20

Christopher expected collection updates to upgrade the existing installed mod set,
not always create a fresh installation. The current `buildUpdatedCopy` flow and the
narrow description-link hotpatch are not that general capability. Make
[existing-install updates](plans/2026-09-20-existing-install-upgrades-priority.md)
the highest next-feature priority; distinguish app updates, targeted game patches
and collection upgrades clearly. Preserve the user's selected setup and truthful
partial-update history. Do not claim a general upgrade path is implemented.

[UI providers and manual checkpoints](plans/2026-09-20-ui-choices-and-manual-checkpoints.md)
are now parked, with design/source findings and the latest choices preserved.
No further research/implementation of those features now. Live testing of the
approved UI candidates may be left to users. This is the historical September 20
boundary; the September 29 entry above now approves Dragonspear UI++ for the next
release and supersedes its earlier deferral.
Parallel patch-delivery work has advanced beyond some historical entries below;
refresh its current owner/source status before redispatching old blockers.

## First whole-mod update implemented in source — 2026-09-21

The next slice is now implemented: Updates can offer **BuffBot 1.8.3-alpha →
1.8.4-alpha in the same game folder**, with affected/full/optional-save backups,
explicit confirmation, separate version history, Undo and interrupted Restore.
See [scope, checks and remaining acceptance](existing-install-upgrades-2026-09-21.md).
Original receipts and saves stay unchanged. Other whole-mod upgrades are not yet
supported; do not advertise this as Update All or a complete collection upgrade.

Automated fixture/frontend/native checks were run, and both release archives'
hashes/tree fingerprints were verified. **No real WeiDU upgrade, interactive
app/game test or publication was performed for this new route.**
The September 22 local package now includes it; Christopher must be present for
its acceptance test. Public alpha.18 remains untouched.

## Existing-install comparison groundwork — 2026-09-21

The parallel preparation is now partly implemented: Updates can explicitly compare
a registered installation with the bundled collection using its recorded choices,
including exclusions and settings. New optional content stays off; missing choices,
new requirements, changed mod lists and unknown history are reported rather than
silently accepted. This is **read-only groundwork, not a whole-mod updater**.
No Apply action or save-compatibility claim is attached to the comparison.

See [implementation and next steps](existing-install-comparison-2026-09-21.md).
The supported BuffBot transition and durable update/restore history are now in
source as described above. Do not publish the comparison alone as completion of
existing-install upgrades. UI providers and manual checkpoints remain parked.

**Interactive testing waits for Christopher.** Do not launch the app/game, apply
an update, or repeat the earlier disposable-fixture acceptance without his renewed
go-ahead. This turn changed repository source and ran automated fixture tests only.

## alpha.19 local candidate, not published — updated 2026-09-22

Source is at app **0.1.0-alpha.19**, collection **0.1.0-alpha.17** (record corrections
plus the approved interval-save controls default above; mod versions and order
are unchanged). The patch catalog's
`minimum_app_version` 0.1.0-alpha.18 still holds. [Release notes](release-notes-alpha19.md)
are prepared. The September 22 signed local Windows candidate now includes the
latest update-route and interval-save work; it supersedes the earlier September 20
local candidate. Packaged interactive acceptance and publication remain pending.
The earlier missing-key report was incorrect: the existing updater key is present
at the documented developer signing path, and its public companion matches the
app's configured key. No key was created or rotated. Christopher subsequently
stopped unattended interactive testing; the earlier instruction to launch a
disposable Updates fixture is superseded. Arrange acceptance with him before
continuing that step. Keep the published alpha.18 assets/channel unchanged until
publication is authorized.

Issues [#6](https://github.com/Chrizhermann/chriz-bg-collection/issues/6),
[#7](https://github.com/Chrizhermann/chriz-bg-collection/issues/7) and
[#8](https://github.com/Chrizhermann/chriz-bg-collection/issues/8) now have local
fixes: transcribed existing curation approvals, corrected update coverage and
duplicate history, and removed stale fallback metadata. The combined-playtest
tool retains published sources and rejects incompatible explicit local overrides.
No new curation decision or work from Christopher was needed. See the alpha.19
acceptance record for final check/build results when available; do not infer full
gameplay acceptance from these source checks.

## Updates patch flow implemented; packaging still pending — 2026-09-20

Christopher accepted the description-link pilot and loaded-save behavior, then
approved integrating it into Updates. The [production implementation record](existing-install-patches-acceptance-2026-09-20.md)
now covers per-install checks, signed catalog, exact pinned downloads, independent
backup choices, Apply/Undo/Restore, launch blocking during recovery, patch-aware
consistency and diagnostic export. This remains a **narrow description-link fix**,
not newer Assassin mechanics or a whole-mod upgrade. No RC/stream writes or full
installation took place in this implementation turn. Public alpha.18 is unchanged.

Verification: 24 engine patch tests plus a successful real-download/WeiDU
synthetic apply/repeat/Undo test; 15 native unit and 15 diagnostics tests;
179 frontend tests, typecheck and web build. Twenty isolated mock renders across
five viewport sizes had no horizontal overflow or page errors. Native packaged
Updates interaction and public publication are not yet claimed.

**Packaging blocker resolved — 2026-09-20.** The bundled selectable `manifest/`
public-alpha profile had six selected runs without accepted static evidence and
three without tail approvals, while the curated profile carried both. Root cause:
cb07eca added the runs to `manifest/` (the profile *and* the generator base) and
4c9199b recorded evidence only in the generated curated output, so alpha.16, .17
and .18 all shipped a fallback profile that failed validation. Fixed without
weakening anything: a new fallback-profile test covers those runs (303e629), six
accepted static-test records cite it, and Christopher extended his 2026-09-16
curated tail approval to public-alpha (c8f1593). Both profiles now validate with
no findings, the curated generator reconciles by identity and still reproduces the
committed recipe byte for byte, and the 40 command-contract tests pass.

See [alpha.19 status](#alpha19-local-candidate-not-published--updated-2026-09-21) above for the
current release state.

The remaining work is now indexed in [Next work](plans/2026-09-20-next-work-index.md):
more targeted patches, My installs/unique names, customization, randomiser ideas,
challenge mode, companion changes, poison XP, regeneration on rest, website vanilla
differences and deferred community reports. The index distinguishes delivered work
from ideas, and flags ownership/context gaps. Choose the next bounded batch with
Christopher after finishing this delivery; do not redispatch every old plan.

## Existing-install patch pilot — 2026-09-20

Christopher approved the pragmatic three-category design, full-backup recommendation
and no automatic save editing. The requested Opus 5 xhigh design review completed.
The [first pilot](hotpatch-pilot-acceptance-2026-09-20.md) applied the owner-maintained
Artisan description-link adapter to the named old RC, verified six intended table
cell changes/no TLK change, detected repeat application without invoking WeiDU, and
rolled back to the original log/resource hashes. Full game and separate profile
backups remain under the user's Games/CEBG-Backups directory. Twenty focused tests
pass. On Christopher's subsequent request, the same patch was **reapplied in a
second transaction** (`kit-links-native-test-20260920`), and Christopher accepted
its native descriptions and loaded-save behavior. The stream game remains untouched. A separate, backed-up engine-name
switch selects an isolated profile with 786 hash-verified copied files; see the
acceptance record for the launcher, test checklist and restore-first undo caveat.

The subsequent production Updates integration is recorded above; do not repeat
the accepted RC test. The supervised pilot is not itself a public Apply/Undo
release, and no public version/feed was changed. The agent did not launch the game
or edit original saves. See the [accepted plan](plans/2026-09-20-existing-install-patch-pilot.md)
for exact scope and guardrails; do not restart a full installation for this work.

## Alpha.18 published with reviewed Modpack fixes — 2026-09-20

Christopher approved the completed compatibility release. The
[alpha.18 source candidate](alpha18-candidate-2026-09-20.md) now pins Modpack
v0.2.0-alpha.7, with app alpha.18 / collection alpha.16 identities. Recommended
selection is unchanged at 448 components / 50 runs; the actual download/extraction
and focused recipe checks pass. The user's preceding alpha.17 installation has a
successful 50-run receipt; no repeat full install or live-game writes were made.
The background WeiDU fix is included in the published alpha.18 package. Its
[release and verification record](package-acceptance-alpha18-2026-09-20.md) covers
the signed download, tests, public assets and update feed. The alpha.18 setup is
also in Downloads; the website owner has the verified publication handoff.
Use [the release notes](release-notes-alpha18.md) for the included changes.
The wiki and newly proposed Baeloth work remain deferred, not release blockers.

## Next-release triage: Randomiser feedback — 2026-09-18

[Queued report and feature ideas](plans/2026-09-18-randomiser-next-release-triage.md):
unconfirmed Mode 2 / BG-style proficiency conflict; editable RR/EE item pools;
expanded cursed items; per-new-game randomisation; optional combat-focused reward
placement. Christopher explicitly requested recording only, no investigation now.
Discuss scope before promising the larger features for the next release.

## Queued UX follow-up: My installs and unique names — 2026-09-16

Christopher finds alpha.17's My installs view cluttered and its duplicate generic
incomplete-install labels ambiguous. The [cleanup roadmap follow-up](plans/2026-09-07-managed-install-cleanup.md#follow-up-roadmap-my-installs-clarity-and-unique-names--september-16)
records compact layout, persistent unique display names, automatic numbered
defaults, legacy-entry disambiguation and clear post-removal selection. Planned
only; no new app build, game changes or delay to his full-install test.

## Alpha.17: SoD fix, disclosure polish and explicit cleanup — 2026-09-16

The [alpha.17 candidate record](alpha17-candidate-2026-09-16.md) tracks app alpha.17 /
collection alpha.15 with SoD Remix v0.6.11, the previously unbundled download retries,
technical-log/scroll fixes, collapsible framed Customize categories and confirmed
managed-install deletion. Defaults remain 448 components / 50 runs. Static, fixture
and bounded headless UI checks pass; consult that record for package status.

Christopher has **chosen a fresh test install**, superseding the proposed supervised
197 recovery. Do not repair or delete his failed alpha.16 installation automatically.
He will test the full install and deletion later. No new live installation/deletion
test, stream-game modification, public release or updater-channel change is implied.
Automatic repair of sealed partial batches remains a separate feature.

## Incremental download retries and viewer follow-up — 2026-09-16

[Retry implementation](download-retry-acceptance-2026-09-16.md): five attempts with
0/2/10/30-second retry waits, ordinary technical-log notices, unchanged integrity
and resume protections. 47 focused tests pass. Source only; the earlier alpha.16
setup must be rebuilt before this can be advertised or published as included.
Viewer diagnostics 2 confirms Xan recovered and all 34 archives are ready, but
BG1 staging still fails at the Documents save child. The [Documents handoff](handoffs/2026-09-16-documents-folder-robustness.md)
records the evidence and remaining security-history question; root cause not proven.

## Deferred until tomorrow: player poison kill XP — reported 2026-09-16

Christopher reports that enemies killed by poison originating from the player
appear not to award kill experience. **Unconfirmed; do not investigate or fix now.**

Next-session plan: identify the exact poison source and affected installation,
compare a poison finishing tick with a direct-damage kill, and check kill/XP
attribution. Route the confirmed cause to the owning mod repo (owner unknown yet),
then make a focused fix and regression check. Do not assume all poison sources or
the base engine are affected. No game/save changes or agent dispatch requested now.

## Alpha.16 candidate assembled — 2026-09-16

The [candidate record](alpha16-candidate-2026-09-16.md) supersedes the preparation
states below. All owner releases are published and pinned, including modpack alpha.6
with Yeslick188/continuity199/Safana189/Imoen620. Recommended selection is 448
components / 50 runs; dragons remain opt-in. Public-alpha validation passes.
Christopher performs the installation/playtest; do not start another agent-run game
installation. See that record for package verification and publication state.

## Full candidate release first; web docs staged — 2026-09-16

Christopher's current decision is to include every **implemented** candidate in
the next new-install release, subject only to exact source/artifact, selection and
order validity before one full packaged-candidate install. Artisan and Bardic have
the required user acceptance and are releasing through their owners now. SoD Khalid
115 is released in v0.6.10 with bounded entry/briefing acceptance. SR/RR is included;
Lightning 80 is default and 81 is the mutually exclusive optional alternative.
Official Safana plus core 189 are included for Christopher's candidate test; unfinished
Bard/Abettor conversion is not. Yeslick 188 is implemented but awaits its owner
integration before 199/EET_end; it has no existing-save migration. Dragons 110/111
remain opt-in Challenge components.

**Latest wrap-up instruction:** Christopher will run the full installation and
playtest himself. Finish exact package intake, necessary recipe/dependency checks
and the installer build; do not start another automated full install, gameplay
round or broad review. Hotpatch implementation follows the new-install release.

Do not claim the provisional alpha.16/alpha.14 labels or historical 436/44 count as
final, and do not claim that every source is pinned, built, shipped or integration-tested.
The separate [full-candidate and web-docs handoff](handoffs/2026-09-16-full-candidate-and-web-docs.md)
has the exact release-first sequence. The web task may prepare/publish accurate docs
with upcoming changes clearly labelled, but the current download stays in place until
the new release is verified. Existing-game changes come later as opt-in guarded
patches—not generic override copying—and no game write/hotpatch work is authorized now.

## Documents-folder handling requirement — 2026-09-16

Christopher expects moved/OneDrive/missing Documents to be handled automatically
where Windows permits. A viewer's `stage:bg1` / `os error 2` points to the CEBG save
profile, not evidence that C:/D: separation is unsupported. Version/diagnostics are
pending; cause is unconfirmed. Current code resolves redirected Documents and
creates its own child but assumes the parent exists and returns generic I/O errors.
The [bounded hardening handoff](handoffs/2026-09-16-documents-folder-robustness.md)
records creation-at-install, clearer diagnostics, retry and ownership boundaries.
No fix/reproduction/release yet; do not advise deleting caches or disabling OneDrive.

## Yeslick kit gap and Dispel diagnosis — 2026-09-15

Christopher requests both for the next patch. The [owning-mod handoff](handoffs/2026-09-15-yeslick-bg1-kit-and-dispel.md)
records read-only source/save evidence: YeslickNPC 1 kits BG2 Yeslick only;
the current BG1 saved actor is an unkitted Fighter/Cleric. Add BG1 Alaghor coverage
in modpack before continuity 199/EET_end, respecting vanilla opt-out. Do not make
continuity reset player builds. Existing saved actors need a separate repair.

Dispel 410 is already in public modpack and the draft, but absent from the older
stream install. Its SPIN112 still affects allies and bypasses the dispel-level
contest; Combined's spell has the intended hostile-only 1.5x scaling at all 40
headers. No game/save changes or native casting test in this pass.

The extra final SoD bridge/Mislead 257/266 check is **waived as a release gate**
by Christopher, not recorded as passed. TLK changes remain high-risk and require
target-specific validation. Release and existing-install triage plans agree.

## Existing-install patch triage — 2026-09-14

[Patch candidates and remaining tests](plans/2026-09-14-existing-install-update-triage.md)
now complement the release plan. Separate component/resource compatibility from
saved-state applicability; never infer either from WeiDU.log alone. Dedicated
description, Lightning and Abettor tails exist, while full Artisan/Bardic saved-grant
migration does not. Modpack 199 remains new-campaign/pre-EET_end work; 189/620 and
SoD 115 have specific event boundaries. Local TLK appends are possible; copying
another install's TLK/compiled string numbers is not the proposed approach.
Source/planning checks only: no game changes or generic patch executor implemented.

## Expanded next-release plan — 2026-09-13

Christopher wants the next release within the next couple of days, including
implemented mod work whose live tests or owner releases are still pending. The
[next-release scope and acceptance plan](plans/2026-09-13-next-release-scope-and-acceptance.md)
is the current planning entry point. It inventories Artisan kits/modals/descriptions,
Bardic, both SoD work lines (bridge/filler plus Khalid 115), dragons, modpack
continuity/Imoen/Safana, the retained SR Lightning implementation, BuffBot 1.8.4 and
the current CEBG draft. Planned-only work is separated for discussion.

This expands the older narrow release proposal, **not the recipe yet**. Current
draft 436/44 and historical Combined 446/50 are not the final expanded count.
Reuse Combined for applicable native tests; consolidate sources and do one final
frozen packaged-candidate install, not a rebuild for every feature. September 16
resolved the earlier SR/RR, Safana and Lightning choices; the current top-of-file
handoff supersedes this historical discussion. No game/source-pin changes or
collection release were made by this planning pass.

## Challenge Mode recorded for later — 2026-09-13

Christopher requested future Challenge Mode components: SoD changes, stronger
dragons, wand-to-scroll replacements, selected-fight area locks and practical
enforcement of his run rules. The complete 16-rule screenshot inventory, new
requests, SoD filler exceptions, owner routing and unresolved policy details are
in [the Challenge Mode outlook](plans/2026-09-13-challenge-mode-outlook.md).
Documentation/preparation only: do not start implementation, change defaults or
delay the next patch on its account. Subsequent source inventory identifies
implemented optional SoD **257**, the Insane bridge sequencers, as an existing
challenge piece. Native acceptance of its final split is not recorded; Christopher
waived the extra check as a next-release gate on September 15. It does not mean
the full Challenge Mode rules are implemented.

## Acton Balthis no longer default — 2026-09-13

At Christopher's request, BG2 Unfinished Business **25: The Murder of Acton
Balthis** is now optional and unchecked in the draft alpha.16 / collection
alpha.14. Catalog, base manifest/preset and generated recipe agree. No other
component depends on it; all other selections and source pins remain unchanged.
The recommended plan is now **436 components / 44 runs**, retaining CDTweaks 2380.
37 focused tests pass; public-alpha validation has no findings. Opting into this
quest adds exactly `ub-bg2/25`, with no collateral selection changes.
Existing games, historical installation order and published release evidence were
not changed. This remains an unpublished next-patch change.

## Racial kit unlock default approved — 2026-09-12

Draft alpha.16 / collection alpha.14 now includes **CDTweaks 2380**, default-checked
and optional, after the main Artisan/Bardic kit additions. It fills the elf mage
specialist availability gap; internal/NPC-only kits and gnome mage policy stay
unchanged. Recommended plan: **437 components / 44 runs**. Switching it off removes
only `cdtweaks-bg2/2380` and returns 436. 36 focused tests pass; public-alpha
validation has no findings. Native WeiDU 249 fixtures using copied stream and
Combined tables pass install/uninstall with exact restoration; all 66 newly
eligible kit/race choices pass the ability-minimum checks. Completed in-engine
character creation is not yet tested. The pre-existing dwarf/Gallant CHA mismatch
is tracked separately in the community follow-up plan below. No user game/save
changes, packaging or publication.

## Optional choices and community follow-ups — 2026-09-11

Draft alpha.16 / collection alpha.14 now offers **24 additional SCS and 44
CDTweaks choices**, all unchecked. Recommended selections remain exactly unchanged:
**436 components / 44 runs**. Alternative groups have real mutual conflicts;
CDTweaks 2310/2311 remain alongside default 2312 in the late spell-scan run.
SCS 4130/4135/4140 have visible **Not recommended / experimental** cautions after
community feedback. 4130 also explains and enforces SCS's native SR exclusion.
Do not infer that source screening proves every combination works in-game.

35 focused Python tests and 12 Customize UI tests passed; public-alpha validation
has no findings; eight CLI planning cases and pinned SCS/CDTweaks native order were
checked. No game was modified, no installer test was started, and nothing was
packaged/published. Historical release acceptance records were preserved.

Christopher deferred the unresolved **Kivan/Jozzi** viewer case until he can
reproduce it; the existing component 130 guard was already shipped, not a promised
new fix for that case. Druid instant-cure availability is a future SR/BG Rebalance
balance discussion. **Automatic regeneration on rest** is a separate requested
QoL feature, researched but not implemented; owning-mod prototype scope is in
`docs/handoffs/2026-09-11-regeneration-on-rest.md`.

Next-release scope, source-screening caveats, Kivan follow-up, and the explicitly
deferred **website differences-from-vanilla comparison** are tracked in
`docs/plans/2026-09-11-optional-tweaks-and-community-followups.md`. The website list
must be discussed with Christopher/Twitch task after release, not published as an
unreviewed exhaustive claim now.

Also recorded there: Christopher's **elf Mage character-creation screenshot** shows
only Mage, Diviner, Enchanter and Wild Mage. Read-only checks of stream/combined
`K_M_E.2DA` and `mgsrcreq.2da` confirm exactly that list. Selected Artisan 1 unlocks
base classes, not the missing specialist choices. **CDTweaks 2380** is the
kit-unlock counterpart; default inclusion was subsequently approved and implemented
as recorded above. Gnome policy is separate. No game changes; not the earlier
Red Wizard/SR issue.

## Red Wizard default restored after independent review — 2026-09-10

Christopher asked another agent to investigate the Red Wizard/SR exclusion and
restore the kit as default unless a substantiated blocker was found. The review
found no basis for a blanket ban. Artisan NPC **5102** is now default-checked with
or without Spell Revisions, still optional. Other exclusions and source pins are
unchanged. Draft alpha.16 / collection alpha.14 has **436 components / 44 runs**.
This correction is not yet packaged or published.
Verification: 27 focused tests pass, public-alpha recipe validation has no findings,
and CLI plans confirm 436 components with Red Wizard / 435 without, retaining SR.

Evidence, original rule history and actionable mod-repo follow-ups are in
`docs/handoffs/2026-09-10-red-wizard-sr-default.md`. There are narrower starting-spell
and custom-kit spellbook-cleanup imperfections, plus an EET Edwin variant gap;
do not misrepresent them as a demonstrated universal SR incompatibility. Do not
silently enable the old Edwin amulet/slot patches.

Neither the stream nor Combined-20260908 has been modified: both still lack 5102.
The 446-component combined acceptance below does not cover Red Wizard. Focused
native Edwin testing and any existing-save migration remain separate work.

## Kivan quest fix default reconfirmed; already installed — 2026-09-09

Christopher requested default inclusion through the owning task **Fix Kivan Sahaguin
quest** (`01a08666-0eff-77e0-a178-bda4a232ff78`). This is already provided by the
normal hash-pinned `chriz-bg-modpack v0.2.0-alpha.5` dependency, component **130**;
do not add a duplicate standalone installer or select a stale checkout's stub.
The recipe keeps it `mandatory` when BG1NPC quest component **10** and SCS general
AI **6000** are selected, after EET import and SCS (public recipe: post-EET_end).
It is independent of Kivan's Archer choice. The dependency was tightened from
mere BG1NPC mod presence to component 10 after a regression test demonstrated that
turning quests off previously left the guard selected. The default component set
is unchanged; this authored/generated recipe correction is not yet published.

Read-only proof: Combined-20260908 `game/WeiDU.log:408` records modpack 130 once;
no standalone fix entry. Both `override/X#SAHA01.BCS` and `X#SAHA02.BCS` are 225 bytes,
SHA256 `f63e741a2b3c39eab66e34f18b3fa8830022465e2bf54c762b73267c282a45e7`, matching
the handoff's standalone verification exactly. No completed-install files, receipts,
saves, frozen recipes or source games were changed. The stale migration/stub
documentation has been corrected in `COLLECTION_TAIL_FIXES.md`.

Reference-only handoff package:
`%USERPROFILE%\Documents\Codex\2026-09-09\kivan-quest-fix\KIVAN_QUEST_FIX-v1.0-verified.zip`,
verified SHA256 `7fc3769c0c18e32305c1ceefb9727b27a5331edb1165f04c786bb1c0eb035d23`.
Its `verification.json` records four passing installer cases, not new native
playtest acceptance. Do not use this local ZIP as a public dependency.
Test the quest from before the encounter: dialogue should finish before lower SCS
combat AI takes over. This prevents the interruption; it does not resurrect dead
quest actors or repair an already failed saved encounter. No new release authorized.

## Combined playtest completed and retained — 2026-09-09

**Installation accepted: 446/446 components (27 BG1 + 419 BG2).** All 391
previously completed components were preserved; no full reinstall or sibling
uninstall/replay was needed. Corrected SoD 256 and all ten remaining frozen runs
completed, including dragons, Safana 189, Imoen 620, SR 60/80, Klatu 2150 and
BuffBot 1/0. BG Radar Overlay **2.5.2.0** is also installed from official Latest.

Keep `%USERPROFILE%\CEBG-Tests\Combined-20260908` for Christopher's testing.
Launch `game\InfinityLoader.exe`; Radar is
`game\BG Radar Overlay\BG Radar Overlay.exe`. The isolated save folder is
`%USERPROFILE%\OneDrive\Documents\CEBG Combined Playtest 2026-09-08 - c2a832a75741`.
**Native launch/gameplay acceptance is still pending.** This is a local experimental
snapshot, not a public recipe release or a stream-install update. The source and
stream games remain untouched. No installation worker remains active.

Fresh CLI `report <root> --json` returns `ok: true`. The separately published
`.chriz/install-receipt.json` records recovery `sod-256-append-20260909`, version
`local-40712587d45b (repaired)`, exact final component order, source provenance and
attempt evidence. Receipt SHA256:
`51d670ab9fab44ca4fd6d282c4530fb352b1ebe49a6f6e2975585426ce7edcbd`.
The original failed receipt/ledger/frozen recipe were not rewritten; the original
terminal receipt SHA256 remains
`710311d47ccab96da032787f3e0ace6281e7f0a30400f0a9d2084f45b5c86938`.
Keep `.chriz/recoveries/sod-256-append-20260909/` immutable: it is hash-bound evidence.
Do not rerun either installation worker or repair mode. The read-only follow-up
`cebg-overnight-install-follow-up` is now **PAUSED** after verified completion.

Focused recovery tests and example/CLI builds passed, followed by actual retained
installation, acceptance and fresh report verification. The acceptance example
uses canonical forward-slash evidence paths on Windows. Current CLI understands
the additive append-recovery receipt; public GUI compatibility with this local
recovery metadata has not been tested. The SoD owner received the successful 256
installation evidence; native bridge playtest and separate release approval remain
outstanding. Next: use the disposable checkpoints in
`docs/plans/2026-09-08-combined-playtest-and-alpha16.md`.

## Combined playtest completion authorization and execution history — 2026-09-09

Christopher explicitly requested: **"then finish the installation."** Retain
`%USERPROFILE%\CEBG-Tests\Combined-20260908` and all 391 completed components.
Apply only the verified SoD 256 source fix below, append that missing component,
then execute the ten untouched frozen runs through BuffBot (446 total). Do not
rollback/replay the 34 successful SoD siblings or start a new installation.

Recovery executed in `installer-v0-real-alpha`: an incident-specific worker
and a narrowly authorized append-order recovery verifier. The original failure
receipt, ledger and recipe remain immutable; successful completion must publish
a separate recovery receipt with replacement-source and physical-order evidence.
The follow-up was active and read-only during execution; it is now paused.

**04:31 +09:00:** corrected 256 installed successfully, preserving all 364 old BG2
rows byte-for-byte (392 total including BG1). Evidence/backup is under the retained
root's `.chriz/recoveries/sod-256-append-20260909/`. `tools/recover-combined-bridge.ps1`
continues the frozen tail. A helper-only HGO executable-naming no-op was detected
with unchanged before/after logs; its evidence is retained, and the real HGO
operation uses stem `hiddengameplayoptions-bg2-attempt2`. Do not replay 256 or HGO.
The remaining runs and final recovery acceptance subsequently completed (see above).

## SoD 256 fix ready; minimal recovery rehearsal passed — 2026-09-09

The owning SoD task delivered commit `29e123a79b9f03334ab88ce93e28c300b287a8e0`
on `codex/issue-14-bridge-finale` / draft PR22. It fixes the missing review of
Artisan's MO1 equipment-policy pair, preserving the pair after validating its
delivery shape and SPLPROT semantics. It does not disable unknown-effect rejection
or require a Kitpack source change. Only runtime source changed:
`chriz-sod-remix/lib/comp256_creatures.tpa` (worktree SHA256
`6740293f96cb2d3d1c510b29984fc23106609eb3bd2f108f6099ccd9346b6387`;
committed LF blob `b730bae065bc1d47fcadd117eb5eefe1438afb02831c94a6eac38a9a8340eeca`).
The owner reports all 63 focused tests and exact-commit CI green.

Final disposable `fixed-complete` rehearsal with WeiDU249 succeeded: it preserved
all 364 original BG2 canonical log rows in order and appended only 256 as row365;
21 new assets, six intended existing-resource edits, zero removals/uninstalls.
The later installed SoD siblings have no conflicting recorded resource writes.
All 472 protected files and 129 read resources from the retained source were
verified unchanged. The source report and handoff are under the bridge worktree's
`research/data/issue14-fullstack-20260909/`; report SHA256
`b13efb10c573df8d6234b8c26f65d3d4a8d8236d670dd21def130cfd52ff09fd` was checked here.

This supports preserving **all 391 completed components** with a component-only
repair. It is not yet an accepted recovery of the retained installation: the
collection still needs narrow missing-component/source/order reconciliation and
evidence before continuing the untouched tail. Do not use blind Resume or alter
the original failed receipt. Its hash remains
`710311d47ccab96da032787f3e0ace6281e7f0a30400f0a9d2084f45b5c86938` (rechecked here).
No retained game write, public release, new installation or native playtest was
performed by this handoff. The ready-fix notification has been delivered; the
check-only follow-up is paused pending the recovery decision, and must be
reactivated when a recovery worker starts.

## Combined playtest stopped; targeted recovery assessed — 2026-09-09

The 446-component local test stopped at **01:49 +09:00** in SoD Remix 256. No
installer process remains. Preserve `%USERPROFILE%\CEBG-Tests\Combined-20260908`;
**do not delete it or start another full installation**. Current active rows:
27 BG1 + 364 BG2 = **391**. EET, SCS, modpack continuity and EET_end completed;
the ten later runs (including dragons, Safana/Imoen, Lightning, Klatu and BuffBot)
have not started. Public recipe and stream were not changed.

Cause: `BDOLONEI.CRE` contains Artisan MonkRevision-Mystic's opcode 326 delivery
`C0PR#MO1`; `comp256_creatures.tpa` omitted it from its reviewed donor-effect list.
The failure occurred during preflight and WeiDU reports zero component-256 files
to roll back. Other 34/35 SoD components completed. Root error is around line 10907
of the game's `SETUP-CHRIZ-SOD-REMIX.DEBUG`; retain that and the immutable terminal
receipt `.chriz/attempts/terminal-0000000234-19d855a4d1b269e8/receipt.json`.

Christopher asked whether the installation can be salvaged and to send the problem
to its owner.
The SoD task **Find next roadmap item** (`01a071d8-870a-7b02-bbde-e33d0f0b051f`)
received the exact logs, source commit, cause, focused regression request and
read-only-install boundary. It is working on the source fix; no public release or
game mutation was requested from that task.

Read-only recovery evidence: the SoD before-log matches the previous successful
run's after-hash. Its 330 BG2 active rows are retained exactly; the current log adds
only the 34 successful SoD siblings in relative authored order. The existing
supervised recovery mechanism can retain **all 27 BG1 + 330 pre-SoD BG2 components**
and redo only the SoD run plus the untouched tail. It requires reverse rollback of
those 34 SoD siblings, verification of the original prefix, a corrected 35-component
SoD run, complete tail evidence, and a separate recovery receipt. The existing
operator scripts are incident-specific, so this case still needs its own small
operator adapter; it is not a generic CLI recovery command. Ordinary Resume is
not valid, and no repair has been performed or accepted yet.

A smaller append-only 256 repair is potentially possible but not currently
supported by the recovery verifier. The owner must confirm compatibility after
the already-installed 260–910 siblings, not merely after 255; otherwise do not
invent an out-of-order success receipt or disable exact-order verification. The
owner was asked to clarify its 'never uninstall 255' note for both approaches.

Hourly check-only follow-up `cebg-overnight-install-follow-up` is now ACTIVE for
this fix/test (the old no-SR one-time follow-up had been paused). Stay quiet on this
already-reported failure; notify a ready fix, new failure, required decision or
verified completion. Pause when awaiting user direction or completed; reactivate
the same follow-up whenever an authorized recovery worker is launched.

## Combined playtest launched — 2026-09-08

Christopher confirmed modpack completion and asked us to continue. Bridge/filler
source is ready at `3b34eaee19dcb9043f3b72c77bf5b92adbed05ca`; clean modpack
integration is `85cbc42551ca256b167cfdb1ffc4bd3c22df85e4`. Preserve Sarah 190 and
existing companions; use Safana **189** (191 is reserved), continuity 199 and Imoen
620. The bridge hold and modpack source-integration blocker below are resolved.

The separate ignored `target/combined-playtest-20260908/recipe` is assembled from
seven frozen local sources. It preserves all 435 current recommended components
and adds exactly 11 experimental components, with no duplicates: **446 / 50 runs**.
Authoring validation has zero findings. Focused source/recipe tests pass; generated
archives are deterministic. Public recipe/feed and stream remain unchanged.

One hidden CLI installation started at **2026-09-08 22:54:18 +09:00**, PID **60364**,
managed root `%USERPROFILE%\CEBG-Tests\Combined-20260908`, preset
`chris-recommended`, unique name/save profile `CEBG Combined Playtest 2026-09-08`.
It reuses `C:\CEBG-creator-full-cache`; seven local ZIPs and Evandra's existing
official EXE use the normal verified manual intake. CDTweaks/IWDification archives
were reused from the app cache. Initial source checks passed; the engine created
its managed ledger/frozen recipe and is acquiring/verifying the selected artifacts
at this checkpoint. Completion/native playtest are not yet claimed. **Do not start
another copy or edit this recipe during the run.**

Worker and full stdout/stderr are retained under ignored
`target/combined-playtest-20260908/run/`. Frozen worker SHA256:
`1e251c6af996bd64d039fa0a25ce65b02baae87353629e7756c9f70e7b937318`.
The first launch guard stopped before creating a game because the shared debug
binary had been rebuilt during recipe checks; the final copied binary was
revalidated and then launched. This was not a game-install restart.

Safana core v0.5 is added only as prerequisite for 189, addressing the inventory
gate documented in her curation. No Safana Bard/Abettor preset. Continuity 199
follows companion conversions immediately before EET_end; Safana/189/620 stay
late. SR/RR compatibility is after both core mods and before SCS; SR60's scans
target joinable NPCs, not those hostile RR actors. Lightning80 is after SR60 and
the other spell modifiers, then Klatu and BuffBot last. See the
[updated plan](plans/2026-09-08-combined-playtest-and-alpha16.md) for playtest saves.

## Combined test on hold for SoD bridge — 2026-09-08

Christopher wants to wait for the SoD bridge implementation (issue 14), then test
it together with the other SoD changes in one combined installation. Do not start
that installation until the bridge has a testable source snapshot. Refresh its
commit/component requirements before assembly; this is not a requirement to
publish the mod first. The prepared alpha.16 source remains saved, with no change
to public releases or the stream installation. See the
[combined playtest plan](plans/2026-09-08-combined-playtest-and-alpha16.md).

## Alpha.16 source prepared; combined playtest mapped — 2026-09-08

Proposed release is app alpha.16 / collection alpha.14: existing startup loading
feedback, default-optional Klatu armor QoL, and released Bardic `v2.9c-balance.4`.
435 recommended components / 44 runs; no other curation/pin changes. App version
markers and notices agree. The Bardic ZIP was fetched from its official release
and independently checked (5,177,696 bytes, SHA256
`ca7bb2b70ad50b5b6c0fa59a051e53b90c42d3cc9f98fd187a5c1ed40fc1efa7`).
Draft [release notes](release-notes-alpha16.md) are ready. Public remains app
alpha.15 / collection alpha.13: no new setup package, upload or feed update yet.

Checks: frontend typecheck + 146 tests + production build; 30 Python recipe/credits/
updater tests; 16 native curated/release tests, 8 update-ledger tests and 3 package
contract tests. Rust formatting passes. The package signature test remains unrun
until a new signed setup exists. No fresh game install or new live acceptance.
The draft ledger now covers the actual artifact/run/feature update addresses;
the focused regression reproduced the missing coverage before correction.

The [combined-playtest plan](plans/2026-09-08-combined-playtest-and-alpha16.md) shows
how most implemented gated work can share one fresh SR-on EET install with several
disposable saves. Do not bulldoze a stream clone. Important assembly fix: the old
modpack continuity snapshot reuses Sarah's published component 190 and lacks the
newer 192–198 conversions. Port it onto current modpack, preserve those IDs and
allocate a new continuity ID (199 proposed), integrating 620 deliberately. Split
its required pre-EET_end work from the existing late repairs. Keep SCS and SoD in
their current curated phases. SoD PR21 filler changes have code; bridge issue 14
does not. Mutually exclusive Lightning variants and no-SR Darkbloom are not all
simultaneously testable. No mod repo, stream files or saves changed here.

## Armor-thieving QoL — 2026-09-08 (integrated, not published)

Approved: default-checked optional thief skills in armor, **no added penalties**.
CDTweaks 2100 stays excluded. Klatu 2150 is integrated in draft collection alpha.14,
after armor/kit changes and immediately before BuffBot (still last): 435 recommended
components across 44 runs. No other Klatu component or unrelated mod update was added.
The real WeiDU synthetic install/uninstall fixture passed, as did 28 Python tests,
16 native curation/release tests and Rust formatting. No full reinstall was needed.
Exact scope and limits are [recorded here](plans/2026-09-08-armor-thieving-qol.md).
Published app remains alpha.15 / collection alpha.13; alpha.14 release metadata is
draft, including its provisional timestamp. No live game or save was modified.

The requested quick [mod-repo and patch-candidate audit](plans/2026-09-08-mod-repo-readiness.md)
is complete. Bardic balance.4 is now pinned in draft alpha.14; SR/RR's released tail
patch remains subject to the earlier collection deferral. Artisan's merged work is
not release-approved. Armor QoL and the separate kit-description repair are the
first existing-game patch candidates, not already applied patches.

## Visible startup while detecting games — 2026-09-08 (source only)

The blank window during slow first-launch checks is fixed in source: early HTML
fallback, staged loading text/spinner and retryable startup errors. The recommended
recipe and installed games are unchanged. Frontend checks and a delayed browser
preview pass; include this in the next app patch, not a collection recipe update.
Published app remains alpha.15 / collection alpha.13. See
[cause and acceptance](issues/startup-black-window-2026-09-08.md).

## Alpha.15 published; no-SR installation recovered — 2026-09-07

Christopher approved publishing BG Rebalance v0.3.2, updating the collection pin,
and recovering `%USERPROFILE%\Games\ChrizEasyBG-No-SR-Test` without a full restart.
The original alpha.12 recipe failed only Rebalance 401; its seven successful
siblings form the exact top tail after 372 BG2 components. Use the existing
supervised recovery lineage, not a rewritten ledger or an unsafe Resume bypass.
BG Rebalance v0.3.2 is published and independently verified. The existing no-SR
copy now has all 429 components (BG1 27 / BG2 402), including 401 and BuffBot,
with no full restart or repetition of the earlier 372 BG2 components. Its separate
supervised recovery receipt is published; the historical failure evidence is
unchanged. Launch `game/InfinityLoader.exe`; gameplay acceptance remains pending.
Do not run the recovery stages again. BG Radar Overlay 2.5.0.0 is installed and
verified too; final WeiDU logs remain unchanged. Collection alpha.13/app alpha.15
are published, the public updater feed is verified, and the website task has the
release tuple for deployment. Public setup is also in Downloads. See
[scope and evidence](issues/no-sr-tempus401-2026-09-07.md) and
[package acceptance](patch-acceptance-alpha15-2026-09-07.md).

The earlier Alpha13 test completed and Christopher subsequently reported that its
quick gameplay smoke test looked good. Bardic Wonders' agent captured the needed
compatibility evidence separately. Its separate fix is not included in this patch.
The no-SR two-hour follow-up is paused because its failure was inspected already.

## Overnight installation succeeded and retained — 2026-09-07, 07:57 KST

`%USERPROFILE%\Games\CEBG-Tests\Alpha13-20260907` completed successfully:
43 actual WeiDU runs, all 434 frozen components, BG1 27 / BG2 407 exact final log
rows and hashes verified. SoD Remix's 32 components and BuffBot 1/0 are installed.
No restart/deletion or repetition of the five completed pre-EET runs occurred.
Worker 40540 has exited; final ledger is 236. Do not resume/reinstall this completed
copy. It is retained for Christopher's in-game check; gameplay has not been tested.

The launcher is `game/InfinityLoader.exe`. BG Radar Overlay 2.5.0.0 was also installed
through the existing verified add-on helper and its executable receipt hash checked;
mod logs stayed unchanged. Overnight heartbeat is paused. Desktop app itself still
needs the alpha.14 update from Downloads because unattended setup was blocked.
Do not confuse the receipt's frozen original app alpha.13 label with its unchanged
current collection alpha.12 or the alpha.14 engine used to recover it.

Nonblocking installed-with-warning findings are preserved for later compatibility
work, notably Bardic Wonders 2004 skipping Symphony of the Dark Children because
its finite Abettor controller was unrecognized. Do not silently fix the completed
copy. Exact evidence and acceptance limits: [completion report](overnight-install-complete-2026-09-07.md).

## Public source and release preparation — 2026-09-07

Christopher authorized publishing the existing source repository under MIT for
CEBG-owned code. The publication line integrates the finalized app alpha.14 source
and retains recipe alpha.12 / 434 recommended components. Current source/build instructions
are in [BUILDING.md](BUILDING.md); the [publication record](publication-source-2026-09-07.md)
tracks the public ref, hosted Windows checks and preserved binary/update URLs.

Public source preparation is complete. [Windows CI run 34064618951](https://github.com/Chrizhermann/chriz-bg-collection/actions/runs/34064618951)
passed at `36b59971107fe6ef02f1ed312fb904fe4f82fc93`: 620 normal native tests,
139 frontend tests, 42 Python tests, formatting/Clippy/notices and an unsigned NSIS
build. The downloaded CI setup's checksum and unsigned status were verified.
Issue 2's fresh updater-harness startup failure is fixed by test-only manifest
linkage. Build-readiness fixes preserve the recipe/pins and released alpha.14 bytes;
they include test corrections and one equivalent launcher consistency lint cleanup.

The original source audit is historical. EET excerpt attribution is recorded in
[the attribution review](audits/2026-09-07-eet-attribution.md), with upstream rights
preserved. The dependency notices cover separately licensed dependencies; MIT is
not a relicensing of those components. No third-party game/mod archive belongs here.

The installer task's subsequent shortcut/preferences/recovery work is independent
of this publication line. Do not replace its dirty worktree or the running alpha.13
installation. Publishing source does not publish a new app version or alter the
existing `chriz-easy-bg` downloads/updater. Authenticode enrollment remains separate.

## Alpha.14 published; overnight test continued without restart — 2026-09-07

- App **0.1.0-alpha.14** is [published](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.14).
  Signed public setup/download verified; live alpha update feed points to it.
  Collection **alpha.12**, all 434 recommended components/pins, and user selections
  are unchanged. Build source commit: `5610783590ad49b24101d5d3a6a85b018abadb7e`.
- Christopher's exact `%USERPROFILE%\Games\CEBG-Tests\Alpha13-20260907` installation
  resumed at EET attempt 2 with the new release engine. Ledger 156 proves EET
  completed; EET components 0/100 are recorded. Main mods are now installing.
  Five prior successful runs were preserved. Nothing was deleted or restarted.
- Background CLI worker started as PID 40540; check its current identity/status,
  newest ledger and `target/cebg-overnight/alpha14-resume-20260907/stderr.log` before
  doing anything. Do not start a duplicate worker. Same native registry ID:
  `install-5f63e5d7939d3f509139`. The CLI uses the native app's frozen-campaign engine,
  but native GUI process controls cannot attach to this background worker.
- The old failed alpha.13 app closed normally. Unattended NSIS execution was blocked
  by the tool environment before starting, so the installed app remains alpha.13;
  do not retry that blocked operation by another route. The verified alpha.14 setup
  is in Downloads for Christopher to apply. This does not affect the running patched
  game-install engine. Native updater apply/restart acceptance remains unclaimed.
- Reused heartbeat `cebg-overnight-install-follow-up` checks every 20 minutes,
  staying quiet while progress is normal. On completion verify the receipt, retain
  the successful copy and pause the heartbeat. On failure preserve evidence and use
  bounded diagnosis/safe recovery. Cleanup/restart of this exact test is authorized
  only as a fallback; stream games, source games, saves and shared cache are protected.
- Website alpha.14 deployment is verified live at `/collection`: PR 9, merge
  `a47e11d6a170ad09a6b95cf28cddc5ea64bc3025`, all 76 site tests and public
  version/hash/download checks passed. No Discord/email was posted.
  The separate source audit task reports the source repository is now public and
  `origin/main` includes build source `5610783` plus audited docs/CI. Do not override
  its work or force-push. Remaining cleanup UI is documented, deferred, not implemented.

See [patch acceptance](patch-acceptance-alpha14-2026-09-07.md),
[incident](issues/alpha13-eet-pre-spawn-2026-09-07.md), and
[cleanup roadmap](plans/2026-09-07-managed-install-cleanup.md).

## Launcher friction patch + current EET recovery incident — 2026-09-07

**Overnight update:** the pre-spawn recovery/process fixes pass focused tests and
Christopher's exact `CEBG-Tests/Alpha13-20260907` copy resumed at EET attempt 2,
ledger 155, at 06:37 KST. Five previous mod runs were retained. Background worker
PID at launch is 40540; logs: `target/cebg-overnight/alpha14-resume-20260907/`.
Do not launch another worker or restart the stack. App alpha.14 is now published;
recipe alpha.12 is unchanged. Full install completion is not yet established.
Christopher authorized releasing a fix and only using cleanup/restart if recovery
cannot safely work. Cleanup UI is deferred: [roadmap](plans/2026-09-07-managed-install-cleanup.md).

Source-only launcher patch: separate Play shortcut names with safe collision handling,
unfinished setup preferences restored with fresh checks, and visible native error
details. Windows shortcut tests 7, native command contracts 39, frontend tests 139,
TypeScript and Vite build pass. These changes are included in alpha.14 without
changing the recipe. See [scope/acceptance](plans/2026-09-07-launcher-friction-patch.md).

Christopher's live alpha.13 test stopped before invoking EET: the process check saw
`game/EET/bin/win32/x86_64/weidu.exe`. The five preceding mod runs succeeded. That
process was absent during inspection; EET has only a before-log, identical to current
WeiDU.log. Alpha.13 mishandles this never-spawned evidence shape, so he was
asked to preserve the folder and hold off on Retry in that old app. Process-liveness
and pre-spawn recovery fixes are complete in the alpha.14 source. Do not restart the entire
installation or claim other agents caused the process match without evidence.
See [incident/evidence](issues/alpha13-eet-pre-spawn-2026-09-07.md).

## EET Documents-path and quiet-install fixes — 2026-09-07 (app alpha.13)

The community EET failure is an upstream Windows path parser truncating a spaced
Documents registry value, not a timeout or missing first game launch. A narrowly
hash-checked staged-file correction runs before EET core, preserving original
downloads/cache and recording exported per-attempt compatibility evidence.
Focused tests and a real verified WeiDU249 `--nogame` macro test pass.

The separate quiet-install notice now waits five minutes, clears on resumed output/
next successful step, truthfully says the process is still running and offers
**Keep waiting**. Frontend 111, Python 35 and focused native/engine checks pass.
App alpha.13 is publicly published, with signed package/public-download verification
and the live alpha Updates feed checked. Recipe alpha.12 and all 434 recommended
choices/pins remain unchanged. The website owner has the verified download and is
updating `/collection`; website deployment is not yet confirmed here.
See [acceptance](patch-acceptance-alpha13-2026-09-07.md)
and [release notes](releases/0.1.0-alpha.13.md).

Christopher chose that the affected tester can **start new with the fixed version**.
Do not spend more time on manual recovery instructions: the old frozen recipe also
contains SoD0.6.7, whereas the current recipe includes corrected SoD0.6.8. No game
or tester folder was modified or restarted by this task; no full install is queued.

## Installation-folder hotfix — 2026-09-07 (app alpha.12)

A community user had to manually create the default user Games directory. The exact
engine bug was reproduced: startup required the immediate install parent to exist,
although read-only path inspection accepted a new nested path. Startup now creates
missing ordinary ancestors and exclusively claims the final root, retaining reparse,
occupied-folder and other safety checks. Both location views explain automatic folder
creation, and the main Browse control remains available after a rejected location.

App alpha.12 is published and signature/download/tamper-checked, with all 110 frontend,
27 orchestrator, 19 preflight, 39 native command-contract, 3 package and 35 Python
checks passing. The recipe is unchanged at alpha.12: the same 434 recommended
components/source pins. No game install or native-window control was used.
See [folder-hotfix acceptance](patch-acceptance-alpha12-2026-09-07.md) and
[release notes](releases/0.1.0-alpha.12.md). The [public release](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.12)
and mutable alpha updater feed passed anonymous URL/hash/signature checks. A verified
setup is in Downloads. The [website](https://bg.chrizfader.org/collection) update is
deployed: site PR 5, merge `2f13d9196a0efb47f5c84fe97a227c2f15adbd85`. Owner confirmed
75 passing site tests, builds/CI, live content and an independent public setup hash
match. Full website handoff evidence is in the acceptance note above.

## Community SoD 900 fix — 2026-09-06 (published alpha.11)

The supplied diagnostics confirm an overly strict SoD Remix v0.6.7 chest guard,
not a Steam-version defect or user error. Component 900 rejected three existing
items, rolled back zero files, and WeiDU then installed 910. Earlier 310 active
components remain unchanged; BuffBot was not reached. Preserve the failed folder.
See [recovery assessment](issues/sod900-community-recovery-2026-09-06.md) and
[patch acceptance](patch-acceptance-alpha11-2026-09-06.md).

SoD Remix v0.6.8 is publicly released and freshly verified; CEBG now pins it without
changing the approved 434-component recommended selection. App alpha.11 / recipe
alpha.12 signed package is published and passed signature/download/tamper checks,
including a fresh anonymous public download. The mutable alpha feed now offers it.
[Public release](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.11)
and local Downloads copy are ready; the website owner is deploying the page update.
This patch includes the clarity/support work below and syntax-safe JSON diagnostic
redaction. It replaces misleading mandatory-restart wording but does not add an
unsafe Continue button: automatic targeted recovery is still private issue 3.
A supervised repair can target the SoD run and six remaining runs, without replaying
the historical 310 components, once the unchanged local evidence is available.
No game/save was modified and no full installation was restarted.

## Post-alpha clarity/support patch — 2026-09-06 (included in alpha.11)

Christopher approved a lightweight clarity/dependency pass, grouped alternatives,
installed-mod visibility, diagnostics from completed installs and first-play help.
Source implementation is in the `installer-v0-real-alpha` worktree; public app
alpha.10 / recipe alpha.11 is superseded by the published patch above; all game installations remain untouched.
See [the section-by-section audit](clarity-audit-2026-09-06.md).

- All 26 categories / 499 choices retain source/group context; ambiguous identification,
  stronghold and stacking options have concrete explanations. Repeated description text
  is suppressed, not expanded into boilerplate. Existing charcoal/brass styling stays.
- 33 explicit alternative groups use compact selectors with None/unchanged. Defaults
  remain optional, initially checked. Atomic switching only replaces group siblings;
  ordinary readiness, parent, dependency and external-conflict checks still apply.
  Three legacy BG1 NPC groups had inconsistent parents/conflicts; the authored curation
  establishes their exclusivity and those constraints are now consistent.
- My installs has a searchable receipt/current-WeiDU list, reported versions and
  missing/extra entries; it does not claim resource-byte or in-game activation checks.
  Diagnostics can be exported after restart for known installs with a terminal receipt.
  Exports stay local and warn about paths/logs before sharing. First-play tips collapse.
- Diagnostic ZIPs now start with a readable outcome/version/run-evidence summary.
  Empty BuffBot `attempts` means no finalized run record in that receipt, not proof
  that WeiDU never started or that the tester made a mistake. Raw evidence and later
  resume history must be checked. A crash before the first terminal receipt still
  prevents ZIP export; preserve the install folder and its `.chriz` evidence.
  See [diagnostic interpretation and verification](diagnostics-reading-2026-09-06.md).
- Christopher will run the install himself to check BuffBot. The community cause is
  established by its diagnostics above; do not run another full install or modify r5/
  the stream installation merely to reproduce this already confirmed failure.

Release integration: these additive presentation fields require the alpha.11 app binary;
old alpha.10 strictly rejects them. App/recipe versions are bumped together and the
new recipe ledger requires app alpha.11. Do not
publish this as a recipe-only update to alpha.10 or overwrite its immutable artifacts.
Native updater apply/restart and pause-close acceptance remain the previously recorded
follow-ups, not completed by the automated/package checks.

## Public alpha publication checkpoint — 2026-09-06

**Published and complete:** https://bg.chrizfader.org/collection now serves the
alpha download/guide/overview/roadmap/credits. Website owner verified production
merge `0cda80e65a0a9c0a5a2cb1b7a8b2b579ab8f92ca`, Cloudflare build
`7e3a258a-3af7-4593-a92f-a1ecaba12552`, 57 passing site tests, successful GitHub/
Workers builds, live page/content/header/route checks and an independent installer
hash match. Public setup and updater feed also passed this task's anonymous hash/
signature checks. See [publication evidence](publication-acceptance-2026-09-06.md).
No further full install or review loop is queued. Tomorrow's follow-ups are native
updater apply/restart and pause-close acceptance, public alpha reports, and the
already-approved post-alpha customization improvements. The fresh updater mock
harness's loader failure is private issue2, not a demonstrated app startup failure.

Christopher explicitly authorized publication after announcing the forthcoming
download on Discord. App `0.1.0-alpha.10` / recipe `0.1.0-alpha.11` is the candidate.
The separate public distribution repo `Chrizhermann/chriz-easy-bg` now exists;
**this collection source repository remains private**. Website task
`Build interactive BG run page` owns `/collection`, quick guide at top, overview,
roadmap, credits and deployment, gated on the exact verified public binary URL/hash.

Final source intake includes official Windows Evandra download-or-skip and SoD
Remix `v0.6.7`/component910. Christopher accepted the six-person skip Yes route;
the owner released commit `a6826ca1452cbe500c5e111f117b56a983a55d0d` with 82 Windows/
Linux tests. The first recipe's 31 SoD components now include 910 as the 32nd;
291/901 remain excluded. The early skip prompt is default inside the SoD toggle.
No full installation was restarted. R5 and all game/save sources remain untouched.

Frontend typecheck/all101 tests, 23 Python curation/generator tests, native command
contracts39, focused recipe/archive/materialization/validation tests passed.
RAR extraction safety review issues were fixed and rechecked; all183 Evandra files
match the accepted reference. Public bundle resources now exclude creator/live
reference TSVs and authoring inventories, and include application/UnRAR notices.
Signed packaging and public download/feed verification are next; do not claim
native updater apply/restart, close/pause interaction or another end-to-end install.
See `docs/releases/0.1.0-alpha.10.md` for the player-facing scope/limits.

## What this is

The umbrella/orchestrator for the whole modded-BG stack: manifest + install order + presets
+ (future) install driver. It composes the chriz-* WeiDU mod repos with 80-ish third-party
mods **without redistributing them**. Architecture + rationale: chriz-bg-rebalance
`docs/plans/2026-07-03-umbrella-analysis.md` (user-approved 2026-07-03).

## Current priority — curation-derived recipe, not historical replay (2026-09-05)

**Latest approvals (2026-09-06):** [release intake and website coordination](plans/2026-09-06-release-intake-and-website-roadmap.md).
Christopher reports r5 gameplay without crashes/issues. Include released utility
XP610, Yoshimo/Hexxat choices220-223 and SoDv0.6.6 ending290 in the next recipe;
Hexxat's approved default is Shadowdancer221;222/223 remain alternatives. Imoen620 and dragon
work remain WIP. FullSoDskip is reportedly nearly ready: owning-task readiness
refresh requested before intake. Website task owns roadmap aggregation; later
UI-mod selection includes compatibility checks. Recipealpha10 source integration
and focused verification are complete; see [intake evidence](recipe-intake-2026-09-06.md).
Safe pause/app-close is implemented with focused automated checks;
native interaction and real WeiDU pause acceptance remain pending. These changes
were packaged and signed locally as app alpha.9 / recipe alpha.10; see
[package evidence](package-acceptance-2026-09-06.md). R5 is not being modified or rebuilt.
Christopher approved friendlier exclusive-choice dropdown/radio controls for a
post-alpha patch, not the initial release. Current conflict blocking/reasons remain.

**Evandra public acquisition:** automatic official download was approved, but the
ordinary HTTP probe received a Cloudflare browser challenge. Christopher then
approved an upfront download-or-skip exception and supplied official Windows
`evandra-v2.2.exe`. Recipe alpha.11 now pins its exact size/hash rather than the
private aggregate; explicit RAR SFX extraction never executes the downloaded EXE.
The early UI gate and native readiness checks are implemented and focused tests
pass; final payload comparison/package integration are underway. See the
[Evandra public acquisition checkpoint](handoffs/2026-09-06-evandra-public-acquisition.md).
Do not bypass the challenge, mirror the mod, or publish the older private contract.

**Latest instruction (2026-09-06 KST): [targeted r5 recovery](targeted-recovery-2026-09-06.md).**
Christopher approved repairing just the failed modpack rather than repeating the full
installation. **R5 has now completed all 430 components through supervised recovery**:
16/16 corrected modpack components and the three remaining commands passed, preserving
the prior 383 active BG2 entries. Final exact-order audit passed. Original managed
failure history remains intact. [Managed recovery completion](managed-recovery-acceptance-2026-09-06.md)
now passed too: separate receipt and registry, final isolated save identity, exact
versions, launcher consistency, CLI report and idempotent publication. Radar 2.5.0.0
is installed and its second current-release check reused the installation. Next is
current native package/launcher, game startup/save-reload and app-update apply/restart
acceptance, not another mod installation.
R6 was deliberately stopped before mod installation. Its cleanup was tool-rejected;
do not retry deletion by another route, resume it or start a full copy. The recovered
r5 retains earlier Bardic balance.2 and all approved selections. No game smoke yet.
The status below is the preceding r6
snapshot and is superseded by the targeted recovery record.

### Historical r6 context — superseded by completed r5 recovery above

The user has now requested the complete corrected installation/test/cleanup flow. Continue
from [the current acceptance run](curated-full-acceptance-2026-09-05.md), not the invalid
overnight replay. Recipe reconstruction and source/order verification run in parallel.

Start with [the approved reconciliation path](plans/2026-09-05-curation-reconciliation.md).
The overnight `creator-full-current` recipe bypassed recorded curation; it is invalid as
the curated collection. **Do not repair Bristlelick or resume that selection.** Curation
files are intact. Restore them as the authority, preserve deferred mod work, check changed
ordering against mod documentation/source. **Fresh r6 is running** at
`%USERPROFILE%\CEBG-Tests\CEBG-Curated-20260906-r6` with recipe **alpha.9**.
Released modpack **alpha.5** fixes both r5 failures (170 Xan / 192 Viconia), with
public-component installation, repeat and uninstall checks on the actual captured r5
input resources. Bardic Wonders **balance.3** is also pinned. Both packages were
independently downloaded, hash/layout verified and extracted by the author verifier.
The resolved **43 runs / 430 components have identical selections, order and arguments**;
all 30 approved SoD components remain, Aura and 290 remain deferred. No new companion
or utility-XP defaults were invented. See the [current acceptance record](curated-full-acceptance-2026-09-05.md)
for the exact worker, startup retry and checkpoints. Full integration is not yet proven.

R5 remains read-only terminal evidence at `%USERPROFILE%\Games\CEBG-Curated-20260905-r5`,
installation `install-e6325c7c98451ad4901e`. Ledger 222 is `fresh_copy_required`;
14 other modpack components succeeded and the previous 383 WeiDU rows were unchanged.
The [resolved owning-repo handoff](handoffs/2026-09-06-modpack-xan-viconia.md) retains
the original diagnosis. Do not resume r5 or change its recipe/seal. SoD and BG Rebalance passed.
SoD v0.6.5 is published, independently downloaded/hash-verified, and pinned. Its mod bytes
match the candidate Christopher accepted in focused gameplay/save-reload testing. See the
[owning-repo handoff](handoffs/2026-09-05-sod-remix-component-120.md).

The prior native-order and conditional Randomiser-answer failures are fixed. R4 then hit
a recipe TP2-alias mismatch after all 30 SoD components succeeded. The nested canonical
path is corrected with regression coverage; no component choices changed. Failed r3/r4
remain immutable and must not resume. App alpha.8 is built and installed locally and fixes
the misleading Retry option when a new copy is required. Radar Latest 2.5.0.0 download,
layout and seven engine tests passed; add it only to a successfully completed copy. The
[current acceptance record](curated-full-acceptance-2026-09-05.md) has the exact worker,
frozen hash, logs, remaining launch/update acceptance and cleanup boundaries. Full fresh
collection acceptance is **not complete**. Older packages/status below are historical;
there is still no public CEBG release/update channel. Installer acceptance is the priority.

Christopher's [short alpha-release path](plans/2026-09-05-alpha-release-short-path.md)
avoids another whole install merely to start it through a different entrypoint. Source
update notifications and the post-app-update/new-setup action are now visually tested
([evidence](updates-ui-acceptance-2026-09-05.md), 80 frontend tests); the installed alpha.8
predates this slice. Next signed candidate must include it. Public Evandra acquisition
needs the verified original standalone package, not the creator's private aggregate ZIP.
The explicit seven-copy cleanup request was tool-rejected too; see the acceptance record.
Christopher is now deleting those copies manually. Do not race his cleanup or assume each
folder still exists. No agent workaround deletion is allowed; the agent removed nothing.

Christopher requested a separate destination for future tests. Use
`%USERPROFILE%\CEBG-Tests\<unique-run-name>` for NEW agent-owned disposable installations,
not the user's normal Games folder or any folder directly under C:. Christopher rejected
the earlier root-level proposal; it was not created or used. Keep frozen r5 where it is; no move,
rename or restart is implied. Public/player install defaults are unchanged. This separation
is for clear ownership and cleanup, not a workaround or guarantee about deletion policy.
Preserve small diagnostics and retire failed full copies when permitted; surface a blocked
cleanup backlog before starting additional large test copies.

Christopher also permits today's finished mod work in the first alpha. The
[cross-repo intake queue](plans/2026-09-05-first-alpha-mod-intake.md) records two
released modpack/Bardic updates now included in alpha.9, utility XP 610 awaiting
selection policy, unselected Yoshimo/Hexxat additions, and remaining WIP. New choices
belong in a later versioned candidate; do not mutate r6, invent defaults, or let WIP
become an installer blocker.

Latest queued requirements are captured in [recovery and casual customization](plans/2026-09-05-recovery-and-casual-customization.md).
Dual update notification is implemented in source. **Safe pause/app-close handling is
missing and is the next installer priority**: Cancel currently kills WeiDU and must not be
advertised as Pause. A read-only recovery matrix and bounded test plan are documented.
The [failure-detail slice](failure-ui-acceptance-2026-09-06.md) is now implemented
in source: retain the actual report reason, identify safely attributable missing
component log rows, keep fresh-copy recovery strict, and do not announce empty
progress as complete. 91 frontend / 9 focused CLI tests and responsive checks passed;
not in installed alpha.8. The fresh r6 CLI binary includes these error-detail improvements.
The requested [common Customize controls](customization-acceptance-2026-09-06.md) are now
implemented in source: six bundles, category bulk actions, preserved preferences and
visible collateral, with 88 frontend / 16 recipe-view / 4 authoring tests passing.
Headless real-catalog layout checks passed. Not in the installed alpha.8 yet; authoring
metadata is now in alpha.9 with no recommended-selection delta. Yeslick's original-class
route currently also excludes combined modpack dispel fix 410; owning-repo follow-up is
recorded. Typed advanced inputs and broader Customize QoL remain; no default curation changed.
Optional confirmed SoD skip with +250,000 protagonist-only XP was dispatched to
the owning SoD task; separate from component 290 and frozen r4, not a first-alpha blocker.
Its native ground-pile probe subsequently failed (owning commit 19e221b); that prototype
must not enter the recipe. Released SoD v0.6.5 is unaffected.

## Historical overnight continuation — CEBG app alpha.3 / recipe alpha.2 (2026-09-05)

Start with [`overnight-2026-09-05.md`](overnight-2026-09-05.md): current code changes,
real test paths, cleanup ownership, release boundary and remaining acceptance. The active
worktree is `installer-v0-real-alpha`, **not** `4d39`. Both Steam source games are now
verified clean. The dotted-artifact freeze/evidence failures are fixed. A compact UI,
full-creator recipe profile, Radar add-on, human receipt versions, lazy mod-list consistency
and signed-app updater are implemented; real full-install acceptance is still in progress.
The first overnight follow-up fixed EET's Windows staged-path argument; EET completed.
The second follow-up stopped on malformed Bristlelick source; the later curation audit
supersedes the proposed repair because Bristlelick was excluded. See the current priority
above. Signed app alpha.3 is built locally but its historical full profile must not be used.
No public release/channel has been published. The old status below is historical.

## Earlier Chriz Easy BG 0.1 alpha status (2026-09-05)

Branch `codex/installer-v0-real-alpha` is pushed. The installer engine and UI are
functional, and the public-alpha recipe validates and resolves **35 runs**. All **30
selected artifacts** have passed cold-cache acquisition, extraction, and payload
verification. Notable ready pins include public CHRIZ-BG-MODPACK `0.2.0-alpha.1`,
CHRIZ-SOD-REMIX `0.6.4`, Artisan's Kitpack `chriz-v1.3.1`, Spell Revisions
`v4.21-chriz.3`, and public CHRIZ-BG-REBALANCE `v0.3.1` with its ten-component curated
fresh selection.

Tasks 19, 20, and 23 are implemented: public-alpha omissions and evidence are
release-enforced; immutable recipe envelopes, update classification, and packaging are in
place; and the Tauri command surface includes restart-safe discovery and resume through the
immutable campaign index. Task 24 has a tested three-track update-center foundation, but
the production updater channel is deliberately still unconfigured.

The player-facing app is now **Chriz Easy BG (CEBG)**. With no registered installation it
opens directly on one compact install screen with detected sources, editable install name
and location, recommended choices, a clear readiness state, and one primary Install action.
With an existing installation it opens as a launcher with Play, Open game folder, install
switching, collapsed technical paths, and recovery for resumable or moved installs. The
optional CEBG desktop shortcut is checked by default and points back to the registry-verifying
launcher rather than directly to the game. Startup remains registry-first, so a completed
game can be launched even when its original source installs are unavailable.

Fresh verification after this UX slice passes 54 frontend tests, 37 native app tests, full
workspace tests and Clippy, and public-alpha validation with zero findings. Responsive checks
pass at 1920x1080, 1366x768, 768x1024, and 375x812: desktop keeps the primary action visible
without a scrollbar, and constrained screens use one column with normal internal scrolling.
The final local NSIS lifecycle smoke passed for the **4,413,788 byte**
`Chriz Easy BG_0.1.0-alpha.1_x64-setup.exe` with SHA-256
`7BF879133A0E49E67A811F85BCAFF98FC61AFA34B32207314F51375D4F42F227`: its branding and
version were correct, all 78 bundled manifest files hash-matched, the installed app stayed
alive for five seconds, and silent uninstall removed the isolated smoke directory.

There is **no full game-install acceptance and no public release yet**. The immediate E2E
blocker is a genuinely clean Steam BG:EE+SoD 2.7.3 source; the detected Steam BG2 source is
clean, while the available BG1 source is rejected for mod residue. These checks were
read-only and no game directory was modified. Production Minisign and Tauri updater keys
also remain to be provisioned, and Task 24's signed updater UI remains pending.

Immediate next actions only:

1. Obtain or restore a genuinely clean Steam BG:EE+SoD 2.7.3 source.
2. Run the first full installer-driven clean EET build and focused smoke before the public
   installer release.
3. Provision the production signing keys/channel and publish the first explicitly alpha
   build only after those gates pass.

Other blocked items remain later work and are not expanded here. Older status sections
below are retained as historical implementation context.

## Status (2026-09-02 — curation snapshot integrated; real-alpha implementation active)

The authoritative component-catalog snapshot from the dirty `main` checkout is preserved
in commit `1eedd8e` and integrated here without modifying that checkout. The normalized
decision semantics and per-component notes live under `docs/curation/components/`. Start
with [`docs/next-session.md`](next-session.md), use
[`FOLLOW_UPS.md`](curation/components/FOLLOW_UPS.md) for the complete evidence-backed queue,
and use [`COLLECTION_TAIL_FIXES.md`](curation/components/COLLECTION_TAIL_FIXES.md) for the
22-fix migration inventory. Explicit choices and source/release/acceptance gates remain;
do not treat every `default` row as currently installable.

The active implementation branch is `codex/installer-v0-real-alpha`. The approved current
milestone is defined by `docs/plans/2026-09-02-installer-v0-real-alpha-design.md` (`ae5855d`)
and `docs/plans/2026-09-02-installer-v0-real-alpha-implementation.md` plus its separate
27-task ledger (`fd73921`). This supersedes the fixture-only Recipe Preview as the first
release milestone. The new plan explicitly selects `ureq`'s `rustls`,
`platform-verifier`, and `win-system-proxy` features.

**Current RC status (2026-09-03):** `C:\BG-EET-RC-20260902` has been played for an
extended session without a crash, and BuffBot works. It is not suitable as the public
alpha: Item Randomiser v8.1's physical CRE-item removal left invalid inventory offsets
and references in 95 post-Randomiser creature resources, which can duplicate or omit loot
and equipment. There is no evidence of a crash or save-file corruption. Released v8.1.1
fixes the source defect; a clean rebuilt RC, new game, and Tarnesh loot smoke remain open.

## Engine Phase 1 baseline

The branch includes the `feat/engine-phase1` lineage through Task 6. Historical plan =
`docs/plans/2026-08-20-engine-phase1-implementation.md` plus its status ledger. Crate
`engine/` provides lib `bg_engine` and bin `chriz-bg-install`.

| Task | State | Notes |
|---|---|---|
| 0 scaffold, 1 schema types (`manifest.rs`) | done, reviewed | |
| 2 loader (`loader.rs`) | done, independently reviewed | original `a4cdb6a`; fixes `05fe9f7` + `05b8058`; 16 loader tests |
| 3 validators (`validate.rs`, 9 rules) | done, independently reviewed | original `5feb0a8` (merged `65e3bb8`); fix `a0c812b`; coverage `71f0387`; 41 tests |
| 4 resolve (`resolve.rs`) | done, independently reviewed | original `13df297` (merged `98c5515`); fix `4c61a8c`; 16 tests |
| 5 events (`events.rs`) | done, reviewed | |
| 6 snapshot session persistence (`session.rs`) | done | `39a6692`; nine focused tests |
| 10 fake-game builder (`tests/support/fakegame.rs`) | done, reviewed | KEY/BIF/TLK writer |
| 7 WeiDU invocation, 8 log-diff verify, 9 runner, 11–16 | not started | historical Phase-1 ledger only |

The real-alpha plan is authoritative for next-work order. Its Task 3 intentionally replaces
the Phase-1 Task 6 snapshot model with a create-once, append-only hash-chained campaign
ledger; do not confuse those two separately numbered tasks.

**Task 2 fix-up completed locally (2026-09-01):** schema is probed before strict parsing;
stem mismatch wins over duplicate-id defence; `.toml` matching is case-insensitive; file
symlinks are followed and broken ones return pathful I/O errors; roots are canonicalized;
non-UTF-8 stems have an explicit error; `Manifest::conventional_mod_path` clearly documents
that it is not an actual loaded-file lookup; fixture mods are copied as a directory. Follow-up
tests cover a real duplicate id on case-sensitive filesystems and case-variant extensions;
Windows symlink tests skip only unsupported/permission-denied link creation. Two independent
reviews found no blocking issues; their bounded follow-up requests are now covered. The
focused RED run failed in all five intended cases before implementation.

**Task 3 review completed locally (2026-09-01):** manual sources no longer inherit the
non-manual HTTPS/hash rule; hexadecimal hashes accept either case; one explicit order slot
cannot repeat a component; split `eet_end` entries must form the final contiguous main-phase
block; and aggregation is covered with two independent errors. The four intended regressions
failed before implementation and pass after `a0c812b`.

**Task 4 review completed locally (2026-09-01):** a selected choice may add a component to
the only explicit slot of an unsplit mod even when that slot did not list it originally.
Split mods still require unambiguous explicit placement. The regression failed before the
fix and passes after `4c61a8c`.

**Historical installer v0 preview:** `docs/plans/2026-09-01-installer-v0-design.md` and its
implementation plan established the guided wizard and engine-owned UI boundaries. The
static prototype at `docs/prototypes/installer-v0/index.html` remains a no-op Recipe Preview;
it does not download, copy, or install anything and is not the current release milestone.

**How to build (Windows):** Rust 1.97 stable-msvc via rustup; run cargo from **PowerShell**
with `$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"` — Git Bash's coreutils `link.exe`
shadows the MSVC linker. Work in an isolated worktree; never develop in the dirty primary
checkout or either protected game/archive directory. Repo files are CRLF.

**Conventions:** TDD per task (failing test → implement → green → commit "engine: …");
`#[serde(deny_unknown_fields)]` on all manifest structs; errors carry `PathBuf`s; no
`unwrap`/`expect` in library code; doc comments on public items; never write under
`C:\Games\…` (read-only reference); never hand-edit `manifest/install-order.tsv`;
curation content is Chris-only — the engine is curation-independent.

## Background (2026-08-19 — installer app designed)

- **Installer-app foundation:** `docs/plans/2026-08-19-installer-app-design.md` established
  the public EET-only, copy-then-install architecture with Tauri 2 and a headless Rust
  engine. The 2026-09-02 real-alpha design linked above is the current extension.
- `manifest/install-order.tsv` — recaptured by `fbd914b` from the reference WeiDU.log:
  **451 entries / 91 mods**. This resolved the earlier 414-vs-364 discrepancy; the 364
  figure was wrong. Never regenerate it except from the live reference install.
- `manifest/mod-sources.tsv` — 89 source rows are classified. The integrated curation
  snapshot points historical local fixes at the migration inventory, but several immutable
  pins and release artifacts still need refreshing before recipe freeze.
- `presets/` and `app/` are not started. The headless `engine/` contains the Phase-1
  baseline; the current real-alpha plan owns the remaining engine, UI, and release work.
- Parked: [#1 EET XP scaling fix](https://github.com/Chrizhermann/chriz-bg-collection/issues/1)
  (future chriz-layer component; home repo TBD).

## Hard guardrails (user directives)

1. **The game folder (`C:\Games\Baldur's Gate II Enhanced Edition modded\`) is a READ-ONLY
   reference.** Read WeiDU.log / EET_MODDING_GUIDE.md / mod folders freely; never write,
   install, or test there.
2. **Never redistribute third-party mods.** Private archiving of hard-to-find zips is
   acceptable in a private repo, but the public-facing design is links + versions. If the
   repo ever goes public, archived third-party content must be dropped/licensing-reviewed.
3. `gh` CLI auth is shared across concurrent agent sessions — `gh auth status` before any
   gh op; this repo needs `Chrizhermann`.

## Work queue

1. Follow the dependency graph and delivery waves in
   `docs/plans/2026-09-02-installer-v0-real-alpha-implementation.md.tasks.json`; do not use
   the older Phase-0 list or Recipe Preview as the schedule.
2. Resolve only Chris's remaining content choices in `docs/next-session.md`, then normalize
   those catalog rows. Its curation decisions remain authoritative, but its older engine/UI
   scheduling paragraphs are superseded by the real-alpha plan.
3. Refresh and freeze every selected immutable source, hash, license/provenance record, and
   split BG1/BG2 run. Keep blocked or unimplemented choices unavailable.
4. Implement engine and UI slices with TDD in isolated worktrees, integrating reviewed
   commits only after their focused and workspace checks pass.
5. Treat Christopher's cold-cache install, recovery rehearsal, InfinityLoader boot,
   BG1 start, and save/reload as separate live acceptance evidence—not as implied by static
   tests or catalog review.

## Known wrinkles for the driver (from the reference install's history)

- EET requires a BG1EE+SoD source install to import from; EET_end must be the last "core"
  entry, but ~50 additive components legitimately sit after it (see install-order.tsv tail).
- WeiDU v24900 template pattern: copy `Setup-Branwen.exe` as `Setup-<modname>.exe`.
- Some mods prompt interactively despite force-install flags — capture required extra args
  per mod in mod-sources.tsv `notes` as they're discovered.
- Test installs must set a distinct `engine_name` in `engine.lua` so they don't share the
  Documents user dir (saves/baldur.lua) with the live install.
