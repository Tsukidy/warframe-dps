// The document store (frontend.md §5): one `CalcInput` plus UI-only state. Every user action is a named action;
// document actions run as Immer recipes whose patches feed undo/redo (history.ts). UI state is not undoable.
//
// Feature tasks may implement the stubbed parts of actions (legality-aware weapon switching needs the candidates
// route); the action names and signatures are the contract.
import { create, type StoreApi, type UseBoundStore } from "zustand";
import { enablePatches, produce, produceWithPatches, type Draft } from "immer";
import type {
  BuffPick,
  CalcInput,
  CalcReport,
  Caster,
  ComboKey,
  CondOverride,
  ConditionMode,
  EnemySetup,
  EvalSettings,
  ImportReport,
  Message,
  ParamValue,
  Riven,
  SlotContent,
  SlotEntry,
  VariantDoc,
  View,
  WarframeDoc,
  WeaponItemDoc,
} from "../api/generated";
import { History } from "./history";
import { newDocument } from "./defaults";
import { setPath } from "./paths";
import type { Baseline } from "./baseline";

enablePatches();

/** A slot id (`gen0` … `subsumed`, server-api.md §4). */
export type SlotId = SlotEntry["slot"];

/** UI-only state (not part of the document, not undoable). */
export interface UiState {
  /** The build name shown in the top bar. */
  buildName: string;
  /** Selected attack (index into the report's attacks) and layer view. */
  selectedAttack: number;
  view: View;
  /** Chart or table per chart id (frontend.md §11 "show as table"). */
  chartModes: Record<string, "chart" | "table">;
  /** Open/collapsed panels by id. */
  panels: Record<string, boolean>;
  /** Pane shown on narrow screens (frontend.md §3). */
  narrowPane: "loadout" | "results" | "target";
  /** Auto-MC switch (frontend.md §5 `useCalcMc` "can be switched off"). */
  autoMc: boolean;
  /** Bumped by "re-roll" to force a new MC request with a fresh seed. */
  mcNonce: number;
}

export interface WeaponSwitchResult {
  /** Slots removed by the switch (shown in the undoable toast). */
  dropped: SlotEntry[];
}

/**
 * The result of decoding the boot code (`?b=` in the URL or the stored last build; frontend.md §9). The shell shows
 * it as a dismissible notice; features may read it too and dismiss it with `setBootDecode(null)`.
 */
export interface BootDecode {
  /** The code that was decoded. */
  code: string;
  /** The decode report (present when the decode succeeded). */
  report?: ImportReport;
  /** Why the decode failed (the document fell back to a new build), and the server's issues. */
  error?: Message;
  issues?: Message[];
}

/** Whether a decode report has anything to show. */
export function reportHasEntries(r: ImportReport | undefined): boolean {
  if (!r) return false;
  return (
    r.unmapped.length + r.converted.length + r.defaults_applied.length + r.retired_items.length + r.warnings.length > 0
  );
}

export interface StoreState {
  doc: CalcInput;
  /** False until the boot sequence put a real document in (URL code, last build or defaults). */
  ready: boolean;
  ui: UiState;
  baseline: Baseline | null;
  canUndo: boolean;
  canRedo: boolean;
  /** Changes on every document change (for subscribers that only need "something changed"). */
  revision: number;
  /** The boot code's decode result until dismissed (null: no code, or dismissed). */
  bootDecode: BootDecode | null;

  // Document lifecycle
  /** Replaces the whole document (URL code, import, optimizer "Apply"); undoable unless `record` is false. */
  replaceDoc: (doc: CalcInput, opts?: { record?: boolean; label?: string }) => void;
  /** Generic document edit for features (named actions below cover the common cases). */
  edit(label: string, recipe: (draft: Draft<CalcInput>) => void, coalesceKey?: string): void;

  /** Records (or with `null` dismisses) the boot code's decode result. */
  setBootDecode: (result: BootDecode | null) => void;

