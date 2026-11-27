//! Melee resolution (07 §5-§7): attack speed and the k/60 snap (E-7.1, E-7.2), wind-up, combo duration, Initial Combo,
//! heavy efficiency η, combo gains and their bonuses (E-7.12-E-7.17), the heavy-attack synthesis inputs (E-7.40), the
//! melee instance base of E-7.20 / E-7.20b. Owner: T08b.

use wfd_model::MeleeData;
use wfd_model::MeleeTables;

use crate::StageCtx;
use crate::stats::StatVecF;

/// Melee stats of one attack (07 §5-§7).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeleeResolved {
    /// Attack speed AS (E-7.2).
    pub attack_speed: f64,
    /// Heavy wind-up WU (s, E-7.5 with wind-up speed).
    pub wind_up: f64,
    /// Heavy swing time T_sw (s).
    pub swing: f64,
    /// Combo duration D_cb (s); `f64::INFINITY` = no expiry (E-7.12).
    pub combo_dur: f64,
    /// Initial Combo IC (E-7.15).
    pub initial_combo: f64,
    /// Heavy efficiency η (fraction of the combo kept after a heavy, E-7.16).
    pub heavy_eff: f64,
    /// Regeneration rate ρ_IC of Initial Combo (E-7.15).
    pub ic_regen: f64,
    /// k_tier (hits per tier, 07 §6; D-063).
    pub k_tier: f64,
    /// M_cap.
    pub m_cap: f64,
    /// M_bcap.
    pub m_bcap: f64,
    /// Combo gain chance p_gain (E-7.13).
    pub p_gain: f64,
    /// Extra combo gain chance p_acc (`combo_acc`, E-7.13).
    pub p_acc: f64,
    /// Relentless Combination chance (`combo_rc`, E-7.13b).
    pub p_rc: f64,
    /// Tennokai chance (E-7.17).
    pub tennokai_chance: f64,
}

/// 07 §5-§7: the melee stats of an attack from its melee data, the stat vector and the resolved tables. T08b.
pub fn melee_resolved(
    ctx: &StageCtx,
    md: &MeleeData,
    sv: &StatVecF,
    t: &MeleeTables,
) -> MeleeResolved {
    let _ = (ctx, md, sv, t);
    todo!("T08b")
}
