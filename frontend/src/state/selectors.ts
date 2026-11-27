// Selectors for `useAppStore(selector)`: components subscribe to the smallest slice they render, so a slot change
// re-renders the grid cell, results and charts only (frontend.md §12 "Re-render scope").
import type { SlotEntry } from "../api/generated";
import type { SlotId, StoreState } from "./store";

export const selectDoc = (s: StoreState) => s.doc;
export const selectReady = (s: StoreState) => s.ready;
export const selectBuild = (s: StoreState) => s.doc.build;
export const selectScenario = (s: StoreState) => s.doc.scenario;
export const selectWeapon = (s: StoreState) => s.doc.build.weapon;
export const selectEnemy = (s: StoreState) => s.doc.scenario.enemy;
export const selectToggles = (s: StoreState) => s.doc.scenario.toggles;
export const selectEval = (s: StoreState) => s.doc.scenario.eval;
export const selectSlots = (s: StoreState) => s.doc.build.slots;
export const selectUi = (s: StoreState) => s.ui;
export const selectView = (s: StoreState) => s.ui.view;
export const selectBaseline = (s: StoreState) => s.baseline;
export const selectCanUndo = (s: StoreState) => s.canUndo;
export const selectCanRedo = (s: StoreState) => s.canRedo;

/** The entry of one slot (stable reference while that slot is unchanged). */
export const selectSlot =
  (slot: SlotId) =>
  (s: StoreState): SlotEntry | undefined =>
    s.doc.build.slots.find((e) => e.slot === slot);
