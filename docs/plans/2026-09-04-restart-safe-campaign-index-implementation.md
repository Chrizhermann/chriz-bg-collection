# Restart-safe Campaign Index Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers-extended-cc:executing-plans to implement this plan task-by-task.

**Goal:** Discover and resume interrupted managed builds after an application restart without accepting frontend paths or scanning folders.

**Architecture:** Add a create-once start index next to the existing create-once completion registry. Publish it only after ledger record zero verifies, then derive every resumable card and resume path by replaying that exact indexed ledger; completed records take precedence.

**Tech Stack:** Rust, serde, immutable campaign ledger, Tauri 2, vanilla TypeScript, Vitest.

---

### Task 1: Add the ledger-backed start index

**Files:**
- Modify: `engine/tests/registry.rs`
- Modify: `engine/src/registry.rs`

1. Add failing tests for create-once start publication, resumable replay, moved/corrupt ledger state, fresh-copy state, and identity conflicts.
2. Run `cargo test -p chriz-bg-engine --test registry` and confirm the new API is missing.
3. Add `managed-campaigns/<install-id>.json`, containing only schema, install id, canonical root, and recipe digest. Derive availability by replaying the indexed ledger.
4. Rerun the focused registry suite and commit the green slice.

### Task 2: Publish only after durable ledger identity

**Files:**
- Modify: `engine/tests/orchestrator.rs`
- Modify: `engine/src/orchestrator.rs`
- Modify: `engine/src/cli.rs`

1. Add a failing orchestrator test proving the lifecycle hook observes a replayable record-zero ledger before preflight.
2. Run the focused orchestrator test and confirm the hook does not exist.
3. Add the narrow dependency hook and implement it in the production CLI with the start index. Identical resume publication remains idempotent.
4. Run focused orchestrator/CLI tests and commit the green slice.

### Task 3: Resolve restarted resume through the index

**Files:**
- Modify: `app/src-tauri/tests/command_contract.rs`
- Modify: `app/src-tauri/src/bridge.rs`

1. Add a failing test that creates a ledger/index, reconstructs `NativeBridge`, and resumes by exact install id; unknown, stale, and fresh-copy ids fail closed.
2. Run `cargo test -p chriz-bg-app --test command_contract` and confirm restart resume fails as process-local-only.
3. Merge completed and started projections with completed precedence; add `resumable`; resolve `resume_build` only through a verified start record.
4. Run focused app tests and commit the green slice.

### Task 4: Expose the single Resume action

**Files:**
- Modify: `app/src/contracts.ts`
- Modify: `app/src/backend.ts`
- Modify: `app/src/screens/home.ts`
- Modify: `app/src/app.ts`
- Modify: `app/tests/backend-contract.test.ts`
- Modify: `app/tests/wizard.test.ts`

1. Add failing adapter/UI tests proving only `resumable` cards show Resume and that it invokes `resume_build` with the card id before entering Build.
2. Run the two focused Vitest files and confirm the action is absent.
3. Add the `resumable` projection and route the button through the existing build event flow; do not add new screens or styling.
4. Run `npm.cmd run check`, focused/full Rust checks, formatting, and diff checks; commit the green slice.
