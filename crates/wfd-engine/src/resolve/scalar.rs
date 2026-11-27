//! ST-4 base scalar Y (00 E-11; 04 §2: CO behaviours, ρ_co, B_ref, G_F; melee projectiles 04 §3.4, E-4.5; the
//! `cond_flat_excluded` form of 04 E-4.3). C depends on the target state, so the scalar is kept affine in C (08 E-8.9
//! step 2). Owner: T13.

use wfd_model::{CoBehaviour, CoSpec};

use crate::StageCtx;
use crate::resolve::base::BaseVector;

/// The base-damage buckets of 01 E-1.11 (contract review ENG-6): A, or on a melee-projectile attack the melee bucket
/// A_m, and the universal weapon-damage bucket A_w (04 §3.4; `mproj_mode` selects the form).
///
/// `a` includes the exalted Strength terms of 01 E-1.1 models 2 and 3 (`Weapon.exalted`: `MultInA` adds m_r·Str − 1,
/// `Peacemaker` adds p_r·Str, at the rank r of T-31 and the build's Strength), which T13 adds when it fills the
/// buckets; model 1 is `AssembledWeapon.m_str` on the base instead (contract review N-3).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ABuckets {
    /// A (`base_dmg`); A_m on `two_bucket` and `single_bucket` attacks.
    pub a: f64,
    /// A_w (`base_dmg_universal`) on melee-projectile attacks; 0 otherwise.
    pub a_w: f64,
}

/// E-11 as an affine function of C (08 E-8.9 step 2): `Y(C) = max(0, y_const + y_per_c·C + y_per_cx·C_x)`, where
/// C_x ⊆ C is the flat-excluded part of a `cond_flat_excluded` attack (`CoResolved.c_x`, 0 elsewhere).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScalarY {
    /// Y0 = B'·max(0, 1 + A), or its melee-projectile form (01 E-1.11; two_bucket: B'·max(0, 1 + A_m)·max(0, 1 + A_w)).
    pub y0: f64,
    /// The part without C (`Y0·G_F`).
    pub y_const: f64,
    /// The coefficient β of C (08 E-8.9 step 2 table): `B_co` (adding, before r_in and κ_ms per pellet),
    /// `B_co·max(0, 1 + A_w)` (adding, two_bucket, 04 E-4.5), `Y0·ρ_co·G_F` (multiplying), 0 (none).
    pub y_per_c: f64,
    /// The coefficient of C_x: `(B_ref − B')·max(0, 1 + A)·G_F` for the multiplying behaviour (04 E-4.3 row 36,
    /// OQ-442: `[B'·max(0, 1 + A)·(1 + C − C_x) + B_ref·max(0, 1 + A)·C_x]·G_F`), 0 for the others; C_x itself is 0
    /// on every attack without flat-excluded entries. Addition, contract review ENG-6.
    pub y_per_cx: f64,
    /// The CO behaviour applied.
    pub behaviour: CoBehaviour,
}

impl ScalarY {
    /// Y at a given C with no flat-excluded part (E-11 with the final clamp `Y ← max(0, Y)`): `at_split(c, 0)`.
    /// T13.
    pub fn at(&self, c: f64) -> f64 {
        self.at_split(c, 0.0)
    }

    /// Y at a given C whose flat-excluded part is `c_x` (04 E-4.3, with the final clamp). Addition, contract review
    /// ENG-6. T13.
    pub fn at_split(&self, c: f64, c_x: f64) -> f64 {
        let _ = (c, c_x);
        todo!("T13")
    }
}

/// E-11 / 04 E-4.3, E-4.5: the base scalar from the base vector, the A buckets, G_F and the attack's CO spec (its
/// `mproj` mode and flags). The `a: ABuckets` parameter replaces `a: f64` (contract review ENG-6). T13.
pub fn base_scalar(
    ctx: &StageCtx,
    base: &BaseVector,
    a: ABuckets,
    g_f: f64,
    co: &CoSpec,
) -> ScalarY {
    let _ = (ctx, base, a, g_f, co);
    todo!("T13")
}
