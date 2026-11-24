//! Build-code routes (server-api.md §7, AD-18): `POST /api/build/encode`, `/api/build/decode`,
//! `/api/build/export-inspiration`, and the import report of `wfd-buildcode` (work-breakdown.md T02b, T10).

use crate::ids::StableKey;
use crate::message::Message;
use crate::scenario::CalcInput;
use serde::{Deserialize, Serialize};

/// `POST /api/build/encode` response (the body is a `CalcInput`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EncodeResponse {
    /// `"wfd1-…"`: base64url of raw DEFLATE of the canonical compact JSON of the `CalcInput`.
    pub code: Box<str>,
    /// `"/?b=wfd1-…"`, relative (the port varies between runs).
    pub path: Box<str>,
}

/// `POST /api/build/decode` body: our code, our URL, or a warframe-damage.com URL.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DecodeRequest {
    /// The pasted text.
    pub text: Box<str>,
}

/// `POST /api/build/decode` response.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DecodeResponse {
    /// The decoded document.
    pub input: CalcInput,
    /// What the text was.
    pub source: CodeSource,
    /// What decoding changed or could not map.
    pub report: ImportReport,
}

/// Kind of a decoded text.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum CodeSource {
    /// Our share code or URL.
    Wfd1,
    /// warframe-damage.com link `?p=<LZString>&n=1`.
    InspirationLz,
    /// Legacy warframe-damage.com link (base64 without `n`).
    InspirationBase64,
}

/// Report of a decode or an inspiration import (server-api.md §7 `DecodeResponse.report`; frontend.md §9).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ImportReport {
    /// Fields or names without a mapping.
    pub unmapped: Vec<UnmappedField>,
    /// Unit conversions (`import.converted`).
    pub converted: Vec<Message>,
    /// Defaults applied for fields the inspiration never sets (`import.default_applied`, e.g. `o.v.stucks = 0`,
    /// `I:feat` §15).
    pub defaults_applied: Vec<Message>,
    /// Retired items that were dropped.
    pub retired_items: Vec<StableKey>,
    /// Other warnings (migrations, attacks matched by index only, unknown local items).
    pub warnings: Vec<Message>,
}

/// The report of `wfd-buildcode::decode` (our codes) has the shape of an import report.
pub type DecodeReport = ImportReport;

/// A field the importer could not map.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct UnmappedField {
    /// Link key path (`"m.3"`, `"r.0.p"`).
    pub field: Box<str>,
    /// The value found there.
    pub value: Box<str>,
}

/// `POST /api/build/export-inspiration` response (oracle harness; hidden in the UI).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ExportInspirationResponse {
    /// The warframe-damage.com URL.
    pub url: Box<str>,
}
