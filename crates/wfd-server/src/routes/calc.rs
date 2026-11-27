//! `POST /api/calc` (server-api.md §6.1): `CalcRequest` → `CalcReport`. EV for every attack, group and present view;
//! MC only when `want.mc` is present (passes V and L, L0 with `want.parity` on armored enemies), with seed `"auto"`
//! handling; runs in `spawn_blocking` on the interactive pool with a `CancelOnDrop` guard (§11). Rule violations never
//! block a calculation (09 §1.5); 422 `api.invalid` only for malformed documents.
//!
//! Owner: T34 (skeleton by T02d).

use std::sync::Arc;

use axum::Router;
use axum::routing::post;

use crate::error::ApiError;
use crate::state::AppState;

/// The routes of this module.
pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/api/calc", post(calc))
}

/// `POST /api/calc`.
pub async fn calc() -> ApiError {
    ApiError::not_implemented("POST /api/calc")
}
