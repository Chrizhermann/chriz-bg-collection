// @vitest-environment jsdom

import { fireEvent, getByRole, waitFor } from "@testing-library/dom";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  installationNameEditor,
  installationNameNote,
  sanitizeInstallationName,
  savedInstallationName,
} from "../src/components/installation-name";

const installations = [
  { installId: "first", name: "My adventure" },
  { installId: "second", name: "Other adventure" },
];
afterEach(() => { document.body.replaceChildren(); });

describe("installation display names", () => {
  it("saves every name, numbering taken ones and cleaning what Windows cannot use", () => {
    expect(installationNameNote("  new adventure  ", installations)).toBeNull();
    expect(savedInstallationName(" MY ADVENTURE ", installations)).toBe("MY ADVENTURE (2)");
    expect(installationNameNote(" MY ADVENTURE ", installations)).toBe("Will be saved as “MY ADVENTURE (2)”.");
    expect(installationNameNote("MY ADVENTURE", installations, "first")).toBeNull();
    expect(savedInstallationName("älven", [{ installId: "unicode", name: "ÄLVEN" }])).toBe("älven (2)");
    expect(savedInstallationName("My adventure", [...installations, { installId: "legacy", name: "My adventure" }], "first")).toBe("My adventure (2)");
    expect(savedInstallationName("Run", [{ installId: "a", name: "Run" }, { installId: "b", name: "run (2)" }])).toBe("Run (3)");
    for (const [typed, saved] of [
      ["Bad/name", "Bad-name"], ["Bad\\name", "Bad-name"], ["Bad:name", "Bad-name"], ["Bad?name", "Bad-name"],
      ["Bad\nname", "Badname"], ["<a|b*c>", "-a-b-c-"], ["a".repeat(81), "a".repeat(80)],
    ] as const) {
      expect(sanitizeInstallationName(typed)).toBe(saved);
      expect(installationNameNote(typed, [])).toBe(`Will be saved as “${saved}”.`);
    }
    for (const empty of ["", " ", ".", "..", " ... ", "\u0001"]) {
      expect(sanitizeInstallationName(empty)).toBeNull();
      expect(savedInstallationName(empty, [])).toBe("Chriz Easy BG");
      expect(savedInstallationName(empty, [{ installId: "default", name: "Chriz Easy BG" }])).toBe("Chriz Easy BG (2)");
    }
    expect(installationNameNote("🐉".repeat(80), [])).toBeNull();
    const long = "L".repeat(80);
    expect(savedInstallationName(long, [{ installId: "long", name: long }])).toBe(`${"L".repeat(76)} (2)`);
  });

  it("starts collapsed and renames only the exact stable ID with a trimmed name", async () => {
    const onRename = vi.fn().mockResolvedValue(undefined);
    const control = installationNameEditor({ installId: "first", name: "My adventure", installations, onRename });
    document.body.append(control);
    expect((control as HTMLDetailsElement).open).toBe(false);
    expect(control.textContent).toContain("not your game or save folders");
    (control as HTMLDetailsElement).open = true;
    const input = getByRole(control, "textbox", { name: "Installation name" }) as HTMLInputElement;
    fireEvent.input(input, { target: { value: "  Dragon run  " } });
    fireEvent.submit(control.querySelector("form")!);
    await waitFor(() => expect(onRename).toHaveBeenCalledExactlyOnceWith("first", "Dragon run"));
    expect(input.value).toBe("Dragon run");
    expect(getByRole(control, "status").textContent).toBe("Installation renamed to “Dragon run”.");
  });

  it("shows a calm note for a taken name and saves it numbered", async () => {
    const onRename = vi.fn().mockResolvedValue(undefined);
    const control = installationNameEditor({ installId: "first", name: "My adventure", installations, onRename });
    document.body.append(control);
    (control as HTMLDetailsElement).open = true;
    const input = getByRole(control, "textbox") as HTMLInputElement;
    fireEvent.input(input, { target: { value: " OTHER ADVENTURE " } });
    expect(input.getAttribute("aria-invalid")).toBeNull();
    expect(control.querySelector("[role=alert]:not([hidden])")).toBeNull();
    expect(control.textContent).toContain("Will be saved as “OTHER ADVENTURE (2)”.");
    expect((getByRole(control, "button", { name: "Save name" }) as HTMLButtonElement).disabled).toBe(false);
    fireEvent.submit(control.querySelector("form")!);
    await waitFor(() => expect(onRename).toHaveBeenCalledExactlyOnceWith("first", "OTHER ADVENTURE (2)"));
    expect(getByRole(control, "status").textContent).toBe("Installation renamed to “OTHER ADVENTURE (2)”.");
  });

  it("keeps the entered name and displays a backend conflict as plain text", async () => {
    const onRename = vi.fn().mockRejectedValue(new Error("Another installation claimed this name. <img src=x>"));
    const control = installationNameEditor({ installId: "first", name: "My adventure", installations, onRename });
    document.body.append(control);
    (control as HTMLDetailsElement).open = true;
    const input = getByRole(control, "textbox") as HTMLInputElement;
    fireEvent.input(input, { target: { value: "My next run" } });
    fireEvent.submit(control.querySelector("form")!);
    await waitFor(() => expect(getByRole(control, "alert").textContent).toContain("Another installation claimed"));
    expect(input.value).toBe("My next run");
    expect(control.querySelector("img")).toBeNull();
    expect((getByRole(control, "button", { name: "Save name" }) as HTMLButtonElement).disabled).toBe(false);
  });

  it("prevents duplicate submissions while saving and honors an external operation lock", async () => {
    let finish!: () => void;
    const onRename = vi.fn(() => new Promise<void>((resolve) => { finish = resolve; }));
    const control = installationNameEditor({ installId: "first", name: "My adventure", installations, onRename });
    document.body.append(control);
    (control as HTMLDetailsElement).open = true;
    fireEvent.submit(control.querySelector("form")!);
    fireEvent.submit(control.querySelector("form")!);
    expect(onRename).toHaveBeenCalledTimes(1);
    expect((getByRole(control, "textbox") as HTMLInputElement).disabled).toBe(true);
    finish();
    await waitFor(() => expect((getByRole(control, "textbox") as HTMLInputElement).disabled).toBe(false));

    const locked = installationNameEditor({ installId: "first", name: "My adventure", installations, onRename, disabled: true });
    document.body.append(locked);
    (locked as HTMLDetailsElement).open = true;
    fireEvent.submit(locked.querySelector("form")!);
    expect(onRename).toHaveBeenCalledTimes(1);
    expect((getByRole(locked, "button") as HTMLButtonElement).disabled).toBe(true);
  });
});
