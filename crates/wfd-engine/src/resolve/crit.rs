//! ST-7 crit (00 E-4..E-6; 02 E-2.4-E-2.8, NP-1). The base crit multiplier is quantized in f32 in this one function
//! (engine.md §12). Owner: T13.

use wfd_model::N_TIER_BINS;

/// The crit multiplier of 00 E-4 / 02 E-2.5.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CritMult {
    /// x32, the f32 sum CDM0 + CDM0+ (+ cond·CDM0c).
    pub x: f32,
    /// code = floor(f32(f32(clamp(x, 0, 32)·127.96875) + 0.5)), 0 … 4095.
    pub code: u16,
    /// Q = f32(code·32/4095) as f64 (the correctly rounded quotient, 02 OQ-222 decision).
    pub q: f64,
    /// CDM = (Q·(1 + ΣCD_rel) + ΣCD_abs)·M_cdf (f64).
    pub cdm: f64,
}

/// The crit-tier distribution of one instance (02 E-2.7): at most three adjacent tiers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TierDist {
    /// The lowest tier with nonzero probability: floor(CC_r) + π_p.
    pub base: u32,
    /// P(k = base), P(k = base + 1), P(k = base + 2).
    pub p: [f64; 3],
    /// `E[k] = CC_r + p_vig·min(1, CC_r) + π_p`.
    pub mean: f64,
}

impl TierDist {
    /// The distribution folded into the 8 output bins of 08 §8.4 (index 7 = "7 or more"). T13.
    pub fn bins(&self) -> [f64; N_TIER_BINS] {
        todo!("T13")
    }
}

/// E-4 (NP-1): crit multiplier from CDM0, CDM0+ (`cd_base_add`), ΣCD_rel, ΣCD_abs and M_cdf. The Prelude term
/// CDM0c is part of `cd_base_add` when its condition holds (02 E-2.5). T13.
pub fn crit_multiplier(
    cdm0: f64,
    cd_base_add: f64,
    cd_rel: f64,
    cd_abs: f64,
    m_cdf: f64,
) -> CritMult {
    let _ = (cdm0, cd_base_add, cd_rel, cd_abs, m_cdf);
    todo!("T13")
}

/// E-5: `CC = max(CC_min, ((CC0 + CC0+)·(1 + ΣCC_rel) + ΣCC_abs)·M_cc)` with `final_mul` = M_cc. T13.
pub fn crit_chance(
    cc0: f64,
    cc_base_add: f64,
    cc_rel: f64,
    cc_abs: f64,
    cc_floor: f64,
    final_mul: f64,
) -> f64 {
    let _ = (cc0, cc_base_add, cc_rel, cc_abs, cc_floor, final_mul);
    todo!("T13")
}

/// E-6 / 02 E-2.7: the tier distribution of an instance with chance `cc` (CC_r = max(0, cc)), Vigilante `p_vig` and
/// Critical Promotion `tier_add` (π_p). T13.
pub fn crit_tiers(cc: f64, p_vig: f64, tier_add: u8) -> TierDist {
    let _ = (cc, p_vig, tier_add);
    todo!("T13")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() <= 1e-9 * b.abs().max(1.0)
    }

    #[test]
    #[ignore = "T13"]
    fn tv_02_01_quantized_crit_multiplier() {
        // TV-02-01 (engine.md §14): CDM0 2.0, ΣCD_rel 1.2 → code 256, Q = 2.000488519668579, CDM = 4.401074743270875
        let m = crit_multiplier(2.0, 0.0, 1.2, 0.0, 1.0);
        assert_eq!(m.code, 256);
        assert_eq!(m.q.to_bits(), 2.000488519668579_f64.to_bits());
        assert_eq!(
            f64::from(m.q as f32).to_bits(),
            m.q.to_bits(),
            "Q is a binary32 value (NP-1)"
        );
        assert!(close(m.cdm, 4.401074743270875));
    }

    #[test]
    #[ignore = "T13"]
    fn e_2_7_tier_distributions() {
        // CC 1.2, no Vigilante: P(1) = 0.8, P(2) = 0.2, E[k] = 1.2 (02 E-2.7, TV-02-02)
        let d = crit_tiers(1.2, 0.0, 0);
        assert_eq!(d.base, 1);
        assert!(close(d.p[0], 0.8) && close(d.p[1], 0.2) && close(d.p[2], 0.0));
        assert!(close(d.mean, 1.2));
        // CC 0.5, p_vig 0.3: P(0) = 0.5, P(1) = 0.35, P(2) = 0.15, E[k] = 0.65
        let d = crit_tiers(0.5, 0.3, 0);
        assert_eq!(d.base, 0);
        assert!(close(d.p[0], 0.5) && close(d.p[1], 0.35) && close(d.p[2], 0.15));
        assert!(close(d.mean, 0.65));
        // Critical Promotion shifts every tier by π_p
        assert_eq!(crit_tiers(0.5, 0.0, 1).base, 1);
    }

    #[test]
    #[ignore = "T13"]
    fn e_5_crit_chance_floor() {
        // ((0.12 + 0)·(1 + 1.5) + 0)·1 = 0.3; the floor CC_min wins when larger
        assert!(close(crit_chance(0.12, 0.0, 1.5, 0.0, 0.0, 1.0), 0.3));
        assert!(close(crit_chance(0.12, 0.0, 1.5, 0.0, 0.5, 1.0), 0.5));
    }
}
