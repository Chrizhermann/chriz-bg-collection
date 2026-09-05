# Optional EET SoD skip — owning-repository handoff

User request: early alpha/weekend convenience feature; not a blocker for tonight's
initial installer alpha. Owner: `Chrizhermann/chriz-sod-rebalance`, existing task
`019f6539-74ef-7260-93c3-00b63cee296a`. Handoff dispatched 2026-09-05.

## Exact requested flow

After Sarevok is defeated and the player is brought to the sleeping room, offer:

> Do you want to skip SoD? You will get 250000 experience on your main character if you skip.

- Yes -> confirm skipping -> confirm Yes grants +250,000 protagonist XP and skips onward;
  confirm No returns to the opening question.
- No -> confirm playing SoD -> confirm Yes continues normally without bonus;
  confirm No returns to the opening question.

The prompt/award is once-only. Do not replace additive XP with a set-to-250,000 total,
party-wide XP or party-divided XP. Repeated clicks, canceled confirmations and save/reload
must not duplicate the award or transition. Main character is not an arbitrary portrait slot.

## Bounded scope and checks

1. Inspect authoritative EET documentation/source and existing skip mods. Prefer a proven
   supported campaign transition over a bare teleport. Respect upstream licensing.
2. Verify the actual requested post-Sarevok bedroom hook. If that description crosses
   different EET/SoD sequence points, report the precise choice before changing timing.
3. Keep this separately selectable and distinct from component 290 or the existing
   30-component v0.6.5 bundle. Propose its component number/home; collection selection/pin
   changes wait for its own acceptance, never modify frozen r4.
4. Preserve necessary campaign globals and party/inventory transition behavior. Ask only
   if a meaningful unchosen story/inventory decision prevents a simple implementation.
5. TDD/synthetic checks plus a tiny disposable gameplay checklist covering both choices,
   both confirmation-cancel loops, exact once-only protagonist XP and save/reload.
6. Return tested commit, limitations and release readiness. No new publication is
   automatically authorized by this handoff. Never edit the user's stream game/saves.

The collection task continues installer acceptance and recovery work in parallel.

## Inventory choice approved (2026-09-05)

The owning task returned design commit `e17064c053bce8840e5014565fc6c3050fa940a0`
on `codex/optional-sod-skip`, with the proposal in
`docs/design/wave1/06-optional-sod-skip.md`. It reports the requested bedroom hook is
valid, but normal import only sees current party possessions and could miss the stored
backpacks, off-party imported Imoen's gear, and imported finale ground loot.

Christopher approved including those belongings in **EET's normal BG2 equipment-import
handling**. This is not permission to carry every item into BG2 or bypass import rules.
Verify actual EET behavior and ground-pile provenance; do not invent item destinations,
sweep unrelated room loot, or add SoD rewards, refunds or quest completion rewards.

Approval was sent to the owning task to resume bounded implementation and focused tests.
Proposed component 910 and requirements remain the owning task's design, not a released
or collection-selected component. No candidate code or live acceptance has been reported
yet. Published v0.6.5 and frozen collection r4 remain unchanged.
