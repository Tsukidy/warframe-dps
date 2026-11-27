//! Extra hits (00 E-14; 04 §6, E-4.13-E-4.14): Xata's Whisper, Toxic Lash and the other `extra_hit` mechanics, one per
//! pellet of a parent direct or radial instance, never from DoT ticks, none after a kill. Owner: T14.

use wfd_model::{DamageType, DamageVec};

use crate::StageCtx;
use crate::resolve::{ExtraHitResolved, XhProc};

/// The weapon side of the parent hit an extra hit reads (E-14).
///
/// Two forms carry it (contract review ENG-R4, pass 3): [`crate::instance::InstanceOutcome::w`] holds it at crit
/// factor 1 (c = 1: `w` and `total` without c_k), while [`extra_hit`] expects the crit-scaled form below (W_t(k) =
/// c_k·W_t at c = 1). A caller in MC multiplies by the rolled c_k first; the EV kernel passes the c = 1 form and
/// scales the result by the parent's conditional crit factors (`DerivedHit::crit_scaled`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HitWeaponSide {
    /// W_t(k) per type (after quantization, crit and Part) in the crit-scaled form; at c = 1 in `InstanceOutcome`.
    pub w: DamageVec,
    /// H_w = Σ_t W_t(k).
    pub total: f64,
    /// F of the parent (the extra hit applies 1 + F again).
    pub faction: f64,
    /// Part of the parent hit (Xata applies it again).
    pub part: f64,
}

/// One extra hit (E-14).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExtraHitOut {
    /// XH = H_w·p_xh·(1 + F)·(Xata ? Part : 1), weapon side.
    pub damage: f64,
    /// Its single damage type e_xh.
    pub ty: DamageType,
    /// How it procs: rolls the weapon's SC (Xata: VOI), one forced proc (Toxic Lash: TOX, no "+1"), or none.
    /// Replaces `forced: Option<StatusType>` (contract review ENG-23).
    pub status: XhProc,
}

/// E-14: the extra hit of kind `xh` on the parent hit `parent`. T14.
pub fn extra_hit(ctx: &StageCtx, parent: &HitWeaponSide, xh: &ExtraHitResolved) -> ExtraHitOut {
    let _ = (ctx, parent, xh);
    todo!("T14")
}
