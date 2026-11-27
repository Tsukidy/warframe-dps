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
 * { start_combo }` T-09, `toggles.stealth = { rank }` T-08; server-api.md §4).
 */
export const NUMBER_MEMBER: Readonly<Record<string, string>> = {
  "toggles.heavy": "start_combo",
  "toggles.stealth": "rank",
};

/**
 * Fields a generic control cannot bind: buff toggles (`buffs`: the BuffsPanel renders `ScenarioSchema.buffs`) and
 * weapon-variant pickers (`build.variant.*`: the loadout feature renders them from the weapon detail).
 */
export function isGenericField(t: Pick<ToggleSchema, "field" | "control">): boolean {
  if (t.field === "buffs" || t.field.startsWith("buffs.")) return false;
  if (t.field.startsWith("build.variant")) return false;
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

/** The document value for a number entered in a control. */
export function fromNumber(field: string, n: number | null): unknown {
  if (n === null) return null;
  const member = NUMBER_MEMBER[field];
  return member ? { [member]: n } : n;
}