  // Weapon and slots (frontend.md §5)
  /**
   * Switches the weapon (frontend.md §5). Installed slots for which `keep(entry)` is true stay, the stance included
   * (the caller asks the candidates route which are legal for the new weapon; default: keep everything); Incarnon
   * perks (`inc2`-`inc5`) and the arch-gun arcane (`arcane1`) are always cleared, as are variant picks, weapon-item
   * settings, `melee_plans` and `pellet_hit`. The stance combo stays only while a kept stance holds it.
   */
  setWeapon(weapon: number, keep?: (entry: SlotEntry) => boolean): WeaponSwitchResult;
  placeItem: (slot: SlotId, content: SlotContent) => void;
  /** Moves the content of `from` to `to`; an occupied `to` swaps. */
  moveItem: (from: SlotId, to: SlotId) => void;
  removeItem: (slot: SlotId) => void;
  setItemRank: (slot: SlotId, rank: number | undefined) => void;
  setItemParam: (slot: SlotId, param: string, value: ParamValue | undefined) => void;
  setRiven: (slot: SlotId, riven: Riven) => void;
  setVariant: (variant: VariantDoc) => void;
  setWeaponItem: (patch: Partial<WeaponItemDoc>) => void;
  setStanceCombo: (combo: ComboKey | null) => void;
  setWarframe: (patch: Partial<WarframeDoc> | null) => void;

  // Scenario
  /** Sets a schema field by its path (`toggles.hit_location`, `eval.window_s`, `build.variant.progenitor`). */
  setToggle: (path: string, value: unknown) => void;
  setEnemy: (patch: Partial<EnemySetup>) => void;
  /** Adds or updates a buff pick (`null` removes it). */
  setBuff: (buff: number, pick: Omit<BuffPick, "buff"> | null) => void;
  setBuffCaster: (buff: number, caster: Caster | undefined) => void;
  setCustom: (id: string, value: number | null) => void;
  resetCustom: () => void;
  setConditionsMode: (mode: ConditionMode) => void;
  /** Sets (or with `null` clears) a per-effect condition override. */
  setOverride: (effect: string, override: CondOverride | null) => void;
  setEval: (patch: Partial<EvalSettings>) => void;

  // History
  undo: () => void;
  redo: () => void;
  clearHistory: () => void;

  // Baseline
  pinBaseline: (report: CalcReport | null) => void;
  setBaselineReport: (report: CalcReport) => void;
  clearBaseline: () => void;

  // UI
  setUi: (patch: Partial<UiState>) => void;
  setChartMode: (chart: string, mode: "chart" | "table") => void;
  togglePanel: (panel: string, open?: boolean) => void;
  reroll: () => void;
}

export const DEFAULT_UI: UiState = {
  buildName: "",
  selectedAttack: 0,
  view: "armor",
  chartModes: {},
  panels: {},
  narrowPane: "results",
  autoMc: true,
  mcNonce: 0,
};

/**
 * Slots a weapon switch always clears (frontend.md §5: Incarnon perks and arch-gun arcanes). The stance is not here:
 * a stance legal for the new weapon (same melee class) stays, like any other legal mod.
 */
const CLEARED_ON_SWITCH = new Set<SlotId>(["inc2", "inc3", "inc4", "inc5", "arcane1"]);

function slotIndex(slots: SlotEntry[], slot: SlotId): number {
  return slots.findIndex((s) => s.slot === slot);
}

