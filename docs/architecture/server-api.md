Status: reviewed (architecture)

# Server and HTTP API (`wfd-server`, binary `warframe-dps`)

One Rust binary serves the JSON API and the built frontend on localhost and opens the browser (CLAUDE.md). This
document defines startup behaviour, every route with its request and response shapes, the server-sent events, static
serving, configuration, logging and security.

Contents: 1 Process and startup · 2 Conventions · 3 Route table · 4 Shared document shapes · 5 Catalogue and schema
routes · 6 Calculation routes · 7 Build-code routes · 8 Optimizer routes and SSE · 9 Data routes and SSE · 10 Images
and static files · 11 Execution model · 12 Configuration, logging, security

---

## 1. Process and startup

### 1.1 Command line (clap 4)

```text
warframe-dps [serve] [--port N] [--no-browser] [--listen ADDR] [--data-dir DIR] [--bundle FILE] [--threads N]
                     [--log-level LEVEL]
warframe-dps calc --code <wfd1-…> | --input calc.json [--mc-runs N] [--json]   # headless calculation (tests, scripts)
warframe-dps optimize --request req.json [--node-budget N] [--json]            # headless optimizer, NDJSON OptEvents
warframe-dps import <warframe-damage.com URL> [--json]                         # prints CalcInput + ImportReport
warframe-dps data status | check | refresh [--curated DIR] | list | activate <id> | verify
```

### 1.2 `serve` startup sequence

1. Read configuration (§12.1); initialize logging.
2. Load data: the newest bundle in `<data_dir>/bundles/` whose `schema_version` equals the binary's, else the
   embedded bundle (`--bundle FILE` forces one). A corrupt user bundle is logged and skipped. Decode target < 100 ms.
3. Build `AppState` (§11): `ArcSwap<GameData>`, thread pools, job registry, HTTP client (images, data refresh only).
4. Bind `127.0.0.1:<port>` (default 7878) and `[::1]:<port>` when IPv6 is available. If the port is taken, request
   `GET http://127.0.0.1:<port>/api/meta`: if it answers with `app_id = "warframe-dps"`, another instance is running,
   so open the browser on it and exit 0. Otherwise try the next 10 ports, then an OS-assigned port. `--listen ADDR`
   binds another address explicitly and prints a warning (LAN use is opt-in).
5. Print `warframe-dps <version> — http://127.0.0.1:<port>/ (data <data_version>)`.
6. Open the browser with `webbrowser::open` unless `--no-browser` or `open_browser = false`.
7. Serve until Ctrl-C (graceful shutdown: cancel jobs, flush logs).

---

## 2. Conventions

| Rule | Definition |
|---|---|
| Format | JSON (`application/json; charset=utf-8`); enums are `snake_case` strings; maps are objects with string keys |
| Ids | items, weapons and enemies are referenced by `key` (StableKey, integer); dense ids never leave the server |
| Units | model units: fractions for chances and bonuses (0.3 = 30 %), seconds, HP; the UI converts for display |
| Types | every shape below is a `wfd-model` type with `#[derive(TS)]`; `cargo xtask gen-ts` writes `frontend/src/api/generated/*.ts`; CI fails when stale. Shapes are given in TypeScript notation, which is exactly the generated form |
| Errors | non-2xx responses have body `{ "error": Message }`; `Message = { key: MessageKey, args: Record<string, string \| number> }`; keys are i18n keys (`frontend/src/i18n/en.json`) |
| Status codes | 200 ok · 202 job accepted · 204 no content · 304 not modified · 400 malformed JSON (`api.bad_json`) · 404 unknown key (`api.not_found`) · 409 conflict (`opt.job_running`, `data.refresh_running`) · 413 body over 1 MB · 421 bad Host · 403 bad Origin · 422 invalid input (`api.invalid` with `issues: Message[]`) · 500 internal (`api.internal` with `error_id`) |
| Versioning | `CalcInput.v` (document version, currently 1); `/api/meta.schema_version` (bundle) and `api_version` (this API, currently 1) |
| Caching | catalogue and detail routes send `ETag: "<data_version>"`; clients may send `If-None-Match` |

---

## 3. Route table

