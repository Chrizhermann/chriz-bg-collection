import type { BuildSnapshot } from "../contracts";
import { actionButton, element, screenIntro } from "../components/app-shell";
import { campaignLedger } from "../components/campaign-ledger";
import { errorDetails } from "../components/error-details";
import { statusCard } from "../components/status-card";
import { technicalLog, type TechnicalLogState } from "../components/technical-log";

export interface BuildActions {
  readonly backToSetup: () => void | Promise<void>;
  readonly advance: () => void | Promise<void>;
  readonly retry: () => void | Promise<void>;
  readonly supplyManual: () => void | Promise<void>;
  readonly openManualSource: () => void | Promise<void>;
  readonly pause: () => void | Promise<void>;
  readonly stopNow: () => void | Promise<void>;
  readonly diagnostics: () => void | Promise<void>;
  readonly diagnosticsAvailable: boolean;
  readonly fixture: boolean;
  readonly retryAvailable: boolean;
  readonly logState: TechnicalLogState;
  readonly updateLogState: (state: TechnicalLogState) => void;
}

export function buildScreen(snapshot: BuildSnapshot, actions: BuildActions): HTMLElement {
  const page = element("div", "screen-stack build-screen");
  page.append(screenIntro("", "Installation progress", ""));
  const tone = snapshot.state === "failed" ? "danger" : snapshot.state === "running" || snapshot.state === "complete" ? "ok" : "warning";
  const stateCard = statusCard(snapshot.headline, snapshot.detail, tone);
  if (snapshot.failureReason) stateCard.insertBefore(element("p", "failure-reason", snapshot.failureReason), stateCard.children[1] ?? null);
  const controls = element("div", "inline-actions");
  if (snapshot.recoveryAction) stateCard.append(element("p", "recovery-action", snapshot.recoveryAction));
  if (snapshot.state === "failed" && snapshot.commandError?.technical_detail) {
    stateCard.append(errorDetails(snapshot.commandError.technical_detail, snapshot.commandError.code === "unsafe_target"));
  }
  if (snapshot.state === "waiting-manual") {
    stateCard.append(element("p", "path", snapshot.manualArchiveName ?? ""));
    controls.append(actionButton("Open download page", actions.openManualSource, "quiet"));
    if (actions.fixture) controls.append(actionButton("I added the archive", actions.advance));
    else if (actions.retryAvailable) controls.append(actionButton("Choose downloaded archive", actions.supplyManual));
  } else if (snapshot.state === "attention") {
    controls.append(actionButton("Keep waiting", actions.advance));
  } else if (snapshot.state === "failed") {
    if (snapshot.freshCopyRequired) controls.append(actionButton("Start new installation", actions.backToSetup));
    else if (actions.retryAvailable) controls.append(actionButton("Retry failed step", actions.retry));
    else controls.append(actionButton("Back to setup", actions.backToSetup));
    if (actions.diagnosticsAvailable) controls.append(actionButton("Export diagnostics", actions.diagnostics, "quiet"));
  } else if (snapshot.state === "paused") {
    controls.append(actionButton("Resume installation", actions.retry));
  } else if (snapshot.state === "running") {
    controls.append(actions.fixture
      ? actionButton("Finish fixture build", actions.advance)
      : actionButton("Pause after current mod", actions.pause));
    if (!actions.fixture) controls.append(actionButton("Stop now (may need repair)", actions.stopNow, "quiet"));
  }
  if (controls.childElementCount > 0) stateCard.append(controls);
  page.append(stateCard, campaignLedger(snapshot.phases, true), technicalLog(snapshot.logTail, actions.logState, actions.updateLogState));
  return page;
}
