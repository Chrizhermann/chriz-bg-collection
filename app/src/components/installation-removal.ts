import type { InstallationRemovalPreview, InstallationRemovalResult } from "../contracts";
import { actionButton, element } from "./app-shell";

function unusedProfile(path: string): HTMLElement[] {
  return [element("p", "muted", "Saved-game profile deleted too, since the game never used it:"), element("p", "path", path)];
}

/** What the app reports once a removal has succeeded. */
export function installationRemovalNotice(preview: InstallationRemovalPreview, result: InstallationRemovalResult): string {
  if (preview.action === "unlist") return "Installation removed from My installs. Its folder was not changed.";
  const removed = preview.action === "forget" ? "Installation entry removed" : "Installation deleted";
  if (result.unusedSavePathKept) {
    return `${removed}. Its unused saved-game profile could not be deleted; it stays at ${result.unusedSavePathKept}.`;
  }
  if (preview.unusedSavePath) return `${removed}, along with its unused saved-game profile.`;
  return preview.action === "forget" ? `${removed}. No files were deleted.` : `${removed}. Saved games outside its folder were kept.`;
}

function approximateSize(value: string | null | undefined): string {
  if (value == null || !/^\d+$/.test(value)) return "unavailable";
  const amount = BigInt(value);
  const units = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
  let unit = 0;
  let divisor = 1n;
  while (amount / divisor >= 1024n && unit < units.length - 1) { unit++; divisor *= 1024n; }
  const tenths = (amount * 10n + divisor / 2n) / divisor;
  return `${tenths / 10n}${tenths % 10n === 0n ? "" : `.${tenths % 10n}`} ${units[unit]}`;
}

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
  const unlist = preview.action === "unlist";
  const title = element("h2", undefined, forget ? "Forget this installation?"
    : unlist ? "Remove this installation from My installs?" : "Delete this installation?");
  title.id = "removal-title";
  const description = element("p", undefined, forget
    ? preview.unusedSavePath
      ? "This folder is missing. Remove its entry from My installs; only its unused saved-game profile below is deleted."
      : "This folder is missing. Remove its entry from My installs; no files will be deleted."
    : unlist
      ? "CEBG can no longer confirm that this folder is this installation, so it will not delete anything in it. Only the entry leaves My installs; the folder stays where it is."
      : "Permanently delete this installation folder and everything inside it, including any files or saves you manually placed there. This cannot be undone.");
  description.id = "removal-description";
  dialog.append(title, element("strong", undefined, preview.displayName), description, element("p", "path removal-path", preview.managedRoot));
  if (unlist) {
    if (preview.reason) dialog.append(element("p", "muted", preview.reason));
    dialog.append(element("p", "muted", "If you no longer need this folder, delete it yourself after checking that it holds nothing you want to keep."));
  } else if (!forget) {
    const size = approximateSize(preview.approximateSizeBytes);
    dialog.append(element("p", "muted", `Approximate folder size: ${size}.${size === "unavailable" ? "" : " Disk space freed may differ."}`));
    dialog.append(element("p", "muted", "Your original game sources and shared downloads stay untouched. Saves stored outside this folder are kept."));
    if (preview.preservedSavePath) {
      dialog.append(element("p", "muted", "Saved-game profile kept:"), element("p", "path", preview.preservedSavePath));
    }
    if (preview.unusedSavePath) dialog.append(...unusedProfile(preview.unusedSavePath));
    dialog.append(element("p", "muted", "Export diagnostics first if you want to keep the failure logs."));
  } else if (preview.unusedSavePath) {
    dialog.append(...unusedProfile(preview.unusedSavePath));
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
    : error !== null ? "Review removal again" : forget ? "Forget installation" : unlist ? "Remove from list" : "Delete permanently", confirm, "danger");
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
