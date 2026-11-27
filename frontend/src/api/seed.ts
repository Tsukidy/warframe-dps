// MC seeds (frontend.md §5 "Seed"; spec 00 §6.1 T-29; server-api.md §6.1 `want.mc.seed`).
//
// `eval.seed = "auto"` (the default of new documents, parity row 25): the MC hook draws a fresh seed in
// 0 … 2^53 − 1 for every MC request and sends it in `want.mc.seed`; the document, and so the share code, keeps
// `"auto"`. A fixed seed is a number in the document and is not repeated in `want`.
import type { CalcInput, CalcRequest, CalcWant, View } from "./generated";

/** Largest seed: 2^53 − 1 (JSON-safe integer; server-api.md §4). */
export const MAX_SEED = Number.MAX_SAFE_INTEGER;

/** Source of random 32-bit words (crypto in the browser; injectable in tests). */
export type RandomWords = () => [number, number];

export const cryptoWords: RandomWords = () => {
  const a = new Uint32Array(2);
  crypto.getRandomValues(a);
  return [a[0] ?? 0, a[1] ?? 0];
};

/** A uniformly drawn 53-bit seed: 21 high bits and 32 low bits. */
export function drawSeed(words: RandomWords = cryptoWords): number {
  const [hi, lo] = words();
  return (hi >>> 11) * 2 ** 32 + (lo >>> 0);
}

export interface McOptions {
  parity?: boolean;
  breakdown?: boolean;
  views?: View[];
}

/** The `/api/calc` request of the MC hook: `want.mc` with the drawn seed when the document says `"auto"`. */
export function buildMcRequest(input: CalcInput, opts: McOptions = {}, words: RandomWords = cryptoWords): CalcRequest {
  const ev = input.scenario.eval;
  const mc: NonNullable<CalcWant["mc"]> = { runs: ev.mc_runs, series_bin_s: ev.series_bin_s };
  if (ev.seed === "auto") mc.seed = drawSeed(words);
  const want: CalcWant = { mc, parity: opts.parity ?? true };
  if (opts.breakdown != null) want.breakdown = opts.breakdown;
  if (opts.views) want.views = opts.views;
  return { input, want };
}

/** The `/api/calc` request of the EV hook (no `want.mc`). */
export function buildEvRequest(input: CalcInput, opts: McOptions = {}): CalcRequest {
  const want: CalcWant = {};
  if (opts.breakdown != null) want.breakdown = opts.breakdown;
  if (opts.views) want.views = opts.views;
  return { input, want };
}
