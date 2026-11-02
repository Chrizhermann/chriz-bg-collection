import type { SelectionEvaluation } from "../contracts";
import { exclusiveChoiceChanges, type ExclusiveChoiceGroup } from "../customization";
import { element } from "./app-shell";

export const GAME_INTERFACE_GROUP = "game-interface";

/** New-install control only. The recipe, not this view, owns compatibility. */
export function gameInterfaceSelector(
  evaluation: SelectionEvaluation,
  group: ExclusiveChoiceGroup,
  change: (changes: Record<string, boolean>, focusId: string) => void | Promise<void>,
): HTMLElement {
  const section = element("section", "game-interface-choice");
  section.setAttribute("aria-labelledby", "game-interface-label");
  const label = element("label", "exclusive-choice-label", "Game interface");
  label.id = "game-interface-label";
  label.htmlFor = "game-interface";
  const select = element("select", "exclusive-choice-select");
  select.id = "game-interface";
  select.setAttribute("aria-describedby", "game-interface-description game-interface-scope game-interface-effects");
  const original = element("option", undefined, "Default interface (Recommended)");
  original.value = "";
  select.append(original);
  for (const control of group.controls) {
    const available = control.selected || control.choiceAvailable === true;
    const option = element("option", undefined, `${control.title}${available ? "" : " (Unavailable)"}`);
    option.value = control.id;
    option.disabled = !available;
    select.append(option);
  }
  select.value = group.controls.find(control => control.selected)?.id ?? "";
  const description = element("p", "exclusive-choice-description");
  description.id = "game-interface-description";
  const scope = element("p", "category-hint", "New installations only; existing games stay unchanged.");
  scope.id = "game-interface-scope";
  const effects = element("div", "game-interface-effects");
  effects.id = "game-interface-effects";
  effects.setAttribute("role", "status");
  const updateDescription = () => {
    const selected = group.controls.find(control => control.id === select.value);
    description.textContent = selected?.description ?? "Keep the familiar game interface with your selected quality-of-life additions.";
    effects.replaceChildren();
    // Requested preferences remain intact: the engine suppresses incompatible
    // add-ons and restores them when their requirements are satisfied again.
    const omitted = selected ? evaluation.view.controls.filter(control =>
      evaluation.normalizedSelection.features[control.id]
      && control.conflicts?.includes(selected.id)
      && control.choiceGroup !== GAME_INTERFACE_GROUP
    ) : [];
    if (omitted.length) {
      effects.append(element("p", undefined, "Also leaves out:"));
      const list = element("ul");
      for (const control of omitted) list.append(element("li", undefined, control.title));
      effects.append(list, element("p", "category-hint", "Your choices are remembered if you switch back."));
    }
  };
  updateDescription();
  select.addEventListener("change", () => {
    updateDescription();
    void change(exclusiveChoiceChanges(group, select.value), select.id);
  });
  const unavailable = group.controls.filter(control => !control.selected && control.choiceAvailable !== true);
  if (unavailable.length) {
    const reasons = element("details", "game-interface-unavailable");
    reasons.append(element("summary", undefined, "Why are some interfaces unavailable?"));
    const list = element("ul");
    for (const control of unavailable) list.append(element("li", undefined, `${control.title}: ${control.unavailableReason ?? "Not available with this setup."}`));
    reasons.append(list);
    section.append(reasons);
  }
  section.prepend(label, select, description, scope, effects);
  return section;
}
