Status: draft (architecture)

# Work breakdown

Ordered implementation tasks, one agent each. Every task lists what it depends on, the files it owns (only the owner
writes them), the public interfaces it must provide or honour, and the commands that must pass for acceptance. Tasks
in the same wave run in parallel; a later wave starts when its dependencies are accepted, not when the whole previous
wave is.

Read first: `overview.md` (decisions), `domain-model.md` (types), and the document of your area. The spec
(`docs/mechanics/`) is normative for every number; topic documents 07 and 08 are pending (section 2.4); 09 (build
rules) is a draft and is implemented by T04.

Contents: 1 Waves and critical path · 2 Rules for every task · 3 Wave 0 · 4 Wave 1 · 5 Wave 2 · 6 Wave 3 · 7 Wave 4 ·
8 Ownership map

---

## 1. Waves and critical path

```text
Wave 0  T01 workspace + wfd-model ──► T02 crate skeletons (all public signatures stubbed)
Wave 1  T03 fixtures · T04 wfd-data · T05 datagen sources · T06 engine compile · T07 engine enemy · T08 schedules ·
        T09 spec-vector harness · T10 buildcode · T11 server skeleton · T12 frontend foundation
Wave 2  T13 resolve · T14 status + mechanics · T15 EV · T16 MC · T17 outputs/compat/probes · T18 datagen join/emit ·
        T19 effects machinery · T20-T25 curation · T26 frontend loadout · T27 frontend target + results
Wave 3  T28 optimizer core · T29 optimizer driver · T30 server integration · T31 frontend optimizer + data ·
        T32 oracle harness · T33 performance · T34 e2e completion · T35 packaging
Wave 4  T36 accuracy audit · T37 documentation pass
```

| Task | Depends on |
|---|---|
| T01 | — |
| T02 | T01 |
| T03, T04, T05, T06, T07, T08, T09, T10, T11, T12 | T02 (T09 also T03; T10 import part also T04) |
| T13 | T06 |
| T14 | T13 |
| T15 | T07, T08, T13, T14 |
| T16 | T07, T08, T13, T14 |
| T17 | T15, T16 |
| T18 | T04, T05 |
| T19 | T18 |
| T20-T25 | T19 (may start authoring against the schema after T01; `check` needs T19) |
| T26, T27 | T12 (live API from T30; MSW mocks before) |
| T28 | T15 |
| T29 | T28, T16 |
| T30 | T11, T17, T29, T10, T18 |
| T31 | T12, T30 (mocks before) |
| T32 | T17, T10 |
| T33 | T15, T16, T28 |
| T34 | T26, T27, T31, T30 |
| T35 | T30, T34 |
| T36 | T32, T20-T25 |
| T37 | all |

Critical path: T01 → T02 → T06 → T13 → T14 → T15 → T28 → T29 → T30 → T34 → T35. Curation (T20-T25) is the largest
effort and runs alongside waves 2-3; the mini fixture bundle lets every other task proceed before it finishes.

---

## 2. Rules for every task

### 2.1 Contract freeze

- T01 and T02 define every shared interface: `wfd-model` types, crate public APIs, engine internal types and stage
  signatures, API DTOs, message keys, the curated TOML schema, the bundle JSON layout. After T02 is accepted they are
  frozen.
- A task that needs a change to a frozen interface records it in `CONTRACTS.md` (date, task, change, reason,
  affected tasks), makes the smallest backward-compatible change (new optional field, new enum variant with
  `#[serde(other)]` fallback, new function), and regenerates TypeScript types. A breaking change needs the manager's
  approval before it is merged.
- Workspace dependencies are declared once by T01 in `[workspace.dependencies]`; crates use `dep = { workspace = true }`.
  A new dependency is a `CONTRACTS.md` entry. Frontend dependencies are declared by T12 in `frontend/package.json`.

### 2.2 File ownership

Only a file's owner (section 8) edits it. To avoid conflicts on shared collections:

- Message keys: Rust `MessageKey` variants live in `crates/wfd-model/src/message/<area>.rs`, one file per area owned
  by the area's task; frontend strings live in `frontend/src/i18n/en/<feature>.json`, one file per feature folder,
  merged at build time.
- Fixture data: `data/fixtures/mini/` is owned by T03; a later task that needs more fixture entries adds a new file
  `data/fixtures/mini/extra/<task-id>.json` (the loader merges `extra/*.json`).
- Server routes: one module per route group (`crates/wfd-server/src/routes/<group>.rs`).
- Curated data: each curation task owns disjoint directories of `data/curated/` (section 8).

### 2.3 Definition of done (every task)

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace                     # plus the task's own acceptance commands
cargo xtask spec-sync --check              # once T01 has created it
```

Frontend tasks add `npm --prefix frontend run lint && npm --prefix frontend run typecheck && npm --prefix frontend test -- --run`.
Code cites the spec (`// E-12`, `// D-026`, `// 03 §10.4`) where it implements a rule. Unsourced mechanics are not
added (CLAUDE.md). No commits of `research/external/` or `.cache/`.

### 2.4 Spec-pending work

