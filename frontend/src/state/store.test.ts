import { describe, expect, it } from "vitest";
import { createAppStore } from "./store";
import { newDocument } from "./defaults";
import type { SlotEntry } from "../api/generated";

const item = (key: number) => ({ item: { item: key } });

function store() {
  return createAppStore({ doc: newDocument(1204, 9001) });
}

describe("store actions", () => {
  it("places, swaps and removes slot content", () => {
    const s = store();
    s.getState().placeItem("gen0", item(3301));
    s.getState().placeItem("gen1", item(3302));
    expect(s.getState().doc.build.slots).toEqual([
      { slot: "gen0", content: item(3301) },
      { slot: "gen1", content: item(3302) },
    ]);
    s.getState().placeItem("gen0", item(3303));
    expect(s.getState().doc.build.slots[0]).toEqual({ slot: "gen0", content: item(3303) });
    s.getState().moveItem("gen0", "gen1");
    const bySlot = (slot: string) => s.getState().doc.build.slots.find((e) => e.slot === slot)?.content;
    expect(bySlot("gen1")).toEqual(item(3303));
    expect(bySlot("gen0")).toEqual(item(3302));
    s.getState().moveItem("gen1", "gen5");
    expect(bySlot("gen5")).toEqual(item(3303));
    expect(bySlot("gen1")).toBeUndefined();
    s.getState().removeItem("gen5");
    expect(s.getState().doc.build.slots).toHaveLength(1);
  });

  it("sets item rank and params and drops empty params", () => {
    const s = store();
    s.getState().placeItem("gen0", item(3301));
    s.getState().setItemRank("gen0", 3);
    s.getState().setItemParam("gen0", "element", "HEA");
    expect(s.getState().doc.build.slots[0]?.content).toEqual({ item: { item: 3301, rank: 3, params: { element: "HEA" } } });
    s.getState().setItemParam("gen0", "element", undefined);
    s.getState().setItemRank("gen0", undefined);
    expect(s.getState().doc.build.slots[0]?.content).toEqual(item(3301));
  });

  it("writes toggles by schema path, relative to the scenario or the build", () => {
    const s = store();
    s.getState().setToggle("toggles.hit_location", "head");
    s.getState().setToggle("eval.window_s", 30);
    s.getState().setToggle("conditions.mode", "none");
    s.getState().setToggle("build.variant.progenitor", { element: "TOX", v10: 600 });
    s.getState().setToggle("toggles.heavy", { start_combo: 4 });
    const d = s.getState().doc;
    expect(d.scenario.toggles.hit_location).toBe("head");
    expect(d.scenario.eval.window_s).toBe(30);
    expect(d.scenario.conditions.mode).toBe("none");
    expect(d.build.variant.progenitor).toEqual({ element: "TOX", v10: 600 });
    expect(d.scenario.toggles.heavy).toEqual({ start_combo: 4 });
  });

  it("adds, updates and removes buffs, custom stats and overrides", () => {
    const s = store();
    s.getState().setBuff(3601, { params: { value: 0.3 } });
    s.getState().setBuff(3601, { params: { value: 0.5 } });
    s.getState().setBuff(3602, {});
    expect(s.getState().doc.scenario.buffs).toEqual([{ buff: 3601, params: { value: 0.5 } }, { buff: 3602 }]);
    s.getState().setBuffCaster(3601, { ally: { strength: 2, duration: 1 } });
    expect(s.getState().doc.scenario.buffs[0]?.caster).toEqual({ ally: { strength: 2, duration: 1 } });
    s.getState().setBuff(3601, null);
    expect(s.getState().doc.scenario.buffs).toEqual([{ buff: 3602 }]);

    s.getState().setCustom("crit_chance", 0.2);
    expect(s.getState().doc.scenario.custom).toEqual({ crit_chance: 0.2 });
    s.getState().setCustom("crit_chance", null);
    expect(s.getState().doc.scenario.custom).toEqual({});

    s.getState().setOverride("3301:0", { force: false });
    expect(s.getState().doc.scenario.conditions.overrides).toEqual({ "3301:0": { force: false } });
    s.getState().setOverride("3301:0", null);
    expect(s.getState().doc.scenario.conditions.overrides).toEqual({});
  });

  it("switching weapons keeps legal slots and clears perks, the second arcane and variant picks", () => {
    const s = store();
    s.getState().placeItem("gen0", item(3301));
    s.getState().placeItem("gen1", item(3302));
    s.getState().placeItem("inc2", { perk: { index: 0 } });
    s.getState().placeItem("arcane1", item(3500));
    s.getState().setVariant({ progenitor: { element: "HEA", v10: 600 } });
    s.getState().setStanceCombo("neutral");
    const { dropped } = s.getState().setWeapon(1300, (e: SlotEntry) => e.slot !== "gen1");
    const d = s.getState().doc;
    expect(d.build.weapon).toBe(1300);
    expect(d.build.slots).toEqual([{ slot: "gen0", content: item(3301) }]);
    expect(dropped.map((e) => e.slot).sort()).toEqual(["arcane1", "gen1", "inc2"]);
    expect(d.build.variant).toEqual({});
    expect(d.build.stance_combo).toBeUndefined();
    // The switch is one undo step.
    s.getState().undo();
    expect(s.getState().doc.build.weapon).toBe(1204);
    expect(s.getState().doc.build.slots).toHaveLength(4);
  });

  it("switching weapons keeps a legal stance and its combo, and drops an illegal one with the combo", () => {
    const s = store();
    s.getState().placeItem("stance", item(3600));
    s.getState().setStanceCombo("neutral");
    let { dropped } = s.getState().setWeapon(1300);
    expect(dropped).toEqual([]);
    expect(s.getState().doc.build.slots).toEqual([{ slot: "stance", content: item(3600) }]);
    expect(s.getState().doc.build.stance_combo).toBe("neutral");

    ({ dropped } = s.getState().setWeapon(1204, (e: SlotEntry) => e.slot !== "stance"));
    expect(dropped.map((e) => e.slot)).toEqual(["stance"]);
    expect(s.getState().doc.build.slots).toEqual([]);
    expect(s.getState().doc.build.stance_combo).toBeUndefined();
  });

  it("updates the enemy and eval settings", () => {
    const s = store();
    s.getState().setEnemy({ level: 100, steel_path: true });
    s.getState().setEval({ seed: 42, mc_runs: 500 });
    expect(s.getState().doc.scenario.enemy).toMatchObject({ enemy: 9001, level: 100, steel_path: true });
    expect(s.getState().doc.scenario.eval).toMatchObject({ seed: 42, mc_runs: 500 });
  });

  it("keeps UI state out of the document and out of history", () => {
    const s = store();
    s.getState().setUi({ view: "health", buildName: "Test" });
    s.getState().setChartMode("dps", "table");
    s.getState().togglePanel("frame");
    expect(s.getState().ui).toMatchObject({ view: "health", buildName: "Test", chartModes: { dps: "table" }, panels: { frame: true } });
    expect(s.getState().canUndo).toBe(false);
    s.getState().reroll();
    expect(s.getState().ui.mcNonce).toBe(1);
  });

  it("pins a baseline with the current document", () => {
    const s = store();
    s.getState().pinBaseline(null);
    const b = s.getState().baseline;
    expect(b?.doc).toBe(s.getState().doc);
    s.getState().placeItem("gen0", item(3301));
    expect(s.getState().baseline?.doc.build.slots).toHaveLength(0);
    s.getState().clearBaseline();
    expect(s.getState().baseline).toBeNull();
  });
});