| Method | Path | Request | Response |
|---|---|---|---|
| GET | `/api/meta` | — | `Meta` |
| GET | `/api/catalog` | — | `Catalog` (ETag) |
| GET | `/api/weapons/{key}` | — | `WeaponDetail` |
| GET | `/api/items/{key}` | — | `ItemDetail` |
| GET | `/api/enemies/{key}` | — | `EnemyDetail` |
| GET | `/api/schema` | — | `ScenarioSchema` |
| POST | `/api/candidates` | `CandidatesRequest` | `Candidate[]` |
| POST | `/api/calc` | `CalcRequest` | `CalcReport` |
| POST | `/api/timeline` | `TimelineRequest` | `Timeline` |
| POST | `/api/explain` | `ExplainRequest` | `Explanation` |
| POST | `/api/build/encode` | `CalcInput` | `{ code: string, path: string }` |
| POST | `/api/build/decode` | `{ text: string }` (our code, our URL, or a warframe-damage.com URL) | `DecodeResponse` |
| POST | `/api/build/export-inspiration` | `CalcInput` | `{ url: string }` or 422 (oracle harness; hidden in the UI) |
| POST | `/api/optimize` | `OptRequest` | 202 `{ job: string }` · 409 `opt.job_running` with `args.job` |
| GET | `/api/optimize/{job}/events` | header `Last-Event-ID` optional | `text/event-stream` of `OptEvent` |
| GET | `/api/optimize/{job}` | — | `JobStatus<OptResult>` |
| DELETE | `/api/optimize/{job}` | — | 202; the job ends with `cancelled` carrying its anytime result |
| GET | `/api/data/status` | — | `DataStatus` |
| POST | `/api/data/check` | — | `DataCheck` |
| POST | `/api/data/refresh` | `{ curated?: "embedded" }` | 202 `{ job: string }` · 409 `data.refresh_running` |
| GET | `/api/data/refresh/{job}/events` | — | `text/event-stream` of `DataEvent` |
| DELETE | `/api/data/refresh/{job}` | — | 202 |
| POST | `/api/data/activate` | `{ bundle: string }` (`"embedded"` or a bundle id from `DataStatus.bundles`) | `DataStatus` |
| GET | `/api/img/{key}` | — | image bytes of the weapon, item or frame with that StableKey, or 204 when unavailable |
| GET | `/*` | — | embedded frontend; unknown non-`/api` paths return `index.html` |

---

## 4. Shared document shapes

```ts
// CalcInput: build + scenario (domain-model.md §9). Also the content of share codes.
// x: uniqueName of every session-local key (≥ 2^31) the document uses (AD-18); omitted when empty.
type CalcInput = { v: 1; build: BuildDoc; scenario: ScenarioDoc; x?: Record<string /* key */, string /* uniqueName */> };

type BuildDoc = {                                     // spec 09 §2.1
  weapon: number;                                   // StableKey
  weapon_item?: { rank?: number; catalyst?: boolean; capacity_override?: number;
                  polarities?: Partial<Record<SlotId, Polarity>>; genesis_installed?: boolean };
  variant: {
    kitgun?: { grip: number; loader: number };
    zaw?: { grip: number; link: number };
    progenitor?: { element: DamageType; v10: number };        // tenths of a percent: 250-579 or 600 (09 E-9.70)
  };
  slots: SlotEntry[];                                 // non-empty slots only
  stance_combo?: string;                              // ComboKey
  warframe?: { frame?: number; reactor?: boolean; polarities?: Partial<Record<SlotId, Polarity>>;
               frame_complete?: boolean };
  companion_set_mods?: number[];                      // declared companion / robotic-weapon set pieces (09 E-9.40)
};
type SlotId = `gen${0|1|2|3|4|5|6|7}` | "exilus" | "stance" | "arcane0" | "arcane1" | `inc${2|3|4|5}`
            | `fgen${0|1|2|3|4|5|6|7}` | "fexilus" | "aura0" | "aura1" | "farcane0" | "farcane1"
            | `shard${0|1|2|3|4}` | "subsumed";
type SlotEntry = { slot: SlotId; content: SlotContent };
type SlotContent =
  | { item: { item: number; rank?: number; params?: Record<string, ParamValue> } }
  | { riven: { class: RivenClass; traits: { trait: string; value: number }[] } }
  | { perk: { index: number } }
  | { shard: { item: number; tauforged: boolean } }
  | { subsumed: { ability: number; replaced?: string } };
type Violation = { key: string /* 09 rule key, e.g. "cap.exceeded" */; severity: "illegal" | "illegal_unverified" | "warning";
                   slots: SlotId[]; args: Record<string, string | number> };
type ParamValue = number | string | boolean;

type ScenarioDoc = {
  enemy: { enemy: number; level: number; steel_path: boolean; eximus: boolean; party: 1 | 2 | 3 | 4 };
  toggles: Toggles;
  buffs: { buff: number; params?: Record<string, ParamValue>;
           caster?: "self" | { ally: { strength: number } } }[];   // 04 E-4.10; default "self" (Strength = T-20)
  custom: Record<string, number>;                     // custom stat id → value (model units)
  conditions: { mode: "all_max" | "none" | "simulated";
                overrides: Record<string /* EffectKey */, { force: boolean } | { stacks: number }
                                                         | { uptime: { u: number; stacks?: number } }> };
  eval: { window_s: number; ttk_horizon_s: number; mc_runs: number;
          seed: "auto" | number;                        // T-29; integer 0 … 2^53 − 1; "auto" = fresh seed per MC request
          series_bin_s: number };
  profile?: "game" | { compat: string[] };            // compat: CompatFlags names; oracle harness only
};
type Toggles = {
  hit_location: "body" | "head" | "weak_point";      // T-03
  co_multiplicative: boolean;                         // T-04
  co_status_count: { mode: "expected" | "weapon_types" | "manual"; n?: number };   // T-05
  external_viral: number | null;                      // T-06
  armor_strip: number;                                // T-07
  stealth: { rank: number } | null;                   // T-08
  heavy: { start_combo: number } | null;              // T-09
  light_combo: number | null;                         // T-10
  ability_combo: number | null;                       // T-11
  sniper_combo: number | null;                        // T-12
  strength: number;                                   // T-20
  wp_cc_multiplicative: boolean;                      // T-21
  combine_groups: number[];                           // T-22
  distance_m: number;                                 // T-27
  unique_dr: boolean;                                 // T-28
  capacity?: "ignore" | "current" | "any_forma" | { budget: number };   // proposed T-30 (09 E-9.15), default any_forma
  melee_plan?: MeleePlanSel;                          // proposed T-32 (07 §8.1, OQ-722), default "auto"
  zoom_level?: number | null;                         // index into the weapon's zoom levels (04 D-406), default null
};
type MeleePlanSel = "auto" | "light_loop" | "slide_loop" | { heavy_loop: { policy: "full_regen" | "back_to_back" } }
  | { build_spend: { k: number } } | "tennokai" | { slam_loop: { t_slam: number } } | { heavy_slam_loop: { t_slam: number } }
  | { glaive_cycle: { throw: "charged" | "quick" } };
type Message = { key: string; args: Record<string, string | number>; severity?: "info" | "warning" | "error" };
```

