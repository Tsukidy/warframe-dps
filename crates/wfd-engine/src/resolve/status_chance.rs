//! Status chance and proc weights (00 E-8, E-9; 02 §4.6-§4.7). Owner: T13.

use wfd_model::{DamageVec, N_ST, StatusSet};

/// E-8: `SC = ((SC0 + SC0+)·(1 + ΣSC_rel) + ΣSC_abs)·M_sc` per pellet. T13.
pub fn status_chance(sc0: f64, sc_base_add: f64, sc_rel: f64, sc_abs: f64, m_sc: f64) -> f64 {
    let _ = (sc0, sc_base_add, sc_rel, sc_abs, m_sc);
    todo!("T13")
}

/// E-9: `w_s = q_s·[s not immune] / Σ_u q_u·[u not immune]` over the quantized composition (D-024), per status
/// type. T13.
pub fn proc_weights(q: &DamageVec, immune: StatusSet) -> [f64; N_ST] {
    let _ = (q, immune);
    todo!("T13")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "T13"]
    fn e_8_status_chance() {
        // ((0.26 + 0)·(1 + 0.9) + 0.1)·1 = 0.594
        assert!((status_chance(0.26, 0.0, 0.9, 0.1, 1.0) - 0.594).abs() < 1e-12);
    }
}
