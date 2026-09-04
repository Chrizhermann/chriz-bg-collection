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
  page.append(screenIntro("Installation verified", "Chriz Easy BG is ready", "Your game and its installation record are ready."));
  page.append(statusCard("Installation record saved", review ? `Review ${review.digest} is bound to ${review.destination}.` : "The installation record is available in the game folder." , "ok"));
  const installActions = element("div", "inline-actions");
  if (actions.launch !== null && actions.openFolder !== null) {
    installActions.append(
      actionButton("Play Chriz Easy BG", actions.launch),
      actionButton("Open game folder", actions.openFolder, "quiet"),
    );
  }
  page.append(installActions, screenActions(null, actionButton("My installs", actions.home)));
  return page;
}
