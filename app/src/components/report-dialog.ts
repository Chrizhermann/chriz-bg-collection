import { flavor } from "../flavor";
import { REPORT_CONTACT_LIMIT, REPORT_TEXT_LIMIT, type ReportKind } from "../report";
import { actionButton, element } from "./app-shell";

/** The diagnostics ZIP of the installation a report is about. */
export type ReportDiagnostics =
  | { readonly state: "none" }
  | { readonly state: "ready" }
  | { readonly state: "saving" }
  | { readonly state: "saved"; readonly path: string };

export interface ReportDialogModel {
  readonly kind: ReportKind;
  /** Whether the player chooses between a problem and feedback. */
  readonly kindSelectable: boolean;
  /** The details the app adds, as they will be sent; null while they are prepared. */
  readonly details: string | null;
  readonly diagnostics: ReportDiagnostics;
  /** Whether direct sending attaches the installation's diagnostics ZIP. */
  readonly includeDiagnostics: boolean;
  /** The result of the last action in the dialog. */
  readonly status: { readonly tone: "ok" | "warning"; readonly text: string } | null;
  /** The whole report, shown for copying by hand when the clipboard cannot be used. */
  readonly manualCopy: string | null;
  /** Direct submission is running. */
  readonly sending: boolean;
  /** An action of the dialog is running. */
  readonly busy: boolean;
}

export interface ReportDialogActions {
  readonly setKind: (kind: ReportKind) => void;
  readonly setIncludeDiagnostics: (included: boolean) => void;
  /** Sends the report directly from the app; `text` and `contact` are what the player wrote. */
  readonly send: (text: string, contact: string) => void;
  /** Opens the product's GitHub issue page with the report; `text` is what the player wrote. */
  readonly openIssue: (text: string) => void;
  readonly copy: (text: string) => void;
  readonly openChat: () => void;
  readonly saveDiagnostics: () => void;
  readonly showDiagnostics: () => void;
  readonly close: () => void;
}

/**
 * The dialog keeps its own nodes while the app renders around it, so the text the player is
 * writing and the focus survive progress updates of a running installation.
 */
export interface ReportDialog {
  readonly overlay: HTMLElement;
  update(model: ReportDialogModel): void;
  focus(): void;
}

/** Puts `text` on the clipboard; false when the browser refuses both ways. */
export async function copyText(text: string): Promise<boolean> {
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      return true;
    }
  } catch {
    // Fall back to the selection-based copy below.
  }
  const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  const area = document.createElement("textarea");
  area.value = text;
  area.readOnly = true;
  area.className = "report-copy-buffer";
  document.body.append(area);
  area.select();
  let copied = false;
  try {
    copied = document.execCommand("copy");
  } catch {
    copied = false;
  }
  area.remove();
  previous?.focus({ preventScroll: true });
  return copied;
}

