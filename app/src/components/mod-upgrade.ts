import type { ManagedCopyUpdate, ModUpgradePreview } from "../contracts";
import { actionButton, element } from "./app-shell";

export interface ModUpgradeActions {
  readonly modUpgrade?: ModUpgradePreview | null;
  readonly modUpgradeBusy?: boolean;
  readonly modUpgradeError?: string;
  readonly modUpgradeErrorDetail?: string;
  readonly onUndoModUpgrade?: () => void | Promise<void>;
  readonly onRestoreModUpgrade?: () => void | Promise<void>;
}

const labels: Readonly<Record<ModUpgradePreview["state"], string>> = {
  available: "Historical BuffBot update",
  retired: "Historical BuffBot update",
  "already-current": "BuffBot is already current",
  "not-selected": "BuffBot is not installed in this copy",
  unsupported: "This copy cannot use the BuffBot update",
  applied: "BuffBot updated",
  "needs-recovery": "Restore this installation before playing",
};

function historyDescription(entry: ModUpgradePreview["history"][number]): string {
  switch (entry.state) {
    case "restored":
      return `Restored BuffBot ${entry.fromVersion}`;
    case "applied":
      return `Updated BuffBot to ${entry.toVersion} (from ${entry.fromVersion})`;
    case "prepared":
      return `BuffBot update to ${entry.toVersion} was not finished`;
    case "restoring":
      return `Restoration of BuffBot ${entry.fromVersion} was not finished`;
    default:
      return `BuffBot update needs attention (status: ${entry.state || "unknown"})`;
  }
}

function confirmation(
  title: string, description: string, copy: ManagedCopyUpdate, label: string,
  callback: () => void | Promise<void>, disabled: boolean,
): HTMLElement {
  const container = element("div", "mod-upgrade-confirmation");
  const panel = element("div", "patch-backups");
  panel.hidden = true;
  panel.setAttribute("role", "group");
  panel.setAttribute("aria-label", title);
  panel.append(element("h4", undefined, title), element("p", "update-note", description), element("p", "path patch-path", `${copy.name}: ${copy.path}`));
  const confirm = actionButton(label, () => {
    if (disabled || confirm.disabled || panel.hidden) return;
    confirm.disabled = true;
    return callback();
  }, "primary compact");
  confirm.disabled = disabled;
  const cancel = actionButton("Cancel", () => { panel.hidden = true; open.hidden = false; open.focus(); }, "quiet compact");
  cancel.disabled = disabled;
  panel.append(confirm, cancel);
  const open = actionButton(title, () => { if (!disabled) { panel.hidden = false; open.hidden = true; confirm.focus(); } }, "quiet compact");
  open.disabled = disabled;
  container.append(open, panel);
  return container;
}

/** A bounded, explicit action. Rendering never checks or changes the installation. */
export function modUpgradeResult(copy: ManagedCopyUpdate, actions: ModUpgradeActions, disabled = false): HTMLElement {
  const result = element("article", "patch-result mod-upgrade-result");
  const blocked = disabled || actions.modUpgradeBusy === true;
  const preview = actions.modUpgrade?.installId === copy.installId ? actions.modUpgrade : null;
  result.setAttribute("aria-busy", String(actions.modUpgradeBusy === true));
  if (preview) {
    result.dataset.upgradeState = preview.state;
    result.append(element("h3", undefined, "BuffBot"), element("p", preview.state === "needs-recovery" ? "patch-state patch-error" : "patch-state", labels[preview.state]));
    if (preview.installedVersion !== null) result.append(element("p", "update-note", `Recorded version: ${preview.installedVersion}`));
    if (preview.state === "applied") {
      result.append(element("p", "update-note", "Works with your current saves. BuffBot settings and other mods stay unchanged."));
    }
    // Recovery and unavailable-Undo reasons must never disappear into optional details.
    const visibleDetail = preview.state === "needs-recovery" || preview.state === "unsupported"
      || (preview.state === "applied" && !preview.canUndo);
    if (visibleDetail) result.append(element("p", preview.state === "needs-recovery" ? "patch-error" : "update-note", preview.detail));
    if (preview.state === "applied" && preview.canUndo && actions.onUndoModUpgrade) {
      result.append(confirmation("Undo BuffBot update", "Restore the game files from before this BuffBot update. Close the game first. Saved games and progress made since updating are not restored.", copy,
        "Confirm undo", () => actions.onUndoModUpgrade?.(), blocked));
    }
    if (preview.state === "needs-recovery" && preview.canRestore && actions.onRestoreModUpgrade) {
      result.append(confirmation("Restore BuffBot backup", "Recover the game files from the interrupted update. Keep the game closed until restoration succeeds. Saved games are not changed.", copy,
        "Restore backup now", () => actions.onRestoreModUpgrade?.(), blocked));
    }
    const details = element("details", "update-diagnostics");
    details.append(element("summary", undefined, "BuffBot update details"));
    if (!visibleDetail) details.append(element("p", "update-note", preview.detail));
    details.append(element("p", "update-note", preview.saveCompatibility),
      element("p", "update-note", `Base collection: ${preview.baseRecipeVersion}. Updating BuffBot does not update every mod or change this collection version.`));
    result.append(details);
    if (preview.history.length > 0) {
      const history = element("details", "update-diagnostics");
      history.append(element("summary", undefined, "BuffBot update history"));
      for (const entry of preview.history) history.append(element("p", "update-note", historyDescription(entry)));
      result.append(history);
    }
  }
  if (actions.modUpgradeError) {
    const error = element("p", "patch-error", actions.modUpgradeError);
    error.setAttribute("role", "alert");
    result.append(error);
    if (actions.modUpgradeErrorDetail?.trim()) {
      const details = element("details", "update-diagnostics");
      details.append(element("summary", undefined, "BuffBot update error details"), element("p", "path patch-path", actions.modUpgradeErrorDetail));
      result.append(details);
    }
  }
  if (actions.modUpgradeBusy) {
    const status = element("p", "patch-progress", "Working on the BuffBot update… Keep CEBG open and the game closed.");
    status.setAttribute("role", "status");
    result.append(status);
  }
  return result;
}