Example (Braton Prime vs Lancer level 100 Steel Path, the spec 00 §3.8 build; keys illustrative):

```json
{ "v": 1,
  "build": { "weapon": 1204, "variant": {},
    "slots": [
      { "slot": "gen0", "content": { "item": { "item": 3301 } } },
      { "slot": "gen1", "content": { "item": { "item": 3302 } } },
      { "slot": "gen2", "content": { "item": { "item": 3303 } } },
      { "slot": "gen3", "content": { "item": { "item": 3304 } } },
      { "slot": "gen4", "content": { "item": { "item": 3305 } } },
      { "slot": "gen5", "content": { "item": { "item": 3306 } } },
      { "slot": "gen6", "content": { "item": { "item": 3307 } } },
      { "slot": "gen7", "content": { "item": { "item": 3308 } } } ] },
  "scenario": {
    "enemy": { "enemy": 9001, "level": 100, "steel_path": true, "eximus": false, "party": 1 },
    "toggles": { "hit_location": "body", "co_multiplicative": false, "co_status_count": { "mode": "expected" },
                 "external_viral": null, "armor_strip": 0, "stealth": null, "heavy": null, "light_combo": null,
                 "ability_combo": null, "sniper_combo": null, "strength": 1.0, "wp_cc_multiplicative": false,
                 "combine_groups": [], "distance_m": 0, "unique_dr": true },
    "buffs": [], "custom": {},
    "conditions": { "mode": "all_max", "overrides": {} },
    "eval": { "window_s": 20, "ttk_horizon_s": 60, "mc_runs": 1000, "seed": 0, "series_bin_s": 1 } } }
```

A document that uses a session-local key (an item new since the embedded bundle, added by an in-app refresh) carries
its uniqueName; the decoder resolves `x` entries by uniqueName first:

```json
{ "v": 1, "build": { "weapon": 2147483651, "variant": {}, "slots": [] }, "scenario": { "…": "…" },
  "x": { "2147483651": "/Lotus/Weapons/Tenno/LongGuns/NewRifle/NewRifleWeapon" } }
```

Defaults: every `Toggles` field and every `eval` field has the default of domain-model.md §9.3; the server accepts
missing fields and fills defaults (`#[serde(default)]`), so old share codes keep working.

---

## 5. Catalogue and schema routes

### 5.1 `GET /api/meta`

```ts
type Meta = {
  app_id: "warframe-dps"; app_version: string; api_version: 1;
  schema_version: number; data_version: string; game_baseline: string;
  bundle: { id: string; source: "embedded" | "user"; created_at: string };
  sources: { source: string; revision: string; fetched_at?: string }[];
  licences: { source: string; licence: string; url: string; attribution: string }[];
  coverage: Record<string /* item kind */, Record<EffectStatus, number>>;
  config: { optimizer_threads: number; interactive_threads: number; mc_runs_default: number; images_fetch: boolean };
  local_keys: boolean;                                   // the active bundle has session-local keys (AD-18)
  features: { refresh: boolean };
};
```

### 5.2 `GET /api/catalog`

One response with every summary the pickers need (about 0.5 MB, fetched once per `data_version`):

```ts
type Catalog = {
  data_version: string;
  weapons: { key: number; name: string; slot: WeaponSlot; class: WeaponClass; traits: string[];
             image: boolean /* GET /api/img/{key} */; incarnon: boolean; modular?: "kitgun" | "zaw"; progenitor: boolean;
             mastery: number }[];
  items: { key: number; name: string; kind: ItemKind; polarity?: Polarity; drain: number; max_rank: number;
           rarity: Rarity; set?: number; image: boolean; status: EffectStatus; stats: string[] /* stat keys for search by stat */;
           exilus?: boolean; color?: ShardColor; subsumable?: boolean }[];
  enemies: { key: number; name: string; faction: string; base_level: number;
             layers: ("OG" | "SH" | "HP" | "AR")[] }[];
  frames: { key: number; name: string; variant_of?: number; polarities: Polarity[]; aura_polarities: Polarity[];
            exilus_polarity?: Polarity; aura_slots: number; abilities: { name: string; self_buff?: number }[];
            helminth_host?: number; image: boolean }[];     // AD-26, 09 §2.2 and §10
  factions: { slug: string; name: string }[];
  sets: { id: number; name: string; members: number[] }[];
  names: Record<string /* NameKey */, string>;          // English now; per-locale later
};
```

