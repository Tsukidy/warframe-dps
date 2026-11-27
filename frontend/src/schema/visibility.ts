// Visibility and binding of schema-driven controls (AD-17; server-api.md §5.6 `ToggleSchema.visible_if`).
import type { ToggleSchema, VisibilitySchema, WeaponSlot } from "../api/generated";

/** What the current weapon offers, for `visible_if`. */
export interface VisibilityContext {
  weaponSlot?: WeaponSlot;
  /** Roles of the weapon's attacks (any match shows the control). */
  attackRoles?: readonly string[];
  traits?: readonly string[];
  /** Show `advanced` controls. */
  advanced?: boolean;
}

function anyOf(wanted: readonly string[] | undefined, have: readonly string[] | undefined): boolean {
  if (!wanted || wanted.length === 0) return true;
  if (!have) return false;
  return wanted.some((w) => have.includes(w));
}

/** Whether a control is shown: every listed condition must match (an empty or absent list matches). */
export function isVisible(v: VisibilitySchema | undefined, advanced: boolean, ctx: VisibilityContext): boolean {
  if (advanced && !ctx.advanced) return false;
  if (!v) return true;
  return (
    anyOf(v.weapon_slot, ctx.weaponSlot ? [ctx.weaponSlot] : undefined) &&
    anyOf(v.attack_role, ctx.attackRoles) &&
    anyOf(v.trait, ctx.traits)
  );
}

/**
 * Fields whose value is an object with one numeric member: the control edits that member (`toggles.heavy =
 * { start_combo }` T-09, `toggles.stealth = { rank }` T-08; server-api.md §4). The object switches the toggle on;
 * a missing member means the server's default (spec 00 §6.1: T-09 start combo 12, T-08 the weapon's maximum rank;
 * CONTRACTS.md T01a C-15 and T12), so switching on writes `{}` and never guesses a number.
 */
export const NUMBER_MEMBER: Readonly<Record<string, string>> = {
  "toggles.heavy": "start_combo",
  "toggles.stealth": "rank",
};

/**
 * Fields a generic control cannot bind (server-api.md §5.6 `toggles[].field`): buff toggles (`buffs`: the BuffsPanel
 * renders `ScenarioSchema.buffs`), weapon-variant pickers (`build.variant.*`: the loadout feature renders them from
 * the weapon detail), optimizer-request fields (`opt.*`, T-36: only the optimizer page writes them; a calculator
 * panel would write them into the scenario, which the server rejects) and `custom` controls (a component of their
 * own per toggle id).
 */
export function isGenericField(t: Pick<ToggleSchema, "field" | "control">): boolean {
  if (t.field === "buffs" || t.field.startsWith("buffs.")) return false;
  if (t.field.startsWith("build.variant")) return false;
  if (t.field.startsWith("opt.")) return false;
  if (t.control.kind === "custom") return false;
  if (t.control.kind === "select" && (!t.control.options || t.control.options.length === 0)) return false;
  return true;
}

/** Reads the numeric value of a control from the document value. */
export function toNumber(field: string, value: unknown): number | null {
  if (value === null || value === undefined) return null;
  if (typeof value === "number") return value;
  const member = NUMBER_MEMBER[field];
  if (member && typeof value === "object") {
    const v = (value as Record<string, unknown>)[member];
    return typeof v === "number" ? v : null;
  }
  return null;
}

/** Whether an optional-number toggle is on: a number, or for `NUMBER_MEMBER` fields any object (`{}` included). */
export function isOn(field: string, value: unknown): boolean {
  if (typeof value === "number") return true;
  return NUMBER_MEMBER[field] !== undefined && typeof value === "object" && value !== null;
}

/**
 * The document value written when an optional-number toggle is switched on: `{}` (the server default) for
 * `NUMBER_MEMBER` fields, else `fallback`.
 */
export function switchedOnValue(field: string, fallback: number): unknown {
  return NUMBER_MEMBER[field] !== undefined ? {} : fallback;
}

/** The document value for a number entered in a control. */
export function fromNumber(field: string, n: number | null): unknown {
  if (n === null) return null;
  const member = NUMBER_MEMBER[field];
  return member ? { [member]: n } : n;
}
