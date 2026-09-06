import type { ManagedInstallation } from "../contracts";
import { actionButton, element, screenActions, screenIntro } from "../components/app-shell";
import { errorDetails } from "../components/error-details";

export interface HomeActions {
  readonly begin: () => void | Promise<void>;
  readonly launch: (installId: string) => void | Promise<void>;
  readonly openFolder: (installId: string) => void | Promise<void>;
  readonly resume: (installId: string) => void | Promise<void>;
  readonly select: (installId: string) => void | Promise<void>;
  readonly createShortcut: (installId: string) => void | Promise<void>;
  readonly diagnostics: (installId: string) => void | Promise<void>;
}

export interface ShortcutFeedback {
  readonly installId: string;
  readonly state: "created" | "failed";
  readonly path?: string;
  readonly technicalDetail?: string;
}

export interface AddonFeedback {
  readonly installId: string;
  readonly state: "installing" | "failed";
}

function installationPicker(
  installations: readonly ManagedInstallation[],
  selectedId: string,
  selectInstallation: (installId: string) => void | Promise<void>,
): HTMLElement | null {
  if (installations.length < 2) return null;
  const field = element("div", "install-switcher");
  const label = element("label", undefined, "Switch install");
  const select = element("select");
  for (const installation of installations) {
    const option = element("option", undefined, installation.name);
    option.value = installation.id;
    option.selected = installation.id === selectedId;
    select.append(option);
  }
  select.addEventListener("change", () => void selectInstallation(select.value));
  label.append(select);
  field.append(label);
  return field;
}

function installationDetails(installation: ManagedInstallation): HTMLDetailsElement {
  const details = element("details", "installation-details");
  details.append(element("summary", undefined, "Installation details"));
  const rows = element("dl", "installation-detail-list");
  const appendRow = (label: string, value: string | null): void => {
    if (value !== null) rows.append(element("dt", undefined, label), element("dd", "path", value));
  };
  appendRow("Install location", installation.path);
  appendRow("Game launcher", installation.launchPath);
  appendRow("Installation receipt", installation.receiptPath);
  appendRow("Collection version", installation.recipeVersion ?? null);
  appendRow("BG Radar Overlay", installation.radarVersion ?? null);
  appendRow("Mod check", installation.consistency?.detail ?? null);
  details.append(rows);
  return details;
}

function componentLabel(tp2: string): string {
  const file = tp2.replaceAll("\\", "/").split("/").at(-1) ?? tp2;
  return file.replace(/^setup-/i, "").replace(/\.tp2$/i, "") || tp2;
}

function installedMods(installation: ManagedInstallation): HTMLDetailsElement | null {
  const consistency = installation.consistency;
  const components = consistency?.components;
  if (consistency === undefined || components === undefined || components.length === 0) return null;
  const details = element("details", "installation-details installed-mods");
  details.append(element("summary", undefined, `Installed mods (${consistency.modCount} mods, ${consistency.componentCount} components)`));
  const explanation = consistency.state === "matches"
    ? "The current WeiDU logs match the completed installation record. Titles and versions below are reported by those logs."
    : "Compared with the completed installation record. Missing and extra rows are called out below; changed order is reported by the mod check above. Titles and versions are reported by the current logs.";
  details.append(element("p", "muted", explanation));
  const searchLabel = element("label", "installed-mod-search", "Find an installed mod");
  const search = element("input") as HTMLInputElement;
  search.type = "search";
  search.placeholder = "Name, version, component, or TP2";
  searchLabel.append(search);
  details.append(searchLabel);
  const list = element("ul", "installed-mod-list");
  for (const component of components) {
    const item = element("li", `installed-mod ${component.status}`);
    item.append(element("strong", undefined, component.title ?? componentLabel(component.tp2)));
    const facts = [
      component.version,
      `Component ${component.component}`,
      component.target,
      component.status === "installed" ? "Recorded and present" : component.status === "missing" ? "Recorded but missing" : "Present but not recorded",
      component.tp2,
    ].filter((value): value is string => value !== null);
    item.append(element("span", "installation-component-meta", facts.join(" · ")));
    item.dataset.search = `${component.title ?? ""} ${facts.join(" ")}`.toLocaleLowerCase();
    list.append(item);
  }
  search.addEventListener("input", () => {
    const query = search.value.trim().toLocaleLowerCase();
    for (const item of list.querySelectorAll<HTMLElement>(".installed-mod")) {
      item.hidden = query.length > 0 && !(item.dataset.search ?? "").includes(query);
    }
  });
  details.append(list);
  return details;
}

export function firstPlayGuide(installation: ManagedInstallation): HTMLDetailsElement {
  const guide = element("details", "installation-details first-play-guide");
  guide.append(element("summary", undefined, "First-play tips"));
  const steps = element("ol");
  steps.append(element("li", undefined, "Start the game with Play Chriz Easy BG, then begin or load your adventure normally."));
  const buffBotInstalled = installation.consistency?.components?.some((component) =>
    component.status === "installed"
      && component.target === "BG2"
      && component.component === 0
      && component.tp2.replaceAll("\\", "/").toLowerCase().includes("buffbot/"));
  if (buffBotInstalled) {
    steps.append(element("li", undefined, "In game, press F11 to open BuffBot and choose the buffs you want automated."));
  }
  if (installation.radarVersion) {
    steps.append(element("li", undefined, `BG Radar Overlay ${installation.radarVersion} is installed. Play Chriz Easy BG starts it with the game; its executable is also in the BG Radar Overlay folder inside the game folder.`));
  } else {
    steps.append(element("li", undefined, "BG Radar Overlay is optional. You can add it from Updates after setup."));
  }
  guide.append(steps);
  return guide;
}

