// @vitest-environment jsdom

import { getByRole, getByText } from "@testing-library/dom";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it } from "vitest";
import type { ManagedInstallation } from "../src/contracts";
import { homeScreen } from "../src/screens/home";

describe("launcher support", () => {
  afterEach(() => document.body.replaceChildren());

  it("shows recorded component status and only advertises BuffBot when present", async () => {
    const diagnostics: string[] = [];
    const installation: ManagedInstallation = {
      id: "ready", name: "Ready game", path: "D:\\CEBG", status: "Ready to play",
      receiptPath: "D:\\CEBG\\.chriz\\install-receipt.json", launchPath: "D:\\CEBG\\game\\InfinityLoader.exe",
      completedAtMillis: 1, available: true, resumable: false, recipeVersion: "alpha.11", radarVersion: "2.5.0.0",
      consistency: { state: "changed", detail: "The mod list changed.", componentCount: 2, modCount: 1, components: [
        { target: "BG2", tp2: "buffbot/setup-buffbot.tp2", component: 0, title: "BuffBot: In-Game Buff Automation", version: "v1.8.3-alpha", status: "installed" },
        { target: "BG2", tp2: "lost/setup-lost.tp2", component: 4, title: null, version: null, status: "missing" },
        { target: "BG2", tp2: "extra/setup-extra.tp2", component: 9, title: "Extra tweak", version: null, status: "extra" },
      ] },
    };
    const root = homeScreen([installation], installation, {
      begin: () => {}, launch: () => {}, openFolder: () => {}, resume: () => {}, select: () => {}, createShortcut: () => {},
      diagnostics: (installId) => { diagnostics.push(installId); },
    });
    document.body.append(root);

    await userEvent.setup().click(getByText(root, /Installed mods/));
    expect(getByText(root, "BuffBot: In-Game Buff Automation")).toBeTruthy();
    expect(root.textContent).toContain("v1.8.3-alpha · Component 0 · BG2 · Recorded and present");
    expect(root.textContent).toContain("Recorded but missing");
    expect(root.textContent).toContain("Present but not recorded");
    const search = getByRole(root, "searchbox", { name: "Find an installed mod" });
    await userEvent.setup().type(search, "setup-extra.tp2");
    expect(getByText(root, "Extra tweak").closest("li")?.hidden).toBe(false);
    expect(getByText(root, "BuffBot: In-Game Buff Automation").closest("li")?.hidden).toBe(true);
    expect(root.textContent).toContain("press F11 to open BuffBot");
    expect(root.textContent).toContain("BG Radar Overlay 2.5.0.0 is installed");
    expect(getByText(root, "First-play tips").closest<HTMLDetailsElement>("details")?.open).toBe(false);
    await userEvent.setup().click(getByRole(root, "button", { name: "Export diagnostics" }));
    expect(diagnostics).toEqual(["ready"]);
  });

  it("does not make a BuffBot activation claim without an installed log row", () => {
    const installation: ManagedInstallation = {
      id: "plain", name: "Plain game", path: "D:\\CEBG", status: "Ready", receiptPath: null,
      launchPath: "D:\\CEBG\\game\\InfinityLoader.exe", completedAtMillis: 1, available: true, resumable: false,
      consistency: { state: "matches", detail: "Matches.", componentCount: 1, modCount: 1, components: [
        { target: "BG2", tp2: "buffbot/setup-buffbot.tp2", component: 0, title: null, version: null, status: "missing" },
      ] },
    };
    const root = homeScreen([installation], installation, {
      begin: () => {}, launch: () => {}, openFolder: () => {}, resume: () => {}, select: () => {}, createShortcut: () => {}, diagnostics: () => {},
    });
    expect(root.textContent).not.toContain("F11");
    expect(root.textContent).toContain("Radar Overlay is optional");
  });
});
