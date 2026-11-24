//! Compatibility profile (domain-model.md §9.5, AD-20, engine.md §10).
//!
//! `CompatFlags` switches register rows marked `Compat` to the inspiration's behaviour, for oracle tests only (spec
//! 00 §4.6). The flag list is generated into `compat_generated.rs` by `cargo xtask spec-sync` (T01c). Documents carry
//! flag names (readable form: `["D001_QUANT_1_16", …]`), never the u128, and the type has no TypeScript declaration
//! of its own (`Array<string>`).

use crate::serde_util::flags_serde;

pub use crate::compat_generated::CompatFlags;

flags_serde!(CompatFlags, u128, Upper, opaque);
