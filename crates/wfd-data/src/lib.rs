//! `wfd-data`: INTERIM file carried by T02d, not T02b's skeleton (CONTRACTS.md, "2026-11-27 · T02d").
//!
//! T02b writes this crate's skeleton in parallel with T02d, and `wfd-opt::optimize` (work-breakdown.md T02d) names
//! `GameData`. So that `wfd-opt` builds on T02d's branch, this file declares that one name and nothing else. **On
//! merge, T02b's version of this file replaces this one entirely** (the owner's file wins, work-breakdown.md §8);
//! `wfd-opt` and `wfd-server` use only the type name `GameData` from this crate.

/// The loaded game data: bundle and indexes (work-breakdown.md T02b). Interim declaration (T02d).
pub struct GameData {
    _private: (),
}
