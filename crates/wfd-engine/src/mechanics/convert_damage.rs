//! The `convert_damage` mechanic kind: 01 §4.4 and D-009 (Impact-to-Puncture, Chromatic Blade, Reactive Storm:
//! composition, no magnitude slot). Owner: T14.

/// The `convert_damage` kind (domain-model.md §11.2).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ConvertDamage;

super::mechanic_skeleton!(ConvertDamage, "T14");
