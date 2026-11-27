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
pub struct Engine<'b> { bundle: &'b Bundle }            // cheap to create, Sync

impl<'b> Engine<'b> {
    pub fn new(bundle: &'b Bundle) -> Self;

    /// The server's and the CLI's entry point: everything for the calculator page.
    pub fn calculate(&self, build: &Build, scenario: &Scenario, want: &Want, cancel: &CancelToken)
        -> Result<CalcReport, EngineError>;

    // ---- levels (section 3); the optimizer and the tests use them directly ----
    /// L0: per scenario (enemy scaling, faction relations, caps, T-02 mode, slot allocation, buffs, custom stats,
    /// T-41 hit fractions, T-43 external statuses, melee tables and plans).
    pub fn compile_scenario(&self, build: &Build, scenario: &Scenario, objective: Option<&ObjectiveSpec>)
        -> Result<CompiledScenario, EngineError>;
    /// L1: one candidate (mod, arcane, riven, perk, shard, stance, set bonus) in a scenario.
    pub fn compile_pick(&self, cs: &CompiledScenario, pick: &Pick) -> CompiledItem;
    /// L2: a whole build: Σ contributions + finalizers (sets, shards, resolved scalars, element order).
    pub fn compile_build(&self, cs: &CompiledScenario, build: &Build) -> Result<CompiledBuild, EngineError>;
    /// L3: one attack (ST-1..ST-7) plus firing stats.
    pub fn resolve(&self, cs: &CompiledScenario, cb: &CompiledBuild, attack: AttackIdx) -> ResolvedAttack;
    /// Deterministic shot/hit schedule (06 E-6.11; melee 07 E-7.30 with the scenario's MeleePlan), identical for EV and MC.
    pub fn schedule(&self, cs: &CompiledScenario, ra: &ResolvedAttack, horizon: f64) -> Schedule;
    /// L4: expected values (section 7); `req.build` is the compiled build, the evaluator resolves and schedules the unit.
    pub fn ev(&self, cs: &CompiledScenario, unit: &EvalUnit, req: &EvRequest, ws: &mut Workspace) -> EvOut;
    /// L4: Monte Carlo (section 8). Runs in parallel with feature `parallel`; results independent of thread count.
    pub fn mc(&self, cs: &CompiledScenario, unit: &EvalUnit, req: &McRequest, cancel: &CancelToken)
        -> Result<McOut, Cancelled>;
    /// Per-hit event log of one MC run (the inspiration's "More details").
    pub fn timeline(&self, cs: &CompiledScenario, cb: &CompiledBuild, unit: &EvalUnit, view: View, run: u32, limit: u32)
        -> Timeline;
    /// Read one spec symbol (section 11.2); used by spec vectors and the Explain drawer.
    pub fn probe(&self, build: &Build, scenario: &Scenario, path: &ProbePath) -> Result<ProbeValue, ProbeError>;
    pub fn explain(&self, build: &Build, scenario: &Scenario, path: &ProbePath) -> Result<Explanation, ProbeError>;

    /// Optimizer-facing scoring kernel (section 7.9); scores are ordered keys (`Score`), not plain f64.
    pub fn kernel<'a>(&'a self, cs: &'a CompiledScenario, objective: &'a ObjectiveSpec) -> Kernel<'a>;
}

/// What a calculation should produce (the server maps CalcRequest.want onto it).
pub struct Want { pub ev: bool /* always true in practice */, pub mc: Option<McWant>,
                  pub parity: bool, pub breakdown: bool, pub trace: bool, pub views: Option<SmallVec<[View; 5]>> }
pub struct McWant { pub runs: u32, pub seed: u64, pub series_bin_s: f64 }   // T-29 resolved by the server
/// L4 requests carry the compiled build (L2) of the evaluated build.
pub struct EvRequest<'r> { pub build: &'r CompiledBuild, pub fidelity: Fidelity, pub views: &'r [View],
                           pub series_bin_s: f64, pub breakdown: bool }
pub struct McRequest<'r> { pub build: &'r CompiledBuild, pub runs: u32, pub seed: u64, pub views: &'r [View],
                           pub series_bin_s: f64, pub parity: bool, pub breakdown: bool }
/// An attack or an attack group (T-22 combined or parent+child summed) evaluated together.
pub struct EvalUnit { pub attacks: SmallVec<[AttackIdx; 3]>, pub combined: bool }
pub enum Pick { Mod { item: ItemId, rank: u8, params: SmallVec<[(ParamKey, ParamValue); 1]> }, Riven(Riven),
                Arcane { item: ItemId, rank: u8 }, Perk { tier: u8, index: u8 }, Shard { item: ItemId, tauforged: bool },
                Stance { item: ItemId, combo: ComboKey }, FrameMod { item: ItemId, rank: u8 }, Aura { item: ItemId, rank: u8 },
                KitgunParts { grip: u16, loader: u16 }, ZawParts { grip: u16, link: u16 }, Progenitor { element: DamageType } }
