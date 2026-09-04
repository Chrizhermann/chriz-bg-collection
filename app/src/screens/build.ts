import type { BuildSnapshot } from "../contracts";
import { actionButton, element, screenIntro } from "../components/app-shell";
import { campaignLedger } from "../components/campaign-ledger";
import { statusCard } from "../components/status-card";
import { technicalLog, type TechnicalLogState } from "../components/technical-log";

export interface BuildActions {
  readonly advance: () => void | Promise<void>;
  readonly retry: () => void | Promise<void>;
  readonly supplyManual: () => void | Promise<void>;
  readonly cancel: () => void | Promise<void>;
  readonly diagnostics: () => void | Promise<void>;
  readonly fixture: boolean;
  readonly retryAvailable: boolean;
  readonly logState: TechnicalLogState;
  readonly updateLogState: (state: TechnicalLogState) => void;
}

export function buildScreen(snapshot: BuildSnapshot, actions: BuildActions): HTMLElement {
  const page = element("div", "screen-stack");
  page.append(screenIntro("Step 5 of 5", "Build your campaign", actions.fixture ? "A resumable fixture demonstrates pauses, attention, failure recovery, and completion." : "The engine is installing the exact reviewed recipe into your separate managed copy."));
  const tone = snapshot.state === "failed" ? "danger" : snapshot.state === "running" || snapshot.state === "complete" ? "ok" : "warning";
  const stateCard = statusCard(snapshot.headline, snapshot.detail, tone);
  const controls = element("div", "inline-actions");
  if (snapshot.state === "waiting-manual") {
    stateCard.append(element("p", "path", snapshot.manualArchiveName ?? ""));
    if (actions.fixture) controls.append(actionButton("I added the archive", actions.advance));
    else if (actions.retryAvailable) controls.append(actionButton("Choose downloaded archive", actions.supplyManual));
  } else if (snapshot.state === "attention") {
    controls.append(actionButton("Continue build", actions.advance));
  } else if (snapshot.state === "failed") {
    if (actions.retryAvailable) controls.append(actionButton("Retry failed step", actions.retry));
    if (actions.fixture) controls.append(actionButton("Export diagnostics", actions.diagnostics, "quiet"));
  } else if (snapshot.state === "running") {
    controls.append(actions.fixture
      ? actionButton("Finish fixture build", actions.advance)
      : actionButton("Cancel build", actions.cancel, "quiet"));
  }
  stateCard.append(controls);
  page.append(stateCard, campaignLedger(snapshot.phases, true), technicalLog(snapshot.logTail, actions.logState, actions.updateLogState));
  return page;
}
