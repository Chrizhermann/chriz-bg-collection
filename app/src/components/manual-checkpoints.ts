import type { ManualBreakpoint, ManualCheckpointOption, ManualCheckpointStatus } from "../contracts";
import { flavor } from "../flavor";
import { actionButton, element } from "./app-shell";
import { persistDisclosure } from "./disclosure";

/** The game folder a pause belongs to. A standalone collection has one game, so it names none. */
function targetLabel(target: string): string | null {
  return flavor.sources.length > 1 ? target.toUpperCase() : null;
}

function joinFacts(...facts: readonly (string | null)[]): string {
  return facts.filter((fact): fact is string => fact !== null).join(" · ");
}

export function sameBreakpoint(a: ManualBreakpoint, b: ManualBreakpoint): boolean {
  return a.runId === b.runId && a.component === b.component;
}

export function invalidBreakpoints(anchors: readonly ManualBreakpoint[], options: readonly ManualCheckpointOption[]): readonly ManualBreakpoint[] {
  return anchors.filter(anchor => !options.some(option => sameBreakpoint(anchor, option) && option.unavailableReason === null));
}

function boundaryLabel(option: ManualCheckpointOption): string {
  return `${option.modLabel || option.modId}: ${option.componentLabel || `Component ${option.component}`} (${option.component})`;
}

export interface ManualBreakpointEditorModel {
  readonly options: readonly ManualCheckpointOption[];
  readonly selected: readonly ManualBreakpoint[];
  readonly phaseTitles: Readonly<Record<string, string>>;
  readonly disabled?: boolean;
  readonly open?: boolean;
  readonly onOpen: (open: boolean) => void;
  readonly onChange: (anchors: readonly ManualBreakpoint[]) => void;
}

export function manualBreakpointEditor(model: ManualBreakpointEditorModel): HTMLElement {
  const details = element("details", "manual-breakpoints setup-advanced");
  details.open = model.open ?? false;
  const title = element("summary", undefined, `Advanced: Add your own mods${model.selected.length ? ` (${model.selected.length} pauses)` : ""}`);
  persistDisclosure(details, title, model.onOpen);
  details.append(title);
  const body = element("div", "manual-breakpoint-body");
  body.append(element("p", "manual-mod-warning", flavor.text.manualModsWarning));
  body.append(element("p", "muted", "Add pauses after selected components finish. No pauses are added by default. Keep the game and all other installers closed before continuing."));
  const guide = element("details", "manual-mod-guide");
  guide.append(element("summary", undefined, "Manual-mod guide"));
  const steps = element("ol");
  for (const instruction of [
    "Choose a completed component after which your add-on belongs. For example, an add-on that needs EEex belongs after the appropriate EEex component. Check the add-on’s own requirements first.",
    flavor.text.manualPauseFolderStep,
    `Close the manual installer, check changes in ${flavor.shortName}, then explicitly accept the displayed additions. A pause with no additions can continue unchanged.`,
    flavor.text.manualPauseLimitsStep,
  ]) steps.append(element("li", undefined, instruction));
  guide.append(steps);
  body.append(guide);
  const selectedList = element("ol", "manual-breakpoint-list");
  // The engine's selected plan supplies installation order; unresolved anchors stay visible.
  const ordered = [...model.options.filter(option => model.selected.some(anchor => sameBreakpoint(anchor, option))),
    ...model.selected.filter(anchor => !model.options.some(option => sameBreakpoint(anchor, option)))];
  for (const anchor of ordered) {
    const option = model.options.find(candidate => sameBreakpoint(candidate, anchor));
    const item = element("li", "manual-breakpoint-row");
    const copy = element("div");
    copy.append(element("strong", undefined, option ? boundaryLabel(option) : `${anchor.runId}, component ${anchor.component}`));
    copy.append(element("p", "muted", option ? joinFacts(targetLabel(option.target), model.phaseTitles[option.phase] ?? option.phase) : "This component is no longer selected. Remove this pause or restore the component."));
    if (option?.unavailableReason) copy.append(element("p", "unavailable-reason", option.unavailableReason));
    const remove = actionButton("Remove", () => model.onChange(model.selected.filter(candidate => !sameBreakpoint(candidate, anchor))), "quiet");
    remove.setAttribute("aria-label", `Remove pause after ${option ? boundaryLabel(option) : `${anchor.runId} component ${anchor.component}`}`);
    remove.disabled = model.disabled ?? false;
    item.append(copy, remove);
    selectedList.append(item);
  }
  body.append(selectedList);
  if (invalidBreakpoints(model.selected, model.options).length > 0) {
    const invalid = element("p", "unavailable-reason", "A pause no longer matches your selected setup. Correct it before installing.");
    invalid.setAttribute("role", "alert");
    body.append(invalid);
  }
  const searchLabel = element("label", undefined, "Find a selected component");
  const search = element("input");
  search.id = "manual-breakpoint-search";
  search.type = "search";
  search.placeholder = flavor.sources.length > 1 ? "Mod, component, phase or BG1 / BG2" : "Mod, component or phase";
  search.disabled = model.disabled ?? false;
  searchLabel.htmlFor = search.id;
  searchLabel.append(search);
  const choiceLabel = element("label", undefined, "Pause after");
  const choice = element("select");
  choice.id = "manual-breakpoint-component";
  choiceLabel.htmlFor = choice.id;
  choiceLabel.append(choice);
  const reason = element("p", "muted");
  reason.setAttribute("role", "status");
  const add = actionButton("Add pause", () => {
    const option = model.options[Number(choice.value)];
    if (!option || option.unavailableReason || model.disabled || model.selected.some(anchor => sameBreakpoint(anchor, option))) return;
    model.onChange([...model.selected, { runId: option.runId, component: option.component }]);
  }, "quiet");
  const updateChoice = (): void => {
    const option = choice.value === "" ? undefined : model.options[Number(choice.value)];
    add.disabled = !option || !!option.unavailableReason || !!model.disabled;
    reason.textContent = option?.unavailableReason ?? (choice.options.length === 0 ? "No matching components available for another pause." : "The pause begins only after the installer finishes this component.");
  };
  const filter = (): void => {
    const previous = choice.value;
    choice.replaceChildren();
    const query = search.value.trim().toLocaleLowerCase();
    model.options.forEach((option, index) => {
      if (model.selected.some(anchor => sameBreakpoint(anchor, option))) return;
      const label = joinFacts(boundaryLabel(option), targetLabel(option.target), model.phaseTitles[option.phase] ?? option.phase);
      if (!`${label} ${option.runId}`.toLocaleLowerCase().includes(query)) return;
      const entry = element("option", undefined, label);
      entry.value = String(index);
      choice.append(entry);
    });
    if ([...choice.options].some(option => option.value === previous)) choice.value = previous;
    choice.disabled = model.disabled === true || choice.options.length === 0;
    updateChoice();
  };
  search.addEventListener("input", filter);
  choice.addEventListener("change", updateChoice);
  filter();
  body.append(searchLabel, choiceLabel, reason, add);
  details.append(body);
  return details;
}

