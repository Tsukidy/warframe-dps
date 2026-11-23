Status: draft (architecture)

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
    /// L0: per scenario (enemy scaling, faction relations, caps, T-02 mode, slot allocation, buffs, custom stats).
    pub fn compile_scenario(&self, build: &Build, scenario: &Scenario, objective: Option<&ObjectiveSpec>)
        -> Result<CompiledScenario, EngineError>;
    /// L1: one candidate (mod, arcane, riven, perk, shard, stance, set bonus) in a scenario.
    pub fn compile_pick(&self, cs: &CompiledScenario, pick: &Pick) -> CompiledItem;
    /// L2: a whole build: Σ contributions + finalizers (sets, shards, resolved scalars, element order).
    pub fn compile_build(&self, cs: &CompiledScenario, build: &Build) -> Result<CompiledBuild, EngineError>;
    /// L3: one attack (ST-1..ST-7) plus firing stats.
    pub fn resolve(&self, cs: &CompiledScenario, cb: &CompiledBuild, attack: AttackIdx) -> ResolvedAttack;
    /// Deterministic shot/hit schedule (06 E-6.11, 07), identical for EV and MC.
    pub fn schedule(&self, cs: &CompiledScenario, ra: &ResolvedAttack, horizon: f64) -> Schedule;
    /// L4: expected values (section 7).
    pub fn ev(&self, cs: &CompiledScenario, unit: &EvalUnit, req: &EvRequest, ws: &mut Workspace) -> EvOut;
    /// L4: Monte Carlo (section 8). Runs in parallel with feature `parallel`; results independent of thread count.
    pub fn mc(&self, cs: &CompiledScenario, unit: &EvalUnit, req: &McRequest, cancel: &CancelToken)
        -> Result<McOut, Cancelled>;
    /// Per-hit event log of one MC run (the inspiration's "More details").
    pub fn timeline(&self, cs: &CompiledScenario, unit: &EvalUnit, view: View, run: u32, limit: u32) -> Timeline;
    /// Read one spec symbol (section 11.2); used by spec vectors and the Explain drawer.
    pub fn probe(&self, build: &Build, scenario: &Scenario, path: &ProbePath) -> Result<ProbeValue, ProbeError>;
    pub fn explain(&self, build: &Build, scenario: &Scenario, path: &ProbePath) -> Result<Explanation, ProbeError>;

    /// Optimizer-facing scoring kernel (section 7.9).
    pub fn kernel<'a>(&'a self, cs: &'a CompiledScenario, objective: &'a ObjectiveSpec) -> Kernel<'a>;
}

/// What a calculation should produce (the server maps CalcRequest.want onto it).
pub struct Want { pub ev: bool /* always true in practice */, pub mc: Option<McWant /* runs, seed, bin */>,
                  pub parity: bool, pub breakdown: bool, pub trace: bool, pub views: Option<SmallVec<[View; 5]>> }
/// An attack or an attack group (T-22 combined or parent+child summed) evaluated together.
pub struct EvalUnit { pub attacks: SmallVec<[AttackIdx; 3]>, pub combined: bool }
pub enum Pick { Mod { item: ItemId, rank: u8, params: SmallVec<[(ParamKey, ParamValue); 1]> }, Riven(Riven),
                Arcane { item: ItemId, rank: u8 }, Perk { tier: u8, index: u8 }, Shard { item: ItemId, tauforged: bool },
                Stance { item: ItemId, combo: ComboKey }, FrameMod { item: ItemId, rank: u8 }, Aura { item: ItemId, rank: u8 },
                KitgunParts { grip: u16, loader: u16 }, ZawParts { grip: u16, link: u16 }, Progenitor { element: DamageType } }
