import type { DestinationEvaluation } from "../contracts";
import { actionButton, element, screenActions, screenIntro } from "../components/app-shell";
import { statusCard } from "../components/status-card";

export function destinationScreen(
  evaluation: DestinationEvaluation,
  onInspect: (path: string) => void | Promise<void>,
  onBrowse: () => void | Promise<void>,
  back: () => void,
  next: () => void,
): HTMLElement {
  const page = element("div", "screen-stack");
  page.append(screenIntro("", "Choose an install location", "Choose a new folder for Chriz Easy BG."));
  const field = element("div", "card field-card");
  const label = element("label", undefined, "Install location");
  label.htmlFor = "destination-path";
  const input = element("input");
  input.id = "destination-path";
  input.type = "text";
  input.value = evaluation.path;
  input.addEventListener("change", () => void onInspect(input.value));
  const browseButton = actionButton("Browse…", onBrowse, "quiet");
  browseButton.setAttribute("aria-label", "Browse for install location");
  const space = evaluation.requiredSpace !== undefined && evaluation.availableSpace !== undefined
    ? element("p", "path", `Requires ${evaluation.requiredSpace}; ${evaluation.availableSpace} available.`)
    : element("p", "path", "CEBG will check the available space before installing.");
  field.append(label, input, browseButton, space);
  const continueButton = actionButton("Continue", next);
  continueButton.disabled = !evaluation.safe;
  page.append(field, statusCard(evaluation.title, evaluation.detail, evaluation.safe ? "ok" : "danger"), screenActions(back, continueButton));
  return page;
}
