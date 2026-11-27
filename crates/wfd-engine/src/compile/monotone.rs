//! Per-scenario slot classes (engine.md §3; domain-model.md §10; optimizer.md §4). The declared class of a stat row
//! holds only while nothing reads the slot's quantity in a non-monotone way: this module collects every *reader* of
//! the scenario and of the candidate pool and downgrades the slots feeding the read quantity to `Unknown`, unless the
//! reader is monotone (upward-closed predicate and non-negative gated value). Data-independent code in one function;
//! one fixture per reader kind (`tests/monotone_readers.rs`, T15c; testing.md §2.3). Owner: T06.

use smallvec::SmallVec;
use wfd_model::{Monotone, N_STATS, ResolvedStat};

use crate::compile::pick::CompiledItem;
use crate::compile::scenario::CompiledScenario;
use crate::stats::EffectRef;

/// A trigger, gate, condition, mechanic parameter or `per` scalar that reads an outcome a slot feeds
/// (domain-model.md §10 reader table).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reader {
    /// The quantity read.
    pub reads: ReadQuantity,
    /// Upward-closed predicate in the read quantity and non-negative gated value: causes no downgrade.
    pub monotone: bool,
    /// The effect that reads.
    pub source: EffectRef,
}

/// The quantities a reader can read (domain-model.md §10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReadQuantity {
    /// The crit-tier outcome of an instance (downgrades `cc_*`, `tier_raise`; and `cd_*` when `crit_damage`).
    CritTier {
        /// The reader also reads crit damage.
        crit_damage: bool,
    },
    /// The proc outcome of an instance (downgrades `sc_*`, `status_dur`, `forced.*`).
    ProcOutcome,
    /// Schedule marks: first or last shot, reload, reloaded fraction (downgrades `magazine`, `magazine_flat`,
    /// `ammo_eff`, `fire_rate`, `reload_speed`, `charge_rate`).
    ScheduleMark,
    /// A resolved stat through `Per::Resolved` or `Cond::Attack` (downgrades the slots that feed it).
    Resolved(ResolvedStat),
}

/// The slot classes of a scenario given the candidate pool, and the readers held by candidates (the optimizer's
/// coupled candidates). Readers held by `cs.fixed` downgrade for the whole search (domain-model.md §10). T06.
pub fn monotone_classes(
    cs: &CompiledScenario,
    pool: &[CompiledItem],
) -> ([Monotone; N_STATS], SmallVec<[Reader; 4]>) {
    let _ = (cs, pool);
    todo!("T06")
}
