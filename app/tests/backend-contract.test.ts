import { describe, expect, it } from "vitest";

import { FixtureBackend } from "../src/backend";

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
