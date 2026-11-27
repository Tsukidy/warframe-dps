//! The `extra_hit` mechanic kind: E-14 and 04 §6 (Xata's Whisper, Toxic Lash and the other extra-hit sources; ids
//! `xh/<source>/<name>`). Owner: T14.

/// The `extra_hit` kind (domain-model.md §11.2).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExtraHit;

super::mechanic_skeleton!(ExtraHit, "T14");
