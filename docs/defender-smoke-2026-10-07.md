# October 7 — Defender launch check

## Result

The October 1 Radar Fix candidate passed the bounded installation/Play/details
reproduction on October 7, approximately **05:54–06:04 KST**. Defender reported
no threats in either explicit file scan and no new detection/remediation events
during installation, game launch, the details/path-copy sequence, or cleanup.
This is local test evidence, not Microsoft clearance or a guarantee for other
machines/builds. It does not establish the original alert's precise cause.

Christopher explicitly authorized agent-controlled testing and cleanup today,
superseding the older user-clicks-only instructions for this check. Protection
was not disabled; no exclusions, quarantine restores, binary alterations or
security-prompt automation were used.

## Candidate and checks

- App **0.1.0-alpha.19**, bundled collection **0.1.0-alpha.18**; the October 1
  candidate documented in `package-acceptance-alpha19-radar-2026-10-01.md`.
- Setup SHA-256:
  `bde1e7d37524b8015e0b1b64d338be58b5bda7612fbc5b0dca7d698f8c384137`.
- Installed and extracted executable SHA-256:
  `d1850439a0eec13c4c8200f268df53a025f018e559718ad0473b024e1f42ad4f`.
  Installed executable remained byte-identical after the game test.
- Defender security intelligence **1.459.576.0**, platform **4.18.26080.4**,
  engine **1.1.26080.3**. Signature update completed successfully and reported
  no updates needed. Antivirus, real-time, behavior and downloaded-file
  protection were enabled before and after testing.
- Explicit custom scans of the exact setup and extracted app both returned
  **found no threats**, exit 0. `-DisableRemediation` was used only for these
  diagnostic scans: Microsoft documents that it ignores exclusions and reports
  findings in command output without scan-triggered remediation. It does not
  disable real-time protection. No protection preference was changed.
- The exact per-user NSIS setup installed with `/S`, exit 0. It reused a stale
  registered developer smoke path, not the assumed LocalAppData destination;
  the installed executable's identity was checked before it was launched.
- Selected completed **Chriz Easy BG Test 19**; receipt and both WeiDU logs
  matched its managed completion records. All 15 Radar payload files and the
  EEex archive/active loader files matched their recorded sources.
- With **Start Radar with the game unchecked**, clicked the actual CEBG Play
  button. The observed process chain was CEBG → Test19 InfinityLoader → Test19
  Baldur.exe. Game reached its main menu and Load Game menu without a startup
  error; no save was loaded, created, overwritten or deleted through the UI.
- While the game remained open, expanded CEBG Installation details, selected
  the install path and copied it. CEBG stayed responsive. Game ran for roughly
  four minutes, then exited through its normal confirmation.
- Defender events 1116/1117 were checked against the pre-test time boundary;
  **zero new events**, including a final check at **06:04 KST**. The baseline
  contained older detections; those were not mistaken for new failures.

## Limits and separate findings

1. **At the first check, Radar approval/decline was untested** (approval was
   subsequently tested in the follow-up below). A pre-existing elevated
   stream Radar process remained alive. Normal close was ineffective and process
   termination was access-denied; it was left alone. It may attach to any Baldur
   process, so this is not a fully Radar-free runtime comparison. No additional
   Radar process was launched by unchecked Play. Do not claim the existing
   overlay proves the new opt-in launcher works. No UAC prompt was invoked or
   acted on during this test.
2. **Existing exclusions could not be inspected** with the ordinary token:
   `-CheckExclusion` returned access denied. This limits conclusions about
   real-time scanning coverage; the explicit custom file scans ignored
   exclusions, and the protection-status checks succeeded.
3. **Local profile isolation does not isolate Steam Cloud.** Merely opening the
   game/menu downloaded 30 cloud-save copies into Test19, about 82.4 MB unpacked.
   All 30 cloud filenames/sizes matched the Steam app-257350 account cache;
   Test19's native UI calls `Infinity_UpdateCloudSaveState()`. Original three
   local save-folder timestamps remained unchanged; an interval conflict was
   imported with `_CLOUD_VERSION`. No claim is made about unobserved remote
   transfers. Do not use shared cloud saves for a future save/reload smoke.
4. A pre-existing October 6 Defender event concerns `weinstall.exe` in a separate
   BuffBot development directory (`Trojan:Win32/Pomal!rfn`). It predates this test,
   is not a fresh CEBG alert and was not executed/restored here. Any packaging
   work using that helper must investigate its provenance and status separately.

## Cleanup and release boundary

Test game/loader and CEBG were closed. The temporary installed app was removed
with its own silent uninstaller, exit 0; its exact developer smoke directory and
uninstall registration are gone. The candidate setup/signature, game installs,
diagnostics and backups remain. The old elevated Radar was not killed. Imported
cloud saves were deliberately retained, avoiding game/cloud deletion semantics.
No stream game or save content was edited by this test.

This candidate predates the newer October 6 owner-mod work. Passing this check
does not add physical-resistance/Hardiness/anti-magic/cloud/bracers changes or the
new BuffBot pin to its bundled recipe. Integrate ready owner releases, regenerate
and validate the recipe, rebuild, then check the exact final executable. Do not
publish the old package under a changelog claiming those additions.

Microsoft's same-day portal check still showed the generic no-detection reply,
with pending badges; this test sent no new submission or email. Publication is
not waiting for a Microsoft permission grant, but the remaining packaging scope
and test limits above must be reported accurately.

## Follow-up: approved Radar launch, 06:25–06:29 KST

Christopher closed the earlier Radar process and approved the new administrator
prompt. With no game or Radar running at baseline, the same candidate was
temporarily installed under `target/radar-smoke-oct07` (exit 0; executable hash
unchanged). Unchecked Play reached Test19's main menu with no Radar process.
After the game exited, checked Play launched Test19 and its own Radar executable.
CEBG displayed "Game and Radar started" and the Radar window appeared. No save
was loaded or manually saved/deleted. This verifies launch, not combat readings.

Physical Escape stopped computer control. The game and CEBG subsequently closed;
the elevated test Radar process was left alone. There were zero new Defender
1116/1117 events and all four protection flags remained enabled. The temporary
app was uninstalled with exit 0 and its directory was absent. Declining the
administrator prompt remains untested. The authorized release work continues;
no additional computer control is required to build and publish the package.