pub struct CancelToken(Arc<AtomicBool>);
```

Report types (`CalcReport`, `EvOut`, `McOut`, `Timeline`, `Explanation`) are API DTOs in `wfd-model::api::calc` so
the frontend's types are generated from one crate (server-api.md §5).

---

## 2. Module layout

```text
crates/wfd-engine/src/
  lib.rs  error.rs  cancel.rs
  spec_constants.rs     formula constants of the spec only, each with its E-number (I-1)
  stats.rs              StatVec (fixed point), StatMask, SparseContrib, Coeff, MechSet
  state.rs              StateVar, state-variable table, distributions
  compile/              scenario.rs (L0), pick.rs (L1, T-02 table), scope.rs, build.rs (L2), sets.rs, shards.rs,
                        riven.rs, buffs.rs, custom.rs, passives.rs, conditions.rs (class derivation, overrides)
  assemble.rs           kitgun, zaw, progenitor (E-1 V_t), exalted Strength scaling
  elements.rs           element combination (01 §4.11), ElementSolver (all distinct outcomes, for the optimizer)
  resolve/              base.rs (E-1, 01 §4.2-4.7), composition.rs (E-2, 01 §4.9-4.12), quantize.rs (E-3),
                        scalar.rs (E-11, 04 §2), weapon_mult.rs (ST-5: F, G, G_F), part.rs (E-7), crit.rs (E-4..E-6),
                        status_chance.rs (E-8, E-9), multishot.rs (E-10), firing.rs (06 §5), melee.rs (07)
  schedule/             ranged.rs (06 E-6.11), melee.rs (07, SPEC-PENDING(07)), segment.rs (run-length form)
  enemy/                scaling.rs (NP-2), armor.rs (E-15), layers.rs (E-12 enemy side, ST-9, CD-08), dr.rs (innate,
                        attenuation, special DR, T-28), parts.rs (body parts, head-crit)
  status/               rules.rs (03 §4.1 rules from the bundle), dot.rs (E-13), heat.rs (03 §10), blast.rs (03 §11),
                        magnetic_break.rs (03 §13), distinct.rs (03 §14)
  mechanics/            one module per mechanic kind (domain-model.md §11.2), each with `ev` and `mc` functions
  ev/                   kernel.rs (one target-state point), poisson.rs, joint.rs (DR, crit joint enumeration),
                        gates.rs (gate splitting), steady.rs, window_exact.rs, window_fast.rs, bound.rs, ttk.rs,
                        hit.rs, group.rs, heat_mean.rs, cond_stacks.rs (04 E-4.9)
  mc/                   rng.rs (Philox4x32-10), keys.rs (mechanic ids), events.rs, state.rs, sim.rs, views.rs,
                        layered.rs, summary.rs, timeline.rs, parity.rs
  outputs.rs            CalcReport assembly, spec 00 §4.5 mapping
  compat.rs             CompatFlags helpers
  trace.rs              TraceSink (no-op by default), touched register rows
  probe.rs              ProbePath parser, probe registry
```

Each `resolve/`, `enemy/` and `status/` function is pure, takes a `&StageCtx` (rules, compat flags, trace sink), and
cites its equation in its doc comment, so a reviewer can put the spec and the code side by side.

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
    pub profile: CompatFlags,
    pub fixed: SmallVec<[SparseContrib; 2]>,   // buffs, custom stats, weapon/attack passives, zoom, toggles that are stats
    pub state_vars: StateVarTable,        // which StateVars exist in this scenario
    pub slot_class: [Monotone; N_STATS],  // per-scenario monotonicity (optimizer.md §4)
    pub mech_slots: MechSlotTable,        // allocation of `mech.<n>` slots
    pub mult_slots: MultSlotTable,        // allocation of `mult.<n>` slots from Rules.mult_groups
    pub objective: Option<ObjectiveSpec>, // attacks and weights for the optimizer
    pub window: f64, pub horizon: f64,
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

**Element outcome.** `elements::combine(entries: &[(OrderKey, DamageType, f64)]) -> ElemOutcome` implements 01
§4.11 (slot order, exilus after slot 7, riven bottom-up, innate group HCET, non-combining additions).
`ElementSolver::outcomes(set) -> SmallVec<[(ElemOutcome, Vec<SlotAssignment>); 3]>` returns every distinct outcome
an unordered set of element entries can reach, with one slot assignment realizing each (optimizer.md §5).

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
    pub melee: Option<MeleeResolved>,                                  // AS, combo, heavy (07, SPEC-PENDING)
    pub xh: [ExtraHitResolved; 8], pub mech: MechResolved,             // E-14 and mechanic parameters
    pub gates: GateSet,                                                // dynamic gates that feed nonlinear stages (7.4)
    pub mask: StatusSet,                                               // statuses this attack (or externals) can apply
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

`schedule::ranged::generate` implements 06 E-6.11 exactly: shots with cycle start, fire time, hit time, magazine
number and index, ammo before and after, cost, `first`/`last` flags, burst position, spool index and rate, Kohm
pellets, Peacemaker burst, ramp level; reloads (start, end, `from_empty`); sniper decay times; child hits per shot
(06 §9). `schedule::melee::generate` implements 07 (stance combos with hit multipliers and timings, combo counter,
heavy attacks, slams; SPEC-PENDING(07): until 07 lands it follows `wiki-melee-mechanics.md` §1-§5 and spec 00 T-09,
T-10). The schedule never depends on random draws.

```rust
pub struct Schedule { pub shots: Vec<ShotRec>, pub reloads: Vec<ReloadRec>, pub segments: SmallVec<[Segment; 16]>,
                      pub cycle: CycleStats /* EFR, shots per magazine, duty ρ, T_mag, T_rel */ }
