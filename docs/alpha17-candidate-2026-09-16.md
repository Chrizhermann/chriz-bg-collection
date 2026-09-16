# Alpha.17 candidate — September 16

App **0.1.0-alpha.17**, collection **0.1.0-alpha.15**. This is a local candidate;
public installer/update-channel publication is separate.

## Scope

- Pin SoD Remix **v0.6.11**, fixing the component 115/197 conflict. Published ZIP:
  2,946,094 bytes, SHA-256
  `b2537d41424e53aa4c1ddd4c5d96e8d761644339adc649645215dcda8f856599`.
  Actual downloader/hash/extraction verification passed. No other mod selections
  or defaults changed: **448 components / 50 runs** with recommended settings.
  Dragons 110/111 and bridge sequencers 257 remain optional, adding three choices
  when explicitly enabled for Christopher's test.
- Include the already-implemented incremental download retry changes that were
  absent from the earlier alpha.16 binary.
- Fix technical-log expansion being lost during progress redraws. Mouse and
  keyboard activation persist; detached toggle events cannot overwrite current state.
- Remove forced full-height sizing from the scrollable grid child. Content that
  fits does not scroll; smaller windows and genuinely longer content still scroll.
- Make Customize categories independently collapsible with framed headers and
  readable labels. Disclosure state survives selection redraws.
- Add **Delete installation** to failed/incomplete and completed managed copies,
  with exact-folder confirmation. Missing folders offer entry-only forgetting.
  See [cleanup behavior and safeguards](plans/2026-09-07-managed-install-cleanup.md).
- Use a neutral unavailable-install heading: a failed build is not necessarily
  a missing folder.

## Verification

- 39 Python recipe/curation tests pass; public-alpha validation has zero findings.
  Effective default planning retains five previously documented conditional omissions.
- TypeScript checking and all **157 app tests** pass.
- **10 engine removal fixtures** and **1 native confirmation test** pass. Fixtures
  cover cancellation, source/cache/saves preservation, completed records, missing
  roots, active locks/processes, directory replacement, Windows junction refusal,
  partial deletion and retry after final metadata cleanup fails.
- Headless Chromium on isolated mock data: progress fits 1440x960 without overflow;
  760x600 retains accessible scrolling. Log click/Enter/Space survives redraw;
  category collapse persists. Confirmation has visible target and keyboard focus
  containment; Escape cancels without calling removal. No browser console errors.
- Rust Clippy passes for the engine/app and all targets with warnings denied.
  The existing recovery-receipt fixture was updated for the new optional
  `append_missing_components` field; its focused test passes.
- Windows x64 NSIS packaging passes. All four package checks pass, including
  verification of this setup's updater signature against the embedded public key.

## Local package

- User copy: `C:\Users\chris\Downloads\Chriz-Easy-BG-0.1.0-alpha.17-setup.exe`
- Build: `target/x86_64-pc-windows-msvc/release/bundle/nsis/Chriz Easy BG_0.1.0-alpha.17_x64-setup.exe`
- Size: **5,573,696 bytes**.
- SHA-256: `92d6d93ad8ce9422df59486d62ee0ee94f2dae9adff0b453fe0c9034d07115d7`.
- Local updater metadata: `target/cebg-release/0.1.0-alpha.17/`.
  Nothing was uploaded to a public release or update channel.
- The updater signature is not Windows Authenticode/code-signing approval.
  The setup was built and verified, not run on Christopher's machine.

## Test and recovery boundaries

Christopher chose to test a fresh installation after receiving this fixed version.
Do not perform the earlier proposed supervised repair or automatically remove the
failed copy. No real installation, source game, save, cache, or launcher shortcut
was deleted or modified by these checks. Live deletion acceptance and the next full
installation/playtest belong to Christopher and remain untested here.

This patch does not add automatic repair of sealed partial WeiDU batches. The SoD
owner has a proven targeted recovery approach, but ordinary app resume still must
not bypass the frozen-install safety checks. A public guarded repair flow remains
separate work. Existing-game hotpatching also remains separate.