### 5.3 `GET /api/weapons/{key}`

```ts
type WeaponDetail = {
  key: number; unique_name: string; name: string; slot: WeaponSlot; class: WeaponClass; trigger: Trigger;
  magazine?: { size: number; ammo_max?: number; infinite: boolean; battery?: { delay: number; delay_empty: number; rate: number } };
  reload: { kind: "standard" | "per_shell" | "regenerate" | "none"; time?: number };
  attacks: { key: { slug: string; idx: number }; name: string; roles: string[]; shot: ShotType; parent?: number;
             base: Record<DamageType, number>; cc: number; cd: number; sc: number; multishot: number;
             fire: { trigger: Trigger; rate: number; charge_time?: number; burst?: { count: number; delay: number } };
             falloff?: { start: number; end: number; min_factor: number }; radius?: number; forced: StatusType[];
             is_heavy?: boolean; incarnon_charges?: number; co: { behaviour: string; ratio: number; row?: number } }[];
  groups: { members: number[]; combinable: boolean; summed: boolean }[];
  incarnon?: { tiers: { tier: number; perks: { index: number; key: number; name: string; effects_text: Message[] }[] }[] };
  modular?: { kind: "kitgun" | "zaw"; grips: PartSummary[]; loaders?: PartSummary[]; links?: PartSummary[] };
  progenitor?: { elements: DamageType[]; min: number; max: number; default: number };
  stances: { key: number; name: string;
              combos: { key: string; kind: ComboType; name: string; source: "unique" | "stance" | "stanceless" }[] }[];
                                                         // compatible stances (melee), combos resolved per E-7.0b
  melee?: { class: string; plans: string[] /* 07 §8.1 plans valid for this weapon */; heavy?: number /* attack index */;
            ability_combo: boolean };
  zoom: { name: string; effects_text: Message[] }[];  // zoom levels (04 D-406)
  slot_layout: { slots: SlotId[]; arcane_slots: number };
  polarities: Polarity[]; exilus_polarity?: Polarity; max_rank: number; mastery: number;
  display: Record<string, number | string>;            // accuracy, noise, flight speed, range …
  image: boolean; provenance: { source: string; revision: string }[];
};
```

### 5.4 `GET /api/items/{key}`

```ts
type ItemDetail = {
  key: number; unique_name: string; name: string; kind: ItemKind; polarity?: Polarity; drain: number;
  max_rank: number; rarity: Rarity; set?: { id: number; name: string; members: number[]; bonus_text: Message[] };
  params: { key: string; kind: "percent" | "number" | "choice" | "bool"; min?: number; max?: number;
            choices?: { value: string; label: string }[]; default: ParamValue; label: string }[];
  effects: EffectView[];                               // structured, rendered by the UI through i18n
  status: EffectStatus; sources: string[]; upstream_text?: string; incompatible: number[];
  compat: { slots: string[]; trigger?: Trigger; categories: string[] };
  image: boolean;
};
type EffectView = { id: string; target: { stat?: string; element?: string; faction?: string; mult?: string; mechanic?: string };
                    value_max: number | null; per: string[]; condition?: { class: "a" | "b" | "c" | "schedule" | "formula";
                    trigger?: string; chance?: number; stacks?: { max: number; duration: number } };
                    scope_text: Message[]; note?: string };
```

### 5.5 `GET /api/enemies/{key}`

```ts
type EnemyDetail = {
  key: number; name: string; faction: string; base_level: number;
  base: { health: number; shield: number; armor: number; overguard: number };
  eximus: { health?: number; shield?: number; available: boolean };
  parts: { name: string; mult: number; weak_point: boolean; head: boolean; head_crit: boolean }[];
  immune: StatusType[]; caps: Partial<Record<StatusType, number>>;
  innate_dr: { OG: number; SH: number; HP: number }; status_dr: number;
  special_dr?: { kind: string; note: Message };      // attenuation, flat DR, adaptation (unique-DR warning)
  flags: string[]; party_scaled: boolean;
  faction_mods: Record<DamageType, number>;
};
```

### 5.6 `GET /api/schema`

Everything the schema-driven panels render (AD-17):

```ts
type ScenarioSchema = {
  toggles: { id: string /* "T-03" */; field: string /* path in ScenarioDoc */; group: string; label: string /* i18n key */;
             help: string; control: { kind: "bool" | "number" | "percent" | "select" | "optional_number";
             min?: number; max?: number; step?: number; options?: { value: string; label: string }[] };
             default: unknown; visible_if?: { weapon_slot?: WeaponSlot[]; attack_role?: string[]; trait?: string[] };
             advanced: boolean }[];
  buffs: { key: number; id: string; name: string; group: string; toggle?: string; inspiration_key?: string;
           strength_scaled: boolean /* caster Strength applies, 04 E-4.10 */; subsumable: boolean;
           params: ItemDetail["params"] }[];
  melee_plans: { id: string; label: string; help: string;
                 params: { key: string; kind: "number" | "select"; default: unknown; options?: string[] }[] }[];  // T-32
  custom_stats: { id: string; key: number; label: string; unit: "percent" | "number"; min: number; max: number;
                  inspiration_key: string }[];
  riven_traits: { id: string; label: string; unit: "percent" | "faction_multiplier" | "absolute";
                  classes: RivenClass[]; element?: DamageType; range?: { min: number; max: number } }[];
  slot_layouts: Record<WeaponSlot, { slots: SlotId[] }>;
  views: { view: View; label: string; help: string }[];
  conditions: { modes: string[]; classes: { id: "a" | "b" | "c"; label: string; help: string }[] };
};
```

