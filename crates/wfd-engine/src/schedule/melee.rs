//! Melee schedule generator (07 E-7.30) for one [`MeleePlan`] (07 §8.1): strikes from the resolved combo tables
//! (E-7.0b), timing from AS, W and T_sw (07 §5), the expected-path combo count c̄ with gains, expiry, Power Spike
//! decay, heavy spend and Initial Combo regeneration (07 §6), heavy hit lists H_j (E-7.40), 9-C spawned attacks,
//! Melee Duplicate and wave insertion order (07 §8.4), the records of 07 §8.2 and the steady quantities of E-7.37.
//! Owner: T08b.

use wfd_model::MeleeTables;

use crate::StageCtx;
use crate::compile::melee::MeleePlan;
use crate::resolve::ResolvedAttack;
use crate::schedule::Schedule;

/// Heavy and combo records of 07 §8.2.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MeleeTrack {
    /// One record per heavy attack.
    pub heavies: Vec<HeavyRec>,
    /// Every expected-path change of the combo count.
    pub combo: Vec<ComboRec>,
}

/// A heavy attack (07 §8.2 `heavy` record).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeavyRec {
    /// Input time (s).
    pub input: f64,
    /// Execution time (s).
    pub exec: f64,
    /// Heavy index h ∈ {1, 2}.
    pub h: u8,
    /// Tennokai heavy.
    pub tennokai: bool,
    /// c̄⁻ before the heavy.
    pub combo_before: f64,
    /// c̄⁺ after the heavy.
    pub combo_after: f64,
    /// M_h, the combo multiplier the heavy uses.
    pub m_h: f64,
}

/// One expected-path change of the combo count (07 §8.2 `combo` record).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComboRec {
    /// Time (s).
    pub t: f64,
    /// Kind of change.
    pub kind: ComboChange,
    /// c̄ after the change.
    pub combo: f64,
    /// M before the change.
    pub m_old: f64,
    /// M after the change.
    pub m_new: f64,
}

/// Kinds of combo changes (07 §8.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComboChange {
    /// Gain from a hit.
    Gain,
    /// Timer expiry (E-7.14).
    Expiry,
    /// Power Spike decay step (E-7.14).
    Decay,
    /// Heavy spend (E-7.16).
    Spend,
    /// Initial Combo regeneration starts (E-7.15).
    RegenStart,
    /// Initial Combo regeneration ends.
    RegenEnd,
    /// Tier increase.
    Tier,
}

/// 07 E-7.30: the schedule of melee attack `ra` under `plan` with the resolved `tables` up to `horizon`. The start
/// count is E-7.31's c(0) = IC, or max(IC, k_tier·(M_start − 1)) when `plan.m_start` is set (T-09), with IC and
/// k_tier from `ra.melee` (contract review ENG-14); `plan.light_combo_fixed` (T-10) fixes M and M_b of the
/// light-type instances while the count still evolves (E-7.31, E-7.18). Every hit record carries its
/// `ShotRec::melee` fields (07 §8.2). The caller has checked `event_bound`. T08b.
pub fn melee_schedule(
    ctx: &StageCtx,
    ra: &ResolvedAttack,
    plan: &MeleePlan,
    tables: &MeleeTables,
    horizon: f64,
) -> Schedule {
    let _ = (ctx, ra, plan, tables, horizon);
    todo!("T08b")
}
