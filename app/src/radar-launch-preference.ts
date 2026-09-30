/** A launch preference only: native validation still decides what can be run. */
export class RadarLaunchPreferences {
  readonly #choices = new Map<string, boolean>();

  get(installId: string): boolean {
    if (!this.#choices.has(installId)) {
      let enabled = false;
      try {
        enabled = window.localStorage.getItem(`cebg.start-radar.${installId}`) === "true";
      } catch {
        // An unavailable preference store leaves Radar opt-in.
      }
      this.#choices.set(installId, enabled);
    }
    return this.#choices.get(installId) === true;
  }

  set(installId: string, enabled: boolean): void {
    this.#choices.set(installId, enabled);
    try {
      window.localStorage.setItem(`cebg.start-radar.${installId}`, String(enabled));
    } catch {
      // Keep the explicit choice for this app session even if storage is denied.
    }
  }
}
