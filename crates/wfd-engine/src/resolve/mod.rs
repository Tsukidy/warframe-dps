//! L3: resolution of one attack, ST-1..ST-7 plus firing stats (engine.md §4). Every stage function is pure, takes a
//! [`crate::StageCtx`] and cites its equation; the result [`ResolvedAttack`] is a fixed-size `Copy` struct (no heap).
//! Terms that depend on target state (Puncture crit chance, Cold crit damage, the CO count, Viral, Magnetic, armor)
//! are not folded in; the EV kernel and the MC simulator apply them from the state. Owner: T13; `firing.rs` T08a,
//! `melee.rs` T08b.

pub mod base;
pub mod composition;
pub mod crit;
pub mod firing;
pub mod melee;
pub mod multishot;
pub mod part;
pub mod quantize;
pub mod scalar;
pub mod status_chance;
pub mod weapon_mult;

pub use base::{BaseVector, base_vector};
pub use composition::{combine_elements, ratios};
pub use crit::{CritMult, TierDist, crit_chance, crit_multiplier, crit_tiers};
pub use firing::{FiringResolved, firing_stats};
pub use melee::{MeleeResolved, melee_resolved};
pub use multishot::multishot;
pub use part::part_factor;
pub use quantize::quantize;
pub use scalar::{ScalarY, base_scalar};
pub use status_chance::{proc_weights, status_chance};
pub use weapon_mult::{WeaponMult, weapon_mult};

use wfd_model::{
    AttackIdx, AttackRoles, CoBehaviour, DamageType, DamageVec, N_ST, StatusSet, StatusType,
};

/// Number of extra-hit kinds (`xh.<k>` slots, E-14, 04 §6).
pub const N_XH: usize = 8;
/// Number of mechanic magnitude slots (`mech.<n>`, domain-model.md §10).
pub const N_MECH: usize = 16;
/// Largest number of binary gates split exactly by EV gate splitting (engine.md §7.4: ≤ 4 gates, ≤ 16 resolutions).
pub const MAX_SPLIT_GATES: usize = 4;

/// One attack resolved for a build in a scenario (engine.md §4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedAttack {
    /// The attack.
    pub attack: AttackIdx,
    /// Its roles.
    pub roles: AttackRoles,
    /// B'_t (E-1).
    pub base: DamageVec,
    /// B' = Σ_t B'_t (E-1).
    pub base_total: f64,
    /// B_ref (E-1; no Incarnon flat).
    pub base_ref: f64,
    /// r_t (E-2).
    pub ratio: DamageVec,
    /// q_t (E-3).
    pub ratio_q: DamageVec,
    /// A, the base-damage bucket (E-11).
    pub a: f64,
    /// C as affine terms with the CO behaviour, ρ_co, B_co (E-11, 04 §2).
    pub co: CoResolved,
    /// Π G_F (static part; schedule-driven factors are applied per shot).
    pub g_f: f64,
    /// F, the faction bucket (01 §4.15).
    pub faction: f64,
    /// Π G (static part; 04 §3).
    pub g: f64,
    /// P_part, ΣWP, ΣHS, M_son per hit location (E-7).
    pub part: PartResolved,
    /// CC parts, Q (NP-1), CDM parts, p_vig (E-4..E-6).
    pub crit: CritResolved,
    /// SC (E-8), w_s (E-9), φ_s, A_sd(+s), SD, caps, DoT coefficients, external stacks.
    pub status: StatusResolved,
    /// MS (E-10).
    pub ms: f64,
    /// Multishot locked (Acuity, E-10).
    pub ms_locked: bool,
    /// FR, EFR, CT, burst, magazine, cost, AE, reload, duty, ramp, spool, gauge (06 §5).
    pub firing: FiringResolved,
    /// AS, W, D_cb, IC, η, combo gains, heavy H_j (07 §5-§7); `None` for non-melee attacks.
    pub melee: Option<MeleeResolved>,
    /// Extra-hit kinds (E-14), indexed by `xh.<k>`.
    pub xh: [ExtraHitResolved; N_XH],
    /// Mechanic magnitudes.
    pub mech: MechResolved,
    /// Dynamic gates that feed nonlinear stages (engine.md §7.4).
    pub gates: GateSet,
    /// Statuses this attack (or externals) can apply.
    pub mask: StatusSet,
    /// T-41 pellet hit fraction f_a (06 E-6.24, 08 E-8.2, E-8.17; 1 = every instance hits). Addition to engine.md §4
    /// (CONTRACTS.md T02a).
    pub hit_fraction: f64,
}

/// The CO family of the attack (E-11; 04 §2): C = `c_flat + c_status·n + c_cold·n_COL` (+ condition-stack
/// coefficients of the build), applied per `behaviour`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoResolved {
    /// `adding`, `multiplying` or `none` (04 §2.14 catalogue, defaults D1-D6; `none` under T-02 `none`).
    pub behaviour: CoBehaviour,
    /// ρ_co.
    pub rho: f64,
    /// B_co = ρ_co·B_ref, or the catalogue's fixed B_co^abs.
    pub b_co: f64,
    /// κ = co_ms_factor on pellets j ≥ 1 of a shot (else 1).
    pub kappa: f64,
    /// Constant part of C.
    pub c_flat: f64,
    /// Coefficient of n (`co_per_status`).
    pub c_status: f64,
    /// Coefficient of n_COL (`co_per_cold_stack`).
    pub c_cold: f64,
    /// T-04: CO multiplicative.
    pub multiplicative: bool,
}

