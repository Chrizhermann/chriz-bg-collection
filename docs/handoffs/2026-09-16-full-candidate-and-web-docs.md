# Full candidate release and web documentation handoff

Status: **release-first preparation**, 2026-09-16. No source code, game files,
source pins, package, release, website download or live install was changed by this
handoff.

## Decided candidate scope

- Include all implemented candidates in the next new-install collection release.
  This does not invent owner delivery for unfinished work.
- Artisan and Bardic have completed user acceptance and are releasing through their
  owners now. Collection integration remains a separate, later verification.
- Include released SoD v0.6.10 component 115. The owner record verifies the
  reproduced Bridgefort entry/control issue and Adirran briefing, not a complete
  fort or campaign playthrough. The separate 257/266 check is waived as a gate,
  not passed.
- Include SR/RR compatibility. Set classic bouncing Lightning 80 as default; offer
  81 as the one mutually exclusive optional alternative.
- Include official Safana plus core 189. Christopher will test it on the candidate.
  Do not include unfinished Bard/Abettor conversion without a named owner delivery.
- Yeslick 188 is implemented at `f76b7fbf33d86c4c9501215a41529bd833ca0900` with
  53 passing tests and one skip. It still needs owner integration before 199/EET_end;
  no existing-save migration exists.
- Dragons 110/111 remain opt-in Challenge components, not defaults or compulsory
  full-install/pre-install combat gates.

## Required sequence

1. Verify only exact source/artifact provenance, selection validity and install order,
   including dependencies and mutually exclusive choices.
2. Freeze and build the actual packaged candidate. Christopher's latest instruction
   is that **he will run the installation and playtest himself**. Do not start an
   unattended install or additional review/playtest rounds while wrapping up.
3. Hand over the installer and a short list of genuinely outstanding feature checks.
   Record source checks, package verification and user gameplay acceptance separately.
4. Complete package/update/receipt/website consistency checks, then publish the new
   release. Do not promise a final count until the generated frozen recipe establishes it.
5. Only after the new-install release, implement opt-in existing-game patching with
   source/version checks, installation-local TLK handling, recorded patch state,
   backups and rollback eligibility. No generic copy-override operation is safe.

## Website boundary

The separately dispatched web task may build the interactive BG run page and prepare
or publish accurate documentation now. It must label upcoming candidate changes as
upcoming, preserve the current download until the new release is verified, and avoid
claiming that provisional alpha.16/alpha.14, 436/44, source pins, packages or full
integration are final. The Documents-folder case remains a recorded troubleshooting
and hardening handoff, not a diagnosed or implemented fix.

## Existing-install risk boundary

Existing-install work is intentionally later. Besides visibly broken saves, hazards
include silent stale quest state, cached CRE/area instances, residual effects,
installation-local TLK numbers and mod-version incompatibility. Any future patch
must fail closed for unknown state, preserve per-install backups and receipts, and
state rollback limits; it cannot safely migrate every save or undo later XP/ability
changes.
