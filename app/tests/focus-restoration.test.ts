// @vitest-environment jsdom
import { getByRole, queryByRole, waitFor } from "@testing-library/dom";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { mountApp } from "../src/app";
import { FixtureBackend } from "../src/backend";
import type { NormalizedSelection, SelectionEvaluation } from "../src/contracts";
import { loadSetupDraft } from "../src/setup-draft";

class FocusBackend extends FixtureBackend {
  requests: NormalizedSelection[] = [];
  constructor(readonly featureId: string) { super(); }
  override getStatus() {
    return Promise.resolve({ mode: "native" as const, engineVersion: "test", recipeVersion: "focus-test", startupInstallId: null });
  }
  override listManagedInstallations() { return Promise.resolve([]); }
  override async evaluateBuild(selection: NormalizedSelection): Promise<SelectionEvaluation> {
    this.requests.push(structuredClone(selection));
    const selected = selection.features[this.featureId] ?? false;
    return {
      view: { categories: ["rules"], controls: [{
        id: this.featureId, title: "Stronger SoD bridge mages", description: "Optional difficulty choice.",
        category: "rules", decision: "optional", readiness: "ready", parent: null,
        selected, interactive: true, unavailableReason: null, inputs: [],
      }] },
      normalizedSelection: { platform: "windows", features: { [this.featureId]: selected }, inputs: {} },
      findings: [], plan: { phases: [] }, selectedChoiceCount: Number(selected),
    };
  }
}

afterEach(() => { document.body.replaceChildren(); localStorage.clear(); vi.restoreAllMocks(); });

describe("focus after changing production-style component IDs", () => {
  it.each([
    "feature:chriz-sod-remix:component-257",
    "feature:example.v1:component-257",
  ])("retains the choice and focuses the exact in-app ID without an error: %s", async featureId => {
    const backend = new FocusBackend(featureId);
    const start = vi.spyOn(backend, "startBuild");
    // A document-wide getElementById would select this unrelated element first.
    const outside = document.createElement("input");
    outside.id = `feature-${featureId}`;
    const root = document.createElement("div");
    document.body.append(outside, root);
    const app = await mountApp(root, backend);
    await app.navigate("setup");
    root.querySelector<HTMLDetailsElement>(".setup-advanced")!.open = true;
    // The intentional duplicate ID interferes with label association, so locate
    // this sole checkbox by its scoped role rather than its accessible name.
    const checkbox = () => getByRole(root, "checkbox") as HTMLInputElement;
    for (const selected of [true, false]) {
      const before = checkbox();
      await userEvent.setup().click(before);
      await waitFor(() => {
        expect(backend.requests.at(-1)?.features[featureId]).toBe(selected);
        expect(checkbox()).not.toBe(before);
        expect(checkbox().checked).toBe(selected);
        expect(queryByRole(root, "alert")).toBeNull();
        expect(document.activeElement).toBe(checkbox());
      });
      expect(loadSetupDraft(await backend.getStatus()).draft?.selection.features[featureId]).toBe(selected);
    }
    expect(document.activeElement).not.toBe(outside);
    expect(start).not.toHaveBeenCalled();
  });
});
