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
