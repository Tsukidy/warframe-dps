//! ST-5 weapon multipliers (01 §4.15 faction F; 04 §3 multiplier groups G, G_F, M_OG and the other stages of
//! `Rules.mult_groups`, domain-model.md §14). Owner: T13.

use crate::StageCtx;
use crate::compile::scenario::MultSlotTable;
use crate::stats::StatVecF;

/// The static weapon-side multipliers of one attack.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeaponMult {
    /// F, the faction bucket (applied as 1 + F).
    pub faction: f64,
    /// Π G (groups of stage `G`).
    pub g: f64,
    /// Π G_F (groups of stage `GF`).
    pub g_f: f64,
    /// M_OG (stage `overguard_only`).
    pub m_og: f64,
    /// Π of stage `dot_only` groups (M_dot parts of the weapon side).
    pub dot_only: f64,
}

/// ST-5: the static multipliers from the `mult.<n>` slots of `sv` and the faction bucket. T13.
pub fn weapon_mult(ctx: &StageCtx, sv: &StatVecF, slots: &MultSlotTable) -> WeaponMult {
    let _ = (ctx, sv, slots);
    todo!("T13")
}