### 5.7 `POST /api/candidates`

```ts
type CandidatesRequest = { input: CalcInput; slot: SlotId; query?: string; stat?: string };
type Candidate = { key: number; legal: true } | { key: number; legal: false; reason: Message };
```

Returns every item of the slot's kind for the weapon (blocked ones with the reason: incompatible with an installed
mod, wrong trigger, already installed, duplicate set rule), filtered by `query` (localized substring and fuzzy) or
`stat` (stat key), ordered by relevance. Legality is `wfd-data::legality`, the same code the decoder and the
optimizer use.

---

## 6. Calculation routes

### 6.1 `POST /api/calc`

```ts
type CalcRequest = {
  input: CalcInput;
  want: { mc?: { runs?: number; seed?: number /* required when eval.seed is "auto": the frontend draws it */;
                series_bin_s?: number }; parity?: boolean; breakdown?: boolean; trace?: boolean; views?: View[] };
};
type CalcReport = {
  data_version: string; code: string;                // share code of the inputs
  weapon: WeaponPanel; enemy: EnemyPanel;
  attacks: AttackReport[]; groups: GroupReport[];
  conditions: ConditionView[]; touched_rows: string[];
  violations: Violation[];                           // spec 09 rule violations; never block evaluation (09 §1.5)
  capacity?: { used: number; capacity: number; forma: number; umbra_forma: number; mode: string };   // 09 E-9.14-9.16
  trace?: TraceEntry[]; warnings: Message[]; timing_us: number;
};
type WeaponPanel = { attacks: { key: { slug: string; idx: number }; name: string;
    base: StatBlock; modded: StatBlock; per_type: Partial<Record<DamageType, number>>; total: number }[];
  magazine?: { base: number; modded: number }; incarnon_magazine?: number;
  reload?: { base: number; modded: number }; extra: Record<string, { base: number; modded: number }> };
type StatBlock = { cc: number; cd: number; sc: number; multishot: number; fire_rate: number; charge_time?: number;
                   burst_delay?: number; radius?: number; punch_through?: number; ammo_cost?: number;
                   falloff?: { start: number; end: number; min_factor: number }; forced: StatusType[] };
type EnemyPanel = { level: number; pools: { OG: number; SH: number; HP: number }; armor0: number; armor_eff: number;
                    dr: number; innate_dr: { OG: number; SH: number; HP: number }; immune: StatusType[];
                    special_dr?: Message; faction_mods: Partial<Record<DamageType, number>>; party: number };
type AttackReport = { key: { slug: string; idx: number }; name: string; views: ViewReport[] };
type ViewReport = { view: View; ev: ViewEv; mc?: ViewMc };
type ViewEv = { dps_burst: number; dps_sustained: number; dps_window: number; per_shot: number; per_magazine: number;
                per_type: Partial<Record<DamageType, number>>; per_source: { source: string; dps: number }[];
                stacks: { status: StatusType; mean: number; p_any: number; p_cap: number }[];
                distinct_mean: number; crit_tiers: number[] /* 8 bins: tier 0 … 6, 7 = "7 or more" */; series: number[];
                pending_dot: number;
                ttk?: TtkEv;                                   // layered view only (08 E-8.21)
                hit_fresh: number; hit_infight: number; ammo_window: number };   // 08 §10
type TtkEv = { band: "killed" | "extrapolated" | "never"; seconds: number | null /* null = never */; kappa: number;
               shots_to_kill?: number; ammo_to_kill?: number; shield_regen_possible: boolean };
type Summary = { mean: number; sd: number; ci95: [number, number]; median: number; p5: number; p25: number;
                 p75: number; p95: number; min: number; max: number };
type ViewMc = { runs: number; seed: number; dps_window: Summary;
                series: { mean: number; p10: number; p90: number }[];
                uptime: { status: StatusType; uptime: number[]; mean_stacks: number[] }[];
                crit_hist: number[] /* 8 bins as crit_tiers */; breakdown: { source: string; dps: number }[];
                pending_dot: number;
                ttk?: { kill_probability: number; median: number | null /* null = not killed */; mean_killed?: number;
                        p5: number | null; p95: number | null; shots_to_kill?: number; ammo_to_kill?: number;
                        shield_regen_possible: boolean };   // pass L, layered view only (08 E-8.20)
                parity?: { queue_dps: number; queue_dps_min: number; queue_dps_max: number; avg_direct: number;
                           avg_status: number; avg_total: number; median_total: number;
                           ttk?: number | null } };          // 08 E-8.22: Health, Shield, Overguard rows: pass L0 median;
                                                           // Armor row: pass L median (D-802); null = not killed
type GroupReport = { members: number[]; combined: boolean; views: ViewReport[] };
type ConditionView = { effect: string /* EffectKey */; item: number; class: ("a" | "b" | "c")[];
                       treatment: "on" | "off" | "expected" | "simulated" | "uptime" | "schedule" | "formula";
                       value: number; override?: unknown };
type TraceEntry = { probe: string; equation: string; value: number; inputs: { probe: string; value: number }[];
                    contributions: { effect: string; value: number }[]; rows: string[] };
```