/// The body-part terms of one hit location (E-7, 02 E-2.10, E-2.8).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HitPart {
    /// Part = Part_A·Part_B·M_son.
    pub factor: f64,
    /// Head-crit factor h of the part.
    pub h: f64,
    /// Critical Promotion π_p.
    pub tier_add: u8,
    /// The part is a weak point (weak-point terms apply).
    pub weak_point: bool,
    /// The part has the headshot flag (headshot terms apply).
    pub headshot: bool,
    /// `crit_disabled` part or `is_object` target: tier forced to 0 (02 E-2.6).
    pub crit_disabled: bool,
}

/// Body-part terms of the attack (E-7): the direct-hit location of T-03 and the radial instance's part.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PartResolved {
    /// P_p of the T-03 part.
    pub p_part: f64,
    /// k_wp (1.5 on headshot parts, 1.0 on other weak points).
    pub k_wp: f64,
    /// ΣWP.
    pub wp: f64,
    /// ΣHS.
    pub hs: f64,
    /// ΣBP (bodyshot penalty).
    pub bp: f64,
    /// M_son (Sonar and Detect Vulnerability spots, 02 E-2.10).
    pub m_son: f64,
    /// Direct hits.
    pub direct: HitPart,
    /// Radial instances (body part, or the parent's part for `radial_parent_part` attacks).
    pub radial: HitPart,
}

/// Crit terms of the attack without the target-state terms (E-4..E-6, 02 E-2.4-E-2.7).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CritResolved {
    /// CC0 of the attack data.
    pub cc0: f64,
    /// CC0+.
    pub cc_base_add: f64,
    /// ΣCC_rel.
    pub cc_rel: f64,
    /// ΣCC_abs.
    pub cc_abs: f64,
    /// CC_min.
    pub cc_floor: f64,
    /// M_cc (custom p.d factor).
    pub cc_final_mul: f64,
    /// CC at zero Puncture stacks (E-5).
    pub cc: f64,
    /// The quantized base crit multiplier without the Prelude term (NP-1).
    pub q: CritMult,
    /// The quantized base crit multiplier with the Prelude term CDM0c (NP-1); equal to `q` without it.
    pub q_cond: CritMult,
    /// Prelude threshold θ_P (strict); `None` without the perk.
    pub theta_p: Option<f64>,
    /// ΣCD_rel.
    pub cd_rel: f64,
    /// ΣCD_abs,static (without Cold).
    pub cd_abs: f64,
    /// M_cdf.
    pub m_cdf: f64,
    /// p_vig (Vigilante).
    pub p_vig: f64,
}

/// Status terms of the attack (E-8, E-9, E-13; 03 §4.1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatusResolved {
    /// SC per pellet (E-8).
    pub sc: f64,
    /// Proc weights w_s (E-9).
    pub w: [f64; N_ST],
    /// Forced procs φ_s per pellet (02 §4.7.2).
    pub forced: [f64; N_ST],
    /// A_sd.
    pub a_sd: f64,
    /// A_sd,s.
    pub a_sd_type: [f64; N_ST],
    /// SD.
    pub sd: f64,
    /// SD_s.
    pub sd_type: [f64; N_ST],
    /// Stack caps cap_s with the build's increases (E-3.2; enemy caps applied at L0).
    pub caps: [u8; N_ST],
    /// DoT coefficient K_s·(1 + E^dot_s) per DoT status (E-13).
    pub dot_k: [f64; N_ST],
    /// External stacks N_ext,s (T-43 / T-06 and `external.*` stats, max).
    pub n_ext: [u8; N_ST],
}

/// One extra-hit kind (E-14, 04 §6).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ExtraHitResolved {
    /// p_xh (0 = kind absent).
    pub p: f64,
    /// The damage type e_xh.
    pub ty: Option<DamageType>,
    /// The kind multiplies by Part (Xata's Whisper).
    pub part: bool,
    /// The forced proc of the kind (Toxic Lash: TOX); `None` = rolls the weapon's SC (Xata: VOI).
    pub forced: Option<StatusType>,
}

/// Mechanic magnitudes of the attack (`mech.<n>` slots after aggregation).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MechResolved {
    /// Magnitude per `mech.<n>` slot.
    pub mag: [f64; N_MECH],
}

/// Dynamic gates of the attack that feed nonlinear stages (engine.md §7.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GateSet {
    /// Condition-table indices of the gates split exactly.
    pub ids: [u16; MAX_SPLIT_GATES],
    /// Number of gates in `ids`.
    pub len: u8,
    /// More gates existed and were linearized (`ev.approx.gate_split_limit`).
    pub linearized: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hot_path_types_are_fixed_size_copy() {
        // engine.md §4, §13: ResolvedAttack, PointState and the schedule records are fixed-size Copy values (no heap).
        fn assert_copy<T: Copy + Send + Sync>() {}
        assert_copy::<ResolvedAttack>();
        assert_copy::<crate::ev::PointState>();
        assert_copy::<crate::schedule::ShotRec>();
        assert_copy::<crate::schedule::ReloadRec>();
        assert_copy::<crate::compile::MeleePlan>();
    }
}
