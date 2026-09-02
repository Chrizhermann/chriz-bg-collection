// @vitest-environment jsdom

import { getByLabelText, getByRole, getByText, queryByText } from "@testing-library/dom";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it } from "vitest";

import { FixtureBackend } from "../src/backend";
import { mountApp } from "../src/app";
import { technicalLog } from "../src/components/technical-log";

describe("guided collection wizard", () => {
  afterEach(() => document.body.replaceChildren());

  it("walks all seven wizard screens, preserves Back, and freezes Review before Build", async () => {
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, new FixtureBackend());

    expect(getByRole(root, "heading", { level: 1, name: "Welcome" })).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Begin setup" }));
    expect(getByRole(root, "heading", { level: 1, name: "Find your games" })).toBeTruthy();

    const bg1 = getByLabelText(root, "Baldur's Gate source") as HTMLSelectElement;
    const bg2 = getByLabelText(root, "Baldur's Gate II source") as HTMLSelectElement;
    const originalBg2 = bg2.value;
    await user.selectOptions(bg1, "bg1-modified");
    expect(bg2.value).toBe(originalBg2);
    expect(getByText(root, "Files differ from a clean store installation.")).toBeTruthy();
    await user.selectOptions(bg1, "bg1-fresh");
    await user.click(getByRole(root, "button", { name: "Continue" }));

    expect(getByRole(root, "heading", { level: 1, name: "Choose a destination" })).toBeTruthy();
    expect(getByText(root, "Safe separate destination")).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Continue" }));

    expect(getByRole(root, "heading", { level: 1, name: "Shape your campaign" })).toBeTruthy();
    const mandatory = getByRole(root, "checkbox", { name: /Curated foundation/ }) as HTMLInputElement;
    const recommended = getByRole(root, "checkbox", { name: /Recommended rules balance/ }) as HTMLInputElement;
    const optional = getByRole(root, "checkbox", { name: /Companion conversations/ }) as HTMLInputElement;
    const blocked = getByRole(root, "checkbox", { name: /Experimental quest restoration/ });
    expect(mandatory.checked).toBe(true);
    expect(mandatory.getAttribute("aria-disabled")).toBe("true");
    expect(recommended.checked).toBe(true);
    expect(recommended.hasAttribute("aria-disabled")).toBe(false);
    expect(optional.checked).toBe(false);
    expect(optional.hasAttribute("aria-disabled")).toBe(false);
    expect(blocked.getAttribute("aria-disabled")).toBe("true");
    expect(getByText(root, "Deferred until its installer can be reproduced safely.")).toBeTruthy();
    expect(getByText(root, "Experimental quest restoration remains visible but is omitted.")).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Continue" }));

    expect(getByRole(root, "heading", { level: 1, name: "Review the campaign ledger" })).toBeTruthy();
    expect(root.querySelectorAll("[data-ledger-phase]")).toHaveLength(5);
    expect(getByText(root, "Experimental quest restoration remains visible but is omitted.")).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Back" }));
    expect(getByRole(root, "heading", { level: 1, name: "Shape your campaign" })).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Continue" }));
    await user.click(getByRole(root, "button", { name: "Freeze review and build" }));

    expect(getByRole(root, "heading", { level: 1, name: "Build your campaign" })).toBeTruthy();
    expect(getByText(root, "Manual archive needed")).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "I added the archive" }));
    expect(getByText(root, "Your attention is needed")).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Continue build" }));
    expect(getByText(root, "A fixture step failed")).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Export diagnostics" }));
    expect(getByText(root, /Diagnostics exported to/)).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Retry failed step" }));
    expect(getByText(root, "Build in progress")).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Finish fixture build" }));

    expect(getByRole(root, "heading", { level: 1, name: "Campaign complete" })).toBeTruthy();
    expect(getByText(root, "Immutable install receipt")).toBeTruthy();
  });

  it("surfaces every non-fresh source reason", async () => {
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    const handle = await mountApp(root, new FixtureBackend());
    await handle.navigate("games");

    const cases = [
      ["bg1-modified", "Files differ from a clean store installation."],
      ["bg1-old", "The installed game version is not supported."],
      ["bg1-nosod", "Siege of Dragonspear is required for this EET recipe."],
    ] as const;
    for (const [id, message] of cases) {
      await user.selectOptions(getByLabelText(root, "Baldur's Gate source"), id);
      expect(getByText(root, message)).toBeTruthy();
    }
    await user.selectOptions(getByLabelText(root, "Baldur's Gate II source"), "bg2-store");
    expect(getByText(root, "This storefront layout has not been verified yet.")).toBeTruthy();
  });

  it("bounds the selectable technical tail and allows auto-scroll to be paused", async () => {
    let logState = { paused: false, open: false };
    const log = technicalLog(
      Array.from({ length: 250 }, (_, index) => `line-${index}`),
      logState,
      (state) => { logState = state; },
    );
    document.body.append(log);
    const output = log.querySelector("pre");
    expect(output?.tabIndex).toBe(0);
    expect(output?.textContent?.split("\n")).toHaveLength(200);
    expect(output?.textContent).not.toContain("line-0\n");
    const pause = getByRole(log, "button", { name: "Pause auto-scroll" });
    await userEvent.setup().click(pause);
    expect(pause.getAttribute("aria-pressed")).toBe("true");
    expect(pause.textContent).toBe("Resume auto-scroll");
    expect(logState.paused).toBe(true);
    const restored = technicalLog(["later line"], logState);
    expect(getByRole(restored, "button", { name: "Resume auto-scroll" })).toBeTruthy();
  });

  it("shows returning managed installations and the separate Updates screen", async () => {
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, new FixtureBackend());

    await user.click(getByRole(root, "button", { name: "Campaigns" }));
    expect(getByRole(root, "heading", { level: 1, name: "Your campaigns" })).toBeTruthy();
    expect(getByText(root, "Chriz EET — Stream test")).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Updates" }));
    expect(getByRole(root, "heading", { level: 1, name: "Updates" })).toBeTruthy();
    expect(getByText(root, "Existing campaigns are never patched in place.")).toBeTruthy();
    expect(queryByText(root, "Update now")).toBeNull();
  });

  it("does not let a late evaluation overwrite a newer selection", async () => {
    const backend = new FixtureBackend({ evaluationDelays: [0, 80, 0] });
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    const handle = await mountApp(root, backend);
    await handle.navigate("setup");

    const optional = getByRole(root, "checkbox", { name: /Companion conversations/ });
    await user.click(optional);
    await user.click(optional);
    await new Promise((resolve) => window.setTimeout(resolve, 100));

    expect((optional as HTMLInputElement).checked).toBe(false);
    expect(getByText(root, "2 campaign choices selected")).toBeTruthy();
  });
});
