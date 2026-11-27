//! The `stack_present_statuses` mechanic kind: 04 §6.6 E-4.20 (Melee Afflictions: N stacks of every status present on
//! a knocked-down, lifted or ragdolled target, with the averaged seed). Kind added by the contract review (DC-4,
//! CONTRACTS.md). Owner: T14.

/// The `stack_present_statuses` kind (domain-model.md §11.2).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StackPresentStatuses;

super::mechanic_skeleton!(StackPresentStatuses, "T14");
