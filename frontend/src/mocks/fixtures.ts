// Typed API responses built from the mini fixture digest (`fixture.generated.json`, scripts/gen-mocks.mjs).
// Catalogue, schema and details carry fixture data; calculation results are synthetic placeholders (deterministic
// functions of the document) so the UI can be developed and tested before the engine exists. They are not engine
// numbers and no test may treat them as such.
import digestJson from "./fixture.generated.json";
import type {
  AttackDetail,
  AttackReport,
  BuffSchema,
  CalcInput,
  CalcReport,
  CalcRequest,
  Catalog,
  CustomStatSchema,
  DamageType,
  EnemyDetail,
  ItemDetail,
  ItemSummary,
  Meta,
  ParamView,
  ScenarioSchema,
  SlotLayoutSchema,
  StatusType,
  Summary,
  ToggleSchema,
  View,
  ViewReport,
  WeaponDetail,
  WeaponSlot,
  WeaponSummary,
  PerDamageType,
} from "../api/generated";

interface DigestAttack extends Omit<AttackDetail, "co" | "base"> {
  base: PerDamageType;
  co: AttackDetail["co"];
}
interface DigestWeapon extends WeaponSummary {
  unique_name: string;
  max_rank: number;
  polarities: WeaponDetail["polarities"];
  exilus_polarity?: WeaponDetail["exilus_polarity"];
  trigger: WeaponDetail["trigger"];
  magazine: WeaponDetail["magazine"] | null;
  reload: WeaponDetail["reload"];
  attacks: DigestAttack[];
}
interface DigestItem extends ItemSummary {
  unique_name: string;
  params: ParamView[];
  sources: string[];
  upstream_text?: string;
  incompatible: number[];
  buff?: { group: BuffSchema["group"]; inspiration_key: string | null; strength_scaled: boolean; subsumable: boolean; toggle: string | null; default_caster?: "self" | "ally"; duration_scaled?: boolean };
  custom?: { inspiration_key: string };
}
interface DigestEnemy {
  key: number;
  name: string;
  faction: string;
  base_level: number;
  layers: Catalog["enemies"][number]["layers"];
  base: EnemyDetail["base"];
  eximus: EnemyDetail["eximus"];
  parts: EnemyDetail["parts"];
  immune: StatusType[];
  innate_dr: EnemyDetail["innate_dr"];
  status_dr: number;
  flags: EnemyDetail["flags"];
  party_scaled: boolean;
  faction_mods: PerDamageType;
}
export interface Digest {
  meta: {
    data_version: string;
    game_baseline: string;
    schema_version: number;
    created_at: string;
    sources: Meta["sources"];
    licences: Meta["licences"];
    coverage: Meta["coverage"];
  };
  weapons: DigestWeapon[];
  items: DigestItem[];
  enemies: DigestEnemy[];
  frames: Catalog["frames"];
  factions: Catalog["factions"];
  sets: Catalog["sets"];
  slot_layouts: Record<string, SlotLayoutSchema["slots"]>;
  toggles: ToggleSchema[];
  riven_traits: { id: string; input: string; classes: string[]; element?: DamageType }[];
  names: Record<string, string>;
}

export const digest = digestJson as unknown as Digest;

export function meta(): Meta {
  const m = digest.meta;
  return {
    app_id: "warframe-dps",
    app_version: "0.0.0-mock",
    api_version: 1,
    schema_version: m.schema_version,
    data_version: m.data_version,
    game_baseline: m.game_baseline,
    bundle: { id: "embedded", source: "embedded", created_at: m.created_at },
    sources: m.sources,
    licences: m.licences,
    coverage: m.coverage,
    config: { optimizer_threads: 3, interactive_threads: 4, mc_runs_default: 1000, images_fetch: false },
    local_keys: false,
    features: { refresh: false },
  };
}

export function catalog(): Catalog {
  return {
    data_version: digest.meta.data_version,
    weapons: digest.weapons.map((w) => ({
      key: w.key,
      name: w.name,
      slot: w.slot,
      class: w.class,
      traits: w.traits,
      image: false,
      incarnon: w.incarnon,
      ...(w.modular ? { modular: w.modular } : {}),
      progenitor: w.progenitor,
      mastery: w.mastery,
    })),
    items: digest.items.map((i) => ({
      key: i.key,
      name: i.name,
      kind: i.kind,
      ...(i.polarity ? { polarity: i.polarity } : {}),
      drain: i.drain,
      max_rank: i.max_rank,
      rarity: i.rarity,
      ...(i.set != null ? { set: i.set } : {}),
      image: false,
      status: i.status,
      stats: i.stats,
      ...(i.exilus ? { exilus: true } : {}),
      ...(i.color ? { color: i.color } : {}),
      ...(i.subsumable ? { subsumable: true } : {}),
    })),
    enemies: digest.enemies.map((e) => ({
      key: e.key,
      name: e.name,
      faction: e.faction,
      base_level: e.base_level,
      layers: e.layers,
    })),
    frames: digest.frames.map((f) => ({ ...f, image: false })),
    factions: digest.factions,
    sets: digest.sets,
    names: digest.names,
  };
}

