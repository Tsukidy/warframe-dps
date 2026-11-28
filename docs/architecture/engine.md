Status: reviewed (architecture)

# Engine (`wfd-engine`)

The engine implements the damage spec (`docs/mechanics/`): resolution of a build into per-attack stats (ST-1..ST-7),
the deterministic schedule (06, 07), the enemy side (ST-8, ST-9), status effects (ST-10), and the two evaluators of
spec 00 §4: the expected-value (EV) evaluator and the event-driven Monte Carlo (MC) simulator. It does no I/O, holds
no global state, and depends only on `wfd-model`. The optimizer uses its lower-level `Kernel` API (optimizer.md).

Contents: 1 API · 2 Module layout · 3 Compile levels · 4 Resolve (ST-1..ST-7) · 5 Schedules · 6 Enemy side · 7 EV
evaluator · 8 MC simulator · 9 Outputs · 10 Compatibility profile · 11 Traces, probes, Explain · 12 Numerics and
determinism · 13 Performance · 14 Spec test vectors · 15 Errors and warnings

---

## 1. API

```rust
// crates/wfd-engine/src/lib.rs
pub struct Engine<'b> { bundle: &'b Bundle }            // cheap to create, Sync, Copy

impl<'b> Engine<'b> {
    pub fn new(bundle: &'b Bundle) -> Self;
    pub fn bundle(&self) -> &'b Bundle;

    /// The server's and the CLI's entry point: everything for the calculator page.
    pub fn calculate(&self, build: &Build, scenario: &Scenario, want: &Want, cancel: &CancelToken)
        -> Result<CalcReport, EngineError>;

    // ---- levels (section 3); the optimizer and the tests use them directly ----
    /// L0: per scenario (enemy scaling, faction relations, caps, T-02 mode, slot allocation, the condition table,
    /// buffs, custom stats, the T-41 hit fractions and T-43 external statuses, melee tables and plans).
    pub fn compile_scenario(&self, build: &Build, scenario: &Scenario, objective: Option<&ObjectiveSpec>)
        -> Result<CompiledScenario, EngineError>;
    /// L1: one candidate (mod, arcane, riven, perk, shard, stance, set bonus) in a scenario.
    pub fn compile_pick(&self, cs: &CompiledScenario, pick: &Pick) -> CompiledItem;
    /// L2: a whole build: Σ contributions + finalizers (sets, shards, resolved scalars, element order).
    pub fn compile_build(&self, cs: &CompiledScenario, build: &Build) -> Result<CompiledBuild, EngineError>;
    /// L3: one attack (ST-1..ST-7) plus firing stats, every dynamic gate at its mean:
    /// = resolve_point(cs, &cb.point(), attack, GateAssign::Mean).
    pub fn resolve(&self, cs: &CompiledScenario, cb: &CompiledBuild, attack: AttackIdx) -> ResolvedAttack;
    /// L3 on a borrowed build point for one resolution of the dynamic gates (section 7.4); no heap allocation.
    pub fn resolve_point(&self, cs: &CompiledScenario, x: &BuildPoint, attack: AttackIdx, gates: GateAssign)
        -> ResolvedAttack;
    /// Deterministic shot/hit schedule (06 E-6.11; melee 07 E-7.30 with the scenario's MeleePlan), identical for EV
    /// and MC; the caller has checked `event_bound` (section 15).
    pub fn schedule(&self, cs: &CompiledScenario, ra: &ResolvedAttack, horizon: f64) -> Schedule;
    /// `event_bound` then `schedule`: ScheduleTooLong beyond 10^7 events, an infinite rate included.
    pub fn try_schedule(&self, cs: &CompiledScenario, ra: &ResolvedAttack, horizon: f64)
        -> Result<Schedule, EngineError>;
    /// The evaluation units of CalcReport.groups, in that order (08 §9: T-22 combined families, parent+child sums).
    pub fn units(&self, cs: &CompiledScenario, cb: &CompiledBuild) -> Vec<EvalUnit>;
    /// L4: expected values (section 7). Warnings and errors go to the Workspace (section 15).
    pub fn ev(&self, cs: &CompiledScenario, unit: &EvalUnit, req: &EvRequest, ws: &mut Workspace) -> EvOut;
    /// L4: Monte Carlo (section 8). Runs in parallel with feature `parallel`; results independent of thread count.
    pub fn mc(&self, cs: &CompiledScenario, unit: &EvalUnit, req: &McRequest, cancel: &CancelToken)
        -> Result<McOut, EngineError>;
    /// Per-hit event log of one MC run (the inspiration's "More details"), replayed under `req.seed`.
    pub fn timeline(&self, cs: &CompiledScenario, cb: &CompiledBuild, unit: &EvalUnit, view: View,
                    req: &TimelineReq) -> Result<Timeline, EngineError>;
    /// Read one spec symbol (section 11.2); used by spec vectors and the Explain drawer.
    pub fn probe(&self, build: &Build, scenario: &Scenario, path: &ProbePath) -> Result<ProbeValue, ProbeError>;
    pub fn explain(&self, build: &Build, scenario: &Scenario, path: &ProbePath) -> Result<Explanation, ProbeError>;

    /// Optimizer-facing scoring kernel (section 7.9); scores are ordered keys (`Score`), not plain f64.
    pub fn kernel<'a>(&'a self, cs: &'a CompiledScenario, objective: &'a ObjectiveSpec) -> Kernel<'a>;
}

/// What a calculation should produce (the server maps CalcRequest.want onto it).
pub struct Want { pub ev: bool /* always true in practice */, pub mc: Option<McWant>, pub parity: bool,
                  pub breakdown: bool, pub trace: bool, pub views: Option<SmallVec<[View; 5]>> }
/// Resolved MC parameters (T-29 with the scenario's eval defaults; "auto" seeds are drawn by the frontend).
pub struct McWant { pub runs: u32, pub seed: u64, pub series_bin_s: f64 }
/// An attack or an attack group (T-22 combined or parent+child summed) evaluated together; attacks[0] fires.
pub struct EvalUnit { pub attacks: SmallVec<[AttackIdx; 3]>, pub combined: bool }
/// L4 requests carry the compiled build (C-32); the evaluators resolve (L3) and schedule the unit's attacks.
pub struct EvRequest<'r> { pub build: &'r CompiledBuild, pub fidelity: Fidelity, pub views: &'r [View],
                           pub series_bin_s: f64, pub breakdown: bool }
pub struct McRequest<'r> { pub build: &'r CompiledBuild, pub runs: u32, pub seed: u64, pub views: &'r [View],
                           pub series_bin_s: f64, pub parity: bool, pub breakdown: bool }
/// The replayed run (API-1, C-32): seed = the E-8.16 key (TimelineRequest.seed, else the scenario's eval.seed).
pub struct TimelineReq { pub seed: u64, pub run: u32, pub limit: u32 }
/// `Pick` is `wfd_model::Pick` (crates/wfd-model/src/pick.rs), re-exported by wfd-engine at this path (ruling T02b-2:
/// wfd-data's legality::check_slot names the same type and does not depend on the engine). Its shape:
pub use wfd_model::Pick; // enum Pick { Mod { item: ItemId, rank: u8, params: SmallVec<[(ParamKey, ParamValue); 1]> },
                // Riven(Riven), Arcane { item: ItemId, rank: u8 }, Perk { tier: u8, index: u8 },
                // Shard { item: ItemId, tauforged: bool }, Stance { item: ItemId, combo: ComboKey },
                // FrameMod { item: ItemId, rank: u8 }, Aura { item: ItemId, rank: u8 }, KitgunParts { grip: u16, loader: u16 },
                // ZawParts { grip: u16, link: u16 }, Progenitor { element: DamageType } }
pub struct CancelToken(Arc<AtomicBool>);                // clones share the flag; MC checks it between runs

/// Context of every pure stage function (rules, compat flags, trace sink; a no-op sink unless Want.trace).
pub struct StageCtx<'a> { pub rules: &'a Rules, pub compat: CompatFlags, pub trace: &'a dyn TraceSink }
/// Context of the EV kernel, the mechanic traits and the MC simulator (contract review ENG-1, ENG-2): the stage
/// context, the compiled scenario (scaled enemy, toggles, condition table), the dynamic terms of the attack being
/// evaluated and the build's active condition ids.
pub struct EvalCtx<'a> { pub stage: StageCtx<'a>, pub cs: &'a CompiledScenario, pub coeffs: &'a [Coeff],
                         pub conditions: &'a [u16] }
```

`mc` and `timeline` return `EngineError`, not `Cancelled` alone: a run can also fail with `RngIndexRange` (08 E-8.16,
section 8.2) or `ScheduleTooLong` (C-35). A caller that ranks finalists keeps its EV ranking when `mc` returns
`Cancelled` (optimizer.md §9: cancel never loses the best build).

Report types (`CalcReport`, `EvOut`, `McOut`, `Timeline`, `Explanation`, `ParityOut`, `ProbeValue`, `Summary`,
`TtkDist`, `TtkEv`, `ViewEv`, `ViewMc`) are API DTOs in `wfd-model::api::calc`, re-exported by the engine, so the
frontend's types are generated from one crate (server-api.md §5). `Metric` is `wfd_model::api::Metric` re-exported.

---

## 2. Module layout

```text
crates/wfd-engine/src/
  lib.rs                Engine, StageCtx, EvalCtx, Want, McWant, EvalUnit (T17)
  error.rs  cancel.rs   EngineError, Cancelled, CancelToken (complete; T17, ruling C-31)
  instance.rs           Instance, InstanceKind, InstanceOutcome, TierOutcome, ProcOutcome, DerivedHit, ProcApply:
                        what the kernel, the mechanics and the simulator exchange about one instance (T14)
  spec_constants.rs     formula constants of the spec only, each with its E-number (I-1)
  stats.rs              StatVec (fixed point), StatVecF, StatMask, SparseContrib, Coeff, ElemEntry, MechanicInst,
                        StaticTerm, MechSet, SourceIndex
  state.rs              StateVar, StateVarTable
  stage_registry.rs     stage adapters of the spec-vector harness (T09; section 14)
  compile/              scenario.rs (L0), pick.rs (L1, T-02 table), scope.rs, build.rs (L2), sets.rs, shards.rs,
                        riven.rs, buffs.rs (05 §13.1 ability gate), custom.rs, passives.rs, conditions.rs (class
                        derivation, overrides, the condition table), monotone.rs (per-scenario slot classes from
                        readers, domain-model.md §10), melee.rs (E-7.0b table resolution, MeleePlanSel → MeleePlan)
  assemble.rs           kitgun, zaw, progenitor (E-1 V_t), exalted Strength scaling
  elements.rs           element combination (01 §4.11), ElementSolver (all distinct outcomes, for the optimizer)
  resolve/              mod.rs (ResolvedAttack and its parts, GateAssign), base.rs (E-1, 01 §4.2-4.7), composition.rs
                        (E-2, 01 §4.9-4.12, Table 4.10-A, compose), quantize.rs (E-3), scalar.rs (E-11, 04 §2,
                        §3.4), weapon_mult.rs (ST-5: F, G, G_F, M_OG, dot_only), part.rs (E-7), crit.rs (E-4..E-6,
                        02 E-2.5a), status_chance.rs (E-8, E-9), multishot.rs (E-10), firing.rs (06 §5, schedule
                        inputs), melee.rs (07 §5-§7: attack speed, wind-up, combo stats, Tennokai, heavy synthesis)
  schedule/             mod.rs (Schedule, ShotRec, MeleeHit, event_bound), ranged.rs (06 E-6.11), melee.rs (07 E-7.30,
                        plans of 07 §8.1, steady E-7.37), segment.rs
  enemy/                scaling.rs (NP-2, ScaledEnemy), armor.rs (E-15), layers.rs (E-12 enemy side, ST-9, CD-08),
                        dr.rs (innate, attenuation, special DR, T-28), parts.rs (body parts, head-crit)
  status/               rules.rs (03 §4.1 rules from the bundle, CAP_UNBOUNDED), dot.rs (E-13), heat.rs (03 §10),
                        blast.rs (03 §11), magnetic_break.rs (03 §13), distinct.rs (03 §14), extra_hit.rs (E-14)
  mechanics/            mod.rs (traits, MechInst, mechanic_impl) and one module per mechanic kind of
                        domain-model.md §11.2: extra_hit, proc_on_event, convert_damage, repeat_hit, flat_damage_on_proc,
                        proc_after_ticks, counter_bonus, random_multiplier, spawn_attack, stack_present_statuses
  ev/                   mod.rs (EvRequest), kernel.rs (E-8.9: point, PointState, EvAcc, ProcMass), poisson.rs (E-8.3),
                        joint.rs (crit, ARM and attenuation enumeration, E-8.9 steps 3-6), gates.rs (gate splitting),
                        steady.rs (E-8.13, E-8.14), window_exact.rs (E-8.12), heat.rs (E-8.5-E-8.8), dot.rs (E-8.11),
                        distinct.rs (E-8.10), cond_stacks.rs (04 E-4.9, 08 E-8.15), window_fast.rs, bound.rs, ttk.rs
                        (E-8.21, E-8.26), hit.rs (08 §10), group.rs (E-8.23), series.rs (08 §8.1-§8.4 curves),
                        score.rs (Score keys), opt_kernel.rs (Kernel, ObjectiveSpec, BuildPoint, Workspace; 7.9)
  mc/                   mod.rs (McRequest, TimelineReq), rng.rs (Philox4x32-10, E-8.16), keys.rs (mechanic id
                        registry, FNV-1a), draw.rs (E-8.17), events.rs (McEvent, heap key, McSink), state.rs (08 §6.2
                        run state), sim.rs (E-8.18), passes.rs (V, L, L0), summary.rs (E-8.19, E-8.25), ttk.rs (E-8.20),
                        timeline.rs (08 §8.9), parity.rs (E-8.22)
  outputs.rs            CalcReport assembly, spec 00 §4.5 mapping, SourceKind
  compat.rs             CompatFlags helpers
  trace.rs              TraceEntry, TraceSink (no-op by default), touched register rows
  probe.rs              ProbePath parser, probe registry
```

