// @vitest-environment jsdom

import { getByLabelText, getByRole, getByText, queryByText } from "@testing-library/dom";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it } from "vitest";

import { BackendCommandError, FixtureBackend, NativeBackend, type EventChannelFactory, type InvokeCommand } from "../src/backend";
import type { RunEventEnvelope } from "../src/contracts";
import { mountApp } from "../src/app";
import { technicalLog } from "../src/components/technical-log";

describe("guided collection wizard", () => {
  afterEach(() => document.body.replaceChildren());

  it("walks all seven wizard screens, preserves Back, and freezes Review before Build", async () => {
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, new FixtureBackend());

    expect(getByRole(root, "heading", { level: 1, name: "Welcome" })).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Begin setup" }));
    expect(getByRole(root, "heading", { level: 1, name: "Find your games" })).toBeTruthy();

    const bg1 = getByLabelText(root, "Baldur's Gate source") as HTMLSelectElement;
    const bg2 = getByLabelText(root, "Baldur's Gate II source") as HTMLSelectElement;
    const originalBg2 = bg2.value;
    await user.selectOptions(bg1, "bg1-modified");
    expect(bg2.value).toBe(originalBg2);
    expect(getByText(root, "Files differ from a clean store installation.")).toBeTruthy();
    await user.selectOptions(bg1, "bg1-fresh");
    await user.click(getByRole(root, "button", { name: "Continue" }));

    expect(getByRole(root, "heading", { level: 1, name: "Choose a destination" })).toBeTruthy();
    expect(getByText(root, "Safe separate destination")).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Continue" }));

    expect(getByRole(root, "heading", { level: 1, name: "Shape your campaign" })).toBeTruthy();
    const mandatory = getByRole(root, "checkbox", { name: /Curated foundation/ }) as HTMLInputElement;
    const recommended = getByRole(root, "checkbox", { name: /Recommended rules balance/ }) as HTMLInputElement;
    const optional = getByRole(root, "checkbox", { name: /Companion conversations/ }) as HTMLInputElement;
    const blocked = getByRole(root, "checkbox", { name: /Experimental quest restoration/ });
    expect(mandatory.checked).toBe(true);
    expect(mandatory.getAttribute("aria-disabled")).toBe("true");
    expect(recommended.checked).toBe(true);
    expect(recommended.hasAttribute("aria-disabled")).toBe(false);
    expect(optional.checked).toBe(false);
    expect(optional.hasAttribute("aria-disabled")).toBe(false);
    expect(blocked.getAttribute("aria-disabled")).toBe("true");
    expect(getByText(root, "Deferred until its installer can be reproduced safely.")).toBeTruthy();
    expect(getByText(root, "Experimental quest restoration remains visible but is omitted.")).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Continue" }));

    expect(getByRole(root, "heading", { level: 1, name: "Review the campaign ledger" })).toBeTruthy();
    expect(root.querySelectorAll("[data-ledger-phase]")).toHaveLength(5);
    expect(getByText(root, "Experimental quest restoration remains visible but is omitted.")).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Back" }));
    expect(getByRole(root, "heading", { level: 1, name: "Shape your campaign" })).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Continue" }));
    await user.click(getByRole(root, "button", { name: "Freeze review and build" }));

    expect(getByRole(root, "heading", { level: 1, name: "Build your campaign" })).toBeTruthy();
    expect(getByText(root, "Manual archive needed")).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "I added the archive" }));
    expect(getByText(root, "Your attention is needed")).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Continue build" }));
    expect(getByText(root, "A fixture step failed")).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Export diagnostics" }));
    expect(getByText(root, /Diagnostics exported to/)).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Retry failed step" }));
    expect(getByText(root, "Build in progress")).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Finish fixture build" }));

    expect(getByRole(root, "heading", { level: 1, name: "Campaign complete" })).toBeTruthy();
    expect(getByText(root, "Immutable install receipt")).toBeTruthy();
  });

  it("surfaces every non-fresh source reason", async () => {
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    const handle = await mountApp(root, new FixtureBackend());
    await handle.navigate("games");

    const cases = [
      ["bg1-modified", "Files differ from a clean store installation."],
      ["bg1-old", "The installed game version is not supported."],
      ["bg1-nosod", "Siege of Dragonspear is required for this EET recipe."],
    ] as const;
    for (const [id, message] of cases) {
      await user.selectOptions(getByLabelText(root, "Baldur's Gate source"), id);
      expect(getByText(root, message)).toBeTruthy();
    }
    await user.selectOptions(getByLabelText(root, "Baldur's Gate II source"), "bg2-store");
    expect(getByText(root, "This storefront layout has not been verified yet.")).toBeTruthy();
  });

  it("bounds the selectable technical tail and allows auto-scroll to be paused", async () => {
    let logState = { paused: false, open: false };
    const log = technicalLog(
      Array.from({ length: 250 }, (_, index) => `line-${index}`),
      logState,
      (state) => { logState = state; },
    );
    document.body.append(log);
    const output = log.querySelector("pre");
    expect(output?.tabIndex).toBe(0);
    expect(output?.textContent?.split("\n")).toHaveLength(200);
    expect(output?.textContent).not.toContain("line-0\n");
    const pause = getByRole(log, "button", { name: "Pause auto-scroll" });
    await userEvent.setup().click(pause);
    expect(pause.getAttribute("aria-pressed")).toBe("true");
    expect(pause.textContent).toBe("Resume auto-scroll");
    expect(logState.paused).toBe(true);
    const restored = technicalLog(["later line"], logState);
    expect(getByRole(restored, "button", { name: "Resume auto-scroll" })).toBeTruthy();
  });

  it("shows returning managed installations and the separate Updates screen", async () => {
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, new FixtureBackend());

    await user.click(getByRole(root, "button", { name: "Campaigns" }));
    expect(getByRole(root, "heading", { level: 1, name: "Your campaigns" })).toBeTruthy();
    expect(getByText(root, "Chriz EET — Stream test")).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Updates" }));
    expect(getByRole(root, "heading", { level: 1, name: "Updates" })).toBeTruthy();
    expect(getByText(root, "Existing campaigns are never patched in place.")).toBeTruthy();
    expect(queryByText(root, "Update now")).toBeNull();
  });

  it("does not let a late evaluation overwrite a newer selection", async () => {
    const backend = new FixtureBackend({ evaluationDelays: [0, 80, 0] });
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    const handle = await mountApp(root, backend);
    await handle.navigate("setup");

    const optional = getByRole(root, "checkbox", { name: /Companion conversations/ });
    await user.click(optional);
    await user.click(optional);
    await new Promise((resolve) => window.setTimeout(resolve, 100));

    expect((optional as HTMLInputElement).checked).toBe(false);
    expect(getByText(root, "2 campaign choices selected")).toBeTruthy();
  });

  it("uses the native discovery, destination, review, and event path without fixture controls", async () => {
    const calls: string[] = [];
    let emitNative: ((event: unknown) => void) | undefined;
    const channelFactory: EventChannelFactory = (onMessage) => {
      emitNative = onMessage;
      return { nativeChannel: true };
    };
    const selection = { platform: "windows", features: {}, inputs: {} };
    const evaluation = {
      view: { categories: ["Foundation"], controls: [] },
      normalized_selection: selection,
      findings: [],
      plan: { phases: [{ id: "main", title: "Build main", detail: "Install the reviewed recipe." }] },
      selected_choice_count: 0,
    };
    const invoke: InvokeCommand = async (command, args) => {
      calls.push(command);
      switch (command) {
        case "bootstrap": return { mode: "native", engine_version: "0.1.0", recipe_version: null };
        case "discover_games": return {
          bg1_candidates: [{ id: "native-bg1", label: "BG1 clean", path: "C:\\BG1", storefront: "steam", build: "2.7.3.0", freshness: "fresh", eligible: true, findings: [] }],
          bg2_candidates: [{ id: "native-bg2", label: "BG2 clean", path: "C:\\BG2", storefront: "steam", build: "2.7.3.0", freshness: "fresh", eligible: true, findings: [] }],
          selected_bg1_id: "native-bg1",
          selected_bg2_id: "native-bg2",
        };
        case "evaluate_build": return evaluation;
        case "inspect_destination":
          expect(args).toEqual({ path: "D:\\Native Campaign", bg1CandidateId: "native-bg1", bg2CandidateId: "native-bg2" });
          return { path: "D:\\Native Campaign", safe: true, title: "Ready", detail: "Isolated." };
        case "freeze_review": return { review_token: "native-review", digest: "11".repeat(32), destination: "D:\\Native Campaign", game_labels: ["BG1 clean", "BG2 clean"], evaluation };
        case "start_build": return { run_id: "native-run" };
        default: throw new Error(`Unexpected native command ${command}`);
      }
    };
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();

    await mountApp(root, new NativeBackend(invoke, channelFactory));

    expect(calls.slice(0, 3)).toEqual(["bootstrap", "discover_games", "evaluate_build"]);
    await user.click(getByRole(root, "button", { name: "Begin setup" }));
    await user.click(getByRole(root, "button", { name: "Continue" }));
    const destination = getByLabelText(root, "Campaign destination") as HTMLInputElement;
    await user.clear(destination);
    await user.type(destination, "D:\\Native Campaign");
    destination.dispatchEvent(new Event("change", { bubbles: true }));
    await new Promise((resolve) => window.setTimeout(resolve, 0));
    await user.click(getByRole(root, "button", { name: "Continue" }));
    await user.click(getByRole(root, "button", { name: "Continue" }));
    await user.click(getByRole(root, "button", { name: "Freeze review and build" }));

    expect(getByRole(root, "heading", { level: 1, name: "Build your campaign" })).toBeTruthy();
    expect(queryByText(root, "Finish fixture build")).toBeNull();
    expect(getByRole(root, "button", { name: "Cancel build" })).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Campaigns" }));
    expect(getByRole(root, "heading", { level: 1, name: "Build your campaign" })).toBeTruthy();
    expect(getByRole(root, "button", { name: "Cancel build" })).toBeTruthy();
    (emitNative as (event: RunEventEnvelope | unknown) => void)({
      run_id: "native-run",
      sequence_as_string: "1",
      event: { type: "campaign_finished", install_id: "native-install" },
    });
    await new Promise((resolve) => window.setTimeout(resolve, 0));
    expect(getByRole(root, "heading", { level: 1, name: "Campaign complete" })).toBeTruthy();
  });

  it("surfaces a rejected destination and does not retain the previous safe state", async () => {
    class RejectingDestinationBackend extends FixtureBackend {
      #calls = 0;

      override inspectDestination(path: string, bg1CandidateId: string, bg2CandidateId: string) {
        this.#calls += 1;
        if (this.#calls === 1) return super.inspectDestination(path, bg1CandidateId, bg2CandidateId);
        return Promise.reject(new BackendCommandError({
          code: "destination_unsafe",
          message: "That folder is occupied.",
          recovery_action: "Choose a new empty folder.",
          technical_detail: "fixture rejection",
        }));
      }
    }
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    const handle = await mountApp(root, new RejectingDestinationBackend());
    await handle.navigate("destination");

    const destination = getByLabelText(root, "Campaign destination") as HTMLInputElement;
    await user.clear(destination);
    await user.type(destination, "D:\\Occupied");
    destination.dispatchEvent(new Event("change", { bubbles: true }));
    await new Promise((resolve) => window.setTimeout(resolve, 0));

    expect(getByRole(root, "alert").textContent).toContain("Choose a new empty folder.");
    expect((getByRole(root, "button", { name: "Continue" }) as HTMLButtonElement).disabled).toBe(true);
  });

  it("does not start when the server-frozen review differs from what was displayed", async () => {
    class ChangedReviewBackend extends FixtureBackend {
      started = false;

      override getStatus() {
        return Promise.resolve({ mode: "native" as const, engineVersion: "0.1.0", recipeVersion: null });
      }

      override async freezeReview(selection: Parameters<FixtureBackend["freezeReview"]>[0], destination: string, bg1CandidateId: string, bg2CandidateId: string) {
        const review = await super.freezeReview(selection, destination, bg1CandidateId, bg2CandidateId);
        return { ...review, evaluation: { ...review.evaluation, selectedChoiceCount: review.evaluation.selectedChoiceCount + 1 } };
      }

      override startBuild(reviewToken: string, onEvent: Parameters<FixtureBackend["startBuild"]>[1]) {
        this.started = true;
        return super.startBuild(reviewToken, onEvent);
      }
    }
    const backend = new ChangedReviewBackend();
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    const handle = await mountApp(root, backend);
    await handle.navigate("destination");
    const destination = getByLabelText(root, "Campaign destination") as HTMLInputElement;
    await user.clear(destination);
    await user.type(destination, "D:\\Fresh Campaign");
    destination.dispatchEvent(new Event("change", { bubbles: true }));
    await new Promise((resolve) => window.setTimeout(resolve, 0));
    await user.click(getByRole(root, "button", { name: "Continue" }));
    await user.click(getByRole(root, "button", { name: "Continue" }));
    await user.click(getByRole(root, "button", { name: "Freeze review and build" }));
    await new Promise((resolve) => window.setTimeout(resolve, 0));

    expect(backend.started).toBe(false);
    expect(getByRole(root, "heading", { level: 1, name: "Review the campaign ledger" })).toBeTruthy();
    expect(getByRole(root, "alert").textContent).toContain("Read the refreshed Review");
  });

  it("lets the player leave an unrecoverable failed build after its terminal snapshot", async () => {
    class UnrecoverableBuildBackend extends FixtureBackend {
      override getStatus() {
        return Promise.resolve({ mode: "native" as const, engineVersion: "0.1.0", recipeVersion: null });
      }

      override startBuild(_reviewToken: string, onEvent: Parameters<FixtureBackend["startBuild"]>[1]) {
        queueMicrotask(() => {
          onEvent({ runId: "run-broken", sequenceAsString: "1", event: { type: "campaign_started", install_id: "install-broken", resumed: false } });
          onEvent({ runId: "run-broken", sequenceAsString: "2", event: { type: "error", step_id: null, message: "The worker ended before it could create a durable report." } });
        });
        return Promise.resolve({ runId: "run-broken" });
      }

      override getRunSnapshot(runId: string) {
        return Promise.resolve({
          runId,
          status: "failed" as const,
          events: [],
          report: null,
          error: {
            code: "build_worker_failed",
            message: "The worker ended unexpectedly.",
            recovery_action: "Return to Campaigns and start with a new destination.",
            technical_detail: "fixture worker panic",
          },
        });
      }
    }
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    const handle = await mountApp(root, new UnrecoverableBuildBackend());
    await handle.navigate("destination");
    const destination = getByLabelText(root, "Campaign destination") as HTMLInputElement;
    await user.clear(destination);
    await user.type(destination, "D:\\Broken Campaign");
    destination.dispatchEvent(new Event("change", { bubbles: true }));
    await new Promise((resolve) => window.setTimeout(resolve, 0));
    await user.click(getByRole(root, "button", { name: "Continue" }));
    await user.click(getByRole(root, "button", { name: "Continue" }));
    await user.click(getByRole(root, "button", { name: "Freeze review and build" }));
    await new Promise((resolve) => window.setTimeout(resolve, 0));
    await new Promise((resolve) => window.setTimeout(resolve, 0));

    expect(getByText(root, "The build stopped safely")).toBeTruthy();
    expect(queryByText(root, "Retry failed step")).toBeNull();
    await user.click(getByRole(root, "button", { name: "Campaigns" }));
    expect(getByRole(root, "heading", { level: 1, name: "Your campaigns" })).toBeTruthy();
  });

  it("preserves a failed native snapshot and Retry when resume is rejected", async () => {
    class RejectingResumeBackend extends FixtureBackend {
      override getStatus() {
        return Promise.resolve({ mode: "native" as const, engineVersion: "0.1.0", recipeVersion: null });
      }

      override startBuild(_reviewToken: string, onEvent: Parameters<FixtureBackend["startBuild"]>[1]) {
        queueMicrotask(() => {
          onEvent({ runId: "run-failed", sequenceAsString: "1", event: { type: "campaign_started", install_id: "install-failed", resumed: false } });
          onEvent({ runId: "run-failed", sequenceAsString: "2", event: { type: "error", step_id: null, message: "The build stopped safely for this test." } });
        });
        return Promise.resolve({ runId: "run-failed" });
      }

      override getRunSnapshot(runId: string) {
        return Promise.resolve({
          runId,
          status: "failed",
          events: [],
          report: {
            install_id: "install-failed",
            managed_root: "D:\\Failed Campaign",
            plan_sha256: "11".repeat(32),
            status: { status: "failed", step_id: "install:test", reason: "fixture failure" },
          },
          error: null,
        });
      }

      override resumeBuild() {
        return Promise.reject(new BackendCommandError({
          code: "build_already_running",
          message: "The build cannot resume yet.",
          recovery_action: "Keep the failed copy and try Retry again.",
          technical_detail: "fixture rejection",
        }));
      }
    }
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    const handle = await mountApp(root, new RejectingResumeBackend());
    await handle.navigate("destination");
    const destination = getByLabelText(root, "Campaign destination") as HTMLInputElement;
    await user.clear(destination);
    await user.type(destination, "D:\\Failed Campaign");
    destination.dispatchEvent(new Event("change", { bubbles: true }));
    await new Promise((resolve) => window.setTimeout(resolve, 0));
    await user.click(getByRole(root, "button", { name: "Continue" }));
    await user.click(getByRole(root, "button", { name: "Continue" }));
    await user.click(getByRole(root, "button", { name: "Freeze review and build" }));
    await new Promise((resolve) => window.setTimeout(resolve, 0));
    const retry = getByRole(root, "button", { name: "Retry failed step" });

    await user.click(retry);
    await new Promise((resolve) => window.setTimeout(resolve, 0));

    expect(getByText(root, "The build stopped safely")).toBeTruthy();
    expect(getByRole(root, "button", { name: "Retry failed step" })).toBeTruthy();
    expect(getByRole(root, "alert").textContent).toContain("Keep the failed copy");
  });
});
