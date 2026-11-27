//! The `repeat_hit` mechanic kind: 04 E-4.15 and 08 E-8.12 (Melee Duplicate on tier-1 crits; ids `dup/<name>`). Owner:
//! T14.

/// The `repeat_hit` kind (domain-model.md §11.2).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RepeatHit;

super::mechanic_skeleton!(RepeatHit, "T14");
