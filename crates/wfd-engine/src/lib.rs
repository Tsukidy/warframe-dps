//! `wfd-engine`: INTERIM file carried by T02d, not T02a's skeleton (CONTRACTS.md, "2026-11-27 · T02d").
//!
//! T02a writes this crate's skeleton in parallel with T02d, and `wfd-opt`'s frozen signatures (work-breakdown.md
//! T02d) name three engine types. So that `wfd-opt` builds and tests on T02d's branch, this file declares exactly
//! those names with their documented shapes and nothing else. **On merge, T02a's version of this file replaces this
//! one entirely** (the owner's file wins, work-breakdown.md §8); `wfd-opt` and `wfd-server` use only `Engine`,
//! `Kernel` and `Score` (its four public fields and `Ord`) from this crate.

use std::cmp::Ordering;
use std::marker::PhantomData;
use wfd_model::Bundle;

/// The engine (engine.md §1): `pub struct Engine<'b> { bundle: &'b Bundle }`. Interim declaration (T02d).
pub struct Engine<'b> {
    _bundle: PhantomData<&'b Bundle>,
}

/// The optimizer-facing scoring kernel (engine.md §7.9). Interim declaration (T02d).
pub struct Kernel<'a> {
    _cs: PhantomData<&'a ()>,
}

/// Ordered score key (engine.md §7.9): lower `band` is better, then higher `primary`, `tie`, `tie2` (f64 compared
/// with `total_cmp`); the greater key is the better build. Interim declaration (T02d).
#[derive(Clone, Copy)]
pub struct Score {
    /// 0 killed or DPS/hit metric, 1 extrapolated, 2 never (08 E-8.26).
    pub band: u8,
    /// Metric value, or −T_EV.
    pub primary: f64,
    /// First tie-break.
    pub tie: f64,
    /// Second tie-break.
    pub tie2: f64,
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .band
            .cmp(&self.band)
            .then_with(|| self.primary.total_cmp(&other.primary))
            .then_with(|| self.tie.total_cmp(&other.tie))
            .then_with(|| self.tie2.total_cmp(&other.tie2))
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Score {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Score {}
