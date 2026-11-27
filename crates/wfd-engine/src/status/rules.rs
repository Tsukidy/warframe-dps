//! The status rules of 03 §4.1 from `Rules.status` (durations, caps, DoT coefficients K_s, tick offsets δ_s,
//! stacking kinds), with SD and SD_s applied (CD-06, 03 E-3.1) and enemy caps (03 E-3.2). Owner: T14.

/// The stored stack cap of a status without a cap: `cap_s = ∞` (03 §4.1 base cap ∞ of SLA, HEA (pool), ELE, TOX,
/// when neither the enemy (05 §4.4) nor E-3.2 lowers it). Every finite cap is far below it (the largest is COR's
/// `10 + β_COR`), so `cap == CAP_UNBOUNDED` is the only test for "uncapped" in `StatusResolved.caps`,
/// `ScaledEnemy.caps`, the EV capped sums (08 E-8.3) and MC insertion (03 §6.2).
pub const CAP_UNBOUNDED: u8 = u8::MAX;
