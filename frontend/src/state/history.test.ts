import { describe, expect, it } from "vitest";
import { produceWithPatches, enablePatches } from "immer";
import { History, HISTORY_LIMIT } from "./history";
import { createAppStore } from "./store";
import { newDocument } from "./defaults";

enablePatches();

describe("History", () => {
  it("undoes and redoes patches", () => {
    const h = new History<{ n: number }>();
    let s = { n: 0 };
    const [s1, p1, i1] = produceWithPatches(s, (d) => void (d.n = 1));
    h.record({ patches: p1, inverse: i1, label: "a" });
    s = s1;
    expect(h.canUndo).toBe(true);
    s = h.undo(s) ?? s;
    expect(s.n).toBe(0);
    expect(h.canRedo).toBe(true);
    s = h.redo(s) ?? s;
    expect(s.n).toBe(1);
    expect(h.undo({ n: 1 })).toEqual({ n: 0 });
    expect(h.undo({ n: 0 })).toBeUndefined();
  });

  it("keeps at most 200 steps", () => {
    const h = new History<{ n: number }>();
    let s = { n: 0 };
    for (let i = 1; i <= HISTORY_LIMIT + 25; i++) {
      const [next, p, inv] = produceWithPatches(s, (d) => void (d.n = i));
      h.record({ patches: p, inverse: inv, label: "x" });
      s = next;
    }
    expect(h.size).toBe(200);
    let steps = 0;
    for (;;) {
      const prev = h.undo(s);
      if (!prev) break;
      s = prev;
      steps++;
    }
    expect(steps).toBe(200);
    expect(s.n).toBe(25);
  });

  it("coalesces steps with the same key inside the window", () => {
    const h = new History<{ n: number }>(200, 800);
    let s = { n: 0 };
    for (const [i, at] of [
      [1, 0],
      [2, 300],
      [3, 600],
      [4, 2000],
    ] as const) {
      const [next, p, inv] = produceWithPatches(s, (d) => void (d.n = i));
      h.record({ patches: p, inverse: inv, label: "type", coalesceKey: "n" }, at);
      s = next;
    }
    expect(h.size).toBe(2);
    s = h.undo(s) ?? s;
    expect(s.n).toBe(3);
    s = h.undo(s) ?? s;
    expect(s.n).toBe(0);
  });
});

describe("store undo/redo", () => {
  it("undoes document actions in order and a new action clears redo", () => {
    const s = createAppStore({ doc: newDocument(1204, 9001) });
    const st = () => s.getState();
    st().placeItem("gen0", { item: { item: 3301 } });
    st().setEnemy({ level: 50 });
    st().undo();
    expect(st().doc.scenario.enemy.level).toBe(1);
    expect(st().doc.build.slots).toHaveLength(1);
    expect(st().canRedo).toBe(true);
    st().redo();
    expect(st().doc.scenario.enemy.level).toBe(50);
    st().undo();
    st().removeItem("gen0");
    expect(st().canRedo).toBe(false);
    st().undo();
    st().undo();
    expect(st().doc.build.slots).toHaveLength(0);
    expect(st().canUndo).toBe(false);
  });

  it("records replaceDoc as one step, or clears history when not recorded", () => {
    const s = createAppStore({ doc: newDocument(1204, 9001) });
    s.getState().placeItem("gen0", { item: { item: 3301 } });
    s.getState().replaceDoc(newDocument(1300, 9002));
    expect(s.getState().doc.build.weapon).toBe(1300);
    s.getState().undo();
    expect(s.getState().doc.build.weapon).toBe(1204);
    expect(s.getState().doc.build.slots).toHaveLength(1);
    s.getState().replaceDoc(newDocument(1400, 9003), { record: false });
    expect(s.getState().canUndo).toBe(false);
    expect(s.getState().ready).toBe(true);
  });

  it("does not record actions that change nothing", () => {
    const s = createAppStore({ doc: newDocument(1204, 9001) });
    s.getState().removeItem("gen3");
    s.getState().moveItem("gen0", "gen0");
    expect(s.getState().canUndo).toBe(false);
  });
});
