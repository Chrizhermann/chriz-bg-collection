// @vitest-environment jsdom

import { fireEvent, getByRole, queryByRole, waitFor } from "@testing-library/dom";
import { afterEach, describe, expect, it, vi } from "vitest";
import { mountApp } from "../src/app";
import { FixtureBackend } from "../src/backend";
import type { ManagedCopyUpdate, ModUpdatePreview } from "../src/contracts";
import { modUpdateSection, type ModUpdateActions } from "../src/components/mod-update";
import { updatesScreen } from "../src/screens/updates";

const copy: ManagedCopyUpdate = {
  installId: "first", name: "Chriz Easy BG", path: "D:\\Games\\Chriz Easy BG",
  installedRecipeVersion: "0.1.1", state: "unknown", detail: "",
};
const offer = { modId: "buffbot", title: "BuffBot" } as const;

afterEach(() => document.body.replaceChildren());

function preview(overrides: Partial<ModUpdatePreview> = {}): ModUpdatePreview {
  return {
    installId: "first", modId: "buffbot", title: "BuffBot", state: "available",
    detail: "Brings BuffBot in this game to its newest release. Your saves, BuffBot settings and exported presets stay as they are.",
    installedVersion: "1.12.6", targetVersion: "1.16.0", releaseTitle: "BuffBot 1.16.0 — party presets",
    releaseUrl: "https://github.com/Chrizhermann/bg-eeex-buffbot/releases/tag/v1.16.0",
    refusals: [], reviewToken: "bound-review",
    canUndo: false, undoTitle: null, canRestore: false,
    fullBackupBytes: "4294967296", saveBackupBytes: "52428800", availableBytes: "107374182400",
    backupPath: "C:\\Users\\Player\\AppData\\Local\\Chriz BG Collection\\hotpatch-backups\\first",
    ...overrides,
  };
}

function actions(overrides: Partial<ModUpdateActions> = {}): ModUpdateActions {
  return { onCheckModUpdate: vi.fn(), onApplyModUpdate: vi.fn(), onUndoModUpdate: vi.fn(), onRestoreModUpdate: vi.fn(), ...overrides };
}

function render(callbacks: ModUpdateActions, disabled = false, target: ManagedCopyUpdate = copy): HTMLElement {
  const section = modUpdateSection(target, offer, callbacks, disabled);
  document.body.append(section);
  return section;
}

function checked(overrides: Partial<ModUpdatePreview> = {}, extra: Partial<ModUpdateActions> = {}): ModUpdateActions {
  return actions({ modUpdates: { buffbot: preview(overrides) }, ...extra });
}

const STATE_LABELS = {
  available: "Update ready",
  "up-to-date": "Up to date",
  "not-installed": "Not installed in this game",
  refused: "Not offered for this game",
  unavailable: "Could not check right now",
  "needs-recovery": "Restore the game files before playing",
} as const;

