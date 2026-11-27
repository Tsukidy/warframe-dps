//! L1: one candidate pick compiled in a scenario (engine.md §3; domain-model.md §18, §12.2 T-02 table). Built by
//! `Engine::compile_pick`. Owner: T06.

use smallvec::SmallVec;
use wfd_model::SetId;

use crate::compile::monotone::Reader;
use crate::stats::{SparseContrib, StatMask};

/// One item (or pick) compiled for the objective attacks of a scenario (domain-model.md §18).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CompiledItem {
    /// One contribution per objective attack (scope resolved per attack).
    pub per_attack: SmallVec<[SparseContrib; 2]>,
    /// Mod drain at the picked rank (09 §5).
    pub drain: i8,
    /// Incompatibility bits within the candidate pool (09 §4, D-093).
    pub incompat_mask: u64,
    /// Set membership (09 E-9.40).
    pub set: Option<SetId>,
    /// Slots this item can change (dominance, relevance).
    pub touched: StatMask,
    /// Touches an `Unknown` slot: a composition candidate (optimizer.md §3).
    pub has_unknown: bool,
    /// Readers the item holds (crit-tier, proc, schedule-mark or resolved-stat readers, domain-model.md §10), which
    /// `monotone_classes` turns into slot downgrades. Addition to domain-model.md §18 (CONTRACTS.md T02a).
    pub readers: SmallVec<[Reader; 1]>,
}
