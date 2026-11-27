//! Dominance inside a pool (optimizer.md §2 step 7).
//!
//! Owner: T32. Candidate B is dropped if some candidate A has `contrib(A) ≥ contrib(B)` component-wise on every `Inc`
//! slot, equal `Unknown` slots, elements and mechanics, the same set membership, no larger drain under a capacity rule,
//! and an incompatibility mask that is a subset of B's (so A is available wherever B is). Dropped items are reported as
//! `opt.excluded.dominated` with the dominating item.

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "T32: dominance never drops a candidate of the brute-force argmax (optimizer.md §2 step 7)"]
    fn dominance_keeps_the_brute_force_optimum() {
        todo!("T32")
    }
}
