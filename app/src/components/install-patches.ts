import type { ManagedCopyUpdate, PatchPreview } from "../contracts";
import { actionButton, element } from "./app-shell";

export interface InstallPatchActions {
  readonly patches?: PatchPreview | null;
  readonly onCheckPatches?: (installId: string) => void | Promise<void>;
  readonly onApplyPatch?: (fullBackup: boolean, saveBackup: boolean) => void | Promise<void>;
  readonly onUndoPatch?: () => void | Promise<void>;
  readonly onRestorePatch?: () => void | Promise<void>;
  readonly patchBusy?: boolean;
  readonly patchError?: string;
  readonly patchInstallId?: string;
}

/** Add explicitly inspected patch states after the ordinary update notification is rendered. */
export function appendPatchAvailability(button: HTMLButtonElement, patches: readonly PatchPreview[]): void {
  const available = patches.filter((patch) => patch.state === "available").length;
  const recovery = patches.filter((patch) => patch.state === "needs-recovery").length;
  button.dataset.patchAvailable = String(available > 0);
  button.dataset.patchRecovery = String(recovery > 0);
  if (available === 0 && recovery === 0) return;
  const container = button.closest<HTMLElement>(".update-nav-item") ?? button.parentElement;
  const tooltip = container?.querySelector<HTMLElement>(".update-tooltip");
  if (!tooltip) return;
  const notes: string[] = [];
  if (available > 0) notes.push(`Current-game fixes available for ${available} checked installation${available === 1 ? "" : "s"}.`);
  if (recovery > 0) notes.push(`Patch recovery needed for ${recovery} checked installation${recovery === 1 ? "" : "s"}; restore its backup before playing.`);
  tooltip.textContent = `${tooltip.textContent ?? ""} ${notes.join(" ")}`.trim();
  button.dataset.updateAvailable = "true";
  let badge = button.querySelector<HTMLElement>(".update-badge");
  if (!badge) {
    badge = element("span", "update-badge");
    badge.setAttribute("aria-hidden", "true");
    button.append(badge);
  }
  badge.textContent = recovery > 0 ? "Attention" : "New";
}

const stateLabels: Readonly<Record<PatchPreview["state"], string>> = {
  available: "Can be applied to your current run",
  "already-fixed": "Already fixed",
  "not-selected": "This mod was not selected",
  unsupported: "Not supported for this installation",
  applied: "Fix applied to this installation",
  "needs-recovery": "Restore this installation before playing",
};

function bytes(value: string): bigint | null {
  return /^\d+$/.test(value) ? BigInt(value) : null;
}

function formatBytes(value: string): string {
  const amount = bytes(value);
  if (amount === null) return "Not available";
  const units = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
  let divisor = 1n;
  let unit = 0;
  while (amount / divisor >= 1024n && unit < units.length - 1) {
    divisor *= 1024n;
    unit++;
  }
  const tenths = (amount * 10n + divisor / 2n) / divisor;
  return `${tenths / 10n}${tenths % 10n === 0n ? "" : `.${tenths % 10n}`} ${units[unit]}`;
}