const VIEWS: View[] = ["health", "armor", "shield", "overguard", "layered"];
const LAYOUT_SLOTS: WeaponSlot[] = ["primary", "secondary", "melee", "arch_gun", "arch_melee", "exalted", "companion"];

export function schema(): ScenarioSchema {
  const buffs: BuffSchema[] = digest.items
    .filter((i) => i.kind === "buff" && i.buff)
    .map((i) => ({
      key: i.key,
      id: i.unique_name,
      name: i.name,
      group: i.buff?.group ?? ("ability" as const),
      ...(i.buff?.toggle ? { toggle: i.buff.toggle } : {}),
      ...(i.buff?.inspiration_key ? { inspiration_key: i.buff.inspiration_key } : {}),
      strength_scaled: i.buff?.strength_scaled ?? false,
      duration_scaled: i.buff?.duration_scaled ?? false,
      default_caster: i.buff?.default_caster ?? "self",
      subsumable: i.buff?.subsumable ?? false,
      params: i.params,
    }));
  const custom: CustomStatSchema[] = digest.items
    .filter((i) => i.kind === "custom_stat")
    .map((i) => {
      const p = i.params[0];
      return {
        id: i.custom?.inspiration_key ?? i.unique_name,
        key: i.key,
        label: `custom.${i.custom?.inspiration_key ?? i.key}`,
        unit: p?.kind === "percent" ? "percent" : "number",
        min: p?.min ?? 0,
        max: p?.max ?? 0,
        inspiration_key: i.custom?.inspiration_key ?? "",
      };
    });
  const slot_layouts: ScenarioSchema["slot_layouts"] = {};
  for (const s of LAYOUT_SLOTS) {
    const slots = digest.slot_layouts[s] ?? digest.slot_layouts.primary ?? [];
    slot_layouts[s] = { slots };
  }
  return {
    toggles: digest.toggles,
    buffs,
    melee_plans: [],
    custom_stats: custom,
    riven_traits: digest.riven_traits.map((t) => ({
      id: t.id,
      label: `riven.trait.${t.id}`,
      unit: t.input === "faction_multiplier" ? "faction_multiplier" : t.input === "absolute" ? "absolute" : "percent",
      classes: t.classes as ScenarioSchema["riven_traits"][number]["classes"],
      ...(t.element ? { element: t.element } : {}),
    })),
    slot_layouts,
    views: VIEWS.map((v) => ({ view: v, label: `view.${v}.label`, help: `view.${v}.help` })),
    conditions: {
      modes: ["all_max", "none", "simulated"],
      classes: (["a", "b", "c"] as const).map((c) => ({ id: c, label: `condclass.${c}.label`, help: `condclass.${c}.help` })),
    },
  };
}

export function weaponDetail(key: number): WeaponDetail | undefined {
  const w = digest.weapons.find((x) => x.key === key);
  if (!w) return undefined;
  const slots = digest.slot_layouts[w.slot] ?? digest.slot_layouts.primary ?? [];
  return {
    key: w.key,
    unique_name: w.unique_name,
    name: w.name,
    slot: w.slot,
    class: w.class,
    trigger: w.trigger,
    ...(w.magazine ? { magazine: w.magazine } : {}),
    reload: w.reload,
    attacks: w.attacks,
    groups: [],
    stances: [],
    zoom: [],
    slot_layout: { slots, arcane_slots: slots.filter((s) => s.startsWith("arcane")).length },
    polarities: w.polarities,
    ...(w.exilus_polarity ? { exilus_polarity: w.exilus_polarity } : {}),
    max_rank: w.max_rank,
    mastery: w.mastery,
    display: {},
    image: false,
    provenance: [{ source: "fixture", revision: digest.meta.data_version }],
  };
}

export function itemDetail(key: number): ItemDetail | undefined {
  const i = digest.items.find((x) => x.key === key);
  if (!i) return undefined;
  const set = i.set != null ? digest.sets.find((s) => s.id === i.set) : undefined;
  return {
    key: i.key,
    unique_name: i.unique_name,
    name: i.name,
    kind: i.kind,
    ...(i.polarity ? { polarity: i.polarity } : {}),
    drain: i.drain,
    max_rank: i.max_rank,
    rarity: i.rarity,
    ...(set ? { set: { ...set, bonus_text: [] } } : {}),
    params: i.params,
    effects: [],
    status: i.status,
    sources: i.sources,
    ...(i.upstream_text ? { upstream_text: i.upstream_text } : {}),
    incompatible: i.incompatible,
    compat: { slots: [], categories: [] },
    image: false,
  };
}

export function enemyDetail(key: number): EnemyDetail | undefined {
  const e = digest.enemies.find((x) => x.key === key);
  if (!e) return undefined;
  return {
    key: e.key,
    name: e.name,
    faction: e.faction,
    base_level: e.base_level,
    base: e.base,
    eximus: e.eximus,
    parts: e.parts,
    immune: e.immune,
    caps: {},
    innate_dr: e.innate_dr,
    status_dr: e.status_dr,
    flags: e.flags,
    party_scaled: e.party_scaled,
    faction_mods: e.faction_mods,
  };
}

