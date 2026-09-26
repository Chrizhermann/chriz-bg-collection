import type { InstalledUpdatePlan, ManagedCopyUpdate } from "../contracts";
import { actionButton, element } from "./app-shell";
import { modUpgradeResult, type ModUpgradeActions } from "./mod-upgrade";

export interface InstalledUpdateActions extends ModUpgradeActions {
  readonly installedUpdatePlan?: InstalledUpdatePlan | null;
  readonly installedUpdateInstallId?: string;
  readonly installedUpdateBusy?: boolean;
  readonly installedUpdateError?: string;
  readonly installedUpdateErrorDetail?: string;
  readonly onCheckInstalledUpdate?: (installId: string) => void | Promise<void>;
  readonly onSelectInstalledUpdate?: (installId: string) => void;
}

const runLabels: Readonly<Record<InstalledUpdatePlan["runs"][number]["state"], string>> = {
  unchanged: "Unchanged",
  unassessed: "Update support not assessed",
  "requires-migration": "Setup changes need review",
};

function runRow(run: InstalledUpdatePlan["runs"][number]): HTMLElement {
  const row = element("article", "update-change");
  row.append(element("h3", undefined, run.modName));
  row.append(element("p", "update-note", run.targetVersion === null
    ? `Installed: ${run.installedVersion} · Target version unavailable`
    : `${run.installedVersion} → ${run.targetVersion}`));
  row.append(element("p", "update-note", runLabels[run.state]));
  const details = element("details", "update-diagnostics");
  details.append(element("summary", undefined, `${run.modName} details`), element("p", undefined, run.detail));
  details.append(element("p", "path patch-path", `Install step: ${run.runId} · Components: ${run.components.join(", ")}`));
  row.append(details);
  return row;
}

function comparisonResult(plan: InstalledUpdatePlan): HTMLElement {
  const result = element("div", "installed-update-result");
  const status = element("p", "update-note", plan.state === "blocked" ? "Comparison needs attention" : "Comparison complete");
  status.setAttribute("role", "status");
  result.append(status, element("p", "update-note", `Base collection: ${plan.baseRecipeVersion} · Compared with: ${plan.targetRecipeVersion}`));
  result.append(element("p", "update-note", "Comparison only — no game files changed"));
  result.append(element("p", "update-note", plan.choicesPreserved
    ? "Your existing choices are preserved in this comparison. New optional content stays off."
    : "Your choices need review before this setup can be carried forward."));

  if (plan.issues.length > 0) {
    const issues = element("ul", "update-note");
    for (const issue of plan.issues.slice(0, 3)) issues.append(element("li", undefined, issue.message));
    result.append(issues);
    if (plan.issues.length > 3) {
      const more = element("details", "update-diagnostics");
      more.append(element("summary", undefined, `More choices to review (${plan.issues.length - 3})`));
      const remaining = element("ul");
      for (const issue of plan.issues.slice(3)) remaining.append(element("li", undefined, issue.message));
      more.append(remaining);
      result.append(more);
    }
  }

  const changed = plan.runs.filter(run => run.state !== "unchanged");
  const unchanged = plan.runs.filter(run => run.state === "unchanged");
  if (changed.length > 0) {
    result.append(element("p", "update-note", "Differences alone do not mean an update is supported. Only an explicit supported-update action can change this installation."));
    const list = element("div", "installed-update-changes");
    for (const run of changed) list.append(runRow(run));
    result.append(list);
  } else if (plan.state === "comparison-ready" && plan.runs.length > 0) {
    result.append(element("p", "update-note", "No installed-mod differences were found in this comparison."));
  }

  if (unchanged.length > 0) {
    const details = element("details", "updates-installs installed-update-unchanged");
    details.append(element("summary", undefined, `Unchanged install steps (${unchanged.length})`));
    for (const run of unchanged) details.append(runRow(run));
    result.append(details);
  }

  const details = element("details", "update-diagnostics");
  details.append(element("summary", undefined, "Comparison details"), element("p", undefined, plan.detail));
  for (const issue of plan.issues) {
    details.append(element("p", "path patch-path", `${issue.rule}${issue.featureId === null ? "" : ` · ${issue.featureId}`}`));
  }
  result.append(details);
  return result;
}

