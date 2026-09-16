import type { InstallationRemovalPreview } from "../contracts";
import { actionButton, element } from "./app-shell";

export function installationRemovalDialog(
  preview: InstallationRemovalPreview,
  busy: boolean,
  error: string | null,
  confirm: () => void,
  cancel: () => void,
): HTMLElement {
  const overlay = element("div", "dialog-overlay");
  const dialog = element("section", "removal-dialog");
  dialog.setAttribute("role", "alertdialog");
  dialog.setAttribute("aria-modal", "true");
  dialog.setAttribute("aria-labelledby", "removal-title");
  dialog.setAttribute("aria-describedby", "removal-description");
  const forget = preview.action === "forget";
  const title = element("h2", undefined, forget ? "Forget this installation?" : "Delete this installation?");
  title.id = "removal-title";
  const description = element("p", undefined, forget
    ? "This folder is missing. Remove its entry from My installs; no files will be deleted."
    : "Permanently delete this installation folder and everything inside it, including any files or saves you manually placed there. This cannot be undone.");
  description.id = "removal-description";
  dialog.append(title, element("strong", undefined, preview.displayName), description, element("p", "path removal-path", preview.managedRoot));
  if (!forget) {
    dialog.append(element("p", "muted", "Your original game sources and shared downloads stay untouched. Saves stored outside this folder are kept."));
    if (preview.preservedSavePath) {
      dialog.append(element("p", "muted", "Saved-game profile kept:"), element("p", "path", preview.preservedSavePath));
    }
    dialog.append(element("p", "muted", "Export diagnostics first if you want to keep the failure logs."));
  }
  if (error !== null) {
    const feedback = element("p", "removal-error", error);
    feedback.setAttribute("role", "alert");
    dialog.append(feedback);
  }
  const actions = element("div", "inline-actions");
  const cancelButton = actionButton("Cancel", cancel, "quiet");
  cancelButton.dataset.action = "cancel-removal";
  cancelButton.disabled = busy;
  const confirmButton = actionButton(busy ? error === null ? "Removing…" : "Checking…"
    : error !== null ? "Review removal again" : forget ? "Forget installation" : "Delete permanently", confirm, "danger");
  confirmButton.disabled = busy;
  actions.append(cancelButton, confirmButton);
  dialog.append(actions);
  dialog.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && !busy) {
      event.preventDefault();
      cancel();
    }
    if (event.key === "Tab" && !busy) {
      if (event.shiftKey && document.activeElement === cancelButton) {
        event.preventDefault();
        confirmButton.focus();
      } else if (!event.shiftKey && document.activeElement === confirmButton) {
        event.preventDefault();
        cancelButton.focus();
      }
    }
  });
  overlay.append(dialog);
  return overlay;
}
