# The Tweaks Anthology: snapshot 7649ced against v18

14 October 2026. Inventory for the curated recipe's move from Tweaks Anthology v18
(`031ec93`) to the unreleased master snapshot
[`7649ced`](https://github.com/Gibberlings3/Tweaks-Anthology/commit/7649ced6cd25865874d787ec1a9abbc67b068729),
134 commits later. The fallback `manifest/` profile does not install Tweaks Anthology.
Christopher's decisions are in [CDTWEAKS.md](curation/components/CDTWEAKS.md); this page
records the evidence behind them.

Method: static reading of both TP2 files, the component code (`lib/comp_<N>.tpa` and its
includes), the English text and the commit history, plus read-only checks of the reference
EET install. No WeiDU run happened during the audit; the lab installs listed at the end are the
first runs of this snapshot.

## The pin

- GitHub's commit archive, sha256 `117cd79c…075f7`, 23,871,463 bytes
  ([evidence](../manifest/evidence/release-intake-20261014.toml)). Unlike the v18 release zip,
  everything sits under one wrapper folder, `Tweaks-Anthology-<commit>/cdtweaks/`, next to
  repository files. The artifact uses the `single-wrapper` root rule and publishes only
  `cdtweaks`.
- The TP2 still declares `VERSION ~v18~`, so WeiDU.log lines end in `: v18`. A log can't tell
  the snapshot from the release; the archive hash can.
- `install-order.tsv` and the creator-full captures keep v18's name for `1257`, because they
  record the reference install.

## What stays the same

- All 175 components the curated recipe runs or offers keep their numbers, labels, groups and
  install conditions. Only two headers differ: `1257` (new name, now also moves Ajantis) and
  the grandmastery choice (`2212` and `2213` join `2210`/`2211`).
- All 446 components present in both versions keep their relative order, and both curated
  runs (`cdtweaks-bg2` and `2310`–`2312`) still match the snapshot's native order.
- Outside our selection: `2211` is renamed "BG2 grandmastery bonuses"; `4020` now refuses to
  install after SCS `4100`; `260` is gone and `3470` is deprecated.

## Fixes that touch our components

| Fix | Component | Verdict | Commit |
|---|---|---|---|
| SoD Death Cam no longer hangs in two cutscenes | `3300` | Confirmed: `bd3000.bcs` and `bdboarb2.bcs` get `EndCutSceneMode()` after `Kill(Player6)` | `abc2a0d` |
| Black Blade of Disaster no longer stacks extra attacks with grandmastery | `2210` | Confirmed, but see the Spell Revisions caveat below. The stacking was a regression from after v18 | `6ea032f`, `659d46d` |
| Mages can decline the Sphere | `1161` | Confirmed: Lavok's states 30 and 62 get accept/decline replies; they match our `LAVOK.DLG` | `05b872a` |
| Familiars no longer get maximum hit points | `3010`–`3012` | Confirmed for the eight vanilla familiars (and their `*25` versions); familiars added by mods still get them | `24b8dd2` |
| Kit ability table labels that SCS reads | `2400` | Confirmed: ranger CLAB rows are renumbered `ABILITY1…N`, including Artisan's ranger kits; contents unchanged | `8c3f06d` |

## Other changes to components we install

- **`1257` Move NPCs (Default) now moves Ajantis** to the Friendly Arm Inn exterior (BG2300,
  704,3291) by editing his start-of-game entry in `baldur.gam`. It includes `1258`'s code.
  Checked against BG1NPC v32, Artisan's Kitpack `3101` and BG1UB: none of them moves or
  spawns him or edits `baldur.gam`, so there is no conflict. His script replacement finds
  nothing to change on our stack.
- **`2210` True grandmastery (Default)** now builds its tables from templates. The only table
  change: High Mastery gives +5 damage instead of +4. To-hit, speed and extra attacks are
  unchanged. The new `2213` reproduces v18's `2210` exactly.
- **`10` Remove Helmet Animations** went back to patching helmet items on EE games; helmets
  added by later mods are not covered.
- **`20` Imoen's mage avatar** also patches 14 cutscene and dream copies of Imoen.
- **`110`, `2520`**: about 290 more armour and shield entries, mostly mod items.
- **`1035` Cloakwood "first area only"** adds a block meant to pause Coran's wyvern timer. It
  tests `InParty(Myself)` inside area scripts, so it most likely does nothing.
- **`2090` No experience cap** gains a workaround that only applies with Monastic Orders.
- **`3123` NPCs Don't Fight** removes Ajantis's attack on evil party members from state 4 of
  `BAJANT`; the state number is hard-coded.
- **`3340`–`3347` party movement speed**: longer follower list (mod NPCs, familiars) and a
  one-time ability grant inside a short cutscene.
- **`3390` Automatic tracking** now waits until combat is over and no enemy is near.

## Spell Revisions caveat: Black Blade of Disaster

The snapshot's grandmastery code also edits `BLAKBLAD.ITM` unless the item already grants
grandmastery itself (vanilla opcode 233 or the EE Fixpack's opcode 272). Spell Revisions
installs before Tweaks Anthology and replaces the item with one that has neither. So with
Spell Revisions, `2210` gives SR's Black Blade +3/2 attacks per round and raises its melee
header from +5/+5 to +8 to-hit / +10 damage. v18 never touched this item. Without Spell
Revisions, the vanilla or Fixpack item is left alone. Christopher's decision is below.

## New components

| # | Name | EEex | Status in the collection |
|---|---|---|---|
| `3500` | Summoned Creatures Clear Fog of War | required | **Default** |
| `3510` | Add Ribald's Special Stock to His Regular Store | – | **Default** |
| `2760` | Disable stealing at fences | used when present | **Optional**, part of **Challenge Mode** |
| `1258` | Move NPCs: Move Ajantis to the Friendly Arm Inn | – | Not offered: the default `1257` already includes it, and `1258` refuses to install after `1257` |
| `2212` | EE-style grandmastery bonuses | – | Not offered (grandmastery alternative) |
| `2213` | IWD grandmastery bonuses (v18's `2210` numbers) | – | Not offered (grandmastery alternative) |
| `2750` | Force the AI to cast AoE spells on the ground | required | Not offered |
| `2770` | Standardize buying/selling markups for all stores | required | Not offered |
| `2780` | P&P Free Action | – | Not offered |
| `3520` | "Bank" XP when dualing a character | required | Not offered |
| `3530`/`3531` | Always Import SoD Items (all but Voidhammer / Voidsword) | – | Not offered |
| `4240` | Get back what Yoshimo has "taken" from you | required | Not offered |
| `6360` | Quick To Master racial feat for humans | required | Not offered |

**`3500`**: at install time, every creature file marked as a summon gets a permanent effect
that clears fog of war for allies; so do the devas, planetars, elementals and Korax. Summons
that later mods add or replace are not covered. Our EEex includes the fix for the
fog-clearing creature limit. The recipe makes `3500` require the EEex feature, because
WeiDU would otherwise skip it.

**`3510`**: Ribald's 50 gp special store is retired; a one-time scene adds its items to his
normal store. The dialogue states it relies on match our `RIBALD.DLG`. Items that later mods
add to his special store are not merged.

**`2760`**: fences (stores that buy stolen goods) lose "can steal" and gain "reputation does
not affect prices". With EEex, as in our stack, this happens when a store opens, so fences
added by later mods are covered. At fences this overrides the store-reputation choice
`3070`–`3073`: the low-reputation discount and the fixed 80%/60% prices don't apply there.

## `3220` Sensible Entrance Points (not new, now on EET)

v18 did not allow `3220` on EET; the snapshot does.

- **BG1 part**: adds five named entrances (Beregost east, and four south/north exits). On
  our world map every link into those areas is already named, so nothing changes there.
- **SoA part (new)**: adds a southern entrance to the Bridge District (AR0500) and points 34 of
  its 39 world-map links there, so the party arrives from the south when travelling from
  areas further south. A trigger there brings Inspector Aegisfield to that entrance while the
  party hasn't heard about the murders yet.

## Readme gaps upstream

The snapshot readme has no entries for `1258`, `2212`, `2213`, `2780`, `3500`, `3510`, `3530`
or `3531`, and was not updated for the changes to `1257`, `1161`, `2210`, `3220` or `3300`.
It still says going from 4 to 5 pips adds +1 damage.

## Christopher's decisions (2026-10-14)

- Pin the `7649ced` snapshot the same way as EET.
- New defaults `3500`, `3510`, `3220`; `1258` only if it doesn't conflict with our Ajantis mods.
- `2210` stays the grandmastery default, with High Mastery's +5 damage in its label.
- `2760` joins Challenge Mode.
- Not added: `2670`, `2212`, `2213`, `3520`, `4240`, `6360`, `3530`/`3531`, `2780`, `2770`,
  `2750`.

## Christopher's decisions (2026-10-30)

1. **Black Blade with Spell Revisions** (above): alpha.22 keeps the stronger blade. How
   strong it should be is decided later, close to vanilla (balance item under "After
   alpha.22" in `docs/curation/components/FOLLOW_UPS.md`).
2. **Ajantis**: keep `1257`, which moves him by default. `1251`–`1256` plus `1258` are not
   offered.
