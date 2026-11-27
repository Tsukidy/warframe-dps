//! Our share codes (AD-18; server-api.md §7): `wfd1-` + base64url (no padding) of raw DEFLATE of the canonical compact
//! JSON of a `CalcInput`: keys sorted, defaults omitted, StableKeys as integers, `x` with the uniqueName of every
//! session-local key, `eval.seed = "auto"` stored as such. URL form `/?b=<code>` (relative; the port varies between
//! runs, server-api.md §7 "Sharing"). Sample codes of every released document version stay in `tests/codes/v<n>/` as
//! tests forever (work-breakdown.md T10).

use crate::error::CodeError;
use wfd_model::CalcInput;
use wfd_model::api::DecodeReport;

/// Prefix of a version-1 share code (AD-18).
pub const CODE_PREFIX: &str = "wfd1-";

/// Encodes a document as a `wfd1-…` code (AD-18): canonical compact JSON (keys sorted, defaults omitted, slots in
/// canonical order), raw DEFLATE, base64url without padding. Deterministic: equal documents give equal codes.
///
/// `input.x` must already map every session-local key the document uses (AD-18): the encoder has no game data and
/// cannot add the uniqueNames, and a code without them does not decode on another machine. Every caller therefore
/// encodes a document made by `GameData::to_doc` (after `GameData::resolve` for a document from the frontend): the
/// routes `/api/build/encode` and `/api/calc` (`CalcReport.code`) and `RankedBuild.code` of the optimizer
/// (server-api.md §7; contract review API-21, API-7). Implemented by T10.
pub fn encode(input: &CalcInput) -> String {
    let _ = input;
    todo!("T10: wfd1 encoder (AD-18, server-api.md §7)")
}

/// Decodes our code or URL: a bare `wfd1-…` code, `/?b=wfd1-…`, or a full `http://127.0.0.1:<port>/?b=…` URL; older
/// document versions are migrated ([`migrate`]) and reported. Inspiration links are not ours: they are
/// [`crate::import_inspiration`]'s, and give [`CodeError::Unrecognized`] here. The decoder does not look keys up (it
/// has no game data); `GameData::resolve` does, by uniqueName for the keys of `x` (AD-18). The decode route
/// (server-api.md §7) therefore runs decode → `GameData::resolve` → `GameData::to_doc` and returns that document, with
/// the resolve warnings in `DecodeReport.warnings` and the keys of `doc.item_retired` in `retired_items` (contract
/// review API-21). Implemented by T10.
pub fn decode(text: &str) -> Result<(CalcInput, DecodeReport), CodeError> {
    let _ = text;
    todo!("T10: wfd1 decoder (AD-18, server-api.md §7)")
}

/// Reads the JSON of a code of document version `v` as the current `CalcInput` (`wfd_model::DOC_VERSION`), applying
/// each version step in turn; `v` above `DOC_VERSION` is [`CodeError::VersionUnsupported`], JSON that is not a
/// version-`v` document [`CodeError::Corrupt`] at stage `json`. Implemented by T10.
pub fn migrate(v: u16, raw: serde_json::Value) -> Result<CalcInput, CodeError> {
    let _ = (v, raw);
    todo!("T10: document migration (server-api.md §7)")
}
