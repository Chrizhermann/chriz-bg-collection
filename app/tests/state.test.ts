import { describe, expect, it } from "vitest";

import { initialState, reduce } from "../src/state";

describe("installer state", () => {
  it("starts at Welcome and cannot reach Build without a frozen review", () => {
    const state = initialState();

    expect(state.route).toBe("welcome");
    expect(reduce(state, { type: "continue" }).route).toBe("welcome");
  });

  it("can continue to Build after the review is frozen", () => {
    const state = {
      route: "welcome",
      frozenReview: { digest: "fixture-review-digest" },
    } as const;

    expect(reduce(state, { type: "continue" }).route).toBe("build");
  });
});
