// @vitest-environment jsdom
import { describe, expect, it } from "vitest";
import { campaignLedger } from "../src/components/campaign-ledger";

describe("installation progress announcements", () => {
  it("does not announce completion when a resumed run has no projected phases", () => {
    const ledger = campaignLedger([], true);
    expect(ledger.textContent).not.toContain("All phases complete");
    expect(ledger.querySelector('[role="status"]')?.textContent).toBe("Installation progress is not available yet.");
  });

  it("does not announce completion when all phases are still pending", () => {
    const ledger = campaignLedger([{ id: "main", title: "Mods", detail: "Install mods", state: "pending" }], true);
    expect(ledger.querySelector('[role="status"]')?.textContent).toBe("Waiting to start installation steps.");
  });

  it("announces completion only for a nonempty set of completed phases", () => {
    const ledger = campaignLedger([{ id: "main", title: "Mods", detail: "Install mods", state: "done" }], true);
    expect(ledger.querySelector('[role="status"]')?.textContent).toBe("All phases complete");
  });
});
