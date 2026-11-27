Status: reviewed (architecture)

# Work breakdown

Ordered implementation tasks, one agent each. Every task lists what it depends on, the files it owns (only the owner
writes them), the public interfaces it must provide or honour, the spec sections it implements, and the commands that
must pass for acceptance. Tasks in the same wave run in parallel; a later task starts when its own dependencies are
accepted, not when the whole previous wave is.

Read first: `overview.md` (decisions), `domain-model.md` (types), and the document of your area. The spec
(`docs/mechanics/00-09`) is normative for every number and rule; all ten documents exist, so every engine and data task
names the spec sections it implements, and no code is written against research findings where a spec section exists
(section 2.4).

Contents: 1 Waves and critical path · 2 Rules for every task · 3 Wave 0 · 4 Wave 1 · 5 Wave 2 · 6 Wave 3 · 7 Wave 4 ·
8 Ownership map

---

## 1. Waves and critical path

```text
Wave 0  T01a workspace + model types ─► T01b API DTOs + TS/JSON schema ─┐
                                    └─► T01c xtask (spec-sync, lints)   ├─► T02a engine skeleton · T02b data + buildcode
                                                                        │   skeletons · T02c datagen skeleton ·
                                                                        └   T02d optimizer + server skeletons
Wave 1  T03 mini fixtures · T03b rule tables · T03c melee hand tables · T04 wfd-data · T05 datagen sources ·
        T06 engine compile · T07 engine enemy · T08a ranged schedule · T08b melee (07) · T09 spec-vector twins ·
        T10 build codes · T11 server skeleton · T12 frontend foundation
Wave 2  T13 resolve · T14 status + mechanics · T15a EV kernel + steady · T15b EV window walk · T15c EV bound, fast,
        TTK, Kernel · T16 MC · T17 outputs, compat, probes · T18 datagen join/derive/emit · T19 effects machinery ·
        T20-T29 curation (ten batches) · T30 frontend loadout · T31 frontend target + results
Wave 3  T32 optimizer core · T33 optimizer driver · T34 server integration · T35 frontend optimizer + data pages ·
        T36 oracle harness · T37 performance · T38 end-to-end completion · T39 packaging
Wave 4  T40 accuracy audit · T41 documentation pass
```

| Task | Name | Depends on |
|---|---|---|
| T01a | Workspace and model types | — |
| T01b | API DTOs, message keys, TypeScript and JSON-schema generation | T01a |
| T01c | xtask foundation: spec-sync, lint-engine, ci, doc-check | T01a |
| T01d | Contract sync to the final spec (manager-added) | T01a, T01b, T01c, T03 |
| T02a | Engine skeleton and internal contracts | T01a, T01b |
| T02b | `wfd-data` and `wfd-buildcode` skeletons | T01a, T01b |
| T02c | `wfd-datagen` skeleton | T01a, T01b |
| T02d | `wfd-opt` and `wfd-server` skeletons | T01a, T01b |
| T03 | Mini fixture bundle and scenarios | T01a |
| T03b | Rule tables (real data) | T01a |
| T03c | Melee hand tables 9-A to 9-E (real data) | T01a |
| T04 | `wfd-data` and legality (09) | T02b (T03 for tests) |
| T05 | Data pipeline: sources, fetching, parsing | T02c |
| T06 | Engine compile L0-L2, monotone classes, elements | T02a (T03 for tests) |
| T07 | Engine enemy side (05) | T02a (T03 for tests) |
| T08a | Engine ranged firing stats and schedule (06) | T02a (T03 for tests) |
| T08b | Engine melee: tables, timing, combo, plans, schedule (07) | T02a (T03 for tests) |
| T09 | Spec-vector harness and twins (00-09) | T02a, T03 |
| T10 | Build codes and inspiration links | T02b (T04 for import; T03 for tests) |
| T11 | Server skeleton | T02d (T04 for real data) |
| T12 | Frontend foundation | T01b |
| T13 | Engine resolve (ST-1..ST-7) and assembly | T06, T08a, T08b |
| T14 | Status effects and mechanics | T13, T07 |
| T15a | EV kernel and steady metrics | T13, T14, T07, T08a, T08b |
| T15b | EV window walk | T15a |
| T15c | EV bound, fast window, TTK, optimizer kernel | T15b |
| T16 | Monte Carlo simulator | T13, T14, T07, T08a, T08b |
| T17 | Outputs, compatibility profile, traces, probes, EV vs MC | T15c, T16 |
| T18 | Data pipeline: join, derive, spec tables, emit, bundle | T04, T05, T03b, T03c |
| T19 | Effects machinery, validators, curation tools, effect smoke | T18 |
| T20-T29 | Curation batches | T19 (authoring may start after T01a against `data/curated/schema.json`) |
| T30 | Frontend loadout | T12 (MSW mocks; live API once T34 is accepted) |
| T31 | Frontend target and results | T12 (as T30) |
| T32 | Optimizer core | T15c, T04 |
| T33 | Optimizer driver | T32, T16 |
| T34 | Server integration | T11, T17, T33, T10, T18 |
| T35 | Frontend optimizer and data pages | T12, T34 (mocks before) |
| T36 | Oracle harness and golden recordings | T17, T10 |
| T37 | Performance | T15c, T16, T32 |
| T38 | End-to-end completion | T30, T31, T35, T34 |
| T39 | Packaging | T34, T38 |
| T40 | Accuracy audit | T36, T33, T20-T29 |
| T41 | Documentation pass | all |

Critical path: T01a → T01b → T02a → T06 → T13 → T14 → T15a → T15b → T15c → T32 → T33 → T34 → T38 → T39. Curation
(T20-T29) is the largest effort and runs alongside waves 2-3; the mini fixture bundle lets every other task proceed
before it finishes, and coverage thresholds ratchet per batch instead of blocking anything (pipeline.md §7).

---

## 2. Rules for every task

### 2.1 Contract freeze

- T01a-T01c and T02a-T02d define every shared interface: `wfd-model` types, crate public APIs, engine internal types,
  stage and mechanic signatures, MC state and event types, API DTOs, message keys, the curated TOML schema, the bundle
  JSON layout, the xtask command list. Each is frozen when its task is accepted.
- A task that needs a change to a frozen interface records it in `CONTRACTS.md` (date, task, change, reason,
  affected tasks), makes the smallest backward-compatible change (new optional field, new enum variant with
  `#[serde(other)]` fallback, new function), and regenerates TypeScript types. A breaking change needs the manager's
  approval before it is merged.
- Workspace dependencies are declared once by T01a in `[workspace.dependencies]`; crates use `dep = { workspace = true }`.
  A new dependency is a `CONTRACTS.md` entry. Frontend dependencies are declared by T12 in `frontend/package.json`
  with the exact versions of frontend.md §1.

### 2.2 File ownership

Only a file's owner (section 8) edits it. To avoid conflicts on shared collections:

- Message keys: Rust `MessageKey` variants live in `crates/wfd-model/src/message/<area>.rs`, one file per area owned
  by the area's task; frontend strings live in `frontend/src/i18n/en/<feature>.json`, one file per feature folder,
  merged at build time.
- Fixture data: `data/fixtures/mini/` is owned by T03; a later task that needs more fixture entries adds a new file
  `data/fixtures/mini/extra/<task-id>.json` (the loader merges `extra/*.json`).
- Server routes: one module per route group (`crates/wfd-server/src/routes/<group>.rs`).
- xtask: `xtask/src/main.rs` (T01c) registers every subcommand of overview §3 from the start; each subcommand is its
  own module owned by the task named in section 8 (a stub prints the owning task until then). The owning task of a
  command may add its `mod x;` line and its `run: Some(x::run)` entry to `xtask/src/main.rs`, nothing else
  (CONTRACTS.md C-21). `xtask/Cargo.toml` is owned by T01c; other tasks change it through a CONTRACTS.md entry.
- Curated data: each curation task owns disjoint directories of `data/curated/` and its own coverage file
  `data/validation/coverage/<task>.toml` (section 8).
- `data/generated/` is a build output: any data task regenerates it with `wfd-datagen build --offline` and commits the
  diff with the source change that caused it; conflicts are resolved by regenerating, never by hand.

### 2.3 Definition of done (every task)

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace                     # plus the task's own acceptance commands
cargo xtask spec-sync --check              # once T01c is accepted
```

Frontend tasks add `npm --prefix frontend run lint && npm --prefix frontend run typecheck && npm --prefix frontend test -- --run`.
Data tasks first run `cargo run -p wfd-datagen -- fetch --from-lock` once per fresh worktree (pipeline.md §3.3; it
needs network access to the allowed hosts of pipeline.md §3.1); no task depends on the gitignored
`research/external/cache/`. Code cites the spec (`// E-12`, `// D-026`, `// 03 §10.4`) where it implements a rule.
Unsourced mechanics are not added (CLAUDE.md). No commits of `research/external/` or `.cache/`.

### 2.4 Spec alignment

- Every topic document 00-09 exists; each engine and data task below names the sections it implements, and the spec
  wins over research findings and over this architecture where they differ (overview §6.2).
- Spec conflicts and proposals are requests to the manager (overview §6.2, R-1 to R-3) with a provisional choice; the
  proposed toggles T-30, T-31 and T-32 are `Proposed` entries of `TOGGLE_MAP` (domain-model.md §9.3).
- DSL constructs added by the architecture review without a spec vector yet (`count_within`, `counter_bonus.reset_on
  = heavy`, hit-outcome gates; `Per::Resolved` power curves and `Per::ReloadedFraction` beyond the Plated Round form
  of TV-04-16; R-2) are implemented with unit tests built from the 04 catalogue rows that describe them; items that use
  them stay `draft` (pipeline.md §6.2).

---

## 3. Wave 0

### T01a Workspace and model types

**Depends**: none. **Owns**: `Cargo.toml`, `rust-toolchain.toml`, `.gitignore` (adds `.cache/`, `frontend/dist`,
`frontend/node_modules`, `target/`), `CONTRACTS.md`, `crates/wfd-model/**` except `src/api/**`, `src/message/**` and
`src/compat_generated.rs` (an empty placeholder is created here and handed to T01c).

**Provides**:

