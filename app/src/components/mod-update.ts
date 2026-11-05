import type { ManagedCopyUpdate, ModUpdatePreview } from "../contracts";
import { actionButton, element } from "./app-shell";
import { bytes, formatBytes } from "./install-patches";

/** A whole installed mod the app can bring to its newest release in a finished installation. */
export interface ModUpdateOffer {
  /** The mod's id in the recipe and in the app's update catalog. */
  readonly modId: string;
  readonly title: string;
}

/** Whole-mod updates offered under Updates, one card each. */
export const MOD_UPDATES: readonly ModUpdateOffer[] = [{ modId: "buffbot", title: "BuffBot" }];

/**
 * Whole-mod updates such as BuffBot. They share one history: Undo always takes back the latest
 * change, so a card offers Undo only when that change is its own mod's update.
 */
export interface ModUpdateActions {
  /** Checked results by mod id; a result for another installation is never shown. */
  readonly modUpdates?: Readonly<Record<string, ModUpdatePreview | undefined>>;
  readonly modUpdateErrors?: Readonly<Record<string, string | undefined>>;
  readonly onCheckModUpdate?: (installId: string, modId: string) => void | Promise<void>;
  readonly onApplyModUpdate?: (modId: string, fullBackup: boolean, saveBackup: boolean) => void | Promise<void>;
  readonly onUndoModUpdate?: (modId: string) => void | Promise<void>;
  readonly onRestoreModUpdate?: (modId: string) => void | Promise<void>;
  /** A mod update operation runs; every card waits, since they share one history. */
  readonly modUpdateBusy?: boolean;
  /** The mod whose operation runs. */
  readonly modUpdateBusyId?: string;
}

const stateLabels: Readonly<Record<ModUpdatePreview["state"], string>> = {
  available: "Update ready",
  "up-to-date": "Up to date",
  "not-installed": "Not installed in this game",
  refused: "Not offered for this game",
  unavailable: "Could not check right now",
  "needs-recovery": "Restore the game files before playing",
};

function list(className: string, heading: string, items: readonly string[]): HTMLElement {
  const block = element("div", className);
  const entries = element("ul");
  for (const item of items) entries.append(element("li", undefined, item));
  block.append(element("p", "update-note", heading), entries);
  return block;
}

/** Explicit per-install check of one mod: rendering this section never reads or changes a game. */
export function modUpdateSection(copy: ManagedCopyUpdate, offer: ModUpdateOffer, actions: ModUpdateActions, disabled = false): HTMLElement {
  const busyHere = actions.modUpdateBusy === true && actions.modUpdateBusyId === offer.modId;
  const blocked = disabled || actions.modUpdateBusy === true;
  const section = element("section", "install-patches mod-update-section");
  section.dataset.modId = offer.modId;
  const headingId = `mod-update-heading-${offer.modId}`;
  section.setAttribute("aria-labelledby", headingId);
  section.setAttribute("aria-busy", String(busyHere));
  const title = element("h2", undefined, offer.title);
  title.id = headingId;
  const headingRow = element("div", "update-section-heading");
  const check = actionButton(`Check for a ${offer.title} update`, () => {
    if (!blocked && copy.state !== "stale") return actions.onCheckModUpdate?.(copy.installId, offer.modId);
  }, "quiet compact");
  check.disabled = blocked || !actions.onCheckModUpdate || copy.state === "stale";
  headingRow.append(title, check);
  section.append(headingRow, element("p", "update-note",
    `Brings ${offer.title} in this game to its newest release. Every file it changes keeps an exact copy, so Undo puts the game back.`));

  const checked = actions.modUpdates?.[offer.modId];
  const preview = checked?.installId === copy.installId && checked.modId === offer.modId ? checked : null;
  if (copy.state === "stale") {
    section.append(element("p", "update-note", `This folder is unavailable. Restore its location before checking for a ${offer.title} update.`));
  } else if (!preview) {
    section.append(element("p", "update-note", "Not checked yet. Your game stays unchanged until you choose Update."));
  } else {
    const result = element("article", "mod-update-result");
    result.dataset.modUpdateState = preview.state;
    result.append(element("p", "patch-state", stateLabels[preview.state]));
    if (preview.installedVersion !== null) result.append(element("p", "update-note mod-update-installed", `Installed: ${preview.installedVersion}`));
    if (preview.targetVersion !== null) result.append(element("p", "update-note mod-update-target", `Newest release: ${preview.targetVersion}`));
    if (preview.releaseTitle !== null) result.append(element("p", "update-note mod-update-release", preview.releaseTitle));
    if (preview.detail) result.append(element("p", "update-note", preview.detail));
    if (preview.refusals.length > 0) result.append(list("mod-update-refusals", "Why it is not offered:", preview.refusals));
    if (preview.state === "available") result.append(backupChoices(offer, preview, actions, blocked));
    if (preview.state === "needs-recovery" && preview.canRestore && actions.onRestoreModUpdate) {
      const restore = actionButton("Restore game files", () => !blocked ? actions.onRestoreModUpdate?.(offer.modId) : undefined, "primary compact");
      restore.disabled = blocked;
      result.append(restore);
    }
    if (preview.canUndo && actions.onUndoModUpdate) {
      const undo = actionButton(`Undo ${preview.undoTitle ?? `the ${offer.title} update`}`, () => !blocked ? actions.onUndoModUpdate?.(offer.modId) : undefined, "quiet compact");
      undo.disabled = blocked;
      result.append(undo);
    }
    if (preview.state === "available" || preview.state === "needs-recovery" || preview.canUndo) {
      result.append(element("p", "update-note", "Close the game first. Undo puts the game files back; progress you saved after the update stays in those saves."));
    }
    section.append(result);
  }
  const error = actions.modUpdateErrors?.[offer.modId];
  if (error) {
    const alert = element("p", "patch-error", error);
    alert.setAttribute("role", "alert");
    section.append(alert);
  }
  if (busyHere) {
    const status = element("p", "patch-progress", "Checking, backing up, updating or restoring… Keep the app open and the game closed.");
    status.setAttribute("role", "status");
    section.append(status);
  }
  return section;
}

