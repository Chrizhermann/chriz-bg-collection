// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";
import { fireEvent, getByRole, queryAllByRole } from "@testing-library/dom";
import type { FeatureControl, SelectionEvaluation } from "../src/contracts";
import { bulkCategoryChanges } from "../src/customization";
import { setupScreen } from "../src/screens/setup";

const infinity = "feature:game-interface:infinity";
const sod = "feature:game-interface:eet-sod";
const skill = "feature:stratagems:component-4115";
function control(id: string, extra: Partial<FeatureControl> = {}): FeatureControl {
  return { id, title: id, description: "Test description", category: "interface", decision: "optional", readiness: "ready", parent: null, selected: false, interactive: true, unavailableReason: null, inputs: [], ...extra };
}
function fixture(selected = ""): SelectionEvaluation {
  const controls = [
    control(infinity, { title: "Infinity UI++", description: "A redesigned interface. Community playtesting is still needed.", groupLabel: "Game interface", choiceGroup: "game-interface", choiceAvailable: true, conflicts: [sod], selected: selected === infinity }),
    control(sod, { title: "EET: SoD style", groupLabel: "Game interface", choiceGroup: "game-interface", choiceAvailable: true, conflicts: [infinity], selected: selected === sod }),
    control(skill, { title: "Thief skills in steps of five", decision: "default", selected: selected !== infinity, conflicts: [infinity] }),
  ];
  return { view: { categories: ["interface"], controls }, normalizedSelection: { platform: "windows", features: { [infinity]: selected === infinity, [sod]: selected === sod, [skill]: true }, inputs: {} }, findings: [], plan: { phases: [] }, selectedChoiceCount: controls.filter(c => c.selected).length };
}
function render(e = fixture()) {
  const changes: Record<string, boolean>[] = [];
  const page = setupScreen(e, () => {}, () => {}, () => {}, { search: "", category: "", advancedOpen: true }, { change: value => { changes.push(value); }, reset: () => {} });
  document.body.append(page);
  return { page, changes, select: getByRole(page, "combobox", { name: "Game interface" }) as HTMLSelectElement };
}
afterEach(() => document.body.replaceChildren());

describe("new-install game interface selector", () => {
  it("keeps one interface choice inside Customize's common changes, not a main install preset", () => {
    const { page, select } = render();
    expect(select.value).toBe("");
    expect(getByRole(select, "option", { name: "Default interface (Recommended)" })).toBeTruthy();
    expect(queryAllByRole(page, "combobox", { name: "Game interface" })).toHaveLength(1);
    expect(queryAllByRole(page, "checkbox", { name: "Infinity UI++" })).toHaveLength(0);
    expect(page.textContent).toContain("existing games stay unchanged");
    expect(select.closest(".common-choices")).not.toBeNull();
    expect(page.querySelector(":scope > .game-interface-choice")).toBeNull();
    expect(page.querySelector(".game-interface-choice")!.compareDocumentPosition(page.querySelector(".setup-advanced")!) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  });
  it("puts the common mod switches before the interface row", () => {
    const e = fixture();
    const { page } = render({ ...e, view: { ...e.view, controls: [...e.view.controls, control("mod:spell-rev", { title: "Spell Revisions", selected: true })] } });
    const common = page.querySelector(".common-choices")!;
    expect(common.querySelector(".common-choice-grid")!.compareDocumentPosition(common.querySelector(".game-interface-choice")!) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(common.querySelector("#bundle-spell-revisions")).not.toBeNull();
  });
  it("switches providers atomically without overwriting add-on preferences", () => {
    const { page, select, changes } = render();
    fireEvent.change(select, { target: { value: infinity } });
    expect(changes).toEqual([{ [infinity]: true, [sod]: false }]);
    expect(page.querySelector("#game-interface-effects")!.textContent).toContain("Thief skills in steps of five");
    expect(page.textContent).toContain("Community playtesting");
    fireEvent.change(select, { target: { value: "" } });
    expect(changes.at(-1)).toEqual({ [infinity]: false, [sod]: false });
    expect(page.querySelector("#game-interface-effects")!.textContent).toBe("");
  });
  it("still explains suppressed requested choices after backend reevaluation", () => {
    const { page } = render(fixture(infinity));
    expect(page.querySelector("#game-interface-effects")!.textContent).toContain("Thief skills in steps of five");
  });
  it("does not list already-unchecked options as collateral", () => {
    const e = fixture(infinity);
    const { page } = render({ ...e, normalizedSelection: { ...e.normalizedSelection, features: { ...e.normalizedSelection.features, [skill]: false } } });
    expect(page.querySelector("#game-interface-effects")!.textContent).toBe("");
  });
  it("shows blocked providers with an explanation", () => {
    const e = fixture();
    const { page, select } = render({ ...e, view: { ...e.view, controls: e.view.controls.map(c => c.id === infinity ? { ...c, readiness: "blocked", choiceAvailable: false, unavailableReason: "Needs a reviewed game-build update." } : c) } });
    expect((getByRole(select, "option", { name: "Infinity UI++ (Unavailable)" }) as HTMLOptionElement).disabled).toBe(true);
    expect(page.textContent).toContain("Needs a reviewed game-build update.");
  });
  it("never changes a UI provider through category bulk actions", () => {
    const e = fixture(sod);
    expect(bulkCategoryChanges(e, "interface", false)).toEqual({ [skill]: false });
    expect(bulkCategoryChanges(e, "interface", true)).toEqual({ [skill]: true });
  });
  it("does not invent provider choices for an older recipe", () => {
    const e = fixture();
    const legacy = { ...e, view: { ...e.view, controls: [e.view.controls[2]!] } };
    const page = setupScreen(legacy, () => {}, () => {}, () => {}, undefined, { change: () => {}, reset: () => {} });
    expect(page.querySelector(".game-interface-choice")).toBeNull();
  });
});
