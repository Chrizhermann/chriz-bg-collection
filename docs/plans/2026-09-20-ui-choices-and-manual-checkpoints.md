# Near-term: game UI choices and manual installation checkpoints

Status: initially parked September 20 behind existing-install updates; those
broader updates were subsequently lowered in priority. **September 29: Dragonspear
UI++ is approved for the next-release UI batch**, alongside the existing approved
providers. **Manual checkpoints are also explicitly included in the next release**
by Christopher's September 29 clarification. Both implementations remain pending;
this scope decision does not relax unexpected-change checks or claim a release.

Latest user decisions supersede tentative scope/testing language below:

- Infinity UI++ is approved for the initial options; leave live acceptance to users.
- EET SoD-style GUI is approved. BG1/BG2 visual styles are welcome through supported
  providers. These are intended as presentation choices, not gameplay changes;
  do not assume replacing UI scripts cannot affect menus or compatibility.
- LeUI is approved for the initial options. Do not hold this work for agent-led
  exhaustive gameplay testing; document the unverified 2.7.3 behavior honestly.
- Dragonspear UI++ is approved September 29 as an optional next-release choice,
  superseding the earlier deferral. Keep the recommended interface unchanged.
- UI choices remain new-install-only. Manual checkpoints retain the confirmed
  limited scope and no-game-hotpatches-after-manual-additions policy.
- Community testing is an accepted delivery route. Keep focused automated checks
  and basic file/history safety verification, but do not require exhaustive
  agent-led installation/gameplay coverage or Christopher's personal acceptance
  for every UI and manual-mod combination. Report the actual verification boundary
  and fix known deterministic failures rather than certifying untested combinations.

## 1. Choose a game interface

Keep **Chriz's recommended interface** selected and the one-click flow unchanged.
In Customize, offer one compact **Game interface** selector, a small preview and
one-sentence description per supported choice. This selects the game's interface,
not a theme for the CEBG launcher. Full UI replacements are mutually exclusive;
compatible small UI add-ons remain separate features.

Preparation before implementing:

- Initial shortlist is now approved above. Do not advertise every arbitrary UI
  as supported or broaden to additional providers without a request.
- Pin each provider's source/version/hash and author component, language, EET/game
  support, order, dependencies and conflicts. Keep third-party content in its repo.
- Audit the current default as a group of features, not an assumed single custom
  UI mod. The current curated recipe includes Bubb's Spell Menu, EEex, Hidden
  Gameplay Options and BuffBot; it does not name a full UI-replacement provider.
- Reuse existing choice-group controls and dependency evaluation. Before committing
  a switch, show **Also changes:** affected add-ons/options. Disable known conflicts
  with a reason; restore only remembered choices that remain compatible on return.
- Position each provider according to its actual installer requirements. Bubb's
  Spell Menu must see the final replacement UI. Check EEex, BuffBot, Hidden Gameplay
  Options and later UI edits, including EET stages; do not blindly install every UI last.
- Hidden Gameplay Options already has documented provider-specific exclusions:
  GUI options 300-304, plus some individual settings. Recheck exact current sources
  when authoring rules; do not lose unrelated gameplay settings just because their
  settings screen cannot be installed. The follow-up below records a brief upstream
  documentation check, not full compatibility acceptance.
- **Confirmed by Christopher: new installations only.** Do not build a workflow
  for replacing the UI of a completed installation in this feature.

