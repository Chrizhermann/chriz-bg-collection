import type { DestinationEvaluation, GameCandidate, SelectionEvaluation } from "../contracts";
import { actionButton, element, screenActions, screenIntro } from "../components/app-shell";
import { campaignLedger } from "../components/campaign-ledger";

export function reviewScreen(
  evaluation: SelectionEvaluation,
  destination: DestinationEvaluation,
  games: readonly GameCandidate[],
  back: () => void,
  freeze: () => void | Promise<void>,
): HTMLElement {
  const page = element("div", "screen-stack");
  page.append(screenIntro("Step 4 of 5", "Review the campaign ledger", "This is the exact player-facing plan that will be frozen before the build begins."));
  const layout = element("div", "review-layout");
  const summary = element("section", "card review-summary");
  summary.append(element("h2", undefined, "Frozen inputs"));
  const list = element("dl", "review-list");
  const entries = [
    ["BG1 source", games[0]?.label ?? "Not selected"],
    ["BG2 source", games[1]?.label ?? "Not selected"],
    ["Destination", destination.path],
    ["Choices", `${evaluation.selectedChoiceCount} selected`],
  ];
  entries.forEach(([term, value]) => list.append(element("dt", undefined, term), element("dd", undefined, value)));
  summary.append(list);
  layout.append(summary, campaignLedger(evaluation.plan.phases));
  page.append(layout);
  if (evaluation.findings.length > 0) {
    const notices = element("section", "card plan-notices");
    notices.setAttribute("aria-labelledby", "review-notices-title");
    const heading = element("h2", undefined, "Review notices");
    heading.id = "review-notices-title";
    const list = element("ul");
    evaluation.findings.forEach((finding) => list.append(element("li", undefined, finding.message)));
    notices.append(heading, list);
    page.append(notices);
  }
  page.append(screenActions(back, actionButton("Freeze review and build", freeze)));
  return page;
}
