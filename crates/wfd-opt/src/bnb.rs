//! Branch-and-bound over the monotone remainder of the main slots (optimizer.md §5.2).
//!
//! Owner: T32. `S0` is the fixed part (context and composition set, exact i64); the pool is compacted to the active
//! slots and sorted by marginal value at S0; a suffix top-r table makes the relaxation O(active slots) per node; nodes
//! are pruned when `Kernel::bound(S + TOP[i][k − depth]) < θ` or when drain plus the suffix minimum drain exceeds the
//! capacity; leaves are scored with `WindowFast` (`WindowExact` when `search.certify`), and every leaf whose key is
//! ≥ θ = relax(s_best, 4ε) enters the keep-set. Children are visited in descending marginal value. TTK keys
//! (`(band, −T_EV, κ, −ammo_to_kill)`, E-8.26) are compared across bands. Workers check the job control every
//! [`crate::CANCEL_CHECK_NODES`] nodes.

use crate::batch::BatchCtx;
use crate::problem::MainProblem;
use crate::{JobControl, Score};
use wfd_engine::Kernel;

/// Solves one main-slot problem from the threshold and budget of its batch (optimizer.md §5.2, §8). Deterministic:
/// the nodes visited depend only on `problem` and `batch`, never on other items or on thread timing; a cancelled job or
/// an exhausted budget returns what was found with the unexplored subtree's bound as [`MainSolution::gap`].
///
/// Owner: T32.
pub fn solve_main(
    kernel: &Kernel,
    problem: &MainProblem,
    batch: &BatchCtx,
    ctl: &JobControl,
) -> MainSolution {
    let _ = (kernel, problem, batch, ctl);
    todo!("T32: optimizer.md §5.2, §8")
}

/// The result of [`solve_main`] for one work item.
#[derive(Debug, Clone, PartialEq)]
pub struct MainSolution {
    /// The keep-set: leaves whose key is ≥ the item's final θ, in the order they were found.
    pub keep: Vec<KeptLeaf>,
    /// s_best: the best leaf key found; `None` when no leaf was reached.
    pub best: Option<Score>,
    /// Bound of the subtree left unexplored (budget exhausted or job stopped); `None` when the search finished.
    pub gap: Option<Score>,
    /// Node counters of the item.
    pub stats: NodeStats,
}

/// A leaf of the keep-set.
#[derive(Debug, Clone, PartialEq)]
pub struct KeptLeaf {
    /// Its key (`WindowFast`, or `WindowExact` under `certify`).
    pub score: Score,
    /// The chosen monotone candidates, as indices into [`MainProblem::pool`], ascending.
    pub picks: Vec<u32>,
}

/// Node counters (`SearchStats` of `wfd_model::api`, optimizer.md §1).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NodeStats {
    /// Leaves scored.
    pub evaluated: u64,
    /// Bounds computed.
    pub bounded: u64,
    /// Nodes pruned.
    pub pruned: u64,
    /// Leaves reached.
    pub leaves: u64,
}

impl NodeStats {
    /// Adds another item's counters (merged in canonical item order, §8; addition is order-independent).
    pub fn add(&mut self, other: &NodeStats) {
        self.evaluated += other.evaluated;
        self.bounded += other.bounded;
        self.pruned += other.pruned;
        self.leaves += other.leaves;
    }
}

#[cfg(test)]
mod tests {
    use super::NodeStats;

    #[test]
    fn node_stats_add() {
        let mut a = NodeStats {
            evaluated: 1,
            bounded: 2,
            pruned: 3,
            leaves: 4,
        };
        a.add(&NodeStats {
            evaluated: 10,
            bounded: 20,
            pruned: 30,
            leaves: 40,
        });
        assert_eq!(
            a,
            NodeStats {
                evaluated: 11,
                bounded: 22,
                pruned: 33,
                leaves: 44
            }
        );
    }

    #[test]
    #[ignore = "T32: brute-force exactness on 100 random small problems, certify and keep-set rule (testing.md §6)"]
    fn bnb_matches_brute_force() {
        todo!("T32")
    }
}
