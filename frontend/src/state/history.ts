// Patch-based undo/redo (frontend.md §5): every document action records the Immer patches and their inverse;
// at most 200 steps are kept. Consecutive edits with the same coalesce key within `coalesceMs` merge into one
// step (typing in a number field is one undo step).
import { applyPatches, type Patch } from "immer";

export const HISTORY_LIMIT = 200;

export interface HistoryEntry {
  patches: Patch[];
  inverse: Patch[];
  /** Action name, for debugging and future "undo <action>" labels. */
  label: string;
  coalesceKey?: string;
  at: number;
}

export class History<T> {
  private past: HistoryEntry[] = [];
  private future: HistoryEntry[] = [];

  constructor(
    readonly limit = HISTORY_LIMIT,
    readonly coalesceMs = 800,
  ) {}

  get canUndo(): boolean {
    return this.past.length > 0;
  }

  get canRedo(): boolean {
    return this.future.length > 0;
  }

  get size(): number {
    return this.past.length;
  }

  /** Records a step. A new step clears the redo stack. */
  record(entry: Omit<HistoryEntry, "at">, now = Date.now()): void {
    if (entry.patches.length === 0) return;
    this.future = [];
    const prev = this.past[this.past.length - 1];
    if (
      prev &&
      entry.coalesceKey !== undefined &&
      prev.coalesceKey === entry.coalesceKey &&
      now - prev.at <= this.coalesceMs
    ) {
      prev.patches = [...prev.patches, ...entry.patches];
      prev.inverse = [...entry.inverse, ...prev.inverse];
      prev.at = now;
      return;
    }
    this.past.push({ ...entry, at: now });
    if (this.past.length > this.limit) this.past.splice(0, this.past.length - this.limit);
  }

  /** Applies the inverse of the last step to `state`; returns the new state, or undefined when nothing to undo. */
  undo(state: T): T | undefined {
    const e = this.past.pop();
    if (!e) return undefined;
    this.future.push(e);
    return applyPatches(state as object, e.inverse) as T;
  }

  /** Re-applies the last undone step. */
  redo(state: T): T | undefined {
    const e = this.future.pop();
    if (!e) return undefined;
    this.past.push(e);
    return applyPatches(state as object, e.patches) as T;
  }

  clear(): void {
    this.past = [];
    this.future = [];
  }
}
