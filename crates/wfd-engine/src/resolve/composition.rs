//! ST-2 composition (00 E-2; 01 §4.9-§4.12: physical, stance physical, Phoenix Talons, elements). Owner: T13.

use wfd_model::DamageVec;

use crate::StageCtx;
use crate::elements::ElemOutcome;
use crate::resolve::base::BaseVector;
use crate::stats::StatVecF;

/// 01 §4.11 element combination, the stage form of [`crate::elements::combine`] (one implementation, T06).
pub use crate::elements::combine as combine_elements;

/// E-2: the composition ratios r_t from the base vector, the physical buckets of `sv`, the stance physical
/// multipliers `stance_phys` (IMP, PUN, SLA; 07) and the element outcome. T13.
pub fn ratios(
    ctx: &StageCtx,
    base: &BaseVector,
    sv: &StatVecF,
    stance_phys: [f64; 3],
    el: &ElemOutcome,
) -> DamageVec {
    let _ = (ctx, base, sv, stance_phys, el);
    todo!("T13")
}