pub struct CancelToken(Arc<AtomicBool>);                // new, cancel, is_cancelled, check -> Result<(), Cancelled>
/// Context of every pure stage function (resolve/, enemy/, status/, schedule/).
pub struct StageCtx<'a> { pub rules: &'a Rules, pub compat: CompatFlags, pub trace: &'a dyn TraceSink }
```

Report types (`CalcReport`, `EvOut`, `McOut`, `Timeline`, `Explanation`, `ProbeValue`) are API DTOs in
`wfd-model::api::calc` so the frontend's types are generated from one crate (server-api.md §5); `wfd-engine`
re-exports them with `ViewEv`, `ViewMc`, `TtkEv`, `TtkDist`, `ParityOut` and `Summary`.

The L4 calls take the compiled build: `EvRequest.build`, `McRequest.build` and the `cb` argument of `timeline` were
added by T02a (CONTRACTS.md T02a), because an evaluation unit names attacks, not a build. Every public signature of
this section and of the stage functions of work-breakdown.md T02a exists in the crate; bodies are `todo!("Txx")` until
the owning task of work-breakdown.md §8 implements them.

---

## 2. Module layout

```text
crates/wfd-engine/src/
  lib.rs  error.rs  cancel.rs   (Engine, StageCtx, Want, McWant, EvalUnit, Pick; EngineError; CancelToken)
  spec_constants.rs     formula constants of the spec only, each with its E-number (I-1)
  stats.rs              StatVec (fixed point), StatVecF, StatMask, SparseContrib, Coeff, ElemEntry, MechanicInst,
                        StaticTerm, EffectRef, MechSet, SourceIndex
  state.rs              StateVar, state-variable table, distributions
  compile/              scenario.rs (L0), pick.rs (L1, `CompiledItem`, T-02 table), scope.rs, build.rs (L2), sets.rs, shards.rs,
                        riven.rs, buffs.rs, custom.rs, passives.rs, conditions.rs (class derivation, overrides),
                        monotone.rs (per-scenario slot classes from readers, domain-model.md §10), melee.rs (E-7.0b
                        table resolution, `plan_selection` and MeleePlanSel → MeleePlan)
  assemble.rs           kitgun, zaw, progenitor (E-1 V_t), exalted Strength scaling
  elements.rs           element combination (01 §4.11), ElementSolver (all distinct outcomes, for the optimizer)
  resolve/              base.rs (E-1, 01 §4.2-4.7), composition.rs (E-2, 01 §4.9-4.12), quantize.rs (E-3),
                        scalar.rs (E-11, 04 §2), weapon_mult.rs (ST-5: F, G, G_F), part.rs (E-7), crit.rs (E-4..E-6),
                        status_chance.rs (E-8, E-9), multishot.rs (E-10), firing.rs (06 §5), melee.rs (07 §5, §7:
                        attack speed, wind-up, combo stats, melee instance base E-7.20/E-7.20b, heavy synthesis)
  schedule/             ranged.rs (06 E-6.11), melee.rs (07 E-7.30, plans of 07 §8.1, steady E-7.37), segment.rs
  enemy/                scaling.rs (NP-2), armor.rs (E-15), layers.rs (E-12 enemy side, ST-9, CD-08), dr.rs (innate,
                        attenuation, special DR, T-28), parts.rs (body parts, head-crit)
  status/               rules.rs (03 §4.1 rules from the bundle), dot.rs (E-13), heat.rs (03 §10), blast.rs (03 §11),
                        magnetic_break.rs (03 §13), distinct.rs (03 §14), extra_hit.rs (E-14)
  mechanics/            mod.rs (traits of 2.1, `MechInst`, `mechanic_impl`), one module per mechanic kind
                        (domain-model.md §11.2: extra_hit, proc_on_event, convert_damage, repeat_hit,
                        flat_damage_on_proc, proc_after_ticks, counter_bonus, random_multiplier, spawn_attack), each a
                        unit type implementing both traits
  ev/                   kernel.rs (E-8.9, one instance at one state), poisson.rs (E-8.3), joint.rs (crit, ARM and
                        attenuation enumeration, E-8.9 steps 3-6), gates.rs (gate splitting), steady.rs (E-8.13,
                        E-8.14), window_exact.rs (E-8.12), heat.rs (E-8.5-E-8.8), dot.rs (E-8.11), distinct.rs
                        (E-8.10), cond_stacks.rs (04 E-4.9, 08 E-8.15), window_fast.rs, bound.rs, ttk.rs (E-8.21,
                        E-8.26), hit.rs (08 §10), group.rs (E-8.23), series.rs (08 §8.1-§8.4 curves), score.rs
                        (Score keys), opt_kernel.rs (Kernel, ObjectiveSpec, Fidelity, BuildPoint, Workspace, section 7.9)
  mc/                   rng.rs (Philox4x32-10, McRng, E-8.16), keys.rs (mechanic id registry, FNV-1a, E-8.16), draw.rs
                        (E-8.17), events.rs (McEvent, McEventKind, Phase, Anchor, McSink), state.rs (McState, 08 §6.2),
                        sim.rs (E-8.18), passes.rs (V, L, L0), summary.rs (E-8.19, oracle_compare of E-8.25), ttk.rs
                        (E-8.20), timeline.rs (08 §8.9), parity.rs (E-8.22)
  outputs.rs            CalcReport assembly, spec 00 §4.5 mapping, SourceKind (breakdown sources)
  compat.rs             CompatFlags helpers
  trace.rs              TraceSink (no-op by default), touched register rows
  probe.rs              ProbePath parser, probe registry
  stage_registry.rs     stage adapters of the spec-vector harness (T09)