export interface ManualCheckpointActions {
  readonly status: ManualCheckpointStatus | null;
  readonly busy: boolean;
  readonly error?: string;
  readonly openFolder: () => void;
  readonly reload: () => void;
  readonly check: () => void;
  readonly continueUnchanged: () => void;
  readonly accept: () => void;
}

export function manualCheckpointPanel(actions: ManualCheckpointActions): HTMLElement {
  const panel = element("section", "manual-checkpoint-panel");
  panel.setAttribute("aria-label", "Manual installation checkpoint");
  panel.setAttribute("aria-busy", String(actions.busy));
  const status = actions.status;
  if (status) {
    panel.append(element("p", undefined, `Install your additions in this managed ${targetLabel(status.target) ?? "game"} folder:`), element("p", "path", status.targetPath));
    const boundaries = element("dl", "review-list");
    boundaries.append(element("dt", undefined, "Completed"), element("dd", undefined, boundaryLabel(status.after)),
      element("dt", undefined, "Next"), element("dd", undefined, status.next ? joinFacts(boundaryLabel(status.next), targetLabel(status.next.target)) : "Final verification"));
    panel.append(boundaries, element("p", "muted", `Finish and close your manual installer and game before checking. Keep every WeiDU.log entry intact. You can close ${flavor.shortName} while paused.`));
    panel.append(element("p", "manual-mod-warning", `Before making changes, back up this managed game folder. ${flavor.shortName} does not automatically create a full checkpoint backup.`));
    const folder = actionButton("Open game folder", actions.openFolder, "quiet");
    folder.disabled = actions.busy;
    panel.append(folder);
    if (status.reviewToken) {
      if (status.additions.length > 0) {
        const heading = element("h3", undefined, "Review your manual additions");
        heading.id = "manual-checkpoint-review";
        heading.tabIndex = -1;
        panel.append(heading);
        const list = element("ul", "manual-addition-list");
        for (const addition of status.additions) {
          const item = element("li");
          item.append(element("strong", undefined, addition.description || addition.tp2),
            element("p", "path", `${addition.tp2} · Component ${addition.component} · Language ${addition.language}`));
          list.append(item);
        }
        panel.append(list, element("p", "manual-mod-warning", flavor.text.manualAdditionsAcceptNote));
      } else {
        const unchanged = element("p", undefined, "No manual additions found. Continue with the remaining installation steps.");
        unchanged.id = "manual-checkpoint-review";
        unchanged.tabIndex = -1;
        unchanged.setAttribute("role", "status");
        panel.append(unchanged);
      }
    }
    if (status.diagnostics.length) {
      const details = element("details", "manual-checkpoint-details");
      details.append(element("summary", undefined, "Technical details"));
      const notes = element("ul", "manual-checkpoint-diagnostics");
      status.diagnostics.forEach(note => notes.append(element("li", undefined, note)));
      details.append(notes);
      panel.append(details);
    }
  }
  if (actions.error) {
    const error = element("p", "unavailable-reason", actions.error);
    error.id = "manual-checkpoint-error";
    error.tabIndex = -1;
    error.setAttribute("role", "alert");
    panel.append(error);
  }
  const controls = element("div", "inline-actions");
  if (!status) {
    const refresh = actionButton(actions.busy ? "Loading checkpoint…" : "Reload checkpoint", actions.reload, "quiet");
    refresh.disabled = actions.busy;
    controls.append(refresh);
  } else {
    const check = actionButton(status.reviewToken ? "Check changes again" : "Check changes and continue", actions.check, "quiet");
    const unchanged = actionButton("Continue unchanged", actions.continueUnchanged, "quiet");
    check.disabled = actions.busy;
    unchanged.disabled = actions.busy || status.additions.length > 0;
    controls.append(check);
    if (!status.reviewToken) controls.append(unchanged);
    if (status.reviewToken) {
      const accept = actionButton(status.additions.length ? "Accept additions and continue" : "Continue unchanged", actions.accept);
      accept.disabled = actions.busy;
      controls.append(accept);
    }
  }
  panel.append(controls);
  return panel;
}
