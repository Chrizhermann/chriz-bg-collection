import type { GameCandidate, GameDiscovery } from "../contracts";
import { actionButton, element, screenActions, screenIntro } from "../components/app-shell";

function candidateSelect(
  labelText: string,
  candidates: readonly GameCandidate[],
  selectedId: string,
  browse: () => void | Promise<void>,
): { wrapper: HTMLElement; select: HTMLSelectElement; finding: HTMLElement } {
  const wrapper = element("div", "source-field card");
  const id = `source-${labelText.startsWith("Baldur's Gate II") ? "bg2" : "bg1"}`;
  const label = element("label", undefined, labelText);
  label.htmlFor = id;
  const select = element("select");
  select.id = id;
  candidates.forEach((candidate) => {
    const option = element("option", undefined, candidate.label);
    option.value = candidate.id;
    option.selected = candidate.id === selectedId;
    select.append(option);
  });
  const path = element("p", "path");
  const finding = element("div", "source-finding");
  const browseButton = actionButton("Browse…", browse, "quiet");
  browseButton.setAttribute("aria-label", `Browse for ${labelText}`);
  wrapper.append(label, select, browseButton, path, finding);
  const refresh = (): void => {
    const candidate = candidates.find((entry) => entry.id === select.value) ?? candidates[0];
    path.textContent = candidate?.path ?? "";
    finding.replaceChildren();
    candidate?.findings.forEach((message) => finding.append(element("p", candidate.eligible ? "finding ok" : "finding warning", message)));
  };
  select.addEventListener("change", refresh);
  refresh();
  return { wrapper, select, finding };
}

export function gamesScreen(
  discovery: GameDiscovery,
  selectedBg1Id: string,
  selectedBg2Id: string,
  onSelect: (game: "bg1" | "bg2", id: string) => void,
  onBrowse: (game: "bg1" | "bg2") => void | Promise<void>,
  back: () => void,
  next: () => void | Promise<void>,
): HTMLElement {
  const page = element("div", "screen-stack");
  page.append(screenIntro("", "Find your games", "Choose your original Baldur's Gate games. CEBG will check that they are ready to use."));
  const fields = element("div", "source-grid");
  const bg1 = candidateSelect("Baldur's Gate source", discovery.bg1Candidates, selectedBg1Id, () => onBrowse("bg1"));
  const bg2 = candidateSelect("Baldur's Gate II source", discovery.bg2Candidates, selectedBg2Id, () => onBrowse("bg2"));
  const continueButton = actionButton("Continue", next);
  const refreshEligibility = (): void => {
    const first = discovery.bg1Candidates.find((candidate) => candidate.id === bg1.select.value);
    const second = discovery.bg2Candidates.find((candidate) => candidate.id === bg2.select.value);
    continueButton.disabled = !(first?.eligible && second?.eligible);
  };
  bg1.select.addEventListener("change", () => { onSelect("bg1", bg1.select.value); refreshEligibility(); });
  bg2.select.addEventListener("change", () => { onSelect("bg2", bg2.select.value); refreshEligibility(); });
  refreshEligibility();
  fields.append(bg1.wrapper, bg2.wrapper);
  page.append(fields, screenActions(back, continueButton));
  return page;
}
