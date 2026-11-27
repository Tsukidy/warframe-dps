//! Formula constants of the spec (overview.md I-1): game numbers come from the bundle; the engine holds only the
//! constants that the spec's equations themselves contain, each with its equation number. `xtask lint-engine` rejects
//! float literals other than 0, 1, 2 and 0.5 in every other engine source file. Owner: T13 (each implementing task
//! adds the constants its equations need here, with the E-number).

/// Quantization grid of the composition ratios: `q_t = sign(r_t)·floor(|r_t|·32 + 0.5)/32` (00 E-3, CD-02).
pub const QUANT_GRID: f64 = 32.0;

/// Clamp of the base crit multiplier before quantization: `clamp(x, 0, 32)` (00 E-4, 02 E-2.5, NP-1).
pub const CDM_CLAMP: f32 = 32.0;

/// Largest crit-multiplier code: `code = floor(clamp(x, 0, 32)·4095/32 + 0.5)` (00 E-4, 02 E-2.5, NP-1).
pub const CDM_CODES: u32 = 4095;

/// ε = 1e-9 of the arithmetic guards, never a time tolerance: the DoT tick count `floor(6·(1 + SD + SD_s) + ε)`
/// (CD-06, 03 E-3.1) and the ammo and magazine arithmetic of 06 (E-6.6 magazine rounding, E-6.8 `can_fire`,
/// `from_empty` a_left ≤ ε, the HUD counter of E-6.13, the repetition test of E-6.28; 06 E-6.11 "1e-9 elsewhere in
/// this document is not a time tolerance"). Time tests ("τ ≤ t_win + ε" and every other equality, ordering, expiry
/// and window test) are exact integer comparisons on the grid of [`TIME_GRID_PER_S`] (CD-14, 08 E-8.0a).
pub const ARITH_EPS: f64 = 1e-9;

/// Time grid of both evaluators and of the schedule tests: `τ̂ = round_half_even(τ·10^6)` µs (08 E-8.0a, CD-14,
/// 06 E-6.11, 03 §7). Every time equality, ordering, expiry and window test compares grid values.
pub const TIME_GRID_PER_S: f64 = 1e6;

/// z-value of the EV-MC consistency test: pass iff `|dps_window − x̄| ≤ 2.5758293035489004·SE + 0.02·|x̄|` (08 E-8.24).
pub const DEVIATION_Z: f64 = 2.5758293035489004;

/// Relative term of the EV-MC consistency test (08 E-8.24).
pub const DEVIATION_REL: f64 = 0.02;

/// Minimum MC runs for the EV-MC consistency test (08 E-8.24, "MC with n_runs ≥ 1000").
pub const DEVIATION_MIN_RUNS: u32 = 1000;

/// Display quantum q of the OC2 comparison for the DPS and per-bullet values (08 E-8.25).
pub const OC2_QUANTUM_DPS: f64 = 1.0;

/// Display quantum q of the OC2 comparison for the TTK median, 0.001 s (`toFixed(3)`, 08 E-8.25).
pub const OC2_QUANTUM_TTK: f64 = 0.001;
