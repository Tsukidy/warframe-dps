//! Mechanic kinds (domain-model.md §11.2; engine.md §2.1, frozen by T02a). Every kind implements the EV trait
//! [`MechanicEv`] and the MC trait [`MechanicMc`]; this module (T14) writes the implementations and the evaluators
//! (T15a-T15c, T16) only call them, through [`mechanic_impl`]. The state types the traits name belong to the
//! evaluators and are re-exported here: [`PointState`], [`EvAcc`], [`ProcMass`] (`ev`, T15a) and [`McState`],
//! [`McEvent`], [`McRng`], [`McSink`], [`MechanicId`] (`mc`, T16); the instance types ([`InstanceOutcome`],
//! [`DerivedHit`], [`ProcApply`]) are [`crate::instance`]'s. Every method gets the evaluation context
//! [`crate::EvalCtx`] (stage context, compiled scenario with the scaled enemy and the toggles, the attack's dynamic
//! terms) and the outcome of the instance it reacts to (contract review ENG-1). Owner: T14.

pub mod convert_damage;
pub mod counter_bonus;
pub mod extra_hit;
pub mod flat_damage_on_proc;
pub mod proc_after_ticks;
pub mod proc_on_event;
pub mod random_multiplier;
pub mod repeat_hit;
pub mod spawn_attack;
pub mod stack_present_statuses;

use smallvec::SmallVec;
use wfd_model::{EffectKey, Mechanic, MechanicKind, View};

pub use crate::ev::kernel::{EvAcc, PointState, ProcMass};
pub use crate::instance::{DerivedHit, InstanceOutcome, ProcApply};
pub use crate::mc::events::{Anchor, McEvent, McEventKind, McSink, Phase, SubKey};
pub use crate::mc::keys::MechanicId;
pub use crate::mc::rng::McRng;
pub use crate::mc::state::McState;

use crate::EvalCtx;
use crate::resolve::ResolvedAttack;
use crate::stats::EffectRef;

/// One mechanic of a build resolved for one attack: the kind with its typed parameters and its magnitude (the
/// aggregated `xh.<k>` or `mech.<n>` slot, or the effect value for kinds without a slot).
#[derive(Debug, Clone, PartialEq)]
pub struct MechInst {
    /// The mechanic and its parameters.
    pub mechanic: Mechanic,
    /// The magnitude (chance, p_xh, bonus per count …; domain-model.md §11.2).
    pub magnitude: f64,
    /// The effect it came from.
    pub source: EffectRef,
    /// The effect's key (derived MC ids `"eff/<effect key>"`, 08 E-8.16).
    pub key: EffectKey,
    /// The hashed MC ids of [`MechanicMc::mechanic_ids`] for this instance, in the same order (FNV-1a of each
    /// string, 08 E-8.16), so `mc_instance` draws without hashing. Filled where the instance is built, from the
    /// strings L0 registered in `CompiledScenario.mech_ids`. Addition to engine.md §2.1 (CONTRACTS.md T02a).
    pub ids: SmallVec<[MechanicId; 2]>,
}

/// The EV side of a mechanic kind (engine.md §2.1).
pub trait MechanicEv {
    /// Expected contribution of the mechanic to one instance at one state on `view` (E-8.9 terms, proc masses,
    /// extra damage): `out` is the instance's expected outcome (tier distribution with the conditional crit factors,
    /// expected procs, weapon side at crit factor 1). Derived instances go to `acc.hits` on the weapon side (the
    /// kernel applies the enemy side jointly with the parent's state, 08 E-8.9 step 1); DoT procs to `acc.dots`.
    #[allow(clippy::too_many_arguments)]
    fn ev_instance(
        &self,
        cx: &EvalCtx,
        inst: &MechInst,
        ra: &ResolvedAttack,
        st: &PointState,
        out: &InstanceOutcome,
        view: View,
        acc: &mut EvAcc,
    );
    /// Expected proc events the mechanic adds for the instance `out` of one proc event e (feeds h_{s,e}, m_{s,e};
    /// 08 E-8.2), added into `procs`.
    fn ev_procs(
        &self,
        cx: &EvalCtx,
        inst: &MechInst,
        ra: &ResolvedAttack,
        st: &PointState,
        out: &InstanceOutcome,
        procs: &mut ProcMass,
    );
}

