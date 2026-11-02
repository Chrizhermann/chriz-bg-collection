import type { OperationProgress } from "../contracts";
import { progressLabel, progressPercent, type OperationFeedback } from "../operation-feedback";
import { actionButton, element } from "./app-shell";

export function progressDisplay(progress?: OperationProgress, label = progressLabel(progress)): HTMLElement {
  const region = element("div", "operation-progress");
  const caption = element("p", "operation-stage", label);
  caption.setAttribute("role", "status");
  const bar = element("progress", "operation-meter");
  bar.max = 100;
  const percent = progressPercent(progress);
  if (percent !== null) bar.value = percent;
  bar.setAttribute("aria-label", label);
  region.append(caption, bar);
  if (percent !== null) region.append(element("span", "operation-count", `${percent}% of this step`));
  else {
    const spinner = element("span", "operation-spinner");
    spinner.setAttribute("aria-hidden", "true");
    caption.prepend(spinner);
  }
  return region;
}

export function operationFeedback(feedback: OperationFeedback, dismiss: () => void): HTMLElement {
  const panel = element("section", `operation-feedback operation-${feedback.state}`);
  panel.setAttribute("aria-label", feedback.title);
  const heading = element("h2", undefined, feedback.title);
  heading.setAttribute("role", feedback.state === "error" ? "alert" : "status");
  const row = element("div", "operation-heading");
  row.append(heading);
  if (feedback.state !== "working") row.append(actionButton("Dismiss", dismiss, "quiet compact"));
  panel.append(row, element("p", undefined, feedback.detail));
  if (feedback.state === "working") panel.append(progressDisplay(feedback.progress));
  return panel;
}
