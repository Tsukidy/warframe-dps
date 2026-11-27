//! ST-2 composition (00 E-2; 01 §4.9-§4.12: physical, stance physical, Phoenix Talons, conversions after the physical
//! step, elements, the weakness-element map of Table 4.10-A). 01 §4.1: steps 9-10 are repeated per stance hit when
//! S^phys_t differs (EC-01-34) and per target layer when a target-dependent effect is active (EC-01-38, EC-01-39), so
//! L3 keeps the inputs ([`ComposeInputs`]) and the evaluators recompose per strike with [`compose`] (contract review
//! ENG-4). Owner: T13.

use wfd_model::{DamageType, DamageTypeSet, DamageVec, N_ST, StatusSet};

use crate::StageCtx;
use crate::elements::ElemOutcome;
use crate::resolve::base::BaseVector;
use crate::stats::StatVecF;

/// 01 §4.11 element combination, the stage form of [`crate::elements::combine`] (one implementation, T06).
pub use crate::elements::combine as combine_elements;

/// Largest number of `convert_damage` mechanics acting after the physical step on one attack (01 E-1.9;
/// domain-model.md §11.2); a build with more on one attack is rejected at L2 (`EngineError::UnsupportedMechanic`).
pub const MAX_CONVERSIONS: usize = 2;

/// One `convert_damage` mechanic of the attack (domain-model.md §11.2; 01 E-1.9, D-009, D-101), resolved at L1: the
/// `param` target applied, the scope checked.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ConvResolved {
    /// The converted types (`from`; `all` = every type).
    pub from: DamageTypeSet,
    /// The target.
    pub to: ConvTo,
    /// The fraction converted (default 1; 0 = an empty entry).
    pub fraction: f64,
}

/// Target of a conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ConvTo {
    /// A fixed type (resolved from a type or an item parameter).
    Type(DamageType),
    /// The target's weakness element for conversions on this layer ([`TargetWeakness::convert_to`]; Reactive
    /// Storm, Table 4.10-A).
    #[default]
    TargetWeakness,
}

/// The target-dependent inputs of 01 E-1.9 for one target layer (Table 4.10-A, keyed on 05's `damage_faction`):
/// T07 and L3 fill one for the non-overguard layers and one for the overguard layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TargetWeakness {
    /// The type a `target_weakness` conversion converts to on this layer; `None` = the conversion is skipped (the
    /// weakness is Impact, the overguard layer is present, or the health type is not listed: Reactive Storm "stays
    /// IMP", EC-01-38, D-101).
    pub convert_to: Option<DamageType>,
    /// The type `target_weakness` element entries take on this layer (`ElemOutcome.weakness`); `None` = no element
    /// (an unlisted health type: Sentient Incision, D-102, OQ-110). On the overguard layer COR (EC-01-39).
    pub element: Option<DamageType>,
}

/// The inputs of 01 E-1.6-E-1.9 of one attack (fixed size, `Copy`), kept in `ResolvedAttack.compose` so the
/// composition can be recomputed per stance strike and per target layer without the stat vector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComposeInputs {
    /// B'_t / B' per type (E-1.6, E-1.9).
    pub share: DamageVec,
    /// P_t, the physical bonuses (IMP, PUN, SLA; E-1.6).
    pub phys: [f64; 3],
    /// pt, Phoenix Talons (E-1.7).
    pub pt: f64,
    /// E_t per type after the element combination of the attack, `target_weakness` entries excluded
    /// (`ElemOutcome.sums`).
    pub elem: DamageVec,
    /// Σ of the `target_weakness` element entries (`ElemOutcome.weakness`).
    pub weakness: f64,
    /// The `convert_damage` mechanics acting after the physical step (`conv[..n_conv]`).
    pub conv: [ConvResolved; MAX_CONVERSIONS],
    /// Number of live entries of `conv`.
    pub n_conv: u8,
    /// Table 4.10-A on the layers other than overguard (index 0) and on the overguard layer (index 1).
    pub layers: [TargetWeakness; 2],
}

/// One composition (01 E-1.9, E-1.10; 00 E-9): r_t, q_t and the proc weights w_s.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Composition {
    /// r_t (E-2).
    pub ratio: DamageVec,
    /// q_t (E-3).
    pub ratio_q: DamageVec,
    /// w_s (E-9).
    pub w: [f64; N_ST],
}

/// E-2 / 01 E-1.9: the composition ratios r_t from the base vector, the physical buckets of `sv`, the stance physical
/// multipliers `stance_phys` (S^phys_t of IMP, PUN, SLA; 07 §7.2), the element outcome, the conversions acting after
/// the physical step `conv` (Reactive Storm) and the target-dependent inputs `tw` of the layer (Table 4.10-A). The
/// `conv` and `tw` parameters are additions (contract review ENG-4). T13.
pub fn ratios(
    ctx: &StageCtx,
    base: &BaseVector,
    sv: &StatVecF,
    stance_phys: [f64; 3],
    el: &ElemOutcome,
    conv: &[ConvResolved],
    tw: &TargetWeakness,
) -> DamageVec {
    let _ = (ctx, base, sv, stance_phys, el, conv, tw);
    todo!("T13")
}

/// 01 E-1.9, E-1.10 and 00 E-9 from the kept inputs: the composition of one stance strike with the physical
/// multipliers `stance_phys` (07 §8.2 `MeleeHit.s_phys`; `[0.0; 3]` for an attack without stance physical bonus) on
/// the overguard layer (`og`) or the others, with the target's status immunities `immune`. The evaluators call it
/// per strike when `stance_phys` differs from the attack's own and per layer (contract review ENG-4). T13.
pub fn compose(
    ctx: &StageCtx,
    c: &ComposeInputs,
    stance_phys: [f64; 3],
    og: bool,
    immune: StatusSet,
) -> Composition {
    let _ = (ctx, c, stance_phys, og, immune);
    todo!("T13")
}
