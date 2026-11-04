import { element } from "./app-shell";

export interface NamedInstallation {
  readonly installId: string;
  readonly name: string;
}

/** The name CEBG uses when nothing but spaces or dots is entered. */
export const DEFAULT_INSTALLATION_NAME = "Chriz Easy BG";
const MAX_INSTALLATION_NAME_CHARACTERS = 80;

/**
 * Mirrors the engine's sanitizing, which never refuses a name: edge spaces and control
 * characters are dropped, each of < > : " / \ | ? * becomes "-", and the name is cut to 80
 * characters. Null when nothing but spaces or dots is left.
 */
export function sanitizeInstallationName(value: string): string | null {
  const cleaned = [...value]
    .filter((character) => !/^[\u0000-\u001f\u007f-\u009f]$/u.test(character))
    .map((character) => (/^[<>:"/\\|?*]$/u.test(character) ? "-" : character))
    .join("")
    .trim();
  const name = [...cleaned].slice(0, MAX_INSTALLATION_NAME_CHARACTERS).join("").trimEnd();
  return /^\.*$/u.test(name) ? null : name;
}

function numberedName(base: string, number: number): string {
  if (number === 1) return base;
  const suffix = ` (${number})`;
  return `${[...base].slice(0, MAX_INSTALLATION_NAME_CHARACTERS - suffix.length).join("").trimEnd()}${suffix}`;
}

/**
 * The name CEBG will save: sanitized, the default name for an empty one, and numbered
 * " (2)", " (3)", … when another installation has it. Advisory; the backend repeats it under a
 * cross-process lock.
 */
export function savedInstallationName(
  value: string,
  installations: readonly NamedInstallation[],
  excludingInstallId?: string,
): string {
  const base = sanitizeInstallationName(value) ?? DEFAULT_INSTALLATION_NAME;
  const taken = new Set(installations
    .filter((copy) => copy.installId !== excludingInstallId)
    .map((copy) => copy.name.trim().toLowerCase()));
  for (let number = 1; number <= 10_000; number += 1) {
    const candidate = numberedName(base, number);
    if (!taken.has(candidate.toLowerCase())) return candidate;
  }
  return base;
}

/** A calm note when the saved name differs from what was entered; null when it is the same. */
export function installationNameNote(
  value: string,
  installations: readonly NamedInstallation[],
  excludingInstallId?: string,
): string | null {
  const saved = savedInstallationName(value, installations, excludingInstallId);
  return saved === value.trim() ? null : `Will be saved as “${saved}”.`;
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
  input.setAttribute("aria-describedby", `${id}-hint ${id}-note ${id}-error`);
  const hint = element("p", "muted");
  hint.id = `${id}-hint`;
  hint.textContent = "Changes the name shown in CEBG, not your game or save folders.";
  const note = element("p", "field-help");
  note.id = `${id}-note`;
  note.setAttribute("aria-live", "polite");
  note.hidden = true;
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

  // Every name can be saved: one Windows cannot use as a folder name, or one another
  // installation has, is shown as the name it will be saved as.
  const describe = (): void => {
    const text = installationNameNote(input.value, options.installations, options.installId);
    note.textContent = text ?? "";
    note.hidden = text === null;
    error.hidden = true;
  };
  input.addEventListener("input", () => {
    result.hidden = true;
    describe();
  });
  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    if (busy || options.disabled) return;
    busy = true;
    save.disabled = true;
    input.disabled = true;
    result.hidden = true;
    save.textContent = "Saving name…";
    try {
      const name = savedInstallationName(input.value, options.installations, options.installId);
      await options.onRename(options.installId, name);
      input.value = name;
      note.hidden = true;
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
  form.append(label, input, hint, note, error, save, result);
  details.append(summary, form);
  return details;
}