/// The MC side of a mechanic kind (engine.md §2.1).
pub trait MechanicMc {
    /// Live effect on one instance (P4a of its shot, 03 §7.1; 08 E-8.18): draws through `rng` with the mechanic's
    /// ids at the instance's anchor and may push derived events, derived instances (`sink.hits`), procs for P4b
    /// (`sink.procs`) and a factor on the instance's G (`sink.g_mult`). `out` is the instance's outcome after its own
    /// draws of 02 4.8 (rolled tier and c_k, delivered procs, weapon side at crit factor 1).
    ///
    /// The simulator calls every mechanic of the instance once per instance, never once per view (08 E-8.18: "every
    /// damage instance is evaluated once per view of the pass" applies to the derived hits, which the simulator
    /// evaluates per view), so the per-(anchor, mechanic) ordinals of E-8.16 advance once; on an enemy with
    /// overguard each of the two view timelines of pass V (08 §4.5) calls it with its own [`McState`] and draw
    /// source, which see the same keyed draws.
    #[allow(clippy::too_many_arguments)]
    fn mc_instance(
        &self,
        cx: &EvalCtx,
        inst: &MechInst,
        ra: &ResolvedAttack,
        st: &mut McState,
        ev: &McEvent,
        out: &InstanceOutcome,
        rng: &mut McRng,
        sink: &mut McSink,
    );
    /// The MC mechanic id **strings** the instance draws with (08 E-8.16: fixed ids, or derived ids such as
    /// `"eff/<effect key>"`, `"xh/<source>/<name>"`). L0 registers each in `CompiledScenario.mech_ids`
    /// ([`crate::mc::MechIdTable::insert`]), which hashes it and fails with `EngineError::MechanicIdCollision` when
    /// two distinct strings share a hash (engine.md §8.2, E-8.17): the check needs the strings, not the hashes.
    fn mechanic_ids(&self, inst: &MechInst) -> SmallVec<[Box<str>; 2]>;
}

/// A mechanic kind: both evaluator sides, shareable across MC threads.
pub trait MechanicImpl: MechanicEv + MechanicMc + Sync {}

/// The implementation of a mechanic's kind (one static unit value per kind of domain-model.md §11.2).
pub fn mechanic_impl(kind: &Mechanic) -> &'static dyn MechanicImpl {
    match kind.kind() {
        MechanicKind::ExtraHit => &extra_hit::ExtraHit,
        MechanicKind::ProcOnEvent => &proc_on_event::ProcOnEvent,
        MechanicKind::ConvertDamage => &convert_damage::ConvertDamage,
        MechanicKind::RepeatHit => &repeat_hit::RepeatHit,
        MechanicKind::FlatDamageOnProc => &flat_damage_on_proc::FlatDamageOnProc,
        MechanicKind::ProcAfterTicks => &proc_after_ticks::ProcAfterTicks,
        MechanicKind::CounterBonus => &counter_bonus::CounterBonus,
        MechanicKind::RandomMultiplier => &random_multiplier::RandomMultiplier,
        MechanicKind::SpawnAttack => &spawn_attack::SpawnAttack,
        MechanicKind::StackPresentStatuses => &stack_present_statuses::StackPresentStatuses,
    }
}

/// Implements [`MechanicEv`], [`MechanicMc`] and [`MechanicImpl`] for a kind's unit type with bodies of the owning
/// task.
macro_rules! mechanic_skeleton {
    ($ty:ident, $task:literal) => {
        impl $crate::mechanics::MechanicEv for $ty {
            fn ev_instance(
                &self,
                cx: &$crate::EvalCtx,
                inst: &$crate::mechanics::MechInst,
                ra: &$crate::resolve::ResolvedAttack,
                st: &$crate::mechanics::PointState,
                out: &$crate::mechanics::InstanceOutcome,
                view: wfd_model::View,
                acc: &mut $crate::mechanics::EvAcc,
            ) {
                let _ = (cx, inst, ra, st, out, view, acc);
                todo!($task)
            }
            fn ev_procs(
                &self,
                cx: &$crate::EvalCtx,
                inst: &$crate::mechanics::MechInst,
                ra: &$crate::resolve::ResolvedAttack,
                st: &$crate::mechanics::PointState,
                out: &$crate::mechanics::InstanceOutcome,
                procs: &mut $crate::mechanics::ProcMass,
            ) {
                let _ = (cx, inst, ra, st, out, procs);
                todo!($task)
            }
        }
        impl $crate::mechanics::MechanicMc for $ty {
            fn mc_instance(
                &self,
                cx: &$crate::EvalCtx,
                inst: &$crate::mechanics::MechInst,
                ra: &$crate::resolve::ResolvedAttack,
                st: &mut $crate::mechanics::McState,
                ev: &$crate::mechanics::McEvent,
                out: &$crate::mechanics::InstanceOutcome,
                rng: &mut $crate::mechanics::McRng,
                sink: &mut $crate::mechanics::McSink,
            ) {
                let _ = (cx, inst, ra, st, ev, out, rng, sink);
                todo!($task)
            }
            fn mechanic_ids(
                &self,
                inst: &$crate::mechanics::MechInst,
            ) -> smallvec::SmallVec<[Box<str>; 2]> {
                let _ = inst;
                todo!($task)
            }
        }
        impl $crate::mechanics::MechanicImpl for $ty {}
    };
}
use mechanic_skeleton;

#[cfg(test)]
mod tests {
    use super::*;
    use wfd_model::{ProcAfterTicksParams, StatusSet, TickAdds};

    #[test]
    fn dispatch_covers_a_kind_without_panicking() {
        // `mechanic_impl` is an exhaustive match over `MechanicKind`; a new kind fails to compile here first.
        let m = Mechanic::ProcAfterTicks(ProcAfterTicksParams {
            combined: StatusSet::GAS,
            stacks_at_least: 10,
            adds: TickAdds::default(),
        });
        let _implementation: &'static dyn MechanicImpl = mechanic_impl(&m);
    }
}
