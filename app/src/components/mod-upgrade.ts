import type { ManagedCopyUpdate, ModUpgradePreview } from "../contracts";
import { actionButton, element } from "./app-shell";

export interface ModUpgradeActions {
  readonly modUpgrade?: ModUpgradePreview | null;
  readonly modUpgradeBusy?: boolean;
  readonly modUpgradeError?: string;
  readonly modUpgradeErrorDetail?: string;
  readonly onApplyModUpgrade?: (fullBackup: boolean, backupSaves: boolean) => void | Promise<void>;
  readonly onUndoModUpgrade?: () => void | Promise<void>;
  readonly onRestoreModUpgrade?: () => void | Promise<void>;
}

const labels: Readonly<Record<ModUpgradePreview["state"], string>> = {
  available: "BuffBot update available",
  "already-current": "BuffBot is already current",
  "not-selected": "BuffBot is not installed in this copy",
  unsupported: "This copy cannot use the BuffBot update",
  applied: "BuffBot updated",
  "needs-recovery": "Restore this installation before playing",
};

function bytes(value: string): bigint | null {
  return /^\d+$/.test(value) ? BigInt(value) : null;
}

function formatBytes(value: string): string {
  const amount = bytes(value);
  if (amount === null) return "not available";
  const units = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
  let unit = 0;
  let divisor = 1n;
  while (amount / divisor >= 1024n && unit < units.length - 1) { unit++; divisor *= 1024n; }
  const tenths = (amount * 10n + divisor / 2n) / divisor;
  return `${tenths / 10n}${tenths % 10n === 0n ? "" : `.${tenths % 10n}`} ${units[unit]}`;
}

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

function backupChoices(preview: ModUpgradePreview, copy: ManagedCopyUpdate, actions: ModUpgradeActions, blocked: boolean): HTMLElement {
  const choices = element("div", "patch-backups");
  choices.append(element("p", "update-note", "Affected files and uninstall records are always backed up. Any extra backups you select must finish before updating."));
  const fullLabel = element("label", "patch-backup-choice");
  const full = element("input");
  full.type = "checkbox";
  full.checked = true;
  full.disabled = blocked;
  fullLabel.append(full, element("span", undefined, `Back up the entire playable game folder (recommended; ${formatBytes(preview.fullBackupBytes)})`));
  const savesLabel = element("label", "patch-backup-choice");
  const saves = element("input");
  saves.type = "checkbox";
  saves.checked = false;
  const saveUnavailable = bytes(preview.saveBackupBytes) === null;
  saves.disabled = blocked || saveUnavailable;
  savesLabel.append(saves, element("span", undefined, `Also copy saved games (${formatBytes(preview.saveBackupBytes)})`));
  const space = element("p", "update-note");
  const review = element("div");
  const update = () => {
    const fullBytes = full.checked ? bytes(preview.fullBackupBytes) : 0n;
    const saveBytes = saves.checked ? bytes(preview.saveBackupBytes) : 0n;
    const required = fullBytes === null || saveBytes === null ? null : fullBytes + saveBytes;
    const available = bytes(preview.availableBytes);
    space.textContent = `Extra backup space: ${required === null ? "not available" : formatBytes(String(required))}, plus the affected-file backup. Free space: ${formatBytes(preview.availableBytes)}.`;
    const cannotApply = blocked || !preview.reviewToken || !actions.onApplyModUpgrade || required === null || available === null || required > available;
    const selectedFull = full.checked;
    const selectedSaves = !saveUnavailable && saves.checked;
    review.replaceChildren(confirmation("Review BuffBot update",
      `Update only BuffBot from ${preview.installedVersion ?? "unknown"} to ${preview.targetVersion} in this folder. Other mods and your saved games are not changed. The base collection remains ${preview.baseRecipeVersion}. Close the game first. ${selectedFull ? "Full game backup included." : "Affected-file backup only; no full game backup."} ${selectedSaves ? "Saved games will also be copied." : "Saved games will not be copied."}`,
      copy, "Update BuffBot now", () => actions.onApplyModUpgrade?.(selectedFull, selectedSaves), cannotApply));
  };
  full.addEventListener("change", update);
  saves.addEventListener("change", update);
  update();
  choices.append(fullLabel, savesLabel);
  if (saveUnavailable) choices.append(element("p", "update-note", "Saved games cannot be copied automatically. Back them up manually if wanted; this update never edits saves."));
  choices.append(space, element("p", "update-note", "Backup location:"), element("p", "path patch-path", preview.backupPath), review);
  return choices;
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
    result.append(element("p", "update-note", preview.installedVersion === null
      ? `Available version: ${preview.targetVersion}` : `Installed: ${preview.installedVersion} · Available: ${preview.targetVersion}`));
    if (preview.state === "available" || preview.state === "applied") {
      result.append(element("p", "update-note", "Works with your current saves. BuffBot settings and other mods stay unchanged."));
    }
    // Recovery and unavailable-Undo reasons must never disappear into optional details.
    const visibleDetail = preview.state === "needs-recovery" || preview.state === "unsupported"
      || (preview.state === "applied" && !preview.canUndo);
    if (visibleDetail) result.append(element("p", preview.state === "needs-recovery" ? "patch-error" : "update-note", preview.detail));
    if (preview.state === "available") result.append(backupChoices(preview, copy, actions, blocked));
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
