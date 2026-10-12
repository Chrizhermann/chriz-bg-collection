// @vitest-environment jsdom
import axe from "axe-core";
import { fireEvent, getByRole, waitFor, within } from "@testing-library/dom";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

import { mountApp } from "../src/app";
import { FixtureBackend, NativeBackend } from "../src/backend";
import type { FeatureControl, FeatureGroup, NormalizedSelection, SelectionEvaluation } from "../src/contracts";
import { featureGroupChanges } from "../src/customization";
import { setupScreen } from "../src/screens/setup";

const EMOTION = "feature:chriz-bg-rebalance:component-301";
const BAELOTH = "feature:chriz-bg-modpack:component-235";
const DRAGONS = "feature:chriz-bg-rebalance:component-110";
const BRIDGE = "feature:chriz-sod-remix:component-257";
const MEMBERS = [EMOTION, BAELOTH, DRAGONS, BRIDGE];
const FLAIL = "feature:chriz-bg-rebalance:component-302";
const SCS = "feature:stratagems:mandatory-components";
const TITLES: Record<string, string> = {
  [EMOTION]: "Emotion, Courage and Hope don't stack",
  [BAELOTH]: "Baeloth: normal Sorcerer spells known",
  [DRAGONS]: "Extra Challenge: lethal dragon melee",
  [BRIDGE]: "Extra Challenge: Insane bridge sequencers",
  [FLAIL]: "Flail of the Ages balance",
  [SCS]: "Curated SCS core",
};
const DESCRIPTION = "Stricter rules for players who want a harder game. Turns on every Challenge Mode change at once; you can still switch each one on or off under its own mod.";

/** Mirrors the engine: members are optional, dragon melee requires SCS, state is derived. */
function evaluate(features: Readonly<Record<string, boolean>>): SelectionEvaluation {
  const desired: Record<string, boolean> = { [FLAIL]: true, [SCS]: true, ...Object.fromEntries(MEMBERS.map(id => [id, false])), ...features };
  const controls: FeatureControl[] = [SCS, FLAIL, ...MEMBERS].map(id => {
    const unavailableReason = id === DRAGONS && !desired[SCS] ? "Requires Curated SCS core." : null;
    return {
      id, title: TITLES[id]!, description: `${TITLES[id]} description`, category: "difficulty",
      sourceLabel: id.includes("modpack") ? "Chriz BG Modpack" : "Chriz BG Rebalance",
      decision: id === FLAIL || id === SCS ? "default" : "optional", readiness: "ready", parent: null,
      selected: Boolean(desired[id]) && unavailableReason === null, interactive: unavailableReason === null,
      unavailableReason, inputs: [],
    };
  });
  const byId = new Map(controls.map(control => [control.id, control]));
  const selectedMembers = MEMBERS.filter(id => byId.get(id)!.selected);
  const availableMembers = MEMBERS.filter(id => byId.get(id)!.interactive);
  const unavailableMembers = MEMBERS.filter(id => !byId.get(id)!.interactive).map(featureId => ({ featureId, reason: byId.get(featureId)!.unavailableReason! }));
  const group: FeatureGroup = {
    id: "challenge-mode", title: "Challenge Mode", description: DESCRIPTION, members: MEMBERS,
    state: selectedMembers.length === 0 ? "off" : availableMembers.every(id => selectedMembers.includes(id)) ? "on" : "mixed",
    selectedMembers, availableMembers, unavailableMembers,
    disabledReason: availableMembers.length === 0 ? "No Challenge Mode change is available with the current choices." : null,
  };
  return {
    view: { categories: ["difficulty"], controls },
    normalizedSelection: { platform: "windows", features: desired, inputs: {} },
    findings: [], plan: { phases: [] }, selectedChoiceCount: controls.filter(control => control.selected).length,
    featureGroups: [group],
  };
}

function render(evaluation: SelectionEvaluation, change: (changes: Record<string, boolean>, focusId: string) => void = () => {}) {
  const page = setupScreen(evaluation, () => {}, () => {}, () => {}, { search: "", category: "", advancedOpen: true }, { change, reset: () => {} });
  document.body.append(page);
  return page;
}

function groupCheckbox(root: HTMLElement): HTMLInputElement {
  return getByRole(root, "checkbox", { name: "Challenge Mode" }) as HTMLInputElement;
}

afterEach(() => { document.body.replaceChildren(); localStorage.clear(); });

