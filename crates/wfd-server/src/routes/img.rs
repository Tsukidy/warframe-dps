//! `GET /api/img/{key}` (server-api.md §10): image of the weapon, item or frame with that StableKey, from the disk cache
//! `<data_dir>/images/<sha256 of textureLocation>.png`, fetched once from the Public Export when `images.fetch` is on;
//! 204 when unavailable. No client-supplied path reaches the file system or the network.
//!
//! Owner: T34 (skeleton by T02d). Wired by T11 with fetching off by default in tests (work-breakdown.md T11); T34 adds
//! the image proxy with disk cache (CONTRACTS.md, T02d).

use std::sync::Arc;

use axum::Router;
use axum::routing::get;

use crate::error::ApiError;
use crate::state::AppState;

/// The routes of this module.
pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/api/img/{key}", get(image))
}

/// `GET /api/img/{key}`.
pub async fn image() -> ApiError {
    ApiError::not_implemented("GET /api/img/{key}")
}
