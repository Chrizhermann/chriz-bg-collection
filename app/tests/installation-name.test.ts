// @vitest-environment jsdom

import { fireEvent, getByRole, waitFor } from "@testing-library/dom";
import { afterEach, describe, expect, it, vi } from "vitest";
import { installationNameEditor, installationNameError } from "../src/components/installation-name";

const installations = [
  { installId: "first", name: "My adventure" },
  { installId: "second", name: "Other adventure" },
];
afterEach(() => { document.body.replaceChildren(); });

describe("installation display names", () => {
  it("validates trimmed names, legacy duplicates and case-insensitive Unicode names", () => {
    expect(installationNameError("  new adventure  ", installations)).toBeNull();
    expect(installationNameError(" MY ADVENTURE ", installations)).toContain("already exists");
    expect(installationNameError("MY ADVENTURE", installations, "first")).toBeNull();
    expect(installationNameError("älven", [{ installId: "unicode", name: "ÄLVEN" }])).toContain("already exists");
    expect(installationNameError("My adventure", [...installations, { installId: "legacy", name: "My adventure" }], "first")).toContain("already exists");
    for (const name of ["", " ", ".", "..", "Bad/name", "Bad\\name", "Bad:name", "Bad?name", "Bad\nname", "a".repeat(81)]) {
      expect(installationNameError(name, [])).not.toBeNull();
    }
    expect(installationNameError("🐉".repeat(80), [])).toBeNull();
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

  it("shows an inline duplicate error and never submits it", () => {
    const onRename = vi.fn();
    const control = installationNameEditor({ installId: "first", name: "My adventure", installations, onRename });
    document.body.append(control);
    (control as HTMLDetailsElement).open = true;
    const input = getByRole(control, "textbox") as HTMLInputElement;
    fireEvent.input(input, { target: { value: " OTHER ADVENTURE " } });
    expect(input.getAttribute("aria-invalid")).toBe("true");
    expect(getByRole(control, "alert").textContent).toContain("already exists");
    expect((getByRole(control, "button", { name: "Save name" }) as HTMLButtonElement).disabled).toBe(true);
    fireEvent.submit(control.querySelector("form")!);
    expect(onRename).not.toHaveBeenCalled();
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
