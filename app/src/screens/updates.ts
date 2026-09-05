import type { UpdateSummary } from "../contracts";
import { actionButton, element, screenIntro } from "../components/app-shell";

export interface UpdateActions {
  readonly checkAgain: () => void | Promise<void>;
  readonly installApplication: (version: string) => void | Promise<void>;
  readonly buildUpdatedCopy: (version: string) => void | Promise<void>;
  readonly installRadar?: () => void | Promise<void>;
  readonly installationName?: string;
  readonly radarVersion?: string | null;
}

const stateLabels: Readonly<Record<string, string>> = {
  "up-to-date": "Up to date", available: "Update available",
  "local-available": "New installation available",
  "requires-app": "Update CEBG first", offline: "Could not connect",
  invalid: "Could not verify update", replayed: "Outdated update ignored",
  unavailable: "Not available yet", "not-installed": "Not installed",
};

function versionRow(
  title: string, currentVersion: string | null, availableVersion: string | null,
  state: string, changelog: HTMLElement, action?: HTMLButtonElement, note?: string, currentLabel = "Installed",
): HTMLElement {
  const row = element("article", "update-row");
  if (["available", "requires-app", "local-available"].includes(state) && availableVersion !== null) row.classList.add("available");
  const installed = element("div", "update-installed");
  installed.append(element("h2", undefined, title), element("p", undefined, currentVersion === null ? "Not installed" : `${currentLabel}: ${currentVersion === "bundled" ? "Bundled alpha" : currentVersion}`));
  if (note !== undefined) installed.append(element("p", "update-purpose", note));
  const latest = element("div", "update-latest");
  if (availableVersion !== null) {
    changelog.hidden = true;
    changelog.id = `changelog-${title.toLowerCase().replaceAll(" ", "-")}`;
    const version = actionButton(availableVersion, () => {
      changelog.hidden = !changelog.hidden;
      version.setAttribute("aria-expanded", String(!changelog.hidden));
    }, "version-link");
    version.setAttribute("aria-label", `${title} ${availableVersion} changelog`);
    version.setAttribute("aria-expanded", "false");
    version.setAttribute("aria-controls", changelog.id);
    latest.append(element("span", "update-version-label", "Latest version"), version);
  } else if (state === "up-to-date" && currentVersion !== null) {
    latest.append(element("span", "update-version-label", `Latest: ${currentVersion}`));
  }
  latest.append(element("span", `update-state ${state}`, stateLabels[state] ?? state));
  row.append(installed, latest);
  if (action !== undefined) row.append(action);
  if (availableVersion !== null) row.append(changelog);
  return row;
}

