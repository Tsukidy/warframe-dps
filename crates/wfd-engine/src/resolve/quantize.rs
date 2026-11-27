//! ST-3 quantization (00 E-3, CD-02; D-001 compat): `q_t = sign(r_t)·floor(|r_t|·32 + 0.5)/32` per final type, in f64
//! (exact in binary). Owner: T13.

use wfd_model::DamageVec;

use crate::StageCtx;

/// E-3: the quantized ratios q_t. T13.
pub fn quantize(ctx: &StageCtx, ratio: &DamageVec) -> DamageVec {
    let _ = (ctx, ratio);
    todo!("T13")
}

#[cfg(test)]
mod tests {
    use super::*;
    use wfd_model::{CompatFlags, DamageType, N_DT};

    #[test]
    #[ignore = "T13"]
    fn e_3_rounds_half_up_on_the_32_grid_with_sign() {
        let rules = crate::test_support::mini_rules();
        let ctx = StageCtx::new(&rules, CompatFlags::empty());
        let mut r = [0.0; N_DT];
        r[DamageType::IMP.index()] = 0.3; // floor(9.6 + 0.5) / 32 = 10/32
        r[DamageType::PUN.index()] = -0.3;
        r[DamageType::COR.index()] = 1.8125; // on the grid: unchanged (00 §3.8 q_COR)
        let q = quantize(&ctx, &r);
        assert!((q[DamageType::IMP.index()] - 0.3125).abs() < 1e-15);
        assert!((q[DamageType::PUN.index()] + 0.3125).abs() < 1e-15);
        assert!((q[DamageType::COR.index()] - 1.8125).abs() < 1e-15);
    }
}