export function createReportDialog(initial: ReportDialogModel, actions: ReportDialogActions): ReportDialog {
  const overlay = element("div", "dialog-overlay report-overlay");
  const dialog = element("section", "decision-dialog report-dialog");
  dialog.setAttribute("role", "dialog");
  dialog.setAttribute("aria-modal", "true");
  dialog.setAttribute("aria-labelledby", "report-title");
  dialog.setAttribute("aria-describedby", "report-intro");
  const title = element("h2", undefined, initial.kindSelectable ? "Report a problem or send feedback" : "Report this problem");
  title.id = "report-title";
  const intro = element("p", undefined,
    `Send a report straight from ${flavor.shortName} (no account needed), or use GitHub or Discord below. Nothing is sent until you click Send report.`);
  intro.id = "report-intro";
  dialog.append(title, intro);

  const kinds = element("fieldset", "report-kind");
  kinds.append(element("legend", undefined, "What is it?"));
  const kindInputs = new Map<ReportKind, HTMLInputElement>();
  for (const [kind, label] of [["problem", "A problem"], ["feedback", "Feedback or an idea"]] as const) {
    const input = element("input");
    input.type = "radio";
    input.name = "report-kind";
    input.id = `report-kind-${kind}`;
    input.value = kind;
    input.addEventListener("change", () => { if (input.checked) actions.setKind(kind); });
    const choice = element("label");
    choice.append(input, element("span", undefined, label));
    kinds.append(choice);
    kindInputs.set(kind, input);
  }
  if (initial.kindSelectable) dialog.append(kinds);

  const textLabel = element("label", "report-label");
  textLabel.htmlFor = "report-text";
  const text = element("textarea", "report-text");
  text.id = "report-text";
  text.rows = 5;
  text.maxLength = REPORT_TEXT_LIMIT;
  text.setAttribute("aria-describedby", "report-text-help");
  const textHelp = element("p", "muted",
    `Up to ${REPORT_TEXT_LIMIT} characters. Optional when ${flavor.shortName} already captured an error above.`);
  textHelp.id = "report-text-help";
  dialog.append(textLabel, text, textHelp);

  const detailsLabel = element("h3", undefined, `What ${flavor.shortName} adds`);
  detailsLabel.id = "report-details-title";
  const details = element("pre", "report-details");
  details.id = "report-details";
  details.tabIndex = 0;
  details.setAttribute("aria-labelledby", "report-details-title");
  const detailsHelp = element("p", "muted",
    "Your home folder, your Windows user name, the computer name and passwords are removed from these details.");
  dialog.append(detailsLabel, details, detailsHelp);

  dialog.append(element("h3", undefined, "Send it"));
  const directBox = element("div", "report-channel report-direct");
  const contactLabel = element("label", "report-contact-label", "How to reach you (optional)");
  contactLabel.htmlFor = "report-contact";
  const contact = element("input", "report-contact");
  contact.type = "text";
  contact.id = "report-contact";
  contact.maxLength = REPORT_CONTACT_LIMIT;
  contact.placeholder = "Discord name or email, if Chriz may reply";
  contact.autocomplete = "off";

  const includeDiagnosticsRow = element("label", "report-checkbox");
  const includeDiagnostics = element("input");
  includeDiagnostics.type = "checkbox";
  includeDiagnostics.id = "report-include-diagnostics";
  includeDiagnostics.addEventListener("change", () => {
    actions.setIncludeDiagnostics(includeDiagnostics.checked);
  });
  includeDiagnosticsRow.append(
    includeDiagnostics,
    element("span", undefined, "Include the diagnostics ZIP for this installation"),
  );

  const send = actionButton("Send report", () => actions.send(text.value, contact.value));
  send.id = "report-send";
  const directHelp = element("p", "muted");
  directHelp.id = "report-direct-help";
  directBox.append(contactLabel, contact, includeDiagnosticsRow, send, directHelp);
  dialog.append(directBox);

  dialog.append(element("h3", undefined, "Or post it yourself"));
  const github = actionButton("Open GitHub issue", () => actions.openIssue(text.value), "quiet");
  github.id = "report-open-issue";
  const githubRow = element("div", "report-channel");
  githubRow.append(github, element("p", "muted",
    `Opens ${flavor.report.issuesLabel} in your browser. You need a GitHub account, and reports there are public.`));
  const copy = actionButton("Copy text", () => actions.copy(text.value), "quiet");
  copy.id = "report-copy";
  const chat = actionButton("Open Discord", actions.openChat, "quiet");
  chat.id = "report-open-chat";
  const chatButtons = element("div", "inline-actions");
  chatButtons.append(copy, chat);
  const chatRow = element("div", "report-channel");
  chatRow.append(chatButtons, element("p", "muted",
    `Copy the text, then paste it in ${flavor.report.chatLabel}${flavor.report.chatChannel ? `, channel ${flavor.report.chatChannel}` : ""}. You need a Discord account.`));
  dialog.append(githubRow, chatRow);

  const manualCopy = element("textarea", "report-text report-manual-copy");
  manualCopy.id = "report-manual-copy";
  manualCopy.readOnly = true;
  manualCopy.rows = 6;
  manualCopy.setAttribute("aria-label", "The whole report, to copy by hand");
  manualCopy.hidden = true;
  dialog.append(manualCopy);

  const diagnostics = element("div", "report-channel report-diagnostics");
  const diagnosticsTitle = element("h3", undefined, "Diagnostics ZIP");
  const save = actionButton("Save diagnostics ZIP", actions.saveDiagnostics, "quiet");
  save.id = "report-save-diagnostics";
  const show = actionButton("Show in folder", actions.showDiagnostics, "quiet");
  show.id = "report-show-diagnostics";
  const diagnosticsButtons = element("div", "inline-actions");
  diagnosticsButtons.append(save, show);
  const diagnosticsPath = element("p", "path");
  diagnosticsPath.id = "report-diagnostics-path";
  diagnostics.append(diagnosticsTitle, diagnosticsButtons, diagnosticsPath, element("p", "muted",
    "Save a local copy of this installation's logs to inspect yourself, or to attach on GitHub or Discord."));
  dialog.append(diagnostics);

  const status = element("p", "report-status");
  status.id = "report-status";
  status.setAttribute("role", "status");
  const close = actionButton("Close", actions.close, "quiet");
  close.id = "report-close";
  const footer = element("div", "inline-actions report-footer");
  footer.append(close);
  dialog.append(status, footer);

  dialog.addEventListener("keydown", (event) => {
    if (event.key === "Escape") {
      event.preventDefault();
      actions.close();
      return;
    }
    if (event.key !== "Tab") return;
    const focusable = Array.from(dialog.querySelectorAll<HTMLElement>("input:not(:disabled), textarea, pre[tabindex], button:not(:disabled)"))
      .filter(candidate => !candidate.hidden && candidate.closest("[hidden]") === null);
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (first === undefined || last === undefined) return;
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  });
  overlay.append(dialog);

  let shownManualCopy: string | null = null;
  const update = (model: ReportDialogModel): void => {
    dialog.setAttribute("aria-busy", String(model.busy));
    for (const [kind, input] of kindInputs) {
      input.checked = kind === model.kind;
      input.disabled = model.busy;
    }
    textLabel.textContent = model.kind === "problem" ? "What happened, and what did you expect?" : "Your feedback or idea";
    details.textContent = model.details ?? "Preparing the details…";
    const ready = model.details !== null && !model.busy;
    const hasDiagnostics = model.diagnostics.state !== "none";

    contact.disabled = model.busy;
    includeDiagnosticsRow.hidden = !hasDiagnostics;
    includeDiagnostics.checked = hasDiagnostics && model.includeDiagnostics;
    includeDiagnostics.disabled = model.busy;
    send.disabled = !ready;
    send.textContent = model.sending ? "Sending…" : "Send report";
    directHelp.textContent = hasDiagnostics && model.includeDiagnostics
      ? "Sends this text and the redacted diagnostics ZIP through bg.chrizfader.org to Chriz's private inbox (your IP is hashed briefly against spam)."
      : "Sends this text through bg.chrizfader.org to Chriz's private inbox (your IP is hashed briefly against spam).";

    github.disabled = !ready;
    copy.disabled = !ready;
    chat.disabled = model.busy;

    diagnostics.hidden = !hasDiagnostics;
    save.disabled = model.busy || model.diagnostics.state === "saving";
    save.textContent = model.diagnostics.state === "saving" ? "Saving…"
      : model.diagnostics.state === "saved" ? "Save another copy" : "Save diagnostics ZIP";
    show.hidden = model.diagnostics.state !== "saved";
    show.disabled = model.busy;
    diagnosticsPath.hidden = model.diagnostics.state !== "saved";
    diagnosticsPath.textContent = model.diagnostics.state === "saved" ? `Saved to ${model.diagnostics.path}` : "";

    manualCopy.hidden = model.manualCopy === null;
    if (model.manualCopy !== null && model.manualCopy !== shownManualCopy) {
      manualCopy.value = model.manualCopy;
      manualCopy.focus({ preventScroll: true });
      manualCopy.select();
    }
    shownManualCopy = model.manualCopy;

    status.textContent = model.status?.text ?? "";
    status.className = `report-status${model.status ? ` ${model.status.tone}` : ""}`;
    close.disabled = model.busy;
  };
  update(initial);

  return {
    overlay,
    update,
    focus: () => (initial.kindSelectable ? kindInputs.get(initial.kind) ?? text : text).focus({ preventScroll: true }),
  };
}
