import type { BackendStatus, BuildSnapshot, GameCandidate, GameLogStatus, ManagedInstallation, PendingDecision, Storefront } from "./contracts";
import { flavor, sourceFor, type SourceSlot } from "./flavor";

/**
 * What a report says. The player can send it straight from the app, or open the product's GitHub
 * issue page or Discord server with this text.
 */
export type ReportKind = "problem" | "feedback";

/** What the player reports, from the place in the app where they opened the dialog. */
export interface ReportSubject {
  /** A fixed kind, or null when the player chooses between a problem and feedback. */
  readonly kind: ReportKind | null;
  /** A short line that names the problem; "" when the app knows none. */
  readonly summary: string;
  /** What happened, one fact per line. Values may hold paths until they are redacted. */
  readonly facts: readonly string[];
  /** The installation whose diagnostics the player can save from the dialog. */
  readonly installId: string | null;
  /** The control that gets the focus back when the dialog closes. */
  readonly returnFocusId: string;
}

/** The longest text the player can add; a prefilled link has to hold it. */
export const REPORT_TEXT_LIMIT = 1200;
/** Maximum length of the optional reply handle or email address. */
export const REPORT_CONTACT_LIMIT = 120;
const FACT_LIMIT = 400;
const TITLE_LIMIT = 100;

const STOREFRONTS: Readonly<Record<Storefront, string>> = {
  steam: "Steam", gog: "GOG", beamdog: "Beamdog", microsoft: "Microsoft Store", other: "another source",
};

function clip(text: string, limit: number): string {
  const single = text.replace(/\s+/gu, " ").trim();
  return single.length <= limit ? single : `${single.slice(0, limit - 1).trimEnd()}…`;
}

function fact(label: string, value: string | null | undefined): string[] {
  const text = value?.trim();
  return text ? [`${label}: ${clip(text, FACT_LIMIT)}`] : [];
}

/** The app, collection and game versions, none of which holds a path. */
export function environmentFacts(
  status: BackendStatus,
  games: readonly { readonly slot: SourceSlot; readonly candidate: GameCandidate }[],
  installation: ManagedInstallation | null,
): string[] {
  const setup = status.profiles?.find(profile => profile.id === status.selectedProfile)?.label;
  const collection = status.recipeVersion === null ? null : `${status.recipeVersion}${setup ? ` (${setup})` : ""}`;
  return [
    `App: ${flavor.productName} ${status.applicationVersion ?? status.engineVersion}`,
    ...fact("Collection", collection),
    ...games.map(({ slot, candidate }) => `Game: ${sourceFor(slot)?.name ?? slot} ${candidate.build ?? "unknown version"}, ${STOREFRONTS[candidate.storefront]}${candidate.tested === false ? ", less-tested copy" : ""}`),
    ...(installation === null ? [] : [
      `Installation: collection ${installation.recipeVersion ?? "unknown"}, ${installation.status}${installation.customized ? ", with manual mods" : ""}`,
    ]),
  ];
}

/** A report without a subject, from the button that is always in the header. */
export function generalSubject(installId: string | null): ReportSubject {
  return { kind: null, summary: "", facts: [], installId, returnFocusId: "report-feedback" };
}

/** An installation that stopped. */
export function failedInstallationSubject(build: BuildSnapshot, stepId: string | null, installId: string | null): ReportSubject {
  const phase = build.phases.find(candidate => candidate.state === "failed") ?? build.phases.find(candidate => candidate.state === "current");
  const where = phase?.title ?? stepId;
  const error = build.commandError;
  return {
    kind: "problem",
    summary: where ? `Installation stopped at "${clip(where, 60)}"` : "Installation stopped",
    facts: [
      ...fact("What happened", build.stoppedByAnswer ? `${build.headline} (stopped by the player's answer)` : build.headline),
      ...fact("Phase", phase?.title),
      ...fact("Step", stepId),
      ...fact("Reason", build.failureReason),
      ...fact("Error", error ? `${error.code}: ${error.message}` : null),
      ...fact("Error details", error?.technical_detail),
    ],
    installId,
    returnFocusId: "report-installation",
  };
}

/** A mod part whose installer failed, while its question is open. */
export function failedComponentSubject(decision: PendingDecision): ReportSubject {
  const sentence = /^.*?[.?!](?=\s|$)/u.exec(decision.message)?.[0] ?? decision.message;
  return {
    kind: "problem",
    summary: clip(sentence.replace(/[.?!]$/u, ""), TITLE_LIMIT),
    facts: [
      ...fact("What happened", decision.message),
      ...fact("Step", decision.stepId),
      ...decision.details.flatMap(detail => fact("Detail", detail)),
    ],
    installId: decision.installId,
    returnFocusId: "decision-report",
  };
}

/** A finished installation that left mods out or used changed downloads. */
export function leftOutSubject(notes: readonly string[], installId: string): ReportSubject {
  return {
    kind: "problem",
    summary: "Installation finished with mods left out or changed",
    facts: notes.flatMap(note => fact("Installation summary", note)),
    installId,
    returnFocusId: "report-summary",
  };
}

/** A crash, freeze or other problem in the game of one installation. */
export function gameProblemSubject(installation: ManagedInstallation, gameLog: GameLogStatus | null): ReportSubject {
  const crashes = gameLog?.crashReports;
  return {
    kind: "problem",
    summary: "",
    facts: [
      "What happened: a problem in the game (crash, freeze or bug)",
      ...fact("Game log", gameLog === null ? null : gameLog.setting === "on" || gameLog.setting === "other" ? "on" : gameLog.setting === "off" ? "off" : null),
      ...fact("Crash reports in the crash folder", crashes == null ? null : String(crashes)),
    ],
    installId: installation.id,
    returnFocusId: "report-game-problem",
  };
}

function firstLine(text: string): string {
  return text.split(/\r?\n/u).map(line => line.trim()).find(line => line.length > 0) ?? "";
}

/** The title of the GitHub issue and the first line of a copied report. */
export function reportTitle(kind: ReportKind, summary: string, text: string): string {
  const subject = summary || firstLine(text) || (kind === "problem" ? "Problem report" : "Feedback");
  return clip(`[${kind === "problem" ? "Problem" : "Feedback"}] ${subject}`, TITLE_LIMIT);
}

/** The report as the player sends it: their words, then the details the app adds. */
export function reportBody(kind: ReportKind, text: string, details: string, diagnostics: boolean): string {
  const words = text.trim() || (kind === "problem"
    ? "(Describe what happened and what you expected.)"
    : "(Write your feedback or idea here.)");
  return [
    words,
    "",
    `Details from ${flavor.productName}:`,
    details,
    ...(diagnostics ? ["", "Diagnostics ZIP: attach it to this report if you saved one."] : []),
  ].join("\n");
}