Topic documents 07 (melee) and 08 (evaluation and outputs) are not written. Tasks implement
`docs/mechanics/00-overview.md` and the research findings for those areas, mark the code `// SPEC-PENDING(07|08):
<what>`, and keep the interfaces of this architecture. `cargo xtask spec-sync` lists the markers; T37 (or the task
that follows the document's arrival) clears them. Spec 09 (build rules) is a draft: implement it as written; its
proposed toggles T-30 and T-31 are `Proposed` entries of `TOGGLE_MAP` (domain-model.md §9.3).

---

## 3. Wave 0

### T01 Workspace and `wfd-model` contracts

**Depends**: none. **Owns**: `Cargo.toml`, `rust-toolchain.toml`, `.gitignore` (adds `.cache/`, `frontend/dist`,
`frontend/node_modules`, `target/`), `CONTRACTS.md`, `crates/wfd-model/**`, `xtask/**` (initial commands),
`frontend/src/api/generated/**` (generated output only).

**Provides**:

- Workspace with the eight members of overview.md §3 (empty `lib.rs`/`main.rs` for crates other than `wfd-model`),
  `[workspace.dependencies]` for every crate of overview.md §3.1 and frontend-independent dev tools (proptest,
  insta, criterion), `[workspace.lints]` (`unsafe_code = "forbid"`, `clippy::float_cmp = "warn"` in engine crates),
  release profile (`lto = "fat"`, `codegen-units = 1`).
- `wfd-model` exactly as `domain-model.md` §2-§11, §14, §15.3 (types, serde, `ts` and `schema` features), plus:
  - `src/api/{calc,opt,data,catalog,schema,buildcode,error}.rs`: every request/response shape of `server-api.md`
    (`CalcInput`, `CalcRequest`, `CalcReport` and its parts, `TimelineRequest`, `Timeline`, `ExplainRequest`,
    `Explanation`, `CandidatesRequest`, `Candidate`, `Meta`, `Catalog`, `WeaponDetail`, `ItemDetail`, `EnemyDetail`,
    `ScenarioSchema`, `EncodeResponse`, `DecodeResponse`, `OptRequest`, `OptEvent`, `OptResult`, `JobStatus`,
    `DataStatus`, `DataCheck`, `DataEvent`).
  - `src/message/mod.rs`: `pub struct Message { pub key: MessageKey, pub args: BTreeMap<Box<str>, MsgArg>, pub severity: Option<Severity> }`,
    `MessageKey` assembled from per-area enums (`message/{api,doc,engine,opt,data,import}.rs`).
  - `src/scenario.rs`: `TOGGLE_MAP` and `CompatFlags` with one flag per register row marked `Compat`.
  - `src/stats.rs`: the `define_stats!` registry of domain-model.md §10 with metadata accessors
    `StatKey::meta(self) -> &'static StatMeta { toml: &str, symbol: &str, aggregate: Aggregate, monotone: Monotone, owner: &str }`
    and `StatKey::parse(&str) -> Option<StatKey>` (handles `"forced.SLA"`, `"stack_cap.COR"`, `"phys.IMP"` …).
  - `src/curated.rs`: the authoring form of curated files (`CuratedItemFile`, `IncarnonFamilyFile`, header fields of
    domain-model.md §15.1, binding fields), `deny_unknown_fields`.
- `xtask` commands: `gen-ts [--check]` (ts-rs export to `frontend/src/api/generated/`), `gen-schema [--check]`
  (`data/curated/schema.json`), `spec-sync [--check]` (CompatFlags vs spec 00 §5.2 rows marked `Compat`; `TOGGLE_MAP`
  vs spec 00 §6.1; `SPEC-PENDING` listing), `lint-engine` (I-1, I-2; passes on empty crates), `ci` (runs testing.md §10
  gates that exist).

**Acceptance**:

```text
cargo build --workspace && cargo clippy --workspace --all-targets -- -D warnings
cargo test -p wfd-model            # includes: every TOML block of domain-model.md §16 parses into the authoring types;
                                   # every JSON example of server-api.md parses; serde round trip of every API type
cargo xtask gen-ts --check && cargo xtask gen-schema --check && cargo xtask spec-sync --check
```

### T02 Crate skeletons and internal contracts

**Depends**: T01. **Owns** (until handed over to the implementing tasks of section 8): the initial version of every
file listed below. Bodies are `todo!("Txx")` naming the implementing task; tests that need them are `#[ignore =
"Txx"]`.

**Provides** (exact public signatures; the implementing task may add private items but not change these without a
`CONTRACTS.md` entry):

```rust
// wfd-data (T04)
pub struct GameData { /* bundle + indexes */ }
impl GameData {
    pub fn embedded() -> Result<GameData, DataError>;
    pub fn from_bytes(bytes: &[u8]) -> Result<GameData, DataError>;
    pub fn from_json_dir(dir: &Path) -> Result<GameData, DataError>;      // merges extra/*.json
    pub fn bundle(&self) -> &Bundle;
    pub fn by_key(&self, k: StableKey) -> Option<Entity<'_>>;
    pub fn by_unique_name(&self, n: &str) -> Option<Entity<'_>>;
    pub fn search(&self, q: &SearchQuery) -> Vec<SearchHit>;
    pub fn catalog(&self) -> Catalog;                                     // /api/catalog
    pub fn weapon_detail(&self, k: StableKey) -> Option<WeaponDetail>;    // and item_detail, enemy_detail
    pub fn schema(&self) -> ScenarioSchema;
    pub fn candidates(&self, input: &CalcInput, slot: SlotId, query: Option<&str>, stat: Option<&str>) -> Vec<Candidate>;
    pub fn resolve(&self, input: &CalcInput) -> Result<Resolved, Vec<Message>>;   // Resolved { build, scenario, violations }
    pub fn to_doc(&self, b: &Build, s: &Scenario) -> CalcInput;
}
pub mod legality {                                                                  // spec 09
    pub fn check(gd: &GameData, b: &Build, s: &Scenario) -> Vec<Violation>;
    pub fn check_slot(gd: &GameData, b: &Build, slot: SlotId, pick: &Pick) -> Vec<Violation>;
    pub fn min_forma(gd: &GameData, b: &Build, mode: CapacityMode) -> FormaReport;    // E-9.14-9.16
    pub fn set_counts(gd: &GameData, b: &Build) -> SmallVec<[(SetId, u8); 4]>;        // E-9.40
    pub fn riven_range(gd: &GameData, weapon: WeaponId, riven: &Riven) -> Vec<TraitRange>;   // §12
}
pub fn encode_bundle(b: &Bundle) -> Vec<u8>;  pub fn decode_bundle(bytes: &[u8]) -> Result<Bundle, DataError>;

// wfd-engine (T06, T07, T08, T13-T17): engine.md §1 API, plus
pub struct StageCtx<'a> { pub rules: &'a Rules, pub compat: CompatFlags, pub trace: &'a dyn TraceSink }
// resolve/
pub fn base_vector(ctx: &StageCtx, atk: &Attack, w: &AssembledWeapon, sv: &StatVecF) -> BaseVector;          // E-1
pub fn combine_elements(entries: &[OrderedElem]) -> ElemOutcome;                                              // 01 §4.11
pub fn ratios(ctx: &StageCtx, base: &BaseVector, sv: &StatVecF, stance_phys: [f64; 3], el: &ElemOutcome) -> DamageVec; // E-2
pub fn quantize(ctx: &StageCtx, ratio: &DamageVec) -> DamageVec;                                             // E-3
pub fn base_scalar(ctx: &StageCtx, base: &BaseVector, a: f64, g_f: f64, co: &CoSpec) -> ScalarY;              // E-11
pub fn crit_multiplier(cdm0: f64, cd_base_add: f64, cd_rel: f64, cd_abs: f64, m_cdf: f64) -> CritMult;        // E-4, NP-1
pub fn crit_chance(cc0: f64, cc_base_add: f64, cc_rel: f64, cc_abs: f64, cc_floor: f64, final_mul: f64) -> f64; // E-5
pub fn crit_tiers(cc: f64, p_vig: f64) -> TierDist;                                                           // E-6
pub fn part_factor(p_part: f64, k_wp: f64, wp: f64, hs: f64, m_son: f64) -> f64;                              // E-7
pub fn status_chance(sc0: f64, sc_base_add: f64, sc_rel: f64, sc_abs: f64, m_sc: f64) -> f64;                 // E-8
pub fn proc_weights(q: &DamageVec, immune: StatusSet) -> [f64; N_ST];                                         // E-9
pub fn multishot(ms0: f64, ms_base_add: f64, sum_ms: f64, locked: bool) -> f64;                               // E-10
// enemy/
pub fn scale_enemy(ctx: &StageCtx, e: &Enemy, setup: &EnemyInstance) -> ScaledEnemy;                          // 05 §6-§8
pub fn np2_blend(f1: f32, f2: f32, s: f32) -> f32;                                                            // NP-2
pub fn effective_armor(a0: f64, strips: &ArmorStrips, heat_strip: f64, n_cor: u8) -> f64;                     // E-15
pub fn armor_dr(a: f64) -> f64;
pub fn instance_on_layer(ctx: &StageCtx, w: &DamageVec, layer: Layer, st: &TargetStateView, en: &ScaledEnemy) -> f64; // E-12
pub fn route_layered(ctx: &StageCtx, dmg: &LayeredHit, pools: &mut Pools, t: f64) -> RouteOut;                // ST-9, CD-08
// status/
pub fn dot_tick(ctx: &StageCtx, snap: &ProcSnapshot, s: StatusType) -> f64;                                   // E-13
pub fn extra_hit(ctx: &StageCtx, parent: &HitWeaponSide, xh: &ExtraHitResolved) -> ExtraHitOut;              // E-14
// schedule/
pub fn ranged_schedule(ctx: &StageCtx, ra: &ResolvedAttack, horizon: f64) -> Schedule;                       // 06 E-6.11
pub fn melee_schedule(ctx: &StageCtx, ra: &ResolvedAttack, stance: Option<&StanceCombo>, horizon: f64) -> Schedule; // 07 (pending)
// stage registry for spec vectors
pub fn stage_registry() -> &'static [(&'static str, fn(&serde_json::Value) -> Result<serde_json::Value, StageError>)];

// wfd-opt (T28, T29): optimizer.md §1
pub fn optimize(engine: &Engine, data: &GameData, req: &OptRequest, ctl: &JobControl, sink: &dyn EventSink) -> Result<OptResult, OptError>;

// wfd-buildcode (T10)
pub fn encode(input: &CalcInput) -> String;                                  // "wfd1-…"
pub fn decode(text: &str) -> Result<(CalcInput, DecodeReport), CodeError>;   // our codes and URLs only
pub fn migrate(v: u16, raw: serde_json::Value) -> Result<CalcInput, CodeError>;
pub fn import_inspiration(url_or_param: &str, data: &GameData) -> Result<(CalcInput, ImportReport), CodeError>;
pub fn export_inspiration(input: &CalcInput, data: &GameData) -> Result<String, CodeError>;

// wfd-datagen (T05, T18, T19): pipeline.md §2 library API (fetch, build, diff, Progress)

// wfd-server (T11, T30)
pub fn router(state: Arc<AppState>) -> axum::Router;                         // every route of server-api.md §3, 501 until wired
pub async fn serve(cfg: Config) -> anyhow::Result<()>;
```

Engine internal types are created with their fields as in engine.md §3-§8 (`CompiledScenario`, `CompiledItem`,
`SparseContrib`, `Coeff`, `StatVec`, `StatVecF`, `MechSet`, `ElemOutcome`, `ResolvedAttack` and its parts,
`Schedule`, `ShotRec`, `ReloadRec`, `Segment`, `CycleStats`, `ScaledEnemy`, `PointState`, `Workspace`, `EvalUnit`,
`Pick`, `Kernel`, `ObjectiveSpec`, `Fidelity`, `BuildPoint`, `ProbePath`, `TraceSink`).

**Acceptance**:

```text
cargo build --workspace && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
cargo doc --workspace --no-deps          # every public item documented with its spec reference or owning task
```

---

## 4. Wave 1

### T03 Mini fixture bundle and scenarios

**Depends**: T02. **Owns**: `data/fixtures/**`.

**Provides**: `data/fixtures/mini/` in the `data/generated/` JSON layout (pipeline.md §8.1) with the content of
testing.md §11 (weapons, about 70 items covering every row of domain-model.md §17, enemies), `stable_keys.json`,
`rules.json` (status rules transcribed from 03 §4.1, faction table from 05 §5.1, scaling from 05 §6.1, mult groups of
04 §3, slot layouts, toggles, riven traits), `names/en.json`; `data/fixtures/scenarios/*.json` (`CalcInput`s, at least
`braton_prime_lancer_l100_sp.json` = spec 00 §3.8, a Furis Incarnon scenario, a melee stance scenario, a Kuva
scenario, a shielded Corpus scenario, an overguard Eximus scenario); `data/fixtures/reference_weapons.toml`. Values
come from the cached wiki modules (`research/external/cache/poc/out/`) and the spec; every fixture item cites its
source in a `sources` field.

**Acceptance**:

```text
cargo test -p wfd-model --test fixtures     # every fixture file deserializes into model types; the spec 00 §3.8 inputs
                                            # (Braton Prime, Lancer, the 8 mods) have exactly the spec values
```

### T04 `wfd-data`

**Depends**: T02 (T03 for tests). **Owns**: `crates/wfd-data/**`.

**Provides**: bundle codec (pipeline.md §8.2), `build.rs` embedding (source directory `WFD_DATA_DIR`, default
`data/generated/` when it exists, else `data/fixtures/mini/`), `GameData` and the API of T02, indexes by key and
unique name, search (localized substring + fuzzy, by kind, faction, stat key), catalogue/detail/schema assembly,
`resolve` (returning `Resolved { build, scenario, violations }`, domain-model.md §9.2) and `to_doc`, and
`legality` implementing spec 09 in full: slot counts and kinds (E-9.1-9.3), compatibility tree and tags (§3),
exclusion graph and duplicates (§4), ranks, drain and per-rank values (§5), polarity drain, capacity and minimum Forma
(§6: `pub fn min_forma(..) -> FormaReport`, shared with the optimizer), exilus (§7), arcane families (§8), shards
(§9), Helminth restriction (§10), set counting (§11, `pub fn set_counts(..)`, read by the engine's finalizer), riven
legality and value ranges (§12), Incarnon availability (§13), valence (§14). Every rule returns a `Violation` with the
09 rule key and severity; validation never blocks evaluation (09 §1.5).

**Acceptance**:

```text
WFD_DATA_DIR=data/fixtures/mini cargo test -p wfd-data      # round trip JSON → bundle → GameData; legality cases
                                                            # (incompatible pair, Semi-only mod, second riven, illegal slot);
                                                            # every TV-09-* vector (tests/spec_vectors_09.rs)
cargo bench -p wfd-data --bench load -- --quick             # decode < 100 ms (fixture size reported)
```

### T05 Data pipeline: sources and parsing

**Depends**: T02. **Owns**: `crates/wfd-datagen/src/{fetch,lua,wiki,pe,wfcd}/**`, `crates/wfd-datagen/tests/fixtures/upstream/**`,
`data/sources.lock.toml`.

**Provides**: `PoliteClient` (pipeline.md §3.1), cache layout and index, lock file read/write, the three fetchers,
the mlua sandbox and Lua → JSON conversion (§4.1), typed wire structs with typo aliases and `rest` maps, PE LZMA index
and manifest parsing, WFCD parsing. `wfd-datagen fetch` CLI.

**Acceptance**:

```text
cargo test -p wfd-datagen --lib fetch lua wiki pe wfcd     # local HTTP mock (ETag, revid, 429 + Retry-After, 404, cancel);
                                                           # sandbox limits; typo aliases; every fixture module converts
cargo test -p wfd-datagen -- --ignored full_cache          # local only, when research/external/cache exists: all modules
                                                           # convert; counts 911 weapons, 1,984 attacks, 1,019 enemies,
                                                           # 1,550 mods, 166 arcanes, 132 stances (DS:wiki)
```

### T06 Engine compile (L0-L2), state variables, elements

**Depends**: T02 (T03 for tests). **Owns**: `crates/wfd-engine/src/{compile/,stats.rs,state.rs,elements.rs}`.

**Provides**: `Engine::compile_scenario`, `compile_pick`, `compile_build`; `StatVec` fixed point (1e-9 units,
saturating checks in debug), `SparseContrib`, `Coeff`, `MechSet`, `StatMask`; scope resolution per attack and hit kind;
the T-02 compile table and class derivation (domain-model.md §12); per-effect overrides; finalizers (sets with
`member_scale`, shard counts, Strength, `Per::Resolved` with cycle detection); riven, buff, custom-stat and passive
compilation; mech and mult slot allocation; per-scenario monotone classes; `combine_elements` and `ElementSolver`
(01 §4.10-4.11, OQ-024).

**Acceptance**:

```text
cargo test -p wfd-engine compile elements               # every example of domain-model.md §16 compiles to the stated
                                                        # contributions in all three T-02 modes; TV-01-01, -07..-11,
                                                        # -19, -21, -23 element vectors pass
cargo test -p wfd-engine --test properties incremental permutation
```

### T07 Engine enemy side (05)

**Depends**: T02. **Owns**: `crates/wfd-engine/src/enemy/**`.

**Provides**: `scale_enemy` (NP-2 f32 blend, Steel Path incl. flat bonuses D-055, Eximus, party, overguard with
`q = L − 1` D-042), `effective_armor` (E-15 incl. clamp, ability strips, Corrosive Projection, T-07), `armor_dr`,
`instance_on_layer` (E-12 enemy side incl. φ, Viral, Magnetic, V, armor floor of 1, innate DR, attenuation),
`route_layered` (ST-9, Toxin routing and exceptions, overflow, shield gate CD-08), special DR and T-28; compat flags
D-042..D-055.

**Acceptance**:

```text
cargo test -p wfd-engine enemy                 # spec 00 §3.8 step 1 (HP 73,621.375; A0 1355; DR 0.637574) and step 5
                                               # (per-type values); all TV-05-* stage vectors; one test per compat flag of 05
```

### T08 Engine schedules (06, 07)

**Depends**: T02. **Owns**: `crates/wfd-engine/src/schedule/**`, `crates/wfd-engine/src/resolve/firing.rs`,
`crates/wfd-engine/src/resolve/melee.rs`.

**Provides**: modded timing stats (06 §5: fire rate incl. bow ×2, semi cap, Cannonade lock, charge, burst delay,
magazine, ammo cost and efficiency cap, reload variants, reload lead), `ranged_schedule` (06 E-6.11, first/last
flags, Incarnon form, beam ramp, sniper combo, spool, Kohm, Peacemaker, children, reload-triggered attacks), duty and
steady means (06 §10), segment compression; `melee_schedule` from `wiki-melee-mechanics.md` §1-§5 (attack speed,
stance hits and timing, combo counter, heavy attacks with HAE, slams), marked SPEC-PENDING(07).

**Acceptance**:

```text
cargo test -p wfd-engine schedule firing        # all TV-06-* vectors; spec 00 §3.8 step 2 (FR 15.328, T_mag 4.8930,
                                                # duty 0.694733); unit tests per trigger class and reload style
```

### T09 Spec-vector harness and twins

**Depends**: T02, T03. **Owns**: `crates/wfd-engine/tests/{spec_vectors.rs,spec_conformance.rs,spec_vectors/**}`,
`crates/wfd-engine/src/stage_registry.rs`, `xtask/src/spec_vectors.rs`.

**Provides**: twin files for every vector of 00 (§3.8 as `TV-00-01`), 01, 02, 03, 04, 05, 06 and 09 (engine.md §14
format; 09 twins are executed by T04's `wfd-data` harness);
the transcription check; the execution harness (stage and scenario kinds, tolerances, `pending = "Txx"` marking);
the stage registry entries for every stage function of T02; `cargo xtask spec-vectors [--check]` (extraction of
fenced `testvector` blocks).

**Acceptance**:

```text
cargo test -p wfd-engine --test spec_conformance       # every TV id has a twin; every expected number occurs verbatim
cargo test -p wfd-engine --test spec_vectors           # runs; non-pending vectors pass; pending count reported per task
cargo xtask spec-vectors --check
```

### T10 Build codes and inspiration links

**Depends**: T02 (T04 for import; T03 for tests). **Owns**: `crates/wfd-buildcode/**`,
`data/curated/aliases/inspiration.toml`.

**Provides**: `encode`, `decode`, `migrate` (server-api.md §7, AD-18: canonical compact JSON, defaults omitted,
raw DEFLATE, base64url, `wfd1-` prefix); URL forms (`?b=`); `import_inspiration` (LZString `n=1` and legacy base64;
key map of `I:feat` §15; aliases for names, slot ids 0-29, Incarnon `{mId, pId}`, riven rows with unit rules, `o.*`
toggles, `p.*` custom stats; slot mapping exactly as spec 09 §2.4 (0-7 → `gen0`…`gen7`, 8 → `exilus`, 9 → `stance`,
10 → `arcane0`, 11-12 → `farcane0/1`, 13-16 → `fgen0`…`fgen3`, 17 → `aura0`, 18 → `genesis_installed`, 19-22 →
`inc2`…`inc5`, 23-27 → `shard0`…`shard4`, 28-29 → arch-gun `arcane0/1`); `ImportReport`); `export_inspiration`
(fails with a message when the scenario uses something the inspiration cannot express).

**Acceptance**:

```text
cargo test -p wfd-buildcode      # proptest round trip encode/decode; sample codes in tests/codes/v1 decode to stored
                                 # documents; research/test-cases/user-example-furis.json imports with zero unmapped
                                 # entries (on fixtures + aliases); export → import identity on supported scenarios
```

### T11 Server skeleton

**Depends**: T02 (T04 for real data). **Owns**: `crates/wfd-server/**` (handed to T30 for the integration routes).

**Provides**: CLI (server-api.md §1.1), startup sequence (§1.2: config, data loading, port fallback, single-instance
detection, browser opening), `AppState`, thread pools, job registry with ring buffers, SSE plumbing (framing,
`Last-Event-ID` replay, keep-alive), `CancelOnDrop`, static embedding with placeholder (`build.rs`), Host/Origin/body
limit middleware, logging, config file. Routes backed by `wfd-data` (meta, catalog, details, schema, candidates,
data status) work; calculation, optimizer and refresh routes return 501 `api.not_implemented` until T30.

**Acceptance**:

```text
cargo test -p wfd-server         # oneshot tests of every implemented route on the fixture bundle; 501 routes listed;
                                 # SSE replay test with a fake job; 421/403/413 security tests; port fallback;
                                 # second instance detects the first
cargo run -p wfd-server -- --no-browser --port 0 &  curl -s http://127.0.0.1:<port>/api/meta   # app_id = warframe-dps
```

### T12 Frontend foundation

**Depends**: T02 (generated TS types). **Owns**: `frontend/` except `src/features/{loadout,target,results,optimizer,data,share}/**`
and their `src/i18n/en/<feature>.json` files.

**Provides**: Vite + React + TS project (frontend.md §1-§2), `package.json` with every dependency of frontend.md §1,
ESLint (incl. `formatjs/no-literal-string-in-jsx`), Tailwind v4 with `design/tokens.css` and themes, design primitives,
damage-type tokens and glyph sprite, i18n infrastructure (react-intl provider, per-feature message files merged at
build, generated `MessageId` type, pseudo-locale), API client and query hooks for every route (typed by
`src/api/generated`), SSE client with resume, Zustand store with the action list of frontend.md §5 (actions may be
stubs for features), undo/redo, URL sync, baseline, prefs, TanStack Router routes, app shell and responsive layout,
MSW mocks from `data/fixtures/mini`, Playwright config that starts `warframe-dps --no-browser --port 0`.

**Acceptance**:

```text
npm --prefix frontend ci && npm --prefix frontend run lint && npm --prefix frontend run typecheck
npm --prefix frontend test -- --run          # store, history, URL sync, formatters, token contrast, schema-driven renderer
npm --prefix frontend run build              # dist builds; JS ≤ 600 kB gzip at this stage
npx --prefix frontend playwright test e2e/shell.spec.ts   # shell renders with mocks, dark and light, axe clean
```

---

## 5. Wave 2

### T13 Engine resolve (ST-1..ST-7) and assembly

**Depends**: T06. **Owns**: `crates/wfd-engine/src/{resolve/ (except firing.rs, melee.rs), assemble.rs, spec_constants.rs}`.

**Provides**: `Engine::resolve` and the stage functions of T02 (`base_vector`, `ratios`, `quantize`, `base_scalar`,
`crit_multiplier`, `crit_chance`, `crit_tiers`, `part_factor`, `status_chance`, `proc_weights`, `multishot`),
kitgun/zaw assembly, progenitor V_t, Incarnon flat split (D-005), exalted Strength, Phoenix Talons (D-007), compat
flags D-001..D-024 that belong to these stages.

**Acceptance**:

```text
cargo test -p wfd-engine resolve assemble     # all TV-01-* and TV-02-* vectors; spec 00 §3.8 steps 2-4
                                              # (CC 0.30, CDM 4.4010749, MS 1.90, q values, Y 92.75, Σ H_w 404.332031)
cargo test -p wfd-engine --test properties quantization
```

### T14 Status effects and mechanics

**Depends**: T13. **Owns**: `crates/wfd-engine/src/{status/,mechanics/}`.

**Provides**: status rules from the bundle (03 §4.1), durations and caps (CD-06, CD-07), DoT tick E-13 (seed, "+1"
D-025, inheritance, elemental scaling, faction twice, status damage, `M_dot`), Heat episode, pool, inherit and strip
ramp (03 §10), Blast fuse and detonation (03 §11), Gas and Electricity (03 §12), Magnetic break proc (03 §13),
distinct count (03 §14), extra hits E-14, and every mechanic kind of domain-model.md §11.2 with an `ev` and an `mc`
function each.

**Acceptance**:

```text
cargo test -p wfd-engine status mechanics     # all TV-03-* vectors; spec 00 §3.8 steps 6-7 (Slash tick 78.533656,
                                              # Xata 162.945809 / 59.055875); TV-04-* extra-hit vectors
cargo test -p wfd-engine --test mechanics     # every mechanic kind in EV and MC on a minimal scenario
```

### T15 EV evaluator and optimizer kernel

**Depends**: T07, T08, T13, T14. **Owns**: `crates/wfd-engine/src/ev/**`, `crates/wfd-engine/benches/{steady,window,bound}.rs`.

**Provides**: the kernel and the four fidelities (engine.md §7), Poisson helpers, joint DR and crit enumeration, gate
splitting, condition-stack expectations (04 E-4.9), window walk, Heat mean, groups (T-22), EV TTK, hit metrics,
`Engine::ev`, `Kernel::{score, bound}`. SPEC-PENDING(08) for algorithm details 08 will fix.

**Acceptance**:

```text
cargo test -p wfd-engine ev                         # spec 00 §3.8 steps 5, 8-10 (394.7458, 750.0170, 11,496.26,
                                                    # 7,986.83, state B 1106.3037, DoT 2,162.23, 34,381.33, 23,885.83)
cargo test -p wfd-engine --test properties monotone bound fast_error
cargo bench -p wfd-engine --bench steady --bench window --bench bound -- --quick   # engine.md §13 targets within 2×
```

### T16 Monte Carlo simulator

**Depends**: T07, T08, T13, T14. **Owns**: `crates/wfd-engine/src/mc/**`, `crates/wfd-engine/benches/mc.rs`,
`crates/wfd-engine/tests/ev_vs_mc.rs`.

**Provides**: Philox4x32-10 with known-answer tests, keying (U-6), event loop with CD-04 phases, state, views in one
pass, layered pass, summaries, timeline, parity outputs (spec 00 §4.5), `Engine::mc`, `Engine::timeline`.

**Acceptance**:

```text
cargo test -p wfd-engine mc                    # Random123 KATs; thread determinism (1 vs 8); parity outputs on fixtures
cargo test -p wfd-engine --test ev_vs_mc       # spec 00 §4.4 tolerance on the golden scenario corpus (needs T15)
cargo bench -p wfd-engine --bench mc -- --quick     # ≥ 5 M events/s/core within 2×
```

### T17 Outputs, compatibility profile, traces, probes

**Depends**: T15, T16. **Owns**: `crates/wfd-engine/src/{outputs.rs,compat.rs,trace.rs,probe.rs,lib.rs}`,
`crates/wfd-engine/tests/{compat.rs,effects_coverage.rs,effect_smoke.rs,golden_oracle.rs (comparison side)}`.

**Provides**: `Engine::calculate`, `probe`, `explain`; `CalcReport` assembly with every field of engine.md §9; weapon
and enemy panels; condition views; touched register rows; warnings; compat flag wiring for every remaining `Compat`
row; the probe registry of engine.md §11.2; coverage and smoke test suites.

**Acceptance**:

```text
cargo test -p wfd-engine                       # whole crate: spec vectors 00-06 with no pending entry left for T13-T17;
                                               # compat flags one by one; coverage matrix rows; snapshot of CalcReport for
                                               # the §3.8 scenario (insta)
cargo xtask lint-engine
```

### T18 Data pipeline: join, derive, spec tables, emit, bundle

**Depends**: T04, T05. **Owns**: `crates/wfd-datagen/src/{join,derive,spec_tables,emit,report}/**`, `crates/wfd-datagen/src/main.rs`,
`data/spec_tables.toml`, `data/validation.toml`, `data/generated/**` (generated), `data/NOTICE.md`,
`data/curated/rules/{attack_roles,categories}.toml`, `data/curated/overrides/{attacks,co_names}/**`.

**Provides**: pipeline.md §5 and §8: join with the precedence table, derive (attack keys and roles, parents and
groups, triggers, body parts, compatibility, modular parts, stable keys), spec-table ingestion (CO catalogue, status
summary, faction modifiers, scaling coefficients), canonical JSON emit, `REPORT.md`/`report.json`, licences, `build`,
`check`, `diff`, `report`, `spec-tables` commands; the first committed `data/generated/` (effects empty except tier 2,
items `unmodelled`).

**Acceptance**:

```text
cargo run -p wfd-datagen -- build --offline && git diff --stat data/generated   # reproducible: a second run changes nothing
cargo test -p wfd-datagen                       # join thresholds (with full cache), modular assembly vs 1,200 combos,
                                                # spec-table snapshots, anchors, stable-key property test, diff snapshot
WFD_DATA_DIR=data/generated cargo test -p wfd-data   # the real bundle loads; size and load time reported
```

### T19 Effects machinery: tiers, validators, curation tools

**Depends**: T18. **Owns**: `crates/wfd-datagen/src/{effects,curated,validate,render}/**`,
`data/curated/rules/stat_lines.toml`, `data/curated/priority.toml`.

**Provides**: tier 1 stat-line importer (04 §5.1 table as patterns), curated loader (per-item files, Incarnon
family files), binding and number checks, per-rank checks against WFCD `levelStats`, canonical English renderer,
fingerprints, stale detection and auto-patching (pipeline.md §7), class and order derivation checks, all validators
with passing and failing fixtures, `curate todo|scaffold|rebind|render`, coverage thresholds.

**Acceptance**:

```text
cargo test -p wfd-datagen effects curated validate render    # each validator has a failing and a passing fixture
cargo run -p wfd-datagen -- check                             # clean on the committed data
cargo run -p wfd-datagen -- build --offline                   # tier 1 maps ≥ 65 % of weapon mods with zero unmapped-
                                                              # line errors on them (REPORT.md coverage table)
```

### T20-T25 Curation batches

**Depends**: T19 (authoring may start after T01 against `data/curated/schema.json`). Each batch **owns** its
directories, follows pipeline.md §6.3, and for every item either sets `verified` with sources or justifies
`unmodelled`/`no_damage_effect` in `notes`.

| Task | Owns (under `data/curated/`) | Scope |
|---|---|---|
| T20 | `weapon_mod/{rifle,shotgun,sniper,bow,primary,launcher}/`, `overrides/weapons/` (primary weapons) | primary weapon mods incl. conditional and Galvanized ones; primary hand tables of 06 §4.3 |
| T21 | `weapon_mod/{pistol,secondary}/`, `overrides/weapons/` (secondary weapons, separate files) | secondary mods; secondary hand tables |
| T22 | `weapon_mod/{melee,exalted}/`, `stance/`, `overrides/weapons/` (melee weapons) | melee mods, stances and stance CO ratios, exalted-weapon mods (Chromatic Blade, Reactive Storm) |
| T23 | `weapon_arcane/`, `frame_arcane/`, `set/` | weapon and warframe arcanes, set bonuses |
| T24 | `shard/`, `aura/`, `frame_mod/`, `buff/`, `custom/`, `rules/{mult_groups,slot_layout,riven_traits,riven_formula,body_parts,special_dr,toggles,factions}.toml` | archon shards, auras, frame mods, ability/focus/companion/squad buffs (T-13..T-19 among them), the 26 custom stats, rule tables, riven trait mapping (04 §8) |
| T25 | `incarnon/`, `overrides/enemies/`, `overrides/progenitor.toml`, `aliases/enemies.toml`, `weapon_mod/{archgun,companion,augment}/` | Incarnon families (all of `F:mods` §9), enemy overrides (05 §4.5), progenitor elements, arch-gun, companion and augment mods |

**Acceptance (each)**:

```text
cargo run -p wfd-datagen -- check                         # clean
cargo run -p wfd-datagen -- build --offline               # coverage thresholds of data/validation.toml for the batch's kinds
cargo test -p wfd-engine --test effect_smoke              # every item of the batch reaches its declared stats in scope
```

### T26 Frontend loadout

**Depends**: T12 (T30 for the live API; MSW before). **Owns**: `frontend/src/features/{loadout,share}/**`,
`frontend/src/i18n/en/{loadout,share}.json`, `frontend/e2e/parity/PAR-{01,03,05..16,30..32}-*.spec.ts`.

**Provides**: every loadout component of frontend.md §4 (weapon picker, variant controls, mod grid with React Aria
DnD and keyboard, mod picker with candidates, stance and combo, arcanes, Incarnon track, riven editor, frame panel,
shards), share and import dialogs with the import report.

**Acceptance**:

```text
npm --prefix frontend run lint && npm --prefix frontend run typecheck && npm --prefix frontend test -- --run
npx --prefix frontend playwright test e2e/parity/PAR-01 e2e/parity/PAR-03 … e2e/parity/PAR-32 e2e/keyboard.spec.ts
                                   # against MSW first, then against the real binary once T30 is accepted; axe clean
```

### T27 Frontend target and results

**Depends**: T12. **Owns**: `frontend/src/features/{target,results}/**`, `frontend/src/i18n/en/{target,results}.json`,
`frontend/e2e/parity/PAR-{02,04,17..29}-*.spec.ts`.

**Provides**: enemy picker and panel, schema-driven options, buffs and custom stats, conditions control with
per-effect overrides, combine-attacks dialog, simulation panel; results pane (headline cards with MC strip, layer tabs,
attack cards and table with parity columns, charts with table views, weapon stats panel, hit timeline dialog, Explain
drawer, assumptions and warnings, baseline bar).

**Acceptance**: as T26 with its parity specs, plus `e2e/a11y.spec.ts` for its pages.

---

## 6. Wave 3

### T28 Optimizer core

**Depends**: T15. **Owns**: `crates/wfd-opt/src/{problem,pool,dominance,composition,bnb,capacity,keys}.rs`,
`crates/wfd-opt/benches/**`.

**Provides**: optimizer.md §2-§5: problem compilation, pools via `wfd-data::legality`, status and rule filters on
effects, relevance, dominance, composition-set enumeration with `ElementSolver`, the main-slot branch-and-bound
(active-slot compaction, suffix top-r tables, threshold and keep-set rule, capacity and polarity DP), parallel work
split, atomic threshold, canonical keys. Internal entry point
`pub fn solve_main(kernel: &Kernel, problem: &MainProblem, theta: &SharedTheta, ctl: &JobControl) -> MainSolution`.

**Acceptance**:

```text
cargo test -p wfd-opt bnb composition dominance capacity   # brute-force exactness on 100 random small problems;
                                                           # keep-set rule with perturbed WindowFast; determinism 1/2/8 threads
cargo bench -p wfd-opt -- --quick                          # rifle fixture proven optimal; nodes/s reported
```

### T29 Optimizer driver

**Depends**: T28, T16. **Owns**: `crates/wfd-opt/src/{lib,context,local,portfolio,riven,finalists,events,presets}.rs`,
`crates/wfd-opt/tests/**`.

**Provides**: `optimize` (optimizer.md §1, §6-§11): context groups, block-coordinate ascent with context beam,
cross-group local search, per-mix pass, riven search, buff portfolio, finalists with `WindowExact`, MC confirmation
with CRN, diagnostics (leave-one-out, next-best, condition sensitivity, buff table), `OptEvent` stream, cancellation,
time and node budgets, presets.

**Acceptance**:

```text
cargo test -p wfd-opt              # constraints proptests; cancellation ≤ 100 ms with a valid result; determinism with
                                   # node_budget; inspiration-compatible preset applied; "beats WFOpt" once T32 recordings exist
cargo run -p wfd-server -- optimize --request data/fixtures/scenarios/opt_rifle.json --node-budget 2000000 --json
```

### T30 Server integration

**Depends**: T11, T17, T29, T10, T18. **Owns**: `crates/wfd-server/src/routes/{calc,timeline,explain,build,optimize,data,img}.rs`
(the rest of the crate stays with T11's files, edited only with a CONTRACTS.md note).

**Provides**: every route of server-api.md §3 wired to the engine, optimizer, build codes and the data pipeline
(refresh jobs, activation, rollback, check); image proxy with disk cache; CLI `calc`, `optimize`, `import`, `data *`.

**Acceptance**:

```text
cargo test -p wfd-server           # every route on the fixture bundle (insta snapshots); SSE optimizer run and cancel;
                                   # superseded calc cancels MC; data refresh from a local mock source set swaps the
                                   # bundle without restart while a job keeps its snapshot
cargo run -p wfd-server -- calc --input data/fixtures/scenarios/braton_prime_lancer_l100_sp.json --json
```

### T31 Frontend optimizer and data pages

**Depends**: T12, T30 (mocks before). **Owns**: `frontend/src/features/{optimizer,data,about}/**`,
`frontend/src/i18n/en/{optimizer,data,about}.json`, `frontend/e2e/parity/PAR-{34..39}-*.spec.ts`, `frontend/e2e/data.spec.ts`.

**Provides**: optimizer page (objective, constraints with locks on the mirrored grid, groups, buff portfolio,
prohibited list with persistence and defaults, run panel with SSE progress and resume, log, results with Apply,
Compare and BuildDiff); data page (status, check, refresh with progress and report, activate, rollback); about page.

**Acceptance**: lint, typecheck, unit tests; `npx --prefix frontend playwright test e2e/parity/PAR-34 … PAR-39
e2e/data.spec.ts` against the real binary with a node budget; axe clean.

### T32 Oracle harness and golden tests

**Depends**: T17, T10. **Owns**: `tools/oracle/**`, `xtask/src/oracle.rs`, `crates/wfd-engine/tests/golden/oracle/**`.

**Provides**: testing.md §5: Node + Playwright harness (local inspiration, intercepts, seeded `Math.random`, DOM
readers, `WFOpt.runFull` recording), scenario corpus generation, `cargo xtask oracle-record`, ≥ 30 recorded
oracle-safe scenarios, the three comparison classes in `golden_oracle.rs`.

**Acceptance**:

```text
npm --prefix tools/oracle ci && cargo xtask oracle-record --cases research/test-cases/user-example-furis.json --runs 20
cargo test -p wfd-engine --test golden_oracle        # classes 1-3 pass on the committed recordings
```

### T33 Performance

**Depends**: T15, T16, T28. **Owns**: `benches/baseline.json` files, `xtask/src/bench.rs`,
`docs/architecture/performance.md`.

**Provides**: `cargo xtask bench-check`; profiling pass on kernel, window walk, MC and branch-and-bound; measured
numbers of engine.md §13 and optimizer.md §5.3 recorded.

**Acceptance**: `cargo xtask bench-check` green; `performance.md` lists each target with its measurement and machine.

### T34 End-to-end completion

**Depends**: T26, T27, T31, T30. **Owns**: `frontend/e2e/{a11y,visual,pseudo-locale,share,import}.spec.ts`,
`frontend/e2e/__screenshots__/**`.

**Provides**: every must-have row of frontend.md §6 has a passing `PAR-nn` spec against the real binary; keyboard-only
flows; axe on every page and dialog; visual baselines (dark and light, 375/900/1440 px); pseudo-locale run; Furis
inspiration-link import.

**Acceptance**: `cargo xtask ci` green end to end.

### T35 Packaging

**Depends**: T30, T34. **Owns**: `xtask/src/dist.rs`, release notes template.

**Provides**: `cargo xtask dist` (`npm ci && npm run build`, then `cargo build --release -p wfd-server`), binaries for
Linux, macOS and Windows, NOTICE and licences next to the binary and on the About page.

**Acceptance**: on a machine with no network and an empty data directory, the built binary starts, opens the browser,
serves the calculator with the embedded bundle, and `warframe-dps calc --input <scenario>` prints the spec 00 §3.8
numbers.

---

## 7. Wave 4

### T36 Accuracy audit

**Depends**: T32, T20-T25. **Owns**: `research/findings/accuracy-audit.md`.

**Provides**: oracle comparisons over the full corpus in both profiles; in-game cases; every disagreement classified
(our bug, inspiration difference already in the register, new register row proposed to the manager, open question);
"beats WFOpt" on the oracle weapon set with the inspiration-compatible preset.

**Acceptance**: all golden tests green or explained in the audit; proposed register rows listed for the manager.

### T37 Documentation pass

**Depends**: all. **Owns**: `docs/architecture/*.md`, `docs/data/*.md` (edits only), `CONTRACTS.md` summary.

**Provides**: these documents updated to match what was built (CONTRACTS.md entries folded in), statuses moved from
`draft` to `cross-checked` after review, SPEC-PENDING markers cleared where 07-09 have landed, research proposals
marked `superseded`.

---

## 8. Ownership map

| Path | Owner |
|---|---|
| `Cargo.toml`, `rust-toolchain.toml`, `.gitignore`, `CONTRACTS.md` (structure) | T01 |
| `crates/wfd-model/**` | T01 (changes via CONTRACTS.md) |
| `xtask/src/{main,gen_ts,gen_schema,spec_sync,lint_engine,ci}.rs` | T01 |
| `xtask/src/spec_vectors.rs` / `oracle.rs` / `bench.rs` / `dist.rs` | T09 / T32 / T33 / T35 |
| skeleton of every other crate | T02, then the owners below |
| `crates/wfd-data/**` | T04 |
| `crates/wfd-datagen/src/{fetch,lua,wiki,pe,wfcd}/**` | T05 |
| `crates/wfd-datagen/src/{join,derive,spec_tables,emit,report}/**`, `main.rs` | T18 |
| `crates/wfd-datagen/src/{effects,curated,validate,render}/**` | T19 |
| `crates/wfd-engine/src/{compile/,stats.rs,state.rs,elements.rs}` | T06 |
| `crates/wfd-engine/src/enemy/**` | T07 |
| `crates/wfd-engine/src/schedule/**`, `resolve/{firing,melee}.rs` | T08 |
| `crates/wfd-engine/src/stage_registry.rs`, `tests/{spec_vectors.rs,spec_conformance.rs,spec_vectors/**}` | T09 |
| `crates/wfd-engine/src/{resolve/ (rest),assemble.rs,spec_constants.rs}` | T13 |
| `crates/wfd-engine/src/{status/,mechanics/}`, `tests/mechanics.rs` | T14 |
| `crates/wfd-engine/src/ev/**`, `tests/properties.rs` | T15 |
| `crates/wfd-engine/src/mc/**`, `tests/ev_vs_mc.rs` | T16 |
| `crates/wfd-engine/src/{lib,outputs,compat,trace,probe}.rs`, `tests/{compat,effects_coverage,effect_smoke,golden_oracle}.rs` | T17 |
| `crates/wfd-opt/src/{problem,pool,dominance,composition,bnb,capacity,keys}.rs`, `benches/` | T28 |
| `crates/wfd-opt/src/{lib,context,local,portfolio,riven,finalists,events,presets}.rs`, `tests/` | T29 |
| `crates/wfd-buildcode/**`, `data/curated/aliases/inspiration.toml` | T10 |
| `crates/wfd-server/**` except `src/routes/{calc,timeline,explain,build,optimize,data,img}.rs` | T11 |
| `crates/wfd-server/src/routes/{calc,timeline,explain,build,optimize,data,img}.rs` | T30 |
| `data/fixtures/**` (later tasks add `data/fixtures/mini/extra/<task>.json`) | T03 |
| `data/spec_tables.toml`, `data/validation.toml`, `data/NOTICE.md`, `data/generated/**` | T18 (regenerated by every data task through the build command) |
| `data/sources.lock.toml` | T05 |
| `data/curated/**` | T18, T19, T20-T25 per the tables above |
| `frontend/` (foundation, shell, design, state, api, i18n infrastructure) | T12 |
| `frontend/src/features/{loadout,share}/**` | T26 |
| `frontend/src/features/{target,results}/**` | T27 |
| `frontend/src/features/{optimizer,data,about}/**` | T31 |
| `frontend/e2e/parity/PAR-nn-*.spec.ts` | the feature task of the row (T26, T27, T31) |
| `frontend/e2e/{a11y,visual,pseudo-locale,share,import}.spec.ts`, screenshots | T34 |
| `tools/oracle/**`, `crates/wfd-engine/tests/golden/oracle/**` | T32 |
| `docs/architecture/performance.md` | T33 |
| `research/findings/accuracy-audit.md` | T36 |
| `docs/architecture/*.md`, `docs/data/*.md` (after wave 3) | T37 |

`data/generated/` is a build output: any task may regenerate it with `wfd-datagen build --offline`, and the diff is
committed together with the source change that caused it; conflicts are resolved by regenerating, never by hand.
