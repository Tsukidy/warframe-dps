//! Canonical build keys and the memo cache (optimizer.md §8).
//!
//! Owner: T32. The canonical build key is xxh3-128 of (sorted non-element item ids, element outcome with realized
//! order, context positions); it orders equal scores (`(Score desc, canonical key asc)`) and keys the memo cache
//! (`FxHashMap<u128, f64>`, per work item, LRU 1 M entries), which stores f64 scores (no rounding) and is scoped to one
//! `CompiledScenario`, so it never serves a stale score (`I:opt` Q12) and a hit returns exactly what a recomputation
//! would.
//!
//! The scenario is never part of a key: the cache's scope is the compiled scenario itself. The manager's ruling of
//! 2026-11-27 (CONTRACTS.md, "Rulings on T01d") applies to any key that does include it: the resolved `Scenario` is
//! not postcard-safe (`Toggles` holds `skip_serializing_if` fields), so such a key must make the encoding safe first or
//! hash another, positional encoding, and record the choice in CONTRACTS.md.

/// A canonical build key (xxh3-128, §8).
pub type CanonKey = u128;
