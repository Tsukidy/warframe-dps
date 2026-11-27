//! Finalists, Monte Carlo confirmation and diagnostics (optimizer.md §7).
//!
//! Owner: T33. The merged keep-set (every build above the final θ, plus the per-mix bests) is re-scored with
//! `WindowExact` (TTK: EV TTK); the top `keep` are the finalists, sorted by `(Score desc, canonical key asc)`; a
//! keep-set over 100,000 builds raises θ to the 100,000-th score and marks the tail heuristic. The top
//! `mc_confirm.finalists` run `runs` MC runs each with the same seed (common random numbers, 08 E-8.16) and are flagged
//! when they fail 08 E-8.24. Diagnostics of the best build: leave-one-out, next-best replacement per slot, condition
//! sensitivity (T-02 = `none`), and the capacity and Forma report (09 E-9.16) unless the mode is `ignore`.
