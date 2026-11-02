# Incremental archive retry — 2026-09-16

Status: implemented and focused tests passed; **source only, not packaged/published**.
Christopher requested a fast retry, another short retry, then 10/30-second waits
instead of consuming three attempts back-to-back.

## Behavior

- Collection archive downloads and authoring verification now use five attempts:
  initial request, immediate retry, then retries after **2, 10, 30 seconds**.
  These delays are between failures, not absolute timestamps. Total added wait
  before final failure is 42 seconds, excluding network-transfer time.
- Successful requests return immediately. Non-retryable failures keep their existing
  behavior; missing releases, unsafe redirects and integrity failures are not
  ignored. Retry classification and all hash/length checks are unchanged.
- Technical log messages identify the artifact, next attempt/limit and wait duration.
  These are ordinary output events, not failure/attention banners.
- After exhaustion the existing Retry failed step/resume path remains available;
  completed verified archives are retained/reused. No automatic manual-download
  fallback or alternate unverified source is introduced.
- Radar retains its separately authored two-attempt budget. Explicit smaller test
  budgets still apply; additional caller-requested attempts cap each wait at 30s.
- Pause remains at the existing acquisition step boundary, not an interruptible
  network/backoff wait. This change does not claim immediate cancellation support.
  Server Retry-After handling/jitter is outside this bounded change.

## Verification

PowerShell, Windows MSVC target, locked dependency graph:

- Unit retry/cache tests: **5 passed**. Fake waits prove exact intervals, success at
  every attempt, fatal-error exit, no final sleep, smaller budgets and a 30s cap.
- HTTP acquisition: **10 passed**, using loopback fixtures. Includes 500 then
  recovery/cache reuse, 404 without retries, configured exhaustion, redirects and
  existing range/ETag resume cases.
- Cache tests: **3 passed** (hash rejection, concurrent publication, verified hit).
- CLI tests: **29 passed**, including durable resume, diagnostics and source guards.
- `git diff --check` passed. No live game installation, external download retry
  experiment, UI takeover or full suite was needed.

TDD: tests first failed because the retry helper did not exist, then passed after
implementation. Schedule tests never sleep; the three-attempt HTTP fixture waits
only two seconds. Existing alpha.16 setup/updater artifacts must be rebuilt before
claiming this fix is included.