Behaviour: EV for every attack, group and present view always (fast); MC only when `want.mc` is present (passes V
and L; pass L0 in addition when `want.parity` is set and the enemy has armor, 08 §6.4). The
frontend sends an EV request (debounced 120 ms) and a separate MC request (debounced 400 ms); superseded requests are
aborted by the browser, which drops the handler future and cancels the work (`CancelOnDrop`, §11). Rule violations
of spec 09 (incompatible mods, capacity, riven legality …) never block a calculation: they are returned in
`violations` (09 §1.5). 422 `api.invalid` is returned only for malformed documents (unknown key, retired item that
cannot be dropped, content impossible in its slot, such as a riven in `aura0`).

### 6.2 `POST /api/timeline`

```ts
type TimelineRequest = { input: CalcInput; attack: { slug: string; idx: number } | { group: number };
                         view: View; seed?: number; run?: number; limit?: number /* default 2000 hits */ };
type Timeline = { duration: number; hits: TimelineHit[]; reloads: { n: number; start: number; end: number }[];
                  heat_episodes: { ignition: number; end: number }[];   // 08 §8.9
                  truncated: boolean };
type TimelineHit = { n: number; t: number; damage: number; crit_tier: number; crit_mult?: number; armor_eff?: number;
                     armor_reduction?: number;
                     stance_mult?: number; heavy_combo_mult?: number; extra_hits: { kind: string; damage: number }[];
                     statuses_applied: StatusType[]; tick_values: Partial<Record<StatusType, number>>;
                     stacks: Partial<Record<StatusType, number>>; flags: ("first" | "last" | "after_reload")[] };
```

### 6.3 `POST /api/explain`

```ts
type ExplainRequest = { input: CalcInput; probe: string };     // engine.md §11.2 probe path
type Explanation = { probe: string; value: number; equation: string; formula: Message;
                     inputs: { probe: string; label: Message; value: number }[];
                     contributions: { effect: string; item?: number; label: Message; value: number }[];
                     rows: { id: string; text: Message }[] };
```

---

## 7. Build-code routes

```ts
// POST /api/build/encode   body: CalcInput
type EncodeResponse = { code: string /* "wfd1-…" */; path: string /* "/?b=wfd1-…", relative: the port varies */ };
// POST /api/build/decode   body: { text }
type DecodeResponse = {
  input: CalcInput; source: "wfd1" | "inspiration_lz" | "inspiration_base64";
  report: { unmapped: { field: string; value: string }[]; converted: Message[];
            defaults_applied: Message[];        // e.g. inspiration o.v.stucks = 0 (I:feat §15)
            retired_items: number[]; warnings: Message[] };
};
```

Code format (AD-18): `wfd1-` + base64url (no padding) of raw DEFLATE of the canonical compact JSON of `CalcInput`
(keys sorted, defaults omitted, StableKeys as integers, `x` with the uniqueName of every session-local key,
`eval.seed = "auto"` stored as such). `wfd-buildcode` is the only encoder and decoder; versions are migrated by
`wfd-buildcode::migrate`, and sample codes of every released version are kept as tests forever.

Sharing: the server may run on another port after a restart (§1.2 port fallback), so the share action copies the bare
code; the receiving user pastes it into the Import dialog (or opens `/?b=<code>` on their own instance). `path` is
relative and used only for the page's own URL bar (replaceState).

---

## 8. Optimizer routes and SSE

### 8.1 `POST /api/optimize`

```ts
type OptRequest = {
  input: CalcInput;
  objective: { metric: "dps_window" | "dps_burst" | "dps_sustained" | "ttk" | "hit_infight" | "hit_fresh";   // 08 §10
               view: "auto" | View; attacks: { attack: { slug: string; idx: number }; weight: number }[];
               heavy?: { start_combo: number }; window_s?: number };
  constraints: { locked: SlotId[]; must_include: number[]; prohibited: number[];
                 elements: { require: DamageType[]; forbid: DamageType[] };
                 rules: ("use_weak_point" | "no_faction" | "no_on_kill" | "keep_progenitor" | "use_incarnon"
                         | "allow_next_shot" | "exclude_draft" | "exclude_stale")[];
                 capacity: { mode: "ignore" | "current" | "any_forma" | { budget: number };   // 09 E-9.15 (proposed T-30)
                             umbra_forma: boolean; ranks: "max" | "allow_lower" /* proposed T-31 */ };
                 riven: "use_given" | "none" | { search: { traits: 2 | 3 | 4; allow_negative: boolean; quality: number } } };
  groups: { exilus: boolean; weapon_arcanes: boolean; stance: boolean; melee_plan: boolean; incarnon: boolean;
            frame_mods: number /* 0-8 general frame slots */; frame_exilus: boolean;
            aura: boolean; frame_arcanes: boolean;
            shards: "off" | "any" | { owned: { item: number; tauforged: boolean; count: number }[] };
            modular_parts: boolean; progenitor: boolean };
  buffs: { allowed: { buff: number; params?: Record<string, ParamValue> }[]; portfolio: boolean; strength?: number };
  search: { mode: "exact" | "balanced" | "quick"; certify: boolean; time_limit_s?: number; node_budget?: number; keep: number;
            mc_confirm?: { finalists: number; runs: number; seed: number }; threads?: number; context_beam: number;
            preset: "default" | "inspiration_compatible" };
};
```