```

Each `resolve/`, `enemy/` and `status/` function is pure, takes a `&StageCtx` (rules, compat flags, trace sink), and
cites its equation in its doc comment, so a reviewer can put the spec and the code side by side.

### 2.1 Mechanic interfaces (frozen by T02a)

Every mechanic kind of domain-model.md §11.2 implements two traits; `mechanics/` (T14) writes the implementations, the
evaluators (T15a-c, T16) only call them, so the three tasks code against signatures that exist before any of them
starts:

```rust
// crates/wfd-engine/src/mechanics/mod.rs
pub trait MechanicEv {
    /// Expected contribution of the mechanic to one instance at one state (E-8.9 terms, proc masses, extra damage).
    fn ev_instance(&self, inst: &MechInst, ra: &ResolvedAttack, st: &PointState, view: View, acc: &mut EvAcc);
    /// Expected proc events the mechanic adds for one proc event e (feeds h_{s,e}, m_{s,e}; E-8.2).
    fn ev_procs(&self, inst: &MechInst, ra: &ResolvedAttack, st: &PointState, out: &mut ProcMass);
}
pub trait MechanicMc {
    /// Live effect on one instance: draws through `rng` with the mechanic's id, may push derived events.
    fn mc_instance(&self, inst: &MechInst, ra: &ResolvedAttack, st: &mut McState, ev: &McEvent, rng: &mut McRng,
                   out: &mut McSink);
    fn mechanic_ids(&self, inst: &MechInst) -> SmallVec<[MechanicId; 2]>;   // registered at L0 (section 8.2)
}
pub trait MechanicImpl: MechanicEv + MechanicMc + Sync {}
pub fn mechanic_impl(kind: &Mechanic) -> &'static dyn MechanicImpl;     // one static unit type per kind
pub struct MechInst { pub mechanic: Mechanic, pub magnitude: f64, pub source: EffectRef, pub key: EffectKey }
pub struct PointState { /* μ_s, N_c,s, Z_s, N_ext,s, P_H and P_kh, combo c̄, sniper combo, condition-stack and counter
                          expectations, schedule marks and pellet index of the instance; fixed size */ }
pub struct EvAcc { /* direct per type and per source; DoT registrations; extra hits; procs (one view at a time) */ }
pub struct McState { /* 08 §6.2 run state of one pass */ }
pub struct McEvent { pub t: f64, pub t_us: i64 /* τ̂, E-8.0a */, pub phase: Phase, pub seq: u64, pub kind: McEventKind,
                     pub anchor: Anchor /* (a, i, j) */ }
pub struct McRng<'r> { /* Philox key, run, current anchor, per-mechanic ordinal counters (&'r mut OrdinalTable) */ }
```

The state types belong to the evaluators that fill them and are re-exported by `mechanics`: `PointState`, `EvAcc`,
`ProcMass` live in `ev/kernel.rs` (T15a); `McState`, `McEvent`, `McEventKind`, `Phase`, `Anchor`, `McSink`,
`McRng` and `MechanicId` in `mc/` (T16). `Phase` orders P1 transitions < P2 damage < P3 Heat end < P4a instances <
P4b procs < P4c zero-delay ticks < P4d detonation (03 §7.1, 08 E-8.18); the MC heap orders events by `(t_us, phase,
seq)`.


---

## 3. Compile levels

| Level | Input | Output | When | Typical cost |
|---|---|---|---|---|
| L0 | build skeleton (weapon, variant), scenario | `CompiledScenario` | once per scenario | microseconds |
| L1 | one candidate pick | `CompiledItem` (domain-model.md §18) | once per (pick, scenario) | tens of µs for ~1,000 picks |
| L2 | a build | `CompiledBuild`: `StatVec` per objective attack, element entries ordered, coefficients, mechanics | per candidate; incremental in search | 50-250 ns |
| L3 | (build, attack) | `ResolvedAttack` | per candidate per attack | 100-200 ns |
| L4 | resolved attack(s) + schedule | EV or MC outputs | per candidate | 0.14 µs per target-state point |

```rust
pub struct CompiledScenario {
    pub weapon: AssembledWeapon,          // kitgun/zaw parts applied, progenitor V_t, exalted Strength factor
    pub enemy: ScaledEnemy,               // pools per layer, A0, φ_t, caps, immunities, parts, DR, attenuation (05)
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
    pub objective: Option<ObjectiveSpec>, // attacks and weights for the optimizer
    pub window: f64, pub horizon: f64,
    pub hit_fraction: SmallVec<[f64; 8]>, // T-41 f_a per AttackIdx (1 without a `pellet_hit` entry)
    pub external: [u8; N_ST],             // T-43 N_ext,s with T-06 folded in, 0 on immune types (03 §6.3)
}
pub struct StatVec(pub [i64; N_STATS]);    // 1e-9 units; to_f64() once at L3
pub struct CompiledBuild { pub sv: SmallVec<[StatVec; 2]>, pub elements: ElemOutcome, pub coeffs: SmallVec<[Coeff; 4]>,
                           pub mechanics: MechSet, pub sources: SourceIndex /* for traces */ }
