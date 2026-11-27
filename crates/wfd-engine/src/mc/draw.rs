//! Mapping draws to outcomes (08 E-8.17): Bernoulli, floor + Bernoulli, categorical in the enumeration order of
//! overview 2.2, uniform over a list. Owner: T16.

/// `Bernoulli(p)`: success iff u < p (p ≤ 0 never, p ≥ 1 always; D-017). T16.
pub fn bernoulli(u: f64, p: f64) -> bool {
    let _ = (u, p);
    todo!("T16")
}

/// `floor + Bernoulli(f)`: `floor(x) + [u < frac(x)]` (pellets, crit tier, proc count). T16.
pub fn floor_bernoulli(u: f64, x: f64) -> u32 {
    let _ = (u, x);
    todo!("T16")
}

/// `categorical(w)`: the first s with u < Σ_{t ≤ s} w_t; if rounding leaves u ≥ Σ w, the last s with w_s > 0. T16.
pub fn categorical(u: f64, w: &[f64]) -> usize {
    let _ = (u, w);
    todo!("T16")
}

/// Uniform over a list of length `len`: index floor(u·len). T16.
pub fn uniform_index(u: f64, len: usize) -> usize {
    let _ = (u, len);
    todo!("T16")
}
