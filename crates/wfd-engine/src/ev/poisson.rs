//! Capped Poisson expectations (08 E-8.3; A-3, A-4): `E[f(n)]` by a capped Poisson sum (one `exp` plus a
//! recurrence), never `f(E[n])` (CD-13). Owner: T15a.

/// 08 E-8.3: `E[min(N, cap)]` for N ~ Poisson(μ). T15a.
pub fn capped_poisson_mean(mu: f64, cap: u8) -> f64 {
    let _ = (mu, cap);
    todo!("T15a")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "T15a"]
    fn tv_08_04_capped_mean() {
        // TV-08-04: μ = 9.254705777777776, cap 5: E[n] = 4.929175484660303
        let e = capped_poisson_mean(9.254705777777776, 5);
        assert!((e - 4.929175484660303).abs() <= 1e-9 * 4.929175484660303);
        assert!(capped_poisson_mean(0.0, 10).abs() < 1e-300);
    }
}