```

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

Spec 00 §4.2 binds the algorithm; 08 (SPEC-PENDING) owns the details. This is the structure that implements it.

### 7.1 The kernel

`ev::kernel::point(ra, state, view, ws) -> PointOut` evaluates one target-state point: given the expected state at
time τ (Poisson means μ_s(τ) for every status in `ra.mask`, combo tier, condition-stack expectations, shot marks) and
a view, it returns the expected direct damage per pellet, the expected DoT damage per proc of each DoT status landing
at τ, the expected extra-hit damage, and the expected procs per pellet.

| Quantity | EV treatment |
|---|---|
| Crit | `E[c] = 1 + E[k]·(h·CDM − 1)`, linear in CC (CD-13); with Vigilante via E-6 |
| Puncture CC and Cold CDM | `E[c]` is enumerated over the joint support of Puncture stacks (0..cap) and Cold stacks (0..cap + Frozen) under independent capped Poisson laws: at most 6 × 11 states |
| Viral, Magnetic, Cold, Puncture functions | `E[f(n)]` by a capped Poisson sum (one `exp` plus a recurrence), never `f(E[n])` |
| Armor | `E[1 − DR(A_eff)]` over the joint distribution of Corrosive stacks, Heat presence and the Heat-strip ramp state |
| Distinct count n | `E[n] = Σ_s (1 − e^{−μ_s})` + externals (T-06); T-05 `weapon_types` and `manual` replace it; sequential-pellet correction of spec 00 §4.2 for pellet j of a shot |
| CO | per the attack's `CoSpec` (E-11): `adding` adds `ρ_co·B_ref·E[C]`, `multiplying` multiplies by `1 + E[C]`; C is affine in n, n_COL and condition stacks, so its expectation is exact under independence |
| Proc counts | `SC·w_s + φ_s` per pellet; mechanic procs (`proc_on_event`) from event rates; exclusive groups `1 − Π(1 − p_i)` |
| DoT | per proc: ticks at their landing times with the target state at that time (CD-05); weapon-side seed with the inducing hit's expected crit (OQ-017 decision) |
| Extra hits | E-14 per pellet, with the parent's expected crit and part factors |
| Condition stacks (`simulated`, class (b)) | 04 E-4.9: single stack `1 − q`; independent timers capped Poisson; `shared_lose_one` Markov chain; walked through time in window mode |
| Products of state variables | expectation of a product of independent variables is the product of expectations (Galvanized stacks × n); this is a documented approximation class (§7.6) |

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
- **Heat**: one shared refreshing timer; while present the pool sums every Heat proc since ignition; EV uses the
  window mean of spec 00 §4.2 for steady outputs and walks the pool expectation in window mode (03 §10).
- **Blast**: fuse per stack, detonation at 10 stacks (4 on bosses) by the expected-count approximation of 03 §11.
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

- **EV TTK** walks the `layered` view (OG → SH → HP, Toxin routing, CD-08 overflow, no shield gate) with the window
  machinery and finds the first time the expected cumulative damage reaches the pools (bisection over the piecewise
  integral). Beyond `t_ttk_max` it extrapolates with `dps_sustained` and sets `extrapolated = true` (spec 00 §4.2).
  `Bound` yields a valid lower bound of TTK for the optimizer.
- **Hit metrics** (optimizer metrics of the inspiration): `hit_fresh` = expected damage of one shot on the fresh
  target; `hit_infight` = expected damage of one shot at the steady state; one kernel point each.

### 7.6 Approximation classes

EV results are exact for the spec's definitions except these documented approximations (each listed by 08 when it
lands): Poisson live-stack model; independence across status types; independence of condition stacks and target
state; gate-split limit; Blast detonation expectation; Heat window mean in steady outputs. `CalcReport` sets the
deviation flag when EV `dps_window` lies outside the MC 99 % CI widened by 2 % (spec 00 §4.4).

### 7.7 Outputs of the EV evaluator

```rust
pub struct EvOut { pub views: SmallVec<[ViewEv; 5]> }
pub struct ViewEv {
    pub view: View, pub dps_burst: f64, pub dps_sustained: f64, pub dps_window: f64,
    pub per_shot: f64, pub per_magazine: f64, pub per_type: DamageVec,
    pub per_source: SmallVec<[(SourceKind, f64); 8]>,      // direct, DoT per status, extra hits, specials
    pub stacks: SmallVec<[StackStat; 8]>,                   // E[n_s], P(n_s ≥ 1), P(n_s = cap) (steady)
    pub distinct_mean: f64, pub crit_tiers: [f64; 4],
    pub series: Vec<f64>,                                   // expected damage per bin (window walk)
    pub ttk: Option<TtkEv>,                                 // { seconds, extrapolated }
    pub hit_fresh: f64, pub hit_infight: f64,
}
```

### 7.8 Cost

Measured by the P-O prototype on one 2.1 GHz core: one target-state point about 140 ns; a 300-shot `WindowExact`
about 42 µs. The calculator evaluates every attack and every present view with `WindowExact` on each change (a few
hundred µs per weapon); `Steady`, `WindowFast` and `Bound` exist for the optimizer.

### 7.9 Kernel API for the optimizer

```rust
pub struct Kernel<'a> { cs: &'a CompiledScenario, objective: &'a ObjectiveSpec, statics: AttackStatics<'a> }
pub struct ObjectiveSpec { pub metric: Metric, pub view: View, pub attacks: SmallVec<[(AttackIdx, f64); 2]>, pub window: f64 }
pub enum Metric { DpsWindow, DpsBurst, DpsSustained, Ttk, HitFresh, HitInFight }
pub enum Fidelity { Steady, WindowFast, WindowExact }
pub struct BuildPoint<'p> { pub sv: &'p [StatVec], pub elements: &'p ElemOutcome, pub coeffs: &'p [Coeff], pub mech: &'p MechSet }

