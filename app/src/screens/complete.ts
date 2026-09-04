import type { FrozenReview } from "../contracts";
import { actionButton, element, screenActions, screenIntro } from "../components/app-shell";
import { statusCard } from "../components/status-card";

export interface CompleteActions {
  readonly home: () => void | Promise<void>;
  readonly launch: (() => void | Promise<void>) | null;
  readonly openFolder: (() => void | Promise<void>) | null;
}

export function completeScreen(review: FrozenReview | null, actions: CompleteActions): HTMLElement {
  const page = element("div", "screen-stack");
  page.append(screenIntro("Build verified", "Campaign complete", "The reviewed managed campaign and its durable evidence are ready."));
  page.append(statusCard("Immutable install receipt", review ? `Review ${review.digest} is bound to ${review.destination}.` : "The campaign receipt is available from its managed copy." , "ok"));
  const campaignActions = element("div", "inline-actions");
  if (actions.launch !== null && actions.openFolder !== null) {
    campaignActions.append(
      actionButton("Launch game", actions.launch),
      actionButton("Open folder", actions.openFolder, "quiet"),
    );
  }
  page.append(campaignActions, screenActions(null, actionButton("View campaigns", actions.home)));
  return page;
}
