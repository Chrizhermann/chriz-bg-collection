import type { FeatureControl, SelectionEvaluation } from "./contracts";

export type ChoiceControl = FeatureControl;

export interface ExclusiveChoiceGroup {
  readonly id: string;
  readonly label: string;
  readonly category: string;
  readonly controls: readonly ChoiceControl[];
  readonly allowNone: boolean;
}

export interface CommonBundle {
  readonly id: string;
  readonly title: string;
  readonly description: string;
  readonly ids: readonly string[];
}

const originalConversions = new Set([
  ...[110, 190, 192, 193, 194, 195, 196, 198, 220, 221, 222, 223].map(n => `feature:chriz-bg-modpack:component-${n}`),
  ...[1, 2, 3, 4].map(n => `feature:xan:component-${n}`),
  ...[4080, 4090, 4130, 4131, 4132, 4133].map(n => `feature:cdtweaks:component-${n}`),
  "feature:yeslicknpc:component-0", "feature:yeslicknpc:component-1",
]);

export function commonBundles(evaluation: SelectionEvaluation): CommonBundle[] {
  const controls = evaluation.view.controls;
  const existing = new Set(controls.map(c => c.id));
  const bundles: CommonBundle[] = [
    { id: "original-companions", title: "Original companion classes & kits", description: "Use the games’ original classes and mod companions’ author defaults. Keep portraits, voices, spellbook tweaks and unrelated fixes. This changes a new installation, not an existing save.", ids: controls.filter(c => originalConversions.has(c.id) || c.id.startsWith("feature:artisanskitpack-npc:component-")).map(c => c.id) },
    { id: "randomiser", title: "Item Randomiser", description: "Randomize item locations. Turning this off also removes its options.", ids: ["mod:randomiser"] },
    { id: "artisan", title: "Artisan’s Kitpack", description: "Class and kit overhauls, companion kit assignments, and associated tweaks. All three parts switch together.", ids: ["mod:artisanskitpack", "mod:artisanskitpack-npc", "mod:artisanskitpack-tweak"] },
    { id: "bardic", title: "Artisan’s Bardic Wonders", description: "Bard kits, abilities and related patches. Dependent companion choices follow automatically.", ids: existing.has("mod:bardicwonders") ? ["mod:bardicwonders"] : controls.filter(c => c.id.startsWith("feature:bardicwonders:")).map(c => c.id) },
    { id: "spell-revisions", title: "Spell Revisions", description: "Revised spells and their compatibility fixes. Turning this off can make previously incompatible choices available.", ids: ["mod:spell-rev"] },
    { id: "sod-remix", title: "SoD Remix", description: "Include or exclude the curated SoD Remix bundle. The bundle includes an optional in-game prompt to skip the SoD story; this switch does not skip it automatically.", ids: ["mod:chriz-sod-remix"] },
  ];
  return bundles.map(b => ({ ...b, ids: b.ids.filter(id => existing.has(id)) })).filter(b => b.ids.length > 0);
}

export function bundleSelected(e: SelectionEvaluation, bundle: CommonBundle): boolean {
  if (bundle.id === "original-companions") {
    return bundle.ids.filter(id => id !== "feature:yeslicknpc:component-0").every(id => !e.normalizedSelection.features[id]);
  }
  // Use requested state: temporarily unavailable children should not make the
  // parent switch appear off, or forget the user's preferences.
  return bundle.ids.some(id => e.normalizedSelection.features[id]);
}

export function bundleChanges(e: SelectionEvaluation, bundle: CommonBundle, selected: boolean, remembered?: Readonly<Record<string, boolean>>): Record<string, boolean> {
  const controls = new Map(e.view.controls.map(c => [c.id, c]));
  const restoring = bundle.id === "original-companions" ? !selected : selected;
  const changes = Object.fromEntries(bundle.ids.filter(id => controls.get(id)?.decision !== "mandatory").map(id => [id,
    restoring ? remembered?.[id] ?? (controls.get(id)?.decision === "default") : false,
  ]));
  if (bundle.id === "original-companions" && selected) {
    const regular = "feature:yeslicknpc:component-0";
    const changed = "feature:yeslicknpc:component-1";
    if (regular in changes) changes[regular] = Boolean(e.normalizedSelection.features[regular] || e.normalizedSelection.features[changed]);
  }
  return changes;
}

function conflictsWith(a: FeatureControl, b: FeatureControl): boolean {
  return Boolean(a.conflicts?.includes(b.id) || b.conflicts?.includes(a.id));
}

function hasSymmetricConflict(a: FeatureControl, b: FeatureControl): boolean {
  return Boolean(a.conflicts?.includes(b.id) && b.conflicts?.includes(a.id));
}

/** Treat authored choice metadata as a display hint, not as authority. */
export function exclusiveChoiceGroups(e: SelectionEvaluation): ExclusiveChoiceGroup[] {
  const candidates = new Map<string, ChoiceControl[]>();
  for (const control of e.view.controls as readonly ChoiceControl[]) {
    if (!control.choiceGroup || !control.groupLabel) continue;
    const controls = candidates.get(control.choiceGroup) ?? [];
    controls.push(control);
    candidates.set(control.choiceGroup, controls);
  }

  const groups: ExclusiveChoiceGroup[] = [];
  for (const [id, controls] of candidates) {
    const first = controls[0]!;
    const hasMandatory = controls.some(control => control.decision === "mandatory");
    const availabilityKnown = controls.every(control => typeof control.choiceAvailable === "boolean");
    const sameScope = controls.every(control =>
      control.category === first.category
      && control.parent === first.parent
      && control.groupLabel === first.groupLabel
    );
    const pairwiseAlternatives = controls.every((control, index) =>
      controls.slice(index + 1).every(peer => hasSymmetricConflict(control, peer))
    );
    if (controls.length < 2 || hasMandatory || !availabilityKnown || !sameScope || !pairwiseAlternatives) continue;
    groups.push({
      id,
      label: first.groupLabel!,
      category: first.category,
      controls,
      allowNone: true,
    });
  }
  return groups;
}

export function exclusiveChoiceChanges(group: ExclusiveChoiceGroup, selectedId: string): Record<string, boolean> {
  const changes: Record<string, boolean> = {};
  for (const control of group.controls) {
    if (control.decision !== "mandatory") changes[control.id] = control.id === selectedId;
  }
  return changes;
}

export function bulkCategoryChanges(e: SelectionEvaluation, category: string, selected: boolean): Record<string, boolean> {
  const controls = e.view.controls;
  const candidates = controls.filter(c => c.category === category && c.interactive && c.decision !== "mandatory");
  if (!selected) return Object.fromEntries(candidates.map(c => [c.id, false]));
  const retained = controls.filter(c => c.selected);
  const changes: Record<string, boolean> = {};
  // Existing choices win. Among otherwise equal alternatives, authored order
  // wins; never request both then let the resolver silently omit both.
  for (const candidate of candidates) {
    if (retained.some(c => c.id !== candidate.id && conflictsWith(candidate, c))) continue;
    changes[candidate.id] = true;
    retained.push(candidate);
  }
  return changes;
}
