//! Finalists, Monte Carlo confirmation and diagnostics (optimizer.md §7).
//!
//! Owner: T33. The merged keep-set (every build above the final θ, plus the per-mix bests) is re-scored with
//! `WindowExact` (TTK: EV TTK); the top `keep` are the finalists, sorted by `(Score desc, canonical key asc)`; a
//! keep-set over 100,000 builds raises θ to the 100,000-th score and marks the tail heuristic. The top
//! `mc_confirm.finalists` run `runs` MC runs each with the same seed (common random numbers, 08 E-8.16) and are flagged
//! when they fail 08 E-8.24. Ranking after MC (08 §10; contract review API-10): when the metric is `dps_window` and the
//! flag fires for any finalist, the finalists are re-ranked by the MC mean (`opt.log.mc_rerank` with the gap classes),
//! and each keeps its EV rank in `RankedBuild.ev_rank`; for every other metric the flag is reported and the EV ranking
//! stays. A cancel during MC (`Err(EngineError::Cancelled)` from `Engine::mc`) keeps the EV-ranked finalists (C-35).
//! Diagnostics of the best build: leave-one-out, next-best replacement per slot, condition sensitivity (T-02 =
//! `none`), and the capacity and Forma report (09 E-9.16) with its violations (`RankedBuild.capacity`, `.violations`)
//! unless the mode is `ignore`.
