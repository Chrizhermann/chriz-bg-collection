# BG Radar Overlay Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers-extended-cc:executing-plans to implement this plan task-by-task.

**Goal:** Let Chriz Easy BG discover, download, verify, install, and update the latest BG Radar Overlay release inside a managed game without touching reference installs.

**Architecture:** A standalone engine module reads GitHub's Latest-release JSON, requires the release asset's advertised SHA-256, and reuses the content-addressed artifact cache for bounded downloads. It extracts the upstream 7z into a private staging directory, publishes files beneath `game/BG Radar Overlay`, and stores an exact file receipt beneath `.chriz/addons` so later updates replace only unchanged owned files and preserve user-created configuration.

**Tech Stack:** Rust, `ureq`, `sevenz-rust2`, SHA-256, Serde, existing `ArtifactCache` and `EventSink`.

---

### Task 1: Freeze release metadata and API behavior

**Files:**
- Create: `engine/tests/radar.rs`
- Create: `engine/src/radar.rs`
- Modify: `engine/src/lib.rs`

1. Write integration tests for GitHub Latest parsing, serializable status states, and malformed/missing digest rejection.
2. Run `cargo test -p chriz-bg-engine --test radar` and confirm the missing module/API fails.
3. Implement the bounded GitHub metadata fetch and release validation.
4. Re-run the focused test.

### Task 2: Install and safely update the add-on

**Files:**
- Modify: `engine/tests/radar.rs`
- Modify: `engine/src/radar.rs`
- Modify: `engine/Cargo.toml`
- Modify: `Cargo.lock`

1. Add failing tests using locally generated 7z archives and loopback HTTP.
2. Cover first install, update, preservation of unowned `config.cfg`, modified-owned-file refusal, and unmanaged collision refusal.
3. Implement verified acquisition, bounded/safe extraction, transactional owned-file publication, and the durable receipt.
4. Re-run the focused test after each red/green slice.

### Task 3: Verify and document provenance

**Files:**
- Create: `docs/research/bg-radar-overlay.md`

1. Record the authoritative repository, release/asset IDs, tag, lengths, hashes, upstream installation behavior, and installed archive layout.
2. Run focused tests, engine tests, formatting, and Clippy without claiming live game acceptance.
3. Leave all changes uncommitted for parent-agent integration, as explicitly requested.
