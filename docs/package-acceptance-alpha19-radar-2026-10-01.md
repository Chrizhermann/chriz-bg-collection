# Alpha 19 Radar Fix candidate verification

**October 7 follow-up:** the exact candidate passed the authorized installation,
Play and details/path-copy Defender reproduction; see
[results and remaining limits](defender-smoke-2026-10-07.md). Radar-off and approved
Radar-on launch passed; declined consent remains untested. This does not change
the candidate's bundled mod versions.

The October 1 candidate is built and verified locally. It adds optional Radar
launching with Windows consent to app **0.1.0-alpha.19**, retaining collection
**0.1.0-alpha.18**. It is not published, manually launch-tested, or cleared by
Microsoft. The earlier Defender incident remains open.

## Local setup and identity

`%USERPROFILE%\Downloads\Chriz Easy BG alpha19 - Radar Fix 2026-10-01-setup.exe`

The matching updater signature is alongside it with `.sig` appended. The setup
is 6,015,101 bytes. Verified SHA-256 values:

- Setup: `bde1e7d37524b8015e0b1b64d338be58b5bda7612fbc5b0dca7d698f8c384137`
- Signature file: `be23a0a6ca08419b1f858e0bf5b32b093b74bea4ee5e36368ab496b37ce19a26`
- Extracted app: `d1850439a0eec13c4c8200f268df53a025f018e559718ad0473b024e1f42ad4f`

The Downloads setup/signature hashes were checked again after the job finished.
The Tauri signature was cryptographically verified against the bundled public
key. It is not an Authenticode publisher signature.

## Completed checks

Background job `d58c62d1-6a50-4121-9669-deb278861fa5` exited 0 at
2026-09-30 19:09:56 UTC / October 1 04:09:56 KST. Its status and both logs were
read after completion. Evidence remains local under
`target/work/oct01-release-candidate`; no diagnostic logs are bundled or published.

- Frontend: 396 tests passed across 34 files; TypeScript and production build passed.
- Rust workspace: 860 passed, zero failed, 14 pre-existing opt-in tests ignored.
- The package signature test was then explicitly enabled against the new setup:
  all four package-configuration/signature tests passed. The other 13 opt-in tests
  were not rerun by this job; they require selected artifacts, network or integration
  fixtures. This result is not a new live updater-download or game acceptance claim.
- Python: 138 tests reported, zero failed, one skipped (137 executed successfully).
  The skipped test requires explicitly selected real-WeiDU pilot artifacts.
- Rust formatting and Clippy with warnings denied passed.
- Dependency notices and pinned BG Rebalance native component order passed.
- NSIS packaging succeeded; 223 bundled resources matched the intended inputs.
  Extracted app comparison passed with only the known NSIS bundle marker normalized.
- Source hashes were unchanged throughout verification and packaging. The starting
  HEAD was `405491cd8632a73771dd7503fc6a509c7384be32`; the candidate also includes
  the approved working-tree changes recorded by `source-snapshot.json`. Do not
  describe the starting HEAD alone as the complete candidate source.

The local updater feed and checksums were prepared in
`target/work/oct01-release-candidate/release-metadata`. They point to the intended
future alpha.19 release; generating them does not publish the assets or channel.

## Remaining checks and publication

No setup, CEBG app, game or real Radar/UAC interaction was launched by this job.
Christopher owns the short Radar check: unchecked stays off; opting in asks for
Windows consent; approval actually opens Radar; declining leaves the game running
and gives a clear message. No repeat full installation is needed.

Keep the public release, updater feed, website and Discord post on hold until
the Defender incident is assessed and Christopher makes the release decision.
See [handover](handover.md) and the [minimal manual checklist](next-release-smoke-check.md).