impl Kernel<'_> {
    /// Score a complete build at the given fidelity (higher is better; TTK is negated).
    pub fn score(&self, x: &BuildPoint, fid: Fidelity, ws: &mut Workspace) -> f64;
    /// Upper bound of `score(y, WindowExact)` for every y whose Inc slots are ≤ `upper`'s and whose Unknown
    /// slots, elements and mechanics equal `upper`'s (optimizer.md §4).
    pub fn bound(&self, upper: &BuildPoint, ws: &mut Workspace) -> f64;
}
pub struct Workspace { /* per-thread scratch arrays; no heap allocation in score/bound after warm-up */ }
```

---

## 8. MC simulator

### 8.1 State and events

- **Target state**: per-status stack lists with expiry times (`ArrayVec<f64, 16>`, independent timers, replacement
  beyond cap, CD-07), DoT instances in a slab with snapshotted weapon-side seeds (CD-05), the shared Heat timer, pool
  and strip ramp, Blast fuses, pools per layer for the `layered` pass.
- **Attacker state**: magazine, reload, combo counter, sniper combo, ramp, spool, condition-stack counters per effect
  (04 E-4.8 state machine), mechanic counters (`counter_bonus`).
- **Events**: shot (from the schedule), pellet hit, radial hit, extra hit, proc application, stack expiry, DoT tick
  (one shared Heat tick per second), Blast fuse expiry and detonation, Heat strip step, reload start and end, combo
  gain and expiry, condition stack gain and expiry, sniper combo decay, beam ramp, Magnetic break proc.
- **Order**: a binary heap keyed `(time, phase, seq)`; `phase` encodes CD-04 (damage from the state before the
  instance, then crit and procs rolled, damage applied, procs applied; Electricity and Gas tick 0 after procs) and 03
  §7.1; `seq` is creation order.

### 8.2 Random numbers

Philox4x32-10 in `mc/rng.rs` (about 60 lines, checked against the Random123 known-answer vectors). Key =
`(seed, run)`; counter = `(attack index, event index, mechanic id, draw index)` (spec U-6). `MechanicId` is a
`#[repr(u16)]` enum (`CritTier`, `TierRaise`, `PelletCount`, `ProcCount`, `ProcType`, `ExtraHitStatus`,
`RepeatHit`, `RandomMultiplier`, …) plus a hash of the `EffectKey` for effect-driven draws (condition triggers with
chance, `proc_on_event`). Draws are exact `f64` in [0, 1) from 53 bits; chances are never rounded (D-017). The same
key and counter give the same draw in any build, so finalists are compared with common random numbers.

