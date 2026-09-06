// @vitest-environment jsdom

import { afterEach, describe, expect, it, vi } from "vitest";
import type { BackendStatus } from "../src/contracts";
import {
  clearSetupDraft,
  loadSetupDraft,
  saveSetupDraft,
  SETUP_DRAFT_STORAGE_KEY,
  type SetupDraft,
} from "../src/setup-draft";

const status: BackendStatus = {
  mode: "native", engineVersion: "0.1.0", recipeVersion: "recipe-12", startupInstallId: null,
  selectedProfile: "recommended",
  profiles: [
    { id: "recommended", label: "Recommended", description: "Recommended choices" },
    { id: "full", label: "Full", description: "Full choices" },
  ],
};

function draft(): SetupDraft {
  return {
    version: 1,
    recipeVersion: "recipe-12",
    profileId: "full",
    installationName: "My adventure",
    destinationPath: "D:\\My adventure",
    destinationAutomatic: false,
    sourcePaths: { bg1: "D:\\Clean BG1", bg2: "E:\\Clean BG2" },
    selection: {
      platform: "windows",
      features: { "companion-conversations": true },
      inputs: { "companion-conversations": {
        enabled: { kind: "boolean", value: true },
        amount: { kind: "integer", value: 3 },
        choice: { kind: "choice", value: "friendly" },
      } },
    },
    createDesktopShortcut: false,
  };
}

afterEach(() => {
  vi.restoreAllMocks();
  window.localStorage.clear();
});

describe("setup draft preferences", () => {
  it("round-trips only allowed preferences for the same recipe and an available profile", () => {
    const expected = draft();
    saveSetupDraft({ ...expected, reviewToken: "do-not-store", build: { state: "running" }, receipt: "do-not-store" } as SetupDraft);
    expect(JSON.parse(window.localStorage.getItem(SETUP_DRAFT_STORAGE_KEY)!)).toEqual(expected);
    expect(loadSetupDraft(status)).toEqual({ draft: expected, notice: null });
    clearSetupDraft();
    expect(loadSetupDraft(status)).toEqual({ draft: null, notice: null });
  });

  it.each([
    ["recipe changed", { ...status, selectedProfile: "full", recipeVersion: "recipe-13" }],
    ["recipe unknown", { ...status, selectedProfile: "full", recipeVersion: null }],
    ["profile removed", { ...status, profiles: status.profiles?.slice(0, 1) }],
  ])("discards incompatible choices when %s", (_label, currentStatus) => {
    saveSetupDraft(draft());
    const loaded = loadSetupDraft(currentStatus);
    expect(loaded.draft).toBeNull();
    expect(loaded.notice).toContain("saved setup");
    expect(window.localStorage.getItem(SETUP_DRAFT_STORAGE_KEY)).toBeNull();
  });

  it.each([
    "not json",
    JSON.stringify({ ...draft(), version: 2 }),
    JSON.stringify({ ...draft(), frozenReview: { reviewToken: "old authority" } }),
    JSON.stringify({ ...draft(), evaluation: { safe: true } }),
    JSON.stringify({ ...draft(), sourcePaths: { bg1: { path: "D:\\Forged", eligible: true }, bg2: null } }),
    JSON.stringify({ ...draft(), selection: { platform: "windows", features: { mod: "yes" }, inputs: {} } }),
    JSON.stringify({ ...draft(), selection: { platform: "windows", features: {}, inputs: { mod: { count: { kind: "integer", value: 1.5 } } } } }),
    JSON.stringify({ ...draft(), selection: { platform: "windows", features: {}, inputs: { mod: { choice: { kind: "choice", value: "yes", token: "secret" } } } } }),
  ])("rejects malformed or authority-bearing stored data: %s", (stored) => {
    window.localStorage.setItem(SETUP_DRAFT_STORAGE_KEY, stored);
    expect(loadSetupDraft(status).draft).toBeNull();
    expect(window.localStorage.getItem(SETUP_DRAFT_STORAGE_KEY)).toBeNull();
  });

  it("keeps storage denial nonfatal on reads, writes, and cleanup", () => {
    for (const method of ["getItem", "setItem", "removeItem"] as const) {
      vi.spyOn(Storage.prototype, method).mockImplementation(() => { throw new Error("Storage unavailable"); });
    }
    expect(() => saveSetupDraft(draft())).not.toThrow();
    expect(loadSetupDraft(status)).toEqual({ draft: null, notice: null });
    expect(() => clearSetupDraft()).not.toThrow();
  });

  it("clears only the draft that belongs to a confirmed completed destination", () => {
    saveSetupDraft(draft());
    clearSetupDraft("D:\\Another installation");
    expect(loadSetupDraft(status).draft).toEqual(draft());
    clearSetupDraft("d:/my adventure/");
    expect(window.localStorage.getItem(SETUP_DRAFT_STORAGE_KEY)).toBeNull();
  });
});
