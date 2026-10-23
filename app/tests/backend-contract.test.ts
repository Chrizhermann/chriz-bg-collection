import { describe, expect, it, vi } from "vitest";

import {
  BackendCommandError,
  FixtureBackend,
  NativeBackend,
  type EventChannelFactory,
  type InvokeCommand,
} from "../src/backend";
import type { OperationProgress, RunEventEnvelope, UpdateSummary } from "../src/contracts";

describe("opt-in installation lookup diagnostics", () => {
  const wire = {
    checked_at: "2026-09-22T07:00:00Z",
    network_state: "unconfigured",
    application: { state: "up-to-date", current_version: "0.1.0-alpha.19", available_version: null, detail: "Current app." },
    recipe: { state: "up-to-date", current_version: "0.1.0-alpha.17", available_version: null, disposition: "up-to-date", detail: "Bundled collection.", changes: [] },
    managed_copies: [
      { install_id: "ready", name: "Ready game", path: "D:\\Ready", installed_recipe_version: "0.1.0-alpha.17", state: "up-to-date", detail: "Available." },
      { install_id: "old", name: "Chriz Easy BG — incomplete installation", path: "D:\\Old", installed_recipe_version: null, state: "stale", detail: "Folder unavailable." },
    ],
  } as const;
  const expected: UpdateSummary = {
    checkedAt: wire.checked_at,
    networkState: "unconfigured",
    application: { state: "up-to-date", currentVersion: "0.1.0-alpha.19", availableVersion: null, detail: "Current app.", releaseNotes: undefined },
    recipe: { state: "up-to-date", currentVersion: "0.1.0-alpha.17", availableVersion: null, disposition: "up-to-date", detail: "Bundled collection.", changes: [] },
    managedCopies: [
      { installId: "ready", name: "Ready game", path: "D:\\Ready", installedRecipeVersion: "0.1.0-alpha.17", state: "up-to-date", detail: "Available." },
      { installId: "old", name: "Chriz Easy BG — incomplete installation", path: "D:\\Old", installedRecipeVersion: null, state: "stale", detail: "Folder unavailable." },
    ],
    radar: undefined,
  };

  it.each([undefined, false])("does not log when the native flag is %s", async (enabled) => {
    const invoke = vi.fn<InvokeCommand>(async () => enabled === undefined ? wire : { ...wire, lookup_trace_enabled: enabled });

    await expect(new NativeBackend(invoke).getUpdates()).resolves.toEqual(expected);
    expect(invoke.mock.calls).toEqual([["check_updates", undefined]]);
  });

  it("logs exactly the received identities and states without changing their projection", async () => {
    const invoke = vi.fn<InvokeCommand>(async (command) => command === "check_updates" ? { ...wire, lookup_trace_enabled: true } : undefined);

    await expect(new NativeBackend(invoke).getUpdates()).resolves.toEqual(expected);
    expect(invoke.mock.calls).toEqual([
      ["check_updates", undefined],
      ["record_install_lookup_client", { report: {
        checked_at: wire.checked_at,
        copies: [
          { install_id: "ready", name: "Ready game", path: "D:\\Ready", state: "up-to-date" },
          { install_id: "old", name: "Chriz Easy BG — incomplete installation", path: "D:\\Old", state: "stale" },
        ],
      } }],
    ]);
  });

  it("keeps Updates working if the diagnostic command is rejected", async () => {
    const invoke = vi.fn<InvokeCommand>(async (command) => {
      if (command === "check_updates") return { ...wire, lookup_trace_enabled: true };
      throw new Error("Diagnostic log is not writable.");
    });

    await expect(new NativeBackend(invoke).getUpdates()).resolves.toEqual(expected);
    expect(invoke).toHaveBeenCalledTimes(2);
    expect(invoke.mock.calls[1]?.[0]).toBe("record_install_lookup_client");
  });
});

