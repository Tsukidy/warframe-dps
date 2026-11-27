//! Melee resolution (07 §5-§7): attack speed and the k/60 snap (E-7.1, E-7.2), wind-up, combo duration, Initial Combo,
//! heavy efficiency η, combo gains and their bonuses (E-7.12-E-7.17), the heavy-attack synthesis inputs (E-7.40), the
//! melee instance base of E-7.20 / E-7.20b. Owner: T08b.

use wfd_model::{FinisherMode, MeleeData, MeleeTables};

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
    /// Tennokai window T_tk (s; E-7.17: 2, or `tennokai_window` when a mod sets it). Addition, contract review ENG-21.
    pub tennokai_window: f64,
    /// Deterministic Tennokai spacing n_tk of the `tennokai` plan (E-7.17, 07 §8.3: `tennokai_every` when a mod sets
    /// it, else ceil(1/p_tk)); 0 when no Tennokai mod is equipped. Addition, contract review ENG-21.
    pub tennokai_every: u32,
    /// Tennokai wind-up WU_tk (s, E-7.4, E-7.17). Addition, contract review ENG-21.
    pub wind_up_tk: f64,
    /// Power Spike decay step δ_PS (E-7.14); 0 without Power Spike. Addition, contract review ENG-21.
    pub power_spike: f64,
    /// External combo gain rate (points/s, `combo_gain_ext`, class (c), default 0; E-7.13: adds points, never
    /// refreshes the timer). Addition, contract review ENG-21.
    pub combo_gain_ext: f64,
    /// η for heavy slams (E-7.16: sources scoped to heavy slams, the Furax Incarnon form's +0.90, enter only here);
    /// `heavy_eff` is the heavy attacks'. Addition, contract review ENG-21.
    pub heavy_eff_slam: f64,
    /// T-08 stealth damage bonus SDB of 04 E-4.16 (`1 + 0.2·min(rank, 30)`, the rank of T-08 or the weapon's
    /// maximum) for `stealth_ok` instances (07 §7.5); `None` when T-08 is off. Addition, contract review ENG-13.
    pub stealth_sdb: Option<f64>,
    /// M_ab of an ability-combo weapon (07 E-7.44, T-11; a G member, and M_b = min(M_ab, 12) for the combo-scaled
    /// bonuses, E-7.18); `None` for other weapons. Addition, contract review ENG-13.
    pub m_ab: Option<f64>,
    /// T-34 finisher mode (04 §7.2, E-4.17; 07 §9.11); `None` = off. Addition, contract review ENG-13.
    pub finisher: Option<FinisherMode>,
}

/// 07 §5-§7: the melee stats of an attack from its melee data, the stat vector and the resolved tables; the
/// scenario-derived fields (`stealth_sdb`, `m_ab`, `finisher`) are set at L3 from the toggles (T-08, T-11, T-34), as
/// for `ScheduleInputs`. T08b.
pub fn melee_resolved(
    ctx: &StageCtx,
    md: &MeleeData,
    sv: &StatVecF,
    t: &MeleeTables,
) -> MeleeResolved {
    let _ = (ctx, md, sv, t);
    todo!("T08b")
}
