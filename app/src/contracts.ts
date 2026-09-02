export interface BackendStatus {
  readonly mode: "fixture" | "native";
  readonly engineVersion: string;
  readonly recipeVersion: string | null;
}