describe("managed patch recovery projection", () => {
  it("maps recovery independently from the folder being available", async () => {
    const backend = new NativeBackend(async () => [{
      id: "recover", name: "My game", path: "D:\\CEBG", status: "Patch recovery needed",
      receipt_path: null, launch_path: "D:\\CEBG\\game\\InfinityLoader.exe", completed_at_millis: 1,
      available: true, resumable: false, patch_recovery_needed: true,
    }]);
    const [installation] = await backend.listManagedInstallations();
    expect(installation).toMatchObject({ available: true, resumable: false, patchRecoveryNeeded: true });
    expect(installation).not.toHaveProperty("patch_recovery_needed");
  });
});

describe("explicit installation removal", () => {
  it("uses only the registry id and confirmed token on the native command boundary", async () => {
    const preview = { installId: "managed-one", displayName: "Test", managedRoot: "D:\\Test", action: "delete", approximateSizeBytes: "12884901888", preservedSavePath: null, confirmationToken: "bound-token" };
    const invoke = vi.fn(async (command: string) => command === "preview_installation_removal" ? preview : undefined);
    const backend = new NativeBackend(invoke);
    await expect(backend.previewInstallationRemoval("managed-one")).resolves.toEqual(preview);
    expect(invoke).toHaveBeenCalledTimes(1); // Closing confirmation requires no mutation call.
    await backend.removeInstallation("managed-one", "bound-token");
    expect(invoke).toHaveBeenLastCalledWith("remove_installation", { installId: "managed-one", confirmationToken: "bound-token" });
  });
  it("fixture rejects mismatched confirmation and only removes the confirmed entry", async () => {
    const backend = new FixtureBackend({ managedInstallations: [{ id: "one", name: "One", path: "D:\\One", status: "Ready to play", receiptPath: null, launchPath: null, completedAtMillis: null, available: true, resumable: false }] });
    const first = await backend.previewInstallationRemoval("one");
    await expect(backend.removeInstallation("other", first.confirmationToken)).rejects.toThrow();
    expect(await backend.listManagedInstallations()).toHaveLength(1);
    const confirmed = await backend.previewInstallationRemoval("one");
    await backend.removeInstallation("one", confirmed.confirmationToken);
    expect(await backend.listManagedInstallations()).toHaveLength(0);
    await expect(backend.removeInstallation("one", confirmed.confirmationToken)).rejects.toThrow();
  });
});

describe("engine-shaped fixture backend", () => {
  it("suggests the safe CEBG installation defaults", async () => {
    await expect(new FixtureBackend().getInstallationDefaults()).resolves.toEqual({
      name: "Chriz Easy BG",
      path: "C:\\Users\\Player\\Games\\Chriz Easy BG",
    });
  });

  it("returns semantic recipe controls without exposing WeiDU component numbers", async () => {
    const backend = new FixtureBackend();
    const evaluation = await backend.evaluateBuild({
      platform: "windows",
      features: {},
      inputs: {},
    });

    expect(evaluation.view.controls.map((control) => control.decision)).toEqual(
      expect.arrayContaining(["default", "optional", "mandatory"]),
    );
    expect(evaluation.view.controls.map((control) => control.readiness)).toEqual(
      expect.arrayContaining(["ready", "experimental", "blocked"]),
    );
    expect(evaluation.view.controls.every((control) => "parent" in control && "inputs" in control)).toBe(true);
    expect(JSON.stringify(evaluation)).not.toMatch(/component[_-]?(number|id)|components/i);
    expect(evaluation.plan.phases).toHaveLength(5);
    expect(evaluation.plan.phases.at(-1)?.title).toBe("Finalization and reviewed tail");
  });

  it("models every source freshness finding and keeps the game selectors independent", async () => {
    const discovery = await new FixtureBackend().discoverGames();

    expect(discovery.bg1Candidates.map((candidate) => candidate.freshness)).toEqual(
      expect.arrayContaining(["fresh", "modified", "unsupported-version", "missing-sod"]),
    );
    expect(discovery.bg2Candidates.map((candidate) => candidate.freshness)).toContain(
      "unverified-storefront",
    );
    expect(discovery.selectedBg1Id).not.toBe(discovery.selectedBg2Id);
  });

  it("exposes fixture snapshots for manual attention, failure, retry, diagnostics, and completion", async () => {
    const backend = new FixtureBackend();

    expect((await backend.getBuildSnapshot()).state).toBe("waiting-manual");
    expect((await backend.advanceBuild()).state).toBe("attention");
    expect((await backend.advanceBuild()).state).toBe("failed");
    expect((await backend.retryBuild()).state).toBe("running");
    expect((await backend.advanceBuild()).state).toBe("complete");
    expect((await backend.exportDiagnostics("fixture-install"))?.path).toMatch(/diagnostics/i);
  });
});