describe("mod update card", () => {
  it("checks only when asked, for its own mod", () => {
    const callbacks = actions();
    const section = render(callbacks);
    expect(getByRole(section, "heading", { name: "BuffBot" })).toBeTruthy();
    expect(section.textContent).toContain("Brings BuffBot in this game to its newest release. Every file it changes keeps an exact copy, so Undo puts the game back.");
    expect(section.textContent).toContain("Not checked yet. Your game stays unchanged until you choose Update.");
    expect(callbacks.onCheckModUpdate).not.toHaveBeenCalled();
    fireEvent.click(getByRole(section, "button", { name: "Check for a BuffBot update" }));
    expect(callbacks.onCheckModUpdate).toHaveBeenCalledWith("first", "buffbot");
  });

  it.each(Object.entries(STATE_LABELS))("labels the %s state", (state, label) => {
    const section = render(checked({ state: state as ModUpdatePreview["state"], reviewToken: state === "available" ? "bound-review" : null }));
    expect(section.querySelector(".patch-state")?.textContent).toBe(label);
    expect(section.querySelector<HTMLElement>(".mod-update-result")?.dataset.modUpdateState).toBe(state);
    expect(queryByRole(section, "button", { name: "Update BuffBot" }) !== null).toBe(state === "available");
  });

  it("offers an available update with versions, release title and backup choices", () => {
    const callbacks = checked();
    const section = render(callbacks);
    expect(section.querySelector(".mod-update-installed")?.textContent).toBe("Installed: 1.12.6");
    expect(section.querySelector(".mod-update-target")?.textContent).toBe("Newest release: 1.16.0");
    expect(section.querySelector(".mod-update-release")?.textContent).toBe("BuffBot 1.16.0 — party presets");
    // The release page is not opened from here: the app has no external-link helper.
    expect(section.querySelector("a")).toBeNull();
    expect(section.textContent).toContain("Your saves, BuffBot settings and exported presets stay as they are.");
    expect(section.textContent).toContain("Close the game first. Undo puts the game files back; progress you saved after the update stays in those saves.");
    expect((getByRole(section, "checkbox", { name: /Back up the whole game folder/ }) as HTMLInputElement).checked).toBe(true);
    expect((getByRole(section, "checkbox", { name: /Copy your saved games/ }) as HTMLInputElement).checked).toBe(false);
    expect(section.textContent).toContain("4 GiB");
    expect(queryByRole(section, "button", { name: /^Undo/ })).toBeNull();
    fireEvent.click(getByRole(section, "checkbox", { name: /Back up the whole game folder/ }));
    fireEvent.click(getByRole(section, "checkbox", { name: /Copy your saved games/ }));
    fireEvent.click(getByRole(section, "button", { name: "Update BuffBot" }));
    expect(callbacks.onApplyModUpdate).toHaveBeenCalledWith("buffbot", false, true);
  });

  it("keeps Update disabled without a review token and never copies saves it cannot read", () => {
    const callbacks = checked({ reviewToken: null, saveBackupBytes: "unavailable" });
    const section = render(callbacks);
    const apply = getByRole(section, "button", { name: "Update BuffBot" }) as HTMLButtonElement;
    expect(apply.disabled).toBe(true);
    fireEvent.click(apply);
    expect(callbacks.onApplyModUpdate).not.toHaveBeenCalled();
    expect((getByRole(section, "checkbox", { name: /Copy your saved games/ }) as HTMLInputElement).disabled).toBe(true);
    expect(section.textContent).toContain("Saved games cannot be copied automatically for this installation.");
  });

  it("explains a refusal and offers no way to update", () => {
    const refusals = ["Snowytoes Hamlet comes after it in the mod list, but the installer did not put it there; WeiDU would have to install it again."];
    const section = render(checked({ state: "refused", reviewToken: null, refusals, detail: "The BuffBot update is not offered for this game. Your game has not been changed." }));
    expect(section.querySelector(".mod-update-refusals")?.textContent).toBe(`Why it is not offered:${refusals[0]}`);
    expect(section.textContent).toContain("Your game has not been changed.");
    expect(queryByRole(section, "button", { name: "Update BuffBot" })).toBeNull();
    expect(section.textContent).not.toContain("Close the game first.");
  });

  it("leaves out versions it does not know", () => {
    const section = render(checked({ state: "not-installed", installedVersion: null, targetVersion: null, releaseTitle: null, reviewToken: null }));
    expect(section.querySelector(".mod-update-installed")).toBeNull();
    expect(section.querySelector(".mod-update-target")).toBeNull();
    expect(section.querySelector(".mod-update-release")).toBeNull();
  });

  it("offers Undo of the update when it is the latest change", () => {
    const callbacks = checked({ state: "up-to-date", reviewToken: null, installedVersion: "1.16.0", canUndo: true, undoTitle: "BuffBot 1.16.0" });
    const section = render(callbacks);
    expect(section.textContent).toContain("Close the game first.");
    fireEvent.click(getByRole(section, "button", { name: "Undo BuffBot 1.16.0" }));
    expect(callbacks.onUndoModUpdate).toHaveBeenCalledWith("buffbot");
  });

  it("asks to restore an unfinished change before playing", () => {
    const callbacks = checked({ state: "needs-recovery", reviewToken: null, installedVersion: null, targetVersion: null, canRestore: true });
    const section = render(callbacks);
    expect(section.textContent).toContain("Restore the game files before playing");
    expect(section.textContent).toContain("Close the game first.");
    fireEvent.click(getByRole(section, "button", { name: "Restore game files" }));
    expect(callbacks.onRestoreModUpdate).toHaveBeenCalledWith("buffbot");
    const stuck = render(checked({ state: "needs-recovery", reviewToken: null, canRestore: false }));
    expect(queryByRole(stuck, "button", { name: "Restore game files" })).toBeNull();
  });

  it("locks every control while any mod update runs and shows progress only on its own card", () => {
    const busy = render(checked({}, { modUpdateBusy: true, modUpdateBusyId: "buffbot" }));
    for (const control of busy.querySelectorAll("button, input")) expect((control as HTMLButtonElement).disabled).toBe(true);
    expect(busy.getAttribute("aria-busy")).toBe("true");
    expect(busy.querySelector(".patch-progress")?.textContent).toBe("Checking, backing up, updating or restoring… Keep the app open and the game closed.");
    const other = render(checked({}, { modUpdateBusy: true, modUpdateBusyId: "another-mod" }));
    for (const control of other.querySelectorAll("button")) expect((control as HTMLButtonElement).disabled).toBe(true);
    expect(other.querySelector(".patch-progress")).toBeNull();
  });

  it("locks every control while a hotpatch runs", () => {
    const callbacks = checked({ canUndo: true, undoTitle: "BuffBot 1.16.0" });
    const section = render(callbacks, true);
    for (const control of section.querySelectorAll("button, input")) expect((control as HTMLButtonElement).disabled).toBe(true);
    fireEvent.click(getByRole(section, "button", { name: "Check for a BuffBot update" }));
    fireEvent.click(getByRole(section, "button", { name: "Update BuffBot" }));
    fireEvent.click(getByRole(section, "button", { name: "Undo BuffBot 1.16.0" }));
    expect(callbacks.onCheckModUpdate).not.toHaveBeenCalled();
    expect(callbacks.onApplyModUpdate).not.toHaveBeenCalled();
    expect(callbacks.onUndoModUpdate).not.toHaveBeenCalled();
  });

  it("shows no result of another installation or mod, and no check of an unavailable folder", () => {
    expect(render(checked({ installId: "second" })).textContent).toContain("Not checked yet");
    expect(render(checked({ modId: "other" })).textContent).toContain("Not checked yet");
    const stale = render(actions(), false, { ...copy, state: "stale" });
    expect((getByRole(stale, "button", { name: "Check for a BuffBot update" }) as HTMLButtonElement).disabled).toBe(true);
    expect(stale.textContent).toContain("This folder is unavailable.");
  });

  it("shows the last error as an alert", () => {
    const section = render(actions({ modUpdateErrors: { buffbot: "The update could not be completed. WeiDU failed." } }));
    expect(getByRole(section, "alert").textContent).toBe("The update could not be completed. WeiDU failed.");
  });
});

