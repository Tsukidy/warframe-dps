//! Statistics over runs (08 E-8.19: mean, sample sd, normal-approximation CIs, quantiles, distribution-free median
//! CI99) and the oracle comparison OC2 of 08 E-8.25. Owner: T16.

use wfd_model::api::Summary;

/// 08 E-8.19 over per-run values (R = 1: sd and CIs `None`). T16.
pub fn summary(x: &[f64]) -> Summary {
    let _ = x;
    todo!("T16")
}

/// The OC2 comparison of two samples (08 E-8.25): medians with distribution-free CI99 intervals, each widened by
/// max(0.01·m, q).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OracleVerdict {
    /// The widened intervals intersect.
    pub pass: bool,
    /// Median of the oracle sample X.
    pub x_median: f64,
    /// Median of our sample Y.
    pub y_median: f64,
    /// Widened CI99 of X.
    pub x_interval: [f64; 2],
    /// Widened CI99 of Y.
    pub y_interval: [f64; 2],
}

/// 08 E-8.25 OC2: X = the inspiration's per-run values, Y = ours, display quantum q = 1 (the DPS and per-bullet
/// quantities; T16 adds the TTK quantum and the +∞ rules of E-8.25). T16.
pub fn oracle_compare(x: &[f64], y: &[f64]) -> OracleVerdict {
    let _ = (x, y);
    todo!("T16")
}