function backupChoices(offer: ModUpdateOffer, preview: ModUpdatePreview, actions: ModUpdateActions, blocked: boolean): HTMLElement {
  const choices = element("div", "patch-backups");
  choices.append(element("p", "update-note", "Copies of the changed files are always kept. Extra backups must finish before the update starts."));
  const fullLabel = element("label", "patch-backup-choice");
  const full = element("input");
  full.type = "checkbox";
  full.checked = true;
  full.disabled = blocked;
  fullLabel.append(full, element("span", undefined, `Back up the whole game folder first (recommended; ${formatBytes(preview.fullBackupBytes)})`));
  const saveLabel = element("label", "patch-backup-choice");
  const savesUnavailable = bytes(preview.saveBackupBytes) === null;
  const saves = element("input");
  saves.type = "checkbox";
  saves.checked = false;
  saves.disabled = blocked || savesUnavailable;
  saveLabel.append(saves, element("span", undefined, `Copy your saved games too (${formatBytes(preview.saveBackupBytes)})`));
  const space = element("p", "update-note patch-space");
  const updateSpace = () => {
    const fullBytes = full.checked ? bytes(preview.fullBackupBytes) : 0n;
    const saveBytes = saves.checked ? bytes(preview.saveBackupBytes) : 0n;
    const required = fullBytes !== null && saveBytes !== null ? formatBytes(String(fullBytes + saveBytes)) : "Not available";
    space.textContent = `Backup space required: ${required}. Available: ${formatBytes(preview.availableBytes)}.`;
  };
  full.addEventListener("change", updateSpace);
  saves.addEventListener("change", updateSpace);
  updateSpace();
  const apply = actionButton(`Update ${offer.title}`, () => {
    if (!blocked && preview.reviewToken && actions.onApplyModUpdate) return actions.onApplyModUpdate(offer.modId, full.checked, !savesUnavailable && saves.checked);
  }, "primary compact");
  apply.disabled = blocked || !preview.reviewToken || !actions.onApplyModUpdate;
  choices.append(fullLabel, saveLabel);
  if (savesUnavailable) choices.append(element("p", "update-note", `Saved games cannot be copied automatically for this installation. The update does not edit saves.`));
  choices.append(space, element("p", "update-note", "Backup location:"), element("p", "path patch-path", preview.backupPath), apply);
  return choices;
}
