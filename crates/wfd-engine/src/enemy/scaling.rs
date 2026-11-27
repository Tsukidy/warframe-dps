//! Enemy scaling (05 §6-§8): level curves and the smoothstep blend in f32 (NP-2, `f1 + (f2 − f1)·s`), Steel Path,
//! Eximus, party scaling and overguard; the scaled pools, A0 (05 E-5.10), type modifiers φ_t per layer, caps and
//! immunities (05 §4.4), the T-03 part (05 §4.3), innate and class DR, attenuation data. Owner: T07.

use smallvec::SmallVec;
use wfd_model::{
    Attenuation, DamageTypeSet, DamageVec, Enemy, EnemyId, FactionId, Layer, N_ST, StatusSet,
};

use crate::StageCtx;

/// An enemy scaled for one scenario (`CompiledScenario.enemy`).
#[derive(Debug, Clone, PartialEq)]
pub struct ScaledEnemy {
    /// The enemy record.
    pub enemy: EnemyId,
    /// Its level.
    pub level: u16,
    /// Its faction (damage-type table and bane relations).
    pub faction: FactionId,
    /// Scaled pools OG, SH, HP, indexed by `Layer::index` (0 = layer absent).
    pub pools: [f64; Layer::N],
    /// A0 = clamp(floor(A_base·M_armor(q)), 200, 2700), 0 without armor (05 E-5.10).
    pub armor0: f64,
    /// Type modifiers φ_t per layer (OG row: 1 except the VOI factor of E-12), indexed by `Layer::index`.
    pub phi: [DamageVec; Layer::N],
    /// Status caps after the enemy's caps (03 E-3.2, 05 §4.4); [`crate::status::CAP_UNBOUNDED`] = no cap (03 §4.1
    /// base cap ∞ with no enemy cap).
    pub caps: [u8; N_ST],
    /// Status immunities.
    pub immune: StatusSet,
    /// The shield bypass set B_SH (05 E-5.15).
    pub shield_bypass: DamageTypeSet,
    /// Innate and class DR per layer, multiplied (05 E-5.20), indexed by `Layer::index`.
    pub dr: [f64; Layer::N],
    /// Attenuation data (05 §15.1), `None` without.
    pub attenuation: Option<Attenuation>,
    /// The T-03 hit part and the body part (radial instances).
    pub parts: SmallVec<[ScaledPart; 2]>,
    /// Damage-object target (`is_object`: no crits, 02 E-2.6).
    pub is_object: bool,
    /// Shields may regenerate during the fight (05 §10.5).
    pub shield_regen_possible: bool,
}

/// One body part of the scaled enemy (05 §4.3; 02 E-2.8, E-2.10).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScaledPart {
    /// P_p (damage multiplier of the part).
    pub multiplier: f64,
    /// h (head-crit factor).
    pub headcrit: f64,
    /// π_p (Critical Promotion).
    pub tier_add: u8,
    /// Weak point.
    pub weak_point: bool,
    /// Headshot flag.
    pub headshot: bool,
    /// `non_body` part (neither headshot nor bodyshot).
    pub non_body: bool,
    /// `crit_disabled` part.
    pub crit_disabled: bool,
    /// Part DR (05 E-5.20).
    pub part_dr: f64,
}

/// 05 §6-§8: the enemy scaled for the scenario's enemy setup. T07.
pub fn scale_enemy(ctx: &StageCtx, e: &Enemy, setup: &wfd_model::EnemyInstance) -> ScaledEnemy {
    let _ = (ctx, e, setup);
    todo!("T07")
}

/// NP-2: the smoothstep blend of two scaling curves in f32, evaluated as `f1 + (f2 − f1)·s` (05 §6.3). T07.
pub fn np2_blend(f1: f32, f2: f32, s: f32) -> f32 {
    let _ = (f1, f2, s);
    todo!("T07")
}
