// @vitest-environment jsdom
import axe from "axe-core";
import { fireEvent, getByRole, waitFor, within } from "@testing-library/dom";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

import { mountApp } from "../src/app";
import { FixtureBackend, NativeBackend } from "../src/backend";
import type { BackendStatus, FeatureControl, FeatureGroup, NormalizedSelection, SelectionEvaluation } from "../src/contracts";
import { featureGroupChanges } from "../src/customization";
import { setupScreen } from "../src/screens/setup";
import { SETUP_DRAFT_STORAGE_KEY, type SetupDraft } from "../src/setup-draft";

const EMOTION = "feature:chriz-bg-rebalance:component-301";
const BAELOTH = "feature:chriz-bg-modpack:component-235";
const MEMBERS = [EMOTION, BAELOTH];
const STRICT = "feature:chriz-bg-rebalance:component-330";
const GENIUS = "feature:chriz-bg-rebalance:component-331";
const REBALANCE = "mod:chriz-bg-rebalance";
const LABEL = "Potions: strict (Potions of Genius don't stack either)";
const STRICT_TITLE = "Potions: one dose of each type at a time";
const GENIUS_TITLE = "Potions: one dose of each type at a time; Genius still stacks";

/**
 * Mirrors the engine for Challenge Mode with two members and the potion variant: the potion
 * options need BG Rebalance, 331 is the recipe default, and state is derived like a member.
 */
function evaluate(features: Readonly<Record<string, boolean>>): SelectionEvaluation {
  const desired: Record<string, boolean> = {
    [REBALANCE]: true, [GENIUS]: true, [STRICT]: false, ...Object.fromEntries(MEMBERS.map(id => [id, false])), ...features,
  };
  const rebalance = desired[REBALANCE] !== false;
  const member = (id: string, title: string): FeatureControl => ({
    id, title, description: `${title} description`, category: "difficulty", sourceLabel: "Chriz BG Rebalance",
    decision: "optional", readiness: "ready", parent: null, selected: Boolean(desired[id]), interactive: true,
    unavailableReason: null, inputs: [],
  });
  const potion = (id: string, title: string, decision: "default" | "optional", other: string): FeatureControl => ({
    id, title, description: `${title} description`, category: "items", sourceLabel: "BG Rebalance",
    groupLabel: "Potion uniqueness", choiceGroup: "potion-uniqueness", choiceAvailable: rebalance, conflicts: [other],
    decision, readiness: "ready", parent: REBALANCE, selected: rebalance && Boolean(desired[id]) && !(id === GENIUS && desired[STRICT]),
    interactive: rebalance, unavailableReason: rebalance ? null : "Requires BG Rebalance.", inputs: [],
  });
  const controls = [
    member(EMOTION, "Emotion, Courage and Hope don't stack"),
    member(BAELOTH, "Baeloth: normal Sorcerer spells known"),
    potion(STRICT, STRICT_TITLE, "optional", GENIUS),
    potion(GENIUS, GENIUS_TITLE, "default", STRICT),
  ];
  const byId = new Map(controls.map(control => [control.id, control]));
  const selectedMembers = MEMBERS.filter(id => byId.get(id)!.selected);
  const variant = {
    choiceGroup: "potion-uniqueness", option: STRICT, label: LABEL, options: [STRICT, GENIUS], recipeDefault: [GENIUS],
    selected: byId.get(STRICT)!.selected, available: rebalance, unavailableReason: rebalance ? null : "Requires BG Rebalance.",
  };
  const group: FeatureGroup = {
    id: "challenge-mode", title: "Challenge Mode", description: "Stricter rules.", members: MEMBERS,
    state: selectedMembers.length === 0 && !variant.selected ? "off"
      : selectedMembers.length === MEMBERS.length && (variant.selected || !variant.available) ? "on" : "mixed",
    selectedMembers, availableMembers: MEMBERS, unavailableMembers: [], disabledReason: null, variants: [variant],
  };
  return {
    view: { categories: ["difficulty", "items"], controls },
    normalizedSelection: { platform: "windows", features: desired, inputs: {} },
    findings: [], plan: { phases: [] }, selectedChoiceCount: controls.filter(control => control.selected).length,
    featureGroups: [group],
  };
}

