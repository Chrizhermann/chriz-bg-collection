import type { BackendStatus } from "./contracts";
import { FLAVORS, type Flavor, type SourceDefinition, type SourceSlot } from "./flavors";

export type { Flavor, FlavorFeatures, FlavorId, SourceDefinition, SourceSlot } from "./flavors";

/** The build's flavor: `vite --mode iwd` builds Chriz Easy IWD, every other mode Chriz Easy BG. */
export const flavor: Flavor = FLAVORS[import.meta.env.MODE === "iwd" ? "iwd" : "bg"];

/** Source slots the build installs from, in display order. */
export const sourceSlots: readonly SourceSlot[] = flavor.sources.map(source => source.slot);

export function sourceFor(slot: SourceSlot): SourceDefinition | undefined {
  return flavor.sources.find(source => source.slot === slot);
}

/**
 * Why the backend's collection cannot serve this build, or null when it matches. The BG build
 * expects no collection; a standalone build expects its own flavor, game slots and roles.
 */
export function collectionMismatch(status: Pick<BackendStatus, "collection">): string | null {
  const collection = status.collection;
  if (flavor.id === "bg") {
    return collection === undefined || collection === null
      ? null
      : `This build is for the Baldur's Gate collection, but the backend serves the ${collection.flavor} collection.`;
  }
  if (collection === undefined || collection === null) {
    return `This build is for the ${flavor.id} collection, but the backend serves the Baldur's Gate collection.`;
  }
  if (collection.flavor !== flavor.id) {
    return `This build is for the ${flavor.id} collection, but the backend serves the ${collection.flavor} collection.`;
  }
  const expected = flavor.sources.map(source => `${source.slot}:${source.role}`).join(",");
  const actual = collection.sources.map(source => `${source.slot}:${source.role}`).join(",");
  if (expected !== actual) {
    return `This build expects the game sources ${expected}, but the backend reports ${actual || "none"}.`;
  }
  return null;
}
