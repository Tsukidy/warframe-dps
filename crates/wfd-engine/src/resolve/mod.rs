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
pub use composition::{
    ComposeInputs, Composition, ConvResolved, ConvTo, MAX_CONVERSIONS, TargetWeakness,
    combine_elements, compose, ratios,
};
pub use crit::{
    CritMult, TierDist, crit_chance, crit_multiplier, crit_multiplier_terms, crit_tiers, doughty,
};
pub use firing::{ChildLink, FiringResolved, ScheduleInputs, firing_stats};
pub use melee::{MeleeResolved, melee_resolved};
pub use multishot::multishot;
pub use part::part_factor;
pub use quantize::quantize;
pub use scalar::{ABuckets, ScalarY, base_scalar};
pub use status_chance::{proc_weights, status_chance};
pub use weapon_mult::{WeaponMult, weapon_mult};

use wfd_model::{
    AttackIdx, AttackRoles, CoBehaviour, CoCountMode, CoFlags, DamageType, DamageVec, Layer,
    MprojMode, N_ST, StatusSet, StatusType,
};

use crate::enemy::ArmorStrips;

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
    /// r_t (E-2), on every layer but overguard (the `health`, `armor`, `shield` views; 01 E-1.9 with the
    /// non-overguard row of Table 4.10-A).
    pub ratio: DamageVec,
    /// q_t (E-3), as `ratio`.
    pub ratio_q: DamageVec,
    /// q_t on the overguard layer when a target-dependent effect makes it differ (01 §4.1 steps 9-10 "per target
    /// layer"; Table 4.10-A: Reactive Storm stays IMP, Sentient Incision becomes COR while overguard is present;
    /// EC-01-38, EC-01-39); `None` = `ratio_q`. Its proc weights are `status.w_og`. Addition, contract review ENG-4.
    pub ratio_q_og: Option<DamageVec>,
    /// The inputs of the composition, for the per-strike and per-layer recomputation of E-1.9 ([`compose`]: a
    /// stance hit whose S^phys_{j,t} differs from the attack's, 07 §7.2, EC-01-34). Addition, contract review ENG-4.
    pub compose: ComposeInputs,
    /// A, the base-damage bucket (E-11); on melee-projectile attacks the melee-damage bucket A_m (04 §3.4).
    pub a: f64,
    /// A_w, the universal weapon-damage bucket of melee-projectile attacks (`base_dmg_universal`; 04 §3.4, E-4.5,
    /// 01 E-1.11); 0 otherwise. Addition, contract review ENG-6.
    pub a_w: f64,
    /// C as affine terms with the CO behaviour, ρ_co, B_co (E-11, 04 §2).
    pub co: CoResolved,
    /// Π G_F (static part; schedule-driven factors are applied per shot).
    pub g_f: f64,
    /// F, the faction bucket (01 §4.15).
    pub faction: f64,
    /// Π G (static part; 04 §3).
    pub g: f64,
    /// Π of the G members scoped to pellets j ≥ 1 of a shot (`Scope.hit.pellet = ExtraOnly`: the Braton Incarnon "+Y
    /// on multishot pellets", 01 §4.8 a G member; domain-model.md §11.1); 1 without one. Applied with `g` on those
    /// pellets. Addition, contract review ENG-5.
    pub g_extra_pellets: f64,
    /// M_OG, the weapon-side overguard factor (stage `overguard_only`; 05 E-5.17), applied on the OG layer and
    /// snapshotted in `ProcSnapshot.m_og`. Addition, contract review ENG-5.
    pub m_og: f64,
    /// Π of the stage `dot_only` groups (M_dot parts of the weapon side, 03 §9; 00 E-13). Addition, contract review
    /// ENG-5.
    pub dot_only: f64,
    /// The build's target-side terms: vulnerabilities and armor strips (05 E-5.10, E-5.11, E-5.19). Addition,
    /// contract review ENG-5, M-4.
    pub target: TargetTerms,
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
    /// FR, EFR, CT, burst, magazine, cost, AE, reload, duty, ramp, spool, gauge (06 §5), and every other input of
    /// the ranged schedule ([`ScheduleInputs`]).
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

