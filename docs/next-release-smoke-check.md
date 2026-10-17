# Next release: one install, short manual check

Christopher reconfirmed this testing policy on September 29: complete the approved
implementation and automated checks, then use **one combined fresh installation**.
Do not turn each component or UI provider into a separate required manual test.
**September 30 status:** the fresh Test19 installation completed, and Christopher
used Play to launch it. Do not repeat the full installation or use the outdated
September 29 setup named in earlier instructions. The remaining gameplay checks below
are not yet recorded as accepted; do not infer that they failed either.

**October 7 Defender/Radar follow-up:** Christopher authorized a bounded
reproduction using the October 1 Radar Fix candidate and the completed Test19
installation. Installation, Play, installation-details/path copying and cleanup
produced no new Defender detection. Radar-off and approved Radar-on launches
passed. The original September 30 alert's precise cause remains unknown; these
results do not establish a confirmed false positive or cover every later build.
See the [launch evidence and limits](defender-smoke-2026-10-07.md) and
[candidate verification](package-acceptance-alpha19-radar-2026-10-01.md).
No save was loaded or manually saved/deleted during those checks, and declining
Radar's administrator prompt remains untested. Existing exclusions could not
be inspected, limiting the real-time scanning claim; explicit scans ignored
exclusions. Steam Cloud imported saves into Test19, so a separate local profile
alone is not sufficient isolation for any future save/reload check.

That one-time test authorization does not authorize another live check. Do not
restore quarantine, change antivirus protection or launch a new live check
without Christopher's agreement. See the [current handover](handover.md) and
[original incident record](package-acceptance-alpha19-dragon-order-2026-09-30.md).

## Test selection

Use the recommended setup, including Spell Revisions. Keep story-based utility XP,
Evandra's Sorcerer conversion/portrait, the Flail change and Emotion changes on.
Add the optional generic unidentified-item visuals (Modpack 630); its conflicting
Randomiser appearance option 570 must be off. Ordinary randomization can remain on.
Shifted dialogue may be included, but does not require a separate playtest gate.
Use one preferred interface; other interface combinations go to community testing.
Previously selected difficulty options may stay on; no new difficulty selection
or changed default follows from this checklist.

CEBG must provide these choices from verified owner releases before the test.
Do not edit the stream install, use an unreleased branch archive as a release, or
remove WeiDU.log entries to force an installation through.

## Required manual smoke check

1. Installation completion and a game launch are already reported. Confirm basic
   menus, make a save, and reload it if that has not already been checked.
   Report an installer error once;
   keep its diagnostics rather than repeatedly retrying an unexplained failure.
2. Briefly inspect Evandra at first recruitment: Sorcerer, intended portrait and
   a usable starting spellbook/level-up. This is not an XP catch-up acceptance
   test: Fade/Evandra's separate catch-up report remains open.
3. Check the new mechanics with quick console access if convenient: +5 Flail
   allows Haste, and an unidentified item uses generic art until identified.
   No replay to obtain these items and no prolonged proc-rate measurement.

If convenient, compare one scribing award in BG1/SoD with one in SoA. The automated
campaign-transition and reward-table tests are the main evidence for utility XP;
this optional sample is not another campaign playthrough or a release blocker.
One normal conversation in a shifted form is also optional.

## App check: recorded result and remaining coverage

The approved October 7 follow-up started without a game or Radar running.
Unchecked Play reached Test19's main menu without launching Radar. After the
game exited, checked Play requested Radar's Windows approval; Christopher
approved it and Radar appeared. Those launch paths passed on the exact October 1
candidate; they are not still pending. They do not prove combat readings or
runtime acceptance of later packaged binaries.

Declining the administrator prompt remains untested. If a further app check is
agreed, confirm that declining leaves the game running and shows a clear message.
No second full game installation is required for that check. This remaining
coverage is not permission for an agent to click or launch anything while
Christopher streams, or to repeat the already completed checks automatically.

## Agent-side checks before/after that install

- Before packaging: recipe hashes, component availability, default/exclusion rules,
  package contents, relevant source tests and new breakpoint safety tests.
- After the user completes the install and permits inspection: read-only final
  receipt/WeiDU checks and effective-resource checks can verify selected components,
  Flail save penalties and utility-XP configuration without extra manual repetition.
- Manual breakpoints: automated pause/reopen/accept/resume/finalize coverage is
  required. A short user presentation check is useful, but a second full install
  or a matrix of personally supplied mods is not required.

## Not repeated as release gates

The accepted BuffBot update/undo and Artisan description patch, a full SoD
playthrough, bridge/Mislead replay, every kit, every UI provider, every inventory
surface, and an uninstall/reinstall cycle for every new component. Detailed owner
checklists remain available for targeted follow-up/community testing, not as an
automatic expansion of Christopher's required work.

Record actual results and limitations. Community testing does not excuse a
source-proven startup crash, unsafe resume path or missing required component.
Publishing the CEBG app/updater/website remains a separate release decision after
the candidate is prepared and Christopher has assessed it.
