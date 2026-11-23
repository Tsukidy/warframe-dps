Status: draft (proposal)

# Architecture proposal A: optimizer performance first

Author's priority: the expected-value (EV) evaluator must score very large numbers of candidate builds per second on
every core, without giving up any accuracy the spec (`docs/mechanics/00-overview.md`, "the spec") demands. Every
layer of this proposal (data bundle, effect encoding, engine pipeline, optimizer) is shaped so that a build can be
*compiled once into a flat stat vector*, *changed incrementally*, and *bounded from above cheaply*. A competing
proposal with a different priority exists; section 10 lists the points a judge should weigh.

Inputs read: `CLAUDE.md`; `docs/mechanics/00-overview.md`; the topic documents that appeared while this was written
(`02`, `03`, `04`, `05`, `06`; read for their data shapes: the condition model and effect state machine of 04 §4,
the ingestable CO catalogue of 04 §2.14, the stat-line bucket table of 04 §5.1, the schedule generator of 06 §6, the
hand-authored tables of 06 §4.3 and 05 §4.5, the test-vector form of 02 §9); `research/findings/`
`inspiration-feature-inventory.md` (parity checklist), `inspiration-data-model.md`, `inspiration-optimizer.md`,
`inspiration-dps-aggregation.md`, `data-source-wfcd.md`, `data-source-wiki-modules.md`,
`data-source-public-export-and-connector.md`, `wiki-additive-vs-multiplicative-edge-cases.md`,
`wiki-mods-arcanes-rivens.md`, `wiki-warframe-side-buffs.md`, `oracle-feasibility.md`;
`research/test-cases/user-example-furis.json`.

Spec references use the spec's own identifiers (E-n, ST-n, CD-nn, D-nnn, T-nn, OQ-nnn). Short keys `F:*`, `I:*`, `DS:*`
are those of spec section 0.2.

---

## TL;DR (key decisions)

1. **Seven Rust crates** in one workspace: `wfd-model` (types and the effect DSL; the contract every task codes
   against), `wfd-data` (bundle loading), `wfd-engine` (IO-free calculation), `wfd-opt` (search), `wfd-buildcode`
   (share strings and inspiration-link import), `wfd-datagen` (pipeline), `wfd-server` (the `warframe-dps` binary).
   Frontend in `frontend/` (React 19 + TypeScript + Vite), embedded into the binary with `rust-embed`.
2. **Data**: `wfd-datagen` fetches wiki Lua modules (evaluated with `mlua`, vendored Lua 5.1), WFCD and the Public
   Export, joins on `uniqueName`, applies hand-curated **TOML effect tables**, validates them against the wiki's
   own description text, and writes a reviewable JSON snapshot (`data/generated/`) plus a compact
   `postcard`+zstd bundle (about 0.5 MB) that `wfd-data` embeds. `warframe-dps data update` rebuilds it locally after a
   game update.
3. **One effect DSL for every source** (mods, arcanes, shards, Incarnon perks, set bonuses, buffs, custom stats,
   riven traits, weapon innates). An effect is `{target bucket, value expression, condition, scope}`. Conditions carry
   the spec's T-02 class, so the compiler turns each effect into either a constant or a coefficient on a small set of
   **state variables** (distinct statuses, Cold stacks, combo tier, stacks of a trigger, first/last shot).
4. **Builds compile into a flat fixed-point vector** (`StatVec = [i64; N]`, nano-units). Sums are exact and
   order-independent, so incremental updates (`+ contrib(new) − contrib(old)`) are bit-identical to a full recompute.
   Faction, weapon class, attack applicability and T-02 mode are resolved at compile time per scenario.
5. **The engine has four EV fidelities sharing one kernel**: `Steady` (dps_burst/sustained), `WindowExact` (the
   spec's shot-by-shot walk; what the UI shows and what the optimizer's final ranking uses), `WindowFast` (segment
   quadrature for long windows, verified against Exact), and `Bound` (a provable upper bound of WindowExact, used to
   prune). A throwaway prototype measured 0.49 µs per build for compile + resolve + steady EV and about 0.14 µs per
   target-state point (section 0.3).
6. **The optimizer is exact where it can be and says so.** Composition-changing mods (elements, physical) and
   progenitor element are enumerated; everything else in the 8 main slots is solved by a parallel **branch-and-bound**
   whose bound is valid because the scored metric is monotone in those buckets (declared per stat, checked per
   scenario, property-tested). Small groups (arcane, exilus, stance and combo, Incarnon tiers, frame arcanes, aura,
   shards as multisets, kitgun/zaw parts) are handled by block-coordinate ascent with warm-started B&B, then a
   cross-group local search. Finalists are re-scored with `WindowExact` and confirmed by Monte Carlo with common
   random numbers. The result reports an **optimality gap**, a best build **per damage mix**, leave-one-out values,
   and the EV-versus-MC deviation.
7. **Server**: Axum 0.8 on `127.0.0.1`, REST + **server-sent events** for optimizer progress, `DELETE` for cancel,
   Host/Origin checks against DNS rebinding, `webbrowser` to open the page, `tracing` logs, `clap` CLI with headless
   `calc` and `optimize` subcommands (used by tests and by agents).
8. **Frontend**: three-pane calculator with a formula inspector (every bucket, every contributing item), charts the
   inspiration lacks (DPS over time with MC bands, damage breakdown, status uptime, crit tiers), a virtualized hit
   timeline, and a full optimizer page. Every string goes through `i18next` keys, enforced by a lint rule; the
   server sends message codes, never English text.
9. **Tests**: every spec test vector (`TV-nn-mm`) gets a machine-readable twin that CI checks number by number against
   the Markdown, so the spec stays the single source of expected numbers; plus pipeline validation, oracle golden tests, optimizer exactness
   and determinism tests, bound-validity property tests, Playwright end-to-end tests mapped one to one onto the
   parity checklist, and benchmark thresholds.

---

## 0. Principles and performance budget

### 0.1 Principles

| # | Principle | Consequence |
|---|---|---|
| P1 | Separate what is fixed per scenario, per item, per build and per time point | Four compile levels (section 4.2); each level is computed once and reused by every evaluation below it. |
| P2 | Everything additive is a vector add | All bonus buckets, factor groups (each `G` and `G_F` group is an additive sum inside one factor), extra-hit fractions, proc-on-event chances and lock flags live in one `StatVec`. Non-additive structure is limited to element order, set-piece counts and shard counts, which have dedicated, cheap finalizers. |
| P3 | Exact arithmetic for sums | Fixed-point `i64` nano-units make sums associative. A local-search delta, a branch-and-bound prefix sum and a from-scratch compile give the same bits, so caching and determinism are free. |
| P4 | One kernel, several fidelities | Steady, window, bound and TTK are the same per-point target-state kernel called at different time points. Accuracy fixes land once. |
| P5 | Approximations may prune or screen, never decide | The optimizer's final ranking always uses `WindowExact` (the spec's EV definition) and MC confirmation. Fast paths are either provable bounds or carry a tested error margin that widens the kept set. |
| P6 | No IO, no allocation in the hot path | `wfd-engine` takes a `&Bundle` and plain structs; per-thread `Workspace` holds scratch arrays; `SmallVec`/`ArrayVec` for short lists. |
| P7 | Determinism by construction | EV is single-threaded per build; parallelism is across builds; MC uses a counter-based RNG keyed per U-6; all reductions are in a fixed order; transcendental functions come from the `libm` crate so results do not depend on the platform's C library. |

### 0.2 Cost model of one evaluation

| Level | Computed | When | Typical cost |
|---|---|---|---|
| L0 scenario | scaled enemy pools, armor, faction modifier per type, caps, immunities, body-part data, Poisson helper constants, condition-mode resolution | once per `Scenario` | microseconds; irrelevant |
| L1 item | each candidate item's contribution vector per objective attack, element entries, coefficients on state variables, flags, drain, incompatibility mask | once per (item, scenario); about 1,000 items | tens of microseconds total |
| L2 build | `StatVec` sum (8-20 sparse contributions), set and shard finalizers, element combination | per candidate; incremental in search | 50-250 ns |
| L3 attack | `ResolvedAttack`: base vector, composition, 1/32 quantization, Y terms, F, G, G_F, crit, status, proc weights, multishot, firing parameters, duty | per candidate per objective attack | about 100-200 ns |
| L4 evaluation | target-state kernel at one or more time points, DoT landing, extra hits | per candidate | 0.14 µs per point |

### 0.3 Prototype measurement (throwaway, not part of the repository)

To check that the targets below are realistic, a 150-line Rust prototype (scratchpad only, not committed) reproduced the operation
count of the spec's pipeline: a 96-slot fixed-point vector summed from 10 random sparse contributions out of a pool
of 60, resolve with 18 damage types and 1/32 quantization, an f32 crit-multiplier quantization (NP-1), per-status
Poisson expectations truncated at the caps for Viral, Corrosive (joint with Heat presence, 22 armor evaluations with
`sqrt`), Cold and Puncture, the expected distinct-status count over 15 statuses, an Adding-CO term, and five DoT
terms. Release build, LTO, one core of an Intel Xeon at 2.1 GHz (the sandbox machine):

| Operation | Measured |
|---|---|
| compile (sparse fixed-point sum) + resolve | 246 ns per build |
| + steady-state EV (one target-state point) | 486 ns per build, about 2.0 M builds/s/core |
| window EV with 16 target-state points | 2.9 µs |
| window EV with 48 points | 7.1 µs |
| window EV evaluated at all 300 shots of a 20 s Braton Prime window | 42 µs |

Conclusion: the target-state point (about 140 ns) dominates, so the number of points per evaluation is the lever.
Hence `Bound` and `WindowFast` use few points, and `WindowExact` is reserved for the UI and for finalists.

### 0.4 Performance targets (acceptance criteria for the engine and optimizer tasks)

Per core of a 2.1 GHz-class x86-64 machine, release profile (`lto = "fat"`, `codegen-units = 1`), one objective
attack, one layer view:

| Operation | Target | Use |
|---|---|---|
| L2+L3 compile and resolve | ≤ 300 ns | everything |
| `Fidelity::Steady` | ≤ 1 µs (≥ 1 M builds/s/core) | dps_burst, dps_sustained, hit metrics |
| `Fidelity::Bound` (8 intervals) | ≤ 1.5 µs | branch-and-bound pruning |
| `Fidelity::WindowFast` | ≤ 10 µs (≥ 100 k builds/s/core) | screening of B&B leaves |
| `Fidelity::WindowExact` | ≤ 0.2 µs per shot or tick event | UI numbers, finalists |
| EV TTK (layered walk) | ≤ 2× WindowExact | TTK metric finalists |
| Monte Carlo | ≥ 5 M events/s/core; 1,000 runs of a 20 s Braton Prime window ≤ 100 ms on 8 cores | UI distributions, confirmation |
| Optimizer, rifle, conditions `all_max`, 8 main slots + exilus + arcane | proven optimum of the main slots for each visited context; whole run ≤ 60 s on 8 cores | default run |
| Cancel latency | ≤ 100 ms | abort button |

---

## 1. Cargo workspace layout

### 1.1 Repository tree

```text
warframe-dps/
├── Cargo.toml                    # [workspace], shared [workspace.dependencies], release profile
├── rust-toolchain.toml           # channel = "stable"
├── crates/
│   ├── wfd-model/                # types, enums, effect DSL, Build, Scenario, Bundle schema (no logic beyond validation)
│   ├── wfd-data/                 # bundle (de)serialization, embedded default bundle, indexes, name search
│   ├── wfd-engine/               # compile, resolve, schedules, EV, MC, outputs (IO-free)
│   ├── wfd-opt/                  # optimizer (depends on engine, rayon)
│   ├── wfd-buildcode/            # share-string codec, inspiration link import
│   ├── wfd-datagen/              # data pipeline library + `wfd-datagen` CLI
│   └── wfd-server/               # Axum server, CLI; binary name `warframe-dps`
├── xtask/                        # `cargo xtask {dist, gen-ts, gen-schema, oracle, bench-check}`
├── frontend/                     # React + TS + Vite app; `npm run build` → frontend/dist
├── data/
│   ├── curated/                  # hand-authored TOML effect tables (committed; section 2.4)
│   ├── generated/                # pipeline output as sorted, pretty JSON per category (committed, reviewable diffs)
│   └── NOTICE.md                 # attribution and licences of the data
├── tests/
│   ├── golden/                   # oracle scenarios and recorded outputs (JSON)
│   └── fixtures/mini-bundle.json # tiny hand-made bundle (Braton Prime, Lancer, Serration, ...) for engine tests
├── tools/oracle/                 # Node + Playwright harness running the inspiration offline (I:oracle)
└── docs/ research/               # unchanged
```

### 1.2 Crates

| Crate | Responsibility | Depends on (workspace) | Main external dependencies |
|---|---|---|---|
| `wfd-model` | Enumerations in the spec's order (`DamageType`, `StatusType`, `Faction`, `Layer`, `View`), the stat layout macro, effect DSL types (serde for TOML), item/weapon/enemy/bundle schema, `Build`, `Scenario`, toggles T-01..T-29, ids. Feature `ts` derives TypeScript types (`ts-rs`); feature `schema` derives JSON Schema (`schemars`) for curated TOML. | — | `serde`, `smallvec`, `arrayvec`, `bitflags`; optional `ts-rs`, `schemars` |
| `wfd-data` | `Bundle::load(bytes)`, embedded default bundle (`include_bytes!`), override bundle from the user data directory, dense `ItemId` ↔ `uniqueName` ↔ `StableKey` indexes, compatibility indexes (slot kind × weapon tag → items), prefix/substring name search, `ArcSwap<Bundle>` holder for hot reload. | model | `postcard`, `ruzstd` (pure-Rust decoder), `arc-swap` |
| `wfd-engine` | Everything in the spec: compile, element combination, kitgun/zaw/progenitor assembly, ST-1..ST-10, enemy model, schedules, EV (four fidelities), MC, TTK, outputs, compatibility profile. No IO, no threads except an optional `rayon` feature for parallel MC runs. | model | `smallvec`, `arrayvec`, `libm`; optional `rayon` |
| `wfd-opt` | Problem compilation, pools and dominance, composition enumeration, parallel B&B, context search, local search, capacity/polarity, MC confirmation, reports, progress and cancellation. | model, engine | `rayon`, `parking_lot`, `rustc-hash`, `xxhash-rust` |
| `wfd-buildcode` | `wfd1-…` share strings (postcard + base64url + CRC-16), inspiration link import (`lz-str` decode, key map of `I:feat` §15, name aliases). | model, data | `postcard`, `base64`, `crc`, `lz-str`, `serde_json` |
| `wfd-datagen` | Fetch/cache (polite client), Lua evaluation, typed wiki structs, WFCD/PE parsing, join, normalization, curated tables, validators, scaffolding, snapshot and bundle writers. Library used by the CLI and (feature `datagen`) by the server's `data update`. | model | `mlua` (`lua51`, `vendored`, `error-send`), `ureq`, `lzma-rs`, `sha2`, `serde_json` (`preserve_order`), `toml`, `zstd`, `regex` |
| `wfd-server` | Axum routes, SSE, static embedding, browser opening, config, logging, CLI (`serve`, `calc`, `optimize`, `data update/status/verify`). | all | `axum` 0.8, `tokio`, `tower-http`, `rust-embed`, `tracing`, `tracing-subscriber`, `tracing-appender`, `clap`, `directories`, `webbrowser`, `uuid` |

Dependency direction is strictly `model ← data ← engine ← opt ← server`, with `buildcode` and `datagen` beside
them. `wfd-engine` does not depend on `wfd-data`: it takes `&Bundle` (a `wfd-model` type), so engine tests run on
`tests/fixtures/mini-bundle.json` without the pipeline.

### 1.3 Frontend location and build

`frontend/` is an npm project (Vite). `cargo xtask dist` runs `npm ci && npm run build` and then
`cargo build --release -p wfd-server`. `wfd-server/build.rs` checks for `frontend/dist/index.html`; if it is missing
(a pure Rust developer build), it embeds a one-page placeholder that says how to build the frontend, so
`cargo build`/`cargo test` never need Node. In development, Vite's dev server proxies `/api` to the Rust server.
`cargo xtask gen-ts` writes the generated API types to `frontend/src/api/generated/` (checked in; CI fails when they
are stale).

### 1.4 Release profile

```toml
[profile.release]
lto = "fat"
codegen-units = 1
panic = "abort"          # the server catches nothing across threads that needs unwinding; optimizer workers report via channels
debug = "line-tables-only"

[profile.bench]
inherits = "release"
panic = "unwind"
```

No `target-cpu=native` (the binary must run on any x86-64/arm64 machine); the hot loops are written so that LLVM
auto-vectorizes the fixed-width `i64`/`f64` array operations at the baseline ISA.

---

## 2. Data pipeline

### 2.1 Sources and their roles

| Source | Used for | Not used for | Licence handling |
|---|---|---|---|
| wiki Lua modules (`DS:wiki`) | weapon attacks (1,984 records: damage per pellet, crit, status, fire rate, multishot, falloff, forced procs, charge, burst, spool, Incarnon forms, ammo cost), enemies (1,019, incl. body-part prose, proc resists, base level), `DamageTypes/data` faction table, `Stances/data`, `Modular/data` (kitgun/zaw parts), `Mods/data` and `Arcane/data` identity (polarity, drain, max rank, `Incompatible`, `Type`, description text, `UpgradeTypes`) | effect values (prose only) | CC BY-NC-SA 3.0: attribution in `data/NOTICE.md`, in the bundle manifest and on the About page; derived bundle is share-alike and non-commercial |
| WFCD warframe-items (`DS:wfcd`), pinned tag | catalogue cross-check, `imageName`, English names and (later) i18n names keyed by `uniqueName`, per-rank `levelStats` text (used only to verify rank scaling of curated effects) | attacks (lossy), enemies (stale), effects | MIT; keep copyright notice |
| DE Public Export (`DS:pe`) | canonical `uniqueName` keys, freshness (`buildLabel`, manifest hashes), top-level stat cross-check, riven `upgradeEntries` base values, mod drain/rarity cross-check | attacks, enemies, effects | no licence stated (`DS:pe` open question): used for identifiers, freshness and cross-checks; flagged for the user's decision before any public release |
| Spec tables (`docs/mechanics/`) | ingestable tables owned by the spec: CO catalogue per attack (04 §2.14), status summary (03 §4.1), faction modifiers (05 §5.1) | — | project content |
| Curated tables (`data/curated/`, this project) | stat-line patterns (04 §5.1) and every effect that is not a simple stat line: conditional mods, arcanes, shards, Incarnon perks, sets, buffs, custom stats, riven stat mapping, attack overrides (06 §4.3: spool cadence, beam ramp start, semi-auto caps, per-shell reloads …), enemy overrides (05 §4.5: special DR, attenuation, Toxin-blocking shields, head-crit capability) | — | project content; each entry cites its source (CLAUDE.md) |

### 2.2 Stages and commands

