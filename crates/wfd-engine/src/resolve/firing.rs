//! Firing stats (06 §5, §10): fire rate and its locks, charge time, bursts, magazine, ammo cost and efficiency, reload,
//! duty, beam ramp, spool, gauge-gated attacks (E-6.14a with T-42), effective fire rate. Owner: T08a.

use wfd_model::{Attack, Trigger};

use crate::StageCtx;
use crate::assemble::AssembledWeapon;
use crate::stats::StatVecF;

/// Firing stats of one attack (06 §5).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FiringResolved {
    /// Trigger class (06 §4.2).
    pub trigger: Trigger,
    /// FR after mods and locks (06 §5.1).
    pub fire_rate: f64,
    /// Effective fire rate EFR (06 §10).
    pub efr: f64,
    /// Charge time CT (06 §5.3); 0 for attacks without charge.
    pub charge_time: f64,
    /// Shots per burst (1 without bursts).
    pub burst_count: u16,
    /// Delay between the shots of a burst (s).
    pub burst_delay: f64,
    /// Magazine N_mag (06 §5.5); `f64::INFINITY` without a magazine.
    pub magazine: f64,
    /// Ammo cost per shot (06 §5.6).
    pub ammo_cost: f64,
    /// Ammo efficiency AE (clamped, 06 §5.6).
    pub ammo_eff: f64,
    /// Reload time T_rel (06 §5.7).
    pub reload_time: f64,
    /// Magazine time T_mag (06 §10).
    pub t_mag: f64,
    /// Duty ρ = T_mag / (T_mag + T_rel) (06 §10).
    pub duty: f64,
    /// Beam ramp time (s; 0 = none).
    pub ramp_time: f64,
    /// Spool shots (0 = none, 06 §5.2).
    pub spool_shots: u32,
    /// Gauge-gated attack (06 §6.7, E-6.14a: no magazine cost, no steady metrics).
    pub gauge_gated: bool,
    /// T-42 gauge uses per minute u (set at L3 from the scenario; 0 = the first use only).
    pub gauge_uses_per_min: f64,
}

/// 06 §5: firing stats of `atk` on the assembled weapon with the stat vector `sv`. T08a.
pub fn firing_stats(
    ctx: &StageCtx,
    atk: &Attack,
    w: &AssembledWeapon,
    sv: &StatVecF,
) -> FiringResolved {
    let _ = (ctx, atk, w, sv);
    todo!("T08a")
}
