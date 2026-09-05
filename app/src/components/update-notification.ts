import type { UpdateSummary } from "../contracts";
import { element } from "./app-shell";

export interface UpdateAvailability {
  readonly application: boolean;
  readonly collection: boolean;
  readonly radar: boolean;
}

export function getUpdateAvailability(updates: UpdateSummary): UpdateAvailability {
  const application = updates.application.state === "available" && updates.application.availableVersion !== null;
  const collection = (updates.recipe.state === "available" && updates.recipe.availableVersion !== null)
    || (updates.recipe.state === "requires-app" && application)
    || updates.managedCopies.some((copy) => copy.state === "update-available");
  const radar = updates.radar?.state === "available"
    && updates.radar.currentVersion !== null
    && updates.radar.availableVersion !== null;
  return { application, collection, radar };
}

function availabilityText(availability: UpdateAvailability): string {
  const names = [
    availability.application ? "CEBG app" : null,
    availability.collection ? "collection" : null,
    availability.radar ? "BG Radar Overlay" : null,
  ].filter((name): name is string => name !== null);
  if (names.length === 0) return "Check CEBG app, collection, and BG Radar Overlay versions.";
  if (names.length === 1) return `New ${names[0]} update available.`;
  const joined = names.length === 2 ? names.join(" and ") : `${names[0]}, ${names[1]}, and ${names[2]}`;
  return `New updates available: ${joined}.`;
}

export function updateUpdatesControl(button: HTMLButtonElement, updates: UpdateSummary): void {
  const availability = getUpdateAvailability(updates);
  const available = availability.application || availability.collection || availability.radar;
  const container = button.closest<HTMLElement>(".update-nav-item") ?? button.parentElement;
  if (container === null) return;
  container.querySelectorAll(".update-badge, .update-tooltip").forEach((node) => node.remove());
  button.dataset.updateAvailable = String(available);
  button.removeAttribute("title");
  const tooltip = element("span", "update-tooltip", availabilityText(availability));
  tooltip.id = "updates-availability-tooltip";
  tooltip.setAttribute("role", "tooltip");
  container.append(tooltip);
  button.setAttribute("aria-describedby", tooltip.id);
  if (button.dataset.updateTooltipBound !== "true") {
    button.dataset.updateTooltipBound = "true";
    button.addEventListener("keydown", (event) => {
      if (event.key === "Escape") container.querySelector<HTMLElement>(".update-tooltip")?.setAttribute("hidden", "");
    });
    button.addEventListener("focus", () => container.querySelector<HTMLElement>(".update-tooltip")?.removeAttribute("hidden"));
    container.addEventListener("mouseenter", () => container.querySelector<HTMLElement>(".update-tooltip")?.removeAttribute("hidden"));
  }
  if (available) {
    const badge = element("span", "update-badge", "New");
    badge.setAttribute("aria-hidden", "true");
    button.append(badge);
  }
}
