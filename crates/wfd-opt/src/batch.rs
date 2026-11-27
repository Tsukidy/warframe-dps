//! Deterministic batches (optimizer.md §8).
//!
//! Owner: T32. Work items are generated in a canonical order (composition outcome × first two DFS levels for the main
//! search, options of a group for context ascent, finalists for MC) and processed in batches of [`BATCH_SIZE`] items
//! whose items run in parallel. θ is published only at batch barriers: every item of batch b starts from θ_b, computed
//! from the completed results of batches 0 .. b − 1 merged in canonical item order, so which nodes an item visits
//! depends only on the item and θ_b. With a node budget N, each item gets a deterministic share (N divided over the
//! items of the batch in proportion to their subtree-size estimates, remainders in canonical order; budget an item does
//! not use goes to the next batch); an item that exhausts its share stops and records its subtree's bound as the gap.

use crate::Score;

/// W: items per batch, independent of the thread count (§8).
pub const BATCH_SIZE: usize = 64;

/// What a work item of a batch starts from (optimizer.md §8, work-breakdown.md T02d).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BatchCtx {
    /// θ_b: the threshold published at the barrier before this batch (§5.2 `θ = relax(s_best, 4ε)`); nodes whose
    /// `Kernel::bound` is below it are pruned.
    pub theta: Score,
    /// The item's deterministic node budget; `None` = unbounded (no `node_budget`).
    pub budget: Option<u64>,
}

impl BatchCtx {
    /// A context without a threshold: θ = [`Score::WORST`] (band 2, every float −∞; the engine's initial incumbent),
    /// which is at most every key the kernel returns (bands 0-2, no NaN), so it prunes nothing (§5.2 prunes
    /// `bound < θ`). The first batch of a search without a warm start.
    pub fn without_threshold(budget: Option<u64>) -> Self {
        BatchCtx {
            theta: Score::WORST,
            budget,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "T32: node_budget results identical for 1, 2 and 8 threads (optimizer.md §8)"]
    fn batch_results_identical_across_thread_counts() {
        todo!("T32")
    }
}
