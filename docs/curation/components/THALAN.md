# THALAN — components

Listed at release version **v4.5.1** (CrevsDaak/thalan). Not in the reference install.
1 entries, 0 installed. ✓ = installed.

## UI/dependency notes

- Christopher's decision (2026-10-23): Thalantyr Item Upgrade is an optional, unchecked
  third-party mod, labelled experimental until the combined lab install, and offered as
  is. Recommended does not include it. Since 2026-10-25 (Christopher) it is in "Items"
  instead of "Quest mods".
- Thalantyr in High Hedge combines BG1 items, potions or scrolls and gold into improved
  items; his store gains five BG2-style bags and cases and two Remove Curse scrolls. Its
  dialogue additions are `EXTEND_TOP` blocks on THALAN states 2 and 35, plus a rumour reply
  on state 1 on EET.
- The TP2 requires `GAME_IS bgee eet tutu tutu_totsc totsc bgt`, or BG2:EE with
  `oh3020.are` (TP2 L83). It has no component refusal that CEBG's stack can trigger.
- Balance fact, not a fix: on EE games the mod does not register its combined protection
  items in `ITEMEXCL.2DA` (its TotSC branch does), so the upgraded rings, cloak and amulet
  that absorb a Ring or Cloak of Protection or The Protector can be worn with other
  protection items.
- PPE's core component patches one THALAN line (Melicamp's chicken transition, a
  `REPLACE_ACTION_TEXT`); the two edits look order-tolerant but are untested together.
  Tweaks Anthology `1080` also adds a bag to `ighhedg.sto`; both coexist.
- Item Randomiser (curated Mode 1) keeps every recipe possible: relocated ingredients stay
  in BG1, and High Hedge's store stock is only in its unused Mode 2 lists.

## Collection behavior

- Run `thalan-bg2` installs in the `main` phase after `ntotsc-bg2` (Thalantyr's readme
  says to install it after NTotSC) and the other content mods, and before SCS, the Item
  Randomiser and EET_end. The public-alpha manifest runs it directly before the Item
  Randomiser; the curated recipe runs it directly before IWDification and the Tweaks
  Anthology. Its ini says `After = EET` and `Before = EET_end`.
- Tweaks Anthology store, bag, icon and two-handed tweaks (`3040`, `1120`, `110`, `170`,
  `180`, `2020`–`2035`) only cover items and stores present when they install, so
  Thalantyr goes first.
- The Item Randomiser duplicate-source audit found no hits, so no `no_random` rows are
  needed.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Thalantyr Item Upgrade |  |  |  | optional |
