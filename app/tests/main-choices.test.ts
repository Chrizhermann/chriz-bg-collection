// @vitest-environment jsdom
import axe from "axe-core";
import { fireEvent, getByRole, getByText, queryByRole, waitFor, within } from "@testing-library/dom";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

import { mountApp } from "../src/app";
import { FixtureBackend } from "../src/backend";
import type { FeatureControl, FeatureGroup, GameDiscovery, NormalizedSelection, SelectionEvaluation } from "../src/contracts";
import { installScreen, type InstallScreenActions, type InstallScreenModel } from "../src/screens/install";

const EMOTION = "feature:chriz-bg-rebalance:component-301";
const BAELOTH = "feature:chriz-bg-modpack:component-235";
const DRAGONS = "feature:chriz-bg-rebalance:component-110";
const MEMBERS = [EMOTION, BAELOTH, DRAGONS];
const FLAIL = "feature:chriz-bg-rebalance:component-302";
const SCS = "feature:stratagems:mandatory-components";
const TITLES: Record<string, string> = {
  [EMOTION]: "Emotion, Courage and Hope don't stack",
  [BAELOTH]: "Baeloth: normal Sorcerer spells known",
  [DRAGONS]: "Extra Challenge: lethal dragon melee",
  [FLAIL]: "Flail of the Ages balance",
  [SCS]: "Curated SCS core",
};
const DESCRIPTION = "Stricter rules for players who want a harder game. Turns on every Challenge Mode change at once; you can still switch each one on or off under its own mod.";
const ALL_ON = Object.fromEntries(MEMBERS.map(id => [id, true]));
const ALL_OFF = Object.fromEntries(MEMBERS.map(id => [id, false]));

/** Mirrors the engine: members are optional, dragon melee requires SCS, state is derived. */
function evaluate(features: Readonly<Record<string, boolean>>, withGroup = true): SelectionEvaluation {
  const desired: Record<string, boolean> = { [FLAIL]: true, [SCS]: true, ...ALL_OFF, ...features };
  const controls: FeatureControl[] = [SCS, FLAIL, ...MEMBERS].map(id => {
    const unavailableReason = id === DRAGONS && !desired[SCS] ? "Requires Curated SCS core." : null;
    return {
      id, title: TITLES[id]!, description: `${TITLES[id]} description`, category: "difficulty",
      sourceLabel: "Chriz BG Rebalance", decision: id === FLAIL || id === SCS ? "default" : "optional",
      readiness: "ready", parent: null, selected: Boolean(desired[id]) && unavailableReason === null,
      interactive: unavailableReason === null, unavailableReason, inputs: [],
    };
  });
  const byId = new Map(controls.map(control => [control.id, control]));
  const selectedMembers = MEMBERS.filter(id => byId.get(id)!.selected);
  const availableMembers = MEMBERS.filter(id => byId.get(id)!.interactive);
  const group: FeatureGroup = {
    id: "challenge-mode", title: "Challenge Mode", description: DESCRIPTION, members: MEMBERS,
    state: selectedMembers.length === 0 ? "off" : availableMembers.every(id => selectedMembers.includes(id)) ? "on" : "mixed",
    selectedMembers, availableMembers,
    unavailableMembers: MEMBERS.filter(id => !byId.get(id)!.interactive).map(featureId => ({ featureId, reason: byId.get(featureId)!.unavailableReason! })),
    disabledReason: null,
  };
  return {
    view: { categories: ["difficulty"], controls },
    normalizedSelection: { platform: "windows", features: desired, inputs: {} },
    findings: [], plan: { phases: [] }, selectedChoiceCount: controls.filter(control => control.selected).length,
    ...(withGroup ? { featureGroups: [group] } : {}),
  };
}

function discovery(): GameDiscovery {
  const candidate = (id: string, label: string, path: string) => ({
    id, label, path, storefront: "steam" as const, build: "2.7.3.0", freshness: "fresh" as const, eligible: true,
    findings: ["Clean supported installation."],
  });
  return {
    selectedBg1Id: "bg1", selectedBg2Id: "bg2",
    bg1Candidates: [candidate("bg1", "BG:EE + SoD — Steam — clean", "C:\\Games\\Baldur's Gate Enhanced Edition")],
    bg2Candidates: [candidate("bg2", "BGII:EE — Steam — clean", "C:\\Games\\Baldur's Gate II Enhanced Edition")],
  };
}

function model(evaluation: SelectionEvaluation, overrides: Partial<InstallScreenModel> = {}): InstallScreenModel {
  return {
    discovery: discovery(), selectedBg1Id: "bg1", selectedBg2Id: "bg2", installationName: "Chriz Easy BG",
    destination: { path: "C:\\Games\\Chriz Easy BG", safe: true, title: "Ready", detail: "" },
    evaluation, evaluationPending: false, starting: false, createDesktopShortcut: true,
    manualDownloads: [], manualDownloadGateVisible: false, manualDownloadCheckingArtifactId: null, manualDownloadError: null,
    ...overrides,
  };
}

