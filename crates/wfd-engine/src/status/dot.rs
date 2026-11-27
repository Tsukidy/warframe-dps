//! DoT ticks (00 E-13; 03 E-3.4, E-3.5): weapon-side terms snapshotted at proc creation (CD-05), the "+1"
//! accumulator (D-025), tick counts `floor(6·(1 + SD + SD_s) + ε)` (CD-06), ELE/GAS tick groups, the Heat pool, Blast
//! fuses. Owner: T14.

use wfd_model::StatusType;

use crate::StageCtx;

/// Weapon-side terms of one DoT proc, snapshotted at its creation (E-13, CD-05).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProcSnapshot {
    /// seed_p = Y·(1 + F)·G·b·c_k·Part_dot (not quantized).
    pub seed: f64,
    /// E^dot_s, the elemental DoT scaling of s (01 E-1.15).
    pub elem_dot: f64,
    /// F_tick,p (the inducing instance's F; Heat inherit overrides it, 03 §10.3).
    pub f_tick: f64,
    /// A_sd.
    pub a_sd: f64,
    /// A_sd,s.
    pub a_sd_type: f64,
    /// V_sd,p, the status-damage vulnerability live at creation.
    pub v_sd: f64,
    /// M_OG,p of the source attack (overguard only).
    pub m_og: f64,
    /// Proc time t_p (s).
    pub t_proc: f64,
    /// Number of ticks N_s (CD-06).
    pub ticks: u32,
}

/// E-13 for a single proc of status `s`: the weapon-side tick `(seed_p + 1)·X_p` with
/// `X_p = K_s·(1 + E^dot_s)·(1 + F_tick,p)·(1 + A_sd + A_sd,s)·V_sd,p` (K_s from `ctx.rules`), before the
/// target-condition dips M_tick(τ) and the enemy side evaluated at tick time. T14.
pub fn dot_tick(ctx: &StageCtx, snap: &ProcSnapshot, s: StatusType) -> f64 {
    let _ = (ctx, snap, s);
    todo!("T14")
}
