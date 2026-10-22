# Archive downloads: stall detection instead of a 30-second cap — 2026-10-22

Status: implemented and tested; **source only, not packaged/published**. Builds on the
[incremental archive retry](download-retry-acceptance-2026-09-16.md).

## Problem

The engine's HTTP agent set `timeout_global(30 s)`. In ureq 3.4.0 the global timeout
runs from DNS lookup to the end of the response body, so every archive download attempt
stopped 30 seconds after it started, even while bytes were still arriving. The same
client serves collection cache downloads (installs, patches, mod upgrades) and
`chriz-bg-author artifact inspect/verify`.

- codeload.github.com ignores `Range` (200, no `Content-Range` or `Accept-Ranges`), so a
  retry starts from zero. A whole archive had to arrive within one 30-second attempt:
  EET (201 MiB) needed about 56 Mbit/s, Infinity UI (244 MiB) about 68 Mbit/s, PPE
  (369 MiB, planned optional) more. Slower connections could never install, however
  often they retried.
- GitHub release assets resume (206, strong ETag, exact `Content-Range`), so progress
  survived, but BGGO (1.39 GB) still needed about 78 Mbit/s to finish within the five
  attempts of one run; slower users had to press Retry repeatedly.
- Reproduction: a loopback server streaming a 24 MiB zip at 600 KiB/s without Range
  support. `artifact inspect` failed after 30.1 s with `timeout: global` after 18.4 MB.

## ureq 3.4.0 timeout semantics

- Each phase timeout is checked from the start of every wait, plus the immediately
  preceding phase's timeout counted from that phase's recorded end, plus global and
  per-call. Only global and per-call cap the whole transfer.
- `timeout_recv_body` is checked anew before every wait for body input: a per-wait
  stall limit, not a total.
- `timeout_recv_response` also limits the body phase, counted from the end of the
  headers, so it would cap the body too. It is not set.
- `timeout_send_request` also bounds the wait for the response headers (preceding-phase
  rule). Without it, a server that accepts the request and never answers hangs forever.
- A wait that starts after a passed deadline gets one more second instead of failing at
  once. A total limit therefore fails only a wait it interrupts; the regression test
  paces its chunks so that the limit falls inside a wait.

## Behavior

- No global or per-call timeout: a download that keeps progressing has no total time
  limit.
- Connection setup and header wait stay bounded: resolve 30 s, connect 30 s,
  send-request 60 s (also the header wait). Named constants in
  `engine/src/acquire/http.rs`.
- Stall detection: an attempt fails when no body bytes arrive for 60 s. The failure is
  the same retryable transport error as before; retry schedule, resume (strong ETag,
  exact 206) and hash/length verification are unchanged.
- App self-update: no change was needed. tauri-plugin-updater 2.11.0 applies the
  15-second builder timeout only to the feed check and hands out the update with
  `timeout: None` (plugin fix #2572), so the setup download is not capped.

## Verification

PowerShell, Windows MSVC target, locked dependency graph, no external network in
automated tests.

- New engine tests (`acquire::cache` and `acquire::http` unit tests; loopback servers,
  300 ms injected limits through `HttpClient::with_timeouts`, about 1.3 s in total):
  a steady 1.2-second body downloads completely and verifies; a body that stalls after
  12,000 bytes fails within the stall window as a retryable `timeout: receive body`,
  keeps its partial, and the next attempt resumes with `Range`/`If-Range` and passes the
  hash check; a server that never sends headers fails within the header bound
  (`timeout: send request`); the production agents have no global, per-call or
  recv-response timeout and use the named 30/30/60/60-second constants.
- TDD: with the new constructor still mapping to a single global limit, all four new
  tests failed (`timeout: global`, or a 60 s global where none is allowed). Mutation
  checks after the fix: adding `timeout_recv_response`, `timeout_global` or
  `timeout_per_call` fails the steady test; removing `timeout_send_request` fails the
  header test; removing `timeout_recv_body` fails the stall test.
- App: new `a_slow_update_download_outlasts_the_feed_check_timeout` in
  `updater_download_acceptance.rs`: the real plugin checks a loopback feed with a
  1-second timeout, then downloads a freshly signed 60 KB setup streamed over 2.4 s and
  verifies its signature. Setting `update.timeout` to the check timeout makes it fail
  (`TimedOut`), so the test detects a reintroduced cap.
- `cargo test --locked --workspace --no-fail-fast`: **1,032 passed, 0 failed,
  16 ignored**. `cargo fmt --all -- --check`, `cargo clippy --locked --workspace
  --all-targets -- -D warnings` and `git diff --check` passed.
  `python -m unittest discover -s tools/tests`: **180 tests OK** (1 skipped).
- Loopback reproduction with the built `chriz-bg-author artifact inspect` (24 MiB zip at
  600 KiB/s, no Range): passed after 41.1 s (failed after 30.1 s before). The same
  server stalling after 2 MiB: failed after 63.5 s with `timeout: receive body`.
- Live, fresh cache, `chriz-bg-author artifact verify`: EET official (codeload,
  211,137,765 bytes) passed in 60.4 s; HQ Sound Clips BG2EE 1.3 (GitHub release asset via redirect, 206,363,285 bytes) passed
  in 65.2 s. Times include extraction and verification.

## Limitations

- codeload still cannot resume: a stall restarts that archive from zero on the next
  attempt.
- A connection that delivers a few bytes at least once a minute is never treated as
  stalled; there is no minimum speed.
- Follow-up, not fixed: the app's setup download (tauri-plugin-updater) has no stall
  detection. The plugin offers only a total timeout, so a setup download that stalls
  after the headers can hang indefinitely.
- Pause still acts at the acquisition step boundary, not during a network wait.
