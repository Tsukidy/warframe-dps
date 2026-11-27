//! `wfd-engine`: INTERIM file carried by T02d, not T02a's skeleton (CONTRACTS.md, "2026-11-27 · T02d").
//!
//! T02a writes this crate's skeleton in parallel with T02d, and `wfd-opt`'s frozen interface (work-breakdown.md
//! T02d) names four engine types. So that `wfd-opt` builds and tests on T02d's branch, this file declares exactly
//! those names with their documented shapes and nothing else. **On merge, T02a's version of this file replaces this
//! one entirely** (the owner's file wins, work-breakdown.md §8); `wfd-opt` and `wfd-server` use only `Engine`,
//! `Kernel`, `CancelToken` (`new`, `cancel`, `is_cancelled`, `Clone`, `Default`, `Debug`) and `Score` (its four
//! public fields, `WORST`, `Debug` and `Ord`) from this crate, all re-exported at the root of T02a's version.

use std::cmp::Ordering;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering as AtomicOrdering;
use wfd_model::Bundle;

/// The engine (engine.md §1): `pub struct Engine<'b> { bundle: &'b Bundle }`. Interim declaration (T02d).
pub struct Engine<'b> {
    _bundle: PhantomData<&'b Bundle>,
}

/// The optimizer-facing scoring kernel (engine.md §7.9). Interim declaration (T02d).
pub struct Kernel<'a> {
    _cs: PhantomData<&'a ()>,
}

/// A shared cancellation flag (engine.md §1 `CancelToken(Arc<AtomicBool>)`); clones share the flag. Interim
/// declaration (T02d).
#[derive(Debug, Clone, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    /// A token that is not cancelled.
    pub fn new() -> Self {
        CancelToken::default()
    }

    /// Requests cancellation; every clone sees it.
    pub fn cancel(&self) {
        self.0.store(true, AtomicOrdering::Relaxed);
    }

    /// Whether cancellation was requested.
    pub fn is_cancelled(&self) -> bool {
        self.0.load(AtomicOrdering::Relaxed)
    }
}

/// Ordered score key (engine.md §7.9): lower `band` is better, then higher `primary`, `tie`, `tie2` (f64 compared
/// with `total_cmp`); the greater key is the better build. Interim declaration (T02d).
#[derive(Debug, Clone, Copy)]
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

impl Score {
    /// The worst key: band 2, every float −∞ (the initial incumbent).
    pub const WORST: Score = Score {
        band: 2,
        primary: f64::NEG_INFINITY,
        tie: f64::NEG_INFINITY,
        tie2: f64::NEG_INFINITY,
    };
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