/** Explicit per-install check: rendering this section never scans or patches a game. */
export function installPatchesSection(copies: readonly ManagedCopyUpdate[], actions: InstallPatchActions, disabled = false): HTMLElement {
  const blocked = disabled || actions.patchBusy === true;
  const section = element("section", "install-patches");
  section.setAttribute("aria-labelledby", "install-patches-heading");
  section.setAttribute("aria-busy", String(actions.patchBusy === true));
  const title = element("h2", undefined, "Fixes for your current game");
  title.id = "install-patches-heading";
  section.append(title, element("p", "update-note", "Check one installation for supported, individual fixes. This does not upgrade all of its mods."));
  if (copies.length === 0) {
    section.append(element("p", "update-note", "Complete an installation first to check for fixes."));
    return section;
  }

  const requestedId = actions.patchInstallId ?? actions.patches?.installId;
  const initial = copies.find((copy) => copy.installId === requestedId)
    ?? copies.find((copy) => copy.state !== "stale") ?? copies[0];
  const toolbar = element("div", "patch-toolbar");
  const label = element("label", "patch-install-label", "Installation to check");
  const selector = element("select");
  selector.id = "patch-installation";
  label.htmlFor = selector.id;
  for (const copy of copies) {
    const option = element("option", undefined, `${copy.name} — ${copy.path} (${copy.installId})`);
    option.value = copy.installId;
    option.selected = copy.installId === initial.installId;
    selector.append(option);
  }
  label.append(selector);
  const selectedCopy = () => copies.find((copy) => copy.installId === selector.value);
  const check = actionButton("Check this installation", () => {
    const copy = selectedCopy();
    if (!blocked && copy && copy.state !== "stale") return actions.onCheckPatches?.(copy.installId);
  }, "quiet compact");
  toolbar.append(label, check);
  section.append(toolbar);
  const content = element("div", "patch-content");
  section.append(content);

  function renderSelection(): void {
    content.replaceChildren();
    selector.disabled = blocked;
    const copy = selectedCopy();
    check.disabled = blocked || !actions.onCheckPatches || !copy || copy.state === "stale";
    if (!copy) return;
    content.append(element("p", "path patch-path", copy.path));
    const patch = actions.patches?.installId === copy.installId ? actions.patches : null;
    if (copy.state === "stale") {
      content.append(element("p", "update-note", "This folder is unavailable. Restore its location before checking for fixes."));
    } else if (!patch) {
      content.append(element("p", "update-note", "Not checked yet. Your game files stay unchanged until you choose Apply."));
    } else {
      const result = element("article", "patch-result");
      result.dataset.patchState = patch.state;
      result.append(element("h3", undefined, patch.title), element("p", "patch-state", stateLabels[patch.state]), element("p", "update-note", patch.detail));
      result.append(element("p", "update-note", "This description-only fix repairs which existing kit descriptions character creation displays. It does not update kit abilities, change dialog.tlk or edit saved games."));
      result.append(element("p", "update-note", `Base collection: ${patch.baseRecipeVersion}`));
      result.append(element("p", "update-note patch-history", patch.appliedPatchIds.length > 0
        ? `Applied fixes: ${patch.appliedPatchIds.join(", ")}` : "Applied fixes: none recorded"));
      if (patch.state === "available") {
        result.append(backupChoices(patch, { ...actions, patchBusy: blocked }));
      } else if (patch.state === "applied" && patch.canUndo && actions.onUndoPatch) {
        const undo = actionButton("Undo this fix", () => !blocked ? actions.onUndoPatch?.() : undefined, "quiet compact");
        undo.disabled = blocked;
        result.append(undo);
      } else if (patch.state === "needs-recovery" && patch.canRestore && actions.onRestorePatch) {
        const restore = actionButton("Restore backup", () => !blocked ? actions.onRestorePatch?.() : undefined, "primary compact");
        restore.disabled = blocked;
        result.append(restore);
      }
      if (patch.state === "available" || patch.state === "applied" || patch.state === "needs-recovery") {
        result.append(element("p", "update-note", "Close the game before applying, undoing or restoring. Undo restores game files; it cannot reverse progress saved after a patch. Keep a pre-patch save to return to that earlier state."));
      }
      content.append(result);
    }
    if (actions.patchError && (!actions.patchInstallId || actions.patchInstallId === copy.installId)) {
      const error = element("p", "patch-error", actions.patchError);
      error.setAttribute("role", "alert");
      content.append(error);
    }
    if (actions.patchBusy) {
      const status = element("p", "patch-progress", "Checking, backing up, applying or restoring… Keep CEBG open and the game closed.");
      status.setAttribute("role", "status");
      content.append(status);
    }
  }

  selector.addEventListener("change", renderSelection);
  renderSelection();
  return section;
}

function backupChoices(patch: PatchPreview, actions: InstallPatchActions): HTMLElement {
  const choices = element("div", "patch-backups");
  choices.append(element("p", "update-note", "Affected-file backup is always included. Selected extra backups must finish successfully before the fix is applied."));
  const fullLabel = element("label", "patch-backup-choice");
  const full = element("input");
  full.type = "checkbox";
  full.checked = true;
  full.disabled = actions.patchBusy === true;
  fullLabel.append(full, element("span", undefined, `Back up the entire playable game folder first (recommended; ${formatBytes(patch.fullBackupBytes)})`));
  const saveLabel = element("label", "patch-backup-choice");
  const saveSnapshotUnavailable = bytes(patch.saveBackupBytes) === null;
  const saves = element("input");
  saves.type = "checkbox";
  saves.checked = false;
  saves.disabled = actions.patchBusy === true || saveSnapshotUnavailable;
  saveLabel.append(saves, element("span", undefined, `Copy saved games to the backup folder too (${formatBytes(patch.saveBackupBytes)})`));
  const space = element("p", "update-note patch-space");
  const updateSpace = () => {
    const fullBytes = full.checked ? bytes(patch.fullBackupBytes) : 0n;
    const saveBytes = saves.checked ? bytes(patch.saveBackupBytes) : 0n;
    const required = fullBytes !== null && saveBytes !== null ? formatBytes(String(fullBytes + saveBytes)) : "Not available";
    space.textContent = `Backup space required: ${required}, plus the affected-file backup. Available: ${formatBytes(patch.availableBytes)}.`;
  };
  full.addEventListener("change", updateSpace);
  saves.addEventListener("change", updateSpace);
  updateSpace();
  const apply = actionButton("Apply this fix", () => {
    if (!actions.patchBusy && patch.reviewToken && actions.onApplyPatch) return actions.onApplyPatch(full.checked, !saveSnapshotUnavailable && saves.checked);
  }, "primary compact");
  apply.disabled = actions.patchBusy === true || !patch.reviewToken || !actions.onApplyPatch;
  choices.append(fullLabel, saveLabel);
  if (saveSnapshotUnavailable) choices.append(element("p", "update-note", "Saved games cannot be copied automatically for this installation. Make a manual backup if you want one; the fix does not edit saves."));
  choices.append(space, element("p", "update-note", "Backup location:"), element("p", "path patch-path", patch.backupPath), apply);
  return choices;
}
