//! `wfd-engine`: the damage engine of warframe-dps (docs/architecture/engine.md; spec docs/mechanics/00-09).
//!
//! The engine resolves a build into per-attack stats (ST-1..ST-7, [`resolve`]), generates the deterministic shot and
//! hit schedule (06, 07; [`schedule`]), applies the enemy side (ST-8, ST-9; [`enemy`]) and status effects (ST-10;
//! [`status`], [`mechanics`]), and evaluates with the two evaluators of spec 00 §4: expected values ([`ev`]) and the
//! event-driven Monte Carlo simulator ([`mc`]). It does no I/O, holds no global state and depends only on `wfd-model`;
//! the optimizer uses the lower-level [`Kernel`] API (engine.md §7.9, optimizer.md).
//!
//! **Skeleton (T02a).** This crate was created by T02a with the frozen public signatures of work-breakdown.md T02a and
//! the internal types of engine.md §2.1-§8. Every body that implements a spec rule is `todo!("Txx")`, naming the task
//! that implements it (work-breakdown.md §8 assigns each file); tests that need such a body are
//! `#[ignore = "Txx"]`. An implementing task may add private items; a change to a public signature is a
//! `CONTRACTS.md` entry (work-breakdown.md §2.1). Structural helpers with no spec content (constructors, accessors,
//! the [`Score`] order, [`CancelToken`], error messages, the mechanic dispatch) are implemented here.
//!
//! Module owners (work-breakdown.md §8): `compile/` (except `melee.rs`), `stats`, `state`, `elements` T06; `enemy/`
//! T07; `schedule/{mod,ranged,segment}`, `resolve/firing` T08a; `compile/melee`, `resolve/melee`, `schedule/melee`
//! T08b; `stage_registry` T09; `resolve/` (rest), `assemble`, `spec_constants` T13; `status/`, `mechanics/` T14;
//! `ev/` T15a-T15c; `mc/` T16; `lib`, `outputs`, `compat`, `trace`, `probe` T17. `error` and `cancel` have no
//! implementing task: their content is complete.
//!
//! Features: `parallel` (default) runs MC runs on the rayon pool; results do not depend on the thread count (I-5,
//! engine.md §8.3). The crate builds without default features (no rayon), so a WebAssembly build stays possible
//! (overview.md §3.1).

#![warn(missing_docs)]

pub mod assemble;
pub mod cancel;
pub mod compat;
pub mod compile;
pub mod elements;
pub mod enemy;
pub mod error;
pub mod ev;
pub mod mc;
pub mod mechanics;
pub mod outputs;
pub mod probe;
pub mod resolve;
pub mod schedule;
pub mod spec_constants;
pub mod stage_registry;
pub mod state;
pub mod stats;
pub mod status;
pub mod trace;

use smallvec::SmallVec;
use wfd_model::{AttackIdx, Build, Bundle, CompatFlags, Rules, Scenario, View};