describe("native command adapter", () => {
  it("maps installation defaults from the narrow native command", async () => {
    const invoke = vi.fn<InvokeCommand>(async (command, args) => {
      expect(command).toBe("installation_defaults");
      expect(args).toBeUndefined();
      return {
        name: "Chriz Easy BG",
        path: "C:\\Users\\Player\\Games\\Chriz Easy BG",
      };
    });

    await expect(new NativeBackend(invoke).getInstallationDefaults()).resolves.toEqual({
      name: "Chriz Easy BG",
      path: "C:\\Users\\Player\\Games\\Chriz Easy BG",
    });
  });

  it("maps the four read-only command contracts without exposing component numbers", async () => {
    const selection = { platform: "windows", features: {}, inputs: {} } as const;
    const invoke = vi.fn<InvokeCommand>(async (command, args) => {
      switch (command) {
        case "bootstrap":
          return { mode: "native", engine_version: "0.1.0", recipe_version: null, startup_install_id: "install-ready" };
        case "discover_games":
          return {
            bg1_candidates: [{ id: "bg1", label: "BG:EE + SoD — Steam — clean", path: "C:\\BGEE", storefront: "steam", build: "2.7.3.0", freshness: "fresh", eligible: true, findings: ["Clean."] }],
            bg2_candidates: [{ id: "bg2", label: "BGII:EE — Steam — modified", path: "C:\\BG2EE", storefront: "steam", build: null, freshness: "modified", eligible: false, findings: ["Modified."] }],
            selected_bg1_id: "bg1",
            selected_bg2_id: "bg2",
          };
        case "inspect_game_path":
          expect(args).toEqual({ role: "bgee_sod", path: "C:\\Chosen BGEE" });
          return { id: "chosen", label: "BG:EE + SoD — Steam — clean", path: "C:\\Chosen BGEE", storefront: "steam", build: "2.7.3.0", freshness: "fresh", eligible: true, findings: ["Clean."] };
        case "evaluate_build":
          expect(args).toEqual({ selection });
          return {
            view: {
              categories: ["Foundation"],
              controls: [{
                id: "foundation",
                title: "Foundation",
                description: "Required collection foundation.",
                category: "Foundation",
                decision: "mandatory",
                readiness: "ready",
                parent: null,
                selected: true,
                interactive: false,
                unavailable_reason: null,
                inputs: [],
              }],
            },
            normalized_selection: { platform: "windows", features: { foundation: true }, inputs: {} },
            findings: [{ rule: "fixture", feature_id: "foundation", message: "Visible finding." }],
            plan: { phases: [{ id: "main", title: "Build the main campaign", detail: "Install the selected main recipe." }] },
            selected_choice_count: 1,
          };
        default:
          throw new Error(`Unexpected command ${command}`);
      }
    });
    const backend = new NativeBackend(invoke);

    await expect(backend.getStatus()).resolves.toEqual({ mode: "native", engineVersion: "0.1.0", recipeVersion: null, startupInstallId: "install-ready" });
    const discovery = await backend.discoverGames();
    expect(discovery).toMatchObject({ selectedBg1Id: "bg1", selectedBg2Id: "bg2" });
    expect(discovery.bg1Candidates[0]).toMatchObject({ storefront: "steam", build: "2.7.3.0" });
    expect(discovery.bg1Candidates[0]?.storefront).toBe("steam");
    expect(discovery.bg1Candidates[0]?.build).toBe("2.7.3.0");
    await expect(backend.inspectGamePath("bgee_sod", "C:\\Chosen BGEE")).resolves.toMatchObject({ id: "chosen", eligible: true });
    const evaluation = await backend.evaluateBuild(selection);
    expect(evaluation.view.controls[0]?.unavailableReason).toBeNull();
    expect(evaluation.normalizedSelection.features.foundation).toBe(true);
    expect(evaluation.findings[0]?.featureId).toBe("foundation");
    expect(evaluation.selectedChoiceCount).toBe(1);
    expect(JSON.stringify(evaluation)).not.toContain("components");
  });

  it("maps native folder choices through their engine-validated command results", async () => {
    const invoke = vi.fn<InvokeCommand>(async (command, args) => {
      switch (command) {
        case "choose_game_folder":
          expect(args).toEqual({ role: "bgee_sod" });
          return {
            id: "chosen-bg1",
            label: "BG:EE + SoD — chosen clean folder",
            path: "C:\\Chosen BGEE",
            storefront: "steam",
            build: "2.7.3.0",
            freshness: "fresh",
            eligible: true,
            findings: ["Clean supported installation."],
          };
        case "choose_destination_folder":
          expect(args).toEqual({ bg1CandidateId: "chosen-bg1", bg2CandidateId: "bg2" });
          return {
            path: "D:\\Chosen Campaign",
            safe: true,
            title: "Safe separate destination",
            detail: "The source games and their saves will remain untouched.",
          };
        default:
          throw new Error(`Unexpected command ${command}`);
      }
    });
    const backend = new NativeBackend(invoke);

    await expect(backend.chooseGameFolder("bgee_sod")).resolves.toMatchObject({
      id: "chosen-bg1",
      eligible: true,
    });
    await expect(backend.chooseDestinationFolder("chosen-bg1", "bg2")).resolves.toMatchObject({
      path: "D:\\Chosen Campaign",
      safe: true,
    });

    const cancelled = new NativeBackend(async () => null);
    await expect(cancelled.chooseGameFolder("bg2ee")).resolves.toBeNull();
    await expect(cancelled.chooseDestinationFolder("bg1", "bg2")).resolves.toBeNull();
  });

  it("maps a native manual archive selection without accepting a caller-owned path", async () => {
    const invoke = vi.fn<InvokeCommand>(async (command, args) => {
      if (command === "inspect_manual_downloads") {
        expect(args).toEqual({ selection: { platform: "windows", features: {}, inputs: {} } });
        return [{ artifact_id: "manual-fixture", mod_ids: ["evandra"], title: "Evandra", filename: "manual-fixture.zip", length: 1234, ready: false, detail: "Not supplied" }];
      }
      if (command === "supply_manual_archive") {
        expect(args).toEqual({ artifactId: "manual-fixture" });
        return { artifact_id: "manual-fixture", filename: "manual-fixture.zip", sha256: "11".repeat(32), length: 1234 };
      }
      throw new Error(`Unexpected command ${command}`);
    });
    const backend = new NativeBackend(invoke);

    await expect(backend.inspectManualDownloads({ platform: "windows", features: {}, inputs: {} })).resolves.toEqual([{
      artifactId: "manual-fixture", modIds: ["evandra"], title: "Evandra", filename: "manual-fixture.zip", length: 1234, ready: false, detail: "Not supplied",
    }]);

    await expect(backend.supplyManualArchive("manual-fixture")).resolves.toEqual({
      artifactId: "manual-fixture",
      filename: "manual-fixture.zip",
      sha256: "11".repeat(32),
      length: 1234,
    });

    const cancelled = new NativeBackend(async () => null);
    await expect(cancelled.supplyManualArchive("manual-fixture")).resolves.toBeNull();
  });

  it("preserves the serialized recovery contract from rejected native commands", async () => {
    const backend = new NativeBackend(async () => {
      throw {
        code: "game_profiles_failed",
        message: "Game detection is not ready.",
        recovery_action: "Install a verified game-profile release.",
        technical_detail: "manifest/game-builds is missing",
      };
    });

    const error = await backend.discoverGames().catch((caught: unknown) => caught);

    expect(error).toBeInstanceOf(BackendCommandError);
    expect(error).toMatchObject({
      code: "game_profiles_failed",
      recoveryAction: "Install a verified game-profile release.",
      technicalDetail: "manifest/game-builds is missing",
    });
  });

  it("maps destination, frozen review, detached events, snapshots, and scoped controls", async () => {
    const selection = { platform: "windows", features: {}, inputs: {} } as const;
    const events: RunEventEnvelope[] = [];
    type TestChannel = { emit: (event: unknown) => void };
    const channelFactory: EventChannelFactory = (onMessage) => ({ emit: onMessage });
    const invoke = vi.fn<InvokeCommand>(async (command, args) => {
      switch (command) {
        case "inspect_destination":
          expect(args).toEqual({ path: "D:\\Campaign", bg1CandidateId: "bg1", bg2CandidateId: "bg2" });
          return { path: "D:\\Campaign", safe: true, title: "Ready", detail: "Isolated." };
        case "freeze_review":
          expect(args).toEqual({ displayName: "My Baldur's Gate", selection, destination: "D:\\Campaign", bg1CandidateId: "bg1", bg2CandidateId: "bg2" });
          return {
            review_token: "review-opaque",
            digest: "11".repeat(32),
            display_name: "My Baldur's Gate",
            destination: "D:\\Campaign",
            game_labels: ["BG1", "BG2"],
            evaluation: {
              view: { categories: [], controls: [] },
              normalized_selection: selection,
              findings: [],
              plan: { phases: [] },
              selected_choice_count: 0,
            },
          };
        case "start_build":
          expect(args?.reviewToken).toBe("review-opaque");
          (args?.onEvent as TestChannel).emit({
            run_id: "run-1",
            sequence_as_string: "1",
            event: { type: "campaign_started", install_id: "install-1", resumed: false },
          });
          return { run_id: "run-1" };
        case "get_run_snapshot":
          expect(args).toEqual({ runId: "run-1" });
          return { run_id: "run-1", status: "running", events: [], report: null, error: null };
        case "continue_waiting":
        case "pause_run":
        case "cancel_run":
          expect(args).toEqual({ runId: "run-1" });
          return null;
        default:
          throw new Error(`Unexpected command ${command}`);
      }
    });
    const backend = new NativeBackend(invoke, channelFactory);

    await expect(backend.inspectDestination("D:\\Campaign", "bg1", "bg2")).resolves.toMatchObject({ safe: true });
    const review = await backend.freezeReview("My Baldur's Gate", selection, "D:\\Campaign", "bg1", "bg2");
    expect(review.reviewToken).toBe("review-opaque");
    expect(review.displayName).toBe("My Baldur's Gate");
    await expect(backend.startBuild(review.reviewToken, (event) => events.push(event))).resolves.toEqual({ runId: "run-1" });
    expect(events[0]).toMatchObject({ runId: "run-1", sequenceAsString: "1", event: { type: "campaign_started" } });
    await expect(backend.getRunSnapshot("run-1")).resolves.toMatchObject({ runId: "run-1", status: "running" });
    await expect(backend.continueWaiting("run-1")).resolves.toBeUndefined();
    await expect(backend.pauseRun("run-1")).resolves.toBeUndefined();
    await expect(backend.cancelRun("run-1")).resolves.toBeUndefined();
  });

  it("maps managed installations and their scoped native actions", async () => {
    const invoke = vi.fn<InvokeCommand>(async (command, args) => {
      switch (command) {
        case "list_managed_installations":
          expect(args).toBeUndefined();
          return [
            {
              id: "install-ready",
              name: "Ready campaign",
              path: "D:\\Campaigns\\Ready",
              status: "Ready to play",
              receipt_path: "D:\\Campaigns\\Ready\\install-receipt.json",
              launch_path: "D:\\Campaigns\\Ready\\InfinityLoader.exe",
              completed_at_millis: 1_788_451_200_000,
              available: true,
              resumable: false,
            },
            {
              id: "install-restart",
              name: "Incomplete campaign",
              path: "D:\\Campaigns\\Restart",
              status: "Build interrupted — ready to resume",
              receipt_path: null,
              launch_path: null,
              completed_at_millis: null,
              available: false,
              resumable: true,
            },
          ];
        case "open_manual_source":
          expect(args).toEqual({ artifactId: "manual-fixture" });
          return null;
        case "export_diagnostics":
          expect(args).toEqual({ installId: "install-ready" });
          return { path: "D:\\Diagnostics\\install-ready.zip" };
        case "launch_install":
          expect(args).toEqual({ installId: "install-ready", startRadar: false });
          return { installId: "install-ready", radar: { state: "not-requested", detail: null, technicalDetail: null },
            gameSettings: { state: "unchanged", technicalDetail: null }, gameLog: { state: "kept", technicalDetail: null } };
        case "open_install_folder":
          expect(args).toEqual({ installId: "install-ready" });
          return null;
        case "create_desktop_shortcut":
          expect(args).toEqual({ installId: "install-ready" });
          return { path: "C:\\Users\\Player\\Desktop\\Chriz Easy BG.lnk" };
        default:
          throw new Error(`Unexpected command ${command}`);
      }
    });
    const backend = new NativeBackend(invoke);

    await expect(backend.listManagedInstallations()).resolves.toEqual([
      {
        id: "install-ready",
        name: "Ready campaign",
        path: "D:\\Campaigns\\Ready",
        status: "Ready to play",
        receiptPath: "D:\\Campaigns\\Ready\\install-receipt.json",
        launchPath: "D:\\Campaigns\\Ready\\InfinityLoader.exe",
        completedAtMillis: 1_788_451_200_000,
        available: true,
        resumable: false,
      },
      {
        id: "install-restart",
        name: "Incomplete campaign",
        path: "D:\\Campaigns\\Restart",
        status: "Build interrupted — ready to resume",
        receiptPath: null,
        launchPath: null,
        completedAtMillis: null,
        available: false,
        resumable: true,
      },
    ]);
    await expect(backend.openManualSource("manual-fixture")).resolves.toBeUndefined();
    await expect(backend.exportDiagnostics("install-ready")).resolves.toEqual({
      path: "D:\\Diagnostics\\install-ready.zip",
    });
    await expect(backend.launchInstall("install-ready", false)).resolves.toEqual({
      installId: "install-ready", radar: { state: "not-requested", detail: null, technicalDetail: null },
      gameSettings: { state: "unchanged", technicalDetail: null }, gameLog: { state: "kept", technicalDetail: null },
    });
    await expect(backend.openInstallFolder("install-ready")).resolves.toBeUndefined();
    await expect(backend.createDesktopShortcut("install-ready")).resolves.toEqual({
      path: "C:\\Users\\Player\\Desktop\\Chriz Easy BG.lnk",
    });
  });

  it("preserves a cancelled diagnostics export", async () => {
    const backend = new NativeBackend(async (command, args) => {
      expect(command).toBe("export_diagnostics");
      expect(args).toEqual({ installId: "install-failed" });
      return null;
    });

    await expect(backend.exportDiagnostics("install-failed")).resolves.toBeNull();
  });
});

