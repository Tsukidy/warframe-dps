//! L2: a whole build (engine.md §3): Σ contributions + finalizers (set-piece counts, shard counts, Strength,
//! `Per::Resolved` terms in stage order with cycle rejection), element order (domain-model.md §13). Locks
//! (`lock_multishot`, `lock_fire_rate`) are applied at L3. Built by `Engine::compile_build`. Owner: T06.

use smallvec::SmallVec;

use crate::elements::ElemOutcome;
use crate::stats::{Coeff, MechSet, SourceIndex, StatVec};

/// A build compiled in a scenario (engine.md §3).
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledBuild {
    /// One stat vector per objective attack (all attacks for the calculator).
    pub sv: SmallVec<[StatVec; 2]>,
    /// The element outcome (01 §4.11).
    pub elements: ElemOutcome,
    /// Dynamic terms of the build.
    pub coeffs: SmallVec<[Coeff; 4]>,
    /// Mechanic instances of the build.
    pub mechanics: MechSet,
    /// Effect sources per slot, for traces.
    pub sources: SourceIndex,
}
