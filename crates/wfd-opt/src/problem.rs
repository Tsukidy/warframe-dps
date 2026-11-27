//! Problem compilation (optimizer.md §2) and the main-slot problem handed to [`crate::solve_main`] (§5.2).
//!
//! Owner: T32. Steps of §2: (1) `engine.compile_scenario` once with the objective (view `auto` → armor when the enemy
//! has armor, else health); (2) positions and pools ([`crate::pool`]); (3) status filter: `unmodelled` and
//! `no_damage_effect` items are excluded (I-6), `draft` and `stale` items are kept and marked unless the rules say
//! otherwise; (4) rule filters act on effects, not whole items (`NO_FACTION`, `NO_ON_KILL`, `USE_WEAK_POINT`,
//! `ALLOW_NEXT_SHOT`); (5) L1 compile of every remaining candidate per objective attack; (6) relevance derived from the
//! engine; (7) dominance ([`crate::dominance`]). Every item left out is reported as an `ExcludedItem` with its
//! `opt.excluded.*` reason. Capacity follows `constraints.capacity` with the precedence of server-api.md §8.1
//! (`CapacityRule::effective`; CONTRACTS.md T01d).
//!
//! [`MainProblem`]'s name is frozen by [`crate::solve_main`]'s signature; its fields are T32's and may change with
//! the implementation (no other task builds one before T32 is accepted).

/// Width of the active-slot vectors carried in the DFS (optimizer.md §5.2 "active-slot compaction": only the 10-30
/// stat slots any pool candidate touches, as `[i64; 32]`).
pub const MAX_ACTIVE: usize = 32;

/// A fixed-point contribution over the active slots: AD-05 units (1e-9), index = position in the problem's active-slot
/// list. Prefix sums are bit-identical to a full recompute (§5.2).
pub type ActiveVec = [i64; MAX_ACTIVE];

/// Incompatibility bit set over the incompatibility groups of one main pool (§5.2 `incompat`, `group`).
pub type GroupMask = u128;

/// One main-slot search problem (optimizer.md §5.2): a fixed context and composition outcome, and the monotone
/// remainder of the 8 main slots to fill.
///
/// The engine-side base point and the active-slot map (engine.md §7.9 `BuildPoint`, from which `Kernel::bound` and
/// `Kernel::score` are called) are added by T32 with the engine types.
#[derive(Debug, Clone, PartialEq)]
pub struct MainProblem {
    /// k: main slots left free after the composition set.
    pub free_slots: u8,
    /// S0 = Σ contrib(context) + Σ contrib(composition set) over the active slots.
    pub fixed: ActiveVec,
    /// Groups already taken by the context, the composition set and locked items.
    pub fixed_groups: GroupMask,
    /// The monotone pool, sorted by marginal value at S0 (descending), ties in canonical order.
    pub pool: Vec<MainCandidate>,
    /// The capacity constraint; `None` under capacity mode `ignore` (spec 09 E-9.15).
    pub capacity: Option<DrainLimit>,
}

/// A monotone main candidate (optimizer.md §3, §5.2).
#[derive(Debug, Clone, PartialEq)]
pub struct MainCandidate {
    /// Its contribution over the active slots.
    pub contrib: ActiveVec,
    /// Groups it cannot be combined with.
    pub incompat: GroupMask,
    /// Its own groups (set into the DFS mask when it is taken).
    pub group: GroupMask,
    /// Drain after the best possible polarity (a lower bound, spec 09 §6).
    pub drain: i32,
    /// Index of the candidate in the job's compiled candidate list.
    pub source: u32,
}

/// Drain knapsack of the main slots (optimizer.md §5.2 "Capacity, polarity and Forma").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrainLimit {
    /// Capacity available to the main slots.
    pub cap: i32,
    /// Drain already committed by the context and the composition set.
    pub fixed_drain: i32,
}