### 8.3 Passes and parallelism

- One pass per run against a non-depleting target evaluates every instance for the four views at once (the status
  state is identical in all of them) and accumulates four sinks.
- A second pass with the same draws simulates the `layered` target with depletion, shield gate, overflow (CD-08),
  health thresholds (`HealthBelow`), and "no extra hit after a kill", until kill or `t_ttk_max`.
- Runs are independent: with feature `parallel`, `(0..n_runs).into_par_iter()` writes results into a `Vec` indexed by
  run, reduced in run order (I-5). Cancellation is checked between runs.

### 8.4 Outputs

```rust
pub struct McOut { pub views: SmallVec<[ViewMc; 5]>, pub runs: u32, pub seed: u64 }
pub struct ViewMc {
    pub view: View, pub dps_window: Summary,                // mean, sd, ci95, median, p5, p25, p75, p95, min, max
    pub series: Vec<BinSummary>,                            // mean, p10, p90 per bin
    pub uptime: Vec<StatusSeries>, pub crit_hist: [u64; 6], pub breakdown: SmallVec<[(SourceKind, f64); 8]>,
    pub pending_dot: f64,
    pub ttk: Option<TtkDist>,                               // kill probability, median, mean of killed runs, p5, p95
    pub shots_to_kill: Option<Summary>, pub ammo_to_kill: Option<Summary>,
    pub parity: Option<ParityOut>,                          // section 9
}
```

`timeline` re-runs one run and records per hit: time, damage per view, crit multiplier, armor reduction, stance and
heavy-combo multipliers, extra-hit damage, statuses applied, one-tick value per status, live stacks per status, and
reload markers (spec 00 §4.5 "More details").

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
inputs: SmallVec<[(ProbePath, f64); 4]>, contributions: SmallVec<[(EffectRef, f64); 8]>, rows: SmallVec<[&'static str; 2]> }`
into a `TraceSink`. The default sink is a zero-sized no-op, so tracing costs nothing when off. The UI renders e.g.
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
default `panic = "unwind"`, server-api.md §11), one objective attack, one view. They are acceptance criteria of tasks T14, T15 and
T33; CI fails when a benchmark misses its target by more than 2× (CI machines vary) or regresses by more than 25 %
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
| MC | ≥ 5 M events/s/core; 1,000 runs of a 20 s Braton Prime window ≤ 100 ms on 8 cores | UI distributions, confirmation |

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
| 07, 08, 09 | SPEC-PENDING | — |

Compatibility-profile values stated by a document (01 §8 "Compatibility-profile values") are twin entries with
`profile = "compat"`. Mechanics without a numeric vector get ordinary `#[test]`s named after the rule
(`e11_adding_omits_gf`, `cd04_sibling_pellets_no_viral`).

---

## 15. Errors and warnings

```rust
pub enum EngineError {                         // never a panic; serialized as { key, args }
    InvalidBuild(Vec<Message>), MissingData { what: Box<str> }, UnsupportedMechanic { effect: EffectKey },
    ScheduleTooLong { shots: u64 },            // horizon × rate over 10^7 events
}
```

Warnings (`Message` with `MessageKey`) include: `item.unmodelled`, `item.stale`, `item.draft`,
`rule.unverified { row }`, `ev.approx.gate_split_limit`, `ev.mc_deviation { attack, view, ev, ci }`,
`attack.spec_pending { doc }`, `ttk.extrapolated`.
