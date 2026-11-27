// Damage and status types for display (frontend.md §10): a colour token, a glyph in the sprite and a label key per
// type. Colour is never the only cue: every use shows the glyph or the label as well.
import type { DamageType, StatusType } from "../api/generated";
import type { MessageId } from "../i18n/messages";

/** Damage types in spec 00 §2.2 order. */
export const DAMAGE_TYPES = [
  "IMP", "PUN", "SLA", "HEA", "COL", "ELE", "TOX", "BLA", "RAD", "GAS", "MAG", "VIR", "COR", "VOI", "TAU", "TRU",
  "CIN", "FIN",
] as const satisfies readonly DamageType[];

/** Status types in spec 00 §2.2 order. */
export const STATUS_TYPES = [
  "IMP", "PUN", "SLA", "HEA", "COL", "ELE", "TOX", "BLA", "RAD", "GAS", "MAG", "VIR", "COR", "VOI", "TAU", "MIC",
  "KND", "LIF",
] as const satisfies readonly StatusType[];

/** Every id with a token and a glyph. */
export type TypeId = DamageType | StatusType;

export interface TypeStyle {
  /** CSS custom property holding the colour (`--dt-HEA`). */
  token: `--dt-${TypeId}`;
  /** `var(--dt-HEA)`, for inline styles and charts. */
  color: string;
  /** Symbol id in `icons/sprite.svg`. */
  glyph: `dt-${TypeId}`;
  /** Full label message id. */
  label: MessageId;
  /** Short label message id (three letters). */
  short: MessageId;
}

function style(id: TypeId): TypeStyle {
  return {
    token: `--dt-${id}`,
    color: `var(--dt-${id})`,
    glyph: `dt-${id}`,
    label: `damage.${id}.label` as MessageId,
    short: `damage.${id}.short` as MessageId,
  };
}

const ALL: readonly TypeId[] = [...new Set<TypeId>([...DAMAGE_TYPES, ...STATUS_TYPES])];

/** Display data per damage or status type. */
export const TYPE_STYLES: Readonly<Record<TypeId, TypeStyle>> = Object.fromEntries(
  ALL.map((id) => [id, style(id)]),
) as Record<TypeId, TypeStyle>;

export function typeStyle(id: TypeId): TypeStyle {
  return TYPE_STYLES[id];
}
