# BARDICWONDERS — components

Chriz Easy BG's curated recipe pins **v2.9c-balance.9** (tag commit `c0c2e41`,
2026-11-18), replacing balance.8. Chriz Easy BG's base manifest and Chriz Easy IWD
have no Bardic Wonders run. The component selection and native order carry forward.
[balance.9 release](https://github.com/Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch/releases/tag/v2.9c-balance.9)

The table records the reference install refreshed for balance.4
(`37d3e5ab13d3b3520afb3246fa02de125a970b72`). Adapter `3010` first appears in balance.6
(`7db986160fcc5f466ecb8730eabd6bb226ee71c2`).
27 entries, 21 reference-installed. ✓ = installed in the reference. Subgroup = choose one.

## Save and rest fix in balance.9

- Save-compatible projectiles cover recurring Blade, Jester, Skald, Abettor, Dancer,
  Darkbloom, Storm Drummer, Troubadour and Kapellmeister song delivery, Skald's shared
  Combat Casting and Heroic Inspiration, Strategist's recurring abilities and Bardic
  Inspirations (`2002`). Hit-triggered combat effects retain their normal save behavior.
- Bard Song Mechanics Tweak (`2004`) rebuilds owned song wrappers and gives eligible
  external songs private projectiles that retain their targeting and geometry. Each
  finite song reserves IDs only once its delivery fits. Symphony can retain its HLA
  grant when the cue required for the optional party invisibility is unavailable.
  Bard descriptions follow the active song mode and installed options.
- IWD:EE's stock songs keep their native modal controls. Their projectile repairs
  can install independently of finite wrapping with the standalone fork.

The owner verified 211 automated tests, 54 WeiDU 24900 syntax and version checks, and 15
native fixture checks across BG2:EE, EET and IWD:EE. Live playtesting is pending. The
standalone Abettor 1.2.0 tail is the same archive published with balance.8 and is
outside the curated recipe. [balance.9 release](https://github.com/Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch/releases/tag/v2.9c-balance.9)

## Bard progression release and intended default

[Bardic Wonders v2.9c-balance.6](https://github.com/Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch/releases/tag/v2.9c-balance.6)
adds late adapter `3010`. It requires EEex and Rebalance provider `420`; base policy `421`
is optional for standalone users. The intended collection order is `420` → `421` → `3010`,
after the installed kits and other description changes.

The adapter registers Dancer for BG progression through spell level 6, Kapellmeister
and optional Darkbloom for IWD progression through level 8, and Abettor, Storm Drummer,
Troubadour, Deathsinger and Strategist for IWD progression through level 7. Existing kit
slot bonuses/penalties, Skald casting restrictions and caster-level changes remain.
Kit descriptions receive matching progression advantages or disadvantages.

Darkbloom is not required: absent kits are skipped. Installer fixtures passed with and
without it, and with provider `420` alone. Its separate Spell Revisions copied-spell
restriction remains; `3010` adds no SR restriction. Non-native bard casting systems and
unrecognized kits keep their fallback behavior. Artisan's Garrick Troubadour assignment
inherits the recognized Troubadour mapping.

The BG2EE/EET user smoke test passed on 2026-09-20; IWDEE has binary and offline checks
only. See the [selected policy and release integration boundary](../../research/2026-09-19-bard-progression-policy.md).
On 2026-10-13 Christopher made `3010` a recommended default whenever Bardic Wonders is
selected; see the notes below. The reference table below retains its captured provenance.

## UI/dependency notes

- `2001` is not a fix component: it creates the Trademeet vendor Leanne, a store, and a
  large item pack including the powerful Dirge +5 and Lament +5. It is excluded.
- `1006` is unavailable with Spell Revisions. Balance.8 added Spell Revisions-like
  installer fixture coverage for the fork's `1008`, `1009` and `2008` spell copies.
- `1012` and `2009` require EEex. `3001` requires `1012` or `2009` and installs late.
- Component `1008` stays selected for the Troubadour kit, but its interactive Garrick
  assignment must be scripted as **No**; Artisan NPC `99001` is the single deterministic
  Garrick assignment.
- Symphony's party Invisibility at song start and end is covered by installer
  fixtures. Verification in the game remains open.
  [Bardic Wonders issue #5](https://github.com/Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch/issues/5)
- `3010` extends Rebalance's bard spell-level ceilings to Bardic Wonders kits: Dancer
  stops at 6th-level spells, Kapellmeister and Darkbloom reach 8th (bard level 29), the
  other Bardic Wonders bard kits reach 7th (bard level 21). Kits that are not installed
  are skipped. It requires EEex and Rebalance `420`/`421`, and installs late, after the
  Rebalance run. **Default (Christopher's decision, 2026-10-13)** together with
  Rebalance `420`/`421` (see `CHRIZ-BG-REBALANCE.md`).

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 1001 | Bardic Wonders: Blade Overhaul | Kits |  | ✓ | default |
| 1002 | Bardic Wonders: Jester Overhaul | Kits |  | ✓ | default |
| 1003 | Bardic Wonders: Skald Overhaul | Kits |  | ✓ | default |
| 1004 | Bardic Wonders: Abettor of Mask Kit | Kits |  | ✓ | default |
| 1005 | Bardic Wonders: Dancer Kit | Kits |  | ✓ | default |
| 1006 | Bardic Wonders: Darkbloom Bard Kit | Kits |  | ✓ | default |
| 1007 | Bardic Wonders: Storm Drummer Kit | Kits |  | ✓ | default |
| 1008 | Bardic Wonders: Troubadour Kit | Kits |  | ✓ | default |
| 1009 | Bardic Wonders: Deathsinger Kit | Kits |  | ✓ | default |
| 1010 | Bardic Wonders: Strategist Kit | Kits |  | ✓ | default |
| 1011 | Bardic Wonders: Kapellmeister Kit | Kits |  | ✓ | default |
| 1012 | Bardic Wonders: Gallant Paladin Kit | Kits |  | ✓ | default |
| 2001 | Bardic Wonders: Items | Tweaks & Additions |  | ✓ | |
| 2002 | Bardic Wonders: Inspirations | Tweaks & Additions |  | ✓ | default |
| 2007 | Bardic Wonders: High Level Abilities | Tweaks & Additions |  | ✓ | default |
| 2008 | Bardic Wonders: New Bard Spells | Tweaks & Additions |  | ✓ | default |
| 2003 | Bardic Wonders: Armored Casting for Bards | Tweaks & Additions |  | ✓ | default |
| 2004 | Bardic Wonders: Bard Song Mechanics Tweak | Tweaks & Additions |  | ✓ | default |
| 2005 | Bardic Wonders: Item Restriction Tweaks—Bards may use mage items and priest scrolls | Tweaks & Additions |  | ✓ | default |
| 2012 | Bardic Wonders: Item Restriction Tweaks—Bards may use Small and Medium Shields | Tweaks & Additions |  |  | |
| 2013 | Bardic Wonders: Item Restriction Tweaks—Bards may use Helmets | Tweaks & Additions |  |  | |
| 2006 | Bardic Wonders: Bard Song Overhead Visual Effect | Tweaks & Additions |  | ✓ | default |
| 2009 | Bardic Wonders: Mage/Bard and Fighter/Mage/Bard multi-class (EEex, under thief multis) | Tweaks & Additions |  |  | |
| 2010 | Bardic Wonders: 3e Alignment for Bards (any non-lawful) | Tweaks & Additions |  |  | |
| 2011 | Bardic Wonders: Bard Casting Level Begins from 2nd level | Tweaks & Additions |  |  | |
| 3001 | Bardic Wonders: Patch Dialogue Class Checks for: Gallant, Mage/Bard and Fighter/Mage/Bard (install late) | Patches |  | ✓ | default |
| 3010 | Bardic Wonders: Kit-specific bard spell progression (EEex; install late) | Patches |  |  | default |
