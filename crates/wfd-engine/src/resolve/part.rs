//! ST-6 body part (00 E-7; 02 E-2.10, §4.5). Owner: T13.

/// E-7 for a direct hit: `Part = (P_p + k_wp·ΣWP)·(1 + ΣHS)·M_son` with `p_part` = P_p, `k_wp` (1.5 on headshot
/// parts, 1.0 on other weak points), `wp` = ΣWP, `hs` = ΣHS, `m_son` = M_son. The caller passes `wp = 0` off a weak
/// point and `hs = 0` off a headshot part, and applies M_hsu, the `non_body` rule and ΣBP of E-7 (T13).
pub fn part_factor(p_part: f64, k_wp: f64, wp: f64, hs: f64, m_son: f64) -> f64 {
    let _ = (p_part, k_wp, wp, hs, m_son);
    todo!("T13")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "T13"]
    fn body_part_without_bonuses_is_one() {
        assert!((part_factor(1.0, 1.0, 0.0, 0.0, 1.0) - 1.0).abs() < 1e-15);
        // head with ΣWP 0.4 on a 1.5× weak point and ΣHS 0.5: (P_p + 1.5·0.4)·1.5 (E-7)
        assert!((part_factor(2.0, 1.5, 0.4, 0.5, 1.0) - 3.9).abs() < 1e-12);
    }
}