export function homeScreen(
  installations: readonly ManagedInstallation[],
  selected: ManagedInstallation | null,
  actions: HomeActions,
  shortcutFeedback: ShortcutFeedback | null = null,
  addonFeedback: AddonFeedback | null = null,
): HTMLElement {
  const page = element("div", "screen-stack launcher-screen");
  if (selected === null) {
    page.append(screenIntro("", "My installs", "No installations yet. Set up Chriz Easy BG to start playing."));
    page.append(screenActions(null, actionButton("New installation", actions.begin)));
    return page;
  }

  if (selected.available) {
    page.append(screenIntro("", "Ready to play", "Continue your adventure."));
  } else if (selected.resumable) {
    page.append(screenIntro("", "Continue your installation", "Your previous progress is saved and ready to resume."));
  } else {
    page.append(screenIntro("", "Installation not found", "The game folder moved or is no longer available. Reconnect the drive if it is stored elsewhere."));
  }

  const card = element("section", "card launcher-card");
  card.append(
    element("p", `badge ${selected.available ? "ok" : selected.resumable ? "warning" : "danger"}`, selected.status),
    element("h2", undefined, selected.name),
  );
  const metadata = element("div", "installation-metadata");
  if (selected.recipeVersion) metadata.append(element("span", undefined, `Collection ${selected.recipeVersion}`));
  if (selected.consistency !== undefined) {
    const check = selected.consistency;
    const summary = element("span", `consistency-summary ${check.state}`, check.state === "matches"
      ? `${check.modCount} mods · ${check.componentCount} components checked`
      : check.state === "changed" ? "Mods changed since installation" : "Mod check unavailable");
    summary.dataset.consistency = check.state;
    metadata.append(summary);
  }
  if (metadata.childElementCount > 0) card.append(metadata);
  const picker = installationPicker(installations, selected.id, actions.select);
  if (picker !== null) card.append(picker);
  const controls = element("div", "inline-actions launcher-actions");
  let feedback: HTMLElement | null = null;
  if (selected.available) {
    const currentFeedback = shortcutFeedback?.installId === selected.id ? shortcutFeedback : null;
    controls.append(
      actionButton("Play Chriz Easy BG", () => actions.launch(selected.id)),
      actionButton("Open game folder", () => actions.openFolder(selected.id), "quiet"),
      actionButton(
        currentFeedback?.state === "created"
          ? "Recreate desktop shortcut"
          : currentFeedback?.state === "failed"
            ? "Retry desktop shortcut"
            : "Create desktop shortcut",
        () => actions.createShortcut(selected.id),
        "quiet",
      ),
      actionButton("Export diagnostics", () => actions.diagnostics(selected.id), "quiet"),
    );
    if (currentFeedback !== null) {
      feedback = element("div", `shortcut-feedback ${currentFeedback.state === "created" ? "ok" : "warning"}`);
      feedback.setAttribute("role", "status");
      feedback.append(element("strong", undefined, currentFeedback.state === "created" ? "Shortcut created on your desktop." : "The desktop shortcut wasn't created."));
      if (currentFeedback.path !== undefined) feedback.append(element("p", "path", currentFeedback.path));
      if (currentFeedback.state === "failed") {
        feedback.append(element("p", undefined, "Your game is ready to play. Only the optional desktop shortcut failed."));
        if (currentFeedback.technicalDetail) feedback.append(errorDetails(currentFeedback.technicalDetail));
      }
    }
  } else if (selected.resumable) {
    controls.append(
      actionButton("Continue installation", () => actions.resume(selected.id)),
      actionButton("Export diagnostics", () => actions.diagnostics(selected.id), "quiet"),
    );
  } else {
    controls.append(actionButton("Export diagnostics", () => actions.diagnostics(selected.id), "quiet"));
  }
  card.append(controls);
  card.append(element("p", "muted", "Diagnostics stay local; review the ZIP before sharing."));
  if (addonFeedback?.installId === selected.id) {
    const note = element("p", "addon-feedback", addonFeedback.state === "installing"
      ? "Adding BG Radar Overlay… Your game is ready to play."
      : "BG Radar Overlay couldn't be added. Your game is ready; retry the overlay from Updates.");
    note.setAttribute("role", "status");
    card.append(note);
  }
  if (feedback !== null) card.append(feedback);
  card.append(installationDetails(selected));
  const mods = installedMods(selected);
  if (mods !== null) card.append(mods);
  if (selected.available) card.append(firstPlayGuide(selected));
  page.append(card);
  page.append(screenActions(null, actionButton("New installation", actions.begin, "quiet")));
  return page;
}