/// The target-side terms a build contributes (05 E-5.10, E-5.11, E-5.19), from the `vuln_add`, `mult` (stage
/// `target_vuln`) and `strip_*` rows of the attack's stat vector (domain-model.md §10). Abilities of a buff whose
/// source is a Warframe ability are already dropped against `ability_debuff_immune = all` targets (05 §13.1, at
/// lowering).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TargetTerms {
    /// V_ℓ,t of 05 E-5.19 with both groups folded: `(1 + Σ vuln_add_ℓ,t)·Π_g (1 + v_g)` over the `target_vuln` groups
    /// that cover (ℓ, t), indexed by `Layer::index` (what `TargetStateView.vuln` reads).
    pub vuln: [DamageVec; Layer::N],
    /// The %-strips of Σ_add and the point strips of E-5.10: `ability` (`strip_ability`, Strength applied),
    /// `aura` (`strip_aura`·aura_str), `companion` (`strip_companion`), `points` (`strip_points`, P_pts), `toggle`
    /// (T-07, set at L3 from the scenario); `flat` = 1 (f_flat is target state, from s_flat).
    pub strips: ArmorStrips,
    /// Base-armor points removed per qualifying hit (`strip_flat_per_hit`; 05 E-5.11, the qualifying-hit rule of
    /// 05 §9.4 in the evaluator), accumulated into s_flat.
    pub flat_per_hit: f64,
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
    /// The catalogue flags of 04 §2.14 (`direct_target_only`, `no_combo` = `co_omit = {combo}`, `co_dot_mode`:
    /// `NO_DOT_CO` = exclude, `DOT_RECALC_PER_TICK` = per_tick; 04 E-4.3, E-4.7). Addition, contract review ENG-6.
    pub flags: CoFlags,
    /// The melee-projectile bucket mode (04 §3.4; selects the form of 01 E-1.11 and the two-bucket β of 04 E-4.5).
    /// Addition, contract review ENG-6.
    pub mproj: MprojMode,
    /// C_x, the part of the constant C whose entries exclude the Incarnon flat on a `cond_flat_excluded` attack (04
    /// E-4.3 row 36, OQ-442; health-threshold entries, constants per 08 §4.5); 0 otherwise. Addition, contract review
    /// ENG-6.
    pub c_x: f64,
    /// T-05 count mode of n (04 E-4.1, E-4.2; D-056): `expected` (the evaluators' distinct count), `weapon_types`
    /// or `manual`. Addition, contract review ENG-13.
    pub count: CoCountMode,
    /// T-05 `manual` N (1 … 50); 0 in the other modes.
    pub count_manual: u8,
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
    /// The quantized base crit multiplier without the Prelude term (NP-1; [`crit_multiplier_terms`] with
    /// `cdm0c` = 0).
    pub q: CritMult,
    /// The quantized base crit multiplier with the Prelude term CDM0c as the last f32 addend (02 E-2.5, NP-1;
    /// [`crit_multiplier_terms`]); equal to `q` without the perk.
    pub q_cond: CritMult,
    /// Prelude threshold θ_P (strict); `None` without the perk.
    pub theta_p: Option<f64>,
    /// ΣCD_rel.
    pub cd_rel: f64,
    /// ΣCD_abs,static (without Cold and without `doughty`, which 02 E-2.5 adds per instance; ruling C-37).
    pub cd_abs: f64,
    /// M_cdf.
    pub m_cdf: f64,
    /// p_vig (Vigilante).
    pub p_vig: f64,
    /// b_r of Melee Doughty (`doughty_rank`, 02 E-2.5a); 0 without it. Addition, contract review ENG-19, C-37.
    pub doughty_rank: f64,
    /// `doughty` of 02 E-2.5a at the attack's static composition and weapon-side SC ([`doughty`]); the evaluators
    /// recompute it per instance when SC_w has combo-dependent or heavy-only terms or the strike composition differs
    /// (E-2.5a "for this hit"). Never part of `cd_abs`: E-2.5 adds it as its own term of ΣCD_abs,i, and the D-014
    /// compat path adds it unrounded after the head multiplier. Addition, contract review ENG-19, C-37.
    pub doughty: f64,
}

