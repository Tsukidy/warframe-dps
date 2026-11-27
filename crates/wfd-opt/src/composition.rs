//! Composition sets (optimizer.md §3, §5.1).
//!
//! Owner: T32. Main candidates that touch an `Unknown` slot of the compiled scenario (`CompiledScenario.slot_class`:
//! elements, physical mods, conversions, forced procs, mechanics that change proc weights, and every candidate whose
//! slots a reader downgraded) form composition sets: subsets of size 0..k respecting incompatibilities, deduplicated by
//! their element outcome and physical/forced vector. `ElementSolver::outcomes` (engine.md §3) expands each set to every
//! distinct element outcome with a slot ordering that realizes it, so element order is searched exactly. Progenitor
//! element (if not kept) multiplies the sets by up to 7. Element rules (`require`, `forbid`) are checked on outcomes
//! before any inner search; sets are processed in descending order of their root bound. Coupled candidates (set
//! members, `Dec` slots, non-monotone readers) are branched include/exclude first (§3).

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "T32: every distinct element outcome of a set is enumerated once (optimizer.md §5.1)"]
    fn composition_enumerates_every_element_outcome() {
        todo!("T32")
    }
}
