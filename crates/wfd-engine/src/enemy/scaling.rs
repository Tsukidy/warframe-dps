//! Enemy scaling (05 §6-§8): level curves and the smoothstep blend in f32 (NP-2, `f1 + (f2 − f1)·s`), Steel Path,
//! Eximus, party scaling and overguard; the scaled pools, A0 (05 E-5.10), type modifiers φ_t per layer, caps and
//! immunities (05 §4.4), the T-03 part (05 §4.3), innate and class DR, attenuation data. Owner: T07.

use smallvec::SmallVec;
use wfd_model::{
    Attenuation, DamageTypeSet, DamageVec, DrClass, Enemy, EnemyFlags, EnemyId, FactionId, Layer,
    N_ST, StatusSet, Toggles,
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
    /// Status caps after the enemy's caps (03 E-3.2, 05 §4.4) with `og_present` false; [`crate::status::CAP_UNBOUNDED`]
    /// = no cap (03 §4.1 base cap ∞ with no enemy cap).
    pub caps: [u8; N_ST],
    /// The caps with `og_present` true (03 E-3.2: Cold 4, VIP Puncture 3, Evensong Puncture 5), for the `overguard`
    /// view's timeline and the layered passes while OG > 0 (08 §4.5); equal to `caps` without overguard. Addition,
    /// contract review ENG-10.
    pub caps_og: [u8; N_ST],
    /// Status immunities.
    pub immune: StatusSet,
    /// The shield bypass set B_SH (05 E-5.15).
    pub shield_bypass: DamageTypeSet,
    /// The innate DR factor `1 − innate_dr[ℓ]` per layer (05 E-5.20), indexed by `Layer::index`; 1 with T-28 off
    /// (05 §15.5). The class and status factors depend on the instance (`TargetStateView.class`): `dr_class`,
    /// `dr_class_og`, `status_dr`.
    pub dr: [f64; Layer::N],
    /// `status_dr` (05 E-5.20: status instances on SH and HP; Archons 0.5); 0 with T-28 off. Addition, contract review
    /// ENG-9.
    pub status_dr: f64,
    /// Class-conditional DR on SH and HP (05 E-5.20 `dr_class[c]`); zero with T-28 off. Addition, contract review
    /// ENG-9.
    pub dr_class: DrClass,
    /// Class-conditional DR on OG (05 E-5.20 `dr_class_og[c]`); zero with T-28 off. Addition, contract review ENG-9.
    pub dr_class_og: DrClass,
    /// Attenuation data (05 §15.1), `None` without or with T-28 off.
    pub attenuation: Option<Attenuation>,
    /// The enemy's `Rules.special_dr` entry (05 §15: DPS-scaled DR `dps_dr` of E-5.22, adaptation of E-5.23 with its
    /// kind), read through `StageCtx.rules`; `None` without one or with T-28 off (05 §15.5). Addition, contract review
    /// ENG-9.
    pub special_dr: Option<u16>,
    /// T-28 unique DR on (00 §6.1, 05 §15.5); with T-28 off `dr`, `status_dr`, `dr_class`, `dr_class_og`, part DR,
    /// attenuation, M_dps and adaptation are removed here, and the Guardian aura stays. Addition, contract review ENG-9.
    pub unique_dr: bool,
    /// The flags of 05 §4.1 the stages read: `TOXIN_BLOCKED` and `TOXIN_NULLIFY` (05 E-5.24, routing), the shield-
    /// immunity flags (E-5.26), `CC_IMMUNE` (03 E-3.13, 06 E-6.24 stun-gated children), `VIP` (03 E-3.2),
    /// `STRIP_IMMUNE` and `STRIP_IMMUNE_FLAT` (05 §9.6), `ABILITY_DEBUFF_IMMUNE` (05 §13.1), `GUARDIAN_AURA_IMMUNE`
    /// (E-5.27). Addition, contract review ENG-9.
    pub flags: EnemyFlags,
    /// The target is inside a Guardian Eximus aura that affects it (T-23 `aura_guardian`, 05 E-5.27: R_ℓ × 0.1 on SH
    /// and HP, kept with T-28 off; never on `GUARDIAN_AURA_IMMUNE` or `boss` records). Addition, contract review ENG-9.
    pub guardian_aura: bool,
    /// The T-03 hit part (or the T-32 weak spot's part) at index 0 and the body part (radial instances) at index 1,
    /// as chosen by `scale_enemy` from the toggles.
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

/// 05 §6-§8: the enemy scaled for the scenario's enemy setup, with the toggles it needs (contract review ENG-9): T-28
/// (`unique_dr`, 05 §15.5) removes the unique DR, T-03 and T-32 choose the parts (05 §4.3). The `toggles` parameter
/// is an addition. T07.
pub fn scale_enemy(
    ctx: &StageCtx,
    e: &Enemy,
    setup: &wfd_model::EnemyInstance,
    toggles: &Toggles,
) -> ScaledEnemy {
    let _ = (ctx, e, setup, toggles);
    todo!("T07")
}

/// NP-2: the smoothstep blend of two scaling curves in f32, evaluated as `f1 + (f2 − f1)·s` (05 §6.3). T07.
pub fn np2_blend(f1: f32, f2: f32, s: f32) -> f32 {
    let _ = (f1, f2, s);
    todo!("T07")
}
