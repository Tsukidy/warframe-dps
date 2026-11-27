//! The `Bound` fidelity (engine.md §7.2): per interval of 8, the exact shot count × the kernel at an upper-envelope
//! state, plus an upper bound of the DoT ticks landing in the interval; provably ≥ `WindowExact` when the kernel is
//! non-decreasing in every μ_s; property-tested (testing.md §2.3). Owner: T15c.
