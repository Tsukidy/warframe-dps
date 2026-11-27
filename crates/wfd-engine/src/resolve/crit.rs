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

/// E-4 (NP-1) with at most one CDM0+ entry and no Prelude term: exactly
/// `crit_multiplier_terms(cdm0, &[cd_base_add], 0.0, cd_rel, cd_abs, m_cdf)`. With `cd_base_add` = 0 the extra f32
/// addend changes nothing (x + 0 = x exactly). The Prelude term CDM0c is never folded into `cd_base_add`: 02 E-2.5
/// adds it as its own f32 term, and a pre-sum in f64 can change x32 by an ulp and so the code. It has its own
/// registry rows, `cd_base_cond` (CDM0c) and `cd_base_cond_thr` (θ_P), which L3 reads into `CritResolved.q_cond`
/// and `theta_p` (02 §3 symbol table; contract review ENG-R7, NEW-CR-8).
pub fn crit_multiplier(
    cdm0: f64,
    cd_base_add: f64,
    cd_rel: f64,
    cd_abs: f64,
    m_cdf: f64,
) -> CritMult {
    crit_multiplier_terms(cdm0, &[cd_base_add], 0.0, cd_rel, cd_abs, m_cdf)
}

/// E-4 / 02 E-2.5 (NP-1) with the exact f32 summation order: `x32 = ((f32(CDM0) + f32(x_1)) + f32(x_2)) + … +
/// f32(cdm0c)`, left to right, the CDM0+ entries `cdm0_plus` in ascending Incarnon evolution tier (02 E-2.5,
/// `inc2` … `inc5`) and then `cdm0c` = cond_i·CDM0c (0 when the Prelude condition CC_i < θ_P fails or the weapon has
/// no such perk); then `xc = clamp(x32, 0, 32)`, `code = floor(f32(f32(xc·127.96875) + 0.5))`,
/// `Q = f32(code·32/4095)` (the correctly rounded quotient) and the f64 tail `CDM = (Q·(1 + ΣCD_rel) + ΣCD_abs)·M_cdf`.
///
/// The `cd_base_add` slot of a `StatVec` holds only the fixed-point sum of the CDM0+ entries, and L3 always passes
/// that sum as one entry: [`crate::Engine::resolve`] is `resolve_point` on `CompiledBuild::point()`, and a
/// `BuildPoint` carries no per-source entries, on the calculator path as on the optimizer's (contract review ENG-R1,
/// pass 3; this replaces the ENG-19 reading of `CompiledBuild.sources` at L3). Incarnon perks are the only CDM0+
/// source and at most one is active per weapon with current data (02 E-2.5), so the sum is the entry and x32 is exact.
/// A build with more than one active CDM0+ source (none exists in the research) may differ from E-2.5 by one f32 ulp
/// in x32; T17 reports it with `rule.unverified { row = "02 E-2.5" }`, counting the CDM0+ entries of
/// `CompiledBuild.sources`. The Prelude term `cdm0c` comes from the `cd_base_cond` row (02 CDM0c; contract review
/// ENG-R7). Addition to work-breakdown.md T02a. T13.
pub fn crit_multiplier_terms(
    cdm0: f64,
    cdm0_plus: &[f64],
    cdm0c: f64,
    cd_rel: f64,
    cd_abs: f64,
    m_cdf: f64,
) -> CritMult {
    let _ = (cdm0, cdm0_plus, cdm0c, cd_rel, cd_abs, m_cdf);
    todo!("T13")
}

/// 02 E-2.5a (Melee Doughty; ruling C-37): `doughty = min(50, round1(r_pun_share·(100·sc_w)·0.1·b_r))` with
/// `round1(x) = floor(10·x + 0.5)/10`, where `r_pun_share` = r_PUN/Σ_t r_t of the unquantized modded composition of
/// the instance's attack (or strike), `sc_w` = SC_w of 02 E-2.11 without M_sc,t for this hit (combo-dependent and
/// heavy-only terms included; target-side terms excluded, OQ-209) and `b_r` = `doughty_rank`; 0 when `b_r` = 0. The
/// result is its own term of ΣCD_abs,i (02 E-2.5). Addition, contract review ENG-19. T13.
pub fn doughty(r_pun_share: f64, sc_w: f64, b_r: f64) -> f64 {
    let _ = (r_pun_share, sc_w, b_r);
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
    fn e_2_5_f32_sum_is_left_to_right_with_the_prelude_term_last() {
        // 02 E-2.5: x32 = f32(CDM0), then each CDM0+ entry f32(x_e) in catalogue order, then f32(cond·CDM0c).
        let (cdm0, plus, c) = (1.6, [0.1, 0.2], 3.4);
        let expect = ((cdm0 as f32 + plus[0] as f32) + plus[1] as f32) + c as f32;
        let m = crit_multiplier_terms(cdm0, &plus, c, 0.0, 0.0, 1.0);
        assert_eq!(m.x.to_bits(), expect.to_bits());
        // Without CDM0+ entries and Prelude, both forms agree (TV-02-01: code 256).
        let a = crit_multiplier_terms(2.0, &[], 0.0, 1.2, 0.0, 1.0);
        let b = crit_multiplier(2.0, 0.0, 1.2, 0.0, 1.0);
        assert_eq!((a.code, a.q.to_bits()), (b.code, b.q.to_bits()));
        assert_eq!(a.code, 256);
        // W:Critical_Hit "Critical Parallel": Code(1.6 + 0.4) = 256.
        assert_eq!(
            crit_multiplier_terms(1.6, &[0.4], 0.0, 0.0, 0.0, 1.0).code,
            256
        );
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
    fn tv_02_14_doughty_rounds_to_a_tenth_and_caps_at_50() {
        // TV-02-14: share 119 × 2.2 / 312.8, SC_w = 0.2 × (1 + 0.4 + 0.4 × 11) = 1.16, rank factor 1.0: raw
        // 9.708695652173915 → 9.7; share 1.0, SC_w 6.0: raw 60.0 → 50 (cap after rounding). b_r 0: no Doughty.
        let share = 119.0 * 2.2 / 312.8;
        assert!(close(doughty(share, 1.16, 1.0), 9.7));
        assert!(close(doughty(1.0, 6.0, 1.0), 50.0));
        assert!(close(doughty(share, 1.16, 0.0), 0.0));
    }

    #[test]
    #[ignore = "T13"]
    fn e_5_crit_chance_floor() {
        // ((0.12 + 0)·(1 + 1.5) + 0)·1 = 0.3; the floor CC_min wins when larger
        assert!(close(crit_chance(0.12, 0.0, 1.5, 0.0, 0.0, 1.0), 0.3));
        assert!(close(crit_chance(0.12, 0.0, 1.5, 0.0, 0.5, 1.0), 0.5));
    }
}
