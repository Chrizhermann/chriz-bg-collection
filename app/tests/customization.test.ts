// @vitest-environment jsdom
import { describe, expect, it } from "vitest";
import { fireEvent, getByRole, within, waitFor } from "@testing-library/dom";
import userEvent from "@testing-library/user-event";
import type { FeatureControl, SelectionEvaluation, NormalizedSelection } from "../src/contracts";
import { mountApp } from "../src/app";
import { FixtureBackend } from "../src/backend";
import { bulkCategoryChanges, commonBundles, bundleChanges, bundleSelected, exclusiveChoiceChanges, exclusiveChoiceGroups } from "../src/customization";
import { setupScreen } from "../src/screens/setup";

type ChoiceMetadata = { sourceLabel?: string; groupLabel?: string; choiceGroup?: string; choiceAvailable?: boolean | null };

function control(id: string, extra: Partial<FeatureControl & ChoiceMetadata> = {}): FeatureControl & ChoiceMetadata {
  return { id, title: id, description: "Test choice", category: "kits", decision: "default", readiness: "ready", parent: null, selected: true, interactive: true, unavailableReason: null, inputs: [], ...extra };
}
function evaluation(controls: FeatureControl[]): SelectionEvaluation {
  return { view: { categories: [...new Set(controls.map(c => c.category))], controls }, normalizedSelection: { platform: "windows", features: Object.fromEntries(controls.map(c => [c.id, c.selected])), inputs: {} }, findings: [], plan: { phases: [] }, selectedChoiceCount: controls.filter(c => c.selected).length };
}

