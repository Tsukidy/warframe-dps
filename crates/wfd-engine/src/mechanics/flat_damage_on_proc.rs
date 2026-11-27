//! The `flat_damage_on_proc` mechanic kind: 04 §5.4 (Cascadia Empowered: flat damage per status). Owner: T14.

/// The `flat_damage_on_proc` kind (domain-model.md §11.2).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FlatDamageOnProc;

super::mechanic_skeleton!(FlatDamageOnProc, "T14");