Each `resolve/`, `enemy/`, `status/` and `schedule/` function is pure, takes a `&StageCtx` (rules, compat flags, trace
sink), and cites its equation in its doc comment, so a reviewer can put the spec and the code side by side. File owners
are work-breakdown.md §8 (the crate doc of `lib.rs` repeats them); `Cargo.toml`, `error.rs` and `cancel.rs` are T17's,
and any engine task adds its dev-dependencies, `[[bench]]` entries and `EngineError` variants through a CONTRACTS.md
entry (ruling C-31, the C-21 rule).

### 2.1 Mechanic interfaces (frozen by T02a, extended by the contract review)

Every mechanic kind of domain-model.md §11.2 implements two traits; `mechanics/` (T14) writes the implementations, the
evaluators (T15a-c, T16) only call them through `mechanic_impl`, so the three tasks code against signatures that exist
before any of them starts. Every method gets the evaluation context and the outcome of the instance it reacts to
(contract review ENG-1):

```rust
// crates/wfd-engine/src/mechanics/mod.rs
pub trait MechanicEv {
    /// Expected contribution to one instance at one state on `view`: E-8.9 terms, DoT registrations (acc.dots),
    /// derived instances on the weapon side (acc.hits; the kernel applies the enemy side jointly with the parent).
    fn ev_instance(&self, cx: &EvalCtx, inst: &MechInst, ra: &ResolvedAttack, st: &PointState,
                   out: &InstanceOutcome, view: View, acc: &mut EvAcc);
    /// Expected proc events the mechanic adds for the instance `out` of one proc event (feeds h_{s,e}, m_{s,e}; E-8.2).
    fn ev_procs(&self, cx: &EvalCtx, inst: &MechInst, ra: &ResolvedAttack, st: &PointState,
                out: &InstanceOutcome, procs: &mut ProcMass);
}
pub trait MechanicMc {
    /// Live effect on one instance (P4a): draws through `rng` with the mechanic's ids at the instance's anchor; emits
    /// events, derived instances, P4b procs and a G factor into `sink`. Called once per instance, never per view.
    fn mc_instance(&self, cx: &EvalCtx, inst: &MechInst, ra: &ResolvedAttack, st: &mut McState, ev: &McEvent,
                   out: &InstanceOutcome, rng: &mut McRng, sink: &mut McSink);
    /// The MC id strings (fixed ids, or derived "eff/<effect key>", "xh/<source>/<name>" …); L0 registers each in
    /// CompiledScenario.mech_ids, which detects FNV-1a collisions (it needs the strings, section 8.2).
    fn mechanic_ids(&self, inst: &MechInst) -> SmallVec<[Box<str>; 2]>;
}
pub trait MechanicImpl: MechanicEv + MechanicMc + Sync {}
pub fn mechanic_impl(kind: &Mechanic) -> &'static dyn MechanicImpl;     // exhaustive match over MechanicKind
pub struct MechInst { pub mechanic: Mechanic, pub magnitude: f64, pub source: EffectRef, pub key: EffectKey,
                      pub ids: SmallVec<[MechanicId; 2]> /* hashes of mechanic_ids, same order */ }
```

The types the traits read live with their owners and are re-exported by `mechanics`:

```rust
// crates/wfd-engine/src/instance.rs (T14)
pub enum InstanceKind { Direct, Radial, ExtraHit, Duplicate, LingeringTick }          // 08 §4.1
pub struct Instance<'s> { pub kind: InstanceKind, pub anchor: Anchor /* (a, i, j), E-8.0 */, pub attack: AttackIdx,
                          pub shot: &'s ShotRec /* flags of 06 E-6.13 and 07 §7.5, ShotRec::melee */ }
pub enum TierOutcome { Rolled { k: u32, c: f64 },                                     // MC: k and c_k
                       Expected { dist: TierDist, c: [f64; 3] /* E[c | k = dist.base + i], E-8.9 step 3 */ } }
pub enum ProcOutcome { Rolled([u8; N_ST]), Expected(ProcMass) }                       // the instance's own procs
pub struct InstanceOutcome<'s> { pub inst: Instance<'s>, pub part: HitPart,
                                 pub w: HitWeaponSide /* at crit factor 1: W_t(k) = c_k·w */,
                                 pub tier: TierOutcome, pub procs: ProcOutcome }
/// A derived instance on the weapon side; the evaluator applies the enemy side once per view (MC pass V, both
/// og_present timelines), routes it once (layered), or takes its expectation with the parent's state (EV).
pub struct DerivedHit { pub source: SourceKind, pub kind: InstanceKind, pub w: DamageVec, pub crit_scaled: bool,
                        pub weight: f64, pub status: XhProc, pub class: InstanceClass, pub gate_factor: f64 }
/// One P4b application (03 §7.1) a mechanic emits: the procs of an extra hit, proc_on_event, proc_after_ticks,
/// stack_present_statuses (stacks = N of 04 E-4.20).
pub struct ProcApply { pub status: StatusType, pub anchor: Anchor, pub source: AttackIdx, pub stacks: u8,
                       pub duration: f64, pub snap: Option<ProcSnapshot> /* DoT seed terms; plus_one per 00 E-14 */ }

// ev/kernel.rs (T15a): PointState (section 7.1), EvAcc { view, direct, per_type, per_source, dots: [DotReg],
//   hits: [DerivedHit], extra, procs }, DotReg, ProcMass { mass, presence, certain }, PointOut
// mc/events.rs (T16): McEventKind, Phase, SubKey (section 8.1), and
pub struct McEvent { pub t: f64, pub t_us: i64 /* τ̂, 08 E-8.0a */, pub phase: Phase, pub sub_key: SubKey, pub seq: u64,
                     pub kind: McEventKind, pub anchor: Anchor }          // heap key (t_us, phase, sub_key, seq)
pub struct Anchor { pub a: u16, pub i: u32, pub j: u16 }                  // E-8.16 sentinels in section 8.2
pub struct McSink { pub events: SmallVec<[McEvent; 4]>, pub hits: SmallVec<[DerivedHit; 2]>,
                    pub procs: SmallVec<[ProcApply; 4]>,
                    pub g_mult: f64 /* ×G of the instance (random_multiplier, 04 §3.2); 1 by default */ }
// mc/state.rs: McState (section 8.1); mc/rng.rs: McRng; mc/keys.rs: MechanicId
```

Derived instances (extra hits, Melee Duplicate copies, flat damage on proc, Blast-detonation extra hits) are emitted on
the weapon side because 08 E-8.18 evaluates every damage instance once per view: a mechanic that computed the enemy
side itself would have to be called once per view, which would advance its E-8.16 ordinals four times. A derived
instance's procs are the emitting mechanic's: it rolls them with the derived ids of E-8.16 (`"xh/…"`, `"dup/…"`,
`"xd/…"`) and emits `ProcApply` (MC) or adds them through `ev_procs` (EV); `DerivedHit.status` is informational and the
evaluators apply only its damage (contract review ENG-R4). `InstanceOutcome.w` is the c = 1 form of `HitWeaponSide`,
while `extra_hit()` takes the crit-scaled form (MC multiplies by c_k first; EV marks the result `crit_scaled`).

---

## 3. Compile levels

| Level | Input | Output | When | Typical cost |
|---|---|---|---|---|
| L0 | build skeleton (weapon, variant), scenario | `CompiledScenario` | once per scenario | microseconds |
| L1 | one candidate pick | `CompiledItem` (domain-model.md §18) | once per (pick, scenario) | tens of µs for ~1,000 picks |
| L2 | a build | `CompiledBuild`: per build attack a `StatVec`, element outcome, coefficients, mechanics; the active conditions | per candidate; incremental in search | 50-250 ns |
| L3 | (build, attack, gate assignment) | `ResolvedAttack` | per candidate per attack | 100-200 ns |
| L4 | resolved attack(s) + schedule | EV or MC outputs | per candidate | 0.14 µs per target-state point |

```rust
pub struct CompiledScenario {
    pub weapon: AssembledWeapon,          // kitgun/zaw parts applied, progenitor V_t, m_str per attack (N-3)
    pub enemy: ScaledEnemy,               // pools per layer, A0, φ_t, caps (and caps_og), DR, flags, parts (section 6)
    pub faction: FactionId,               // target faction and bane relations, resolved
    pub toggles: Toggles, pub mode: ConditionMode, pub overrides: CompiledOverrides,
    pub melee: Option<CompiledMelee>,     // E-7.0b combo tables for the equipped stance, MeleePlan per attack, specials
    pub profile: CompatFlags,
    pub fixed: SmallVec<[SparseContrib; 2]>,   // buffs, custom stats, weapon/attack passives, zoom, toggles that are stats
    pub state_vars: StateVarTable,        // which StateVars exist in this scenario
    pub slot_class: [Monotone; N_STATS],  // per-scenario monotonicity: declared class downgraded by readers (compile::monotone)
    pub readers: SmallVec<[Reader; 4]>,   // readers held by candidates: the optimizer's coupled candidates
    pub mech_ids: MechIdTable,            // every MC mechanic id of the scenario, FNV-1a hashed, checked distinct
    pub mech_slots: MechSlotTable,        // allocation of `mech.<n>` slots
    pub mult_slots: MultSlotTable,        // allocation of `mult.<n>` slots from Rules.mult_groups
    pub objective: Option<ObjectiveSpec>, // units and weights for the optimizer (section 7.9)
    pub window: f64,                      // t_win (T-01)
    pub ttk_max: f64,                     // t_ttk_max (E-8.20, E-8.21, E-8.26; ENG-15)
    pub horizon: f64,                     // max(t_win, t_ttk_max)
    pub conditions: Vec<CompiledCondition>,  // the condition table (below; ENG-2)
    pub hit_fraction: SmallVec<[f64; 8]>, // T-41 f_a per attack (1 without an entry)
    pub external: [u8; N_ST],             // T-43 N_ext,s with T-06 folded in, 0 on immune types
}
pub struct StatVec(pub [i64; N_STATS]);    // 1e-9 units; to_f64() once at L3; `max` rows via raise_max
pub struct CompiledBuild {                 // index k = build attack (objective attacks, or every attack for the calculator)
    pub attacks: SmallVec<[AttackIdx; 2]>, pub sv: SmallVec<[StatVec; 2]>, pub elements: SmallVec<[ElemOutcome; 2]>,
    pub coeffs: SmallVec<[SmallVec<[Coeff; 4]>; 2]>, pub mechanics: SmallVec<[MechSet; 2]>,
    pub conditions: SmallVec<[u16; 4]>,   // active condition ids, ascending (≤ MAX_COND_VARS)
    pub sources: SourceIndex,             // traces; ordered (slot, layout position, effect); T17 counts CDM0+ (ENG-R1)
}
impl CompiledBuild { pub fn point(&self) -> BuildPoint<'_>; pub fn position(&self, a: AttackIdx) -> Option<usize>; }
// compile/pick.rs (domain-model.md §18)
pub struct CompiledItem { pub per_attack: SmallVec<[SparseContrib; 2]> /* per build attack */, pub drain: i8,
                          pub incompat_mask: u64, pub set: Option<SetId>, pub touched: StatMask, pub has_unknown: bool,
                          pub readers: SmallVec<[Reader; 1]> /* what monotone_classes downgrades */ }
pub struct SparseContrib { pub entries: SmallVec<[(u16, i64); 6]>, pub max_entries: SmallVec<[(u16, i64); 1]>,
                           pub coeffs: SmallVec<[Coeff; 2]>, pub elements: SmallVec<[ElemEntry; 2]>,
                           pub mechanics: SmallVec<[MechanicInst; 1]>, pub statics: SmallVec<[StaticTerm; 1]>,
                           pub conditions: SmallVec<[u16; 1]>, pub sources: SmallVec<[EffectRef; 4]> }
pub struct ElemEntry { pub order: ElemOrder, pub line: u8, pub ty: DamageType,
                       pub weakness: bool /* target_weakness: type per layer (Table 4.10-A) */,
                       pub dot_scaling: bool /* 04 §4.2, 03 §9.3 default resolved */, pub value: i64 }
```

Per-attack scope (contract review ENG-7). The element outcome, the dynamic terms and the mechanics are kept per build
attack like the stat vectors: element contributions, conditional terms and mechanics are scoped (Sentient Incision and
Chromatic Blade skip the slams, 01 Table 4.10-B and E-1.13; heavy-only, slide-only and Tennokai-only terms;
`counter_bonus.applies_to`), so attack b is never evaluated with attack a's elements or mechanics.

**Condition table** (`compile::conditions`, contract review ENG-2). Under the scenario's T-02 mode most conditional
effects compile to constants (domain-model.md §12.2); every condition that stays dynamic gets one entry of
`CompiledScenario.conditions`, allocated at L0 like the `mech.<n>` slots, with the rank applied (09 E-9.13,
`RankCurve::value`). L0 sees no candidate pool, so both allocations take the scenario's fixed sources plus the build's
own picks (calculator, no objective) or, with an objective, every bundle item legal in some position of the build's
search (the weapon's slots, Incarnon tiers and modular parts, plus the build frame's mods, aura, arcanes and shards, by
09 legality), at every rank the optimizer may pick: a superset of any pool built after L0, so `compile_pick` never needs
a new id (contract review ENG-R8, P3-1; the frame positions matter for exalted weapons, whose frame mods Archon
Continuity and Archon Vitality are `proc_on_event` mechanics, 02 E-2.17a). More than `N_MECH` (16) mechanic signatures
fail L0 with `EngineError::MissingData`; T19's `effect-smoke` checks the real bundle. Its index is the condition id of
`StateVar::CondStacks(id)`, `StateVar::Switch(id)`, `GateSet::ids`, `McEventKind::ConditionExpiry` and `Switch`; a
contribution names the ids it brings (`SparseContrib.conditions`), L2 unions them into `CompiledBuild.conditions`, and
position p there indexes the per-condition arrays of the EV state.

