//! L1: one candidate pick compiled in a scenario (engine.md §3; domain-model.md §18, §12.2 T-02 table). Built by
//! `Engine::compile_pick`. Per-rank values of effect values, stack caps, durations and chances are 09 E-9.13 through
//! `wfd_model::RankCurve::value` (the one evaluator, also used by `wfd-data` for TV-09-06; contract review API-16).
//! A conditional effect that stays dynamic contributes the id of its `CompiledCondition` (`compile::conditions`) to
//! `SparseContrib.conditions` and its coefficients on `StateVar::CondStacks(id)` / `StateVar::Switch(id)` (contract
//! review ENG-2). Owner: T06.

use smallvec::SmallVec;
use wfd_model::SetId;

use crate::compile::monotone::Reader;
use crate::stats::{SparseContrib, StatMask};

/// One item (or pick) compiled for the objective attacks of a scenario (domain-model.md §18).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CompiledItem {
    /// One contribution per build attack (scope resolved per attack, `CompiledBuild.attacks` order).
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
