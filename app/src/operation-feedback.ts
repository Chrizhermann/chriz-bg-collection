import type { OperationProgress } from "./contracts";

export interface OperationFeedback {
  readonly title: string;
  readonly detail: string;
  readonly state: "working" | "success" | "error" | "notice";
  readonly installId?: string;
  readonly progress?: OperationProgress;
}

const stages: Readonly<Record<string, string>> = {
  checking: "Checking the installation…", downloading: "Preparing update downloads…",
  "backup-game": "Backing up your game…", "backup-saves": "Backing up saved games…",
  "backup-files": "Backing up affected files…", applying: "Applying the update…",
  verifying: "Verifying the result…", restoring: "Restoring the backup…",
  finishing: "Finishing up…", restarting: "Waiting for CEBG to restart…",
};
export function progressLabel(progress?: OperationProgress): string {
  return progress ? stages[progress.stage] ?? "Working…" : "Preparing…";
}

/** Unknown/malformed counts never invent a percentage or alter an operation. */
export function progressPercent(progress?: OperationProgress): number | null {
  if (!progress?.completed || !progress.total || !/^\d{1,20}$/.test(progress.completed) || !/^\d{1,20}$/.test(progress.total)) return null;
  const done = BigInt(progress.completed), total = BigInt(progress.total);
  return total > 0n && done <= total ? Number(done * 100n / total) : null;
}
export function decodeOperationProgress(value: unknown): OperationProgress | null {
  if (!value || typeof value !== "object") return null;
  const p = value as Record<string, unknown>;
  if (typeof p.stage !== "string" || !Object.hasOwn(stages, p.stage)) return null;
  return { stage: p.stage, completed: typeof p.completed === "string" ? p.completed : null,
    total: typeof p.total === "string" ? p.total : null, unit: p.unit === "bytes" ? "bytes" : "items" };
}

const appUpdateKey = "cebg.pending-app-update.v1";
// Local preference only: no filesystem authority, and never inferred from an online check.
export function rememberAppUpdate(from: string, to: string): void {
  try { localStorage.setItem(appUpdateKey, JSON.stringify({ from, to, at: Date.now() })); } catch { /* optional UI state */ }
}
export function clearAppUpdate(): void {
  try { localStorage.removeItem(appUpdateKey); } catch { /* optional UI state */ }
}
export function completedAppUpdate(currentVersion: string): OperationFeedback | null {
  try {
    const pending = JSON.parse(localStorage.getItem(appUpdateKey) ?? "null");
    if (!pending || typeof pending.to !== "string" || typeof pending.from !== "string" ||
      typeof pending.at !== "number" || Date.now() - pending.at > 30 * 86400000 || pending.at > Date.now()) return null;
    if (!pending.from.trim() || !pending.to.trim() || pending.from === pending.to || pending.to !== currentVersion) return null;
    return { title: "CEBG update complete", detail: `You’re running CEBG ${currentVersion}. Your game installations are unchanged.`, state: "success" };
  } catch { return null; }
}
