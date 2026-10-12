# Challenge Mode group

13 October 2026. Christopher approved the group and its behavior the same day
(below); it is not implemented yet. It covers the small first group and how
its control behaves. The full rule list stays in the
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