const ALL_ON = { ...Object.fromEntries(MEMBERS.map(id => [id, true])), [STRICT]: true, [GENIUS]: false };

function render(evaluation: SelectionEvaluation, change: (changes: Record<string, boolean>, focusId: string) => void = () => {}) {
  const page = setupScreen(evaluation, () => {}, () => {}, () => {}, { search: "", category: "", advancedOpen: true }, { change, reset: () => {} });
  document.body.append(page);
  return page;
}

function groupCheckbox(root: HTMLElement): HTMLInputElement {
  return getByRole(root, "checkbox", { name: "Challenge Mode" }) as HTMLInputElement;
}

function variantItem(root: HTMLElement): HTMLElement {
  return root.querySelector<HTMLElement>(`.feature-group-variant[data-choice-group="potion-uniqueness"]`)!;
}

afterEach(() => { document.body.replaceChildren(); localStorage.clear(); vi.restoreAllMocks(); });

describe("Challenge Mode variant on the card", () => {
  it("lists the variant after the members with its label, its state and the recommended option it replaces", () => {
    const page = render(evaluate({}));
    const card = page.querySelector<HTMLElement>(".feature-group-card")!;
    const items = within(within(card).getByRole("list", { name: "Challenge Mode changes" })).getAllByRole("listitem");
    expect(items.map(item => item.dataset.feature)).toEqual([...MEMBERS, STRICT]);
    const item = variantItem(page);
    expect(item.textContent).toContain(LABEL);
    expect(item.textContent).toContain("Not included");
    expect(item.textContent).toContain(`Instead of the recommended “${GENIUS_TITLE}”`);
    expect(item.textContent).toContain("BG Rebalance");
    document.body.replaceChildren();
    expect(variantItem(render(evaluate(ALL_ON))).textContent).toContain("Included");
  });

  it("counts the variant like a member: picking only the members leaves the group partly on", () => {
    const page = render(evaluate(Object.fromEntries(MEMBERS.map(id => [id, true]))));
    const checkbox = groupCheckbox(page);
    expect(checkbox.indeterminate).toBe(true);
    expect(checkbox.getAttribute("aria-checked")).toBe("mixed");
    expect(page.querySelector(".feature-group-state")!.textContent).toContain("2 of 3");
    document.body.replaceChildren();
    const on = render(evaluate(ALL_ON));
    expect(groupCheckbox(on).checked).toBe(true);
    expect(on.querySelector(".feature-group-state")!.textContent).toContain("all 3 available changes");
  });

  it("switching on picks the variant and switching off restores the recommended option", () => {
    const change = vi.fn();
    fireEvent.click(groupCheckbox(render(evaluate({}), change)));
    expect(change).toHaveBeenCalledWith({ [EMOTION]: true, [BAELOTH]: true, [STRICT]: true, [GENIUS]: false }, "group-challenge-mode");
    document.body.replaceChildren();

    const off = vi.fn();
    fireEvent.click(groupCheckbox(render(evaluate(ALL_ON), off)));
    expect(off).toHaveBeenCalledWith({ [EMOTION]: false, [BAELOTH]: false, [STRICT]: false, [GENIUS]: true }, "group-challenge-mode");
  });

  it("never forces an unavailable variant and shows its reason", () => {
    const evaluation = evaluate({ [REBALANCE]: false });
    const change = vi.fn();
    const page = render(evaluation, change);
    const item = variantItem(page);
    expect(item.textContent).toContain("Unavailable");
    expect(item.textContent).toContain("Requires BG Rebalance.");
    fireEvent.click(groupCheckbox(page));
    const sent = change.mock.calls[0]![0] as Record<string, boolean>;
    expect(sent).toEqual({ [EMOTION]: true, [BAELOTH]: true });
    expect(featureGroupChanges(evaluation.featureGroups![0]!, true)).toEqual(sent);
    // Switching off still restores the recommended option, like deselecting every member.
    expect(featureGroupChanges(evaluation.featureGroups![0]!, false)).toEqual({ [EMOTION]: false, [BAELOTH]: false, [STRICT]: false, [GENIUS]: true });
  });

  it("marks the choice it switches under its own mod, and the option it picks", () => {
    const page = render(evaluate({}));
    const select = getByRole(page, "combobox", { name: "Potion uniqueness" }) as HTMLSelectElement;
    const row = select.closest<HTMLElement>(".exclusive-choice-row")!;
    expect(row.querySelector(".feature-group-badge")?.textContent).toBe("Challenge Mode");
    const strict = [...select.options].find(option => option.value === STRICT)!;
    const genius = [...select.options].find(option => option.value === GENIUS)!;
    expect(strict.textContent).toContain("(Challenge Mode)");
    expect(genius.textContent).toContain("(Recommended)");
    expect(genius.textContent).not.toContain("Challenge Mode");
    fireEvent.input(getByRole(page, "searchbox"), { target: { value: "challenge mode" } });
    expect(row.hidden).toBe(false);
  });

  it.each([
    ["off", {}],
    ["on", ALL_ON],
    ["with the variant unavailable", { [REBALANCE]: false }],
  ] as const)("has no axe violations when %s", async (_label, features) => {
    const page = render(evaluate(features));
    const result = await axe.run(page, { rules: { "color-contrast": { enabled: false } } });
    expect(result.violations).toEqual([]);
  });
});

