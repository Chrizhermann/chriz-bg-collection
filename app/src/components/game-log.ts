import type { GameLogFile, GameLogStatus } from "../contracts";
import { actionButton, element } from "./app-shell";
import { persistDisclosure } from "./disclosure";
import { errorDetails } from "./error-details";
import { flavor } from "../flavor";

/** The collapsed Troubleshooting section of one installation. */
export interface GameLogView {
  readonly open: boolean;
  /** The last check, or null before the section was first opened. */
  readonly status: GameLogStatus | null;
  /** A check or a change is running. */
  readonly busy: boolean;
  readonly error: { readonly message: string; readonly recoveryAction: string; readonly technicalDetail: string } | null;
  /** Why the setting cannot be changed right now, such as a launch or recovery in progress. */
  readonly blockedReason: string | null;
}

export interface GameLogActions {
  readonly toggle: (open: boolean) => void;
  readonly set: (enabled: boolean) => void | Promise<void>;
  readonly refresh: () => void | Promise<void>;
}

function formatBytes(value: string): string {
  if (!/^\d+$/.test(value)) return "Size unknown";
  const amount = BigInt(value);
  const units = ["B", "KiB", "MiB", "GiB", "TiB"];
  let divisor = 1n;
  let unit = 0;
  while (amount / divisor >= 1024n && unit < units.length - 1) {
    divisor *= 1024n;
    unit++;
  }
  const tenths = (amount * 10n + divisor / 2n) / divisor;
  return `${tenths / 10n}${tenths % 10n === 0n ? "" : `.${tenths % 10n}`} ${units[unit]}`;
}

function fileFacts(file: GameLogFile): string {
  switch (file.state) {
    case "missing":
      return "Not written yet.";
    case "not-a-file":
      return "Something other than a file is in its place.";
    case "unreadable":
      return file.detail ? `Couldn't be checked: ${file.detail}` : "Couldn't be checked.";
    case "present": {
      const facts = [file.sizeBytes === null ? null : formatBytes(file.sizeBytes),
        file.modifiedAtMillis === null ? null : `last written ${new Date(file.modifiedAtMillis).toLocaleString()}`]
        .filter((fact): fact is string => fact !== null);
      return facts.length === 0 ? "Present." : `${facts.join(", ")}.`;
    }
  }
}

function crashFacts(count: number | null): string {
  if (count === null) return "Couldn't be checked.";
  if (count === 0) return "No crash reports yet.";
  return count === 1 ? "1 crash report." : `${count} crash reports.`;
}

function helpText(view: GameLogView): string {
  const status = view.status;
  if (status === null) return view.busy ? "Checking the game log…" : "The game log couldn't be checked.";
  if (status.setting === "unavailable") return `The game log can't be changed for this installation. ${status.detail ?? ""}`.trim();
  if (view.blockedReason !== null) return view.blockedReason;
  if (status.setting === "other") {
    return `The game writes its log to ${status.detail ?? "a file"}, as set outside ${flavor.shortName}. ${flavor.shortName} keeps no earlier copies of that file. Turning this off stops that log.`;
  }
  return `Takes effect the next time the game starts. The game starts a new log each time, so ${flavor.shortName} keeps the log of the previous session whenever you press Play.`;
}

/** Hidden from the normal flow: off unless the player opens it and turns the log on. */
export function troubleshootingSection(view: GameLogView, actions: GameLogActions): HTMLDetailsElement {
  const details = element("details", "installation-details troubleshooting");
  details.open = view.open;
  const summary = element("summary", undefined, "Troubleshooting");
  summary.id = "troubleshooting";
  details.append(summary);
  persistDisclosure(details, summary, actions.toggle);
  if (!view.open) return details;

  details.append(element("p", "muted", "If the game crashes or freezes, its own log and Windows crash dumps help find the cause."));
  const status = view.status;
  const checkbox = element("input");
  checkbox.type = "checkbox";
  checkbox.id = "game-log";
  checkbox.checked = status?.setting === "on" || status?.setting === "other";
  checkbox.disabled = status === null || status.setting === "unavailable" || view.busy || view.blockedReason !== null;
  checkbox.setAttribute("aria-describedby", "game-log-help");
  checkbox.addEventListener("change", () => {
    if (!checkbox.disabled) void actions.set(checkbox.checked);
  });
  const label = element("label", "game-log-choice");
  label.append(checkbox, element("span", undefined, "Write a game log for crash reports"));
  const help = element("p", "muted", helpText(view));
  help.id = "game-log-help";
  const option = element("div", "game-log-option");
  option.append(label, help);
  details.append(option);

  if (view.error !== null) {
    const failure = element("div", "shortcut-feedback warning");
    failure.setAttribute("role", "alert");
    failure.append(element("strong", undefined, view.error.message), element("p", undefined, view.error.recoveryAction));
    if (view.error.technicalDetail.trim().length > 0) failure.append(errorDetails(view.error.technicalDetail));
    details.append(failure);
  }

  if (status !== null) {
    const rows = element("dl", "installation-detail-list game-log-files");
    const appendRow = (title: string, path: string, facts: string): void => {
      const value = element("dd");
      value.append(element("span", "path", path), element("span", "muted", facts));
      rows.append(element("dt", undefined, title), value);
    };
    appendRow("Game log", status.log.path, fileFacts(status.log));
    appendRow("Previous session's log", status.previousLog.path, fileFacts(status.previousLog));
    appendRow("Crash reports", status.crashFolder, crashFacts(status.crashReports));
    details.append(rows);
    details.append(element("p", "muted", "After a crash, send that crash's .log and _small.dmp files from the crash reports folder; "
      + "with the game log on, the .log file is a copy of that session's log. For a freeze or another problem without a crash report, "
      + "send the game log, or the previous session's log if you have started the game again since. "
      + "The _big.dmp files take gigabytes; send one only when asked."));
  }
  const refresh = actionButton("Check again", actions.refresh, "quiet");
  refresh.id = "game-log-refresh";
  refresh.disabled = view.busy;
  const controls = element("div", "inline-actions");
  controls.append(refresh);
  details.append(controls);
  return details;
}
