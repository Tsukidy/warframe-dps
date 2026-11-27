//! ST-1 base vector (00 E-1; 01 §4.2-§4.7: progenitor V_t, Incarnon flat split D-005, Strength m_str; melee strike
//! S_j and heavy H_j of 07 E-7.20, E-7.20b). Owner: T13.

use wfd_model::{Attack, DamageVec};

use crate::StageCtx;
use crate::assemble::AssembledWeapon;
use crate::stats::StatVecF;

/// The base vector of one attack (E-1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BaseVector {
    /// B'_t = B0_t·m_str + V_t + X_t.
    pub b: DamageVec,
    /// B' = Σ_t B'_t.
    pub total: f64,
    /// B_ref = Σ_t (B0_t·m_str + V_t).
    pub b_ref: f64,
    /// X_t, the Incarnon flat part (not multiplied by H_j).
    pub x: DamageVec,
}

/// E-1: B'_t, B', B_ref of `atk` on the assembled weapon with the stat vector `sv`. T13.
pub fn base_vector(ctx: &StageCtx, atk: &Attack, w: &AssembledWeapon, sv: &StatVecF) -> BaseVector {
    let _ = (ctx, atk, w, sv);
    todo!("T13")
}
