import type { GameCandidate, GameDiscovery } from "../contracts";
import { actionButton, element, screenActions, screenIntro } from "../components/app-shell";
import { flavor, type SourceSlot } from "../flavor";

function candidateSelect(
  slot: SourceSlot,
  labelText: string,
  candidates: readonly GameCandidate[],
  selectedId: string,
  browse: () => void | Promise<void>,
): { wrapper: HTMLElement; select: HTMLSelectElement; finding: HTMLElement } {
  const wrapper = element("div", "source-field card");
  const id = `source-${slot}`;
  const label = element("label", undefined, labelText);
  label.htmlFor = id;
  const select = element("select");
  select.id = id;
  if (!candidates.some(candidate => candidate.id === selectedId)) {
    const placeholder = element("option", undefined, "Choose source");
    placeholder.value = "";
    placeholder.selected = true;
    select.append(placeholder);
  }
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
    const candidate = candidates.find((entry) => entry.id === select.value);
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
  page.append(screenIntro("", flavor.sources.length === 1 ? "Find your game" : "Find your games", flavor.text.gamesIntro));
  const fields = element("div", "source-grid");
  const selectedIds: Record<SourceSlot, string> = { bg1: selectedBg1Id, bg2: selectedBg2Id };
  const candidatesFor = (slot: SourceSlot): readonly GameCandidate[] => slot === "bg1" ? discovery.bg1Candidates : discovery.bg2Candidates;
  const sources = flavor.sources.map(source => ({
    slot: source.slot,
    field: candidateSelect(source.slot, `${source.name} source`, candidatesFor(source.slot), selectedIds[source.slot], () => onBrowse(source.slot)),
  }));
  const continueButton = actionButton("Continue", next);
  const refreshEligibility = (): void => {
    continueButton.disabled = !sources.every(({ slot, field }) =>
      candidatesFor(slot).find((candidate) => candidate.id === field.select.value)?.eligible);
  };
  for (const { slot, field } of sources) {
    field.select.addEventListener("change", () => { onSelect(slot, field.select.value); refreshEligibility(); });
  }
  refreshEligibility();
  fields.append(...sources.map(({ field }) => field.wrapper));
  page.append(fields, screenActions(back, continueButton));
  return page;
}