Response 202 `{ "job": "opt-7f3a9c" }`. One optimizer job runs at a time (it uses the whole optimizer pool); a second
request gets 409 `opt.job_running` with `args.job`, and the UI offers to cancel the running job. Finished jobs are
kept for one hour.

### 8.2 `GET /api/optimize/{job}/events` (server-sent events)

Each `OptEvent` (optimizer.md §9) is one SSE message whose `event:` field is the variant name in `snake_case` and
whose `data:` is the variant's JSON; `id:` is a per-job sequence number. Keep-alive comments every 15 s. The server
keeps a ring buffer of the last 1,000 events per job: a client reconnecting with `Last-Event-ID` receives everything
after that id, then live events. `progress` events are throttled to 4 per second.

```text
event: started
id: 1
data: {"job":"opt-7f3a9c","plan":{"positions":[{"group":"main","pool":36},{"group":"exilus","pool":9}],"composition_sets":2154,"excluded":[{"key":3390,"reason":{"key":"opt.excluded.unmodelled","args":{}}}]}}

event: progress
id: 17
data: {"phase":"main_search","fraction":0.42,"evaluated":18234000,"bounded":4410021,"pruned":3902112,"rate_per_s":1520000,"elapsed_ms":12000,"eta_s":16.5,"gap":null}

event: incumbent
id: 18
data: {"rank":1,"build":{"input":{"v":1,"build":{},"scenario":{}},"code":"wfd1-…","score":38211.4,"key":{"band":0,"primary":38211.4,"tie":-412.0,"tie2":0.0},"mix":["COR","HEA"],"ev_mc_deviation":false,"draft_items":[]}}

event: done
id: 95
data: {"result":{"best":{},"alternatives":[],"per_mix":[],"proof":{"proven_optimal":{"scope":"main_given_context","assumes_fast_error":0.005}},"leave_one_out":[],"next_best":[],"condition_sensitivity":31002.7,"buff_table":null,"excluded":[],"stats":{}}}
```

Event names: `started`, `phase`, `progress`, `incumbent`, `finalist`, `log`, `done`, `cancelled`, `failed`. After
`done`, `cancelled` or `failed` the server closes the stream.

### 8.3 `GET /api/optimize/{job}` and `DELETE`

```ts
type JobStatus<R> = { job: string; state: "running" | "done" | "cancelled" | "failed"; started_at: string;
                      last_progress?: unknown; result?: R; error?: Message };
```

`DELETE` sets the job's cancel flag (latency ≤ 100 ms); the job finishes with `cancelled` carrying its anytime
result (optimizer.md §9).

---

## 9. Data routes and SSE

```ts
type DataStatus = {
  active: { id: string; source: "embedded" | "user"; data_version: string; game_baseline: string; created_at: string };
  bundles: { id: string; data_version: string; created_at: string; size: number; active: boolean }[];
  refresh_job?: string;
};
type DataCheck = { changed: boolean; pe_build_label: string; wiki_changed_modules: string[]; wfcd_latest_tag: string;
                   checked_at: string };
type DataEvent =
  | { event: "step"; step: "fetch" | "parse" | "join" | "derive" | "effects" | "validate" | "emit"; done: number; total: number }
  | { event: "log"; message: Message }
  | { event: "report"; report: { diff: DataDiffSummary; new_items: number[]; stale: number[]; auto_patched: number[];
                                 conflicts: Message[]; errors: Message[]; warnings: Message[] }; bundle_id: string }
  | { event: "done"; bundle_id: string } | { event: "cancelled" } | { event: "failed"; error: Message };
type DataDiffSummary = { weapons_changed: number; items_changed: number; enemies_changed: number;
                         samples: { path: string; old: string; new: string }[] };
```

Refresh writes the new bundle but does not activate it; the UI shows the report and calls `POST /api/data/activate`
(pipeline.md §9.2). Activation swaps `ArcSwap<GameData>`; in-flight requests and jobs keep their snapshot. The data
SSE stream uses the same framing as §8.2 (`event:` = `step`, `log`, `report`, `done`, `cancelled`, `failed`).

---

## 10. Images and static files

- `GET /api/img/{key}`: `key` is a StableKey (integer); the server looks up the entity's `textureLocation` in the
  active bundle (no client-supplied path ever reaches the file system or the network). Served from
  `<data_dir>/images/<sha256 of textureLocation>.png` if present; otherwise, when `images.fetch = true`, fetched once
  from `https://content.warframe.com/PublicExport/<textureLocation>` (pipeline.md §3.2) through the polite client
  (one request at a time), cached, and served with `Cache-Control: max-age=31536000, immutable`. No texture, offline
  or disabled: 204, and the UI shows a generated glyph tile. Tests, e2e runs and visual baselines run with
  `images.fetch = false`, so they never depend on the network.
