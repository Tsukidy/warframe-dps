// The boot code's decode result reaches the UI (frontend.md §9 "a dismissible report").
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { renderWithProviders } from "../../test/render";
import { useAppStore } from "../../state/store";
import { BootNotice, bootMessages } from "./BootNotice";

const EMPTY = { unmapped: [], converted: [], defaults_applied: [], retired_items: [], warnings: [] };

describe("BootNotice", () => {
  beforeEach(() => {
    // The catalogue (for item names) never arrives in this test.
    vi.stubGlobal("fetch", vi.fn(() => new Promise(() => undefined)));
  });
  afterEach(() => {
    vi.unstubAllGlobals();
    useAppStore.getState().setBootDecode(null);
  });

  it("shows nothing for a clean decode", () => {
    useAppStore.getState().setBootDecode({ code: "wfd1-x", report: EMPTY });
    renderWithProviders(<BootNotice />);
    expect(screen.queryByTestId("boot-notice")).toBeNull();
  });

  it("shows a failed decode as an alert and dismisses it", async () => {
    useAppStore.getState().setBootDecode({ code: "wfd1-bad", error: { key: "import.unrecognized", args: {} } });
    renderWithProviders(<BootNotice />);
    const alert = screen.getByRole("alert");
    expect(alert.textContent).toContain("could not be read");
    expect(alert.textContent).toContain("This is not a build code or a supported link.");
    await userEvent.click(screen.getByRole("button", { name: "Dismiss" }));
    expect(useAppStore.getState().bootDecode).toBeNull();
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("lists the report's warnings, retired items and unmapped fields", () => {
    const b = {
      code: "wfd1-x",
      report: {
        ...EMPTY,
        warnings: [{ key: "import.migrated", args: { from: 1, to: 2 } }],
        retired_items: [4242],
        unmapped: [{ field: "m.3", value: "Foo" }],
      },
    };
    expect(bootMessages(b).map((m) => m.key)).toEqual(["import.migrated", "import.retired_item", "import.unmapped"]);
    useAppStore.getState().setBootDecode(b);
    renderWithProviders(<BootNotice />);
    const status = screen.getByRole("status");
    expect(status.textContent).toContain("Converted from code version 1 to 2.");
    expect(status.textContent).toContain("Item 4242 no longer exists and was removed.");
    expect(status.textContent).toContain("Not imported: m.3 = Foo.");
  });
});
