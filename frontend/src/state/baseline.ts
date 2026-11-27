// Baseline (frontend.md §5 "Baseline"): a pinned second document and its last report. Every headline number shows
// Δ % against it; the optimizer's "Compare" uses the same mechanism.
import type { CalcInput, CalcReport, View } from "../api/generated";

export interface Baseline {
  doc: CalcInput;
  report: CalcReport | null;
  pinnedAt: number;
}

/**
 * Relative change (current − base) / |base|; `null` when there is nothing meaningful to show (no base, a zero or
 * non-finite base).
 */
export function relativeDelta(current: number | null | undefined, base: number | null | undefined): number | null {
  if (current == null || base == null) return null;
  if (!Number.isFinite(current) || !Number.isFinite(base) || base === 0) return null;
  return (current - base) / Math.abs(base);
}

/** The window DPS (EV) of an attack and view in a report, if present. */
export function windowDps(report: CalcReport | null | undefined, attackIdx: number, view: View): number | null {
  const a = report?.attacks[attackIdx];
  const v = a?.views.find((x) => x.view === view);
  return v ? v.ev.dps_window : null;
}

/** Δ of the window DPS of one attack and view against the baseline. */
export function windowDpsDelta(
  current: CalcReport | null | undefined,
  baseline: Baseline | null | undefined,
  attackIdx: number,
  view: View,
): number | null {
  return relativeDelta(windowDps(current, attackIdx, view), windowDps(baseline?.report, attackIdx, view));
}
