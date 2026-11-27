//! Build-code routes (server-api.md §7), through `wfd-buildcode` (the only encoder and decoder, AD-18). Neither codec
//! looks keys up, so every route goes through `GameData` (contract review API-21): encode resolves the document and
//! encodes `GameData::to_doc` of it (so `x` maps every session-local key); decode runs decode or import, then
//! `GameData::resolve` and `GameData::to_doc`, and returns that document with the resolve warnings in the report
//! (`warnings`, and the keys of `doc.item_retired` in `retired_items`).
//!
//! Owner: T34 (skeleton by T02d).

use std::sync::Arc;

use axum::Router;
use axum::routing::post;

use crate::error::ApiError;
use crate::state::AppState;

/// The routes of this module.
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/build/encode", post(encode))
        .route("/api/build/decode", post(decode))
        .route("/api/build/export-inspiration", post(export_inspiration))
}

/// `POST /api/build/encode`: `CalcInput` → `EncodeResponse`.
pub async fn encode() -> ApiError {
    ApiError::not_implemented("POST /api/build/encode")
}

/// `POST /api/build/decode`: `{ text }` (our code, our URL or a warframe-damage.com URL) → `DecodeResponse`.
pub async fn decode() -> ApiError {
    ApiError::not_implemented("POST /api/build/decode")
}

/// `POST /api/build/export-inspiration`: `CalcInput` → `{ url }` or 422 (oracle harness; hidden in the UI).
pub async fn export_inspiration() -> ApiError {
    ApiError::not_implemented("POST /api/build/export-inspiration")
}