function actions(overrides: Partial<InstallScreenActions> = {}): InstallScreenActions {
  const noop = (): void => {};
  return {
    selectSource: noop, browseSource: noop, changeName: noop, changeLocation: noop, browseLocation: noop,
    customize: noop, install: noop, changeDesktopShortcut: noop, openManualSource: noop, chooseManualArchive: noop,
    skipManualDownload: noop, changeFeatures: noop, showGroupDetails: noop, ...overrides,
  };
}

function render(evaluation: SelectionEvaluation, overrides: Partial<InstallScreenActions> = {}, modelOverrides: Partial<InstallScreenModel> = {}): HTMLElement {
  const page = installScreen(model(evaluation, modelOverrides), actions(overrides));
  document.body.append(page);
  return page;
}

function mainCheckbox(root: HTMLElement): HTMLInputElement {
  return getByRole(root, "checkbox", { name: "Challenge Mode" }) as HTMLInputElement;
}

function stateText(root: HTMLElement): string {
  return root.querySelector(".main-choice-state")!.textContent!;
}

afterEach(() => { document.body.replaceChildren(); localStorage.clear(); });

describe("main choices on the install screen", () => {
  it("shows each feature group as a switch between the installation and the setup card", () => {
    const page = render(evaluate({}));
    const card = page.querySelector<HTMLElement>(".main-choices")!;
    expect(within(card).getByRole("heading", { level: 2, name: "Main choices" })).toBeTruthy();
    expect(card.previousElementSibling?.classList.contains("install-settings")).toBe(true);
    expect(card.nextElementSibling?.classList.contains("recommended-setup")).toBe(true);
    expect(card.textContent).toContain(DESCRIPTION);
    expect(within(card).getAllByRole("checkbox")).toHaveLength(1);
    expect(within(card).getByRole("button", { name: "Challenge Mode details" })).toBeTruthy();
  });

  it("shows no card for a recipe without feature groups", () => {
    const page = render(evaluate({}, false));
    expect(page.querySelector(".main-choices")).toBeNull();
    expect(queryByRole(page, "heading", { name: "Main choices" })).toBeNull();
    expect(page.querySelector(".install-settings")!.nextElementSibling?.classList.contains("recommended-setup")).toBe(true);
  });

  it("shows off, on and mixed with the same state sentence as the card in Customize", () => {
    const off = render(evaluate({}));
    expect(mainCheckbox(off).checked).toBe(false);
    expect(mainCheckbox(off).getAttribute("aria-checked")).toBe("false");
    expect(stateText(off)).toBe("Off: no Challenge Mode change is included.");
    document.body.replaceChildren();

    const on = render(evaluate(ALL_ON));
    expect(mainCheckbox(on).checked).toBe(true);
    expect(mainCheckbox(on).getAttribute("aria-checked")).toBe("true");
    expect(stateText(on)).toBe("On: all 3 available changes are included.");
    document.body.replaceChildren();

    const mixed = render(evaluate({ [EMOTION]: true }));
    expect(mainCheckbox(mixed).checked).toBe(false);
    expect(mainCheckbox(mixed).indeterminate).toBe(true);
    expect(mainCheckbox(mixed).getAttribute("aria-checked")).toBe("mixed");
    expect(stateText(mixed)).toBe("Partly on: 1 of 3 available changes are included. Select it to include them all.");
  });

  it("sends the group's member changes as one batch and stops offering the old result", () => {
    for (const [features, expected] of [[{}, ALL_ON], [{ [EMOTION]: true }, ALL_ON], [ALL_ON, ALL_OFF]] as const) {
      const changeFeatures = vi.fn();
      const page = render(evaluate(features), { changeFeatures });
      const install = getByRole(page, "button", { name: "Install Chriz Easy BG" }) as HTMLButtonElement;
      expect(install.disabled).toBe(false);
      fireEvent.click(mainCheckbox(page));
      expect(changeFeatures).toHaveBeenCalledTimes(1);
      expect(changeFeatures).toHaveBeenCalledWith(expected, "main-choice-challenge-mode");
      expect(install.disabled).toBe(true);
      expect(getByRole(page, "heading", { level: 2, name: "Checking your choices" })).toBeTruthy();
      document.body.replaceChildren();
    }
  });

  it("leaves an unavailable member out of the batch", () => {
    const changeFeatures = vi.fn();
    const page = render(evaluate({ [SCS]: false }), { changeFeatures });
    fireEvent.click(mainCheckbox(page));
    expect(changeFeatures).toHaveBeenCalledWith({ [EMOTION]: true, [BAELOTH]: true }, "main-choice-challenge-mode");
  });

  it("is disabled with the reason when no member is available", () => {
    const base = evaluate({});
    const group = { ...base.featureGroups![0]!, availableMembers: [], selectedMembers: [], state: "off" as const,
      disabledReason: "No Challenge Mode change is available with the current choices." };
    const changeFeatures = vi.fn();
    const page = render({ ...base, featureGroups: [group] }, { changeFeatures });
    const checkbox = mainCheckbox(page);
    expect(checkbox.getAttribute("aria-disabled")).toBe("true");
    expect(stateText(page)).toBe("No Challenge Mode change is available with the current choices.");
    fireEvent.click(checkbox);
    expect(changeFeatures).not.toHaveBeenCalled();
    expect(checkbox.checked).toBe(false);
  });

  it("cannot be changed while the installation starts or the setup is being switched", () => {
    for (const overrides of [{ starting: true }, { changingProfile: true }]) {
      const page = render(evaluate({}), {}, overrides);
      expect(mainCheckbox(page).disabled).toBe(true);
      expect((getByRole(page, "button", { name: "Challenge Mode details" }) as HTMLButtonElement).disabled).toBe(true);
      document.body.replaceChildren();
    }
  });

  it("opens the group's details", () => {
    const showGroupDetails = vi.fn();
    const page = render(evaluate({}), { showGroupDetails });
    fireEvent.click(getByRole(page, "button", { name: "Challenge Mode details" }));
    expect(showGroupDetails).toHaveBeenCalledTimes(1);
    expect(showGroupDetails.mock.calls[0]![0].id).toBe("challenge-mode");
  });

  it.each([
    ["off", {}],
    ["mixed", { [EMOTION]: true }],
    ["on", ALL_ON],
  ] as const)("has no axe violations when %s", async (_label, features) => {
    const page = render(evaluate(features));
    const result = await axe.run(page, { rules: { "color-contrast": { enabled: false } } });
    expect(result.violations).toEqual([]);
  });
});

