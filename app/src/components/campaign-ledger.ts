import type { BuildPhase, PhaseSummary } from "../contracts";
import { element } from "./app-shell";

export function campaignLedger(phases: readonly (PhaseSummary | BuildPhase)[], announce = false): HTMLElement {
  const wrapper = element("section", "ledger-card card");
  wrapper.setAttribute("aria-labelledby", "ledger-heading");
  const heading = element("h2", undefined, "Campaign ledger");
  heading.id = "ledger-heading";
  const list = element("ol", "ledger");
  let currentTitle = "";
  phases.forEach((phase) => {
    const item = element("li");
    item.dataset.ledgerPhase = phase.id;
    const state = "state" in phase ? phase.state : "pending";
    item.dataset.state = state;
    if (state === "current" || state === "failed") currentTitle = phase.title;
    const body = element("div");
    body.append(element("strong", undefined, phase.title), element("span", undefined, phase.detail));
    item.append(body);
    list.append(item);
  });
  wrapper.append(heading, list);
  if (announce) {
    const live = element("p", "visually-hidden", currentTitle ? `Current phase: ${currentTitle}` : "All phases complete");
    live.setAttribute("role", "status");
    live.setAttribute("aria-live", "polite");
    live.setAttribute("aria-atomic", "true");
    wrapper.append(live);
  }
  return wrapper;
}
