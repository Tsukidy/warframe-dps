//! Cross-group local search and the per-mix pass (optimizer.md §6.2 steps 4-5).
//!
//! Owner: T33. From the best `context_beam` full builds, 1-swaps and 2-swaps across any two slots of any groups
//! (main ↔ exilus moves and element re-orderings included), with exact fixed-point deltas and `WindowFast` scoring,
//! until no move improves by more than 1e-9 relative; moves are deterministic. Best per damage mix: for the 15 mixes
//! with the highest root bounds, the main search reruns with a mix-local threshold, warm-started (`OptResult.per_mix`).