```text
wfd-datagen fetch   [--refresh]        # 1. network → cache (only stage that touches the network)
wfd-datagen build   [--offline]        # 2-7. cache + data/curated → data/generated/*.json + bundle .bin + reports
wfd-datagen curate todo|scaffold <id>|check
wfd-datagen diff <old-manifest> <new>  # what changed between two data versions
```

| Stage | Module (`wfd-datagen/src/…`) | What it does |
|---|---|---|
| 1 fetch | `fetch/{client.rs, wiki.rs, wfcd.rs, pe.rs}` | `PoliteClient` (one request per host per second, single-flight, `Retry-After`/429/503 back-off, descriptive generic User-Agent, `maxlag=5` on MediaWiki calls; pattern from `DS:pe` part 2). Wiki: one batched `prop=revisions&rvslots=main` call per ≤ 50 modules; module list in `data/sources.toml`; each module cached as `<cache>/wiki/<Module>@<revid>.lua`, unchanged `revid` → no download. WFCD: raw files at the tag pinned in `data/sources.toml`, verified by SHA-256. PE: LZMA index (`lzma-rs`), manifests cached as `<cache>/pe/<file>!<hash>`. Cache dir: `--cache-dir`, default `.cache/wfd-sources/` (gitignored). |
| 2 convert | `lua/{sandbox.rs, to_json.rs}`, `wiki/{weapons.rs, enemies.rs, mods.rs, arcanes.rs, damage_types.rs, stances.rs, modular.rs}` | `mlua` Lua 5.1 sandbox with only `math`, `string`, `table`, stub `mw`/`require` (proven by the PoC in `research/external/cache/poc/lua2json/`). Lua tables → `serde_json::Value` (1..n keys → array; empty table → empty, resolved by the typed struct), `math.huge` → `f64::INFINITY`. Then typed `#[serde(default)]` structs with field aliases for the known typos (`Hitscan`, `IsSlilent`, `Ammocost`, `ForcedProc`, `EximusSHield`, `DS:wiki` §11). |
| 3 join | `join.rs` | One `Catalog` keyed by `uniqueName` (never by name, `DS:pe` §1.8). Weapons: wiki record (attacks) + PE record (presence, top-level stats for cross-check) + WFCD (image, categories). Mods/arcanes: wiki identity + PE drain/rarity + WFCD `levelStats`. Enemies: wiki only (`DS:wfcd` §6: WFCD enemies are stale). Unmatched records are listed in the report with a reason class (beast weapons, innate upgrades, Antique arcanes …). |
| 4 derive | `derive/{attacks.rs, triggers.rs, compat.rs, stable_keys.rs}` | `AttackKind` from `AttackName` + `ShotType` + melee fields (normal, charged, alt-fire, radial child, slam, heavy slam, heavy, slide, thrown, Incarnon form …); parent/child links (with an override table for the inconsistent `AttackParentIndex`, `DS:wiki` §11); attack groups for "combine attack effects"; `Trigger` normalization of the 33 strings; mod compatibility (wiki `Type` + `Mods/itemCompatibility/data` + compat tags); append-only `StableKey` registry for share strings (`data/generated/stable_keys.json`, ids never reused). |
| 5 effects | `effects/{stat_lines.rs, spec_tables.rs}`, `curated/{load.rs, resolve.rs}` | Tier 1 stat-line import, tier 2 spec-table ingestion (Markdown table parser for the tables marked ingestable), tier 3 curated TOML; resolve each entry's `id` (uniqueName) against the catalogue; attach effects (curated beats auto); merge the CO catalogue and attack overrides into attacks; merge enemy overrides. |
| 6 validate | `validate/{schema.rs, refs.rs, numbers.rs, ranks.rs, stats.rs, coverage.rs}` | Section 2.5. Errors stop the build; warnings go to the report; thresholds in `data/validation.toml`. |
| 7 emit | `emit/{snapshot.rs, bundle.rs, report.rs}` | `data/generated/{weapons,items,enemies,stances,modular,factions,riven,stable_keys,manifest}.json` (sorted keys, 2-space indent, floats rounded to 6 significant digits to remove f32 noise, `DS:wfcd` §8) and `data/generated/REPORT.md` (coverage, unmatched, validation warnings, diff summary). `wfd-data/build.rs` turns the committed JSON into `bundle.bin` at compile time (pure Rust, no network, no Lua), so the embedded bundle always equals the reviewed snapshot. |

### 2.3 Bundle format, size, shipping, refresh

```rust
// wfd-model/src/bundle.rs
pub struct Bundle {
    pub manifest: Manifest,
    pub weapons: Vec<Weapon>,              // attacks inline
    pub items: Vec<Item>,                  // mods, arcanes, shards, Incarnon perks, buffs, auras, frame mods, stances, set bonuses
    pub effects: Vec<EffectSet>,           // indexed by ItemId; empty set = "effect not modelled yet"
    pub sets: Vec<SetDef>,
    pub stances: Vec<Stance>,
    pub modular: ModularTables,            // kitgun chambers/grips/loaders, zaw strikes/grips/links
    pub enemies: Vec<Enemy>,
    pub faction_table: FactionTable,       // type_mod[f][t], overguard rule
    pub riven: RivenTables,                // stat → effect mapping, base values per class (PE upgradeEntries)
    pub incompat_groups: Vec<IncompatGroup>,
    pub custom_stats: Vec<CustomStatDef>,  // T-26
    pub names: NameTable,                  // "en" now; other locales are optional packs keyed by uniqueName
    pub stable_keys: Vec<(StableKey, ItemRef)>,
}
pub struct Manifest {
    pub schema_version: u32,               // bumped on any layout change; postcard is not self-describing
    pub data_version: String,              // "2026-11-23+<short hash of inputs>"
    pub game_baseline: String,             // "Hotfix 44.1.4" (spec 1.1)
    pub sources: Vec<SourceStamp>,         // module, revid, timestamp / WFCD tag + sha / PE buildLabel + manifest hashes
    pub curated_rev: String,               // git describe of data/curated at build time
    pub licences: Vec<LicenceNotice>,
    pub coverage: Coverage,                // counts of items with and without effects
}
```

| Property | Choice | Reason |
|---|---|---|
| Wire format | `postcard` (varint, no field names) + zstd level 19; decoded with `ruzstd` | about 1.5 MB raw → about 0.5 MB compressed (estimate: 911 weapons × ~2.2 attacks, 1,550 mods, 1,019 enemies, 132 stances, names); loads in tens of ms; no C dependency at runtime. |
| Reviewable form | `data/generated/*.json` committed | a game update shows up as a readable diff in a pull request; tests can load JSON fixtures. |
| Shipping | default bundle embedded in the binary (`include_bytes!`) | the binary works offline on first start. |
| Override | `<data dir>/bundles/wfdata-<data_version>.bin`; the newest bundle whose `schema_version` matches wins; otherwise the embedded one | user refresh without reinstalling. |
| Hot reload | `ArcSwap<Bundle>` in the server; new requests see the new bundle, running optimizer jobs keep their `Arc` | no restart after `data update`. |

**User refresh after a game update.** `warframe-dps data update` (also a button on the About page, `POST /api/data/update`
with SSE progress) runs stages 1-7 with the curated tables embedded in the binary, writes the override bundle, and
hot-swaps it. Weapons and enemies are fully refreshed. New mods, arcanes or perks without curated effects appear
in the UI marked "effect not modelled" and are excluded from the optimizer pool by default (the user can include them
as zero-effect fillers). Curated entries whose `source_text` no longer matches the wiki description are marked
"possibly changed by an update" in the UI (section 2.5). `data status` prints the bundle in use and its sources; `data
verify` re-runs validation. A future option, not planned now: fetch a newer published curated set from the project's
releases.

**Licence attribution.** `data/NOTICE.md` (copied into the manifest and shown on the About page): wiki content
CC BY-NC-SA 3.0 with a link and "modified: converted, joined, normalized"; WFCD MIT notice; DE Public Export used for
identifiers (no licence stated); "Warframe is a trademark of Digital Extremes; this project is not affiliated".
Images are not bundled: the server proxies WFCD image URLs on demand and caches them in the data directory
(`GET /api/img/{image}`); offline, the UI falls back to generated glyph tiles.

### 2.4 Curated tables

**Format: TOML**, one file per family, one array-of-tables entry per item, keyed by `uniqueName`. TOML because it is
easy to hand-edit, supports comments (citations and reasoning), and has a mature serde crate; `cargo xtask
gen-schema` emits a JSON Schema from the `wfd-model` types so editors with Taplo validate as the curator types.

```text
data/curated/
├── mods/{rifle,shotgun,pistol,sniper,bow,primary,melee,archgun,exalted,augments,companion}.toml
├── frame/{mods,auras}.toml
├── arcanes/{primary,secondary,melee,kitgun,zaw,bow,shotgun,warframe}.toml
├── shards.toml                   # one entry per colour×stat, tauforged factor
├── sets.toml                     # set bonuses by piece count
├── incarnon/<family>.toml        # one file per Incarnon family: tiers, perks, per-variant values
├── buffs/{abilities,focus,companions,squad,debuffs}.toml
├── custom_stats.toml             # the 26 custom stats of T-26 as effects
├── riven.toml                    # riven trait → effect, unit conventions, value formula constants
├── stat_lines.toml               # tier 1: stat-line patterns → targets (04 §5.1)
├── co_overrides.toml             # name-mapping fixes for the ingested CO catalogue (04 §2.14)
├── stance_co.toml                # per stance input CO effectiveness (F:addmul §1.10)
├── attack_overrides.toml         # parent links, beam ramp start, semi-auto cap, heavy durations, sniper combo …
├── enemy_overrides.toml          # special DR, attenuation, toxin_blocked, head-crit parts, caps
├── progenitor.toml               # Kuva/Tenet/Coda weapons and allowed elements
└── inspiration_aliases.toml      # inspiration names → our ids (for link import only, never for data)
```

Entry skeleton (full encoded examples are in section 3.6):

```toml
[[mod]]
id          = "/Lotus/Upgrades/Mods/Rifle/WeaponDamageAmountMod"
name        = "Serration"                      # must equal the catalogue name (checked)
source_text = "+165% Damage"                   # wiki Mods/data Description at authoring time
cite        = ["W:Serration", "F:core §2"]
status      = "verified"                       # verified | draft | unverified (UNVERIFIED in the spec sense)
effects     = [ { stat = "base_dmg", value = 1.65 } ]
```

**Three tiers of effect data, so hand work goes only where it is needed.**

| Tier | Source | Covers | Output |
|---|---|---|---|
| 1. Stat-line import (automatic) | `data/curated/stat_lines.toml`: the stat-line → bucket mapping of 04 §5.1, as regex patterns over the wiki `Description` / WFCD `levelStats` lines (`^\+(?P<v>[\d.]+)% Damage$` → `base_dmg`, `(x2 for Bows)` → `scope_factor`, DE `<DT_…>` tokens → damage types) | single-line unconditional effects, about 70 % of weapon mods (`DS:wfcd` §3.4, `DS:wiki` §4) | effects with `origin = "auto"`; never used when a curated entry exists |
| 2. Spec-table ingestion (automatic) | tables the topic documents mark as ingestable, parsed from the Markdown by `wfd-datagen spec-tables`: the per-attack CO catalogue (04 §2.14: behaviour, ρ_co, `co_ref`, `B_co^abs`, flags), the status summary (03 §4.1), the faction modifier table (05 §5.1) | data that the spec itself owns | generated TOML under `data/generated/spec/`; the spec stays the only place these numbers are written |
| 3. Curated entries (hand-written) | `data/curated/**/*.toml` | conditional, stacking and special effects, arcanes, shards, perks, sets, buffs, riven mapping, and the hand-authored tables of 06 §4.3 and 05 §4.5 | effects with `origin = "curated"` |

Every bundle item records which tier produced its effects; the coverage report counts each tier, and `curate check`
flags auto-imported items whose description has lines the importer could not map (they then need a curated entry).

**How a curator adds a mod's effects.**

1. `wfd-datagen curate todo` lists catalogue items without effects, ordered by a priority list (meta mods first)
   and grouped by file.
2. `wfd-datagen curate scaffold <uniqueName or name>` appends a stub to the right file: `id`, `name`,
   `source_text`, `max_rank`, and *suggested* effects parsed from the description with the simple-line patterns of
   `DS:wfcd` §3.4 (about 70 % of weapon mods), each marked `# suggested`. Conditional lines (`On Kill:`, `Stacks up
   to`, `per Status Type`, `x2 for Bows`) produce a commented template of the condition block, never a guess.
3. The curator fills in the semantics (bucket, condition, scope) from the spec and the cited wiki page.
4. `wfd-datagen curate check` (also run in CI) validates (section 2.5) and prints the rendered English description
   of the encoding next to the source text, so a mismatch is visible at a glance.
5. A new mechanic (a new `Target` or `Trigger` variant) needs an engine change plus a spec test vector; the
   validator rejects unknown variants, so data can never silently reference engine behaviour that does not exist.

### 2.5 Validation

