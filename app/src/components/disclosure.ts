/** Persist activation before an asynchronous progress render can replace the node. */
export function persistDisclosure(
  details: HTMLDetailsElement,
  summary: HTMLElement,
  onChange: (open: boolean) => void,
): void {
  summary.addEventListener("click", (event) => {
    event.preventDefault();
    details.open = !details.open;
    onChange(details.open);
  });
  details.addEventListener("toggle", () => {
    // Native toggle events are queued: a removed render must not overwrite its successor.
    if (details.isConnected) onChange(details.open);
  });
}
