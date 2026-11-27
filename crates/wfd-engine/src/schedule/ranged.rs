//! Ranged schedule generator (06 E-6.11): shots, reloads, bursts, DUPLEX, spool, Kohm, Peacemaker, beam ramp,
//! charge, gauge-gated uses (E-6.14a with T-42), volley groups (06 §6.8), children (E-6.23-E-6.25) and sniper decay
//! (E-6.16), with every time test on the grid of CD-14. Owner: T08a.

use crate::StageCtx;
use crate::resolve::ResolvedAttack;
use crate::schedule::Schedule;

/// 06 E-6.11: the schedule of the firing attack `ra` (and its children) up to `horizon` = max(t_win, t_ttk_max).
/// T08a.
pub fn ranged_schedule(ctx: &StageCtx, ra: &ResolvedAttack, horizon: f64) -> Schedule {
    let _ = (ctx, ra, horizon);
    todo!("T08a")
}