describe("Challenge Mode card", () => {
  it("is the first choice on the setup screen, above common changes, with the recipe description and members", () => {
    const page = render(evaluate({}));
    const card = page.querySelector<HTMLElement>(".feature-group-card")!;
    expect(page.firstElementChild?.classList.contains("setup-toolbar")).toBe(true);
    expect(card.previousElementSibling).toBe(page.firstElementChild);
    const advanced = page.querySelector(".setup-advanced")!;
    expect(card.compareDocumentPosition(advanced) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(card.textContent).toContain(DESCRIPTION);
    const members = within(card).getByRole("list", { name: "Challenge Mode changes" });
    expect(within(members).getAllByRole("listitem").map(item => item.dataset.feature)).toEqual(MEMBERS);
    expect(card.textContent).toContain("under its own mod");
    expect(card.textContent).not.toContain(TITLES[FLAIL]);
  });

  it("shows off, on and mixed as a tri-state checkbox", () => {
    const off = groupCheckbox(render(evaluate({})));
    expect(off.checked).toBe(false);
    expect(off.indeterminate).toBe(false);
    expect(off.getAttribute("aria-checked")).toBe("false");
    document.body.replaceChildren();

    const on = groupCheckbox(render(evaluate(Object.fromEntries(MEMBERS.map(id => [id, true])))));
    expect(on.checked).toBe(true);
    expect(on.indeterminate).toBe(false);
    expect(on.getAttribute("aria-checked")).toBe("true");
    document.body.replaceChildren();

    const page = render(evaluate({ [BRIDGE]: true }));
    const mixed = groupCheckbox(page);
    expect(mixed.checked).toBe(false);
    expect(mixed.indeterminate).toBe(true);
    expect(mixed.getAttribute("aria-checked")).toBe("mixed");
    expect(page.querySelector(".feature-group-state")!.textContent).toContain("1 of 4");
  });

  it("sends every member change as one batch: off and mixed turn all available on, on turns all off", () => {
    for (const [features, expected] of [
      [{}, Object.fromEntries(MEMBERS.map(id => [id, true]))],
      [{ [BRIDGE]: true }, Object.fromEntries(MEMBERS.map(id => [id, true]))],
      [Object.fromEntries(MEMBERS.map(id => [id, true])), Object.fromEntries(MEMBERS.map(id => [id, false]))],
    ] as const) {
      const change = vi.fn();
      const page = render(evaluate(features), change);
      fireEvent.click(groupCheckbox(page));
      expect(change).toHaveBeenCalledTimes(1);
      expect(change).toHaveBeenCalledWith(expected, "group-challenge-mode");
      document.body.replaceChildren();
    }
  });

  it("shows an unavailable member with its reason and never selects it", () => {
    const evaluation = evaluate({ [SCS]: false });
    const change = vi.fn();
    const page = render(evaluation, change);
    const item = page.querySelector<HTMLElement>(`.feature-group-member[data-feature="${DRAGONS}"]`)!;
    expect(item.textContent).toContain("Unavailable");
    expect(item.textContent).toContain("Requires Curated SCS core.");
    fireEvent.click(groupCheckbox(page));
    const sent = change.mock.calls[0]![0] as Record<string, boolean>;
    expect(sent).toEqual({ [EMOTION]: true, [BAELOTH]: true, [BRIDGE]: true });
    expect(DRAGONS in sent).toBe(false);
    expect(featureGroupChanges(evaluation.featureGroups![0]!)).toEqual(sent);
  });

  it("is disabled with a reason when no member is available", () => {
    const base = evaluate({ [SCS]: false });
    const group = { ...base.featureGroups![0]!, members: [DRAGONS], availableMembers: [], selectedMembers: [], state: "off" as const,
      unavailableMembers: [{ featureId: DRAGONS, reason: "Requires Curated SCS core." }], disabledReason: "No Challenge Mode change is available with the current choices." };
    const change = vi.fn();
    const page = render({ ...base, featureGroups: [group] }, change);
    const checkbox = groupCheckbox(page);
    expect(checkbox.getAttribute("aria-disabled")).toBe("true");
    const describedBy = checkbox.getAttribute("aria-describedby")!.split(" ").map(id => document.getElementById(id)!.textContent).join(" ");
    expect(describedBy).toContain("No Challenge Mode change is available");
    fireEvent.click(checkbox);
    expect(change).not.toHaveBeenCalled();
    expect(checkbox.checked).toBe(false);
  });

  it("marks every member under its own mod and leaves other choices unmarked", () => {
    const page = render(evaluate({}));
    const advanced = page.querySelector<HTMLElement>(".setup-advanced")!;
    for (const id of MEMBERS) {
      const row = document.getElementById(`feature-${id}`)!.closest(".control-row")!;
      expect(row.querySelector(".feature-group-badge")?.textContent, id).toBe("Challenge Mode");
    }
    const flail = document.getElementById(`feature-${FLAIL}`)!.closest(".control-row")!;
    expect(flail.querySelector(".feature-group-badge")).toBeNull();
    const search = getByRole(page, "searchbox");
    fireEvent.input(search, { target: { value: "challenge mode" } });
    const visible = [...advanced.querySelectorAll<HTMLElement>(".control-row")].filter(row => !row.hidden);
    expect(visible).toHaveLength(MEMBERS.length);
  });

  it.each([
    ["off", {}],
    ["mixed", { [BRIDGE]: true }],
    ["with an unavailable member", { [SCS]: false, [EMOTION]: true }],
  ] as const)("has no axe violations when %s", async (_label, features) => {
    const page = render(evaluate(features));
    const result = await axe.run(page, { rules: { "color-contrast": { enabled: false } } });
    expect(result.violations).toEqual([]);
  });
});

class GroupBackend extends FixtureBackend {
  readonly requests: NormalizedSelection[] = [];
  override async evaluateBuild(selection: NormalizedSelection): Promise<SelectionEvaluation> {
    this.requests.push(selection);
    return evaluate(selection.features);
  }
}

describe("Challenge Mode in the app", () => {
  it("applies the group in one evaluation, keeps member changes out of Also adjusted, and turns mixed after a hand change", async () => {
    const backend = new GroupBackend();
    const root = document.createElement("div");
    document.body.append(root);
    const handle = await mountApp(root, backend);
    await handle.navigate("setup");
    const user = userEvent.setup();
    const before = backend.requests.length;

    await user.click(groupCheckbox(root));
    await waitFor(() => expect(groupCheckbox(root).checked).toBe(true));
    expect(backend.requests).toHaveLength(before + 1);
    expect(backend.requests.at(-1)?.features).toMatchObject(Object.fromEntries(MEMBERS.map(id => [id, true])));
    expect(backend.requests.at(-1)?.features[FLAIL]).not.toBe(false);
    expect(root.textContent).not.toContain("Also adjusted");
    expect(document.activeElement?.id).toBe("group-challenge-mode");

    const advanced = root.querySelector<HTMLDetailsElement>(".setup-advanced")!;
    advanced.open = true;
    await user.click(document.getElementById(`feature-${BAELOTH}`)!);
    await waitFor(() => expect(groupCheckbox(root).getAttribute("aria-checked")).toBe("mixed"));
    expect(groupCheckbox(root).indeterminate).toBe(true);
    const features = backend.requests.at(-1)!.features;
    expect(features[BAELOTH]).toBe(false);
    for (const id of [EMOTION, DRAGONS, BRIDGE]) expect(features[id], id).toBe(true);

    await user.click(groupCheckbox(root));
    await waitFor(() => expect(groupCheckbox(root).checked).toBe(true));
    expect(backend.requests.at(-1)?.features[BAELOTH]).toBe(true);

    await user.click(groupCheckbox(root));
    await waitFor(() => expect(groupCheckbox(root).getAttribute("aria-checked")).toBe("false"));
    expect(backend.requests.at(-1)?.features).toMatchObject(Object.fromEntries(MEMBERS.map(id => [id, false])));
    root.remove();
  });
});

describe("native group projection", () => {
  it("joins the authored groups with their derived state", async () => {
    const invoke = vi.fn(async (command: string) => {
      expect(command).toBe("evaluate_build");
      return {
        view: {
          categories: ["difficulty"],
          controls: [],
          groups: [{ id: "challenge-mode", title: "Challenge Mode", description: DESCRIPTION, members: [EMOTION, DRAGONS] }],
        },
        groups: [{
          id: "challenge-mode", state: "mixed", selected_members: [EMOTION], available_members: [EMOTION],
          unavailable_members: [{ feature_id: DRAGONS, reason: "Requires Curated SCS core." }],
        }],
        normalized_selection: { platform: "windows", features: {}, inputs: {} },
        findings: [], plan: { phases: [] }, selected_choice_count: 0, manual_checkpoint_options: [],
      };
    });
    const evaluation = await new NativeBackend(invoke).evaluateBuild({ platform: "windows", features: {}, inputs: {} });
    expect(evaluation.featureGroups).toEqual([{
      id: "challenge-mode", title: "Challenge Mode", description: DESCRIPTION, members: [EMOTION, DRAGONS], state: "mixed",
      selectedMembers: [EMOTION], availableMembers: [EMOTION],
      unavailableMembers: [{ featureId: DRAGONS, reason: "Requires Curated SCS core." }], disabledReason: null,
    }]);
  });

  it("omits groups for recipes that have none", async () => {
    const invoke = vi.fn(async () => ({
      view: { categories: [], controls: [] }, normalized_selection: { platform: "windows", features: {}, inputs: {} },
      findings: [], plan: { phases: [] }, selected_choice_count: 0,
    }));
    const evaluation = await new NativeBackend(invoke).evaluateBuild({ platform: "windows", features: {}, inputs: {} });
    expect(evaluation.featureGroups).toBeUndefined();
  });
});
