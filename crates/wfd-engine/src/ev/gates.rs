//! Gate splitting (engine.md §7.4): a dynamic gate that feeds a nonlinear stage is evaluated once per combination of
//! such gates (≤ 4 binary gates, ≤ 16 resolutions) weighted by their probabilities; beyond 4 the rest are linearized
//! with the warning `ev.approx.gate_split_limit`. Owner: T15a.