```rust
pub struct CompiledCondition { pub source: EffectRef, pub key: EffectKey /* "eff/<key>", "eff/<key>/phase" */,
                               pub class: ClassSet, pub kind: CondKind }
pub enum CondKind {
    Stacks(Box<StackCond>),               // class (b): 04 E-4.8 (MC), E-4.9 / 08 E-8.15, E-8.15b (EV)
    Switch { on: f64, period: f64 },      // class (c)/(u), u_e < 1, nonlinear use: 08 E-8.9 step 4c, E-8.18
    Target(TargetPred),                   // class (a) threshold as a gate or nonlinear condition
}
pub struct StackCond { pub gain: Cond /* gain event with filters, chance at rank */, pub active: Cond,
                       pub stacks: Option<StackModel> /* max, duration at rank; None = single stack */ }
```

The slot and coefficient a condition feeds are the `Coeff` terms that read it (domain-model.md §18), which the
evaluators receive through `EvalCtx.coeffs`.

**Finalizers (L2).** Static scalars are resolved once all picks are known: set-piece counts
(`wfd_model::sets::set_counts`, 09 E-9.40, the one implementation that wfd-data's legality also uses; API-6), shard
counts (Violet scaling), Strength, and `Per::Resolved` terms (the stage order guarantees the referenced stat is final,
and the compiler rejects cycles: an effect cannot change a stat its own condition or scalar reads). Melee Doughty is not
a `Per::Resolved` term: its rank factor is the row `doughty_rank` and the crit stage computes 02 E-2.5a per instance
(ruling C-37). Locks (`lock_multishot`, `lock_fire_rate`) are applied at L3. Rows with the `max` aggregate (`cc_floor`,
`external.*`) travel in `SparseContrib.max_entries` and are applied with `StatVec::raise_max` (no inverse: the
optimizer recomputes such a slot when it removes a contribution).

**Element outcome.** `elements::combine(entries: &[OrderedElem]) -> ElemOutcome` (`OrderedElem = (OrderKey,
DamageType, f64)`; `resolve::combine_elements` is the same function) implements 01 §4.11 (slot order, exilus after
slot 7, `arcane<n>` at position 9 with sub-index n and `inc<t>` with t (T03b-R2), riven bottom-up, innate group HCET,
non-combining additions). `ElemOutcome { sums, combined, elem_dot /* 01 E-1.15 */, weakness /* target_weakness
entries */ }`; L2 fills `elem_dot` and `weakness` from the `ElemEntry` flags. `ElementSolver::outcomes(fixed, set,
slots) -> SmallVec<[(ElemOutcome, Vec<SlotAssignment>); 3]>` returns every distinct outcome an unordered set of element
entries can reach, with one slot assignment realizing each (optimizer.md §5).

**05 §13.1 ability gate.** A buff whose `ItemKind::Buff.ability_source` is true contributes no `vuln_add`, no
`target_vuln` group and no `strip_ability` or `strip_points` against an `ABILITY_DEBUFF_IMMUNE` enemy: the lowering in
the scenario context drops them (`compile/buffs.rs`; contract review M-3, DC-16).

---

## 4. Resolve (ST-1..ST-7)

`ResolvedAttack` is a fixed-size `Copy` struct (no heap):

```rust
pub struct ResolvedAttack {
    pub attack: AttackIdx, pub roles: AttackRoles,
    pub base: DamageVec, pub base_total: f64, pub base_ref: f64,      // B'_t, B', B_ref (E-1)
    pub ratio: DamageVec, pub ratio_q: DamageVec,                      // r_t, q_t (E-2, E-3) on the non-OG layers
    pub ratio_q_og: Option<DamageVec>,                                 // q_t on the OG layer when Table 4.10-A differs
    pub compose: ComposeInputs,                                        // inputs of E-1.9 for per-strike/per-layer compose
    pub a: f64, pub a_w: f64,                                          // A (A_m), A_w (01 E-1.11, 04 §3.4)
    pub co: CoResolved,                                                // C as affine terms; behaviour, ρ_co, B_co (E-11)
    pub g_f: f64, pub faction: f64, pub g: f64,                        // Π G_F, F, Π G (static parts)
    pub g_extra_pellets: f64,                                          // G members of pellets j ≥ 1 (Scope.hit.pellet)
    pub m_og: f64, pub dot_only: f64,                                  // M_OG (05 E-5.17), stage dot_only (03 §9)
    pub target: TargetTerms,                                           // V_ℓ,t (E-5.19), strips (E-5.10), s_flat/hit
    pub part: PartResolved,                                            // P_part, ΣWP, ΣHS, M_son per hit location (E-7)
    pub crit: CritResolved,                                            // CC parts, Q (NP-1), CDM parts, p_vig, doughty
    pub status: StatusResolved,                                        // SC, SC_w, w_s (w_og), φ_s, A_sd, SD, caps, caps_og,
                                                                       // DoT coeffs, dot_vuln, heat inherit, n_ext
    pub ms: f64, pub ms_locked: bool,                                  // E-10
    pub firing: FiringResolved,                                        // FR, EFR, CT, burst, magazine, cost, AE, reload,
                                                                       // duty, ramp, spool, gauge, ScheduleInputs
    pub melee: Option<MeleeResolved>,                                  // 07 §5-§7 (below)
    pub xh: [ExtraHitResolved; N_XH], pub mech: MechResolved,          // E-14 and mechanic magnitudes
    pub gates: GateSet,                                                // dynamic gates feeding nonlinear stages (7.4)
    pub mask: StatusSet,                                               // statuses this attack (or externals) can apply
    pub hit_fraction: f64,                                             // T-41 f_a
}
pub struct CoResolved { pub behaviour: CoBehaviour, pub rho: f64, pub b_co: f64, pub kappa: f64, pub c_flat: f64,
                        pub c_status: f64, pub c_cold: f64, pub multiplicative: bool /* T-04 */,
                        pub flags: CoFlags /* direct_target_only, no_combo, co_dot_mode */, pub mproj: MprojMode,
                        pub c_x: f64 /* cond_flat_excluded part */, pub count: CoCountMode, pub count_manual: u8 /* T-05 */ }
pub struct TargetTerms { pub vuln: [DamageVec; Layer::N], pub strips: ArmorStrips, pub flat_per_hit: f64 }
pub struct CritResolved { /* cc0, cc_base_add, cc_rel, cc_abs, cc_floor, cc_final_mul, cc: f64 */ pub q: CritMult,
                          pub q_cond: CritMult, pub theta_p: Option<f64>,
                          /* cd_rel, cd_abs (static: no Cold, no doughty), m_cdf, p_vig: f64 */
                          pub doughty_rank: f64 /* b_r */, pub doughty: f64 /* E-2.5a at the static inputs */ }
pub struct ExtraHitResolved { pub p: f64, pub ty: Option<DamageType>, pub part: bool, pub status: XhProc,
                              pub dur_add: f64, pub on_pellet: bool, pub on_detonation: bool }
pub enum XhProc { Roll, Forced(StatusType), None }
pub enum GateAssign { Mean, Fixed(u8) /* bit k: GateSet.ids[k] on */ }
pub struct MeleeResolved { /* attack_speed, wind_up, swing, combo_dur, initial_combo, heavy_eff, ic_regen, k_tier,
                              m_cap, m_bcap, p_gain, p_acc, p_rc, tennokai_chance, tennokai_window: f64 */
                           pub tennokai_every: u32, /* wind_up_tk, power_spike, combo_gain_ext, heavy_eff_slam: f64 */
                           pub stealth_sdb: Option<f64> /* T-08, 04 E-4.16 */, pub m_ab: Option<f64> /* T-11 */,
                           pub finisher: Option<FinisherMode> /* T-34 */ }
```

| Stage | Function | Spec |
|---|---|---|
| ST-1 base vector | `resolve::base::base_vector(ctx, atk, w, sv)` | E-1, 01 §4.2-4.7 (progenitor, Incarnon flat split D-005, Strength) |
| ST-2 composition | `resolve::composition::ratios(ctx, base, sv, stance_phys, el, conv, tw)`; `compose(ctx, c, stance_phys, og, immune)` | E-2, 01 §4.9-4.12 (physical, stance physical, Phoenix Talons, conversions after the physical step, elements, Table 4.10-A per layer) |
| ST-3 quantization | `resolve::quantize::quantize(ctx, ratio)` | E-3, CD-02, D-001 compat |
| ST-4 base scalar | `resolve::scalar::base_scalar(ctx, base, a: ABuckets, g_f, co) -> ScalarY` | E-11, 04 §2 (behaviours, ρ_co, B_ref, G_F), E-4.3 `cond_flat_excluded` (`ScalarY::at_split`), E-4.5 two-bucket |
| ST-5 weapon multipliers | `resolve::weapon_mult::weapon_mult(ctx, sv, slots) -> WeaponMult` | 01 §4.15 (F), 04 §3 (G, G_F, M_OG, dot_only) |
| ST-6 body part | `resolve::part::part_factor(p_part, k_wp, wp, hs, m_son)` | E-7, 02 §4.5 |
| ST-7 crit | `resolve::crit::{crit_chance, crit_multiplier, crit_multiplier_terms, crit_tiers, doughty}` | E-4 (NP-1 f32, summation order of 02 E-2.5), E-5, E-6, 02 E-2.5a |
| status chance and procs | `resolve::status_chance::{status_chance, proc_weights}` | E-8, E-9, 02 §4.6-4.7 |
| multishot | `resolve::multishot::multishot` | E-10, 02 §4.1 |
| firing stats | `resolve::firing::firing_stats` | 06 §5, §10 |
| melee stats | `resolve::melee::melee_resolved` | 07 §5-§7 |

Terms that depend on target state (Puncture crit chance, Cold crit damage, CO count, Viral, Magnetic, armor) are not
folded into `ResolvedAttack`; they are applied by the kernel (EV) or the simulator (MC) from the state. The
composition is target-dependent where Table 4.10-A applies (Reactive Storm stays IMP, Sentient Incision becomes COR
while overguard is present): `ratio_q_og` and `status.w_og` hold the overguard layer's version, and `compose`
recomputes a stance strike whose `MeleeHit.s_phys` differs from the attack's (01 §4.1 steps 9-10, EC-01-34; ENG-4).
L3 passes the `cd_base_add` slot's sum as the one CDM0+ entry of `crit_multiplier_terms`, on the calculator path as on
the optimizer's (`Engine::resolve` is `resolve_point` on `cb.point()`, and a `BuildPoint` carries no per-source
entries): exact while at most one CDM0+ entry is active, which 02 E-2.5 states for current data; a build with more
gets the warning `rule.unverified { row = "02 E-2.5" }` from T17, which counts the entries of `CompiledBuild.sources`
(contract review ENG-R1, pass 3; C-36, ENG-19). The Prelude of Might term comes from its own rows: `cd_base_cond`
(CDM0c) is the last f32 addend of `q_cond`, and `cd_base_cond_thr` (θ_P) is `theta_p` (`Some` when `cd_base_cond` is
nonzero); the kernel and the simulator choose `q_cond` when the instance's final CC is below θ_P (02 E-2.5, strict;
contract review ENG-R7). L3 stores the quantized results `q` and `q_cond`.

`Engine::resolve_point(cs, x, attack, gates)` is the L3 entry on a borrowed `BuildPoint` (no heap allocation) for one
gate assignment; `Engine::resolve` is `resolve_point(cs, &cb.point(), attack, GateAssign::Mean)` (ENG-8).

---

## 5. Schedules

`schedule::ranged::ranged_schedule(ctx, ra, horizon)` implements 06 E-6.11 exactly: shots with cycle start, fire
time, hit time, magazine number and index, ammo before and after, cost, `first`/`last` flags, burst position, spool
index and rate, Kohm pellets, Peacemaker burst, ramp level; reloads (start, end, `from_empty`, reloaded fraction
`(Mag − left)/Mag` for `Per::ReloadedFraction`); sniper decay times; child hits per shot including delayed explosions
and lingering ticks (06 §9.3-9.4, E-6.25); one shot record per member and pull of a volley group (06 E-6.14b).
`schedule::melee::melee_schedule(ctx, ra, plan, tables, horizon)` implements 07 E-7.30 for one `MeleePlan` (07 §8.1:
`light_loop(C)`, `slide_loop`, `heavy_loop`, `build_spend(K)`, `tennokai`, `slam_loop`, `heavy_slam_loop`,
`glaive_cycle`, `cycle_loop`, `data_attack_loop`): strikes from the resolved combo tables (E-7.0b), timing from AS, W
and T_sw (07 §5), the expected-path combo count c̄ with gains, expiry, Power Spike decay, heavy spend and Initial Combo
regeneration (07 §6), heavy hit lists H_j (E-7.40), 9-C spawned attacks, Melee Duplicate and wave insertion order
(07 §8.4), the hit, heavy and combo records of 07 §8.2, and the steady quantities of E-7.37. The schedule never depends
on random draws. Both generators assume a checked event bound: `schedule::event_bound(ra, plan, tables, horizon)`
returns the bound or `EngineError::ScheduleTooLong` (beyond `MAX_SCHEDULE_EVENTS` = 10^7, an infinite rate included),
and `Engine::try_schedule` runs it first (ENG-16); a melee attack passes its plan and the resolved E-7.0b tables, from
which its strikes per loop and T_loop come (ENG-R3), a ranged attack `None` for both.

