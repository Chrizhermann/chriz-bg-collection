// @vitest-environment jsdom

import { fireEvent, getByRole, getByText, queryByText } from "@testing-library/dom";
import { describe, expect, it } from "vitest";
import type { UpdateSummary } from "../src/contracts";
import { createAppShell } from "../src/components/app-shell";
import { updateUpdatesControl } from "../src/components/update-notification";

function summary(application: UpdateSummary["application"]["state"], recipe: UpdateSummary["recipe"]["state"], radar?: UpdateSummary["radar"]): UpdateSummary {
  return {
    checkedAt: "2026-09-05T12:00:00Z",
    networkState: "online",
    application: { state: application, currentVersion: "1", availableVersion: application === "available" ? "2" : null, detail: "" },
    recipe: { state: recipe, currentVersion: "1", availableVersion: recipe === "available" ? "2" : null, disposition: "deferred-for-next-playthrough", detail: "", changes: [] },
    managedCopies: [], radar,
  };
}

function updatesButton(): { shell: HTMLElement; button: HTMLButtonElement } {
  const shell = createAppShell("home", document.createElement("div"), () => undefined);
  return { shell, button: getByRole(shell, "button", { name: "Updates" }) as HTMLButtonElement };
}

describe("Updates notification", () => {
  it.each([
    ["CEBG app", summary("available", "up-to-date")],
    ["collection", summary("up-to-date", "available")],
    ["CEBG app and collection", summary("available", "available")],
    ["CEBG app and collection", { ...summary("available", "requires-app"), recipe: { ...summary("available", "requires-app").recipe, availableVersion: "2" } }],
    ["BG Radar Overlay", summary("up-to-date", "up-to-date", { state: "available", currentVersion: "1", availableVersion: "2", detail: "" })],
  ])("shows a New badge and names the available %s update", (name, updates) => {
    const { shell, button } = updatesButton();
    updateUpdatesControl(button, updates);
    expect(getByText(shell, "New")).toBeTruthy();
    expect(getByRole(shell, "tooltip").textContent).toContain(name);
    expect(button.getAttribute("aria-label")).toBe("Updates");
    expect(button.getAttribute("aria-describedby")).toBe(getByRole(shell, "tooltip").id);
  });

  it.each([
    summary("up-to-date", "up-to-date"),
    summary("offline", "unavailable"),
    summary("invalid", "replayed"),
    summary("up-to-date", "up-to-date", { state: "not-installed", currentVersion: null, availableVersion: "2", detail: "" }),
    summary("up-to-date", "up-to-date", { state: "available", currentVersion: null, availableVersion: "2", detail: "" }),
  ])("does not mark unavailable, stale, or install-only versions as new", (updates) => {
    const { shell, button } = updatesButton();
    updateUpdatesControl(button, updates);
    expect(queryByText(shell, "New")).toBeNull();
    expect(button.dataset.updateAvailable).toBe("false");
  });

  it("replaces notification details without duplicating badges or tooltips", () => {
    const { shell, button } = updatesButton();
    updateUpdatesControl(button, summary("available", "available"));
    updateUpdatesControl(button, summary("available", "up-to-date"));
    expect(shell.querySelectorAll(".update-badge")).toHaveLength(1);
    expect(shell.querySelectorAll('[role="tooltip"]')).toHaveLength(1);
    expect(getByRole(shell, "tooltip").textContent).toContain("CEBG app");
    expect(getByRole(shell, "tooltip").textContent).not.toContain("collection");
  });

  it("reports a collection update when a managed installation can be rebuilt", () => {
    const updates = summary("up-to-date", "up-to-date");
    const { shell, button } = updatesButton();
    updateUpdatesControl(button, { ...updates, managedCopies: [{
      installId: "game", name: "Game", path: "D:\\Game", installedRecipeVersion: "1",
      state: "update-available", detail: "A newer collection is available.",
    }] });
    expect(getByText(shell, "New")).toBeTruthy();
    expect(getByRole(shell, "tooltip").textContent).toBe("New collection update available.");
  });

  it("dismisses the custom tooltip with Escape until focus is re-entered", () => {
    const { shell, button } = updatesButton();
    updateUpdatesControl(button, summary("available", "up-to-date"));
    const tooltip = getByRole(shell, "tooltip");
    fireEvent.focus(button);
    fireEvent.keyDown(button, { key: "Escape" });
    expect(tooltip.hidden).toBe(true);
    fireEvent.blur(button);
    fireEvent.focus(button);
    expect(tooltip.hidden).toBe(false);
    expect(button.hasAttribute("title")).toBe(false);
  });
});
