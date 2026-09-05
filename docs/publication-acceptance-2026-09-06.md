# CEBG first public alpha — publication evidence

Christopher authorized the finished alpha download on the collection website after
his Discord announcement. No promise was made that every machine or customization
is tested. The collection source/history remains private; the public distribution
repo is `Chrizhermann/chriz-easy-bg`.

## Published binary

- App: `0.1.0-alpha.10`; bundled recipe: `0.1.0-alpha.11`.
- Release: https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.10
- Asset: `Chriz.Easy.BG_0.1.0-alpha.10_x64-setup.exe` (GitHub normalizes spaces to dots).
- Exact size: 5,392,890 bytes.
- SHA-256: `02051c6d19708be3064ab05da5f3e5e03ba85620bb4670c90f4d851359718206`.
- Public setup and `.sig` were downloaded without GitHub credentials using normal
  HTTPS, then the digest and signature against the embedded updater key passed.
- Microsoft Defender custom file scan, remediation disabled: exit0, no threats.
  Antivirus enabled; signature timestamp at scan was 2026-09-05T15:01:00+09:00.
  This is a bounded scanner result, not a security guarantee.

The binary is updater-signed, not Authenticode-signed. It was not executed during
publication. Local signed-package evidence is in
`docs/package-acceptance-alpha10-2026-09-06.md`.

## Recipe/source boundary

- Includes accepted SoD Remix v0.6.7 / skip910 and exact official Evandra Windows
  manual acquisition, with early verification-or-skip.
- Public component/source JSON was derived from the effective recommended preset.
  Independent native CLI resolution matched all434 component IDs in43 runs, with
  zero differences. This is the authored install plan, not a newly installed log.
- No mod archives, private aggregate, private source history or signing keys were
  uploaded. Bundled runtime files exclude creator/reference TSVs and authoring
  inventories. MIT and UnRAR notices are bundled and supplied as release assets.
- No full install, game/save modification, native app control or rejected cleanup
  retry occurred. R5 remains the accepted existing gameplay installation.

## Publication follow-through

The first feed emitted URL-escaped spaces, but GitHub's returned asset name used
dots. The anonymous public download check caught the404 before website handoff.
The packaging metadata script was corrected with two passing PowerShell-invocation
regression tests; the signed binary was not rebuilt. Both versioned metadata and
fixed `alpha/latest.json` channel now name the actual dotted asset. An anonymous
fetch of the live channel followed its URL, downloaded the exact setup, and matched
the SHA-256 above. Feed appalpha10 and recipealpha11 values were checked.
Corrected feed SHA-256: `b76ff7abbfccac3b0a7e68d361ea07a859044f1a82ce5d77e020442ce285f49c`.

The website task received the exact verified binary URL/hash and public component
list, with authorization to deploy `/collection`. Website deployment/public-page
verification is pending at this checkpoint; binary and feed checks are complete. Native updater
apply/restart, safe-pause close behavior and every customization remain alpha
acceptance boundaries rather than claimed live tests.

Private source checkpoint `aa44060` is pushed on `codex/installer-v0-real-alpha`.
The fresh mock-harness loader failure is tracked in private collection issue2;
see `docs/issues/updater-harness-entrypoint.md`. Identical old/new static imports
and absence of mock markers in the release app did not establish an app-startup
defect; the earlier working harness and public signature/download tests passed.