```rust
pub struct Schedule { pub shots: Vec<ShotRec>, pub reloads: Vec<ReloadRec>, pub segments: SmallVec<[Segment; 16]>,
                      pub children: Vec<ChildHits /* parent shot, child attack, count, headshot gate, hit times */>,
                      pub sniper_decay: Vec<f64 /* 06 E-6.16 decay times */>,
                      pub melee: Option<MeleeTrack /* heavy and combo records of 07 §8.2 */>,
                      pub cycle: CycleStats /* EFR, shots per magazine, duty ρ, T_mag, T_rel; melee T_loop */ }
pub struct ShotRec { pub attack: AttackIdx, pub index: u32, pub cycle_start: f64, pub fire: f64, pub hit: f64,
                     pub mag: u32, pub mag_index: u32, pub ammo_before: f64, pub ammo_after: f64, pub cost: f64,
                     pub flags: ShotFlags, pub burst: u32, pub burst_pos: u16, pub spool_index: u32, pub rate: f64,
                     pub pellets: f64, pub peacemaker_burst: u32, pub ramp: f64,
                     pub strike_mult: f64 /* S_j (light, slide strikes and waves, E-7.20) or H_j (heavies, heavy waves,
                                             E-7.20b) */, pub combo: f64 /* c̄⁻ */, pub weight: f64 /* p_vol or 1 */,
                     pub melee: Option<MeleeHit> /* the rest of the 07 §8.2 hit record; ENG-3 */ }
pub struct ShotFlags(pub u16);   // FIRST, LAST, AFTER_RELOAD, RELEASE (06); HEAVY, SLIDE, TENNOKAI, SLAM, RADIAL,
                                 // MELEE_HIT, STEALTH_OK, MELEE_CONTACT, ANIM_READ, INPUT_START (07 §7.5)
pub struct MeleeHit { pub kind: MeleeKind /* 07 §7.1 */, pub input: u8, pub strike: u8,
                      pub s_phys: [f64; 3] /* S^phys_{j,t}, EC-01-34 */, pub forced: [f64; N_ST] /* 07 §7.7 */,
                      pub r_in: f64 /* E-7.21 */, pub heavy: u8 /* h */, pub m: f64, pub m_b: f64,
                      pub gain: f64 /* g */, pub gain_extra: f64, pub n_anim: u8, pub n_thr: u8,
                      pub gunblade_pellets: bool /* E-7.45 pellet rule */ }
pub struct MeleePlan { pub kind: PlanKind, pub combo: ComboType, pub k: f64, pub policy: HeavyPolicy,
                       pub heavy_cancel: Option<f64> /* t_dmg, OQ-701 */, pub t_slam: f64, pub throw: ThrowKind,
                       pub t_cyc: Option<f64>, pub m_start: Option<f64> /* T-09, E-7.31; ENG-14 */,
                       pub light_combo_fixed: Option<f64> /* T-10, E-7.31; ENG-13 */ }
pub struct CompiledMelee { pub tables: MeleeTablesOwned /* E-7.0b result */, pub plans: SmallVec<[(AttackIdx, MeleePlan); 4]> }

pub fn plan_selection(toggles: &Toggles, key: &AttackKey, atk: &Attack) -> MeleePlanSel;            // T-30
pub fn melee_plan(sel: &MeleePlanSel, atk: &Attack, toggles: &Toggles, combo: ComboType, md: &MeleeData) -> MeleePlan;
pub fn resolve_combo_tables<'b>(b: &'b Bundle, w: &Weapon, stance: Option<StanceId>) -> MeleeTables<'b>;  // borrowed
pub fn resolve_combo_refs(b: &Bundle, w: &Weapon, stance: Option<StanceId>) -> MeleeTablesOwned;           // owned (ENG-21)
pub fn event_bound(ra: &ResolvedAttack, plan: Option<&MeleePlan>, tables: Option<&MeleeTables>, horizon: f64)
    -> Result<u64, EngineError>;                                                     // melee: plan and tables (ENG-R3)
```

**Inputs of `ranged_schedule`.** The generator receives no bundle, so `ResolvedAttack` carries every per-attack input
of 06 E-6.11 and of the variants it names, in `FiringResolved.sched` (fixed size, `Copy`; CONTRACTS.md T02a):

```rust
pub struct ScheduleInputs {
    pub timing: TimingBase,              // FR0, ΠM_fr, FR_cap, lock, CT0, BD0, Mag0, cost0 and the static sums ΣFR,
                                         // charge, ΣMag, Mag+, RS, ΣAE: re-evaluation at the points of 06 §5.9
    pub timing_buffs: [TimingBuff; 8], pub n_timing_buffs: u8,  // 06 §5.9 own-event buffs (on reload, from empty,
                                         // first/last shot) with duration and sum deltas; filled at L3 from the build
    pub cost_factor: f64,                // infinite-ammo state: 0 always on, 1 − u for uptime u, 1 otherwise (06 §5.9)
    pub reload: ReloadInputs,            // style magazine{R0} | by_round{t_se, t_sh} | regenerate{δ0, δ0 empty, ρ_rech,
                                         // Shedu} | none; δ_rel data rule, BurstReloadDelay, T-40 (06 E-6.9, E-6.10)
    pub form: Option<FormInputs>,        // Incarnon form: Mag_inc, T_sw, T-39; stops at its magazine end (06 E-6.14)
    pub t_ret: Option<f64>,              // returning projectile (06 §6.4)
    pub volley: Option<VolleyInputs>,    // lead, member count, members (attacks[..members]; ENG-20), cost per pull
                                         // (06 E-6.14b)
    pub on_reload: ReloadTriggered,      // attacks fired at reload start: from empty / partial (06 §9.5)
    pub ramp_start: f64,                 // s_rmp (06 E-6.15)
    pub spool: Option<SpoolInputs>,      // s_sp, Δ_sp (06 E-6.17)
    pub kohm_max_pellets: u8,            // P_max (06 E-6.18)
    pub peacemaker: Option<PeacemakerInputs>,   // 06 E-6.19
    pub sniper: Option<SniperInputs>,    // C_min, D_snp with Harkonar additions, T-12, T-38 (06 E-6.16)
    pub flight_time: f64, pub out_of_range: bool,   // d / v_proj at the T-27 distance; hitscan range (06 E-6.22, §8.4)
    pub children: [ChildLink; 8], pub n_children: u8,   // child attack, parent link, volley member, per-projectile,
                                         // f(a) split, headshot and stun gates, delay, lingering D_e and FR_child
                                         // (06 E-6.23-E-6.25)
}
```

`firing_stats` (T08a) fills the fields that come from the attack and weapon data and the stat vector; L3
(`Engine::resolve_point`, T13) sets the fields that come from the scenario's toggles (T-12, T-27, T-38, T-39, T-40, as
it sets T-42's `gauge_uses_per_min`, and the melee T-08, T-11, T-34 fields) and `timing_buffs` from the compiled build.
The variants are marked by the attack's roles (`WHOLE_MAGAZINE`, `RETURNING`, `RELOAD_TRIGGERED`, `GAUGE_GATED`,
`INCARNON_FORM`), T-41 by `ResolvedAttack.hit_fraction`.

**MeleePlan from the T-30 toggles.** L0 builds one `MeleePlan` per melee attack the scenario evaluates
(`CompiledMelee.plans`) in two steps (domain-model.md §9.3; 07 §8.1, §8.5). `plan_selection` picks the selection: the
attack's entry in `Toggles.melee_plans` (at most one per attack: when the document was resolved, an entry whose attack
key names no attack of the weapon was dropped with `doc.attack_unknown`, and a duplicate entry rejected the document,
`doc.attack_duplicate`; domain-model.md §9.3), else `Toggles.melee_plan` when its plan kind names the attack
(`light_loop`, `build_spend`, `tennokai` the Normal Attack, `heavy_loop` the heavy attack, `slide_loop` the slide
attack, `slam_loop` and `heavy_slam_loop` the slams, `glaive_cycle` the throws, `data_attack_loop` a directly selected
data attack), else `Auto`. `melee_plan` maps `Auto` by attack kind (Normal Attack → `light_loop` of
`BuildDoc.stance_combo`, heavy → `heavy_loop`, slide → `slide_loop`, slam → `slam_loop`, heavy slam →
`heavy_slam_loop`, throws → `glaive_cycle`, no light combo → `cycle_loop`, a directly selected data attack →
`data_attack_loop`), copies the plan parameters, and applies T-09 (`Toggles.heavy`: attacks without a data role use
`heavy_loop`, `m_start` = M_start clamped to M_cap, D-063) and T-10 (`light_combo_fixed`). The start count is computed
by the schedule, because Initial Combo is a build stat (E-7.15) that L0 does not see: c(0) = IC, or
max(IC, k_tier·(M_start − 1)) with T-09, from `ra.melee` (E-7.31; ENG-14).

**Inputs of T-41, T-42 and T-43.** They are scenario toggles that the schedule and the evaluators read, never items:

| Toggle | Compiled into | Read by |
|---|---|---|
| T-41 pellet hit fraction (`Toggles.pellet_hit`, 06 E-6.24, 08 E-8.2, E-8.17) | `CompiledScenario.hit_fraction[a]` (1 without an entry) → `ResolvedAttack.hit_fraction` | the schedule keeps unthinned instance counts (E-6.24: children are not thinned by the parent's fraction; headshot-only children only for parent instances that hit); EV multiplies every instance's existence probability by f_a; MC draws `"hit_frac"` first for every instance of an attack with f_a < 1 (E-8.17) and a missed instance has no damage, procs, extra hits, duplicate or place in the CO order (EC-08-63); the sniper-combo counter of E-6.16 counts hits, not expected hits: EV propagates the exact pmf of the hits p_i (the pellet pmf thinned binomially by f, p = 0 applying the miss decrement; E-6.16a divides by f·MS, 06 §7.2), MC draws one hit per projectile after the multishot draw |
| T-42 gauge uses per minute (`Toggles.gauge_uses_per_min`, 06 §6.7) | `FiringResolved.gauge_uses_per_min`, set at L3 for gauge-gated attacks (`FiringResolved.gauge_gated`) | `ranged_schedule`: E-6.14a uses at s_0 and s_m = s_0 + m·60/u without magazine cost; no steady metrics (`ViewEv.dps_burst` etc. absent, 08 E-8.14) |
| T-43 external statuses (`Toggles.external_statuses`, `Toggles::external_stacks` folds T-06 in; 03 §6.3) | `CompiledScenario.external[s]` (0 on immune types), combined by max with the `external.*` stats into `StatusResolved.n_ext` | EV `PointState.n_ext` (E-8.3 `n = N_ext + min(N, cap − N_ext)`, presence 1, n_ext of E-8.10, Heat strip step 4 and the external episode of 08 §5.4); MC `McState.external` (own stacks use the free slots, no expiry, no damage, no on-proc triggers) |

`segments` is the run-length form (maximal runs of shots with constant spacing and equal fields other than time); a
20 s rifle window is about 3-10 segments. `WindowExact` visits every shot; `WindowFast` and `Bound` work per segment.

---

## 6. Enemy side

`enemy::scaling::scale_enemy(ctx, e, setup, toggles)` computes pools with the f32 blend of NP-2 (`np2_blend`, 05 §6.3),
Steel Path, Eximus, party and overguard rules (05 §7-§8), the caps with `og_present` false and true (03 E-3.2), and
applies T-28 (05 §15.5) and the T-03 / T-32 part choice (05 §4.3) from the toggles (ENG-9, ENG-10);
`enemy::armor::effective_armor(a0, strips, heat_strip, n_cor)` and `armor_dr(a)` compute A_eff and DR(A) (E-15, 05
E-5.10 with ability, aura, companion and T-07 %-strips, point strips and f_flat from s_flat; Heat and Corrosive
strips); `enemy::layers::instance_on_layer(ctx, w, layer, st, en)` applies E-12's enemy side per layer (φ, Viral,
Magnetic, V, armor, armor floor of 1, R_ℓ with the class, status and part DR and the Guardian aura of 05 E-5.20 and
E-5.27, attenuation E-5.21, M_dps E-5.22, the tick rules of E-5.24, CD-09) and `instance_on_layer_types` returns the
per-type values D^ℓ_t for the breakdown of 08 §8.2; `enemy::layers::route_layered(ctx, dmg, pools, t, en, thresholds,
gate)` implements ST-9 (OG → SH → HP, Toxin routing and exceptions, overflow, shield gate CD-08 when `gate`; the EV
TTK path passes `gate = false`, 05 §17) for the `layered` view only; `enemy::dr` implements 05 §15 (attenuation, flat
DR, adaptation, T-28).

