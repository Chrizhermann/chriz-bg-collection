import { describe, expect, it, vi } from "vitest";
import { NativeBackend, type InvokeCommand } from "../src/backend";

describe("clarity metadata transport", () => {
  it("retains authored context and engine switch availability", async () => {
    const invoke = vi.fn<InvokeCommand>(async () => ({
      view: { categories: ["convenience"], controls: [{
        id: "forty", title: "40", description: "Stack up to 40 potions.", category: "convenience",
        decision: "optional", readiness: "ready", parent: null, requires: [], conflicts: ["unlimited"],
        selected: false, interactive: false, unavailable_reason: null, inputs: [],
        source_label: "Tweaks Anthology", group_label: "Potion stacking",
        choice_group: "potion-stacking", choice_available: true,
      }] },
      normalized_selection: { platform: "windows", features: { unlimited: true, forty: false }, inputs: {} },
      findings: [], plan: { phases: [] }, selected_choice_count: 1,
    }));
    const result = await new NativeBackend(invoke).evaluateBuild({ platform: "windows", features: {}, inputs: {} });
    expect(result.view.controls[0]).toMatchObject({
      sourceLabel: "Tweaks Anthology", groupLabel: "Potion stacking", choiceGroup: "potion-stacking",
      choiceAvailable: true, selected: false, interactive: false,
    });
    expect(result.view.controls[0]).not.toHaveProperty("choice_available");
  });

  it("preserves current log rows and missing receipt entries", async () => {
    const components = [
      { target: "BG2", tp2: "BUFFBOT/SETUP-BUFFBOT.TP2", component: 0, title: null, version: null, status: "missing" },
      { target: "BG2", tp2: "EXTRA/SETUP-EXTRA.TP2", component: 1, title: "Extra mod", version: "2", status: "extra" },
    ];
    const invoke = vi.fn<InvokeCommand>(async () => [{
      id: "test", name: "Test", path: "C:\\Example", status: "complete", receipt_path: null,
      launch_path: null, completed_at_millis: 1, available: true, resumable: false,
      consistency: { state: "changed", detail: "Changed", component_count: 1, mod_count: 1, components },
    }]);
    const result = await new NativeBackend(invoke).listManagedInstallations();
    expect(result[0]?.consistency?.components).toEqual(components);
  });
});
