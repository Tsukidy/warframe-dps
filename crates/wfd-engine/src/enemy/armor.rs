//! Armor (00 E-15 / 05 E-5.10): effective armor with ability strips, Corrosive Projection, T-07, flat and point
//! strips, Heat and Corrosive strips; the armor damage reduction DR(A). Owner: T07.

/// The strips of E-15 other than Heat and Corrosive (05 E-5.10, E-5.11).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ArmorStrips {
    /// Σ ability strips·Str (`strip_ability`; dropped against `ability_debuff_immune = all` at lowering, 05 §13.1).
    pub ability: f64,
    /// Σ CP·aura_strength (Corrosive Projection, 05 §9.3; `strip_aura`).
    pub aura: f64,
    /// T-07 external armor strip (0-0.99).
    pub toggle: f64,
    /// Σ companion % strips (`strip_companion`, Sharpened Claws; additive in Σ_add, not gated by 05 §13.1).
    /// Addition, contract review M-4.
    pub companion: f64,
    /// f_flat = max(0, B_A − s_flat)/B_A (flat base-armor strips, 05 E-5.11: s_flat accumulates
    /// `strip_flat_per_hit` per qualifying hit of 05 §9.4; 1 without; 1 on a `flat_only` strip-immune target).
    pub flat: f64,
    /// P_pts, point strips (`strip_points`, Polarize 400·Str; 05 E-5.10).
    pub points: f64,
}

/// E-15 / 05 E-5.10: `A_eff = max(0, A0·f_flat·max(0, 1 − Σ_add) − P_pts)·(1 − c_heat)·(1 − c_corr(n_COR))` with
/// Σ_add = ability + aura + toggle + companion and `heat_strip` = c_heat; A0 on an `armor_strip_immune = all`
/// target (05 §9.6). T07.
pub fn effective_armor(a0: f64, strips: &ArmorStrips, heat_strip: f64, n_cor: u8) -> f64 {
    let _ = (a0, strips, heat_strip, n_cor);
    todo!("T07")
}

/// The armor damage reduction DR(A) of E-12 (05 E-5.10): `arm_t = 1 − DR(A_eff)`. T07.
pub fn armor_dr(a: f64) -> f64 {
    let _ = a;
    todo!("T07")
}
