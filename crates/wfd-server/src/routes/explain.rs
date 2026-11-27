//! `POST /api/explain` (server-api.md §6.3): `ExplainRequest` → `Explanation` for one probe path (engine.md §11.2).
//!
//! Owner: T34 (skeleton by T02d).

use std::sync::Arc;

use axum::Router;
use axum::routing::post;

use crate::error::ApiError;
use crate::state::AppState;

/// The routes of this module.
pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/api/explain", post(explain))
}

/// `POST /api/explain`.
pub async fn explain() -> ApiError {
    ApiError::not_implemented("POST /api/explain")
}