```rust
pub struct ScaledEnemy {
    pub enemy: EnemyId, pub level: u16, pub faction: FactionId,
    pub pools: [f64; Layer::N],           // OG, SH, HP (HP_max for thresholds and attenuation)
    pub armor0: f64, pub phi: [DamageVec; Layer::N],
    pub caps: [u8; N_ST], pub caps_og: [u8; N_ST],          // 03 E-3.2 with og_present false / true (ENG-10)
    pub immune: StatusSet, pub shield_bypass: DamageTypeSet,
    pub dr: [f64; Layer::N],              // 1 − innate_dr[ℓ] (1 with T-28 off)
    pub status_dr: f64, pub dr_class: DrClass, pub dr_class_og: DrClass,     // 05 E-5.20 (0 with T-28 off)
    pub attenuation: Option<Attenuation>, pub special_dr: Option<u16>,       // 05 §15 (None with T-28 off)
    pub unique_dr: bool,                  // T-28
    pub flags: EnemyFlags,                // toxin_blocked, shield immunity, cc_immune, vip, strip immunity, …
    pub guardian_aura: bool,              // 05 E-5.27 (kept with T-28 off)
    pub parts: SmallVec<[ScaledPart; 2]>, // [0] the T-03 / T-32 part, [1] the body part (radial instances)
    pub is_object: bool, pub shield_regen_possible: bool,
}
pub struct InstanceClass { pub radial: bool, pub melee: Option<bool> /* None: special, no dr_class */,
                           pub tick: bool, pub ms_att: f64 /* E-5.21 */ }
pub struct TargetStateView { pub stacks: [u8; N_ST], pub armor_eff: f64, pub vuln: [DamageVec; Layer::N],
                             pub status_instance: bool, pub part_dr: f64, pub class: InstanceClass,
                             pub adapt_res: DamageVec /* layered only */, pub shield_up: bool /* layered only */ }
pub struct LayeredHit { pub og: f64, pub sh: f64, pub hp_bypass: f64, pub hp: f64, pub hp_by_type: DamageVec,
                        pub gate_factor: f64 /* g(instance) of E-5.25: 1, 0 (slam radials) or 0.05 */ }
pub struct Pools { pub og: f64, pub sh: f64, pub hp: f64, pub gate_until: Option<i64> /* µs grid, 08 E-8.0a */,
                   pub adaptation: AdaptationState, pub health_below: u32 /* bit k: below thresholds[k] */ }
pub struct RouteOut { pub dealt: [f64; Layer::N], pub discarded: f64, pub gate_started: bool, pub killed: bool,
                      pub og_broken: bool, pub sh_broken: bool /* Magnetic-break procs, E-5.25 */ }
pub struct ArmorStrips { pub ability: f64, pub aura: f64, pub toggle: f64, pub companion: f64, pub flat: f64,
                         pub points: f64 }
```

The weapon-side overguard factor M_OG is `ResolvedAttack.m_og`, a scalar the caller applies on the OG layer
(`instance_on_layer` leaves it out). `LayeredHit.hp_by_type` is D^HP_t per type, from which routing forms h_t for the
adaptation state (05 E-5.23, E-5.25).

---

## 7. EV evaluator

Spec 00 §4.2 binds the algorithm and 08 §5 fixes it (E-8.2 to E-8.15, assumptions A-1 to A-9). This section maps the
equations onto code; where it is shorter than 08, 08 wins.

### 7.1 The kernel

