// Schema-driven renderer (AD-17; testing.md §8: "a fixture schema with a new toggle renders without code changes").
import { describe, expect, it } from "vitest";
import { screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ToggleSchema } from "../api/generated";
import { renderWithProviders } from "../test/render";
import { useAppStore } from "../state/store";
import { newDocument } from "../state/defaults";
import { SchemaFields } from "./SchemaControl";
import { isGenericField, isOn, isVisible, switchedOnValue, toNumber } from "./visibility";

const toggles: ToggleSchema[] = [
  // A toggle the code has never seen: field, id and label keys are new (label falls back to the key).
  {
    id: "T-99",
    field: "toggles.brand_new_switch",
    group: "scenario",
    label: "toggle.t99.label",
    help: "toggle.t99.help",
    control: { kind: "bool" },
    default: false,
    advanced: false,
  },
  {
    id: "T-01",
    field: "eval.window_s",
    group: "evaluation",
    label: "toggle.t01.label",
    help: "toggle.t01.help",
    control: { kind: "number", min: 1, max: 60, step: 1 },
    default: 20,
    advanced: false,
  },
  {
    id: "T-03",
    field: "toggles.hit_location",
    group: "target",
    label: "toggle.t03.label",
    help: "toggle.t03.help",
    control: {
      kind: "select",
      options: [
        { value: "body", label: "opt.body" },
        { value: "head", label: "opt.head" },
        { value: "weak_point", label: "opt.weak_point" },
      ],
    },
    default: "body",
    advanced: false,
  },
  {
    id: "T-09",
    field: "toggles.heavy",
    group: "melee",
    label: "toggle.t09.label",
    help: "toggle.t09.help",
    control: { kind: "optional_number", min: 1, max: 12, step: 1 },
    default: null,
    visible_if: { weapon_slot: ["melee"] },
    advanced: false,
  },
  {
    id: "T-21",
    field: "toggles.wp_cc_multiplicative",
    group: "scenario",
    label: "toggle.t21.label",
    help: "toggle.t21.help",
    control: { kind: "bool" },
    default: false,
    advanced: true,
  },
  {
    id: "T-13",
    field: "buffs",
    group: "buffs",
    label: "toggle.t13.label",
    help: "toggle.t13.help",
    control: { kind: "optional_number", min: 0, max: 99.99 },
    default: null,
    advanced: false,
  },
];

function reset() {
  useAppStore.getState().replaceDoc(newDocument(1204, 9001), { record: false });
}

