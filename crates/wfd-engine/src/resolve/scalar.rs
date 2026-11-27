//! ST-4 base scalar Y (00 E-11; 04 §2: CO behaviours, ρ_co, B_ref, G_F; melee projectiles 04 §3.4). C depends on the
//! target state, so the scalar is kept affine in C. Owner: T13.

use wfd_model::{CoBehaviour, CoSpec};

use crate::StageCtx;
use crate::resolve::base::BaseVector;

/// E-11 as an affine function of C: `Y(C) = max(0, y_const + y_per_c·C)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScalarY {
    /// Y0 = B'·max(0, 1 + A).
    pub y0: f64,
    /// The part without C (`Y0·G_F`).
    pub y_const: f64,
    /// The coefficient of C: `κ·B_co` (adding, before r_in and κ per pellet) or `Y0·ρ_co·G_F` (multiplying), 0 (none).
    pub y_per_c: f64,
    /// The CO behaviour applied.
    pub behaviour: CoBehaviour,
}

impl ScalarY {
    /// Y at a given C (E-11 with the final clamp `Y ← max(0, Y)`). T13.
    pub fn at(&self, c: f64) -> f64 {
        let _ = c;
        todo!("T13")
    }
}

/// E-11: the base scalar from the base vector, A, G_F and the attack's CO spec. T13.
pub fn base_scalar(ctx: &StageCtx, base: &BaseVector, a: f64, g_f: f64, co: &CoSpec) -> ScalarY {
    let _ = (ctx, base, a, g_f, co);
    todo!("T13")
}