/** Rendering and selecting an installation never invokes a comparison or changes game files. */
export function installedUpdateSection(copies: readonly ManagedCopyUpdate[], actions: InstalledUpdateActions, disabled = false, sharedSelection = false): HTMLElement {
  const section = element("section", "install-patches installed-update-plan");
  section.setAttribute("aria-labelledby", "installed-update-heading");
  section.setAttribute("aria-busy", String(actions.installedUpdateBusy === true || actions.modUpgradeBusy === true));
  const heading = element("h2", undefined, "Installed mods");
  heading.id = "installed-update-heading";
  const headingRow = element("div", "update-section-heading");
  headingRow.append(heading);
  section.append(headingRow, element("p", "update-note", "Check for supported updates to the selected game."));
  if (copies.length === 0) {
    section.append(element("p", "update-note", "Complete an installation first to compare its installed mods."));
    return section;
  }

  const requestedId = actions.installedUpdateInstallId ?? actions.installedUpdatePlan?.installId;
  const initial = copies.find(copy => copy.installId === requestedId)
    ?? copies.find(copy => copy.state !== "stale") ?? copies[0];
  const toolbar = element("div", "patch-toolbar");
  const label = element("label", "patch-install-label", "Installation to compare");
  const selector = element("select");
  selector.id = "installed-update-installation";
  selector.disabled = disabled || actions.modUpgradeBusy === true;
  label.htmlFor = selector.id;
  for (const copy of copies) {
    const option = element("option", undefined, `${copy.name} — ${copy.path} (${copy.installId})`);
    option.value = copy.installId;
    option.selected = copy.installId === initial.installId;
    selector.append(option);
  }
  label.append(selector);
  const check = actionButton(actions.installedUpdateBusy ? "Checking installed mods…" : "Check installed mods", () => {
    const copy = copies.find(entry => entry.installId === selector.value);
    if (!disabled && !actions.installedUpdateBusy && !actions.modUpgradeBusy && copy && copy.state !== "stale") return actions.onCheckInstalledUpdate?.(copy.installId);
  }, "quiet compact");
  if (sharedSelection) headingRow.append(check);
  else {
    toolbar.append(label, check);
    section.append(toolbar);
  }
  const content = element("div");
  section.append(content);

  function renderSelection(): void {
    content.replaceChildren();
    const copy = copies.find(entry => entry.installId === selector.value);
    check.disabled = disabled || actions.installedUpdateBusy === true || actions.modUpgradeBusy === true || !actions.onCheckInstalledUpdate || !copy || copy.state === "stale";
    if (!copy) return;
    if (!sharedSelection) heading.textContent = `Updates for ${copy.name}`;
    const current = copy.installId === (requestedId ?? initial.installId);
    const plan = actions.installedUpdatePlan?.installId === copy.installId ? actions.installedUpdatePlan : null;
    if (copy.state === "stale") {
      if (!sharedSelection) content.append(element("p", "update-note", "This folder is unavailable. Restore its location before comparing installed mods."));
    }
    else if (current && actions.installedUpdateBusy) {
      const progress = element("p", "update-note", "Comparing installed mods and saved choices…");
      progress.setAttribute("role", "status");
      content.append(progress);
    } else if (!plan && !sharedSelection) content.append(element("p", "update-note", "Not compared yet. Select Check installed mods to see the differences."));
    if (current && actions.installedUpdateError) {
      const error = element("p", "patch-error", actions.installedUpdateError);
      error.setAttribute("role", "alert");
      content.append(error);
      if (actions.installedUpdateErrorDetail?.trim()) {
        const details = element("details", "update-diagnostics");
        details.append(element("summary", undefined, "Comparison error details"), element("p", "path patch-path", actions.installedUpdateErrorDetail));
        content.append(details);
      }
    }
    if (current && (actions.modUpgrade?.installId === copy.installId || actions.modUpgradeError || actions.modUpgradeBusy)) {
      content.append(modUpgradeResult(copy, actions, disabled || actions.installedUpdateBusy === true || copy.state === "stale"));
    }
    if (plan && copy.state !== "stale" && !(current && actions.installedUpdateBusy)) {
      const details = element("details", "update-diagnostics installed-mod-comparison");
      details.append(element("summary", undefined, "Mod comparison details"),
        element("p", "update-note", "Compare the mods and choices in this installation with the collection included in CEBG. Checking never changes game files. Supported updates require your confirmation."),
        comparisonResult(plan));
      content.append(details);
    }
  }

  selector.addEventListener("change", () => {
    renderSelection();
    actions.onSelectInstalledUpdate?.(selector.value);
  });
  renderSelection();
  return section;
}
