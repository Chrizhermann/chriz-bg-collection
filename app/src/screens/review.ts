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
  page.append(screenIntro("", "Ready to install", "Check your games and selected setup."));
  const layout = element("div", "review-layout");
  const summary = element("section", "card review-summary");
  summary.append(element("h2", undefined, "Your installation"));
  const list = element("dl", "review-list");
  const entries = [
    ["BG1 source", games[0]?.label ?? "Not selected"],
    ["BG2 source", games[1]?.label ?? "Not selected"],
    ["Install location", destination.path],
    ["Choices", `${evaluation.selectedChoiceCount} selected`],
  ];
  entries.forEach(([term, value]) => list.append(element("dt", undefined, term), element("dd", undefined, value)));
  summary.append(list);
  layout.append(summary, campaignLedger(evaluation.plan.phases));
  page.append(layout);
  if (evaluation.findings.length > 0) {
    const notices = element("section", "card plan-notices");
    notices.setAttribute("aria-labelledby", "review-notices-title");
    const heading = element("h2", undefined, "Setup notes");
    heading.id = "review-notices-title";
    const list = element("ul");
    evaluation.findings.forEach((finding) => list.append(element("li", undefined, finding.message)));
    notices.append(heading, list);
    page.append(notices);
  }
  page.append(screenActions(back, actionButton("Install Chriz Easy BG", freeze)));
  return page;
}