describe("schema-driven controls", () => {
  it("renders a toggle it has never seen and writes it into the document", async () => {
    reset();
    renderWithProviders(<SchemaFields toggles={toggles} context={{ weaponSlot: "primary" }} />);
    const sw = screen.getByRole("switch", { name: "toggle.t99.label" });
    expect((sw as HTMLInputElement).checked).toBe(false);
    await userEvent.click(sw);
    expect((useAppStore.getState().doc.scenario.toggles as Record<string, unknown>).brand_new_switch).toBe(true);
    expect((sw as HTMLInputElement).checked).toBe(true);
  });

  it("renders numbers and selects bound to their document paths", async () => {
    reset();
    renderWithProviders(<SchemaFields toggles={toggles} context={{ weaponSlot: "primary" }} />);
    const window = screen.getByRole("textbox", { name: "Fight window" });
    expect((window as HTMLInputElement).value).toBe("20");
    await userEvent.clear(window);
    await userEvent.type(window, "35{Enter}");
    expect(useAppStore.getState().doc.scenario.eval.window_s).toBe(35);

    const group = screen.getByRole("radiogroup", { name: "Hit location" });
    await userEvent.click(within(group).getByRole("radio", { name: "Head" }));
    expect(useAppStore.getState().doc.scenario.toggles.hit_location).toBe("head");
  });

  it("hides controls by visibility and advanced flags, and skips buff toggles", () => {
    reset();
    renderWithProviders(<SchemaFields toggles={toggles} context={{ weaponSlot: "primary" }} />);
    expect(screen.queryByRole("switch", { name: "Heavy attack" })).toBeNull();
    expect(screen.queryByRole("switch", { name: "Weak-point crit chance multiplicative" })).toBeNull();
    expect(screen.queryByRole("switch", { name: "Roar" })).toBeNull();
  });

  it("binds optional numbers that hold an object member (T-09 heavy start combo)", async () => {
    reset();
    renderWithProviders(<SchemaFields toggles={toggles} context={{ weaponSlot: "melee", advanced: true }} />);
    const heavy = screen.getByRole("switch", { name: "Heavy attack" });
    await userEvent.click(heavy);
    // Switched on without a value: `{}`, the server's default (spec 00 §6.1 T-09 "default 12").
    expect(useAppStore.getState().doc.scenario.toggles.heavy).toEqual({});
    expect((heavy as HTMLInputElement).checked).toBe(true);
    const combo = screen.getByRole("textbox", { name: "Heavy attack" });
    expect((combo as HTMLInputElement).value).toBe("");
    expect(combo.getAttribute("placeholder")).toBe("Default");
    await userEvent.type(combo, "8{Enter}");
    expect(useAppStore.getState().doc.scenario.toggles.heavy).toEqual({ start_combo: 8 });
    await userEvent.clear(combo);
    await userEvent.tab();
    expect(useAppStore.getState().doc.scenario.toggles.heavy).toEqual({});
    await userEvent.click(heavy);
    expect(useAppStore.getState().doc.scenario.toggles.heavy).toBeNull();
    expect(screen.getByRole("switch", { name: "Weak-point crit chance multiplicative" })).toBeTruthy();
  });
});

describe("visibility", () => {
  it("matches every listed condition", () => {
    expect(isVisible(undefined, false, {})).toBe(true);
    expect(isVisible({ weapon_slot: ["melee"] }, false, { weaponSlot: "primary" })).toBe(false);
    expect(isVisible({ weapon_slot: ["melee"], trait: ["exalted"] }, false, { weaponSlot: "melee", traits: ["exalted"] })).toBe(true);
    expect(isVisible({ attack_role: ["heavy"] }, false, { attackRoles: ["normal", "heavy"] })).toBe(true);
    expect(isVisible(undefined, true, {})).toBe(false);
  });

  it("leaves buff and variant fields to their features", () => {
    expect(isGenericField({ field: "buffs", control: { kind: "optional_number" } })).toBe(false);
    expect(isGenericField({ field: "build.variant.kitgun", control: { kind: "select" } })).toBe(false);
    expect(isGenericField({ field: "toggles.unique_dr", control: { kind: "bool" } })).toBe(true);
  });

  it("leaves optimizer-request fields and custom controls to their own components (server-api.md §5.6)", () => {
    expect(isGenericField({ field: "opt.constraints.capacity.ranks", control: { kind: "select", options: [{ value: "max", label: "x" }] } })).toBe(false);
    expect(isGenericField({ field: "toggles.co_status_count", control: { kind: "custom" } })).toBe(false);
  });

  it("reads an object toggle without its member as on, with the server default (T-08, T-09)", () => {
    expect(isOn("toggles.stealth", {})).toBe(true);
    expect(isOn("toggles.heavy", { start_combo: 4 })).toBe(true);
    expect(isOn("toggles.heavy", null)).toBe(false);
    expect(isOn("toggles.headshot_window", 3)).toBe(true);
    expect(isOn("toggles.headshot_window", null)).toBe(false);
    expect(toNumber("toggles.stealth", {})).toBeNull();
    expect(switchedOnValue("toggles.stealth", 0)).toEqual({});
    expect(switchedOnValue("toggles.heavy", 1)).toEqual({});
    expect(switchedOnValue("toggles.headshot_window", 2)).toBe(2);
  });
});
