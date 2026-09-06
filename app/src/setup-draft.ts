import type { BackendStatus, InputValue, NormalizedSelection } from "./contracts";

export const SETUP_DRAFT_STORAGE_KEY = "cebg.setup-draft";
const MAX_STORED_LENGTH = 262_144;

/** Preferences only. Native inspections and a new review still authorize every install. */
export interface SetupDraft {
  readonly version: 1;
  readonly recipeVersion: string;
  readonly profileId: string | null;
  readonly installationName: string;
  readonly destinationPath: string;
  readonly destinationAutomatic: boolean;
  readonly sourcePaths: { readonly bg1: string | null; readonly bg2: string | null };
  readonly selection: NormalizedSelection;
  readonly createDesktopShortcut: boolean;
}

function record(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function exactKeys(value: Record<string, unknown>, keys: readonly string[]): boolean {
  return Object.keys(value).length === keys.length && keys.every(key => Object.hasOwn(value, key));
}

function text(value: unknown, maxLength = 32_768): value is string {
  return typeof value === "string" && value.length <= maxLength && !/[\u0000-\u001f\u007f]/u.test(value);
}

function identifier(value: string): boolean {
  return /^[a-zA-Z0-9][a-zA-Z0-9_.:-]{0,255}$/u.test(value)
    && !["__proto__", "constructor", "prototype"].includes(value);
}

function inputValue(value: unknown): value is InputValue {
  if (!record(value) || !exactKeys(value, ["kind", "value"])) return false;
  switch (value.kind) {
    case "boolean": return typeof value.value === "boolean";
    case "integer": return typeof value.value === "number" && Number.isSafeInteger(value.value);
    case "choice": return text(value.value, 4096);
    default: return false;
  }
}

function selection(value: unknown): value is NormalizedSelection {
  if (!record(value) || !exactKeys(value, ["platform", "features", "inputs"])
    || value.platform !== "windows" || !record(value.features) || !record(value.inputs)) return false;
  return Object.entries(value.features).every(([id, enabled]) => identifier(id) && typeof enabled === "boolean")
    && Object.entries(value.inputs).every(([id, inputs]) => identifier(id) && record(inputs)
      && Object.entries(inputs).every(([inputId, input]) => identifier(inputId) && inputValue(input)));
}

function validDraft(value: unknown): value is SetupDraft {
  if (!record(value) || !exactKeys(value, ["version", "recipeVersion", "profileId", "installationName",
    "destinationPath", "destinationAutomatic", "sourcePaths", "selection", "createDesktopShortcut"])) return false;
  return value.version === 1
    && text(value.recipeVersion, 256) && value.recipeVersion.length > 0
    && (value.profileId === null || (text(value.profileId, 256) && identifier(value.profileId)))
    && text(value.installationName, 512)
    && text(value.destinationPath)
    && typeof value.destinationAutomatic === "boolean"
    && record(value.sourcePaths) && exactKeys(value.sourcePaths, ["bg1", "bg2"])
    && [value.sourcePaths.bg1, value.sourcePaths.bg2].every(path => path === null || text(path))
    && selection(value.selection)
    && typeof value.createDesktopShortcut === "boolean";
}

export function clearSetupDraft(completedDestination?: string): void {
  try {
    if (completedDestination !== undefined) {
      const stored: unknown = JSON.parse(window.localStorage.getItem(SETUP_DRAFT_STORAGE_KEY) ?? "null");
      const pathKey = (path: string) => path.replace(/\\/gu, "/").replace(/\/+$/u, "").toLowerCase();
      if (!validDraft(stored) || pathKey(stored.destinationPath) !== pathKey(completedDestination)) return;
    }
    window.localStorage.removeItem(SETUP_DRAFT_STORAGE_KEY);
  } catch {
    // An unavailable preferences store never blocks installation or Play.
  }
}

export function loadSetupDraft(status: BackendStatus): { draft: SetupDraft | null; notice: string | null } {
  let stored: string | null;
  try {
    stored = window.localStorage.getItem(SETUP_DRAFT_STORAGE_KEY);
  } catch {
    return { draft: null, notice: null };
  }
  if (stored === null) return { draft: null, notice: null };
  let value: unknown;
  try {
    value = stored.length <= MAX_STORED_LENGTH ? JSON.parse(stored) : null;
  } catch {
    value = null;
  }
  if (!validDraft(value)) {
    clearSetupDraft();
    return { draft: null, notice: "Your saved setup could not be read. Check the installation details and choices before installing." };
  }
  const profileAvailable = value.profileId === (status.selectedProfile ?? null)
    || (value.profileId !== null && status.profiles?.some(profile => profile.id === value.profileId) === true);
  // An available different profile is selected and its returned recipe checked by the caller.
  if (!profileAvailable || (value.profileId === (status.selectedProfile ?? null) && value.recipeVersion !== status.recipeVersion)) {
    clearSetupDraft();
    return { draft: null, notice: "Your saved setup belongs to a different collection version or an unavailable mod setup. Check the current recommended choices before installing." };
  }
  return { draft: value, notice: null };
}

export function saveSetupDraft(draft: SetupDraft): void {
  try {
    // Project each level explicitly so callers cannot accidentally serialize runtime authority.
    const preferences: SetupDraft = {
      version: 1,
      recipeVersion: draft.recipeVersion,
      profileId: draft.profileId,
      installationName: draft.installationName,
      destinationPath: draft.destinationPath,
      destinationAutomatic: draft.destinationAutomatic,
      sourcePaths: { bg1: draft.sourcePaths.bg1, bg2: draft.sourcePaths.bg2 },
      selection: {
        platform: draft.selection.platform,
        features: Object.fromEntries(Object.entries(draft.selection.features)),
        inputs: Object.fromEntries(Object.entries(draft.selection.inputs).map(([id, inputs]) => [id,
          Object.fromEntries(Object.entries(inputs).map(([inputId, input]) => [inputId, copyInput(input)])),
        ])),
      },
      createDesktopShortcut: draft.createDesktopShortcut,
    };
    if (!validDraft(preferences)) return;
    const stored = JSON.stringify(preferences);
    if (stored.length <= MAX_STORED_LENGTH) window.localStorage.setItem(SETUP_DRAFT_STORAGE_KEY, stored);
  } catch {
    // Local preferences are optional and never grant filesystem authority.
  }
}

function copyInput(input: InputValue): InputValue {
  switch (input.kind) {
    case "boolean": return { kind: "boolean", value: input.value };
    case "integer": return { kind: "integer", value: input.value };
    case "choice": return { kind: "choice", value: input.value };
  }
}