- Workspace with the members of overview.md §3 (empty `lib.rs`/`main.rs` for the other crates), `[workspace.dependencies]`
  for every crate of overview.md §3.1 and dev tools (proptest, insta, criterion), `[workspace.lints]`
  (`unsafe_code = "forbid"`, `clippy::float_cmp = "warn"` in engine crates), release profile (`lto = "fat"`,
  `codegen-units = 1`, `panic = "unwind"`).
- `wfd-model` exactly as `domain-model.md` §2-§15 (types, serde, `ts` and `schema` features): ids incl. `LOCAL_KEY_BASE`,
  enums incl. `N_TIER_BINS`, `ComboType`, `StrikeType`, weapons, attacks (`lingering`, `synthesized`, `zoom`), melee
  data (`MeleeData`, `HeavyDef`, `ComboTable`, `Combo`, `Strike`, `MeleeRules`, `MeleeTables`), enemies with
  `BodyPart.tier_add`, `Frame`, items with `Caster`, sets, Incarnon, rivens, `Rules`, `Bundle`, documents
  (`BuildDoc`, `ScenarioDoc`, `CalcInput { x }`, `SeedSel`, `Toggles` with `melee_plan`, `zoom_level`, `capacity`),
  resolved forms, `TOGGLE_MAP`, the effect DSL (`Per::Resolved { curve: ResolvedCurve }`, `Per::ReloadedFraction`,
  `TriggerId::CountWithin`, `OutcomeFilter`, `VulnMask`), mechanic kinds, `MultGroupDef`.
  - `src/stats.rs`: the `define_stats!` registry of domain-model.md §10 with metadata accessors
    `StatKey::meta(self) -> &'static StatMeta { toml: &str, symbol: &str, aggregate: Aggregate, monotone: Monotone, owner: &str }`
    and `StatKey::parse(&str) -> Option<StatKey>` (handles `"forced.SLA"`, `"stack_cap.COR"`, `"vuln_add.HP.HEA"` …).
  - `src/curated.rs` (feature `curated`): the authoring form of curated files (`CuratedItemFile`,
    `IncarnonFamilyFile`, rule-table files of pipeline.md §6.1 incl. `rules/melee/*`, header fields of
    domain-model.md §15.1, binding fields), `deny_unknown_fields`, and `load_curated_dir(&Path)`.
  - `pub mod api; pub mod message;` declared with empty modules for T01b.

**Acceptance**:

```text
cargo build --workspace && cargo clippy --workspace --all-targets -- -D warnings
cargo test -p wfd-model --features curated   # every TOML block of domain-model.md §16 parses into the authoring types;
                                             # serde round trip of every document type; SeedSel accepts "auto" and
                                             # 0 … 2^53 − 1 and rejects 2^53; StatKey::parse round-trips every key
```

### T01b API DTOs, message keys, TypeScript and JSON-schema generation

**Depends**: T01a. **Owns**: `crates/wfd-model/src/{api,message}/**`, `xtask/src/{gen_ts,gen_schema}.rs`,
`frontend/src/api/generated/**` (generated output only), `data/curated/schema.json` (generated).

**Provides**:

- `src/api/{calc,opt,data,catalog,schema,buildcode,error}.rs`: every request/response shape of `server-api.md`
  (`CalcRequest`, `CalcReport` and its parts incl. `ViewEv`, `TtkEv`, `ViewMc`, `TtkDist`, `ParityOut`, `Timeline` with
  `heat_episodes`, `TimelineRequest`, `ExplainRequest`, `Explanation`, `CandidatesRequest`, `Candidate`, `Meta`,
  `Catalog` with `frames`, `WeaponDetail`, `ItemDetail`, `EnemyDetail`, `ScenarioSchema` with `melee_plans`,
  `EncodeResponse { code, path }`, `DecodeResponse`, `OptRequest`, `OptEvent`, `OptResult`, `RankedBuild`, `Score`,
  `Proof`, `JobStatus`, `DataStatus`, `DataCheck`, `DataEvent`).
- `src/message/mod.rs`: `pub struct Message { pub key: MessageKey, pub args: BTreeMap<Box<str>, MsgArg>, pub severity: Option<Severity> }`,
  `MessageKey` assembled from per-area enums (`message/{api,doc,engine,opt,data,import,legality}.rs`).
- `cargo xtask gen-ts [--check]` (ts-rs export to `frontend/src/api/generated/`; every `u64` field carries
  `#[ts(type = "number")]`; `CompatFlags` is not exported, domain-model.md §1) and `gen-schema [--check]`.

**Acceptance**:

```text
cargo test -p wfd-model --features ts,schema  # every JSON example of server-api.md parses into its type; serde round
                                              # trip of every API type; the generated .ts files contain no `bigint`
cargo xtask gen-ts --check && cargo xtask gen-schema --check
```

### T01c xtask foundation: spec-sync, lint-engine, ci, doc-check

**Depends**: T01a. **Owns**: `xtask/src/{main,spec_sync,lint_engine,ci,doc_check}.rs`,
`crates/wfd-model/src/compat_generated.rs` (generated).

**Provides**: `xtask` with every subcommand of overview §3 registered (stubs name their owning task, section 8);
`spec-sync [--check]`: parses spec 00 §5.2 and the "Inspiration differences" register of every topic document 01-09,
generates `CompatFlags` (one flag per row whose Toggle column contains `Compat`, 71 rows today, domain-model.md §9.5),
checks `TOGGLE_MAP` against spec 00 §6.1 (`Proposed` rows allowed) and fails on a stale file; `lint-engine` (I-1, I-2;
passes on empty crates); `ci` (runs the testing.md §10 gates that exist); `doc-check` (every path, type, command and
task id named in `docs/architecture/` and `docs/data/` exists or is listed as planned; every document's first line is a
status line).

**Acceptance**:

```text
cargo xtask spec-sync --check            # 71 flags; a test copy of a register with one extra Compat row fails the check
cargo test -p xtask                      # register parser on every docs/mechanics register; lint-engine fixtures
cargo xtask lint-engine && cargo xtask doc-check
```

### T01d Contract sync to the final spec (manager-added)

Added by the manager after T01a-T01c: the spec consistency rounds (docs/mechanics Review notes, "Follow-up round")
renumbered and added toggles and input flags after T01a froze the model types.

**Depends**: T01a, T01b, T01c, T03 accepted and merged; the converged spec. **Owns**, for this change only and with
`CONTRACTS.md` entries: `crates/wfd-model/src/**` (T01a's and T01b's files), the generated files
(`frontend/src/api/generated/**`, `data/curated/schema.json`, `crates/wfd-model/src/compat_generated.rs`, regenerated
with their xtask commands, never edited by hand), `data/fixtures/**` where a renamed type or toggle requires it, and
edits to `domain-model.md`, `server-api.md` and section 2.4 of this document that bring them in line with the spec.

**Provides**:

- `Toggles` and `TOGGLE_MAP` exactly as 00 §6.1 now numbers them: T-01 … T-43 (T-30 melee plan, T-31 exalted ability
  rank, T-32 weak spots, T-33 Strength sources, T-34 finisher mode, T-35 capacity mode, T-36 lower mod ranks, T-37
  caster of T-13 … T-16, T-38 aiming and zoom, T-39 Incarnon switch time, T-40 timed reload, T-41 pellet hit fraction,
  T-42 gauge uses per minute, T-43 external statuses; T-06 stays for link import as T-43 restricted to Viral), each
  with the status 00 gives it.
- Every input field, flag and enum value the spec added during the consistency rounds that `wfd-model` cannot express
  yet. Find the complete list by diffing the input and symbol tables of 00-09 against the model; known examples are
  02 `radial_parent_part` and `torso_only`, 05 part flags `non_body`, `crit_disabled`, `wp_bonus_ok`, `is_object` and
  `cc_immune`, the T-23 options (Eximus kind, auras, external strip rate), 04 `hit_unit = input` and the exclusion rule
  for uncatalogued conditional lines, the 06 `DUPLEX` firing class, the 07 wave attack ids (`Light Waves`,
  `Heavy Waves`) and the 03 / 08 external-status state of T-43.
- Rulings on the open requests C-4 (unbounded stacks, 04 §5.5) and C-12 (two CO specs for one data attack, 04 §2.14),
  each with the smallest backward-compatible change.
- `domain-model.md` and `server-api.md` updated for C-14, C-15 and every change above; all changes recorded in one
  `CONTRACTS.md` section "T01d · Contract sync".

**Acceptance**:

```text
cargo xtask spec-sync --check          # TOGGLE_MAP matches 00 §6.1; CompatFlags match the registers
cargo xtask gen-ts --check && cargo xtask gen-schema --check
cargo test --workspace && cargo xtask doc-check
```

### T02a Engine skeleton and internal contracts

**Depends**: T01a, T01b. **Owns** (until handed over to the implementing tasks of section 8): the initial version of
every `crates/wfd-engine/src/**` file. Bodies are `todo!("Txx")` naming the implementing task; tests that need them
are `#[ignore = "Txx"]`.

**Provides** (exact public signatures; the implementing task may add private items but not change these without a
`CONTRACTS.md` entry): `Engine` API of engine.md §1; the mechanic traits of engine.md §2.1 (`MechanicEv`,
`MechanicMc`, `MechInst`, `PointState`, `EvAcc`, `ProcMass`, `McState`, `McEvent`, `McEventKind`, `Anchor`, `Phase`,
`McRng`, `McSink`, `MechanicId`) and

