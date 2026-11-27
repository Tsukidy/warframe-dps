//! Run state of one pass (08 §6.2; engine.md §8.1): target state (per-status stack lists with expiry times,
//! independent timers and replacement beyond the cap, CD-07; DoT instances with snapshotted seeds, CD-05; the shared
//! Heat timer, pool and strip ramp; Blast fuses; pools per layer) and attacker state (magazine, combo, sniper combo,
//! ramp, spool, condition-stack counters of 04 E-4.8, mechanic counters). Owner: T16.

use arrayvec::ArrayVec;
use wfd_model::N_ST;

use crate::enemy::Pools;
use crate::resolve::N_MECH;
use crate::status::ProcSnapshot;

/// Largest number of live stacks per status in the run state (caps are at most 10, 03 §6.1; external stacks fill
/// slots too).
pub const MAX_STACKS: usize = 16;

/// The state of one pass of one run.
#[derive(Debug, Clone, Default)]
pub struct McState {
    /// Current time (s).
    pub t: f64,
    /// Own stacks per status: expiry times (µs grid), oldest first.
    pub stacks: [ArrayVec<i64, MAX_STACKS>; N_ST],
    /// External stacks N_ext,s (T-43 / T-06).
    pub external: [u8; N_ST],
    /// Live DoT instances (slab; `None` = free slot).
    pub dots: Vec<Option<DotInst>>,
    /// Heat episode state.
    pub heat: HeatState,
    /// Live Blast fuses: expiry times (µs grid).
    pub blast_fuses: ArrayVec<i64, MAX_STACKS>,
    /// Frozen until (µs grid), when frozen.
    pub frozen_until: Option<i64>,
    /// Pools of the layered passes.
    pub pools: Pools,
    /// Ammo left in the magazine.
    pub magazine: f64,
    /// Melee combo counter.
    pub combo: f64,
    /// Sniper combo counter.
    pub sniper_combo: f64,
    /// Condition-stack counters (04 E-4.8), per condition-table index.
    pub cond_stacks: Vec<u16>,
    /// `counter_bonus` counters per `mech.<n>` slot.
    pub counters: [f64; N_MECH],
}

/// One live DoT instance (E-13).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DotInst {
    /// Its status.
    pub status: wfd_model::StatusType,
    /// Snapshotted weapon-side terms.
    pub snap: ProcSnapshot,
    /// Ticks still to land.
    pub ticks_left: u32,
}

/// The shared Heat timer and pool (03 §10).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct HeatState {
    /// The episode is live.
    pub live: bool,
    /// Episode end (µs grid).
    pub end: i64,
    /// Procs in the pool (seeds).
    pub pool: Vec<ProcSnapshot>,
    /// Strip counter k_h (0 … 4).
    pub strip: u8,
}
