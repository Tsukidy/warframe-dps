//! The `counter_bonus` mechanic kind: 04 §5.4, §5.12 and 07 §7.5 (Secondary Enervate, Melee Animosity, Power Throw;
//! `unbounded` counters of ruling C-4). Owner: T14.

/// The `counter_bonus` kind (domain-model.md §11.2).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CounterBonus;

super::mechanic_skeleton!(CounterBonus, "T14");
