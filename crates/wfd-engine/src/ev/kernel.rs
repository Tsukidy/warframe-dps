//! The EV kernel (08 E-8.9; engine.md §7.1): one target-state point for one instance and one view. Given the expected
//! state at time τ (Poisson means μ_s(τ) for every status in `ra.mask`, combo, condition-stack expectations, shot marks)
//! it returns the expected direct damage per pellet, the expected DoT damage per proc of each DoT status landing at τ,
//! the expected extra-hit damage and the expected procs per pellet. Only statuses in `ra.mask` are visited and
//! `e^{−μ}` is cached per status per point. The state types here are the ones the mechanic traits read
//! (`mechanics`, engine.md §2.1); they are fixed size. The kernel gets the evaluation context ([`EvalCtx`]: rules,
//! compat flags, trace, the compiled scenario with the scaled enemy and toggles, the attack's dynamic terms) and the
//! instance it evaluates (contract review ENG-1). Owner: T15a.

use smallvec::SmallVec;
use wfd_model::{DamageVec, N_ST, StatusType, View};

use crate::EvalCtx;
use crate::instance::{DerivedHit, Instance};
use crate::outputs::SourceKind;
use crate::resolve::{N_MECH, ResolvedAttack};
use crate::status::ProcSnapshot;

/// Largest number of dynamic conditions one build holds in EV (`CompiledBuild.conditions`: class (b) stacks,
/// class (c)/(u) on/off switches, dynamic target predicates; `StateVar::CondStacks`, `StateVar::Switch`). A build
/// with more is rejected at L2 (`EngineError::UnsupportedMechanic` naming the first effect beyond the limit).
pub const MAX_COND_VARS: usize = 16;

/// The expected target and attacker state at one point τ (08 E-8.3, E-8.4; engine.md §2.1). Fixed size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointState {
    /// Time τ (s).
    pub t: f64,
    /// μ_s(τ): expected random own procs of s alive at τ (E-8.4).
    pub mu: [f64; N_ST],
    /// N_c,s(τ): certain own procs alive (forced procs, overview 4.2).
    pub certain: [u8; N_ST],
    /// Z_s(τ): probability that no random own proc of s is alive (used when N_c,s = 0).
    pub z: [f64; N_ST],
    /// N_ext,s (T-43 / T-06; 03 §6.3).
    pub n_ext: [u8; N_ST],
    /// P_H, the probability that a Heat episode is live (08 §5.4).
    pub heat_present: f64,
    /// P(k_h = 0 … 4), the Heat strip-counter distribution (E-8.6, E-8.7).
    pub heat_strip: [f64; 5],
    /// c̄, the expected-path melee combo count (07 §6.9); 0 for ranged attacks.
    pub combo: f64,
    /// Sniper combo multiplier at τ (06 §7.2).
    pub sniper_combo: f64,
    /// Expected stacks of each class (b) condition of the build (04 E-4.9, 08 E-8.15), indexed by position in
    /// `CompiledBuild.conditions` ([`EvalCtx::conditions`]); 0 for conditions of another kind.
    pub cond_stacks: [f64; MAX_COND_VARS],
    /// On/off groups of 08 E-8.9 step 4c (A-12): per condition of the build (same index as `cond_stacks`), the
    /// probability that its class (c)/(u) switch is on at this point: u_e for a free switch, 1 or 0 when the walk
    /// evaluates one branch of the mixture (`E = u_e·E[term | on] + (1 − u_e)·E[term | off]`, the same branch for the
    /// instance and every tick of the DoT it seeds). Addition, contract review ENG-11.
    pub switch_on: [f64; MAX_COND_VARS],
    /// ρ_RAG,e, the probability that the target is ragdolled before the shot of the instance's event (08 E-8.27,
    /// A-13, 02 E-2.13a); the kernel evaluates the event in the branches RAG = 0 and RAG = 1 and mixes them (E-8.2,
    /// TV-08-30). 0 in the `overguard` view and on `cc_immune` targets (08 §4.5). Addition, contract review ENG-11.
    pub rag: f64,
    /// s_flat(τ): expected count-weighted flat armor-strip points of the earlier qualifying hits (05 E-5.11, §19;
    /// 08 E-8.9 step 5; gap G-16), from which the kernel forms f_flat. Addition, contract review ENG-11.
    pub s_flat: f64,
    /// `og_present` of the point (03 E-3.2, 08 §4.5): true in the `overguard` view; on the layered EV path (E-8.21)
    /// the expected OG pool > 0. Addition, contract review ENG-11.
    pub og_present: bool,
    /// The expected HP fraction HP/HP_max before the instance on the layered EV path (E-8.21), for health-threshold
    /// predicates under T-02 `simulated` (08 §4.5); 1 in the non-depleting views. Addition, contract review ENG-11.
    pub hp_frac: f64,
    /// Expected counters of `counter_bonus` mechanics, per `mech.<n>` slot.
    pub counters: [f64; N_MECH],
    /// The instance's schedule marks: first and last shot of a magazine, reloaded fraction (06 E-6.13).
    pub marks: ShotMarks,
    /// Index j of the instance within its shot (sequential-pellet correction of 00 §4.2).
    pub pellet: u32,
}

