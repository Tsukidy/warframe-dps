//! Message keys and messages: owned by T01b (work-breakdown.md §8, T01b), which assembles `MessageKey` from per-area
//! enums (`message/{api,doc,engine,opt,data,import,legality}.rs`) and adds `Message` and `Severity`.
//!
//! PLACEHOLDER created by T01a because domain types of T01a reference these two names (`Violation.key`, `.args`,
//! `ParamSpec.label`, `RivenTraitDef.label`, `ToggleDescriptor.label`, `.help`, `Choice.label`, `ControlOption.label`).
//! The JSON forms are fixed by server-api.md §2 and §4: a key is a dotted string (`"cap.exceeded"`), an argument is a
//! string or a number. T01b replaces both definitions, keeping the names and these JSON forms (see CONTRACTS.md).

use serde::{Deserialize, Serialize};

/// An i18n message key (`"cap.exceeded"`, `"param.value"`). JSON: a string.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(type = "string"))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MessageKey(pub Box<str>);

/// A message argument. JSON: a number or a string.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum MsgArg {
    Num(f64),
    Str(Box<str>),
}