pub use assemble::AssembledWeapon;
pub use cancel::CancelToken;
pub use compile::{
    CompiledBuild, CompiledItem, CompiledMelee, CompiledScenario, MeleePlan, MeleeTablesOwned,
    PlanKind, Reader, melee_plan, monotone_classes, plan_selection, resolve_combo_tables,
};
pub use elements::{ElemOutcome, ElementSolver, OrderKey, OrderedElem, SlotAssignment};
pub use enemy::{
    ArmorStrips, LayeredHit, Pools, RouteOut, ScaledEnemy, TargetStateView, armor_dr,
    effective_armor, instance_on_layer, np2_blend, route_layered, scale_enemy,
};
pub use error::{Cancelled, EngineError};
pub use ev::{
    BuildPoint, EvAcc, EvRequest, Fidelity, Kernel, Metric, ObjectiveSpec, PointState, ProcMass,
    Score, Workspace, capped_poisson_mean,
};
pub use mc::{
    Anchor, McEvent, McEventKind, McRequest, McRng, McSink, McState, MechanicId, OracleVerdict,
    Phase, SubKey, oracle_compare, oracle_compare_q, philox4x32_10, rng_mech,
};
pub use mechanics::{MechInst, MechanicEv, MechanicImpl, MechanicMc, mechanic_impl};
pub use probe::{ProbeError, ProbePath};
pub use resolve::{
    BaseVector, ChildLink, CritMult, FiringResolved, MeleeResolved, ResolvedAttack, ScalarY,
    ScheduleInputs, TierDist, base_scalar, base_vector, combine_elements, crit_chance,
    crit_multiplier, crit_multiplier_terms, crit_tiers, firing_stats, melee_resolved, multishot,
    part_factor, proc_weights, quantize, ratios, status_chance,
};
pub use schedule::{
    CycleStats, MeleeTrack, ReloadRec, Schedule, Segment, ShotRec, melee_schedule, ranged_schedule,
};
pub use stage_registry::{StageError, stage_registry};
pub use state::StateVar;
pub use stats::{Coeff, MechSet, SparseContrib, StatMask, StatVec, StatVecF};
pub use status::{ExtraHitOut, HitWeaponSide, ProcSnapshot, dot_tick, extra_hit};
pub use trace::{NoTrace, TraceSink};

/// The report types are API DTOs of `wfd-model::api::calc`, so the frontend's types come from one crate
/// (engine.md §1, server-api.md §5; CONTRACTS.md T01b).
pub use wfd_model::api::{
    CalcReport, EvOut, Explanation, McOut, ParityOut, ProbeValue, Summary, Timeline, TtkDist,
    TtkEv, ViewEv, ViewMc,
};

/// The engine over one bundle: cheap to create, `Sync`, no state of its own (engine.md §1).
#[derive(Debug, Clone, Copy)]
pub struct Engine<'b> {
    bundle: &'b Bundle,
}

impl<'b> Engine<'b> {
    /// An engine over `bundle`.
    pub fn new(bundle: &'b Bundle) -> Self {
        Engine { bundle }
    }