/// Schedule marks of one instance (06 E-6.13, `Cond::Shot`, `Per::ReloadedFraction`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ShotMarks {
    /// First shot of its magazine.
    pub first: bool,
    /// Last shot of its magazine.
    pub last: bool,
    /// Reloaded fraction of the latest reload while it lasts (0 before the first reload).
    pub reloaded_fraction: f64,
}

/// Expected outputs of one point (engine.md §7.1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointOut {
    /// Expected direct damage per pellet on the view.
    pub direct: f64,
    /// Expected direct damage per pellet by final type.
    pub per_type: DamageVec,
    /// Expected DoT damage per proc of each DoT status landing at τ.
    pub dot_per_proc: [f64; N_ST],
    /// Expected extra-hit damage per pellet.
    pub extra: f64,
    /// Expected procs per pellet by status.
    pub procs: [f64; N_ST],
}

/// Accumulator of one instance's expected contributions on one view (engine.md §2.1): direct damage per type and per
/// source, DoT registrations, extra hits, proc masses.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EvAcc {
    /// The view being evaluated.
    pub view: Option<View>,
    /// Expected direct damage.
    pub direct: f64,
    /// Expected direct damage by final type.
    pub per_type: DamageVec,
    /// Expected damage by source (08 §8.2).
    pub per_source: SmallVec<[(SourceKind, f64); 8]>,
    /// DoT procs registered by the instance (their ticks are evaluated at their landing times, CD-05).
    pub dots: SmallVec<[DotReg; 4]>,
    /// Instances derived from this one by its mechanics (extra hits, Melee Duplicate copy, flat damage on proc), on
    /// the weapon side: the kernel applies the enemy side of the view jointly with the parent's state (08 E-8.9 step
    /// 1). Addition, contract review ENG-1 (typed extra hits).
    pub hits: SmallVec<[DerivedHit; 2]>,
    /// Expected damage on the view of the derived instances in `hits` (filled by the kernel).
    pub extra: f64,
    /// Expected procs by status.
    pub procs: [f64; N_ST],
}

/// An expected DoT proc registered at an instance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DotReg {
    /// The status.
    pub status: StatusType,
    /// Expected number of procs (weight).
    pub weight: f64,
    /// Snapshotted weapon-side terms (the inducing hit's expected crit, OQ-017 decision).
    pub snap: ProcSnapshot,
}

/// Expected proc events a mechanic adds for one proc event e (08 E-8.2: h_{s,e} presence, m_{s,e} mass).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ProcMass {
    /// m_{s,e}: expected procs of s.
    pub mass: [f64; N_ST],
    /// h_{s,e}: probability of at least one proc of s.
    pub presence: [f64; N_ST],
    /// Certain procs of s (presence 1, overview 4.2).
    pub certain: [u8; N_ST],
}

/// E-8.9: the kernel at one point for the instance `inst` of attack `ra` on `view`, in the context `cx` (the enemy
/// side of E-12 / 05 E-5.24 through `cx.cs.enemy`, the toggles of `cx.cs`, the attack's dynamic terms
/// `cx.coeffs`). The mechanics of the attack are called with the instance's expected outcome. T15a.
pub fn point(
    cx: &EvalCtx,
    ra: &ResolvedAttack,
    inst: &Instance,
    st: &PointState,
    view: View,
) -> PointOut {
    let _ = (cx, ra, inst, st, view);
    todo!("T15a")
}
