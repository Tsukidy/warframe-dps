//! `GET /api/meta` (server-api.md §5.1): `Meta` (app id, versions, bundle, sources, licences, coverage, config,
//! features). The single-instance check of §1.2 step 4 reads `app_id = "warframe-dps"` from it.
//!
//! Owner: T11 (skeleton by T02d).

use std::sync::Arc;

use axum::Router;
use axum::routing::get;

use crate::error::ApiError;
use crate::state::AppState;

/// The routes of this module.
pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/api/meta", get(meta))
}

/// `GET /api/meta` → `Meta`.
pub async fn meta() -> ApiError {
    ApiError::not_implemented("GET /api/meta")
}
