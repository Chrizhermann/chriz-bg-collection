import { element } from "./app-shell";

/** Keep the native reason available locally without turning every error into a wall of text. */
export function errorDetails(detail: string, open = false): HTMLDetailsElement {
  const details = element("details", "error-details");
  details.open = open;
  details.append(element("summary", undefined, "Error details"), element("pre", undefined, detail));
  return details;
}
