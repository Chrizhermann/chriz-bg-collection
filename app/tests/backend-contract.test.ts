import { describe, expect, it, vi } from "vitest";

import {
  BackendCommandError,
  FixtureBackend,
  NativeBackend,
  type EventChannelFactory,
  type InvokeCommand,
} from "../src/backend";
import type { RunEventEnvelope } from "../src/contracts";

describe("engine-shaped fixture backend", () => {
  it("returns semantic recipe controls without exposing WeiDU component numbers", async () => {
    const backend = new FixtureBackend();
    const evaluation = await backend.evaluateBuild({
      platform: "windows",
      features: {},
      inputs: {},
    });

    expect(evaluation.view.controls.map((control) => control.decision)).toEqual(
      expect.arrayContaining(["default", "optional", "mandatory"]),
    );
    expect(evaluation.view.controls.map((control) => control.readiness)).toEqual(
      expect.arrayContaining(["ready", "experimental", "blocked"]),
    );
    expect(evaluation.view.controls.every((control) => "parent" in control && "inputs" in control)).toBe(true);
    expect(JSON.stringify(evaluation)).not.toMatch(/component[_-]?(number|id)|components/i);
    expect(evaluation.plan.phases).toHaveLength(5);
    expect(evaluation.plan.phases.at(-1)?.title).toBe("Finalization and reviewed tail");
  });

  it("models every source freshness finding and keeps the game selectors independent", async () => {
    const discovery = await new FixtureBackend().discoverGames();

    expect(discovery.bg1Candidates.map((candidate) => candidate.freshness)).toEqual(
      expect.arrayContaining(["fresh", "modified", "unsupported-version", "missing-sod"]),
    );
    expect(discovery.bg2Candidates.map((candidate) => candidate.freshness)).toContain(
      "unverified-storefront",
    );
    expect(discovery.selectedBg1Id).not.toBe(discovery.selectedBg2Id);
  });

  it("exposes fixture snapshots for manual attention, failure, retry, diagnostics, and completion", async () => {
    const backend = new FixtureBackend();

    expect((await backend.getBuildSnapshot()).state).toBe("waiting-manual");
    expect((await backend.advanceBuild()).state).toBe("attention");
    expect((await backend.advanceBuild()).state).toBe("failed");
    expect((await backend.retryBuild()).state).toBe("running");
    expect((await backend.advanceBuild()).state).toBe("complete");
    expect((await backend.exportDiagnostics()).path).toMatch(/diagnostics/i);
  });
});

