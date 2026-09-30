# Flail of the Ages, Free Action, Dispelling Screen and Entropy Shield

Captured from Christopher on September 28. **Backlog/design only:** no immediate
implementation, game action, component selection or next-release commitment.
No need to dispatch another agent tonight. This records user proposals, not
verified current item/spell behavior or a completed balance assessment.

**September 29 update:** Christopher approved implementing section 1 for the next
release, as a default. BG Rebalance component **302** is published in **v0.7.0**
(PR #4, merge `5db2f64`), with 13 passing real-WeiDU disposable tests, verified
vanilla/CEBG inputs and a full-current-description check. The full suite completed
382 cases with 26 existing capture-dependent skips, exit 0; the downloaded release
archive matches the reviewed package. CEBG pin/default integration is underway;
live gameplay testing remains pending. Sections 2–4 remain deferred discussions;
the approval does not authorize a global Free Action change, live-game patch,
save edit or publication without the normal owner release preparation.

## 1. Flail of the Ages rebalance

**Owner:** `chriz-bg-rebalance` (existing base-game item balance, not companion
content or collection orchestration). **Intended CEBG selection:** default once
implemented and integrated; no default changed by this note.

Requested design:

- On the +5 version, keep Free Action's protection from harmful impairment but
  allow beneficial Haste, including Improved Haste. Do not make upgrading the
  weapon impose the reported anti-Haste drawback.
- Give the on-hit Slow a saving throw versus **breath**, with penalties:
  **+3: -2; every +4 variant: -4; +5: -6**. A successful save negates Slow,
  not the hit or its unrelated damage/effects.
- No additional change to proc chance, duration, damage, other item properties
  or lower upgrade tiers was selected. Confirm those before implementation;
  Christopher recalled approximately 33% proc chance but was not certain.
- Preserve the powerful item fantasy without retaining unavoidable Slow merely
  because players use it to counter powerful Planetars. Planetar balance is a
  separate issue, including differences between SR and non-SR setups.
- Prominently document the finished change on the website and in the item
  description/changelog; clearly distinguish it from vanilla behavior.

First assessment: a reasonable starting trade-off, not proven final balance.
Haste compatibility strengthens +5; a save weakens Slow. Repeated successful hits
give repeated opportunities to apply it, so -6 need not make it a weak effect.
Evaluate the combination rather than treating the Free Action change as neutral
QoL or judging only one hit. Verify the chosen save penalties actually reach the
installed resources and cover every +4 variant.

Before coding: identify effective item/effect owners, current proc chance,
duration, refresh/stacking behavior and immunity handling. Check existing tweaks
before duplicating a Free Action fix. Cover the default EET/SR/SCS stack and the
non-SR route; preserve unrelated effects and generate text locally through
WeiDU. No copying another installation's TLK or editing saves. Do not promise
hotpatch eligibility before examining equipped/saved effects.

## 2. Wider Free Action / Haste policy — separate discussion

Christopher is also interested in allowing beneficial Haste through Free Action
more generally, where the protection promises immunity to harmful impairment.
Optional versus default and exact coverage remain **undecided**. The Flail
decision must not silently authorize a global sweep of all movement effects.

- Identify an existing compatible tweak first; distinguish spells, equipment and
  other Free Action sources. Preserve genuine Hold/Slow/immobilization protection.
- Christopher observes that cleric Free Action appears to allow Haste in his
  current setup. Its cause is unverified; do not attribute it to SR by assumption.
- Removing the Flail's Free Action entirely was floated as an alternative for
  discussion, not chosen in place of the retained-protection design above.

## 3. Dispelling Screen — research and design discussion

**Primary owner:** the Spell Revisions patch project. Coordinate later with the
CRPG script-engine / SCS AI work; that project's implementation state was not
checked here. **Default, exact mechanic and release target:** undecided.

Reported concern: a high-level bard/cleric can apparently consume the Screen
with one Remove Magic and strip the remaining dispellable buffs with the next.
Christopher reports Entropy Shield as a possible exception and says sequencers
can produce similar pressure against the party. Reproduce and inspect before
classifying anything as a bug; these are not universal verified claims.

**Later September 28 clarification: establish correctness first.** Christopher
has seen several apparent cases of Remove/Dispel Magic stripping buffs while
Dispelling Screen was still active. This is a suspected bug, not a confirmed
diagnosis. Before redesigning the spell, establish its intended behavior and
reproduce a single dispel versus near-simultaneous separate dispels, checking
actual protection rather than only an icon. A correctness fix may be enough.

Alternative proposals remain open and must not be combined by assumption:

1. Fix only any demonstrated failure of the intended Screen behavior.
2. After consumption, add a short **approximately 0.5–1 gameplay second** grace
   period to protect against near-simultaneous double/triple dispels. Duration
   is tentative; this is distinct from the earlier full-round proposal.
3. Retain the earlier, stronger lingering-protection alternatives below.
4. Exclude both Remove Magic and Dispel Magic (including their SR equivalents)
   from **all sequencers, spell triggers and contingencies**. Discuss separately
   from Screen changes. If chosen, cover player selection and enemy/AI delivery,
   including already-authored combinations; hiding a spell in one picker alone
   would not establish complete coverage. No feasibility or implementation claim.

Christopher's concern is high-level dispel spam stripping defenses in one burst,
on either side of combat. The level-40 bard example illustrates that concern;
it is not a verified assertion that every enemy or protection is vulnerable.
No variant is selected, and no research or implementation is requested tonight.

Earlier proposal: when Screen is consumed, briefly retain protection from
subsequent dispels. These alternatives are NOT equivalent:

- Until the end of the relevant combat round (whose round boundary needs defining).
- A fixed **six gameplay seconds from consumption**.

This changes a one-attempt defense into temporary protection from multiple
attempts. Assess both sides of combat. Settle which attacks consume it, exactly
which removals the lingering protection blocks, and what can still counter it.
Separate Remove/Dispel Magic from Breach, dedicated spell-protection removal,
scripted cleanup and other removal mechanics; do not grant broad immunity by
accident. Check Entropy Shield interaction instead of assuming its implementation.

Guardrails to discuss: additional blocked dispels should not endlessly restart
the grace period; define recasting/stacking and expiry; keep AI detection honest
while the lingering protection is active. AI should have sensible alternative
actions rather than waste successive dispels. Inspect SCS source-level detection
and generated behavior before deciding whether existing AI can remain unchanged
or an adaptation is necessary. This can be an early AI-integration candidate
later, not a reason to start a larger rewrite now.

Eventual focused checks should cover single and chained dispels, a sequencer,
expiry/recast, party/enemy symmetry and the installed default stack. No live
test or claim of engine feasibility was made for this capture.

## 4. Entropy Shield — lower-priority balance discussion

Christopher considers the spell exciting to use and fight against, but its
combined benefits excessive. Preserve that identity while discussing whether
some benefits should be reduced or removed; no specific nerf, numbers, default
or release target selected. Keep separate from the Dispelling Screen bug report.

Route initially to the SR patch discussion, verifying the effective spell's
provider before assigning implementation. Its interaction with dispels needs
checking, not assuming. A bounded search of collection planning/handover notes
found mentions in the Screen proposal but no separate Entropy Shield balance
entry; check the owning project's existing work before creating duplicate tasks.
**Low priority; capture only tonight.**