`ev::kernel::point(cx, ra, inst, st, view) -> PointOut` evaluates one target-state point: given the evaluation context
`cx: &EvalCtx` (rules, compat flags, trace; the compiled scenario with the scaled enemy and the toggles; the attack's
dynamic terms `cx.coeffs` and the build's condition ids), the instance `inst: &Instance` (kind, anchor, shot record),
the expected state at time τ and a view, it returns the expected direct damage per pellet, the expected DoT damage per
proc of each DoT status landing at τ, the expected extra-hit damage, and the expected procs per pellet. It calls the
attack's mechanics with the instance's expected `InstanceOutcome` (section 2.1) and applies the enemy side to their
derived instances (`EvAcc.hits`) jointly with the parent's state (E-8.9 step 1). T02a's `point(ra, st, view)` had
neither context nor instance (contract review ENG-1).

```rust
pub struct PointState {                         // fixed size, Copy (08 E-8.3, E-8.4)
    pub t: f64, pub mu: [f64; N_ST], pub certain: [u8; N_ST], pub z: [f64; N_ST], pub n_ext: [u8; N_ST],
    pub heat_present: f64, pub heat_strip: [f64; 5],         // P_H, P(k_h = 0 … 4) (E-8.5-E-8.7)
    pub combo: f64, pub sniper_combo: f64,                    // c̄ (07 §6.9), M_snp (06 §7.2)
    pub cond_stacks: [f64; MAX_COND_VARS],                    // E[stacks] per build condition (position in cb.conditions)
    pub switch_on: [f64; MAX_COND_VARS],                      // P(switch on) per build condition (E-8.9 step 4c, A-12)
    pub counters: [f64; N_MECH],                              // counter_bonus counters (expected path)
    pub marks: ShotMarks, pub pellet: u32,                    // first/last shot, reloaded fraction; pellet j
    pub rag: f64,                                             // ρ_RAG,e (E-8.27, A-13; branches RAG = 0, 1, TV-08-30)
    pub s_flat: f64,                                          // expected flat strip points (05 E-5.11, E-8.9 step 5)
    pub og_present: bool,                                     // 08 §4.5 (layered EV path: expected OG > 0)
    pub hp_frac: f64,                                         // expected HP/HP_max (layered EV path, thresholds)
}
```

| Quantity | EV treatment |
|---|---|
| Crit | `E[c] = 1 + E[k]·(h·CDM − 1)`, linear in CC (CD-13); with Vigilante via E-6 and critical promotion `tier_add` (02 E-2.6) |
| Hit-outcome gates | the gated term only on the k = 0 branch with weight P(k = 0)·P(no proc of the instance) (domain-model.md §11.1 `OutcomeFilter`) |
| Puncture CC and Cold CDM | `E[c]` is enumerated over the joint support of Puncture stacks (0..cap) and Cold stacks (0..cap + Frozen) under independent capped Poisson laws: at most 6 × 11 states |
| Viral, Magnetic, Cold, Puncture functions | `E[f(n)]` by a capped Poisson sum (one `exp` plus a recurrence), never `f(E[n])` |
| Armor | `E[1 − DR(A_eff)]` over the joint distribution of Corrosive stacks, Heat presence and the Heat-strip ramp state |
| Distinct count n | `E[n] = Σ_s (1 − e^{−μ_s})` + externals (T-06); T-05 `weapon_types` and `manual` replace it; sequential-pellet correction of spec 00 §4.2 for pellet j of a shot |
| CO | per the attack's `CoSpec` (E-11): `adding` adds `ρ_co·B_ref·E[C]`, `multiplying` multiplies by `1 + E[C]`; C is affine in n, n_COL and condition stacks, so its expectation is exact under independence |
| Proc counts | `SC·w_s + φ_s` per pellet; mechanic procs (`proc_on_event`) from event rates; exclusive groups `1 − Π(1 − p_i)` |
| DoT | per proc: ticks at their landing times with the target state at that time (CD-05); weapon-side seed with the inducing hit's expected crit (OQ-017 decision) |
| Extra hits | E-14 per pellet, with the parent's expected crit and part factors |
| Condition stacks (`simulated`, class (b)) | 04 E-4.9 (steady) and 08 E-8.15 (window: refresh, independent timers, exact chains for shared timers) |
| Counters (`counter_bonus`), melee combo | expected path (07 §6.9: c̄ and M(c̄); counters by their expected gain and reset probabilities); gap class G-11 |
| Products of state variables | per status group (E-8.9 steps 1-4): a term's factors are grouped by status group and the expectation is the product over groups; factors reading the same group are expanded jointly (TV-08-09), never `E[C]·E[rest]` |

Only statuses in `ra.mask` are visited, and `e^{−μ}` is cached per status per point.

### 7.2 Fidelities

| Fidelity | Points | Definition | Exactness |
|---|---|---|---|
| `Steady` | 1 (+ the Heat window mean) | μ_s = λ_s·D_s; combo, sniper combo, spool, ramp at steady values (06 §10.4); `dps_burst`; `dps_sustained` via the duty ρ | exact for the spec's steady definitions |
| `WindowExact` | every shot and every DoT tick in [0, t_win] | spec 00 §4.2 window walk: μ_s(τ) = expected procs of earlier shots still alive at τ (two-pointer sliding window per status, O(1) amortized per shot); ticks evaluated at their landing times | the definition of `dps_window`; used for everything the user sees and for final optimizer rankings |
| `WindowFast` | Gauss-Legendre, 3 nodes per smooth piece | segments of evenly spaced shots are replaced by their shot density; μ_s(τ) is piecewise linear with breakpoints at segment ends, segment ends + D_s, and Heat-ramp steps; used only when the window has more than 64 shots (otherwise it is `WindowExact`) | tested ≤ ε = 0.5 % relative to `WindowExact` on the golden corpus and on 10,000 random builds per weapon class; classes over budget fall back to `WindowExact` |
| `Bound` | 8 intervals | per interval [a, b): exact shot count from the schedule × kernel at an upper-envelope state (μ_s ≤ p_s × shots in (a − D_s, b], combo and condition stacks at their interval maxima) + an upper bound of DoT ticks landing in the interval | provable `Bound ≥ WindowExact` when the kernel is non-decreasing in every μ_s; with a `distinct_below` predicate active, that status's μ is bounded on both sides and the kernel is evaluated at both ends; property-tested |

### 7.3 Window walk details

- **Fresh state** (spec 00 §4.1): no statuses, full magazine, combo at Initial Combo, sniper combo 0, ramp and spool
  at start, condition stacks per T-02.
- **Heat**: presence, ignition and end probabilities E-8.5, strip-counter distribution E-8.6 (window) and E-8.7
  (steady), pool damage by the per-ignition decomposition E-8.8; steady outputs use the Heat window mean `H_burst`,
  `H_sust` of E-8.14.
- **DoT**: per-proc landed damage with cap survival R_s(e, u) and the ELE/GAS "+1" per tick group (E-8.11); Blast
  host hits per A-9.
- **Groups** (T-22 and parent+child): `ev::group` merges the schedules of the unit's attacks by time; μ_s(τ) sums the
  procs of every member; each member's damage uses the shared state. Parent+child summed outputs add member damage.
- **Views**: `health`, `armor`, `shield`, `overguard` are factors per instance against a non-depleting target; the
  status state is the same in every view (spec 00 §4.1), so one walk serves four views.

### 7.4 Gate splitting

A dynamic gate (a conditional effect whose truth is random under `simulated`) that feeds a nonlinear stage
(composition and quantization, proc weights, a CC threshold) cannot be linearized. `ev::gates` evaluates the attack
once per combination of such gates (≤ 4 binary gates, ≤ 16 resolutions) weighted by the gates' probabilities, assuming
independence: `ResolvedAttack.gates` names the gates (condition ids), and `Engine::resolve_point(cs, x, attack,
GateAssign::Fixed(mask))` re-resolves the attack for each combination (01 §4.16, E-1.14). Beyond 4 gates it linearizes
the remaining ones and emits warning `ev.approx.gate_split_limit` (`Workspace::warn`). Under `all_max` and `none` every
gate is constant, so no split happens (the optimizer's default).

### 7.5 TTK and hit metrics

- **EV TTK** is E-8.21: the window walk with `T_end = t_ttk_max` on the layered view (expected damage per layer, routing
  05 E-5.25 with no shield gate, fractional overflow, OG overflow discarded, expected Magnetic-break procs); `T_EV` is
  the time of the first consumer after which HP ≤ 0, κ the overkill fraction of that consumer, plus shots and ammo to
  kill. Not killed by `t_ttk_max`: extrapolation per layer with `dps_sustained` and `ttk_extrap = true`; a positive pool
  with zero DPS gives "never". The band of E-8.26 is 0 (killed), 1 (extrapolated) or 2 (never).
- **Hit metrics** (08 §10): `hit_infight[v]` = expected damage of one parent hit at the steady state: direct + extra
  hits + flat-per-status + the credited DoT of its procs (`L_s` of E-8.14); `hit_fresh[v]` = expected damage of the
  first shot on the fresh target: all its instances at the first fire time (pellets, zero-delay children, extra hits)
  plus the credited DoT of their procs evaluated with the state after that shot's P4b and no later shot.
- **Tie-break metrics** (08 §10): `ammo_window` = Σ c_i over shots with s_i ≤ t_win; `ammo_to_kill` from E-8.21.

### 7.6 Approximation classes

EV results are exact for the spec's definitions under assumptions A-1 to A-9 of 08 §5.1; the remaining differences to
MC are the gap classes G-1 to G-11 of 08 §12 (Poisson arrivals, multinomial correlation, Frozen, Blast timing,
minimum of 1 and attenuation at conditional expectations, condition stacks, capped Heat pool, mixed Heat durations,
Knockdown/Lifted exclusivity, EV TTK without the shield gate, melee expected combo path). `CalcReport` sets the
deviation flag when E-8.24 fails: `|dps_window − x̄| > 2.5758293035489004·SE + 0.02·|x̄|` with n_runs ≥ 1000, and the
warning lists the gap classes active in the scenario (08 §8.11).

### 7.7 Outputs of the EV evaluator

The EV outputs are the DTOs of `wfd-model::api::calc` with the shapes accepted by the manager's rulings on C-23 …
C-30 (CONTRACTS.md T01b; spec 08 wins over the earlier required fields): metrics that 08 reports as "not available"
are `Option`, κ exists only in band `killed`, and the view travels next to its result.

```rust
pub struct EvOut { pub views: SmallVec<[(View, ViewEv); 5]> }
pub struct ViewEv {
    pub dps_burst: Option<f64>, pub dps_sustained: Option<f64>,  // 08 E-8.14; None for gauge-gated attacks (C-23)
    pub dps_window: f64,                                    // 08 E-8.12 (headline)
    pub per_shot: Option<f64>, pub per_magazine: Option<f64>, // 08 §8.5; None for gauge-gated attacks, per_magazine
                                                            // also None for N_mag = ∞ (C-26)
    pub per_type: BTreeMap<DamageType, f64>,                // landed direct DPS by type, nonzero types (08 §8.2)
    pub per_source: Vec<SourceDps>,                         // 08 §8.2: { source: "direct.HEA" | "dot.SLA" | "heat" |
                                                            // "blast" | "xh.<EffectKey>" | "duplicate" | "child.<slug>", dps }
    pub stacks: Vec<StackStat>,                             // E[n_s], P(n_s ≥ 1), P(n_s = cap) (steady, 08 §8.3)
    pub distinct_mean: f64,
    pub crit_tiers: [f64; N_TIER_BINS],                     // window distribution, tier k = 0..=7, index 7 = "7 or
                                                            // more" (08 §8.4); the counterpart of ViewMc.crit_hist
    pub crit_tiers_steady: Option<[f64; N_TIER_BINS]>,      // steady distribution; None for gauge-gated (C-30)
    pub series: Vec<f64>,                                   // expected DPS per bin (08 §8.1)
    pub uptime: Vec<StatusSeries>,                          // uptime[s][k], stacks_mean[s][k] (08 §8.3, C-30)
    pub pending_dot: f64,                                   // 08 §8.7
    pub ttk: Option<TtkEv>,                                 // layered view only
    pub hit_fresh: f64, pub hit_infight: Option<f64> /* None for gauge-gated, C-23 */, pub ammo_window: f64,
}
pub struct TtkEv { pub band: TtkBand /* Killed | Extrapolated | Never */,
                   pub seconds: Option<f64> /* T_EV; None = never */,
                   pub kappa: Option<f64> /* present exactly in band Killed (E-8.21, E-8.26; C-24) */,
                   pub shots_to_kill: Option<u32>, pub ammo_to_kill: Option<f64>,
                   pub shield_regen_possible: bool /* 05 §10.5 */,
                   pub breakdown: Vec<SourceDamage> /* damage by source up to T_EV, 08 §8.2 (C-30) */ }
```

The evaluators accumulate breakdowns as `outputs::SourceKind` (`Direct(type)`, `Dot(status)`, `Heat`, `Blast`,
`ExtraHit(mechanic)` (Blast-detonation extra hits included), `FlatOnProc(mechanic)` (ENG-23), `Duplicate`,
`Child(attack)`, `MagneticBreak`); `outputs.rs` maps them to the source ids above (`FlatOnProc` to the `xh.<EffectKey>`
id of its effect: 08 §8.2 lists no source of its own for flat damage on proc).

### 7.8 Cost

Measured by the P-O prototype on one 2.1 GHz core: one target-state point about 140 ns; a 300-shot `WindowExact`
about 42 µs. The calculator evaluates every attack and every present view with `WindowExact` on each change (a few
hundred µs per weapon); `Steady`, `WindowFast` and `Bound` exist for the optimizer.

### 7.9 Kernel API for the optimizer

```rust
/// Holds the engine (rules for the StageCtx of its EvalCtx, L3 resolve_point; contract review ENG-R2); Engine::kernel
/// sets it.
pub struct Kernel<'a> { engine: Engine<'a>, cs: &'a CompiledScenario, objective: &'a ObjectiveSpec,
                        statics: AttackStatics<'a> }
impl<'a> Kernel<'a> { pub fn new(engine: Engine<'a>, cs, objective, statics: AttackStatics<'a>) -> Self;
                      /* engine(), scenario(), objective(), statics() */ }
/// Σ_u w_u·M_u over evaluation units (08 §10); TTK metrics take exactly one unit, a single attack or a T-22 combined
/// unit. `units` replaces `attacks: [(AttackIdx, f64)]` (contract review ENG-24); `attacks()` lists their members.
pub struct ObjectiveSpec { pub metric: Metric, pub view: View, pub units: SmallVec<[(EvalUnit, f64); 2]>, pub window: f64 }
pub use wfd_model::api::Metric;  // DpsWindow, DpsBurst, DpsSustained, Ttk, TtkNoarmor (C-28), HitFresh, HitInfight
pub enum Fidelity { Steady, WindowFast, WindowExact }
/// Borrowed per-build-attack parts, as CompiledBuild holds them (ENG-7); CompiledBuild::point() borrows one.
pub struct BuildPoint<'p> { pub attacks: &'p [AttackIdx], pub sv: &'p [StatVec], pub elements: &'p [ElemOutcome],
                            pub coeffs: &'p [SmallVec<[Coeff; 4]>], pub mech: &'p [MechSet], pub conditions: &'p [u16] }

/// Ordered score key: greater is better. Lower `band` is better, then higher `primary`, then higher `tie`, then
/// higher `tie2` (`a > b` means a is better; optimizer.md §5.2 prunes when bound < θ and keeps max(s_best, s)).
/// DPS and hit metrics: band 0, primary = metric value, tie = −ammo_window (08 §10 tie-break).
/// TTK and ttk_noarmor (E-8.26): band 0 killed by t_ttk_max: primary = −T_EV, tie = κ (larger overkill first),
///               tie2 = −ammo_to_kill; band 1 extrapolated: primary = −T_EV; band 2 never: primary = the fraction of
///               the total pool (OG + SH + HP) removed by t_ttk_max on the expected path of E-8.21, tie =
///               dps_sustained[auto] (C-33: the spec's band 2). Unused fields are 0; bounds set the tie fields to +∞.
#[derive(Clone, Copy)] pub struct Score { pub band: u8, pub primary: f64, pub tie: f64, pub tie2: f64 }
impl Ord for Score { /* greater is better: lower band, then higher primary, tie, tie2; floats compared with
                        total_cmp after Score::key (−0.0 reads as +0.0, NaN as −∞) */ }
impl Score { pub const WORST: Score; pub fn new(band, primary, tie, tie2) -> Score /* −0.0 → +0.0, debug NaN check */;
             pub fn key(x: f64) -> f64; }   // PartialEq and Eq agree with Ord; From conversions with wfd_model::api::Score

impl Kernel<'_> {
    pub fn score(&self, x: &BuildPoint, fid: Fidelity, ws: &mut Workspace) -> Score;
    /// Upper bound, in the order above, of `score(y, WindowExact)` for every y whose Inc slots are ≤ `upper`'s and whose
    /// Unknown slots, elements and mechanics equal `upper`'s (optimizer.md §4).
    pub fn bound(&self, upper: &BuildPoint, ws: &mut Workspace) -> Score;
}
/// Per-thread scratch arrays (no heap allocation in score/bound after warm-up) and the warning and error channel of
/// the EV evaluations (section 15): warn, warnings, take_warnings, fail (first error kept), error, check.
pub struct Workspace { /* scratch: Vec<f64>, warnings: Vec<Message>, error: Option<EngineError> */ }
```

**Why the TTK bound is valid.** `bound` runs the E-8.21 walk on the upper-envelope damage path of section 7.2 (every
consumer's expected damage per layer is an upper bound for all completions) and the upper bound of `dps_sustained`
from the same envelope. Pools then deplete no later than on any completion's path, so: if the envelope kills by
`t_ttk_max`, its band is 0 and `T_bound ≤ T_EV(y)` for every y (band 0) and for y in bands 1-2 the bound's band is
lower, hence better; if the envelope does not kill, its extrapolated time uses smaller remaining pools and larger
`dps_sustained`, so `T_bound ≤ T_EV(y)` in band 1, and band 2 only when every completion is band 2, and then the
envelope path removes at least the fraction of every completion (pools deplete no later on it), so its band-2 primary
is an upper bound (C-33). The tie fields of a bound are set to +∞ (κ ≤ 1 and the ammo terms are then irrelevant).
So `bound(upper) ≥ score(y)` in the lexicographic order for every completion y, which is all branch-and-bound needs;
the property test of testing.md §2.3 checks it on random partial builds for every metric.

---

## 8. MC simulator

### 8.1 State and events

`McState` is the run state of 08 §6.2 (`mc/state.rs`; one per pass, two in pass V on an enemy with overguard, 08 §4.5):

- **Target state**: per-status `Vec<StackEntry>` ordered by application (`t_apply_us`, `t_expire_us`, source attack,
  `seq`, DoT slot): independent timers, replacement of the smallest application time beyond the cap (03 §6.2, CD-07);
  a `Vec`, because SLA, ELE and TOX have cap ∞ and COR 10 + β_COR (03 §4.1), so no fixed size and no panic. DoT
  instances in a slab (`DotInst`: status, seq, snapshotted seed (CD-05), next tick, cancelled, Magnetic break), the
  shared Heat episode (`HeatState`: ignition, end, next pulse, pool, strip counter and pending steps), Blast fuses
  (`BlastFuse`: seq, host-hit snapshot, expiry, source attack, and the anchor of the instance that created the stack,
  for the `"xd/"` draws, ENG-22), Frozen and ragdoll ends, the single-instance statuses VOI, KND, LIF, MIC, external
  stacks, the flat armor strip s_flat, and the layered pools (`Pools`).
- **Attacker state**: schedule cursor per firing attack (shots inserted lazily; the magazine state is read from the
  schedule records), combo counter and timer (07), sniper combo, condition-effect states (04 E-4.8) per condition id,
  on/off switches per condition id (A-12), `counter_bonus` counters per `mech.<n>` slot.
- **Events and processing**: exactly E-8.18 (08 §6.3): shot (inserted lazily from the schedule), delayed child or
  lingering tick (`McEventKind::Child`), stack/DoT expiry, Frozen end, ragdoll end, Heat strip step, condition expiry or
  decay step, combo expiry (07), sniper decay (06), DoT tick (ELE/GAS grouped at processing time), Heat pulse, Heat
  episode end, Blast fuse expiry, Magnetic-break tick (layered passes), on/off switch. Each instance's mechanics are
  called once per instance in P4a with its `InstanceOutcome` (section 2.1); their derived instances are evaluated once
  per view (pass V) or routed once (layered passes), their procs applied in P4b after the instance's own.
- **Order**: a binary heap keyed `(τ̂, phase, sub_key, seq)` (08 §4.3, §6.2, E-8.0a): τ̂ the µs grid time; `Phase` =
  P1 transitions, P2 damage, P3 Heat end, P4 shot (1-4); `SubKey` = the 08 §4.2 order key (firing attack, shot index,
  child attack) in P4, the creation sequence number of the oldest proc of the tick group in P2, 0 in P1 and P3; `seq`
  the per-run creation counter. The steps a-d of a shot (instances, procs, zero-delay ticks, DoT detonation) run inside
  that shot's P4 step, so a stack applied in P4b of S affects every later shot at τ (03 §7.1, §7.2); they are not heap
  phases. The accumulators and the heap are the simulator's own (`mc/sim.rs`).

### 8.2 Random numbers

`mc/rng.rs` implements spec 08 E-8.16 and E-8.17, reproduced verbatim (the spec is normative; this copy is for the
implementer's convenience; ENG-18 refreshed it):

```text
E-8.16 Counter-based RNG (overview U-6, 4.3)
  generator:  Philox4x32-10 (Salmon et al., Random123), multipliers 0xD2511F53, 0xCD9E8D57, Weyl constants
              0x9E3779B9, 0xBB67AE85; round: (hi0, lo0) = mulhilo(0xD2511F53, c0), (hi1, lo1) = mulhilo(0xCD9E8D57, c2),
              c ← (hi1 ⊕ c1 ⊕ k0, lo1, hi0 ⊕ c3 ⊕ k1, lo0); the key is bumped by the Weyl constants before rounds 2..10
  key:        (k0, k1) = (seed mod 2^32, floor(seed / 2^32))                       seed: u64 (T-29, default 0)
  counter:    c0 = run (u32)
              c1 = rng_mech(name) = FNV-1a 32-bit of the mechanic id's UTF-8 bytes (offset 0x811C9DC5, prime 0x01000193)
              c2 = (a << 20) | i                     a < 4096 attack index, i < 2^20 shot index of the anchor
              c3 = (j << 16) | b                     j < 65536 sub-instance index, b < 65536 block index
  draw:       the k-th draw (k = 0, 1, 2 ...) of a given (run, anchor, mechanic) in processing order uses block b = k >> 1;
              half 0 (k even): x_lo = x0, x_hi = x1;  half 1 (k odd): x_lo = x2, x_hi = x3;
              m = ((x_hi << 32) | x_lo) >> 11, i.e. u = floor((x1·2^32 + x0)/2^11)·2^−53 ∈ [0, 1) for half 0
  anchors (E-8.0 for instances):
     per-instance draws (02 4.8: "crit_roll" ... "st"; "enc" once per instance tried, "enc_type" on the instance that
              succeeded, 02 E-2.17; "deb", "deb_type", "deb_comp", 02 E-2.17b; "hit_frac" of T-41, drawn first, only
              when f_a < 1): the instance's anchor
     shot-level draws ("ms_pellets", 02 E-2.2, E-2.3): j = 0xFFFF
     reload-level draws: a = the firing attack whose magazine reloads, i = reload number, j = 0xFFFE
     per-run draws (the random phase of a class (c)/(u) on/off process, E-8.18): a = 0, i = 0xFFFFF, j = 0xFFFD,
              mechanic "eff/<effect id>/phase"
     conditional effects ("eff/<effect id>", trigger draws of 04 E-4.8 with p < 1): the anchor of the triggering
              unit (the instance; for a shot unit its j = 0 instance; for a reload unit the reload anchor)
     melee draws (07: "combo_acc", "combo_rc", "combo_curse", "glaive_volatile"): the anchor of the hit or tick that
              rolls them (a Volatile explosion: the anchor of its glaive throw hit)
  derived instances: extra hits use the parent anchor and the mechanic id "xh/<source>/<name>" (source = catalogue id
              of 04, e.g. "xh/xata/sc_count"); Melee Duplicate: "dup/<name>"; Xata's Whisper detonation extra hits
              (04 E-4.13a): "xd/<name>" on the anchor of the instance whose proc created the detonating stack
  an index outside its range is an error (no wrap-around); i = 0xFFFFF and j ≥ 0xFFFD are reserved for the sentinel
              anchors above (E-8.0 keeps every instance anchor below them)
```

```text
E-8.17 Mapping draws to outcomes
  Bernoulli(p):         success iff u < p          (p ≤ 0 never, p ≥ 1 always; exact fractions, D-017)
  floor + Bernoulli(f): n + [u < f]                (pellets, crit tier, proc count: 02 E-2.2, E-2.6, E-2.12)
  categorical(w):       the first s in the enumeration order of overview 2.2 with u < Σ_{t ≤ s} w_t; if rounding
                        leaves u ≥ Σ w, the last s with w_s > 0
  uniform over a list:  index floor(u·L)
```

Implementation rules:

- `MechanicId(u32)` is the FNV-1a hash of the mechanic id **string**, computed at L0 (`mc/keys.rs`): the fixed ids of
  02 4.8, 07 and T-41 (`FIXED_MECHANIC_IDS`, in the E-8.17 draw order: "hit_frac", "ms_pellets", "crit_roll",
  "crit_vig", "sc_count", "sc_type", "cedo_type", "aug_type", "aug_ele", "ib", "welt", "hm", "st", "enc", "enc_type",
  "deb", "deb_type", "deb_comp", "combo_acc", "combo_rc", "combo_curse", "glaive_volatile") are a `const` string list;
  derived ids ("xh/<source>/<name>", "dup/<name>", "xd/<name>", "eff/<effect key>", "eff/<effect key>/phase") are
  formed from the bundle at L0. There is no Rust enum of mechanic ids.
- L0 hashes every id the scenario can use into `CompiledScenario.mech_ids` (`MechIdTable::insert(&str)`, which needs
  the strings) and fails with `EngineError::MechanicIdCollision` if two distinct strings hash to the same value; a unit
  test asserts that the fixed list and every derived id of the mini fixture and of the real bundle (in
  `cargo xtask effect-smoke`) are distinct.
- `McRng::new(seed, run, &mut OrdinalTable)` holds the key and the run and keeps one ordinal counter per (anchor,
  mechanic) in the reusable `OrdinalTable`; `set_anchor(anchor)` moves to an anchor (`Anchor { a, i, j }` with the
  sentinels `SHOT_LEVEL_J` 0xFFFF, `RELOAD_J` 0xFFFE, `RUN_J` 0xFFFD and `RUN_I` 0xFFFFF) and returns
  `EngineError::RngIndexRange` for an index outside the ranges (a ≥ 4096, i ≥ 2^20, j ≥ 65536), never wrapped;
  `next_u(mechanic)` takes the next ordinal of its mechanic at the current anchor, and records the first range error
  (block b ≥ 65536 after 131,072 draws of one mechanic at one anchor) instead of wrapping or panicking, returning 0.0;
  the simulator calls `McRng::check()` after each event and fails the evaluation with that error.
- Two builds evaluated with the same seed see the same draw for the same (run, anchor, mechanic, ordinal): common
  random numbers for finalist confirmation. Known answers TV-08-01 (Philox) and TV-08-02 (keyed draws and the FNV-1a
  ids) are unit tests of `mc/rng.rs` and `mc/keys.rs`.

### 8.3 Passes and parallelism

Passes per run (08 §6.4), all with the same keys, so the same anchors see the same draws:

| Pass | Target | Horizon | Runs when |
|---|---|---|---|
| V (views) | non-depleting; every instance evaluated once per present view (health, armor, shield, overguard), four sinks; on an enemy with overguard two status timelines, `og_present` true (caps `caps_og`, no ragdoll) for the `overguard` view and false for the others, with the same keyed draws (08 §4.5) | shots with s_i ≤ t_win, then DoT continuation for pending damage | always |
| L (layered) | depletion OG → SH → HP, shield gate, overflow (CD-08), Magnetic break, adaptation, health thresholds (`HealthBelow`); no extra hit after a kill; no instance after the kill | until kill or `t_ttk_max` | always (TTK, shots and ammo to kill) |
| L0 (layered, armor ignored) | as L with `arm_t = 1` (the inspiration's Health, Shield and Overguard row TTK, D-802) | as L | only when `Want.parity` and A0 > 0 |

Runs are independent: with feature `parallel`, `(0..n_runs)` is split into chunks (the chunk count follows the pool
policy of AD-15), each run writes its results into a `Vec` slot indexed by run, and the reduction is in run order
(I-5), so results do not depend on the chunking. Cancellation is checked between runs.

### 8.4 Outputs

As for EV, the MC outputs are the `wfd-model::api::calc` DTOs with the accepted shapes of C-27 and C-29 (CONTRACTS.md
T01b): one-run statistics have no sd or CI, the crit histogram is normalised, and +∞ ("not killed") is `null`.

```rust
pub struct McOut { pub views: SmallVec<[(View, ViewMc); 5]>, pub runs: u32, pub seed: u64 }
pub struct ViewMc {
    pub runs: u32, pub seed: u64,
    pub dps_window: Summary,                                // E-8.19, below
    pub series: Vec<BinSummary>,                            // mean, p10, p90 per bin
    pub uptime: Vec<StatusSeries>,
    pub crit_hist: [f64; N_TIER_BINS],                      // fractions of the direct instances, tier k = 0..=7,
                                                            // index 7 = "7 or more" (08 §8.4 "normalised", C-29)
    pub breakdown: Vec<SourceDps>, pub pending_dot: f64,
    pub ttk: Option<TtkDist>,                               // pass L; layered view only
    pub parity: Option<ParityOut>,                          // section 9; present for the four non-layered views
}
pub struct Summary { pub mean: f64, pub sd: Option<f64>, pub ci95: Option<[f64; 2]>, // None for one run (E-8.19, C-27)
                     pub ci99: Option<[f64; 2]> /* E-8.24 uses it */, pub median: f64, pub p5: f64, pub p25: f64,
                     pub p75: f64, pub p95: f64, pub min: f64, pub max: f64 }
pub struct TtkDist { pub kill_probability: f64, pub median: Option<f64> /* None = not killed */,
                     pub mean_killed: Option<f64>, pub p5: Option<f64>, pub p95: Option<f64>,
                     pub shots_to_kill: Option<f64>, pub ammo_to_kill: Option<f64>, /* medians over killed runs (E-8.20) */
                     pub shield_regen_possible: bool,
                     pub breakdown: Vec<SourceDamage> /* up to the kill, Magnetic break included (08 §8.2, §8.6; C-30) */ }
pub struct ParityOut {                                      // E-8.22, per attack and view, medians over runs
    pub queue_dps: f64, pub queue_dps_min: f64, pub queue_dps_max: f64,
    pub avg_direct: f64, pub avg_status: f64, pub avg_total: f64, pub median_total: f64,
    pub ttk: Option<Option<f64>>,                           // absent = not computed; Some(None) = "not killed";
}                                                           // Health, Shield, Overguard rows: pass L0 median; Armor
                                                            // row: pass L median (D-802)
```

`timeline` re-runs one run (pass V) under `TimelineReq { seed, run, limit }`: the seed is the E-8.16 key of the MC
request whose run r the user picked (`TimelineRequest.seed`, else the scenario's `eval.seed`; API-1, C-32), so the
replay reproduces that run. It records per parent hit (08 §8.9): time, damage
on the view, crit tier and factor c_k, effective armor and DR, stance and combo multipliers, extra-hit damage,
statuses applied, the one-tick weapon-side value of each DoT proc applied, live stacks before the hit; reload records
and Heat episode boundaries (ignition, end) interleaved by time.

---

## 9. Outputs (`CalcReport`)

```rust
// wfd-model/src/api/calc.rs
pub struct CalcReport {
    pub data_version: Box<str>, pub code: Box<str> /* share code of the inputs, filled by the server */,
    pub weapon: WeaponPanel,          // per attack: base vs modded stats, per-type damage q_t·Y·(1+F), magazine, Incarnon magazine …
    pub enemy: EnemyPanel,            // scaled pools, armor and DR, innate DR, immunities, caps, unique-DR notes, faction weaknesses
    pub attacks: Vec<AttackReport>,   // per attack: per view { ev: ViewEv, mc: Option<ViewMc> }
    pub groups: Vec<GroupReport>,     // parent + child sums, T-22 combined units, in Engine::units order
    pub conditions: Vec<ConditionView>, // every conditional effect of the build: class, mode, current treatment, override
    pub touched_rows: Vec<Box<str>>,  // register rows (D-xxx) exercised, shown as "assumptions"
    pub trace: Option<Trace>,
    pub warnings: Vec<Message>,       // { key, args, severity }: unmodelled items, UNVERIFIED rules, EV-MC deviation …
    pub timing_us: u64,
}
```

Mapping of spec 00 §4.5:

| Spec output | Field |
|---|---|
| Result panels Armor, Health, Shield, Overguard | `attacks[].views[]` with `view` ∈ {armor, health, shield, overguard}; present when the enemy has the layer |
| Random queue DPS (`queue_dps_parity`) | `ViewMc.parity.queue_dps` (one run, or median/min/max over runs) |
| avg per bullet (direct, status, total), median per bullet | `ParityOut.{avg_direct, avg_status, avg_total, median_total}` |
| TTK | `ViewMc.ttk` (MC layered), `ViewEv.ttk` (EV approximation) |
| More details timeline | `POST /api/timeline` → `Timeline` |
| Weapon panel, enemy panel | `weapon`, `enemy` |
| Added: burst, sustained, EV window, MC distribution, series, breakdowns, stacks, distinct count, uptime, crit tiers, per shot and magazine, shots and ammo to kill, kill probability, pending DoT, group sums, deviation flag | `ViewEv`, `ViewMc`, `groups`, `warnings` |

---

## 10. Compatibility profile

`Profile::Compat(CompatFlags)` switches register rows marked `Compat` to the inspiration's behaviour, for oracle
tests only (spec 00 §4.6). Each branch is a one-line flag test in the stage that owns the row (`// D-0xx`). The
trace records every register row a scenario exercised (`touched_rows`), whether or not its flag is set; the oracle
harness uses it to decide oracle-safety automatically: in the game profile no touched rows; in the compat profile
no touched rows whose Toggle column is `—` (testing.md §5).

---

## 11. Traces, probes, Explain

### 11.1 Traces

With `Want.trace = true` every stage writes `TraceEntry { probe: ProbePath, equation: &'static str /* "E-5" */,
value: f64, inputs: SmallVec<[(ProbePath, f64); 4]>, contributions: SmallVec<[(EffectRef, f64); 8]>,
rows: SmallVec<[&'static str; 2]> }` into a `TraceSink` (`trait TraceSink: Sync { fn enabled(&self) -> bool; fn
record(&self, entry: TraceEntry); fn touch_row(&self, row: &'static str); }`, shared by the parallel MC runs through
`StageCtx.trace`). The default sink `NoTrace` is a zero-sized no-op, so tracing costs nothing when off; `touch_row`
records every register row a stage exercises, flag or not (section 10). The UI renders e.g. "Critical chance 75.0 % =
12 % base × (1 + 150 % Point Strike) + 45 % Arcane Avenger (absolute, E-5)".

### 11.2 Probe paths

```text
probe     := "enemy." name | "attack[" slug "]." name | "attack[" slug "].view[" view "]." name | "group[" n "]." name
name      := ident ( "[" index "]" )? ( "." ident )*
index     := DamageType | StatusType | integer
```

| Probe (examples) | Spec symbol |
|---|---|
| `enemy.pool.HP`, `enemy.pool.SH`, `enemy.pool.OG`, `enemy.armor0`, `enemy.armor_eff`, `enemy.dr`, `enemy.type_mod[COR]` | 05 |
| `attack[normal_attack].base[IMP]`, `.base_total`, `.base_ref`, `.ratio[COR]`, `.ratio_q[COR]`, `.Y`, `.A`, `.C`, `.faction`, `.G`, `.GF` | E-1..E-11 |
| `.cc`, `.cd_code`, `.cd_q`, `.cd`, `.e_k`, `.e_c`, `.p_tier[2]`, `.part`, `.sc`, `.w[SLA]`, `.ms` | E-4..E-10 |
| `.fr`, `.efr`, `.t_mag`, `.t_rel`, `.duty`, `.shots_per_mag` | 06 |
| `.proc_rate[COR]`, `.stack_mean[PUN]`, `.distinct_mean` | spec 00 §4.2 |
| `.view[armor].hit.k0[IMP]`, `.view[armor].hit.k0.total`, `.view[armor].per_pellet`, `.view[armor].dps_burst`, `.dps_sustained`, `.dps_window`, `.dot_tick[SLA].k0`, `.xh.k0`, `.ttk` | E-12..E-14, spec 00 §4.1 |
| state selectors: `@fresh` (default), `@steady`, `@stacks(COR=10,PUN=5)` appended to a view probe | spec 00 §3.8 states A and B |

An unknown probe is a test failure, so a topic document can force the engine to expose an intermediate value by
naming it in a vector; the probe registry (`probe.rs`) lists every supported path with its symbol.

### 11.3 Explain

`Engine::explain(path)` returns `Explanation { value, equation, formula_key: MessageKey, inputs, contributions,
rows }`, rendered by the UI's Explain drawer for any number that has a probe.

---

## 12. Numerics and determinism

- `f64` everywhere (U-4); fixed-point `i64` only for bucket sums; explicit `f32` arithmetic at NP-1 (crit multiplier
  quantization) and NP-2 (enemy scaling blend `f1 + (f2 − f1)·s`), each in one function with unit tests against the
  spec's worked numbers (Q = 2.000488519668579 for CDM0 = 2.0; Grineer health 294.4855 at level 100).
- Quantization `q_t = sign(r)·floor(|r|·32 + 0.5)/32` in `f64` (exact in binary).
- `libm` for `exp`, `ln`, `pow` so EV and MC are bit-identical across Linux, macOS and Windows; `sqrt` is IEEE-exact.
  Rust does not contract `a*b + c` into FMA; operation order is what the code says.
- No hash-map iteration reaches results; ordered reductions; canonical tie-breaks.
- The engine never rounds except where a rule says so (quantization, armor floor, magazine rounding); display
  rounding `floor(x + 0.5)` (U-5) happens in the UI.

---

## 13. Performance

Targets per core of a 2.1 GHz-class x86-64 machine, release profile (`lto = "fat"`, `codegen-units = 1`,
default `panic = "unwind"`, server-api.md §11), one objective attack, one view. They are acceptance criteria of tasks
T15a-c, T16 and T37 (work-breakdown.md); CI fails when a benchmark misses its target by more than 2× (CI machines vary) or regresses by more than 25 %
against `benches/baseline.json`.

| Operation | Target | Use |
|---|---|---|
| L2 + L3 compile and resolve | ≤ 300 ns | everything |
| `Steady` | ≤ 1 µs (≥ 1 M builds/s/core) | burst, sustained, hit metrics |
| `Bound` (8 intervals) | ≤ 1.5 µs | pruning |
| `WindowFast` | ≤ 10 µs | screening of B&B leaves |
| `WindowExact` | ≤ 0.2 µs per shot or tick event | UI numbers, finalists |
| EV TTK | ≤ 2× `WindowExact` | TTK metric |
| `calculate` with EV for all attacks and views of one weapon | ≤ 5 ms | interactive recalculation |
| MC | ≥ 5 M events/s/core | UI distributions, confirmation |
| MC, interactive (`/api/calc`) | 1,000 runs of a 20 s Braton Prime window, passes V and L (and L0 with parity) ≤ 300 ms on 2 threads, the interactive pool's share while an optimizer job runs (AD-15); ≤ 100 ms on 8 threads when no job runs | the frontend's 700 ms MC budget (frontend.md §12) |

Benchmarks: `crates/wfd-engine/benches/{resolve,steady,window,bound,mc}.rs` (criterion 0.8) on the spec 00 §3.8
scenario and fixed fixtures. `ResolvedAttack`, `PointState` and `Workspace` are fixed-size; no heap allocation in L3
and L4 after warm-up.

---

## 14. Spec test vectors

The spec stays the only source of expected numbers. Vectors become tests in three steps:

1. **Machine twin.** Each `TV-nn-mm` of `docs/mechanics/nn-*.md` is transcribed once into
   `crates/wfd-engine/tests/spec_vectors/nn.toml`. Topic documents may instead embed fenced ```` ```testvector ````
   blocks in the same TOML form; `cargo xtask spec-vectors` extracts those into the twin file automatically.
2. **Transcription check** (`tests/spec_conformance.rs`): every `TV-nn-mm` id in the Markdown has a twin entry and
   vice versa, and every expected number in the twin occurs verbatim in the Markdown text of that vector. A
   transcription typo fails CI.
3. **Execution** (`tests/spec_vectors.rs`): stage vectors call a stage function by name through the stage registry;
   scenario vectors build the inputs over `data/fixtures/mini/`, run the engine and read probes. Tolerances are the
   document's (02 §9: f32 values bit-exact, f64 within 1e-9 relative; 01 §8: 1e-9 relative).

```toml
# crates/wfd-engine/tests/spec_vectors/02.toml
[vector."TV-02-01"]
source = "02 §9"
kind = "stage"
stage = "crit_multiplier"
tolerance = { rel = 1e-9, f32_exact = true }

[[vector."TV-02-01".case]]
input = { cdm0 = 2.0, cd_rel = 1.2, cd_abs = 0.0, m_cdf = 1.0 }
expect = { code = 256, q = 2.000488519668579, cdm = 4.401074743270875 }

[vector."TV-00-01"]
source = "00 §3.8"
kind = "scenario"
scenario = "scenarios/braton_prime_lancer_l100_sp.json"     # CalcInput over data/fixtures/mini
tolerance = { rel = 1e-6 }

[vector."TV-00-01".expect]
"enemy.pool.HP" = 73621.375
"enemy.armor0" = 1355
"attack[normal_attack].ratio_q[COR]" = 1.8125
"attack[normal_attack].cd" = 4.4010749
"attack[normal_attack].view[armor].hit.k0.total" = 195.387510
"attack[normal_attack].view[armor].per_pellet" = 394.7458
"attack[normal_attack].view[armor].dps_burst" = 11496.26
```

Mapping of the documents. Counts are of the spec at this review (2026-11-27: 323 ids in 01-09, plus `TV-00-01`,
engine.md's name for the 00 §3.8 worked example, which carries no TV id in 00); the transcription check derives the
id list from the documents, never from a fixed number.

| Document | Vectors | Test kind | Executed by (work-breakdown.md acceptance) |
|---|---|---|---|
| 00 §3.8 worked example | TV-00-01 (steps 1-10, states A and B) | scenario | T07 (step 1, 5), T08a (step 2), T13 (steps 2-4), T14 (steps 6-7), T15a (steps 5, 8-9), T15b (step 10), T17 (snapshot) |
| 01 §8 | TV-01-01..TV-01-39 (composition, quantization, element order, faction, weakness map, DoT hand-off) | stage (`base_vector`, `combine_elements`, `ratios`, `quantize`, `faction`) and scenario | T13 (all), T06 (element vectors) |
| 02 §9 | TV-02-01..TV-02-26 (crit quantization, tiers, Vigilante, Doughty, body part, status, procs, multishot) | stage (`crit_multiplier`, `crit_tiers`, `crit_chance`, `doughty`, `part_factor`, `status_chance`, `proc_weights`, `multishot`) | T13 |
| 03 §19 | TV-03-01..TV-03-40 (durations, caps, DoT ticks, Heat, Blast, Gas, Magnetic break, distinct count) | stage (`dot_tick`, `heat_strip`, …) and scenario | T14 |
| 04 §15 | TV-04-01..TV-04-49 (CO behaviours, G/G_F, condition state machine, extra hits, rivens, sets, custom stats) | stage and scenario | T14 (extra-hit and mechanic vectors), T06 (lowering, rivens, sets, custom stats), T13 (CO behaviours, G/G_F) |
| 05 §24 | TV-05-01..TV-05-43 (scaling f32, armor, layers, DR, routing) | stage (`np2_blend`, `effective_armor`, `armor_dr`, `instance_on_layer`, `route_layered`) | T07 |
| 06 §16 | TV-06-01..TV-06-45 (schedules, ramp, spool, sniper combo, duty) | stage (`schedule`) | T08a |
| 07 §14 | TV-07-01..TV-07-31 (attack speed and loop timing, combo ramp, heavy damage and loops, combo duration, zaw assembly, stance strikes, gunblades, table resolution, gain rounding, stance normalisation, build and spend, Tennokai, fallbacks, waves, slams, glaive cycle, Blood Rush before consumption, expected gains, tiers, expiry during regeneration, steady states, stance CO ratio) | stage (`melee_timing`, `combo_gain`, `heavy_hits`, `zaw_assemble`) and scenario (`melee_schedule` with a `MeleePlan`) | T08b: -01..-06, -08, -09, -11, -13..-17, -19..-21 (-21: the table used), -23..-28 (-28: period and keys), -31; T13: -07, -22 (with T08b's tables and waves), -29, -30; T14: -18 (with T08b's heavy); T15a: -28's dps_burst; T18 (`wfd-datagen`, `crates/wfd-datagen/tests/spec_vectors_07.rs`): -10, -12 and -21's heavy hit lists, because the pipeline computes them |
| 08 §17 | TV-08-01..TV-08-32 (Philox, keyed draws and FNV-1a ids, crit-only EV vs MC, capped Poisson, window walk, Heat pool and strip counter, correlated CO count, DoT survival, steady metrics, window edge, TTK, parity buckets, statistics, oracle comparison, arrival-time steady mode, tick at expiry, two-time CO terms, realistic window, Corrosive and Heat, TTK extrapolation, status draws and Encumber keying, survival in one event, coupled crit factor, shared_lose_all, partial series bin, Melee Duplicate EV, live-count distribution, ragdoll mixing, n_COL,j, DUPLEX steady arrival) | stage (`philox`, `rng_mech`, `draw`, `capped_poisson`, `heat_strip_counter`, `summary`, `parity_buckets`, `oracle_compare`) and scenario | T15a: -04, -08..-11, -17, -19, -24..-26, -28..-32; T15b: -03 (EV: `dps_window` by the window walk), -05..-07, -12, -18 (EV: landed and pending ticks of the window), -20, -21, -27; T15c: -13 (EV), -22; T16: -01..-03, -13 (MC), -14..-16, -18 (MC), -23; T17: -03, -05, -06 (EV vs MC, E-8.24) |
| 09 §19 | TV-09-01..TV-09-18 (drain, polarity, capacity, Forma, rivens, valence, set counting) | executed by `wfd-data` (`crates/wfd-data/tests/spec_vectors_09.rs`, legality functions; `wfd_model::sets::set_counts` for TV-09-09, `RankCurve::value` for TV-09-06) | T04; twins in `crates/wfd-engine/tests/spec_vectors/09.toml` so the transcription check covers them |

**Stage names and the registry** (`stage_registry.rs`, T09; contract review ENG-17). A stage vector names a registry
entry, not a Rust path. Entries whose function exists in the frozen skeleton (`base_vector`, `combine_elements`,
`ratios`, `quantize`, `crit_multiplier`, `crit_tiers`, `crit_chance`, `doughty`, `part_factor`, `status_chance`,
`proc_weights`, `multishot`, `np2_blend`, `effective_armor`, `armor_dr`, `instance_on_layer`, `route_layered`,
`dot_tick`, `philox`, `rng_mech`, `draw`, `capped_poisson`, `summary`, `oracle_compare`) are T09's adapters. The other
names are registered by the task that writes the function behind them, through a CONTRACTS.md entry that adds the
adapter and its registry row (the C-21 rule): `faction` and `zaw_assemble` T13, `melee_timing`, `combo_gain` and
`heavy_hits` T08b, `heat_strip` T14 (03 E-3.8: the strip as a function of the step counter k_h, TV-03-07, -19,
-20), `heat_strip_counter` T15a (08 E-8.7, steady, TV-08-08) and T15b (08 E-8.6, window, TV-08-07, -21),
`parity_buckets` T16. The two Heat names are distinct stages (contract review ENG-17, pass 3). `heat_strip_counter` has
one registry entry: T15a adds it with the steady adapter (E-8.7), and T15b extends that adapter for the window inputs
(E-8.6) through a CONTRACTS.md entry instead of adding a second entry (`registry_names_are_unique`; contract review
P3-5). Until then T09 marks
the vectors `pending = "Txx"` with that task. This table and the acceptance lines of work-breakdown.md T07-T18 make
one assignment; T09's `pending` values follow it.

Compatibility-profile values stated by a document (01 §8 "Compatibility-profile values") are twin entries with
`profile = "compat"`. Mechanics without a numeric vector get ordinary `#[test]`s named after the rule
(`e11_adding_omits_gf`, `cd04_sibling_pellets_no_viral`).

---

## 15. Errors and warnings

```rust
#[non_exhaustive]                              // ruling C-31: later engine tasks add variants (CONTRACTS.md entry)
pub enum EngineError {                         // never a panic; serialized as { key, args }
    InvalidBuild(Vec<Message>), MissingData { what: Box<str> }, UnsupportedMechanic { effect: EffectKey },
    ScheduleTooLong { shots: u64 },            // horizon × rate over 10^7 events (schedule::event_bound)
    MechanicIdCollision { a: Box<str>, b: Box<str> },   // two mechanic id strings with one FNV-1a hash (8.2)
    RngIndexRange { what: Box<str>, value: u64 },       // anchor index outside the E-8.16 ranges (8.2)
    Cancelled,                                 // a superseded calculation (CancelToken; server-api.md §11)
}
impl EngineError { pub fn message(&self) -> Message; pub fn messages(&self) -> Vec<Message>; }  // { key, args }; the
                                               // 422 issues of InvalidBuild follow the head message
pub struct Cancelled;                          // CancelToken::check; From<Cancelled> for EngineError
```

Where errors arise (contract review ENG-16): `schedule::event_bound(ra, plan, tables, horizon)` returns
`ScheduleTooLong` when horizon × rate exceeds `MAX_SCHEDULE_EVENTS` (10^7), an infinite rate included (`FR0 = ∞` with
no charge or burst spacing, 06 E-6.11), so no schedule generator loops or truncates silently; `Engine::try_schedule`
checks it first.
`Engine::calculate`, `compile_scenario`, `compile_build`, `mc`, `timeline` and `try_schedule` return `Result`. The EV
entry points (`Engine::ev`, `Kernel::score`, `Kernel::bound`) return plain values: they record an error with
`Workspace::fail` (the first is kept; the value is then meaningless, a score `Score::WORST`) and the caller reads it
with `Workspace::check` after the call.

Warnings (`Message` with `MessageKey`) include: `item.unmodelled`, `item.stale`, `item.draft`,
`rule.unverified { row }`, `ev.approx.gate_split_limit`, `ev.steady.no_convergence { attack, view }`,
`ev.mc_deviation { attack, view, ev, ci, gap_classes }`, `ttk.extrapolated`, `ttk.shield_regen_possible`,
`effect.no_spec_vector { effect }` (a `draft` effect whose DSL construct has no spec vector yet, overview R-2). The EV
evaluator's warnings (`ev.approx.gate_split_limit`, `ev.steady.no_convergence`, `ttk.extrapolated`) go to
`Workspace::warn`; T17 drains them with `Workspace::take_warnings` into `CalcReport.warnings`.
