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

/// 08 E-8.25 OC2 for the DPS and per-bullet quantities: [`oracle_compare_q`] with the display quantum q = 1
/// ([`crate::spec_constants::OC2_QUANTUM_DPS`]).
pub fn oracle_compare(x: &[f64], y: &[f64]) -> OracleVerdict {
    oracle_compare_q(x, y, crate::spec_constants::OC2_QUANTUM_DPS)
}

/// 08 E-8.25 OC2: X = the inspiration's per-run values, Y = ours (each already truncated or rounded as the page
/// displays it), display quantum `q` (1 for the DPS and per-bullet values, 0.001 s for the TTK median:
/// [`crate::spec_constants::OC2_QUANTUM_TTK`]). Medians with distribution-free CI99 intervals (E-8.19), each widened
/// by max(0.01·m, q); pass iff the widened intervals intersect. Values +∞ (TTK runs that did not kill): both medians
/// +∞ pass, exactly one +∞ fails, otherwise the rule above with an interval end at +∞ kept at +∞ (the widening uses
/// the finite median). Addition to work-breakdown.md T02a (CONTRACTS.md T02a). T16.
pub fn oracle_compare_q(x: &[f64], y: &[f64], q: f64) -> OracleVerdict {
    let _ = (x, y, q);
    todo!("T16")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "T16"]
    fn tv_08_16_oracle_comparison() {
        // TV-08-16 (E-8.25): identical samples always pass.
        let x: Vec<f64> = (0..200).map(|k| 18_000.0 + f64::from(k) * 5.0).collect();
        let v = oracle_compare(&x, &x);
        assert!(v.pass);
        assert!((v.x_median - v.y_median).abs() < 1e-9);
    }

    #[test]
    #[ignore = "T16"]
    fn ttk_quantum_and_infinite_medians() {
        // E-8.25: q = 0.001 s for TTK; both medians +∞ pass, exactly one +∞ fails.
        let inf = vec![f64::INFINITY; 11];
        let fin: Vec<f64> = (0..11).map(|k| 4.0 + f64::from(k) * 0.001).collect();
        let q = crate::spec_constants::OC2_QUANTUM_TTK;
        assert!(oracle_compare_q(&inf, &inf, q).pass);
        assert!(!oracle_compare_q(&inf, &fin, q).pass);
        // Medians 4.005 and 5.0: widened by max(0.01·m, 0.001), the intervals do not meet; with q = 1 they would.
        let far: Vec<f64> = fin.iter().map(|t| t + 0.995).collect();
        assert!(!oracle_compare_q(&fin, &far, q).pass);
        assert!(oracle_compare_q(&fin, &far, 1.0).pass);
    }
}
