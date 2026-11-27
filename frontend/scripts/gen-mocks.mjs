// Builds the MSW mock digest `src/mocks/fixture.generated.json` from the mini fixture bundle `data/fixtures/mini`
// (frontend.md §2 "mocks/handlers.ts (MSW, built from data/fixtures/mini via a generated catalogue JSON)").
//
// The digest keeps the fixture's records in a slim, display-oriented form (names resolved through `names/en.json`,
// dense ids replaced by StableKeys); `src/mocks/handlers.ts` turns it into typed API responses. Records are read
// as data: nothing here interprets game mechanics. Extra fixture files (`mini/extra/*.json`, work-breakdown.md
// §2.2) are appended in file-name order, as the Rust loader does.
//
// Usage: node scripts/gen-mocks.mjs [--fixture DIR] [--out FILE]
import { existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const argv = process.argv.slice(2);
const opt = (name, dflt) => {
  const i = argv.indexOf(name);
  return i >= 0 && argv[i + 1] ? resolve(argv[i + 1]) : dflt;
};
const fixture = opt("--fixture", join(root, "../data/fixtures/mini"));
const out = opt("--out", join(root, "src/mocks/fixture.generated.json"));

const read = (rel) => JSON.parse(readFileSync(join(fixture, rel), "utf8"));
const readIf = (rel, dflt) => (existsSync(join(fixture, rel)) ? read(rel) : dflt);

const WEAPON_FILES = ["primary", "secondary", "melee", "archgun", "archmelee", "exalted", "companion", "modular"];
const ITEM_FILES = [
  "weapon_mod",
  "weapon_arcane",
  "frame_mod",
  "aura",
  "frame_arcane",
  "shard",
  "stance",
  "incarnon",
  "buff",
  "custom",
];
const DAMAGE_TYPES = [
  "IMP", "PUN", "SLA", "HEA", "COL", "ELE", "TOX", "BLA", "RAD", "GAS", "MAG", "VIR", "COR", "VOI", "TAU", "TRU",
  "CIN", "FIN",
];

const manifest = read("manifest.json");
const names = { ...readIf("names/en.json", {}) };
const rules = read("rules.json");
let weapons = WEAPON_FILES.flatMap((f) => readIf(`weapons/${f}.json`, []));
let items = ITEM_FILES.flatMap((f) => readIf(`items/${f}.json`, []));
let enemies = existsSync(join(fixture, "enemies"))
  ? readdirSync(join(fixture, "enemies"))
      .filter((f) => f.endsWith(".json"))
      .sort()
      .flatMap((f) => read(`enemies/${f}`))
  : [];
let sets = readIf("sets.json", []);
let frames = readIf("frames.json", []);
const extraDir = join(fixture, "extra");
if (existsSync(extraDir)) {
  for (const f of readdirSync(extraDir).filter((f) => f.endsWith(".json")).sort()) {
    const x = read(`extra/${f}`);
    weapons = weapons.concat(x.weapons ?? []);
    items = items.concat(x.items ?? []);
    enemies = enemies.concat(x.enemies ?? []);
    sets = sets.concat(x.sets ?? []);
    frames = frames.concat(x.frames ?? []);
    Object.assign(names, x.names ?? {});
  }
}

const name = (key) => names[key] ?? String(key).split("/").pop();
const kindSlug = (kind) => (typeof kind === "string" ? kind : Object.keys(kind)[0]);
const kindData = (kind) => (typeof kind === "string" ? {} : Object.values(kind)[0]);
const itemKeyById = new Map(items.map((it) => [it.id, it.key]));
const factions = rules.factions ?? [];
const perType = (arr) =>
  Object.fromEntries(DAMAGE_TYPES.map((t, i) => [t, Array.isArray(arr) ? (arr[i] ?? 0) : 0]));
const reload = (r) => {
  if (r == null || r === "none") return { kind: "none" };
  if (typeof r === "string") return { kind: r };
  const [kind, v] = Object.entries(r)[0];
  return { kind, time: v?.time ?? undefined };
};
const statsOf = (effects) => [
  ...new Set(
    (effects ?? []).flatMap((e) => [e.stat, e.element ? `element.${e.element}` : null].filter(Boolean)),
  ),
];

const digest = {
  generated_from: "data/fixtures/mini",
  meta: {
    data_version: manifest.data_version,
    game_baseline: manifest.game_baseline,
    schema_version: manifest.schema_version,
    created_at: manifest.created_at,
    sources: (manifest.sources ?? []).map((s) => ({
      source: `${s.source}:${s.id}`,
      revision: s.revision,
      ...(s.timestamp ? { fetched_at: s.timestamp } : {}),
    })),
    licences: (manifest.licences ?? []).map((l) => ({
      source: l.source,
      licence: l.licence,
      url: l.url,
      attribution: l.attribution,
    })),
    coverage: manifest.coverage ?? {},
  },
  weapons: weapons.map((w) => ({
    key: w.key,
    unique_name: w.unique_name,
    name: name(w.name),
    slot: w.slot,
    class: w.class,
    traits: w.traits ?? [],
    image: w.image != null,
    incarnon: w.incarnon != null || (w.traits ?? []).includes("incarnon"),
    ...(w.modular ? { modular: typeof w.modular === "string" ? w.modular : kindSlug(w.modular) } : {}),
    progenitor: w.progenitor != null,
    mastery: w.mastery ?? 0,
    max_rank: w.max_rank ?? 30,
    polarities: w.polarities ?? [],
    ...(w.exilus_polarity ? { exilus_polarity: w.exilus_polarity } : {}),
    trigger: w.trigger,
    magazine: w.magazine
      ? {
          size: w.magazine.size,
          infinite: !!w.magazine.infinite,
          ...(w.magazine.ammo_max != null ? { ammo_max: w.magazine.ammo_max } : {}),
        }
      : null,
    reload: reload(w.reload),
    attacks: (w.attacks ?? []).map((a) => ({
      key: a.key,
      name: name(a.name),
      roles: a.roles ?? [],
      shot: a.shot,
      ...(a.parent != null ? { parent: a.parent } : {}),
      base: perType(a.base),
      cc: a.cc,
      cd: a.cd,
      sc: a.sc,
      multishot: a.multishot,
      fire: {
        trigger: a.fire?.trigger ?? w.trigger,
        rate: a.fire?.rate ?? 1,
        ...(a.fire?.charge_time != null ? { charge_time: a.fire.charge_time } : {}),
      },
      ...(a.falloff ? { falloff: a.falloff } : {}),
      ...(a.radius != null ? { radius: a.radius } : {}),
      forced: a.forced ?? [],
      ...(a.is_heavy != null ? { is_heavy: a.is_heavy } : {}),
      co: { behaviour: a.co?.behaviour ?? "none", ratio: a.co?.ratio ?? 1, ...(a.co?.row != null ? { row: a.co.row } : {}) },
    })),
  })),
  items: items.map((it) => {
    const kind = kindSlug(it.kind);
    const kd = kindData(it.kind);
    return {
      key: it.key,
      unique_name: it.unique_name,
      name: name(it.name),
      kind,
      ...(it.polarity ? { polarity: it.polarity } : {}),
      drain: it.base_drain ?? 0,
      max_rank: it.max_rank ?? 0,
      rarity: it.rarity ?? "common",
      ...(it.set != null ? { set: it.set } : {}),
      image: it.image != null,
      status: it.status,
      stats: statsOf(it.effects),
      ...(kd.exilus ? { exilus: true } : {}),
      ...(kd.color ? { color: kd.color } : {}),
      ...(kd.subsumable ? { subsumable: true } : {}),
      params: (it.params ?? []).map((p) => ({
        key: p.key,
        kind: p.kind,
        ...(p.min != null ? { min: p.min } : {}),
        ...(p.max != null ? { max: p.max } : {}),
        ...(p.choices ? { choices: p.choices.map((c) => (typeof c === "string" ? { value: c, label: c } : c)) } : {}),
        default: p.default ?? 0,
        label: p.label ?? `param.${p.key}`,
      })),
      sources: it.sources ?? [],
      ...(it.upstream_text ? { upstream_text: it.upstream_text } : {}),
      incompatible: (it.incompat ?? []).map((id) => itemKeyById.get(id) ?? id),
      ...(kind === "buff" ? { buff: kd } : {}),
      ...(kind === "custom_stat" ? { custom: kd } : {}),
    };
  }),
  enemies: enemies.map((e) => ({
    key: e.key,
    name: name(e.name),
    faction: factions[e.faction]?.slug ?? String(e.faction),
    base_level: e.base_level,
    layers: [
      ...((e.overguard ?? 0) > 0 ? ["OG"] : []),
      ...((e.shield ?? 0) > 0 ? ["SH"] : []),
      "HP",
      ...((e.armor ?? 0) > 0 ? ["AR"] : []),
    ],
    base: { health: e.health ?? 0, shield: e.shield ?? 0, armor: e.armor ?? 0, overguard: e.overguard ?? 0 },
    eximus: { available: !(e.eximus?.off ?? true), ...(e.eximus?.health != null ? { health: e.eximus.health } : {}) },
    parts: (e.parts ?? []).map((p) => ({
      key: p.name,
      name: p.name,
      mult: p.part_mult ?? 1,
      weak_point: !!p.is_weak_point,
      head: !!p.is_head,
      head_crit: (p.headcrit ?? 1) > 1,
    })),
    immune: e.immune ?? [],
    innate_dr: { OG: e.innate_dr?.[0] ?? 0, SH: e.innate_dr?.[1] ?? 0, HP: e.innate_dr?.[2] ?? 0 },
    status_dr: e.status_dr ?? 0,
    flags: e.flags ?? [],
    party_scaled: e.party_scaling != null,
    faction_mods: perType(factions[e.faction]?.type_mods ?? DAMAGE_TYPES.map(() => 1)),
  })),
  frames: frames.map((f) => ({
    key: f.key,
    name: name(f.name),
    ...(f.variant_of != null ? { variant_of: f.variant_of } : {}),
    polarities: f.polarities ?? [],
    aura_polarities: f.aura_polarities ?? [],
    ...(f.exilus_polarity ? { exilus_polarity: f.exilus_polarity } : {}),
    aura_slots: f.aura_slots ?? 1,
    abilities: (f.abilities ?? []).map((a) => ({
      name: name(a.name),
      ...(a.self_buff != null ? { self_buff: itemKeyById.get(a.self_buff) ?? a.self_buff } : {}),
    })),
    ...(f.helminth_host != null ? { helminth_host: itemKeyById.get(f.helminth_host) ?? f.helminth_host } : {}),
    image: f.image != null,
  })),
  factions: factions.map((f) => ({ slug: f.slug, name: name(f.name) })),
  sets: sets.map((s) => ({
    id: s.id,
    name: name(s.name),
    members: (s.members ?? []).map((id) => itemKeyById.get(id) ?? id),
  })),
  slot_layouts: Object.fromEntries((rules.slot_layouts ?? []).map((l) => [l.class, l.slots.map((s) => s.id)])),
  toggles: rules.toggles ?? [],
  riven_traits: (rules.riven_traits ?? []).map((t) => ({
    id: t.id,
    input: t.input,
    classes: Object.keys(t.coeff ?? {}),
    ...(t.template?.element ? { element: t.template.element } : {}),
  })),
  names,
};

const text = JSON.stringify(digest) + "\n";
if (!existsSync(out) || readFileSync(out, "utf8") !== text) writeFileSync(out, text);