```

**Finalizers (L2).** Static scalars are resolved once all picks are known: set-piece counts (09 counting, active
weapon rule), shard counts (Violet scaling), Strength, and `Per::Resolved` terms (Melee Doughty reads the resolved
Puncture status chance; the stage order guarantees the referenced stat is final, and the compiler rejects cycles: an
effect cannot change a stat its own condition or scalar reads). Locks (`lock_multishot`, `lock_fire_rate`) are applied
at L3.

`CompiledItem` (L1, domain-model.md §18) lives in `compile/pick.rs` (domain-model.md §18 names `compile/item.rs`;
the module layout above is the reference) and carries, in addition to §18's fields, `readers: SmallVec<[Reader; 1]>`:
the crit-tier, proc, schedule-mark and resolved-stat readers the item holds, from which
`monotone_classes(cs, pool) -> ([Monotone; N_STATS], SmallVec<[Reader; 4]>)` derives the slot downgrades.

**Element outcome.** `elements::combine(entries: &[OrderedElem]) -> ElemOutcome` (`OrderedElem = (OrderKey,
DamageType, f64)`; the stage registry names it `combine_elements`) implements 01 §4.11 (slot order, exilus after slot
7, riven bottom-up, innate group HCET, non-combining additions).
`ElementSolver::outcomes(fixed: &[OrderedElem], set: &[ElemCandidate], slots: &[OrderKey]) -> SmallVec<[(ElemOutcome,
Vec<SlotAssignment>); 3]>` returns every distinct outcome an unordered set of element entries can reach in the free
slot positions, with one slot assignment realizing each (optimizer.md §5).

---

## 4. Resolve (ST-1..ST-7)

`ResolvedAttack` is a fixed-size `Copy` struct (no heap):

```rust
pub struct ResolvedAttack {
    pub attack: AttackIdx, pub roles: AttackRoles,
    pub base: DamageVec, pub base_total: f64, pub base_ref: f64,      // B'_t, B', B_ref (E-1)
    pub ratio: DamageVec, pub ratio_q: DamageVec,                      // r_t, q_t (E-2, E-3)
    pub a: f64, pub co: CoResolved,                                    // A; C as affine terms; behaviour, ρ_co, B_co (E-11)
    pub g_f: f64, pub faction: f64, pub g: f64,                        // Π G_F, F, Π G (static parts; schedule-driven factors per shot)
    pub part: PartResolved,                                            // P_part, ΣWP, ΣHS, M_son per hit location (E-7)
    pub crit: CritResolved,                                            // CC parts, Q (NP-1), CDM parts, p_vig (E-4..E-6)
    pub status: StatusResolved,                                        // SC (E-8), w_s (E-9), φ_s, A_sd(+s), SD, caps, DoT coeffs
    pub ms: f64, pub ms_locked: bool,                                  // E-10
    pub firing: FiringResolved,                                        // FR, EFR, CT, burst, magazine, cost, AE, reload, duty, ramp, spool
    pub melee: Option<MeleeResolved>,                                  // AS, W, D_cb, IC, η, combo gains, heavy H_j (07 §5-§7)
    pub xh: [ExtraHitResolved; 8], pub mech: MechResolved,             // E-14 and mechanic parameters
    pub gates: GateSet,                                                // dynamic gates that feed nonlinear stages (7.4)
    pub mask: StatusSet,                                               // statuses this attack (or externals) can apply
    pub hit_fraction: f64,                                             // T-41 f_a (06 E-6.24, 08 E-8.2, E-8.17)
}
```

| Stage | Function | Spec |
|---|---|---|
| ST-1 base vector | `resolve::base::base_vector` | E-1, 01 §4.2-4.7 (progenitor, Incarnon flat split D-005, Strength) |
| ST-2 composition | `resolve::composition::ratios` | E-2, 01 §4.9-4.12 (physical, stance physical, Phoenix Talons, elements) |
| ST-3 quantization | `resolve::quantize::quantize` | E-3, CD-02, D-001 compat |
| ST-4 base scalar | `resolve::scalar::base_scalar` | E-11, 04 §2 (behaviours, ρ_co, B_ref, G_F) |
| ST-5 weapon multipliers | `resolve::weapon_mult` | 01 §4.15 (F), 04 §3 (G) |
| ST-6 body part | `resolve::part` | E-7, 02 §4.5 |
| ST-7 crit | `resolve::crit` | E-4 (NP-1 f32), E-5, E-6, 02 §4.2-4.4 |
| status chance and procs | `resolve::status_chance` | E-8, E-9, 02 §4.6-4.7 |
| multishot | `resolve::multishot` | E-10, 02 §4.1 |
| firing stats | `resolve::firing` | 06 §5, §10 |

Terms that depend on target state (Puncture crit chance, Cold crit damage, CO count, Viral, Magnetic, armor) are not
folded into `ResolvedAttack`; they are applied by the kernel (EV) or the simulator (MC) from the state.

---

## 5. Schedules

`schedule::ranged::ranged_schedule(ctx, ra, horizon)` implements 06 E-6.11 exactly: shots with cycle start, fire time, hit time, magazine
number and index, ammo before and after, cost, `first`/`last` flags, burst position, spool index and rate, Kohm
pellets, Peacemaker burst, ramp level; reloads (start, end, `from_empty`, reloaded fraction `(Mag − left)/Mag` for
`Per::ReloadedFraction`); sniper decay times; child hits per shot including delayed explosions and lingering ticks
(06 §9.3-9.4, E-6.25). `schedule::melee::melee_schedule(ctx, ra, plan, tables, horizon)` implements 07 E-7.30 for
one `MeleePlan` (07 §8.1:
`light_loop(C)`, `slide_loop`, `heavy_loop`, `build_spend(K)`, `tennokai`, `slam_loop`, `heavy_slam_loop`,
`glaive_cycle`): strikes from the resolved combo tables (E-7.0b), timing from AS, W and T_sw (07 §5), the expected-path
combo count c̄ with gains, expiry, Power Spike decay, heavy spend and Initial Combo regeneration (07 §6), heavy hit
lists H_j (E-7.40), 9-C spawned attacks, Melee Duplicate and wave insertion order (07 §8.4), the hit, heavy and combo
records of 07 §8.2, and the steady quantities of E-7.37. The schedule never depends on random draws.

```rust
pub struct Schedule { pub shots: Vec<ShotRec>, pub reloads: Vec<ReloadRec>, pub segments: SmallVec<[Segment; 16]>,
                      pub children: Vec<ChildHits /* per shot and child attack: count, headshot gate, hit times */>,
                      pub sniper_decay: Vec<f64 /* 06 E-6.16 decay times */>,
                      pub melee: Option<MeleeTrack /* heavy and combo records of 07 §8.2 */>,
                      pub cycle: CycleStats /* EFR, shots per magazine, duty ρ, T_mag, T_rel; melee hits/s, T_loop */ }