```rust
pub struct StageCtx<'a> { pub rules: &'a Rules, pub compat: CompatFlags, pub trace: &'a dyn TraceSink }
pub trait MechanicImpl: MechanicEv + MechanicMc + Sync {}
pub fn mechanic_impl(kind: &Mechanic) -> &'static dyn MechanicImpl;                                          // T14
// compile/ (T06; compile/melee.rs T08b)
pub fn monotone_classes(cs: &CompiledScenario, pool: &[CompiledItem]) -> ([Monotone; N_STATS], SmallVec<[Reader; 4]>); // domain-model §10
pub fn resolve_combo_tables<'b>(b: &'b Bundle, w: &Weapon, stance: Option<StanceId>) -> MeleeTables<'b>;     // 07 E-7.0b
pub fn melee_plan(sel: &MeleePlanSel, atk: &Attack, toggles: &Toggles, combo: ComboType, md: &MeleeData) -> MeleePlan; // 07 §8.1, E-7.31
// resolve/ (T13; firing.rs T08a; melee.rs T08b)
pub fn base_vector(ctx: &StageCtx, atk: &Attack, w: &AssembledWeapon, sv: &StatVecF) -> BaseVector;          // E-1
pub fn combine_elements(entries: &[OrderedElem]) -> ElemOutcome;                                              // 01 §4.11
pub fn ratios(ctx: &StageCtx, base: &BaseVector, sv: &StatVecF, stance_phys: [f64; 3], el: &ElemOutcome) -> DamageVec; // E-2
pub fn quantize(ctx: &StageCtx, ratio: &DamageVec) -> DamageVec;                                             // E-3
pub fn base_scalar(ctx: &StageCtx, base: &BaseVector, a: f64, g_f: f64, co: &CoSpec) -> ScalarY;              // E-11
pub fn crit_multiplier(cdm0: f64, cd_base_add: f64, cd_rel: f64, cd_abs: f64, m_cdf: f64) -> CritMult;        // E-4, NP-1
pub fn crit_chance(cc0: f64, cc_base_add: f64, cc_rel: f64, cc_abs: f64, cc_floor: f64, final_mul: f64) -> f64; // E-5
pub fn crit_tiers(cc: f64, p_vig: f64, tier_add: u8) -> TierDist;                                             // E-6, 02 E-2.6
pub fn part_factor(p_part: f64, k_wp: f64, wp: f64, hs: f64, m_son: f64) -> f64;                              // E-7
pub fn status_chance(sc0: f64, sc_base_add: f64, sc_rel: f64, sc_abs: f64, m_sc: f64) -> f64;                 // E-8
pub fn proc_weights(q: &DamageVec, immune: StatusSet) -> [f64; N_ST];                                         // E-9
pub fn multishot(ms0: f64, ms_base_add: f64, sum_ms: f64, locked: bool) -> f64;                               // E-10
pub fn firing_stats(ctx: &StageCtx, atk: &Attack, w: &AssembledWeapon, sv: &StatVecF) -> FiringResolved;      // 06 §5
pub fn melee_resolved(ctx: &StageCtx, md: &MeleeData, sv: &StatVecF, t: &MeleeTables) -> MeleeResolved;      // 07 §5-§7
// enemy/ (T07)
pub fn scale_enemy(ctx: &StageCtx, e: &Enemy, setup: &EnemyInstance) -> ScaledEnemy;                          // 05 §6-§8
pub fn np2_blend(f1: f32, f2: f32, s: f32) -> f32;                                                            // NP-2
pub fn effective_armor(a0: f64, strips: &ArmorStrips, heat_strip: f64, n_cor: u8) -> f64;                     // E-15
pub fn armor_dr(a: f64) -> f64;
pub fn instance_on_layer(ctx: &StageCtx, w: &DamageVec, layer: Layer, st: &TargetStateView, en: &ScaledEnemy) -> f64; // E-12, vuln[ℓ][t]
pub fn route_layered(ctx: &StageCtx, dmg: &LayeredHit, pools: &mut Pools, t: f64) -> RouteOut;                // ST-9, CD-08
// status/ (T14)
pub fn dot_tick(ctx: &StageCtx, snap: &ProcSnapshot, s: StatusType) -> f64;                                   // E-13
pub fn extra_hit(ctx: &StageCtx, parent: &HitWeaponSide, xh: &ExtraHitResolved) -> ExtraHitOut;              // E-14
// schedule/ (ranged T08a, melee T08b)
pub fn ranged_schedule(ctx: &StageCtx, ra: &ResolvedAttack, horizon: f64) -> Schedule;                       // 06 E-6.11
pub fn melee_schedule(ctx: &StageCtx, ra: &ResolvedAttack, plan: &MeleePlan, tables: &MeleeTables, horizon: f64) -> Schedule; // 07 E-7.30
// ev/ (T15a-c)
pub fn capped_poisson_mean(mu: f64, cap: u8) -> f64;                                                          // 08 E-8.3
// mc/ (T16)
pub fn philox4x32_10(ctr: [u32; 4], key: [u32; 2]) -> [u32; 4];                                              // 08 E-8.16
pub fn rng_mech(name: &str) -> u32;                                                                           // FNV-1a, E-8.16
pub fn oracle_compare(x: &[f64], y: &[f64]) -> OracleVerdict;                                                 // 08 E-8.25 C2
// stage registry for spec vectors (T09 fills it)
pub fn stage_registry() -> &'static [(&'static str, fn(&serde_json::Value) -> Result<serde_json::Value, StageError>)];
```

Engine internal types are created with their fields as in engine.md §2.1-§8 (`CompiledScenario` with `slot_class`,
`readers`, `mech_ids`, `melee`; `CompiledItem`, `SparseContrib`, `Coeff`, `StatVec`, `StatVecF`, `MechSet`,
`ElemOutcome`, `ResolvedAttack` and its parts, `Schedule`, `ShotRec`, `ReloadRec`, `Segment`, `CycleStats`,
`MeleePlan`, `CompiledMelee`, `MeleeTrack`, `ScaledEnemy`, `Workspace`, `EvalUnit`, `Pick`, `Kernel`, `ObjectiveSpec`,
`Metric`, `Fidelity`, `BuildPoint`, `Score`, `EvOut`/`McOut` re-exports, `ProbePath`, `TraceSink`, `EngineError`).

**Acceptance**:

```text
cargo build -p wfd-engine --no-default-features && cargo build -p wfd-engine
cargo clippy -p wfd-engine --all-targets -- -D warnings && cargo test -p wfd-engine
cargo doc -p wfd-engine --no-deps        # every public item documented with its spec reference or owning task
```

### T02b `wfd-data` and `wfd-buildcode` skeletons

**Depends**: T01a, T01b. **Owns** (until handed over): initial `crates/wfd-data/**`, `crates/wfd-buildcode/**`.

**Provides**:

```rust
// wfd-data (T04)
pub struct GameData { /* bundle + indexes */ }
impl GameData {
    pub fn embedded() -> Result<GameData, DataError>;
    pub fn from_bytes(bytes: &[u8]) -> Result<GameData, DataError>;
    pub fn from_json_dir(dir: &Path) -> Result<GameData, DataError>;      // merges extra/*.json
    pub fn bundle(&self) -> &Bundle;
    pub fn by_key(&self, k: StableKey) -> Option<Entity<'_>>;             // Weapon | Item | Enemy | Frame
    pub fn by_unique_name(&self, n: &str) -> Option<Entity<'_>>;
    pub fn search(&self, q: &SearchQuery) -> Vec<SearchHit>;
    pub fn catalog(&self) -> Catalog;                                     // /api/catalog incl. frames
    pub fn weapon_detail(&self, k: StableKey) -> Option<WeaponDetail>;    // and item_detail, enemy_detail
    pub fn schema(&self) -> ScenarioSchema;
    pub fn candidates(&self, input: &CalcInput, slot: SlotId, query: Option<&str>, stat: Option<&str>) -> Vec<Candidate>;
    pub fn resolve(&self, input: &CalcInput) -> Result<Resolved, Vec<Message>>;   // x map first (AD-18); violations inside
    pub fn to_doc(&self, b: &Build, s: &Scenario) -> CalcInput;                   // adds x for every local key
    pub fn image_path(&self, k: StableKey) -> Option<&str>;                       // textureLocation (server-api §10)
}
pub mod legality {                                                                  // spec 09
    pub fn check(gd: &GameData, b: &Build, s: &Scenario) -> Vec<Violation>;
    pub fn check_slot(gd: &GameData, b: &Build, slot: SlotId, pick: &Pick) -> Vec<Violation>;
    pub fn min_forma(gd: &GameData, b: &Build, mode: CapacityMode) -> FormaReport;    // E-9.14-9.19
    pub fn set_counts(gd: &GameData, b: &Build) -> SmallVec<[(SetId, u8); 4]>;        // E-9.40
    pub fn riven_range(gd: &GameData, weapon: WeaponId, riven: &Riven) -> Vec<TraitRange>;   // §12
}
pub fn encode_bundle(b: &Bundle) -> Vec<u8>;  pub fn decode_bundle(bytes: &[u8]) -> Result<Bundle, DataError>;

// wfd-buildcode (T10)
pub fn encode(input: &CalcInput) -> String;                                  // "wfd1-…"
pub fn decode(text: &str) -> Result<(CalcInput, DecodeReport), CodeError>;   // our codes and URLs only
pub fn migrate(v: u16, raw: serde_json::Value) -> Result<CalcInput, CodeError>;
pub fn import_inspiration(url_or_param: &str, data: &GameData) -> Result<(CalcInput, ImportReport), CodeError>;
pub fn export_inspiration(input: &CalcInput, data: &GameData) -> Result<String, CodeError>;
```

**Acceptance**: `cargo build -p wfd-data -p wfd-buildcode && cargo test -p wfd-data -p wfd-buildcode && cargo doc -p
wfd-data -p wfd-buildcode --no-deps`.

### T02c `wfd-datagen` skeleton

**Depends**: T01a, T01b. **Owns** (until handed over): initial `crates/wfd-datagen/**`.

**Provides**: the library API of pipeline.md §2 (`PipelineConfig`, `InputTree`, `SpecSource`, `LockMode`, `fetch`,
`build`, `diff`, `Progress`, `BuildOutput`), the CLI of pipeline.md §2 with every subcommand (bodies `todo!`), and the
module signatures the data tasks share:

```rust
pub fn fetch_from_lock(cfg: &PipelineConfig, p: &dyn Progress, cancel: &AtomicBool) -> Result<FetchReport, DatagenError>; // T05
pub fn eval_wiki_modules(sources: &[WikiSource], limits: &LuaLimits) -> Result<BTreeMap<Box<str>, serde_json::Value>, DatagenError>; // T05
pub fn join(w: &WikiRecords, pe: &PeRecords, wfcd: &WfcdRecords) -> Result<Joined, DatagenError>;          // T18
pub fn normalise_stances(raw: &serde_json::Value, report: &mut Report) -> Vec<ComboTable>;                  // T18, 07 E-7.0
pub fn heavy_def(w: &JoinedWeapon, tables: &[ComboTable], rules: &MeleeRules) -> Option<HeavyDef>;           // T18, 07 E-7.40, E-7.41
pub fn ingest_spec_tables(spec: &SpecSource, cfg: &SpecTablesConfig) -> Result<SpecTables, DatagenError>;   // T18
pub fn attach_effects(j: &mut Joined, tiers: &EffectInputs, report: &mut Report) -> Result<(), DatagenError>; // T19
pub fn validate(j: &Joined, v: &ValidationConfig, report: &mut Report) -> Result<(), DatagenError>;         // T19
```