class VariantBackend extends FixtureBackend {
  readonly requests: NormalizedSelection[] = [];
  override async evaluateBuild(selection: NormalizedSelection): Promise<SelectionEvaluation> {
    this.requests.push(selection);
    return evaluate(selection.features);
  }
}

/** Native-mode backend so saved setups are read, with the variant evaluation. */
class SavedSetupBackend extends VariantBackend {
  override getStatus(): Promise<BackendStatus> {
    return Promise.resolve({
      mode: "native", engineVersion: "0.1.0", recipeVersion: "recipe-21", startupInstallId: null,
      selectedProfile: "recommended", profiles: [{ id: "recommended", label: "Recommended", description: "Recommended choices" }],
    });
  }
}

function saveDraft(features: Record<string, boolean>): void {
  const draft: SetupDraft = {
    version: 1, recipeVersion: "recipe-21", profileId: "recommended",
    installationName: "Stream run", destinationPath: "D:\\Stream run", destinationAutomatic: false,
    sourcePaths: { bg1: null, bg2: null },
    selection: { platform: "windows", features, inputs: {} },
    createDesktopShortcut: false,
  };
  localStorage.setItem(SETUP_DRAFT_STORAGE_KEY, JSON.stringify(draft));
}

async function openSetup(backend: VariantBackend) {
  const root = document.createElement("div");
  document.body.append(root);
  const handle = await mountApp(root, backend);
  await handle.navigate("setup");
  return root;
}

