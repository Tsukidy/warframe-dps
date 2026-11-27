//! The `WindowExact` walk (08 E-8.12; engine.md §7.2-§7.3): every shot and every DoT tick in [0, t_win] from the
//! fresh state, μ_s(τ) by a two-pointer sliding window per status, one walk for the four non-depleting views.
//! Owner: T15b.