describe("common customization bundles", () => {
  it("turns off every Artisan installer in one change while retaining child preferences", () => {
    const e = evaluation([control("mod:artisanskitpack"), control("mod:artisanskitpack-npc"), control("mod:artisanskitpack-tweak"), control("feature:artisanskitpack:component-1000")]);
    const bundle = commonBundles(e).find(b => b.id === "artisan")!;
    expect(bundleChanges(e, bundle, false)).toEqual({ "mod:artisanskitpack": false, "mod:artisanskitpack-npc": false, "mod:artisanskitpack-tweak": false });
  });

  it("uses the Bardic parent when present and covers the older catalog without one", () => {
    for (const parent of [false, true]) {
      const e = evaluation([...(parent ? [control("mod:bardicwonders")] : []), control("feature:bardicwonders:component-1008"), control("feature:bardicwonders:component-3001", { category: "compatibility" })]);
      const bundle = commonBundles(e).find(b => b.id === "bardic")!;
      expect(bundleChanges(e, bundle, false)).toEqual(parent ? { "mod:bardicwonders": false } : { "feature:bardicwonders:component-1008": false, "feature:bardicwonders:component-3001": false });
    }
  });

  it("original classes replaces Yeslick's route, excludes conversions, and preserves spell, portrait and quest fixes", () => {
    const e = evaluation([
      control("feature:yeslicknpc:component-0", { selected: false, decision: "optional", interactive: false, unavailableReason: "Choose one class" }),
      control("feature:yeslicknpc:component-1"),
      control("feature:chriz-bg-modpack:component-192"),
      control("feature:chriz-bg-modpack:component-220"), control("feature:chriz-bg-modpack:component-221"),
      control("feature:chriz-bg-modpack:component-222", { selected: false }), control("feature:chriz-bg-modpack:component-223", { selected: false }),
      control("feature:artisanskitpack-npc:component-5102", { selected: false, interactive: false }),
      control("feature:xan:component-1"), control("feature:xan:mandatory-components", { decision: "mandatory" }),
      control("feature:chriz-bg-modpack:component-197"), control("feature:chriz-bg-modpack:component-130", { decision: "mandatory" }),
      control("feature:sirene-bg2:component-5"), control("feature:sarah:component-1"),
    ]);
    const bundle = commonBundles(e).find(b => b.id === "original-companions")!;
    expect(bundleSelected(e, bundle)).toBe(false);
    expect(bundleChanges(e, bundle, true)).toEqual({
      "feature:yeslicknpc:component-0": true, "feature:yeslicknpc:component-1": false,
      "feature:chriz-bg-modpack:component-192": false,
      "feature:chriz-bg-modpack:component-220": false, "feature:chriz-bg-modpack:component-221": false,
      "feature:chriz-bg-modpack:component-222": false, "feature:chriz-bg-modpack:component-223": false,
      "feature:artisanskitpack-npc:component-5102": false,
      "feature:xan:component-1": false,
    });
  });

  it("does not add an excluded NPC when requesting original classes", () => {
    const e = evaluation([control("feature:yeslicknpc:component-0", { selected: false, decision: "optional" }), control("feature:yeslicknpc:component-1", { selected: false })]);
    const b = commonBundles(e)[0]!;
    expect(bundleChanges(e, b, true)["feature:yeslicknpc:component-0"]).toBe(false);
  });

  it("restores the user's previous alternatives rather than checking every option", () => {
    const e = evaluation([control("feature:bardicwonders:component-1001"), control("feature:bardicwonders:component-1002", { decision: "optional", selected: false })]);
    const b = commonBundles(e).find(b => b.id === "bardic")!;
    expect(bundleChanges(e, b, true, e.normalizedSelection.features)).toEqual(e.normalizedSelection.features);
  });

  it("category actions preserve mandatory and unavailable choices and choose only one compatible alternative", () => {
    const e = evaluation([control("core", { decision: "mandatory", interactive: false }), control("blocked", { readiness: "blocked", interactive: false }), control("a", { selected: false, conflicts: ["b"] }), control("b", { selected: false, conflicts: ["a"] })]);
    expect(bulkCategoryChanges(e, "kits", true)).toEqual({ a: true });
    expect(bulkCategoryChanges(e, "kits", false)).toEqual({ a: false, b: false });
  });

  it("groups only structurally safe, fully symmetric explicit alternatives", () => {
    const valid = evaluation([
      control("a", { selected: true, decision: "default", parent: "mod", choiceGroup: "route", groupLabel: "Class route", conflicts: ["b"], choiceAvailable: true }),
      control("b", { selected: false, decision: "optional", parent: "mod", choiceGroup: "route", groupLabel: "Class route", conflicts: ["a"], choiceAvailable: true }),
    ]);
    const group = exclusiveChoiceGroups(valid)[0]!;
    expect(group.label).toBe("Class route");
    expect(group.allowNone).toBe(true);
    expect(exclusiveChoiceChanges(group, "b")).toEqual({ a: false, b: true });
    expect(exclusiveChoiceChanges(group, "")).toEqual({ a: false, b: false });

    const required = { ...group, controls: [control("required", { decision: "mandatory" }), ...group.controls], allowNone: false };
    expect(exclusiveChoiceChanges(required, "b")).toEqual({ a: false, b: true });

    for (const controls of [
      [control("a", { choiceGroup: "route", groupLabel: "Route", conflicts: ["b"], choiceAvailable: true }), control("b", { selected: false, choiceGroup: "route", groupLabel: "Route", choiceAvailable: true })],
      [control("a", { parent: "one", choiceGroup: "route", groupLabel: "Route", conflicts: ["b"], choiceAvailable: true }), control("b", { selected: false, parent: "two", choiceGroup: "route", groupLabel: "Route", conflicts: ["a"], choiceAvailable: true })],
      [control("a", { choiceGroup: "route", groupLabel: "Route", conflicts: ["b"], choiceAvailable: true }), control("b", { selected: false, category: "rules", choiceGroup: "route", groupLabel: "Route", conflicts: ["a"], choiceAvailable: true })],
      [control("a", { choiceGroup: "route", groupLabel: "Route", conflicts: ["b"] }), control("b", { selected: false, choiceGroup: "route", groupLabel: "Route", conflicts: ["a"] })],
      [control("a", { decision: "mandatory", choiceGroup: "route", groupLabel: "Route", conflicts: ["b"], choiceAvailable: true }), control("b", { selected: false, choiceGroup: "route", groupLabel: "Route", conflicts: ["a"], choiceAvailable: true })],
    ]) expect(exclusiveChoiceGroups(evaluation(controls))).toEqual([]);
  });

  it("switches an explicit choice atomically, offers none for optional groups, and explains blocked options", async () => {
    const e = evaluation([
      control("fighter", { title: "NPC class: Fighter", description: "Keep the original class.", selected: true, decision: "default", parent: "npc", sourceLabel: "Base game", choiceGroup: "npc-class", groupLabel: "NPC class", conflicts: ["cleric", "mage"], choiceAvailable: true }),
      control("cleric", { title: "Cleric", description: "Use the authored conversion.", selected: false, decision: "optional", parent: "npc", sourceLabel: "Modpack", choiceGroup: "npc-class", groupLabel: "NPC class", conflicts: ["fighter", "mage"], interactive: false, choiceAvailable: true }),
      control("mage", { title: "Mage", description: "Requires another mod.", selected: false, decision: "optional", parent: "npc", sourceLabel: "Expansion", choiceGroup: "npc-class", groupLabel: "NPC class", conflicts: ["fighter", "cleric"], interactive: false, choiceAvailable: false, unavailableReason: "Requires Spell Revisions" }),
    ]);
    const batches: Record<string, boolean>[] = [];
    const view = setupScreen(e, () => {}, () => {}, () => {}, { search: "", category: "", advancedOpen: true }, { change: (patch) => { batches.push(patch); }, reset: () => {} });
    document.body.append(view);
    const select = getByRole(view, "combobox", { name: "NPC class" });
    expect((select as HTMLSelectElement).value).toBe("fighter");
    expect(within(select).getByRole("option", { name: "Fighter (Recommended)" })).toBeTruthy();
    expect((within(select).getByRole("option", { name: "Mage (Unavailable)" }) as HTMLOptionElement).disabled).toBe(true);
    expect(view.textContent).toContain("Mage: Requires Spell Revisions");
    expect(view.textContent).toContain("Sources: Base game, Modpack, Expansion");
    await userEvent.setup().selectOptions(select, "cleric");
    expect(batches).toEqual([{ fighter: false, cleric: true, mage: false }]);
    await userEvent.setup().selectOptions(select, "");
    expect(batches.at(-1)).toEqual({ fighter: false, cleric: false, mage: false });
    view.remove();
  });

  it("keeps legacy checkboxes and searches discreet source and group labels", async () => {
    const e = evaluation([
      control("legacy", { title: "Legacy default", sourceLabel: "Hidden author", groupLabel: "Legacy route" }),
      control("other", { title: "Other choice", selected: false, category: "rules" }),
      control("duplicate", { title: "Same text", description: "Same text", selected: false, category: "rules" }),
    ]);
    const view = setupScreen(e, () => {}, () => {}, () => {}, { search: "", category: "", advancedOpen: true }, { change: () => {}, reset: () => {} });
    document.body.append(view);
    expect((getByRole(view, "checkbox", { name: "Legacy default" }) as HTMLInputElement).checked).toBe(true);
    const duplicateRow = view.querySelector("#feature-duplicate")!.closest(".control-row") as HTMLElement;
    expect(within(duplicateRow).queryByText("Same text", { selector: "p" })).toBeNull();
    const search = getByRole(view, "searchbox", { name: "Find a mod or option" });
    await userEvent.setup().type(search, "hidden author");
    expect(view.querySelector("#feature-legacy")?.closest(".control-row")?.hasAttribute("hidden")).toBe(false);
    expect(view.textContent).toContain("Group: Legacy route. Source: Hidden author");
    view.remove();
  });

  it("describes the SoD prompt without claiming that the bundle toggle skips the story", () => {
    const bundle = commonBundles(evaluation([control("mod:chriz-sod-remix")])).find(item => item.id === "sod-remix")!;
    expect(bundle.description).toContain("optional in-game prompt");
    expect(bundle.description).toContain("does not skip it automatically");
  });

  it("shows common changes ahead of collapsed category controls and submits one batch", () => {
    const e = evaluation([control("mod:randomiser", { category: "rules" }), control("mod:artisanskitpack"), control("mod:artisanskitpack-npc"), control("mod:artisanskitpack-tweak")]);
    const batches: Record<string, boolean>[] = [];
    const view = setupScreen(e, () => {}, () => {}, () => {}, { search: "", category: "" }, { change: (patch) => { batches.push(patch); }, reset: () => {} });
    document.body.append(view);
    const artisan = getByRole(view, "checkbox", { name: "Artisan’s Kitpack" });
    fireEvent.click(artisan);
    expect(batches).toEqual([{ "mod:artisanskitpack": false, "mod:artisanskitpack-npc": false, "mod:artisanskitpack-tweak": false }]);
    const advanced = view.querySelector<HTMLDetailsElement>(".setup-advanced")!;
    expect(advanced.open).toBe(false);
    advanced.open = true;
    expect(within(advanced).getAllByRole("button", { name: /Include all compatible/ })).toHaveLength(2);
    expect(getByRole(view, "button", { name: "Reset to Chriz’s setup" })).toBeTruthy();
    view.remove();
  });

  it("the app evaluates one atomic bundle request, explains collateral, and restores previous choices", async () => {
    class BundleBackend extends FixtureBackend {
      requests: NormalizedSelection[] = [];
      override async evaluateBuild(selection: NormalizedSelection) {
        this.requests.push(selection);
        const e = evaluation([control("mod:artisanskitpack"), control("mod:artisanskitpack-npc"), control("mod:artisanskitpack-tweak"), control("mazzy-fix", { title: "Mazzy kit fix", decision: "mandatory", interactive: false })]);
        const desired = { ...e.normalizedSelection.features, ...selection.features };
        return { ...e, normalizedSelection: { ...selection, features: desired }, view: { ...e.view, controls: e.view.controls.map(c => ({ ...c, selected: c.id === "mazzy-fix" ? desired["mod:artisanskitpack-npc"]! : desired[c.id]! })) } };
      }
    }
    const backend = new BundleBackend();
    const root = document.createElement("div");
    document.body.append(root);
    const handle = await mountApp(root, backend);
    await handle.navigate("setup");
    const before = backend.requests.length;
    await userEvent.setup().click(getByRole(root, "checkbox", { name: "Artisan’s Kitpack" }));
    await waitFor(() => expect(root.textContent).toContain("Left out: Mazzy kit fix"));
    expect(backend.requests).toHaveLength(before + 1);
    expect(backend.requests.at(-1)?.features).toMatchObject({ "mod:artisanskitpack": false, "mod:artisanskitpack-npc": false, "mod:artisanskitpack-tweak": false });
    expect(document.activeElement?.id).toBe("bundle-artisan");
    await userEvent.setup().click(getByRole(root, "checkbox", { name: "Artisan’s Kitpack" }));
    await waitFor(() => expect(root.textContent).toContain("Included: Mazzy kit fix"));
    expect(backend.requests.at(-1)?.features).toMatchObject({ "mod:artisanskitpack": true, "mod:artisanskitpack-npc": true, "mod:artisanskitpack-tweak": true });
    await userEvent.setup().click(getByRole(root, "button", { name: "Reset to Chriz’s setup" }));
    await waitFor(() => expect(backend.requests.at(-1)?.features).toEqual({}));
    root.remove();
  });
});
