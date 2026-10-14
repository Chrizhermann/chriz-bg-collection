# Challenge Mode group

13 October 2026. Christopher approved the group and its behavior the same day
(below). Implemented in the installer on 13 October (recipe `[[feature_groups]]`,
engine state, CLI `--group`, desktop card); the website step is still open. It covers the small first group and how
its control behaves. On 14 October the group gained its first [variant](#variants-switching-a-choice):
strict potion uniqueness. The full rule list stays in the
[Challenge Mode outlook](2026-09-13-challenge-mode-outlook.md#complete-rule-inventory).

Christopher's decision: "a big choice at the top of the installer" that
switches all Challenge Mode components on or off; they can still be swapped in
and out individually, but each must be marked as part of Challenge Mode; not a
default. He always turns it on for his stream installs.

## Members

Every implemented challenge component is already an optional, unchecked
feature in the next-release recipe:

| Feature | Owner component | Effect | Requires | Membership |
|---|---|---|---|---|
| Emotion, Courage and Hope don't stack | Rebalance 301 | Casting either Emotion removes the other; recasting refreshes | — | Settled (Oct 7) |
| Baeloth: normal Sorcerer spells known | Modpack 235 | Legal Sorcerer spell selection, SR or vanilla | — | Settled (Oct 4/7) |
| Extra Challenge: lethal dragon melee | Rebalance 110 | Stronger SCS dragon melee at high difficulty, including death-protection interactions | SCS Smarter Dragons, EEex | Settled (Oct 13) |
| Extra Challenge: Insane bridge sequencers | SoD Remix 257 | Harder sequencers for the SoD bridge mages on Insane | SoD Remix | Settled (Oct 13) |
| No stealing from fences | Tweaks Anthology 2760 | Fences can't be stolen from, and reputation no longer changes their prices | Tweaks Anthology (curated profile only) | Settled (Oct 14) |

Variant (see [below](#variants-switching-a-choice)): Potion uniqueness picks the strict
Rebalance `330` (Potions of Genius don't stack either) instead of the recommended `331`.
Settled October 14.

A possible later member is the extra layer for the new Degrodel fight
(Modpack, in progress), if Christopher puts it here.

No other implemented component enforces a rule from the inventory. Wands,
traps, Dispelling/explosive arrows, Wish and Time Stop limits, protection
scrolls, Ravager, area exits and the remaining rules have no implementation
yet. They are not part of this first group.

Not members, as already decided: Flail of the Ages, elemental arrows, the 90%
physical cap, the dragon Wing Buffet spacing and the other normal defaults.
The group never changes them.

## Checkbox behavior

- **Placement:** a prominent "Challenge Mode" choice at the top of the
  installer's selection screen, above the mod list, with a short explanation
  and its members listed (Christopher, October 13; replaces the September 29
  placement among the mod checkboxes). It is not an advanced setting and not
  an indivisible bundle.
- **Marking:** each member keeps its normal place under its own mod and shows a
  visible "Challenge Mode" mark there, so a hand-picked member is still
  recognizable.
- **On:** selects every member whose own requirements are met. A member whose
  requirement is missing, such as lethal dragon melee without SCS Smarter
  Dragons, shows as unavailable with the reason. It is never forced on.
- **Off:** deselects every member.
- **Mixed:** shown when some, but not all, available members are selected,
  usually after changing a member by hand. Clicking a mixed group selects all
  available members.
- **Individual choices are preserved.** Changing one member never changes
  another. The group state is computed from the members and is not stored
  separately, so existing profiles, presets and install records stay valid.
- **Defaults:** Chris's Recommended keeps the group off. Nothing else in the
  preset changes.

## Implementation outline

1. Recipe: an optional group record naming member feature IDs. Validation
   rejects mandatory/default members, unknown features and members of two
   groups.
2. Engine: expose the group and its computed on/off/mixed state in the recipe
   view; selection stays per feature.
3. App: tri-state checkbox with the member list; reuse the existing
   unavailable-requirement messages.
4. Tests: on/off/mixed transitions, unavailable members, preserved individual
   choices, unchanged normal defaults, both profiles.
5. Website: describe only the shipped members; list the rest as planned.

## Variants: switching a choice

Christopher, October 14: Challenge Mode picks strict potion uniqueness instead
of the recommended rule. A member cannot do that, because members are optional
features outside any choice group. A **variant** names a choice group and the
option the group picks in it. It is general: a later Challenge choice, such as a
grandmastery variant, uses the same field.

```toml
[[feature_groups]]
id = "challenge-mode"
members = [...]
variants = [
  { choice_group = "potion-uniqueness", option = "feature:chriz-bg-rebalance:component-330", label = "Potions: strict (Potions of Genius don't stack either)" },
]
```

`label` is optional; without it the installer shows the option's own title.

**Validation** (recipe errors):

- The choice group exists and has at least two options, none of them mandatory,
  so the group can always deselect the alternatives.
- The option is an optional feature of that choice group, and not its recipe
  default (the option marked `default`).
- At most one group sets a variant for a given choice group, and a group names
  each choice group once.
- A label, if given, is not empty. A group lists at least one member or variant.
- Members and variant options of one group must not conflict with each other;
  switching the group on would otherwise disable both.

**Behavior:**

- **On** picks the variant's option and deselects the other options of its choice
  group. An option whose own requirements are missing, such as the potion rule
  without BG Rebalance, is shown as unavailable with the reason and never forced.
- **Off** restores the choice group's recipe default (`331` for potions).
- **State:** a variant counts like a member. The group is on when every available
  member is selected and every available variant option is picked, off when none
  is, and mixed otherwise. Picking another option of the choice by hand after
  switching the group on makes it mixed, like deselecting a member. An unavailable
  variant does not keep the group mixed.
- **Selections stay per feature.** The group state is still derived and never
  stored. A setup saved before variants existed (members on, `331` picked)
  reloads unchanged and shows the group as mixed; switching it on picks `330`.

**CLI:** `--group challenge-mode=on|off` applies the same changes after the
`--feature` choices. An explicitly chosen option of the variant's choice group,
anywhere on the command line, keeps the group from switching that choice. The
plan summary adds "X of Y available variants picked"; the JSON output lists each
variant with `selected`, `available` and `unavailable_reason`, and the recipe view
adds its `label`, `options` and `recipe_default`.

**App:** the Challenge Mode card lists the variant after the members with its
label, its state, the mod it comes from and the recommended option it replaces,
which switching the group off restores. The choice's own row under its mod shows
the Challenge Mode mark, and the option the group picks says "(Challenge Mode)".

**Compatibility:** groups without variants read, write and hash exactly as
before. Older apps reject the new `variants` field, so the release that ships
this recipe must set `minimum_app_version` to the app that adds variants.
