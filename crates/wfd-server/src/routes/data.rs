//! Data routes and the refresh SSE stream (server-api.md §9; pipeline.md §9.2): status, update check, refresh jobs
//! with embedded inputs (feature `refresh`), activation (`ArcSwap<GameData>` swap; in-flight requests and jobs keep their
//! snapshot).
//!
//! Owner: T34 (skeleton by T02d). `GET /api/data/status` is wired by T11 before T34 starts (work-breakdown.md T11;
//! CONTRACTS.md, T02d).

use std::sync::Arc;

use axum::Router;
use axum::routing::{delete, get, post};

use crate::error::ApiError;
use crate::state::AppState;

/// The routes of this module.
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/data/status", get(status))
        .route("/api/data/check", post(check))
        .route("/api/data/refresh", post(refresh))
        .route("/api/data/refresh/{job}/events", get(refresh_events))
        .route("/api/data/refresh/{job}", delete(cancel_refresh))
        .route("/api/data/activate", post(activate))
}

/// `GET /api/data/status` → `DataStatus`; the user bundles are listed from their headers (`wfd_data::read_header`,
/// no body decoded; contract review API-2).
pub async fn status() -> ApiError {
    ApiError::not_implemented("GET /api/data/status")
}

/// `POST /api/data/check` → `DataCheck` (`wfd_datagen::fetch::check_updates`, T05; request T02c-R3).
pub async fn check() -> ApiError {
    ApiError::not_implemented("POST /api/data/check")
}

/// `POST /api/data/refresh`: `{ curated?: "embedded" }` → 202 `{ job }` · 409 `data.refresh_running`.
pub async fn refresh() -> ApiError {
    ApiError::not_implemented("POST /api/data/refresh")
}

/// `GET /api/data/refresh/{job}/events`: `text/event-stream` of `DataEvent`; a refresh stopped by validation ends with
/// `failed` carrying the run's report (`DatagenError::Validation`, contract review API-23).
pub async fn refresh_events() -> ApiError {
    ApiError::not_implemented("GET /api/data/refresh/{job}/events")
}

/// `DELETE /api/data/refresh/{job}`: 202.
pub async fn cancel_refresh() -> ApiError {
    ApiError::not_implemented("DELETE /api/data/refresh/{job}")
}

/// `POST /api/data/activate`: `{ bundle }` → `DataStatus`.
pub async fn activate() -> ApiError {
    ApiError::not_implemented("POST /api/data/activate")
}
