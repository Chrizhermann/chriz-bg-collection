import type { FeatureControl, InputValue } from "../contracts";
import { element } from "./app-shell";
import "./feature-inputs.css";

export type InputChange = (featureId: string, inputId: string, value: InputValue, focusId: string) => void | Promise<void>;

/** Render the engine's typed inputs without changing their defaults or limits. */
export function featureInputs(
  feature: FeatureControl,
  change?: InputChange,
  validity: (id: string, valid: boolean) => void = () => {},
): HTMLElement {
  const container = element("div", "feature-inputs");
  for (const input of feature.inputs) {
    const { spec } = input;
    const id = `input-${feature.id}-${spec.id}`;
    const row = element("div", "feature-input");
    const title = feature.id === "feature:randomiser:component-510" && spec.id === "chance-percent"
      ? "Chance to leave an item in place (%)"
      : spec.id.replaceAll("-", " ").replace(/^./, letter => letter.toUpperCase());
    const label = element("label", undefined, title);
    label.htmlFor = id;
    const field = spec.kind === "choice" ? element("select") : element("input");
    field.id = id;
    field.disabled = !input.interactive || !change;
    const hint = element("p", "feature-input-hint", spec.kind === "integer"
      ? `Choose a whole number from ${spec.min} to ${spec.max}. Default: ${spec.default}.`
      : "Used when this option is included.");
    hint.id = `${id}-hint`;
    const error = element("p", "feature-input-error");
    error.id = `${id}-error`;
    error.setAttribute("role", "alert");
    error.hidden = true;
    field.setAttribute("aria-describedby", `${hint.id} ${error.id}`);
    if (spec.kind === "choice" && field instanceof HTMLSelectElement) {
      for (const item of spec.options) {
        const option = element("option", undefined, item.title);
        option.value = item.id;
        field.append(option);
      }
      field.value = String(input.value.value);
    } else if (field instanceof HTMLInputElement) {
      if (spec.kind === "boolean") {
        field.type = "checkbox";
        field.checked = input.value.value === true;
      } else if (spec.kind === "integer") {
        field.type = "number";
        field.required = true;
        field.min = String(spec.min);
        field.max = String(spec.max);
        field.step = "1";
        field.value = String(input.value.value);
      }
    }
    const read = (): InputValue | null => {
      if (spec.kind === "boolean") return { kind: "boolean", value: (field as HTMLInputElement).checked };
      if (spec.kind === "choice") return spec.options.some(option => option.id === field.value)
        ? { kind: "choice", value: field.value } : null;
      const value = Number(field.value);
      return field.value.trim() !== "" && Number.isSafeInteger(value) && value >= spec.min && value <= spec.max
        ? { kind: "integer", value } : null;
    };
    const validate = (): InputValue | null => {
      const value = read();
      const message = value !== null ? "" : spec.kind === "integer"
        ? `Enter a whole number from ${spec.min} to ${spec.max}.` : "Choose one of the listed options.";
      field.setCustomValidity(message);
      field.setAttribute("aria-invalid", String(value === null));
      error.textContent = message;
      error.hidden = value !== null;
      validity(id, value !== null);
      return value;
    };
    field.addEventListener("input", validate);
    field.addEventListener("change", () => {
      if (field.disabled) return;
      const value = validate();
      if (value !== null) void change?.(feature.id, spec.id, value, id);
    });
    row.append(label, field, hint, error);
    container.append(row);
  }
  return container;
}