**Acceptance**: `cargo build -p wfd-datagen && cargo test -p wfd-datagen && cargo run -p wfd-datagen -- --help`.

### T02d `wfd-opt` and `wfd-server` skeletons

**Depends**: T01a, T01b. **Owns** (until handed over): initial `crates/wfd-opt/**`, `crates/wfd-server/**`.

**Provides**:

```rust
// wfd-opt (T32, T33): optimizer.md §1
pub fn optimize(engine: &Engine, data: &GameData, req: &OptRequest, ctl: &JobControl, sink: &dyn EventSink) -> Result<OptResult, OptError>;
pub fn solve_main(kernel: &Kernel, problem: &MainProblem, batch: &BatchCtx, ctl: &JobControl) -> MainSolution; // T32, §5, §8
pub struct BatchCtx { pub theta: Score /* θ_b, optimizer.md §8 */, pub budget: Option<u64> }
// wfd-server (T11, T34)
pub fn router(state: Arc<AppState>) -> axum::Router;                         // every route of server-api.md §3, 501 until wired
pub async fn serve(cfg: Config) -> anyhow::Result<()>;
```

**Acceptance**: `cargo build -p wfd-opt -p wfd-server && cargo test -p wfd-opt -p wfd-server`.

---

## 4. Wave 1

### T03 Mini fixture bundle and scenarios

**Depends**: T01a. **Owns**: `data/fixtures/**`.

**Provides**: `data/fixtures/mini/` in the `data/generated/` JSON layout (pipeline.md §8.1) with the content of
testing.md §11 (weapons incl. a melee weapon with a stance, a stanceless table and a unique override, its synthesized
heavy; about 70 items covering every row of domain-model.md §17; enemies; two frames incl. Jade), `stable_keys.json`,
`rules.json` (the subset of every rule table its scenarios need, transcribed from the spec with citations),
`combo_tables.json`, `frames.json`, `names/en.json`; `data/fixtures/scenarios/*.json` (`CalcInput`s: at least
`braton_prime_lancer_l100_sp.json` = spec 00 §3.8, a Furis Incarnon scenario, a melee stance scenario with a
`MeleePlan`, a Kuva scenario, a shielded Corpus scenario, an overguard Eximus scenario, and
`real/braton_prime_lancer_l100_sp.json` whose items are referenced through `x` uniqueNames so it decodes on the real
bundle); `data/fixtures/reference_weapons.toml` (uniqueNames). Every fixture item cites its source.

**Acceptance**:

```text
cargo test -p wfd-model --test fixtures     # every fixture file deserializes into model types; the spec 00 §3.8 inputs
                                            # (Braton Prime, Lancer, the 8 mods) have exactly the spec values
```

### T03b Rule tables (real data)

**Depends**: T01a. **Owns**: `data/curated/rules/{mult_groups,slot_layout,riven_traits,riven_formula,body_parts,special_dr,toggles,factions,categories,helminth,frames}.toml`.

**Provides**: the rule tables of pipeline.md §6.1 that wave-1 code needs, transcribed with citations: slot layouts and
accepted kinds per equipment class (09 §2, E-9.1-9.3), multiplier groups with stages, DoT inheritance and vulnerability
masks (04 §3, 05 E-5.19), toggle descriptors for T-01..T-29 and proposed T-30..T-32 (00 §6.1, 07 OQ-722, 09 OQ-915),
riven trait mapping and formula constants (04 §8, 09 §12), body-part defaults and head-crit rules (02 §4.5, 05 §4.3),
special DR (05 §4.5, §15), faction-table exceptions (05 §5.1), category bits (wiki `Mods/itemCompatibility/data`),
Helminth restrictions and hosts (09 E-9.32), aura slot overrides (Jade, 09 §2.2).

**Acceptance**:

```text
cargo test -p wfd-model --features curated --test rules_tables   # every file loads into the authoring types; every
                                                                 # toggle id of spec 00 §6.1 has a descriptor; every
                                                                 # slot id of 09 §2.1 appears in a layout
```

### T03c Melee hand tables (real data)

**Depends**: T01a. **Owns**: `data/curated/rules/melee/{heavy_class,heavy_kappa,attack_triggers,stance_co,finishers,combo_specials}.toml`.

**Provides**: 07 §4.4 tables as data: 9-A class heavy table (07 §9.1, `W:Melee` §Heavy Attack Damage), 9-B weapon
heavy factors κ_w (E-7.41 members), 9-C melee attack triggers (07 §9.7), 9-D stance CO ratios r_in (63 rows, 07 §7.6,
04 §2.12), 9-E finisher class multipliers (07 §9.11), combo specials (07 §6: k_tier, M_cap, M_bcap, D0_cb overrides,
innate Initial Combo); each row cites its source line.

**Acceptance**:

```text
cargo test -p wfd-model --features curated --test melee_tables   # files load; 63 stance CO rows; every 9-A class is a
                                                                 # MeleeClass; TV-07-10 inputs present in 9-A
```

### T04 `wfd-data` and legality (09)

**Depends**: T02b (T03 for tests). **Owns**: `crates/wfd-data/**`.

**Provides**: bundle codec (pipeline.md §8.2), `build.rs` embedding (source directory `WFD_DATA_DIR`, default
`data/generated/` when it exists, else `data/fixtures/mini/`), `GameData` and the API of T02b, indexes by key and
unique name (local keys of user bundles included), `x`-map resolution by uniqueName (AD-18), search, catalogue
(frames included)/detail/schema assembly, `resolve` and `to_doc`, and `legality` implementing spec 09 in full: slot
counts and kinds (E-9.1-9.3), compatibility (§3), exclusion and duplicates (§4), ranks and drain (§5), capacity,
polarity and minimum Forma under the capacity modes (§6, E-9.14-9.19), exilus (§7), arcane families (§8), shards
(§9), Helminth with the frame's host rules (§10, E-9.32), set counting (§11, E-9.40-9.41), rivens (§12), Incarnon
(§13), valence (§14, E-9.70). Every rule returns a `Violation` with the 09 rule key and severity; validation never
blocks evaluation (09 §1.5).

**Acceptance**:

```text
WFD_DATA_DIR=data/fixtures/mini cargo test -p wfd-data      # round trip JSON → bundle → GameData; legality cases
                                                            # (incompatible pair, Semi-only mod, second riven, illegal slot,
                                                            # Jade's second aura, Helminth host rule); x-map decode;
                                                            # every TV-09-* vector (tests/spec_vectors_09.rs)
cargo bench -p wfd-data --bench load -- --quick             # decode < 100 ms (fixture size reported)
```

### T05 Data pipeline: sources, fetching, parsing

**Depends**: T02c. **Owns**: `crates/wfd-datagen/src/{fetch,lua,wiki,pe,wfcd}/**`,
`crates/wfd-datagen/tests/fixtures/upstream/**`, `data/sources.lock.toml`.

**Provides**: `PoliteClient` (pipeline.md §3.1, allowed hosts only), cache layout and index, lock file read/write,
the three fetchers, `fetch --from-lock` (wiki by revid, PE by hash, WFCD by tag; `data.lock_input_gone` on a missing
input), WFCD version discovery through `registry.npmjs.org`, the mlua sandbox and Lua → JSON conversion (§4.1, incl.
`Stances/data` with `addSharedCombos` and `Warframes/data`), typed wire structs with typo aliases and `rest` maps, PE
LZMA index and manifest parsing (incl. `ExportWarframes`, `ExportManifest`), WFCD parsing. `wfd-datagen fetch` CLI.

**Acceptance**:

```text
cargo test -p wfd-datagen --lib fetch lua wiki pe wfcd     # local HTTP mock (ETag, revid, 429 + Retry-After, 404, cancel,
                                                           # --from-lock by revid and hash); sandbox limits; typo aliases;
                                                           # every fixture module converts
cargo run -p wfd-datagen -- fetch --from-lock              # fills an empty cache; every input matches the lock
cargo test -p wfd-datagen -- --ignored full_cache          # on that cache: all modules convert; counts 911 weapons,
                                                           # 1,984 attacks, 1,019 enemies, 1,550 mods, 166 arcanes,
                                                           # 132 stances, 126 warframes (DS:wiki)
```

### T06 Engine compile L0-L2, monotone classes, elements

**Depends**: T02a (T03 for tests). **Owns**: `crates/wfd-engine/src/{compile/ (except melee.rs),stats.rs,state.rs,elements.rs}`.

**Spec**: 04 §4 (conditions, T-02 compile rules, E-4.8 classes), 01 §4.10-4.11, domain-model.md §10-§13 and §18.

**Provides**: `Engine::compile_scenario`, `compile_pick`, `compile_build`; `StatVec` fixed point (1e-9 units,
saturating checks in debug), `SparseContrib`, `Coeff`, `MechSet`, `StatMask`; scope resolution per attack and hit kind
(incl. `layers`, `outcome`); vulnerability masks (`vuln_add` per layer and type); the T-02 compile table and class
derivation (domain-model.md §12; class (c) under `simulated` is the constant `u·v·m`); per-effect overrides; finalizers
(sets with `member_scale`, shard counts, Strength incl. caster Strength of buffs, `Per::Resolved` with `Step` and `Pow`
curves and cycle detection); riven, buff, custom-stat and passive compilation; mech and mult slot allocation; MC
mechanic id registration with FNV-1a collision check (engine.md §8.2); `monotone_classes` (readers and downgrades,
domain-model.md §10); `combine_elements` and `ElementSolver` (01 §4.10-4.11, OQ-024).

**Acceptance**:

```text
cargo test -p wfd-engine compile elements               # every example of domain-model.md §16 and §18 compiles to the
                                                        # stated contributions in all three T-02 modes (Galvanized
                                                        # Aptitude under simulated: constant 0.80, no CondStacks);
                                                        # TV-01-01, -07..-11, -19, -21, -23 element vectors pass;
                                                        # monotone_classes downgrades each reader kind of domain-model §10
cargo test -p wfd-engine --test properties incremental permutation
```

### T07 Engine enemy side (05)