    /// The bundle the engine reads.
    pub fn bundle(&self) -> &'b Bundle {
        self.bundle
    }

    /// The server's and the CLI's entry point: everything for the calculator page (engine.md §1, §9; spec 00 §4.5).
    /// T17.
    pub fn calculate(
        &self,
        build: &Build,
        scenario: &Scenario,
        want: &Want,
        cancel: &CancelToken,
    ) -> Result<CalcReport, EngineError> {
        let _ = (build, scenario, want, cancel);
        todo!("T17")
    }

    // ---- levels (engine.md §3); the optimizer and the tests use them directly ----

    /// L0: per scenario (enemy scaling, faction relations, caps, T-02 mode, slot allocation, buffs, custom stats,
    /// the T-41 hit fractions and T-43 external statuses, melee tables and plans; engine.md §3, §5). T06 (melee part
    /// T08b).
    pub fn compile_scenario(
        &self,
        build: &Build,
        scenario: &Scenario,
        objective: Option<&ObjectiveSpec>,
    ) -> Result<CompiledScenario, EngineError> {
        let _ = (build, scenario, objective);
        todo!("T06")
    }

    /// L1: one candidate (mod, arcane, riven, perk, shard, stance, set bonus) in a scenario (domain-model.md §18).
    /// T06.
    pub fn compile_pick(&self, cs: &CompiledScenario, pick: &Pick) -> CompiledItem {
        let _ = (cs, pick);
        todo!("T06")
    }

    /// L2: a whole build: Σ contributions + finalizers (sets, shards, resolved scalars, element order). T06.
    pub fn compile_build(
        &self,
        cs: &CompiledScenario,
        build: &Build,
    ) -> Result<CompiledBuild, EngineError> {
        let _ = (cs, build);
        todo!("T06")
    }

    /// L3: one attack (ST-1..ST-7) plus firing stats (engine.md §4). T13.
    pub fn resolve(
        &self,
        cs: &CompiledScenario,
        cb: &CompiledBuild,
        attack: AttackIdx,
    ) -> ResolvedAttack {
        let _ = (cs, cb, attack);
        todo!("T13")
    }

    /// Deterministic shot/hit schedule (06 E-6.11; melee 07 E-7.30 with the scenario's [`MeleePlan`]), identical for
    /// EV and MC (engine.md §5). Dispatches to [`ranged_schedule`] (T08a) or [`melee_schedule`] (T08b).
    pub fn schedule(&self, cs: &CompiledScenario, ra: &ResolvedAttack, horizon: f64) -> Schedule {
        let _ = (cs, ra, horizon);
        todo!("T08a, T08b")
    }

    /// L4: expected values of one evaluation unit (engine.md §7). `req.build` is the compiled build; the evaluator
    /// resolves and schedules the unit's attacks itself. T15a (window walk T15b).
    pub fn ev(
        &self,
        cs: &CompiledScenario,
        unit: &EvalUnit,
        req: &EvRequest,
        ws: &mut Workspace,
    ) -> EvOut {
        let _ = (cs, unit, req, ws);
        todo!("T15a")
    }

    /// L4: Monte Carlo (engine.md §8). Runs in parallel with feature `parallel`; results independent of the thread
    /// count. A cancelled run set is [`EngineError::Cancelled`]; an RNG index outside the ranges of 08 E-8.16
    /// (reported by [`McRng::check`]) is [`EngineError::RngIndexRange`]. engine.md §1 returns
    /// `Result<McOut, Cancelled>`, which cannot carry the second error (CONTRACTS.md T02a, C-35). T16.
    pub fn mc(
        &self,
        cs: &CompiledScenario,
        unit: &EvalUnit,
        req: &McRequest,
        cancel: &CancelToken,
    ) -> Result<McOut, EngineError> {
        let _ = (cs, unit, req, cancel);
        todo!("T16")
    }

    /// Per-hit event log of one MC run (pass V; the inspiration's "More details", 08 §8.9). The compiled build `cb`
    /// is an addition to engine.md §1 (CONTRACTS.md T02a, C-32): a run needs the build's resolved attacks. The run
    /// can fail as [`Engine::mc`] does ([`EngineError::RngIndexRange`]; CONTRACTS.md T02a, C-35). T16.
    pub fn timeline(
        &self,
        cs: &CompiledScenario,
        cb: &CompiledBuild,
        unit: &EvalUnit,
        view: View,
        run: u32,
        limit: u32,
    ) -> Result<Timeline, EngineError> {
        let _ = (cs, cb, unit, view, run, limit);
        todo!("T16")
    }

    /// Read one spec symbol (engine.md §11.2); used by spec vectors and the Explain drawer. T17.
    pub fn probe(
        &self,
        build: &Build,
        scenario: &Scenario,
        path: &ProbePath,
    ) -> Result<ProbeValue, ProbeError> {
        let _ = (build, scenario, path);
        todo!("T17")
    }

    /// The Explain drawer's view of one probe (engine.md §11.3). T17.
    pub fn explain(
        &self,
        build: &Build,
        scenario: &Scenario,
        path: &ProbePath,
    ) -> Result<Explanation, ProbeError> {
        let _ = (build, scenario, path);
        todo!("T17")
    }

    /// Optimizer-facing scoring kernel (engine.md §7.9); scores are ordered keys ([`Score`]), not plain f64. T15c.
    pub fn kernel<'a>(
        &'a self,
        cs: &'a CompiledScenario,
        objective: &'a ObjectiveSpec,
    ) -> Kernel<'a> {
        let _ = (cs, objective);
        todo!("T15c")
    }
}

/// Context of every pure stage function of `resolve/`, `enemy/`, `status/` and `schedule/` (work-breakdown.md
/// T02a): the bundle's rules, the active compatibility flags (empty in the game profile, engine.md §10) and the trace
/// sink (a no-op unless `Want.trace`, engine.md §11.1).
#[derive(Clone, Copy)]
pub struct StageCtx<'a> {
    /// `Bundle.rules`.
    pub rules: &'a Rules,
    /// The compatibility flags of `Profile::Compat`, empty in the game profile.
    pub compat: CompatFlags,
    /// Where stages write their trace entries.
    pub trace: &'a dyn TraceSink,
}