- Static frontend: `wfd-server/build.rs` copies `frontend/dist/` into `$OUT_DIR/web/` when `frontend/dist/index.html`
  exists, and otherwise writes a one-page placeholder `index.html` there that says how to build the frontend
  (`cargo:rerun-if-changed=../../frontend/dist`). The crate embeds that directory with
  `static WEB: include_dir::Dir = include_dir!("$OUT_DIR/web")` (include_dir interpolates environment variables, and
  `OUT_DIR` is set for crates with a build script), so the embedded folder always exists: `cargo build` and
  `cargo test` never need Node and never write into the source tree, and debug and release builds behave the same.
  A release build through `cargo xtask dist` builds the frontend first; `cargo xtask dist-smoke` fails if the
  placeholder was embedded. Hashed assets (`/assets/*`) get `Cache-Control: public, max-age=31536000, immutable`;
  `index.html` gets `no-cache`. MIME types via `mime_guess`. Any non-`/api` path that is not a file returns
  `index.html` (SPA routing: `/`, `/optimizer`, `/data`, `/about`).
- With feature `refresh`, `include_dir!` also embeds `data/curated/` and `data/generated/spec/` (the in-app pipeline's
  inputs, pipeline.md §9.2).
- Development: the Vite dev server proxies `/api` to the Rust server (`frontend/vite.config.ts`).

---

## 11. Execution model

```rust
// crates/wfd-server/src/state.rs
pub struct AppState {
    pub data: ArcSwap<GameData>,
    pub interactive: rayon::ThreadPool,        // `cores` threads (config interactive_threads): MC runs of /api/calc;
                                               // MC work split into ≤ 2 chunks while an optimizer job runs, `cores`
                                               // chunks otherwise (AD-15; results independent of the split)
    pub optimizer: rayon::ThreadPool,          // max(1, cores − 1) threads (config optimizer_threads)
    pub jobs: JobRegistry,                     // optimizer and data-refresh jobs: control, event ring buffer, result
    pub config: Config,
    pub http: ureq::Agent,                     // images and data refresh only
}
```

- `/api/calc`, `/api/timeline`, `/api/explain` run in `tokio::task::spawn_blocking`, installing the interactive pool
  for MC parallelism. A `CancelOnDrop` guard sets the engine's `CancelToken` when axum drops the handler future (the
  browser aborted a superseded request), so stale MC work stops between runs.
- Optimizer jobs run on a dedicated OS thread inside the optimizer pool; events go through a bounded channel into the
  job's ring buffer, and from there to SSE subscribers (`tokio::sync::broadcast` + ring buffer for replay).
- Data refresh jobs run on a blocking thread; they use only the HTTP agent and the file system.
- A panic in a handler is caught by `tower_http::catch_panic`, logged with an error id, and returned as 500
  `api.internal { error_id }`; a panic in a job marks the job `failed`. The release profile therefore keeps the
  default `panic = "unwind"` so the server survives a bug in one request.

---

## 12. Configuration, logging, security

### 12.1 Configuration

`<config_dir>/warframe-dps/config.toml` (`directories::ProjectDirs`); precedence CLI > environment (`WFD_*`) > file >
defaults.

| Key | Default | Meaning |
|---|---|---|
| `port` | 7878 | first port tried |
| `open_browser` | true | open the browser on start |
| `optimizer_threads` | cores − 1 | optimizer pool size |
| `interactive_threads` | cores | calc MC pool size; MC uses at most 2 of them while an optimizer job runs (AD-15) |
| `mc_runs_default` | 1000 | default `eval.mc_runs` in new scenarios |
| `data_dir` | platform data dir | bundles, images, logs, cache |
| `images.fetch` | true | proxy WFCD images on first use |
| `log.level` | `info` | `RUST_LOG` also honoured |
| `log.file` | true | daily rolling file in `<data_dir>/logs`, 7 kept |

The app makes no network request except image fetching (switchable off) and user-started data checks and refreshes.

### 12.2 Logging

`tracing` + `tracing-subscriber` (`EnvFilter`) to stderr in human format, plus `tracing-appender` daily file;
`tower_http::trace::TraceLayer` spans per request (method, path, status, latency); optimizer phases and data steps at
`info`; job ids in spans.

### 12.3 Security (localhost app)

- Bind loopback only by default.
- `Host` must be `127.0.0.1:<port>`, `localhost:<port>` or `[::1]:<port>` (defeats DNS rebinding); otherwise 421.
- Non-GET requests must carry no `Origin` or a same-origin `Origin`; otherwise 403. No CORS headers.
- Request body limit 1 MB (`tower_http::limit`); 413 above.
- Data refresh and images fetch only the allow-listed hosts of pipeline.md §3.1 (`wiki.warframe.com`,
  `content.warframe.com`, `raw.githubusercontent.com`, `registry.npmjs.org`).
- No authentication: anything that can reach the loopback port and passes the Host/Origin checks is the user.
