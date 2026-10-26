import type { LaunchInstallResult, ManagedInstallation } from "../contracts";
import { actionButton, element, screenActions, screenIntro } from "../components/app-shell";
import { errorDetails } from "../components/error-details";
import { troubleshootingSection, type GameLogView } from "../components/game-log";
import { installationNameEditor } from "../components/installation-name";
import { flavor } from "../flavor";

export interface HomeActions {
  readonly begin: () => void | Promise<void>;
  readonly launch: (installId: string, startRadar: boolean) => void | Promise<void>;
  readonly setStartRadar?: (installId: string, enabled: boolean) => void;
  readonly openFolder: (installId: string) => void | Promise<void>;
  readonly resume: (installId: string) => void | Promise<void>;
  readonly select: (installId: string) => void | Promise<void>;
  readonly createShortcut: (installId: string) => void | Promise<void>;
  readonly diagnostics: (installId: string) => void | Promise<void>;
  readonly remove?: (installId: string) => void | Promise<void>;
  readonly removalDisabled?: boolean;
  readonly rename?: (installId: string, name: string) => Promise<void>;
  /** The game's own log for crash reports, in the collapsed Troubleshooting section. */
  readonly gameLog?: {
    readonly toggle: (installId: string, open: boolean) => void;
    readonly set: (installId: string, enabled: boolean) => void | Promise<void>;
    readonly refresh: (installId: string) => void | Promise<void>;
  };
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

export interface LaunchState {
  readonly startRadar: boolean;
  readonly pending: boolean;
  readonly feedback?: LaunchInstallResult;
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
  const used = new Set<string>();
  for (const installation of installations) {
    const duplicate = installations.filter(copy => copy.name.toLowerCase() === installation.name.toLowerCase()).length > 1;
    const folder = installation.path.replace(/[\\/]+$/, "").split(/[\\/]/).at(-1) ?? installation.path;
    const base = duplicate ? `${installation.name} — ${folder}` : installation.name;
    let label = base;
    for (let suffix = 2; used.has(label.toLowerCase()); suffix++) label = `${base} (${suffix})`;
    used.add(label.toLowerCase());
    const option = element("option", undefined, label);
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
  if (flavor.features.radar) appendRow("BG Radar Overlay", installation.radarVersion ?? null);
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
      // A standalone collection has one game, so its log target names no game.
      flavor.sources.length > 1 ? component.target : null,
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
  steps.append(element("li", undefined, `Start the game with Play ${flavor.productName}, then begin or load your adventure normally.`));
  const buffBotInstalled = flavor.features.buffBotTip && installation.consistency?.components?.some((component) =>
    component.status === "installed"
      && component.target === "BG2"
      && component.component === 0
      && component.tp2.replaceAll("\\", "/").toLowerCase().includes("buffbot/"));
  if (buffBotInstalled) {
    steps.append(element("li", undefined, "In game, press F11 to open BuffBot and choose the buffs you want automated."));
  }
  if (flavor.features.radarTip) {
    steps.append(element("li", undefined, installation.radarVersion
      ? `BG Radar Overlay ${installation.radarVersion} is installed. Select Start Radar with the game before pressing Play if you want to use it. Windows may ask for administrator approval. Its executable is in the BG Radar Overlay folder inside the game folder.`
      : "BG Radar Overlay is optional. You can add it from Updates after setup."));
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
  launchState: LaunchState = { startRadar: false, pending: false },
  gameLog: GameLogView | null = null,
): HTMLElement {
  const page = element("div", "screen-stack launcher-screen");
  if (selected === null) {
    page.append(screenIntro("", "My installs", `No installations yet. Set up ${flavor.productName} to start playing.`));
    page.append(screenActions(null, actionButton("New installation", actions.begin)));
    return page;
  }

  const patchRecoveryNeeded = selected.patchRecoveryNeeded === true;
  if (patchRecoveryNeeded) {
    page.append(screenIntro("", "Installation needs recovery", "Open Updates to check this installation and restore its backup before playing."));
  } else if (selected.available) {
    page.append(screenIntro("", "Ready to play", "Continue your adventure."));
  } else if (selected.resumable) {
    page.append(screenIntro("", "Continue your installation", "Your previous progress is saved and ready to resume."));
  } else {
    page.append(screenIntro("", "Installation needs attention", "This copy is incomplete or unavailable. Check its status below before starting again or removing it."));
  }

  const card = element("section", "card launcher-card");
  card.append(
    element("p", `badge ${patchRecoveryNeeded ? "danger" : selected.available ? "ok" : selected.resumable ? "warning" : "danger"}`, patchRecoveryNeeded ? "Patch recovery required" : selected.status),
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
  if (selected.customized) card.append(element("p", "manual-mod-warning", `Customized — manual mods added. Game hotpatches and whole-mod updates are disabled; ${flavor.shortName} app updates still work.`));
  const picker = installationPicker(installations, selected.id, actions.select);
  if (picker !== null) card.append(picker);
  if (actions.rename) card.append(installationNameEditor({ installId: selected.id, name: selected.name,
    installations: installations.map(copy => ({ installId: copy.id, name: copy.name })),
    disabled: actions.removalDisabled, onRename: actions.rename }));
  const controls = element("div", "inline-actions launcher-actions");
  let feedback: HTMLElement | null = null;
  if (selected.available) {
    const currentFeedback = shortcutFeedback?.installId === selected.id ? shortcutFeedback : null;
    const radar = element("input");
    radar.type = "checkbox";
    radar.id = "start-radar";
    radar.checked = flavor.features.radar && Boolean(selected.radarVersion) && launchState.startRadar;
    radar.disabled = !selected.radarVersion || patchRecoveryNeeded || launchState.pending;
    radar.setAttribute("aria-describedby", "radar-launch-help");
    radar.addEventListener("change", () => {
      if (!radar.disabled) actions.setStartRadar?.(selected.id, radar.checked);
    });
    const radarLabel = element("label", "radar-launch-choice");
    radarLabel.append(radar, element("span", undefined, "Start Radar with the game"));
    const radarHelp = element("p", "muted", patchRecoveryNeeded
      ? "Finish patch recovery in Updates before using Radar."
      : selected.radarVersion ? "Windows may ask for administrator approval." : "Add BG Radar Overlay from Updates to use this option.");
    radarHelp.id = "radar-launch-help";
    const radarOption = element("div", "radar-launch-option");
    radarOption.append(radarLabel, radarHelp);
    if (flavor.features.radar) card.append(radarOption);
    const play = actionButton(launchState.pending ? "Starting game…" : `Play ${flavor.productName}`,
      () => !patchRecoveryNeeded && !launchState.pending ? actions.launch(selected.id, radar.checked) : undefined);
    play.disabled = patchRecoveryNeeded || launchState.pending;
    controls.append(
      play,
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
        feedback.append(element("p", undefined, patchRecoveryNeeded
          ? "The optional shortcut can be retried after patch recovery."
          : "Your game is ready to play. Only the optional desktop shortcut failed."));
        if (currentFeedback.technicalDetail) feedback.append(errorDetails(currentFeedback.technicalDetail));
      }
    }
  } else if (selected.resumable && !patchRecoveryNeeded) {
    controls.append(
      actionButton(selected.status === "Paused for your mods" ? "Review manual-mod pause" : "Continue installation", () => actions.resume(selected.id)),
      actionButton("Export diagnostics", () => actions.diagnostics(selected.id), "quiet"),
    );
  } else {
    controls.append(actionButton("Export diagnostics", () => actions.diagnostics(selected.id), "quiet"));
  }
  card.append(controls);
  if (launchState.feedback?.installId === selected.id) {
    const { radar } = launchState.feedback;
    const warning = radar.state === "failed" || radar.state === "cancelled";
    const notice = element("div", `shortcut-feedback ${warning ? "warning" : "ok"}`);
    notice.setAttribute("role", warning ? "alert" : "status");
    notice.append(element("strong", undefined, radar.state === "cancelled"
      ? "Game started, but Radar wasn't started because administrator approval was cancelled."
      : radar.state === "failed" ? "Game started, but Radar couldn't start."
        : radar.state === "started" ? "Game and Radar started." : "Game started."));
    if (radar.detail) notice.append(element("p", undefined, radar.detail));
    if (radar.technicalDetail) notice.append(errorDetails(radar.technicalDetail));
    card.append(notice);
    const { gameSettings } = launchState.feedback;
    if (gameSettings.state === "failed") {
      const settings = element("div", "shortcut-feedback warning");
      settings.setAttribute("role", "alert");
      settings.append(
        element("strong", undefined, "The game's settings file couldn't be updated."),
        element("p", undefined, flavor.text.gameSettingsFailed),
      );
      if (gameSettings.technicalDetail) settings.append(errorDetails(gameSettings.technicalDetail));
      card.append(settings);
    }
    const { gameLog: keptLog } = launchState.feedback;
    if (keptLog.state === "failed") {
      const log = element("div", "shortcut-feedback warning");
      log.setAttribute("role", "alert");
      log.append(
        element("strong", undefined, "The log of your last game session couldn't be kept."),
        element("p", undefined, "The game started anyway. Before the next start, close any program that has the game's log files open, such as a game that was still running."),
      );
      if (keptLog.technicalDetail) log.append(errorDetails(keptLog.technicalDetail));
      card.append(log);
    }
  }
  if (actions.remove) {
    const removal = actionButton("Delete / remove installation…", () => actions.remove!(selected.id), "danger-quiet");
    removal.id = "remove-installation";
    removal.disabled = actions.removalDisabled ?? false;
    card.append(removal);
  }
  card.append(element("p", "muted", "Diagnostics stay local; review the ZIP before sharing."));
  if (addonFeedback?.installId === selected.id) {
    const note = element("p", "addon-feedback", patchRecoveryNeeded
      ? "Finish patch recovery in Updates before using the game or overlay."
      : addonFeedback.state === "installing"
        ? "Adding BG Radar Overlay… Your game is ready to play."
        : "BG Radar Overlay couldn't be added. Your game is ready; retry the overlay from Updates.");
    note.setAttribute("role", "status");
    card.append(note);
  }
  if (feedback !== null) card.append(feedback);
  card.append(installationDetails(selected));
  const mods = installedMods(selected);
  if (mods !== null) card.append(mods);
  if (selected.available && !patchRecoveryNeeded) card.append(firstPlayGuide(selected));
  const gameLogActions = actions.gameLog;
  if (selected.available && gameLog !== null && gameLogActions !== undefined) {
    card.append(troubleshootingSection(gameLog, {
      toggle: (open) => gameLogActions.toggle(selected.id, open),
      set: (enabled) => gameLogActions.set(selected.id, enabled),
      refresh: () => gameLogActions.refresh(selected.id),
    }));
  }
  page.append(card);
  page.append(screenActions(null, actionButton("New installation", actions.begin, "quiet")));
  return page;
}
