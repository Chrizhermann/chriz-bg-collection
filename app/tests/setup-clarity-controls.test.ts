// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { fireEvent, getByRole } from "@testing-library/dom";
import type { FeatureControl, InputControl, SelectionEvaluation } from "../src/contracts";
import { featureInputs } from "../src/components/feature-inputs";
import { setupScreen, type SetupViewState } from "../src/screens/setup";
import { initialState, reduce } from "../src/state";
import { saveSetupDraft, loadSetupDraft } from "../src/setup-draft";

const id = "feature:randomiser:component-510";
function control(inputs: InputControl[] = []): FeatureControl {
  return { id, title: "Chance to leave an item in place", description: "Set the percentage chance.", category: "randomisation", decision: "optional", readiness: "ready", parent: "mod:randomiser", selected: true, interactive: true, unavailableReason: null, inputs };
}
const numberInput: InputControl = { spec: { kind: "integer", id: "chance-percent", min: 0, max: 100, default: 10 }, value: { kind: "integer", value: 10 }, interactive: true };
function evaluation(): SelectionEvaluation {
  return { view: { categories: ["randomisation"], controls: [control([numberInput])] }, normalizedSelection: { platform: "windows", features: { [id]: true }, inputs: {} }, findings: [], plan: { phases: [] }, selectedChoiceCount: 1 };
}
afterEach(() => { document.body.replaceChildren(); localStorage.clear(); });

describe("typed Customize inputs", () => {
  it.each([0, 10, 100])("sends and saves the integer %s without treating zero as empty", value => {
    const changed = vi.fn();
    const page = setupScreen(evaluation(), () => {}, () => {}, () => {}, { search: "", category: "", advancedOpen: true }, { change: () => {}, reset: () => {}, input: changed });
    document.body.append(page);
    const field = getByRole(page, "spinbutton", { name: /Chance to leave/ });
    fireEvent.change(field, { target: { value: String(value) } });
    expect(changed).toHaveBeenCalledWith(id, "chance-percent", { kind: "integer", value }, `input-${id}-chance-percent`);
    const state = reduce(initialState(), { type: "set-input", featureId: id, inputId: "chance-percent", value: { kind: "integer", value } });
    saveSetupDraft({ version: 1, recipeVersion: "preview", profileId: null, installationName: "Preview", destinationPath: "", destinationAutomatic: true, sourcePaths: { bg1: null, bg2: null }, selection: state.selection, createDesktopShortcut: false });
    expect(loadSetupDraft({ mode: "native", recipeVersion: "preview", engineVersion: "preview", startupInstallId: null }).draft?.selection.inputs[id]["chance-percent"]).toEqual({ kind: "integer", value });
  });
  it.each(["", "-1", "101", "1.5"])("rejects %j with a visible explanation and prevents Done", value => {
    const changed = vi.fn();
    const page = setupScreen(evaluation(), () => {}, () => {}, () => {}, { search: "", category: "", advancedOpen: true }, { change: () => {}, reset: () => {}, input: changed });
    document.body.append(page);
    const field = getByRole(page, "spinbutton");
    fireEvent.input(field, { target: { value } });
    fireEvent.change(field);
    expect(changed).not.toHaveBeenCalled();
    expect(getByRole(page, "alert").textContent).toContain("whole number from 0 to 100");
    expect((getByRole(page, "button", { name: "Done" }) as HTMLButtonElement).disabled).toBe(true);
    fireEvent.change(field, { target: { value: "25" } });
    expect((getByRole(page, "button", { name: "Done" }) as HTMLButtonElement).disabled).toBe(false);
  });
  it("keeps omitted inputs visible but uneditable and never dispatches them", () => {
    const changed = vi.fn();
    const root = featureInputs(control([{ ...numberInput, interactive: false }]), changed);
    document.body.append(root);
    const field = getByRole(root, "spinbutton") as HTMLInputElement;
    expect(field.disabled).toBe(true);
    expect(field.value).toBe("10");
    fireEvent.change(field, { target: { value: "20" } });
    expect(changed).not.toHaveBeenCalled();
  });
  it("uses typed boolean and choice values", () => {
    const changed = vi.fn();
    const root = featureInputs(control([
      { spec: { kind: "boolean", id: "keep-items", default: false }, value: { kind: "boolean", value: false }, interactive: true },
      { spec: { kind: "choice", id: "style", default: "small", options: [{ id: "small", title: "Small", answer: "1" }, { id: "big", title: "Large", answer: "2" }] }, value: { kind: "choice", value: "small" }, interactive: true },
    ]), changed);
    document.body.append(root);
    fireEvent.change(getByRole(root, "checkbox"), { target: { checked: true } });
    fireEvent.change(getByRole(root, "combobox"), { target: { value: "big" } });
    expect(changed.mock.calls.map(call => call[2])).toEqual([{ kind: "boolean", value: true }, { kind: "choice", value: "big" }]);
  });
});

describe("search and category explanations", () => {
  it("opens search matches temporarily and restores a collapsed category after clearing", async () => {
    const state: SetupViewState = { search: "", category: "", advancedOpen: true, categoryOpen: { randomisation: false } };
    const page = setupScreen(evaluation(), () => {}, () => {}, () => {}, state, { change: () => {}, reset: () => {} });
    document.body.append(page);
    const category = page.querySelector<HTMLDetailsElement>("details[data-category='randomisation']")!;
    const search = getByRole(page, "searchbox");
    expect(category.open).toBe(false);
    fireEvent.input(search, { target: { value: "chance" } });
    expect(category.open).toBe(true);
    await new Promise(resolve => setTimeout(resolve, 0));
    expect(state.categoryOpen?.randomisation).toBe(false);
    fireEvent.input(search, { target: { value: "" } });
    expect(category.open).toBe(false);
    expect(page.textContent).toContain("including choices hidden by your search");
    expect(page.textContent).toContain("not a recommendation");
  });
  it("does not call conditional required components always included", () => {
    const e = evaluation();
    const page = setupScreen({ ...e, view: { ...e.view, controls: [{ ...control(), decision: "mandatory" }] } }, () => {}, () => {}, () => {});
    expect(page.textContent).toContain("Required when applicable");
    expect(page.textContent).not.toContain("Always included");
  });
});
