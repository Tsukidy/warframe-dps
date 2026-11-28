Status: reviewed (architecture)

# Architecture overview

This is the entry document of the system design. It fixes the crate layout, the data flow, and the decisions that
every implementation task must honour. The detailed contracts live in the sibling documents:

| Document | Contents |
|---|---|
| `docs/architecture/domain-model.md` | Rust domain types, the stat registry, the effect encoding with encoded examples, toggles and conditions |
| `docs/data/pipeline.md` | data sources, fetching, Lua parsing, joining, curated tables, validation, bundle, refresh, licences |
| `docs/architecture/engine.md` | engine API, compile levels, stages, the two evaluators, determinism, performance, spec test vectors |
| `docs/architecture/optimizer.md` | search algorithm, bounds, parallelism, scoring, constraints, progress and cancellation |
| `docs/architecture/server-api.md` | every HTTP route with JSON shapes, SSE events, static serving, startup |
| `docs/architecture/frontend.md` | stack, layout, components, state, parity mapping, i18n, sharing, visual design, accessibility |
| `docs/architecture/testing.md` | test layers, tools, oracle golden tests |
| `docs/architecture/work-breakdown.md` | ordered implementation tasks with exact interfaces and acceptance commands |

The damage mechanics are owned by `docs/mechanics/` (the spec). This architecture implements the spec; where code and
spec disagree, one of them is a bug (CLAUDE.md). Spec identifiers (E-n, ST-n, CD-nn, D-nnn, T-nn, OQ-nnn, TV-nn-mm)
are used verbatim. All ten topic documents (00-09) exist as drafts; this revision was checked against all of them, in
particular 07 (melee: combo tables, plans, synthesized heavy attacks), 08 (EV algorithm, MC keying, passes V, L and
L0, outputs, optimizer metrics, oracle procedure) and 09 (build rules: slot ids, legality semantics "validation never
blocks evaluation", capacity modes). Where the spec documents conflict with each other, section 6.2 lists the request
to the manager; no code is written against research findings where a spec section exists.

---

## 1. Requirements recap

| Source | Requirement |
|---|---|
| CLAUDE.md | Local web app: one Rust binary (Axum) serves the API and the built frontend on localhost and opens the browser. No Tauri. |
| CLAUDE.md | English only, every UI string through an i18n layer; languages are added later as data. |
| CLAUDE.md | Optimizer runs locally in Rust, may take a while, uses all CPU cores. |
| CLAUDE.md | Out of scope: monthly top-20 chart, donations, Telegram. |
| CLAUDE.md | Game data from open sources (wiki Lua modules, WFCD, DE Public Export). The inspiration is a reference and a test oracle, never a data source. |
| Brief | Backend in Rust. Frontend may be anything; it must look clearly better than warframe-damage.com and match every feature of its parity checklist (`research/findings/inspiration-feature-inventory.md` "Parity checklist"). |
| Spec 00 §4 | Two evaluators over the same resolved stats: deterministic expected value (EV) for the optimizer and the headline numbers, event-driven seeded Monte Carlo (MC) for distributions, TTK and validation. |
| Spec 00 §6 | Every toggle T-01..T-43 (spec 00 §6.1) is part of the scenario, saved in the build link, and changes the engine exactly as defined. |

---

## 2. System diagram

```text
                         ┌──────────────────────── build time / data refresh ────────────────────────┐
 wiki.warframe.com ─┐    │                                                                            │
 (Lua data modules) │    │  wfd-datagen                                                               │
 DE Public Export ──┼──► │  fetch ─► parse (mlua) ─► join ─► derive ─► effects ─► validate ─► emit    │
 WFCD warframe-items┘    │    │                                  ▲ ▲                          │       │
                         │  .cache/wfd-sources/ (gitignored)     │ │                          ▼       │
                         │                  data/curated/*.toml ─┘ │      data/generated/*.json      │
                         │   docs/mechanics ingestable tables ─────┘      (committed, reviewable)     │
                         └────────────────────────────────────────────────────────┬───────────────────┘
                                                                                  │ wfd-data/build.rs
                                                                                  ▼
 ┌────────────────────────────────── warframe-dps (one binary) ───────────────────────────────────────────┐
 │  embedded bundle.bin ─► wfd-data::GameData ◄── user bundle (<data_dir>/bundles/*.bin after a refresh)  │
 │  embedded data/curated, data/spec_tables.toml, data/generated/spec (refresh inputs, feature "refresh") │
 │        (ArcSwap)            │                                                                            │
 │                             ▼                                                                            │
 │   wfd-engine (no I/O): compile L0-L2 ─► resolve L3 ─► schedule ─► EV kernel (4 fidelities) │ MC V/L/L0   │
 │                             ▲                                                                            │
 │   wfd-opt (no I/O): pools ─► composition sets ─► parallel branch-and-bound ─► context search ─► finalists│
 │                             ▲                                                                            │
 │   wfd-buildcode: share codes, inspiration-link import                                                    │
 │                             ▲                                                                            │
 │   wfd-server (Axum 0.8): REST + SSE on 127.0.0.1, embedded frontend (include_dir), opens the browser     │
 └─────────────────────────────┬────────────────────────────────────────────────────────────────────────────┘
                               │ HTTP (JSON, SSE)
                               ▼
        Browser: React 19 + TypeScript + Vite, React Aria Components, Tailwind v4, react-intl
        calculator  ·  optimizer  ·  data/about  (types generated from Rust with ts-rs)
```

---

## 3. Workspace and repository layout

```text
warframe-dps/
├── Cargo.toml                 # [workspace], [workspace.dependencies], [workspace.lints], release profile
├── rust-toolchain.toml        # channel = "stable"
├── CONTRACTS.md               # change log of frozen interfaces (work-breakdown §2)
├── crates/
│   ├── wfd-model/             # types only: enums, ids, stat registry, effect DSL, weapon/enemy/item/bundle schema,
│   │                          # Build/Scenario, toggles, compat flags, API DTOs, message keys (features: ts, schema)
│   ├── wfd-data/              # bundle codec, embedded bundle (build.rs), GameData + indexes, search, legality (spec 09)
│   ├── wfd-engine/            # compile, resolve, enemy, schedules, EV, MC, outputs, compat, probes (no I/O)
│   ├── wfd-opt/               # optimizer (no I/O)
│   ├── wfd-buildcode/         # wfd1 share codes, inspiration link import/export
│   ├── wfd-datagen/           # pipeline library + `wfd-datagen` CLI
│   └── wfd-server/            # binary `warframe-dps`: Axum, SSE, static frontend, config, CLI, data refresh
├── xtask/                     # cargo xtask: gen-ts, gen-schema, spec-sync, spec-vectors, lint-engine, ci,
│                              # effect-smoke, oracle-record, oracle-audit, bench-check, dist, dist-smoke, doc-check
├── data/
│   ├── curated/               # hand-authored TOML, one file per item; rules/ (incl. rules/melee/) (pipeline.md §6)
│   ├── generated/             # pipeline output: canonical JSON + REPORT.md + LICENSE (commit policy: AD-11)
│   ├── fixtures/mini/         # tiny hand-written bundle in the generated JSON format (tests, parallel work)
│   ├── sources.lock.toml      # pinned upstream revisions of the committed bundle
│   ├── spec_tables.toml       # which spec tables are ingested (pipeline.md §5.2)
│   ├── validation/            # base.toml (allow-lists) + coverage/<task>.toml (ratcheting minimums per batch)
│   └── NOTICE.md              # data attribution and licences
├── frontend/                  # React app; `npm run build` → frontend/dist (embedded by wfd-server)
├── tools/oracle/              # Node + Playwright harness that drives the inspiration offline
├── docs/  research/           # as today
└── .cache/                    # gitignored fetch cache
```

### 3.1 Crates and dependency direction

| Crate | Depends on (workspace) | Key external crates (version line on 2026-11-23) |
|---|---|---|
| `wfd-model` | — | serde 1.0, bitflags 2.13, smallvec 1 (not the 2.0 beta), arrayvec 0.7; optional ts-rs 12.0 (`ts`), schemars 1.2 (`schema`), toml 1.2 (`curated`: authoring-form loaders) |
| `wfd-data` | model | serde, postcard 1.2, ruzstd 0.9 (pure-Rust decoder, and the encoder of bundles written at run time, pipeline.md §8.2), arc-swap 1.9, blake3 1.8; build-deps serde_json, postcard, zstd 0.14 (level 19 for the embedded bundle) |
| `wfd-engine` | model | libm 0.2, smallvec, arrayvec; optional rayon 1.12 (`parallel`, default) |
| `wfd-opt` | model, data, engine, buildcode (the share code of every ranked build, AD-18) | rayon 1.12, rustc-hash 2.1, xxhash-rust 0.8, parking_lot 0.13 |
| `wfd-buildcode` | model, data | miniz_oxide 0.9, base64 0.23, lz-str 0.2, serde_json |
| `wfd-datagen` | model (`curated`), data | mlua 0.12 (`lua51`, `vendored`, `send`), ureq 3.5, lzma-rs 0.3, toml 1.2, serde_json, regex, sha2, clap 4.6, include_dir 0.7 (the `InputTree::Embedded` variant, pipeline.md §2) |
| `wfd-server` | all of the above (`wfd-datagen` behind feature `refresh`, default on) | axum 0.8, tokio 1.54, tower-http 0.7, include_dir 0.7 (frontend `dist` and, with `refresh`, `data/curated/`, `data/spec_tables.toml` and `data/generated/spec/`), mime_guess 2.0, webbrowser 1.2, directories 6.0, clap 4.6, tracing 0.1, tracing-subscriber, tracing-appender |
| `xtask` | model (`curated`), data, engine (vector harness, effect smoke) | clap, toml, serde_json, regex |

```text
wfd-model ◄── wfd-data ◄── wfd-opt ◄── wfd-server
    ▲  ▲          ▲   ▲      │  │           │
    │  └── wfd-engine ◄──────┘  │           │
    │             ▲             │           │
    ├── wfd-buildcode ◄─────────┘ ◄─────────┤
    └── wfd-datagen (writes bundles via wfd-data) ── feature "refresh"
```

`wfd-engine` depends on `wfd-model` only: it takes a `&Bundle` (a model type), so its tests load
`data/fixtures/mini/` through `serde_json` and never need the pipeline or the generated bundle. Tests that run over the
real curated data (the effect smoke test, testing.md §3) live in `xtask effect-smoke`, which depends on `wfd-data` and
the generated bundle. `wfd-engine` and `wfd-opt` do no I/O, hold no
global state, and build without default features (no rayon), so a WebAssembly build stays possible. Only
`wfd-server` touches the network (images, user-started data refresh), the file system and the browser.

---

## 4. Data flow

1. **Build time (developer).** `wfd-datagen fetch` fills `.cache/wfd-sources/` politely (pipeline.md §3).
   `wfd-datagen build --offline` parses wiki Lua modules with `mlua`, joins the three sources on `uniqueName`, derives
   attack roles, parent links and stable keys, attaches effects (tier 1 automatic stat lines, tier 2 tables ingested
   from `docs/mechanics/`, tier 3 curated TOML), validates everything, and writes `data/generated/*.json` plus
   `REPORT.md`. A pull request shows the data change as a readable diff.
2. **Compile time.** `wfd-data/build.rs` deserializes `data/generated/` into model types, checks the schema, and
   embeds `bundle.bin` (postcard + zstd). The binary works offline on first start.
3. **Runtime, calculator.** The browser edits a `BuildDoc` + `ScenarioDoc` (JSON, items referenced by `StableKey`).
   `POST /api/calc` resolves them against `GameData`, runs the engine (EV always; MC when asked), and returns a
   `CalcReport` with the weapon panel, enemy panel, per-attack per-view outputs, charts, warnings and the share code.
4. **Runtime, optimizer.** `POST /api/optimize` starts one job on the optimizer thread pool; progress, incumbents and
   the result stream over SSE; `DELETE` cancels and returns the best build found so far.
5. **Runtime, data refresh.** The About → Data page (or `warframe-dps data refresh`) runs the same pipeline library
   in-process with the curated tables, the spec-table list and the ingested spec tables (`data/generated/spec/*.json`)
   embedded in the binary, writes a user bundle to the data directory, and swaps it in atomically (`ArcSwap`); running
   jobs keep their old snapshot. Items new since the embedded bundle get session-local keys whose share codes carry the
   uniqueName (AD-18).

---

## 5. Judging outcome

Two proposals were judged: **P-C** (`research/findings/architecture-proposal-correctness-and-extensibility.md`) and
**P-O** (`research/findings/architecture-proposal-optimizer-performance.md`).

| Criterion | P-C (correctness, extensibility) | P-O (optimizer performance) | Better |
|---|---|---|---|
| Effect encoding expresses every mechanic of spec 00 and the `wiki-*` findings | Strongest: closed set of mechanic kinds (extra hit, proc on event with exclusive groups, repeat hit, counter bonus, spawned attack, conversion, flat damage on proc, proc after ticks), resolved-stat scalars (Melee Doughty), attack-role conditions (Pain Points, `AttackPred::Roles`), item parameters (Chromatic Blade), target-weakness elements, gate splitting for conditional composition in EV, derived condition class. Written before 02-06, so its stack model lacks 04's `refresh` and `decay` fields | Aligned with 02-06 (04 §4 trigger ids and state machine, 04 §2.14 CO catalogue ingested, 06 §4.3 tables), `scope_factor` (x2 for heavies or bows). One-off behaviours are untyped `hook = <key>` engine functions, which is item-specific code by another name | P-C, with P-O's alignment to 04 |
| Parity with every checklist row | Every row mapped, one end-to-end test per row | Every row mapped, `PAR-nn` test ids | Tie |
| Optimizer speed and result quality | Plausible design (lanes, B&B, element classes) but unmeasured; proxy search on steady EV, then only the top 200 re-scored with the window metric, so the window optimum can be missed | Measured prototype (246 ns compile+resolve, 486 ns steady EV, 140 ns per target-state point, 42 µs for a 300-shot window). Four fidelities of one kernel including a provable `Bound`; composition/monotone split with an exact `ElementSolver`; keep-set rule that provably keeps the window optimum despite the fast screen; optimality gap reported | P-O |
| Simplicity and update after a patch | One TOML file per item (parallel curation without merge conflicts), number binding to upstream text, per-rank cross-check, stale detection and auto-patching of number-only drift, `unmodelled` never silently zero | Three effect tiers: about 70 % of weapon mods imported automatically from stat lines, spec tables ingested from the spec (no copy), curated TOML only for the rest; one file per family | Merge: P-O's tiers + P-C's per-item files, binding and auto-patching |
| Testability | Spec vectors as fenced blocks, coverage matrix with a fixture per row, effect smoke tests, compat flags checked against the register, traces that record touched register rows (oracle-safety decided automatically) | Machine twins of the Markdown vectors with a transcription check (fits the vectors 01-06 already wrote as tables), bound-validity and monotonicity property tests, brute-force exactness, cross-platform hash equality | Merge |
| Risk | Bounds "checked symbolically" without a plan for failures; ShortId hash codes break on uniqueName changes | Hooks invite special cases; one-file-per-family curation conflicts | Merge |

**Verdict.** P-O is the base: its compile levels, fixed-point stat vector, kernel fidelities and optimizer are the
hardest parts to retrofit, are measured, and are already aligned with topic documents 02-06. Grafted from P-C: the
effect IR semantics (mechanic kinds replace hooks; derived condition classes; resolved-stat scalars and conditions;
item parameters; per-effect condition overrides; gate splitting), the curation workflow (one file per item, number
binding, per-rank checks, auto-patching, `unmodelled` status), the engine invariants and lints (no item names in
engine code, register-synchronized compat flags, ordered reductions), traces with touched register rows, the
schema-driven option panels, React Aria for accessible drag and drop, the Explain drawer, baseline comparison and
undo/redo. Designed here because both missed or under-specified it: the exact build-code format (StableKey JSON,
deflated), the attack identity rule across data updates, the toggle placement map (where each T-nn lives), merged
attack groups in EV (T-22), the effect-status lattice and its effect on optimizer pools, the contract freeze protocol
for parallel agents, and the thread-pool split between interactive calculation and optimizer jobs. The licence
question is left to the user (AD-11).

---

## 6. Key decisions

| ID | Decision | Rationale |
|---|---|---|
| AD-01 | Seven library/binary crates plus `xtask`; engine and optimizer are I/O-free | testable without the pipeline; the server is the only I/O edge; WASM remains possible |
| AD-02 | All shared types (domain, API DTOs, message keys) live in `wfd-model`; the frontend's TypeScript types are generated from it with `ts-rs` (`cargo xtask gen-ts`), and CI fails when they are stale | one contract crate frozen first lets backend and frontend tasks run in parallel |
| AD-03 | Effects: one DSL for every source (mods, arcanes, shards, Incarnon perks, sets, buffs, custom stats, riven traits, weapon passives). Targets: registry stat, element, faction, multiplier group, or a mechanic from a closed list. No item-specific engine code (`xtask lint-engine` fails on item names or `/Lotus/` in `wfd-engine`/`wfd-opt`) | a patch or a new item is a data change; new mechanics are reviewed once as a new kind with a spec vector |
| AD-04 | Every effect compiles per scenario to a constant or an affine form in a small set of state variables (distinct statuses, stacks per status, combo tier, own condition stacks, schedule marks). EV takes expectations over the distributions; MC reads live values | both evaluators read the same resolved stats (spec 00 §4.1) and CD-13 is enforced structurally |
| AD-05 | Bucket sums are fixed-point `i64` (1e-9 units) in a `StatVec`; conversion to `f64` once per resolved attack | exact incremental updates and order-independent sums: determinism and caching come free |
| AD-06 | EV has four fidelities of one target-state kernel: `Steady`, `WindowExact` (the spec's definition, used for every number the user sees and for final rankings), `WindowFast` (screening, tested error ε), `Bound` (provable upper bound for pruning) | speed without trading accuracy: approximations may prune or screen, never decide |
| AD-07 | MC: Philox4x32-10 keyed exactly per spec 08 E-8.16/E-8.17 (key = seed halves; counter = run, FNV-1a of the mechanic id string, attack and shot anchor, sub-instance and block); runs in parallel, reduced in run order; passes V (the four non-depleting views at once), L (layered) and L0 (layered with armor ignored, parity only) of 08 §6.4 | bit-reproducible for any thread count; common random numbers per (run, anchor, mechanic, ordinal); TV-08-01/02 known answers |
| AD-08 | Optimizer: enumerate composition sets (elements, physical, conversions, forced procs, and every candidate whose slots the compiled scenario downgrades to `Unknown`) with an exact element-order solver; solve the monotone remainder of the 8 main slots by parallel branch-and-bound; search context groups (exilus, arcanes, stance and combo, melee plan, Incarnon tiers, frame slots, shards, parts, progenitor, buffs) by joint enumeration, block-coordinate ascent with a context beam, and cross-group local search; re-score finalists with `WindowExact`; confirm with MC | main-slot optimum proven under a stated assumption (optimizer.md §5.2), honest gap otherwise, every weakness of the inspiration's optimizer addressed (optimizer.md §10) |
| AD-09 | Data: wiki Lua modules (mlua, vendored Lua 5.1, sandboxed) for attacks, enemies, stances, modular parts, faction table; Public Export for uniqueName, drain, polarity, mod sets, riven base values, freshness; WFCD for names, images, i18n, per-rank text checks; curated TOML for effects | per the data-source findings; join on uniqueName only |
| AD-10 | `data/generated/` (canonical JSON) is committed; the binary embeds a postcard+zstd bundle built from it at compile time | game patches become reviewable diffs; builds are reproducible offline |
| AD-11 | **Licence and publication of wiki-derived data: open decision for the user (Q-U1).** The data is CC BY-NC-SA 3.0 (wiki) plus MIT (WFCD); committing it to a public repository or shipping it inside released binaries is a publication decision that belongs to the user, not to this architecture. Interim rule until the user answers (the manager records the answer in CLAUDE.md "Decisions"): `data/generated/` is committed to this private repository (that is not redistribution), carries `data/generated/LICENSE` (CC BY-NC-SA 3.0) and `data/NOTICE.md` (wiki module titles and revids, WFCD MIT notice, "Warframe content © Digital Extremes; not affiliated"); T39 builds release binaries but publishes nothing. Raw upstream files are never committed (`.cache/` and `research/external/` stay gitignored); Public Export fields are limited to identifiers and numbers WFCD already redistributes. If the user declines, the fallback is one `.gitignore` line for `data/generated/` plus `wfd-datagen fetch --from-lock && wfd-datagen build --offline` before every build. Without `WFD_DATA_DIR` and without `data/generated/`, `wfd-data/build.rs` embeds `data/fixtures/mini/` and emits a `cargo:warning` with that hint, so every crate keeps compiling (`wfd-datagen` too, which creates the directory); `xtask dist` (T39) refuses to package a binary whose bundle is a fixture (ruling T02c-R5 of the contract review) | the user owns licensing and publication; the interim keeps reviewable diffs and offline builds without publishing anything |
| AD-12 | Curated tables: TOML, one file per item, numbers bound to the upstream text and checked at every rank; stale entries detected; number-only drift auto-patched in generated data and flagged "auto-updated, not reviewed" | curation is the largest effort; parallel curators must not conflict; updates after a patch are mostly mechanical |
| AD-13 | In-app data refresh (feature `refresh`, default on) runs the same pipeline library with embedded inputs: `data/curated/`, the spec-table list `data/spec_tables.toml` and the ingested spec tables `data/generated/spec/*.json` (embedded through the server's `build.rs`, with an empty placeholder while `data/spec_tables.toml` or `data/generated/spec/` is missing, so a refresh then fails with `DatagenError::Config` naming it; rulings T02c-R6, NEW-CR-1); it never reads `docs/` | users update after a hotfix without a new release; the binary has no access to the spec Markdown |
| AD-14 | Server: Axum 0.8 on 127.0.0.1, REST + SSE (no WebSocket), Host/Origin checks, embedded frontend, browser opened on start, single-instance detection | progress is one-way; cancel is `DELETE`; SSE reconnects for free |
| AD-15 | Thread pools: optimizer pool `max(1, cores − 1)` threads, one job at a time; interactive pool of `cores` threads whose MC work is split into at most 2 chunks while an optimizer job runs and into `cores` chunks otherwise (results are independent of the split, I-5) | the calculator stays responsive while the optimizer saturates the machine, and uses the whole machine when it does not |
| AD-16 | Frontend: React 19 + TypeScript + Vite; React Aria Components (accessible comboboxes, grid lists with keyboard drag and drop, dialogs); Tailwind v4 with CSS-variable tokens; Zustand + Immer (undo/redo); TanStack Query and Router; react-intl (ICU); Recharts; TanStack Virtual | accessibility of the mod grid is the hard UI problem; React Aria solves it; the rest is the most agent-familiar stack |
| AD-17 | Option, buff, custom-stat and toggle panels are rendered from schemas the server sends (`GET /api/schema`) | a new buff or toggle needs no frontend change |
| AD-18 | Share code `wfd1-` + base64url(raw deflate(canonical JSON with StableKeys)); keys from the committed `stable_keys.json` are global; keys a user's in-app refresh assigns to new uniqueNames are session-local (≥ 2^31) and every code or persisted list that contains one also carries `x: { key: uniqueName }` for it, resolved by uniqueName on decode; the copy action shares the bare code (the `?b=` URL works only on the machine that made it); Rust is the only codec; inspiration links (`?p=…&n=1` LZString, legacy base64) import with a report | forward-compatible JSON, codes survive data updates and refreshes on other machines, one implementation |
| AD-19 | Determinism: `libm` for transcendental functions, no FMA contraction, `BTreeMap`/sorted `Vec` in results, ordered parallel reductions, canonical tie-breaks | identical numbers across thread counts and platforms |
| AD-20 | Compatibility profile: `CompatFlags` (u128) with exactly one flag per register row whose switch kind is `Compat` (spec 00 §5.1 rule 3: the first word of each `+`-separated part of the Toggle cell before any parenthesis, `:`, `;` or `,`, so `— (Compat path of D-065)` is no flag), over the registers of every topic document (00 §5.2 and each "Inspiration differences" table of 01-09; the count follows the registers: 74 flags when T01d landed, 2026-11-27); `xtask spec-sync` generates the flag list and fails on a mismatch; hidden from the UI and from the generated TypeScript (flags travel as names) | oracle tests only (spec 00 §4.6) |
| AD-21 | Every resolved number can carry a trace (contributing items, equation id, register rows touched); the UI shows it in an Explain drawer; the oracle harness uses touched rows to decide whether a scenario is oracle-safe | correctness is visible to users and reviewers |
| AD-22 | Spec test vectors: each `TV-nn-mm` of 00-09 has a machine twin (`crates/wfd-engine/tests/spec_vectors/NN.toml`) whose expected numbers must occur verbatim in the Markdown; fenced `testvector` blocks in future topic documents are extracted automatically | the spec stays the only source of expected numbers |
| AD-24 | Build legality is spec 09, implemented once in `wfd-data::legality` (slot ids `gen0`…`gen7`, `exilus`, `arcane0/1`, `inc2`…`inc5`, `fgen0`…`fgen7`, `fexilus`, `aura0/1`, `farcane0/1`, `shard0`…`shard4`, `subsumed`); violations are reported and never block evaluation; the optimizer treats `illegal` rules as constraints, with capacity mode `any_forma` by default | one rule set for the UI, the decoder and the optimizer; inspiration links and free-form inputs still evaluate (09 §1.5) |
| AD-23 | Default optimizer metric `dps_window` (spec 00 §4.1 headline); metrics, TTK ordering and tie-breaks exactly as spec 08 §10 (E-8.26); default optimizer pool includes `verified`, `auto` and `draft` effects (marked), never `unmodelled` | matches the headline the user sees |
| AD-25 | Melee follows spec 07 as data plus one generator: combo tables per combo type resolved per E-7.0b (unique overrides, stance, stanceless class table), strikes expanded per E-7.0, a synthesized heavy attack per weapon with pipeline-computed hit lists (E-7.40, E-7.41), hand tables 9-A to 9-E in `data/curated/rules/melee/`, and a `MeleePlan` in the scenario (T-30) consumed by `melee_schedule` (E-7.30) | the melee model is data-driven like the ranged one; plans make heavy, Tennokai and build-and-spend play comparable |
| AD-26 | Warframe identity is data: `Bundle.frames` from `Module:Warframes/data` joined to `ExportWarframes` (polarities, aura slot count, abilities, Helminth host rules of 09 §10) | Helminth legality, Jade's two auras and warframe capacity (09 E-9.15) need the frame record |

### 6.1 Invariants with their enforcement

| ID | Invariant | Mechanism | Test |
|---|---|---|---|
| I-1 | Game numbers come from the bundle; the engine holds only formula constants of the spec | constants in `wfd-engine/src/spec_constants.rs`, each with its E-number | `xtask lint-engine`: float literals outside that file and tests fail (allow-list 0, 1, 2 and 0.5) |
| I-2 | No item-specific branches in engine or optimizer | items reach code only as compiled effects and attack data | `xtask lint-engine` greps for `/Lotus/` everywhere, comments included, and for every item name of the mini fixture outside comments; a whole identifier that is model or spec vocabulary (a generated `CompatFlags` name, a stat-registry row or variant, a resolved-stat name, a melee class name, a melee plan kind, a spec symbol of the `docs/mechanics` symbol tables or a mechanic id of 08 E-8.16/E-8.17, read from the spec at lint time) does not count, and custom-stat records are not items (CONTRACTS.md C-22, LINT-GLAIVE) |
| I-3 | EV and MC share resolution | both consume `ResolvedAttack` + `Schedule`; state-dependent terms are affine forms | EV-vs-MC property test (spec 00 §4.4) |
| I-4 | Same build in any non-element slot order gives bit-identical results | fixed-point sums | proptest over slot permutations |
| I-5 | MC independent of thread count | counter-based RNG, ordered reduction | 1 vs 8 thread test |
| I-6 | Unknown is never silently zero | items without effects are `unmodelled`: badged in the UI, excluded from optimizer pools, listed in `CalcReport.warnings` | e2e on a fixture item |
| I-7 | Discrepancy switches only where the register allows | `CompatFlags` generated from every topic register; `Toggles` checked against the toggle table | `xtask spec-sync` |
| I-8 | No UI literal strings | ICU catalogue, server messages are `{ key, args }` | ESLint `formatjs/no-literal-string-in-jsx`; Rust test that every `MessageKey` has an `en.json` entry |

### 6.2 Spec alignment and requests to the manager

Every topic document 00-09 is available, so no task writes code against research findings where a spec section
exists, and there are no `SPEC-PENDING` markers. Implementation tasks cite the spec section they implement
(work-breakdown.md). Spec 09 is implemented once by `wfd-data::legality`. Spec-level conflicts found during the review
are requests to the manager; until the manager decides, the architecture uses the provisional choice stated:

| Request | Conflict | Provisional choice |
|---|---|---|
| R-1 Toggle numbering | 07 OQ-722 proposes T-30 "melee plan"; 09 OQ-915 proposes T-30 "capacity mode" and T-31 "lower ranks" | Settled: spec 00 §6.1 numbers them T-30 melee plan, T-35 capacity mode, T-36 lower mod ranks; `TOGGLE_MAP` lists T-01 … T-43, all `Spec` (T01d) |
| R-2 Spec vectors for new DSL constructs | the review added `count_within` triggers, `counter_bonus.reset_on = heavy`, hit-outcome gates, `Per::Resolved` power curves (`Pow`) and `Per::ReloadedFraction` (domain-model.md §11) for items 04 catalogues; 04 has vectors for Plated Round (TV-04-16) but not for the others | engine unit tests from the 04 catalogue rows; items that use a construct without a spec vector stay `draft` (pipeline.md §6.2) |
| R-3 Licence decision Q-U1 | AD-11 | interim rule of AD-11 |

---

## 7. Risks

| Risk | Mitigation |
|---|---|
| Curation of about 900 items is the long pole | tier 1 auto-import covers about 70 % of weapon mods; scaffold tool; ten parallel curation tasks (Incarnon split per weapon class); coverage thresholds ratchet per batch instead of blocking the build; partial coverage is usable because `unmodelled` items are visible |
| A stat wrongly classified monotone lets the optimizer prune a better build | downgrades derived from the compiled scenario (every effect or mechanic whose trigger, gate, scalar or condition reads a crit or proc outcome, a schedule mark or a resolved stat downgrades the slots feeding it, optimizer.md §4); monotonicity fixtures per reader kind; bound-validity property tests; brute-force exactness tests; a wrong class costs speed, never correctness |
| `WindowFast` error exceeds ε for some weapon class | ε measured per class in CI; classes over budget use `WindowExact` |
| EV approximations (Poisson stacks, independence, gate-split limit) differ from MC | spec 00 §4.4 tolerance tests; per-scenario deviation flag shown in the UI |
| Spec 07-09 are drafts and may still change | interfaces follow the drafts section by section; the contract change protocol (work-breakdown §2); spec-sync detects register and toggle changes |
| mlua needs a C compiler | only `wfd-datagen` and the server's `refresh` feature; `--no-default-features` builds without it |
| Wiki licence conditions | AD-11 (user decision Q-U1 pending) |

---

## Review notes

The independent architecture review (2026-11-23) raised 32 issues. Each was checked against the spec, the findings, npm,
the inspiration bundle and the network; 31 were applied and one was rejected.

Rejected:

- Progenitor value search (minor, optimizer.md §6.1): the inspiration does not choose the percentage. Its optimizer
  enumerates the seven elements of `pe` (`JS L21921`) at `tb()`, which reads the user's `#pElVal` field and falls back to
  60 (`JS L17746-17755`); the inventory's "picks the progenitor element and %" overstates it. Our search (element
  enumerated, value from the build, default 60 %) already matches; a value search would also not be monotone (the value
  changes the damage mix, so it would have to be enumerated over the 331 legal values of 09 E-9.70 for no parity gain).

Issues applied with a deviation from the suggested fix:

- Melee heavy attacks: 07 §9.1 says the engine synthesizes the heavy attack; the pipeline only computes its hit lists
  (E-7.40, E-7.41 need Public Export data) and emits the synthesized attack record, the engine resolves it.
- Spec vectors for new constructs (`count_within`, `reset_on = heavy`, outcome gates) cannot be written by the
  architecture; they are requested from the spec owner (R-2) and the items stay `draft` until then.
- Toggle renumbering and the licence decision cannot be made here; they are requests R-1 and R-3 with provisional
  choices, and CLAUDE.md is left to the manager.
- Offline data builds: instead of committing trimmed upstream snapshots (which would add raw wiki content to the
  repository), `wfd-datagen fetch --from-lock` restores the exact locked inputs before `build --offline`.
- WindowFast error: the proof is labelled "optimal assuming the WindowFast error ≤ ε"; a `certify` search option
  re-scores every surviving leaf with `WindowExact` and drops the assumption (optimizer.md §5.2).
- Interactive MC target: instead of a fixed 2-thread pool, the interactive pool has `cores` threads and splits MC work
  into at most 2 chunks while an optimizer job runs (AD-15); the 2-thread target is stated in engine.md §13.
- Image host: `cdn.warframestat.us` is unreachable here, so images come from the Public Export texture paths on
  `content.warframe.com` (already an allowed host, checked: 200 for a Braton icon); WFCD versions are discovered through
  `registry.npmjs.org` (`@wfcd/items`, version = tag). Tests and visual baselines run with image fetching off.
- Task sizes: T01 and T02 are split by crate and concern, T15 into three tasks, curation into ten tasks with the
  Incarnon families split per weapon class, and the rule tables moved into two wave-1 tasks (work-breakdown.md);
  task numbers changed accordingly (performance is T37, packaging T39).
