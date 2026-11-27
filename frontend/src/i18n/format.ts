// Number formatting for display (frontend.md §8). Every function takes the locale, so components use them through
// `useFormat()` and tests can call them directly.
//
// - Damage: `formatDamage` rounds with floor(x + 0.5) (spec 00 U-5, 08 §8.12) and groups digits per locale.
// - Percent: fractions in model units (0.3 = 30 %), one decimal (frontend.md §8).
// - Times: seconds with three decimals (08 §8.12).
// - Compact (`34.4 k`): only for dense tables; tooltips show the full number.
import { useIntl } from "react-intl";
import { useMemo } from "react";

/** Values the server encodes as strings because JSON has no infinity (CONTRACTS.md T01b, `ext_f64`). */
export type ExtNumber = number | "Infinity" | "-Infinity";

/** The display rounding of spec 00 U-5: floor(x + 0.5). Never used for engine values. */
export function roundU5(x: number): number {
  return Math.floor(x + 0.5);
}

/** Converts an `ext_f64` value to a JS number (±Infinity for the string forms). */
export function extNumber(x: ExtNumber): number {
  if (x === "Infinity") return Number.POSITIVE_INFINITY;
  if (x === "-Infinity") return Number.NEGATIVE_INFINITY;
  return x;
}

const INFINITY = "∞";

function finiteOr(x: number, f: (v: number) => string): string {
  if (Number.isNaN(x)) return "–";
  if (x === Number.POSITIVE_INFINITY) return INFINITY;
  if (x === Number.NEGATIVE_INFINITY) return `−${INFINITY}`;
  return f(x);
}

/** A damage number: floor(x + 0.5), digit grouping (`34,381`). */
export function formatDamage(x: ExtNumber, locale: string): string {
  return finiteOr(extNumber(x), (v) =>
    new Intl.NumberFormat(locale, { maximumFractionDigits: 0 }).format(roundU5(v) + 0),
  );
}

/** A fraction as a percent with `digits` decimals (default 1): 0.3 → `30.0%`. */
export function formatPercent(fraction: number, locale: string, digits = 1): string {
  return finiteOr(fraction, (v) =>
    new Intl.NumberFormat(locale, {
      style: "percent",
      minimumFractionDigits: digits,
      maximumFractionDigits: digits,
    }).format(v),
  );
}

/** A signed percent for deltas: +12.0% / −3.5%. */
export function formatDeltaPercent(fraction: number, locale: string, digits = 1): string {
  return finiteOr(fraction, (v) =>
    new Intl.NumberFormat(locale, {
      style: "percent",
      signDisplay: "exceptZero",
      minimumFractionDigits: digits,
      maximumFractionDigits: digits,
    }).format(v),
  );
}

/** Seconds with three decimals (08 §8.12): 1.2 → `1.200 s`. */
export function formatSeconds(seconds: ExtNumber, locale: string): string {
  return finiteOr(extNumber(seconds), (v) =>
    new Intl.NumberFormat(locale, {
      style: "unit",
      unit: "second",
      unitDisplay: "short",
      minimumFractionDigits: 3,
      maximumFractionDigits: 3,
    }).format(v),
  );
}

/** A plain number with at most `digits` decimals. */
export function formatNumber(x: ExtNumber, locale: string, digits = 2): string {
  return finiteOr(extNumber(x), (v) => new Intl.NumberFormat(locale, { maximumFractionDigits: digits }).format(v));
}

/** Compact notation for dense tables (`34.4K`); damage is rounded with U-5 first. */
export function formatCompact(x: ExtNumber, locale: string): string {
  return finiteOr(extNumber(x), (v) =>
    new Intl.NumberFormat(locale, { notation: "compact", maximumFractionDigits: 1 }).format(roundU5(v)),
  );
}

/** The formatters bound to the current locale. */
export function useFormat() {
  const { locale } = useIntl();
  return useMemo(
    () => ({
      damage: (x: ExtNumber) => formatDamage(x, locale),
      percent: (x: number, digits?: number) => formatPercent(x, locale, digits),
      deltaPercent: (x: number, digits?: number) => formatDeltaPercent(x, locale, digits),
      seconds: (x: ExtNumber) => formatSeconds(x, locale),
      number: (x: ExtNumber, digits?: number) => formatNumber(x, locale, digits),
      compact: (x: ExtNumber) => formatCompact(x, locale),
    }),
    [locale],
  );
}