**Depends**: T02a (T03 for tests). **Owns**: `crates/wfd-engine/src/enemy/**`.

**Spec**: 05 §4-§15 (records, faction modifiers, scaling NP-2, Steel Path, Eximus, overguard, armor E-15, layers E-12,
vulnerabilities E-5.19, routing ST-9 and E-5.25, shield gate CD-08, special DR, T-28).

**Provides**: `scale_enemy`, `effective_armor`, `armor_dr`, `instance_on_layer` (φ, Viral, Magnetic, `vuln[ℓ][t]` with
masks, armor floor of 1, innate DR, attenuation), `route_layered`, special DR and T-28; compat flags of 05.

**Acceptance**:

```text
cargo test -p wfd-engine enemy                 # spec 00 §3.8 step 1 (HP 73,621.375; A0 1355; DR 0.637574) and step 5
                                               # (per-type values); all TV-05-* stage vectors; one test per compat flag of 05
```

### T08a Engine ranged firing stats and schedule (06)

**Depends**: T02a (T03 for tests). **Owns**: `crates/wfd-engine/src/schedule/{mod,ranged,segment}.rs`,
`crates/wfd-engine/src/resolve/firing.rs`.

**Spec**: 06 §4-§10 (trigger classes, modded timing stats, E-6.11 schedule, Incarnon form, beam ramp, spool, sniper
combo, Kohm, Peacemaker, children and lingering children E-6.25, reload-triggered attacks, steady means).

**Provides**: `firing_stats` (fire rate incl. bow ×2, semi cap, Cannonade lock, charge, burst delay, magazine, ammo
cost and efficiency cap, reload variants, reload lead), `ranged_schedule` (first/last flags, reload records with the
reloaded fraction for `Per::ReloadedFraction`), duty and steady means, segment compression.

**Acceptance**:

```text
cargo test -p wfd-engine schedule::ranged firing   # all TV-06-* vectors; spec 00 §3.8 step 2 (FR 15.328, T_mag 4.8930,
                                                   # duty 0.694733); unit tests per trigger class and reload style
```

### T08b Engine melee: tables, timing, combo, plans, schedule (07)

**Depends**: T02a (T03 for tests; T03c tables are read through `Rules.melee`). **Owns**:
`crates/wfd-engine/src/{compile/melee.rs,resolve/melee.rs,schedule/melee.rs}`.

**Spec**: 07 §4.2 (E-7.0b table resolution), §5 (attack speed, wind-up, swing, throw), §6 (combo counter: tiers, gain,
duration, Power Spike, Initial Combo, heavy consumption and efficiency, Tennokai, combo-scaled bonuses, expected path),
§7 (melee instance base E-7.20, E-7.20b, stance physical, forced procs, flags), §8 (plans, records, algorithm E-7.30,
Melee Duplicate and waves, start state E-7.31, steady E-7.37), §9 (heavy, slam, slide, glaive, gunblade, exalted,
Incarnon melee, finishers).

**Provides**: `resolve_combo_tables`, `melee_plan`, `melee_resolved`, `melee_schedule(ctx, ra, plan, tables,
horizon)` with the hit, heavy and combo records of 07 §8.2.

**Acceptance**:

```text
cargo test -p wfd-engine melee                 # TV-07-01..-06, -08, -09, -11, -13..-16 (TV-07-07 with T13's assembly,
                                               # TV-07-10 and -12 in wfd-datagen); one test per plan of 07 §8.1;
                                               # E-7.0b: unique override, stance combo, FromWeapon fallback
```

### T09 Spec-vector harness and twins (00-09)

**Depends**: T02a, T03. **Owns**: `crates/wfd-engine/tests/{spec_vectors.rs,spec_conformance.rs,spec_vectors/**}`,
`crates/wfd-engine/src/stage_registry.rs`, `xtask/src/spec_vectors.rs`.

**Provides**: twin files for every vector of 00 (§3.8 as `TV-00-01`), 01, 02, 03, 04, 05, 06, 07, 08 and 09
(engine.md §14 format; 09 twins are executed by T04's harness, TV-07-10/-12 by T18's); the transcription check; the
execution harness (stage and scenario kinds, tolerances, `pending = "Txx"` naming the task that makes it pass); the
stage registry entries for every stage function of T02a; `cargo xtask spec-vectors [--check]`.

**Acceptance**:

```text
cargo test -p wfd-engine --test spec_conformance       # every TV id of 00-09 (156 + TV-00-01) has a twin; every expected
                                                       # number occurs verbatim in its vector's Markdown
cargo test -p wfd-engine --test spec_vectors           # runs; non-pending vectors pass; pending count reported per task
cargo xtask spec-vectors --check
```

### T10 Build codes and inspiration links

**Depends**: T02b (T04 for import; T03 for tests). **Owns**: `crates/wfd-buildcode/**`,
`data/curated/aliases/inspiration.toml`.

**Provides**: `encode`, `decode`, `migrate` (server-api.md §7, AD-18: canonical compact JSON, defaults omitted, `x`
map for local keys, `seed` `"auto"` kept, raw DEFLATE, base64url, `wfd1-` prefix); URL forms (`?b=`);
`import_inspiration` (LZString `n=1` and legacy base64; key map of `I:feat` §15; aliases for names, slot ids 0-29,
Incarnon `{mId, pId}`, riven rows with unit rules, `o.*` toggles, `p.*` custom stats; slot mapping exactly as spec 09
§2.4); `ImportReport`; `export_inspiration` (fails with a message when the scenario uses something the inspiration
cannot express).

**Acceptance**:

```text
cargo test -p wfd-buildcode      # proptest round trip encode/decode incl. documents with local keys decoded against a
                                 # bundle with different local numbering; sample codes in tests/codes/v1 decode to stored
                                 # documents; research/test-cases/user-example-furis.json imports with zero unmapped
                                 # entries (fixtures + aliases); export → import identity on supported scenarios
```

### T11 Server skeleton

**Depends**: T02d (T04 for real data). **Owns**: `crates/wfd-server/**` except the routes of T34.

**Provides**: CLI (server-api.md §1.1), startup sequence (§1.2: config, data loading, port fallback, single-instance
detection, browser opening through `webbrowser`, which honours `$BROWSER`), `AppState`, thread pools per AD-15, job
registry with ring buffers, SSE plumbing (framing, `Last-Event-ID` replay, keep-alive), `CancelOnDrop`, static
embedding (`build.rs` copies `frontend/dist` or writes the placeholder into `$OUT_DIR/web`, `include_dir!`,
server-api.md §10), feature `refresh` embedding of `data/curated/` and `data/generated/spec/`, Host/Origin/body-limit
middleware, logging, config file. Routes backed by `wfd-data` (meta, catalog, details, schema, candidates, data
status, `img/{key}` with fetching off by default in tests) work; calculation, optimizer and refresh routes return 501
`api.not_implemented` until T34.

**Acceptance**:

```text
cargo test -p wfd-server         # oneshot tests of every implemented route on the fixture bundle; 501 routes listed;
                                 # SSE replay test with a fake job; 421/403/413 security tests; port fallback;
                                 # second instance detects the first; placeholder page served without frontend/dist
cargo test -p wfd-server --test smoke    # spawns `warframe-dps --no-browser --port 0`, reads the printed URL,
                                         # GET /api/meta → app_id = "warframe-dps"
```

### T12 Frontend foundation

**Depends**: T01b (generated TS types). **Owns**: `frontend/` except `src/features/{loadout,target,results,optimizer,data,about,share}/**`
and their `src/i18n/en/<feature>.json` files.

**Provides**: Vite + React + TS project (frontend.md §1-§2) with the exact versions of frontend.md §1 (TypeScript
6.0.3, typescript-eslint 8.74.0, ESLint 10.15.0, eslint-plugin-formatjs 8.1.4), ESLint (type-aware, incl.
`formatjs/no-literal-string-in-jsx`), Tailwind v4 with `design/tokens.css` and themes, design primitives, damage-type
tokens and glyph sprite, glyph tiles for missing images, i18n infrastructure (react-intl provider, per-feature message
files merged at build, generated `MessageId` type, pseudo-locale), API client and query hooks for every route (typed by
`src/api/generated`; the MC hook draws the seed when `eval.seed` is `"auto"`), SSE client with resume, Zustand store
with the action list of frontend.md §5 (actions may be stubs for features), undo/redo, URL sync, baseline, prefs,
TanStack Router routes, app shell and responsive layout, MSW mocks from `data/fixtures/mini`, Playwright config that
starts `warframe-dps --no-browser --port 0` with `images.fetch = false`.

**Acceptance**:

```text
npm --prefix frontend ci && npm --prefix frontend ls typescript typescript-eslint   # 6.0.3 and 8.74.0
npm --prefix frontend run lint && npm --prefix frontend run typecheck
npm --prefix frontend test -- --run          # store, history, URL sync, seed auto, formatters, token contrast,
                                             # schema-driven renderer
npm --prefix frontend run build              # dist builds; JS ≤ 600 kB gzip at this stage
npx --prefix frontend playwright test e2e/shell.spec.ts   # shell renders with mocks, dark and light, axe clean
```

---

## 5. Wave 2

### T13 Engine resolve (ST-1..ST-7) and assembly

**Depends**: T06, T08a, T08b. **Owns**: `crates/wfd-engine/src/{resolve/ (except firing.rs, melee.rs), assemble.rs, spec_constants.rs}`.

**Spec**: 01 §4 (E-1, E-2, E-3, faction F), 02 §4 (E-4..E-10, critical promotion E-2.6), 04 §2-§3 (E-11, G, G_F),
06 §11.1 and 07 E-7.42 (assembly).

**Provides**: `Engine::resolve` (a full `ResolvedAttack`, with `firing` from T08a and `melee` from T08b) and the stage
functions of T02a (`base_vector`, `ratios`, `quantize`, `base_scalar`, `crit_multiplier`, `crit_chance`, `crit_tiers`,
`part_factor`, `status_chance`, `proc_weights`, `multishot`), kitgun/zaw assembly, progenitor V_t, Incarnon flat split
(D-005), exalted Strength, Phoenix Talons (D-007), compat flags of 01 and 02.

**Acceptance**:

```text
cargo test -p wfd-engine resolve assemble     # all TV-01-* and TV-02-* vectors, TV-07-07; spec 00 §3.8 steps 2-4
                                              # (CC 0.30, CDM 4.4010749, MS 1.90, q values, Y 92.75, Σ H_w 404.332031)
cargo test -p wfd-engine --test properties quantization
```

### T14 Status effects and mechanics

**Depends**: T13, T07. **Owns**: `crates/wfd-engine/src/{status/,mechanics/}`, `crates/wfd-engine/tests/mechanics.rs`.

**Spec**: 03 (rules §4.1, durations and caps CD-06/CD-07, DoT E-13 with "+1" D-025, Heat §10, Blast §11, Gas and
Electricity §12, Magnetic break §13, distinct count §14), 04 §5-§6 (extra hits E-14, mechanic catalogue rows), 02
§4.7 (forced and event procs).

**Provides**: the status functions of T02a and every mechanic kind of domain-model.md §11.2 implementing `MechanicEv`
and `MechanicMc` (engine.md §2.1), incl. `repeat_hit` and `random_multiplier` on crit tiers, `counter_bonus` with
`reset_on` (crit tiers, heavy, other input), hit-outcome gates, `CountWithin` triggers, `mechanic_impl`.

**Acceptance**:

```text
cargo test -p wfd-engine status mechanics     # all TV-03-* vectors; spec 00 §3.8 steps 6-7 (Slash tick 78.533656,
                                              # Xata 162.945809 / 59.055875); TV-04-* extra-hit and mechanic vectors;
                                              # 04-catalogue unit tests for the R-2 constructs
cargo test -p wfd-engine --test mechanics     # every mechanic kind in EV and MC on a minimal scenario
```

### T15a EV kernel and steady metrics

**Depends**: T13, T14, T07, T08a, T08b. **Owns**: `crates/wfd-engine/src/ev/{mod,kernel,poisson,joint,gates,distinct,dot,steady,cond_stacks,hit,score}.rs`.

**Spec**: 08 §5.1-§5.3, §5.5-§5.7, §5.9, §5.10 steady part (A-1..A-9, E-8.2, E-8.3, E-8.7, E-8.9, E-8.10, E-8.11,
E-8.13, E-8.14), 04 E-4.9, 08 §10 hit metrics.

**Provides**: `ev::kernel::point` (engine.md §7.1, incl. outcome-gate branch weights and critical promotion), capped
Poisson helpers, joint crit/armor enumeration, gate splitting (§7.4), distinct count with sequential pellets, DoT per
proc with cap survival, Heat steady strip distribution, `Steady` fidelity (`dps_burst`, `dps_sustained`), steady
condition stacks, `hit_infight` and `hit_fresh` with credited DoT, `Score` ordering.

**Acceptance**:

```text
cargo test -p wfd-engine ev::kernel ev::steady ev::hit   # TV-08-04, -08, -09, -10, -11; spec 00 §3.8 steps 5, 8-9
                                                         # (394.7458, 750.0170, 11,496.26, 7,986.83, state B 1106.3037)
cargo bench -p wfd-engine --bench steady -- --quick      # engine.md §13 Steady target within 2×
```

### T15b EV window walk

**Depends**: T15a. **Owns**: `crates/wfd-engine/src/ev/{window_exact,heat,group,series}.rs`.

**Spec**: 08 §4.4-§4.6, §5.4 (E-8.5, E-8.6, E-8.8), §5.8 (E-8.4, E-8.12), §5.10 window part (E-8.15), §8.1-§8.7, §9
(E-8.23).

**Provides**: `WindowExact` (two-pointer per-status windows, ticks at landing times), Heat window presence, strip
counter and pool, window condition stacks, groups and T-22 units, series, stacks and uptime curves, crit-tier
distribution, per-shot and per-magazine outputs, pending DoT, `Engine::ev`.

**Acceptance**:

```text
cargo test -p wfd-engine ev::window ev::heat ev::group   # TV-08-05, -06, -07, -12; spec 00 §3.8 step 10 (DoT 2,162.23,
                                                         # 34,381.33, 23,885.83)
cargo bench -p wfd-engine --bench window -- --quick
```

### T15c EV bound, fast window, TTK, optimizer kernel

**Depends**: T15b. **Owns**: `crates/wfd-engine/src/ev/{bound,window_fast,ttk,opt_kernel}.rs`,
`crates/wfd-engine/tests/{properties.rs,monotone_readers.rs}`, `crates/wfd-engine/benches/{steady,window,bound}.rs`.

**Spec**: 08 §7.2 (E-8.21), §10 (E-8.26), engine.md §7.2 and §7.9.

**Provides**: `WindowFast`, `Bound`, EV TTK (band, T_EV, κ, shots and ammo to kill, extrapolation,
`shield_regen_possible`), `Kernel::{score, bound}` returning `Score` keys for every metric, the property tests of
testing.md §2.3 and the reader fixtures.

**Acceptance**:

```text
cargo test -p wfd-engine ev::ttk ev::bound         # TV-08-13 EV part; E-8.26 ordering across bands
cargo test -p wfd-engine --test properties monotone bound fast_error
cargo test -p wfd-engine --test monotone_readers   # every reader fixture of testing.md §2.3
cargo bench -p wfd-engine --bench bound -- --quick # engine.md §13 targets within 2×
```

### T16 Monte Carlo simulator

**Depends**: T13, T14, T07, T08a, T08b. **Owns**: `crates/wfd-engine/src/mc/**`, `crates/wfd-engine/benches/mc.rs`.

**Spec**: 08 §6 (E-8.16 to E-8.19, passes V, L, L0), §7.1 (E-8.20), §8.8-§8.9 (E-8.22, timeline), E-8.25 comparison
function.

**Provides**: Philox4x32-10 and keying exactly per E-8.16/E-8.17 (`mc/rng.rs`, `mc/keys.rs`: string mechanic ids
hashed with FNV-1a, ordinals per (run, anchor, mechanic), range errors), event loop with the phases of E-8.18, run
state, pass V (four views), pass L, pass L0 (parity and A0 > 0 only), summaries (E-8.19 incl. median CI99), TTK
distribution, parity outputs, timeline with reload records and Heat episode boundaries, `oracle_compare`,
`Engine::mc`, `Engine::timeline`.

**Acceptance**:

```text
cargo test -p wfd-engine mc                    # TV-08-01 (Philox known answers), TV-08-02 (keyed draws, FNV-1a ids),
                                               # TV-08-14, -15, -16; TV-08-03 and -13 MC parts; thread determinism
                                               # (1 vs 8); L0 TTK on an armored fixture; parity outputs on fixtures
cargo bench -p wfd-engine --bench mc -- --quick     # ≥ 5 M events/s/core within 2×
```

### T17 Outputs, compatibility profile, traces, probes, EV vs MC

**Depends**: T15c, T16. **Owns**: `crates/wfd-engine/src/{outputs.rs,compat.rs,trace.rs,probe.rs,lib.rs}`,
`crates/wfd-engine/tests/{compat.rs,effects_coverage.rs,ev_vs_mc.rs,golden_oracle.rs (comparison side)}`.

**Spec**: 00 §4.5, 08 §8, §11.1, §11.4, §12 (E-8.24).

**Provides**: `Engine::calculate`, `probe`, `explain`; `CalcReport` assembly with every field of engine.md §9; weapon
and enemy panels; condition views; touched register rows; warnings incl. the deviation flag with gap classes; compat
flag wiring for every remaining `Compat` row; the probe registry of engine.md §11.2; coverage suite; EV-vs-MC suite.

**Acceptance**:

```text
cargo test -p wfd-engine                       # whole crate: spec vectors 00-08 with no pending entry left for
                                               # T06-T17; compat flags one by one; coverage matrix rows; insta snapshot
                                               # of CalcReport for the §3.8 scenario
cargo test -p wfd-engine --test ev_vs_mc       # E-8.24 on TV-08-03, -05, -06 and the fixture corpus; gap classes asserted
cargo xtask lint-engine
```

### T18 Data pipeline: join, derive, spec tables, emit, bundle

**Depends**: T04, T05, T03b, T03c. **Owns**: `crates/wfd-datagen/src/{join,derive,spec_tables,emit,report}/**`,
`crates/wfd-datagen/src/main.rs`, `crates/wfd-datagen/tests/spec_vectors_07.rs`, `data/spec_tables.toml`,
`data/validation/base.toml`, `data/generated/**` (generated), `data/NOTICE.md`, `data/curated/rules/attack_roles.toml`,
`data/curated/overrides/{attacks,co_names}/**`.

**Spec**: pipeline.md §5 and §8; 07 E-7.0 (stance normalisation), E-7.0b data links, §9.1 (synthesized heavy, E-7.40,
E-7.41); 09 §2.2 (frames).

**Provides**: join with the precedence table (frames and image paths included), derive (attack keys and roles,
parents and groups, triggers, body parts, compatibility, modular parts, stance normalisation and combo tables, heavy
synthesis, melee triggers and specials, frames, stable keys), spec-table ingestion (CO catalogue, status summary,
faction modifiers, scaling coefficients) with `spec-tables --check`, canonical JSON emit, `REPORT.md`/`report.json`,
licences, `build`, `check`, `diff`, `report` commands; the first committed `data/generated/` (effects empty except
tier 2, items `unmodelled`).

**Acceptance** (independent of curation progress):

```text
cargo run -p wfd-datagen -- fetch --from-lock
cargo run -p wfd-datagen -- build --offline --out target/gen1 && cargo run -p wfd-datagen -- build --offline --out target/gen2
diff -r target/gen1 target/gen2 && diff -r target/gen1 data/generated      # reproducible, and the committed output is current
cargo run -p wfd-datagen -- spec-tables --check
cargo test -p wfd-datagen                       # spec-table snapshots, anchors, stable-key property test, diff snapshot,
                                                # TV-07-10 and TV-07-12 (tests/spec_vectors_07.rs)
cargo test -p wfd-datagen -- --ignored full_cache   # join thresholds, modular assembly vs 1,200 combos, every melee weapon
                                                    # resolves a Neutral combo table
WFD_DATA_DIR=data/generated cargo test -p wfd-data   # the real bundle loads; size and load time reported
```

### T19 Effects machinery, validators, curation tools, effect smoke

**Depends**: T18. **Owns**: `crates/wfd-datagen/src/{effects,curated,validate,render}/**`,
`data/curated/rules/stat_lines.toml`, `data/curated/priority.toml`, `xtask/src/effect_smoke.rs`.

