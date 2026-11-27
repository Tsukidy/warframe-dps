//! The Monte Carlo simulator (engine.md §8; spec 08 §6, E-8.16-E-8.20, E-8.22, E-8.25): counter-based random numbers
//! keyed by (run, anchor, mechanic, ordinal), the event loop of E-8.18 with the phases of 03 §7.1, the passes V, L
//! and L0, statistics over runs, the timeline of one run and the parity rows. With feature `parallel` runs are
//! chunked on the rayon pool and reduced in run order (I-5). Owner: T16.

pub mod draw;
pub mod events;
pub mod keys;
pub mod parity;
pub mod passes;
pub mod rng;
pub mod sim;
pub mod state;
pub mod summary;
pub mod timeline;
pub mod ttk;

pub use events::{Anchor, McEvent, McEventKind, McSink, Phase};
pub use keys::{FIXED_MECHANIC_IDS, MechIdTable, MechanicId, rng_mech};
pub use rng::{McRng, OrdinalTable, philox4x32_10};
pub use state::McState;
pub use summary::{OracleVerdict, oracle_compare, summary};

use wfd_model::View;

use crate::compile::CompiledBuild;

/// What one MC evaluation of a unit computes (`Engine::mc`, engine.md §1, §8). The compiled build is part of the
/// request (an addition to engine.md §1, CONTRACTS.md T02a).
#[derive(Debug, Clone, Copy)]
pub struct McRequest<'r> {
    /// The compiled build (L2).
    pub build: &'r CompiledBuild,
    /// Number of runs (T-29).
    pub runs: u32,
    /// Seed (T-29).
    pub seed: u64,
    /// Views of pass V (the present non-layered views; `Layered` adds pass L).
    pub views: &'r [View],
    /// Width of the series bins (s).
    pub series_bin_s: f64,
    /// Parity rows of 08 §8.8 (pass L0 when A0 > 0).
    pub parity: bool,
    /// Per-source breakdowns.
    pub breakdown: bool,
}
