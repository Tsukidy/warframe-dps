//! Positions and pools (optimizer.md §2 step 2).
//!
//! Owner: T32. Positions: 8 main slots (one shared pool, no duplicates), exilus, weapon arcane(s), stance × combo,
//! Incarnon tiers, frame mods, aura, frame arcanes, shard multiset, kitgun/zaw parts, progenitor element, buff scenario
//! (`OptGroup` of `wfd_model::api`). Locked slots become fixed. Eligibility per position comes from
//! `wfd-data::legality` (spec 09; the same code the UI and the decoder use): slot kind, categories and tags, forbidding
//! tags, Semi-only trigger, incompatibility (both directions of the wiki lists plus the hidden-parent rule, D-093),
//! rank and capacity feasibility, `prohibited`, `must_include`. Under-ranked versions of a mod (`ranks = allow_lower`,
//! T-36) are extra candidates in one incompatibility group. User-supplied items (riven, valence, locked slots) are never
//! rewritten (09 §1.5).