describe("native command adapter", () => {
  it("maps the four read-only command contracts without exposing component numbers", async () => {
    const selection = { platform: "windows", features: {}, inputs: {} } as const;
    const invoke = vi.fn<InvokeCommand>(async (command, args) => {
      switch (command) {
        case "bootstrap":
          return { mode: "native", engine_version: "0.1.0", recipe_version: null };
        case "discover_games":
          return {
            bg1_candidates: [{ id: "bg1", label: "BG:EE + SoD — Steam — clean", path: "C:\\BGEE", storefront: "steam", build: "2.7.3.0", freshness: "fresh", eligible: true, findings: ["Clean."] }],
            bg2_candidates: [{ id: "bg2", label: "BGII:EE — Steam — modified", path: "C:\\BG2EE", storefront: "steam", build: null, freshness: "modified", eligible: false, findings: ["Modified."] }],
            selected_bg1_id: "bg1",
            selected_bg2_id: "bg2",
          };
        case "inspect_game_path":
          expect(args).toEqual({ role: "bgee_sod", path: "C:\\Chosen BGEE" });
          return { id: "chosen", label: "BG:EE + SoD — Steam — clean", path: "C:\\Chosen BGEE", storefront: "steam", build: "2.7.3.0", freshness: "fresh", eligible: true, findings: ["Clean."] };
        case "evaluate_build":
          expect(args).toEqual({ selection });
          return {
            view: {
              categories: ["Foundation"],
              controls: [{
                id: "foundation",
                title: "Foundation",
                description: "Required collection foundation.",
                category: "Foundation",
                decision: "mandatory",
                readiness: "ready",
                parent: null,
                selected: true,
                interactive: false,
                unavailable_reason: null,
                inputs: [],
              }],
            },
            normalized_selection: { platform: "windows", features: { foundation: true }, inputs: {} },
            findings: [{ rule: "fixture", feature_id: "foundation", message: "Visible finding." }],
            plan: { phases: [{ id: "main", title: "Build the main campaign", detail: "Install the selected main recipe." }] },
            selected_choice_count: 1,
          };
        default:
          throw new Error(`Unexpected command ${command}`);
      }
    });
    const backend = new NativeBackend(invoke);

    await expect(backend.getStatus()).resolves.toEqual({ mode: "native", engineVersion: "0.1.0", recipeVersion: null });
    const discovery = await backend.discoverGames();
    expect(discovery).toMatchObject({ selectedBg1Id: "bg1", selectedBg2Id: "bg2" });
    expect(discovery.bg1Candidates[0]).toMatchObject({ storefront: "steam", build: "2.7.3.0" });
    expect(discovery.bg1Candidates[0]?.storefront).toBe("steam");
    expect(discovery.bg1Candidates[0]?.build).toBe("2.7.3.0");
    await expect(backend.inspectGamePath("bgee_sod", "C:\\Chosen BGEE")).resolves.toMatchObject({ id: "chosen", eligible: true });
    const evaluation = await backend.evaluateBuild(selection);
    expect(evaluation.view.controls[0]?.unavailableReason).toBeNull();
    expect(evaluation.normalizedSelection.features.foundation).toBe(true);
    expect(evaluation.findings[0]?.featureId).toBe("foundation");
    expect(evaluation.selectedChoiceCount).toBe(1);
    expect(JSON.stringify(evaluation)).not.toContain("components");
  });

  it("preserves the serialized recovery contract from rejected native commands", async () => {
    const backend = new NativeBackend(async () => {
      throw {
        code: "game_profiles_failed",
        message: "Game detection is not ready.",
        recovery_action: "Install a verified game-profile release.",
        technical_detail: "manifest/game-builds is missing",
      };
    });

    const error = await backend.discoverGames().catch((caught: unknown) => caught);

    expect(error).toBeInstanceOf(BackendCommandError);
    expect(error).toMatchObject({
      code: "game_profiles_failed",
      recoveryAction: "Install a verified game-profile release.",
      technicalDetail: "manifest/game-builds is missing",
    });
  });

  it("maps destination, frozen review, detached events, snapshots, and scoped controls", async () => {
    const selection = { platform: "windows", features: {}, inputs: {} } as const;
    const events: RunEventEnvelope[] = [];
    type TestChannel = { emit: (event: unknown) => void };
    const channelFactory: EventChannelFactory = (onMessage) => ({ emit: onMessage });
    const invoke = vi.fn<InvokeCommand>(async (command, args) => {
      switch (command) {
        case "inspect_destination":
          expect(args).toEqual({ path: "D:\\Campaign", bg1CandidateId: "bg1", bg2CandidateId: "bg2" });
          return { path: "D:\\Campaign", safe: true, title: "Ready", detail: "Isolated." };
        case "freeze_review":
          expect(args).toEqual({ selection, destination: "D:\\Campaign", bg1CandidateId: "bg1", bg2CandidateId: "bg2" });
          return {
            review_token: "review-opaque",
            digest: "11".repeat(32),
            destination: "D:\\Campaign",
            game_labels: ["BG1", "BG2"],
            evaluation: {
              view: { categories: [], controls: [] },
              normalized_selection: selection,
              findings: [],
              plan: { phases: [] },
              selected_choice_count: 0,
            },
          };
        case "start_build":
          expect(args?.reviewToken).toBe("review-opaque");
          (args?.onEvent as TestChannel).emit({
            run_id: "run-1",
            sequence_as_string: "1",
            event: { type: "campaign_started", install_id: "install-1", resumed: false },
          });
          return { run_id: "run-1" };
        case "get_run_snapshot":
          expect(args).toEqual({ runId: "run-1" });
          return { run_id: "run-1", status: "running", events: [], report: null, error: null };
        case "continue_waiting":
        case "cancel_run":
          expect(args).toEqual({ runId: "run-1" });
          return null;
        default:
          throw new Error(`Unexpected command ${command}`);
      }
    });
    const backend = new NativeBackend(invoke, channelFactory);

    await expect(backend.inspectDestination("D:\\Campaign", "bg1", "bg2")).resolves.toMatchObject({ safe: true });
    const review = await backend.freezeReview(selection, "D:\\Campaign", "bg1", "bg2");
    expect(review.reviewToken).toBe("review-opaque");
    await expect(backend.startBuild(review.reviewToken, (event) => events.push(event))).resolves.toEqual({ runId: "run-1" });
    expect(events[0]).toMatchObject({ runId: "run-1", sequenceAsString: "1", event: { type: "campaign_started" } });
    await expect(backend.getRunSnapshot("run-1")).resolves.toMatchObject({ runId: "run-1", status: "running" });
    await expect(backend.continueWaiting("run-1")).resolves.toBeUndefined();
    await expect(backend.cancelRun("run-1")).resolves.toBeUndefined();
  });

  it("still fails closed for deferred native launch and diagnostics operations", async () => {
    const backend = new NativeBackend(async () => {
      throw new Error("No connected command should be invoked");
    });

    await expect(backend.exportDiagnostics()).rejects.toMatchObject({
      code: "command_not_available",
    });
  });
});
