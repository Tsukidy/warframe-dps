//! i18n message keys and arguments carried by domain types (`Violation.key`, `.args`, `ParamSpec.label`,
//! `Choice.label`, `RivenTraitDef.label`, `ToggleDescriptor.label`, `.help`, `ControlOption.label`).
//!
//! The JSON forms are fixed by server-api.md §2 and §4: a key is a dotted string (`"cap.exceeded"`), an argument is
//! a string or a number. Curated TOML (`rules/toggles.toml`) writes keys as strings too.
//!
//! T01a owns this module. The `message` module (owned by T01b, work-breakdown.md §8) builds `Message`, `Severity` and
//! the per-area key enums on top of these two types and re-exports them (`pub use crate::i18n::{MessageKey,
//! MsgArg};`), so domain types and API DTOs share one key type. A change to either type is a CONTRACTS.md entry.

use serde::{Deserialize, Serialize};

/// An i18n message key (`"cap.exceeded"`, `"param.value"`). JSON: a string.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(type = "string"))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MessageKey(pub Box<str>);

impl MessageKey {
    /// A key from its dotted name.
    pub fn new(key: &str) -> Self {
        MessageKey(key.into())
    }

    /// The dotted name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A message argument. JSON: a number or a string.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum MsgArg {
    /// A number (model units).
    Num(f64),
    /// A string (a name, a slot id).
    Str(Box<str>),
}
