import type { BackendStatus } from "./contracts";

export interface Backend {
  getStatus(): Promise<BackendStatus>;
}

export class FixtureBackend implements Backend {
  getStatus(): Promise<BackendStatus> {
    return Promise.resolve({
      mode: "fixture",
      engineVersion: "0.1.0",
      recipeVersion: null,
    });
  }
}
