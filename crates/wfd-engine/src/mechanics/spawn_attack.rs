//! The `spawn_attack` mechanic kind: 06 §9 and D-074 (Nightwatch Napalm, Volatile Quick Return: spawned child attacks).
//! Owner: T14.

/// The `spawn_attack` kind (domain-model.md §11.2).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SpawnAttack;

super::mechanic_skeleton!(SpawnAttack, "T14");
