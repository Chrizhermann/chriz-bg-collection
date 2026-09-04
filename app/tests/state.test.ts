import { describe, expect, it } from "vitest";

import type { SelectionEvaluation } from "../src/contracts";
import { initialState, reduce } from "../src/state";

const evaluation = (count: number): SelectionEvaluation => ({
  view: { categories: [], controls: [] },
  normalizedSelection: { platform: "windows", features: {}, inputs: {} },
  findings: [],
  plan: { phases: [] },
  selectedChoiceCount: count,
});

describe("installer state", () => {
  it("keeps Build unreachable until Review has been frozen", () => {
    const state = initialState();
    expect(reduce(state, { type: "navigate", route: "build" }).route).toBe("welcome");

    const frozen = reduce(state, {
      type: "review-frozen",
      review: { reviewToken: "fixture-token", digest: "fixture", displayName: "Chriz Easy BG", destination: "D:\\Fixture", gameLabels: [], evaluation: evaluation(2) },
    });
    expect(reduce(frozen, { type: "navigate", route: "build" }).route).toBe("build");
  });

  it("rejects evaluation responses older than the latest selection revision", () => {
    const requested = reduce(initialState(), { type: "evaluation-requested", revision: 2 });
    const stale = reduce(requested, { type: "evaluation-resolved", revision: 1, evaluation: evaluation(99) });
    const current = reduce(stale, { type: "evaluation-resolved", revision: 2, evaluation: evaluation(2) });

    expect(stale.evaluation).toBeNull();
    expect(current.evaluation?.selectedChoiceCount).toBe(2);
  });
});
