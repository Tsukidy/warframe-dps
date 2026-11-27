// Paths into the document. Toggle descriptors name their field as a dotted path (`toggles.hit_location`,
// `eval.window_s`, `conditions.mode`, `build.variant.progenitor`; server-api.md §5.6, domain-model.md §9.3): a path
// starting with `build.` is relative to the CalcInput, any other path to its `scenario`.
import type { CalcInput } from "../api/generated";

function root(path: string): { base: "build" | "scenario"; parts: string[] } {
  const parts = path.split(".").filter((p) => p.length > 0);
  if (parts[0] === "build") return { base: "build", parts: parts.slice(1) };
  if (parts[0] === "scenario") return { base: "scenario", parts: parts.slice(1) };
  return { base: "scenario", parts };
}

/** Reads the value at `path` (undefined when absent). */
export function getPath(doc: CalcInput, path: string): unknown {
  const { base, parts } = root(path);
  let cur: unknown = doc[base];
  for (const p of parts) {
    if (cur === null || typeof cur !== "object") return undefined;
    cur = (cur as Record<string, unknown>)[p];
  }
  return cur;
}

/**
 * Writes `value` at `path` in a draft document (use inside an Immer recipe). Missing intermediate objects are
 * created; `undefined` deletes the field (optional fields take their defaults on the server).
 */
export function setPath(draft: CalcInput, path: string, value: unknown): void {
  const { base, parts } = root(path);
  if (parts.length === 0) throw new Error(`empty document path: ${path}`);
  let cur = draft[base] as unknown as Record<string, unknown>;
  for (const p of parts.slice(0, -1)) {
    const next = cur[p];
    if (next === null || typeof next !== "object") cur[p] = {};
    cur = cur[p] as Record<string, unknown>;
  }
  const last = parts[parts.length - 1] as string;
  if (value === undefined) delete cur[last];
  else cur[last] = value;
}
