# IWDIFICATION — components

Listed at installed version **v11**.
23 entries, 8 installed. ✓ = installed. Subgroup = choose one.

## UI/dependency notes

- `140`/`141` and `120`/`121` are optional upstream subgroups. The collection exposes
  `140` and `120` as the defaults; choosing neither remains valid, while the excluded
  alternatives are not shown.
- Component `120` is compatible with ordinary Artisan thief kits because it discovers
  their CLAB tables through `KITLIST.2DA`. Artisan's Mage/Thief Arcane Trickster is the
  exception: its KITLIST CLAB cell is `*`, so IWDification misses `C0ATR.2DA`. Keep `120`
  as the selected default, with a required collection-tail patch for Arcane Trickster.
- Component `120` is unavailable with Skills and Abilities `40450`/`40460` or Tweaks
  Anthology `6280`, which provide alternative Evasion implementations. RR's activated
  high-level Evasion is only a balance overlap, not the same technical component.
- Components `30` and `40` are included whenever IWDification is selected. SCS
  `1500`/`1510` are alternative IWD spell-pack providers and must be unavailable while
  these components are active.

## Integration/follow-up notes

- Install Spell Revisions core first so `30`/`40` take their SR compatibility paths.
  Install IWDification after NPC and kit mods, but before SCS.
- Add the missing Arcane Trickster level-7 Evasion grant to `C0ATR.2DA` in the maintained
  collection tail layer.
- The IWD arcane and divine spell packs need a later balance review in the collection's
  balance layer; that is not an installer dependency for `30`/`40`.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 10 | Icewind Dale Casting Graphics (Andyr) | Miscellaneous Changes |  | ✓ | default |
| 20 | Commoners Use Drab Colors | Miscellaneous Changes |  |  | |
| 60 | Two-Handed Axe Item Pack | Miscellaneous Changes |  | ✓ | default |
| 90 | Expanded Polymorph Self | Miscellaneous Changes |  |  | optional |
| 130 | Use IWD Damage Animations | Miscellaneous Changes |  | ✓ | default |
| 140 | Add High Quality Items | Miscellaneous Changes | Randomized Enemy Equipment | ✓ | default |
| 141 | Do Not Add High Quality Items | Miscellaneous Changes | Randomized Enemy Equipment |  | |
| 190 | Increase Spear Damage | Miscellaneous Changes |  | ✓ | default |
| 50 | Bard Class Update: Add IWD Bard Songs | Class Updates |  |  | |
| 180 | Bard Class Update: Selectable Bard Songs for Jesters and Skalds | Class Updates |  |  | |
| 150 | Bard Class Update: Use IWD Spell Progression | Class Updates |  |  | |
| 70 | Druid Class Update: Use IWD Shapeshifting and Ability Progression | Class Updates |  |  | |
| 71 | Druid Class Update: Allow Elves to be Druids | Class Updates |  |  | |
| 100 | Paladin Class Update: IWD Abilities and Skills | Class Updates |  |  | |
| 160 | Paladin Class Update: Use IWD Spell Progression | Class Updates |  |  | |
| 110 | Change Tracking for all rangers and ranger kits | Class Updates | Ranger Class Update: Tracking |  | |
| 111 | Change Tracking only for non-kitted rangers | Class Updates | Ranger Class Update: Tracking |  | |
| 170 | Ranger Class Update: Use IWD Spell Progression | Class Updates |  |  | |
| 120 | Add evasion to thieves and all thief kits | Class Updates | Thief Class Update: Evasion | ✓ | default |
| 121 | Add evasion only to non-kitted thieves | Class Updates | Thief Class Update: Evasion |  | |
| 30 | IWD Arcane Spell Pack | Additional Spells |  | ✓ | mandatory |
| 40 | IWD Divine Spell Pack | Additional Spells |  | ✓ | mandatory |
| 80 | Additional Portrait Icons | Additional Spells |  |  | |