class GroupBackend extends FixtureBackend {
  readonly requests: NormalizedSelection[] = [];
  override discoverGames(): Promise<GameDiscovery> { return Promise.resolve(discovery()); }
  override listManagedInstallations() { return Promise.resolve([]); }
  override async evaluateBuild(selection: NormalizedSelection): Promise<SelectionEvaluation> {
    this.requests.push(selection);
    return evaluate(selection.features);
  }
}

describe("main choices in the app", () => {
  it("switches Challenge Mode from the install screen and shows the same state in Customize", async () => {
    const backend = new GroupBackend();
    const root = document.createElement("div");
    document.body.append(root);
    await mountApp(root, backend);
    const user = userEvent.setup();
    expect(getByText(root, "2 choices included")).toBeTruthy();
    const before = backend.requests.length;

    await user.click(mainCheckbox(root));
    await waitFor(() => expect(mainCheckbox(root).checked).toBe(true));
    expect(backend.requests).toHaveLength(before + 1);
    expect(backend.requests.at(-1)?.features).toMatchObject(ALL_ON);
    expect(stateText(root)).toBe("On: all 3 available changes are included.");
    expect(getByText(root, "5 choices included")).toBeTruthy();
    expect(document.activeElement?.id).toBe("main-choice-challenge-mode");
    expect((getByRole(root, "button", { name: "Install Chriz Easy BG" }) as HTMLButtonElement).disabled).toBe(false);

    // Details opens Customize at the group's card, which shows the same state.
    await user.click(getByRole(root, "button", { name: "Challenge Mode details" }));
    await waitFor(() => expect(getByRole(root, "heading", { level: 1, name: "Customize your installation" })).toBeTruthy());
    const card = root.querySelector<HTMLElement>(".feature-group-card")!;
    expect(document.activeElement?.id).toBe("group-challenge-mode");
    expect((document.getElementById("group-challenge-mode") as HTMLInputElement).checked).toBe(true);
    expect(card.querySelector(".feature-group-state")!.textContent).toBe("On: all 3 available changes are included.");

    // One member switched off by hand in Customize turns the main choice to "partly on".
    root.querySelector<HTMLDetailsElement>(".setup-advanced")!.open = true;
    await user.click(document.getElementById(`feature-${BAELOTH}`)!);
    await waitFor(() => expect(root.querySelector(".feature-group-state")!.textContent).toContain("2 of 3"));
    await user.click(getByRole(root, "button", { name: "Done" }));
    await waitFor(() => expect(getByRole(root, "heading", { level: 1, name: "Install Chriz Easy BG" })).toBeTruthy());
    expect(mainCheckbox(root).getAttribute("aria-checked")).toBe("mixed");
    expect(mainCheckbox(root).indeterminate).toBe(true);
    expect(stateText(root)).toBe("Partly on: 2 of 3 available changes are included. Select it to include them all.");

    // The main choice includes them all again, then switches every member off.
    await user.click(mainCheckbox(root));
    await waitFor(() => expect(mainCheckbox(root).checked).toBe(true));
    expect(backend.requests.at(-1)?.features[BAELOTH]).toBe(true);
    await user.click(mainCheckbox(root));
    await waitFor(() => expect(mainCheckbox(root).getAttribute("aria-checked")).toBe("false"));
    expect(backend.requests.at(-1)?.features).toMatchObject(ALL_OFF);
    expect(backend.requests.at(-1)?.features[FLAIL]).not.toBe(false);
    root.remove();
  });
});
