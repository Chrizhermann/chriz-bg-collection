// @vitest-environment jsdom

import axe from "axe-core";
import { getByRole, waitFor } from "@testing-library/dom";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it } from "vitest";

import { mountApp } from "../src/app";
import { FixtureBackend } from "../src/backend";
import styles from "../src/styles.css?raw";

const routes = [
  "home",
  "updates",
  "welcome",
  "games",
  "destination",
  "setup",
  "review",
  "build",
  "complete",
] as const;

describe("CEBG security and accessibility", () => {
  afterEach(() => document.body.replaceChildren());

  it("has no axe violations on any screen", async () => {
    const root = document.createElement("div");
    document.body.append(root);
    const handle = await mountApp(root, new FixtureBackend());

    for (const route of routes) {
      await handle.navigate(route);
      // jsdom cannot supply the canvas API used by axe's visual contrast rule.
      // The palette is checked in the rendered-browser smoke; keep every DOM rule active here.
      const result = await axe.run(root, {
        rules: { "color-contrast": { enabled: false } },
      });
      expect(result.violations, route).toEqual([]);
    }
  }, 20_000);

  it("renders hostile backend strings as inert text", async () => {
    const attack = '<img src=x onerror="document.body.dataset.pwned=1"><script>bad()</script>';
    const root = document.createElement("div");
    document.body.append(root);
    const handle = await mountApp(
      root,
      new FixtureBackend({
        managedInstallations: [{
          id: "hostile-install",
          name: attack,
          path: attack,
          status: "Ready to play",
          receiptPath: attack,
          launchPath: attack,
          completedAtMillis: 1,
          available: true,
          resumable: false,
        }],
        textOverrides: {
          campaignName: attack,
          logLine: attack,
          gamePath: attack,
          featureTitle: attack,
          unavailableReason: attack,
        },
      }),
    );

    await handle.navigate("home");
    expect(root.textContent).toContain(attack);
    await handle.navigate("welcome");
    await handle.navigate("games");
    expect(root.textContent).toContain(attack);
    await handle.navigate("setup");
    expect(root.textContent).toContain(attack);
    await handle.navigate("build");
    expect(root.textContent).toContain(attack);
    expect(root.querySelector("img, script")).toBeNull();
    expect(document.body.dataset.pwned).toBeUndefined();
  });

  it("moves focus, keeps unavailable reasons keyboard discoverable, and isolates live announcements", async () => {
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, new FixtureBackend());

    await user.click(getByRole(root, "button", { name: "Customize" }));
    expect(document.activeElement).toBe(getByRole(root, "heading", { level: 1 }));
    const handle = await mountApp(root, new FixtureBackend());
    await handle.navigate("setup");
    const blocked = getByRole(root, "checkbox", { name: /Experimental quest restoration/ });
    expect(blocked.getAttribute("aria-disabled")).toBe("true");
    expect(root.querySelector("#blocked-restoration-reason")?.getAttribute("tabindex")).toBe("0");
    const optional = getByRole(root, "checkbox", { name: /Companion conversations/ }) as HTMLInputElement;
    optional.focus();
    await user.keyboard(" ");
    await waitFor(() => {
      const current = getByRole(root, "checkbox", { name: /Companion conversations/ }) as HTMLInputElement;
      expect(current.checked).toBe(true);
      expect(document.activeElement).toBe(current);
    });

    await user.click(getByRole(root, "button", { name: "Done" }));
    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));
    expect(root.querySelectorAll('[aria-live="polite"]')).toHaveLength(1);
    expect(root.querySelector("pre")?.hasAttribute("aria-live")).toBe(false);
  });

  it("keeps the complete primary workflow keyboard operable", async () => {
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, new FixtureBackend());

    const activate = async (name: string): Promise<void> => {
      const button = getByRole(root, "button", { name });
      button.focus();
      await user.keyboard("{Enter}");
    };

    await activate("Install Chriz Easy BG");
    await activate("I added the archive");
    await activate("Keep waiting");
    await activate("Retry failed step");
    await activate("Finish fixture build");
    expect(getByRole(root, "heading", { level: 1, name: "Ready to play" })).toBeTruthy();
  });

  it("keeps compact layouts and motion preferences in the shared design contract", () => {
    expect(styles).toMatch(/--iron:/);
    expect(styles).toMatch(/--vellum:/);
    expect(styles).toMatch(/--brass:/);
    expect(styles).toMatch(/--teal:/);
    expect(styles).toMatch(/--ember:/);
    expect(styles).toMatch(/min-(height|block-size):\s*44px/);
    expect(styles).toMatch(/@media\s*\(max-width:\s*20rem\)/);
    expect(styles).toMatch(/prefers-reduced-motion:\s*reduce/);
    expect(styles).toMatch(/overflow-wrap:\s*anywhere/);
  });
});
