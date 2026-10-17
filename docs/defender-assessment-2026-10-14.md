# Defender assessment — October 14, 2026

The owner chose to investigate the reported detections first and select any
Windows signing provider later. WeiDU's helper detection and CEBG's September 30
behavior detection are separate cases. Neither has an express Microsoft
false-positive clearance in the evidence checked today.

## Fresh file checks

Defender reported platform **4.18.26090.9**, engine **1.1.26090.9** and security
intelligence **1.461.20.0**. Signature update exited 0 with no updates needed.
Antivirus, real-time, behavior-monitor and downloaded-file protection were all
enabled. Custom scans used `-ScanType 3 -DisableRemediation`; the latter makes the
diagnostic scan report findings without scan-triggered remediation, not disable
real-time protection. No protection preferences or exclusions were changed.

| Input | SHA-256 | Custom scan result |
| --- | --- | --- |
| Published alpha.21 setup, 6,101,746 bytes | `c2a725a80409505c46d5724da50c27df7a7e58001fd74d0c2f999f86bb9527ed` | No threats, exit 0 |
| September 30 Dragon and Order Fix setup, 6,025,583 bytes | `3d94c594470486030e3792f94c22ef42bc7cc68e9b2894969fb3c653ace8c6f9` | No threats, exit 0 |
| Fresh official WeiDU 249 Windows x64 ZIP, 2,154,755 bytes | `b156910cbec69359fc2e42f6739aa959d49047d6fd3dc172f6bed88ffad8f927` | `Trojan:Win32/Pomal!rfn` on `WeiDU-Windows/weinstall.exe`, exit 2 |

Both setup hashes match their existing acceptance records:
[alpha.21](package-acceptance-alpha21-2026-10-14.md) and
[September 30 candidate](package-acceptance-alpha19-dragon-order-2026-09-30.md).
These are file-scan results. No setup, app, game or loader was launched by this assessment;
they do not clear the original runtime detection.

The WeiDU ZIP was downloaded from the
[official v249.00 release asset](https://github.com/WeiDUorg/weidu/releases/download/v249.00/WeiDU-Windows-249-amd64.zip).
Its complete size and hash match the existing recipe pin. Reading the member
inside that archive yielded **324,096 bytes**, SHA-256
`19619576bd03353191c0ac4bcdf9515e26ab216ad1bca03396d588e62c533fa0`.
The member was not extracted, executed or restored from quarantine. This proves
the detection reproduces on the pinned upstream archive; it does not recover or
prove the exact bytes of the previously quarantined file.

## Source and preparation findings

CEBG already publishes only `weidu.exe` from the WeiDU archive. The
[recipe pin](../manifest/artifacts/weidu-249-amd64.toml) verifies both the full
archive and the selected executable. The
[extractor](../engine/src/acquire/archive.rs) validates archive structure before
extracting approved roots, with
[tests for omitted executables](../engine/tests/acquire_archive.rs). No CEBG
runtime use of `weinstall.exe` was found. A separate, one-off
`prepare_install.initial.py` preparation script extracted the whole archive;
that script and its prepared/live folders were not changed.

The bounded first-party source audit found no Defender-setting changes,
exclusion creation or protection-disabling behavior. It did identify ordinary
shell opening, process queries, verified game/overlay launch and signed updater
operations; none is established as the September 30 trigger. No speculative
executable changes or security exceptions were introduced.

## Microsoft analysis and remaining scope

An authenticated portal check found the existing CEBG submission and its
diagnostic supplement still **In progress**, alongside a generic analyst reply
reporting no detection. The original uploaded sample was a different, newer
Selection Fix executable; the supplement had already corrected the relevant
hash. That reply is not express clearance of the originally quarantined app.
No new CEBG submission was made today. The official WeiDU archive was submitted
at the owner's request on October 14 at **22:44:30 KST**. The portal confirmed
submission and displays an automated detection; an analyst decision is pending.
The private submission record is retained for the owner. Automated labels are
not an analyst determination or a confirmed false-positive result.

The [approved October 7 reproduction](defender-smoke-2026-10-07.md) passed on the
October 1 Radar Fix candidate: installation, Play, details/path copying and
cleanup produced no new detection; Radar-off and approved Radar-on launches
passed. Its exact candidate, exclusion-inspection and Steam Cloud limits still
apply. Declining Radar consent remains untested, and the original detection's
precise cause remains unknown. No new runtime test or game/save change was made
for this assessment.

Tauri updater signatures authenticate CEBG updates; they do not provide Windows
Authenticode publisher identity. Microsoft's
[software developer guidance](https://learn.microsoft.com/en-us/defender-xdr/developer-faq)
supports consistent publisher signing, but signing does not guarantee that
Defender will accept a program or prevent behavior detections. Provider selection
and enrollment are deferred by the owner's choice. Next evidence is Microsoft's
analysis of the exact reported samples, not a broad folder exclusion.

Downloaded binaries and private diagnostics stay under ignored `target/` only.
This public note omits account details, submission identifiers, personal paths
and private logs. It does not authorize another live test or a new release.
