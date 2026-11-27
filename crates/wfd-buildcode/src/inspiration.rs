//! warframe-damage.com build links (feature parity, spec 00 §6; AD-18): `?p=<LZString>&n=1` and the legacy base64
//! `?p=` without `n`. The link object carries the keys `w e l sp ex j jv z k m`, `o.{…}` (toggles), `p.{a … aa}`
//! (custom stats) and `r` (riven rows); its serialisation is `reCalc` in `min.pretty.js@7bb6 L9676-9785`
//! (research/findings/inspiration-update-2026-11-27.md §3.3), and the key map is `I:feat` §15 and `I:model` §16.
//!
//! Import maps names through `data/curated/aliases/inspiration.toml` and the items' `inspiration_name`, slots exactly
//! as spec 09 §2.4 (slots 0-29, Incarnon `{mId, pId}`, riven rows with the unit rules of 04 §8.2), `o.*` toggles and
//! `p.*` custom stats, and reports what it could not map; it never rejects a link it could parse (09 §2.4).

use crate::error::CodeError;
use wfd_data::GameData;
use wfd_model::CalcInput;
use wfd_model::api::{CodeSource, ImportReport};

/// Imports an inspiration link: a full URL, a query string, or the bare `p` parameter (LZString when `n=1`, else
/// legacy base64). Names, slots, rivens, toggles and custom stats are mapped against `data` (09 §2.4); everything
/// that has no mapping is listed in the report (`unmapped`, `converted`, `defaults_applied`, `warnings`) and never
/// rejects the link. Errors only for text that is no link or does not decode.
///
/// The custom stat `p.u` (`abilityCombo`, 00 §6.3) is the T-11 value: it is written to `toggles.ability_combo`, not
/// to `ScenarioDoc.custom` (contract review M-18). Session-local keys of the result carry their uniqueNames in `x`
/// (the document comes from `GameData::to_doc`, AD-18).
///
/// The [`CodeSource`] says which encoding the link used ([`CodeSource::InspirationLz`] or
/// [`CodeSource::InspirationBase64`], never [`CodeSource::Wfd1`]): `DecodeResponse.source` of server-api.md §7, which
/// only the importer knows for a bare `p` parameter (it carries no `n`). The plan's `(CalcInput, ImportReport)` has
/// no place for it (CONTRACTS.md T02b, request T02b-4); the order is that of `DecodeResponse`. Implemented by T10.
pub fn import_inspiration(
    url_or_param: &str,
    data: &GameData,
) -> Result<(CalcInput, CodeSource, ImportReport), CodeError> {
    let _ = (url_or_param, data);
    todo!("T10: inspiration link import (09 §2.4, I:feat §15)")
}

/// Exports a document as an inspiration URL (`?p=<LZString>&n=1`) for the oracle harness (T36; hidden in the UI).
/// Fails with [`CodeError::ExportUnsupported`] when the scenario uses something the inspiration cannot express, and
/// with [`CodeError::Invalid`] when the document does not resolve against `data`. For the supported scenarios,
/// export followed by import gives the document back (work-breakdown.md T10). Implemented by T10.
pub fn export_inspiration(input: &CalcInput, data: &GameData) -> Result<String, CodeError> {
    let _ = (input, data);
    todo!("T10: inspiration link export (work-breakdown.md T10)")
}
