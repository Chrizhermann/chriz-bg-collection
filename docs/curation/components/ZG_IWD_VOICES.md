# ZG_IWD_VOICES — components

Listed at release version **1.2.0** (GraionDilach/Icewind-Dale-Voice-Pack-For-EE-2.6; the
repository is archived). Not in the reference install.
1 entries, 0 installed. ✓ = installed.

## UI/dependency notes

- Christopher's decision (2026-10-23): the Icewind Dale Voice Pack is an optional,
  unchecked third-party mod, labelled experimental until the combined lab install, and
  offered as is. Recommended does not include it. Since 2026-10-25 (Christopher) it is in
  "Audio", with the other voice packs, instead of "Quest mods".
- It appends its 30 Icewind Dale player soundsets (`ZGI101`–`ZGI183`, menu names in its own
  `M_zgiwd.lua`) to `charsnd.2da` and keeps the table at 99 rows, so the CEBG voicepacks'
  preflight passes in either order. No soundset name collides with CEBG's voicepacks.
- The installer fails while `sod-dlc.zip` exists (TP2 L19). CEBG merges the SoD DLC in
  `bg1-preparation`, so this cannot trigger.
- Sounds are copied to `lang/en_us/sounds` (hard-coded); other game languages would get no
  audio.
- Open point for the lab: it uses Graion's soundset helper, whose EET row layout differs
  from CamDawg's. On EET probably only three of the seven "order acknowledged" lines of
  each set play; this is unverified in game. Graion's BG NPC Voice Pack in the reference
  install uses the same layout.

## Collection behavior

- Run `zg-iwd-voices-bg2` installs in the `main` phase directly after
  `hq-soundclips-bg2ee-bg2`: after EET and the DLC Merger, with no ordering constraint
  against the Item Randomiser or CEBG's voicepacks. Its ini says `After = EET`.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Icewind Dale Voice Pack for EE 2.6+ |  |  |  | optional |
