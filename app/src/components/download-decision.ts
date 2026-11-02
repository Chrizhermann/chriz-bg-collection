import type { DecisionAnswer, PendingDecision } from "../contracts";
import { actionButton, element } from "./app-shell";
import { persistDisclosure } from "./disclosure";

/**
 * The modal question about a download that an installation waits for. The engine renders
 * the question, its details, the offered answers and their order; this component only shows
 * them, puts the initial focus on the engine's chosen answer and keeps the focus inside the
 * dialog until the player answers. It names no product, so both app flavors share it.
 */
export interface DownloadDecisionModel {
  readonly decision: PendingDecision;
  readonly busy: boolean;
  readonly error?: string;
  readonly detailsOpen?: boolean;
  readonly onDetailsOpen?: (open: boolean) => void;
  readonly answer: (answer: DecisionAnswer) => void;
}

export function decisionAnswerId(answer: DecisionAnswer): string {
  return `decision-answer-${answer.replaceAll("_", "-")}`;
}

export function downloadDecisionDialog(model: DownloadDecisionModel): HTMLElement {
  const { decision } = model;
  const overlay = element("div", "dialog-overlay");
  const dialog = element("section", "decision-dialog");
  dialog.setAttribute("role", "dialog");
  dialog.setAttribute("aria-modal", "true");
  dialog.setAttribute("aria-labelledby", "decision-title");
  dialog.setAttribute("aria-describedby", "decision-message");
  dialog.setAttribute("aria-busy", String(model.busy));
  const title = element("h2", undefined, "A download needs your answer");
  title.id = "decision-title";
  const message = element("p", undefined, decision.message);
  message.id = "decision-message";
  dialog.append(title, message);
  if (decision.details.length > 0) {
    const details = element("details", "decision-details");
    details.open = model.detailsOpen ?? false;
    const summary = element("summary", undefined, "Details");
    summary.id = "decision-details-toggle";
    persistDisclosure(details, summary, open => model.onDetailsOpen?.(open));
    details.append(summary);
    const list = element("ul");
    for (const detail of decision.details) list.append(element("li", "path", detail));
    details.append(list);
    dialog.append(details);
  }
  if (model.busy) {
    const saving = element("p", "muted", "Saving your answer…");
    saving.setAttribute("role", "status");
    dialog.append(saving);
  }
  if (model.error) {
    const error = element("p", "removal-error", model.error);
    error.id = "download-decision-error";
    error.setAttribute("role", "alert");
    dialog.append(error);
  }
  const actions = element("div", "inline-actions");
  for (const offered of decision.answers) {
    const button = actionButton(offered.label, () => { if (!model.busy) model.answer(offered.answer); },
      offered.answer === decision.focus ? "primary" : "quiet");
    button.id = decisionAnswerId(offered.answer);
    button.dataset.answer = offered.answer;
    button.disabled = model.busy;
    actions.append(button);
  }
  dialog.append(actions);
  dialog.addEventListener("keydown", (event) => {
    // The question can only be answered; Escape does not dismiss it.
    if (event.key === "Escape") {
      event.preventDefault();
      return;
    }
    if (event.key !== "Tab") return;
    const focusable = Array.from(dialog.querySelectorAll<HTMLElement>("summary, button:not(:disabled)"));
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (first === undefined || last === undefined) {
      event.preventDefault();
      return;
    }
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  });
  overlay.append(dialog);
  return overlay;
}

/**
 * Puts the focus on the engine's chosen answer; while the answer is saved, on the dialog
 * itself; after a failure, on the error. A re-render of an unchanged dialog keeps the focus
 * on `previousFocusId`, the control the player had moved to.
 */
export function focusDownloadDecision(
  root: HTMLElement,
  model: Pick<DownloadDecisionModel, "decision" | "busy" | "error">,
  previousFocusId?: string,
): void {
  const dialog = root.querySelector<HTMLElement>(".decision-dialog");
  if (dialog === null) return;
  const previous = previousFocusId
    ? Array.from(dialog.querySelectorAll<HTMLElement>("[id]")).find(candidate => candidate.id === previousFocusId)
    : undefined;
  let target: HTMLElement | null;
  if (previous !== undefined && !(previous instanceof HTMLButtonElement && previous.disabled)) {
    target = previous;
  } else if (model.busy) {
    dialog.tabIndex = -1;
    target = dialog;
  } else if (model.error) {
    target = dialog.querySelector<HTMLElement>("#download-decision-error");
    if (target !== null) target.tabIndex = -1;
  } else {
    target = dialog.querySelector<HTMLElement>(`#${decisionAnswerId(model.decision.focus)}`)
      ?? dialog.querySelector<HTMLElement>("button");
  }
  target?.focus({ preventScroll: true });
}
