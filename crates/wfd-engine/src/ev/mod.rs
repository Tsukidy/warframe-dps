//! The expected-value evaluator (engine.md §7; spec 00 §4.2, 08 §5, E-8.2-E-8.15, assumptions A-1 to A-9). Where this
//! module is shorter than spec 08, 08 wins. Owners: T15a (`mod`, `kernel`, `poisson`, `joint`, `gates`, `distinct`,
//! `dot`, `steady`, `cond_stacks`, `hit`, `score`), T15b (`window_exact`, `heat`, `group`, `series`), T15c (`bound`,
//! `window_fast`, `ttk`, `opt_kernel`).

pub mod bound;
pub mod cond_stacks;
pub mod distinct;
pub mod dot;
pub mod gates;
pub mod group;
pub mod heat;
pub mod hit;
pub mod joint;
pub mod kernel;
pub mod opt_kernel;
pub mod poisson;
pub mod score;
pub mod series;
pub mod steady;
pub mod ttk;
pub mod window_exact;
pub mod window_fast;

pub use kernel::{DotReg, EvAcc, PointOut, PointState, ProcMass};
pub use opt_kernel::{
    AttackStatics, BuildPoint, Fidelity, Kernel, Metric, ObjectiveSpec, Workspace,
};
pub use poisson::capped_poisson_mean;
pub use score::Score;

use wfd_model::View;

use crate::compile::CompiledBuild;

/// What one EV evaluation of a unit computes (`Engine::ev`, engine.md §1, §7). The compiled build is part of the
/// request (an addition to engine.md §1, CONTRACTS.md T02a): the evaluator resolves (L3) and schedules the unit's
/// attacks from it.
#[derive(Debug, Clone, Copy)]
pub struct EvRequest<'r> {
    /// The compiled build (L2).
    pub build: &'r CompiledBuild,
    /// Fidelity of the window metrics (the calculator uses `WindowExact`, engine.md §7.8).
    pub fidelity: Fidelity,
    /// Views to evaluate (each present view of the enemy; `Layered` adds the EV TTK of E-8.21).
    pub views: &'r [View],
    /// Width of the series bins (s; 08 §8.1).
    pub series_bin_s: f64,
    /// Per-source breakdowns (08 §8.2).
    pub breakdown: bool,
}