Acceptance when resumed: keep basic package/selection/dependency checks, but leave
live UI/playtesting to users as requested. Publish a short optional feedback checklist
(start/load, character creation/level-up, inventory, spellbook, dialogue, options,
Bubb's menu, BuffBot). Do not require agent-led full installs or live smoke tests
for every provider. Do not label community-test candidates proven compatible or
ignore a source-proven deterministic install failure.

### Candidate shortlist and follow-up decisions — September 20

These are established candidates, not a statistically ranked popularity list.
No reliable comparable usage totals were established in this quick check.

- **Infinity UI++: first integration candidate.** The author's page declares EET
  and Bubb's Spell Menu support and four skins: Dragonspear, IWD, BG Stone and BG2
  Roots. Official releases list v1.27; v1.24/v1.25 explicitly address game patch 2.7.
  The author still lists SCS UI-related components under known issues: check our
  pinned SCS difficulty controls before enabling, without assuming all SCS fails.
  Sources: [author overview](https://forums.beamdog.com/discussion/87209/mod-infinity-ui-v1-24),
  [official releases](https://github.com/Renegade0/InfinityUI/releases).
- **EET's own SoD-style GUI: candidate for a small integration.** Upstream
  `EET_gui` component 0 copies the SoD GUI; component 1 instead requires IWD-in-EET,
  so the latter is not just a cosmetic switch for our current stack. The current
  CEBG EET artifact publishes only EET/EET_end, and a production recipe test
  explicitly excludes EET_gui. It is not already selectable. Add explicit artifact
  publication/manifest/order/evidence if approved, not just an extra checkbox.
  Source: [upstream installer](https://raw.githubusercontent.com/Gibberlings3/EET/master/EET_gui/EET_gui.tp2).
- **Lefreut's Enhanced UI (LeUI): classic-looking candidate**, with BG2, BG1EE and
  SoD variants. EET support is declared, but current latest is v4.9.1 and the README
  documents that version for game 2.6. Do not call our 2.7.3 stack supported until
  checked. Sources: [main/readme](https://github.com/r-e-d/LeUI),
  [BG1 skin](https://github.com/r-e-d/LeUI-BG1EE),
  [SoD skin](https://github.com/r-e-d/LeUI-SoD),
  [latest release](https://github.com/r-e-d/LeUI/releases/latest).
- **Dragonspear UI++: approved September 29 for the next release.** It is a separate
  provider, not Infinity UI++'s Dragonspear skin. Pin and check its own package,
  install order and specific conflicts; do not inherit Infinity's compatibility
  rules without evidence. Praetor reports it worked after manual installation,
  but his altered WeiDU.log is not a clean installation acceptance result.
  Source: [official releases](https://github.com/Renegade0/DragonspearUI/releases).
- **IWDEE-UI-BGEE: later candidate**, reproducing the IWD interface with declared
  EET support. Not needed to expand the first batch when Infinity already offers
  an IWD skin; these are different implementations, not interchangeable evidence.
  Source: [author documentation](https://www.nexusmods.com/baldursgate2ee/mods/102?tab=docs).

Christopher is happy to omit optional Hidden Gameplay Options where needed.
However, incompatibility is not blanket: its documentation explicitly supports
LeUI/Dragonspear/Infinity for Improved Cheat Menu, while its GUI Improvements group
is limited to default GUIs; Infinity's author recommends HGO. Prefer omitting
conflicting/redundant components and retaining explicitly supported optional ones.
If support is uncertain, disabling HGO for that provider is an acceptable bounded
first-version proposal, not a reason for a large compatibility rewrite. Show the
change clearly. [HGO documentation](https://github.com/Argent77/A7-HiddenGameplayOptions#components).

### Community compatibility lead: Infinity UI++ and SCS thief points — September 20

Sauler reports that SCS **4115, Thieves assign skill points in multiples of five**,
causes a startup Lua error with Infinity UI++. CEBG currently selects 4115 by
default (`feature:stratagems:component-4115`); this is a specific UI-changing
component to account for when the parked interface work resumes, not a reason to
exclude SCS as a whole. The reported failure has not been reproduced here.

Reference supplied by the author:
[IWDEE Tweaks and Fixes, component 2](https://github.com/Sauler89/IWDEE-Tweaks-and-Fixes#2-thief-skill-points-in-multiples-of-five).
Its README describes five-point allocation and rounded `THIEFSKL.2DA` awards, and
explicitly says **not to combine it with SCS 4115**. Treat this as an IWD-specific
implementation reference, not a BG/EET-ready add-on or a verified fix for CEBG.

When UI integration resumes, check the exact pinned versions and whether a small
properly attributed adaptation can preserve the five-point feature. Check license
and game-specific assumptions before reusing code. Otherwise propose omitting
4115 for Infinity UI++ with a clear explanation in the interface selector. Do not
stack both implementations or silently change the current default recipe.
Recorded only: no implementation, install, or live test is authorized by this note;
existing-install updates remain the priority and UI work remains parked.

The preceding paragraph records the September 20 boundary, not the current
priority. See the September 29 approval at the top of this plan.

## 2. Pause to install your own mods (advanced)

### September 29 report: manual UI addition rejected at finalization

Praetor reports pausing after EEex, installing Dragonspear UI++ manually, and
CEBG failing finalization on the extra WeiDU.log entries. He removed those entries
and continued. This is a reported reproduction, not locally reproduced evidence.
Do not recommend editing out installation history: the added mod still exists,
while the log no longer honestly records the installed stack.

Keep this as a checkpoint regression case: accept an explicitly reviewed manual
addition at an intended checkpoint, preserve its WeiDU entries and provenance,
resume the remaining managed work, and successfully finalize with both histories.
Unapproved changes must still stop; ordinary pause/resume is not permission for
arbitrary log changes. Official Dragonspear integration should instead declare
its expected components in the normal recipe, requiring no manual pause or edit.

### Confirmed design

Put this behind a collapsed **Advanced → Add your own mods** section, off by
default. First-use warning: “Manual mods can conflict with this setup. You choose
their compatibility and install order. CEBG records your changes, but cannot
guarantee them. Adding manual mods disables game hotpatches for this installation;
CEBG app updates still work.” Link a short guide.

Minimal UI: **Add pause** → searchable selected component with mod name, number,
phase and target game; show a compact ordered list with remove controls. Support
multiple pauses. Freeze stable `(run_id, component)` anchors with the selection;
if selection changes remove an anchor, require correction rather than silently
moving the pause. Do not expose drag-and-drop reordering of the curated recipe.

At a checkpoint:

1. Finish and verify the selected component in a fully exited WeiDU invocation;
   save a durable checkpoint before handing over. Never suspend a running WeiDU
   child and invite another installer to modify the same files.
2. Show **Paused for your mods**, the exact managed BG1 or BG2 folder, last completed
   component and next planned component, with **Open game folder**, **Check changes
   and continue**, and **Continue without additions**. Source games stay untouched.
   Preserve this state across app close/reopen without treating it as a failure.
3. Before continuing, require game/install processes to be closed, inspect the
   changed WeiDU entries and available diagnostics, show the detected additions,
   and obtain explicit confirmation. Persist acceptance atomically and resume
   only the remaining plan. A no-change continuation is valid.
4. Keep the original recipe receipt immutable and record checkpoint/addition
   provenance separately, including TP2, language, components, order, reported
   versions and available diagnostics. Do not present reported versions as verified
   source hashes. Mark the result **Customized — manual mods added**.

Confirmed initial scope: accept new appended WeiDU components
at declared checkpoints. “Appended” describes the install log, not a restriction
to mods that only create files: normal resource/TLK patching is expected. Reject
removal/reordering/reinstallation of already managed components, pre-installation
of future managed components and conflicting source/backup namespaces. Arbitrary
file-copy mods are outside this first scope. Christopher clarified that this is a
courtesy feature for experienced supporters, not a general-purpose mod manager:
do not add a removal/reinstallation or earlier-stack-editing workflow.

Do not globally disable drift detection. Extend reconciliation, receipts and final
verification to understand approved manual additions interleaved with managed runs.
Unapproved drift outside a checkpoint remains an error. Expected resource/TLK
changes are not themselves a reason to fail; a matching WeiDU log is also not proof
of success or compatibility. Preserve available failure diagnostics and explain
unverifiable changes without automatically destroying/restarting the installation.

Other constraints to cover before implementation:

- Current pause points are whole pipeline steps/runs, not individual components.
  Split only requested batches into planned sub-invocations, preserving component
  order, owned prompt answers, arguments and correctly timed postconditions. Mods
  with cross-component invocation assumptions need explicit indivisible groups;
  unavailable boundaries must explain why, never pause mid-component.
- Keep separate BG1/BG2 baselines. Once BG1 has been imported into EET, a later BG1
  edit will not automatically be imported again. Show only meaningful target slots;
  EET finalization and content-scanning passes make placement consequential.
- Preflight manual mod names/paths against later managed archive publication. Do
  not weaken existing ownership/collision checks to overwrite manual files.
- Reacquire the CEBG mutation lock and recheck the baseline during continuation;
  prevent two CEBG workers or a still-running manual installer from racing.
- Diagnostics and final consistency must retain both managed and manual history.
  Conservative initial hotpatch policy: once manual additions are accepted,
  disable **game hotpatches for that installation** and explain why. Do not silently
  re-enable them because a later log no longer shows the added component. A pause
  with no changes does not itself disable patches. CEBG app updates and installing
  a newer collection into a separate copy remain available. A reviewed built-in
  UI choice is not a manual addition; its patch eligibility remains provider-specific.
- Document a worked example, both target folders, ordering responsibility, normal
  resume vs manual checkpoints, backups, known limitations and troubleshooting.

Focused automated checks: two checkpoints (including inside a multi-component run), no additions,
successful additions, reopen while paused, wrong target, failed manual install,
changed managed prefix, namespace collision, final receipt and patch eligibility.
Use small fixtures and basic package checks; broad real-mod combinations and live
playtesting may be covered by the community. Do not add a mandatory full-install
matrix or repeated personal playtests as a release gate. Praetor's reported path
must be represented by an accepted-addition/finalization regression check.

## Source pointers from this quick pass

- `recipes/curated-full-current/collection.toml`: actual runs and UI-related choices.
- `docs/curation/components/BUBB_SPELL_MENU_EXTENDED.md` and
  `HIDDENGAMEPLAYOPTIONS.md`: recorded ordering/provider restrictions.
- `engine/src/manifest.rs`: `Run`, `ComponentRef`, feature choice/dependency fields.
- `app/src/screens/setup.ts`: existing grouped alternatives and disclosures.
- `engine/src/orchestrator.rs`: schedule, boundary pause and frozen resume.
- `engine/src/weidu/verify.rs`: strict expected-prefix/suffix reconciliation.
- `engine/src/receipt.rs`: receipt and per-attempt log-diff validation.
- `engine/src/acquire/materialize.rs`: owned publication/collision protections.

Approved UI batch: current default plus Infinity UI++, EET's SoD GUI, LeUI and
(added September 29) Dragonspear UI++. New-install-only and additive-log-only
checkpoint scope remain confirmed. Provider integration and checkpoint support
are both in the next-release scope and remain unfinished; scope approval is not
implementation or release.
