//! L2: a whole build (engine.md §3): Σ contributions + finalizers (set-piece counts, shard counts, Strength,
//! `Per::Resolved` terms in stage order with cycle rejection), element order (domain-model.md §13). Locks
//! (`lock_multishot`, `lock_fire_rate`) are applied at L3. Built by `Engine::compile_build`. Owner: T06.

use smallvec::SmallVec;
use wfd_model::AttackIdx;

use crate::elements::ElemOutcome;
use crate::ev::BuildPoint;
use crate::stats::{Coeff, MechSet, SourceIndex, StatVec};

/// A build compiled in a scenario (engine.md §3).
///
/// Every per-attack part is indexed by the build attack k: `attacks[k]` is the attack of `sv[k]`, `elements[k]`,
/// `coeffs[k]` and `mechanics[k]`. The build attacks are the scenario's objective attacks
/// (`ObjectiveSpec::attacks`, in that order) when it has an objective, else every attack of the weapon in
/// `AttackIdx` order (the calculator). Element contributions, dynamic terms and mechanics are scoped per attack like
/// the constants (`CompiledItem.per_attack`; 01 Table 4.10-B and E-1.13: Sentient Incision and Chromatic Blade skip
/// the slams; heavy-only, slide-only and Tennokai-only terms; `counter_bonus.applies_to`), so one attack is never
/// evaluated with another's element outcome or mechanics (contract review ENG-7).
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledBuild {
    /// The build attacks (addition, contract review ENG-7).
    pub attacks: SmallVec<[AttackIdx; 2]>,
    /// One stat vector per build attack.
    pub sv: SmallVec<[StatVec; 2]>,
    /// The element outcome (01 §4.11) per build attack.
    pub elements: SmallVec<[ElemOutcome; 2]>,
    /// Dynamic terms per build attack.
    pub coeffs: SmallVec<[SmallVec<[Coeff; 4]>; 2]>,
    /// Mechanic instances per build attack.
    pub mechanics: SmallVec<[MechSet; 2]>,
    /// The ids of the build's dynamic conditions (`CompiledScenario.conditions`), ascending: the union of the
    /// `SparseContrib.conditions` of every contribution, scenario-fixed sources included. Position p is the index of
    /// the per-condition arrays of the EV state (`PointState.cond_stacks`, `.switch_on`); at most
    /// `ev::kernel::MAX_COND_VARS` (L2 rejects more with `EngineError::UnsupportedMechanic`). Addition, contract
    /// review ENG-2.
    pub conditions: SmallVec<[u16; 4]>,
    /// Effect sources per slot, for traces and for T17's count of active CDM0+ entries (02 E-2.5; [`SourceIndex`]).
    /// L3 does not read it (contract review ENG-R1, pass 3).
    pub sources: SourceIndex,
}

impl CompiledBuild {
    /// The build as a borrowed [`BuildPoint`] (the form `Engine::resolve_point` and the optimizer's kernel read;
    /// contract review ENG-8).
    pub fn point(&self) -> BuildPoint<'_> {
        BuildPoint {
            attacks: &self.attacks,
            sv: &self.sv,
            elements: &self.elements,
            coeffs: &self.coeffs,
            mech: &self.mechanics,
            conditions: &self.conditions,
        }
    }

    /// The build attack index k of `attack`, if the build holds it.
    pub fn position(&self, attack: AttackIdx) -> Option<usize> {
        self.attacks.iter().position(|a| *a == attack)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_borrows_every_per_attack_part() {
        let cb = CompiledBuild {
            attacks: smallvec::smallvec![AttackIdx(0), AttackIdx(3)],
            sv: smallvec::smallvec![StatVec::ZERO, StatVec::ZERO],
            elements: smallvec::smallvec![ElemOutcome::default(), ElemOutcome::default()],
            coeffs: smallvec::smallvec![SmallVec::new(), SmallVec::new()],
            mechanics: smallvec::smallvec![MechSet::default(), MechSet::default()],
            conditions: smallvec::smallvec![2, 5],
            sources: SourceIndex::default(),
        };
        let x = cb.point();
        assert_eq!(x.attacks.len(), x.sv.len());
        assert_eq!(x.elements.len(), 2);
        assert_eq!(x.conditions, &[2, 5]);
        assert_eq!(cb.position(AttackIdx(3)), Some(1));
        assert_eq!(cb.position(AttackIdx(1)), None);
    }
}