impl<'a> StageCtx<'a> {
    /// A context without tracing.
    pub fn new(rules: &'a Rules, compat: CompatFlags) -> Self {
        StageCtx {
            rules,
            compat,
            trace: &trace::NO_TRACE,
        }
    }

    /// A context that writes to `trace`.
    pub fn traced(rules: &'a Rules, compat: CompatFlags, trace: &'a dyn TraceSink) -> Self {
        StageCtx {
            rules,
            compat,
            trace,
        }
    }
}

impl std::fmt::Debug for StageCtx<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StageCtx")
            .field("compat", &self.compat)
            .field("trace_enabled", &self.trace.enabled())
            .finish_non_exhaustive()
    }
}

/// What a calculation should produce; the server maps `CalcRequest.want` onto it (engine.md §1).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Want {
    /// Expected values (always true in practice).
    pub ev: bool,
    /// Monte Carlo with its resolved parameters; `None` = EV only.
    pub mc: Option<McWant>,
    /// Parity outputs of 08 §8.8 (pass L0 when the enemy has armor, 08 §6.4).
    pub parity: bool,
    /// Per-source breakdowns (08 §8.2).
    pub breakdown: bool,
    /// Stage traces (engine.md §11.1).
    pub trace: bool,
    /// Views to evaluate; `None` = every view the enemy has.
    pub views: Option<SmallVec<[View; 5]>>,
}

/// Resolved MC parameters of a calculation (T-29; `CalcRequest.want.mc` with the scenario's `eval` defaults applied
/// by the server).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct McWant {
    /// Number of runs, 1-100,000 (T-29).
    pub runs: u32,
    /// Seed 0 ..= 2^53 − 1 (T-29; `"auto"` is drawn by the frontend).
    pub seed: u64,
    /// Width of the series bins (s; 08 §8.1 `out_bin`).
    pub series_bin_s: f64,
}

/// An attack or an attack group evaluated together: a T-22 combined family, or a parent and its children summed
/// (08 §9, E-8.23; engine.md §1).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EvalUnit {
    /// The member attacks; the first is the unit's firing attack.
    pub attacks: SmallVec<[AttackIdx; 3]>,
    /// `true`: one shared target timeline (T-22 on, CD-12); `false`: members evaluated on their own and summed.
    pub combined: bool,
}

/// One candidate of the optimizer or one installed piece of a build, compiled at L1 by [`Engine::compile_pick`]
/// (engine.md §1, §3; domain-model.md §18). The type is `wfd_model::Pick`, re-exported here (manager ruling T02b-2:
/// `wfd-data::legality::check_slot` names the same type, and `wfd-data` does not depend on the engine).
pub use wfd_model::Pick;

#[cfg(test)]
pub(crate) mod test_support {
    //! Fixture access for unit tests (the mini bundle of T03, read through serde as overview.md §3.1 says).

    /// The rules table of the mini fixture (`data/fixtures/mini/rules.json`).
    pub fn mini_rules() -> wfd_model::Rules {
        serde_json::from_str(include_str!("../../../data/fixtures/mini/rules.json"))
            .expect("mini fixture rules.json deserializes into Rules")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_is_sync_send_copy() {
        fn assert_traits<T: Sync + Send + Copy>() {}
        assert_traits::<Engine<'static>>();
        assert_traits::<StageCtx<'static>>();
    }

    #[test]
    fn stage_ctx_defaults_to_no_trace() {
        let rules = test_support::mini_rules();
        let ctx = StageCtx::new(&rules, CompatFlags::empty());
        assert!(!ctx.trace.enabled());
        assert!(format!("{ctx:?}").contains("StageCtx"));
    }
}