pub struct MeleePlan { pub kind: PlanKind, pub combo: ComboType, pub k: f64, pub policy: HeavyPolicy,
                       pub heavy_cancel: Option<f64> /* t_dmg, OQ-701 */, pub t_slam: f64, pub throw: ThrowKind,
                       pub t_cyc: Option<f64> /* cycle_loop, data_attack_loop */, pub start_count: f64 /* E-7.31 */ }
pub struct CompiledMelee { pub tables: MeleeTablesOwned /* E-7.0b result */, pub plans: SmallVec<[(AttackIdx, MeleePlan); 4]> }

pub fn plan_selection(toggles: &Toggles, key: &AttackKey, atk: &Attack) -> MeleePlanSel;            // T-30
pub fn melee_plan(sel: &MeleePlanSel, atk: &Attack, toggles: &Toggles, combo: ComboType, md: &MeleeData) -> MeleePlan;
```

**MeleePlan from the T-30 toggles.** L0 builds one `MeleePlan` per melee attack the scenario evaluates
(`CompiledMelee.plans`) in two steps (domain-model.md §9.3; 07 §8.1, §8.5). `plan_selection` picks the selection: the
attack's entry in `Toggles.melee_plans` (at most one per attack; an unknown attack key or a duplicate entry was already
rejected when the document was resolved, `doc.attack_unknown` / `doc.attack_duplicate`), else `Toggles.melee_plan`
when its plan kind names the attack (`light_loop`, `build_spend`, `tennokai` the Normal Attack, `heavy_loop` the heavy
attack, `slide_loop` the slide attack, `slam_loop` and `heavy_slam_loop` the slams, `glaive_cycle` the throws,
`data_attack_loop` a directly selected data attack), else `Auto`. `melee_plan` maps `Auto` by attack kind (Normal
Attack → `light_loop` of `BuildDoc.stance_combo`, heavy → `heavy_loop`, slide → `slide_loop`, slam → `slam_loop`, heavy
slam → `heavy_slam_loop`, throws → `glaive_cycle`, no light combo → `cycle_loop`, a directly selected data attack →
`data_attack_loop`), copies the plan parameters, and applies the T-09 rule of E-7.31 (`Toggles.heavy`: attacks without
a data role use `heavy_loop`, start count `max(IC, k_tier·(M_start − 1))`). T-10 is read by the schedule.

**Inputs of T-41, T-42 and T-43.** They are scenario toggles that the schedule and the evaluators read, never items:

| Toggle | Compiled into | Read by |
|---|---|---|
| T-41 pellet hit fraction (`Toggles.pellet_hit`, 06 E-6.24, 08 E-8.2, E-8.17) | `CompiledScenario.hit_fraction[a]` (1 without an entry) → `ResolvedAttack.hit_fraction` | the schedule keeps unthinned instance counts (E-6.24: children are not thinned by the parent's fraction; headshot-only children only for parent instances that hit); EV multiplies every instance's existence probability by f_a; MC draws `"hit_frac"` first for every instance of an attack with f_a < 1 (E-8.17) and a missed instance has no damage, procs, extra hits, duplicate or place in the CO order (EC-08-63); the sniper-combo count of E-6.16 uses the expected hit count |
| T-42 gauge uses per minute (`Toggles.gauge_uses_per_min`, 06 §6.7) | `FiringResolved.gauge_uses_per_min`, set at L3 for gauge-gated attacks (`FiringResolved.gauge_gated`) | `ranged_schedule`: E-6.14a uses at s_0 and s_m = s_0 + m·60/u without magazine cost; no steady metrics (`ViewEv.dps_burst` etc. absent, 08 E-8.14) |
| T-43 external statuses (`Toggles.external_statuses`, `Toggles::external_stacks` folds T-06 in; 03 §6.3) | `CompiledScenario.external[s]` (0 on immune types), combined by max with the `external.*` stats into `StatusResolved.n_ext` | EV `PointState.n_ext` (E-8.3 `n = N_ext + min(N, cap − N_ext)`, presence 1, n_ext of E-8.10, Heat strip step 4 and the external episode of 08 §5.4); MC `McState.external` (own stacks use the free slots, no expiry, no damage, no on-proc triggers) |

`segments` is the run-length form (maximal runs of shots with constant spacing and equal fields other than time); a
20 s rifle window is about 3-10 segments. `WindowExact` visits every shot; `WindowFast` and `Bound` work per segment.

---

## 6. Enemy side

`enemy::scaling` computes pools with the f32 blend of NP-2 (05 §6.3), Steel Path, Eximus, party and overguard rules
(05 §7-§8); `enemy::armor` computes A0 and A_eff (E-15, with ability strips, Corrosive Projection, T-07, Heat and
Corrosive strips); `enemy::layers::instance` applies E-12's enemy side per layer (φ, Viral, Magnetic, V, armor, armor
floor of 1, innate DR, attenuation, CD-09) and E-13's tick rules; `enemy::layers::route` implements ST-9 (OG → SH →
HP, Toxin routing and exceptions, overflow, shield gate CD-08) for the `layered` view only; `enemy::dr` implements
05 §15 (attenuation, flat DR, adaptation, T-28).

---

## 7. EV evaluator

Spec 00 §4.2 binds the algorithm and 08 §5 fixes it (E-8.2 to E-8.15, assumptions A-1 to A-9). This section maps the
equations onto code; where it is shorter than 08, 08 wins.

### 7.1 The kernel

`ev::kernel::point(ra, state, view) -> PointOut` evaluates one target-state point: given the expected state at
time τ (Poisson means μ_s(τ) for every status in `ra.mask`, combo tier, condition-stack expectations, shot marks) and
a view, it returns the expected direct damage per pellet, the expected DoT damage per proc of each DoT status landing
at τ, the expected extra-hit damage, and the expected procs per pellet.

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
independence. Beyond 4 gates it linearizes the remaining ones and emits warning `ev.approx.gate_split_limit`. Under
`all_max` and `none` every gate is constant, so no split happens (the optimizer's default).

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
`ExtraHit(mechanic)`, `Duplicate`, `Child(attack)`, `MagneticBreak`); `outputs.rs` maps them to the source ids above.

### 7.8 Cost

Measured by the P-O prototype on one 2.1 GHz core: one target-state point about 140 ns; a 300-shot `WindowExact`
about 42 µs. The calculator evaluates every attack and every present view with `WindowExact` on each change (a few
hundred µs per weapon); `Steady`, `WindowFast` and `Bound` exist for the optimizer.

### 7.9 Kernel API for the optimizer

```rust
pub struct Kernel<'a> { cs: &'a CompiledScenario, objective: &'a ObjectiveSpec, statics: AttackStatics<'a> }
pub struct ObjectiveSpec { pub metric: Metric, pub view: View, pub attacks: SmallVec<[(AttackIdx, f64); 2]>, pub window: f64 }
pub use wfd_model::api::Metric;   // DpsWindow | DpsBurst | DpsSustained | Ttk | TtkNoarmor (C-28) | HitInfight | HitFresh
pub enum Fidelity { Steady, WindowFast, WindowExact }
pub struct BuildPoint<'p> { pub sv: &'p [StatVec], pub elements: &'p ElemOutcome, pub coeffs: &'p [Coeff], pub mech: &'p MechSet }