describe("BuffBot under Updates", () => {
  async function updates(backend: FixtureBackend): Promise<HTMLElement> {
    const root = document.createElement("div");
    document.body.append(root);
    const app = await mountApp(root, backend);
    await app.navigate("updates");
    return root;
  }

  it.each(["patchBusy", "modUpgradeBusy"] as const)("disables every BuffBot action while %s", async (busy) => {
    const summary = { ...await new FixtureBackend().getUpdates(), managedCopies: [copy] };
    for (const state of ["available", "up-to-date", "needs-recovery"] as const) {
      const callbacks = checked({ state, canUndo: state === "up-to-date", canRestore: state === "needs-recovery" });
      const controls = {
        ...callbacks, [busy]: true,
        checkAgain: vi.fn(), installApplication: vi.fn(), buildUpdatedCopy: vi.fn(),
      };
      const root = updatesScreen(summary, controls);
      const card = root.querySelector<HTMLElement>(".mod-update-section")!;
      const buttons = [...card.querySelectorAll<HTMLButtonElement>("button")];
      expect(buttons.length).toBeGreaterThan(1);
      for (const button of buttons) {
        expect(button.disabled, button.textContent ?? "").toBe(true);
        fireEvent.click(button);
      }
      for (const action of [callbacks.onCheckModUpdate, callbacks.onApplyModUpdate, callbacks.onUndoModUpdate, callbacks.onRestoreModUpdate]) {
        expect(action).not.toHaveBeenCalled();
      }
      const idle = updatesScreen(summary, { ...controls, [busy]: false });
      expect(idle.querySelector<HTMLButtonElement>(".mod-update-section button")?.disabled).toBe(false);
    }
  });

  it("checks only when asked, updates and undoes", async () => {
    const backend = new FixtureBackend();
    const inspect = vi.spyOn(backend, "inspectModUpdate");
    const apply = vi.spyOn(backend, "applyModUpdate");
    const restore = vi.spyOn(backend, "restoreModUpdate");
    const root = await updates(backend);
    const card = () => root.querySelector<HTMLElement>(".mod-update-section")!;
    expect(card().textContent).toContain("Not checked yet. Your game stays unchanged until you choose Update.");
    expect(inspect).not.toHaveBeenCalled();
    fireEvent.click(getByRole(root, "button", { name: "Check for a BuffBot update" }));
    await waitFor(() => expect(getByRole(root, "button", { name: "Update BuffBot" })).toBeTruthy());
    expect(inspect).toHaveBeenCalledWith("fixture-install", "buffbot");
    expect(card().textContent).toContain("Installed: 1.12.6");
    fireEvent.click(getByRole(root, "button", { name: "Update BuffBot" }));
    await waitFor(() => expect(apply).toHaveBeenCalledWith("fixture-install", "buffbot", "fixture-mod-update-fixture-install-buffbot", true, false, expect.any(Function)));
    await waitFor(() => expect(getByRole(root, "button", { name: "Undo BuffBot 1.16.0" })).toBeTruthy());
    expect(card().textContent).toContain("Up to date");
    expect(root.textContent).toContain("BuffBot updated");
    fireEvent.click(getByRole(root, "button", { name: "Undo BuffBot 1.16.0" }));
    await waitFor(() => expect(restore).toHaveBeenCalledWith("fixture-install", "buffbot", true, expect.any(Function)));
    await waitFor(() => expect(getByRole(root, "button", { name: "Update BuffBot" })).toBeTruthy());
    expect(root.textContent).toContain("BuffBot update undone");
  });

  it("never runs next to another game operation", async () => {
    const backend = new FixtureBackend();
    let finish: () => void = () => {};
    const waiting = new Promise<void>(resolve => { finish = resolve; });
    const applyModUpdate = backend.applyModUpdate.bind(backend);
    vi.spyOn(backend, "applyModUpdate").mockImplementation(async (...args) => {
      await waiting;
      return applyModUpdate(...args);
    });
    const root = await updates(backend);
    const button = (name: string) => getByRole<HTMLButtonElement>(root, "button", { name });
    fireEvent.click(button("Check for a BuffBot update"));
    await waitFor(() => expect(button("Update BuffBot").disabled).toBe(false));
    fireEvent.click(button("Update BuffBot"));
    await waitFor(() => expect(button("Check for a BuffBot update").disabled).toBe(true));
    for (const other of root.querySelectorAll<HTMLButtonElement>(".install-patches:not(.mod-update-section) button")) {
      expect(other.disabled).toBe(true);
    }
    finish();
    await waitFor(() => expect(button("Check for a BuffBot update").disabled).toBe(false));
  });
});
