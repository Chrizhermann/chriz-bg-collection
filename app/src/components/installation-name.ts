import { element } from "./app-shell";
import { flavor } from "../flavor";

export interface NamedInstallation {
  readonly installId: string;
  readonly name: string;
}

/** Advisory inline check; the backend repeats it under a cross-process lock. */
export function installationNameError(
  value: string,
  installations: readonly NamedInstallation[],
  excludingInstallId?: string,
): string | null {
  const name = value.trim();
  if (!name) return "Enter a name for this installation.";
  if ([...name].length > 80) return "Keep the installation name to 80 characters or fewer.";
  if (/[\u0000-\u001f\u007f-\u009f]/u.test(name)) return "Installation names cannot contain control characters.";
  if (name === "." || name === ".." || /[<>:"/\\|?*]/u.test(name)) {
    return 'Installation names cannot contain < > : " / \\ | ? * or consist only of dots.';
  }
  if (installations.some((copy) => copy.installId !== excludingInstallId && copy.name.trim().toLowerCase() === name.toLowerCase())) {
    return `An installation named “${name}” already exists. Choose a different name.`;
  }
  return null;
}

export interface InstallationNameEditorOptions {
  readonly installId: string;
  readonly name: string;
  readonly installations: readonly NamedInstallation[];
  readonly disabled?: boolean;
  readonly onRename: (installId: string, name: string) => void | Promise<void>;
}

let editorSequence = 0;

/** Compact display-only rename control; never changes a path or save profile. */
export function installationNameEditor(options: InstallationNameEditorOptions): HTMLElement {
  const details = element("details", "installation-name-editor");
  const summary = element("summary");
  summary.textContent = "Rename installation";
  const form = element("form", "installation-name-form");
  const id = `installation-name-${++editorSequence}`;
  const label = element("label");
  label.htmlFor = id;
  label.textContent = "Installation name";
  const input = element("input");
  input.id = id;
  input.type = "text";
  input.autocomplete = "off";
  input.value = options.name;
  input.disabled = options.disabled ?? false;
  input.setAttribute("aria-describedby", `${id}-hint ${id}-error`);
  const hint = element("p", "muted");
  hint.id = `${id}-hint`;
  hint.textContent = `Changes the name shown in ${flavor.shortName}, not your game or save folders.`;
  const error = element("p", "error-message");
  error.id = `${id}-error`;
  error.setAttribute("role", "alert");
  error.hidden = true;
  const result = element("p", "success-message");
  result.setAttribute("role", "status");
  result.hidden = true;
  const save = element("button", "button quiet", "Save name");
  save.type = "submit";
  // The form handles both keyboard Enter and pointer submission, exactly once.
  save.disabled = options.disabled ?? false;
  let busy = false;

  const check = (): string | null => {
    const problem = installationNameError(input.value, options.installations, options.installId);
    error.textContent = problem ?? "";
    error.hidden = problem === null;
    input.setAttribute("aria-invalid", String(problem !== null));
    save.disabled = busy || Boolean(options.disabled) || problem !== null;
    return problem;
  };
  input.addEventListener("input", () => {
    result.hidden = true;
    check();
  });
  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    if (busy || options.disabled || check()) return;
    busy = true;
    save.disabled = true;
    input.disabled = true;
    result.hidden = true;
    save.textContent = "Saving name…";
    try {
      const name = input.value.trim();
      await options.onRename(options.installId, name);
      input.value = name;
      result.textContent = `Installation renamed to “${name}”.`;
      result.hidden = false;
    } catch (failure) {
      error.textContent = failure instanceof Error ? failure.message : "The name could not be saved. Try again.";
      error.hidden = false;
    } finally {
      busy = false;
      input.disabled = options.disabled ?? false;
      save.disabled = options.disabled ?? false;
      save.textContent = "Save name";
    }
  });
  form.append(label, input, hint, error, save, result);
  details.append(summary, form);
  return details;
}
