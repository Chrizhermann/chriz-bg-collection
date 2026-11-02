# BARDICWONDERS — components

Listed at reference version **—**; refreshed for Chris's fork release
**v2.9c-balance.4** (`37d3e5ab13d3b3520afb3246fa02de125a970b72`).
Row `3010` comes from the pinned release **v2.9c-balance.6**
(`7db986160fcc5f466ecb8730eabd6bb226ee71c2`); it did not exist in the reference install.
27 entries, 21 reference-installed. ✓ = installed in the reference. Subgroup = choose one.

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
- `1006` is unavailable with Spell Revisions. The fork's `1008`, `1009`, and `2008`
  still need semantic Spell Revisions fixture coverage before broad compatibility is claimed.
- `1012` and `2009` require EEex. `3001` requires `1012` or `2009` and installs late.
- Component `1008` stays selected for the Troubadour kit, but its interactive Garrick
  assignment must be scripted as **No**; Artisan NPC `99001` is the single deterministic
  Garrick assignment.
- The released fork now includes the selected Skald, Dancer, Jester, Abettor, and
  shared HLA corrections described in its release README. The complete release has
  not been validated in live combat; Symphony's intended one-round party
  invisibility remains a known issue.
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