/// Status terms of the attack (E-8, E-9, E-13; 03 §4.1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatusResolved {
    /// SC per pellet (E-8) at zero Tau stacks: SC_w·Π_v (1 + v) over the static target-side factors (02 E-2.11).
    pub sc: f64,
    /// SC_w, the weapon side of 02 E-2.11 (without M_sc,t), which 02 E-2.5a (`doughty`) reads. Addition, contract
    /// review ENG-19.
    pub sc_w: f64,
    /// Proc weights w_s (E-9), as `ResolvedAttack.ratio_q`.
    pub w: [f64; N_ST],
    /// Proc weights on the overguard layer, from `ResolvedAttack.ratio_q_og`; `None` = `w`. Addition, contract
    /// review ENG-4.
    pub w_og: Option<[f64; N_ST]>,
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
    /// Stack caps cap_s with the build's increases (E-3.2; enemy caps applied at L0) with `og_present` false;
    /// [`crate::status::CAP_UNBOUNDED`] = no cap (03 §4.1 base cap ∞ of SLA, HEA, ELE, TOX).
    pub caps: [u8; N_ST],
    /// The caps with `og_present` true (03 E-3.2: Cold 4, no Frozen; VIP Puncture 3; Evensong Puncture 5): the
    /// `overguard` view's timeline (08 §4.5, E-8.4) and the layered passes while OG > 0. Equal to `caps` on an enemy
    /// without overguard. Addition, contract review ENG-10.
    pub caps_og: [u8; N_ST],
    /// DoT coefficient K_s·(1 + E^dot_s) per DoT status (E-13).
    pub dot_k: [f64; N_ST],
    /// V_sd of 00 E-13 from the build (`dot_vuln`, custom p.o; the target state's part is live), snapshotted into
    /// `ProcSnapshot.v_sd`. Addition, contract review ENG-5.
    pub dot_vuln: f64,
    /// Heat inherit (`heat_inherit`, D-028, 03 §10.3). Addition, contract review ENG-5.
    pub heat_inherit: f64,
    /// F_inh (`heat_inherit_faction`, 03 §10.3: F_tick,p = F_inh of the external first Heat source; TV-03-33).
    /// Addition, contract review ENG-5 (the row of M-2).
    pub heat_inherit_faction: f64,
    /// External stacks N_ext,s (T-43 / T-06 and `external.*` stats, max).
    pub n_ext: [u8; N_ST],
}

/// One extra-hit kind (E-14, 04 §6; the `extra_hit` parameters of domain-model.md §11.2).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ExtraHitResolved {
    /// p_xh (0 = kind absent), with `melee_factor` applied on melee attacks (Toxic Lash 2.0).
    pub p: f64,
    /// The damage type e_xh.
    pub ty: Option<DamageType>,
    /// The kind multiplies by Part (Xata's Whisper).
    pub part: bool,
    /// How the extra hit procs (`status`: `roll_weapon_sc` (Xata: VOI by the weapon's SC), `forced` (Toxic Lash:
    /// TOX), `none`). Replaces `forced: Option<StatusType>`, whose `None` could not say "no status" (contract review
    /// ENG-23).
    pub status: XhProc,
    /// Status duration added to the extra hit's own procs (`status_duration_add`; no catalogue user, 04 E-4.14).
    pub dur_add: f64,
    /// Triggered by each pellet hit of a parent direct or radial instance (`triggers` `pellet_hit`).
    pub on_pellet: bool,
    /// Triggered by a Blast detonation (`triggers` `blast_detonation`, 04 E-4.13a: the detonation extra hit with
    /// the `"xd/"` ids of 08 E-8.16). Addition, contract review ENG-23.
    pub on_detonation: bool,
}

/// The status behaviour of an extra hit (04 §6; model `XhStatus` with the forced status resolved from e_xh).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum XhProc {
    /// Rolls the weapon's status chance (Xata's Whisper: a Void proc, 04 E-4.13).
    #[default]
    Roll,
    /// One forced proc of the status (Toxic Lash: TOX, with no "+1" on its DoT, 00 E-14).
    Forced(StatusType),
    /// No status.
    None,
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
    /// Condition ids (`CompiledScenario.conditions`) of the gates split exactly.
    pub ids: [u16; MAX_SPLIT_GATES],
    /// Number of gates in `ids`.
    pub len: u8,
    /// More gates existed and were linearized (`ev.approx.gate_split_limit`).
    pub linearized: bool,
}

/// One resolution of an attack's dynamic gates for `Engine::resolve_point` (engine.md §7.4; contract review ENG-8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GateAssign {
    /// Every dynamic gate at its mean (linearized): the calculator's default and every evaluation without gate
    /// splitting; under `all_max` and `none` every gate is constant, so this is exact.
    #[default]
    Mean,
    /// Gate `GateSet::ids[k]` of the attack is on iff bit k is set (k < `MAX_SPLIT_GATES`); the gates beyond the
    /// split limit stay at their mean.
    Fixed(u8),
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
