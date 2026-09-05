import type { BuildPhase, PhaseSummary } from "../contracts";
import { element } from "./app-shell";

const phaseCopy: Readonly<Record<string, readonly [string, string]>> = {
  preparation: ["Copy your games", "Prepare a separate game folder."],
  "bg1-preparation": ["Prepare Baldur's Gate", "Add the required BG1 fixes and content."],
  bg1: ["Prepare Baldur's Gate", "Add the required BG1 fixes and content."],
  "bg2-preparation": ["Prepare Baldur's Gate II", "Add the required BG2 fixes."],
  "eet-initialization": ["Combine the games", "Connect BG1, SoD and BG2 with EET."],
  merge: ["Combine the games", "Connect BG1, SoD and BG2 with EET."],
  main: ["Install your mods", "Add your chosen content and rules."],
  "eet-finalization": ["Finish the game setup", "Complete the combined game world."],
  "post-eet-end": ["Add the finishing touches", "Install the final fixes and utilities."],
  final: ["Add the finishing touches", "Complete the game setup and final fixes."],
};

export function campaignLedger(phases: readonly (PhaseSummary | BuildPhase)[], announce = false): HTMLElement {
  const wrapper = element("section", "ledger-card card");
  wrapper.setAttribute("aria-labelledby", "ledger-heading");
  const heading = element("h2", undefined, "Installation steps");
  heading.id = "ledger-heading";
  const list = element("ol", "ledger");
  let currentTitle = "";
  phases.forEach((phase) => {
    const [title, detail] = phaseCopy[phase.id] ?? [phase.title, phase.detail];
    const item = element("li");
    item.dataset.ledgerPhase = phase.id;
    const state = "state" in phase ? phase.state : "pending";
    item.dataset.state = state;
    if (state === "current" || state === "failed") currentTitle = title;
    const body = element("div");
    body.append(element("strong", undefined, title), element("span", undefined, detail));
    item.append(body);
    list.append(item);
  });
  wrapper.append(heading, list);
  if (announce) {
    const allDone = phases.length > 0 && phases.every((phase) => "state" in phase && phase.state === "done");
    const announcement = currentTitle ? `Current phase: ${currentTitle}`
      : allDone ? "All phases complete"
      : phases.length === 0 ? "Installation progress is not available yet."
      : "Waiting to start installation steps.";
    const live = element("p", "visually-hidden", announcement);
    live.setAttribute("role", "status");
    live.setAttribute("aria-live", "polite");
    live.setAttribute("aria-atomic", "true");
    wrapper.append(live);
  }
  return wrapper;
}
