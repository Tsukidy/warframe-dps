//! Mechanic kinds (domain-model.md §11.2; engine.md §2.1, frozen by T02a). Every kind implements the EV trait
//! [`MechanicEv`] and the MC trait [`MechanicMc`]; this module (T14) writes the implementations and the evaluators
//! (T15a-T15c, T16) only call them, through [`mechanic_impl`]. The state types the traits name belong to the
//! evaluators and are re-exported here: [`PointState`], [`EvAcc`], [`ProcMass`] (`ev`, T15a) and [`McState`],
//! [`McEvent`], [`McRng`], [`McSink`], [`MechanicId`] (`mc`, T16). Owner: T14.

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
pub use crate::mc::events::{Anchor, McEvent, McEventKind, McSink, Phase, SubKey};
pub use crate::mc::keys::MechanicId;
pub use crate::mc::rng::McRng;
pub use crate::mc::state::McState;

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
    /// Expected contribution of the mechanic to one instance at one state (E-8.9 terms, proc masses, extra damage).
    fn ev_instance(
        &self,
        inst: &MechInst,
        ra: &ResolvedAttack,
        st: &PointState,
        view: View,
        acc: &mut EvAcc,
    );
    /// Expected proc events the mechanic adds for one proc event e (feeds h_{s,e}, m_{s,e}; 08 E-8.2).
    fn ev_procs(&self, inst: &MechInst, ra: &ResolvedAttack, st: &PointState, out: &mut ProcMass);
}

/// The MC side of a mechanic kind (engine.md §2.1).
pub trait MechanicMc {
    /// Live effect on one instance: draws through `rng` with the mechanic's id, may push derived events.
    fn mc_instance(
        &self,
        inst: &MechInst,
        ra: &ResolvedAttack,
        st: &mut McState,
        ev: &McEvent,
        rng: &mut McRng,
        out: &mut McSink,
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
                inst: &$crate::mechanics::MechInst,
                ra: &$crate::resolve::ResolvedAttack,
                st: &$crate::mechanics::PointState,
                view: wfd_model::View,
                acc: &mut $crate::mechanics::EvAcc,
            ) {
                let _ = (inst, ra, st, view, acc);
                todo!($task)
            }
            fn ev_procs(
                &self,
                inst: &$crate::mechanics::MechInst,
                ra: &$crate::resolve::ResolvedAttack,
                st: &$crate::mechanics::PointState,
                out: &mut $crate::mechanics::ProcMass,
            ) {
                let _ = (inst, ra, st, out);
                todo!($task)
            }
        }
        impl $crate::mechanics::MechanicMc for $ty {
            fn mc_instance(
                &self,
                inst: &$crate::mechanics::MechInst,
                ra: &$crate::resolve::ResolvedAttack,
                st: &mut $crate::mechanics::McState,
                ev: &$crate::mechanics::McEvent,
                rng: &mut $crate::mechanics::McRng,
                out: &mut $crate::mechanics::McSink,
            ) {
                let _ = (inst, ra, st, ev, rng, out);
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