**Provides**: tier 1 stat-line importer (04 §5.1 table as patterns), curated loader (per-item files, Incarnon family
files), binding and number checks, per-rank checks against WFCD `levelStats`, canonical English renderer,
fingerprints, stale detection and auto-patching (pipeline.md §7), class and order derivation checks, the `draft` cap
for R-2 constructs, all validators with passing and failing fixtures, coverage files with ratcheting minimums,
`curate todo|scaffold|rebind|render`, and `cargo xtask effect-smoke` (testing.md §3).

**Acceptance** (independent of curation progress: coverage minimums are 0 until a batch raises its own):

```text
cargo test -p wfd-datagen effects curated validate render    # each validator has a failing and a passing fixture
cargo run -p wfd-datagen -- check                             # exit 0 on the committed data
cargo run -p wfd-datagen -- build --offline --out target/gen && diff -r target/gen data/generated
                                                              # tier 1 maps ≥ 65 % of weapon mods with zero unmapped-
                                                              # line errors on them (REPORT.md coverage table)
cargo xtask effect-smoke                                      # runs over every curated item present (possibly none) and
                                                              # over tier-1 items; mechanic ids of the real bundle distinct
```

### T20-T29 Curation batches

**Depends**: T19 (authoring may start after T01a against `data/curated/schema.json`). Each batch **owns** its
directories and its coverage file `data/validation/coverage/<task>.toml`, follows pipeline.md §6.3, and for every item
either sets `verified` with sources (or `draft` for R-2 constructs) or justifies `unmodelled`/`no_damage_effect` in
`notes`.

| Task | Owns (under `data/curated/`) | Scope |
|---|---|---|
| T20 | `weapon_mod/{rifle,shotgun,sniper,bow,primary,launcher}/`, `overrides/weapons/primary/` | primary weapon mods incl. conditional and Galvanized ones (Primed Chamber, Plated Round); primary hand tables of 06 §4.3 |
| T21 | `weapon_mod/{pistol,secondary}/`, `overrides/weapons/secondary/` | secondary mods (Secondary Enervate); secondary hand tables |
| T22 | `weapon_mod/{melee,exalted}/`, `stance/`, `overrides/weapons/melee/` | melee mods (Melee Duplicate, Melee Animosity, Power Throw), stance corrections, exalted-weapon mods (Chromatic Blade, Reactive Storm) |
| T23 | `weapon_arcane/`, `frame_arcane/`, `set/` | weapon and warframe arcanes (Devouring Attrition, Overwhelming Attrition), set bonuses |
| T24 | `shard/`, `aura/`, `frame_mod/` | archon shards, auras, warframe mods |
| T25 | `buff/`, `custom/` | ability, focus, companion, squad, debuff and environment buffs (T-13..T-19 among them, Strength-scaled flags, subsumable flags), the 26 custom stats |
| T26 | `incarnon/primary/` | Incarnon families of primary weapons (`F:mods` §9; wikitext only, `DS:wiki`) |
| T27 | `incarnon/secondary/` | Incarnon families of secondary weapons |
| T28 | `incarnon/melee/` | Incarnon families of melee weapons (incl. Phenmor-style `count_within` perks as `draft`) |
| T29 | `overrides/enemies/`, `overrides/progenitor.toml`, `aliases/enemies.toml`, `weapon_mod/{archgun,companion,augment}/`, `overrides/weapons/{archgun,companion,exalted}/` | enemy overrides (05 §4.5), progenitor elements, arch-gun, companion and augment mods |

**Acceptance (each)**:

```text
cargo run -p wfd-datagen -- check                         # exit 0
cargo run -p wfd-datagen -- build --offline --out target/gen && diff -r target/gen data/generated   # regenerated and committed
cargo run -p wfd-datagen -- report --coverage data/validation/coverage/<task>.toml   # exit 0: the batch's raised minimums hold
cargo xtask effect-smoke --dir <each owned directory>     # every item of the batch reaches its declared stats in scope
```

### T30 Frontend loadout

**Depends**: T12 (T34 for the live API; MSW before). **Owns**: `frontend/src/features/{loadout,share}/**`,
`frontend/src/i18n/en/{loadout,share}.json`, `frontend/e2e/parity/PAR-{01,03,05..16,30..32}-*.spec.ts`.

**Provides**: every loadout component of frontend.md §4 (weapon picker, variant controls, mod grid with React Aria
DnD and keyboard, mod picker with candidates, stance, combo and melee plan, arcanes, Incarnon track, polarity editor and
capacity bar, riven editor, frame panel with frame picker, shards, Helminth), share and import dialogs (bare code copy,
import report).

**Acceptance**:

```text
npm --prefix frontend run lint && npm --prefix frontend run typecheck && npm --prefix frontend test -- --run
npx --prefix frontend playwright test e2e/parity/PAR-01 e2e/parity/PAR-03 e2e/parity/PAR-05 … e2e/parity/PAR-16 \
    e2e/parity/PAR-30 e2e/parity/PAR-31 e2e/parity/PAR-32 e2e/keyboard.spec.ts     # against MSW; axe clean
```

### T31 Frontend target and results

**Depends**: T12 (as T30). **Owns**: `frontend/src/features/{target,results}/**`, `frontend/src/i18n/en/{target,results}.json`,
`frontend/e2e/parity/PAR-{02,04,17..29}-*.spec.ts`.

**Provides**: enemy picker and panel, schema-driven options (zoom level, melee plan parameters), buffs with caster
Strength, custom stats, conditions control with per-effect overrides, combine-attacks dialog, simulation panel (seed
auto or fixed); results pane (headline cards with MC strip, layer tabs, attack cards and table with parity columns incl.
L0-based TTK, charts with table views, weapon stats panel, hit timeline dialog with Heat episodes, Explain drawer,
assumptions, warnings and violations, baseline bar).

**Acceptance**: as T30 with its parity specs (`PAR-02`, `PAR-04`, `PAR-17` … `PAR-29`), plus `e2e/a11y.spec.ts` for
its pages.

---

## 6. Wave 3

### T32 Optimizer core

**Depends**: T15c, T04. **Owns**: `crates/wfd-opt/src/{problem,pool,dominance,composition,bnb,capacity,keys,batch}.rs`,
`crates/wfd-opt/benches/**`.

**Provides**: optimizer.md §2-§5 and §8: problem compilation, pools via `wfd-data::legality`, status and rule filters
on effects, relevance, dominance, coupled-candidate branching on readers, composition-set enumeration with
`ElementSolver`, the main-slot branch-and-bound on `Score` keys (active-slot compaction, suffix top-r tables,
threshold and keep-set rule, `certify`, capacity and polarity DP), deterministic batches with barrier thresholds and
per-item budgets, f64 memo cache, canonical keys, `solve_main`.

**Acceptance**:

```text
cargo test -p wfd-opt bnb composition dominance capacity batch   # brute-force exactness on 100 random small problems
                                                                 # (certify: same argmax, no ε assumption); keep-set rule
                                                                 # with perturbed WindowFast; reader-coupled branching on
                                                                 # the monotone_readers fixtures; TTK keys across bands;
                                                                 # node_budget results identical for 1, 2 and 8 threads
cargo bench -p wfd-opt -- --quick                                # rifle fixture proven optimal; nodes/s reported
```

### T33 Optimizer driver

**Depends**: T32, T16. **Owns**: `crates/wfd-opt/src/{lib,context,local,portfolio,riven,finalists,events,presets}.rs`,
`crates/wfd-opt/tests/**`.

**Provides**: `optimize` (optimizer.md §1, §6-§11): context groups (stance × combo × plan, Incarnon, frame slots,
shards, parts, progenitor element), block-coordinate ascent with context beam, cross-group local search, per-mix pass,
riven search, buff portfolio, finalists with `WindowExact`, MC confirmation with CRN and E-8.24 flags, diagnostics
(leave-one-out, next-best, condition sensitivity, buff table), `OptEvent` stream, cancellation, time and node budgets,
presets.

**Acceptance**:

```text
cargo test -p wfd-opt              # constraints proptests; cancellation ≤ 100 ms with a valid result; identical results
                                   # for 1, 2 and 8 threads with node_budget, repeated 5 times; inspiration-compatible
                                   # preset applied
cargo run -p wfd-server -- optimize --request data/fixtures/scenarios/opt_rifle.json --node-budget 2000000 --json
```

### T34 Server integration

