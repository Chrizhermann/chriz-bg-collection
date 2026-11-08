# CD_IMOBG1 — components

Listed at release version **1.0.2** (ChrizFader's voicepacks v1.2.0).
1 entries, 1 installed. ✓ = installed. Subgroup = choose one.

## Integration note

A player voicepack writes its selected registration regardless of what `charsnd.2da` holds:
one selectable player-character soundset column, its `M_cdsnd.lua` menu name and its sound files.
Its audio is unchanged and it changes no NPC,
item or spell. It requires the EE 2.6 soundset layout (99 `charsnd.2da` rows) and installs
on BG:EE, SoD, BG2:EE and EET. The collection installs only `cd_imobg1/` with its pinned
WeiDU, never the bundled `setup-cd_imobg1.exe`, in its own run after EET_end and before
BuffBot. Christopher's decision (2026-10-22): every voicepack is a default under
"Voicepacks".

The reference install contained an older local build; this catalog lists the published release.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Imoen (BG1) Soundset | Voicepacks |  | ✓ | default |
