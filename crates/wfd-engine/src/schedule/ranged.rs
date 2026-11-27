//! Ranged schedule generator (06 E-6.11): shots, reloads, bursts, DUPLEX, spool, Kohm, Peacemaker, beam ramp,
//! charge, gauge-gated uses (E-6.14a with T-42), volley groups (06 §6.8), children (E-6.23-E-6.25) and sniper decay
//! (E-6.16), with every time test on the grid of CD-14. Owner: T08a.

use crate::StageCtx;
use crate::resolve::ResolvedAttack;
use crate::schedule::Schedule;

/// 06 E-6.11: the schedule of the firing attack `ra` (and its children) up to `horizon` = max(t_win, t_ttk_max).
/// Every input beyond the resolved timing values is in `ra.firing.sched` ([`crate::resolve::ScheduleInputs`]: timing
/// data and the 06 §5.9 buffs, reload style and lead, form, returning, volley, reload-triggered, spool, Kohm,
/// Peacemaker, sniper combo, flight time, child links); the roles and `ra.hit_fraction` complete them. A volley
/// group emits one shot record per member and pull (`VolleyInputs::attacks`, 06 E-6.14b). The caller has checked
/// `event_bound` (engine.md §15), so the generator never meets an unbounded schedule. T08a.
pub fn ranged_schedule(ctx: &StageCtx, ra: &ResolvedAttack, horizon: f64) -> Schedule {
    let _ = (ctx, ra, horizon);
    todo!("T08a")
}
