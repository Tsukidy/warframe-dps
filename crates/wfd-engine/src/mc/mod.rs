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

pub use events::{Anchor, McEvent, McEventKind, McSink, Phase, SubKey};
pub use keys::{FIXED_MECHANIC_IDS, MechIdTable, MechanicId, rng_mech};
pub use rng::{McRng, OrdinalTable, philox4x32_10};
pub use state::McState;
pub use summary::{OracleVerdict, oracle_compare, oracle_compare_q, summary};

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

/// Which run `Engine::timeline` replays (08 §8.9; contract review API-1, C-32): the seed, the run index and the hit
/// limit of `TimelineRequest` after the server's defaults (seed: the request's, else the scenario's `eval.seed`, which
/// must not be `"auto"`; run 0; limit 2000).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimelineReq {
    /// The 08 E-8.16 key: the seed of the MC request whose run is replayed (T-29).
    pub seed: u64,
    /// Run index r (counter word c0).
    pub run: u32,
    /// Maximum number of parent hits recorded.
    pub limit: u32,
}
