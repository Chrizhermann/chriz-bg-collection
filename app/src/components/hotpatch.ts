import type { HotpatchChange, HotpatchPreview, ManagedCopyUpdate } from "../contracts";
import { actionButton, element } from "./app-shell";
import { bytes, formatBytes } from "./install-patches";

export interface HotpatchActions {
  readonly hotpatch?: HotpatchPreview | null;
  readonly onCheckHotpatch?: (installId: string) => void | Promise<void>;
  readonly onApplyHotpatch?: (fullBackup: boolean, saveBackup: boolean) => void | Promise<void>;
  readonly onUndoHotpatch?: () => void | Promise<void>;
  readonly onRestoreHotpatch?: () => void | Promise<void>;
  readonly hotpatchBusy?: boolean;
  readonly hotpatchError?: string;
}

const stateLabels: Readonly<Record<HotpatchPreview["state"], string>> = {
  available: "Ready for your installation",
  refused: "Not offered for this installation",
  "up-to-date": "Up to date",
  "needs-recovery": "Restore the game files before playing",
};

/** What a game already in progress gets from one change. */
export const saveApplicabilityLabels: Readonly<Record<HotpatchChange["saveApplicability"], string>> = {
  "current-save": "Your current game gets this.",
  "before-npc-join": "Your current game gets this if that companion has not joined yet.",
  "before-area-visit": "Your current game gets this if you have not been to that area yet.",
  "before-event": "Your current game gets this if that event has not happened yet.",
  "next-playthrough": "For your next playthrough.",
  "new-game-only": "Only for new games.",
  unknown: "Whether your current game gets this is not known yet.",
};

function changelog(preview: HotpatchPreview): HTMLElement | null {
  const hotpatch = preview.hotpatch;
  if (hotpatch === null || hotpatch.changes.length === 0) return null;
  const details = element("details", "hotpatch-changelog");
  details.append(element("summary", undefined, "What changes"));
  for (const change of hotpatch.changes) {
    const item = element("div", "update-change");
    item.append(element("h4", undefined, change.title), element("p", undefined, change.summary),
      element("p", "hotpatch-save", saveApplicabilityLabels[change.saveApplicability]));
    if (change.conditionNote !== null) item.append(element("p", "update-note", change.conditionNote));
    details.append(item);
  }
  return details;
}

function list(className: string, heading: string, items: readonly string[]): HTMLElement {
  const block = element("div", className);
  const entries = element("ul");
  for (const item of items) entries.append(element("li", undefined, item));
  block.append(element("p", "update-note", heading), entries);
  return block;
}

/** Explicit per-install check: rendering this section never reads or changes a game. */
export function hotpatchSection(copy: ManagedCopyUpdate, actions: HotpatchActions, disabled = false): HTMLElement {
  const blocked = disabled || actions.hotpatchBusy === true;
  const section = element("section", "hotpatch-section");
  section.setAttribute("aria-labelledby", "hotpatch-heading");
  section.setAttribute("aria-busy", String(actions.hotpatchBusy === true));
  const title = element("h2", undefined, "Hotpatch");
  title.id = "hotpatch-heading";
  const headingRow = element("div", "update-section-heading");
  const check = actionButton("Check for a hotpatch", () => {
    if (!blocked && copy.state !== "stale") return actions.onCheckHotpatch?.(copy.installId);
  }, "quiet compact");
  check.disabled = blocked || !actions.onCheckHotpatch || copy.state === "stale";
  headingRow.append(title, check);
  section.append(headingRow, element("p", "update-note",
    "Patches this whole installation, the game you are playing now included. Every file it changes keeps an exact copy, so Undo puts the game back."));

  const preview = actions.hotpatch?.installId === copy.installId ? actions.hotpatch : null;
  if (copy.state === "stale") {
    section.append(element("p", "update-note", "This folder is unavailable. Restore its location before checking for a hotpatch."));
  } else if (!preview) {
    section.append(element("p", "update-note", "Not checked yet. Your game stays unchanged until you choose Apply."));
  } else {
    const result = element("article", "hotpatch-result");
    result.dataset.hotpatchState = preview.state;
    result.append(element("h3", undefined, preview.hotpatch?.title ?? "Hotpatch"),
      element("p", "patch-state", stateLabels[preview.state]), element("p", "update-note", preview.detail));
    const notes = changelog(preview);
    if (notes !== null) result.append(notes);
    if (preview.refusals.length > 0) result.append(list("hotpatch-refusals", "Why it is not offered:", preview.refusals));
    if (preview.notes.length > 0) result.append(list("hotpatch-notes", "Allowed differences from the recommended setup:", preview.notes));
    result.append(element("p", "update-note", `Collection: ${preview.baseRecipeVersion}`),
      element("p", "update-note hotpatch-history", preview.applied.length > 0
        ? `Applied hotpatches: ${preview.applied.map((applied) => applied.title).join(", ")}` : "Applied hotpatches: none"));
    if (preview.state === "available") result.append(backupChoices(preview, actions, blocked));
    if (preview.state === "needs-recovery" && preview.canRestore && actions.onRestoreHotpatch) {
      const restore = actionButton("Restore game files", () => !blocked ? actions.onRestoreHotpatch?.() : undefined, "primary compact");
      restore.disabled = blocked;
      result.append(restore);
    }
    if (preview.canUndo && actions.onUndoHotpatch) {
      const undo = actionButton(`Undo ${preview.undoTitle ?? "the last hotpatch"}`, () => !blocked ? actions.onUndoHotpatch?.() : undefined, "quiet compact");
      undo.disabled = blocked;
      result.append(undo);
    }
    if (preview.state === "available" || preview.state === "needs-recovery" || preview.canUndo) {
      result.append(element("p", "update-note", "Close the game first. Undo puts the game files back; progress you saved with the hotpatch stays in those saves."));
    }
    section.append(result);
  }
  if (actions.hotpatchError) {
    const error = element("p", "patch-error", actions.hotpatchError);
    error.setAttribute("role", "alert");
    section.append(error);
  }
  if (actions.hotpatchBusy) {
    const status = element("p", "patch-progress", "Checking, backing up, applying or restoring… Keep the app open and the game closed.");
    status.setAttribute("role", "status");
    section.append(status);
  }
  return section;
}

function backupChoices(preview: HotpatchPreview, actions: HotpatchActions, blocked: boolean): HTMLElement {
  const choices = element("div", "patch-backups");
  choices.append(element("p", "update-note", "Copies of the changed files are always kept. Extra backups must finish before the hotpatch is applied."));
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
  const apply = actionButton("Apply hotpatch", () => {
    if (!blocked && preview.reviewToken && actions.onApplyHotpatch) return actions.onApplyHotpatch(full.checked, !savesUnavailable && saves.checked);
  }, "primary compact");
  apply.disabled = blocked || !preview.reviewToken || !actions.onApplyHotpatch;
  choices.append(fullLabel, saveLabel);
  if (savesUnavailable) choices.append(element("p", "update-note", "Saved games cannot be copied automatically for this installation. The hotpatch does not edit saves."));
  choices.append(space, element("p", "update-note", "Backup location:"), element("p", "path patch-path", preview.backupPath), apply);
  return choices;
}