/// Ordered score key, compared lexicographically: lower `band` is better, then higher `primary`, then higher `tie`.
/// DPS and hit metrics: band 0, primary = metric value, tie = −ammo_window (08 §10 tie-break).
/// TTK (E-8.26): band 0 killed: primary = −T_EV, tie = κ (larger overkill first), then −ammo_to_kill;
///               band 1 extrapolated: primary = −T_EV; band 2 never: primary = dps_sustained on the layered view's
///               last layer. `tie2` carries the second tie-break so the order is total.
///               `a > b` means a is better; PartialEq and Eq agree with Ord (bitwise via total_cmp).
#[derive(Clone, Copy)] pub struct Score { pub band: u8, pub primary: f64, pub tie: f64, pub tie2: f64 }
impl Ord for Score { /* band asc, primary desc, tie desc, tie2 desc; f64 compared with total_cmp */ }
impl From<Score> for wfd_model::api::Score { … }   // and back: the DTO carries the same four fields

impl Kernel<'_> {
    pub fn score(&self, x: &BuildPoint, fid: Fidelity, ws: &mut Workspace) -> Score;
    /// Upper bound, in the order above, of `score(y, WindowExact)` for every y whose Inc slots are ≤ `upper`'s and whose
    /// Unknown slots, elements and mechanics equal `upper`'s (optimizer.md §4).
    pub fn bound(&self, upper: &BuildPoint, ws: &mut Workspace) -> Score;
}
pub struct Workspace { /* per-thread scratch arrays; no heap allocation in score/bound after warm-up */ }
impl<'a> Kernel<'a> { pub fn new(cs: &'a CompiledScenario, objective: &'a ObjectiveSpec, statics: AttackStatics<'a>) -> Self; }
```

**Why the TTK bound is valid.** `bound` runs the E-8.21 walk on the upper-envelope damage path of section 7.2 (every
consumer's expected damage per layer is an upper bound for all completions) and the upper bound of `dps_sustained`
from the same envelope. Pools then deplete no later than on any completion's path, so: if the envelope kills by
`t_ttk_max`, its band is 0 and `T_bound ≤ T_EV(y)` for every y (band 0) and for y in bands 1-2 the bound's band is
lower, hence better; if the envelope does not kill, its extrapolated time uses smaller remaining pools and larger
`dps_sustained`, so `T_bound ≤ T_EV(y)` in band 1, and band 2 only when every completion is band 2, with an upper
bound of the band-2 primary. The tie fields of a bound are set to +∞ (κ ≤ 1 and the ammo terms are then irrelevant).
So `bound(upper) ≥ score(y)` in the lexicographic order for every completion y, which is all branch-and-bound needs;
the property test of testing.md §2.3 checks it on random partial builds for every metric.

---

## 8. MC simulator

### 8.1 State and events

- **Target state**: per-status stack lists with expiry times on the µs grid (`ArrayVec<i64, 16>`, independent timers, replacement
  beyond cap, CD-07), DoT instances in a slab with snapshotted weapon-side seeds (CD-05), the shared Heat timer, pool
  and strip ramp, Blast fuses, pools per layer for the `layered` pass.
- **Attacker state**: magazine, reload, combo counter, sniper combo, ramp, spool, condition-stack counters per effect
  (04 E-4.8 state machine), mechanic counters (`counter_bonus`).
- **Events and processing**: exactly E-8.18 (08 §6.3): shot (inserted lazily from the schedule), stack/DoT expiry,
  Frozen end, Heat strip step, condition expiry or decay step, combo expiry (07), sniper decay (06), DoT tick (ELE/GAS
  grouped at processing time), Heat pulse, Heat episode end, Blast fuse expiry, Magnetic-break tick (layered passes);
  phases P1-P4 of 03 §7.1 with P4 split into (a) instances in order, (b) all procs of the shot, (c) zero-delay ticks.
- **Order**: a binary heap keyed `(time, phase, seq)` (08 §4.3); `seq` is creation order.

### 8.2 Random numbers

`mc/rng.rs` implements spec 08 E-8.16 and E-8.17, reproduced verbatim (the spec is normative; this copy is for the
implementer's convenience):

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
              half 0 (k even): words (x0, x1); half 1 (k odd): (x2, x3);  m = ((x_hi << 32) | x_lo) >> 11;  u = m·2^−53 ∈ [0, 1)
  shot-level draws (02 "ms_pellets", "enc", "enc_type"): j = 0xFFFF; reload-level draws: i = reload number, j = 0xFFFE
  derived instances: extra hits use the parent anchor and the mechanic id "xh/<source>/<name>" (source = catalogue id
              of 04, e.g. "xh/xata/sc_count"); Melee Duplicate: "dup/<name>"; conditional effects: "eff/<effect id>"
  an index outside its range is an error (no wrap-around)
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
  02 4.8 and 07 ("crit_roll", "crit_vig", "sc_count", "sc_type", "cedo_type", "ib", "welt", "hm", "st", "ms_pellets",
  "enc", "enc_type", "combo_acc", "combo_rc", …) are a `const` string list; derived ids ("xh/<source>/<name>",
  "dup/<name>", "eff/<effect key>") are formed from the bundle at L0. There is no Rust enum of mechanic ids.
- L0 hashes every id the scenario can use into `CompiledScenario.mech_ids` and fails with
  `EngineError::MechanicIdCollision` if two distinct strings hash to the same value; a unit test asserts that the
  fixed list and every derived id of the mini fixture and of the real bundle (in `xtask effect-smoke`) are distinct.
- `McRng` holds the key and the run and keeps one ordinal counter per (anchor, mechanic) in a reusable
  `OrdinalTable`; a draw (`next_u(mechanic)`) takes the next ordinal of its mechanic at the current anchor
  (`set_anchor`). `Anchor { a: u16, i: u32, j: u16 }` names the sentinel indices of E-8.16 (`SHOT_LEVEL_J` 0xFFFF,
  `RELOAD_J` 0xFFFE, `RUN_J` 0xFFFD with `RUN_I` 0xFFFFF). Anchors outside the ranges of E-8.16 (a ≥ 4096, i ≥ 2^20, j or b ≥ 65536) are
  `EngineError::RngIndexRange`, never wrapped.
- Two builds evaluated with the same seed see the same draw for the same (run, anchor, mechanic, ordinal): common
  random numbers for finalist confirmation. Known answers TV-08-01 (Philox) and TV-08-02 (keyed draws and the FNV-1a
  ids) are unit tests of `mc/rng.rs` and `mc/keys.rs`.

### 8.3 Passes and parallelism

Passes per run (08 §6.4), all with the same keys, so the same anchors see the same draws:

| Pass | Target | Horizon | Runs when |
|---|---|---|---|
| V (views) | non-depleting; every instance evaluated once per present view (health, armor, shield, overguard), four sinks | shots with s_i ≤ t_win, then DoT continuation for pending damage | always |
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

`timeline` re-runs one run (pass V; run index chosen by the user) and records per parent hit (08 §8.9): time, damage
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
    pub groups: Vec<GroupReport>,     // parent + child sums, T-22 combined units
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
record(&self, entry: TraceEntry); fn touch_row(&self, row: &'static str); }`). The default sink `NoTrace` is a
zero-sized no-op, so tracing costs nothing when off. The UI renders e.g.
"Critical chance 75.0 % = 12 % base × (1 + 150 % Point Strike) + 45 % Arcane Avenger (absolute, E-5)".

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

Mapping of the existing documents:

| Document | Vectors | Test kind |
|---|---|---|
| 00 §3.8 worked example | TV-00-01 (steps 1-10, states A and B) | scenario |
| 01 §8 | TV-01-01..TV-01-23 (composition, quantization, element order, faction) | stage (`base_vector`, `combine_elements`, `ratios`, `quantize`, `faction`) and scenario |
| 02 §9 | TV-02-01.. (crit quantization, tiers, Vigilante, body part, status, procs, multishot) | stage |
| 03 §19 | TV-03-.. (durations, caps, DoT ticks, Heat, Blast, Gas, Magnetic break, distinct count) | stage and scenario |
| 04 §15 | TV-04-.. (CO behaviours, G/G_F, condition state machine, extra hits, rivens, sets, custom stats) | stage and scenario |
| 05 §24 | TV-05-.. (scaling f32, armor, layers, DR, routing) | stage |
| 06 §16 | TV-06-.. (schedules, ramp, spool, sniper combo, duty) | stage (`schedule`) |
| 07 §14 | TV-07-01..TV-07-16 (attack speed and loop timing, combo ramp, heavy damage and loops, combo duration, zaw assembly, stance strikes, gunblade, heavy table resolution, gain rounding, stance normalisation E-7.0, build and spend, Tennokai, fallbacks, wave falloff) | stage (`melee_timing`, `combo_gain`, `heavy_hits`, `zaw_assemble`) and scenario (`melee_schedule` with a `MeleePlan`); TV-07-10 and TV-07-12 (E-7.0, E-7.40) are also executed by `wfd-datagen` (`crates/wfd-datagen/tests/spec_vectors_07.rs`) because the pipeline computes them |
| 08 §17 | TV-08-01..TV-08-16 (Philox known answers, keyed draws and FNV-1a ids, crit-only EV vs MC, capped Poisson, window walk, Heat pool and strip counter, correlated CO count, DoT survival, steady metrics, window-edge accounting, TTK, parity buckets, statistics, oracle comparison) | stage (`philox`, `rng_mech`, `draw`, `capped_poisson`, `heat_strip`, `summary`, `parity_buckets`, `oracle_compare`) and scenario |
| 09 §19 | TV-09-* (drain, polarity, capacity, Forma, rivens, valence, set counting) | executed by `wfd-data` (`crates/wfd-data/tests/spec_vectors_09.rs`, legality functions); twins in `crates/wfd-engine/tests/spec_vectors/09.toml` so the transcription check covers them |

Compatibility-profile values stated by a document (01 §8 "Compatibility-profile values") are twin entries with
`profile = "compat"`. Mechanics without a numeric vector get ordinary `#[test]`s named after the rule
(`e11_adding_omits_gf`, `cd04_sibling_pellets_no_viral`).

---

## 15. Errors and warnings

```rust
pub enum EngineError {                         // never a panic; serialized as { key, args }
    InvalidBuild(Vec<Message>), MissingData { what: Box<str> }, UnsupportedMechanic { effect: EffectKey },
    ScheduleTooLong { shots: u64 },            // horizon × rate over 10^7 events
    MechanicIdCollision { a: Box<str>, b: Box<str> },   // two mechanic id strings with one FNV-1a hash (8.2)
    RngIndexRange { what: Box<str>, value: u64 },       // anchor index outside the E-8.16 ranges (8.2)
    Cancelled,                                          // `calculate` cancelled through its CancelToken (T02a)
}
impl EngineError { pub fn message(&self) -> Message; /* EngineKey + args */ pub fn messages(&self) -> Vec<Message>; }
```

Warnings (`Message` with `MessageKey`) include: `item.unmodelled`, `item.stale`, `item.draft`,
`rule.unverified { row }`, `ev.approx.gate_split_limit`, `ev.mc_deviation { attack, view, ev, ci, gap_classes }`,
`ttk.extrapolated`, `ttk.shield_regen_possible`, `effect.no_spec_vector { effect }` (a `draft` effect whose DSL
construct has no spec vector yet, overview R-2).
