// @vitest-environment jsdom
import { fireEvent, getByRole, waitFor } from "@testing-library/dom";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { mountApp } from "../src/app";
import { FixtureBackend } from "../src/backend";
import type { BackendStatus, FeatureControl, NormalizedSelection, SelectionEvaluation } from "../src/contracts";
import { loadSetupDraft } from "../src/setup-draft";

const featureId = "feature:randomiser:component-510";
const inputId = "chance-percent";

class PercentageBackend extends FixtureBackend {
  requests: NormalizedSelection[] = [];

  override getStatus(): Promise<BackendStatus> {
    return Promise.resolve({ mode: "native", engineVersion: "test", recipeVersion: "input-flow-test", startupInstallId: null });
  }

  override listManagedInstallations() { return Promise.resolve([]); }

  override async evaluateBuild(selection: NormalizedSelection): Promise<SelectionEvaluation> {
    this.requests.push(structuredClone(selection));
    const selected = selection.features[featureId] ?? false;
    const value = selection.inputs[featureId]?.[inputId] ?? { kind: "integer" as const, value: 10 };
    if (value.kind !== "integer" || !Number.isInteger(value.value) || value.value < 0 || value.value > 100) {
      throw new Error("Invalid percentage received by test backend");
    }
    const controls: FeatureControl[] = [
      { id: "mod:randomiser", title: "Item Randomiser", description: "Randomize items.", category: "randomisation", decision: "default", readiness: "ready", parent: null, selected: true, interactive: true, unavailableReason: null, inputs: [] },
      { id: featureId, title: "Chance to leave an item in place", description: "Set the percentage chance.", category: "randomisation", decision: "optional", readiness: "ready", parent: "mod:randomiser", selected, interactive: true, unavailableReason: null,
        inputs: [{ spec: { kind: "integer", id: inputId, min: 0, max: 100, default: 10 }, value, interactive: selected }] },
    ];
    return {
      view: { categories: ["randomisation"], controls },
      normalizedSelection: { platform: "windows", features: { "mod:randomiser": true, [featureId]: selected }, inputs: { [featureId]: { [inputId]: value } } },
      findings: [], plan: { phases: [] }, selectedChoiceCount: selected ? 2 : 1,
    };
  }
}

async function open() {
  const backend = new PercentageBackend();
  const start = vi.spyOn(backend, "startBuild");
  const root = document.createElement("div");
  document.body.append(root);
  const app = await mountApp(root, backend);
  await app.navigate("setup");
  root.querySelector<HTMLDetailsElement>(".setup-advanced")!.open = true;
  return { root, backend, app, start, user: userEvent.setup() };
}

function percentage(root: HTMLElement): HTMLInputElement {
  return getByRole(root, "spinbutton", { name: "Chance to leave an item in place (%)" }) as HTMLInputElement;
}

async function setPercentage(opened: Awaited<ReturnType<typeof open>>, value: number) {
  await opened.user.click(getByRole(opened.root, "checkbox", { name: "Chance to leave an item in place" }));
  await waitFor(() => expect(percentage(opened.root).disabled).toBe(false));
  const previousField = percentage(opened.root);
  fireEvent.change(previousField, { target: { value: String(value) } });
  await waitFor(() => {
    expect(opened.backend.requests.at(-1)?.inputs[featureId]?.[inputId]).toEqual({ kind: "integer", value });
    expect(percentage(opened.root)).not.toBe(previousField);
    expect(percentage(opened.root).value).toBe(String(value));
  });
}

afterEach(() => { document.body.replaceChildren(); localStorage.clear(); vi.restoreAllMocks(); });

describe("typed percentage through the application controller", () => {
  it.each([0, 37, 100])("sends %s to evaluateBuild and retains it after redraw and navigation", async value => {
    const opened = await open();
    expect(percentage(opened.root).value).toBe("10");
    expect(percentage(opened.root).disabled).toBe(true);
    await setPercentage(opened, value);
    await opened.user.click(getByRole(opened.root, "button", { name: "Done" }));
    await opened.app.navigate("setup");
    expect(percentage(opened.root).value).toBe(String(value));
    expect(percentage(opened.root).disabled).toBe(false);
    expect(opened.start).not.toHaveBeenCalled();
  });

  it("restores the saved input after remount and reset restores the default in the draft and on screen", async () => {
    const first = await open();
    await setPercentage(first, 37);
    expect(loadSetupDraft(await first.backend.getStatus()).draft?.selection.inputs[featureId]?.[inputId]).toEqual({ kind: "integer", value: 37 });
    first.root.remove();

    const second = await open();
    expect(second.backend.requests[0]?.inputs[featureId]?.[inputId]).toEqual({ kind: "integer", value: 37 });
    expect(percentage(second.root).value).toBe("37");
    expect(percentage(second.root).disabled).toBe(false);
    await second.user.click(getByRole(second.root, "button", { name: "Reset to Chriz’s setup" }));
    await waitFor(() => {
      expect(second.backend.requests.at(-1)?.inputs).toEqual({});
      expect(percentage(second.root).value).toBe("10");
      expect(percentage(second.root).disabled).toBe(true);
    });
    const resetDraft = loadSetupDraft(await second.backend.getStatus()).draft;
    expect(resetDraft?.selection.inputs[featureId]?.[inputId]).toEqual({ kind: "integer", value: 10 });
    expect(resetDraft?.selection.features[featureId]).toBe(false);
    expect(first.start).not.toHaveBeenCalled();
    expect(second.start).not.toHaveBeenCalled();
  });
});