describe("Challenge Mode variant in the app", () => {
  it("turns partly on when the recommended potion rule is picked again by hand", async () => {
    const backend = new VariantBackend();
    const root = await openSetup(backend);
    const user = userEvent.setup();
    await user.click(groupCheckbox(root));
    await waitFor(() => expect(groupCheckbox(root).checked).toBe(true));
    expect(backend.requests.at(-1)?.features).toMatchObject({ [STRICT]: true, [GENIUS]: false });

    root.querySelector<HTMLDetailsElement>(".setup-advanced")!.open = true;
    await user.selectOptions(getByRole(root, "combobox", { name: "Potion uniqueness" }), GENIUS);
    await waitFor(() => expect(groupCheckbox(root).getAttribute("aria-checked")).toBe("mixed"));
    expect(backend.requests.at(-1)?.features).toMatchObject({ [STRICT]: false, [GENIUS]: true, [EMOTION]: true, [BAELOTH]: true });
    root.remove();
  });

  it("restores a setup saved before the variant existed unchanged, with the group partly on", async () => {
    // Switching Challenge Mode on stored only its members before the variant existed.
    saveDraft({ [EMOTION]: true, [BAELOTH]: true, [GENIUS]: true });
    const backend = new SavedSetupBackend();
    const root = await openSetup(backend);
    await waitFor(() => expect(groupCheckbox(root).getAttribute("aria-checked")).toBe("mixed"));
    expect(backend.requests[0]?.features).toEqual({ [EMOTION]: true, [BAELOTH]: true, [GENIUS]: true });
    expect((getByRole(root, "combobox", { name: "Potion uniqueness" }) as HTMLSelectElement).value).toBe(GENIUS);
    expect(variantItem(root).textContent).toContain("Not included");
    root.remove();
  });

  it("restores a setup saved with the variant picked as Challenge Mode on", async () => {
    saveDraft(ALL_ON);
    const backend = new SavedSetupBackend();
    const root = await openSetup(backend);
    await waitFor(() => expect(groupCheckbox(root).checked).toBe(true));
    expect(backend.requests[0]?.features).toEqual(ALL_ON);
    expect((getByRole(root, "combobox", { name: "Potion uniqueness" }) as HTMLSelectElement).value).toBe(STRICT);
    root.remove();
  });
});

describe("native variant projection", () => {
  function wire(stateVariants: unknown) {
    return {
      view: {
        categories: [], controls: [],
        groups: [{
          id: "challenge-mode", title: "Challenge Mode", description: "Stricter rules.", members: [EMOTION],
          variants: [{ choice_group: "potion-uniqueness", option: STRICT, label: LABEL, options: [STRICT, GENIUS], recipe_default: [GENIUS] }],
        }],
      },
      groups: [{
        id: "challenge-mode", state: "mixed", selected_members: [EMOTION], available_members: [EMOTION], unavailable_members: [],
        variants: stateVariants,
      }],
      normalized_selection: { platform: "windows", features: {}, inputs: {} },
      findings: [], plan: { phases: [] }, selected_choice_count: 0, manual_checkpoint_options: [],
    };
  }

  it("joins each authored variant with its derived state", async () => {
    const invoke = vi.fn(async () => wire([{ choice_group: "potion-uniqueness", option: STRICT, selected: false, available: false, unavailable_reason: "Requires BG Rebalance." }]));
    const evaluation = await new NativeBackend(invoke).evaluateBuild({ platform: "windows", features: {}, inputs: {} });
    expect(evaluation.featureGroups?.[0]?.variants).toEqual([{
      choiceGroup: "potion-uniqueness", option: STRICT, label: LABEL, options: [STRICT, GENIUS], recipeDefault: [GENIUS],
      selected: false, available: false, unavailableReason: "Requires BG Rebalance.",
    }]);
  });

  it("leaves out a group whose variant states don't match its variants instead of showing it half-switched", async () => {
    const strict = { choice_group: "potion-uniqueness", option: STRICT, selected: true, available: true };
    for (const states of [
      undefined,
      [],
      [{ ...strict, option: GENIUS }],
      [{ ...strict, choice_group: "other-choice" }],
      [strict, { ...strict, choice_group: "other-choice" }],
    ]) {
      const invoke = vi.fn(async () => wire(states));
      const evaluation = await new NativeBackend(invoke).evaluateBuild({ platform: "windows", features: {}, inputs: {} });
      expect(evaluation.featureGroups).toBeUndefined();
    }
  });
});
