//! Context search (optimizer.md §6.1, §6.2 steps 1-3).
//!
//! Owner: T33. A complete build is `(context, main)`; the context is everything outside the 8 main slots (exilus,
//! weapon arcane(s), stance × combo × melee plan (T-30), Incarnon perk per tier, frame mods, aura, frame arcanes, shard
//! multiset, kitgun/zaw parts, progenitor element, buff scenario). Groups the request's `GroupSwitches` leave off stay
//! as in the input. Strategy per group as the §6.1 table (joint enumeration of perks, stance × combo, shards, parts;
//! a small branch-and-bound for frame mods). Algorithm: seed from the user's context; block-coordinate ascent (score
//! every option of a group with the current main fixed by `WindowFast`, re-solve main exactly for the top 3, accept the
//! best; repeat until no group changes); keep the `context_beam` best distinct contexts after each sweep. Proof scope
//! `ProvenOptimal { scope: MainGivenContext }` when the final context's branch-and-bound finished.
