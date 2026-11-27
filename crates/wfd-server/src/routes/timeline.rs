//! `POST /api/timeline` (server-api.md §6.2): `TimelineRequest` → `Timeline`, the per-hit event log of one MC run.
//!
//! Owner: T34 (skeleton by T02d).

use std::sync::Arc;

use axum::Router;
use axum::routing::post;

use crate::error::ApiError;
use crate::state::AppState;

/// The routes of this module.
pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/api/timeline", post(timeline))
}

/// `POST /api/timeline`.
pub async fn timeline() -> ApiError {
    ApiError::not_implemented("POST /api/timeline")
}