function summary(x: number): Summary {
  return {
    mean: x,
    sd: x * 0.05,
    ci95: [x * 0.99, x * 1.01],
    median: x,
    p5: x * 0.92,
    p25: x * 0.97,
    p75: x * 1.03,
    p95: x * 1.08,
    min: x * 0.85,
    max: x * 1.15,
  };
}

/** A synthetic placeholder report: per attack, the base damage sum × fire rate, scaled by the slot count. */
export function calcReport(req: CalcRequest, code: string): CalcReport {
  const input = req.input;
  const w = digest.weapons.find((x) => x.key === input.build.weapon);
  const enemy = digest.enemies.find((x) => x.key === input.scenario.enemy.enemy);
  const mods = 1 + 0.25 * input.build.slots.length;
  const window = input.scenario.eval.window_s;
  const bins = Math.max(1, Math.round(window / Math.max(input.scenario.eval.series_bin_s, 0.1)));
  const attacks: AttackReport[] = (w?.attacks ?? []).map((a) => {
    const base = Object.values(a.base).reduce((s, v) => s + v, 0);
    const rate = typeof a.fire.rate === "number" ? a.fire.rate : 1;
    const dps = base * rate * a.multishot * mods;
    const per_type = Object.fromEntries(Object.entries(a.base).filter(([, v]) => v > 0).map(([k, v]) => [k, v * rate * mods]));
    const views: ViewReport[] = (enemy?.layers.includes("AR") ? (["health", "armor"] as View[]) : (["health"] as View[])).map((view) => ({
      view,
      ev: {
        dps_burst: dps * 1.1,
        dps_sustained: dps * 0.9,
        dps_window: view === "armor" ? dps * 0.7 : dps,
        per_shot: base * mods,
        per_type,
        per_source: [{ source: "direct", dps }],
        stacks: [],
        distinct_mean: 0,
        crit_tiers: [0.5, 0.4, 0.1, 0, 0, 0, 0, 0],
        series: Array.from({ length: bins }, () => dps),
        pending_dot: 0,
        hit_fresh: base * mods,
        ammo_window: window * rate,
      },
      ...(req.want.mc
        ? {
            mc: {
              runs: req.want.mc.runs ?? input.scenario.eval.mc_runs,
              seed: req.want.mc.seed ?? (typeof input.scenario.eval.seed === "number" ? input.scenario.eval.seed : 0),
              dps_window: summary(dps),
              series: Array.from({ length: bins }, () => ({ mean: dps, p10: dps * 0.9, p90: dps * 1.1 })),
              uptime: [],
              crit_hist: [0.5, 0.4, 0.1, 0, 0, 0, 0, 0],
              breakdown: [{ source: "direct", dps }],
              pending_dot: 0,
            },
          }
        : {}),
    }));
    return { key: a.key, name: a.name, views };
  });
  return {
    data_version: digest.meta.data_version,
    code,
    weapon: {
      attacks: (w?.attacks ?? []).map((a) => {
        const block = { cc: a.cc, cd: a.cd, sc: a.sc, multishot: a.multishot, fire_rate: a.fire.rate, forced: a.forced };
        return { key: a.key, name: a.name, base: block, modded: block, per_type: {}, total: Object.values(a.base).reduce((s, v) => s + v, 0) };
      }),
      extra: {},
    },
    enemy: {
      level: input.scenario.enemy.level,
      pools: { OG: enemy?.base.overguard ?? 0, SH: enemy?.base.shield ?? 0, HP: enemy?.base.health ?? 0 },
      armor0: enemy?.base.armor ?? 0,
      armor_eff: enemy?.base.armor ?? 0,
      dr: 0,
      innate_dr: enemy?.innate_dr ?? { OG: 0, SH: 0, HP: 0 },
      immune: enemy?.immune ?? [],
      faction_mods: {},
      party: input.scenario.enemy.party,
    },
    attacks,
    groups: [],
    conditions: [],
    touched_rows: [],
    violations: [],
    warnings: [],
    timing_us: 1,
  };
}

/** A mock share code: `wfd1-mock-` + base64url of the JSON (the real codec is Rust, AD-18). */
export function encodeMock(input: CalcInput): string {
  const json = JSON.stringify(input);
  const b64 = btoa(String.fromCharCode(...new TextEncoder().encode(json)));
  return `wfd1-mock-${b64.replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "")}`;
}

export function decodeMock(text: string): CalcInput | undefined {
  const m = /wfd1-mock-([A-Za-z0-9_-]+)/.exec(text);
  if (!m?.[1]) return undefined;
  try {
    const b64 = m[1].replace(/-/g, "+").replace(/_/g, "/");
    const bytes = Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
    return JSON.parse(new TextDecoder().decode(bytes)) as CalcInput;
  } catch {
    return undefined;
  }
}
