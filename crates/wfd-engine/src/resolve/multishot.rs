//! Multishot (00 E-10; 02 E-2.1, D-019). Owner: T13.

/// E-10: `MS = max(1, (MS0 + MS0+)·(1 + ΣMS))`; with an Acuity lock `MS = MS0`. T13.
pub fn multishot(ms0: f64, ms_base_add: f64, sum_ms: f64, locked: bool) -> f64 {
    let _ = (ms0, ms_base_add, sum_ms, locked);
    todo!("T13")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "T13"]
    fn e_10_floor_and_lock() {
        assert!((multishot(1.0, 0.0, 0.9, false) - 1.9).abs() < 1e-12);
        assert!((multishot(1.0, 0.0, -0.5, false) - 1.0).abs() < 1e-12);
        assert!((multishot(1.0, 0.0, 0.9, true) - 1.0).abs() < 1e-12);
    }
}