describe("native progress and rename boundaries", () => {
  type ProgressCallback = (progress: OperationProgress) => void;
  type TestProgressChannel = { emit: (event: unknown) => void };
  const progress: OperationProgress = { stage: "backup-game", completed: "5", total: "10", unit: "bytes" };

  const operations: [string, Record<string, unknown>, (backend: NativeBackend, callback: ProgressCallback) => Promise<unknown>][] = [
    ["install_app_update", { version: "0.1.0-alpha.19" }, (backend, callback) => backend.installAppUpdate("0.1.0-alpha.19", callback)],
    ["apply_mod_upgrade", { installId: "chosen", reviewToken: "review", fullBackup: true, backupSaves: false }, (backend, callback) => backend.applyModUpgrade("chosen", "review", true, false, callback)],
    ["restore_mod_upgrade", { installId: "chosen", undo: true }, (backend, callback) => backend.restoreModUpgrade("chosen", true, callback)],
    ["apply_install_patch", { installId: "chosen", reviewToken: "review", fullBackup: false, saveBackup: true }, (backend, callback) => backend.applyInstallPatch("chosen", "review", false, true, callback)],
    ["undo_install_patch", { installId: "chosen" }, (backend, callback) => backend.undoInstallPatch("chosen", callback)],
    ["restore_install_patch", { installId: "chosen" }, (backend, callback) => backend.restoreInstallPatch("chosen", callback)],
  ];

  it.each(operations)("routes decoded %s progress without changing command authority", async (command, expected, operation) => {
    let channel!: TestProgressChannel;
    const channelFactory: EventChannelFactory = onMessage => (channel = { emit: onMessage });
    const received = vi.fn<ProgressCallback>();
    const invoke = vi.fn<InvokeCommand>(async (name, args) => {
      expect(name).toBe(command);
      expect(args).toEqual({ ...expected, onProgress: channel });
      for (const malformed of [null, 7, "applying", [], {}, { stage: "not-a-known-stage" }]) channel.emit(malformed);
      channel.emit(progress);
      return undefined;
    });
    await operation(new NativeBackend(invoke, channelFactory), received);
    expect(received).toHaveBeenCalledExactlyOnceWith(progress);
    channel.emit({ ...progress, completed: "10" });
    expect(received).toHaveBeenCalledTimes(1);
    expect(invoke).toHaveBeenCalledTimes(1);
  });

  it("closes a progress channel after a rejected operation as well", async () => {
    let channel!: TestProgressChannel;
    const callback = vi.fn<ProgressCallback>();
    const backend = new NativeBackend(async () => {
      channel.emit(progress);
      throw { code: "write_failed", message: "Update failed", recovery_action: "Keep the game closed", technical_detail: "fixture failure" };
    }, onMessage => (channel = { emit: onMessage }));
    await expect(backend.restoreModUpgrade("chosen", false, callback)).rejects.toMatchObject({ code: "write_failed" });
    channel.emit({ ...progress, stage: "finishing", completed: "10" });
    expect(callback).toHaveBeenCalledExactlyOnceWith(progress);
  });

  it("does not create a progress channel when no listener was requested", async () => {
    const channelFactory = vi.fn<EventChannelFactory>();
    const invoke = vi.fn<InvokeCommand>(async () => undefined);
    await new NativeBackend(invoke, channelFactory).installAppUpdate("0.1.0-alpha.19");
    expect(channelFactory).not.toHaveBeenCalled();
    expect(invoke).toHaveBeenCalledExactlyOnceWith("install_app_update", { version: "0.1.0-alpha.19" });
  });

  it("renames by registry ID and proposed display name only", async () => {
    const invoke = vi.fn<InvokeCommand>(async () => ({ name: "My campaign" }));
    const backend = new NativeBackend(invoke);
    await expect(backend.renameInstallation("chosen", "  My campaign  ")).resolves.toEqual({ name: "My campaign" });
    expect(invoke).toHaveBeenCalledExactlyOnceWith("rename_installation", { installId: "chosen", name: "  My campaign  " });
  });

  it("preserves a rejected rename without retrying or falling back to a folder operation", async () => {
    const invoke = vi.fn<InvokeCommand>(async () => {
      throw { code: "invalid_name", message: "That name is already used", recovery_action: "Choose another name", technical_detail: "duplicate" };
    });
    await expect(new NativeBackend(invoke).renameInstallation("chosen", "Existing name")).rejects.toMatchObject({ code: "invalid_name", recoveryAction: "Choose another name" });
    expect(invoke).toHaveBeenCalledTimes(1);
  });
});
