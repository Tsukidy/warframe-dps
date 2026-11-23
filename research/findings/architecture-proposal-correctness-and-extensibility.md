Status: draft (proposal)

# Architecture proposal: correctness and extensibility first

Author priority: **correctness and extensibility**. Game mechanics live in data and in a small set of typed rules;
code implements the spec's stage formulas once and never names an individual mod, weapon or enemy. Every number the
engine uses can be traced to a data row with a citation, and updating after a game patch is a data change in the
common case.

Inputs read for this proposal: `docs/mechanics/00-overview.md` (the spec; topic documents 01-09 did not exist yet
when this was written), `research/findings/inspiration-*.md`, `data-source-*.md`, `wiki-*.md`,
`oracle-feasibility.md`, the lua2json proof of concept, and real records from `research/external/cache/poc/out/`.
Citations use the spec's keys (`F:addmul`, `I:opt`, `DS:wiki`, ...). Crate and npm versions are the current ones on
crates.io / npm on 2026-11-23 (checked with `cargo search` / `npm view`).

Contents:

- [0. Summary of decisions](#0-summary-of-decisions)
- [1. Principles and invariants](#1-principles-and-invariants)
- [2. Cargo workspace and repository layout](#2-cargo-workspace-and-repository-layout)
- [3. Data pipeline](#3-data-pipeline)
- [4. Domain model and the effect system](#4-domain-model-and-the-effect-system)
- [5. Engine](#5-engine)
- [6. Optimizer](#6-optimizer)
- [7. Server](#7-server)
- [8. Frontend](#8-frontend)
- [9. Testing strategy](#9-testing-strategy)
- [10. Work breakdown](#10-work-breakdown)
- [11. Risks, costs and open questions](#11-risks-costs-and-open-questions)

---

## 0. Summary of decisions

| Area | Decision | Main reason |
|---|---|---|
| Workspace | 9 crates: `wf-model`, `wf-effects`, `wf-data`, `wf-engine`, `wf-optimizer`, `wf-buildcode`, `wf-datagen`, `wf-server` (binary `warframe-dps`), `xtask`; frontend in `frontend/` | engine and optimizer have no I/O; the effect language is its own crate with its own validator |
| Effect system | A **stat registry** (one declarative table in code that mirrors spec §2.4 and §3.1a) plus a **typed effect IR** (stat or mechanic, value, scope, condition, stacks, order). Curated items are written in TOML and lowered to the IR. Separate multipliers are **factor groups declared in data** (`G`, `G_F`, DoT-only, overguard-only) | a new Eclipse-like multiplier, a new buff or a re-valued mod is a data edit; the spec's buckets are one table, not scattered `if`s |
| Values | Every bucket total is an **affine form in scalars** (`c + Σ coef·scalar`: stacks, distinct statuses, Cold stacks, combo tier, indicators). EV evaluates scalars by expectation over their distributions, MC by their live value | both evaluators consume the *same* resolved stats, so they cannot drift apart; CD-13 ("never f(E[n])") is enforced structurally |
| Item-specific code | Forbidden in engine crates; a test fails if `/Lotus/` or an item name appears in `wf-engine` or `wf-optimizer` sources | special cases go to data (CO catalogue, overrides) or become a general mechanic kind |
| Data sources | wiki Lua modules (mlua, vendored Lua 5.1) for attacks, enemies, stances, kitgun/zaw parts, faction tables; DE Public Export for uniqueName, canonical top-level stats, drain/polarity, riven bases; WFCD for names, i18n, images; curated TOML for every effect | per `DS:wiki`, `DS:pe`, `DS:wfcd` conclusions; join on uniqueName only |
| Curated tables | One TOML file per item (`data/curated/items/<kind>/<family>/<slug>.toml`), rules in `data/curated/rules/*.toml`; every entry carries `status`, `sources`, and a fingerprint of the upstream text it was checked against; numbers are **bound** to the upstream text and cross-checked per rank | catches DE value changes automatically and lets the refresh auto-patch number-only drift |
| Bundle | Canonical pretty JSON committed under `data/generated/` (reviewable diffs per patch); `wf-data/build.rs` validates it and embeds a postcard+zstd bundle (~1.5-2.5 MB, estimate); user refreshes write a newer bundle to the app data dir | a patch is a reviewable diff; a broken data file fails the build, not the user |
| Data refresh | `warframe-dps data refresh` and a button in the About/Data page run the same pipeline library in-process (feature `refresh`, default on) and report diffs, stale curated entries and unmodelled new items | the user can update after a hotfix without a new release |
| Engine | Stage functions named after spec stages ST-1..ST-10, one module per topic document; EV and MC evaluators over one `Prepared` scenario; counter-based Philox4x32-10 RNG keyed per spec U-6; compatibility profile as a `CompatFlags` bitset named after register rows | traceability from code to spec, bit-reproducible MC |
| Spec test vectors | Topic documents carry ```` ```testvector ```` TOML blocks; `cargo xtask spec-vectors` extracts them to `crates/wf-engine/tests/spec_vectors/`; CI fails if stale | the spec and the tests cannot diverge silently |
| Optimizer | Compile the build space to per-item **lane vectors**; outer parallel enumeration of element classes x progenitor x arcane x exilus; inner **branch-and-bound** with monotone per-lane bounds (beam fallback for non-monotone items); exact local search; MC confirmation of finalists with common random numbers | exact where possible, deterministic, all cores, beats each of WFOpt's 13 weaknesses (`I:opt` §9) |
| Server | Axum 0.8: REST + SSE for optimizer and data-refresh progress; loopback only, Host/Origin checks; `rust-embed` frontend; opens the browser; TOML config; `tracing` logs | |
| Frontend | React 19 + TypeScript + Vite; **React Aria Components** (accessible combobox, grid list with keyboard drag and drop, dialogs); Zustand + Immer (undo/redo via patches); TanStack Query; react-intl (ICU); uPlot; Tailwind v4 with design tokens; TS types generated from Rust (`ts-rs`) | accessibility of drag-and-drop mod slots and comboboxes is the hard UI problem; React Aria solves it |
| UI extensibility | Options, buffs, custom stats and toggles are rendered from **schemas served by the server**; a new buff needs no frontend change | data-driven end to end |
| Explainability | Every resolved number carries an optional trace (source item, effect, bucket, spec equation); the UI has an "Explain" drawer for any number | correctness is visible to the user and to reviewers |
| Build sharing | `wfd1.` + base64url(deflate(canonical JSON of `ScenarioDoc` v1 with short ids)); Rust is the only codec; imports warframe-damage.com links (`?p=…&n=1` LZString and legacy base64) with an import report; exports inspiration links for the oracle harness | one codec implementation, versioned migrations |

---

## 1. Principles and invariants

The proposal is judged by whether these invariants hold; each has a mechanism and a test.

| ID | Invariant | Mechanism | Test |
|---|---|---|---|
| P-1 | Every game number used by the engine comes from the bundle, never from a Rust literal, except constants the spec defines as formula constants (1/32 grid, 4095/32, DR curve constants) | stat registry and `rules/*.toml`; constants live in `wf-engine/src/spec_constants.rs` with an `E-n` citation each | `xtask lint-literals`: float literals outside `spec_constants.rs` and tests fail CI (allow-list: 0.0, 1.0, 2.0 in arithmetic identities) |
| P-2 | No item-specific branches in engine or optimizer | items reach the engine only as compiled effects and attack data | test greps `crates/wf-engine/src` and `crates/wf-optimizer/src` for `/Lotus/` and for every item name in the fixture bundle |
| P-3 | Every spec bucket and mechanic is expressible in the IR | §4.6 coverage matrix; each row has a fixture item | `effects_coverage` test: one fixture per row, evaluated against a hand-computed value |
| P-4 | EV and MC share resolution | both consume `Prepared` (§5.2); bucket totals are `Affine` forms | `ev_mc_consistency` property test (spec §4.4 tolerance) |
| P-5 | Same build, any slot order (except element order), bit-identical result | contributions are summed in canonical order (stat, source id, effect index) | proptest: permute non-element slots, compare `f64::to_bits` |
| P-6 | MC results independent of thread count | counter-based RNG keyed by `(seed, run, attack, event, mechanic)`; runs merged in run order | test with rayon pools of 1 and 8 threads |
| P-7 | Every curated entry is checked against upstream text at every rank available | number binding + WFCD `levelStats` per-rank cross-check (§3.6) | pipeline `check` stage; CI runs it on the committed data |
| P-8 | Unknown is never silently zero | items without curated effects carry `status = "unmodelled"`; the UI badges them, the optimizer excludes them by default, the evaluate response lists them as warnings | e2e test on a fixture with an unmodelled mod |
| P-9 | Discrepancy decisions are switchable only where the register says so | `CompatFlags` has exactly one flag per `Compat` row; `Toggles` exactly one field per `T-nn` | `xtask spec-sync` compares flag and toggle names with the register and toggle tables in `00-overview.md` |
| P-10 | UI strings never hard-coded | ICU message catalogue; server messages are keys + arguments | ESLint `formatjs/no-literal-string-in-jsx`; Rust test that `Warning` and optimizer log types carry `MessageKey` |

---

## 2. Cargo workspace and repository layout

### 2.1 Repository tree

```text
warframe-dps/
  Cargo.toml                     # [workspace], shared [workspace.dependencies], lints
  rust-toolchain.toml            # channel = "stable"
  crates/
    wf-model/                    # domain types, ids, enums; serde; no logic beyond invariants
    wf-effects/                  # stat registry, effect IR, TOML authoring schema, lowering, validator, text renderer
    wf-data/                     # bundle format, loader, interning, indexes, search, legality (spec 09); build.rs embeds data
    wf-engine/                   # resolve (ST-1..ST-7), enemy side (ST-8/9), status (ST-10), schedules, EV, MC, outputs
    wf-optimizer/                # problem compilation, branch-and-bound, beam, local search, finalists, jobs
    wf-buildcode/                # build codes (wfd1), ScenarioDoc migrations, inspiration import/export
    wf-datagen/                  # data pipeline library + `wf-datagen` CLI (fetch, parse, join, curate, check, emit, diff)
    wf-server/                   # binary `warframe-dps`: Axum, SSE, static frontend, config, browser, data refresh
  xtask/                         # cargo xtask: gen-ts, spec-vectors, spec-sync, lint-literals, dist, oracle-record
  data/
    curated/                     # hand-authored, reviewed (TOML)
      items/{mods,arcanes,shards,auras,frame_mods,stances,sets}/...
      incarnon/<line>.toml       # Incarnon Genesis lines and native Incarnon weapons
      buffs/<id>.toml            # Roar, Eclipse, Xata's Whisper, ... (ability, focus, companion, squad)
      custom_stats.toml          # the 26 custom stats of spec 6.3
      overrides/{weapons,enemies,attacks}/*.toml
      rules/                     # status.toml, factions.toml, scaling.toml, factor_groups.toml, slot_layout.toml,
                                 # co_catalog.toml, riven_stats.toml, body_parts.toml, special_dr.toml, toggles.toml
      aliases/inspiration.toml   # inspiration name -> uniqueName, slot ids, perk indices
    sources.lock.toml            # pinned upstream revisions (WFCD tag, wiki revids, PE index hash)
    generated/                   # canonical pretty JSON emitted by wf-datagen (committed; reviewable diffs)
      manifest.json  weapons/{primary,secondary,melee,archgun,archmelee,companion,modular}.json
      enemies.json  items.json  stances.json  modular.json  rules.json  i18n/en.json  report.json
    fixtures/mini/               # tiny hand-written bundle in the same JSON format (tests and parallel work)
  frontend/                      # React + Vite app (section 8); `npm run build` -> frontend/dist
  tools/oracle/                  # Node + Playwright harness driving the inspiration offline (I:oracle)
  docs/ research/                # as today
```

### 2.2 Crates

| Crate | Responsibility | Depends on (workspace) | Key external crates |
|---|---|---|---|
| `wf-model` | `DamageType`, `StatusType`, `Layer`, ids (`ItemId`, `ShortId`, `AttackKey`, `FactionId`), `Weapon`, `Attack`, `Enemy`, `Item`, `Build`, `ScenarioDoc`, `Toggles` (T-01..T-29), `CompatFlags` | — | serde, bitflags, smallvec, thiserror, ts-rs (feature `ts`) |
| `wf-effects` | stat registry (`stats!` table), `Effect` IR, authoring schema (`EffectDoc`), lowering (`compile`), validation, canonical English renderer for cross-checks, condition-class derivation | wf-model | toml 1.2, serde |
| `wf-data` | `Bundle` header and body, `GameData` (interned, indexed), search index, legality (slot fit, incompatibility, capacity, polarity), loader for embedded and user bundles | wf-model, wf-effects | postcard 1.2, zstd 0.14, blake3 1.8 |
| `wf-engine` | `Engine::prepare/evaluate_ev/simulate/timeline/explain`; stage modules per spec document | wf-model, wf-effects, wf-data | rayon 1.12 (feature `parallel`, default) |
| `wf-optimizer` | `Optimizer::run(request, sink, cancel)`; lanes, bounds, search, finalists | wf-engine, wf-data | rayon |
| `wf-buildcode` | `encode/decode` of `ScenarioDoc`, version migrations, inspiration import/export | wf-model, wf-data | lz-str 0.2, miniz_oxide, base64 |
| `wf-datagen` | pipeline library and CLI | wf-model, wf-effects, wf-data | mlua 0.12 (`lua51`, `vendored`), ureq 3, lzma-rs 0.3, serde_json, toml, clap |
| `wf-server` | `warframe-dps` binary | all of the above (`wf-datagen` behind feature `refresh`, default on) | axum 0.8, tokio 1.54, tower-http 0.7, rust-embed 8, open 5, clap 4, tracing 0.1, directories |
| `xtask` | dev automation | wf-model (ts generation), wf-engine (vector extraction) | clap, toml |

Dependency graph (acyclic, I/O only at the edges):

```text
wf-model <- wf-effects <- wf-data <- wf-engine <- wf-optimizer <- wf-server
                              ^            ^                         ^
                              |            +---- wf-buildcode -------+
                              +-------------- wf-datagen ------------+ (feature "refresh")
```

`wf-engine` and `wf-optimizer` do no I/O, hold no global state and compile with `--no-default-features` without
rayon (a WebAssembly build stays possible later). `wf-server` is the only crate that touches the network (data refresh
through `wf-datagen`), the filesystem (config, logs, user bundles) and the browser.

Workspace lints (`[workspace.lints]`): `unsafe_code = "forbid"` everywhere, `clippy::float_cmp = "warn"` in engine
crates (comparisons go through `approx_eq` helpers in tests), `missing_docs = "warn"` on public engine APIs.

---

## 3. Data pipeline

### 3.1 Sources and what each provides

| Source | Fetch | Provides (authoritative fields) | Licence |
|---|---|---|---|
| wiki Lua modules | MediaWiki API, `prop=revisions` batched ≤ 50 titles, `maxlag=5`, ~60 modules, ~6 MB (`DS:wiki` §1) | weapon attacks (1984 records: per-attack damage map, crit, status, fire rate, multishot, falloff, forced procs, charge, burst, spool, ammo cost, Incarnon form attacks, `AttackParentIndex`), enemies (1019, body parts, `ProcResists`, Eximus fields), `DamageTypes/data` faction table, `Stances/data` combos, `Modular/data` kitgun and zaw parts, mod compatibility and `Incompatible` lists, riven disposition | CC BY-NC-SA 3.0 |
| DE Public Export | `index_en.txt.lzma` + 17 manifests (~15 MB, `DS:pe` §1.9) | canonical `uniqueName`, top-level weapon stats, trigger, MR, polarity, drain, max rank, mod sets (`ExportModSet`), arcane catalogue, riven `upgradeEntries` base values, `buildLabel` freshness | none stated (`DS:pe` open question 1) |
| WFCD warframe-items | pinned tag, raw GitHub files (`DS:wfcd` §8) | English and localized names/descriptions keyed by uniqueName, image names, per-rank `levelStats` text (used for **validation**, not as effects) | MIT |
| Curated tables | in-repo TOML | every effect, CO catalogue, overrides, rules (status values, scaling curves, factor groups, slot layouts, special DR) | project |

### 3.2 Pipeline stages

`wf-datagen` is a library with a thin CLI; the server's refresh calls the same library.

```text
fetch  -> data cache (content-addressed)          sources::{wiki, pe, wfcd}::fetch(&PoliteClient, &Lock)
parse  -> wire records per source                 sources::wiki::lua::eval_module (mlua sandbox) -> serde_json::Value -> typed wire structs
join   -> canonical entities keyed by uniqueName  join::{weapons, enemies, items, stances, modular}
curate -> apply rules, overrides, effects         curate::{load_curated, apply_overrides, compile_effects}
check  -> invariants + text cross-checks          check::{schema, refs, ranges, text, coverage}
emit   -> data/generated/*.json + bundle + report emit::{canonical_json, bundle, report}
```

CLI:

```text
wf-datagen fetch   [--offline] [--update-lock]        # refresh the cache; --update-lock bumps data/sources.lock.toml
wf-datagen build   [--out data/generated]             # parse+join+curate+check+emit (offline, from cache)
wf-datagen check                                       # curate+check only; CI runs this
wf-datagen diff <old-dir|bundle> <new-dir|bundle>      # field-level diff: "Furis/Normal Attack crit_chance 0.05 -> 0.07"
wf-datagen curate todo [--kind mods] [--limit 50]      # items without verified effects, sorted by kind and slot
wf-datagen curate new <name|uniqueName>                # writes a draft TOML prefilled from upstream text
wf-datagen curate rebind <file>                        # recomputes number bindings and the upstream fingerprint
wf-datagen report                                      # coverage, conflicts, stale entries (markdown + json)
```

**Fetching.** A `PoliteClient` modelled on warframe-connector (`DS:pe` Part 2): one request per second per host,
retries only on 429/503/509 and connection errors (≤ 3, honouring `Retry-After` up to 60 s), 404s cached, a generic
User-Agent `warframe-dps-datagen/<version> (local tool)` with no personal data. Conditional requests: PE index by
ETag and per-line hash; wiki modules by `revid` (one `prop=revisions&rvprop=ids|timestamp` query tells which modules
changed); WFCD by pinned tag. Cache layout: `cache/<source>/<sha256>.<ext>` plus `cache/index.json`
(url → sha256, etag, revid, fetched_at). `data/sources.lock.toml` pins the exact inputs of the committed bundle, so
`wf-datagen build --offline` is reproducible.

**Lua evaluation.** Port of the PoC (`DS:wiki` §9): mlua 0.12 with `lua51` + `vendored`, only `math`, `string`,
`table` loaded, stubs for `mw.loadData`, `mw.site` and `require`. Each module is evaluated with an instruction
limit (`set_hook` every 10^6 instructions, abort after 10^9) and a memory limit (`Lua::set_memory_limit(256 MB)`),
because modules are untrusted code. Conversion rules exactly as the PoC (array detection, `math.huge` →
`"Infinity"`, empty table ambiguity resolved per field by the typed wire struct). Wire structs use
`#[serde(default)]`, normalize known typos (`Hitscan`, `Ammocost`, `ForcedProc`, `EximusSHield`, `IsSlilent`) through
`#[serde(alias)]`, and skip `_IgnoreEntry`.

**Join.** On `uniqueName` only (`DS:pe` §1.8). Field precedence is a table in code (`join::PRECEDENCE`), printed into
the report:

| Field | First source | Fallback | Conflict handling |
|---|---|---|---|
| identity, slot, trigger, MR, polarities, drain, max rank, mod sets | PE | wiki | report only |
| top-level weapon crit/status/fire rate/magazine/reload | PE | wiki | if PE and the wiki's first attack differ by > 1e-4 relative, report a conflict; per-attack values come from the wiki |
| attacks, forced procs, falloff, charge, burst, spool, Incarnon form attacks | wiki | — | — |
| names, descriptions, images, locales | WFCD | wiki `Name` | — |
| enemies (all fields) | wiki | — | WFCD enemies never used (`DS:wfcd` §6) |
| faction damage-type table | wiki `DamageTypes/data` | `rules/factions.toml` override per faction | override must cite a source |
| mod compatibility / incompatible lists | wiki | PE `compatName` | union of both directions (D-093) |
| effects | curated only | — | — |

**Attack roles.** The join assigns every attack a stable `AttackKey { index, slug }` and a set of `AttackRole`
flags derived from wiki fields (`AttackName`, `ShotType`, `AttackParentIndex`, `IncarnonCharges`, charge/heavy/slam
markers) through a small rule table in `rules/attack_roles.toml`; anomalies (`DS:wiki` §11: inconsistent
`AttackParentIndex`) are fixed in `overrides/attacks/*.toml`. Effects scope on roles, never on attack names.

**Kitguns and zaws** are assembled from parts at runtime, not from the 1200 pre-expanded kitgun records (`DS:wiki`
§3: the combos predate the last parts edit). The bundle stores `ModularParts` (chambers, grips, loaders, strikes,
links) and `wf-data::modular::assemble(parts, picks) -> Weapon` implements spec 06/07's assembly rule. A test
asserts `assemble` equals every pre-expanded wiki combo on damage and fire rate.

### 3.3 Curated tables: layout and format

Format: **TOML**, one file per item, path `data/curated/items/<kind>/<family>/<slug>.toml` (for example
`items/mods/rifle/serration.toml`, `items/arcanes/primary/primary_deadhead.toml`). One file per item keeps reviews
small, avoids merge conflicts when several curators work in parallel, and gives each item its own history. Rules are
one file per topic in `data/curated/rules/`. TOML was chosen over RON (unfamiliar to curators), YAML (implicit
typing pitfalls) and JSON (no comments). The authoring schema is versioned (`schema = 1`) and published as JSON Schema
(`schemars`) for editor completion.

Common header of an item file:

```toml
schema = 1
item = "/Lotus/Upgrades/Mods/Rifle/WeaponDamageAmountMod"   # uniqueName: the join key
label = "Serration"                                          # must equal the bundle's English name (checked)
status = "verified"            # draft | verified | stale | unmodelled | no_damage_effect
sources = ["W:Serration", "F:core §2"]
notes = ""                     # free text; UNVERIFIED claims must say UNVERIFIED here and in the effect comment
upstream = { field = "wiki.Description", sha256 = "9b1c…", text = "+165% Damage" }   # written by `curate rebind`

[[effect]]
stat = "base_damage"
value = { max = 1.65, curve = "linear", bind = "n1%" }       # n1% = first number of upstream text, as a percent
```

`status` semantics: `draft` (tool-generated, not reviewed; allowed in the bundle but flagged in the UI), `verified`
(reviewed, sources cited), `stale` (set automatically when the upstream text fingerprint changes; numbers may have been
auto-patched, §3.6), `unmodelled` (known to affect damage but not encoded yet; P-8), `no_damage_effect` (recoil, zoom
without stat changes, ...; the optimizer drops it from pools).

### 3.4 How a curator adds a mod's effects

1. `wf-datagen curate todo --kind mods` lists items without a `verified` file, with their upstream text.
2. `wf-datagen curate new "Hunter Munitions"` writes `items/mods/sets/hunter_munitions.toml` with the header, the
   upstream text of every rank as comments, and **draft effects** for every line that matches one of the ~40 simple
   line patterns (`DS:wfcd` §3.4: `+N% Status Chance`, `xN Damage to Grineer`, `+N% <DT_FIRE_COLOR>Heat`, ...).
   Non-simple lines (`On Kill:`, `per Status Type`, `stacks with Combo Multiplier`, `(x2 for Heavy Attacks)`) are
   emitted as `# TODO` comments with the matching IR construct suggested.
3. The curator writes or corrects the effects, cites sources (`W:` page or `F:` finding), sets `status = "verified"`.
4. `wf-datagen check` validates (§3.6). `cargo test -p wf-engine --test effect_smoke` evaluates the item on its
   reference weapon (§9.2) to prove that the effect reaches the stat it claims to change.
5. The PR diff shows the TOML and the changed lines of `data/generated/items.json`.

### 3.5 Rules files (typed, data-driven mechanics parameters)

| File | Content (owner spec document) | Example row |
|---|---|---|
| `rules/status.toml` | per status: base duration, cap, tick count rule, first-tick delay, DoT coefficient, debuff function parameters (Viral `2 + 0.25(n−1)`, Corrosive `min(1, 0.20 + 0.06 n)`, Cold, Puncture, Heat strip ramp) (`03`) | `[COR] duration = 8.0; cap = 10; strip = { base = 0.20, per_stack = 0.06, max = 1.0 }` |
| `rules/factions.toml` | faction ids, display keys, damage-type modifier tables, sub-faction relations for banes (`Kuva Grineer` takes Grineer banes), scaling curve family (`05`, D-046, D-008) | `[faction.kuva_grineer] modifiers = { IMP = 1.5, COR = 1.5, HEA = 0.5 }; banes = ["grineer"]` |
| `rules/scaling.toml` | level-scaling curves per faction family and layer, smoothstep range, Steel Path factors, Eximus formulas (`05`, NP-2) | `[grineer.health] f1 = { c = 0.015, e = 2.12 }; f2 = { c = 10.7332, e = 0.72 }` |
| `rules/factor_groups.toml` | every separate multiplier group: id, stage (`final_omitted` = G_F, `other` = G, `dot_only`, `overguard_only`, `part_spot`), additive-inside flag, citation (`04`) | `[[group]] id = "eclipse"; stage = "final_omitted"; sources = ["F:addmul §1.6"]` |
| `rules/co_catalog.toml` | per weapon attack: CO behaviour (`adding`, `multiplying`, `none`), ratio `B_co/B_ref`, stance input ratios (`04`, D-057) | `[[attack]] weapon = "<Paris uniqueName>"; role = "charged"; behavior = "adding"; ratio = 0.5` |
| `rules/slot_layout.toml` | slot ids per equipment class, accepted item kinds, element-order class per slot, exilus position (`09`, OQ-024) | `[primary] main = 8; exilus = { order = "after_main" }; arcane = 1` |
| `rules/riven_stats.toml` | riven stat id → PE tag → effect template → UI unit (`04`, `09`) | §4.7 example 5 |
| `rules/body_parts.toml` | default part multipliers, head-crit capability rules, 1x-headshot weapon list (`02`, `05`) | |
| `rules/special_dr.toml` | attenuation and flat DR per enemy family (`05`, D-047) | |
| `rules/toggles.toml` | UI descriptors of T-01..T-29 (group, i18n key, range, default, visibility predicate) | `[T-13] group = "buffs"; kind = "percent"; default = 0.30` |

Rules are part of the bundle (`rules.json`) and are read by the engine through typed structs. Changing Viral's
multiplier after a patch is a one-line edit with a citation.

### 3.6 Validation, including against source text

`wf-datagen check` runs these checks; any `error` fails CI and the build; `warning`s go into `report.json` and the UI's
data page.

| Check | Rule | Severity |
|---|---|---|
| Schema | TOML deserializes into `EffectDoc` (unknown keys rejected); stat names exist in the registry; scalar and condition names exist | error |
| Referential | every `item` uniqueName exists in the joined catalogue; every curated item has a catalogue entry; every catalogue mod/arcane/shard has a curated file or is reported as `missing` (treated as `unmodelled`) | error / warning |
| Applicability sanity | a stat that only exists for one weapon class (combo, attack speed, multishot on melee) used on an item that cannot be equipped by that class; scope roles that no compatible weapon has | error |
| Number binding | every `bind = "nK…"` refers to an existing number token of the upstream text; the bound value equals the token after unit conversion (`%` → /100, `x1.55` faction → 0.55, seconds) | error |
| Unbound numbers | every number token of the upstream max-rank text is bound by some effect, or listed in `ignore_numbers = [...]` with a reason | error |
| Per-rank cross-check | for every rank 0..max present in WFCD `levelStats`, the effect value at that rank, rendered by `wf-effects::render`, matches the WFCD line within the rounding of the displayed text (WFCD rounds to 1 decimal, `DS:wfcd` §3.4); this detects non-linear rank curves | error |
| Fingerprint | the upstream text sha256 equals the one in the file; otherwise the entry becomes `stale` in the generated data and the report lists the old and new text | warning |
| Rule invariants | status caps > 0, chances are fractions (`0 ≤ cc ≤ 20`), every attack has ≥ 1 damage type and a fire model, every weapon has ≥ 1 attack, every Incarnon line has one perk list per evolution tier | error |
| Coverage | count of verified/draft/unmodelled per kind; regressions versus the committed `report.json` | warning |

**Auto-patching number drift.** When a fingerprint changes and the new text has the same token structure as the old
(only numbers differ), the refresh rewrites bound values in the *generated* data (never in the curated source),
marks the entry `stale (auto-patched)` and lists it in the refresh report. Example: DE changes Galvanized Aptitude from
"+40% Direct Damage per Status Type" to "+45% …": the `co` effect bound to `n2%` becomes 0.45 at once, the UI shows
"auto-updated from game text, not yet reviewed", and a curator later reviews and commits.

### 3.7 Bundle format, size, embedding and refresh

**Canonical JSON (`data/generated/`).** Pretty, keys sorted, floats printed with the shortest round-trip
representation, one file per entity kind (weapons split by slot). This is what reviewers diff after a game patch.
Estimated size 8-12 MB (wiki module JSON alone is 6.4 MB for weapons, enemies, mods, arcanes, stances and modular
parts; measured with `du` on `research/external/cache/poc/out/`).

**Binary bundle.** `wf-data/build.rs` reads `data/generated/`, deserializes into `wf-model` types (schema check),
runs `wf-effects::validate` on all compiled effects, and writes `OUT_DIR/bundle.bin`:

```text
magic "WFDPSB01" | u32 schema_version | u32 header_len | header (postcard) | body (zstd level 19 over postcard)
header = BundleHeader { schema_version, data_version: DataVersion, content_hash: [u8;32] (blake3 of body),
                        created_at, sources: Vec<SourceRev>, counts: Counts, licences: Vec<LicenceNotice> }
DataVersion { game_build_label: "2026.10.28.09.54", wiki_max_revid, wfcd_tag: "v1.1276.51", pe_index_hash, curated_rev }
```

Estimated embedded size 1.5-2.5 MB (postcard is roughly 3-4x smaller than JSON for numeric records, zstd another
3-5x); startup decode under 50 ms on one core (estimate; `wf-data` benchmark `bench_load` guards it). Images are not
bundled: the server proxies WFCD/CDN images on first use into a disk cache (`/api/img/{name}`) and the UI shows a
neutral placeholder offline.

**User refresh after a game update.** The About → Data page shows the active `DataVersion` and a "Check for
updates" button. `POST /api/data/check` asks the PE index (ETag) and the wiki revids whether anything changed (2-3
requests). `POST /api/data/refresh` runs fetch + build in a background task with SSE progress (typical: ~60 wiki
modules in 2 batched calls, ≤ 17 PE manifests, WFCD files for the pinned or newest tag; ~25 MB; about 1-2 minutes at
the polite rate), then shows the refresh report: field diffs (`wf-datagen diff` against the active bundle), new
items (unmodelled until curated), stale and auto-patched curated entries, conflicts. "Activate" writes
`<data_dir>/bundles/wfdps-<build_label>-<hash8>.bundle` and switches the server's `ArcSwap<GameData>`; the previous
bundle stays for rollback. The curated tables used by a refresh are the ones embedded in the binary
(`include_dir!` of `data/curated/` at compile time); `--curated <dir>` lets a user point at a newer checkout. Builds
referring to removed items decode with a warning per missing item.

**Licence attribution.** The bundle header carries `LicenceNotice` records (wiki CC BY-NC-SA 3.0 with the module
titles and revids used, WFCD MIT copyright line, "Warframe content and data © Digital Extremes" for PE-derived
fields, our curated tables). The About page renders them; `ATTRIBUTION.md` at the repo root is generated from the same
records. Because CC BY-NC-SA is share-alike and non-commercial, the derived bundle is distributed under CC BY-NC-SA
3.0 and the app's About page says so. Committing `data/generated/` to the repository is a redistribution; that needs
the user's confirmation (§11 open question Q1). The alternative, if the user declines, is to keep `data/generated/`
out of git and require `wf-datagen build` before `cargo build` (build.rs then fails with a clear message).

---
## 4. Domain model and the effect system

### 4.1 What is code and what is data

| Lives in code (changes need a spec change) | Lives in data (changes are curation) |
|---|---|
| `DamageType` (18) and `StatusType` (18) enums, layers, crit tiers (spec §2.2) | factions and their type modifiers, scaling curves, sub-faction bane relations |
| the stat registry: which buckets exist and where each enters the formula (spec §2.4, §3.1a) | which factor groups exist and their stage (`G`, `G_F`, DoT-only, OG-only, spot) |
| stage formulas E-1..E-15, CD-01..CD-13, T-02 class semantics | every item's effects, rank curves, stacks, durations, conditions, scopes |
| mechanic kinds (extra hit, proc on event, repeat hit, ...) and their event semantics | which items use which mechanic kind with which parameters |
| scalar kinds (stacks, distinct statuses, combo tier, ...) and their EV expectation rules | status durations, caps, coefficients, debuff functions' parameters |
| schedule generators (spec 06, 07) | per-attack CO behaviour and ratio, beam ramp starts, semi-auto caps, stance hit tables |
| `Toggles` fields T-01..T-29 and `CompatFlags` (register rows) | UI descriptors of toggles, buffs, custom stats; slot layouts; riven stat table |

The rule for a new game mechanic: if it fits an existing stat, scalar, condition or mechanic kind, it is data. If not,
it needs a new registry entry or mechanic kind (code, reviewed against the spec) and from then on it is data.

### 4.2 Core domain types (`wf-model`)

```rust
// ids.rs
pub struct ItemId(pub Arc<str>);          // DE uniqueName, the only join key and the only persistent id
pub struct ShortId(pub [u8; 6]);          // base32 of blake3(uniqueName) truncated; used in build codes; collisions checked by the pipeline
pub struct Sym(pub u32);                  // interned index, valid for one loaded bundle; never serialized
pub struct AttackKey { pub index: u8, pub slug: SmolStr }   // slug from the wiki AttackName ("incarnon_form")
pub struct FactionId(pub SmolStr);        // data-driven (Techrot and Scaldra arrived recently; no enum)

// weapon.rs
pub struct Weapon {
    pub id: ItemId, pub name: I18nKey,
    pub class: WeaponClass,               // Primary | Secondary | Melee | ArchGun | ArchMelee | Companion | Exalted | Amp
    pub categories: CategorySet,          // bitset: rifle, shotgun, bow, sniper, pistol, dual_pistols, glaive, whip, ...
    pub compat_tags: Vec<Tag>,            // PROJECTILE, AOE, BEAM, BATTERY, augment and stance-class tags
    pub trigger: Trigger,
    pub magazine: Option<MagazineSpec>,   // size, ammo max, infinite, battery {delay, delay_empty, rate}
    pub reload: ReloadSpec,               // Standard{time} | PerRound{start_end, per_round} | Regenerate{..}
    pub attacks: Vec<Attack>,
    pub attack_groups: Vec<AttackGroup>,  // parent+child summaries and "combine attack effects" (T-22)
    pub zoom_levels: Vec<ZoomLevel>,      // each zoom level is an effect list gated by `aiming`
    pub passives: Vec<Effect>,            // innate effects in the same IR (Cedo CO, IPS amplifiers, Sagek, ...)
    pub incarnon: Option<IncarnonRef>,    // line id + variant key (perk values differ per variant)
    pub progenitor: Option<ProgenitorSpec>, // allowed elements, value range [0.25, 0.60]
    pub strength_scaling: StrengthScaling,  // None | BaseDamage | CytePartial(f64) ... (exalted, spec 01)
    pub modular: Option<ModularRef>,      // kitgun chamber / zaw strike: needs parts in the build
    pub polarities: Vec<Polarity>, pub exilus_polarity: Option<Polarity>, pub max_rank: u8,
    pub provenance: Provenance,           // source + revision per entity, overridden fields listed
}

pub struct Attack {
    pub key: AttackKey, pub name: I18nKey,
    pub roles: AttackRoles,               // bitflags NORMAL ALT CHARGED UNCHARGED INCARNON_FORM RADIAL CHILD BEAM
                                          //          HEAVY SLAM HEAVY_SLAM SLIDE AERIAL THROWN FINISHER
    pub parent: Option<u8>,
    pub damage: DamageVec,                // [f64; 18] per pellet in HP, spec type order
    pub crit_chance: f64, pub crit_mult: f64, pub status_chance: f64, pub multishot: f64,   // fractions (U-1)
    pub fire: FireModel,                  // Auto{rate} | Semi{rate} | Burst{count, delay, rate} | Charge{time, partial}
                                          // | Held{tick_rate} | Spool{start_frac, shots} | Melee{..}
    pub shot_type: ShotType,              // Hitscan | Projectile | Radial | Discharge | Melee
    pub ammo_cost: f64, pub magazine_override: Option<MagazineSpec>,   // Incarnon charges live here
    pub falloff: Option<Falloff>, pub radius: Option<f64>,
    pub forced_procs: Vec<StatusType>,
    pub headshot: HeadshotRule,           // Normal | OneX | NoBodyPart (radial)
    pub co: CoSpec,                       // { behavior: Adding | Multiplying | None, ratio: f64 } from rules/co_catalog.toml
    pub sniper_combo: Option<SniperComboSpec>, pub beam_ramp: Option<RampSpec>, pub semi_cap: Option<f64>,
    pub passives: Vec<Effect>,            // attack-level innate effects (Rubico zoom crit, ...)
}

// enemy.rs
pub struct Enemy {
    pub id: ItemId, pub name: I18nKey, pub faction: FactionId, pub base_level: u16,
    pub health: f64, pub shield: f64, pub armor: f64, pub overguard: f64,
    pub eximus: EximusSpec, pub steel_path_health_bonus: f64,           // D-055
    pub body_parts: Vec<BodyPart>,        // { name, mult, weak_point, head, headcrit } parsed from `Multis` (+ overrides)
    pub status_immunities: StatusSet, pub status_cap_override: Option<u8>,
    pub innate_dr: LayerArray<f64>, pub status_dr: f64, pub type_mod_override: Option<DamageVec>,
    pub special_dr: Option<SpecialDrId>, pub toxin_blocked: bool, pub party_scaled: bool,
}

// item.rs: mods, arcanes, shards, auras, frame mods, stances, perks, buffs, custom stats share one shape
pub struct Item {
    pub id: ItemId, pub short: ShortId, pub name: I18nKey, pub kind: ItemKind,
    pub params: Vec<ParamSpec>,           // user-chosen parameters (Chromatic Blade element, buff value)
    pub effects: Vec<Effect>,             // compiled IR (4.4)
    pub status: CurationStatus, pub sources: Vec<Citation>,
}
pub enum ItemKind {
    Mod { family: ModFamily, polarity: Polarity, base_drain: i8, max_rank: u8, compat: CompatRule,
          incompatible: Vec<ItemId>, set: Option<SetId>, trigger_req: Option<Trigger> },
    Arcane { class: ArcaneClass, max_rank: u8 },  // weapon (primary/secondary/melee/kitgun/zaw/bow/...) or warframe
    Shard { color: ShardColor, tauforged: bool },
    Aura { polarity: Polarity, capacity: i8, max_rank: u8 },
    Stance { class_tag: Tag, combos: Vec<StanceCombo>, polarity: Polarity, capacity: i8 },
    Perk { line: IncarnonLineId, evo: u8 },
    Buff { group: BuffGroup }, CustomStat,
}
pub struct SetSpec { pub id: SetId, pub members: Vec<ItemId>,
                     pub bonus: Vec<Effect>,               // may use scalar `set_pieces`
                     pub member_scale: Option<Vec<f64>> }  // Sacrificial: member values x1.25 with 2 pieces
pub struct IncarnonLine { pub id: IncarnonLineId, pub variants: Vec<(VariantKey, ItemId)>,
                          pub tiers: Vec<IncarnonTier> }   // tier { evo, perks: Vec<Item(kind = Perk)> }
pub struct Faction {                      // from rules/factions.toml (+ wiki DamageTypes/data); data-driven ids
    pub id: FactionId, pub name: I18nKey,
    pub type_mods: DamageVec,             // 1.5 / 1.0 / 0.5 per type (D-046); overguard uses its own rule (05)
    pub bane_of: Vec<FactionId>,          // which faction bonuses apply: Kuva Grineer takes Grineer banes (D-008)
    pub weakness_element: Option<DamageType>,   // for `target_weakness` elements (Sentient Incision)
    pub scaling: ScalingFamilyId,         // curves in rules/scaling.toml
}
pub struct Riven { pub slot: SlotId, pub class: RivenClass, pub lines: Vec<RivenLine> }  // lines in listed order
pub struct RivenLine { pub stat: RivenStatId, pub value: f64 }    // fraction (faction lines: multiplier − 1)
pub struct Buff { pub id: BuffId, pub group: BuffGroup, pub params: Vec<ParamSpec>, pub effects: Vec<Effect>,
                  pub inspiration_key: Option<SmolStr> }          // Roar, Eclipse, Xata's Whisper, Vex Armor, ...

// build.rs
pub struct Build {
    pub weapon: WeaponPick,               // { id, rank, modular: Option<ModularPicks>, progenitor: Option<(DamageType, f64)> }
    pub slots: Vec<(SlotId, ItemPick)>,   // SlotId from rules/slot_layout.toml: main0..main7, exilus, stance, arcane0/1,
                                          // frame0..frame3, aura, frame_arcane0/1, shard0..shard4
    pub riven: Option<Riven>,             // compiled through rules/riven_stats.toml into ordinary effects
    pub incarnon: [Option<PerkId>; 5],    // one perk per evolution tier (spec 09)
    pub stance_combo: Option<ComboKey>,
}
pub struct ItemPick { pub item: ItemId, pub rank: Option<u8>, pub params: BTreeMap<ParamId, ParamValue> }
pub struct Scenario {
    pub build: Build, pub enemy: EnemySetup,       // { id, level, steel_path, eximus, party }
    pub toggles: Toggles,                          // one typed field per T-01..T-29, named t01_window ... t29_mc
    pub buffs: Vec<BuffSetting>,                   // { buff, params } (Roar value, Xata value, ...)
    pub custom: Vec<(CustomStatId, f64)>,          // T-26
    pub conditions: ConditionPolicy,               // T-02 mode + per-effect overrides (4.4.4)
    pub attacks: AttackSelection,                  // which attacks, groups, stance combo, heavy mode
    pub hit_mix: HitMix,                           // body / head / weak point fractions; T-03 sets one of them to 1
    pub eval: EvalSettings,                        // window, horizon, n_runs, seed, compat profile
}
```

`ScenarioDoc` (in `wf-model::doc`) is the serialized, versioned form used by the API and build codes; `Scenario` is
its validated, interned form (`ScenarioDoc::resolve(&GameData) -> Result<Scenario, Vec<DocError>>`).

### 4.3 The stat registry

The registry is one declarative macro table in `wf-effects/src/registry.rs`. Each row fixes the TOML name, the spec
symbol, how entries combine, where the bucket enters the formula, the unit and whether the stat can change damage
(optimizer relevance). From the table the crate generates the `Stat` enum, lane indices, the TOML parser, the canonical
English renderer (used by text cross-checks), the TypeScript enum and a Markdown table for `docs/architecture/`.
Curators cannot pick an operation: the stat decides it. A separate multiplier is never written as "multiply"; it is a
`factor.<group>` whose group is declared in `rules/factor_groups.toml`.

| TOML key | Spec symbol | Combine | Enters | Notes |
|---|---|---|---|---|
| `base_damage` | `sum_base_dmg` (A) | sum | ST-4 | heavy-only, slide-only damage are this stat with an attack-role scope |
| `base_damage.universal` | melee-projectile second base bucket | sum | ST-4 | `F:addmul` §12; enters only on attacks with role THROWN or melee projectile |
| `physical.{IMP,PUN,SLA}` | `sum_phys[t]` | sum | ST-2 | stance physical comes from stance data, not effects |
| `element.{HEA,COL,ELE,TOX}` | `sum_elem[t]` | ordered sum | ST-2 | carries an `OrderKey` (4.4.5) |
| `element.{BLA,RAD,GAS,MAG,VIR,COR}` | `sum_elem[t]` | sum | ST-2 | already-combined additions (riven Viral, Biotic Rounds, Melee Exposure) |
| `element_uncombined.{t}` | `sum_elem[t]`, non-combining | sum | ST-2 | Nourish, Ember-augment `heatAdd`; element may be `target_weakness` |
| `incarnon_flat` | `incarnon_flat` X | sum (HP) | ST-1 | excluded from `B_ref` (E-1, E-11) |
| `faction` (+ `faction = id \| "any"`) | `sum_faction` F | sum over entries matching the target's faction relations | ST-5, DoT, extra hits | Roar uses `"any"` |
| `co` | `sum_co` C | sum, affine in scalars | ST-4 | behaviour comes from the attack (`Attack.co`) |
| `factor.<group>` | one factor of `G_F`, `G`, `M_dot`, `M_OG` or `M_son` | sum inside the group, product across groups | by group stage | Eclipse, chambers, Mecha, Fortifier, Sonar, finisher, slam |
| `strength` | `strength` | sum | scenario | Crimson shard |
| `crit_chance.base_add` / `.rel` / `.abs` / `.floor` / `.final_mult` | `cc_base_add`, `sum_cc_rel`, `sum_cc_abs`, `cc_floor`, custom `p.d` | sum / sum / sum / max / sum | E-5 | |
| `crit_damage.base_add` / `.rel` / `.abs` / `.field_mult` | `cd_base_add`, `sum_cd_rel`, `sum_cd_abs`, `cd_field_mult` | sum / sum / sum / product | E-4 | |
| `crit_tier_raise` | `tier_raise_chance` | sum | E-6 | Vigilante |
| `weak_point_damage` / `headshot_mult` | `sum_wp_dmg`, `sum_hs_mult` | sum | E-7 | |
| `status_chance.base_add` / `.rel` / `.abs` / `.mult` | E-8 terms | sum / sum / sum / product of (1+x) | E-8 | Tau multiplier is a rule, not an effect |
| `status_damage`, `status_damage.{s}` | `sum_status_dmg`, `status_dmg_type[s]` | sum | E-13 | |
| `status_duration` | `sum_status_dur` | sum | CD-06 | |
| `status_cap.{s}` | `stack_cap[s]` | sum (integer) | CD-07 | Emerald shard |
| `multishot` / `multishot.base_add` / `multishot.lock` | E-10 | sum / sum / any | E-10 | lock = Acuity |
| `fire_rate` / `fire_rate.lock` / `charge_rate` | spec 06 | sum / any / sum | schedule | lock = Cannonade |
| `magazine` / `magazine.flat` / `reload_speed` / `ammo_efficiency` | spec 06 | sum / sum / sum / sum (clamped 0.75) | schedule | |
| `attack_speed` / `initial_combo` / `combo_duration` / `heavy_efficiency` / `wind_up_speed` | spec 07 | sum (efficiency clamped 0.9) | schedule | |
| `target.vulnerability` (+ `group`) | `vuln` V | product across groups, sum inside the additive subgroup | ST-8 | Molecular Prime, custom `p.x` |
| `target.armor_strip` / `target.armor_strip.aura` | E-15 ability strips, CP | sum | E-15 | Corrosive Projection uses `.aura` |
| `target.status.{s}` | external stacks (T-06) | sum (stacks) | target state | |
| `display.{range,punch_through,projectile_speed,recoil,zoom,...}` | none | sum | weapon panel only | `damage_relevant = false`; optimizer drops them |

### 4.4 The effect IR (`wf-effects`)

#### 4.4.1 Types

```rust
pub struct Effect {
    pub target: Target,                 // what it changes
    pub value: ValueExpr,               // how much (before rank/variant/param resolution)
    pub scope: Scope,                   // to which weapons, attacks and hits it applies (static filter)
    pub gate: Option<Condition>,        // when it is active (dynamic, T-02 class derived from the kind)
    pub stacks: Option<StackSpec>,      // stacking state (own counter), if any
    pub order: Option<OrderClass>,      // only for combining elements; the build assigns the slot part
    pub note: Option<UnverifiedNote>,   // carries UNVERIFIED markers through to the UI and the trace
}

pub enum Target {
    Stat(Stat),                         // registry entry, e.g. Stat::CoBonus, Stat::Element(DamageType::HEA)
    Factor(FactorGroupId),              // `factor.<group>`
    Faction(FactionSel),                // FactionSel::Any | FactionSel::Id(FactionId)
    Mechanic(Mechanic),                 // 4.4.6
}

pub enum ValueExpr {
    Const(f64),
    Ranked { max: f64, curve: RankCurve },          // Linear: max*(r+1)/(R+1) | Table(Vec<f64>) | Fixed
    Variant(BTreeMap<VariantKey, f64>),             // Incarnon per-variant values (Furis 28, MK1-Furis 34)
    Param(ParamId),                                 // user parameter of the item or buff
    Linear { base: Box<ValueExpr>, terms: Vec<(Box<ValueExpr>, ScalarProduct)> },   // base + Σ coef·Π scalars
    TableBy { scalar: Scalar, values: Vec<f64> },  // value indexed by a static scalar (Vigilante by set pieces)
}

pub enum Scalar {                       // every scalar has a fixed T-02 class and a fixed EV rule (5.4)
    Stacks(CondId),                     // the effect's own stack counter            class from the gain trigger
    DistinctStatuses,                   // n (T-05)                                  (a)
    StatusStacks(StatusType),           // n_s                                       (a)
    StatusPresent(StatusType),          // 1[n_s >= 1]                               (a)
    TargetHealthBelow(f64),             // 1[hp/max < x], layered pass only          (a)
    ComboTier, ComboTierAbove1,         // M(c), M(c)-1                              (b)
    SniperCombo,                        //                                           (b)
    SetPieces(SetId),                   // static
    ShardCount(ShardColorSet),          // static (Violet shard)
    Strength,                           // static (scenario Str + strength effects)
    Resolved(ResolvedStat, Step),       // static after stage ordering (Melee Doughty: per 0.10 Puncture SC)
    MagazineSqrt,                       // static after schedule stats (Primary Plated Round)
}
pub type ScalarProduct = SmallVec<[Scalar; 2]>;     // e.g. [Stacks(galv), DistinctStatuses]

pub struct Scope {
    pub weapon: WeaponFilter,           // classes, categories (bow, sniper, ...), compat tags, exalted yes/no
    pub attack: RoleFilter,             // any_of / none_of AttackRoles
    pub hit: HitFilter,                 // Direct | Radial | DotTick | ExtraHit; BodyPart::{Body, Head, WeakPoint}
}

pub enum Condition {                    // class(a) target, (b) own events, (c) external; derived, never authored
    TargetHasStatus(StatusType),        TargetStatusCountAtLeast(u8),  TargetFrozen,  TargetMarked,
    TargetHealthBelow(f64),                                                                                   // (a)
    OnHit(HitFilter), OnCrit, OnProc(StatusType), Aiming, OnReload,                                          // (b)
    FirstShotOfMagazine, LastShotOfMagazine { min_magazine: u16 },                                           // (b), schedule-driven
    External { trigger: ExternalTrigger, chance: Option<ValueExpr>, duration: Option<f64> },                  // (c)
    StatBelow { stat: ResolvedStat, threshold: f64 },  StatAtLeast { stat: ResolvedStat, threshold: f64 },    // formula, not T-02
    All(Vec<Condition>), Any(Vec<Condition>), Not(Box<Condition>),
}
pub enum ExternalTrigger { OnKill, OnWeakPointKill, OnMeleeKill, OnAbilityCast, Invisible, WallLatch, Holster,
                           OnDamaged, Overshields, EnergyAbove(f64), Custom(SmolStr) }

pub struct StackSpec { pub max: u16, pub duration: f64, pub decay: Decay /* One | All | PerStack */,
                       pub gain: Condition /* event that adds a stack */, pub per_gain: u16 }

pub enum OrderClass { Slot, Riven, Innate, Progenitor, InnateHcet, NonCombining }
```

#### 4.4.2 Scopes versus conditions

A **scope** is static applicability and is never affected by T-02: "only heavy attacks" (Killing Blow), "not the
Incarnon form" (Extended Volley), "only bows" (bow variants of fire-rate values), "only on weak-point hits" (Primary
Acuity's crit chance; riven Weak Point Critical Chance, D-010), "only DoT ticks" (status damage). Scope filtering happens
once per attack and hit kind during resolution, so a scoped effect costs nothing on hits it does not apply to.

A **condition** is dynamic: its truth depends on target state, own events or external events, and T-02 decides how it
is treated. The condition class is computed from the condition kind and from the scalars the value uses (an effect
using `DistinctStatuses` is class (a) even without a gate). An effect with several classes (Galvanized Aptitude's CO:
stacks gained on kill (c), multiplied by distinct statuses (a)) is active only when every involved class is active.

`StatBelow`/`StatAtLeast` (Prelude of Might: "with Critical Chance below 40%") are evaluated as part of the formula in
every T-02 mode, using the resolved stat at hit time (it includes Puncture's crit chance, which is target state); the
resolver orders stages so the referenced stat is final before the condition is read, and rejects cycles at
compile time (an effect cannot change a stat its own condition reads). Whether spec 04 wants these under T-02 is open
question Q4 in §11.

#### 4.4.3 Stacks

`StackSpec` replaces the inspiration's "WITH_COND replaces the base value" convention (`I:model` §1.3) with explicit
state: maximum, duration, decay (`One`: lose one stack per timeout, Galvanized; `All`; `PerStack`: independent timers,
Galvanized Scope), gain event and stacks per gain. T-02 `all_max` sets the counter to `max` from t = 0; `none` disables
the effect; `simulated` runs the counter from its gain events (class (b)) or from the user's uptime/stack input
(class (c)), as spec §6.2 defines.

#### 4.4.4 Condition policy (T-02 plus overrides)

```rust
pub struct ConditionPolicy {
    pub mode: ConditionMode,                          // AllMax (default) | None | Simulated
    pub overrides: BTreeMap<EffectRef, CondOverride>, // per item effect: Force(bool) | Stacks(u16) | Uptime(f64)
}
```

Overrides are our improvement over the inspiration's single global switch (`I:opt` §12.4): the UI lists every
conditional effect of the current build with its class and lets the user pin it. The optimizer reports sensitivity to
conditions by re-scoring finalists with `mode = None`.

#### 4.4.5 Element ordering

Combining elements carry an `OrderClass`; resolution turns it into
`OrderKey { class_rank, slot_index, sub_index }` using `rules/slot_layout.toml` (main slots by index, exilus after slot
8 per OQ-024, riven lines bottom-up with the sub-index rules of `F:mods` §8.5, innate and progenitor and Violet shard
last in Heat > Cold > Electricity > Toxin order per D-004 and D-006). Spec 01's combination algorithm consumes the
sorted list `Vec<(OrderKey, DamageType, f64)>`. Element order is therefore data (slot classes) plus one algorithm.

#### 4.4.6 Mechanic kinds

Each kind is implemented once in `wf-engine::mechanics`, in both evaluators, against the spec section named here.

| Kind | Parameters | Used by | Spec |
|---|---|---|---|
| `extra_hit` | `fraction`, `damage_type`, `keeps_body_part`, `status` (`forced` \| `roll_weapon_sc` \| `none`), `melee_factor`, `status_duration_add`, `triggers` (`pellet_hit`, `blast_detonation`) | Xata's Whisper, Toxic Lash, Resupply, Silken Stride, Demonium, Sangodae, Gastroparesis | E-14 |
| `proc_on_event` | `on` (`damage_instance{crit}`, `proc{status}`, `hit`), `chance`, `chance_if` (`fire_rate_below`), `proc` (status \| `random{set}` \| `component_of_combined`), `count`, `exclusive_group` | Hunter Munitions, Internal Bleeding, Magnetic Welt, Archon Continuity, Archon Vitality, Secondary Cryogenic, Secondary Encumber | E-9, `F:addmul` §8 |
| `forced_proc` | `status`, `per` (`pellet` \| `shot`) | weapon traits (stance hits come from stance data) | E-9 |
| `convert_damage` | `from` (types \| `all`), `to` (type \| `param`), `fraction` | Impact-to-Puncture, Chromatic Blade, Reactive Storm | ST-1/ST-2, D-009 |
| `repeat_hit` | `on` (`crit_tier = 1`), `chance` | Melee Duplicate | `04` |
| `flat_damage_on_proc` | `statuses`, `amount`, `scaling` | Cascadia Empowered | `04` |
| `proc_after_ticks` | `combined`, `ticks`, `adds` (`component`) | Primary Debilitate | `03` |
| `counter_bonus` | `stat`, `per_hit`, `reset_on` (`crit_tier >= 2`), `reset_after` | Secondary Enervate | `04` |
| `spawn_attack` | `template`, `fraction`, `on` | Volatile Quick Return explosion, Nightwatch Napalm | `06`, `07` |

Exclusive groups implement "at most one forced Slash per instance, `P = 1 − (1 − a)(1 − b)`" (`F:addmul` §8) for every
member of the group without naming Hunter Munitions in code.

### 4.5 Encoded examples

All values at max rank unless the item has a curve. Comments marked UNVERIFIED repeat the spec's or the findings'
uncertainty and are carried into the UI through `note`. UniqueNames are from `Module:Mods/data` and
`Module:Arcane/data` (`research/external/cache/poc/out/`).

**1. Serration** (base-damage bucket):

```toml
item = "/Lotus/Upgrades/Mods/Rifle/WeaponDamageAmountMod"
label = "Serration"
sources = ["W:Serration", "F:core §2"]
upstream = { field = "wiki.Description", text = "+165% Damage", sha256 = "…" }

[[effect]]
stat = "base_damage"
value = { max = 1.65, curve = "linear", bind = "n1%" }
```

**2. Galvanized Aptitude** (relative status chance plus a CO-family bonus with stacks; adding or multiplying is decided
per attack by `Attack.co`, not by the mod):

```toml
item = "/Lotus/Upgrades/Mods/Rifle/WeaponStatusChanceSPMod"
label = "Galvanized Aptitude"
sources = ["W:Galvanized_Aptitude", "F:mods §6.1", "F:addmul §1.1-1.4"]
upstream = { field = "wiki.Description", text = "+80% Status Chance\nOn Kill:\n+40% Direct Damage per Status Type affecting the target for 20s. Stacks up to 2x.", sha256 = "…" }

[[effect]]
stat = "status_chance.rel"
value = { max = 0.80, bind = "n1%" }

[[effect]]
stat = "co"
value = { max = 0.40, bind = "n2%" }
per = ["stacks", "distinct_statuses"]          # C += 0.40 · stacks · n   (classes (c) and (a))
stacks = { max = 2, bind_max = "n4", duration = 20.0, bind_duration = "n3", decay = "one", gain = { external = "on_kill" } }
```

**3. Condition Overload** (CO family, no stacks; slams get no CO because their attack `co.behavior = "none"`):

```toml
item = "/Lotus/Upgrades/Mods/Melee/WeaponDamageIfVictimProcActive"
label = "Condition Overload"
sources = ["W:Condition_Overload", "W:Condition_Overload_(Mechanic)", "F:addmul §1"]

[[effect]]
stat = "co"
value = { max = 0.80, bind = "n1%" }
per = ["distinct_statuses"]
```

**4. Primary Deadhead** (weapon arcane: stacking base damage on weak-point kills, passive headshot multiplier):

```toml
item = "/Lotus/Upgrades/CosmeticEnhancers/Offensive/PrimaryDamageOnNoMelee"
label = "Primary Deadhead"
sources = ["W:Primary_Deadhead", "F:addmul §5", "F:core §2"]
ignore_numbers = [{ token = "n5%", reason = "recoil, no damage effect" }]

[[effect]]
stat = "base_damage"
value = { max = 1.20, curve = "linear", bind = "n1%" }      # per stack; 3 stacks = +3.60 (F:core §2 example)
per = ["stacks"]
stacks = { max = 3, bind_max = "n3", duration = 24.0, bind_duration = "n2", decay = "all", gain = { external = "on_weak_point_kill" } }
note = "UNVERIFIED: decay rule not stated on the arcane page"

[[effect]]
stat = "headshot_mult"
value = { max = 0.30, curve = "linear", bind = "n4%" }
```

**5. Hunter Munitions** (proc on a critical damage instance, shared exclusive group with Internal Bleeding):

```toml
item = "/Lotus/Upgrades/Mods/Sets/Hunter/PrimaryHunterMunitionsMod"
label = "Hunter Munitions"
sources = ["W:Hunter_Munitions", "F:addmul §8", "D-016"]

[[effect]]
mechanic = "proc_on_event"
on = { event = "damage_instance", crit = "any" }    # one roll per critical instance, tier-independent (OQ-018)
chance = { max = 0.30, curve = "linear", bind = "n1%" }
proc = "SLA"
exclusive_group = "forced_slash"
```

For comparison, Internal Bleeding: `on = { event = "proc", status = "IMP" }`, `chance = { max = 0.35 }`,
`chance_if = [{ when = { fire_rate_below = 2.5 }, factor = 2.0 }]`, same `exclusive_group`.

**6. A riven** (user input, not a curated file). The Furis riven of `research/test-cases/user-example-furis.json`:

```json
{ "slot": "main7", "lines": [ { "stat": "crit_damage", "value": 1.114 }, { "stat": "multishot", "value": 1.481 },
                              { "stat": "status_damage", "value": 1.114 }, { "stat": "zoom", "value": -0.793 } ] }
```

Each riven stat is mapped once in `rules/riven_stats.toml` to an effect template and a UI unit, and validated against
the riven value formula of `F:mods` §8.3 (`b × 90 × 1.2 × L × W`, `b` from the PE `upgradeEntries`):

```toml
[[stat]]
id = "crit_damage";          pe_tag = "WeaponCritDamageMod";  unit = "percent"
effect = { stat = "crit_damage.rel" }
[[stat]]
id = "faction_grineer";      pe_tag = "WeaponFactionDamageGrineer"; unit = "faction_multiplier"   # UI x1.45, stored 0.45
effect = { faction = "grineer" }
[[stat]]
id = "weak_point_crit_chance"; pe_tag = "WeaponWeakPointCritChanceMod"; unit = "percent"           # D-010: real stat, not ignored
effect = { stat = "crit_chance.rel", scope = { hit = { body_part = ["weak_point"] } } }
[[stat]]
id = "heat";                 pe_tag = "WeaponFireDamageMod";  unit = "percent"
effect = { stat = "element.HEA", order = "riven" }
[[stat]]
id = "zoom";                 unit = "percent"
effect = "none"                                                   # no damage effect
```

**7. Roar** (buff: one user parameter, faction bucket for any faction):

```toml
id = "buff.roar"
kind = "buff"
group = "ability"
inspiration_key = "o.r"
sources = ["W:Roar", "F:buffs §1.1", "T-13"]

[[param]]
id = "value"; unit = "percent"; default = 0.30; min = 0.0; max = 99.99
helper = { formula = "rank_value * strength", rank_values = [0.10, 0.15, 0.25, 0.50], subsumed = [0.02, 0.09, 0.15, 0.30] }

[[effect]]
faction = "any"
value = { param = "value" }
```

Eclipse is the same shape with `factor = "eclipse"` (group `eclipse`, stage `final_omitted`, so Adding-CO attacks
omit it, D-058).

**8. Xata's Whisper** (extra hit):

```toml
id = "buff.xatas_whisper"
kind = "buff"
group = "ability"
inspiration_key = "o.x"
sources = ["W:Xata's_Whisper", "W:Extra_Hit", "F:buffs §1.3", "E-14", "D-071"]

[[param]]
id = "value"; unit = "percent"; default = 0.26; min = 0.0; max = 99.99

[[effect]]
mechanic = "extra_hit"
fraction = { param = "value" }
damage_type = "VOI"
keeps_body_part = true                 # E-14: (Xata ? Part : 1)
status = "roll_weapon_sc"
triggers = ["pellet_hit", "blast_detonation"]
note = "UNVERIFIED: blast-detonation trigger magnitude (D-071); modelled in MC only"
```

Toxic Lash: `damage_type = "TOX"`, `keeps_body_part = false`, `status = "forced"`, `melee_factor = 2.0`,
`status_duration_add = 0.25` (D-037).

**9. Violet archon shard, Primary Electricity** (non-mod element ordered with innate elements, value depends on the
number of Crimson, Azure and Violet shards, Tauforged x1.5):

```toml
id = "shard.violet.primary_electricity"
kind = "shard"
color = "violet"
tauforged_factor = 1.5                 # applied to every value of this item when tauforged = true
inspiration_name = "+30% Primary Electricity Damage"
sources = ["W:Violet_Archon_Shard", "F:buffs §5", "D-006", "OQ-024"]

[[effect]]
stat = "element.ELE"
order = "innate_hcet"
value = { base = 0.30, terms = [{ coef = 0.10, per = [{ shard_count = ["crimson", "azure", "violet"] }] }] }
scope = { weapon = { class = ["primary", "archgun"] } }
# check: 5 normal Violet = 5 * (0.30 + 0.10*5) = 4.00; F:buffs §5 says 400 %
```

**10. Incarnon evolution perks** (Furis line; per-variant values, per-attack scope, a resolved-stat condition):

```toml
# data/curated/incarnon/furis.toml
line = "furis"
variants = { furis = "/Lotus/Weapons/Tenno/Pistol/AutoPistol", mk1_furis = "<MK1-Furis uniqueName>" }
sources = ["W:Furis_Incarnon_Genesis"]

[[tier]]
evo = 2
  [[tier.perk]]
  id = "haven_foray"; inspiration_index = 0
    [[tier.perk.effect]]
    stat = "incarnon_flat"
    value = { variant = { furis = 28, mk1_furis = 34 } }
    [[tier.perk.effect]]
    stat = "incarnon_flat"
    value = 30
    gate = { external = "overshields" }
    note = "UNVERIFIED: whether +30 adds to or replaces +X"

[[tier]]
evo = 3
  [[tier.perk]]
  id = "extended_volley"; inspiration_index = 1
    [[tier.perk.effect]]
    stat = "magazine.flat"
    value = 25
    scope = { attack = { none_of = ["incarnon_form"] } }   # Genesis page: "does not apply to the Incarnon form"

[[tier]]
evo = 4
  [[tier.perk]]
  id = "prelude_of_might"; inspiration_index = 1
    [[tier.perk.effect]]
    stat = "crit_damage.base_add"
    value = 3.0
    gate = { stat_below = { stat = "crit_chance", threshold = 0.40 } }   # final CC incl. Puncture bonus (Genesis note)
```

**11. Blood Rush** (relative crit chance scaled by the combo tier above 1; class (b), T-10 can fix the combo):

```toml
item = "/Lotus/Upgrades/Mods/Melee/Event/ComboCritChanceMod"
label = "Blood Rush"
sources = ["W:Blood_Rush", "F:addmul §2"]

[[effect]]
stat = "crit_chance.rel"
value = { max = 0.40, curve = "linear", bind = "n1%" }
per = ["combo_tier_above_1"]            # CC = CC0 · (1 + ΣCC_rel + 0.4·(M − 1)) + ΣCC_abs
```

**12. Arcane Avenger** (warframe arcane, absolute crit chance, external trigger with chance and duration; class (c)):

```toml
item = "/Lotus/Upgrades/CosmeticEnhancers/Defensive/CritChanceOnDamage"
label = "Arcane Avenger"
sources = ["W:Arcane_Avenger", "F:core §8", "F:addmul §5"]

[[effect]]
stat = "crit_chance.abs"
value = { max = 0.45, curve = "linear", bind = "n2%" }
gate = { external = "on_damaged", chance = { max = 0.21, bind = "n1%" }, duration = 12.0, bind_duration = "n3" }
# upstream: "On Damaged:\n21% chance for +45% Critical Chance for 12s"; all three numbers are bound
```

**More, one line each** (all from the spec's bucket list):

| Item | Encoding |
|---|---|
| Primed Chamber + Charged Chamber | `factor = "chamber"` (group stage `other`, additive inside the group), `gate = { first_shot_of_magazine = true }` |
| Synth Charge | `factor = "synth_charge"`, `gate = { last_shot_of_magazine = { min_magazine = 6 } }` |
| Killing Blow | `stat = "base_damage"`, `scope = { attack = { any_of = ["heavy"] } }` |
| Primary Acuity | `weak_point_damage` and `crit_chance.rel` with `scope.hit.body_part = ["weak_point"]`; `stat = "multishot.lock"` |
| Semi-Rifle Cannonade | `base_damage` + `stat = "fire_rate.lock"`; item `trigger_req = "Semi"` |
| Secondary Shiver | `stat = "co"`, `per = [{ status_stacks = "COL" }]`, value 0.45 (D-060) |
| Kunai Incarnon Swift Conclusion | `stat = "co"`, `gate = { target_health_below = 0.5 }` (layered pass only) |
| Vigilante set | set bonus `stat = "crit_tier_raise"`, `value = { table_by = "set_pieces", values = [0.05, 0.10, 0.15, 0.20, 0.25, 0.30] }` |
| Gladiator set | set bonus `stat = "crit_chance.rel"`, `value = 0.10`, `per = ["set_pieces", "combo_tier_above_1"]` |
| Sacrificial set | `member_scale = [1.0, 1.25]` (index = pieces − 1) |
| Corrosive Projection | `stat = "target.armor_strip.aura"`, value 0.18 per aura |
| Emerald shard Corrosion cap | `stat = "status_cap.COR"`, value 2 (tauforged 3: `tauforged_values` overrides the x1.5 rule for integers) |
| Secondary Fortifier | `factor = "overguard_fortifier"` (stage `overguard_only`) |
| Mecha Empowered | `factor = "mecha"` (stage `other`) and `factor = "mecha_dot"` (stage `dot_only`), `gate = { target_marked = true }` (D-041) |
| Melee Doughty | `stat = "crit_damage.rel"`, `per = [{ resolved = "status_chance_of.PUN", step = 0.10 }]` |
| Chromatic Blade | `params = [{ id = "element", choices = ["HEA","TOX","COL","ELE"] }]`; `mechanic = "convert_damage"`, `from = "all"`, `to = { param = "element" }`; status chance `per = ["strength"]` |
| Sentient Incision (`add_weakness_dmg`) | `stat = "element_uncombined"`, `element = "target_weakness"` (lookup in `rules/factions.toml`) |
| Galvanized Scope | `crit_chance.rel` with `gate = { aiming = true }`, `stacks = { decay = "per_stack", gain = { on_hit = { body_part = ["weak_point"] } } }` |
| Custom stat `p.b` "Base Damage per Status" | `custom_stats.toml`: `{ id = "base_per_status", inspiration_key = "p.b", effect = { stat = "co", per = ["distinct_statuses"], value = { param = "value" } } }` |

### 4.6 Coverage matrix: spec mechanics to IR constructs

Every row has a fixture item in `data/fixtures/mini/` and a hand-computed expected value in the `effects_coverage`
test (P-3).

| Spec mechanic (00-overview) | IR construct | Fixture |
|---|---|---|
| Additive base damage A, incl. negatives (§3.1a) | `base_damage` | Serration, negative riven Damage |
| Melee projectile second base bucket | `base_damage.universal` + role scope | Arcane Arachne on a glaive throw |
| Physical P_t; stance physical S^phys | `physical.*`; stance data | Rupture; stance hit with `SlashMultiplier` |
| Elemental E_t with slot order, riven bottom-up, exilus, innate HCET | `element.*` + `OrderClass` | Infected Clip + Stormbringer (COR); riven Heat/Cold; Kuva innate + progenitor |
| Already-combined and non-combining elements | `element.{secondary}`, `element_uncombined.*` | Biotic Rounds, Nourish |
| Progenitor V_t (T-24) | `WeaponPick.progenitor` + `Weapon.progenitor` | Kuva weapon at 0.60 Toxin |
| Incarnon flat X, excluded from B_ref | `incarnon_flat` | Haven Foray |
| Exalted Strength scaling | `Weapon.strength_scaling` + `Scalar::Strength` | exalted fixture weapon |
| Faction F, sub-faction relations, Roar, riven faction | `faction` + `rules/factions.toml` | Primed Bane vs Kuva Grineer; Roar |
| CO adding / multiplying / none per attack, ratio ρ_co | `co` + `Attack.co` | Galvanized Aptitude on hitscan (adding), on Arca Plasmor (multiplying), on a slam (none), Paris charged (ratio 0.5) |
| CO per Cold stack; CO under a health threshold | `co` + `per status_stacks.COL`; `gate target_health_below` | Secondary Shiver; Kunai perk |
| Stance CO input ratio | stance data `co_ratio` | Gnashing Payara row |
| G_F (CO-omitted) and G (other) multipliers, additive pairs | `factor.<group>` + stages | Eclipse; Primed + Charged Chamber |
| First/last shot | `first_shot_of_magazine`, `last_shot_of_magazine` | Primed Chamber, Synth Charge |
| Heavy-only, slam, finisher | role scopes; `factor.slam`, `factor.finisher` | Killing Blow, Nira's set, Covert Lethality |
| Body part ΣWP (1.5x on head), ΣHS, Sonar | `weak_point_damage`, `headshot_mult`, `factor.sonar` (stage `part_spot`) | Primary Acuity, Deadhead, Sonar buff |
| CC relative, absolute, floor, base add, custom final multiplier | `crit_chance.*` | Point Strike, Arcane Avenger, Smeeta Charm, Piercing Navigator, custom `p.d` |
| Combo-scaled CC / SC, Gladiator set | `per combo_tier_above_1`, set bonus | Blood Rush, Weeping Wounds, Gladiator x3 |
| CD relative, absolute (before tiers, D-014), base add, field multiplier | `crit_damage.*` | Vital Sense, Arcane Crepuscular, Prelude of Might, Electric Shield |
| Tier raise (Vigilante) | `crit_tier_raise` | Vigilante x6 |
| SC relative, absolute, base add, multiplier | `status_chance.*` | Rifle Aptitude, Napalm Grenades, Elemental Balance, custom `p.k` |
| Forced procs, Hunter Munitions, Internal Bleeding | `forced_proc`, `proc_on_event` + exclusive group | trait weapon, both mods together |
| Multishot and Acuity lock | `multishot`, `multishot.lock` | Split Chamber, Primary Acuity |
| Status damage (global and per type) | `status_damage`, `status_damage.*` | Elementalist, Emerald shard |
| Status duration, caps | `status_duration`, `status_cap.*` | Lasting Sting, Emerald Corrosion |
| DoT-only multipliers M_dot | `factor.<group>` stage `dot_only` | custom `p.o`, Mecha second dip |
| Extra hits (Xata, Toxic Lash) | `extra_hit` | both buffs |
| Melee Duplicate | `repeat_hit` | Melee Duplicate |
| Vulnerability (multiplicative and additive subgroup) | `target.vulnerability` | Molecular Prime, custom `p.x` |
| Armor strip (ability, aura) and external Viral | `target.armor_strip(.aura)`, `target.status.VIR` | Corrosive Projection, T-07, T-06 |
| Overguard-only multiplier | `factor.<group>` stage `overguard_only` | Secondary Fortifier |
| Fire rate, lock, charge, magazine, reload, ammo efficiency cap | `fire_rate`, `fire_rate.lock`, `charge_rate`, `magazine(.flat)`, `reload_speed`, `ammo_efficiency` | Speed Trigger, Cannonade, Extended Volley |
| Melee timing stats | `attack_speed`, `initial_combo`, `combo_duration`, `heavy_efficiency`, `wind_up_speed` | Fury, Ready Steel aura, Focus Energy |
| Sniper combo, beam ramp, spool, falloff, semi cap | attack data + `rules/*` + toggles T-12, T-27 | sniper, beam and spool fixture weapons |
| Set bonuses (count-based, member scale) | `SetSpec` | Vigilante, Gladiator, Sacrificial |
| Resolved-stat conditions and stat-from-stat | `StatBelow`, `Scalar::Resolved` | Prelude of Might, Melee Doughty |
| Target-dependent element | `element = "target_weakness"` | Sentient Incision |
| Item parameters | `ParamSpec`, `convert_damage{to = param}` | Chromatic Blade |
| Zoom levels | `Weapon.zoom_levels` gated by `aiming` | Rubico |
| Custom stats (26) and toggles (29) | `custom_stats.toml`, `Toggles` + `rules/toggles.toml` | one per custom stat |
| Kitgun and zaw assembly | `ModularParts` + `assemble` | Catchmoon/Haymaker/Splat; a zaw |
| Incarnon form attack and perks per attack | role `INCARNON_FORM`, `IncarnonLine`, role scopes | Furis line |

### 4.7 Lowering: from TOML to the compiled form

`wf-effects::compile(doc, &Catalogue) -> Vec<Effect>` checks names, desugars sugar (`for_attack.heavy = { factor = 2 }`
becomes two scoped effects; `tauforged_factor` multiplies values), and binds text tokens. At build-resolution time
`wf-engine::resolve` turns an `Effect` plus the build context (rank, variant, params, set counts, shard counts) into a
`Contribution` whose value is an `Affine`:

```rust
pub struct Contribution {
    pub source: SourceRef,                     // (item Sym, effect index) for traces and canonical ordering
    pub lane: Lane,                            // stat lane, factor group lane, faction lane, or mechanic slot
    pub value: Affine,                         // c + Σ coef · Π scalars
    pub gate: Gate,                            // { classes: ClassMask, predicate: Option<CompiledCondition>, stack: Option<CondId> }
    pub order: Option<OrderKey>,
}
pub struct Affine { pub c: f64, pub terms: SmallVec<[(ScalarProduct, f64); 2]> }
```

Galvanized Aptitude at rank 10 on a rifle lowers to two contributions:

```text
Contribution { lane: Stat(status_chance.rel), value: Affine{ c: 0.80, terms: [] }, gate: always }
Contribution { lane: Stat(co), value: Affine{ c: 0.0, terms: [([Stacks(#57), DistinctStatuses], 0.40)] },
               gate: { classes: A|C, stack: #57 (max 2, 20 s, decay one, gain OnKill) } }
```

and at rank 4 the coefficients are 0.80·5/11 and 0.40·5/11. Bucket totals are sums of contributions in canonical
order (P-5), so the `co` lane of a build with Galvanized Aptitude and the custom stat `p.b` is one `Affine` with two
terms that the EV evaluator takes the expectation of and the MC evaluator evaluates against live state.

---
## 5. Engine

### 5.1 Public API (`wf-engine`)

```rust
pub struct Engine { data: Arc<GameData> }

impl Engine {
    pub fn new(data: Arc<GameData>) -> Self;

    /// Resolve the scenario: contributions -> per-attack stats (ST-1..ST-7) -> schedules (06/07) -> enemy (05).
    pub fn prepare(&self, sc: &Scenario, opts: PrepareOptions) -> Result<Prepared, PrepareError>;

    /// Deterministic expected-value evaluator (spec §4.2).
    pub fn evaluate_ev(&self, p: &Prepared, req: &EvRequest) -> EvReport;

    /// Event-driven Monte Carlo (spec §4.3). Runs are parallel (rayon) and merged in run order.
    pub fn simulate(&self, p: &Prepared, req: &McRequest, cancel: &CancelToken) -> Result<McReport, Cancelled>;

    /// Per-hit event log of one run (the inspiration's "More details" timeline).
    pub fn timeline(&self, p: &Prepared, attack: &AttackKey, seed: u64, run: u32) -> Timeline;

    /// Spec-symbol access for test vectors and the Explain drawer: `cc`, `cd`, `ratio_q[COR]`, `dps_burst`, ...
    pub fn observe(&self, p: &Prepared, ev: Option<&EvReport>, obs: &Observable) -> Option<f64>;
    pub fn explain(&self, p: &Prepared, obs: &Observable) -> Explanation;
}

pub struct PrepareOptions { pub trace: bool, pub compat: CompatFlags }
pub struct EvRequest  { pub views: LayerViewSet, pub modes: EvModes /* steady, window, ttk */, pub breakdown: bool }
pub struct McRequest  { pub views: LayerViewSet, pub n_runs: u32, pub seed: u64, pub series_bin: f64,
                        pub layered_ttk: bool, pub parity: bool }
```

The optimizer uses a lower-level, allocation-free API on top of the same stage functions (§6.2):
`LaneModel::compile(&Engine, &Scenario, &SearchSpace) -> LaneModel` and `LaneModel::score(&Lanes) -> f64`.

### 5.2 Internal pipeline

```text
Scenario
  └─ collect      every equipped item, perk, shard, set bonus, buff, custom stat, weapon/attack passive, zoom level
                  -> Vec<Contribution> (4.7), canonical order (P-5)
  └─ scope        per selected attack and hit kind (direct-body, direct-head, direct-weak-point, radial, dot, extra-hit)
                  -> AttackLanes { lanes: [Affine; N_LANES], factors: Vec<(FactorGroupId, Affine)>, mechanics, elements }
  └─ stages       st1_base_vector -> st2_composition -> st3_quantize -> st4_base_scalar -> st5_weapon_multipliers
                  -> st6_body_part -> st7_crit                       (each a pure fn citing its E-number)
                  -> PreparedAttack { stats: AttackStats, mechanics, co }
  └─ schedule     ranged (06) or melee (07) generator -> Schedule { shots: Vec<ShotEvent>, cycle: CycleStats }
  └─ enemy        st8 inputs: scaled pools, armor, per-type modifiers per layer, immunities, caps (05)
  └─ Prepared ── evaluate_ev  (expectations over scalar distributions)
              └─ simulate     (live scalar values in an event loop)
              └─ outputs      (§5.6)
```

Stage functions are pure, take an explicit `&StageCtx` (rules, compat flags, trace sink), and return plain structs.
Each lives in a module named after its owner document, so a reviewer can put the spec and the code side by side:

| Module | Spec owner | Contents |
|---|---|---|
| `composition` | 01 | E-1..E-3, element combination, progenitor, Incarnon flat split, conversions, faction relations |
| `crit_status` | 02 | E-4..E-10, body part, proc weights, forced procs, exclusive proc groups |
| `status` | 03 | status rules from `rules/status.toml`, E-13, Heat pool, Blast fuse, Gas, Magnetic break, distinct count |
| `conditional` | 04 | CO term (E-11), factor groups, condition classes and T-02, stacks, extra hits (E-14), vulnerability catalogue |
| `enemy` | 05 | scaling (NP-2), armor E-15, layers, routing, overflow, attenuation, special DR |
| `ranged_schedule`, `melee_schedule` | 06, 07 | schedule generators, cycle stats, combo, sniper combo, ramp, spool |
| `eval_ev`, `eval_mc`, `outputs` | 08 | evaluators, views, TTK, parity outputs, compat profile |
| `mechanics` | 02-04 | one impl per mechanic kind (4.4.6) for both evaluators |
| `spec_constants` | all | the only place with numeric literals, each with an E-number (P-1) |

Build legality (spec 09) is not in the engine: `wf-data::legality` answers "may this item go in this slot of this
build" for the UI, the build decoder and the optimizer, so all three agree.

### 5.3 Why bucket totals are affine forms

Every lane is `Affine = c + Σ coef · Π scalars`. The two evaluators differ only in how they read scalars:

```rust
pub trait ScalarEnv { fn get(&self, s: Scalar) -> f64; }          // MC: live target state, counters, combo
pub struct Expectation<'a> { dists: &'a ScalarDistributions }       // EV: distributions (5.4)
```

Consequences:

- One resolution feeds both evaluators, so they cannot disagree on which bonuses apply (P-4).
- Spec CD-13 is enforced: a nonlinear function of a scalar (Viral multiplier, armor DR, `cold_cd`, quantization of a
  conditional element) never receives `E[scalar]`; the EV evaluator integrates it over the scalar's distribution.
- The optimizer sees which lanes are constant for a given scenario (`terms.is_empty()`), which lanes are linear in
  independent scalars (expectation is exact and cheap) and which need enumeration; this classification drives its fast
  path (§6.2).

### 5.4 Expected-value evaluator

The evaluator follows spec §4.2; spec 08 owns the algorithm, this is the structure that implements it.

1. **Proc rates.** From `PreparedAttack` and the schedule: expected pellets per shot `MS`, procs per pellet per
   status `SC·w_s + φ_s`, plus `proc_on_event` mechanics (Hunter Munitions `p·P(crit)`; exclusive groups combine as
   `1 − Π(1 − p_i)`), giving `λ_s` per second (steady) or per shot (window).
2. **Scalar distributions.** For each status: live-proc count Poisson(μ_s) capped at `cap_s` (enemy caps and
   `status_cap.*` applied), so `P(n_s = k)` for k = 0..cap; presence `1 − e^{−μ_s}`; `E[n]` by linearity plus external
   statuses (T-06). Own stack counters (`Stacks(id)`): `all_max` → point mass at max; `simulated` class (b) → renewal
   model from the trigger's event rate, duration and decay; class (c) → the user's uptime/stack input as a point mass
   or Bernoulli. Combo tier, sniper combo, ramp: point values per schedule position (window) or steady values.
3. **Factorization.** The damage of one instance (E-12, E-13, E-14) is a product of factors. The evaluator groups
   factors by the scalar sets they read. Factors over disjoint scalar sets multiply their expectations (independence
   across status types is the spec's model); factors that share scalars (armor DR reads Corrosive stacks and Heat
   presence; crit chance reads Puncture stacks and crit damage reads Cold stacks inside the same `E[c]`) are evaluated
   by enumerating the joint support of the shared scalars (at most 11 x 11 x 2 states in practice).
4. **Nonlinear conditional stages.** When a gated contribution feeds a nonlinear stage (a conditional element changes
   quantization, e.g. Melee Exposure under `simulated`), the evaluator splits on that gate's state (≤ 4 such gates;
   beyond that it falls back to the linear approximation and emits warning `ev.approx.gate_split_limit`).
5. **Modes.** Steady (`dps_burst`, `dps_sustained`) with μ_s = λ_s·D_s; window (`dps_window`, EV TTK) walks the
   schedule shot by shot with μ_s(τ) from earlier shots and lands DoT ticks at their times (Heat pool by its window
   mean, spec §4.2). Little's law for steady DoT.
6. **Outputs** (§5.6), each with an optional per-source and per-type breakdown.

### 5.5 Monte Carlo simulator

- **State.** Target: per-status stack lists with expiry times (independent timers, replacement beyond cap, CD-07), DoT
  instances with snapshotted weapon-side seeds (CD-05), shared Heat pool and strip ramp, Blast fuses, pools per layer
  for the `layered` pass. Attacker: magazine, reload state, combo counter, sniper combo, ramp, spool, own stack
  counters per `CondId`, mechanic counters (`counter_bonus`).
- **Events.** A binary heap ordered by `(time, phase, seq)`, where `phase` encodes CD-04 order (damage computed from
  the state before the instance, then crit and procs rolled, damage applied, procs applied; Electricity and Gas tick 0
  after procs) and `seq` is creation order.
- **RNG.** Philox4x32-10 implemented in `eval_mc::rng` (about 60 lines, tested against the Random123 known-answer
  vectors). Key = `seed`; counter = `(run, attack_index << 16 | mechanic_id, event_index_lo, event_index_hi)`.
  `MechanicId` is a `#[repr(u16)]` enum (`CritTier`, `TierRaise`, `PelletCount`, `ProcCount`, `ProcType`,
  `ProcOnEvent(k)`, `ExtraHitStatus`, `RepeatHit`, ...). Draws are exact `f64` in [0, 1) from 53 bits; no rounding of
  chances (D-017). Same key and counter give the same draw in any build, so finalists are compared with common random
  numbers.
- **Two passes per run** (spec §4.3): non-depleting layer views, then the `layered` pass with depletion, shield gate
  and overflow (CD-08) to TTK or `t_ttk_max`, reusing the same draws.
- **Parallelism.** `(0..n_runs).into_par_iter().map(run).collect::<Vec<RunResult>>()` then a sequential reduction in run
  order (P-6). Cancellation is checked between runs.

### 5.6 Outputs

```rust
pub struct EvReport { pub attacks: Vec<AttackEv>, pub groups: Vec<GroupEv>, pub warnings: Vec<Warning> }
pub struct AttackEv { pub attack: AttackKey, pub views: Vec<ViewEv> }
pub struct ViewEv {
    pub view: LayerView,                 // health | armor | shield | overguard | layered
    pub dps_burst: f64, pub dps_sustained: f64, pub dps_window: f64,
    pub per_type: DamageVec, pub per_source: Vec<(SourceKind, f64)>,   // direct, DoT per status, extra hits, specials
    pub stacks: Vec<StackStat>,          // E[n_s], P(n_s >= 1), P(n_s = cap_s)
    pub distinct_mean: f64, pub dmg_per_shot: f64, pub dmg_per_mag: f64,
    pub crit_tiers: Vec<f64>, pub ttk_approx: Option<TtkApprox>,       // with `extrapolated` flag
}
pub struct McReport { pub attacks: Vec<AttackMc>, pub groups: Vec<GroupMc>, pub ev_deviation: Vec<DeviationFlag> }
pub struct ViewMc {
    pub view: LayerView, pub dps_window: Summary /* mean, sd, ci95, median, p5, p25, p75, p95 */,
    pub series: Vec<BinSummary>, pub uptime: Vec<StatusSeries>, pub crit_hist: Vec<u64>,
    pub ttk: Option<TtkDist>, pub shots_to_kill: Option<Summary>, pub ammo_to_kill: Option<Summary>,
    pub pending_dot: f64, pub breakdown: Vec<(SourceKind, f64)>,
    pub parity: Option<ParityMetrics>,   // queue_dps_parity, avg per bullet (direct/status/total), median per bullet, TTK
}
```

Every output in spec §4.5 (parity and added) has a field; `outputs::tests` checks the mapping table row by row.

### 5.7 Compatibility profile

`CompatFlags` (bitflags in `wf-model`) has one flag per register row whose Toggle column says `Compat`, named after
the row: `D001_QUANT_1_16`, `D002_QUANT_SCOPE`, `D003_NO_CD_QUANT`, `D014_ABS_CD_AFTER_TIERS`, ... `D089_TTK_SIM`.
`CompatFlags::INSPIRATION` sets all of them. Every branch on a flag is a one-line `if ctx.compat.contains(..)` in the
stage function that owns the row, with a `// D-0xx` comment. `xtask spec-sync` parses the register in
`00-overview.md` and fails when a `Compat` row has no flag or a flag has no row (P-9).

The trace records every register row a scenario actually exercised (`Trace::touched_rows`), for example "D-016 Hunter
Munitions roll" or "D-026 Heat strip ramp". The oracle harness uses it to decide automatically whether a scenario is
oracle-safe (spec §4.6): in the game profile, no touched rows; in the compat profile, no touched rows with Toggle `—`.

### 5.8 Numerics and determinism

- `f64` everywhere (U-4). `numeric::np1_quantize_crit_mult(f64) -> f64` and `numeric::np2_blend(f1, f2, s) -> f64`
  compute in `f32` exactly as E-4 and NP-2 specify, with unit tests against the spec's worked numbers
  (Q = 2.0004885 for CDM0 = 2.0; Grineer health 294.4855 at level 100).
- Quantization `q_t = sign(r)·floor(|r|·32 + 0.5)/32` in `f64` (exact in binary).
- Units at the boundary only: the pipeline converts percents to fractions; `wf-model` stores fractions; the UI converts
  back (U-1). Newtypes (`Frac`, `Seconds`, `Hp`) are used in `wf-model` public structs and in API types; hot loops use
  plain `f64`.
- Deterministic containers in all calculation paths (`Vec` indexed by `Sym`, `BTreeMap`); `HashMap` is allowed only
  for caches whose iteration order is never observed (clippy `disallowed_types` in engine crates enforces it).
- Canonical summation order (P-5) and ordered parallel reductions (P-6).

### 5.9 Performance targets

Targets per core on a current desktop x86-64 CPU, release build, measured by `criterion` benchmarks in
`crates/wf-engine/benches/` on the spec §3.8 scenario (Braton Prime, 9 items, Lancer level 100 Steel Path). They are
targets, not measurements; CI reports regressions above 20 %.

| Operation | Target |
|---|---|
| `prepare` (resolve + schedule) for one attack | ≤ 20 µs |
| `evaluate_ev` steady (burst + sustained), one attack, one view | ≥ 50,000 builds/s/core including `prepare` |
| `LaneModel::score` steady, optimizer fast path (no allocation, lanes precompiled) | ≥ 500,000 builds/s/core |
| `evaluate_ev` window, 20 s at ~15 shots/s and MS 1.9 | ≥ 5,000 builds/s/core |
| `simulate`, same scenario (20 s window: ≈ 213 shots, ≈ 405 pellets, plus DoT ticks; from §3.8 FR 15.328, duty 0.6947, MS 1.9) | ≥ 2,000 runs/s/core (1,000 runs in ≈ 60 ms on 8 cores) |
| Optimizer bound evaluation in branch-and-bound | ≥ 5,000,000 nodes/s/core |

The effect IR costs nothing at evaluation time: it is compiled away into lanes during `prepare`, so the data-driven
design is not paid for in the hot loop.

### 5.10 Spec test vectors become unit tests

Topic documents put test vectors (template item 10) in fenced blocks with the info string `testvector`, in TOML. Two
levels:

````text
```testvector
id = "TV-00-03"
spec = ["E-3", "§3.8 step 3"]
level = "stage"
stage = "quantize"                      # name in wf-engine's stage registry
[input]
ratio = { IMP = 0.05, PUN = 0.35, SLA = 0.60, COR = 1.80 }
[expect]
ratio_q = { IMP = 0.0625, PUN = 0.34375, SLA = 0.59375, COR = 1.8125 }
tolerance = { abs = 0.0 }
```

```testvector
id = "TV-00-10"
spec = ["§3.8"]
level = "scenario"
scenario = "scenarios/braton_prime_lancer_l100_sp.toml"   # ScenarioDoc over the fixture bundle
[expect]
"enemy.pool.HP"                            = { value = 73621.375, abs = 1e-6 }
"attack[normal].cd"                        = { value = 4.4010749, rel = 1e-7 }
"attack[normal].view[armor].dps_burst"     = { value = 11496.26, rel = 1e-5 }
"attack[normal].view[armor].steady.dot.SLA" = { value = 2162.23, rel = 1e-4, state = "B" }
```
````

`cargo xtask spec-vectors` extracts every block from `docs/mechanics/*.md` into
`crates/wf-engine/tests/spec_vectors/<doc>.toml` (generated, committed); `tests/spec_vectors.rs` runs them: stage
vectors call the stage function by name through a registry (`stage_registry!` maps names to `fn(&Input) -> Output`
with serde types); scenario vectors decode the `ScenarioDoc`, `prepare`, evaluate and read observables by spec symbol
(`Observable::parse("attack[normal].view[armor].dps_burst")`). A failing vector prints its id and spec section. CI
runs `cargo xtask spec-vectors --check`, so a spec edit without regenerated vectors fails, and a vector without a
matching stage or observable fails too.

### 5.11 Traces and the Explain drawer

With `PrepareOptions { trace: true }` every stage writes `TraceEntry { observable, equation: "E-5", inputs:
Vec<(Observable, f64)>, contributions: Vec<(SourceRef, Lane, Affine)>, touched_rows }` into a `TraceSink`. The
no-op sink is a zero-sized type, so tracing costs nothing when off. The UI renders it as "Critical chance 75.0 % =
12 % base × (1 + 150 % Point Strike) + 45 % Arcane Avenger (absolute, E-5)", with each source linking to its item and
its citation.

---
## 6. Optimizer

### 6.1 Request and result

```rust
pub struct OptimizeRequest {
    pub scenario: ScenarioDoc,            // weapon, enemy, toggles, buffs and the current build (start point, locks)
    pub objective: Objective,
    pub constraints: Constraints,
    pub search: SearchSettings,
}
pub struct Objective {
    pub metric: Metric,                   // DpsWindow (default) | DpsBurst | DpsSustained | Ttk | HitFresh | HitInFight
    pub attacks: Vec<(AttackKey, f64)>,   // weighted multi-attack objective; default: the first attack, weight 1
    pub layer: LayerSel,                  // Auto (armor if armored, else health) | Health | Armor | Shield | Overguard | Layered
    pub rank_by: RankBy,                  // Ev (default) | McMean | McMedian (finalists only)
}
pub struct Constraints {
    pub locked_slots: Vec<SlotId>, pub keep_installed: bool,
    pub must_include: Vec<ItemId>, pub prohibited: Vec<ItemId>,
    pub exclude_elements: Vec<DamageType>, pub require_elements: Vec<DamageType>,
    pub groups: GroupPermissions,         // exilus, arcane(s), stance+combo, incarnon tiers, frame mods (0..4), frame arcanes,
                                          // aura, shards: Off | Fixed | Search
    pub buffs: Vec<BuffPermission>,       // { buff, mode: Off | On | Choose, params }
    pub riven: RivenMode,                 // None | Fixed(Riven) | Explore { max_positive: 3, allow_negative: bool, quality: f64 }
    pub progenitor: ProgenitorMode,       // Locked | Search { value: f64 }
    pub capacity: Option<CapacityMode>,   // { weapon_rank, reactor, forma_budget, allow_underrank, aura_or_stance_bonus }
    pub exclusions: ExclusionRules,       // no_faction, no_on_kill, no_status_duration_only, headshot-only items off when body hits
    pub conditions: ConditionPolicy,      // T-02 mode and overrides during search
}
pub struct SearchSettings {
    pub time_budget: Option<Duration>,    // anytime; default 120 s, usually finishes earlier
    pub threads: usize,                   // 0 = all cores
    pub beam_width: u16,                  // only for fallback tasks (default 64)
    pub finalists: u16,                   // default 20
    pub mc_runs: u32, pub seed: u64,      // finalist confirmation (default 1000, 0)
    pub preset: Preset,                   // Default | InspirationCompatible (§6.8)
}
pub struct OptimizeResult {
    pub builds: Vec<RankedBuild>,         // top distinct builds, at least one per element class among the top 3 classes
    pub best_per_element_class: Vec<(ElementClass, RankedBuild)>,
    pub stats: SearchStats,               // tasks, nodes, evaluations, pruned, elapsed, exhaustive: bool
}
pub struct RankedBuild {
    pub scenario: ScenarioDoc, pub ev: f64, pub mc: Option<Summary>, pub ev_mc_deviation: Option<f64>,
    pub leave_one_out: Vec<(SlotId, f64)>,           // contribution of each equipped item
    pub runner_up: Vec<(SlotId, ItemId, f64)>,       // best replacement per slot and its score
    pub conditions_off: Option<f64>,                 // score with ConditionMode::None
    pub capacity_used: Option<CapacityReport>,
}
```

### 6.2 Problem compilation

1. **Positions and pools.** The build space is a list of positions, each with a pool from `wf-data::legality`: 8 main
   slots (one shared pool, no duplicates), exilus, arcane(s), stance and combo, up to 5 Incarnon tiers (2-3 perks
   each), frame mods, aura, 2 frame arcanes, 5 shard positions (a multiset), riven. Locked slots become fixed
   positions. Items with `status = "unmodelled"` and `no_damage_effect` are dropped (P-8); the result lists them.
   Incompatibility pairs (wiki lists, union of both directions, D-093), duplicate rules, `trigger_req` (Cannonade)
   and arcane class restrictions come from the same legality module the UI uses; the search keeps a per-node
   exclusion set, so an incompatible pair is never generated.
2. **Lanes.** For each chosen attack, `LaneModel::compile` resolves every candidate once in the scenario context and
   stores its contributions as a sparse lane vector (`SmallVec<[(LaneIdx, f64); 6]>` plus a few `Affine` terms for
   scalar-dependent lanes). A build's lanes are sums, so adding or removing an item is an O(entries) delta.
   `LaneModel::score(&Lanes)` evaluates the steady EV closed form (§5.4 steps 1-3) with the scenario's constant parts
   precomputed.
3. **Item classes.** The compiler classifies every candidate:
   - *monotone*: all entries feed lanes in which the relaxed objective (step 5) is non-decreasing; the large majority;
   - *non-monotone*: locks (Primary Acuity's multishot lock, Cannonade's fire-rate lock), negative riven lines,
     threshold conditions (`StatBelow`: Prelude of Might), items that can lower a status share below a threshold;
   - *coupled*: set members (bonus depends on the count), shards (`ShardCount` scalars).
4. **Dominance pruning.** Inside a pool, item B is removed if some item A has `entries(A) ≥ entries(B)` lane by lane,
   the same coupling and conditions, and no higher drain when capacity applies (`I:opt` §12.2 step 1).
5. **Bounds.** `LaneModel::upper_bound(&Lanes)` evaluates a relaxation: every monotone factor at the given (upper)
   lane values, and every target-state factor (Viral, Corrosive strip, Cold, Puncture, distinct count) at its value for
   the largest proc rate any completion could reach (per-status optimistic share times the upper status chance). Each
   relaxed factor is non-decreasing by construction, which the compiler checks symbolically per lane; a lane that
   cannot be proven monotone marks the task as a beam-fallback task.

### 6.3 Search

**Phase 1, outer tasks (parallel).** Tasks = element class × progenitor element (if not locked). An element class is
the canonical outcome of combination given the innate, progenitor and shard elements: the multiset of final element
types (for example `{COR, HEA}`, `{VIR}`, `{RAD, TOX}`); in practice 20-40 classes. Inside a class, damage depends only
on the per-element sums of contributing items, so element lanes are ordinary lanes and the order is recovered at the
end by a realizer (a ≤ 8! permutation search that honours locked slots and riven bottom-up rules; usually trivial).
This removes WFOpt's "elements chosen on a bare weapon" weakness: every class is searched with the full build.

**Phase 2, inner exact search per task.** Branch-and-bound over positions in a fixed order (non-monotone items
first as include/exclude branches, then positions sorted by decreasing best marginal bound). At each node:
`bound = upper_bound(L + Σ_lane top_k_remaining(lane))`, where `top_k_remaining` sums the best `k` remaining
contributions per lane independently (precomputed sorted lists per pool). A node is cut when its bound is below the
`K`-th best complete score found (shared `AtomicU64` of the K-th score, monotone, plus a `Mutex<TopK>`). Because the
relaxation is a valid upper bound, the top-K set is exact and, with ties broken by the canonical build hash, identical
for any thread count and timing. Coupled items: set bonuses are bounded by assuming the maximum count still reachable;
shard positions are expanded as multisets (C(n+4, 5): 792 for the 8 shard bonuses that can matter to one weapon
after pruning, 26,334 for all 18); Incarnon tiers, stance combos, aura, frame
arcanes are ordinary small positions. Capacity mode adds a knapsack check per node (remaining capacity, remaining
forma; the optimal forma placement for a fixed set of mods is greedy by drain saving, so feasibility of a complete
build is exact). Tasks whose lanes are not provably monotone, or a metric without a monotone relaxation, run a
deterministic beam search of width `beam_width` instead and are marked `exhaustive = false` in the result.

Tasks are processed in decreasing order of their root bound; a task whose root bound is below the K-th incumbent is
skipped without expansion. The incumbent is seeded by the user's current build and by a greedy fill per task.

**Phase 3, exact objective and local search.** The proxy in phases 1-2 is the steady EV (`dps_burst` or
`dps_sustained`); for `DpsWindow`, `Ttk` and the hit metrics, the top `N = 200` proxy builds are re-scored with the exact
EV objective (window walk, EV TTK). Then a deterministic local search on the exact objective from the top 20: 1-swaps
and 2-swaps over all positions, element re-ordering within the realizer's options, cross-group coordinate ascent
(arcane, exilus, perks, shards, frame slots, stance combo), and buff selection when a buff's mode is `Choose`. Restarts
use fixed seeds. This also covers metrics whose proxy ranking differs from the final metric (WFOpt Q10).

**Phase 4, finalists.** The top `finalists` distinct builds are simulated with MC (`mc_runs`, common random numbers:
same seed, same mechanic keys) for the objective's views. The result reports mean ± 95 % CI, the EV-versus-MC
deviation per build (spec §4.4 tolerance: 99 % CI plus 2 %; larger deviations are flagged), leave-one-out
contributions, the runner-up per slot, and the score with conditions off. `rank_by = McMean` re-ranks finalists by MC.

**Riven explore mode.** A riven position whose pool is generated from `rules/riven_stats.toml` for the weapon class:
2-3 positive lines at the chosen roll quality plus an optional negative, with legality (at most one riven, element
limits) from spec 09. Lines are lanes, so the riven is an ordinary item to the search.

### 6.4 Parallelism, determinism, caching

- One rayon pool for optimizer jobs (`threads`, default all cores); tasks are independent; nodes inside a task are
  sequential (no fine-grained locking); the only shared state is the top-K incumbent.
- Determinism: results do not depend on thread count or timing (exact pruning plus canonical tie-breaks); beam
  fallback tasks are deterministic sequential algorithms. A test runs the same job with 1 and 8 threads.
- Canonical build key: (sorted non-element items, element class + realized order, other positions) hashed with
  blake3; the memo cache for exact-objective scores is per job, keyed by `(canonical key, context hash)` where the
  context hash covers enemy, toggles, buffs, conditions and window (no flush discipline, avoids WFOpt Q12).
- Memory: lane models are a few KB per attack; the top-K and memo are bounded (memo LRU 1 M entries).

### 6.5 Progress reporting and cancellation

```rust
pub enum OptEvent {
    Started   { plan: Plan /* tasks, positions, pool sizes after pruning, exhaustive_expected */ },
    Phase     { phase: PhaseId, message: MessageKey },
    Progress  { tasks_done: u32, tasks_total: u32, nodes: u64, evaluations: u64, elapsed_ms: u64, best: Option<f64> },
    Improved  { rank: u16, build: ScenarioDoc, score: f64 },
    Finalist  { build: ScenarioDoc, ev: f64, mc: Summary, deviation: Option<f64> },
    Log       { level: LogLevel, key: MessageKey, args: Vec<MessageArg> },   // i18n keys, never English text (P-10)
    Done      { result: OptimizeResult },
    Cancelled { best: Option<OptimizeResult> },
    Failed    { key: MessageKey, args: Vec<MessageArg> },
}
pub trait EventSink: Send + Sync { fn emit(&self, e: OptEvent); }
```

`Progress` is throttled to 4 per second; `Improved` is sent on every top-1 change. Cancellation is an
`Arc<AtomicBool>` checked every 4,096 nodes and between MC runs (worst-case latency well under 100 ms); a cancelled job
returns its best-so-far, and the time budget ends a job the same way.

### 6.6 How this beats the inspiration's weaknesses

| WFOpt weakness (`I:opt` §9) | Our answer |
|---|---|
| 1. Each candidate costs 3 full stochastic simulations | analytic lanes: ≥ 500 k steady scores/s/core and ≥ 5 M bound evaluations/s/core (targets, §5.9) |
| 2. Biased, noisy estimate, winner's curse | deterministic EV objective; MC only to confirm finalists, with CRN and CI |
| 3. 80 % trust tolerance | per-finalist EV-versus-MC check at the spec §4.4 tolerance, flagged in the result |
| 4. Elements chosen on a bare weapon | element classes are outer tasks searched with the full build |
| 5. Element order searched once | realizer plus reorder moves in local search; order is a property of the class, not an accident of slot index |
| 6. Beam pruning by marginal value drops synergies | exact branch-and-bound with valid relaxation bounds; beam only as a flagged fallback |
| 7. Greedy perks, stance (first combo), frame slots | all positions in one search; stance combos are positions; shards as multisets |
| 8. Order dependence of sweeps | exact search, then deterministic local search with restarts |
| 9. Median of 3 seeds | no sampling during search |
| 10. TTK quantized by shot times, tie-break hack | EV TTK is continuous; MC TTK distribution for finalists |
| 11. Single-threaded UI-thread JavaScript | rayon on all cores, server-side, the UI stays responsive |
| 12. Exclusions by regex on tooltip text | typed condition classes and exclusion rules on data (`ExclusionRules`) |
| 13. Buff phase only under "the last scenario" | buffs are positions (`Choose`), or fixed per the user's choice |
| Gaps (`I:opt` §8): no capacity/polarity, no riven search, no set awareness, one attack, no per-slot locks | capacity mode, riven explore, coupled set handling, weighted multi-attack objective, per-slot locks, must-include |

### 6.7 Inspiration-compatible preset

`Preset::InspirationCompatible` sets the inspiration's defaults (`I:feat` §17.1, `I:opt` §2): metric DPS, layer
`Auto`, no faction bonus on, conditions met on, prohibited defaults (Mecha Empowered, Proton Snap, Arcane Blade
Charger, Arcane Primary Charger, Arcane Arachne), status-duration-only items excluded, search window
`clamp(round(100 / attack speed), 5, 20)` s, one frame-mod slot. The audit task uses it to show that our best build
scores at least as high as WFOpt's under our own honest metric on the same scenarios (`I:opt` §12.3).

---

## 7. Server

### 7.1 Process and startup

```text
warframe-dps [serve] [--port 7878] [--no-browser] [--data-dir DIR] [--bundle FILE] [--log-level info]
warframe-dps data {status | check | refresh [--curated DIR] | list | activate <id>}
warframe-dps eval <build-code> [--mc 1000]            # prints EvaluateResponse JSON (debugging, scripted tests)
warframe-dps optimize <build-code> [--metric dps_window] [--time 60]   # headless optimizer, NDJSON events
```

`serve` loads the newest compatible user bundle from `<data_dir>/bundles/` or the embedded one, binds `127.0.0.1`
(and `::1`) on the configured port, and opens `http://127.0.0.1:<port>/` with the `open` crate unless `--no-browser`.
If the port is taken and `GET /api/meta` on it answers with our app id, it opens the browser on the running instance
and exits; otherwise it tries the next 10 ports, then an OS-assigned port.

### 7.2 Routes

| Method and path | Request | Response |
|---|---|---|
| `GET /api/meta` | — | `Meta { app_id, app_version, schema_version, data_version, licences, features }` |
| `GET /api/catalog` | — | `Catalog` (weapons, enemies, items as summaries with short ids, categories, i18n names, image names); `ETag` = bundle hash; immutable |
| `GET /api/weapons/{short}` `/api/enemies/{short}` `/api/items/{short}` | — | details; items include rendered effects, upstream text, curation status, sources |
| `GET /api/schema/scenario` | — | `ScenarioSchema { toggles, buffs, custom_stats, slot_layouts, riven_stats }` for schema-driven UI |
| `POST /api/candidates` | `{ scenario, slot, query? }` | `Vec<Candidate { item, legal: Legality /* Ok \| Blocked(MessageKey, args) */ }>` |
| `POST /api/evaluate` | `EvaluateRequest` | `EvaluateResponse` |
| `POST /api/timeline` | `{ scenario, attack, view, seed, run }` | `Timeline` (per-hit cards, reload markers, stacks) |
| `POST /api/explain` | `{ scenario, observable }` | `Explanation` |
| `POST /api/build/encode` / `decode` | `ScenarioDoc` / `{ code }` | `{ code, url }` / `{ scenario, warnings }` |
| `POST /api/build/import` | `{ text }` (our code or a warframe-damage.com URL) | `{ scenario, report: ImportReport }` |
| `POST /api/optimize` | `OptimizeRequest` | `202 { job_id }` |
| `GET /api/optimize/{id}/events` | `Last-Event-ID` header optional | `text/event-stream` of `OptEvent` (ring buffer of the last 1,000 events for reconnects) |
| `GET /api/optimize/{id}` / `DELETE /api/optimize/{id}` | — | job status and result / cancel (204) |
| `GET /api/data/status`, `POST /api/data/check`, `POST /api/data/refresh`, `GET /api/data/refresh/{id}/events`, `POST /api/data/activate` | — | data management (§3.7) |
| `GET /api/img/{name}` | — | proxied, disk-cached image (placeholder when offline) |
| `GET /*` | — | embedded frontend, SPA fallback to `index.html` |

```rust
pub struct EvaluateRequest {
    pub scenario: ScenarioDoc,
    pub ev: bool,                          // default true
    pub mc: Option<McOptions>,             // { n_runs, seed, series_bin }
    pub views: Option<Vec<LayerView>>,     // default: all views the enemy has
    pub parity: bool,                      // inspiration-style outputs
    pub trace: bool,
}
pub struct EvaluateResponse {
    pub data_version: DataVersion,
    pub warnings: Vec<Warning>,            // { key: MessageKey, args, severity } (unmodelled items, UNVERIFIED rules touched)
    pub weapon: ResolvedWeaponView,        // modded per-attack stats and per-type damage (the inspiration's weapon panel)
    pub enemy: EnemyView,                  // scaled health, shield, armor, overguard, DR, immunities, unique-DR notes
    pub ev: Option<EvReport>, pub mc: Option<McReport>,
    pub conditions: Vec<ConditionView>,    // every conditional effect in the build with class and current treatment
    pub touched_rows: Vec<RegisterRow>,    // D-xxx rows exercised (shown in an "assumptions" list)
    pub trace: Option<Trace>,
}
```

Types are shared with the frontend through `ts-rs` (`#[derive(TS)]` behind feature `ts` in `wf-model`,
`wf-engine`, `wf-optimizer`); `cargo xtask gen-ts` writes `frontend/src/api/generated/`, and CI fails if the output
is stale.

### 7.3 Execution model

- `AppState { data: ArcSwap<GameData>, engine: ArcSwap<Engine>, jobs: JobRegistry, cfg: Config,
  interactive: rayon::ThreadPool, optimizer: rayon::ThreadPool }`.
- `evaluate` runs in `spawn_blocking` on the interactive pool (default 2 threads plus MC parallelism) with a
  `CancelOnDrop` guard: when the browser aborts a superseded request, axum drops the handler future and the guard sets
  the cancel flag, so stale MC work stops.
- One optimizer job runs at a time by default (others queue); its pool has all cores; the interactive pool stays
  responsive because its work is short.
- Data activation swaps `GameData` and `Engine` atomically; in-flight requests finish on the old snapshot.

### 7.4 Static frontend, config, logging, security

- **Embedding**: `rust-embed` with `#[folder = "../../frontend/dist"]`; `build.rs` fails a release build with a clear
  message if `frontend/dist/index.html` is missing (`cargo xtask dist` runs `npm ci && npm run build` and then
  `cargo build --release`). Hashed assets get `Cache-Control: immutable`, `index.html` gets `no-cache`. In debug,
  files are read from disk, and Vite's dev server proxies `/api` to the backend.
- **Config**: `<config_dir>/warframe-dps/config.toml` (`directories` crate): `port`, `open_browser`,
  `threads.optimizer`, `threads.interactive`, `mc.default_runs`, `log.level`, `log.file`, `data.dir`,
  `images.fetch`. CLI flags override the file; `GET /api/meta` reports the effective config. The app makes no network
  request except image fetching (can be switched off) and user-started data checks and refreshes.
- **Logging**: `tracing` + `tracing-subscriber` (`EnvFilter`, `RUST_LOG` honoured), console plus a daily rolling
  file in `<data_dir>/logs` (7 kept); spans per request (method, path, status, latency) and per optimizer job (job id,
  phase, nodes); `tower-http` `CatchPanic` turns panics into a 500 with an error id that is logged.
- **Security**: loopback bind only; `Host` header must be `127.0.0.1:<port>`, `localhost:<port>` or `[::1]:<port>`
  (defeats DNS rebinding); state-changing requests must have no `Origin` or a same-origin `Origin`; no CORS; body
  limit 1 MB; the data refresh fetches only the pinned hosts (wiki, PE, GitHub raw).

---
## 8. Frontend

### 8.1 Framework and libraries

| Choice | Version | Why |
|---|---|---|
| React + TypeScript + Vite | react 19.3, typescript 7.0, vite 8.3 | largest ecosystem and the best-known stack for implementing agents; static build served by Axum |
| React Aria Components | 1.22 | the hard UI parts here are accessibility problems: weapon/enemy/mod comboboxes, a mod grid with **keyboard and screen-reader drag and drop**, dialogs, tooltips, number fields with locale formatting, sliders; React Aria implements the ARIA patterns (including keyboard DnD) instead of us |
| Zustand + Immer | zustand 5.1, immer 11.2 | one serializable `ScenarioDoc` store; Immer patches give undo/redo and minimal diffs for URL sync |
| TanStack Query | 5.108 | request deduplication, cancellation of superseded evaluations (`AbortController`), caching keyed by scenario hash and data version |
| react-intl (ICU MessageFormat) + `@formatjs/cli` | 12.2 / 6.23 | plural and number formatting per locale; message extraction and lint rules enforce "no literal strings" (P-10) |
| uPlot | 1.6 | small, fast time-series charts (DPS over time, status uptime, MC distribution); every chart has a table alternative |
| Tailwind CSS v4 with CSS custom-property design tokens | 4.3 | consistent spacing and colour tokens, themeable (dark and light), no runtime CSS-in-JS |
| fuse.js | 7.5 | fuzzy search over localized names and stat labels in pickers |
| Vitest, Playwright, axe-core | 5.1 / 1.65 / 4.14 | unit, end-to-end, accessibility tests (§9) |

Svelte 5 or Solid would give smaller bundles, but bundle size does not matter for a localhost app, and no comparable
library to React Aria exists for them; accessibility of drag-and-drop mod slots decides.

### 8.2 Layout

Desktop (≥ 1280 px), three persistent columns, no modal for the core workflow:

```text
┌ Top bar: app name · build name (editable) · Undo/Redo · Pin as baseline · Share · Import · Optimizer · Data vX ┐
├──────────── Loadout (left) ──────┬────────── Results (centre) ─────────────┬──── Scenario (right) ────────────┤
│ Weapon picker + variant parts    │ Headline: DPS (window, EV) + Δ baseline │ Enemy picker, level, SP, Eximus, │
│ (kitgun, zaw, progenitor, param) │ MC distribution strip (median, p5-p95)  │ party; scaled pools; immunities  │
│ Mod grid: 8 main + exilus +      │ Layer tabs: Armor | Health | Shield |   │ Options (schema-driven, grouped) │
│ arcane(s) + stance & combo       │ Overguard | Layered (TTK)               │ Buffs (schema-driven)            │
│ Incarnon track (5 tiers)         │ Attack cards (one per attack, groups)   │ Conditions: T-02 mode + per-     │
│ Riven builder (inline)           │ Charts: DPS over time · status uptime · │ effect overrides                 │
│ Warframe section (collapsible):  │ damage breakdown by type and source     │ Custom stats (dialog)            │
│ frame mods, aura, frame arcanes, │ Weapon stats panel (base → modded)      │ Simulation: window, runs, seed   │
│ shards (colour + tauforged)      │ Assumptions: touched register rows      │                                  │
└──────────────────────────────────┴─────────────────────────────────────────┴──────────────────────────────────┘
```

900-1280 px: two columns (Scenario becomes a tab next to Results). Below 900 px: one column with a bottom tab bar
(Loadout, Results, Scenario). The optimizer is its own route (`/optimizer`) with configuration on the left and a live
leaderboard, progress and log on the right; "Apply" and "Compare" bring a result back to the calculator.

### 8.3 Component tree

```text
src/
  main.tsx  App.tsx  routes.tsx                      # "/" calculator, "/optimizer", "/data", "/about"
  api/        client.ts  queries.ts  sse.ts  generated/*.ts (ts-rs)
  i18n/       en.json  messages.ts (typed MessageId union, generated)  I18nProvider.tsx  format.ts
  state/      scenarioStore.ts (Zustand+Immer)  history.ts (patch-based undo/redo)  urlSync.ts  baseline.ts  prefs.ts
  design/     tokens.css  damageTypes.ts (colour + glyph + i18n key per type)  icons/  primitives/ (Button, Card,
              StatRow, NumberField, Toggle, Segmented, Chip, Badge, InfoPopover, DataTable)
  features/
    loadout/   WeaponPicker  ModularParts  ProgenitorPicker  ItemParamEditor  ModGrid  ModSlot  ModTile
               ModSearchSheet (candidates, stat filters, set badges)  StanceSlot  StanceComboSelect
               IncarnonTrack  RivenBuilder  WarframeSection  ShardSlot
    scenario/  EnemyPicker  EnemyPanel  OptionsPanel (renders ScenarioSchema.toggles)  BuffsPanel
               ConditionsControl  CustomStatsDialog  AttackGroupsDialog  SimulationControls
    results/   ResultsView  HeadlineDps  McDistribution  LayerTabs  AttackResultCard  ParitySection
               DpsChart  StatusUptimeChart  BreakdownChart  WeaponStatsPanel  AssumptionsList
               TimelineDialog (virtualized per-hit cards + table view)  ExplainDrawer
    optimizer/ OptimizerPage  ObjectiveForm  ConstraintsForm  GroupPermissionsForm  BuffPermissionsForm
               ProhibitedList  ProgressPanel  LogView  Leaderboard  BuildDiff
    share/     ShareDialog  ImportDialog (with ImportReport)
    data/      DataPage (version, check, refresh with SSE progress, report, activate, rollback)  AboutPage (licences)
```

### 8.4 State management

- **One document.** `scenarioStore` holds `ScenarioDoc` (generated type) plus UI-only state (open panels, selection).
  Every user action is a named store action (`placeItem(slot, pick)`, `moveItem(from, to)`, `setToggle(id, value)`,
  ...). Immer patches feed undo/redo (`history.ts`, 200 steps) and URL sync.
- **Derived server state.** `useEvaluate(doc)` = TanStack Query keyed by `(dataVersion, stableHash(doc))`: EV request
  after 120 ms debounce (fast, deterministic), MC request after 400 ms (separate query, can be switched off);
  superseded requests are aborted. `useCandidates(doc, slot)` for the mod search. The catalog is fetched once per data
  version (`staleTime: Infinity`).
- **Baseline.** "Pin as baseline" stores a second `ScenarioDoc`; every number in results shows Δ % against it; the
  optimizer's "Compare" uses the same mechanism.
- **Persistence.** The current document lives in the URL hash (`#/b/wfd1.…`, debounced 500 ms through
  `/api/build/encode`) and in `localStorage` (last build, prohibited list, UI preferences, theme).

### 8.5 Feature parity with the inspiration's checklist

Every "yes" row of `I:feat` "Parity checklist", with the component that implements it and the end-to-end test file
that proves it (`frontend/e2e/parity/<file>.spec.ts`).

| Inspiration feature | Our implementation | E2E test |
|---|---|---|
| Weapon search (localized substring, keyboard navigation) | `WeaponPicker`: React Aria ComboBox, fuzzy search on localized names and categories, arrow keys and Enter | `weapon-search` |
| All attacks simulated and listed separately | `ResultsView` renders one `AttackResultCard` per attack; parent+child group sums | `attacks-listed` |
| Melee stance slot and combo selector | `StanceSlot` + `StanceComboSelect`; the combo is saved in the build code (inspiration does not) | `stance-combo` |
| Heavy / slam attacks with `isHeavy` auto-mode | attack roles; the heavy toggle (T-09) is disabled with an explanation for explicit heavy attacks | `heavy-auto` |
| Weapon info panel with live modded stats and per-type damage | `WeaponStatsPanel`: base → modded with Δ, per-type bars, per attack | `weapon-panel` |
| Incarnon evolutions and Incarnon-form attack | `IncarnonTrack` (tier steps, perk popovers); form attack card with its own magazine | `incarnon` |
| Kitgun grip and loader; zaw grip and link | `ModularParts` (chamber/strike fixed by weapon, part selects) | `kitgun`, `zaw` |
| Progenitor element and 25-60 % | `ProgenitorPicker` (7 elements, number field + slider, clamped) | `progenitor` |
| Chromatic Blade / Reactive Storm element choice | generic `ItemParamEditor` from `ParamSpec`; saved in the build code | `item-params` |
| Faction-weakness element mods | engine `target_weakness`; the mod tile shows the resolved element for the current enemy | `weakness-element` |
| 8 mod slots + exilus + arcane (+2 arch-gun arcanes) | `ModGrid` from `slot_layouts` in the schema | `slot-layout` |
| 4 warframe mods, aura, 2 warframe arcanes, 5 archon shards incl. Tauforged | `WarframeSection`, `ShardSlot` (colour, bonus, Tauforged switch) | `warframe-section` |
| Mod search by name or stat, compatibility, exclusivity, Semi filter, set bonuses | `ModSearchSheet` with `/api/candidates` (blocked items shown with the reason), stat filter chips, set badge with count | `mod-search` |
| Mod tile interactions (remove, swap by drag) and keyboard | React Aria DnD (mouse, touch, keyboard), context menu (Remove, Move to…, Rank), Delete key | `mod-tiles` |
| Riven builder (4 stats, per-class list, units, bow speed, faction multiplier) | `RivenBuilder` inline; units from `riven_stats`; legal-range hint from the riven formula | `riven` |
| Enemy search by name or faction | `EnemyPicker` grouped by faction | `enemy-search` |
| Level, Steel Path, Eximus, party size, innate DR, immunity, unique-DR warning | `EnemyPanel` | `enemy-panel` |
| Enemy scaling display (health, shield, armor, overguard, Eximus, clamp) | `EnemyPanel` from `EvaluateResponse.enemy` | `enemy-scaling` |
| Every "additional option" (19 controls incl. hidden `o.w`) | `OptionsPanel` generated from `rules/toggles.toml`; T-21 under "Advanced" | `options` (one case per toggle) |
| Custom stat values (26) | `CustomStatsDialog` generated from `custom_stats.toml` | `custom-stats` |
| Combine attack effects | `AttackGroupsDialog` (saved in the build code) | `combine-attacks` |
| Trigger all conditions | `ConditionsControl`: mode (all at max / none / simulated) + per-effect overrides | `conditions` |
| Queue length 1-60 s | `SimulationControls` window slider (advanced up to 600 s) | `window` |
| Random queue re-rolled on every change | MC seed "auto" re-rolls per change; parity section shows a single-run "random queue DPS" | `random-queue` |
| Average over many cycles with min / median / max | fixed `n_runs` with mean, median, min, max and percentiles | `averages` |
| Result panels per layer with expand | `LayerTabs` + focus mode | `layers` |
| Per attack: queue DPS, avg per bullet (direct, status, total), median per bullet, TTK | `ParitySection` in `AttackResultCard` | `parity-outputs` |
| Extended queue timeline | `TimelineDialog` (virtualized cards, reload markers, stacks; table view) | `timeline` |
| Tooltips (mod, arcane, shard info; attack damage types; layer explanations) | React Aria `Tooltip` / `InfoPopover`; item info = upstream text + rendered effects + curation status | `tooltips` |
| Build link encode/decode, copy button | `ShareDialog`, URL hash sync, copy code or URL | `share` |
| (extra) import warframe-damage.com links | `ImportDialog` + `ImportReport` | `import-inspiration` |
| Optimizer: metrics DPS / min TTK / max hit | `ObjectiveForm` (plus burst, sustained, weighted attacks) | `opt-metrics` |
| Optimizer: attack, layer, heavy + combo, search window, beam width, element exclusion | `ObjectiveForm`, `ConstraintsForm` | `opt-settings` |
| Optimizer build rules (headshot, no faction, no on-kill, conditions, keep installed, keep progenitor, Incarnon perks) | `ConstraintsForm`, per-slot locks in the mod grid | `opt-rules` |
| Optimizer warframe slots, strength, allowed buffs with values | `GroupPermissionsForm`, `BuffPermissionsForm` | `opt-groups` |
| Prohibited mods with persistence and defaults | `ProhibitedList` (search by name or stat, defaults from the inspiration preset, `localStorage`) | `opt-prohibited` |
| Progress, log, abort, apply result | `ProgressPanel`, `LogView` (i18n keys), Cancel, `Leaderboard` with Apply and Compare | `opt-run` |

Additions beyond parity: Explain drawer on every number, baseline comparison, EV and MC side by side, DPS-over-time,
status-uptime and breakdown charts, per-effect condition overrides, assumptions list (register rows touched),
curation badges ("unmodelled", "auto-updated, not reviewed"), data refresh page, undo/redo.

### 8.6 i18n

- All UI text is in `src/i18n/en.json` (ICU MessageFormat), ids namespaced by feature (`loadout.modGrid.empty`,
  `options.T-13.label`). A generated `MessageId` union type makes an unknown id a type error; ESLint
  `formatjs/no-literal-string-in-jsx` forbids literal text in JSX; `formatjs extract` in CI finds unused and missing
  ids.
- Server-produced text (warnings, optimizer log, legality reasons, import reports) is `MessageKey` + typed arguments.
  `cargo xtask gen-ts` also exports the list of `MessageKey` variants; CI fails if any has no entry in `en.json`.
- Game names (weapons, mods, enemies, perks) come from the bundle's i18n table keyed by uniqueName; WFCD provides 14
  more locales (`DS:wfcd` §7) that can be added as data later. Toggle, buff and custom-stat labels are i18n keys named
  in their schema rows.
- Numbers, percents and durations are formatted with `Intl` through react-intl; damage numbers use the spec's display
  rounding `floor(x + 0.5)` (U-5) in one `formatDamage` helper.
- Adding a language = adding `src/i18n/<locale>.json` and bundle names; no code change.

### 8.7 Build sharing

- **Our code**: `wfd1.` + base64url(raw deflate(canonical JSON of `ScenarioDoc`)) with items referenced by `ShortId`;
  a typical full build is 300-500 characters (estimate). Rust (`wf-buildcode`) is the only encoder and decoder; the
  UI calls `/api/build/encode` and `/decode`. Every saved field, including the ones the inspiration drops (stance
  combo, window, heavy start combo, light-combo option, combine groups, exalted element choice, party size; `I:feat`
  §15 and quirk 13), is in the document.
- **Versioning**: `ScenarioDoc.v`; `wf-buildcode::migrate` upgrades old versions; `tests/codes/v1/*.txt` keeps sample
  codes of every released version forever and must decode to the stored expected documents.
- **Inspiration import**: `?p=<LZString>&n=1` (and legacy base64 without `n`) → JSON with the keys of `I:feat` §15 →
  `ScenarioDoc`, through `aliases/inspiration.toml` (names, slot ids 0-29, Incarnon `{mId, pId}` → perk via
  `inspiration_index`, riven rows with their unit rules, `o.*` toggles, `p.*` custom stats). The `ImportReport` lists
  unmapped names, value conversions and the inspiration's never-touched defaults (`o.v.stucks = 0` and zero buff
  values, `I:feat` §15). Round-trip test on `research/test-cases/user-example-furis.json`.
- **Inspiration export** (for the oracle harness only): `ScenarioDoc` → inspiration URL, failing when the scenario uses
  something the inspiration cannot express.

### 8.8 Visual design direction

The inspiration is a dense jQuery page: everything in popups, no hierarchy between the headline number and secondary
values, no charts, numbers that jitter on every change. The target is a calm, dark **instrument panel**:

- **Hierarchy**: one headline number per view (window DPS, with the MC range as a thin strip under it), then attack
  cards, then detail. Numbers use tabular figures; Δ against the baseline in a muted colour with an arrow glyph.
- **Tokens**: graphite surfaces (`--surface-0 #0f1216` … `--surface-3`), one restrained accent (a warm "Orokin" gold
  for primary actions and the headline), semantic success/warning/danger; a light theme from the same tokens. Contrast
  of text tokens ≥ 4.5:1, checked by a unit test over `tokens.css`.
- **Damage types**: every type has a colour, a glyph (SVG sprite) and a short label; colour is never the only cue;
  the palette is checked for colour-vision deficiencies.
- **Mod tiles**: compact cards with polarity glyph, rank pips, drain, name and the key stat line rendered from the
  effect (not from DE text), plus curation badges. Empty slots show what they accept.
- **Motion**: short transitions on value changes only; disabled under `prefers-reduced-motion`.
- **Fonts**: self-hosted Inter (UI) with tabular numerals; no external font requests (offline app).

### 8.9 Accessibility

Target WCAG 2.2 AA. Keyboard paths for every action (including drag and drop, through React Aria's keyboard DnD and
equivalent menu actions); visible focus; dialogs trap and restore focus; comboboxes and grids use the ARIA patterns
React Aria implements; results announce a throttled summary in a polite live region ("Window DPS 34,381, plus
12 %"); every chart has a data-table view; targets at least 24 x 24 px; zoom to 200 % without horizontal scrolling of
the main panels; `axe` runs in every end-to-end test (§9.5).

---
## 9. Testing strategy

### 9.1 Engine

| Suite | What | Where |
|---|---|---|
| Spec vectors | every ```` ```testvector ```` block of `docs/mechanics/*.md` (stage and scenario level, §5.10) | `wf-engine/tests/spec_vectors.rs` |
| Coverage | one fixture per §4.6 row with a hand-computed expected value (python3-checked, as the spec does) | `wf-engine/tests/effects_coverage.rs` |
| Stage units | each stage function against its E-number, including NP-1/NP-2 `f32` emulation and quantization edge cases (`r` exactly on a half step, negative ratios) | module tests |
| Mechanics | each mechanic kind in EV and MC on a minimal scenario | `wf-engine/tests/mechanics.rs` |
| Properties (proptest 1.12) | P-5 slot-permutation bit identity; adding a monotone item never lowers direct damage; quantized ratios sum to within ½·n/32 of the unquantized sum; EV `dps_window` within the spec §4.4 tolerance of MC mean on random fixture builds; MC identical for 1 and 8 threads | `wf-engine/tests/properties.rs` |
| Compat | each `CompatFlags` flag changes only its own register row's output on a targeted fixture | `wf-engine/tests/compat.rs` |
| Lints | P-1 literal lint, P-2 item-name grep, `xtask spec-sync` (P-9) | CI |

### 9.2 Data pipeline

- Wire parsing on trimmed upstream fixtures committed under `crates/wf-datagen/tests/fixtures/` (small excerpts, with
  their licence notes), never the network (warframe-connector's rule, `DS:pe` Part 2).
- Lua sandbox: no `io`/`os`/`package`, instruction and memory limits trip on hostile modules, every cached module
  converts.
- Join: match counts reported and asserted against thresholds from the findings (WFCD → PE 668/668; wiki weapons →
  PE ≥ 825/872; wiki mods → PE ≥ 1,451/1,635), so a source change that breaks the join is caught.
- `assemble` equals all 1200 pre-expanded kitgun combos on damage and fire rate.
- Checks (§3.6) each have positive and negative tests; the committed `data/generated/` passes `wf-datagen check`.
- **Effect smoke tests**: for every curated item, evaluate its reference scenario (a weapon of the item's class from
  the bundle, fixed enemy) with and without the item; every declared stat must change in the declared direction on
  the attacks and hits in its scope, and not change outside the scope. This catches mis-scoped effects that would
  otherwise silently do nothing.
- Bundle: JSON → bundle → `GameData` round trip; `bench_load` under 50 ms; `diff` output snapshot (insta 1.49).

### 9.3 Golden tests against the inspiration oracle

`tools/oracle/` (Node + Playwright, per `I:oracle`) loads the local inspiration copy, seeds `Math.random` with
`page.addInitScript`, opens a URL produced by `wf-buildcode`'s inspiration export, and reads the result panels.
`cargo xtask oracle-record` runs it over `research/test-cases/*.json` and a generated scenario list and writes
`crates/wf-engine/tests/golden/oracle/<case>.json` (numbers only, with the inspiration bundle hash and date; committed;
recording is a manual step because `research/external/` is gitignored).

Three comparison classes:

1. **Deterministic panels, compat profile**: modded per-type damage, crit, status and fire rate of the weapon panel,
   and the enemy panel's scaled values must match to the inspiration's display precision.
2. **Statistical, compat profile**: `queue_dps_parity` median over ≥ 200 seeded inspiration runs versus our compat MC;
   pass when the medians agree within the bootstrap 99 % CI plus the spec 08 tolerance. Valid only when the trace's
   `touched_rows` contain no `—` rows (§5.7).
3. **Game profile, oracle-safe scenarios** (no touched rows at all): same comparison with the normal profile.

In-game measurements in `research/test-cases/ingame/*.toml` (highest precedence, spec §1.4) are scenario vectors
with exact observed numbers and their own tolerance; they override oracle expectations when both exist.

### 9.4 Optimizer

- **Brute-force equivalence**: synthetic pools (≤ 14 candidates, 4 main slots, exilus, arcane, 2 shards) where full
  enumeration is feasible; for every metric the top-10 set equals brute force exactly, and `exhaustive = true`.
- **Determinism**: the same job with 1, 2 and 8 threads, repeated 5 times, returns identical results.
- **Constraints** (proptest): locked slots unchanged, must-include present, prohibited absent, no incompatible pair,
  capacity and forma feasibility, at most one riven, element exclusions respected.
- **Cancellation** latency under 200 ms; time budget honoured; partial result returned.
- **Regression against WFOpt** (audit task): on about 15 weapons covering rifle, shotgun, pistol, bow, beam, melee and
  Incarnon, our best build scores at least WFOpt's best under our EV metric (scenarios from the inspiration preset).
- Benchmarks for nodes/s and scores/s (§5.9).

### 9.5 Frontend

- Vitest: store actions and undo/redo, URL sync, formatters (U-5 rounding), schema-driven renderers (a fixture schema
  with a new toggle renders without code changes), contrast test over `tokens.css`.
- Playwright: the parity suite of §8.5 (one spec per row) against `warframe-dps --no-browser --bundle
  data/fixtures/mini` with a fixed MC seed, so numbers are stable; keyboard-only flows (place, swap and remove a mod
  with the keyboard; build a riven; run and cancel the optimizer); `@axe-core/playwright` scan in every test;
  screenshots of key screens in dark and light at 375, 900 and 1440 px; a pseudo-locale run (`en-XA`) to detect
  hard-coded strings and truncation.

### 9.6 CI order

`cargo fmt --check` → `cargo clippy --workspace -- -D warnings` → `cargo xtask spec-sync spec-vectors gen-ts
lint-literals --check` → `wf-datagen check` → `cargo test --workspace` → `npm ci && npm run lint typecheck test
build` → `cargo build --release` → `npx playwright test`. Oracle recording and benchmarks run on demand.

---

## 10. Work breakdown

Tasks are sized for one agent each. Wave 0 freezes the interfaces; inside a wave, tasks run in parallel. Engine tasks
implement the topic documents 01-09; where a document is not final, they implement `00-overview.md` and the findings
and mark the code `SPEC-PENDING(0n)`, which `xtask spec-sync` lists.

### Wave 0 (one agent, first)

| ID | Task | Produces (interfaces everyone honours) | Acceptance |
|---|---|---|---|
| A1 | Contracts and skeleton | workspace and CI; `wf-model` types (ids, enums, `Weapon`, `Attack`, `Enemy`, `Item`, `Build`, `Scenario`, `ScenarioDoc` v1, `Toggles` T-01..T-29, `CompatFlags`); `wf-effects` registry table and IR types and TOML authoring schema (parse only); `wf-data` bundle header/body types; API request/response and `OptEvent` types; `MessageKey` enum; **mini fixture bundle** `data/fixtures/mini/` (about 12 weapons: Braton Prime, Furis + Incarnon line, a Kuva weapon, Paris, a beam, Arca Plasmor, a kitgun's parts, Nikana Prime with a stance, an exalted weapon; about 60 items covering every §4.6 row; 6 enemies incl. Lancer, a Corpus shielded unit, an Eximus) | `cargo build/test --workspace` green; fixture bundle loads; `xtask gen-ts` produces TS; rustdoc on every public type; a `CONTRACTS.md` change log started |

### Wave 1 (parallel after A1)

| ID | Task | Depends | Must honour | Acceptance |
|---|---|---|---|---|
| B1 | `wf-effects`: lowering, sugar, rank curves, variants, params, text binding, renderer, validator, class derivation, JSON Schema | A1 | registry and IR of A1 | every IR construct unit-tested; all fixture items compile; renderer output matches WFCD-style text on 30 sample mods |
| B2 | `wf-datagen` sources: `PoliteClient`, cache, `sources.lock.toml`, mlua sandbox, wire structs for wiki, PE, WFCD | A1 | cache layout §3.2 | all cached modules parse offline; sandbox limit tests; no network in tests |
| B3 | Engine weapon side ST-1..ST-7 (`composition`, `crit_status`, CO term and factor groups of `conditional`), `Prepared`, traces | A1 | `Contribution`/`Affine` contract; stage registry names | spec vectors E-1..E-11 and §3.8 steps 2-4; P-5 proptest; coverage rows for weapon-side stats |
| B4 | Engine enemy side (`enemy`): scaling (NP-2), Steel Path, Eximus, overguard, armor E-15, layers, routing, overflow, attenuation, special DR; compat flags D-042..D-055 | A1 | `rules/scaling.toml`, `rules/special_dr.toml` schemas | §3.8 steps 1 and 5; one test per D-row flag |
| B5 | Schedules (`ranged_schedule`, `melee_schedule`) | A1 | `Schedule`/`CycleStats` contract | unit tests per fire model (auto, semi cap, burst, charge, bow, beam ramp, spool, battery, per-round reload) and melee (stance combos, combo counter, heavy, slam) |
| B6 | Frontend foundation: Vite app, tokens, primitives, i18n infrastructure and lint, API client from generated types, store, history, URL sync, layout shell, MSW mocks from fixtures | A1 | generated TS types, `MessageKey` list | lint, typecheck and Vitest green; responsive shell; axe clean |
| B7 | `wf-buildcode`: `wfd1` codec, migrations scaffold, inspiration import and export | A1 | `ScenarioDoc` v1, `aliases/inspiration.toml` schema | round trips; Furis example imports to the expected document; export → import identity on supported scenarios |
| B8 | `wf-server` skeleton: all routes on fixture data, SSE plumbing, embedding, config, logging, single instance, browser opening, Host/Origin checks | A1 | API types | integration tests per route; security tests; `--no-browser` mode for e2e |

### Wave 2

| ID | Task | Depends | Must honour | Acceptance |
|---|---|---|---|---|
| C1 | Status and DoT (`status`, E-13), extra hits (E-14), all mechanic kinds (4.4.6) | B3 | mechanic kinds contract | spec 03 vectors; §3.8 steps 6-7; mechanics tests |
| C2 | EV evaluator (`eval_ev`, `outputs`) | B3, B4, B5, C1 | `EvReport` | §3.8 steps 8-10; spec 08 vectors; every §4.5 output mapped |
| C3 | MC simulator (`eval_mc`), Philox, timeline, parity outputs | B3, B4, B5, C1 | `McReport`, `Timeline` | Philox known-answer tests; thread determinism; EV-MC consistency suite within tolerance |
| C4 | Pipeline join, attack roles, modular parts, curate, checks, emit, diff, `wf-data` build.rs and loader, legality (spec 09) | B1, B2 | bundle format §3.7, precedence table | `wf-datagen build` reproduces `data/generated/` offline; checks green; join thresholds met; load < 50 ms |
| C5 | Curation: primary mods (rifle, shotgun, bow, sniper) incl. CO catalogue rows for primaries | B1 (C4 for checks on real data) | authoring schema, factor groups | batch `check` clean; smoke tests pass; every item verified or justified `unmodelled` |
| C6 | Curation: secondary mods incl. CO rows | same | same | same |
| C7 | Curation: melee mods, stances, stance CO ratios, melee CO rows | same | same | same |
| C8 | Curation: weapon and warframe arcanes, set bonuses | same | same | same |
| C9 | Curation: Incarnon lines (perks per tier, variants, per-attack scopes, inspiration indices) | same | `IncarnonLine` schema | same |
| C10 | Curation: rules files (status, factions, scaling, factor groups, slot layouts, toggles, riven stats, body parts, special DR), shards, auras, frame mods, buffs, custom stats, weapon/attack overrides | same | rules schemas | same; `xtask spec-sync` green |
| C11 | Frontend loadout features | B6 (B8 for live API) | store actions, schema | parity rows weapon/mods/riven/variants/warframe pass on fixtures |
| C12 | Frontend scenario and results features, Explain drawer, charts, timeline | B6 | `EvaluateResponse` | parity rows enemy/options/results/timeline pass |

### Wave 3

| ID | Task | Depends | Acceptance |
|---|---|---|---|
| D1 | Optimizer core: lane compilation, classification, dominance, relaxation bounds, branch-and-bound, beam fallback, realizer, local search | C2 | brute-force equivalence; ≥ 5 M nodes/s/core target reported |
| D2 | Optimizer jobs: parallel tasks, top-K, finalists with MC and CRN, events, cancellation, presets, capacity, riven explore | D1, C3, B8 | determinism across threads; constraint proptests; cancel < 200 ms |
| D3 | Frontend optimizer UI | B6, D2 (mock events earlier) | parity rows `opt-*` pass |
| D4 | In-app data refresh (CLI and UI), bundle activation and rollback | C4, B8 | refresh against cached fixture "new version" produces the expected report; rollback works |
| D5 | Oracle harness, recording, golden tests | B7, C2, C3 | Furis test case and ≥ 10 recorded cases compared; classes 1-3 of §9.3 implemented |
| D6 | Playwright parity, keyboard, a11y, visual and pseudo-locale suites | C11, C12, D3 | every §8.5 row has a passing spec |

### Wave 4

| ID | Task | Depends | Acceptance |
|---|---|---|---|
| E1 | Accuracy audit against the inspiration and in-game cases; register rows confirmed or reopened | D5 | audit report in `research/findings/`, all golden tests green or explained |
| E2 | Performance pass against §5.9 targets | C2, C3, D1 | benchmark table published |
| E3 | Promote this architecture (as merged) to `docs/architecture/` and the pipeline docs to `docs/data/` | all | reviewed docs |

Critical path: A1 → B3 → C1 → C2 → D1 → D2 → D3 → D6. Curation (C5-C10) runs alongside waves 2-3 and is the largest
effort; the fixture bundle lets every other task proceed before it finishes.

---

## 11. Risks, costs and open questions

### 11.1 Costs of this design and how they are contained

| Cost or risk | Containment |
|---|---|
| Curating about 900 entries (≈ 600 weapon mods, 170 arcanes, 53 Incarnon lines, shards, auras, frame mods, buffs) | `curate new` drafts about 70 % of weapon mods from simple text lines (`DS:wfcd` §3.4); six parallel batches; number binding makes checks mechanical; `unmodelled` is visible, never silent |
| The IR may not express a future mechanic | the "new kind" path (registry row or mechanic kind, reviewed against the spec) is small; the authoring schema is versioned |
| EV approximations under `simulated` conditions (independence of stacks, gate splitting limit) | warnings in the response; MC is available for every scenario; spec §4.4 tolerance tests |
| Weak bounds make branch-and-bound slow on some weapons | anytime search with time budget; root-bound task ordering; beam fallback flagged `exhaustive = false` |
| A JSON → postcard step in `build.rs` adds compile time (about 1-2 s, estimate) | only `wf-data` rebuilds when data changes |
| mlua and a C compiler in the app build (feature `refresh`) | vendored Lua needs only `cc`; the feature can be switched off for a build without in-app refresh |

### 11.2 Why correctness-first does not cost speed or UX

- The data-driven IR is compiled into lanes at `prepare` time; the evaluation kernels see dense arrays, so the
  optimizer's speed targets are those of a hand-coded engine.
- A hand-coded key-per-mechanic engine reproduces the inspiration's failure modes: a riven stat stored under the wrong
  key and silently ignored (`I:model` Q4), Hunter Munitions found by name with a hard-coded 30 % (Q15), `WITH_COND`
  replace semantics (§1.3), labels swapped against meanings (`I:feat` quirk 16). With one registry, every source
  (mod, riven, buff, custom stat) reaches a bucket through the same validated path, and item-specific code is banned.
- After a game patch, value changes are data edits or auto-patched drift, visible as reviewable diffs; new items appear
  as `unmodelled` instead of silently contributing nothing.
- If a competing proposal has a faster kernel or richer UX flows, they fit behind this design's seams: `LaneModel`
  (the optimizer's scoring interface) and the schema-driven UI panels.

### 11.3 Open questions

| ID | Question | Proposed default |
|---|---|---|
| Q1 | May `data/generated/` (wiki-derived, CC BY-NC-SA 3.0) be committed to the repository? | yes, with `ATTRIBUTION.md`; otherwise require `wf-datagen build` before `cargo build` |
| Q2 | Redistribution of DE Public Export derived fields has no stated licence (`DS:pe` open question 1) | ship as derived data with the "© Digital Extremes" notice, like WFCD does |
| Q3 | May the app fetch item images from the WFCD CDN on use? | yes, switchable off (`images.fetch`) |
| Q4 | Should resolved-stat conditions (Prelude of Might's "CC below 40 %") be under T-02 or always evaluated? (for spec 04) | always evaluated, as a formula |
| Q5 | Default optimizer metric: `dps_window` (spec §4.1 headline) or `dps_sustained` | `dps_window` |
| Q6 | Should `draft` (unreviewed) curated items be offered by the optimizer? | yes, marked; `unmodelled` never |
| Q7 | Is an exilus mod's element order "after slot 8" (OQ-024)? | follow the spec decision; it is one row in `rules/slot_layout.toml` |
