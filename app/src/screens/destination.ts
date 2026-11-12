import type { DestinationEvaluation } from "../contracts";
import { actionButton, element, screenActions, screenIntro } from "../components/app-shell";
import { statusCard } from "../components/status-card";
import { flavor } from "../flavor";

export function destinationScreen(
  evaluation: DestinationEvaluation,
  onInspect: (path: string, liveTyping?: boolean) => void | Promise<void>,
  onBrowse: () => void | Promise<void>,
  back: () => void,
  next: () => void,
): HTMLElement {
  const page = element("div", "screen-stack");
  page.append(screenIntro("", "Choose an install location", `Choose a new folder for ${flavor.productName}.`));
  const field = element("div", "card field-card");
  const label = element("label", undefined, "Install location");
  label.htmlFor = "destination-path";
  const input = element("input");
  input.id = "destination-path";
  input.type = "text";
  input.value = evaluation.path;
  let inspectTimer: number | null = null;
  input.addEventListener("input", () => {
    if (inspectTimer !== null) window.clearTimeout(inspectTimer);
    inspectTimer = window.setTimeout(() => {
      inspectTimer = null;
      if (input.isConnected) void onInspect(input.value, true);
    }, 250);
  });
  input.addEventListener("change", () => {
    if (inspectTimer !== null) {
      window.clearTimeout(inspectTimer);
      inspectTimer = null;
    }
    void onInspect(input.value, false);
  });
  const browseButton = actionButton("Browse…", onBrowse, "quiet");
  browseButton.setAttribute("aria-label", "Browse for install location");
  const space = evaluation.requiredSpace !== undefined && evaluation.availableSpace !== undefined
    ? element("p", "path", `Requires ${evaluation.requiredSpace}; ${evaluation.availableSpace} available.`)
    : element("p", "path", `${flavor.shortName} will check the available space before installing.`);
  const help = element("p", "field-help", `Enter a new folder path or browse. ${flavor.shortName} creates missing folders when you install.`);
  help.id = "destination-path-help";
  input.setAttribute("aria-describedby", help.id);
  field.append(label, input, browseButton, help, space);
  const continueButton = actionButton("Continue", next);
  continueButton.disabled = !evaluation.safe;
  page.append(field, statusCard(evaluation.title, evaluation.detail, evaluation.safe ? "ok" : "danger"), screenActions(back, continueButton));
  return page;
}
