# Less companion chatter — deferred to the following release

Christopher approved the direction, then explicitly deferred implementation beyond
the currently prepared combined-test release. Do not rebuild or delay the current
installer for this feature. Resume the bounded plan below for the following version.

Christopher wants fewer unsolicited mod-NPC conversations and nonessential
interjections, closer to vanilla pacing. Some banter should remain. Do not cut
quest dialogue. He prefers a clearly visible, easily disabled default component.
This is a bounded source review and proposal, not an implemented component or a
change to the already-built September 29 installer.

## Recommended scope

- Owner: `chriz-bg-modpack`; CEBG later pins the published component and exposes
  a plain-language option such as **Less companion chatter**.
- Start with Fade and a small reviewed list of comment-only interjections.
  Preserve quests, friendship/romance progression, recruitment, rewards, plot
  actions and player-initiated talks. No global dialogue blocker or random cuts.
- Slow ambient/repeatable chatter only where it has an independent safe timer.
  Do not multiply every timer belonging to an NPC.
- Event-specific interjections cannot generally be delayed until later: the
  event has passed. Reducing these means intentionally omitting selected flavour
  comments, not promising that every original line remains available.
- Later add other companions only after the same per-NPC classification. Do not
  silently expand this into rewriting dialogue or judging all mod authors' prose.
- Christopher should approve the concrete reduction list/strength before edits.

## Bounded source evidence

Read-only local extracted archives under `C:\Games\Baldurs Gate 1 and 2 mods`:

- Fade v5.6: `fade/Scripts/E3Fade.baf:837-904` implements four sequential friendship
  talks with 3800/2600-second timers. Romance uses its own story progression.
  `Dialogues/FadeFlirt.d:1-29` is player-initiated, not an autonomous flirt loop.
  Do not slow these simply because the NPC feels talkative.
- Fade's `Dialogues/misc.d:579`, `:584`, `:1447` contain candidate comment-only
  interjections (`e3faderenaloffer`, `e3faderenalpcaccept`, `e3fadehendakhelp`).
  Their inserted lines contain no explicit plot actions, but copied transitions
  and later crossmod edits still need checking before suppressing them. Nearby
  lines 1461-1462 carry `Enemy()`: blanket removal would be unsafe.
- Evandra v2.2: `readme-evandra.html:70` documents a chapter-limited friendship
  track; `scripts/rh#eva.baf:246-259` implements the gate. `rh#EvaTalksTimer` is
  shared across friendship/romance, including ToB (`scripts/rh#ev25.baf:172`).
  Slowing it can make meaningful content easier to miss; preserve it.
- IEP v5.9: current recommended component 8 provides 20-minute extra banter
  attempts (`iepbanters.tp2`, `scripts/baldur.baf`). This affects ordinary party
  banter, not just mod NPCs; it neither changes their private story timers nor
  provides a global cooldown. A slower existing option is an independent choice,
  not a complete solution to the request. No default changed in this review.

These are source-shape findings, not live acceptance. Local extracted source
versions were identified but not re-hashed against their pinned archives during
this quick review. Implementation must use the pinned versions and verify the
effective dialogue transitions. No game, save, mod source or package was changed.