export function updatesScreen(updates: UpdateSummary, actions: UpdateActions, checking = false, installing = false): HTMLElement {
  const page = element("div", "screen-stack updates-screen");
  const heading = element("div", "updates-heading");
  const check = actionButton(checking ? "Checking…" : "Check for updates", actions.checkAgain, "quiet");
  check.disabled = checking || installing;
  heading.append(screenIntro("", "Updates", ""), check);
  page.append(heading);

  const versions = element("section", "update-versions");
  versions.setAttribute("aria-label", "Installed and available versions");
  const appNotes = element("div", "update-changelog");
  appNotes.append(element("h3", undefined, "What's new"), element("p", undefined, updates.application.releaseNotes ?? "No changelog was included with this update."));
  const appVersion = updates.application.availableVersion;
  versions.append(versionRow("CEBG app", updates.application.currentVersion, appVersion, updates.application.state, appNotes,
    updates.application.state === "available" && appVersion !== null
      ? actionButton("Update CEBG app", () => actions.installApplication(appVersion), "primary compact")
      : undefined,
    updates.application.state === "available" ? "Updates the installer and launcher, not your game files." : undefined));

  const recipeNotes = element("div", "update-changelog");
  recipeNotes.append(element("h3", undefined, "What's new"));
  for (const change of updates.recipe.changes) {
    const item = element("div", "update-change");
    item.append(element("h4", undefined, change.title), element("p", undefined, change.summary));
    if (change.conditionNote !== null) item.append(element("p", "fixture-note", change.conditionNote));
    recipeNotes.append(item);
  }
  if (updates.recipe.changes.length === 0) recipeNotes.append(element("p", undefined, "No changelog was included with this update."));
  const localCollectionAvailable = updates.recipe.state === "up-to-date" && updates.managedCopies.some((copy) => copy.state === "update-available");
  const recipeVersion = localCollectionAvailable ? updates.recipe.currentVersion : updates.recipe.availableVersion;
  const recipeState = localCollectionAvailable ? "local-available" : updates.recipe.state;
  versions.append(versionRow("Collection", updates.recipe.currentVersion, recipeVersion, recipeState, recipeNotes,
    (updates.recipe.state === "available" || localCollectionAvailable) && recipeVersion !== null
      ? actionButton("Create updated installation", () => actions.buildUpdatedCopy(recipeVersion), "quiet compact")
      : undefined,
    updates.recipe.state === "requires-app"
      ? "For your next playthrough. Included with the CEBG app update; your current game stays unchanged."
      : updates.recipe.state === "available" && updates.recipe.disposition === "deferred-for-next-playthrough"
        ? "For your next playthrough"
        : localCollectionAvailable ? "For your next playthrough. Creates a new installation; your current game stays unchanged." : undefined,
    "Included in CEBG"));
  if (updates.radar !== undefined) {
    const radar = updates.radar;
    const notes = element("div", "update-changelog");
    notes.append(element("h3", undefined, "What's new"), element("p", undefined, radar.releaseNotes ?? "No changelog was included with this release."));
    const currentVersion = actions.radarVersion !== undefined ? actions.radarVersion : radar.currentVersion;
    const state = radar.state === "offline" ? "offline" : currentVersion === null ? "not-installed" : currentVersion === radar.availableVersion ? "up-to-date" : radar.state;
    versions.append(versionRow("BG Radar Overlay", currentVersion, radar.availableVersion, state, notes,
      actions.installRadar !== undefined && radar.availableVersion !== null && state !== "up-to-date" && state !== "offline"
        ? actionButton(currentVersion === null ? "Install overlay" : "Update overlay", actions.installRadar, "quiet compact")
        : undefined));
    if (actions.installationName !== undefined) {
      notes.append(element("p", "update-note", `Installs into ${actions.installationName}.`));
    }
  }
  if (installing) {
    versions.querySelectorAll<HTMLButtonElement>("button").forEach((button) => { button.disabled = true; });
  }
  page.append(versions);
  if (updates.recipe.state === "available" && updates.recipe.disposition !== "deferred-for-next-playthrough") {
    page.append(element("p", "update-note", "Collection updates create a new installation. Open the changelog for save compatibility."));
  }
  if (updates.radar !== undefined && actions.installRadar === undefined) {
    page.append(element("p", "update-note", "BG Radar Overlay can be added after your game installation finishes."));
  }

  if (updates.managedCopies.length > 0) {
    const installs = element("details", "updates-installs");
    installs.append(element("summary", undefined, `Your installations (${updates.managedCopies.length})`));
    const list = element("ul", "update-install-list");
    for (const copy of updates.managedCopies) {
      const status = copy.state === "stale" ? "Folder not found" : copy.state === "update-available" ? "New collection available" : copy.state === "up-to-date" ? "Up to date" : "Not checked";
      const item = element("li");
      item.append(element("strong", undefined, copy.name), element("span", undefined, `${copy.installedRecipeVersion ?? "Unknown version"} · ${status}`));
      list.append(item);
    }
    installs.append(list);
    page.append(installs);
  }

  const checkStatus = element("p", "update-note");
  checkStatus.setAttribute("role", "status");
  checkStatus.textContent = installing ? "Installing the update…"
    : checking ? "Checking for updates…"
    : updates.networkState === "unconfigured" ? "Online updates aren't enabled in this alpha yet."
    : updates.networkState === "offline" ? "Could not connect. Try checking again when you're online."
    : updates.checkedAt === null ? "Not checked yet." : `Last checked: ${new Date(updates.checkedAt).toLocaleString()}`;
  page.append(checkStatus);
  if (["offline", "invalid", "replayed"].includes(updates.application.state) || ["offline", "invalid", "replayed"].includes(updates.recipe.state)) {
    const details = element("details", "update-diagnostics");
    details.append(element("summary", undefined, "Check details"));
    for (const result of [updates.application, updates.recipe]) {
      if (["offline", "invalid", "replayed"].includes(result.state)) details.append(element("p", undefined, result.detail));
    }
    page.append(details);
  }
  return page;
}