**Depends**: T11, T17, T33, T10, T18. **Owns**: `crates/wfd-server/src/routes/{calc,timeline,explain,build,optimize,data,img}.rs`
(the rest of the crate stays with T11's files, edited only with a CONTRACTS.md note).

**Provides**: every route of server-api.md §3 wired to the engine (MC passes per `want`, seed `"auto"` handling), the
optimizer, build codes and the data pipeline (refresh jobs with embedded inputs, local keys, activation, rollback,
check); image proxy by key with disk cache; CLI `calc`, `optimize`, `import`, `data *`.

**Acceptance**:

```text
cargo test -p wfd-server           # every route on the fixture bundle (insta snapshots); SSE optimizer run and cancel;
                                   # superseded calc cancels MC; data refresh from a local mock source set with the
                                   # embedded curated and spec inputs swaps the bundle without restart while a job keeps
                                   # its snapshot, and a code made on the refreshed bundle decodes on a fresh AppState
cargo run -p wfd-server -- calc --input data/fixtures/scenarios/braton_prime_lancer_l100_sp.json --json
```

### T35 Frontend optimizer and data pages

**Depends**: T12, T34 (mocks before). **Owns**: `frontend/src/features/{optimizer,data,about}/**`,
`frontend/src/i18n/en/{optimizer,data,about}.json`, `frontend/e2e/parity/PAR-{34..39}-*.spec.ts`, `frontend/e2e/data.spec.ts`.

**Provides**: optimizer page (objective with 08 §10 metrics, constraints with locks on the mirrored grid, groups incl.
melee plan search, buff portfolio, prohibited list with persistence (local keys with uniqueNames) and defaults, run
panel with SSE progress and resume, proof display incl. the ε assumption, log, results with Apply, Compare and
BuildDiff); data page (status, check, refresh with progress and report, activate, rollback); about page.

**Acceptance**: lint, typecheck, unit tests; `npx --prefix frontend playwright test e2e/parity/PAR-34 … e2e/parity/PAR-39
e2e/data.spec.ts` against the real binary with a node budget; axe clean.

### T36 Oracle harness and golden recordings

**Depends**: T17, T10. **Owns**: `tools/oracle/**`, `xtask/src/oracle.rs`, `crates/wfd-engine/tests/golden/oracle/**`.
Runs in the main checkout, where the gitignored inspiration copy `research/external/inspiration/` exists.

**Provides**: testing.md §5: Node + Playwright harness (local inspiration, intercepts, mulberry32 `Math.random` per
08 §11.2, DOM readers, `WFOpt.runFull` recording), scenario corpus generation, `cargo xtask oracle-record`, ≥ 30
recorded oracle-safe scenarios with R ≥ 200 runs and WFOpt results for the oracle weapon set, in the format read by
T17's `golden_oracle.rs` (E-8.25 through `oracle_compare`).

**Acceptance**:

```text
npm --prefix tools/oracle ci && npm --prefix tools/oracle test   # harness unit tests on a committed stub page
cargo test -p wfd-engine --test golden_oracle        # C1-C3 of E-8.25 on the committed recordings (≥ 30 scenarios)
```

### T37 Performance

**Depends**: T15c, T16, T32. **Owns**: `benches/baseline.json` files, `xtask/src/bench.rs`,
`docs/architecture/performance.md`.

**Provides**: `cargo xtask bench-check`; profiling pass on kernel, window walk, MC (interactive 2-thread target) and
branch-and-bound; measured numbers of engine.md §13 and optimizer.md §5.3 recorded.

**Acceptance**: `cargo xtask bench-check` exits 0; `cargo xtask doc-check` (performance.md lists every target of
engine.md §13 with a measurement and the machine).

### T38 End-to-end completion

**Depends**: T30, T31, T35, T34. **Owns**: `frontend/e2e/{a11y,visual,pseudo-locale,share,import}.spec.ts`,
`frontend/e2e/__screenshots__/**`.

**Provides**: every must-have row of frontend.md §6 has a passing `PAR-nn` spec against the real binary; keyboard-only
flows; axe on every page and dialog; visual baselines (dark and light, 375/900/1440 px, glyph tiles); pseudo-locale
run; Furis inspiration-link import; share-code round trip through the Import dialog.

**Acceptance**: `cargo xtask ci` exits 0 end to end.

### T39 Packaging

**Depends**: T34, T38. **Owns**: `xtask/src/{dist,dist_smoke}.rs`, release notes template.

**Provides**: `cargo xtask dist` (`npm ci && npm run build`, then `cargo build --release -p wfd-server`), binaries for
Linux, macOS and Windows, NOTICE and licences next to the binary and on the About page; nothing is published (AD-11).

**Acceptance**:

```text
cargo xtask dist && cargo xtask dist-smoke
# dist-smoke: runs the release binary with an empty temporary --data-dir, --port 0, images.fetch = false,
# BROWSER=<script that records its argument>, and every proxy variable pointing at a closed port (no network);
# asserts: the recorded URL equals the printed one; GET / returns the built index.html (not the placeholder);
# GET /api/meta reports the embedded bundle; `warframe-dps calc --input
# data/fixtures/scenarios/real/braton_prime_lancer_l100_sp.json --json` prints the spec 00 §3.8 numbers within the
# vector tolerance
```

---

## 7. Wave 4

### T40 Accuracy audit

**Depends**: T36, T33, T20-T29. **Owns**: `research/findings/accuracy-audit.md`, `xtask/src/oracle_audit.rs`.

**Provides**: oracle comparisons over the full corpus in both profiles; in-game cases; every disagreement classified
(our bug, inspiration difference already in the register, new register row proposed to the manager, open question);
"beats WFOpt" on the oracle weapon set with the inspiration-compatible preset; `cargo xtask oracle-audit [--check]`.

**Acceptance**:

```text
cargo test -p wfd-engine --test golden_oracle
cargo xtask oracle-audit --check     # fails when a failing comparison or a WFOpt result better than ours has no row in
                                     # the audit's disagreement table; proposed register rows listed for the manager
```

### T41 Documentation pass

**Depends**: all. **Owns**: `docs/architecture/*.md`, `docs/data/*.md` (edits only), `CONTRACTS.md` summary.

**Provides**: these documents updated to match what was built (CONTRACTS.md entries folded in), statuses moved to
`cross-checked` after review, research proposals marked `superseded`.

**Acceptance**: `cargo xtask doc-check` exits 0 (every path, type, command and task id named in the documents exists).

---

## 8. Ownership map

| Path | Owner |
|---|---|
| `Cargo.toml`, `rust-toolchain.toml`, `.gitignore`, `CONTRACTS.md` (structure) | T01a |
| `crates/wfd-model/**` except the rows below | T01a (changes via CONTRACTS.md) |
| `crates/wfd-model/src/{api,message}/**` | T01b |
| `crates/wfd-model/src/compat_generated.rs` (generated) | T01c |
| `xtask/src/{gen_ts,gen_schema}.rs` | T01b |
| `xtask/src/{main,spec_sync,lint_engine,ci,doc_check}.rs`, `xtask/Cargo.toml` | T01c |
| `.cargo/config.toml` (the `xtask` alias) | T01c |
| `xtask/src/spec_vectors.rs` / `effect_smoke.rs` / `oracle.rs` / `bench.rs` / `{dist,dist_smoke}.rs` / `oracle_audit.rs` | T09 / T19 / T36 / T37 / T39 / T40 |
| skeleton of `wfd-engine` / `wfd-data`, `wfd-buildcode` / `wfd-datagen` / `wfd-opt`, `wfd-server` | T02a / T02b / T02c / T02d, then the owners below |
| `crates/wfd-data/**` | T04 |
| `crates/wfd-datagen/src/{fetch,lua,wiki,pe,wfcd}/**`, `tests/fixtures/upstream/**` | T05 |
| `crates/wfd-datagen/src/{join,derive,spec_tables,emit,report}/**`, `main.rs`, `tests/spec_vectors_07.rs` | T18 |
| `crates/wfd-datagen/src/{effects,curated,validate,render}/**` | T19 |
| `crates/wfd-engine/src/{compile/ (except melee.rs),stats.rs,state.rs,elements.rs}` | T06 |
| `crates/wfd-engine/src/enemy/**` | T07 |
| `crates/wfd-engine/src/schedule/{mod,ranged,segment}.rs`, `resolve/firing.rs` | T08a |
| `crates/wfd-engine/src/{compile/melee.rs,resolve/melee.rs,schedule/melee.rs}` | T08b |
| `crates/wfd-engine/src/stage_registry.rs`, `tests/{spec_vectors.rs,spec_conformance.rs,spec_vectors/**}` | T09 |
| `crates/wfd-engine/src/{resolve/ (rest),assemble.rs,spec_constants.rs}` | T13 |
| `crates/wfd-engine/src/{status/,mechanics/}`, `tests/mechanics.rs` | T14 |
| `crates/wfd-engine/src/ev/{mod,kernel,poisson,joint,gates,distinct,dot,steady,cond_stacks,hit,score}.rs` | T15a |
| `crates/wfd-engine/src/ev/{window_exact,heat,group,series}.rs` | T15b |
| `crates/wfd-engine/src/ev/{bound,window_fast,ttk,opt_kernel}.rs`, `tests/{properties,monotone_readers}.rs`, `benches/{steady,window,bound}.rs` | T15c |
| `crates/wfd-engine/src/mc/**`, `benches/mc.rs` | T16 |
| `crates/wfd-engine/src/{lib,outputs,compat,trace,probe}.rs`, `tests/{compat,effects_coverage,ev_vs_mc,golden_oracle}.rs` | T17 |
| `crates/wfd-opt/src/{problem,pool,dominance,composition,bnb,capacity,keys,batch}.rs`, `benches/` | T32 |
| `crates/wfd-opt/src/{lib,context,local,portfolio,riven,finalists,events,presets}.rs`, `tests/` | T33 |
| `crates/wfd-buildcode/**`, `data/curated/aliases/inspiration.toml` | T10 |
| `crates/wfd-server/**` except `src/routes/{calc,timeline,explain,build,optimize,data,img}.rs` | T11 |
| `crates/wfd-server/src/routes/{calc,timeline,explain,build,optimize,data,img}.rs` | T34 |
| `data/fixtures/**` (later tasks add `data/fixtures/mini/extra/<task>.json`), `crates/wfd-model/tests/fixtures.rs` | T03 |
| `data/curated/rules/{mult_groups,slot_layout,riven_traits,riven_formula,body_parts,special_dr,toggles,factions,categories,helminth,frames}.toml` | T03b |
| `data/curated/rules/melee/**` | T03c |
| `data/curated/rules/attack_roles.toml`, `data/curated/overrides/{attacks,co_names}/**` | T18 |
| `data/curated/rules/stat_lines.toml`, `data/curated/priority.toml` | T19 |
| `data/curated/**` item directories | T20-T29 per the table of section 5 |
| `data/spec_tables.toml`, `data/validation/base.toml`, `data/NOTICE.md`, `data/generated/**` | T18 (regenerated by every data task through the build command) |
| `data/validation/coverage/<task>.toml` | the curation task named |
| `data/sources.lock.toml` | T05 (a data task that must run `fetch --update-lock` commits the new lock with its data) |
| `frontend/` (foundation, shell, design, state, api, i18n infrastructure) | T12 |
| `frontend/src/features/{loadout,share}/**` | T30 |
| `frontend/src/features/{target,results}/**` | T31 |
| `frontend/src/features/{optimizer,data,about}/**` | T35 |
| `frontend/e2e/parity/PAR-nn-*.spec.ts` | the feature task of the row (T30, T31, T35) |
| `frontend/e2e/{a11y,visual,pseudo-locale,share,import}.spec.ts`, screenshots | T38 |
| `tools/oracle/**`, `crates/wfd-engine/tests/golden/oracle/**` | T36 |
| `docs/architecture/performance.md`, `crates/wfd-engine/benches/resolve.rs` | T37 |
| `research/findings/accuracy-audit.md`, `crates/wfd-engine/tests/measured.rs`, `research/test-cases/ingame/*.toml` | T40 |
| `docs/architecture/*.md`, `docs/data/*.md` (after wave 3) | T41 |
