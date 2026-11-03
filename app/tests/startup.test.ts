// @vitest-environment jsdom

import { fireEvent, getByRole, queryByRole, waitFor } from "@testing-library/dom";
import axe from "axe-core";
import { afterEach, describe, expect, it, vi } from "vitest";
import { mountApp } from "../src/app";
import { FixtureBackend } from "../src/backend";
import type { GameDiscovery } from "../src/contracts";
import html from "../index.html?raw";
import { loadingScreen } from "../src/screens/loading";

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (error: Error) => void;
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}

function rootElement() {
  const root = document.createElement("div");
  document.body.append(root);
  return root;
}

afterEach(() => { document.body.replaceChildren(); vi.restoreAllMocks(); });

describe("visible startup before slow native checks", () => {
  it("ships readable loading content and CSS before JavaScript starts", () => {
    const page = new DOMParser().parseFromString(html, "text/html");
    expect(page.querySelector('#app [role="status"]')?.textContent).toContain("Getting Chriz Easy BG ready");
    expect(page.querySelector('link[rel="stylesheet"]')?.getAttribute("href")).toBe("/src/styles.css");
  });

  it("paints immediately while bootstrap and the installation registry are pending", async () => {
    const backend = new FixtureBackend();
    const status = await backend.getStatus();
    const gate = deferred<typeof status>();
    vi.spyOn(backend, "getStatus").mockReturnValue(gate.promise);
    const registry = deferred<Awaited<ReturnType<typeof backend.listManagedInstallations>>>();
    vi.spyOn(backend, "listManagedInstallations").mockReturnValue(registry.promise);
    const root = rootElement();
    const mounting = mountApp(root, backend);
    expect(getByRole(root, "status").textContent).toContain("Getting Chriz Easy BG ready");
    expect(root.querySelector('.loading-spinner[aria-hidden="true"]')).not.toBeNull();
    expect(queryByRole(root, "button", { name: "Install Chriz Easy BG" })).toBeNull();
    gate.resolve(status);
    await Promise.resolve();
    expect(getByRole(root, "status").textContent).toContain("Getting Chriz Easy BG ready");
    registry.resolve([]);
    await mounting;
    expect(getByRole(root, "heading", { name: "Install Chriz Easy BG" })).toBeTruthy();
    expect(root.querySelector(".loading-spinner")).toBeNull();
  });

  it("keeps the detection message visible until games and setup are ready", async () => {
    const backend = new FixtureBackend();
    vi.spyOn(backend, "listManagedInstallations").mockResolvedValue([]);
    const discovery = await backend.discoverGames();
    const gate = deferred<GameDiscovery>();
    vi.spyOn(backend, "discoverGames").mockReturnValue(gate.promise);
    const root = rootElement();
    const mounting = mountApp(root, backend);
    await waitFor(() => expect(getByRole(root, "status").textContent).toContain("Looking for your Baldur's Gate games"));
    expect(root.textContent).toContain("slower drives");
    expect(queryByRole(root, "button")).toBeNull();
    gate.resolve(discovery);
    await mounting;
    expect(getByRole(root, "heading", { name: "Install Chriz Easy BG" })).toBeTruthy();
    expect(root.querySelector(".loading-spinner")).toBeNull();
  });

  it("replaces loading with actionable error text if detection fails", async () => {
    const backend = new FixtureBackend();
    vi.spyOn(backend, "listManagedInstallations").mockResolvedValue([]);
    vi.spyOn(backend, "discoverGames").mockRejectedValue(new Error("The source drive is unavailable"));
    const root = rootElement();
    await expect(mountApp(root, backend)).rejects.toThrow("source drive");
    expect(getByRole(root, "alert").textContent).toContain("The source drive is unavailable");
    expect(getByRole(root, "button", { name: "Try again" })).toBeTruthy();
    expect(root.querySelector(".loading-spinner")).toBeNull();
  });

  it("continues showing progress while the recommended mod choices are checked", async () => {
    const backend = new FixtureBackend();
    vi.spyOn(backend, "listManagedInstallations").mockResolvedValue([]);
    const evaluate = backend.evaluateBuild.bind(backend);
    const gate = deferred<void>();
    vi.spyOn(backend, "evaluateBuild").mockImplementation(async (...args) => {
      await gate.promise;
      return evaluate(...args);
    });
    const root = rootElement();
    const mounting = mountApp(root, backend);
    await waitFor(() => expect(getByRole(root, "status").textContent).toContain("Preparing your recommended setup"));
    expect(queryByRole(root, "button")).toBeNull();
    gate.resolve();
    await mounting;
    expect(getByRole(root, "heading", { name: "Install Chriz Easy BG" })).toBeTruthy();
  });

  it("retries a failed startup without leaving a stuck spinner or duplicate screens", async () => {
    const backend = new FixtureBackend();
    vi.spyOn(backend, "listManagedInstallations").mockResolvedValue([]);
    vi.spyOn(backend, "discoverGames").mockRejectedValueOnce(new Error("Drive disconnected"));
    const root = rootElement();
    await expect(mountApp(root, backend)).rejects.toThrow("Drive disconnected");
    fireEvent.click(getByRole(root, "button", { name: "Try again" }));
    await waitFor(() => expect(getByRole(root, "heading", { name: "Install Chriz Easy BG" })).toBeTruthy());
    expect(root.querySelectorAll("main")).toHaveLength(1);
    expect(root.querySelector(".loading-spinner")).toBeNull();
    expect(queryByRole(root, "alert")).toBeNull();
  });

  it("announces loading accessibly without exposing decorative motion", async () => {
    const root = rootElement();
    root.append(loadingScreen("games"));
    const result = await axe.run(root, { rules: { "color-contrast": { enabled: false } } });
    expect(result.violations).toEqual([]);
    expect(getByRole(root, "status").getAttribute("aria-live")).toBe("polite");
  });
});
