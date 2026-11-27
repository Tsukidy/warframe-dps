//! The `random_multiplier` mechanic kind: 04 §3.2 (Devouring Attrition: a chance of a multiplier on tier-0 instances).
//! Owner: T14.

/// The `random_multiplier` kind (domain-model.md §11.2).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RandomMultiplier;

super::mechanic_skeleton!(RandomMultiplier, "T14");
