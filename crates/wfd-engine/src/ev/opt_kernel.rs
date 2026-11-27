//! The optimizer-facing kernel (engine.md §7.9; optimizer.md §4-§7): scores of build points at a fidelity, and upper
//! bounds of every completion of a partial build. No heap allocation in `score` and `bound` after warm-up. Owner:
//! T15c.

use smallvec::SmallVec;
use wfd_model::{Attack, AttackIdx, View};

use crate::compile::CompiledScenario;
use crate::elements::ElemOutcome;
use crate::ev::score::Score;
use crate::stats::{Coeff, MechSet, StatVec};

/// The optimizer metric (08 §10). The model's DTO enum, re-exported: it includes `TtkNoarmor` (ruling C-28) and
/// spells the in-fight hit metric `HitInfight` (engine.md §7.9 wrote `HitInFight`).
pub use wfd_model::api::Metric;

/// What the optimizer maximizes (engine.md §7.9).
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectiveSpec {
    /// The metric.
    pub metric: Metric,
    /// The view (`Layered` for the TTK metrics).
    pub view: View,
    /// Weighted objective attacks.
    pub attacks: SmallVec<[(AttackIdx, f64); 2]>,
    /// Search and ranking window (s; default T-01).
    pub window: f64,
}

/// EV fidelities (engine.md §7.2). `Bound` is [`Kernel::bound`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Fidelity {
    /// One point at the steady state (+ the Heat window mean).
    Steady,
    /// Gauss-Legendre over segments; ≤ 0.5 % from `WindowExact`, else falls back to it.
    WindowFast,
    /// Every shot and every DoT tick in [0, t_win]: the definition of `dps_window`.
    WindowExact,
}

/// A build as the optimizer holds it: borrowed compiled parts (engine.md §7.9).
#[derive(Debug, Clone, Copy)]
pub struct BuildPoint<'p> {
    /// One stat vector per objective attack.
    pub sv: &'p [StatVec],
    /// The element outcome.
    pub elements: &'p ElemOutcome,
    /// Dynamic terms.
    pub coeffs: &'p [Coeff],
    /// Mechanics.
    pub mech: &'p MechSet,
}

/// Static per-attack data the kernel reuses across build points (attack records, schedule skeletons). T15c.
#[derive(Debug, Clone, Default)]
pub struct AttackStatics<'a> {
    /// The objective attacks and their records.
    pub attacks: SmallVec<[(AttackIdx, &'a Attack); 2]>,
}

/// The scoring kernel of one compiled scenario and objective (engine.md §7.9).
#[derive(Debug, Clone)]
pub struct Kernel<'a> {
    cs: &'a CompiledScenario,
    objective: &'a ObjectiveSpec,
    statics: AttackStatics<'a>,
}

impl<'a> Kernel<'a> {
    /// A kernel over prepared statics (`Engine::kernel` builds them).
    pub fn new(
        cs: &'a CompiledScenario,
        objective: &'a ObjectiveSpec,
        statics: AttackStatics<'a>,
    ) -> Self {
        Kernel {
            cs,
            objective,
            statics,
        }
    }

    /// The compiled scenario.
    pub fn scenario(&self) -> &'a CompiledScenario {
        self.cs
    }

    /// The objective.
    pub fn objective(&self) -> &'a ObjectiveSpec {
        self.objective
    }

    /// The per-attack statics.
    pub fn statics(&self) -> &AttackStatics<'a> {
        &self.statics
    }

    /// The score of `x` at fidelity `fid` (08 §10, E-8.26 ordering). T15c.
    pub fn score(&self, x: &BuildPoint, fid: Fidelity, ws: &mut Workspace) -> Score {
        let _ = (x, fid, ws);
        todo!("T15c")
    }

    /// Upper bound, in the order of [`Score`], of `score(y, WindowExact)` for every y whose Inc slots are ≤ `upper`'s
    /// and whose Unknown slots, elements and mechanics equal `upper`'s (optimizer.md §4; engine.md §7.9 gives the
    /// argument for TTK). T15c.
    pub fn bound(&self, upper: &BuildPoint, ws: &mut Workspace) -> Score {
        let _ = (upper, ws);
        todo!("T15c")
    }
}

/// Per-thread scratch arrays of the kernel; no heap allocation in `score` and `bound` after warm-up (engine.md §13).
#[derive(Debug, Clone, Default)]
pub struct Workspace {
    scratch: Vec<f64>,
}

impl Workspace {
    /// An empty workspace (grows on first use).
    pub fn new() -> Self {
        Workspace::default()
    }

    /// Scratch space of at least `n` values (for the owning task's walks).
    pub fn scratch(&mut self, n: usize) -> &mut [f64] {
        if self.scratch.len() < n {
            self.scratch.resize(n, 0.0);
        }
        &mut self.scratch[..n]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_grows_and_reuses() {
        let mut ws = Workspace::new();
        assert_eq!(ws.scratch(4).len(), 4);
        ws.scratch(4)[0] = 1.5;
        assert_eq!(ws.scratch(2).len(), 2);
        assert!((ws.scratch(2)[0] - 1.5).abs() < f64::EPSILON);
    }
}