/** Creates a store (tests create their own; the app uses `useAppStore`). */
export function createAppStore(initial?: { doc?: CalcInput; ready?: boolean; ui?: Partial<UiState> }) {
  const history = new History<CalcInput>();

  return create<StoreState>()((set, get) => {
    const sync = () => ({ canUndo: history.canUndo, canRedo: history.canRedo });

    const edit = (label: string, recipe: (draft: Draft<CalcInput>) => void, coalesceKey?: string) => {
      const [next, patches, inverse] = produceWithPatches(get().doc, recipe);
      if (patches.length === 0) return;
      history.record({ patches, inverse, label, ...(coalesceKey !== undefined ? { coalesceKey } : {}) });
      set((s) => ({ doc: next, revision: s.revision + 1, ...sync() }));
    };

    const editSlot = (label: string, slot: SlotId, f: (e: Draft<SlotEntry>) => void, coalesce = false) =>
      edit(
        label,
        (d) => {
          const i = slotIndex(d.build.slots, slot);
          const e = d.build.slots[i];
          if (e) f(e);
        },
        coalesce ? `${label}:${slot}` : undefined,
      );

    return {
      doc: initial?.doc ?? newDocument(),
      ready: initial?.ready ?? initial?.doc !== undefined,
      ui: { ...DEFAULT_UI, ...initial?.ui },
      baseline: null,
      canUndo: false,
      canRedo: false,
      revision: 0,
      bootDecode: null,

      setBootDecode(result) {
        set({ bootDecode: result });
      },

      replaceDoc(doc, opts = {}) {
        if (opts.record === false) {
          history.clear();
          set((s) => ({ doc, ready: true, revision: s.revision + 1, ...sync() }));
          return;
        }
        edit(opts.label ?? "replaceDoc", () => doc);
        set({ ready: true });
      },
      edit,

      setWeapon(weapon, keep) {
        const dropped: SlotEntry[] = [];
        const keepFn = keep ?? (() => true);
        edit("setWeapon", (d) => {
          d.build.weapon = weapon;
          const kept: SlotEntry[] = [];
          for (const e of d.build.slots) {
            if (!CLEARED_ON_SWITCH.has(e.slot) && keepFn(e)) kept.push(e);
            else dropped.push(JSON.parse(JSON.stringify(e)) as SlotEntry);
          }
          d.build.slots = kept;
          d.build.variant = {};
          delete d.build.weapon_item;
          // A combo belongs to its stance; stanceless default combos depend on the weapon's class.
          if (!kept.some((e) => e.slot === "stance")) delete d.build.stance_combo;
          if (d.scenario.toggles.melee_plans) delete d.scenario.toggles.melee_plans;
          if (d.scenario.toggles.pellet_hit) delete d.scenario.toggles.pellet_hit;
        });
        set((s) => ({ ui: { ...s.ui, selectedAttack: 0 } }));
        return { dropped };
      },
      placeItem(slot, content) {
        edit("placeItem", (d) => {
          const i = slotIndex(d.build.slots, slot);
          if (i >= 0) (d.build.slots[i] as SlotEntry).content = content;
          else d.build.slots.push({ slot, content });
        });
      },
      moveItem(from, to) {
        if (from === to) return;
        edit("moveItem", (d) => {
          const a = d.build.slots.find((s) => s.slot === from);
          if (!a) return;
          const b = d.build.slots.find((s) => s.slot === to);
          a.slot = to;
          if (b) b.slot = from;
        });
      },
      removeItem(slot) {
        edit("removeItem", (d) => {
          const i = slotIndex(d.build.slots, slot);
          if (i >= 0) d.build.slots.splice(i, 1);
        });
      },
      setItemRank(slot, rank) {
        editSlot(
          "setItemRank",
          slot,
          (e) => {
            if ("item" in e.content) {
              if (rank === undefined) delete e.content.item.rank;
              else e.content.item.rank = rank;
            }
          },
          true,
        );
      },
      setItemParam(slot, param, value) {
        editSlot(
          "setItemParam",
          slot,
          (e) => {
            if (!("item" in e.content)) return;
            const params = (e.content.item.params ??= {});
            if (value === undefined) delete params[param];
            else params[param] = value;
            if (Object.keys(params).length === 0) delete e.content.item.params;
          },
          true,
        );
      },
      setRiven(slot, riven) {
        get().placeItem(slot, { riven });
      },
      setVariant(variant) {
        edit("setVariant", (d) => {
          d.build.variant = variant;
        });
      },
      setWeaponItem(patch) {
        edit(
          "setWeaponItem",
          (d) => {
            const wi = { ...(d.build.weapon_item ?? {}), ...patch };
            for (const k of Object.keys(wi) as (keyof WeaponItemDoc)[]) if (wi[k] === undefined) delete wi[k];
            if (Object.keys(wi).length === 0) delete d.build.weapon_item;
            else d.build.weapon_item = wi;
          },
          "setWeaponItem",
        );
      },
      setStanceCombo(combo) {
        edit("setStanceCombo", (d) => {
          if (combo === null) delete d.build.stance_combo;
          else d.build.stance_combo = combo;
        });
      },
      setWarframe(patch) {
        edit("setWarframe", (d) => {
          if (patch === null) delete d.build.warframe;
          else d.build.warframe = { ...(d.build.warframe ?? {}), ...patch };
        });
      },

      setToggle(path, value) {
        edit("setToggle", (d) => setPath(d, path, value), `setToggle:${path}`);
      },
      setEnemy(patch) {
        edit(
          "setEnemy",
          (d) => {
            Object.assign(d.scenario.enemy, patch);
          },
          `setEnemy:${Object.keys(patch).sort().join(",")}`,
        );
      },
      setBuff(buff, pick) {
        edit(
          "setBuff",
          (d) => {
            const i = d.scenario.buffs.findIndex((b) => b.buff === buff);
            if (pick === null) {
              if (i >= 0) d.scenario.buffs.splice(i, 1);
            } else if (i >= 0) d.scenario.buffs[i] = { buff, ...pick };
            else d.scenario.buffs.push({ buff, ...pick });
          },
          `setBuff:${buff}`,
        );
      },
      setBuffCaster(buff, caster) {
        edit("setBuffCaster", (d) => {
          const b = d.scenario.buffs.find((x) => x.buff === buff);
          if (!b) return;
          if (caster === undefined) delete b.caster;
          else b.caster = caster;
        });
      },
      setCustom(id, value) {
        edit(
          "setCustom",
          (d) => {
            if (value === null) delete d.scenario.custom[id];
            else d.scenario.custom[id] = value;
          },
          `setCustom:${id}`,
        );
      },
      resetCustom() {
        edit("resetCustom", (d) => {
          d.scenario.custom = {};
        });
      },
      setConditionsMode(mode) {
        edit("setConditionsMode", (d) => {
          d.scenario.conditions.mode = mode;
        });
      },
      setOverride(effect, override) {
        edit("setOverride", (d) => {
          if (override === null) delete d.scenario.conditions.overrides[effect];
          else d.scenario.conditions.overrides[effect] = override;
        });
      },
      setEval(patch) {
        edit(
          "setEval",
          (d) => {
            Object.assign(d.scenario.eval, patch);
          },
          `setEval:${Object.keys(patch).sort().join(",")}`,
        );
      },

      undo() {
        const next = history.undo(get().doc);
        if (next) set((s) => ({ doc: next, revision: s.revision + 1, ...sync() }));
      },
      redo() {
        const next = history.redo(get().doc);
        if (next) set((s) => ({ doc: next, revision: s.revision + 1, ...sync() }));
      },
      clearHistory() {
        history.clear();
        set(sync());
      },

      pinBaseline(report) {
        set((s) => ({ baseline: { doc: s.doc, report, pinnedAt: Date.now() } }));
      },
      setBaselineReport(report) {
        set((s) => (s.baseline ? { baseline: { ...s.baseline, report } } : {}));
      },
      clearBaseline() {
        set({ baseline: null });
      },

      setUi(patch) {
        set((s) => ({ ui: { ...s.ui, ...patch } }));
      },
      setChartMode(chart, mode) {
        set((s) => ({ ui: produce(s.ui, (u) => void (u.chartModes[chart] = mode)) }));
      },
      togglePanel(panel, open) {
        set((s) => ({ ui: produce(s.ui, (u) => void (u.panels[panel] = open ?? !u.panels[panel])) }));
      },
      reroll() {
        set((s) => ({ ui: { ...s.ui, mcNonce: s.ui.mcNonce + 1 } }));
      },
    };
  });
}

export type AppStore = UseBoundStore<StoreApi<StoreState>>;

/** The app's store. */
export const useAppStore: AppStore = createAppStore();
