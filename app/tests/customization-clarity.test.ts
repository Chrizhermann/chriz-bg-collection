import { describe, expect, it } from "vitest";
import type { FeatureControl, SelectionEvaluation } from "../src/contracts";
import { bundleChanges, commonBundles } from "../src/customization";

function evaluation(ids: readonly string[]): SelectionEvaluation {
  const controls: FeatureControl[] = ids.map(id => ({
    id, title: id, description: "Test choice", category: "collection",
    decision: "default", readiness: "ready", parent: null, selected: true,
    interactive: true, unavailableReason: null, inputs: [],
  }));
  return {
    view: { categories: ["collection"], controls },
    normalizedSelection: { platform: "windows", features: Object.fromEntries(ids.map(id => [id, true])), inputs: {} },
    findings: [], plan: { phases: [] }, selectedChoiceCount: controls.length,
  };
}

describe("common bundle explanations", () => {
  it("names the Yeslick/Keldorn consequence without changing conversion policy", () => {
    const e = evaluation([
      "feature:yeslicknpc:component-0", "feature:yeslicknpc:component-1",
      "feature:chriz-bg-modpack:component-410", "feature:chriz-bg-modpack:component-197",
    ]);
    const bundle = commonBundles(e).find(item => item.id === "original-companions")!;
    expect(bundle.description).toContain("Keeping Yeslick’s original class also leaves out the combined Yeslick/Keldorn dispel fix");
    expect(bundle.description).toContain("portraits, voices and spellbook tweaks");
    expect(bundle.description).toContain("New installations only");
    expect(bundle.description).not.toContain("unrelated fixes");
    expect(bundleChanges(e, bundle, true)).toEqual({
      "feature:yeslicknpc:component-0": true,
      "feature:yeslicknpc:component-1": false,
    });
  });

  it("makes clear that turning SR off leaves separately selected magic changes", () => {
    const e = evaluation(["mod:spell-rev", "mod:iwdification", "feature:cdtweaks:component-2312"]);
    const bundle = commonBundles(e).find(item => item.id === "spell-revisions")!;
    expect(bundle.description).toContain("IWDification’s Icewind Dale spells");
    expect(bundle.description).toContain("other mods’ spell tweaks");
    expect(bundle.description).toContain("Related fixes follow automatically");
    expect(bundleChanges(e, bundle, false)).toEqual({ "mod:spell-rev": false });
  });

  it("explains kit-bundle collateral while retaining the existing parent switches", () => {
    const e = evaluation([
      "mod:artisanskitpack", "mod:artisanskitpack-npc", "mod:artisanskitpack-tweak",
      "mod:bardicwonders", "feature:bardicwonders:component-1008",
    ]);
    const artisan = commonBundles(e).find(item => item.id === "artisan")!;
    const bardic = commonBundles(e).find(item => item.id === "bardic")!;
    expect(artisan.description).toContain("fixes that need them");
    expect(bardic.description).toContain("companion kit choices and patches that need it");
    expect(bundleChanges(e, artisan, false)).toEqual({
      "mod:artisanskitpack": false, "mod:artisanskitpack-npc": false, "mod:artisanskitpack-tweak": false,
    });
    expect(bundleChanges(e, bardic, false)).toEqual({ "mod:bardicwonders": false });
  });

  it("distinguishes the Randomiser from other item changes", () => {
    const e = evaluation(["mod:randomiser", "feature:rr:component-7"]);
    const bundle = commonBundles(e).find(item => item.id === "randomiser")!;
    expect(bundle.description).toContain("eligible item locations");
    expect(bundle.description).toContain("not other item changes");
    expect(bundleChanges(e, bundle, false)).toEqual({ "mod:randomiser": false });
  });
});
