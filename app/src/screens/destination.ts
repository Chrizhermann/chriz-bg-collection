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
  page.append(screenIntro("Step 2 of 5", "Choose a destination", "This must be a new campaign copy outside the store-managed source folders."));
  const field = element("div", "card field-card");
  const label = element("label", undefined, "Campaign destination");
  label.htmlFor = "destination-path";
  const input = element("input");
  input.id = "destination-path";
  input.type = "text";
  input.value = evaluation.path;
  input.addEventListener("change", () => void onInspect(input.value));
  const browseButton = actionButton("Browse…", onBrowse, "quiet");
  browseButton.setAttribute("aria-label", "Browse for campaign destination");
  const space = evaluation.requiredSpace !== undefined && evaluation.availableSpace !== undefined
    ? element("p", "path", `Requires ${evaluation.requiredSpace}; ${evaluation.availableSpace} available in this fixture.`)
    : element("p", "path", "Exact disk-space and write checks are repeated immediately before the build starts.");
  field.append(label, input, browseButton, space);
  const continueButton = actionButton("Continue", next);
  continueButton.disabled = !evaluation.safe;
  page.append(field, statusCard(evaluation.title, evaluation.detail, evaluation.safe ? "ok" : "danger"), screenActions(back, continueButton));
  return page;
}
