// @vitest-environment jsdom

import { fireEvent, getByRole, queryByRole } from "@testing-library/dom";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { HotpatchPreview, ManagedCopyUpdate } from "../src/contracts";
import { hotpatchSection, saveApplicabilityLabels, type HotpatchActions } from "../src/components/hotpatch";

const copy: ManagedCopyUpdate = {
  installId: "first", name: "Chriz Easy IWD", path: "D:\\Games\\Chriz Easy IWD",
  installedRecipeVersion: "0.1.0-alpha.1", state: "unknown", detail: "",
};

afterEach(() => document.body.replaceChildren());

function preview(overrides: Partial<HotpatchPreview> = {}): HotpatchPreview {
  return {
    installId: "first", state: "available", detail: "A harder, fairer Belhifet.",
    baseRecipeVersion: "0.1.0-alpha.1",
    hotpatch: {
      id: "belhifet-1", title: "Belhifet fight", summary: "A harder, fairer Belhifet.",
      changes: [{ title: "Belhifet", summary: "New tactics.", saveApplicability: "before-area-visit", conditionNote: "Saves before the last fight get it." }],
    },
    applied: [], refusals: [], notes: [], reviewToken: "bound-review",
    canUndo: false, undoTitle: null, canRestore: false,
    fullBackupBytes: "4294967296", saveBackupBytes: "52428800", availableBytes: "107374182400",
    backupPath: "C:\\Users\\Player\\AppData\\Local\\Chriz IWD Collection\\hotpatch-backups\\first",
    ...overrides,
  };
}

function actions(overrides: Partial<HotpatchActions> = {}): HotpatchActions {
  return { onCheckHotpatch: vi.fn(), onApplyHotpatch: vi.fn(), onUndoHotpatch: vi.fn(), onRestoreHotpatch: vi.fn(), ...overrides };
}

function render(callbacks: HotpatchActions): HTMLElement {
  const section = hotpatchSection(copy, callbacks);
  document.body.append(section);
  return section;
}

describe("hotpatch section", () => {
  it("checks only when asked and says that the running game is patched too", () => {
    const callbacks = actions();
    const section = render(callbacks);
    expect(getByRole(section, "heading", { name: "Hotpatch" })).toBeTruthy();
    expect(section.textContent).toContain("the game you are playing now included");
    expect(section.textContent).toContain("Not checked yet");
    expect(callbacks.onCheckHotpatch).not.toHaveBeenCalled();
    fireEvent.click(getByRole(section, "button", { name: "Check for a hotpatch" }));
    expect(callbacks.onCheckHotpatch).toHaveBeenCalledWith("first");
  });

  it("offers an available hotpatch with a folded changelog and what running games get", () => {
    const callbacks = actions({ hotpatch: preview() });
    const section = render(callbacks);
    const changelog = section.querySelector("details.hotpatch-changelog") as HTMLDetailsElement;
    expect(changelog.open).toBe(false);
    expect(changelog.textContent).toContain("New tactics.");
    expect(changelog.textContent).toContain(saveApplicabilityLabels["before-area-visit"]);
    expect(changelog.textContent).toContain("Saves before the last fight get it.");
    expect((getByRole(section, "checkbox", { name: /Back up the whole game folder/ }) as HTMLInputElement).checked).toBe(true);
    expect((getByRole(section, "checkbox", { name: /Copy your saved games/ }) as HTMLInputElement).checked).toBe(false);
    expect(section.textContent).toContain("4 GiB");
    fireEvent.click(getByRole(section, "checkbox", { name: /Copy your saved games/ }));
    fireEvent.click(getByRole(section, "button", { name: "Apply hotpatch" }));
    expect(callbacks.onApplyHotpatch).toHaveBeenCalledWith(true, true);
  });

  it("explains a refusal and offers no way to apply", () => {
    const refusals = ["3 mods differ from the recommended setup; hotpatches allow at most 2: Snowytoes added, Lanna added, Ina added."];
    const notes = ["Your own choices in Tweaks Anthology do not affect hotpatches."];
    const section = render(actions({ hotpatch: preview({ state: "refused", reviewToken: null, refusals, notes }) }));
    expect(section.querySelector(".hotpatch-refusals li")?.textContent).toBe(refusals[0]);
    expect(section.querySelector(".hotpatch-notes li")?.textContent).toBe(notes[0]);
    expect(section.textContent).toContain("Not offered for this installation");
    expect(section.textContent).toContain("Belhifet fight");
    expect(queryByRole(section, "button", { name: "Apply hotpatch" })).toBeNull();
  });

  it("offers Undo of the latest hotpatch once it is applied", () => {
    const callbacks = actions({ hotpatch: preview({ state: "up-to-date", hotpatch: null, reviewToken: null, applied: [{ id: "belhifet-1", title: "Belhifet fight" }], canUndo: true, undoTitle: "Belhifet fight" }) });
    const section = render(callbacks);
    expect(section.textContent).toContain("Applied hotpatches: Belhifet fight");
    fireEvent.click(getByRole(section, "button", { name: "Undo Belhifet fight" }));
    expect(callbacks.onUndoHotpatch).toHaveBeenCalled();
  });

  it("asks to restore an unfinished hotpatch before playing", () => {
    const callbacks = actions({ hotpatch: preview({ state: "needs-recovery", hotpatch: null, reviewToken: null, canRestore: true }) });
    const section = render(callbacks);
    expect(section.textContent).toContain("Restore the game files before playing");
    fireEvent.click(getByRole(section, "button", { name: "Restore game files" }));
    expect(callbacks.onRestoreHotpatch).toHaveBeenCalled();
  });

  it("locks every control while an operation runs and shows another installation's result nowhere", () => {
    const busy = render(actions({ hotpatch: preview(), hotpatchBusy: true }));
    for (const button of busy.querySelectorAll("button")) expect((button as HTMLButtonElement).disabled).toBe(true);
    expect(busy.querySelector(".patch-progress")).not.toBeNull();
    const other = render(actions({ hotpatch: preview({ installId: "second" }) }));
    expect(other.textContent).toContain("Not checked yet");
    expect(queryByRole(other, "button", { name: "Apply hotpatch" })).toBeNull();
  });
});
