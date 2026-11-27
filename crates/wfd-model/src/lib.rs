//! `wfd-model`: the shared domain types of warframe-dps (docs/architecture/domain-model.md).
//!
//! Types only: ids, enumerations, the stat registry, the effect DSL, the weapon / enemy / item / bundle schema,
//! build and scenario documents, toggles, compat flags, API DTOs and message keys. No logic beyond construction,
//! validation helpers, (de)serialization and the few pure rules several crates must share exactly (09 E-9.13
//! `RankCurve::value`, 09 E-9.40 [`sets::set_counts`]; AD-24). This crate is the contract every other task codes
//! against; changes follow work-breakdown.md §2.1 and are logged in `CONTRACTS.md`.
//!
//! Features: `ts` (ts-rs derives for the generated TypeScript), `schema` (schemars derives for the JSON schemas),
//! `curated` (the authoring form of `data/curated/*.toml` and its loader).
//!
//! Encodings: every type serializes in a readable form (JSON, TOML) and a compact positional form (postcard, the
//! embedded bundle); see [`serde_util`].

pub mod serde_util;

pub mod build;
pub mod bundle;
pub mod compat;
mod compat_generated;
pub mod effect;
pub mod enemy;
pub mod enums;
pub mod frame;
pub mod i18n;
pub mod ids;
pub mod item;
pub mod mechanic;
pub mod pick;
pub mod rules;
pub mod scenario;
pub mod sets;
pub mod stats;
pub mod weapon;

#[cfg(feature = "curated")]
pub mod curated;

pub mod api;
pub mod message;

pub use build::*;
pub use bundle::*;
pub use compat::CompatFlags;
pub use effect::*;
pub use enemy::*;
pub use enums::*;
pub use frame::*;
pub use i18n::{MessageKey, MsgArg};
pub use ids::*;
pub use item::*;
pub use mechanic::*;
pub use pick::Pick;
pub use rules::*;
pub use scenario::*;
pub use stats::*;
pub use weapon::*;