| Check | Severity | Rule |
|---|---|---|
| Schema | error | TOML parses into `wfd-model` types; unknown keys rejected (`deny_unknown_fields`). |
| References | error | every `id` exists in the catalogue; `name` matches; every weapon/attack referenced by `co_catalogue.toml` exists. |
| Numbers vs source text | warning (error for `status = "verified"`) | the multiset of numbers in `source_text` (percent values ÷ 100 where the stat is a fraction) must equal the multiset of max-rank numbers in the encoding, after the per-stat unit conversion. Numbers that are not effects (durations, stack counts) are matched against the condition block (`duration`, `max`). Leftovers on either side are reported. Example: "+80% Multishot / On Kill: +30% Multishot for 20s. Stacks up to 5x." must account for 0.80, 0.30, 20, 5. |
| Source drift | warning, shown in UI | `source_text` differs from the current wiki `Description` → the entry is flagged "possibly changed by an update". |
| Upgrade types | warning | the wiki `UpgradeTypes` of the item map to a set of expected targets (`WEAPON_DAMAGE_AMOUNT` → `base_dmg`, `WEAPON_FIRE_ITERATIONS` → `multishot`, `WEAPON_PERCENT_BASE_DAMAGE_ADDED` → element, `WEAPON_DAMAGE_IF_VICTIM_PROC_ACTIVE` → CO family …); a mismatch is reported (`UpgradeTypes` is known to be wrong sometimes, `DS:wiki` §11, so not an error). |
| Rank scaling | warning | for every rank r, the encoded value at r (default `max × (r+1)/(max_rank+1)`, or an explicit table) is compared with the number parsed from WFCD `levelStats[r]` within the 1-decimal display rounding. |
| Ranges and units | error | fractions in plausible ranges per stat (declared in the stat layout), stack counts ≥ 1, durations > 0, Tauforged factor 1.5. |
| Coverage | report; error below thresholds | `data/validation.toml` sets minimum coverage per category (for example every mod in the inspiration's parity set and every arcane with a damage `UpgradeType`). |
| Cross-source | warning with allowlist | wiki attack vs PE top-level crit/status/fire rate/magazine per weapon; PE sums attacks for some weapons (`DS:pe` TL;DR), allowlisted by weapon. |
| Spec anchors | error | a small set of hand-checked records (the spec's worked example: Braton Prime normal attack, Lancer, Serration, Split Chamber, Point Strike, Vital Sense, Infected Clip, Stormbringer, Primed Bane of Grineer, Speed Trigger; spec 3.8) must be present with exactly those values. |

---

## 3. Domain model and the effect encoding

All types below live in `wfd-model`. Enumerations follow the spec's order (spec 2.2) so they can index arrays:
`DamageType` (18: `IMP PUN SLA HEA COL ELE TOX BLA RAD GAS MAG VIR COR VOI TAU TRU CIN FIN`), `StatusType` (18),
`Faction` (17), `Layer` (`OG SH HP`), `View` (`Health Armor Shield Overguard Layered`). Damage maps are
`DamageVec = [f64; 18]`. Numbers are fractions inside the model (U-1).

### 3.1 Identifiers

| Type | Meaning | Lifetime |
|---|---|---|
| `UniqueName(Box<str>)` | DE path, the join key of every source | permanent |
| `ItemId(u32)`, `WeaponId(u32)`, `EnemyId(u32)` | dense index into the bundle's vectors | one bundle |
| `StableKey(u32)` | append-only registry entry (`data/generated/stable_keys.json`) | permanent; used in share strings |
| `AttackIdx(u8)` | index into `Weapon::attacks` | one bundle, stable while the wiki keeps attack order (attacks are also matched by name when decoding share strings) |

### 3.2 Weapons, attacks, variants

```rust
pub struct Weapon {
    pub id: WeaponId, pub unique_name: UniqueName, pub name: NameKey, pub family: FamilyId, // family: Incarnon perks, rivens
    pub class: WeaponClass,              // Rifle, Shotgun, Sniper, Bow, Launcher, Pistol, DualPistols, Thrown, melee classes, Archgun, Archmelee, Exalted*, Companion
    pub slot: WeaponSlot,                // Primary, Secondary, Melee, Archgun, Archmelee, Exalted, Companion
    pub categories: CategorySet,         // bitflags used by mod compatibility (wiki Type + itemCompatibility)
    pub compat_tags: SmallVec<[TagId; 4]>,
    pub traits: Traits,                  // PRIME | KUVA | TENET | CODA | INCARNON | KITGUN | ZAW | EXALTED | BATTERY | BEAM ...
    pub trigger: Trigger,                // normalized from 33 strings
    pub attacks: SmallVec<[Attack; 3]>,
    pub groups: SmallVec<[AttackGroup; 1]>,  // parent + children ("combine attack effects", T-22, CD-12)
    pub magazine: f64, pub reload: Reload,   // Reload::{Standard{t}, PerShell{start_end, per_shell}, Battery{delay, delay_empty, rate}}
    pub ammo_max: f64, pub spool: Option<Spool>, pub sniper_combo: Option<SniperCombo>,
    pub zoom: SmallVec<[ZoomLevel; 3]>,
    pub melee: Option<MeleeData>,        // combo duration, follow-through, range, wind-up, heavy total, stance class, default stance
    pub incarnon: Option<IncarnonDef>,   // tiers → perk ItemIds; form attack indices
    pub modular: Option<Modular>,        // Kitgun { chamber } | Zaw { strike }
    pub exalted: Option<ExaltedScaling>, // Strength scaling of base damage (m_str, E-1)
    pub innate: EffectSet,               // weapon passives in the effect DSL (Cedo CO, physical-mod increase, ...)
    pub polarities: SmallVec<[Polarity; 8]>, pub exilus_polarity: Option<Polarity>, pub max_rank: u8,
    pub image: Option<Box<str>>,
}

pub struct Attack {
    pub name: NameKey, pub kind: AttackKind, pub form: Form,     // Form::{Normal, Incarnon}
    pub shot: ShotType,                  // HitScan | Projectile | AoE | Beam | Thrown | DoT | Melee
    pub parent: Option<AttackIdx>,       // radial child of a parent hit (F:weapon §10.1)
    pub base: DamageVec,                 // per pellet (B0_t)
    pub cc: f64, pub cdm: f64, pub sc: f64, pub fire_rate: f64, pub multishot: f64,
    pub charge: Option<Charge>, pub burst: Option<Burst>, pub ammo_cost: f64,
    pub falloff: Option<Falloff>, pub radius: Option<f64>,
    pub forced: SmallVec<[StatusType; 2]>,
    pub co: CoSpec,                      // { behaviour, ratio, co_ref, fixed_abs, flags } from 04 §2.14 (E-11)
    pub headshot: HeadshotRule,          // normal | no headshot multiplier (radial) | 1x weapon
    pub beam: Option<BeamSpec>,          // ramp start (0.2 default, override list; OQ-013)
    pub incarnon_charges: Option<u32>,   // form "magazine"
    pub is_heavy: Option<bool>,          // explicit heavy flag (T-09 auto mode)
    pub explosion_delay: Option<f64>,
}
```

- **Multi-part attacks.** Every wiki attack is its own `Attack`, evaluated on its own (CD-12); `parent` and `groups`
  give the summed parent + child outputs and the shared status timeline of T-22.
- **Incarnon forms** are attacks with `form = Incarnon` (the wiki lists them, `DS:wiki` §3); they use
  `incarnon_charges` as magazine and ignore ammo efficiency. Perks are items (section 3.3) whose `scope.form` selects
  the normal or Incarnon form (the inspiration's `NORMAL`/`INCARNON` sub-blocks, `I:model` §1.3).
- **Kitgun and zaw assembly** is a pure engine function, `assemble::kitgun(&Modular, chamber, grip, loader) ->
  Weapon` and `assemble::zaw(strike, grip, link) -> Weapon`, over `ModularTables` (wiki `Modular/data`; the 1,200
  pre-expanded kitgun records are used only as a cross-check, `DS:wiki` §3). Results are cached per part triple.
- **Variant parameters** are part of the build: `WeaponVariant { kitgun: Option<(GripId, LoaderId)>, zaw:
  Option<(GripId, LinkId)>, progenitor: Option<(Elem, f64)>, exalted_element: Option<DamageType> }`.

### 3.3 Items, rivens, enemies, builds, scenarios

```rust
pub struct Item {
    pub id: ItemId, pub unique_name: UniqueName, pub name: NameKey, pub kind: ItemKind,
    pub compat: Compat,                  // allowed categories/tags, forbidding tags (uncomptag), Semi-only trigger
    pub incompat: SmallVec<[ItemId; 4]>, // union of both directions of the wiki lists (D-093)
    pub polarity: Option<Polarity>, pub base_drain: i8, pub max_rank: u8, pub rarity: Rarity,
    pub set: Option<SetId>, pub image: Option<Box<str>>,
}
pub enum ItemKind {
    WeaponMod { exilus_capable: bool }, Stance { stance: StanceId }, WeaponArcane { class: ArcaneClass },
    FrameMod, Aura, FrameArcane, Shard { color: ShardColor, tauforged: bool },
    IncarnonPerk { family: FamilyId, tier: u8 }, Buff { params: SmallVec<[ParamDef; 2]> }, CustomStat,
}
pub struct RivenInstance { pub class: RivenClass, pub traits: ArrayVec<(RivenTrait, f64), 4> } // values in the model's units
pub struct Enemy {
    pub id: EnemyId, pub name: NameKey, pub faction: Faction, pub base_level: u16,
    pub health: f64, pub shield: f64, pub armor: f64, pub overguard: f64, pub eximus: EximusData,
    pub parts: SmallVec<[BodyPart; 4]>,  // { name, mult, weak_point, head, head_crit } parsed from `Multis`
    pub immune: StatusSet, pub caps: [Option<u8>; 18], pub flags: EnemyFlags, // TOXIN_BLOCKED, EXIMUS_OFF, PARTY_SCALED, BOSS
    pub special_dr: Option<SpecialDr>,   // attenuation, flat DR, adaptation (05 §4.5, enemy_overrides.toml, OQ-022)
    pub sp_health_bonus: f64,            // D-055
}
pub struct Build {
    pub weapon: WeaponId, pub variant: WeaponVariant,
    pub main: [Slot; 8], pub exilus: Slot,          // Slot::{Empty, Mod { item, rank }, Riven(RivenInstance)}
    pub stance: Option<(ItemId, ComboKey)>,
    pub weapon_arcanes: ArrayVec<Option<(ItemId, u8)>, 2>,   // two for archguns
    pub incarnon: Option<[Option<u8>; 5]>,                   // perk index per tier
    pub frame: FrameLoadout,                                 // mods [Slot; 4], aura, arcanes [2], shards [Option<ItemId>; 5]
    pub polarity: Option<PolarityLayout>,                    // only for the capacity constraint
}
pub struct Scenario {
    pub enemy: EnemyInstance,            // enemy, level, steel_path, eximus, party_size (T-23)
    pub toggles: Toggles,                // typed T-01..T-12, T-19..T-22, T-24, T-25, T-27..T-29
    pub buffs: SmallVec<[BuffInstance; 4]>,   // T-13..T-18 and every other ability/focus/companion buff, with params
    pub custom: SmallVec<[(CustomStatKey, f64); 4]>, // T-26
    pub profile: Profile,                // Game | Compat(CompatFlags) — Compat only in oracle tests (spec 4.6)
}
```

The build and the scenario are separate types because the optimizer varies the build under a fixed scenario; the
share string carries both (like the inspiration's link). Archon shards, Incarnon perks, set bonuses, buffs and
custom stats are all items or item-like inputs and use the same effect DSL.

### 3.4 The effect DSL

```rust
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
pub struct EffectDef {
    #[serde(flatten)] pub target: Target,        // which bucket or mechanic
    pub value: ValueExpr,                        // magnitude at max rank (or a parameter)
    #[serde(default)] pub when: Cond,            // gating condition (default Always)
    #[serde(default)] pub scope: Scope,          // where it applies (default: anything the item can equip)
    #[serde(default)] pub scope_factor: SmallVec<[ScopeFactor; 1]>, // e.g. ×2 on heavy attacks, ×2 for bows
    #[serde(default)] pub value_per: Option<Per>, // shorthand: `value = 0.4, value_per = "stack"` ≡ `value = { base = 0.4, per = "stack" }`
}

pub enum Target {                                // TOML: discriminated by the key that is present
    Stat     { stat: StatKey },                  // additive into one flat-vector slot (section 3.5)
    Element  { element: Elem, #[serde(default)] order: ElemOrder }, // ElemOrder::{Slot (combining, by slot), NonCombining, PostModHcet}
    Faction  { faction: FactionSel },            // sum_faction when the target's faction matches (D-008 sub-factions)
    Mult     { mult: MultGroupKey },             // one factor group of G or G_F (registry, section 3.5)
    ExtraHit { extra_hit: ExtraHitKind },        // E-14: { element, part: Once | Twice, status: RollWeaponSc | Forced }
    ProcOn   { proc_on: ProcEvent, status: StatusType }, // CritInstance, ImpactProc, ToxinProc, PunctureProc, AnyProc ...
    Forced   { forced_proc: StatusType },        // forced procs per pellet (φ_s)
    Vuln     { vuln: VulnKind },                 // Multiplicative | Additive subgroup of `vuln` (spec 2.4)
    Strip    { strip: StripKind },               // Ability | CorrosiveProjection (E-15)
    External { external_status: StatusType },   // permanent external stacks (T-06)
    StackCap { stack_cap: StatusType },          // cap increase (Emerald shard: Corrosive +2)
    Lock     { lock: LockKind },                 // MultishotToBase (Acuity), FireRateToBase (Cannonade)
    Convert  { convert: [DamageType; 2] },       // physical conversion (D-009: adds into the target type)
    Hook     { hook: HookKey },                  // named engine behaviour with parameters (Melee Duplicate, Debilitate, ...)
}

pub enum ValueExpr { Const(f64), Spec(ValueSpec) }   // `value = 1.65` or `value = { … }`
pub struct ValueSpec {
    pub base: Option<f64>,                       // at max rank
    #[serde(default)] pub rank: RankScale,       // Linear (max·(r+1)/(R+1), default) | Table(Vec<f64>) | Fixed
    #[serde(default)] pub per: Per,              // None | Stack | ComboTierAboveOne | DistinctStatus | StatusStack(s) | SetPiece | ShardCount{colors}
    #[serde(default)] pub scale: SmallVec<[Scale; 1]>,  // Strength (Str at the caster) | Duration | ...
    pub param: Option<ParamKey>,                 // user-supplied value (buff value, riven trait value)
    #[serde(default)] pub add: SmallVec<[Term; 1]>,     // extra linear terms (Violet shard per qualifying shard)
    pub variants: Option<BTreeMap<FamilyKey, f64>>,     // per-variant values (Incarnon: Furis 28 / MK1-Furis 34)
}

pub enum Cond {
    Always,
    Event  { trigger: Trigger, #[serde(default = "one")] chance: f64, #[serde(default)] stacks: StackModel }, // class (b)/(c); default: one stack while the condition holds
    Target { target: TargetPred },               // T-02 class (a): HasStatus(s), Frozen, HealthBelow(x), DistinctBelow(n)
    Shot   { shot: ShotPos },                    // FirstOfMagazine | LastOfMagazine (schedule-driven in every mode)
    Hit    { hit: HitPred },                     // WeakPoint | Headshot | Body (bound to T-03, not T-02)
    Attack { attack: AttackPred },               // ModdedCcBelow(x) (evaluated on the resolved attack)
    All(Vec<Cond>),
}
pub struct StackModel {                          // field names and values are those of 04 §4.2 (effect state machine E-4.9)
    pub max: u8, #[serde(default = "one")] pub gain: u8, pub duration: f64,
    #[serde(default)] pub stack_timer: StackTimer,   // SharedLoseOne | SharedLoseAll | Independent
    #[serde(default)] pub refresh: Refresh,          // Refresh | NoRefreshWhileActive
    #[serde(default)] pub decay: ValueDecay,         // None | Linear | Garuda (value decay over the duration)
}
pub enum Trigger {                               // ids exactly as 04 §4.1; class() → (b) own event or (c) external
    OnHit, OnCrit, OnTier1Crit, OnWeakPointHit, OnHeadshot, OnStatus(StatusType), OnReload, OnReloadFromEmpty,
    WhileAiming, FirstShot, LastShot, Combo, OnMeleeHitCount, NextShotAfter(TriggerRef),                          // (b)
    OnKill, OnWeakPointKill, OnMeleeKill, OnPrimaryKill, OnAbilityCast, WhileInvisible, OnWallLatch, OnHolster,
    OnDamaged, OnRoll, WhileOvershields, OnShieldBreak, InZone, AllyBuffs, EnergyLevel, ShieldLevel, ArmorLevel,    // (c)
}
pub struct Scope {                               // the "gate" of 04 §4.2: instance properties, never assumed in any T-02 mode
    pub weapon: WeaponClassSet,                  // empty = any weapon the item can equip
    pub attacks: AttackSel,                      // Any | Direct | Radial | Heavy | NonHeavy | Slam | Charged | Named(...)
    pub form: FormSel,                           // Any | Normal | Incarnon
    pub layer: Option<Layer>,                    // e.g. overguard only (Secondary Fortifier)
    pub exalted: ExaltedSel,                     // Any | Never (ability-damage shards, D-068)
}
```

Why this shape is fast: every `Target` except `Element`, `Convert` and `Hook` compiles to "add k to slot i", every
`Cond` compiles to "constant" or "coefficient × state variable", and every `Scope` is resolved per attack at L1.
The engine never interprets the DSL during search.

### 3.5 Flat stat layout and state variables

`wfd-model/src/stats.rs` declares the layout once with a macro; it generates the `StatKey` enum, the TOML names,
the slot indices, unit conversions for validation, display keys for the UI, and the monotonicity class used by the
optimizer (section 5.4):

```rust
define_stats! {
//  key                 slot(s)  spec symbol         aggregation  monotone  owner
    base_dmg            1        "sum_base_dmg"      sum          Inc       "01";
    universal_dmg       1        "melee proj. bucket" sum         Inc       "04";
    incarnon_flat       1        "incarnon_flat"     sum          Inc       "01";
    phys                3        "sum_phys[t]"       sum          Unknown   "01";   // shifts proc weights: enumerated
    elem_nc             10       "sum_elem[t] (non-combining)" sum Unknown  "01";
    faction             1        "sum_faction"       sum          Inc       "01";
    co_per_status       1        "sum_co / n"        sum          Inc       "04";   // coefficient of n
    co_per_cold_stack   1        "sum_co / n_COL"    sum          Inc       "04";
    co_flat             1        "sum_co const"      sum          Inc       "04";
    mult_g              32       "mult_other groups" sum→Π(1+x)   Inc       "04";
    mult_gf             16       "mult_final_omitted groups" sum→Π(1+x) Inc "04";
    cc_base_add cc_rel cc_abs   1 each  "CC0+ ΣCC_rel ΣCC_abs" sum Inc*     "02";   // *Unknown if an AttackPred reads CC
    cc_floor            1        "cc_floor"          max          Inc       "02";
    cc_final_mul        1        "custom p.d"        sum          Inc       "02";
    cd_base_add cd_rel cd_abs cd_field_mul 1 each "CDM0+ ΣCD_rel ΣCD_abs M_cdf" sum Inc "02";
    tier_raise          1        "tier_raise_chance" sum          Inc       "02";
    sc_base_add sc_rel sc_abs   1 each  "SC0+ ΣSC_rel ΣSC_abs" sum Inc*     "02";
    sc_mul              1        "sc_mult"           sum→(1+x)    Inc*      "02";
    multishot multishot_flat 1 each "sum_ms"         sum          Inc       "02";
    wp_dmg hs_mult      1 each   "ΣWP ΣHS"           sum          Inc       "02";
    sonar_mult          1        "sonar_mult"        sum→(1+x)    Inc       "02";
    proc_on_crit        18       "p_hunter (per s)"  sum          Inc       "02";
    proc_on_impact      18       "p_ib (per s)"      sum          Inc       "02";
    proc_on_status      18×k     "Archon Continuity…" sum         Inc       "03";
    forced              18       "forced_procs[s]"   sum          Unknown   "02";
    status_dmg          1        "sum_status_dmg"    sum          Inc       "03";
    status_dmg_type     18       "status_dmg_type[s]" sum         Inc       "03";
    status_dur          1        "sum_status_dur"    sum          Inc       "03";
    dot_vuln            1        "custom p.o (M_dot)" sum→(1+x)   Inc       "03";
    stack_cap_add       18       "stack_cap[s] +"    sum          Inc       "03";
    fire_rate fire_rate_mul magazine magazine_flat reload_speed ammo_eff charge_speed 1 each sum Inc "06";
    attack_speed initial_combo heavy_eff combo_dur wind_up 1 each sum Inc     "07";
    xh                  8        "extra_hit_frac by kind" sum     Inc       "04";
    vuln_mul            8        "vuln multiplicative groups" sum→Π(1+x) Inc "04";
    vuln_add            1        "vuln additive"     sum          Inc       "04";
    strip_ability strip_cp 1 each "E-15 strips"      sum          Inc       "05";
    og_mult             1        "M_OG"              sum          Inc       "04";
    external            18       "external stacks"   max          Unknown   "03";
    strength duration   1 each   "Str"               sum          Inc       "04";
    lock_multishot lock_fire_rate 1 each "locks"     count>0      Dec       "02/06";
    hook                16       "named hooks"       count>0      Unknown   "04";
}   // roughly 300 slots → StatVec = [i64; N] (about 2.4 KB); search code works on a compacted "active" subset (section 5.5)
```

`mult_g`/`mult_gf` slots come from `data/curated/mult_groups.toml`: each group (`chamber`, `synth_charge`,
`eclipse`, `parasitic_link`, `mecha_empowered`, `slam`, `finisher`, `stance`, `heavy`, `sniper_combo` …) declares
`kind = "g" | "g_f"` (the `G_F` list of `F:addmul` §1.6 and spec 2.4), how it enters DoT seeds (`dot = "inherit" |
"twice" | "none"`, E-13 `M_dot`), and whether its members add (Primed/Charged Chamber are additive with each other,
spec 3.1a). An unknown multiplier defaults to a new `g` group (spec 4.2: "an unlisted multiplier defaults to G").

**State variables.** Conditions that the compiler cannot reduce to a constant become coefficients on one of:

| `StateVar` | Meaning | EV value | MC value |
|---|---|---|---|
| `Distinct` | n, distinct statuses counting for CO (T-05) | `Σ_s P(n_s ≥ 1)` + externals, or T-05 `weapon_types`/`manual` | live count before the instance (CD-04) |
| `Stacks(s)` | live stacks of status s | Poisson model (spec 4.2) | live |
| `Present(s)`, `Frozen`, `HealthBelow(x)` | predicates | probabilities from the stack model; health thresholds only in the layered pass | live |
| `ComboTier` | `M(c) − 1` | combo time model (07) or T-10 fixed | live |
| `CondStacks(id)` | stacks of one triggered condition (`simulated` mode) | class (b): E-4.10 of `04` (Poisson trigger approximation, Markov chain for `shared_lose_one`), walked through time in window mode; class (c): `u_e·m_e` (user uptime × stacks, 04 §4.3) | live (E-4.9 state machine) |
| `FirstShot`, `LastShot` | schedule marks | from the schedule (per shot) | per shot |

**Compile rules per T-02 mode** (the whole of spec 6.2 in one table):

| Condition | `all_max` | `none` | `simulated` |
|---|---|---|---|
| `Event`, class (b) | constant `value × max` | 0 | coefficient on `CondStacks(id)`; trigger rate from the attack's own events |
| `Event`, class (c) | constant `value × max` | 0 | coefficient on `CondStacks(id)` with the user's uptime/stack input (default max) |
| `Target` (a), count-based (`per = distinct_status`, `status_stack`) | coefficient on the state variable, n per T-05 | 0 | coefficient on the state variable |
| `Target` (a), threshold | constant (true) | 0 | probability of the predicate |
| `Shot` | schedule-driven | schedule-driven | schedule-driven (spec 6.2 last paragraph) |
| `Hit` | constant from T-03 | constant from T-03 | constant from T-03 |
| `Attack` | evaluated on the resolved attack | same | same |
| `Per::ComboTierAboveOne` | coefficient on `ComboTier` (T-10 may fix it) | same | same |

### 3.6 Encoded examples

Values come from the findings tables cited per entry and are re-checked by the validator against `source_text`.
`uniqueName`s other than Serration's are abbreviated `…` here (the scaffold command fills them in). An entry-level
`scope` is the default scope of all its effects; an effect-level `scope` overrides it. Short effects are written as
inline tables in an `effects = [...]` array, longer ones as `[[mod.effects]]` sub-tables (TOML 1.0 inline tables must
fit on one line). Every TOML block below parses with a standard TOML 1.0 parser (checked with Python `tomllib`).

**1. Serration** (`F:core` §2; base bucket A):

```toml
[[mod]]
id = "/Lotus/Upgrades/Mods/Rifle/WeaponDamageAmountMod"
name = "Serration"
source_text = "+165% Damage"
effects = [ { stat = "base_dmg", value = 1.65 } ]
```
Compiles to `StatVec[base_dmg] += 1_650_000_000` at max rank; rank r gives `1.65·(r+1)/11` (Linear).

**2. Galvanized Aptitude** (`F:mods` §6.1, `F:addmul` §1.1; CO family, adding or multiplying per attack):

```toml
[[mod]]
id = "…"
name = "Galvanized Aptitude"
source_text = "+80% Status Chance\r\nOn Kill:\r\n+40% Direct Damage per Status Type affecting the target for 20s. Stacks up to 2x."

[[mod.effects]]
stat = "sc_rel"
value = 0.80

[[mod.effects]]
stat = "co_per_status"
value = 0.40
value_per = "stack"
when = { trigger = "on_kill", stacks = { max = 2, duration = 20, stack_timer = "shared_lose_one" } }
```
`all_max`: `co_per_status += 0.80` (constant), multiplied by n from T-05 in the kernel. `simulated`: coefficient
`0.40 × CondStacks(id)`, times `Distinct`. Whether C is added (`ρ_co·B_ref·C`) or multiplied (`1 + C`) is **not** a
property of the mod: it is the attack's `CoSpec` (E-11), see example 13.

**3. Condition Overload** (`F:addmul` §1.1; class (a)):

```toml
[[mod]]
id = "…"
name = "Condition Overload"
source_text = "+80% Melee Damage per Status Type affecting the target"
effects = [ { stat = "co_per_status", value = 0.80, scope = { weapon = ["melee"] } } ]
```
The stance-input CO ratio (`F:addmul` §1.10) is data on the stance (`stance_co.toml`), applied by the engine.

**4. Primary Deadhead** (weapon arcane; `F:mods` §11 table, `F:addmul` §5):

```toml
[[arcane]]
id = "…"
name = "Primary Deadhead"
max_rank = 5
source_text = "On Weak Point Kill:\r\n+120% Damage for 24s. Stacks up to 3x.\r\n+30% Headshot Multiplier"   # checked against Arcane/data
scope = { weapon = ["primary"] }

[[arcane.effects]]
stat = "base_dmg"
value = 1.20
value_per = "stack"
when = { trigger = "on_weak_point_kill", stacks = { max = 3, duration = 24, stack_timer = "shared_lose_all" } }   # 04 §4.2 (UNVERIFIED there)

[[arcane.effects]]
stat = "hs_mult"
value = 0.30
```
On-kill → class (c) (04 §4.1: kills are external in every view): `all_max` adds 3.60 to A; `none` adds 0; the optimizer's "no on-kill bonuses" rule drops the
first effect, not the whole arcane, which is more precise than the inspiration's text regex (`I:opt` §9 item 12).

**5. Hunter Munitions** (`F:addmul` §8, D-016, D-070):

```toml
[[mod]]
id = "…"
name = "Hunter Munitions"
set = "hunter"
source_text = "+30% chance to apply a Slash Status Effect on Critical Hits"
effects = [ { proc_on = "crit_instance", status = "SLA", value = 0.30 } ]
```
Compiles to `proc_on_crit[SLA] += 0.30`. The engine combines it with Internal Bleeding as `1 − (1 − a)(1 − b)` (spec
4.2) and counts the proc as a Slash stack (D-070).

**6. A riven** (user input; mapping in `riven.toml`, `F:mods` §8.5, D-010, D-011):

```toml
# data/curated/riven.toml (excerpt)
[trait.critical_chance]
target = { stat = "cc_rel" }
input = "percent"

[trait.multishot]
target = { stat = "multishot" }
input = "percent"
classes = ["rifle", "shotgun", "pistol", "archgun"]

[trait.toxin]
target = { element = "TOX" }          # combines at the riven's slot, bottom-up rule for several elements
input = "percent"

[trait.damage_to_grineer]
target = { faction = ["grineer", "kuva_grineer"] }
input = "multiplier"                  # typed 0.55 → bonus −0.45 (I:feat §6 units)

[trait.weak_point_critical_chance]    # D-010
target = { stat = "cc_rel" }
when = { hit = "weak_point" }
input = "percent"

[trait.weak_point_damage]             # D-011
target = { stat = "wp_dmg" }
input = "percent"

[trait.fire_rate]
target = { stat = "fire_rate" }
input = "percent"
scope_factor = [ { weapon = ["bow"], factor = 2 } ]
```
```json
{ "riven": { "class": "rifle", "traits": [["critical_chance", 1.502], ["multishot", 1.105],
                                          ["toxin", 0.904], ["damage_to_grineer", 0.55]] } }
```
Compiles to `cc_rel +1.502`, `multishot +1.105`, element entry `(TOX, 0.904)` at the riven's slot position,
`faction −0.45` when the target is Grineer or Kuva Grineer. Which sub-factions each faction trait covers is the
curator's call per `W:Faction_Damage_Bonus` (D-008); the example shows the mechanism.

**7. Roar** (buff, T-13; `F:buffs` §1.1):

```toml
[[buff]]
id = "buff:roar"
name = "Roar (Rhino)"
toggle = "T-13"
params = [ { key = "value", default = 0.30, min = 0.0, max = 99.99, unit = "percent", note = "final value, already Strength-scaled" } ]
effects = [ { faction = "any", value = { param = "value" } } ]
```
`faction = "any"` adds to F whatever the target's faction (Roar sits in the faction bucket, spec 3.1a).

**8. Xata's Whisper** (buff, T-15; E-14, D-071):

```toml
[[buff]]
id = "buff:xatas_whisper"
name = "Xata's Whisper (Xaku)"
toggle = "T-15"
params = [ { key = "value", default = 0.26, unit = "percent" } ]
effects = [ { extra_hit = { element = "VOI", part = "twice", status = "roll_weapon_sc" }, value = { param = "value" } } ]
```
Compiles to `xh[xata] += 0.26`; E-14 lives in the engine (faction applied again, Part applied twice).

**9. Violet archon shard, Primary Electricity** (`F:buffs` §5, D-006, OQ-024):

```toml
[[shard]]
id = "shard:violet_primary_electricity"
color = "violet"
tauforged_factor = 1.5
source_text = "+30% Primary Electricity Damage, +10% per Crimson, Azure or Violet shard equipped"
scope = { weapon = ["primary"] }

[[shard.effects]]
element = "ELE"
order = "post_mod_hcet"
value = { base = 0.30, add = [ { coeff = 0.10, per = "shard_count", colors = ["crimson", "azure", "violet"] } ] }
```
The shard-count term is resolved by the shard finalizer (all five shards are known), so the optimizer enumerates
shard multisets as one compiled vector each. `order = "post_mod_hcet"` places the Electricity after modded elements,
with innate and progenitor elements, in HCET order (D-006).

**10. Incarnon evolution perks, Furis** (`F:mods` §9, `I:model` §2.3):

```toml
# data/curated/incarnon/furis.toml
[family]
id = "furis"
weapons = ["Furis", "MK1-Furis"]          # Genesis applies to all variants, with per-variant values

[[perk]]
id = "perk:furis:haven_foray"
tier = 2
name = "Haven Foray"
effects = [ { stat = "incarnon_flat", value = { variants = { "Furis" = 28, "MK1-Furis" = 34 } }, scope = { form = "normal" } } ]

[[perk]]
id = "perk:furis:prelude_of_might"
tier = 4
name = "Prelude of Might"
source_text = "Increase Base Critical Damage Multiplier by +3x when Critical Chance is below 40%"
effects = [ { stat = "cd_base_add", value = 3.0, when = { attack = { modded_cc_below = 0.40 } } } ]
```
Tier numbers and scopes here are illustrative; the curator takes them from the Genesis page. `incarnon_flat` enters
`B'` but not `B_ref` (E-1, E-11). `modded_cc_below` marks the CC slots `Unknown` for this scenario, so the optimizer
branches on CC mods instead of bounding them (section 5.4).

**11. Blood Rush** (`F:addmul` §2; class (b) combo):

```toml
[[mod]]
id = "…"
name = "Blood Rush"
source_text = "+40% Critical Chance per Combo Multiplier"
effects = [ { stat = "cc_rel", value = 0.40, value_per = "combo_tier_above_one", scope = { weapon = ["melee"] } } ]
```
Coefficient on `ComboTier` (`M(c) − 1`): T-10 fixes it, otherwise the combo time model of `07` drives it. The heavy
attack doubling of the True Steel family is a `scope_factor = [ { attacks = "heavy", factor = 2 } ]` on those mods.

**12. Arcane Avenger** (warframe arcane; `F:buffs` §7):

```toml
[[arcane]]
id = "…"
name = "Arcane Avenger"
kind = "frame"
source_text = "On Damaged:\r\n21% chance for +45% Critical Chance for 12s"

[[arcane.effects]]
stat = "cc_abs"
value = 0.45
when = { trigger = "on_damaged", chance = 0.21, stacks = { max = 1, duration = 12, refresh = "refresh" } }
```
Absolute crit chance (E-5). External class (c): `simulated` uses the user's uptime input (default 100 %).

**13. Per-attack CO behaviour** (E-11; ingested from the catalogue table of 04 §2.14, not hand-copied):

The pipeline parses each row of 04 §2.14 (`Weapon(s)`, `Attack`, `Behaviour`, `ρ_co`, `co_ref`, `B_co^abs`,
`Flags`), expands family rows ("Braton/Mk1/Prime/Vandal") to every listed weapon, maps wiki attack names to bundle
attacks, and writes `data/generated/spec/co_catalogue.toml`. Hand-written entries in `data/curated/co_overrides.toml`
are only for name-mapping failures, which the report lists. Generated form of rows 1 and 6:

```toml
[[co]]
row = 1                                    # 04 §2.14 row id
weapon = "…/SapientPrimaryWeapon"          # Acceltra
attack = "Normal Attack"
behaviour = "adding"
ratio = 0.7429
co_ref = "self"

[[co]]
row = 6
weapon = "…"                               # Ambassador
attack = "Alt-fire Hitscan Radial Attack"
behaviour = "adding"
ratio = 0.75
co_ref = "parent"
flags = ["direct_target_only"]
```

In the bundle this becomes `Attack::co = CoSpec { behaviour, ratio, co_ref: SelfAttack | Parent, fixed_abs:
Option<f64>, flags: CoFlags }`; attacks without a row get the default rule of 04 §2.5.

**14-18. More mechanics, one effect each** (as entries of an `effects` array):

```toml
effects = [
  { mult = "chamber", value = 1.00, when = { shot = "first_of_magazine" }, scope = { weapon = ["sniper"] } },  # Primed Chamber
  { stat = "co_per_cold_stack", value = 0.45, scope = { weapon = ["secondary", "archgun"] } },                 # Secondary Shiver (D-060)
  { stat = "tier_raise", value = 0.05, value_per = "set_piece" },                                              # Vigilante set bonus (sets.toml)
  { mult = "eclipse", value = { param = "value" } },        # Eclipse buff: group kind g_f, omitted from the Adding-CO term (D-058)
  { lock = "multishot_to_base", value = 1, when = { hit = "weak_point" } },                                    # Primary Acuity
]
```

### 3.7 Coverage: every bucket and mechanic of the spec has an encoding

| Spec bucket or mechanic | Encoding | Engine stage |
|---|---|---|
| additive base damage A (spec 3.1a) | `stat = base_dmg` (heavy-only melee damage: `scope.attacks = heavy`) | ST-4 |
| melee-projectile universal bucket | `stat = universal_dmg` | ST-4 |
| Incarnon flat X, `B_ref` exclusion | `stat = incarnon_flat`; engine builds `B'` and `B_ref` (E-1) | ST-1 |
| physical P_t, conversions | `stat = phys` (per type); `convert` | ST-2 |
| elemental E_t with slot order, exilus, riven bottom-up, innate/progenitor/Violet HCET, non-combining adds (Nourish, field elements) | `element` with `order` = `slot`, `post_mod_hcet` or `non_combining`; slot order comes from the build; riven entries keep their listed order | ST-2 (element combiner) |
| faction F incl. sub-factions, riven faction | `faction = [...]` resolved at L0 | ST-5, DoT second dip |
| separate multipliers G and CO-omitted G_F | `mult = <group>` from the registry (`kind`, `dot`, additive members) | ST-4, ST-5 |
| CO family C, adding vs multiplying per attack, `ρ_co`, stance input ratio | `co_per_status`, `co_per_cold_stack`, `co_flat` + `Attack::co` + `stance_co.toml` | ST-4 (E-11) |
| crit chance relative/absolute/floor/base add | `cc_rel`, `cc_abs`, `cc_floor`, `cc_base_add` | E-5 |
| crit damage relative/absolute/base add/field | `cd_rel`, `cd_abs`, `cd_base_add`, `cd_field_mul` | E-4 |
| Vigilante tier raise | `tier_raise` (per set piece) | E-6 |
| body part, weak point, headshot, Sonar | `wp_dmg`, `hs_mult`, `sonar_mult`; `when = { hit = … }` | E-7 |
| status chance rel/abs/mult, Tau | `sc_rel`, `sc_abs`, `sc_mul`; Tau from state | E-8 |
| forced procs, Hunter Munitions, Internal Bleeding, Archon Continuity/Vitality | `forced_proc`, `proc_on` | E-9 |
| multishot, Acuity lock | `multishot`, `lock = multishot_to_base` | E-10 |
| status damage (global and per status), duration, DoT vulnerability | `status_dmg`, `status_dmg_type`, `status_dur`, `dot_vuln` | E-13 |
| fire rate (bows ×2), Cannonade lock, charge, magazine, reload, ammo efficiency | `fire_rate` + `scope_factor`, `lock = fire_rate_to_base`, `charge_speed`, `magazine`, `reload_speed`, `ammo_eff` | 06 schedule |
| combo-scaled bonuses (Blood Rush, Weeping Wounds, Gladiator) | `value_per = combo_tier_above_one` (Gladiator also `set_piece`) | state var `ComboTier` |
| conditional stacks with max, gain, duration, timers, refresh, decay (04 §4.2) | `when = { trigger, chance, stacks = { max, gain, duration, stack_timer, refresh, decay } }` | T-02 compile table |
| per-attack and per-weapon-class applicability | `scope = { weapon, attacks, form, layer, exalted }`, `scope_factor` | L1 compile |
| extra hits (Xata, Toxic Lash, Resupply …) | `extra_hit = { element, part, status }` | E-14 |
| target debuffs (vulnerability, armor strip, external Viral, cap increases) | `vuln`, `strip`, `external_status`, `stack_cap` | ST-8, E-15 |
| set bonuses | `sets.toml` effects with `value_per = set_piece`; finalizer counts pieces (U38.5 active-weapon rule, `F:mods` §7.1) | L2 finalizer |
| "trigger all conditions" and its two other modes | T-02 compile table (section 3.5) | L1 compile |
| custom stats (26) | `custom_stats.toml`: one effect each (spec 6.3) | same path |
| one-off behaviours (Melee Duplicate, Debilitate, Nightwatch Napalm, Encumber, Cascadia Empowered) | `hook = <key>` with parameters; each hook is an engine function with its own test vector | engine |

---

## 4. The engine crate (`wfd-engine`)

### 4.1 Public API

```rust
pub struct Engine<'b> { bundle: &'b Bundle, assembly_cache: AssemblyCache }   // cheap to create; Sync

impl<'b> Engine<'b> {
    // Level 0: per scenario
    pub fn compile_scenario(&self, s: &Scenario, weapon: &WeaponRef) -> Result<CompiledScenario, EngineError>;
    // Level 1 + 2: a whole build (the UI path); the optimizer uses the lower-level Kernel below
    pub fn compile_build(&self, b: &Build, cs: &CompiledScenario) -> Result<CompiledBuild, EngineError>;
    // Level 3: one attack
    pub fn resolve(&self, cb: &CompiledBuild, cs: &CompiledScenario, a: AttackIdx) -> ResolvedAttack;
    // Level 4
    pub fn ev(&self, ra: &ResolvedAttack, cs: &CompiledScenario, req: &EvRequest, ws: &mut Workspace) -> EvReport;
    pub fn mc(&self, cb: &CompiledBuild, cs: &CompiledScenario, req: &McRequest) -> McReport;   // parallel runs with feature "rayon"
    // Convenience used by the server and the CLI
    pub fn calculate(&self, b: &Build, s: &Scenario, req: &CalcRequest) -> Result<CalcReport, EngineError>;
}

/// Optimizer-facing kernel: everything needed to score a StatVec without touching the DSL or the bundle again.
pub struct Kernel<'a> { cs: &'a CompiledScenario, attacks: SmallVec<[AttackStatic<'a>; 2]>, objective: &'a Objective }
impl<'a> Kernel<'a> {
    pub fn finalize(&self, sv: &mut StatVec, fin: &Finalizers);                 // sets, shards, locks
    pub fn score(&self, sv: &StatVec, el: &ElemVec, fid: Fidelity, ws: &mut Workspace) -> Score;
    pub fn bound(&self, sv_upper: &StatVec, el: &ElemVec, ws: &mut Workspace) -> Score; // ≥ score(any sv ≤ sv_upper, Exact)
}
pub enum Fidelity { Steady, WindowFast, WindowExact }
```

`EngineError` covers invalid builds (incompatible mod, wrong slot), missing data, and unsupported mechanics; it is
never a panic. All public types are plain data; `CalcReport` derives `Serialize` and `ts_rs::TS`.

### 4.2 Internal pipeline

```text
Scenario ──L0──► CompiledScenario { enemy: ScaledEnemy, faction_mod: [f64;18], caps, immune, parts, toggles, profile,
                                    state_vars: StateVarTable, mult_registry, condition_mode }
Item     ──L1──► CompiledItem { per_attack: SmallVec<[SparseContrib; 2]>, elem: ElemEntries, coeffs: SmallVec<[Coeff; 2]>,
                                hooks, set, incompat_mask, drain, touched: StatMask }
Build    ──L2──► CompiledBuild { sv: [StatVec; n_attacks], elem: ElemVec (combined), coeffs, hooks }  (Σ contributions + finalizers)
(CompiledBuild, attack) ──L3──► ResolvedAttack { base B'_t, B', B_ref, q_t (E-3), A, C-coefficients, ρ_co, behaviour,
                                F, G, G_F, Part, CC parts, CDM (NP-1), SC, w_s (E-9), φ_s, MS, FR/EFR, charge, burst,
                                magazine, reload, duty, DoT coefficients K_s·(1+E^dot_s)·(1+A_sd+A_sd,s), extra-hit fractions }
ResolvedAttack ──schedule──► Schedule { segments: SmallVec<[Segment; 16]> }   (06 ranged, 07 melee; deterministic, CD/4.1)
(ResolvedAttack, Schedule) ──L4──► EV (kernel at time points)  |  MC (event simulation)  ──► outputs (section 4.6)
```

Module layout:

```text
wfd-engine/src/
├── lib.rs, error.rs, profile.rs (Game / Compat flags per register row marked Compat)
├── stats.rs           StatVec (fixed point), SparseContrib, StatMask, Finalizers
├── compile/           scenario.rs, item.rs (DSL → contributions, T-02 table), scope.rs, sets.rs, shards.rs,
│                      riven.rs, buffs.rs, custom.rs, innate.rs
├── elements.rs        element combiner (E-2 elemental part) and ElementSolver (all distinct outcomes of a set, for the optimizer)
├── assemble.rs        kitgun, zaw, progenitor, exalted element (Chromatic Blade, Reactive Storm)
├── resolve/           base.rs (E-1), composition.rs (E-2), quantize.rs (E-3), scalar.rs (E-11), crit.rs (E-4..E-6),
│                      part.rs (E-7), status.rs (E-8, E-9), multishot.rs (E-10), firing.rs (06 stats), melee.rs (07 stats)
├── enemy/             scaling.rs (NP-2), armor.rs (E-15), layers.rs (E-12 enemy side, CD-08), dr.rs (innate, attenuation)
├── schedule/          ranged.rs, melee.rs, segment.rs
├── ev/                kernel.rs (one target-state point), poisson.rs, steady.rs, window_exact.rs, window_fast.rs,
│                      bound.rs, ttk.rs, heat.rs, distinct.rs
├── mc/                rng.rs (Philox4x32-10), events.rs, state.rs, dot.rs, sim.rs, summary.rs, timeline.rs
└── outputs.rs         CalcReport assembly, parity outputs (spec 4.5)
```

**Fixed-point stat vector.** `StatVec([i64; N])` in units of 1e-9. Curated values have at most 6 decimals; riven
values (f32 in the game, NP-1 note in spec 2.1) are rounded to 1e-9, which changes a bonus by at most 5e-10 absolute,
far below every tolerance of the spec. Conversion to f64 happens once at L3. Benefits: exact incremental updates,
order-independent sums (so a build's score does not depend on the path the optimizer took to reach it), and a stable
canonical hash for caching.

**Element combiner.** Input: the ordered element entries of the build (main slots 0-7, then the exilus slot per
OQ-024, riven entries in listed order with the bottom-up rule of `F:mods` §8.5), then innate, progenitor and
`post_mod_hcet` entries in HCET order (D-004, D-006), then non-combining additions. Output: `ElemVec` (bonus per final
element) and the "damage mix" key (sorted final element set) used by the optimizer's per-mix report. The
`ElementSolver` used by the optimizer returns, for an unordered set of element entries, every distinct achievable
`ElemVec` with one slot ordering that realizes it (at most 3 outcomes for 3-4 base elements), so element order is
searched exactly instead of heuristically (`I:opt` §9 item 5).

### 4.3 The EV evaluator (spec 4.2)

The **kernel** evaluates one target-state point: given the `ResolvedAttack`, the expected state at time τ (Poisson
means `μ_s(τ)` for every status the attack or externals can apply, combo tier, condition-stack expectations, shot
marks), and a view, it returns the expected direct damage per pellet, the expected DoT damage per proc of each DoT
status landing at τ, the expected extra-hit damage, and the expected procs per pellet. Inside the kernel:
`E[Vir(n)]`, `E[Mag(n)]`, `E[cold_cd(n)]`, `E[punc_cc(n)]` by a capped Poisson sum (one `exp` plus a recurrence);
`E[1 − DR(A_eff)]` over the joint distribution of Corrosive stacks and Heat presence (and the Heat strip ramp
state); `E[n]` as `Σ_s (1 − e^{−μ_s})` + externals; crit linear in CC (CD-13); Adding/Multiplying CO per E-11 with the
sequential-pellet correction of spec 4.2; extra hits per E-14. Only statuses with non-zero rate or external stacks
are visited (a `StatusMask` from L3), which is why typical builds cost less than the prototype.

| Fidelity | Points | Definition | Exactness |
|---|---|---|---|
| `Steady` | 1 (+ the Heat window mean) | `μ_s = λ_s·D_s`, combo/sniper/spool/ramp at steady values; `dps_burst`, and `dps_sustained` via the duty ρ (spec 4.1) | exact for the spec's steady definitions |
| `WindowExact` | every shot and every DoT tick landing time in `[0, t_win]` | the spec's window walk: `μ_s(τ)` = expected procs of earlier shots still alive at τ, computed with a two-pointer sliding window per status (O(1) amortized per shot), DoT ticks evaluated at their landing times (CD-05) | the definition of `dps_window` |
| `WindowFast` | Gauss-Legendre, 3 nodes per smooth piece | segments of evenly spaced shots are replaced by their shot density; `μ_s(τ)` is then piecewise linear with breakpoints at segment ends and at segment ends + `D_s`, and at Heat-ramp steps; between breakpoints the integrand is smooth, so 3 nodes per piece are enough. Used only when the window has more than 64 shots; otherwise it *is* `WindowExact` | tested: ≤ 0.5 % relative to `WindowExact` on the golden corpus and on 10,000 random builds (CI test); the optimizer widens its keep-set by this ε |
| `Bound` | 8 intervals | per interval `[a, b)`: exact shot count from the schedule × kernel evaluated at an upper-envelope state (`μ_s ≤ p_s × shots in (a − D_s, b]`, combo and condition stacks at their maxima over the interval) + an upper bound of DoT ticks landing in the interval | provable `Bound ≥ WindowExact` whenever the kernel is monotone non-decreasing in every `μ_s` (true unless a "fewer than N statuses" predicate such as Spiteful Defilement is active; then that status's μ is bounded on both sides and the kernel is evaluated at both ends). Property-tested. |

**EV TTK** walks the `layered` view (OG → SH → HP, Toxin routing, CD-08 overflow, no shield gate) using the same
window machinery and solves for the first time the expected cumulative damage reaches the pools (bisection over the
piecewise integral); beyond `t_ttk_max` it extrapolates with `dps_sustained` and flags it (spec 4.2). For the
optimizer's TTK metric, `Bound` yields a valid lower bound of TTK (upper bound of damage).

**Hit metrics** (`hit_infight`, `hit_fresh`, the inspiration's "maximum hit" metrics): expected damage per hit at the
steady state and at the fresh state, one kernel point each.

### 4.4 The Monte Carlo simulator (spec 4.3)

- **RNG**: Philox4x32-10 implemented in `mc/rng.rs` (about 60 lines, no dependency) and checked against the Random123
  known-answer vectors. Key = `(seed, run)`, counter = `(attack, event index, mechanic id, draw)`, exactly the keying
  of U-6. Draws are therefore independent of thread count and of other mechanics, and two builds see the same draws
  per mechanic (common random numbers for optimizer confirmation). Chances are exact fractions (D-017).
- **Events**: a binary heap ordered by `(time, CD-04 phase, creation sequence)` holding shot, pellet, radial hit,
  extra hit, proc application, stack expiry, DoT tick (one shared Heat tick per second), Blast fuse and detonation,
  Heat strip step, reload start/end, combo gain/expiry, condition stack gain/expiry, sniper combo decay, beam ramp,
  Magnetic break proc.
- **Views in one pass**: the status state is identical in the four non-depleting views (spec 4.1), so each damage
  instance is evaluated for `health`, `armor`, `shield` and `overguard` at once and accumulated into four sinks. This
  is 4× fewer events than simulating each layer separately (the inspiration simulates every layer, `I:agg` §7.1). The
  `layered` pass (depletion, shield gate, overflow, kill stops extra hits) reuses the same draws.
- **State**: per-status ring buffers of expiry times (`ArrayVec<f64, 16>`, cap-aware, CD-07), DoT instances in a slab
  with snapshotted weapon-side seeds (CD-05), Heat shared timer and pool, Blast fuse list.
- **Parallelism**: runs are independent; with the `rayon` feature they are split across threads and results are
  written into a `Vec` indexed by run, then reduced in run order (bit-identical for any thread count).
- **Summary**: mean, standard deviation, 95 % CI of the mean, median, p5/p25/p75/p95 of `dps_window_mc` per view;
  `dps_series` with p10/p90 bands; per-type and per-source breakdown; status uptime and mean stacks per bin; crit-tier
  histogram; TTK distribution, kill probability, shots and ammo to kill; pending DoT; and, on request, the per-hit
  timeline of one run (the inspiration's "More details").

### 4.5 Consistency checks inside the engine

`EvReport` carries `dps_window`; `McReport` carries `dps_window_mc`. `CalcReport` sets a deviation flag when the EV
value lies outside the MC 99 % CI widened by 2 % (spec 4.4). Golden scenarios assert the flag is off unless the
scenario is listed as a known EV-approximation class by `08`.

### 4.6 Outputs (`CalcReport`)

```rust
pub struct CalcReport {
    pub data_version: String,
    pub code: String,                       // share string of the inputs (filled by the server, section 7.3)
    pub weapon_panel: WeaponPanel,          // resolved stats ST-1..ST-7 per attack, base vs modded, per-type damage q_t·Y·(1+F)
    pub enemy_panel: EnemyPanel,            // scaled pools, armor and DR, innate DR, immunities, unique-DR notes
    pub attacks: Vec<AttackReport>,         // per attack: per view { ev: EvOut, mc: Option<McOut>, parity: Option<ParityOut> }
    pub groups: Vec<GroupReport>,           // parent + child sums, combined timelines (T-22)
    pub breakdown: Option<Breakdown>,       // per damage type, per source (direct, DoT by status, extra hits, special)
    pub inspector: Option<FormulaTrace>,    // every bucket value with the items that contributed (UI formula inspector)
    pub timeline: Option<Timeline>,         // one MC run, per-hit records
    pub warnings: Vec<Message>,             // { code, params } — translated by the frontend
}
pub struct EvOut { pub burst: f64, pub sustained: f64, pub window: f64, pub ttk: Option<TtkEv>, pub per_shot: f64,
                   pub per_magazine: f64, pub stacks: SmallVec<[StackStat; 6]>, pub distinct: f64, pub crit_tiers: [f64; 4] }
pub struct ParityOut { pub queue_dps: f64, pub avg_direct: f64, pub avg_status: f64, pub avg_total: f64,
                       pub median_total: f64, pub ttk: Option<f64>, pub min: f64, pub max: f64 }   // spec 4.5
```

`FormulaTrace` is cheap because L1 contributions are per item: for every slot of the `StatVec` the engine can list
which items contributed what, and for every stage the intermediate value. This is the data behind the UI's formula
inspector (section 7.5) and behind the spec-vector probes below.

### 4.7 Spec test vectors become unit tests

The topic documents write their test vectors in Markdown, each under a stable id (`TV-02-01` … in 02 §9: tables and
prose, computed with `python3`). They become tests in three steps, with the spec remaining the authority:

1. **Machine form.** Each vector is transcribed once into `docs/mechanics/vectors/<doc>.toml`, keyed by its id (a
   topic document may instead embed a fenced `vector` block in the same TOML form; then no transcription is needed).
   Inputs use a small scenario language over the mini bundle or inline definitions; expectations are probe paths.
2. **Transcription check** (`wfd-engine/tests/spec_conformance.rs`): every `TV-nn-mm` id in the Markdown has a
   machine entry and vice versa, and every expected number in the TOML occurs verbatim in the Markdown text of that
   vector's section. A typo in the transcription fails CI.
3. **Execution** (`wfd-engine/tests/spec_vectors.rs`): build the inputs, run the engine with a `Probe` collector (a
   debug struct filled by every stage when tracing is on), compare each probe path with the document's stated
   tolerance (02 §9: f32 values bit-exact, f64 within 1e-9 relative). An unknown probe path is a test failure, so a
   topic document can force the engine to expose an intermediate value.

```toml
# docs/mechanics/vectors/02.toml
[vector."TV-02-02"]
source = "02 §9 TV-02-02"
input = { cc_base = 0.30, cc_rel = 1.5, cc_abs = 0.45, cd_from = "TV-02-01 row 1", headcrit = 1 }
tolerance = { rel = 1e-9 }

[vector."TV-02-02".expect]
"crit.cc" = 1.2
"crit.p_tier.1" = 0.8
"crit.e_k" = 1.2
"crit.c.1" = 4.401074743270875
"crit.c.2" = 7.802149486541749
"crit.e_c" = 5.08128969192505
```

The worked example of 00 §3.8 becomes `TV-00-01` with probes at every step (`enemy.hp` = 73,621.375, `enemy.armor`
= 1355, `resolve.q.COR` = 1.8125, `resolve.cdm` = 4.4010749, `kernel.fresh.armor.noncrit_total` = 195.387510,
`ev.fresh.armor.per_pellet` = 394.7458, …). Spec tables that the engine transcribes as constants (03 §4.1 status
summary, 05 §5.1 faction table, 05 §6.1 scaling coefficients) get a conformance test that parses the Markdown table
and compares it with the Rust constants. Mechanics without a numeric vector get ordinary `#[test]`s named after their
rule (`e11_adding_omits_gf`, `cd04_sibling_pellets_no_viral`).

### 4.8 Numeric types and determinism

- `f64` everywhere (U-4); fixed-point `i64` only for bucket sums; explicit `f32` arithmetic at NP-1 (crit-multiplier
  quantization) and NP-2 (enemy scaling blend `f1 + (f2 − f1)·s`), each in one function with its own test.
- `libm` for `exp`, `ln`, `pow` so EV and MC are bit-identical across Linux, macOS and Windows; `sqrt` is IEEE-exact
  in std. Rust does not contract `a*b + c` into FMA, so operation order is what the code says.
- No hash-map iteration order reaches results (`BTreeMap` or sorted `Vec` in reports).
- Rounding for display only at the UI (`floor(x + 0.5)`, U-5); the engine never rounds except where a rule says so.
- Determinism tests: `calculate` twice, with 1 and N threads, must give byte-identical serialized reports.

### 4.9 Performance work in the engine

- `ResolvedAttack`, `PointState` and `Workspace` are fixed-size `Copy`/`#[repr(C)]` structs; no heap in L3/L4.
- The kernel iterates statuses through a `StatusMask` of active ones and caches `e^{−μ}` per status per point.
- Schedules are run-length encoded: the generator of 06 (E-6.11) defines per-shot records (fire and hit time,
  magazine index, `first`/`last`, burst position, spool index, ramp level); `Schedule` stores maximal runs of shots
  with constant spacing and equal fields other than time, so a 20 s rifle window is about 3-10 segments (spool, ramp
  and burst phases form short runs). `WindowExact` still visits every shot; `WindowFast` and `Bound` work per segment.
- Benchmarks (`criterion`) in `wfd-engine/benches/{resolve,steady,window,bound,mc}.rs` on fixed fixtures; CI runs
  them in quick mode and fails when a median regresses by more than 25 % against `benches/baseline.json`, or when an
  absolute target of section 0.4 is missed by more than 2× (CI machines vary).

---

## 5. The optimizer (`wfd-opt`)

### 5.1 Request, problem, result

```rust
pub struct OptRequest {
    pub start: Build,                    // current page build: locks, riven, weapon variant, preferences
    pub scenario: Scenario,              // enemy, toggles (T-02 mode, T-03 hit location …), custom stats
    pub objective: Objective,
    pub constraints: Constraints,
    pub groups: GroupSwitches,           // which slot groups the search may change
    pub buffs: BuffPolicy,               // allowed external buffs with values; portfolio on/off
    pub search: SearchParams,
}
pub struct Objective {
    pub metric: Metric,                  // DpsWindow (default) | DpsBurst | DpsSustained | Ttk | HitInFight | HitFresh
    pub view: ViewSel,                   // Auto (armor if armored else health; inspiration parity) | a View
    pub attacks: SmallVec<[(AttackIdx, f64); 2]>,   // weighted multi-attack objective (e.g. normal 0.7 + heavy 0.3)
    pub window: f64,                     // defaults to T-01; one window for search and final ranking (fixes I:opt Q10)
}
pub struct Constraints {
    pub locked: SlotMask,                // per slot, not all-or-nothing (I:opt §8)
    pub must_include: SmallVec<[ItemId; 4]>,
    pub excluded: Vec<ItemId>,           // the "prohibited mods" list; persisted by the frontend
    pub elements: ElemRule,              // require / forbid final damage types (superset of "exclude elements")
    pub rules: RuleFlags,                // NO_FACTION, NO_ON_KILL, KEEP_PROGENITOR, USE_INCARNON, ... (inspiration parity)
    pub capacity: Option<CapacityRule>,  // { capacity: 30 | 60, polarities: Current | AllowForma(n) | Free, ranks: Max | AllowLower }
    pub riven: RivenPolicy,              // UseGiven | None | Search { traits: 2..=4, allow_negative: bool }
}
pub struct GroupSwitches { pub exilus: bool, pub weapon_arcanes: bool, pub stance: bool, pub incarnon: bool,
                           pub frame_mods: u8, pub aura: bool, pub frame_arcanes: bool, pub shards: ShardPolicy,
                           pub modular_parts: bool, pub progenitor: bool }
pub struct SearchParams { pub mode: Mode /* Exact | Balanced | Quick */, pub time_limit: Option<Duration>,
                          pub node_budget: Option<u64> /* deterministic budget */, pub keep: u16 /* finalists, default 32 */,
                          pub mc_confirm: Option<McConfirm> /* default: 12 finalists × 1,000 runs, seed 0 */,
                          pub threads: Option<usize> /* default: all cores − 1 */ }

pub struct OptResult {
    pub best: RankedBuild, pub alternatives: Vec<RankedBuild>,
    pub per_mix: Vec<RankedBuild>,       // best build per final damage-type mix (inspiration phase-3 table, but for full builds)
    pub proof: Proof,                    // ProvenOptimal { scope } | Gap { upper_bound, relative } | Heuristic
    pub leave_one_out: Vec<SlotValue>,   // score without each slot's item
    pub next_best: Vec<SlotAlternative>, // best replacement per slot and its cost
    pub condition_sensitivity: Option<f64>, // best build re-scored with T-02 = none
    pub buff_table: Option<BuffTable>,   // value of each allowed buff on this build, and whether rebuilding for it pays
    pub stats: SearchStats,              // evaluations, bounds, prunes, leaves, per-phase time
}
pub struct RankedBuild { pub build: Build, pub code: String, pub ev: f64 /* WindowExact */, pub mc: Option<McSummary>,
                         pub ev_mc_deviation: bool, pub mix: MixKey }
```

### 5.2 Candidate pools

Per slot group, computed once per run (`pool.rs`):

1. **Eligibility**: slot kind, weapon categories and compat tags, forbidding tags, Semi-only trigger (`I:model` §2.4),
   rank and capacity feasibility, user exclusions, `must_include`. Incompatibility is the union of both directions of
   the wiki `Incompatible` lists plus the hidden-parent rule (D-093), compiled into a `u64` group mask per item; no
   substring tests (`I:opt` Q6).
2. **Rule filters** act on effects, not whole items: "no faction bonus" zeroes `faction` targets, "no on-kill" zeroes
   effects whose trigger is a kill, headshot off makes weak-point-conditional effects zero via T-03. Items whose every
   effect is then zero drop out. Mods that only extend status duration are *not* excluded (they matter to
   `dps_window`; `I:opt` Q9).
3. **Relevance**: an item whose L1 contribution to every objective attack is zero, with no hooks and no element, is
   dropped (recoil, zoom, punch-through …), derived from the engine rather than from a hand-written list (`I:opt` §3).
4. **Dominance**: within the monotone part (section 5.4), item B is dropped if some item A has `contrib(A) ≥
   contrib(B)` component-wise on every `Inc` slot, equal `Unknown` slots, no larger drain under the capacity rule,
   and an incompatibility mask that is a subset of B's (so A is always available wherever B is). Example: an
   Amalgam/normal pair with identical damage stats but extra non-damage stats collapses to one.

### 5.3 Decomposition

A complete build is `(context, main)`. **Context** = everything outside the 8 main slots: exilus, weapon arcane(s),
stance and combo, Incarnon perk per tier, frame mods, aura, frame arcanes, shard multiset, kitgun/zaw parts,
progenitor element, buff scenario. **Main** = the 8 main slots (with the riven as one candidate).

Main-slot candidates split into two classes by the stats they touch:

- **Composition candidates**: anything that touches an `Unknown` stat: elements, physical mods, conversions, forced
  procs, hooks, and any candidate made `Unknown` by a scenario predicate (for example CC stats when Prelude of Might is
  selected). They change the damage-type mix and the proc weights, which can raise or lower the score.
- **Monotone candidates**: everything else (base damage, multishot, crit chance and damage, status chance, fire rate,
  magazine, reload, faction, status damage and duration, …). For these, more is never worse (section 5.4).

The search enumerates **composition sets** (subsets of composition candidates of size 0..8, incompatibilities
respected, deduplicated by their resulting `ElemVec`/physical vector) and, for each, solves the monotone remainder by
branch-and-bound. The element order inside a set is solved exactly by the `ElementSolver` (section 4.2): each set
expands to its ≤ 3 distinct element outcomes.

### 5.4 Monotonicity: why the bound is valid

The bound needs one property: for a fixed context and composition, the scored metric is non-decreasing in every
`Inc` slot of the `StatVec`. Each slot's class is declared in `define_stats!` (section 3.5) with a one-line
justification that cites the formula (for example: `fire_rate` is `Inc` because a faster schedule fires every shot no
later, so shots and DoT ticks landing in `[0, t_win]` cannot decrease, `06`; `cc_rel` is `Inc` because `E[c] = 1 +
E[k]·(h·CDM − 1)` with `h·CDM ≥ 1`, E-4..E-6). The scenario compiler downgrades a slot to `Unknown` when an active
effect has a predicate that reads it (`AttackPred::ModdedCcBelow` → CC slots; `TargetPred::DistinctBelow` → status
chance slots and the kernel's both-ends rule). The claim is enforced three ways:

1. **Property tests** (`proptest`): random weapon, scenario, build and a random increase of one `Inc` slot → the score
   must not decrease (all fidelities). Failing seeds are kept as regression fixtures.
2. **Bound tests**: random partial build + random completion → `bound(relaxed) ≥ WindowExact(completion)`.
3. **Exactness tests**: on small pools (≤ 18 candidates) B&B must return the same argmax as brute force.

If a mechanic is ever found to break monotonicity, the fix is a one-word change of its class to `Unknown`; the search
then enumerates instead of bounding that stat. Correctness degrades to "slower", never to "wrong".

### 5.5 Main-slot branch-and-bound

```text
solve_main(context, comp_set, incumbent θ):
    S0     = Σ contrib(context) + Σ contrib(comp_set)                    // fixed part, exact i64
    k      = 8 − |comp_set| − |locked main slots|
    pool   = monotone candidates, sorted by marginal value at S0 (descending), compacted to the active slots A
    TOP[i][r][a] = sum of the r largest positive values of slot a among pool[i..]   // suffix top-r table, r ≤ 8
    DFS(i, depth, S, mask, drain):
        if depth == k: leaf(S); return
        if Kernel::bound(S + TOP[i][k − depth]) < θ: return                          // prune: no completion reaches θ
        if capacity: if drain + MINDRAIN[i][k − depth] > cap: return
        for j in i .. len(pool) − (k − depth) + 1:
            if pool[j].incompat & mask != 0: continue
            DFS(j + 1, depth + 1, S + contrib(pool[j]), mask | pool[j].group, drain + pool[j].drain)
    leaf(S): if bound(S) < θ: return
             s = score(S, WindowFast); s_best = max(s_best, s); keep (s, build) if s ≥ θ     // θ = (1 − 4ε)·s_best
```

Details that make it fast:

- **Active-slot compaction.** Only the 10-30 `StatVec` slots that any pool candidate touches are carried in the DFS
  (`[i64; 32]`, one cache line pair); `S + contrib` is a fixed-width vector add the compiler vectorizes.
- **Suffix top-r table** makes the relaxation O(active slots) per node; it ignores incompatibilities, which only
  loosens the bound (still valid).
- **Ordering**: composition sets are processed in descending order of their root bound, and children in descending
  marginal value, so strong incumbents appear early; the first incumbent comes from a greedy completion of the best
  set, or from the previous context's solution (warm start).
- **Threshold and keep-set**: let `s_best` be the best `WindowFast` leaf score found so far and ε = 0.5 % the
  tested bound on `|WindowFast − WindowExact| / WindowExact`. Nodes are pruned when `Bound < θ = (1 − 4ε)·s_best`,
  and leaves with `WindowFast ≥ θ` enter the keep-set. Because `Bound ≥ WindowExact` and `s_best ≤ (1 + ε)·O`
  (O = the exact optimum), every build whose exact score is within 2ε of O survives pruning and is kept; a
  fast-path error can therefore never remove the true best build (P5). For metrics where the search score is already
  exact (`Steady`, windows with ≤ 64 shots) ε = 0 and the rule reduces to plain B&B.
- **Parallelism**: `rayon` parallel iteration over (composition set × outcome × first two DFS levels) work items;
  `s_best` is an `AtomicU64` holding the f64 bits of the best leaf score (scores are positive, so integer order
  equals numeric order; updated with `fetch_max`), and every worker derives θ from it; keep-sets are per-thread and
  merged at the end, then filtered with the final θ and sorted by `(score desc, canonical key asc)`. Pruning only
  removes builds below the final θ, so the returned keep-set and optimum do not depend on thread timing.
- **Best per damage mix**: the global θ prunes whole mixes that cannot compete, so the per-mix table is a second,
  cheaper pass: for the 15 mixes with the highest root bounds, the inner B&B reruns with a mix-local threshold,
  warm-started from the first pass.
- **Memoization**: a per-thread `FxHashMap<u128, f32>` keyed by the xxh3-128 hash of the canonical build (sorted ids
  + element outcome) for local search; scoped to one `CompiledScenario`, so it can never serve a stale score
  (`I:opt` Q12).
- **Capacity and polarity** (optional): drain is a knapsack constraint pruned with the suffix minimum-drain table;
  at a leaf the polarity assignment (match halves drain, mismatch ×1.25, `F:mods` §2) is an assignment problem over ≤
  8 slots solved by DP over a 256-state bitmask; with `AllowForma(n)` the DP also chooses up to n slot polarities.
  Under-ranked versions of a mod (`ranks = AllowLower`) are extra candidates in one incompatibility group.

**Cost.** A rifle with 14 composition and 22 monotone candidates has `C(36, 8) = 30.3 M` complete main-slot fillings;
at the `Bound` target of 1.5 µs that is 45 core-seconds with no pruning at all, i.e. under 10 s on 8 cores even in
the worst case. With pruning the expected cost is one to two orders of magnitude lower. A melee weapon with 16 + 40
candidates has `C(56, 8) = 1.42 G` fillings and relies on pruning; the time limit turns it into an anytime search
with a reported gap (section 5.9). These counts are estimates; tasks O1 and P1 measure them and record the numbers
in `docs/architecture/`.

### 5.6 Context search

Context groups are small and few, so they are searched around an exact main solver:

| Group | Options | Strategy |
|---|---|---|
| weapon arcane (archgun: 2) | ~5-15 | enumerate |
| exilus | ~5-20 | enumerate (element exilus mods join the composition set at the exilus position) |
| stance × combo | ~2-5 stances × 2-6 combos | enumerate (fixes "first combo only", `I:opt` §8) |
| Incarnon perks | ≤ 3 per tier, ≤ 4 tiers → ≤ 81 | enumerate jointly |
| frame arcanes | pairs of ~10 → ≤ 45 | enumerate pairs |
| aura | ~5-10 | enumerate |
| frame mods | up to 4 from ~10 | small B&B with the same machinery |
| archon shards | multisets of 5 from the shard types the user owns (14 types → 8,568 multisets) | enumerate; each multiset is one precompiled vector (the shard finalizer resolves Violet scaling) |
| kitgun grip × loader, zaw grip × link | ≤ 100 | enumerate (the inspiration cannot search these) |
| progenitor element | 7 | joins the composition enumeration (value fixed by the user, 25-60 %) |
| buff scenario | allowed buffs | portfolio: every scenario solved in parallel (section 5.8) |

Algorithm (`context.rs`):

1. **Seed**: the user's current context (locked groups stay fixed); empty groups get the option with the best
   marginal value on a greedy main build.
2. **Block-coordinate ascent**: for each group in turn, score every option with the current main fixed
   (`WindowFast`, thousands per second per core); for the top `m = 3` options, re-solve main exactly with B&B
   (warm-started from the current main, so pruning is strong); accept the best. Repeat sweeps until no group changes.
3. **Context beam**: keep the `B = 4` best distinct contexts after each sweep and continue from each (avoids the
   order dependence of a single ascent path, `I:opt` §9 item 8).
4. **Cross-group local search**: from the best `B` full builds, apply 1-swaps and 2-swaps across any two slots of any
   groups (including main ↔ exilus moves and element re-orderings), with exact fixed-point deltas and `WindowFast`
   scoring, until no move improves by more than 1e-9 relative. All moves are deterministic (no random restarts unless
   the user asks; then seeded).

**Proof scope.** `ProvenOptimal { scope: MainGivenContext }` is reported when B&B finished for the final context;
the context itself is proven optimal only for groups that were fully enumerated jointly. Otherwise the result reports
the bound gap of the unfinished B&B or `Heuristic`.

### 5.7 Scoring, finalists and Monte Carlo confirmation

1. Search scores: `Bound` for pruning, `WindowFast` (= `WindowExact` for windows with ≤ 64 shots) for leaves,
   `Steady` for burst/sustained metrics, EV TTK for the TTK metric.
2. **Finalists**: the merged keep-set (every build above the final θ, plus the per-mix bests of the second pass)
   is re-scored with `WindowExact`, and the top `keep` (default 32) by `WindowExact` are the finalists. This is the
   ranking the user sees. If the keep-set grows beyond 100,000 builds (a flat optimum), θ is raised to the
   100,000-th score and the result is marked `Heuristic` for the tail only.
3. **Monte Carlo confirmation**: the top 12 finalists run 1,000 MC runs each with the same seed (common random
   numbers, U-6), in parallel. The report shows mean ± 95 % CI per finalist and flags any finalist whose EV lies
   outside its MC 99 % CI + 2 % (spec 4.4). MC does not reorder builds by default (EV is the noise-free objective);
   the user can choose "rank by MC mean" in the UI. For TTK, MC also reports kill probability and the TTK
   distribution, which EV cannot give.

### 5.8 Constraints and special inputs

| Input | Handling |
|---|---|
| locked slots | per slot; locked items are part of the fixed sum `S0` or of the fixed context |
| incompatibilities | group masks (section 5.2) |
| capacity / polarity / forma / ranks | optional `CapacityRule` (section 5.5); off by default (inspiration parity, D-092) |
| allowed buffs | `BuffPolicy { allowed: [(buff, value)], portfolio: bool }`: every subset of interest ("none", each alone, all) is a separate scenario, solved in parallel; the result table gives the buff's value on the same build and the gain of rebuilding for it, and the user picks the scenario (fixes `I:opt` §9 item 13) |
| rivens | `UseGiven`: the user's riven is one main candidate (composition or monotone by its traits). `Search`: an outer loop over trait combinations of the weapon class with values at the maximum of the value formula (`F:mods` §8.3, disposition 1.2), pruned by the same bound; reports the best riven per trait set |
| Incarnon tiers | one perk per tier, tier I is the form; perks apply to the attacks in their `scope.form` |
| exilus, arcanes | their own groups; archguns get two arcane slots with their class rules (`F:mods` §11) |
| T-02 condition mode | part of the scenario; the optimizer reports the best build's score with conditions `none` as sensitivity |
| element rules | `require` / `forbid` final damage types, checked on composition outcomes before the inner B&B |
| multi-attack objective | the kernel scores each objective attack from its own `StatVec` and sums the weighted metrics; the monotone classes are the union |

### 5.9 Progress reporting and cancellation

- The job owns an `Arc<JobControl { cancel: AtomicBool, progress: watch::Sender<Progress> }>`. Workers check
  `cancel` every 1,024 DFS nodes or 64 evaluations (≤ 100 ms latency at the targets of section 0.4).
- `Progress { phase, phase_index, phases_total, evaluated, bounded, pruned, rate_per_s, fraction: Option<f64>, eta_s:
  Option<f64>, incumbent: Option<IncumbentSummary>, gap: Option<f64>, messages: Vec<Message> }`. `fraction` is exact
  in the B&B phase (work items done / total, weighted by subtree size estimates) and absent in local search.
  Messages are `{ code, params }` (for example `opt.phase.main_bnb.start { sets: 2154 }`), translated by the frontend.
- **Anytime**: on cancel or time limit the current incumbent and keep-set go through steps 2-3 of section 5.7 (fast:
  finalists only) and the result carries `Proof::Gap` or `Heuristic`. Cancel never loses the best build found.
- **Deterministic budget**: `node_budget` replaces the wall-clock limit when reproducibility matters (tests, shared
  results).

### 5.10 How this beats the inspiration's optimizer (`I:opt` §8-§10)

| Inspiration weakness | This design |
|---|---|
| Full stochastic simulator as score, 3 seeds, a few thousand evaluations (§9.1, §9.9) | deterministic EV kernel, ~10^5-10^6 evaluations per second per core |
| Biased fast estimate, winner's curse, a/101 bias, 80 % trust tolerance (§9.2-§9.3, Q1-Q2) | no noise in the search; MC only confirms, with CRN and CIs; EV-vs-MC deviation flagged per build |
| Elements chosen on a bare weapon (§9.4) | composition sets are solved jointly with every other main mod (exact inner solve per set) |
| Element order searched only in phase 3 (§9.5) | `ElementSolver` enumerates every distinct outcome for every set |
| Beam pruning by marginal value; greedy perks/stance/frame slots (§9.6-§9.7) | exact B&B for main slots; joint enumeration for perks, stance × combo, shards; coordinate ascent with a context beam elsewhere |
| Order dependence (§9.8) | context beam + cross-group local search to a fixed point; determinism tests |
| TTK quantisation hack (§9.10, Q7) | EV TTK is continuous; MC gives the distribution |
| Single-threaded UI-thread JS (§9.11) | all cores via rayon; UI stays responsive (separate process) |
| Regex exclusions on tooltip text, hard-coded lists (§9.12, Q9) | rules act on typed effects; relevance derived from the engine |
| Buff phase decides under "all allowed" only (§9.13) | buff portfolio with a per-scenario table and user choice |
| No capacity, polarity, rank, riven search, per-slot locks, kitgun/zaw parts, multi-attack objectives (§8) | all available as options |
| Cache key omits context (Q12); window mismatch between search and final (Q10) | caches scoped to a compiled scenario; one window for search and ranking |

**Parity mode.** For the accuracy audit, `RuleFlags` reproduce the inspiration's defaults (no-faction on,
conditions met on, status-duration-only mods excluded, its prohibited-mods defaults, one frame-mod slot) and the
metrics map to `hit_infight`/`hit_fresh`/DPS/TTK. The acceptance test is: on the oracle weapon set, our best build
scored by our own `WindowExact` is never worse than the inspiration's best build scored the same way.

---

## 6. The server (`wfd-server`, binary `warframe-dps`)

### 6.1 Routes

All JSON; request/response types live in `wfd-model::api` (or the crate that owns them) with `#[derive(Serialize,
Deserialize, TS)]`, exported to `frontend/src/api/generated/`. Errors are `{ "error": { "code": "...", "params": {...} } }`
with a matching HTTP status; codes are i18n keys.

| Method | Path | Request | Response |
|---|---|---|---|
| GET | `/api/meta` | — | `Meta { app_version, schema_version, data_version, game_baseline, sources, licences, coverage }` |
| GET | `/api/weapons?q=&slot=&class=&limit=` | — | `Vec<WeaponSummary { key, name, slot, class, traits, image }>` |
| GET | `/api/weapons/{key}` | — | `WeaponDetail { attacks, groups, incarnon tiers + perks, modular parts, progenitor elements, compatible stances, polarities, base stats }` |
| GET | `/api/items?slot=&weapon=&q=&stat=` | — | `Vec<ItemSummary { key, name, kind, polarity, drain, max_rank, rarity, image, description, modelled, incompat }>`; `stat=` searches by effect target (the inspiration's "search by stat") |
| GET | `/api/enemies?q=` (name or faction) | — | `Vec<EnemySummary>` |
| GET | `/api/enemies/{key}` | — | `EnemyDetail { faction, base_level, pools, parts, immunities, caps, flags, special_dr }` |
| GET | `/api/buffs`, `/api/custom-stats`, `/api/toggles` | — | definitions with parameters, ranges and defaults (the UI renders them from data) |
| POST | `/api/calc` | `CalcRequest { build, scenario, want: { ev, mc: Option<{ runs, seed }>, parity, breakdown, inspector, timeline: Option<{ attack, view, run }> } }` | `CalcReport` (section 4.6) |
| POST | `/api/build/encode` | `{ build, scenario }` | `{ code }` |
| POST | `/api/build/decode` | `{ code }` or `{ url }` (ours or the inspiration's) | `{ build, scenario, warnings: Vec<Message>, unmapped: Vec<String> }` |
| POST | `/api/optimize` | `OptRequest` (section 5.1) | `202 { job_id }`; `409` if a job is running (one job at a time, it uses all cores) |
| GET | `/api/optimize/{id}/events` | — | SSE stream: `progress`, `incumbent`, `message`, `done` (`OptResult`), `error` |
| GET | `/api/optimize/{id}` | — | `{ state, last_progress, result? }` (for reconnects) |
| DELETE | `/api/optimize/{id}` | — | `202`; the job finishes with its anytime result |
| POST | `/api/data/update` | `{ refresh_sources: bool }` | `202 { job_id }`; events at `/api/data/update/{id}/events` |
| GET | `/api/img/{image}` | — | image bytes from the on-disk cache or fetched once from the WFCD image host |
| GET | `/*` | — | embedded frontend; unknown paths fall back to `index.html` |

### 6.2 Server-sent events, not WebSocket

Optimizer progress is one-directional; cancel is a `DELETE`. SSE runs over plain HTTP, reconnects automatically,
and is first-class in Axum (`axum::response::sse::{Sse, Event, KeepAlive}`). Progress comes from a
`tokio::sync::watch` channel, so a reconnecting client immediately gets the latest snapshot; events are throttled to
10 per second; keep-alive every 15 s. A WebSocket would add a protocol and a reconnect state machine for no gain.

### 6.3 Execution model

```rust
struct AppState {
    bundle: ArcSwap<Bundle>,                 // hot-swappable after `data update`
    engine_pool: rayon::ThreadPool,          // MC runs for /api/calc (all cores; jobs are short)
    opt_pool: rayon::ThreadPool,             // optimizer (cores − 1 by default; config `optimizer_threads`)
    jobs: Mutex<HashMap<JobId, JobHandle>>,  // JobHandle { control: Arc<JobControl>, rx: watch::Receiver<Progress>, result: OnceLock<OptResult> }
    config: Config, http: ureq::Agent,       // http only for images and data update
}
```

`/api/calc` runs on `tokio::task::spawn_blocking` and installs `engine_pool` for its MC runs. A typical EV-only
recalculation takes well under a millisecond; with 1,000 MC runs it targets ≤ 100 ms on 8 cores. The frontend
cancels superseded requests with `AbortController`, so stale work is short-lived. Optimizer jobs run on `opt_pool`
in a dedicated OS thread; results are kept for one hour.

### 6.4 Static frontend, browser, single instance

- `#[derive(RustEmbed)] #[folder = "$CARGO_MANIFEST_DIR/../../frontend/dist"]` with the `compression` feature; hashed
  assets get `Cache-Control: immutable`, `index.html` gets `no-cache`; MIME types from `mime_guess`.
- Bind `127.0.0.1:<port>` (default 7878, then the next free port). On success print the URL and call
  `webbrowser::open` unless `--no-browser`. If the port answers `/api/meta` with our app id, another instance is
  already running: open the browser on it and exit (no second server).
- Security for a localhost app: reject requests whose `Host` is not `127.0.0.1:<port>` or `localhost:<port>` (DNS
  rebinding), require same-origin `Origin` on non-GET requests, 1 MB body limit, no CORS. `--listen <addr>` allows LAN
  use explicitly, with a startup warning.

### 6.5 Config, CLI, logging

- **Config** `config.toml` in `directories::ProjectDirs` config dir; precedence CLI > env (`WFD_*`) > file >
  defaults. Keys: `port`, `open_browser`, `optimizer_threads`, `mc_runs_default`, `data_dir`, `log_level`,
  `image_proxy`.
- **CLI** (`clap`):

```text
warframe-dps [serve] [--port N] [--no-browser] [--threads N] [--listen ADDR]
warframe-dps calc --code <wfd1-…> [--mc-runs N] [--json]          # headless calculation (tests, agents, scripting)
warframe-dps optimize --request req.json [--json] [--node-budget N]
warframe-dps import <warframe-damage.com URL> [--json]
warframe-dps data update [--refresh] | status | verify
```

- **Logging**: `tracing` with `tracing-subscriber` (stderr, human format; `RUST_LOG` or `--log-level`) and a daily
  rolling file in the data directory (`tracing-appender`, keep 7 files); `tower_http::trace::TraceLayer` spans per
  request (method, path, status, latency); optimizer phases and data-update steps at `info`; a panic hook logs before
  abort.

---

## 7. The frontend

### 7.1 Framework and libraries

| Choice | Library | Why |
|---|---|---|
| Framework | **React 19 + TypeScript (strict) + Vite** | The UI is a dense form with many interdependent inputs plus charts and a long virtualized timeline. React has the largest ecosystem of accessible primitives and the most predictable behaviour for the agents that will implement it (more training data and fewer API generations than Svelte 5 runes or Solid). Rendering speed is not the bottleneck: the engine runs in Rust. |
| Server state | TanStack Query v5 | caching of catalogue lookups, request deduplication, `AbortSignal` cancellation of superseded `/api/calc` calls, `placeholderData` to keep the last result visible while recalculating |
| Client state | Zustand (+ `immer`) | one store for build + scenario + UI; tiny, typed, selector-based subscriptions keep re-renders local to the changed panel |
| Routing | TanStack Router | typed search params (`?b=<code>`) for the share string |
| Primitives | Radix UI (dialog, popover, combobox via `cmdk`, tabs, tooltip, toggle, slider) styled with Tailwind CSS v4, components owned in-repo (shadcn/ui pattern) | WAI-ARIA behaviour for free; full visual control |
| Drag and drop | `@dnd-kit` (pointer and keyboard sensors) | mod swapping by drag *and* keyboard |
| Charts | Recharts (SVG) for DPS over time, breakdowns, uptime, crit tiers; every chart has a table view | readable, accessible, enough for ≤ 600 points |
| Virtualization | `@tanstack/react-virtual` | hit timeline with thousands of hits |
| i18n | `i18next` + `react-i18next` + `eslint-plugin-i18next` (`no-literal-string`) | section 7.6 |
| Tests | Vitest + Testing Library (units), Playwright + `@axe-core/playwright` (end-to-end), MSW (mock server from the generated types) | section 8 |

### 7.2 Layout and component tree

Desktop (≥ 1280 px): three panes. Left "Loadout", centre "Results", right "Target & simulation". Below 1280 px the
right pane becomes a drawer; below 900 px the panes stack with a tab bar (Loadout / Results / Target).

```text
<App>
├── <TopBar>  logo · <DataVersionBadge> (stale-data warning) · <ShareButton> · <ImportDialog> · <OptimizerLink> · <SettingsMenu> (theme, number format) · <AboutLink>
├── <CalculatorPage>
│   ├── <LoadoutPane>
│   │   ├── <WeaponPicker> (combobox: search, slot/class filters, keyboard nav, images)
│   │   ├── <VariantControls> <KitgunParts> <ZawParts> <ProgenitorPicker> <ExaltedElementPicker>
│   │   ├── <ModGrid> 8 × <ModSlot> + <ExilusSlot> + <StanceSlot>(<ComboSelect>) + <ArcaneSlots>(1 or 2)
│   │   │     └── <ModPicker> popover: search by name or stat, compatibility, incompatibility greying, Semi filter, set badges
│   │   ├── <IncarnonTiers> tier tiles → <PerkPicker>
│   │   ├── <RivenEditor> dialog (4 traits, class-filtered list, units, ≤ 3 base elements, live card preview, value ranges at disposition 1.2)
│   │   ├── <FramePanel> (collapsible) 4 × <FrameModSlot>, <AuraSlot>, 2 × <FrameArcaneSlot>, 5 × <ShardSlot> (colour + stat + tauforged)
│   │   ├── <BuffsPanel> per buff: toggle + value (Roar, Eclipse, Xata, Toxic Lash, Nourish, flat crit, … from /api/buffs)
│   │   └── <CustomStatsDialog> 26 inputs grouped (damage, crit, status, weapon, melee, target)
│   ├── <ResultsPane>
│   │   ├── <HeadlineCards> dps_window (EV) with MC mean ± CI · burst · sustained · TTK (+ kill probability)
│   │   ├── <LayerTabs> Armor | Health | Shield | Overguard (present layers only; "expand" = full-width mode)
│   │   │     └── <AttackTable> per attack row: inspiration columns (queue DPS, avg per bullet direct/status/total, median per bullet, TTK) + ours; group rows (parent + child)
│   │   ├── <Charts> <DpsOverTime> (MC p10-p90 band + EV line) · <DamageBreakdown> (by type / by source) · <StatusUptime> · <CritTiers>
│   │   ├── <HitTimeline> (one MC run: per-hit cards, reload markers, live stacks; drag/scroll; "More details" parity)
│   │   ├── <WeaponStatsPanel> base → modded per attack, per-type damage, deltas
│   │   └── <FormulaInspector> stage-by-stage values (ST-1..ST-9) with contributing items per bucket
│   └── <TargetPane>
│       ├── <EnemyPicker> (name or faction) · level · Steel Path · Eximus · party size
│       ├── <EnemyPanel> pools, armor and DR, innate DR, immunities, unique-DR note, faction weaknesses
│       ├── <OptionsPanel> T-02 conditions (3 modes), T-03 hit location, T-04, T-05, T-06, T-07, T-08..T-12, T-19, T-20, T-24, T-27, T-28, hidden T-21 under "advanced"
│       ├── <CombineAttacksDialog> (T-22)
│       └── <SimulationPanel> window (1-60 s, advanced to 600), MC runs and seed, "re-run" (new seed)
├── <OptimizerPage>
│   ├── <ObjectivePanel> metric · view (auto/explicit) · attacks with weights · heavy + combo · window
│   ├── <ConstraintsPanel> per-slot locks (lock icons on the mirrored mod grid) · rules · element require/forbid · capacity/polarity/forma · riven policy
│   ├── <GroupsPanel> exilus, arcanes, stance, Incarnon, frame mods (0-4), aura, frame arcanes, shards (owned types), parts, progenitor
│   ├── <BuffPortfolioPanel> allowed buffs with values
│   ├── <ProhibitedMods> searchable chips, defaults, persisted in localStorage
│   ├── <RunPanel> start/abort · progress bar with phase · rate · gap · ETA · <IncumbentPreview> (diff vs current build) · <LogPane>
│   └── <ResultPanel> best build · alternatives · best per damage mix · leave-one-out · next-best per slot · condition sensitivity · buff table · MC confirmation · "Apply to calculator"
└── <AboutPage> data sources, licences, data update button with progress, coverage report
```

### 7.3 State and data flow

- `useAppStore` (Zustand) slices: `build` (mirrors Rust `Build`), `scenario` (mirrors `Scenario`), `ui` (open
  panels, selected attack, chart modes), `optimizer` (form state; the job itself is server state).
- Every change of `build`/`scenario` updates the URL (`history.replaceState`, debounced 300 ms) with the share code.
  Encoding needs the stable-key registry, so the code is not computed in the browser: every `CalcReport` includes
  the `code` of its inputs, which keeps exactly one encoder (Rust).
- `useCalc()` = TanStack Query keyed by a stable hash of `(build, scenario, want)`, debounced 120 ms, aborting the
  previous request; EV results arrive first (fast path: `want.mc = None`), then an MC request fills distributions.
  The UI never blocks; stale values are dimmed, not removed.
- Catalogue data (`/api/weapons/{key}`, `/api/items`) is cached indefinitely per `data_version`.

### 7.4 Parity with the inspiration (every row of `I:feat` "Parity checklist")

| Checklist item | Where |
|---|---|
| Weapon search (substring, keyboard nav) | `<WeaponPicker>` (cmdk combobox) |
| All attacks simulated and listed separately | `<AttackTable>` one row per attack, plus group rows |
| Stance slot + combo selector | `<StanceSlot>` + `<ComboSelect>`; combo saved in the share code |
| Heavy/slam attacks with `isHeavy` auto-mode | attack kinds from data; T-09 toggle disabled with a tooltip on explicit-heavy attacks |
| Weapon info panel with live modded stats and per-type damage | `<WeaponStatsPanel>` |
| Incarnon evolutions and form attack | `<IncarnonTiers>`; form attack rows; Incarnon magazine shown |
| Kitgun grip + loader; Zaw grip + link | `<KitgunParts>`, `<ZawParts>` |
| Progenitor element + 25-60 % | `<ProgenitorPicker>` |
| Chromatic Blade / Reactive Storm element | `<ExaltedElementPicker>`, saved in the code (improvement) |
| Faction-weakness element mods | effect target `element` with `faction`-dependent selection resolved at L0 |
| 8 mod slots + exilus + arcane (+2 archgun arcanes) | `<ModGrid>`, `<ArcaneSlots>` |
| 4 frame mods, aura, 2 frame arcanes, 5 shards (tauforged) | `<FramePanel>` |
| Mod search by name or stat, compatibility, exclusivity, Semi filter, set bonuses | `<ModPicker>` with `/api/items?stat=`; set-bonus badge with piece count |
| Remove, swap by drag (+ keyboard) | `<ModSlot>`: Delete/Backspace, context menu, `@dnd-kit` keyboard sensor (Space to lift, arrows, Space to drop) |
| Riven builder (4 stats, class lists, units, bow ×2, faction multiplier) | `<RivenEditor>`; value ranges shown as a bonus |
| Enemy search by name or faction | `<EnemyPicker>` |
| Level, Steel Path, Eximus, party size, innate DR, immunity, unique-DR warning | `<TargetPane>` + `<EnemyPanel>` |
| Enemy scaling incl. Eximus and 200/2700 clamp | engine `enemy/`; values in `<EnemyPanel>` |
| Every "additional option" (19 controls incl. hidden `o.w`) | `<OptionsPanel>` and `<BuffsPanel>` (mapping in spec section 6) |
| Custom stat values (26) | `<CustomStatsDialog>` |
| Combine attack effects | `<CombineAttacksDialog>` |
| Trigger all conditions (default on) | T-02 selector, default `all_max` |
| Queue length 1-60 s | window slider in `<SimulationPanel>` |
| Random queue re-rolled on change; average over cycles with min/median/max | MC with a fixed run count; "re-run" button draws a new seed; parity columns show median/min/max (spec 4.5) |
| Result panels per layer with expand | `<LayerTabs>` + expand |
| Per attack: queue DPS, avg per bullet (direct/status/total), median per bullet, TTK | `<AttackTable>` parity columns |
| Extended queue timeline | `<HitTimeline>` |
| Tooltips (mod, arcane, shard info, attack damage types, layer explanations) | Radix tooltips everywhere; texts are i18n keys; item descriptions rendered from effects + source text |
| Build link encode/decode; copy button | `<ShareButton>` (copies URL with `?b=`), `<ImportDialog>` |
| Optimizer: metrics, attack, layer, heavy + combo, window, beam width, element exclusion | `<ObjectivePanel>`; "beam width" maps to the context beam `B` (advanced) |
| Optimizer: build rules, warframe slot toggles, strength, allowed buffs with values | `<ConstraintsPanel>`, `<GroupsPanel>`, `<BuffPortfolioPanel>` |
| Optimizer: prohibited mods with persistence and defaults | `<ProhibitedMods>` |
| Optimizer: progress, log, abort, apply | `<RunPanel>`, `<ResultPanel>` |
| Localization | i18n layer, English only (CLAUDE.md) |

### 7.5 Beyond parity (what makes it clearly better)

- **Formula inspector**: for the selected attack and view, a waterfall from base damage to final damage per stage
  (base vector, composition and quantization, Y with CO, faction, G, part, crit, enemy side), each bucket expandable to
  the items that contributed and their values. No other calculator shows *why* a number is what it is.
- **Honest statistics**: EV and MC side by side, CI on the headline number, EV-vs-MC deviation badge.
- **Charts**: DPS over time with MC bands (reload dips and status ramp visible), damage by type and by source,
  status uptime and stacks over time, crit-tier distribution.
- **Diff view**: any two builds (current vs optimizer result vs pasted code) compared stat by stat and output by
  output.
- **Data transparency**: unmodelled items are marked; curated entries flagged as possibly changed by an update show a
  warning icon with the source text.

### 7.6 Internationalization

- All UI text is in `frontend/src/i18n/en/{common,loadout,results,target,optimizer,about,errors,messages}.json`,
  ICU-free i18next JSON v4 with plurals and interpolation. Keys are typed via `CustomTypeOptions` so a missing key is a
  compile error; `eslint-plugin-i18next/no-literal-string` fails CI on any literal string in JSX or attributes.
- Numbers, percentages and durations are formatted by i18n formatters (`Intl.NumberFormat` for the active locale).
- Game item names come from the bundle's `NameTable` (English now); adding a locale later means adding a WFCD i18n
  file to the pipeline and a translation folder, with no code change. Mechanic names (damage types, statuses, layers,
  toggles) are i18n keys, not data.
- The server never sends English sentences: warnings, errors and optimizer log lines are `{ code, params }`
  (`opt.phase.context.accepted { group: "arcane", item_key: "…", gain: 0.034 }`).

### 7.7 Build sharing

- **Our share string** `wfd1-<base64url>`: `postcard` serialization of `BuildCodeV1 { data_epoch: u16, weapon:
  StableKey, variant, main: [SlotCode; 8], exilus, stance + combo, arcanes, incarnon: [u8; 5], frame, riven:
  Option<RivenCode>, scenario: ScenarioCode { enemy: StableKey, level, flags, toggles (only non-defaults), buffs,
  custom stats (sparse), combine groups, window } }` followed by a CRC-16. Items are referenced by append-only
  `StableKey`s, so codes survive data updates; ranks are stored only when not max. Typical length 60-140 characters.
  Unlike the inspiration's link, it stores every scenario input (window, stance combo, combine groups, exalted
  element, party size; `I:feat` §15 "not stored" list).
- The page URL carries the code (`?b=wfd1-…`); `<ShareButton>` copies it.
- **Importing a warframe-damage.com link**: `wfd-buildcode::import_inspiration(url)` decodes `p` with LZString
  (`n=1`) or legacy base64, parses the JSON key map of `I:feat` §15 (`w`, `e`, `l`, `sp`, `ex`, `j`, `jv`, `z`, `k`,
  `m.*`, `r.*`, `o.*`, `p.*`), maps names to our items by English name plus `inspiration_aliases.toml` (shards such as
  "+25% Critical Damage", Incarnon `{mId, pId}` tier/perk indices), converts units (percent → fraction, riven faction
  multiplier → bonus), and returns the build with a list of unmapped entries. Tested on
  `research/test-cases/user-example-furis.json`.

### 7.8 Visual design direction

- **Look**: dark-first "Tenno console" aesthetic, not a game-UI imitation and no DE artwork or branding: deep
  blue-grey surfaces (`#0f141b` background, layered `#161d27` / `#1d2633` cards), one warm accent (amber `#f5b544`)
  for primary actions and highlights, cool cyan for MC/statistics. A light theme with the same tokens. All colours
  are CSS custom properties (`--surface-1`, `--accent`, `--dt-heat` …), checked for WCAG AA contrast.
- **Damage types** have a consistent colour + icon (inline SVG) pair used everywhere: slot cards, damage bars, charts,
  timeline. Colour is never the only channel (icon and label too).
- **Typography**: Inter for UI, JetBrains Mono with `tabular-nums` for every number so columns align; numbers are
  large where they matter (headline DPS) and quiet elsewhere.
- **Mod cards**: compact tiles with rarity edge, polarity glyph, rank pips, drain, a damage-type stripe; hover/focus
  shows the full effect text and the contribution to the current result ("+12.4 % DPS").
- **Motion**: 150 ms transitions on value changes (numbers tween), none with `prefers-reduced-motion`.
- **Density**: everything for one weapon visible without scrolling on a 1440×900 screen; collapsible sections
  remember their state.

### 7.9 Accessibility

WCAG 2.2 AA as the target: every control reachable and operable by keyboard (roving tab index in the mod grid; Enter
opens the picker; Delete removes; Ctrl+arrows move a mod; Esc closes), visible focus rings, labelled inputs and
units, `aria-live="polite"` region announcing the headline DPS after recalculation (throttled), charts with a
"show as table" toggle, minimum 24×24 px targets, no information conveyed by colour alone, `prefers-reduced-motion`
honoured. `@axe-core/playwright` runs on every page in CI with zero serious violations allowed.

---

## 8. Testing strategy

| Layer | What | Where | Tooling | CI gate |
|---|---|---|---|---|
| Spec vectors | every `TV-nn-mm` of `docs/mechanics/*.md` via its machine twin (section 4.7), starting with the worked example of spec 3.8; spec tables transcribed as constants | `wfd-engine/tests/{spec_conformance,spec_vectors}.rs` | custom harness, `Probe` paths | transcription check and all vectors pass at the documents' tolerances |
| Rule tests | one `#[test]` per formula or decision without a numeric vector (E-11 cases, CD-04 ordering, D-0xx decisions, NP-1/NP-2 f32 points) | `wfd-engine/src/**` `mod tests` | `cargo test` | all pass |
| Engine properties | fixed-point incremental = full compile (bitwise); monotonicity of every `Inc` slot; `Bound ≥ WindowExact`; relative error of `WindowFast` ≤ ε; determinism across thread counts and repeated runs | `wfd-engine/tests/properties.rs` | `proptest` (fixed seeds in CI, 10k cases nightly) | all pass |
| EV vs MC | spec 4.4: EV `dps_window` inside the MC 99 % CI + 2 % on the golden scenarios, except classes listed by `08` | `wfd-engine/tests/ev_vs_mc.rs` | 10,000 MC runs per scenario (nightly), 1,000 in PR CI | all pass |
| Data pipeline | Lua conversion on small fixture modules; typed parsing incl. typo aliases; join on `uniqueName`; validators (each rule of section 2.5 has a failing and a passing fixture); `curate check` on the real curated tables; the spec anchors | `wfd-datagen/tests/`, CI job `curate-check` | `cargo test`, `insta` snapshots for reports | all pass; coverage thresholds of `data/validation.toml` met |
| Bundle | `data/generated/*.json` → `bundle.bin` → load → same counts and anchors; schema version bump detection | `wfd-data/tests/` | `cargo test` | all pass |
| Golden oracle | inspiration run headless with seeded `Math.random` (`I:oracle`) on oracle-safe scenarios (`08` rules): compatibility profile compares `queue_dps_parity` and the per-bullet outputs; game profile compares only scenarios with no register row | `tools/oracle/` (Node + Playwright) records `tests/golden/oracle/*.json`; `wfd-engine/tests/golden_oracle.rs` compares | tolerance from both sides' sampling error (median of ≥ 200 seeded oracle runs vs ≥ 2,000 of ours) | all pass; recording is an offline task because the bundle is gitignored research material |
| In-game measurements | `research/test-cases/*.json` with measured numbers (precedence 1 of spec 1.4) | `wfd-engine/tests/measured.rs` | — | all pass at the stated measurement precision |
| Optimizer | B&B argmax = brute force on small pools (100 random problems); bound validity; locks, incompatibilities, capacity, element rules honoured; identical result for 1, 2, N threads with `node_budget`; cancel returns within 100 ms with a valid incumbent; "beats WFOpt": on the oracle weapon set our best build ≥ WFOpt's best under our `WindowExact` | `wfd-opt/tests/` | `cargo test`, oracle recordings of `WFOpt.runFull` | all pass |
| Server | every route with `tower::ServiceExt::oneshot`; SSE sequence (progress → done) and reconnect; cancel; Host/Origin rejection; body limit; CLI `calc`/`import` output snapshots | `wfd-server/tests/` | `tokio::test`, `insta` | all pass |
| Frontend units | store logic, formatters, i18n key coverage (every key used exists; no unused keys), component behaviour | `frontend/src/**/*.test.tsx` | Vitest + Testing Library | all pass; `eslint` (incl. `no-literal-string`) clean; `tsc --noEmit` clean |
| End-to-end | one Playwright spec per parity-checklist row (test titles carry `PAR-nn` ids matching section 7.4), keyboard-only flows, share code round trip, Furis inspiration-link import, optimizer run + cancel against the real binary with a node budget, data-version badge | `frontend/e2e/` | Playwright (Chromium), starts `warframe-dps --no-browser --port 0` | all pass |
| Accessibility | axe on every page and open dialog | `frontend/e2e/a11y.spec.ts` | `@axe-core/playwright` | zero serious/critical violations |
| Visual | screenshots of fixed scenarios (dark and light) | `frontend/e2e/visual.spec.ts` | `toHaveScreenshot` | diff ≤ threshold (reviewed updates only) |
| Performance | criterion benches (section 4.9) and an optimizer benchmark (fixed rifle and melee problems, `node_budget`) | `benches/` | `criterion`, `cargo xtask bench-check` | no regression > 25 % vs baseline; targets of section 0.4 within 2× on CI hardware |
| Cross-platform | build + unit tests on Linux, macOS, Windows; EV/MC report hashes of 10 fixed scenarios equal across platforms | CI matrix | — | equal hashes |

---

## 9. Work breakdown

Each task is sized for one agent. "Honours" lists the interfaces the task must implement or must not change; a task
may only change a `wfd-model` type through task M2's owner (or a follow-up task), so parallel tasks stay compatible.
Engine tasks work on `tests/fixtures/mini-bundle.json` and do not wait for the pipeline; frontend tasks work against
MSW mocks generated from the TypeScript types and do not wait for the server.

| ID | Task | Depends on | Honours / provides | Acceptance criteria |
|---|---|---|---|---|
| M1 | Workspace skeleton: crates of section 1.2 (empty libs), `xtask`, release profile, CI (fmt, clippy `-D warnings`, test, frontend lint/test placeholders), `.gitignore` (`.cache/`, `frontend/dist`) | — | crate names, dependency directions (section 1.2) | `cargo build`, `cargo clippy -- -D warnings`, `cargo test` pass; CI green |
| M2 | `wfd-model`: enums (spec 2.2 order), `define_stats!` layout with metadata (section 3.5), effect DSL (section 3.4) with TOML/JSON serde and JSON Schema, `Weapon`/`Attack`/`Item`/`Enemy`/`Bundle`, `Build`, `Scenario`, toggles T-01..T-29, `OptRequest`/`OptResult`, API types, `ts` feature | M1 | sections 3, 5.1, 6.1 | round-trip tests for every type; all 18 examples of section 3.6 parse; `cargo xtask gen-ts` and `gen-schema` produce stable output |
| M3 | `tests/fixtures/mini-bundle.json`: Braton Prime, Furis (+ Incarnon form, two perks), Acceltra (parent/child), Nikana Prime (melee, stance), a kitgun chamber, Lancer, Corrupted Heavy Gunner, a Corpus shielded unit, an Eximus; the mods, arcanes, shards and buffs of section 3.6 and spec 3.8 | M2 | bundle schema | loads; values match the spec's worked example inputs |
| D1 | `wfd-datagen fetch`: `PoliteClient`, wiki batched revisions, WFCD pinned raw files, PE index and manifests, cache layout, `data/sources.toml` | M1 | section 2.2 stage 1 | offline re-run hits only the cache; changed `revid` refetches one module; tests with a local HTTP mock |
| D2 | Lua sandbox and typed wiki structs (weapons, enemies, mods, arcanes, damage types, stances, modular) with typo aliases | M2, D1 (or cached files) | section 2.2 stage 2 | every module of `DS:wiki` §2 converts; counts match `DS:wiki` (911 weapons, 1,984 attacks, 1,019 enemies, 1,550 mods, 166 arcanes, 132 stances) |
| D3 | Join, derive (attack kinds, parents, groups, triggers, compat, stable keys), emit `data/generated/*.json` + report; `wfd-data` loader, embedded bundle via `build.rs`, indexes, name search | D2 | section 2.2 stages 3-4, 7; section 2.3 | uniqueName join rates as in `DS:pe` §1.8 or better; anchors of spec 3.8 present; bundle ≤ 1 MB compressed; load < 100 ms |
| D4 | Effect-data machinery: tier 1 stat-line importer with `stat_lines.toml` (04 §5.1), tier 2 spec-table ingestion (04 §2.14, 03 §4.1, 05 §5.1), tier 3 curated loader and resolver, all validators of section 2.5, `curate todo/scaffold/check`, rendered descriptions | M2, D3 | section 2.4-2.5 | each validator has failing and passing fixtures; tier 1 maps ≥ 65 % of weapon mods with zero unmapped-line errors on them; all 04 §2.14 rows ingested or listed with a reason |
| C1 | Curate primary weapon mods (rifle, shotgun, sniper, bow, launcher) + primary arcanes | D4 | DSL only | `curate check` clean; every mod in the inspiration's primary set covered |
| C2 | Curate secondary mods + secondary arcanes | D4 | DSL only | same |
| C3 | Curate melee mods, stances (incl. `stance_co.toml`), melee arcanes, exalted-weapon mods | D4 | DSL only | same |
| C4 | Curate shards, sets, auras, frame mods, frame arcanes, buffs (abilities, focus, companions, debuffs), custom stats, `mult_groups.toml` | D4 | DSL only | every toggle and custom stat of spec section 6 has an encoding |
| C5 | Curate Incarnon families (perks per tier, variants), `co_overrides.toml`, `attack_overrides.toml` (06 §4.3), `enemy_overrides.toml` (05 §4.5), `riven.toml`, `progenitor.toml`, `inspiration_aliases.toml` | D4 | DSL only | every CO catalogue row maps to a bundle attack; all Incarnon families of `F:mods` §9 present (or listed as missing with reason); every 06 §4.3 table filled |
| E1 | Engine compile: fixed-point `StatVec`, L0 scenario compile, L1 item compile (T-02 table, scopes, scope factors, rule filters), L2 finalizers (sets, shards, locks), riven, buffs, custom stats | M2, M3 | `CompiledScenario`, `CompiledItem`, `CompiledBuild`, `StatVec` API | property test: incremental = full; all section 3.6 examples compile to the stated contributions |
| E2 | Engine resolve: elements (combiner + `ElementSolver`), assembly (kitgun, zaw, progenitor, exalted element), ST-1..ST-7 (`ResolvedAttack`), `Probe`, spec-vector harness and transcription check, `docs/mechanics/vectors/{00,01,02}.toml` | E1, spec 01/02 | `ResolvedAttack`, probe paths `resolve.*`, `crit.*` | TV-00-01 steps 2-4 and every `TV-02-*` pass; `01` vectors pass when published |
| E3 | Engine enemy: scaling (NP-2), Steel Path, Eximus, overguard, armor E-15, faction table, layers, DR, attenuation | M2, M3 | `ScaledEnemy`, probes `enemy.*` | spec 3.8 step 1 passes; `05` vectors pass |
| E4 | Engine schedules: ranged (fire rate, burst, charge, beams, spool, magazine, reload classes, first/last shot, sniper combo) and melee (stance hits, combo, heavy, slam), segment compression | E2, spec 06/07 | `Schedule` | `06`/`07` vectors pass; schedule cost O(segments) |
| E5 | EV: kernel, Poisson helpers, `Steady`, `WindowExact`, `WindowFast`, `Bound`, EV TTK, Heat window mean, hit metrics; `Kernel` API for the optimizer | E2-E4, spec 03/04/08 | `Kernel::{score, bound}`, `Engine::ev` | spec 3.8 steps 5-10 pass; `08` EV vectors pass; properties of section 8 pass; section 0.4 targets met |
| E6 | MC simulator: Philox RNG (KAT), event loop, state, DoT, views in one pass, layered pass, summaries, timeline | E2-E4 | `Engine::mc` | EV-vs-MC tests pass; determinism across threads; ≥ 5 M events/s/core |
| E7 | Outputs and compatibility profile: `CalcReport`, parity outputs, breakdowns, `FormulaTrace`, Compat flags for every register row marked Compat | E5, E6 | `Engine::calculate` | report snapshot tests; Compat flags listed and tested one by one |
| O1 | Optimizer core: problem compile, pools, rule filters, dominance, composition enumeration with `ElementSolver`, main-slot B&B (compaction, suffix tables, capacity/polarity), parallel work split, atomic threshold, keep-set | E5 | `solve_main(&Kernel, &MainProblem, &JobControl) -> MainSolution` | equals brute force on 100 random small problems; deterministic across thread counts; rifle fixture proven optimal within the section 0.4 budget |
| O2 | Optimizer driver: context groups and ascent, context beam, cross-group local search, buff portfolio, riven search, finalists (`WindowExact`), MC confirmation, reports, progress and cancellation | O1, E6 | `optimize(&Engine, OptRequest, &JobControl) -> OptResult`, `Progress` | section 8 optimizer tests pass; cancel ≤ 100 ms; "beats WFOpt" test passes once G1 recordings exist |
| B1 | `wfd-buildcode`: `wfd1-` codec, inspiration link import (LZString, legacy base64, key map, aliases, units) | M2, D3 (names, stable keys) | `encode`, `decode`, `import_inspiration` | round-trip property test; Furis link imports with zero unmapped entries |
| S1 | Server: routes of section 6.1, SSE, job manager, static embedding, browser opening, single-instance check, config, logging, CLI subcommands | E7, O2 (stubs allowed earlier), B1 | section 6 | route tests; SSE tests; Host check test; `warframe-dps calc` works headless |
| S2 | In-app data update: `datagen` feature in the server, `POST /api/data/update` with progress, override bundle, `ArcSwap` hot reload, About page data | D3, D4, S1 | section 2.3 | update from a mocked source set swaps the bundle without restart; running jobs unaffected |
| F1 | Frontend foundation: Vite/React/TS, Tailwind tokens and themes, Radix component kit, i18n setup + lint rule, generated API types, MSW mocks, Zustand store, routing, app shell and layout | M2 (TS types) | section 7.1-7.3, 7.6, 7.8 | lint/typecheck/test clean; shell renders with mocks in dark and light |
| F2 | Calculator loadout and target panes (every input of section 7.4) | F1 | `Build`, `Scenario` TS types | Playwright parity specs for inputs pass against mocks, then against S1 |
| F3 | Results pane: headline cards, layer tabs, attack table (parity columns), charts with table views, hit timeline, weapon stats, formula inspector, diff view | F1 | `CalcReport` TS type | parity specs for outputs pass; axe clean |
| F4 | Optimizer page: objective, constraints, groups, buffs, prohibited list (localStorage), run panel with SSE, results, apply | F1, S1 (SSE) | `OptRequest`, `Progress`, `OptResult` | run + cancel + apply e2e pass against the real binary |
| G1 | Oracle harness and golden tests: scenario format, Playwright recorder for calculator and `WFOpt.runFull`, recordings, Rust comparison tests | E7, B1 | section 8 golden row | ≥ 30 oracle-safe scenarios recorded and passing in Compat profile |
| Q1 | End-to-end and accessibility suite completion; visual baselines; cross-platform CI matrix | F2-F4, S1 | section 8 | all e2e rows of section 7.4 have a passing `PAR-nn` test |
| P1 | Performance: benches, baselines, `bench-check`, profiling pass on kernel and B&B | E5, O1 | section 0.4 | targets met on the reference machine and recorded in `docs/architecture/performance.md` |
| R1 | Packaging and release: `cargo xtask dist`, binaries for three platforms, NOTICE/licences in the About page and next to the binary | S1, F2-F4 | section 2.3 licence handling | a fresh machine runs the binary offline and opens the browser |
| A1 | Promote the merged architecture to `docs/architecture/` (crate layout, data model, API, frontend design) after review | judge's merge | CLAUDE.md layout | reviewed docs; this file marked `superseded` |

Critical path: M1 → M2 → E1 → E2 → E5 → O1 → O2 → S1 → F4. Curation (C1-C5) is the longest-running parallel stream
and should start as soon as D4 lands; until then, engine and optimizer tasks use the mini bundle.

---

## 10. Notes for the judge

**Load-bearing for performance, cheap to keep in any merge:**

1. The compile levels L0-L4 and the fixed-point `StatVec` (exact incremental updates, free determinism and caching).
2. The effect DSL compiling every condition to a constant or a coefficient on a state variable per T-02 (no DSL
   interpretation in the hot loop; one code path for mods, arcanes, shards, perks, buffs, custom stats and rivens).
3. One target-state kernel with four fidelities, including a provable `Bound`; `WindowExact` remains the ranking
   truth, so accuracy is never traded for speed (P5).
4. The composition/monotone split with branch-and-bound and an `ElementSolver`: it turns "heuristic search" into
   "proven optimum of the main slots" for guns and gives an honest gap otherwise.
5. Determinism by construction (Philox keyed per U-6, `libm`, ordered reductions, canonical tie-breaks).

**Negotiable without hurting the priority:** React versus another framework, TOML versus another curated format,
the exact crate split (for example merging `wfd-buildcode` into `wfd-data`), SSE versus WebSocket.

**Risks and mitigations.**

| Risk | Mitigation |
|---|---|
| A stat wrongly classified `Inc` makes B&B prune a better build | property tests and brute-force exactness tests (section 5.4); reclassifying to `Unknown` is a one-word change |
| `WindowFast` error larger than 0.5 % for some class (beam ramp, melee stance cycles, Heat ramp) | measured per class in CI; classes over budget use `WindowExact` (slower, still correct); the keep-set margin uses the measured ε |
| Curated effects are the critical path (hundreds of mods) | scaffold tool, priority ordering, parallel curation tasks, "not modelled" marking so partial coverage is usable |
| `postcard` bundles are schema-brittle | `schema_version`, JSON snapshot as the reviewable source, embedded fallback |
| `08` chooses an EV algorithm detail that differs from section 4.3 | the kernel/fidelity split does not depend on it: `WindowExact` implements whatever `08` defines, and `WindowFast`/`Bound` are re-validated against it by the property tests |
| Licence of DE Public Export data, and CC BY-NC-SA share-alike on the bundle | used only for identifiers and checks; NOTICE and About page; the user decides before any public release |

**Open questions for the manager.**

1. Should `WindowFast` ever be shown in the UI (for example while dragging a slider), or only `Steady`/`WindowExact`?
   This proposal shows `WindowExact` (it is fast enough for interactive use: about 42 µs per attack and view).
2. Should the default optimizer pool include items flagged "possibly changed by an update"? Proposed: yes, with a
   warning in the result.
3. Code licence: the data bundle inherits CC BY-NC-SA 3.0; the code licence is the user's choice.
