# Optional choices and community follow-ups — 2026-09-11

## Decisions for the next release

Christopher approved exposing more SCS and Tweaks Anthology choices, **unchecked**,
after a bounded compatibility screen. Preserve the curated recommended selection;
the historical install log is not permission to re-enable excluded components.
Risky or unsupported choices stay excluded until a focused review, rather than
being offered with a warning as a substitute for a known compatibility rule.

The draft now adds **24 SCS and 44 CDTweaks choices**. Exact IDs, source-screening
notes and remaining exclusions are in the two component catalogs:

- [SCS](../curation/components/STRATAGEMS.md)
- [Tweaks Anthology](../curation/components/CDTWEAKS.md)

This includes JORA's requested SCS **4130, 4135, 4140**. All three carry a visible
**Not recommended:** title and experimental caution after a viewer reported poor
experience/bugs. These reports have not been independently reproduced in CEBG.
SCS itself marks the death-effects behavior experimental and explicitly forbids
4130 with Spell Revisions; the collection enforces and explains that restriction.
Provisions and inn bonuses are independent choices. The provisions rule exempts SoD.

Alternative groups have symmetric conflicts and shared display metadata, so the
existing grouped selector selects one and clears its siblings. The engine omits
conflicting choices even if an invalid combination is submitted outside the UI.
CDTweaks 70 and IWDification 10 are alternative providers of the same casting
graphics. All three CDTweaks spell-save-penalty variants (2310/2311/2312) stay in
the existing late spell-scan run; default 2312 has not moved or changed.

## Verification and release boundary

- 35 focused Python checks passed, including curation coverage, generated recipe,
  combined-playtest derivation, credits and armor-thieving regressions.
- 12 existing Customize UI behavior tests passed.
- Public-alpha recipe validation: no findings.
- Eight CLI planning cases checked the recommended setup, rest/inn choices,
  death-effects SR gate, SR-off selection, save-penalty and graphics alternatives,
  and exclusion of conflicting reputation/city-map choices.
- The recommended plan is still **436 components / 44 runs**, with exactly the
  same selected feature/run/component tuples as the pre-change recipe.
- All offered SCS/CDTweaks run lists match the pinned installers' native order.
- Catalog audit: 1,190 rows; 516 excluded, 229 optional, 302 default, 143 mandatory;
  71 choice groups, 23 optional-none groups, 10 fixed groups.

These are source, selection and ordering checks, **not gameplay acceptance of all
new options or combinations**. No game or saved installation was modified. App
alpha.16 / collection alpha.14 remains a draft; no new package was published here.

## Kivan / Jozzi quest — deferred by Christopher

A viewer reports a black screen after talking to Jozzi; a fish-man is killed by
the party before the scene resumes, then the remaining actors are neutral and no
dialogue follows. Component 130's existing guard was already in public releases.
It blocks lower fish-man AI during the dialogue stage, but does not prevent party
AI attacks, repair already-dead actors, or eliminate duplicate actors sharing a
death variable. That is a plausible remaining case, not a proven diagnosis.

No pre-Jozzi save or diagnostics are currently available. Christopher may reproduce
this later in the weekend or following week; stop investigating until then. Use a
pre-encounter save and verify the actual mod log before proposing a new fix. Do not
promise that updating CEBG alone repairs this saved encounter. Owning task remains
**Fix Kivan Sahaguin quest** / `chriz-bg-modpack`.

## Druid healing and rest QoL

The reported absence of instant druid Cure spells is consistent with SR's
regeneration-based healing design. Christopher clarified this as potential future
balance work in the SR fork and/or BG Rebalance, not a collection selection bug.
Do not silently replace the current spell lists.

Separately requested: automatically use memorized regeneration spells on rest.
See [the owning-mod handoff](../handoffs/2026-09-11-regeneration-on-rest.md).
Feasibility research is complete; implementation and live testing remain pending.

## Elf mage kit availability — restriction-removal coverage gap confirmed

Christopher supplied a character-creation screenshot: choosing elf, then Mage,
offers only **Mage, Diviner, Enchanter and Wild Mage**. Read-only checks of both
the stream reference and Combined-20260908 explain that exact result:

- `K_M_E.2DA` contains only kit rows `0, 24, 25, 30`; `KITLIST.2DA` maps them to
  those four names. The human table contains the other specialists as well.
- `mgsrcreq.2da` still has the elf eligibility flags disabled for the missing
  specialists. This is not merely an editor's naming/display issue.
- Selected Artisan component **1**, `lib/unlock_classes.tpa`, changes base-class
  availability and only the base-Mage row of the specialist racial table. It does
  not populate the missing elf specialist choices. Its name promises all classes,
  not all kits. This is distinct from the earlier Red Wizard/SR exclusion.
- **CDTweaks 2380**, Remove Racial Restrictions for Kits, was missing from the recipe.
  Pinned v18 `lib/comp_2380.tpa:49–58,76–159` updates specialist racial flags and
  adds already-playable kits to race selection tables, explicitly covering elves.
  It preserves internal/NPC-only kits and intentionally skips gnome mage choices.
  [Official documentation](https://gibberlings3.github.io/Documentation/readmes/readme-cdtweaks.html)
  describes the separate class/kit and gnome policies.

Christopher approved default inclusion on 2026-09-12. The draft alpha.16 /
collection alpha.14 now includes 2380 as a default-checked optional choice in the
existing main CDTweaks run, after the relevant Artisan and Bardic kit additions.
It has no artificial dependency on Artisan, Bardic Wonders or Spell Revisions.
The actual installer plan is 437 components / 44 runs; switching this option off
returns 436, with exactly `cdtweaks-bg2/2380` removed and nothing else changed.
36 focused recipe/curation tests pass and public-alpha validation has no findings.

Compatibility evidence (2026-09-12):

- Native WeiDU 249 ran the exact pinned v18 `comp_2380.tpa` against disposable
  fixtures built from copied stream and Combined effective tables. The source
  component SHA-256 is `6c6768dd5c19f3430f2d8aa6ea53fd6de780a94229e6b499f0053a68f37b89bb`.
- Both fixtures change elf Mage choices from four to ten: Mage, all eight normal
  specialists, and Wild Mage. Gnome Mage tables remain byte-exact unchanged, their
  specialist flags stay unchanged, and all four internal-only kits remain hidden.
- Each snapshot gains 66 effective kit/race choices: 60 selection-table additions
  and six Dwarven Defender eligibility-gate unlocks. All pass effective class/kit
  minima versus racial maxima checks (`ABCLASRQ`, `ABCLSMOD`, `ABRACERQ`,
  `ABRACEAD`); no unsupported rows or residual class/school gate blocks were found.
- Uninstall restores override tables byte-for-byte. KEY, BIF and both fixture
  TLKs stay unchanged through install/uninstall; original source-table hashes
  also remain unchanged. No full game copy or full installation was needed.
- Later Bardic 1012 does add the playable Gallant kit, but already makes it
  available in all seven paladin/race tables with class flags enabled. Thus it
  does not require moving 2380 past that late run. Other inspected late Artisan
  components modify existing kits/NPCs, not playable-kit eligibility.
- Local evidence: `Temp/cdtweaks2380-audit-b8da24e554604bdfa1e75e119c898064/`
  (`summary.json`, gate audit and native attempt logs). This is a native component
  harness, not a complete public CDTweaks TP2 run or an in-engine character test.

Separate pre-existing follow-up: Combined already exposes **dwarf Gallant** before
2380, with CHA minimum 18 versus effective dwarf maximum 17. Verify that case in
character creation with the Bardic Wonders owner; do not claim an observed freeze,
silently lower its stats, or attribute it to 2380. It was not changed here.

Completed in-engine elf character creation remains a separate quick live check;
fixtures are not full gameplay acceptance. Gnome policy is separate. No user game
or save has been changed, and this draft has not been packaged or published.

## Website differences from vanilla — after the next release

Prepare a gamer-readable comparison **after** releasing the next version. Discuss
and triage the content with Christopher and/or the Twitch website task before
publishing. Distinguish the recommended setup from optional alternatives and
unreleased roadmap items; source it from the released recipe and owning mods.
Do not autonomously publish a purportedly exhaustive list, or delay the installer
release to write the whole comparison now.
